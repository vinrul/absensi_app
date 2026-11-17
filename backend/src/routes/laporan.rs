use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::error::{AppError, AppResult};
use crate::repo::{penilaian, periode, satker};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/penilaian", get(laporan_penilaian))
}

#[derive(Debug, Deserialize)]
pub struct LaporanQuery {
    pub periode_id: Option<Uuid>,
    pub satker_id: Option<Uuid>,
    pub unit: Option<String>,
    pub format: Option<String>,
}

pub async fn laporan_penilaian(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<LaporanQuery>,
) -> AppResult<Response> {
    if !(user.is_admin_any() || user.is_pimpinan_any() || user.is_atasan()) {
        return Err(AppError::Forbidden(
            "Laporan hanya untuk atasan/pimpinan/admin".into(),
        ));
    }

    let periode_row = periode::find_or_current(&state.pool, query.periode_id).await?;

    let mut scope = crate::routes::personel::scope_for(&state, &user).await?;
    if let Some(satker_id) = query.satker_id {
        if !scope.contains(&satker_id) {
            return Err(AppError::Forbidden(
                "Anda tidak berhak melihat laporan satker tersebut".into(),
            ));
        }
        scope = vec![satker_id];
    }

    let (rows, _) = penilaian::list(
        &state.pool,
        &penilaian::ListFilter {
            periode_id: periode_row.id,
            satker_ids: scope.clone(),
            unit: query.unit.clone().filter(|v| !v.is_empty()),
            kategori: None,
            personel_id: None,
            limit: 1000,
            offset: 0,
        },
    )
    .await?;

    let satker_nama = match query.satker_id {
        Some(id) => satker::find(&state.pool, id)
            .await?
            .map(|row| row.nama)
            .unwrap_or_else(|| "Semua Satker".to_string()),
        None => "Semua Satker".to_string(),
    };

    if query.format.as_deref() == Some("csv") {
        let mut csv = String::from(
            "No,NRP,Nama,Pangkat,Jabatan/Unit,Absensi,Rencana Kerja,Hasil Kerja,Output,Nilai Akhir,Kategori\n",
        );
        for (index, row) in rows.iter().enumerate() {
            csv.push_str(&format!(
                "{},{},{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{}\n",
                index + 1,
                row.personel_nrp,
                row.personel_nama.replace(',', " "),
                row.pangkat,
                format!("{} / {}", row.jabatan, row.unit.clone().unwrap_or_default())
                    .replace(',', " "),
                row.skor_absensi,
                row.skor_rencana_kerja,
                row.skor_hasil_kerja,
                row.skor_output,
                row.nilai_akhir,
                row.kategori.as_str(),
            ));
        }

        let nama_berkas = format!("laporan-{}.csv", periode_row.tanggal_mulai);
        return Ok((
            [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
                (
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{nama_berkas}\""),
                ),
            ],
            csv,
        )
            .into_response());
    }

    Ok(Json(serde_json::json!({
        "periode": {
            "id": periode_row.id,
            "nama": periode_row.nama,
            "tanggal_mulai": periode_row.tanggal_mulai,
            "tanggal_selesai": periode_row.tanggal_selesai,
        },
        "satker": satker_nama,
        "data": rows,
        "dicetak_oleh": user.claims().nama,
        "dicetak_pada": chrono::Utc::now(),
    }))
    .into_response())
}
