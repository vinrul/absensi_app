use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
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
use crate::repo::{ListResponse, pagination};
use crate::service::akses;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_rencana).post(create_rencana))
        .route("/persetujuan", get(list_persetujuan))
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

pub async fn list_persetujuan(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<ListResponse<RencanaRow>>> {
    if !(user.is_admin_any() || user.is_atasan()) {
        return Err(AppError::Forbidden(
            "Hanya atasan atau admin yang dapat melihat persetujuan".into(),
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
    let tahapan = req
        .tahapan
        .unwrap_or_default()
        .into_iter()
        .map(|item| NewTahapan {
            uraian: item.uraian,
            target_kuantitas: item.target_kuantitas,
            satuan: item.satuan,
            bobot: item.bobot,
        })
        .collect();

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
    if !(user.is_admin_any() || user.is_atasan()) {
        return Err(AppError::Forbidden(
            "Hanya atasan atau admin yang dapat memutuskan rencana".into(),
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
