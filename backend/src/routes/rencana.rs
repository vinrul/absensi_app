use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::routing::{get, post};
use base64::Engine as _;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::{BerkasOwner, DasarTugasJenis, JenisRencana, StatusRencana};
use crate::error::{AppError, AppResult};
use crate::repo::lampiran::{self, LampiranRow};
use crate::repo::notifikasi;
use crate::repo::periode;
use crate::repo::personel;
use crate::repo::rencana::{
    self, ListFilter, NewRencana, NewTahapan, RencanaRow, TahapanRow, UpdateRencana,
};
use crate::repo::satker;
use crate::repo::{ListResponse, pagination};
use crate::service::akses;
use crate::service::pdf;
use crate::service::waktu;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_rencana).post(create_rencana))
        .route("/persetujuan", get(list_persetujuan))
        .route("/ekspor", get(ekspor_rencana_pdf))
        .route("/{id}", get(detail_rencana).patch(update_rencana))
        .route("/{id}/ajukan", post(ajukan_rencana))
        .route("/{id}/keputusan", post(keputusan_rencana))
}

#[derive(Debug, Serialize)]
pub struct RencanaDetail {
    #[serde(flatten)]
    pub rencana: RencanaRow,
    pub tahapan: Vec<TahapanRow>,
    pub lampiran: Vec<LampiranRow>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub satker_id: Option<Uuid>,
    pub personel_id: Option<Uuid>,
    pub status: Option<StatusRencana>,
    pub periode_id: Option<Uuid>,
}

pub async fn list_rencana(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ListResponse<RencanaRow>>> {
    let mut scope = crate::routes::personel::scope_for(&state, &user).await?;

    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat rencana kerja satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }

    let (page, per_page, offset) = pagination(query.page, query.per_page);

    // Personel biasa hanya melihat miliknya sendiri.
    let claims = user.claims();
    let personel_id = if claims.is_admin() || claims.is_atasan() || claims.is_pimpinan() {
        query.personel_id
    } else {
        Some(claims.sub)
    };

    let filter = ListFilter {
        satker_ids: scope,
        personel_id,
        status: query.status,
        periode_id: query.periode_id,
        atasan_id: None,
        hanya_diajukan: false,
        limit: per_page,
        offset,
    };

    let (rows, total) = rencana::list(&state.pool, &filter).await?;
    Ok(Json(ListResponse::new(rows, page, per_page, total)))
}

