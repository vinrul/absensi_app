use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::routing::{get, post};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::{BerkasOwner, StatusHasil, StatusRencana};
use crate::error::{AppError, AppResult};
use crate::repo::hasil::{self, HasilRow, ListFilter, NewHasil, SkorTersimpan, UpdateHasil};
use crate::repo::lampiran::{self, LampiranRow};
use crate::repo::notifikasi;
use crate::repo::periode;
use crate::repo::personel;
use crate::repo::rencana;
use crate::repo::satker;
use crate::repo::{ListResponse, pagination};
use crate::service::akses;
use crate::service::{laporan, pdf, scoring, waktu};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_hasil).post(create_hasil))
        .route("/verifikasi", get(list_verifikasi_hasil))
        .route("/ekspor", get(ekspor_hasil_pdf))
        .route("/{id}", get(detail_hasil).patch(update_hasil))
        .route("/{id}/ajukan", post(ajukan_hasil))
        .route("/{id}/verifikasi", post(verifikasi_hasil))
}

#[derive(Debug, Serialize)]
pub struct HasilDetail {
    #[serde(flatten)]
    pub hasil: HasilRow,
    pub lampiran: Vec<LampiranRow>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub satker_id: Option<Uuid>,
    pub personel_id: Option<Uuid>,
    pub status: Option<StatusHasil>,
    pub periode_id: Option<Uuid>,
}

pub async fn list_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ListResponse<HasilRow>>> {
    let mut scope = crate::routes::personel::scope_for(&state, &user).await?;

    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat hasil kerja satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }

    let claims = user.claims();
    let personel_id = if claims.is_admin() || claims.is_atasan() || claims.is_pimpinan() {
        query.personel_id
    } else {
        Some(claims.sub)
    };

    let (page, per_page, offset) = pagination(query.page, query.per_page);
    let (rows, total) = hasil::list(
        &state.pool,
        &ListFilter {
            satker_ids: scope,
            personel_id,
            status: query.status,
            periode_id: query.periode_id,
            atasan_id: None,
            hanya_diajukan: false,
            limit: per_page,
            offset,
        },
    )
    .await?;

    Ok(Json(ListResponse::new(rows, page, per_page, total)))
}

