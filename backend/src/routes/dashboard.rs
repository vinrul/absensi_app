use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::KategoriKinerja;
use crate::error::AppResult;
use crate::repo::penilaian;
use crate::repo::periode;
use crate::service::waktu::hari_ini_wita;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/ringkasan", get(ringkasan))
        .route("/capaian-indikator", get(capaian_indikator))
        .route("/tren", get(tren))
        .route("/sebaran-kategori", get(sebaran_kategori))
        .route("/rekap", get(rekap))
}

#[derive(Debug, Deserialize)]
pub struct FilterQuery {
    pub periode_id: Option<Uuid>,
    pub satker_id: Option<Uuid>,
    pub unit: Option<String>,
}

fn boleh_dashboard(user: &AuthUser) -> bool {
    user.is_admin_any() || user.is_pimpinan_any() || user.is_atasan()
}

async fn siapkan(
    state: &AppState,
    user: &AuthUser,
    query: &FilterQuery,
) -> AppResult<(Uuid, String, Vec<Uuid>)> {
    if !boleh_dashboard(user) {
        return Err(crate::error::AppError::Forbidden(
            "Dashboard hanya untuk atasan/pimpinan/admin".into(),
        ));
    }

    let periode_row = periode::find_or_current(&state.pool, query.periode_id).await?;
    let mut scope = crate::routes::personel::scope_for(state, user).await?;

    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(crate::error::AppError::Forbidden(
                "Anda tidak berhak melihat dashboard satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }

    Ok((periode_row.id, periode_row.nama, scope))
}

#[derive(Debug, Serialize)]
pub struct RingkasanResponse {
    pub periode_id: Uuid,
    pub periode_nama: String,
    pub total_personel: i64,
    pub hadir_hari_ini: i64,
    pub persen_hadir: f64,
    pub total_rencana: i64,
    pub total_hasil: i64,
    pub total_output: i64,
    pub rata_rata_nilai: f64,
}

async fn hitung_ringkasan(
    pool: &PgPool,
    periode_id: Uuid,
    satker_ids: &[Uuid],
) -> AppResult<RingkasanResponse> {
    const TOTAL_PERSONEL: &str = r#"
        SELECT count(*) FROM personel
         WHERE satker_id = ANY($1::uuid[]) AND deleted_at IS NULL AND is_active
           AND jenis_akun = 'personel'
    "#;
    const HADIR: &str = r#"
        SELECT count(DISTINCT personel_id) FROM absensi
         WHERE satker_id = ANY($1::uuid[])
           AND tanggal = $2
           AND jenis = 'masuk'
           AND status_kehadiran IN ('dinas_kantor', 'dinas_luar', 'piket')
    "#;
    const TOTAL_RENCANA: &str = r#"
        SELECT count(*) FROM rencana_kerja
         WHERE satker_id = ANY($1::uuid[]) AND periode_id = $2
    "#;
    const TOTAL_HASIL: &str = r#"
        SELECT count(*) FROM hasil_kerja h
          JOIN rencana_kerja r ON r.id = h.rencana_id
         WHERE h.satker_id = ANY($1::uuid[]) AND r.periode_id = $2 AND h.status = 'terverifikasi'
    "#;
    const TOTAL_OUTPUT: &str = r#"
        SELECT count(*) FROM output o
          JOIN rencana_kerja r ON r.id = o.rencana_id
         WHERE o.satker_id = ANY($1::uuid[]) AND r.periode_id = $2 AND o.status = 'disetujui'
    "#;

    let total_personel: i64 = sqlx::query_scalar(TOTAL_PERSONEL)
        .bind(satker_ids)
        .fetch_one(pool)
        .await?;

    let hadir: i64 = sqlx::query_scalar(HADIR)
        .bind(satker_ids)
        .bind(hari_ini_wita())
        .fetch_one(pool)
        .await?;

    let total_rencana: i64 = sqlx::query_scalar(TOTAL_RENCANA)
        .bind(satker_ids)
        .bind(periode_id)
        .fetch_one(pool)
        .await?;

    let total_hasil: i64 = sqlx::query_scalar(TOTAL_HASIL)
        .bind(satker_ids)
        .bind(periode_id)
        .fetch_one(pool)
        .await?;

    let total_output: i64 = sqlx::query_scalar(TOTAL_OUTPUT)
        .bind(satker_ids)
        .bind(periode_id)
        .fetch_one(pool)
        .await?;

    let rata = penilaian::rata_indikator(pool, periode_id, satker_ids).await?;
    let rata_rata_nilai = rata.map(|(_, _, _, _, nilai)| nilai).unwrap_or(0.0);

    let persen_hadir = if total_personel > 0 {
        (hadir as f64 / total_personel as f64 * 1000.0).round() / 10.0
    } else {
        0.0
    };

    Ok(RingkasanResponse {
        periode_id,
        periode_nama: String::new(),
        total_personel,
        hadir_hari_ini: hadir,
        persen_hadir,
        total_rencana,
        total_hasil,
        total_output,
        rata_rata_nilai,
    })
}

pub async fn ringkasan(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<FilterQuery>,
) -> AppResult<Json<RingkasanResponse>> {
    let (periode_id, periode_nama, scope) = siapkan(&state, &user, &query).await?;
    let mut data = hitung_ringkasan(&state.pool, periode_id, &scope).await?;
    data.periode_nama = periode_nama;
    Ok(Json(data))
}

#[derive(Debug, Serialize)]
pub struct CapaianResponse {
    pub absensi: f64,
    pub rencana_kerja: f64,
    pub hasil_kerja: f64,
    pub output: f64,
    pub nilai_akhir: f64,
}

pub async fn capaian_indikator(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<FilterQuery>,
) -> AppResult<Json<CapaianResponse>> {
    let (periode_id, _nama, scope) = siapkan(&state, &user, &query).await?;
    let rata = penilaian::rata_indikator(&state.pool, periode_id, &scope).await?;

    let (absensi, rencana, hasil, output, nilai) = rata.unwrap_or((0.0, 0.0, 0.0, 0.0, 0.0));

    Ok(Json(CapaianResponse {
        absensi,
        rencana_kerja: rencana,
        hasil_kerja: hasil,
        output,
        nilai_akhir: nilai,
    }))
}

#[derive(Debug, Serialize)]
pub struct TrenPoint {
    pub periode_id: Uuid,
    pub nama: String,
    pub rata_rata: f64,
}

#[derive(Debug, Deserialize)]
pub struct TrenQuery {
    pub bulan: Option<i64>,
    pub satker_id: Option<Uuid>,
}

pub async fn tren(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<TrenQuery>,
) -> AppResult<Json<Vec<TrenPoint>>> {
    if !boleh_dashboard(&user) {
        return Err(crate::error::AppError::Forbidden(
            "Dashboard hanya untuk atasan/pimpinan/admin".into(),
        ));
    }
    let scope = {
        let mut scope = crate::routes::personel::scope_for(&state, &user).await?;
        if let Some(satker_id) = query.satker_id {
            if !scope.contains(&satker_id) {
                return Err(crate::error::AppError::Forbidden(
                    "Anda tidak berhak melihat satker tersebut".into(),
                ));
            }
            scope = vec![satker_id];
        }
        scope
    };

    const SQL: &str = r#"
        SELECT p.id, p.nama, COALESCE(avg(n.nilai_akhir)::float8, 0) AS rata
          FROM periode p
          LEFT JOIN penilaian n
                 ON n.periode_id = p.id
                AND n.satker_id = ANY($1::uuid[])
         GROUP BY p.id, p.nama, p.tanggal_mulai
         ORDER BY p.tanggal_mulai DESC
         LIMIT $2
    "#;

    let batas = query.bulan.unwrap_or(3).clamp(1, 12);
    let rows = sqlx::query_as::<_, (Uuid, String, f64)>(SQL)
        .bind(&scope)
        .bind(batas)
        .fetch_all(&state.pool)
        .await?;

    let mut data: Vec<TrenPoint> = rows
        .into_iter()
        .map(|(periode_id, nama, rata_rata)| TrenPoint {
            periode_id,
            nama,
            rata_rata,
        })
        .collect();

    data.reverse();
    Ok(Json(data))
}

#[derive(Debug, Serialize)]
pub struct SebaranItem {
    pub kategori: KategoriKinerja,
    pub jumlah: i64,
    pub persen: f64,
}

pub async fn sebaran_kategori(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<FilterQuery>,
) -> AppResult<Json<Vec<SebaranItem>>> {
    let (periode_id, _nama, scope) = siapkan(&state, &user, &query).await?;
    let rows = penilaian::sebaran(&state.pool, periode_id, &scope).await?;

    let total: i64 = rows.iter().map(|(_, jumlah)| jumlah).sum();

    let data = KategoriKinerja::ALL
        .iter()
        .map(|kategori| {
            let jumlah = rows
                .iter()
                .find(|(k, _)| k == kategori)
                .map(|(_, jumlah)| *jumlah)
                .unwrap_or(0);
            SebaranItem {
                kategori: *kategori,
                jumlah,
                persen: if total > 0 {
                    (jumlah as f64 / total as f64 * 1000.0).round() / 10.0
                } else {
                    0.0
                },
            }
        })
        .collect();

    Ok(Json(data))
}

#[derive(Debug, Deserialize)]
pub struct RekapQuery {
    pub periode_id: Option<Uuid>,
    pub satker_id: Option<Uuid>,
    pub unit: Option<String>,
    pub per_page: Option<i64>,
    pub page: Option<i64>,
}

pub async fn rekap(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<RekapQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let filter = FilterQuery {
        periode_id: query.periode_id,
        satker_id: query.satker_id,
        unit: query.unit.clone(),
    };
    let (periode_id, periode_nama, scope) = siapkan(&state, &user, &filter).await?;
    let (page, per_page, offset) = crate::repo::pagination(query.page, query.per_page);

    let (rows, total) = penilaian::list(
        &state.pool,
        &penilaian::ListFilter {
            periode_id,
            satker_ids: scope,
            unit: query.unit.filter(|v| !v.is_empty()),
            kategori: None,
            personel_id: None,
            limit: per_page,
            offset,
        },
    )
    .await?;

    Ok(Json(serde_json::json!({
        "periode": { "id": periode_id, "nama": periode_nama },
        "data": rows,
        "meta": { "page": page, "per_page": per_page, "total": total },
    })))
}
