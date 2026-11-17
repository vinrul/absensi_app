use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::{IndikatorKinerja, KategoriKinerja, KeputusanPenilaian};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct PenilaianRow {
    pub id: Uuid,
    pub personel_id: Uuid,
    pub periode_id: Uuid,
    pub satker_id: Uuid,
    pub skor_absensi: f64,
    pub skor_rencana_kerja: f64,
    pub skor_hasil_kerja: f64,
    pub skor_output: f64,
    pub nilai_akhir: f64,
    pub kategori: KategoriKinerja,
    pub catatan_pimpinan: Option<String>,
    pub keputusan: Option<KeputusanPenilaian>,
    pub computed_at: DateTime<Utc>,
    pub personel_nama: String,
    pub personel_nrp: String,
    pub pangkat: String,
    pub jabatan: String,
    pub unit: Option<String>,
}

const LIST_SQL: &str = r#"
    SELECT n.id, n.personel_id, n.periode_id, n.satker_id,
           n.skor_absensi::float8 AS skor_absensi,
           n.skor_rencana_kerja::float8 AS skor_rencana_kerja,
           n.skor_hasil_kerja::float8 AS skor_hasil_kerja,
           n.skor_output::float8 AS skor_output,
           n.nilai_akhir::float8 AS nilai_akhir,
           n.kategori, n.catatan_pimpinan, n.keputusan, n.computed_at,
           p.nama AS personel_nama, p.nrp AS personel_nrp, p.pangkat, p.jabatan, p.unit
      FROM penilaian n
      JOIN personel p ON p.id = n.personel_id
     WHERE n.periode_id = $1
       AND p.jenis_akun = 'personel'
       AND n.satker_id = ANY($2::uuid[])
       AND ($3::text IS NULL OR p.unit = $3)
       AND ($4::kategori_kinerja IS NULL OR n.kategori = $4)
       AND ($5::uuid IS NULL OR n.personel_id = $5)
     ORDER BY n.nilai_akhir DESC, p.nama
     LIMIT $6 OFFSET $7
"#;

const LIST_COUNT: &str = r#"
    SELECT count(*)
      FROM penilaian n
      JOIN personel p ON p.id = n.personel_id
     WHERE n.periode_id = $1
       AND p.jenis_akun = 'personel'
       AND n.satker_id = ANY($2::uuid[])
       AND ($3::text IS NULL OR p.unit = $3)
       AND ($4::kategori_kinerja IS NULL OR n.kategori = $4)
       AND ($5::uuid IS NULL OR n.personel_id = $5)
"#;