/// Ekspor daftar hasil kerja sesuai filter menjadi PDF.
pub async fn ekspor_hasil_pdf(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Response> {
    let mut scope = crate::routes::personel::scope_for(&state, &user).await?;

    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak mengekspor hasil kerja satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }

    let claims = user.claims();
    let personel_id = if claims.is_admin() || claims.is_atasan() || claims.is_pimpinan() {
        query.personel_id
    } else {
        Some(claims.sub)
    };

    let (rows, total) = hasil::list(
        &state.pool,
        &ListFilter {
            satker_ids: scope,
            personel_id,
            status: query.status,
            periode_id: query.periode_id,
            atasan_id: None,
            hanya_diajukan: false,
            limit: 5000,
            offset: 0,
        },
    )
    .await?;

    let nama_satker = match query.satker_id {
        Some(id) => satker::find(&state.pool, id)
            .await?
            .map(|row| row.nama)
            .unwrap_or_else(|| "-".into()),
        None => "Semua Satker".into(),
    };
    let nama_personel = match personel_id {
        Some(id) => personel::find_by_id(&state.pool, id)
            .await?
            .map(|row| match row.nrp {
                Some(nrp) => format!("{} ({})", row.nama, nrp),
                None => row.nama,
            })
            .unwrap_or_else(|| "-".into()),
        None => "Semua Personel".into(),
    };
    let nama_periode = match query.periode_id {
        Some(id) => periode::find(&state.pool, id)
            .await?
            .map(|row| {
                format!(
                    "{} ({} s/d {})",
                    row.nama,
                    laporan::format_tanggal(row.tanggal_mulai),
                    laporan::format_tanggal(row.tanggal_selesai)
                )
            })
            .unwrap_or_else(|| "-".into()),
        None => "Semua Periode".into(),
    };
    let label_status = match query.status {
        Some(status) => label_status_hasil(status).to_string(),
        None => "Semua Status".into(),
    };
    let dicetak_oleh = personel::find_by_id(&state.pool, user.id())
        .await?
        .map(|row| row.nama)
        .unwrap_or_else(|| "-".into());
    let dicetak_pada = (chrono::Utc::now() + chrono::Duration::hours(8))
        .format("%d-%m-%Y %H:%M WITA")
        .to_string();
    let nama_satker_map: HashMap<Uuid, String> = satker::list(&state.pool)
        .await?
        .into_iter()
        .map(|row| (row.id, row.nama))
        .collect();

    let mut baris = String::new();
    for (indeks, row) in rows.iter().enumerate() {
        baris.push_str(&format!(
            "<tr>\
                <td>{}</td>\
                <td><strong>{}</strong><br><span style=\"color:#64748b\">{}</span></td>\
                <td>{}</td>\
                <td><strong>{}</strong><br><span style=\"color:#64748b\">{}</span></td>\
                <td>{}</td>\
                <td>{:.0}%</td>\
                <td>{}</td>\
                <td>{}</td>\
                <td><span class=\"badge badge-{}\">{}</span></td>\
             </tr>",
            indeks + 1,
            laporan::esc(row.personel_nama.as_deref().unwrap_or("-")),
            laporan::esc(row.personel_nrp.as_deref().unwrap_or("-")),
            laporan::esc(
                nama_satker_map
                    .get(&row.satker_id)
                    .map(String::as_str)
                    .unwrap_or("-")
            ),
            laporan::esc(row.rencana_kegiatan.as_deref().unwrap_or("-")),
            laporan::esc(&row.uraian_pelaksanaan),
            laporan::format_tanggal(row.tanggal_realisasi),
            row.progres_persen,
            row.jumlah_realisasi
                .map(|nilai| format!("{nilai}"))
                .unwrap_or_else(|| "-".into()),
            row.skor_total
                .map(|nilai| format!("{nilai:.2}"))
                .unwrap_or_else(|| "-".into()),
            row.status.as_str(),
            label_status_hasil(row.status),
        ));
    }

    let html = laporan::html(&laporan::Laporan {
        judul: "Laporan Hasil Kerja Personel",
        periode: &nama_periode,
        satker: &nama_satker,
        personel: &nama_personel,
        status: &label_status,
        dicetak_oleh: &dicetak_oleh,
        dicetak_pada: &dicetak_pada,
        kolom: vec![
            laporan::Kolom {
                judul: "No",
                lebar: 4,
            },
            laporan::Kolom {
                judul: "Personel",
                lebar: 14,
            },
            laporan::Kolom {
                judul: "Satker",
                lebar: 11,
            },
            laporan::Kolom {
                judul: "Rencana / Kegiatan",
                lebar: 29,
            },
            laporan::Kolom {
                judul: "Tanggal",
                lebar: 9,
            },
            laporan::Kolom {
                judul: "Progres",
                lebar: 7,
            },
            laporan::Kolom {
                judul: "Realisasi",
                lebar: 7,
            },
            laporan::Kolom {
                judul: "Nilai",
                lebar: 7,
            },
            laporan::Kolom {
                judul: "Status",
                lebar: 12,
            },
        ],
        baris,
        jumlah: rows.len(),
        total,
        catatan: None,
    });
    let isi_pdf = pdf::render_pdf(&html).await?;

    let nama_berkas = format!(
        "hasil-kerja-{}.pdf",
        waktu::hari_ini_wita().format("%Y%m%d")
    );
    let disposisi = HeaderValue::from_str(&format!("attachment; filename=\"{nama_berkas}\""))
        .unwrap_or_else(|_| HeaderValue::from_static("attachment"));

    let mut response = Response::new(Body::from(isi_pdf));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    response
        .headers_mut()
        .insert(header::CONTENT_DISPOSITION, disposisi);
    Ok(response)
}

fn label_status_hasil(status: StatusHasil) -> &'static str {
    match status {
        StatusHasil::Draft => "Draft",
        StatusHasil::Diajukan => "Diajukan",
        StatusHasil::Terverifikasi => "Terverifikasi",
        StatusHasil::PerluRevisi => "Perlu Revisi",
    }
}

pub async fn list_verifikasi_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ListResponse<HasilRow>>> {
    if !(user.is_admin_any() || user.is_atasan() || user.is_pimpinan_any()) {
        return Err(AppError::Forbidden(
            "Hanya atasan, pimpinan, atau admin yang dapat melihat verifikasi".into(),
        ));
    }

    let scope = crate::routes::personel::scope_for(&state, &user).await?;
    let (page, per_page, offset) = pagination(query.page, query.per_page);

    let (rows, total) = hasil::list(
        &state.pool,
        &ListFilter {
            satker_ids: scope,
            personel_id: None,
            status: None,
            periode_id: None,
            atasan_id: if user.is_admin_any() {
                None
            } else {
                Some(user.id())
            },
            hanya_diajukan: true,
            limit: per_page,
            offset,
        },
    )
    .await?;

    Ok(Json(ListResponse::new(rows, page, per_page, total)))
}

