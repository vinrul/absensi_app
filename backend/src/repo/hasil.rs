use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::StatusHasil;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct HasilRow {
    pub id: Uuid,
    pub rencana_id: Uuid,
    pub personel_id: Uuid,
    pub satker_id: Uuid,
    pub uraian_pelaksanaan: String,
    pub progres_persen: f64,
    pub jumlah_realisasi: Option<f64>,
    pub kendala: Option<String>,
    pub tanggal_realisasi: NaiveDate,
    pub status: StatusHasil,
    pub skor_kuantitas: Option<f64>,
    pub skor_kualitas: Option<f64>,
    pub skor_waktu: Option<f64>,
    pub skor_total: Option<f64>,
    pub verified_by: Option<Uuid>,
    pub catatan_verifikator: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub personel_nama: Option<String>,
    pub personel_nrp: Option<String>,
    pub rencana_kegiatan: Option<String>,
}

const LIST_SQL: &str = r#"
    SELECT h.id, h.rencana_id, h.personel_id, h.satker_id, h.uraian_pelaksanaan,
           h.progres_persen::float8 AS progres_persen,
           h.jumlah_realisasi::float8 AS jumlah_realisasi,
           h.kendala, h.tanggal_realisasi, h.status,
           h.skor_kuantitas::float8 AS skor_kuantitas,
           h.skor_kualitas::float8 AS skor_kualitas,
           h.skor_waktu::float8 AS skor_waktu,
           h.skor_total::float8 AS skor_total,
           h.verified_by, h.catatan_verifikator, h.verified_at,
           p.nama AS personel_nama, p.nrp AS personel_nrp, r.kegiatan AS rencana_kegiatan
      FROM hasil_kerja h
      JOIN personel p ON p.id = h.personel_id
      JOIN rencana_kerja r ON r.id = h.rencana_id
     WHERE h.satker_id = ANY($1::uuid[])
       AND ($2::uuid IS NULL OR h.personel_id = $2)
       AND ($3::status_hasil IS NULL OR h.status = $3)
     ORDER BY h.created_at DESC
     LIMIT $4 OFFSET $5
"#;

const LIST_COUNT: &str = r#"
    SELECT count(*)
      FROM hasil_kerja h
     WHERE h.satker_id = ANY($1::uuid[])
       AND ($2::uuid IS NULL OR h.personel_id = $2)
       AND ($3::status_hasil IS NULL OR h.status = $3)
"#;