/// Ekspor daftar rencana kerja sesuai filter menjadi PDF (Chrome/Edge headless).
pub async fn ekspor_rencana_pdf(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Response> {
    let mut scope = crate::routes::personel::scope_for(&state, &user).await?;

    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak mengekspor rencana kerja satker tersebut".into(),
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

    // Rekap cetak tidak memakai pagination; batasi agar proses Chrome tetap wajar.
    let (rows, total) = rencana::list(
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
                    format_tanggal(row.tanggal_mulai),
                    format_tanggal(row.tanggal_selesai)
                )
            })
            .unwrap_or_else(|| "-".into()),
        None => "Semua Periode".into(),
    };
    let label_status = match query.status {
        Some(status) => label_status_rencana(status).to_string(),
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

    let html = html_laporan_rencana(
        &rows,
        total,
        &nama_periode,
        &nama_satker,
        &nama_personel,
        &label_status,
        &dicetak_oleh,
        &dicetak_pada,
        &nama_satker_map,
    );
    let isi_pdf = pdf::render_pdf(&html).await?;

    let nama_berkas = format!(
        "rencana-kerja-{}.pdf",
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

pub async fn list_persetujuan(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ListResponse<RencanaRow>>> {
    if !(user.is_admin_any() || user.is_atasan() || user.is_pimpinan_any()) {
        return Err(AppError::Forbidden(
            "Hanya atasan, pimpinan, atau admin yang dapat melihat persetujuan".into(),
        ));
    }

    let scope = crate::routes::personel::scope_for(&state, &user).await?;
    let (page, per_page, offset) = pagination(query.page, query.per_page);

    let filter = ListFilter {
        satker_ids: scope,
        personel_id: None,
        status: None,
        periode_id: query.periode_id,
        atasan_id: if user.is_admin_any() {
            None
        } else {
            Some(user.id())
        },
        hanya_diajukan: true,
        limit: per_page,
        offset,
    };

    let (rows, total) = rencana::list(&state.pool, &filter).await?;
    Ok(Json(ListResponse::new(rows, page, per_page, total)))
}

async fn detail_internal(state: &AppState, id: Uuid) -> AppResult<RencanaDetail> {
    let rencana = rencana::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;

    let tahapan = rencana::tahapan(&state.pool, id).await?;
    let lampiran_rows = lampiran::list(&state.pool, BerkasOwner::Rencana, id).await?;

    Ok(RencanaDetail {
        rencana,
        tahapan,
        lampiran: lampiran_rows,
    })
}

pub async fn detail_rencana(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<RencanaDetail>> {
    let detail = detail_internal(&state, id).await?;

    let boleh = akses::boleh_melihat_personel(
        user.role_scopes(),
        user.satker_akses(),
        user.id(),
        detail.rencana.personel_id,
        detail.rencana.satker_id,
    ) || detail.rencana.atasan_id == Some(user.id());

    if !boleh {
        return Err(AppError::Forbidden(
            "Anda tidak berhak melihat rencana kerja ini".into(),
        ));
    }

    Ok(Json(detail))
}

#[derive(Debug, Deserialize)]
pub struct TahapanRequest {
    pub uraian: String,
    pub target_kuantitas: Option<f64>,
    pub satuan: Option<String>,
    pub bobot: Option<f64>,
    pub program: Option<String>,
    pub kegiatan: Option<String>,
    pub tujuan: Option<String>,
    pub indikator_kinerja: Option<String>,
    pub target_kualitas: Option<String>,
    pub tanggal_mulai: Option<NaiveDate>,
    pub tanggal_selesai: Option<NaiveDate>,
}

fn tahapan_baru(items: Vec<TahapanRequest>) -> Vec<NewTahapan> {
    items
        .into_iter()
        .map(|item| NewTahapan {
            uraian: item.uraian,
            target_kuantitas: item.target_kuantitas,
            satuan: item.satuan,
            bobot: item.bobot,
            program: item.program,
            kegiatan: item.kegiatan,
            tujuan: item.tujuan,
            indikator_kinerja: item.indikator_kinerja,
            target_kualitas: item.target_kualitas,
            tanggal_mulai: item.tanggal_mulai,
            tanggal_selesai: item.tanggal_selesai,
        })
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct CreateRencanaRequest {
    pub periode_id: Option<Uuid>,
    pub program: String,
    pub kegiatan: String,
    pub uraian: String,
    pub tujuan: Option<String>,
    pub indikator_kinerja: Option<String>,
    pub jenis: JenisRencana,
    pub dasar_jenis: Option<DasarTugasJenis>,
    pub dasar_nomor: Option<String>,
    pub dasar_tanggal: Option<NaiveDate>,
    pub dasar_perihal: Option<String>,
    pub target_kuantitas: Option<f64>,
    pub satuan: Option<String>,
    pub target_kualitas: Option<String>,
    pub target_waktu: Option<NaiveDate>,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
    pub tahapan: Option<Vec<TahapanRequest>>,
    pub ajukan: Option<bool>,
}

pub async fn create_rencana(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateRencanaRequest>,
) -> AppResult<Json<RencanaDetail>> {
    let personel = personel::active_personel(&state.pool, user.id()).await?;

    let periode = periode::find_or_current(&state.pool, req.periode_id).await?;
    periode::pastikan_terbuka(&state.pool, periode.id).await?;

    if req.program.trim().is_empty() || req.kegiatan.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Program dan kegiatan wajib diisi".into(),
        ));
    }
    if req.tanggal_selesai < req.tanggal_mulai {
        return Err(AppError::Unprocessable(
            "Tanggal selesai tidak boleh sebelum tanggal mulai".into(),
        ));
    }

    let ajukan = req.ajukan.unwrap_or(false);
    if ajukan && personel.atasan_id.is_none() {
        return Err(AppError::Unprocessable(
            "Atasan langsung belum ditetapkan untuk akun Anda".into(),
        ));
    }

    let id = Uuid::now_v7();
    let tahapan = req.tahapan.map(tahapan_baru).unwrap_or_default();

    rencana::create(
        &state.pool,
        &NewRencana {
            id,
            personel_id: personel.id,
            satker_id: personel.satker_id,
            periode_id: periode.id,
            program: req.program.trim().to_string(),
            kegiatan: req.kegiatan.trim().to_string(),
            uraian: req.uraian,
            tujuan: req.tujuan,
            indikator_kinerja: req.indikator_kinerja,
            jenis: req.jenis,
            dasar_jenis: req.dasar_jenis,
            dasar_nomor: req.dasar_nomor,
            dasar_tanggal: req.dasar_tanggal,
            dasar_perihal: req.dasar_perihal,
            target_kuantitas: req.target_kuantitas,
            satuan: req.satuan,
            target_kualitas: req.target_kualitas,
            target_waktu: req.target_waktu,
            tanggal_mulai: req.tanggal_mulai,
            tanggal_selesai: req.tanggal_selesai,
            atasan_id: personel.atasan_id,
            status: if ajukan {
                StatusRencana::Diajukan
            } else {
                StatusRencana::Draft
            },
            tahapan,
        },
    )
    .await?;

    if ajukan && let Some(atasan_id) = personel.atasan_id {
        notifikasi::kirim(
            &state.pool,
            atasan_id,
            "rencana_diajukan",
            "Rencana kerja menunggu persetujuan",
            &format!("{} mengajukan rencana kerja baru", personel.nama),
            Some("rencana"),
            Some(id),
        )
        .await?;
    }

    Ok(Json(detail_internal(&state, id).await?))
}

pub async fn update_rencana(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRencanaRequest>,
) -> AppResult<Json<RencanaDetail>> {
    let existing = rencana::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;

    if existing.personel_id != user.id() {
        return Err(AppError::Forbidden(
            "Hanya pemilik rencana yang dapat mengubah".into(),
        ));
    }
    if !matches!(
        existing.status,
        StatusRencana::Draft | StatusRencana::PerluRevisi
    ) {
        return Err(AppError::Unprocessable(
            "Rencana yang sudah diajukan tidak dapat diubah".into(),
        ));
    }

    rencana::update(
        &state.pool,
        id,
        &UpdateRencana {
            program: req.program,
            kegiatan: req.kegiatan,
            uraian: req.uraian,
            tujuan: req.tujuan,
            indikator_kinerja: req.indikator_kinerja,
            jenis: req.jenis,
            dasar_jenis: req.dasar_jenis,
            dasar_nomor: req.dasar_nomor,
            dasar_tanggal: req.dasar_tanggal,
            dasar_perihal: req.dasar_perihal,
            target_kuantitas: req.target_kuantitas,
            satuan: req.satuan,
            target_kualitas: req.target_kualitas,
            target_waktu: req.target_waktu,
            tanggal_mulai: req.tanggal_mulai,
            tanggal_selesai: req.tanggal_selesai,
            tahapan: req.tahapan.map(tahapan_baru),
        },
    )
    .await?;

    Ok(Json(detail_internal(&state, id).await?))
}

#[derive(Debug, Deserialize)]
pub struct UpdateRencanaRequest {
    pub program: Option<String>,
    pub kegiatan: Option<String>,
    pub uraian: Option<String>,
    pub tujuan: Option<String>,
    pub indikator_kinerja: Option<String>,
    pub jenis: Option<JenisRencana>,
    pub dasar_jenis: Option<DasarTugasJenis>,
    pub dasar_nomor: Option<String>,
    pub dasar_tanggal: Option<NaiveDate>,
    pub dasar_perihal: Option<String>,
    pub target_kuantitas: Option<f64>,
    pub satuan: Option<String>,
    pub target_kualitas: Option<String>,
    pub target_waktu: Option<NaiveDate>,
    pub tanggal_mulai: Option<NaiveDate>,
    pub tanggal_selesai: Option<NaiveDate>,
    pub tahapan: Option<Vec<TahapanRequest>>,
}

pub async fn ajukan_rencana(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<RencanaDetail>> {
    let existing = rencana::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;

    if existing.personel_id != user.id() {
        return Err(AppError::Forbidden(
            "Hanya pemilik rencana yang dapat mengajukan".into(),
        ));
    }
    if !matches!(
        existing.status,
        StatusRencana::Draft | StatusRencana::PerluRevisi
    ) {
        return Err(AppError::Unprocessable(
            "Rencana ini sudah diajukan atau sudah diputuskan".into(),
        ));
    }

    let personel = personel::active_personel(&state.pool, user.id()).await?;
    let atasan_id = personel.atasan_id.ok_or_else(|| {
        AppError::Unprocessable("Atasan langsung belum ditetapkan untuk akun Anda".into())
    })?;

    rencana::ajukan(&state.pool, id).await?;

    notifikasi::kirim(
        &state.pool,
        atasan_id,
        "rencana_diajukan",
        "Rencana kerja menunggu persetujuan",
        &format!("{} mengajukan rencana kerja", personel.nama),
        Some("rencana"),
        Some(id),
    )
    .await?;

    Ok(Json(detail_internal(&state, id).await?))
}

#[derive(Debug, Deserialize)]
pub struct KeputusanRequest {
    pub status: StatusRencana,
    pub catatan: Option<String>,
}

pub async fn keputusan_rencana(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<KeputusanRequest>,
) -> AppResult<Json<RencanaDetail>> {
    if !(user.is_admin_any() || user.is_atasan() || user.is_pimpinan_any()) {
        return Err(AppError::Forbidden(
            "Hanya atasan, pimpinan, atau admin yang dapat memutuskan rencana".into(),
        ));
    }

    if !matches!(
        req.status,
        StatusRencana::Disetujui | StatusRencana::PerluRevisi | StatusRencana::Ditolak
    ) {
        return Err(AppError::Unprocessable(
            "Keputusan harus disetujui, perlu_revisi, atau ditolak".into(),
        ));
    }

    let existing = rencana::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;
    user.require_satker(existing.satker_id)?;

    let claims = user.claims();
    let boleh = existing.atasan_id == Some(claims.sub) || user.is_admin_any();
    if !boleh {
        return Err(AppError::Forbidden(
            "Hanya atasan langsung yang dapat memutuskan".into(),
        ));
    }
    if existing.status != StatusRencana::Diajukan {
        return Err(AppError::Unprocessable(
            "Rencana ini tidak sedang menunggu persetujuan".into(),
        ));
    }
    if req.status == StatusRencana::PerluRevisi
        && req.catatan.as_deref().unwrap_or("").trim().is_empty()
    {
        return Err(AppError::Unprocessable(
            "Catatan wajib diisi saat meminta revisi".into(),
        ));
    }

    let tambah_revisi = req.status == StatusRencana::PerluRevisi;
    rencana::putuskan(
        &state.pool,
        id,
        req.status,
        req.catatan.as_deref(),
        tambah_revisi,
    )
    .await?;

    let judul = match req.status {
        StatusRencana::Disetujui => "Rencana kerja disetujui",
        StatusRencana::PerluRevisi => "Rencana kerja perlu revisi",
        _ => "Rencana kerja ditolak",
    };

    notifikasi::kirim(
        &state.pool,
        existing.personel_id,
        "rencana_diputuskan",
        judul,
        req.catatan
            .as_deref()
            .unwrap_or("Lihat detail rencana kerja"),
        Some("rencana"),
        Some(id),
    )
    .await?;

    Ok(Json(detail_internal(&state, id).await?))
}

fn format_tanggal(tanggal: NaiveDate) -> String {
    tanggal.format("%d-%m-%Y").to_string()
}

fn esc(teks: &str) -> String {
    teks.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn label_status_rencana(status: StatusRencana) -> &'static str {
    match status {
        StatusRencana::Draft => "Draft",
        StatusRencana::Diajukan => "Diajukan",
        StatusRencana::Disetujui => "Disetujui",
        StatusRencana::PerluRevisi => "Perlu Revisi",
        StatusRencana::Ditolak => "Ditolak",
    }
}

fn label_jenis_rencana(jenis: JenisRencana) -> &'static str {
    match jenis {
        JenisRencana::Harian => "Harian",
        JenisRencana::Mingguan => "Mingguan",
        JenisRencana::Bulanan => "Bulanan",
        JenisRencana::Triwulan => "Triwulan",
        JenisRencana::PenugasanKhusus => "Penugasan Khusus",
    }
}

const CSS_LAPORAN: &str = r#"
@page { size: A4 landscape; margin: 10mm 12mm; }
* { box-sizing: border-box; }
body { font-family: "Segoe UI", Arial, sans-serif; font-size: 9pt; color: #1e293b; margin: 0; }
.kop { width: 100%; border-bottom: 2px solid #0b2545; padding-bottom: 8px; margin-bottom: 10px; }
.kop td { vertical-align: middle; }
.logo { width: 76px; }
.logo img { width: 66px; height: auto; }
.identitas { text-align: center; }
.identitas .instansi { font-size: 9pt; letter-spacing: .06em; text-transform: uppercase; color: #334155; }
.identitas h1 { font-size: 14pt; margin: 2px 0 0; color: #0b2545; text-transform: uppercase; letter-spacing: .03em; }
.meta { display: flex; justify-content: space-between; font-size: 8.5pt; color: #475569; margin-bottom: 6px; }
.filters { font-size: 8.5pt; color: #334155; margin-bottom: 8px; }
.filters span { margin-right: 16px; }
table.data { width: 100%; border-collapse: collapse; }
table.data th { background: #0b2545; color: #fff; font-size: 8pt; text-align: left; padding: 5px 6px; }
table.data td { border-bottom: 1px solid #cbd5e1; padding: 4px 6px; vertical-align: top; }
table.data tr:nth-child(even) td { background: #f8fafc; }
thead { display: table-header-group; }
tr { page-break-inside: avoid; }
.badge { display: inline-block; padding: 1px 6px; border-radius: 8px; font-size: 7.5pt; white-space: nowrap; }
.badge-draft { background: #e2e8f0; color: #334155; }
.badge-diajukan { background: #dbeafe; color: #1d4ed8; }
.badge-disetujui { background: #d1fae5; color: #047857; }
.badge-perlu_revisi { background: #fef3c7; color: #b45309; }
.badge-ditolak { background: #ffe4e6; color: #be123c; }
.footer { display: flex; justify-content: space-between; margin-top: 14px; font-size: 8.5pt; color: #475569; }
.ttd { text-align: right; min-width: 220px; }
.ttd strong { color: #1e293b; }
.catatan { margin-top: 6px; font-size: 8pt; color: #64748b; }
.empty { text-align: center; color: #64748b; padding: 16px; }
"#;

#[allow(clippy::too_many_arguments)]
fn html_laporan_rencana(
    rows: &[RencanaRow],
    total: i64,
    periode: &str,
    satker: &str,
    personel: &str,
    status: &str,
    dicetak_oleh: &str,
    dicetak_pada: &str,
    nama_satker: &HashMap<Uuid, String>,
) -> String {
    let logo = base64::engine::general_purpose::STANDARD.encode(include_bytes!(
        "../../../dashboard/static/logo-polda-bali.png"
    ));

    let mut baris = String::new();
    for (indeks, row) in rows.iter().enumerate() {
        let target = format!(
            "{} {}",
            row.target_kuantitas
                .map(|nilai| {
                    if nilai.fract() == 0.0 {
                        format!("{}", nilai as i64)
                    } else {
                        format!("{nilai}")
                    }
                })
                .unwrap_or_else(|| "-".into()),
            row.satuan.clone().unwrap_or_default()
        );
        baris.push_str(&format!(
            "<tr>\
                <td>{}</td>\
                <td><strong>{}</strong><br><span style=\"color:#64748b\">{}</span></td>\
                <td>{}</td>\
                <td><strong>{}</strong><br><span style=\"color:#64748b\">{}</span></td>\
                <td>{}</td>\
                <td>{}</td>\
                <td>{} – {}</td>\
                <td><span class=\"badge badge-{}\">{}</span></td>\
             </tr>",
            indeks + 1,
            esc(row.personel_nama.as_deref().unwrap_or("-")),
            esc(row.personel_nrp.as_deref().unwrap_or("-")),
            esc(nama_satker
                .get(&row.satker_id)
                .map(String::as_str)
                .unwrap_or("-")),
            esc(&row.kegiatan),
            esc(&row.program),
            label_jenis_rencana(row.jenis),
            esc(target.trim()),
            format_tanggal(row.tanggal_mulai),
            format_tanggal(row.tanggal_selesai),
            row.status.as_str(),
            label_status_rencana(row.status),
        ));
    }

    if rows.is_empty() {
        baris.push_str(
            "<tr><td colspan=\"8\" class=\"empty\">Tidak ada data sesuai filter.</td></tr>",
        );
    }

    let catatan = if total > rows.len() as i64 {
        format!(
            "Menampilkan {} dari {} rencana (batas cetak 5000). Persempit filter untuk hasil lengkap.",
            rows.len(),
            total
        )
    } else {
        format!("Total {} rencana kerja.", rows.len())
    };

    format!(
        r#"<!doctype html>
<html lang="id">
<head><meta charset="utf-8"><style>{css}</style></head>
<body>
  <table class="kop"><tr>
    <td class="logo"><img src="data:image/png;base64,{logo}" alt="Logo SIAP Bali"></td>
    <td class="identitas">
      <div class="instansi">Kepolisian Negara Republik Indonesia</div>
      <div class="instansi">Daerah Bali · Direktorat Reserse Kriminal Khusus</div>
      <h1>Laporan Rencana Kerja Personel</h1>
    </td>
  </tr></table>

  <div class="meta">
    <span><strong>Periode:</strong> {periode}</span>
    <span>Dicetak: {dicetak_pada}</span>
  </div>
  <div class="filters">
    <span><strong>Satker:</strong> {satker}</span>
    <span><strong>Personel:</strong> {personel}</span>
    <span><strong>Status:</strong> {status}</span>
  </div>

  <table class="data">
    <thead><tr>
      <th style="width:4%">No</th>
      <th style="width:15%">Personel</th>
      <th style="width:12%">Satker</th>
      <th style="width:25%">Program / Kegiatan</th>
      <th style="width:8%">Jenis</th>
      <th style="width:8%">Target</th>
      <th style="width:16%">Periode Kerja</th>
      <th style="width:12%">Status</th>
    </tr></thead>
    <tbody>{baris}</tbody>
  </table>

  <p class="catatan">{catatan}</p>
  <div class="footer">
    <div>Dicetak oleh: <strong>{dicetak_oleh}</strong></div>
    <div class="ttd">Bali, {dicetak_pada}<br><br><br><strong>{dicetak_oleh}</strong></div>
  </div>
</body>
</html>"#,
        css = CSS_LAPORAN,
        logo = logo,
        periode = esc(periode),
        satker = esc(satker),
        personel = esc(personel),
        status = esc(status),
        dicetak_pada = esc(dicetak_pada),
        dicetak_oleh = esc(dicetak_oleh),
        baris = baris,
        catatan = esc(&catatan),
    )
}