async fn detail_internal(state: &AppState, id: Uuid) -> AppResult<HasilDetail> {
    let hasil = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;
    let lampiran_rows = lampiran::list(&state.pool, BerkasOwner::Hasil, id).await?;

    Ok(HasilDetail {
        hasil,
        lampiran: lampiran_rows,
    })
}

pub async fn detail_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<HasilDetail>> {
    let detail = detail_internal(&state, id).await?;
    let row = &detail.hasil;

    let boleh = akses::boleh_melihat_personel(
        user.role_scopes(),
        user.satker_akses(),
        user.id(),
        row.personel_id,
        row.satker_id,
    );
    if !boleh {
        return Err(AppError::Forbidden(
            "Anda tidak berhak melihat hasil kerja ini".into(),
        ));
    }

    Ok(Json(detail))
}

#[derive(Debug, Deserialize)]
pub struct CreateHasilRequest {
    pub rencana_id: Uuid,
    pub uraian_pelaksanaan: String,
    pub progres_persen: f64,
    pub jumlah_realisasi: Option<f64>,
    pub kendala: Option<String>,
    pub tanggal_realisasi: NaiveDate,
    pub ajukan: Option<bool>,
}

pub async fn create_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateHasilRequest>,
) -> AppResult<Json<HasilRow>> {
    let personel = personel::active_personel(&state.pool, user.id()).await?;

    let rencana = rencana::find(&state.pool, req.rencana_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;

    if rencana.personel_id != personel.id {
        return Err(AppError::Forbidden(
            "Hasil kerja hanya untuk rencana kerja milik Anda".into(),
        ));
    }
    if rencana.status != StatusRencana::Disetujui {
        return Err(AppError::Unprocessable(
            "Rencana kerja harus disetujui atasan sebelum diisi hasilnya".into(),
        ));
    }

    periode::pastikan_terbuka(&state.pool, rencana.periode_id).await?;

    if req.uraian_pelaksanaan.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Uraian pelaksanaan wajib diisi".into(),
        ));
    }
    if !(0.0..=100.0).contains(&req.progres_persen) {
        return Err(AppError::Unprocessable(
            "Progres harus antara 0 dan 100 persen".into(),
        ));
    }

    let ajukan = req.ajukan.unwrap_or(true);
    if ajukan && rencana.atasan_id.is_none() {
        return Err(AppError::Unprocessable(
            "Atasan langsung belum ditetapkan untuk akun Anda".into(),
        ));
    }

    let id = Uuid::now_v7();
    hasil::create(
        &state.pool,
        &NewHasil {
            id,
            rencana_id: rencana.id,
            personel_id: personel.id,
            satker_id: personel.satker_id,
            uraian_pelaksanaan: req.uraian_pelaksanaan,
            progres_persen: req.progres_persen,
            jumlah_realisasi: req.jumlah_realisasi,
            kendala: req.kendala,
            tanggal_realisasi: req.tanggal_realisasi,
            status: if ajukan {
                StatusHasil::Diajukan
            } else {
                StatusHasil::Draft
            },
        },
    )
    .await?;

    if ajukan && let Some(atasan_id) = rencana.atasan_id {
        notifikasi::kirim(
            &state.pool,
            atasan_id,
            "hasil_diajukan",
            "Hasil kerja menunggu verifikasi",
            &format!("{} mengirim hasil kerja", personel.nama),
            Some("hasil"),
            Some(id),
        )
        .await?;
    }

    let row = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("hasil kerja gagal dibaca".into()))?;
    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
pub struct UpdateHasilRequest {
    pub uraian_pelaksanaan: String,
    pub progres_persen: f64,
    pub jumlah_realisasi: Option<f64>,
    pub kendala: Option<String>,
    pub tanggal_realisasi: NaiveDate,
}

pub async fn update_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateHasilRequest>,
) -> AppResult<Json<HasilDetail>> {
    let existing = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;

    if existing.personel_id != user.id() {
        return Err(AppError::Forbidden(
            "Hanya pemilik hasil kerja yang dapat mengubah".into(),
        ));
    }
    if !matches!(
        existing.status,
        StatusHasil::Draft | StatusHasil::PerluRevisi
    ) {
        return Err(AppError::Unprocessable(
            "Hasil kerja yang sudah diajukan tidak dapat diubah".into(),
        ));
    }

    let rencana = rencana::find(&state.pool, existing.rencana_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;
    periode::pastikan_terbuka(&state.pool, rencana.periode_id).await?;

    if req.uraian_pelaksanaan.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Uraian pelaksanaan wajib diisi".into(),
        ));
    }
    if !(0.0..=100.0).contains(&req.progres_persen) {
        return Err(AppError::Unprocessable(
            "Progres harus antara 0 dan 100 persen".into(),
        ));
    }

    hasil::update(
        &state.pool,
        id,
        &UpdateHasil {
            uraian_pelaksanaan: req.uraian_pelaksanaan,
            progres_persen: req.progres_persen,
            jumlah_realisasi: req.jumlah_realisasi,
            kendala: req.kendala,
            tanggal_realisasi: req.tanggal_realisasi,
        },
    )
    .await?;

    Ok(Json(detail_internal(&state, id).await?))
}

