use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::routing::{delete, get, post};
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::domain::enums::{BerkasKategori, BerkasOwner};
use crate::error::{AppError, AppResult};
use crate::repo::lampiran::{self, LampiranRow};
use crate::repo::{absensi, hasil, output, rencana};
use crate::service::storage;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(upload))
        .route("/raw/{*key}", get(raw))
        .route("/{id}", delete(hapus))
}

fn parse_owner(value: &str) -> AppResult<BerkasOwner> {
    match value {
        "rencana" => Ok(BerkasOwner::Rencana),
        "hasil" => Ok(BerkasOwner::Hasil),
        "output" => Ok(BerkasOwner::Output),
        "absensi" => Ok(BerkasOwner::Absensi),
        other => Err(AppError::Unprocessable(format!(
            "owner_type tidak dikenal: {other}"
        ))),
    }
}

fn parse_kategori(value: &str) -> AppResult<BerkasKategori> {
    match value {
        "lampiran" => Ok(BerkasKategori::Lampiran),
        "bukti" => Ok(BerkasKategori::Bukti),
        "foto" => Ok(BerkasKategori::Foto),
        other => Err(AppError::Unprocessable(format!(
            "kategori tidak dikenal: {other}"
        ))),
    }
}

/// Memastikan pemakai berhak melampirkan berkas pada entitas tersebut.
async fn pastikan_akses(
    state: &AppState,
    user: &AuthUser,
    owner_type: BerkasOwner,
    owner_id: Uuid,
) -> AppResult<Uuid> {
    let (personel_id, satker_id) = match owner_type {
        BerkasOwner::Rencana => {
            let row = rencana::find(&state.pool, owner_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Rencana kerja tidak ditemukan".into()))?;
            (row.personel_id, row.satker_id)
        }
        BerkasOwner::Hasil => {
            let row = hasil::find(&state.pool, owner_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Hasil kerja tidak ditemukan".into()))?;
            (row.personel_id, row.satker_id)
        }
        BerkasOwner::Output => {
            let row = output::find(&state.pool, owner_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Output tidak ditemukan".into()))?;
            (row.personel_id, row.satker_id)
        }
        BerkasOwner::Absensi => {
            let row = absensi::find(&state.pool, owner_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Absensi tidak ditemukan".into()))?;
            (row.personel_id, row.satker_id)
        }
    };

    if personel_id != user.id()
        && !((user.is_admin_any() || user.is_atasan()) && user.satker_akses().contains(&satker_id))
    {
        return Err(AppError::Forbidden(
            "Anda tidak berhak melampirkan berkas pada data ini".into(),
        ));
    }

    Ok(satker_id)
}

pub async fn upload(
    State(state): State<AppState>,
    user: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<LampiranRow>> {
    let mut owner_type: Option<BerkasOwner> = None;
    let mut owner_id: Option<Uuid> = None;
    let mut kategori = BerkasKategori::Lampiran;
    let mut file_name = String::from("berkas");
    let mut bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::BadRequest("Formulir unggahan tidak valid".into()))?
    {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "owner_type" => {
                let value = field
                    .text()
                    .await
                    .map_err(|_| AppError::BadRequest("owner_type tidak valid".into()))?;
                owner_type = Some(parse_owner(value.trim())?);
            }
            "owner_id" => {
                let value = field
                    .text()
                    .await
                    .map_err(|_| AppError::BadRequest("owner_id tidak valid".into()))?;
                owner_id = Some(
                    Uuid::parse_str(value.trim())
                        .map_err(|_| AppError::BadRequest("owner_id harus berupa UUID".into()))?,
                );
            }
            "kategori" => {
                let value = field
                    .text()
                    .await
                    .map_err(|_| AppError::BadRequest("kategori tidak valid".into()))?;
                kategori = parse_kategori(value.trim())?;
            }
            "file" => {
                if let Some(name) = field.file_name() {
                    file_name = name.to_string();
                }
                bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| AppError::BadRequest("berkas gagal dibaca".into()))?
                        .to_vec(),
                );
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let owner_type =
        owner_type.ok_or_else(|| AppError::BadRequest("owner_type wajib diisi".into()))?;
    let owner_id = owner_id.ok_or_else(|| AppError::BadRequest("owner_id wajib diisi".into()))?;
    let bytes = bytes.ok_or_else(|| AppError::BadRequest("Berkas wajib diunggah".into()))?;

    let batas = storage::batas_ukuran(match owner_type {
        BerkasOwner::Absensi => "absensi",
        BerkasOwner::Output => "output",
        _ => "lampiran",
    });

    if bytes.is_empty() {
        return Err(AppError::BadRequest("Berkas kosong".into()));
    }
    if bytes.len() > batas {
        return Err(AppError::PayloadTooLarge(format!(
            "Ukuran berkas maksimal {} MB",
            batas / (1024 * 1024)
        )));
    }

    let (ext, mime) = storage::deteksi_berkas(&bytes).ok_or_else(|| {
        AppError::Unprocessable("Format berkas harus PDF, JPG, PNG, atau MP4".into())
    })?;

    let satker_id = pastikan_akses(&state, &user, owner_type, owner_id).await?;

    let id = Uuid::now_v7();
    let owner_label = match owner_type {
        BerkasOwner::Rencana => "rencana",
        BerkasOwner::Hasil => "hasil",
        BerkasOwner::Output => "output",
        BerkasOwner::Absensi => "absensi",
    };
    let key = storage::kunci_lampiran(satker_id, owner_label, owner_id, id, ext);
    state.storage.save(&key, &bytes).await?;

    let lampiran_id = lampiran::create(
        &state.pool,
        &lampiran::LampiranBaru {
            owner_type,
            owner_id,
            kategori,
            file_name: &file_name,
            mime,
            size_bytes: bytes.len() as i64,
            storage_key: &key,
            uploaded_by: user.id(),
        },
    )
    .await?;

    let row = lampiran::find(&state.pool, lampiran_id)
        .await?
        .ok_or_else(|| AppError::Internal("lampiran gagal dibaca".into()))?;

    Ok(Json(row))
}

pub async fn raw(
    State(state): State<AppState>,
    user: AuthUser,
    Path(key): Path<String>,
) -> AppResult<Response> {
    // Cegah path traversal: hanya karakter aman dan tanpa "..".
    if key.contains("..") || key.contains('\\') || key.starts_with('/') {
        return Err(AppError::BadRequest("Kunci berkas tidak valid".into()));
    }

    let row = lampiran::find_by_storage_key(&state.pool, &key)
        .await?
        .ok_or_else(|| AppError::NotFound("Berkas tidak ditemukan".into()))?;
    pastikan_akses(&state, &user, row.owner_type, row.owner_id).await?;

    let bytes = state.storage.read(&key).await?;
    let mime = mime_guess::from_path(&key).first_or_octet_stream();

    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref())
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=600"),
    );

    Ok(response)
}

pub async fn hapus(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let row = lampiran::find(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Lampiran tidak ditemukan".into()))?;

    pastikan_akses(&state, &user, row.owner_type, row.owner_id).await?;

    lampiran::delete(&state.pool, id).await?;
    state.storage.remove(&row.storage_key).await?;

    Ok(Json(serde_json::json!({ "message": "Lampiran dihapus" })))
}
