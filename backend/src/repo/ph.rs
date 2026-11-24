//! Repositori PH / Strong Point.
//!
//! Modul ini sengaja terpisah dari tabel `absensi` agar jadwal penjagaan
//! tidak mengubah arti status kehadiran harian.

use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::AbsensiJenis;
use crate::error::{AppError, AppResult};
use crate::service::waktu::{akhir_bulan, awal_bulan};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StrongPointRow {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub nama: String,
    pub alamat: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct JadwalPhRow {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub strong_point_id: Uuid,
    pub strong_point_nama: String,
    pub tanggal: NaiveDate,
    pub jam_mulai: NaiveTime,
    pub jam_selesai: NaiveTime,
    pub sesi: Option<String>,
    pub regu: Option<String>,
    pub catatan: Option<String>,
    pub disalin_dari_bulan: Option<NaiveDate>,
    pub jumlah_personel: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct JadwalPhSayaRow {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub strong_point_id: Uuid,
    pub strong_point_nama: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: i32,
    pub tanggal: NaiveDate,
    pub jam_mulai: NaiveTime,
    pub jam_selesai: NaiveTime,
    pub sesi: Option<String>,
    pub regu: Option<String>,
    pub catatan: Option<String>,
    pub peran: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct JadwalMatriksRow {
    pub personel_id: Uuid,
    pub jadwal_ph_id: Uuid,
    pub tanggal: NaiveDate,
    pub sesi: Option<String>,
    pub jam_mulai: NaiveTime,
    pub jam_selesai: NaiveTime,
    pub strong_point_nama: String,
    pub masuk_jam: Option<NaiveTime>,
    pub pulang_jam: Option<NaiveTime>,
    pub menit_telat: i64,
    pub menit_pulang_cepat: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PersonelJadwalRow {
    pub personel_id: Uuid,
    pub nama: String,
    pub nrp: String,
    pub pangkat: String,
    pub unit: Option<String>,
    pub peran: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AbsensiPhRow {
    pub id: Uuid,
    pub client_uuid: Uuid,
    pub jadwal_ph_id: Uuid,
    pub personel_id: Uuid,
    pub personel_nama: String,
    pub personel_nrp: String,
    pub strong_point_id: Uuid,
    pub strong_point_nama: String,
    pub satker_id: Uuid,
    pub jenis: AbsensiJenis,
    pub captured_at: DateTime<Utc>,
    pub jam: NaiveTime,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub akurasi_m: Option<f64>,
    pub jarak_m: Option<f64>,
    pub dalam_radius: bool,
    pub foto_path: Option<String>,
    pub is_offline: bool,
    pub is_mock_suspect: bool,
    pub perlu_verifikasi: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub catatan_verifikator: Option<String>,
    pub menit_telat: i32,
    pub menit_pulang_cepat: i32,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SesiPhRow {
    pub masuk_terakhir: Option<DateTime<Utc>>,
    pub pulang_terakhir: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RekapPhRow {
    pub personel_id: Uuid,
    pub nama: String,
    pub nrp: String,
    pub pangkat: String,
    pub unit: Option<String>,
    pub jumlah_jadwal: i64,
    pub hadir: i64,
    /// Jadwal yang check-in tetapi tidak check-out.
    pub hanya_check_in: i64,
    pub tidak_hadir: i64,
    /// Tanggal jadwal yang tidak memiliki check-in sama sekali.
    pub tanggal_tidak_hadir: Vec<NaiveDate>,
    /// Tanggal jadwal yang hanya memiliki check-in.
    pub tanggal_hanya_check_in: Vec<NaiveDate>,
    pub menunggu_verifikasi: i64,
    pub total_menit_telat: i64,
    pub total_menit_pulang_cepat: i64,
}

/// Satu baris jadwal × personel untuk rekap strong point per tanggal.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RekapPhHarianRow {
    pub tanggal: NaiveDate,
    pub jadwal_ph_id: Uuid,
    pub sesi: Option<String>,
    pub jam_mulai: NaiveTime,
    pub jam_selesai: NaiveTime,
    pub strong_point_nama: String,
    pub strong_point_id: Uuid,
    pub strong_point_latitude: f64,
    pub strong_point_longitude: f64,
    pub strong_point_radius_m: i32,
    pub regu: Option<String>,
    pub personel_id: Uuid,
    pub nama: String,
    pub nrp: String,
    pub pangkat: String,
    pub unit: Option<String>,
    pub masuk_jam: Option<NaiveTime>,
    pub pulang_jam: Option<NaiveTime>,
    pub masuk_latitude: Option<f64>,
    pub masuk_longitude: Option<f64>,
    pub menit_telat: i64,
    pub menit_pulang_cepat: i64,
}

pub struct NewAbsensiPh {
    pub id: Uuid,
    pub client_uuid: Uuid,
    pub jadwal_ph_id: Uuid,
    pub personel_id: Uuid,
    pub strong_point_id: Uuid,
    pub satker_id: Uuid,
    pub jenis: AbsensiJenis,
    pub captured_at: DateTime<Utc>,
    pub jam: NaiveTime,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub akurasi_m: Option<f64>,
    pub jarak_m: Option<f64>,
    pub dalam_radius: bool,
    pub foto_path: Option<String>,
    pub is_offline: bool,
    pub is_mock_suspect: bool,
    pub perlu_verifikasi: bool,
    pub menit_telat: i32,
    pub menit_pulang_cepat: i32,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PenempatanPhRow {
    pub personel_id: Uuid,
    pub nama: String,
    pub nrp: String,
    pub pangkat: String,
    pub unit: Option<String>,
    pub satker_id: Uuid,
    pub strong_point_id: Option<Uuid>,
    pub strong_point_nama: Option<String>,
}

pub struct PenempatanBaru {
    pub personel_id: Uuid,
    pub strong_point_id: Uuid,
}

// ---------------------------------------------------------------- strong point

pub async fn list_strong_points(
    pool: &PgPool,
    satker_ids: &[Uuid],
) -> AppResult<Vec<StrongPointRow>> {
    const SQL: &str = r#"
        SELECT sp.id, sp.satker_id, sp.nama, sp.alamat,
               sp.latitude::float8 AS latitude,
               sp.longitude::float8 AS longitude, sp.radius_m, sp.is_active
          FROM strong_point sp
          JOIN satker s ON s.id = sp.satker_id
         WHERE sp.satker_id = ANY($1::uuid[]) AND sp.deleted_at IS NULL
           AND s.level IN ('polda', 'polres')
         ORDER BY sp.nama
    "#;
    Ok(sqlx::query_as::<_, StrongPointRow>(SQL)
        .bind(satker_ids)
        .fetch_all(pool)
        .await?)
}

pub async fn find_strong_point(pool: &PgPool, id: Uuid) -> AppResult<Option<StrongPointRow>> {
    const SQL: &str = r#"
        SELECT id, satker_id, nama, alamat, latitude::float8 AS latitude,
               longitude::float8 AS longitude, radius_m, is_active
          FROM strong_point
         WHERE id = $1 AND deleted_at IS NULL
    "#;
    Ok(sqlx::query_as::<_, StrongPointRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

/// Unit induk (Polda/Polres) yang menjadi pemilik titik strong point bagi
/// satker-satker yang diberikan. Satker di bawah Polres memakai titik Polres,
/// bukan titik Polda.
pub async fn induk_titik_ids(pool: &PgPool, satker_ids: &[Uuid]) -> AppResult<Vec<Uuid>> {
    const SQL: &str = r#"
        WITH RECURSIVE naik AS (
            SELECT s.id AS asal_id, s.id, s.parent_id, s.level, 1 AS kedalaman
              FROM satker s
             WHERE s.id = ANY($1::uuid[])
            UNION ALL
            SELECT n.asal_id, s.id, s.parent_id, s.level, n.kedalaman + 1
              FROM satker s
              JOIN naik n ON s.id = n.parent_id
        )
        SELECT DISTINCT ON (asal_id) id AS induk_id
          FROM naik
         WHERE level IN ('polda', 'polres')
         ORDER BY asal_id, kedalaman
    "#;
    Ok(sqlx::query_scalar::<_, Uuid>(SQL)
        .bind(satker_ids)
        .fetch_all(pool)
        .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_strong_point(
    pool: &PgPool,
    id: Uuid,
    satker_id: Uuid,
    nama: &str,
    alamat: Option<&str>,
    latitude: f64,
    longitude: f64,
    radius_m: i32,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO strong_point (id, satker_id, nama, alamat, latitude, longitude, radius_m)
         VALUES ($1, $2, $3, $4, $5::float8::numeric, $6::float8::numeric, $7)",
    )
    .bind(id)
    .bind(satker_id)
    .bind(nama)
    .bind(alamat)
    .bind(latitude)
    .bind(longitude)
    .bind(radius_m)
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_strong_point(
    pool: &PgPool,
    id: Uuid,
    nama: Option<&str>,
    alamat: Option<&str>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    radius_m: Option<i32>,
    is_active: Option<bool>,
) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE strong_point
            SET nama = COALESCE($2, nama),
                alamat = COALESCE($3, alamat),
                latitude = COALESCE($4::float8::numeric, latitude),
                longitude = COALESCE($5::float8::numeric, longitude),
                radius_m = COALESCE($6, radius_m),
                is_active = COALESCE($7, is_active)
          WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(nama)
    .bind(alamat)
    .bind(latitude)
    .bind(longitude)
    .bind(radius_m)
    .bind(is_active)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Strong point tidak ditemukan".into()));
    }
    Ok(())
}

pub async fn delete_strong_point(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE strong_point SET deleted_at = now()
          WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Strong point tidak ditemukan".into()));
    }
    Ok(())
}

// ---------------------------------------------------------------- penempatan

pub async fn list_penempatan_ph(
    pool: &PgPool,
    satker_ids: &[Uuid],
) -> AppResult<Vec<PenempatanPhRow>> {
    const SQL: &str = r#"
        SELECT p.id AS personel_id, p.nama, p.nrp, p.pangkat, p.unit, p.satker_id,
               pp.strong_point_id, sp.nama AS strong_point_nama
          FROM personel p
          LEFT JOIN penempatan_ph pp ON pp.personel_id = p.id
          LEFT JOIN strong_point sp ON sp.id = pp.strong_point_id
         WHERE p.satker_id = ANY($1::uuid[])
           AND p.deleted_at IS NULL AND p.is_active
           AND p.jenis_akun = 'personel'
         ORDER BY p.nama
    "#;
    Ok(sqlx::query_as::<_, PenempatanPhRow>(SQL)
        .bind(satker_ids)
        .fetch_all(pool)
        .await?)
}

/// Satu anggota hanya memiliki satu titik strong point. Penyimpanan bersifat
/// upsert sehingga perubahan titik tidak meninggalkan baris lama.
pub async fn simpan_penempatan(
    pool: &PgPool,
    data: &PenempatanBaru,
    created_by: Uuid,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO penempatan_ph (id, personel_id, strong_point_id, created_by)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (personel_id) DO UPDATE
             SET strong_point_id = EXCLUDED.strong_point_id,
                 created_by = EXCLUDED.created_by",
    )
    .bind(Uuid::now_v7())
    .bind(data.personel_id)
    .bind(data.strong_point_id)
    .bind(created_by)
    .execute(pool)
    .await?;
    Ok(())
}

/// Titik tetap untuk daftar anggota. Anggota tanpa penempatan aktif tidak dikembalikan.
pub async fn titik_personel(pool: &PgPool, personel_ids: &[Uuid]) -> AppResult<Vec<(Uuid, Uuid)>> {
    const SQL: &str = r#"
        SELECT pp.personel_id, pp.strong_point_id
          FROM penempatan_ph pp
          JOIN strong_point sp ON sp.id = pp.strong_point_id
         WHERE pp.personel_id = ANY($1::uuid[])
           AND sp.deleted_at IS NULL AND sp.is_active
    "#;
    Ok(sqlx::query_as::<_, (Uuid, Uuid)>(SQL)
        .bind(personel_ids)
        .fetch_all(pool)
        .await?)
}

// ------------------------------------------------------- jadwal bulanan & salinan

#[derive(Debug, Clone, sqlx::FromRow)]
struct JadwalSumberRow {
    id: Uuid,
    strong_point_id: Uuid,
    tanggal: NaiveDate,
    jam_mulai: NaiveTime,
    jam_selesai: NaiveTime,
    sesi: Option<String>,
    regu: Option<String>,
    catatan: Option<String>,
    disalin_dari_bulan: Option<NaiveDate>,
    created_by: Option<Uuid>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct PersonelSumberRow {
    personel_id: Uuid,
    peran: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StatusJadwalRow {
    pub satker_id: Uuid,
    pub jumlah: i64,
    pub jumlah_depan: i64,
    pub punya_personel: bool,
    pub disalin_dari_bulan: Option<NaiveDate>,
}

fn proyeksi_tanggal(sumber: NaiveDate, target_awal: NaiveDate) -> Option<NaiveDate> {
    let selisih = (sumber.weekday().num_days_from_monday() + 7
        - target_awal.weekday().num_days_from_monday())
        % 7;
    let urutan = (sumber.day() - 1) / 7;
    let hari = 1 + selisih + urutan * 7;
    NaiveDate::from_ymd_opt(target_awal.year(), target_awal.month(), hari)
}

async fn ada_jadwal_bulan(pool: &PgPool, satker_id: Uuid, awal: NaiveDate) -> AppResult<bool> {
    const SQL: &str = r#"
        SELECT EXISTS (
            SELECT 1 FROM jadwal_ph
             WHERE satker_id = $1 AND tanggal BETWEEN $2 AND $3
               AND NOT is_arsip
        )
    "#;
    Ok(sqlx::query_scalar::<_, bool>(SQL)
        .bind(satker_id)
        .bind(awal)
        .bind(akhir_bulan(awal))
        .fetch_one(pool)
        .await?)
}

/// Bulan terakhir yang memiliki jadwal sebelum `sebelum` (eksklusif).
pub async fn bulan_jadwal_terakhir(
    pool: &PgPool,
    satker_id: Uuid,
    sebelum: NaiveDate,
) -> AppResult<Option<NaiveDate>> {
    const SQL: &str = r#"
        SELECT date_trunc('month', max(tanggal))::date
          FROM jadwal_ph
         WHERE satker_id = $1 AND tanggal < $2 AND NOT is_arsip
    "#;
    Ok(sqlx::query_scalar::<_, Option<NaiveDate>>(SQL)
        .bind(satker_id)
        .bind(sebelum)
        .fetch_one(pool)
        .await?)
}

/// Menyalin jadwal satu bulan ke bulan lain dengan memetakan tanggal berdasarkan
/// hari dan urutan minggu (Senin ke-1, Selasa ke-2, dst.). Dengan begitu jadwal
/// bulan sebelumnya tetap jatuh pada hari yang sama di bulan tujuan.
pub async fn clone_jadwal_bulan(
    pool: &PgPool,
    satker_id: Uuid,
    sumber_bulan: NaiveDate,
    target_bulan: NaiveDate,
    tandai_salinan: bool,
) -> AppResult<u64> {
    let sumber_awal = awal_bulan(sumber_bulan);
    let sumber_akhir = akhir_bulan(sumber_awal);
    let target_awal = awal_bulan(target_bulan);
    let target_akhir = akhir_bulan(target_awal);
    let kunci = format!("ph-jadwal-{satker_id}-{target_awal}");

    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1)::bigint)")
        .bind(&kunci)
        .execute(&mut *tx)
        .await?;

    let sudah: bool = sqlx::query_scalar(
        "SELECT EXISTS (
             SELECT 1 FROM jadwal_ph
              WHERE satker_id = $1 AND tanggal BETWEEN $2 AND $3
                AND NOT is_arsip
         )",
    )
    .bind(satker_id)
    .bind(target_awal)
    .bind(target_akhir)
    .fetch_one(&mut *tx)
    .await?;
    if sudah {
        return Err(AppError::Conflict(
            "Bulan tujuan sudah memiliki jadwal".into(),
        ));
    }

    const SUMBER: &str = r#"
        SELECT id, strong_point_id, tanggal, jam_mulai, jam_selesai, sesi, regu,
               catatan, disalin_dari_bulan, created_by
          FROM jadwal_ph
         WHERE satker_id = $1 AND tanggal BETWEEN $2 AND $3 AND NOT is_arsip
         ORDER BY tanggal, jam_mulai
    "#;
    let rows = sqlx::query_as::<_, JadwalSumberRow>(SUMBER)
        .bind(satker_id)
        .bind(sumber_awal)
        .bind(sumber_akhir)
        .fetch_all(&mut *tx)
        .await?;

    let mut dibuat = 0u64;
    for row in rows {
        let Some(tanggal) = proyeksi_tanggal(row.tanggal, target_awal) else {
            continue;
        };
        let id = Uuid::now_v7();
        let sesi = row.sesi.as_deref().unwrap_or("pagi");
        let asal = if tandai_salinan {
            Some(row.disalin_dari_bulan.unwrap_or(sumber_awal))
        } else {
            None
        };
        sqlx::query(
            "INSERT INTO jadwal_ph (id, satker_id, strong_point_id, tanggal, jam_mulai,
                                    jam_selesai, sesi, regu, catatan, created_by,
                                    disalin_dari_bulan)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(id)
        .bind(satker_id)
        .bind(row.strong_point_id)
        .bind(tanggal)
        .bind(row.jam_mulai)
        .bind(row.jam_selesai)
        .bind(sesi)
        .bind(row.regu.as_deref())
        .bind(row.catatan.as_deref())
        .bind(row.created_by)
        .bind(asal)
        .execute(&mut *tx)
        .await?;

        let personel = sqlx::query_as::<_, PersonelSumberRow>(
            "SELECT personel_id, peran FROM jadwal_ph_personel WHERE jadwal_ph_id = $1",
        )
        .bind(row.id)
        .fetch_all(&mut *tx)
        .await?;
        for orang in personel {
            sqlx::query(
                "INSERT INTO jadwal_ph_personel (jadwal_ph_id, personel_id, peran)
                 VALUES ($1, $2, $3)",
            )
            .bind(id)
            .bind(orang.personel_id)
            .bind(orang.peran.as_deref())
            .execute(&mut *tx)
            .await?;
        }
        dibuat += 1;
    }

    tx.commit().await?;
    Ok(dibuat)
}

/// Bila bulan yang diminta belum memiliki jadwal dan bulannya sudah berjalan,
/// pakai jadwal bulan terakhir yang tersedia. `batas` adalah awal bulan berjalan.
pub async fn pastikan_jadwal_bulan(
    pool: &PgPool,
    satker_id: Uuid,
    bulan: NaiveDate,
    batas: NaiveDate,
) -> AppResult<u64> {
    let awal = awal_bulan(bulan);
    if awal > awal_bulan(batas) || ada_jadwal_bulan(pool, satker_id, awal).await? {
        return Ok(0);
    }
    let Some(sumber) = bulan_jadwal_terakhir(pool, satker_id, awal).await? else {
        return Ok(0);
    };
    match clone_jadwal_bulan(pool, satker_id, sumber, awal, true).await {
        Ok(jumlah) => Ok(jumlah),
        Err(AppError::Conflict(_)) => Ok(0),
        Err(err) => Err(err),
    }
}

pub async fn pastikan_jadwal_scope(
    pool: &PgPool,
    satker_ids: &[Uuid],
    bulan: NaiveDate,
    batas: NaiveDate,
) -> AppResult<()> {
    for satker_id in satker_ids {
        pastikan_jadwal_bulan(pool, *satker_id, bulan, batas).await?;
    }
    Ok(())
}

/// Statistik jadwal mentah (tanpa memicu fallback) untuk pengingat admin.
pub async fn status_jadwal_bulan(
    pool: &PgPool,
    satker_ids: &[Uuid],
    bulan: NaiveDate,
) -> AppResult<Vec<StatusJadwalRow>> {
    let awal = awal_bulan(bulan);
    let akhir = akhir_bulan(awal);
    let depan_awal = crate::service::waktu::geser_bulan(awal, 1);
    let depan_akhir = akhir_bulan(depan_awal);
    const SQL: &str = r#"
        SELECT s.id AS satker_id,
               (SELECT count(*) FROM jadwal_ph j
                 WHERE j.satker_id = s.id AND j.tanggal BETWEEN $2 AND $3
                   AND NOT j.is_arsip)::bigint AS jumlah,
               (SELECT count(*) FROM jadwal_ph j
                 WHERE j.satker_id = s.id AND j.tanggal BETWEEN $4 AND $5
                   AND NOT j.is_arsip)::bigint AS jumlah_depan,
               EXISTS (
                   SELECT 1 FROM personel p
                    WHERE p.satker_id = s.id AND p.deleted_at IS NULL
                      AND p.is_active AND p.jenis_akun = 'personel'
               ) AS punya_personel,
               (SELECT min(j.disalin_dari_bulan) FROM jadwal_ph j
                 WHERE j.satker_id = s.id AND j.tanggal BETWEEN $2 AND $3
                   AND NOT j.is_arsip) AS disalin_dari_bulan
          FROM satker s
         WHERE s.id = ANY($1::uuid[])
         ORDER BY s.nama
    "#;
    Ok(sqlx::query_as::<_, StatusJadwalRow>(SQL)
        .bind(satker_ids)
        .bind(awal)
        .bind(akhir)
        .bind(depan_awal)
        .bind(depan_akhir)
        .fetch_all(pool)
        .await?)
}

// ------------------------------------------------------------------ jadwal PH

pub async fn list_jadwal(
    pool: &PgPool,
    satker_ids: &[Uuid],
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<JadwalPhRow>> {
    const SQL: &str = r#"
        SELECT j.id, j.satker_id, j.strong_point_id, sp.nama AS strong_point_nama,
               j.tanggal, j.jam_mulai, j.jam_selesai, j.sesi, j.regu, j.catatan,
               j.disalin_dari_bulan,
               (SELECT count(*) FROM jadwal_ph_personel x WHERE x.jadwal_ph_id = j.id)
                   AS jumlah_personel
          FROM jadwal_ph j
          JOIN strong_point sp ON sp.id = j.strong_point_id
         WHERE j.satker_id = ANY($1::uuid[]) AND j.tanggal BETWEEN $2 AND $3
           AND NOT j.is_arsip
         ORDER BY j.tanggal, j.sesi, j.jam_mulai
    "#;
    Ok(sqlx::query_as::<_, JadwalPhRow>(SQL)
        .bind(satker_ids)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

pub async fn find_jadwal(pool: &PgPool, id: Uuid) -> AppResult<Option<JadwalPhRow>> {
    const SQL: &str = r#"
        SELECT j.id, j.satker_id, j.strong_point_id, sp.nama AS strong_point_nama,
               j.tanggal, j.jam_mulai, j.jam_selesai, j.sesi, j.regu, j.catatan,
               j.disalin_dari_bulan,
               (SELECT count(*) FROM jadwal_ph_personel x WHERE x.jadwal_ph_id = j.id)
                   AS jumlah_personel
          FROM jadwal_ph j
          JOIN strong_point sp ON sp.id = j.strong_point_id
         WHERE j.id = $1 AND NOT j.is_arsip
    "#;
    Ok(sqlx::query_as::<_, JadwalPhRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn jadwal_saya(
    pool: &PgPool,
    personel_id: Uuid,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<JadwalPhSayaRow>> {
    const SQL: &str = r#"
        SELECT j.id, j.satker_id, j.strong_point_id, sp.nama AS strong_point_nama,
               sp.latitude::float8 AS latitude, sp.longitude::float8 AS longitude,
               sp.radius_m, j.tanggal, j.jam_mulai, j.jam_selesai, j.sesi,
               j.regu, j.catatan, jp.peran
          FROM jadwal_ph j
          JOIN jadwal_ph_personel jp ON jp.jadwal_ph_id = j.id
          JOIN strong_point sp ON sp.id = j.strong_point_id
         WHERE jp.personel_id = $1 AND j.tanggal BETWEEN $2 AND $3
           AND NOT j.is_arsip
         ORDER BY j.tanggal, j.jam_mulai
    "#;
    Ok(sqlx::query_as::<_, JadwalPhSayaRow>(SQL)
        .bind(personel_id)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

pub async fn jadwal_matriks(
    pool: &PgPool,
    satker_ids: &[Uuid],
    dari: NaiveDate,
    sampai: NaiveDate,
    personel_id: Option<Uuid>,
) -> AppResult<Vec<JadwalMatriksRow>> {
    const SQL: &str = r#"
        SELECT jp.personel_id,
               j.id AS jadwal_ph_id,
               j.tanggal,
               j.sesi,
               j.jam_mulai,
               j.jam_selesai,
               sp.nama AS strong_point_nama,
               max(ap.jam) FILTER (WHERE ap.jenis = 'masuk') AS masuk_jam,
               max(ap.jam) FILTER (WHERE ap.jenis = 'pulang') AS pulang_jam,
               coalesce(
                   max(ap.menit_telat) FILTER (WHERE ap.jenis = 'masuk'),
                   0
               )::bigint AS menit_telat,
               coalesce(
                   max(ap.menit_pulang_cepat) FILTER (WHERE ap.jenis = 'pulang'),
                   0
               )::bigint AS menit_pulang_cepat
          FROM jadwal_ph j
          JOIN jadwal_ph_personel jp ON jp.jadwal_ph_id = j.id
          JOIN strong_point sp ON sp.id = j.strong_point_id
          LEFT JOIN absensi_ph ap
                 ON ap.jadwal_ph_id = j.id AND ap.personel_id = jp.personel_id
         WHERE j.tanggal BETWEEN $1 AND $2
           AND NOT j.is_arsip
           AND j.satker_id = ANY($3::uuid[])
           AND ($4::uuid IS NULL OR jp.personel_id = $4)
         GROUP BY jp.personel_id, j.id, j.tanggal, j.sesi, j.jam_mulai, j.jam_selesai, sp.nama
         ORDER BY j.tanggal, j.jam_mulai
    "#;
    Ok(sqlx::query_as::<_, JadwalMatriksRow>(SQL)
        .bind(dari)
        .bind(sampai)
        .bind(satker_ids)
        .bind(personel_id)
        .fetch_all(pool)
        .await?)
}

pub async fn personel_jadwal(pool: &PgPool, jadwal_id: Uuid) -> AppResult<Vec<PersonelJadwalRow>> {
    const SQL: &str = r#"
        SELECT jp.personel_id, p.nama, p.nrp, p.pangkat, p.unit, jp.peran
          FROM jadwal_ph_personel jp
          JOIN personel p ON p.id = jp.personel_id
         WHERE jp.jadwal_ph_id = $1
         ORDER BY p.nama
    "#;
    Ok(sqlx::query_as::<_, PersonelJadwalRow>(SQL)
        .bind(jadwal_id)
        .fetch_all(pool)
        .await?)
}

pub struct JadwalBaru<'a> {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub strong_point_id: Uuid,
    pub tanggal: NaiveDate,
    pub jam_mulai: NaiveTime,
    pub jam_selesai: NaiveTime,
    pub sesi: &'a str,
    pub regu: Option<&'a str>,
    pub catatan: Option<&'a str>,
    pub disalin_dari_bulan: Option<NaiveDate>,
    pub created_by: Uuid,
    pub personel: &'a [(Uuid, Option<String>)],
}

pub async fn create_jadwal(pool: &PgPool, data: &JadwalBaru<'_>) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO jadwal_ph (id, satker_id, strong_point_id, tanggal, jam_mulai,
                                jam_selesai, sesi, regu, catatan, created_by,
                                disalin_dari_bulan)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(data.id)
    .bind(data.satker_id)
    .bind(data.strong_point_id)
    .bind(data.tanggal)
    .bind(data.jam_mulai)
    .bind(data.jam_selesai)
    .bind(data.sesi)
    .bind(data.regu)
    .bind(data.catatan)
    .bind(data.created_by)
    .bind(data.disalin_dari_bulan)
    .execute(&mut *tx)
    .await?;

    for (personel_id, peran) in data.personel {
        sqlx::query(
            "INSERT INTO jadwal_ph_personel (jadwal_ph_id, personel_id, peran)
             VALUES ($1, $2, $3)",
        )
        .bind(data.id)
        .bind(personel_id)
        .bind(peran)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Menghapus jadwal pada rentang tanggal yang belum memiliki absensi.
/// saat admin meminta generate ulang.
pub async fn hapus_jadwal_tanpa_absensi(
    pool: &PgPool,
    satker_id: Uuid,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<u64> {
    const SQL: &str = r#"
        DELETE FROM jadwal_ph j
         WHERE j.satker_id = $1 AND j.tanggal BETWEEN $2 AND $3
           AND NOT j.is_arsip
           AND NOT EXISTS (
               SELECT 1 FROM absensi_ph ap WHERE ap.jadwal_ph_id = j.id
           )
    "#;
    let result = sqlx::query(SQL)
        .bind(satker_id)
        .bind(dari)
        .bind(sampai)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_jadwal(
    pool: &PgPool,
    id: Uuid,
    strong_point_id: Uuid,
    tanggal: NaiveDate,
    jam_mulai: NaiveTime,
    jam_selesai: NaiveTime,
    sesi: &str,
    regu: Option<&str>,
    catatan: Option<&str>,
    personel: &[(Uuid, Option<String>)],
) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    let result = sqlx::query(
        "UPDATE jadwal_ph
            SET strong_point_id = $2, tanggal = $3, jam_mulai = $4,
                jam_selesai = $5, sesi = $6, regu = $7, catatan = $8
          WHERE id = $1",
    )
    .bind(id)
    .bind(strong_point_id)
    .bind(tanggal)
    .bind(jam_mulai)
    .bind(jam_selesai)
    .bind(sesi)
    .bind(regu)
    .bind(catatan)
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Jadwal PH tidak ditemukan".into()));
    }

    sqlx::query("DELETE FROM jadwal_ph_personel WHERE jadwal_ph_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    for (personel_id, peran) in personel {
        sqlx::query(
            "INSERT INTO jadwal_ph_personel (jadwal_ph_id, personel_id, peran)
             VALUES ($1, $2, $3)",
        )
        .bind(id)
        .bind(personel_id)
        .bind(peran.as_deref())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn delete_jadwal(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM jadwal_ph WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Jadwal PH tidak ditemukan".into()));
    }
    Ok(())
}

pub async fn tanggal_personel(
    pool: &PgPool,
    personel_id: Uuid,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<NaiveDate>> {
    const SQL: &str = r#"
        SELECT DISTINCT j.tanggal
          FROM jadwal_ph j
          JOIN jadwal_ph_personel jp ON jp.jadwal_ph_id = j.id
         WHERE jp.personel_id = $1 AND j.tanggal BETWEEN $2 AND $3
    "#;
    Ok(sqlx::query_scalar::<_, NaiveDate>(SQL)
        .bind(personel_id)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

/// Tanggal dengan check-in strong point; dipakai untuk menetralkan telat absensi kantor.
pub async fn tanggal_personel_hadir(
    pool: &PgPool,
    personel_id: Uuid,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<NaiveDate>> {
    const SQL: &str = r#"
        SELECT DISTINCT j.tanggal
          FROM absensi_ph ap
          JOIN jadwal_ph j ON j.id = ap.jadwal_ph_id
         WHERE ap.personel_id = $1
           AND ap.jenis = 'masuk'
           AND j.tanggal BETWEEN $2 AND $3
         ORDER BY j.tanggal
    "#;
    Ok(sqlx::query_scalar::<_, NaiveDate>(SQL)
        .bind(personel_id)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

pub async fn personel_ditugaskan(
    pool: &PgPool,
    jadwal_id: Uuid,
    personel_id: Uuid,
) -> AppResult<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
             SELECT 1 FROM jadwal_ph_personel
              WHERE jadwal_ph_id = $1 AND personel_id = $2
         )",
    )
    .bind(jadwal_id)
    .bind(personel_id)
    .fetch_one(pool)
    .await?)
}

/// Mendeteksi personel yang sudah punya jadwal PH lain pada tanggal dan jam
/// yang beririsan. `kecuali` dipakai saat memperbarui jadwal yang sama.
pub async fn ada_bentrok(
    pool: &PgPool,
    personel_ids: &[Uuid],
    tanggal: NaiveDate,
    jam_mulai: NaiveTime,
    jam_selesai: NaiveTime,
    kecuali: Option<Uuid>,
) -> AppResult<bool> {
    const SQL: &str = r#"
        SELECT EXISTS (
            SELECT 1
              FROM jadwal_ph j
              JOIN jadwal_ph_personel jp ON jp.jadwal_ph_id = j.id
             WHERE j.tanggal = $1
               AND ($2::uuid IS NULL OR j.id <> $2)
               AND NOT j.is_arsip
               AND jp.personel_id = ANY($3::uuid[])
               AND j.jam_mulai < $5
               AND j.jam_selesai > $4
        )
    "#;
    Ok(sqlx::query_scalar::<_, bool>(SQL)
        .bind(tanggal)
        .bind(kecuali)
        .bind(personel_ids)
        .bind(jam_mulai)
        .bind(jam_selesai)
        .fetch_one(pool)
        .await?)
}

// ---------------------------------------------------------------- absensi PH

pub async fn find_absensi_ph(pool: &PgPool, id: Uuid) -> AppResult<Option<AbsensiPhRow>> {
    const SQL: &str = r#"
        SELECT ap.id, ap.client_uuid, ap.jadwal_ph_id, ap.personel_id,
               p.nama AS personel_nama, p.nrp AS personel_nrp,
               ap.strong_point_id, sp.nama AS strong_point_nama, ap.satker_id,
               ap.jenis, ap.captured_at, ap.jam,
               ap.latitude::float8 AS latitude, ap.longitude::float8 AS longitude,
               ap.akurasi_m::float8 AS akurasi_m, ap.jarak_m::float8 AS jarak_m,
               ap.dalam_radius, ap.foto_path, ap.is_offline, ap.is_mock_suspect,
               ap.perlu_verifikasi, ap.verified_at, ap.catatan_verifikator,
               ap.menit_telat, ap.menit_pulang_cepat
          FROM absensi_ph ap
          JOIN personel p ON p.id = ap.personel_id
          JOIN strong_point sp ON sp.id = ap.strong_point_id
         WHERE ap.id = $1
    "#;
    Ok(sqlx::query_as::<_, AbsensiPhRow>(SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn find_absensi_ph_by_client(
    pool: &PgPool,
    client_uuid: Uuid,
) -> AppResult<Option<AbsensiPhRow>> {
    const SQL: &str = r#"
        SELECT ap.id, ap.client_uuid, ap.jadwal_ph_id, ap.personel_id,
               p.nama AS personel_nama, p.nrp AS personel_nrp,
               ap.strong_point_id, sp.nama AS strong_point_nama, ap.satker_id,
               ap.jenis, ap.captured_at, ap.jam,
               ap.latitude::float8 AS latitude, ap.longitude::float8 AS longitude,
               ap.akurasi_m::float8 AS akurasi_m, ap.jarak_m::float8 AS jarak_m,
               ap.dalam_radius, ap.foto_path, ap.is_offline, ap.is_mock_suspect,
               ap.perlu_verifikasi, ap.verified_at, ap.catatan_verifikator,
               ap.menit_telat, ap.menit_pulang_cepat
          FROM absensi_ph ap
          JOIN personel p ON p.id = ap.personel_id
          JOIN strong_point sp ON sp.id = ap.strong_point_id
         WHERE ap.client_uuid = $1
    "#;
    Ok(sqlx::query_as::<_, AbsensiPhRow>(SQL)
        .bind(client_uuid)
        .fetch_optional(pool)
        .await?)
}

pub async fn exists_absensi_ph(
    pool: &PgPool,
    jadwal_id: Uuid,
    personel_id: Uuid,
    jenis: AbsensiJenis,
) -> AppResult<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
             SELECT 1 FROM absensi_ph
              WHERE jadwal_ph_id = $1 AND personel_id = $2 AND jenis = $3
         )",
    )
    .bind(jadwal_id)
    .bind(personel_id)
    .bind(jenis)
    .fetch_one(pool)
    .await?)
}

/// Waktu check-in dan check-out terakhir untuk menentukan sesi yang masih terbuka.
pub async fn ada_absensi_ph_hari_ini(
    pool: &PgPool,
    personel_id: Uuid,
    tanggal: NaiveDate,
) -> AppResult<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
             SELECT 1 FROM absensi_ph ap
             JOIN jadwal_ph j ON j.id = ap.jadwal_ph_id
              WHERE ap.personel_id = $1 AND j.tanggal = $2
         )",
    )
    .bind(personel_id)
    .bind(tanggal)
    .fetch_one(pool)
    .await?)
}

/// Ada sesi strong point hari ini yang sudah check-in tetapi belum check-out.
pub async fn sesi_ph_terbuka(
    pool: &PgPool,
    personel_id: Uuid,
    tanggal: NaiveDate,
) -> AppResult<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
             SELECT 1 FROM absensi_ph masuk
             JOIN jadwal_ph j ON j.id = masuk.jadwal_ph_id
              WHERE masuk.personel_id = $1
                AND j.tanggal = $2
                AND masuk.jenis = 'masuk'
                AND NOT EXISTS (
                    SELECT 1 FROM absensi_ph pulang
                     WHERE pulang.personel_id = masuk.personel_id
                       AND pulang.jadwal_ph_id = masuk.jadwal_ph_id
                       AND pulang.jenis = 'pulang'
                )
         )",
    )
    .bind(personel_id)
    .bind(tanggal)
    .fetch_one(pool)
    .await?)
}

pub async fn sesi_ph_terakhir(
    pool: &PgPool,
    jadwal_id: Uuid,
    personel_id: Uuid,
) -> AppResult<SesiPhRow> {
    const SQL: &str = r#"
        SELECT
            (SELECT max(captured_at) FROM absensi_ph
              WHERE jadwal_ph_id = $1 AND personel_id = $2 AND jenis = 'masuk')
                AS masuk_terakhir,
            (SELECT max(captured_at) FROM absensi_ph
              WHERE jadwal_ph_id = $1 AND personel_id = $2 AND jenis = 'pulang')
                AS pulang_terakhir
    "#;
    Ok(sqlx::query_as::<_, SesiPhRow>(SQL)
        .bind(jadwal_id)
        .bind(personel_id)
        .fetch_one(pool)
        .await?)
}

pub async fn insert_absensi_ph(pool: &PgPool, data: &NewAbsensiPh) -> AppResult<i64> {
    let result = sqlx::query(
        "INSERT INTO absensi_ph (
             id, client_uuid, jadwal_ph_id, personel_id, strong_point_id, satker_id,
             jenis, captured_at, jam, latitude, longitude, akurasi_m, jarak_m,
             dalam_radius, foto_path, is_offline, is_mock_suspect, perlu_verifikasi,
             menit_telat, menit_pulang_cepat
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                 $10::float8::numeric, $11::float8::numeric, $12::float8::numeric,
                 $13::float8::numeric, $14, $15, $16, $17, $18, $19, $20)
         ON CONFLICT (client_uuid) DO NOTHING",
    )
    .bind(data.id)
    .bind(data.client_uuid)
    .bind(data.jadwal_ph_id)
    .bind(data.personel_id)
    .bind(data.strong_point_id)
    .bind(data.satker_id)
    .bind(data.jenis)
    .bind(data.captured_at)
    .bind(data.jam)
    .bind(data.latitude)
    .bind(data.longitude)
    .bind(data.akurasi_m)
    .bind(data.jarak_m)
    .bind(data.dalam_radius)
    .bind(data.foto_path.as_deref())
    .bind(data.is_offline)
    .bind(data.is_mock_suspect)
    .bind(data.perlu_verifikasi)
    .bind(data.menit_telat)
    .bind(data.menit_pulang_cepat)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() as i64)
}

pub async fn list_absensi_ph_saya(
    pool: &PgPool,
    personel_id: Uuid,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<AbsensiPhRow>> {
    const SQL: &str = r#"
        SELECT ap.id, ap.client_uuid, ap.jadwal_ph_id, ap.personel_id,
               p.nama AS personel_nama, p.nrp AS personel_nrp,
               ap.strong_point_id, sp.nama AS strong_point_nama, ap.satker_id,
               ap.jenis, ap.captured_at, ap.jam,
               ap.latitude::float8 AS latitude, ap.longitude::float8 AS longitude,
               ap.akurasi_m::float8 AS akurasi_m, ap.jarak_m::float8 AS jarak_m,
               ap.dalam_radius, ap.foto_path, ap.is_offline, ap.is_mock_suspect,
               ap.perlu_verifikasi, ap.verified_at, ap.catatan_verifikator,
               ap.menit_telat, ap.menit_pulang_cepat
          FROM absensi_ph ap
          JOIN personel p ON p.id = ap.personel_id
          JOIN strong_point sp ON sp.id = ap.strong_point_id
          JOIN jadwal_ph j ON j.id = ap.jadwal_ph_id
         WHERE ap.personel_id = $1 AND ap.captured_at::date BETWEEN $2 AND $3
           AND NOT j.is_arsip
         ORDER BY ap.captured_at
    "#;
    Ok(sqlx::query_as::<_, AbsensiPhRow>(SQL)
        .bind(personel_id)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

pub async fn list_absensi_ph_jadwal(
    pool: &PgPool,
    jadwal_id: Uuid,
) -> AppResult<Vec<AbsensiPhRow>> {
    const SQL: &str = r#"
        SELECT ap.id, ap.client_uuid, ap.jadwal_ph_id, ap.personel_id,
               p.nama AS personel_nama, p.nrp AS personel_nrp,
               ap.strong_point_id, sp.nama AS strong_point_nama, ap.satker_id,
               ap.jenis, ap.captured_at, ap.jam,
               ap.latitude::float8 AS latitude, ap.longitude::float8 AS longitude,
               ap.akurasi_m::float8 AS akurasi_m, ap.jarak_m::float8 AS jarak_m,
               ap.dalam_radius, ap.foto_path, ap.is_offline, ap.is_mock_suspect,
               ap.perlu_verifikasi, ap.verified_at, ap.catatan_verifikator,
               ap.menit_telat, ap.menit_pulang_cepat
          FROM absensi_ph ap
          JOIN personel p ON p.id = ap.personel_id
          JOIN strong_point sp ON sp.id = ap.strong_point_id
         WHERE ap.jadwal_ph_id = $1
         ORDER BY ap.jam
    "#;
    Ok(sqlx::query_as::<_, AbsensiPhRow>(SQL)
        .bind(jadwal_id)
        .fetch_all(pool)
        .await?)
}

pub async fn rekap_ph(
    pool: &PgPool,
    satker_ids: &[Uuid],
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<RekapPhRow>> {
    const SQL: &str = r#"
        SELECT p.id AS personel_id, p.nama, p.nrp, p.pangkat, p.unit,
               count(DISTINCT jp.jadwal_ph_id) AS jumlah_jadwal,
               count(DISTINCT ap.jadwal_ph_id) FILTER (WHERE ap.jenis = 'masuk') AS hadir,
               count(DISTINCT jp.jadwal_ph_id) FILTER (
                   WHERE EXISTS (
                       SELECT 1 FROM absensi_ph ap2
                        WHERE ap2.jadwal_ph_id = jp.jadwal_ph_id
                          AND ap2.personel_id = p.id
                          AND ap2.jenis = 'masuk'
                   )
                   AND NOT EXISTS (
                       SELECT 1 FROM absensi_ph ap3
                        WHERE ap3.jadwal_ph_id = jp.jadwal_ph_id
                          AND ap3.personel_id = p.id
                          AND ap3.jenis = 'pulang'
                   )
               ) AS hanya_check_in,
               count(DISTINCT jp.jadwal_ph_id)
                   - count(DISTINCT ap.jadwal_ph_id) FILTER (WHERE ap.jenis = 'masuk')
                   AS tidak_hadir,
               coalesce(
                   array_agg(DISTINCT j.tanggal) FILTER (
                       WHERE NOT EXISTS (
                           SELECT 1 FROM absensi_ph ap2
                            WHERE ap2.jadwal_ph_id = jp.jadwal_ph_id
                              AND ap2.personel_id = p.id
                              AND ap2.jenis = 'masuk'
                       )
                   ),
                   '{}'::date[]
               ) AS tanggal_tidak_hadir,
               coalesce(
                   array_agg(DISTINCT j.tanggal) FILTER (
                       WHERE EXISTS (
                           SELECT 1 FROM absensi_ph ap2
                            WHERE ap2.jadwal_ph_id = jp.jadwal_ph_id
                              AND ap2.personel_id = p.id
                              AND ap2.jenis = 'masuk'
                       )
                       AND NOT EXISTS (
                           SELECT 1 FROM absensi_ph ap3
                            WHERE ap3.jadwal_ph_id = jp.jadwal_ph_id
                              AND ap3.personel_id = p.id
                              AND ap3.jenis = 'pulang'
                       )
                   ),
                   '{}'::date[]
               ) AS tanggal_hanya_check_in,
               count(DISTINCT ap.id) FILTER (
                   WHERE ap.perlu_verifikasi AND ap.verified_at IS NULL
               ) AS menunggu_verifikasi,
               coalesce(sum(ap.menit_telat) FILTER (WHERE ap.jenis = 'masuk'), 0)::bigint
                   AS total_menit_telat,
               coalesce(sum(ap.menit_pulang_cepat) FILTER (WHERE ap.jenis = 'pulang'), 0)::bigint
                   AS total_menit_pulang_cepat
          FROM jadwal_ph j
          JOIN jadwal_ph_personel jp ON jp.jadwal_ph_id = j.id
          JOIN personel p ON p.id = jp.personel_id
          LEFT JOIN absensi_ph ap
                 ON ap.jadwal_ph_id = j.id AND ap.personel_id = p.id
         WHERE j.satker_id = ANY($1::uuid[]) AND j.tanggal BETWEEN $2 AND $3
           AND NOT j.is_arsip
         GROUP BY p.id, p.nama, p.nrp, p.pangkat, p.unit
         ORDER BY p.nama
    "#;
    Ok(sqlx::query_as::<_, RekapPhRow>(SQL)
        .bind(satker_ids)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

pub async fn rekap_ph_harian(
    pool: &PgPool,
    satker_ids: &[Uuid],
    dari: NaiveDate,
    sampai: NaiveDate,
    personel_id: Option<Uuid>,
) -> AppResult<Vec<RekapPhHarianRow>> {
    const SQL: &str = r#"
        SELECT j.tanggal, j.id AS jadwal_ph_id, j.sesi, j.jam_mulai, j.jam_selesai,
               sp.nama AS strong_point_nama, sp.id AS strong_point_id,
               sp.latitude::float8 AS strong_point_latitude,
               sp.longitude::float8 AS strong_point_longitude,
               sp.radius_m AS strong_point_radius_m, j.regu,
               p.id AS personel_id, p.nama, p.nrp, p.pangkat, p.unit,
               max(ap.jam) FILTER (WHERE ap.jenis = 'masuk') AS masuk_jam,
               max(ap.jam) FILTER (WHERE ap.jenis = 'pulang') AS pulang_jam,
               (max(ap.latitude) FILTER (WHERE ap.jenis = 'masuk'))::float8 AS masuk_latitude,
               (max(ap.longitude) FILTER (WHERE ap.jenis = 'masuk'))::float8 AS masuk_longitude,
               coalesce(max(ap.menit_telat) FILTER (WHERE ap.jenis = 'masuk'), 0)::bigint
                   AS menit_telat,
               coalesce(max(ap.menit_pulang_cepat) FILTER (WHERE ap.jenis = 'pulang'), 0)::bigint
                   AS menit_pulang_cepat
          FROM jadwal_ph j
          JOIN jadwal_ph_personel jp ON jp.jadwal_ph_id = j.id
          JOIN personel p ON p.id = jp.personel_id
          JOIN strong_point sp ON sp.id = j.strong_point_id
          LEFT JOIN absensi_ph ap
                 ON ap.jadwal_ph_id = j.id AND ap.personel_id = jp.personel_id
         WHERE j.satker_id = ANY($1::uuid[]) AND j.tanggal BETWEEN $2 AND $3
           AND NOT j.is_arsip
           AND ($4::uuid IS NULL OR jp.personel_id = $4)
         GROUP BY j.tanggal, j.id, j.sesi, j.jam_mulai, j.jam_selesai, sp.nama, sp.id,
                  sp.latitude, sp.longitude, sp.radius_m, j.regu,
                  p.id, p.nama, p.nrp, p.pangkat, p.unit
         ORDER BY j.tanggal, j.jam_mulai, p.nama
    "#;
    Ok(sqlx::query_as::<_, RekapPhHarianRow>(SQL)
        .bind(satker_ids)
        .bind(dari)
        .bind(sampai)
        .bind(personel_id)
        .fetch_all(pool)
        .await?)
}

pub async fn verifikasi_absensi_ph(
    pool: &PgPool,
    id: Uuid,
    verified_by: Uuid,
    catatan: Option<&str>,
) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE absensi_ph
            SET perlu_verifikasi = false, verified_by = $2, verified_at = now(),
                catatan_verifikator = $3
          WHERE id = $1",
    )
    .bind(id)
    .bind(verified_by)
    .bind(catatan)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Absensi PH tidak ditemukan".into()));
    }
    Ok(())
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FotoPhRow {
    pub id: Uuid,
    pub tanggal: NaiveDate,
    pub jenis: AbsensiJenis,
    pub jam: NaiveTime,
    pub strong_point_nama: String,
    pub foto_path: String,
}

/// Foto dokumentasi strong point pada rentang tanggal dan cakupan satker.
pub async fn list_foto_ph(
    pool: &PgPool,
    satker_ids: &[Uuid],
    personel_id: Option<Uuid>,
    dari: NaiveDate,
    sampai: NaiveDate,
) -> AppResult<Vec<FotoPhRow>> {
    const SQL: &str = r#"
        SELECT ap.id, j.tanggal, ap.jenis, ap.jam, sp.nama AS strong_point_nama,
               ap.foto_path
          FROM absensi_ph ap
          JOIN jadwal_ph j ON j.id = ap.jadwal_ph_id
          JOIN strong_point sp ON sp.id = ap.strong_point_id
         WHERE ap.satker_id = ANY($1::uuid[])
           AND ($2::uuid IS NULL OR ap.personel_id = $2)
           AND j.tanggal BETWEEN $3 AND $4
           AND NOT j.is_arsip
           AND ap.foto_path IS NOT NULL
         ORDER BY j.tanggal DESC, ap.jam DESC
    "#;
    Ok(sqlx::query_as::<_, FotoPhRow>(SQL)
        .bind(satker_ids)
        .bind(personel_id)
        .bind(dari)
        .bind(sampai)
        .fetch_all(pool)
        .await?)
}

/// Pemilik berkas foto strong point untuk pemeriksaan akses.
pub async fn pemilik_foto_ph(pool: &PgPool, storage_key: &str) -> AppResult<Option<(Uuid, Uuid)>> {
    const SQL: &str = "SELECT personel_id, satker_id FROM absensi_ph WHERE foto_path = $1";
    Ok(sqlx::query_as::<_, (Uuid, Uuid)>(SQL)
        .bind(storage_key)
        .fetch_optional(pool)
        .await?)
}