pub struct ListFilter {
    pub satker_ids: Vec<Uuid>,
    pub personel_id: Option<Uuid>,
    pub status: Option<StatusHasil>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list(pool: &PgPool, filter: &ListFilter) -> AppResult<(Vec<HasilRow>, i64)> {
    let rows = sqlx::query_as::<_, HasilRow>(LIST_SQL)
        .bind(&filter.satker_ids)
        .bind(filter.personel_id)
        .bind(filter.status)
        .bind(filter.limit)
        .bind(filter.offset)
        .fetch_all(pool)
        .await?;

    let total: i64 = sqlx::query_scalar::<_, i64>(LIST_COUNT)
        .bind(&filter.satker_ids)
        .bind(filter.personel_id)
        .bind(filter.status)
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}

pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Option<HasilRow>> {
    const SQL: &str = r#"
        SELECT h.id, h.rencana_id, h.personel_id, h.satker_id, h.uraian_pelaksanaan,
               h.progres_persen::float8 AS progres_persen,
               h.jumlah_realisasi::float8 AS jumlah_realisasi,
               h.kendala, h.tanggal_realisasi, h.status,
               h.skor_kuantitas::float8 AS skor_kuantitas,
               h.skor_kualitas::float8 AS skor_kualitas,
               h.skor_waktu::float8 AS skor_waktu,
               h.skor_total::float8 AS skor_total,
               h.verified_by, h.catatan_verifikator, h.verified_at,
               p.nama AS personel_nama, p.nrp AS personel_nrp, r.kegiatan AS rencana_kegiatan
          FROM hasil_kerja h
          JOIN personel p ON p.id = h.personel_id
          JOIN rencana_kerja r ON r.id = h.rencana_id
         WHERE h.id = $1
    "#;
    Ok(sqlx::query_as::<_, HasilRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn by_personel(
    pool: &PgPool,
    personel_id: Uuid,
    periode_id: Uuid,
) -> AppResult<Vec<HasilRow>> {
    const SQL: &str = r#"
        SELECT h.id, h.rencana_id, h.personel_id, h.satker_id, h.uraian_pelaksanaan,
               h.progres_persen::float8 AS progres_persen,
               h.jumlah_realisasi::float8 AS jumlah_realisasi,
               h.kendala, h.tanggal_realisasi, h.status,
               h.skor_kuantitas::float8 AS skor_kuantitas,
               h.skor_kualitas::float8 AS skor_kualitas,
               h.skor_waktu::float8 AS skor_waktu,
               h.skor_total::float8 AS skor_total,
               h.verified_by, h.catatan_verifikator, h.verified_at,
               p.nama AS personel_nama, p.nrp AS personel_nrp, r.kegiatan AS rencana_kegiatan
          FROM hasil_kerja h
          JOIN personel p ON p.id = h.personel_id
          JOIN rencana_kerja r ON r.id = h.rencana_id
         WHERE h.personel_id = $1 AND r.periode_id = $2
    "#;
    Ok(sqlx::query_as::<_, HasilRow>(SQL)
        .bind(personel_id)
        .bind(periode_id)
        .fetch_all(pool)
        .await?)
}

pub struct NewHasil {
    pub id: Uuid,
    pub rencana_id: Uuid,
    pub personel_id: Uuid,
    pub satker_id: Uuid,
    pub uraian_pelaksanaan: String,
    pub progres_persen: f64,
    pub jumlah_realisasi: Option<f64>,
    pub kendala: Option<String>,
    pub tanggal_realisasi: NaiveDate,
}

pub async fn create(pool: &PgPool, data: &NewHasil) -> AppResult<()> {
    const SQL: &str = r#"
        INSERT INTO hasil_kerja (id, rencana_id, personel_id, satker_id, uraian_pelaksanaan,
                                 progres_persen, jumlah_realisasi, kendala, tanggal_realisasi, status)
        VALUES ($1, $2, $3, $4, $5, $6::float8::numeric, $7::float8::numeric, $8, $9, 'diajukan')
    "#;
    sqlx::query(SQL)
        .bind(data.id)
        .bind(data.rencana_id)
        .bind(data.personel_id)
        .bind(data.satker_id)
        .bind(&data.uraian_pelaksanaan)
        .bind(data.progres_persen)
        .bind(data.jumlah_realisasi)
        .bind(&data.kendala)
        .bind(data.tanggal_realisasi)
        .execute(pool)
        .await?;
    Ok(())
}

pub struct SkorTersimpan {
    pub kuantitas: f64,
    pub kualitas: f64,
    pub waktu: f64,
    pub total: f64,
}

pub async fn verifikasi(
    pool: &PgPool,
    id: Uuid,
    verified_by: Uuid,
    skor: &SkorTersimpan,
    catatan: Option<&str>,
) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE hasil_kerja
           SET status = 'terverifikasi',
               skor_kuantitas = $2::float8::numeric,
               skor_kualitas = $3::float8::numeric,
               skor_waktu = $4::float8::numeric,
               skor_total = $5::float8::numeric,
               verified_by = $6,
               catatan_verifikator = $7,
               verified_at = now()
         WHERE id = $1
    "#;
    let result = sqlx::query(SQL)
        .bind(id)
        .bind(skor.kuantitas)
        .bind(skor.kualitas)
        .bind(skor.waktu)
        .bind(skor.total)
        .bind(verified_by)
        .bind(catatan)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Hasil kerja tidak ditemukan".into()));
    }
    Ok(())
}

pub async fn minta_revisi(
    pool: &PgPool,
    id: Uuid,
    verified_by: Uuid,
    catatan: &str,
) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE hasil_kerja
           SET status = 'perlu_revisi',
               verified_by = $2,
               catatan_verifikator = $3,
               verified_at = now()
         WHERE id = $1
    "#;
    sqlx::query(SQL)
        .bind(id)
        .bind(verified_by)
        .bind(catatan)
        .execute(pool)
        .await?;
    Ok(())
}
