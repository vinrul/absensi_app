use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::{DasarTugasJenis, JenisRencana, StatusRencana};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct RencanaRow {
    pub id: Uuid,
    pub personel_id: Uuid,
    pub satker_id: Uuid,
    pub periode_id: Uuid,
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
    pub status: StatusRencana,
    pub atasan_id: Option<Uuid>,
    pub catatan_atasan: Option<String>,
    pub diajukan_at: Option<DateTime<Utc>>,
    pub diputuskan_at: Option<DateTime<Utc>>,
    pub jumlah_revisi: i32,
    pub personel_nama: Option<String>,
    pub personel_nrp: Option<String>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct TahapanRow {
    pub id: Uuid,
    pub urutan: i32,
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

const LIST_SQL: &str = r#"
    SELECT r.id, r.personel_id, r.satker_id, r.periode_id, r.program, r.kegiatan, r.uraian,
           r.tujuan, r.indikator_kinerja, r.jenis, r.dasar_jenis, r.dasar_nomor, r.dasar_tanggal,
           r.dasar_perihal, r.target_kuantitas::float8 AS target_kuantitas, r.satuan,
           r.target_kualitas, r.target_waktu, r.tanggal_mulai, r.tanggal_selesai, r.status,
           r.atasan_id, r.catatan_atasan, r.diajukan_at, r.diputuskan_at, r.jumlah_revisi,
           p.nama AS personel_nama, p.nrp AS personel_nrp, p.unit
      FROM rencana_kerja r
      JOIN personel p ON p.id = r.personel_id
     WHERE r.satker_id = ANY($1::uuid[])
       AND ($2::uuid IS NULL OR r.personel_id = $2)
       AND ($3::status_rencana IS NULL OR r.status = $3)
       AND ($4::uuid IS NULL OR r.periode_id = $4)
       AND ($5::uuid IS NULL OR r.atasan_id = $5)
       AND (NOT $6::bool OR r.status = 'diajukan')
     ORDER BY r.created_at DESC
     LIMIT $7 OFFSET $8
"#;

const LIST_COUNT: &str = r#"
    SELECT count(*)
      FROM rencana_kerja r
     WHERE r.satker_id = ANY($1::uuid[])
       AND ($2::uuid IS NULL OR r.personel_id = $2)
       AND ($3::status_rencana IS NULL OR r.status = $3)
       AND ($4::uuid IS NULL OR r.periode_id = $4)
       AND ($5::uuid IS NULL OR r.atasan_id = $5)
       AND (NOT $6::bool OR r.status = 'diajukan')
"#;

pub struct ListFilter {
    pub satker_ids: Vec<Uuid>,
    pub personel_id: Option<Uuid>,
    pub status: Option<StatusRencana>,
    pub periode_id: Option<Uuid>,
    pub atasan_id: Option<Uuid>,
    pub hanya_diajukan: bool,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list(pool: &PgPool, filter: &ListFilter) -> AppResult<(Vec<RencanaRow>, i64)> {
    let rows = sqlx::query_as::<_, RencanaRow>(LIST_SQL)
        .bind(&filter.satker_ids)
        .bind(filter.personel_id)
        .bind(filter.status)
        .bind(filter.periode_id)
        .bind(filter.atasan_id)
        .bind(filter.hanya_diajukan)
        .bind(filter.limit)
        .bind(filter.offset)
        .fetch_all(pool)
        .await?;

    let total: i64 = sqlx::query_scalar::<_, i64>(LIST_COUNT)
        .bind(&filter.satker_ids)
        .bind(filter.personel_id)
        .bind(filter.status)
        .bind(filter.periode_id)
        .bind(filter.atasan_id)
        .bind(filter.hanya_diajukan)
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}

pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Option<RencanaRow>> {
    const SQL: &str = r#"
        SELECT r.id, r.personel_id, r.satker_id, r.periode_id, r.program, r.kegiatan, r.uraian,
               r.tujuan, r.indikator_kinerja, r.jenis, r.dasar_jenis, r.dasar_nomor, r.dasar_tanggal,
               r.dasar_perihal, r.target_kuantitas::float8 AS target_kuantitas, r.satuan,
               r.target_kualitas, r.target_waktu, r.tanggal_mulai, r.tanggal_selesai, r.status,
               r.atasan_id, r.catatan_atasan, r.diajukan_at, r.diputuskan_at, r.jumlah_revisi,
               p.nama AS personel_nama, p.nrp AS personel_nrp, p.unit
          FROM rencana_kerja r
          JOIN personel p ON p.id = r.personel_id
         WHERE r.id = $1
    "#;
    Ok(sqlx::query_as::<_, RencanaRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn by_personel(
    pool: &PgPool,
    personel_id: Uuid,
    periode_id: Uuid,
) -> AppResult<Vec<RencanaRow>> {
    const SQL: &str = r#"
        SELECT r.id, r.personel_id, r.satker_id, r.periode_id, r.program, r.kegiatan, r.uraian,
               r.tujuan, r.indikator_kinerja, r.jenis, r.dasar_jenis, r.dasar_nomor, r.dasar_tanggal,
               r.dasar_perihal, r.target_kuantitas::float8 AS target_kuantitas, r.satuan,
               r.target_kualitas, r.target_waktu, r.tanggal_mulai, r.tanggal_selesai, r.status,
               r.atasan_id, r.catatan_atasan, r.diajukan_at, r.diputuskan_at, r.jumlah_revisi,
               p.nama AS personel_nama, p.nrp AS personel_nrp, p.unit
          FROM rencana_kerja r
          JOIN personel p ON p.id = r.personel_id
         WHERE r.personel_id = $1 AND r.periode_id = $2
    "#;
    Ok(sqlx::query_as::<_, RencanaRow>(SQL)
        .bind(personel_id)
        .bind(periode_id)
        .fetch_all(pool)
        .await?)
}

pub async fn tahapan(pool: &PgPool, rencana_id: Uuid) -> AppResult<Vec<TahapanRow>> {
    const SQL: &str = r#"
        SELECT id, urutan, uraian, target_kuantitas::float8 AS target_kuantitas, satuan,
               bobot::float8 AS bobot, program, kegiatan, tujuan, indikator_kinerja,
               target_kualitas, tanggal_mulai, tanggal_selesai
          FROM rencana_tahapan
         WHERE rencana_id = $1
         ORDER BY urutan
    "#;
    Ok(sqlx::query_as::<_, TahapanRow>(SQL)
        .bind(rencana_id)
        .fetch_all(pool)
        .await?)
}

pub struct NewTahapan {
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

pub struct NewRencana {
    pub id: Uuid,
    pub personel_id: Uuid,
    pub satker_id: Uuid,
    pub periode_id: Uuid,
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
    pub atasan_id: Option<Uuid>,
    pub status: StatusRencana,
    pub tahapan: Vec<NewTahapan>,
}

pub async fn create(pool: &PgPool, data: &NewRencana) -> AppResult<()> {
    const SQL: &str = r#"
        INSERT INTO rencana_kerja (
            id, personel_id, satker_id, periode_id, program, kegiatan, uraian, tujuan,
            indikator_kinerja, jenis, dasar_jenis, dasar_nomor, dasar_tanggal, dasar_perihal,
            target_kuantitas, satuan, target_kualitas, target_waktu, tanggal_mulai,
            tanggal_selesai, status, atasan_id
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
            $15::float8::numeric, $16, $17, $18, $19, $20, $21, $22
        )
    "#;

    let mut tx = pool.begin().await?;

    sqlx::query(SQL)
        .bind(data.id)
        .bind(data.personel_id)
        .bind(data.satker_id)
        .bind(data.periode_id)
        .bind(&data.program)
        .bind(&data.kegiatan)
        .bind(&data.uraian)
        .bind(&data.tujuan)
        .bind(&data.indikator_kinerja)
        .bind(data.jenis)
        .bind(data.dasar_jenis)
        .bind(&data.dasar_nomor)
        .bind(data.dasar_tanggal)
        .bind(&data.dasar_perihal)
        .bind(data.target_kuantitas)
        .bind(&data.satuan)
        .bind(&data.target_kualitas)
        .bind(data.target_waktu)
        .bind(data.tanggal_mulai)
        .bind(data.tanggal_selesai)
        .bind(data.status)
        .bind(data.atasan_id)
        .execute(&mut *tx)
        .await?;

    for (index, tahap) in data.tahapan.iter().enumerate() {
        simpan_tahapan(&mut tx, data.id, index, tahap).await?;
    }

    tx.commit().await?;
    Ok(())
}

async fn simpan_tahapan(
    conn: &mut sqlx::PgConnection,
    rencana_id: Uuid,
    index: usize,
    tahap: &NewTahapan,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO rencana_tahapan (
             id, rencana_id, urutan, uraian, target_kuantitas, satuan, bobot,
             program, kegiatan, tujuan, indikator_kinerja, target_kualitas,
             tanggal_mulai, tanggal_selesai
         )
         VALUES (
             $1, $2, $3, $4, $5::float8::numeric, $6, $7::float8::numeric,
             $8, $9, $10, $11, $12, $13, $14
         )",
    )
    .bind(Uuid::now_v7())
    .bind(rencana_id)
    .bind(index as i32 + 1)
    .bind(&tahap.uraian)
    .bind(tahap.target_kuantitas)
    .bind(&tahap.satuan)
    .bind(tahap.bobot)
    .bind(&tahap.program)
    .bind(&tahap.kegiatan)
    .bind(&tahap.tujuan)
    .bind(&tahap.indikator_kinerja)
    .bind(&tahap.target_kualitas)
    .bind(tahap.tanggal_mulai)
    .bind(tahap.tanggal_selesai)
    .execute(conn)
    .await?;
    Ok(())
}

pub struct UpdateRencana {
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
    pub tahapan: Option<Vec<NewTahapan>>,
}

pub async fn update(pool: &PgPool, id: Uuid, data: &UpdateRencana) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE rencana_kerja
           SET program = COALESCE($2, program),
               kegiatan = COALESCE($3, kegiatan),
               uraian = COALESCE($4, uraian),
               tujuan = COALESCE($5, tujuan),
               indikator_kinerja = COALESCE($6, indikator_kinerja),
               jenis = COALESCE($7, jenis),
               dasar_jenis = COALESCE($8, dasar_jenis),
               dasar_nomor = COALESCE($9, dasar_nomor),
               dasar_tanggal = COALESCE($10, dasar_tanggal),
               dasar_perihal = COALESCE($11, dasar_perihal),
               target_kuantitas = COALESCE($12::float8::numeric, target_kuantitas),
               satuan = COALESCE($13, satuan),
               target_kualitas = COALESCE($14, target_kualitas),
               target_waktu = COALESCE($15, target_waktu),
               tanggal_mulai = COALESCE($16, tanggal_mulai),
               tanggal_selesai = COALESCE($17, tanggal_selesai)
         WHERE id = $1
    "#;

    let mut tx = pool.begin().await?;

    sqlx::query(SQL)
        .bind(id)
        .bind(&data.program)
        .bind(&data.kegiatan)
        .bind(&data.uraian)
        .bind(&data.tujuan)
        .bind(&data.indikator_kinerja)
        .bind(data.jenis)
        .bind(data.dasar_jenis)
        .bind(&data.dasar_nomor)
        .bind(data.dasar_tanggal)
        .bind(&data.dasar_perihal)
        .bind(data.target_kuantitas)
        .bind(&data.satuan)
        .bind(&data.target_kualitas)
        .bind(data.target_waktu)
        .bind(data.tanggal_mulai)
        .bind(data.tanggal_selesai)
        .execute(&mut *tx)
        .await?;

    if let Some(tahapan) = &data.tahapan {
        sqlx::query("DELETE FROM rencana_tahapan WHERE rencana_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        for (index, tahap) in tahapan.iter().enumerate() {
            simpan_tahapan(&mut tx, id, index, tahap).await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

pub async fn ajukan(pool: &PgPool, id: Uuid) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE rencana_kerja
           SET status = 'diajukan', diajukan_at = now()
         WHERE id = $1
    "#;
    sqlx::query(SQL).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn putuskan(
    pool: &PgPool,
    id: Uuid,
    status: StatusRencana,
    catatan: Option<&str>,
    tambah_revisi: bool,
) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE rencana_kerja
           SET status = $2,
               catatan_atasan = $3,
               diputuskan_at = now(),
               jumlah_revisi = jumlah_revisi + CASE WHEN $4 THEN 1 ELSE 0 END
         WHERE id = $1
    "#;
    let result = sqlx::query(SQL)
        .bind(id)
        .bind(status)
        .bind(catatan)
        .bind(tambah_revisi)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Rencana kerja tidak ditemukan".into()));
    }
    Ok(())
}