pub struct ListFilter {
    pub periode_id: Uuid,
    pub satker_ids: Vec<Uuid>,
    pub unit: Option<String>,
    pub kategori: Option<KategoriKinerja>,
    pub personel_id: Option<Uuid>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list(pool: &PgPool, filter: &ListFilter) -> AppResult<(Vec<PenilaianRow>, i64)> {
    let rows = sqlx::query_as::<_, PenilaianRow>(LIST_SQL)
        .bind(filter.periode_id)
        .bind(&filter.satker_ids)
        .bind(filter.unit.as_deref())
        .bind(filter.kategori)
        .bind(filter.personel_id)
        .bind(filter.limit)
        .bind(filter.offset)
        .fetch_all(pool)
        .await?;

    let total: i64 = sqlx::query_scalar::<_, i64>(LIST_COUNT)
        .bind(filter.periode_id)
        .bind(&filter.satker_ids)
        .bind(filter.unit.as_deref())
        .bind(filter.kategori)
        .bind(filter.personel_id)
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}

pub async fn find(
    pool: &PgPool,
    personel_id: Uuid,
    periode_id: Uuid,
) -> AppResult<Option<PenilaianRow>> {
    const SQL: &str = r#"
        SELECT n.id, n.personel_id, n.periode_id, n.satker_id,
               n.skor_absensi::float8 AS skor_absensi,
               n.skor_rencana_kerja::float8 AS skor_rencana_kerja,
               n.skor_hasil_kerja::float8 AS skor_hasil_kerja,
               n.skor_output::float8 AS skor_output,
               n.nilai_akhir::float8 AS nilai_akhir,
               n.kategori, n.catatan_pimpinan, n.keputusan, n.computed_at,
               p.nama AS personel_nama, p.nrp AS personel_nrp, p.pangkat, p.jabatan, p.unit
          FROM penilaian n
          JOIN personel p ON p.id = n.personel_id
         WHERE n.personel_id = $1 AND n.periode_id = $2
           AND p.jenis_akun = 'personel'
    "#;
    Ok(sqlx::query_as::<_, PenilaianRow>(SQL)
        .bind(personel_id)
        .bind(periode_id)
        .fetch_optional(pool)
        .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert(
    pool: &PgPool,
    id: Uuid,
    personel_id: Uuid,
    periode_id: Uuid,
    satker_id: Uuid,
    skor_absensi: f64,
    skor_rencana: f64,
    skor_hasil: f64,
    skor_output: f64,
    nilai_akhir: f64,
    kategori: KategoriKinerja,
) -> AppResult<Uuid> {
    const SQL: &str = r#"
        INSERT INTO penilaian (id, personel_id, periode_id, satker_id, skor_absensi,
                               skor_rencana_kerja, skor_hasil_kerja, skor_output, nilai_akhir,
                               kategori, computed_at)
        VALUES ($1, $2, $3, $4, $5::float8::numeric, $6::float8::numeric, $7::float8::numeric,
                $8::float8::numeric, $9::float8::numeric, $10, now())
        ON CONFLICT (personel_id, periode_id)
        DO UPDATE SET satker_id = EXCLUDED.satker_id,
                      skor_absensi = EXCLUDED.skor_absensi,
                      skor_rencana_kerja = EXCLUDED.skor_rencana_kerja,
                      skor_hasil_kerja = EXCLUDED.skor_hasil_kerja,
                      skor_output = EXCLUDED.skor_output,
                      nilai_akhir = EXCLUDED.nilai_akhir,
                      kategori = EXCLUDED.kategori,
                      computed_at = now()
        RETURNING id
    "#;
    let id: Uuid = sqlx::query_scalar(SQL)
        .bind(id)
        .bind(personel_id)
        .bind(periode_id)
        .bind(satker_id)
        .bind(skor_absensi)
        .bind(skor_rencana)
        .bind(skor_hasil)
        .bind(skor_output)
        .bind(nilai_akhir)
        .bind(kategori)
        .fetch_one(pool)
        .await?;
    Ok(id)
}

pub async fn simpan_detail(
    pool: &PgPool,
    penilaian_id: Uuid,
    indikator: IndikatorKinerja,
    skor: f64,
    komponen: serde_json::Value,
) -> AppResult<()> {
    sqlx::query("DELETE FROM penilaian_detail WHERE penilaian_id = $1 AND indikator = $2")
        .bind(penilaian_id)
        .bind(indikator)
        .execute(pool)
        .await?;

    sqlx::query(
        "INSERT INTO penilaian_detail (id, penilaian_id, indikator, skor, komponen)
         VALUES ($1, $2, $3, $4::float8::numeric, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(penilaian_id)
    .bind(indikator)
    .bind(skor)
    .bind(komponen)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn detail(
    pool: &PgPool,
    penilaian_id: Uuid,
) -> AppResult<Vec<(IndikatorKinerja, f64, serde_json::Value)>> {
    const SQL: &str = r#"
        SELECT indikator, skor::float8 AS skor, komponen
          FROM penilaian_detail
         WHERE penilaian_id = $1
    "#;
    Ok(
        sqlx::query_as::<_, (IndikatorKinerja, f64, serde_json::Value)>(SQL)
            .bind(penilaian_id)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn keputusan(
    pool: &PgPool,
    id: Uuid,
    catatan: Option<&str>,
    keputusan: KeputusanPenilaian,
    oleh: Uuid,
) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE penilaian
           SET catatan_pimpinan = COALESCE($2, catatan_pimpinan),
               keputusan = $3,
               dinilai_oleh = $4
         WHERE id = $1
    "#;
    let result = sqlx::query(SQL)
        .bind(id)
        .bind(catatan)
        .bind(keputusan)
        .bind(oleh)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Penilaian tidak ditemukan".into()));
    }
    Ok(())
}

/// Sebaran kategori kinerja pada satu periode.
pub async fn sebaran(
    pool: &PgPool,
    periode_id: Uuid,
    satker_ids: &[Uuid],
) -> AppResult<Vec<(KategoriKinerja, i64)>> {
    const SQL: &str = r#"
        SELECT kategori, count(*)
          FROM penilaian
         WHERE periode_id = $1 AND satker_id = ANY($2::uuid[])
         GROUP BY kategori
    "#;
    Ok(sqlx::query_as::<_, (KategoriKinerja, i64)>(SQL)
        .bind(periode_id)
        .bind(satker_ids)
        .fetch_all(pool)
        .await?)
}

/// Rata-rata nilai per indikator pada satu periode.
pub async fn rata_indikator(
    pool: &PgPool,
    periode_id: Uuid,
    satker_ids: &[Uuid],
) -> AppResult<Option<(f64, f64, f64, f64, f64)>> {
    const SQL: &str = r#"
        SELECT avg(skor_absensi)::float8,
               avg(skor_rencana_kerja)::float8,
               avg(skor_hasil_kerja)::float8,
               avg(skor_output)::float8,
               avg(nilai_akhir)::float8
          FROM penilaian
         WHERE periode_id = $1 AND satker_id = ANY($2::uuid[])
    "#;
    Ok(sqlx::query_as::<
        _,
        (
            Option<f64>,
            Option<f64>,
            Option<f64>,
            Option<f64>,
            Option<f64>,
        ),
    >(SQL)
    .bind(periode_id)
    .bind(satker_ids)
    .fetch_optional(pool)
    .await?
    .and_then(|row| match (row.0, row.1, row.2, row.3, row.4) {
        (Some(a), Some(b), Some(c), Some(d), Some(e)) => Some((a, b, c, d, e)),
        _ => None,
    }))
}