pub async fn ajukan_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<HasilDetail>> {
    let existing = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;

    if existing.personel_id != user.id() {
        return Err(AppError::Forbidden(
            "Hanya pemilik hasil kerja yang dapat mengajukan".into(),
        ));
    }
    if !matches!(
        existing.status,
        StatusHasil::Draft | StatusHasil::PerluRevisi
    ) {
        return Err(AppError::Unprocessable(
            "Hasil kerja ini sudah diajukan atau sudah diverifikasi".into(),
        ));
    }

    let rencana = rencana::find(&state.pool, existing.rencana_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;
    let atasan_id = rencana.atasan_id.ok_or_else(|| {
        AppError::Unprocessable("Atasan langsung belum ditetapkan untuk akun Anda".into())
    })?;
    periode::pastikan_terbuka(&state.pool, rencana.periode_id).await?;

    hasil::ajukan(&state.pool, id).await?;

    let personel = personel::active_personel(&state.pool, user.id()).await?;
    notifikasi::kirim(
        &state.pool,
        atasan_id,
        "hasil_diajukan",
        "Hasil kerja menunggu verifikasi",
        &format!("{} mengirim hasil kerja", personel.nama),
        Some("hasil"),
        Some(id),
    )
    .await?;

    Ok(Json(detail_internal(&state, id).await?))
}

#[derive(Debug, Deserialize)]
pub struct VerifikasiRequest {
    pub skor_kualitas: Option<f64>,
    pub catatan: Option<String>,
    pub setuju: Option<bool>,
}

pub async fn verifikasi_hasil(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<VerifikasiRequest>,
) -> AppResult<Json<HasilRow>> {
    if !(user.is_admin_any() || user.is_atasan() || user.is_pimpinan_any()) {
        return Err(AppError::Forbidden(
            "Hanya atasan, pimpinan, atau admin yang dapat memverifikasi hasil kerja".into(),
        ));
    }

    let hasil_row = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;

    if hasil_row.status != StatusHasil::Diajukan {
        return Err(AppError::Unprocessable(
            "Hasil kerja ini tidak sedang menunggu verifikasi".into(),
        ));
    }

    let rencana_row = rencana::find(&state.pool, hasil_row.rencana_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;
    user.require_satker(rencana_row.satker_id)?;

    let claims = user.claims();
    let boleh = rencana_row.atasan_id == Some(claims.sub) || user.is_admin_any();
    if !boleh {
        return Err(AppError::Forbidden(
            "Hanya atasan langsung yang dapat memverifikasi".into(),
        ));
    }

    let setuju = req.setuju.unwrap_or(true);

    if !setuju {
        let catatan = req.catatan.as_deref().unwrap_or("").trim();
        if catatan.is_empty() {
            return Err(AppError::Unprocessable(
                "Catatan wajib diisi saat meminta revisi".into(),
            ));
        }
        hasil::minta_revisi(&state.pool, id, claims.sub, catatan).await?;
    } else {
        let kualitas = req.skor_kualitas.unwrap_or(80.0);
        if !(0.0..=100.0).contains(&kualitas) {
            return Err(AppError::Unprocessable(
                "Skor kualitas harus antara 0 dan 100".into(),
            ));
        }

        let kuantitas =
            scoring::komponen_kuantitas(hasil_row.jumlah_realisasi, rencana_row.target_kuantitas);
        let waktu = scoring::komponen_waktu(hasil_row.tanggal_realisasi, rencana_row.target_waktu);
        let (k_kuantitas, k_kualitas, k_waktu, total) =
            scoring::skor_hasil(kuantitas, kualitas, waktu);

        hasil::verifikasi(
            &state.pool,
            id,
            claims.sub,
            &SkorTersimpan {
                kuantitas: k_kuantitas,
                kualitas: k_kualitas,
                waktu: k_waktu,
                total,
            },
            req.catatan.as_deref(),
        )
        .await?;

        notifikasi::kirim(
            &state.pool,
            hasil_row.personel_id,
            "hasil_terverifikasi",
            "Hasil kerja terverifikasi",
            &format!("Nilai hasil kerja: {total:.2}"),
            Some("hasil"),
            Some(id),
        )
        .await?;
    }

    let row = hasil::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;
    Ok(Json(row))
}
