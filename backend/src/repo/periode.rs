use chrono::{DateTime, Datelike, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::{IndikatorKinerja, PeriodeStatus, SatkerLevel};
use crate::error::{AppError, AppResult};
use crate::service::waktu;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct PeriodeRow {
    pub id: Uuid,
    pub nama: String,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
    pub status: PeriodeStatus,
    pub dibuka_kembali_at: Option<DateTime<Utc>>,
    pub alasan_buka_kembali: Option<String>,
}

pub async fn list(pool: &PgPool) -> AppResult<Vec<PeriodeRow>> {
    const SQL: &str = r#"
        SELECT id, nama, tanggal_mulai, tanggal_selesai, status, dibuka_kembali_at, alasan_buka_kembali
          FROM periode
         ORDER BY tanggal_mulai DESC
    "#;
    Ok(sqlx::query_as::<_, PeriodeRow>(SQL).fetch_all(pool).await?)
}

pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Option<PeriodeRow>> {
    const SQL: &str = r#"
        SELECT id, nama, tanggal_mulai, tanggal_selesai, status, dibuka_kembali_at, alasan_buka_kembali
          FROM periode
         WHERE id = $1
    "#;
    Ok(sqlx::query_as::<_, PeriodeRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn find_or_current(pool: &PgPool, id: Option<Uuid>) -> AppResult<PeriodeRow> {
    match id {
        Some(id) => find(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Periode tidak ditemukan".into())),
        None => {
            let hari = waktu::hari_ini_wita();
            if let Some(row) = current(pool, hari).await? {
                return Ok(row);
            }
            pastikan_bulan_ini(pool, hari).await
        }
    }
}

/// Periode yang rentang tanggalnya memuat `tanggal`.
pub async fn current(pool: &PgPool, tanggal: NaiveDate) -> AppResult<Option<PeriodeRow>> {
    const SQL: &str = r#"
        SELECT id, nama, tanggal_mulai, tanggal_selesai, status, dibuka_kembali_at, alasan_buka_kembali
          FROM periode
         WHERE $1 BETWEEN tanggal_mulai AND tanggal_selesai
         ORDER BY tanggal_mulai DESC
         LIMIT 1
    "#;
    Ok(sqlx::query_as::<_, PeriodeRow>(SQL)
        .bind(tanggal)
        .fetch_optional(pool)
        .await?)
}

async fn find_by_start(pool: &PgPool, tanggal_mulai: NaiveDate) -> AppResult<Option<PeriodeRow>> {
    const SQL: &str = r#"
        SELECT id, nama, tanggal_mulai, tanggal_selesai, status, dibuka_kembali_at, alasan_buka_kembali
          FROM periode
         WHERE tanggal_mulai = $1
    "#;
    Ok(sqlx::query_as::<_, PeriodeRow>(SQL)
        .bind(tanggal_mulai)
        .fetch_optional(pool)
        .await?)
}

/// Membuat periode bulan berjalan bila belum ada.
///
/// Pembuatan otomatis hanya untuk periode **bulanan berjalan**; penutupan
/// tetap dilakukan admin setelah data bulan tersebut lengkap.
pub async fn pastikan_bulan_ini(pool: &PgPool, hari: NaiveDate) -> AppResult<PeriodeRow> {
    if let Some(row) = current(pool, hari).await? {
        return Ok(row);
    }

    let (awal, akhir) = waktu::rentang_bulan(hari);
    let nama = format!("{} {}", waktu::nama_bulan(hari.month()), hari.year());
    let id = Uuid::now_v7();

    let mut tx = pool.begin().await?;
    let inserted = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO periode (id, nama, tipe, tanggal_mulai, tanggal_selesai, status)
         VALUES ($1, $2, 'bulanan', $3, $4, 'open')
         ON CONFLICT (tanggal_mulai) DO NOTHING
         RETURNING id",
    )
    .bind(id)
    .bind(&nama)
    .bind(awal)
    .bind(akhir)
    .fetch_optional(&mut *tx)
    .await?;

    if inserted.is_some() {
        for (indikator, bobot) in [
            (IndikatorKinerja::Absensi, 20.0_f64),
            (IndikatorKinerja::RencanaKerja, 20.0),
            (IndikatorKinerja::HasilKerja, 30.0),
            (IndikatorKinerja::Output, 30.0),
        ] {
            sqlx::query(
                "INSERT INTO bobot_indikator (id, periode_id, indikator, bobot)
                 VALUES ($1, $2, $3, $4::float8::numeric)",
            )
            .bind(Uuid::now_v7())
            .bind(id)
            .bind(indikator)
            .bind(bobot)
            .execute(&mut *tx)
            .await?;
        }
        tracing::info!(periode = %nama, "periode bulan berjalan dibuat otomatis");
    }

    tx.commit().await?;

    if let Some(row) = current(pool, hari).await? {
        return Ok(row);
    }
    if let Some(row) = find_by_start(pool, awal).await? {
        return Ok(row);
    }
    Err(AppError::Internal(
        "gagal menyiapkan periode bulan berjalan".into(),
    ))
}

pub async fn create(
    pool: &PgPool,
    id: Uuid,
    nama: &str,
    tanggal_mulai: NaiveDate,
    tanggal_selesai: NaiveDate,
) -> AppResult<()> {
    const SQL: &str = r#"
        INSERT INTO periode (id, nama, tipe, tanggal_mulai, tanggal_selesai, status)
        VALUES ($1, $2, 'bulanan', $3, $4, 'open')
    "#;
    sqlx::query(SQL)
        .bind(id)
        .bind(nama)
        .bind(tanggal_mulai)
        .bind(tanggal_selesai)
        .execute(pool)
        .await?;

    for (indikator, bobot) in [
        (IndikatorKinerja::Absensi, 20.0_f64),
        (IndikatorKinerja::RencanaKerja, 20.0),
        (IndikatorKinerja::HasilKerja, 30.0),
        (IndikatorKinerja::Output, 30.0),
    ] {
        set_bobot(pool, id, indikator, bobot).await?;
    }
    Ok(())
}

pub async fn set_bobot(
    pool: &PgPool,
    periode_id: Uuid,
    indikator: IndikatorKinerja,
    bobot: f64,
) -> AppResult<()> {
    const SQL: &str = r#"
        INSERT INTO bobot_indikator (id, periode_id, indikator, bobot)
        VALUES ($1, $2, $3, $4::float8::numeric)
        ON CONFLICT (periode_id, indikator)
        DO UPDATE SET bobot = EXCLUDED.bobot
    "#;
    sqlx::query(SQL)
        .bind(Uuid::now_v7())
        .bind(periode_id)
        .bind(indikator)
        .bind(bobot)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn bobot(pool: &PgPool, periode_id: Uuid) -> AppResult<Vec<(IndikatorKinerja, f64)>> {
    const SQL: &str = r#"
        SELECT indikator, bobot::float8 AS bobot
          FROM bobot_indikator
         WHERE periode_id = $1
    "#;
    let rows = sqlx::query_as::<_, (IndikatorKinerja, f64)>(SQL)
        .bind(periode_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn tutup(pool: &PgPool, id: Uuid, oleh: Uuid) -> AppResult<()> {
    const SQL: &str = r#"
        UPDATE periode
           SET status = 'closed', ditutup_oleh = $2, ditutup_at = now()
         WHERE id = $1
    "#;
    let result = sqlx::query(SQL).bind(id).bind(oleh).execute(pool).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Periode tidak ditemukan".into()));
    }
    Ok(())
}

/// Membuka kembali periode yang sudah ditutup.
///
/// Hanya admin global yang boleh memanggil ini. Alasan disimpan agar
/// perubahan status dapat diaudit. Status kesiapan satker direset karena
/// datanya mungkin berubah setelah periode dibuka kembali.
pub async fn buka(pool: &PgPool, id: Uuid, oleh: Uuid, alasan: Option<&str>) -> AppResult<()> {
    let periode = find(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Periode tidak ditemukan".into()))?;
    if periode.status == PeriodeStatus::Open {
        return Err(AppError::Unprocessable("Periode sudah terbuka".into()));
    }

    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE periode
            SET status = 'open',
                dibuka_kembali_oleh = $2,
                dibuka_kembali_at = now(),
                alasan_buka_kembali = $3
          WHERE id = $1",
    )
    .bind(id)
    .bind(oleh)
    .bind(alasan)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE periode_satker_kesiapan
            SET siap_ditutup = false,
                ditandai_oleh = NULL,
                ditandai_at = NULL,
                catatan = NULL,
                updated_at = now()
          WHERE periode_id = $1",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct KesiapanRow {
    pub satker_id: Uuid,
    pub satker_nama: String,
    pub satker_level: SatkerLevel,
    pub jumlah_personel: i64,
    pub sudah_dinilai: i64,
    pub belum_dinilai: i64,
    pub siap_ditutup: bool,
    pub ditandai_oleh: Option<Uuid>,
    pub ditandai_oleh_nama: Option<String>,
    pub ditandai_at: Option<DateTime<Utc>>,
    pub catatan: Option<String>,
}

pub async fn kesiapan(
    pool: &PgPool,
    periode_id: Uuid,
    satker_ids: &[Uuid],
) -> AppResult<Vec<KesiapanRow>> {
    const SQL: &str = r#"
        SELECT s.id AS satker_id, s.nama AS satker_nama, s.level AS satker_level,
               COALESCE(p.total, 0) AS jumlah_personel,
               COALESCE(n.dinilai, 0) AS sudah_dinilai,
               GREATEST(COALESCE(p.total, 0) - COALESCE(n.dinilai, 0), 0) AS belum_dinilai,
               COALESCE(k.siap_ditutup, false) AS siap_ditutup,
               k.ditandai_oleh,
               dp.nama AS ditandai_oleh_nama,
               k.ditandai_at,
               k.catatan
          FROM satker s
          LEFT JOIN (
              SELECT satker_id, count(*) AS total
                FROM personel
               WHERE jenis_akun = 'personel'
                 AND deleted_at IS NULL
                 AND is_active
               GROUP BY satker_id
          ) p ON p.satker_id = s.id
          LEFT JOIN (
              SELECT p2.satker_id, count(*) AS dinilai
                FROM penilaian n
                JOIN personel p2 ON p2.id = n.personel_id
               WHERE n.periode_id = $1
                 AND p2.jenis_akun = 'personel'
                 AND p2.is_active
               GROUP BY p2.satker_id
          ) n ON n.satker_id = s.id
          LEFT JOIN periode_satker_kesiapan k
                 ON k.periode_id = $1 AND k.satker_id = s.id
          LEFT JOIN personel dp ON dp.id = k.ditandai_oleh
         WHERE s.deleted_at IS NULL
           AND s.level = 'satker'
           AND s.id = ANY($2::uuid[])
         ORDER BY s.nama
    "#;
    Ok(sqlx::query_as::<_, KesiapanRow>(SQL)
        .bind(periode_id)
        .bind(satker_ids)
        .fetch_all(pool)
        .await?)
}

pub async fn tandai_siap(
    pool: &PgPool,
    periode_id: Uuid,
    satker_id: Uuid,
    siap: bool,
    oleh: Uuid,
    catatan: Option<&str>,
) -> AppResult<()> {
    let oleh = if siap { Some(oleh) } else { None };
    let catatan = if siap { catatan } else { None };
    sqlx::query(
        "INSERT INTO periode_satker_kesiapan
             (periode_id, satker_id, siap_ditutup, ditandai_oleh, ditandai_at, catatan)
         VALUES ($1, $2, $3, $4, CASE WHEN $3 THEN now() ELSE NULL END, $5)
         ON CONFLICT (periode_id, satker_id) DO UPDATE
            SET siap_ditutup = EXCLUDED.siap_ditutup,
                ditandai_oleh = EXCLUDED.ditandai_oleh,
                ditandai_at = EXCLUDED.ditandai_at,
                catatan = EXCLUDED.catatan,
                updated_at = now()",
    )
    .bind(periode_id)
    .bind(satker_id)
    .bind(siap)
    .bind(oleh)
    .bind(catatan)
    .execute(pool)
    .await?;
    Ok(())
}

/// Dipakai untuk memastikan periode tidak diubah setelah ditutup.
pub async fn pastikan_terbuka(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let periode = find(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Periode tidak ditemukan".into()))?;
    if periode.status == PeriodeStatus::Closed {
        return Err(AppError::Unprocessable(
            "Periode sudah ditutup dan tidak dapat diubah".into(),
        ));
    }
    Ok(())
}
