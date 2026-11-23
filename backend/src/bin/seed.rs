//! Seed data demo: struktur Polda → Polres → Satker, role ber-scope,
//! Strong Point, jadwal PH, serta aktivitas satu bulan.
//!
//! Pemakaian:
//!   cargo run --bin seed              # isi jika database masih kosong
//!   cargo run --bin seed -- --reset   # hapus semua data lalu isi ulang
//!   cargo run --bin seed -- --reset --vps
//!       # data demo VPS: hanya Ditreskrimsus, absensi & rekap PH 1–19,
//!       # jadwal strong point 1–akhir bulan (Regu 1/Regu 2)

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Datelike, Duration as Delta, NaiveDate, NaiveTime, Timelike, Utc};
use kinerja_api::auth::password;
use kinerja_api::config::Settings;
use kinerja_api::domain::enums::{AppRole, SatkerLevel, StatusKehadiran};
use kinerja_api::service::waktu::hari_ini_wita;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

fn uid(key: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, key.as_bytes())
}

fn wita_to_utc(date: NaiveDate, hour: u32, minute: u32) -> DateTime<Utc> {
    let naive = date.and_hms_opt(hour, minute, 0).expect("jam valid");
    DateTime::from_naive_utc_and_offset(naive, Utc) - Delta::hours(8)
}

/// PDF satu halaman sederhana untuk lampiran contoh (lengkap dengan xref).
fn contoh_pdf(judul: &str, keterangan: &str) -> Vec<u8> {
    let isi =
        format!("BT /F1 16 Tf 72 780 Td ({judul}) Tj 0 -28 Td /F1 11 Tf ({keterangan}) Tj ET");
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>".to_string(),
        format!("<< /Length {} >>\nstream\n{isi}\nendstream", isi.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
    ];

    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (nomor, obj) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.push_str(&format!("{} 0 obj\n{obj}\nendobj\n", nomor + 1));
    }

    let xref = pdf.len();
    pdf.push_str(&format!(
        "xref\n0 {}\n0000000000 65535 f \n",
        objects.len() + 1
    ));
    for offset in offsets {
        pdf.push_str(&format!("{offset:010} 00000 n \n"));
    }
    pdf.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
        objects.len() + 1
    ));
    pdf.into_bytes()
}

fn contoh_png() -> Vec<u8> {
    include_bytes!("../../../dashboard/static/logo-polda-bali.png").to_vec()
}

#[allow(clippy::too_many_arguments)]
async fn simpan_lampiran_contoh(
    pool: &PgPool,
    upload_dir: &Path,
    owner: &str,
    kategori: &str,
    satker_id: Uuid,
    owner_id: Uuid,
    uploaded_by: Uuid,
    id: Uuid,
    nama: &str,
    mime: &str,
    ext: &str,
    isi: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let key = format!("lampiran/{satker_id}/{owner}/{owner_id}/{id}.{ext}");
    let path = upload_dir.join(&key);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&path, isi).await?;

    sqlx::query(
        "INSERT INTO lampiran (id, owner_type, owner_id, kategori, file_name, mime,
                               size_bytes, storage_key, uploaded_by)
         VALUES ($1, $2::berkas_owner, $3, $4::berkas_kategori, $5, $6, $7, $8, $9)",
    )
    .bind(id)
    .bind(owner)
    .bind(owner_id)
    .bind(kategori)
    .bind(nama)
    .bind(mime)
    .bind(isi.len() as i64)
    .bind(&key)
    .bind(uploaded_by)
    .execute(pool)
    .await?;

    Ok(())
}

struct NodeSeed {
    kode: &'static str,
    nama: String,
    level: SatkerLevel,
    parent: Option<&'static str>,
    latitude: f64,
    longitude: f64,
    units: Vec<&'static str>,
}

const POLDA: (&str, &str, f64, f64) = (
    "POLDA-BALI",
    "Kepolisian Daerah Bali",
    -8.6562900,
    115.2214400,
);

const POLDA_SATKER: &[(&str, &str, f64, f64, &[&str])] = &[
    (
        "DITRESKRIMSUS",
        "Ditreskrimsus Polda Bali",
        -8.6565000,
        115.2218000,
        &["Subdit I", "Subdit II", "Subdit III", "Subdit IV"],
    ),
    (
        "DITRESKRIMUM",
        "Ditreskrimum Polda Bali",
        -8.6570000,
        115.2222000,
        &["Subdit I", "Subdit II", "Subdit III"],
    ),
    (
        "BIDPROPAM",
        "Bidpropam Polda Bali",
        -8.6558000,
        115.2209000,
        &["Bagian Propam", "Bagian Paminal", "Bagian Wabprof"],
    ),
];

const POLRES: &[(&str, &str, f64, f64)] = &[
    ("POLRESTA-DPS", "Polresta Denpasar", -8.6560000, 115.2150000),
    ("POLRES-BADUNG", "Polres Badung", -8.5810000, 115.1770000),
    ("POLRES-GIANYAR", "Polres Gianyar", -8.5440000, 115.3260000),
    ("POLRES-TABANAN", "Polres Tabanan", -8.5390000, 115.1240000),
    (
        "POLRES-BULELENG",
        "Polres Buleleng",
        -8.1120000,
        115.0880000,
    ),
    (
        "POLRES-KARANGASEM",
        "Polres Karangasem",
        -8.4500000,
        115.6060000,
    ),
    (
        "POLRES-KLUNGKUNG",
        "Polres Klungkung",
        -8.5360000,
        115.4040000,
    ),
    ("POLRES-BANGLI", "Polres Bangli", -8.4540000, 115.3540000),
    (
        "POLRES-JEMBRANA",
        "Polres Jembrana",
        -8.3620000,
        114.6260000,
    ),
];

fn satker_untuk_polres(kode: &'static str) -> (String, String, String, String) {
    (
        format!("{kode}-RESKRIM"),
        format!("Satreskrim {kode}"),
        format!("{kode}-LANTAS"),
        format!("Satlantas {kode}"),
    )
}

const NAMA_DEPAN: &[&str] = &[
    "I Gede",
    "I Made",
    "I Wayan",
    "I Nyoman",
    "I Ketut",
    "I Putu",
    "Ni Luh",
    "Ni Made",
    "Anak Agung",
    "Ida Bagus",
];
const NAMA_BELAKANG: &[&str] = &[
    "Aditya",
    "Wirawan",
    "Suryana",
    "Artawan",
    "Sudiarta",
    "Mahendra",
    "Pradnyana",
    "Wibawa",
    "Kusuma",
    "Darmawan",
    "Setiawan",
    "Gunawan",
    "Pratama",
    "Nugraha",
    "Santika",
    "Wijaya",
];
const GELAR: &[&str] = &["S.H.", "S.T.", "S.Kom.", "S.Sos.", "S.H., M.H.", "A.Md."];
const PANGKAT: &[&str] = &[
    "BRIPDA", "BRIPTU", "BRIGPOL", "BRIPKA", "AIPDA", "AIPTU", "IPDA", "IPTU", "AKP",
];

const JUMLAH_PERSONEL: usize = 120;

/// Profil isi data demo.
///
/// `Development` mengisi seluruh organisasi (Polda, 9 Polres, 21 satker) agar
/// dashboard dan Android punya banyak contoh. `Vps` hanya mengisi Polda Bali
/// dan Ditreskrimsus dengan rentang tanggal yang dipakai untuk demo di server
/// produksi, tanpa mengubah perilaku seed development.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ProfilSeed {
    Development,
    Vps,
}

impl ProfilSeed {
    fn dari_argumen() -> Self {
        if std::env::args().any(|arg| arg == "--vps") {
            Self::Vps
        } else {
            Self::Development
        }
    }

    fn jumlah_personel(self) -> usize {
        match self {
            Self::Development => JUMLAH_PERSONEL,
            Self::Vps => 40,
        }
    }

    /// Batas tanggal absensi kantor dan rekap strong point pada profil VPS.
    fn batas_hari_absensi(self) -> Option<u32> {
        match self {
            Self::Development => None,
            Self::Vps => Some(19),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Development => "development (semua satker)",
            Self::Vps => "vps (Ditreskrimsus)",
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let settings = Settings::from_env().map_err(|err| format!("konfigurasi: {err}"))?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&settings.database_url)
        .await?;

    // Pastikan skema mutakhir sebelum menulis data (mis. enum status baru).
    sqlx::migrate!("./migrations").run(&pool).await?;

    let reset = std::env::args().any(|arg| arg == "--reset");
    let profil = ProfilSeed::dari_argumen();

    let existing: i64 = sqlx::query_scalar("SELECT count(*) FROM satker")
        .fetch_one(&pool)
        .await?;

    if existing > 0 && !reset {
        println!(
            "Database sudah berisi {existing} satker. Jalankan dengan --reset untuk mengisi ulang."
        );
        return Ok(());
    }

    if reset {
        println!("Menghapus data lama...");
        sqlx::query(
            "TRUNCATE strong_point, jadwal_ph, jadwal_ph_personel, absensi_ph, hari_libur,
                      satker, personel, personel_role, refresh_token, titik_dinas, periode,
                      bobot_indikator, absensi, rencana_kerja, rencana_tahapan, hasil_kerja,
                      output, lampiran, penilaian, penilaian_detail, notifikasi, audit_log CASCADE",
        )
        .execute(&pool)
        .await?;
    }

    // ------------------------------------------------------------- organisasi
    let polda_id = uid(POLDA.0);
    let mut nodes = Vec::new();
    nodes.push(NodeSeed {
        kode: POLDA.0,
        nama: POLDA.1.to_string(),
        level: SatkerLevel::Polda,
        parent: None,
        latitude: POLDA.2,
        longitude: POLDA.3,
        units: vec![],
    });

    for (kode, nama, lat, lng, units) in POLDA_SATKER {
        if profil == ProfilSeed::Vps && *kode != "DITRESKRIMSUS" {
            continue;
        }
        nodes.push(NodeSeed {
            kode,
            nama: (*nama).to_string(),
            level: SatkerLevel::Satker,
            parent: Some(POLDA.0),
            latitude: *lat,
            longitude: *lng,
            units: units.to_vec(),
        });
    }

    for (kode, nama, lat, lng) in POLRES {
        if profil == ProfilSeed::Vps {
            break;
        }
        nodes.push(NodeSeed {
            kode,
            nama: (*nama).to_string(),
            level: SatkerLevel::Polres,
            parent: Some(POLDA.0),
            latitude: *lat,
            longitude: *lng,
            units: vec![],
        });
        let (kode_reskrim, nama_reskrim, kode_lantas, nama_lantas) = satker_untuk_polres(kode);
        nodes.push(NodeSeed {
            kode: Box::leak(kode_reskrim.into_boxed_str()),
            nama: nama_reskrim,
            level: SatkerLevel::Satker,
            parent: Some(kode),
            latitude: *lat + 0.0004,
            longitude: *lng + 0.0004,
            units: vec!["Unit Reskrim", "Unit Pidum", "Unit PPA"],
        });
        nodes.push(NodeSeed {
            kode: Box::leak(kode_lantas.into_boxed_str()),
            nama: nama_lantas,
            level: SatkerLevel::Satker,
            parent: Some(kode),
            latitude: *lat - 0.0004,
            longitude: *lng - 0.0004,
            units: vec!["Unit Laka", "Unit Turjawali", "Unit Regident"],
        });
    }

    for tingkat in [SatkerLevel::Polda, SatkerLevel::Polres, SatkerLevel::Satker] {
        for node in nodes.iter().filter(|n| n.level == tingkat) {
            let jam_masuk = if tingkat == SatkerLevel::Satker {
                NaiveTime::from_hms_opt(7, 30, 0).unwrap()
            } else {
                NaiveTime::from_hms_opt(8, 0, 0).unwrap()
            };
            let jam_pulang = if tingkat == SatkerLevel::Satker {
                NaiveTime::from_hms_opt(15, 30, 0).unwrap()
            } else {
                NaiveTime::from_hms_opt(16, 0, 0).unwrap()
            };
            sqlx::query(
                "INSERT INTO satker (id, parent_id, level, nama, kode, timezone, hari_kerja,
                                     jam_masuk, jam_pulang)
                 VALUES ($1, $2, $3, $4, $5, 'Asia/Makassar', '{1,2,3,4,5}', $6, $7)",
            )
            .bind(uid(node.kode))
            .bind(node.parent.map(uid))
            .bind(node.level)
            .bind(&node.nama)
            .bind(node.kode)
            .bind(jam_masuk)
            .bind(jam_pulang)
            .execute(&pool)
            .await?;
        }
    }

    let satker_nodes: Vec<&NodeSeed> = nodes
        .iter()
        .filter(|n| n.level == SatkerLevel::Satker)
        .collect();

    // Titik dinas dan strong point untuk setiap satker.
    for node in &satker_nodes {
        sqlx::query(
            "INSERT INTO titik_dinas (id, satker_id, nama, jenis, latitude, longitude, radius_m)
             VALUES ($1, $2, $3, 'kantor', $4::float8::numeric, $5::float8::numeric, 100)",
        )
        .bind(uid(&format!("titik-{}", node.kode)))
        .bind(uid(node.kode))
        .bind(format!("Markas {}", node.nama))
        .bind(node.latitude)
        .bind(node.longitude)
        .execute(&pool)
        .await?;

        sqlx::query(
            "INSERT INTO strong_point (id, satker_id, nama, alamat, latitude, longitude, radius_m)
             VALUES ($1, $2, $3, $4, $5::float8::numeric, $6::float8::numeric, 100)",
        )
        .bind(uid(&format!("strong-{}", node.kode)))
        .bind(uid(node.parent.unwrap_or(POLDA.0)))
        .bind(format!("Strong Point {}", node.nama))
        .bind(format!("Pos penjagaan {}", node.nama))
        .bind(node.latitude + 0.0002)
        .bind(node.longitude + 0.0002)
        .execute(&pool)
        .await?;
    }

    // ------------------------------------------------------------- personel
    let hash_superuser = password::hash("Superuser#2026")?;
    let hash_pimpinan = password::hash("Pimpinan#2026")?;
    let hash_admin_polda = password::hash("AdminPolda#2026")?;
    let hash_admin_satker = password::hash("AdminSatker#2026")?;
    let hash_admin_polres = password::hash("AdminPolres#2026")?;
    let hash_pimpinan_polres = password::hash("PimpinanPolres#2026")?;
    let hash_pimpinan_satker = password::hash("PimpinanSatker#2026")?;
    let hash_atasan = password::hash("Atasan#2026")?;
    let hash_personel = password::hash("Polri#2026")?;

    let polda_satker_id = uid(POLDA_SATKER[0].0);
    let polres_dps_id = uid(POLRES[0].0);
    let satker_polres_dps_id = uid(&format!("{}-RESKRIM", POLRES[0].0));

    #[allow(clippy::too_many_arguments)]
    async fn insert_personel(
        pool: &PgPool,
        id: Uuid,
        nrp: &str,
        nama: &str,
        pangkat: &str,
        jabatan: &str,
        unit: Option<&str>,
        satker_id: Uuid,
        atasan_id: Option<Uuid>,
        password_hash: &str,
        roles: &[(AppRole, Option<Uuid>)],
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO personel (id, jenis_akun, nrp, nama, pangkat, jabatan, unit, satker_id,
                                   atasan_id, password_hash, must_change_password, is_active)
             VALUES ($1, 'personel', $2, $3, $4, $5, $6, $7, $8, $9, false, true)",
        )
        .bind(id)
        .bind(nrp)
        .bind(nama)
        .bind(pangkat)
        .bind(jabatan)
        .bind(unit)
        .bind(satker_id)
        .bind(atasan_id)
        .bind(password_hash)
        .execute(pool)
        .await?;

        for (role, scope) in roles {
            sqlx::query(
                "INSERT INTO personel_role (id, personel_id, role, scope_satker_id)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT DO NOTHING",
            )
            .bind(uid(&format!("role-{nrp}-{role:?}-{scope:?}")))
            .bind(id)
            .bind(*role)
            .bind(*scope)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_admin(
        pool: &PgPool,
        id: Uuid,
        username: &str,
        nama: &str,
        pangkat: &str,
        jabatan: &str,
        satker_id: Uuid,
        password_hash: &str,
        role: AppRole,
        scope: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO personel (id, jenis_akun, username, nama, pangkat, jabatan, unit,
                                   satker_id, atasan_id, password_hash, must_change_password,
                                   is_active)
             VALUES ($1, 'admin', $2, $3, $4, $5, NULL, $6, NULL, $7, false, true)",
        )
        .bind(id)
        .bind(username)
        .bind(nama)
        .bind(pangkat)
        .bind(jabatan)
        .bind(satker_id)
        .bind(password_hash)
        .execute(pool)
        .await?;

        sqlx::query(
            "INSERT INTO personel_role (id, personel_id, role, scope_satker_id)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT DO NOTHING",
        )
        .bind(uid(&format!("role-{username}-{role:?}-{scope:?}")))
        .bind(id)
        .bind(role)
        .bind(scope)
        .execute(pool)
        .await?;
        Ok(())
    }

    // Akun admin (pengelola sistem). Admin tidak memiliki NRP.
    insert_admin(
        &pool,
        uid("admin"),
        "superuser",
        "Super User",
        "AKP",
        "Administrator Sistem",
        polda_satker_id,
        &hash_superuser,
        AppRole::AdminSistem,
        None,
    )
    .await?;

    insert_personel(
        &pool,
        uid("pimpinan-polda"),
        "000002",
        "Kombes Pol I Wayan Sudarma, S.H., M.H.",
        "KOMBES POL",
        "Dirreskrimsus",
        Some("Pimpinan"),
        polda_satker_id,
        None,
        &hash_pimpinan,
        &[(AppRole::PimpinanPolda, Some(polda_id))],
    )
    .await?;

    insert_admin(
        &pool,
        uid("admin-polda"),
        "admin_polda",
        "Admin Polda Bali",
        "AKP",
        "Administrator Polda",
        polda_satker_id,
        &hash_admin_polda,
        AppRole::AdminPolda,
        Some(polda_id),
    )
    .await?;

    insert_admin(
        &pool,
        uid("admin-satker-polda"),
        "admin_satker_polda",
        "Admin Satker Polda",
        "AKP",
        "Administrator Satker",
        polda_satker_id,
        &hash_admin_satker,
        AppRole::AdminSatkerPolda,
        Some(polda_satker_id),
    )
    .await?;

    insert_personel(
        &pool,
        uid("pimpinan-satker-polda"),
        "000012",
        "Pimpinan Satker Polda",
        "AKP",
        "Kasubdit",
        Some("Subdit I"),
        polda_satker_id,
        None,
        &hash_pimpinan_satker,
        &[(AppRole::PimpinanSatkerPolda, Some(polda_satker_id))],
    )
    .await?;

    // Akun Polres hanya dibuat pada profil development; profil VPS tidak
    // memiliki satker Polres sehingga baris ini akan melanggar foreign key.
    if profil == ProfilSeed::Development {
        insert_admin(
            &pool,
            uid("admin-polres"),
            "admin_polres",
            "Admin Polresta Denpasar",
            "AKP",
            "Administrator Polres",
            satker_polres_dps_id,
            &hash_admin_polres,
            AppRole::AdminPolres,
            Some(polres_dps_id),
        )
        .await?;

        insert_admin(
            &pool,
            uid("admin-satker-polres"),
            "admin_satker_polres",
            "Admin Satker Polres",
            "AKP",
            "Administrator Satker",
            satker_polres_dps_id,
            &hash_admin_satker,
            AppRole::AdminSatkerPolres,
            Some(satker_polres_dps_id),
        )
        .await?;

        insert_personel(
            &pool,
            uid("pimpinan-polres"),
            "000022",
            "Pimpinan Polresta Denpasar",
            "AKP",
            "Kasatreskrim",
            Some("Unit Reskrim"),
            satker_polres_dps_id,
            None,
            &hash_pimpinan_polres,
            &[(AppRole::PimpinanPolres, Some(polres_dps_id))],
        )
        .await?;

        insert_personel(
            &pool,
            uid("pimpinan-satker-polres"),
            "000023",
            "Pimpinan Satker Polres",
            "AKP",
            "Kanit",
            Some("Unit Reskrim"),
            satker_polres_dps_id,
            None,
            &hash_pimpinan_satker,
            &[(AppRole::PimpinanSatkerPolres, Some(satker_polres_dps_id))],
        )
        .await?;
    }

    let pimpinan_polda_id = uid("pimpinan-polda");
    let mut atasan_map: HashMap<Uuid, Uuid> = HashMap::new();
    let mut personel_per_satker: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    let mut personel_ids: Vec<(Uuid, Uuid, String)> = Vec::new();

    // Satu atasan per satker.
    for (index, node) in satker_nodes.iter().enumerate() {
        let nrp = format!("0001{:02}", index);
        let atasan_id = uid(&format!("atasan-{}", node.kode));
        insert_personel(
            &pool,
            atasan_id,
            &nrp,
            &format!("Atasan {}", node.nama),
            "AKP",
            "Atasan Langsung",
            node.units.first().copied(),
            uid(node.kode),
            Some(pimpinan_polda_id),
            &hash_atasan,
            &[(AppRole::Atasan, None), (AppRole::Personel, None)],
        )
        .await?;
        atasan_map.insert(uid(node.kode), atasan_id);
    }

    // Personel tersebar merata di satker.
    for index in 0..profil.jumlah_personel() {
        let node = satker_nodes[index % satker_nodes.len()];
        let satker_id = uid(node.kode);
        let unit = node.units[(index / satker_nodes.len()) % node.units.len()];
        let nama = format!(
            "{} {}, {}",
            NAMA_DEPAN[index % NAMA_DEPAN.len()],
            NAMA_BELAKANG[(index * 7 + 3) % NAMA_BELAKANG.len()],
            GELAR[index % GELAR.len()]
        );
        let pangkat = PANGKAT[index % PANGKAT.len()];
        let jabatan = if unit.contains("Bagian") {
            "Bamin"
        } else {
            "Penyidik"
        };
        let nrp = format!("85{:02}{:04}", index % 100, index + 1);
        let id = uid(&format!("personel-{nrp}"));
        let atasan_id = atasan_map.get(&satker_id).copied();

        insert_personel(
            &pool,
            id,
            &nrp,
            &nama,
            pangkat,
            &format!("{jabatan} / {unit}"),
            Some(unit),
            satker_id,
            atasan_id,
            &hash_personel,
            &[(AppRole::Personel, None)],
        )
        .await?;

        personel_ids.push((id, satker_id, nrp));
        personel_per_satker.entry(satker_id).or_default().push(id);
    }

    // -------------------------------------------- penempatan strong point
    // Titik anggota bersifat tetap: satu anggota = satu titik. Semua akun
    // personel aktif pada satker (termasuk atasan/pimpinan) ditempatkan ke
    // titik induknya masing-masing.
    for node in &satker_nodes {
        let satker_id = uid(node.kode);
        let anggota: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM personel
              WHERE satker_id = $1 AND jenis_akun = 'personel'
                AND deleted_at IS NULL AND is_active",
        )
        .bind(satker_id)
        .fetch_all(&pool)
        .await?;
        for personel_id in anggota {
            sqlx::query(
                "INSERT INTO penempatan_ph
                     (id, personel_id, strong_point_id, created_by)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (personel_id) DO NOTHING",
            )
            .bind(uid(&format!("penempatan-{personel_id}")))
            .bind(personel_id)
            .bind(uid(&format!("strong-{}", node.kode)))
            .bind(uid("admin-polda"))
            .execute(&pool)
            .await?;
        }
    }

    // -------------------------------------------------------------- periode
    let today = hari_ini_wita();
    let awal_bulan =
        NaiveDate::from_ymd_opt(today.year(), today.month(), 1).expect("tanggal valid");
    let (tahun_akhir, bulan_akhir) = if today.month() == 12 {
        (today.year() + 1, 1)
    } else {
        (today.year(), today.month() + 1)
    };
    let akhir_bulan = NaiveDate::from_ymd_opt(tahun_akhir, bulan_akhir, 1).expect("tanggal valid")
        - Delta::days(1);

    let periode_id = uid(&format!("periode-{awal_bulan}"));
    let nama_periode = format!(
        "{} {}",
        match awal_bulan.month() {
            1 => "Januari",
            2 => "Februari",
            3 => "Maret",
            4 => "April",
            5 => "Mei",
            6 => "Juni",
            7 => "Juli",
            8 => "Agustus",
            9 => "September",
            10 => "Oktober",
            11 => "November",
            _ => "Desember",
        },
        awal_bulan.year()
    );

    sqlx::query(
        "INSERT INTO periode (id, nama, tipe, tanggal_mulai, tanggal_selesai, status)
         VALUES ($1, $2, 'bulanan', $3, $4, 'open')",
    )
    .bind(periode_id)
    .bind(&nama_periode)
    .bind(awal_bulan)
    .bind(akhir_bulan)
    .execute(&pool)
    .await?;

    for (indikator, bobot) in [
        ("absensi", 20.0_f64),
        ("rencana_kerja", 20.0),
        ("hasil_kerja", 30.0),
        ("output", 30.0),
    ] {
        sqlx::query(
            "INSERT INTO bobot_indikator (id, periode_id, indikator, bobot)
             VALUES ($1, $2, $3::indikator_kinerja, $4::float8::numeric)",
        )
        .bind(uid(&format!("bobot-{periode_id}-{indikator}")))
        .bind(periode_id)
        .bind(indikator)
        .bind(bobot)
        .execute(&pool)
        .await?;
    }

    // ------------------------------------------------------- hari libur tetap
    let libur_tetap = [
        (1, 1, "Tahun Baru"),
        (5, 1, "Hari Buruh Internasional"),
        (6, 1, "Hari Lahir Pancasila"),
        (8, 17, "HUT Kemerdekaan RI"),
        (12, 25, "Hari Raya Natal"),
    ];
    for (bulan, hari, nama) in libur_tetap {
        let tanggal =
            NaiveDate::from_ymd_opt(today.year(), bulan, hari).expect("tanggal libur valid");
        sqlx::query(
            "INSERT INTO hari_libur (id, tanggal, nama, jenis, satker_id)
             VALUES ($1, $2, $3, 'nasional', NULL)",
        )
        .bind(uid(&format!("libur-{tanggal}")))
        .bind(tanggal)
        .bind(nama)
        .execute(&pool)
        .await?;
    }
    let libur_tetap_len = libur_tetap.len();

    // -------------------------------------------------------- jadwal PH demo
    // Roster penuh sebulan untuk Ditreskrimsus dapat dimatikan lewat env agar
    // database uji memakai jadwal contoh yang tidak menabrak skenario e2e.
    // Profil VPS selalu memakai roster penuh agar jadwal Regu 1/Regu 2 lengkap
    // sesuai kebutuhan demo, terlepas dari env yang terbawa dari development.
    let roster_penuh = profil == ProfilSeed::Vps
        || std::env::var("APP__SEED_FULL_ROSTER")
            .map(|nilai| {
                !matches!(
                    nilai.trim().to_ascii_lowercase().as_str(),
                    "0" | "false" | "off"
                )
            })
            .unwrap_or(true);
    let mut ph_dates: HashMap<Uuid, HashSet<NaiveDate>> = HashMap::new();
    let mut jadwal_ph_rows: Vec<(Uuid, Uuid, Uuid, NaiveDate, NaiveTime, NaiveTime)> = Vec::new();

    for (satker_index, node) in satker_nodes.iter().enumerate() {
        let satker_id = uid(node.kode);
        let strong_point_id = uid(&format!("strong-{}", node.kode));
        let orang = personel_per_satker
            .get(&satker_id)
            .cloned()
            .unwrap_or_default();
        if orang.is_empty() {
            continue;
        }
        // Ditreskrimsus dibagi dua regu: Regu 1 tanggal 1–15 dan Regu 2 tanggal 16–akhir
        // bulan. Satker lain cukup contoh tanggal 3–5 dengan satu regu.
        let rencana_regu: Vec<(&str, Vec<Uuid>, Vec<u32>)> =
            if node.kode == "DITRESKRIMSUS" && roster_penuh {
                let tengah = orang.len().div_ceil(2);
                vec![
                    (
                        "Regu 1",
                        orang.iter().take(tengah).copied().collect(),
                        (1..=15).filter(|hari| *hari <= akhir_bulan.day()).collect(),
                    ),
                    (
                        "Regu 2",
                        orang.iter().skip(tengah).copied().collect(),
                        (16..=akhir_bulan.day()).collect(),
                    ),
                ]
            } else {
                let anggota: Vec<Uuid> = (0..4.min(orang.len()))
                    .map(|offset| orang[(satker_index + offset) % orang.len()])
                    .collect();
                vec![(
                    "Regu 1",
                    anggota,
                    [3u32, 4, 5]
                        .into_iter()
                        .filter(|hari| *hari <= akhir_bulan.day())
                        .collect(),
                )]
            };
        for (regu, anggota_regu, hari_jadwal) in &rencana_regu {
            if anggota_regu.is_empty() {
                continue;
            }
            for hari in hari_jadwal {
                let hari = *hari;
                let tanggal = NaiveDate::from_ymd_opt(awal_bulan.year(), awal_bulan.month(), hari)
                    .expect("tanggal PH valid");
                if tanggal.weekday().number_from_monday() > 5 {
                    continue;
                }
                let sesi_hari: &[(&str, u32, u32)] = if node.kode == "DITRESKRIMSUS" {
                    &[("pagi", 7, 9), ("sore", 16, 18)]
                } else {
                    &[("pagi", 7, 9)]
                };
                for (sesi, jam_mulai, jam_selesai) in sesi_hari {
                    let jam_mulai =
                        NaiveTime::from_hms_opt(*jam_mulai, 0, 0).expect("jam PH valid");
                    let jam_selesai =
                        NaiveTime::from_hms_opt(*jam_selesai, 0, 0).expect("jam PH valid");
                    let id = uid(&format!("jadwal-ph-{satker_id}-{regu}-{sesi}-{hari}"));
                    sqlx::query(
                        "INSERT INTO jadwal_ph (id, satker_id, strong_point_id, tanggal, jam_mulai,
                         jam_selesai, sesi, regu, catatan, created_by)
                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                    )
                    .bind(id)
                    .bind(satker_id)
                    .bind(strong_point_id)
                    .bind(tanggal)
                    .bind(jam_mulai)
                    .bind(jam_selesai)
                    .bind(*sesi)
                    .bind(regu)
                    .bind(format!("Jadwal Strong Point {} ({sesi})", node.nama))
                    .bind(uid("admin-polda"))
                    .execute(&pool)
                    .await?;

                    for personel_id in anggota_regu {
                        sqlx::query(
                            "INSERT INTO jadwal_ph_personel (jadwal_ph_id, personel_id, peran)
                             VALUES ($1, $2, $3)",
                        )
                        .bind(id)
                        .bind(personel_id)
                        .bind("Anggota")
                        .execute(&pool)
                        .await?;
                        ph_dates.entry(*personel_id).or_default().insert(tanggal);
                    }
                    jadwal_ph_rows.push((
                        id,
                        satker_id,
                        strong_point_id,
                        tanggal,
                        jam_mulai,
                        jam_selesai,
                    ));
                }
            }
        }
    }

    // -------------------------------------------------------------- absensi
    // Development: sampai kemarin. VPS: sampai hari ini, maksimal tanggal 19
    // (dan tetap tidak melewati akhir bulan).
    let mut batas_absensi = akhir_bulan.min(today - Delta::days(1));
    if let Some(hari) = profil.batas_hari_absensi() {
        let batas_profil = NaiveDate::from_ymd_opt(awal_bulan.year(), awal_bulan.month(), hari)
            .expect("batas absensi valid");
        batas_absensi = batas_absensi.max(today).min(batas_profil);
    }
    let mut tanggal = awal_bulan;
    while tanggal <= batas_absensi {
        let iso_weekday = tanggal.weekday().num_days_from_monday() + 1;
        if iso_weekday <= 5 {
            for (index, (personel_id, satker_id, _nrp)) in personel_ids.iter().enumerate() {
                let ph_hari_ini = ph_dates
                    .get(personel_id)
                    .is_some_and(|set| set.contains(&tanggal));
                let pola = (index * 13 + tanggal.day() as usize * 7) % 100;
                let (status, jam_masuk) = if ph_hari_ini {
                    // Pada hari strong point personel tetap wajib absensi kantor;
                    // sengaja dibuat terlambat untuk menguji pengecualian telat.
                    (
                        StatusKehadiran::DinasKantor,
                        NaiveTime::from_hms_opt(8, 30, 0).unwrap(),
                    )
                } else if pola < 80 {
                    (
                        StatusKehadiran::DinasKantor,
                        NaiveTime::from_hms_opt(7, 20, 0).unwrap(),
                    )
                } else if pola < 88 {
                    (
                        StatusKehadiran::DinasKantor,
                        NaiveTime::from_hms_opt(7, 50, 0).unwrap(),
                    )
                } else if pola < 92 {
                    (
                        StatusKehadiran::Piket,
                        NaiveTime::from_hms_opt(7, 25, 0).unwrap(),
                    )
                } else if pola < 96 {
                    (
                        StatusKehadiran::Cuti,
                        NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
                    )
                } else {
                    (
                        StatusKehadiran::TanpaKeterangan,
                        NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
                    )
                };

                let hadir = status.dihitung_hadir();
                let jam_pulang = if hadir && (index + tanggal.day() as usize).is_multiple_of(17) {
                    NaiveTime::from_hms_opt(15, 5, 0).unwrap()
                } else {
                    NaiveTime::from_hms_opt(15, 35, 0).unwrap()
                };

                for (jenis, jam) in [("masuk", jam_masuk), ("pulang", jam_pulang)] {
                    if !hadir && jenis == "pulang" {
                        continue;
                    }
                    let offline = hadir && (index + tanggal.day() as usize).is_multiple_of(41);
                    sqlx::query(
                        "INSERT INTO absensi (id, client_uuid, personel_id, satker_id, tanggal, jenis,
                                              status_kehadiran, captured_at, jam, latitude, longitude,
                                              akurasi_m, jarak_m, dalam_radius, is_offline, perlu_verifikasi)
                         VALUES ($1, $2, $3, $4, $5, $6::absensi_jenis, $7::status_kehadiran, $8, $9,
                                 $10::float8::numeric, $11::float8::numeric, 12, 8.5, true, $12, $12)",
                    )
                    .bind(uid(&format!("absensi-{personel_id}-{tanggal}-{jenis}")))
                    .bind(Uuid::new_v5(
                        &Uuid::NAMESPACE_OID,
                        format!("{personel_id}{tanggal}{jenis}").as_bytes(),
                    ))
                    .bind(personel_id)
                    .bind(satker_id)
                    .bind(tanggal)
                    .bind(jenis)
                    .bind(status)
                    .bind(wita_to_utc(tanggal, jam.hour(), jam.minute()))
                    .bind(jam)
                    .bind(-8.65629_f64)
                    .bind(115.22144_f64)
                    .bind(offline)
                    .execute(&pool)
                    .await?;
                }
            }
        }
        tanggal += Delta::days(1);
    }

    // Absensi PH untuk jadwal yang sudah lewat. Profil VPS boleh mengisi hari
    // ini agar rekap "sampai hari ini" tidak menampilkan semua personel sebagai
    // belum strong point; tanggal yang akan datang tetap dilewati.
    for (jadwal_id, satker_id, strong_point_id, tanggal, jam_mulai, jam_selesai) in &jadwal_ph_rows
    {
        let belum_terjadi = if profil == ProfilSeed::Vps {
            *tanggal > today
        } else {
            *tanggal >= today
        };
        if belum_terjadi
            || profil
                .batas_hari_absensi()
                .is_some_and(|hari| tanggal.day() > hari)
        {
            continue;
        }
        let assignments: Vec<Uuid> = sqlx::query_scalar(
            "SELECT personel_id FROM jadwal_ph_personel WHERE jadwal_ph_id = $1",
        )
        .bind(jadwal_id)
        .fetch_all(&pool)
        .await?;
        for (index, personel_id) in assignments.iter().enumerate() {
            if (index + tanggal.day() as usize).is_multiple_of(5) {
                continue;
            }
            // Sebagian hanya check-in tanpa check-out agar rekap menampilkan kasus tersebut.
            let hanya_masuk = (index + tanggal.day() as usize).is_multiple_of(4);
            for (jenis, jam) in [
                ("masuk", *jam_mulai + Delta::minutes(5)),
                ("pulang", *jam_selesai + Delta::minutes(5)),
            ] {
                if jenis == "pulang" && hanya_masuk {
                    continue;
                }
                let jam = NaiveTime::from_num_seconds_from_midnight_opt(
                    jam.num_seconds_from_midnight(),
                    0,
                )
                .unwrap_or(*jam_mulai);
                let captured = wita_to_utc(*tanggal, jam.hour(), jam.minute());
                let menit_telat = if jenis == "masuk" { 5 } else { 0 };
                let menit_pulang_cepat = 0;
                sqlx::query(
                    "INSERT INTO absensi_ph (
                         id, client_uuid, jadwal_ph_id, personel_id, strong_point_id, satker_id,
                         jenis, captured_at, jam, latitude, longitude, akurasi_m, jarak_m,
                         dalam_radius, is_offline, is_mock_suspect, perlu_verifikasi,
                         menit_telat, menit_pulang_cepat
                     )
                     VALUES ($1, $2, $3, $4, $5, $6, $7::absensi_jenis, $8, $9,
                             $10::float8::numeric, $11::float8::numeric, 8, 2, true, false, false, false,
                             $12, $13)",
                )
                .bind(uid(&format!("absen-ph-{jadwal_id}-{personel_id}-{jenis}")))
                .bind(Uuid::new_v5(
                    &Uuid::NAMESPACE_OID,
                    format!("ph-{jadwal_id}-{personel_id}-{jenis}").as_bytes(),
                ))
                .bind(jadwal_id)
                .bind(personel_id)
                .bind(strong_point_id)
                .bind(satker_id)
                .bind(jenis)
                .bind(captured)
                .bind(jam)
                .bind(-8.65610_f64)
                .bind(115.22164_f64)
                .bind(menit_telat)
                .bind(menit_pulang_cepat)
                .execute(&pool)
                .await?;
            }
        }
    }

    // ------------------------------------------------- rencana/hasil/output
    for (index, (personel_id, satker_id, _nrp)) in personel_ids.iter().enumerate() {
        let atasan_id = atasan_map.get(satker_id).copied();
        for urutan in 1..=2 {
            let rencana_id = uid(&format!("rencana-{personel_id}-{urutan}"));
            let mulai = awal_bulan + Delta::days(1);
            let selesai = awal_bulan + Delta::days(20);
            let status = if (index + urutan).is_multiple_of(9) {
                "perlu_revisi"
            } else {
                "disetujui"
            };
            let revisi = if status == "perlu_revisi" { 1 } else { 0 };

            sqlx::query(
                "INSERT INTO rencana_kerja (id, personel_id, satker_id, periode_id, program, kegiatan,
                                            uraian, tujuan, indikator_kinerja, jenis, dasar_jenis,
                                            dasar_nomor, dasar_tanggal, dasar_perihal, target_kuantitas,
                                            satuan, target_kualitas, target_waktu, tanggal_mulai,
                                            tanggal_selesai, status, atasan_id, catatan_atasan,
                                            diajukan_at, diputuskan_at, jumlah_revisi)
                 VALUES ($1, $2, $3, $4, 'Penegakan Hukum', $5, $6, $7, 'Capaian penyelesaian perkara',
                         'bulanan', 'sp', $8, $9, 'Surat Perintah Penyelidikan', $10::float8::numeric,
                         'laporan', 'Sesuai SOP', $11, $12, $13, $14::status_rencana, $15, $16, $17, $18, $19)",
            )
            .bind(rencana_id)
            .bind(personel_id)
            .bind(satker_id)
            .bind(periode_id)
            .bind(format!("Penyelesaian berkas perkara tahap {urutan}"))
            .bind("Melaksanakan penyelidikan dan penyidikan sesuai surat perintah yang diterbitkan.")
            .bind("Menyelesaikan berkas perkara secara lengkap dan tepat waktu.")
            .bind(format!("SP/{}/XI/2026", index))
            .bind(mulai)
            .bind(if urutan == 1 { 8.0_f64 } else { 5.0 })
            .bind(selesai)
            .bind(mulai)
            .bind(selesai)
            .bind(status)
            .bind(atasan_id)
            .bind(if revisi > 0 {
                Some("Lengkapi dasar tugas dan target kuantitas.")
            } else {
                None
            })
            .bind(wita_to_utc(mulai - Delta::days(2), 9, 0))
            .bind(wita_to_utc(mulai - Delta::days(1), 10, 0))
            .bind(revisi)
            .execute(&pool)
            .await?;

            for tahap in 1..=3 {
                sqlx::query(
                    "INSERT INTO rencana_tahapan (id, rencana_id, urutan, uraian, target_kuantitas, satuan, bobot)
                     VALUES ($1, $2, $3, $4, $5::float8::numeric, 'kegiatan', $6::float8::numeric)",
                )
                .bind(uid(&format!("tahap-{rencana_id}-{tahap}")))
                .bind(rencana_id)
                .bind(tahap)
                .bind(format!("Tahapan {tahap}"))
                .bind(2.0_f64)
                .bind(33.33_f64)
                .execute(&pool)
                .await?;
            }

            let skor_kualitas = 78.0 + ((index + urutan) % 20) as f64;
            let skor_kuantitas = 80.0 + ((index * 3 + urutan) % 21) as f64;
            let skor_waktu = if (index + urutan).is_multiple_of(7) {
                85.0
            } else {
                100.0
            };
            let skor_total = 0.4 * skor_kuantitas + 0.3 * skor_kualitas + 0.3 * skor_waktu;

            let hasil_id = uid(&format!("hasil-{rencana_id}"));
            sqlx::query(
                "INSERT INTO hasil_kerja (id, rencana_id, personel_id, satker_id, uraian_pelaksanaan,
                                          progres_persen, jumlah_realisasi, kendala, tanggal_realisasi,
                                          status, skor_kuantitas, skor_kualitas, skor_waktu, skor_total,
                                          verified_by, catatan_verifikator, verified_at)
                 VALUES ($1, $2, $3, $4, $5, 100, $6::float8::numeric, $7, $8, 'terverifikasi',
                         $9::float8::numeric, $10::float8::numeric, $11::float8::numeric,
                         $12::float8::numeric, $13, $14, $15)",
            )
            .bind(hasil_id)
            .bind(rencana_id)
            .bind(personel_id)
            .bind(satker_id)
            .bind("Pelaksanaan tugas berjalan sesuai rencana dan telah diperiksa atasan langsung.")
            .bind(if urutan == 1 { 7.0_f64 } else { 4.0 })
            .bind(if (index + urutan).is_multiple_of(5) {
                Some("Kendala koordinasi lintas satker")
            } else {
                None
            })
            .bind(selesai - Delta::days(1))
            .bind(skor_kuantitas)
            .bind(skor_kualitas)
            .bind(skor_waktu)
            .bind(skor_total)
            .bind(atasan_id)
            .bind("Realisasi sesuai target, dokumen lengkap.")
            .bind(wita_to_utc(selesai, 14, 0))
            .execute(&pool)
            .await?;

            let skor_kesesuaian = 82.0 + ((index + urutan * 2) % 16) as f64;
            let skor_keabsahan = 85.0 + ((index + urutan) % 13) as f64;
            let skor_output = 0.4 * 100.0 + 0.3 * skor_kesesuaian + 0.3 * skor_keabsahan;

            sqlx::query(
                "INSERT INTO output (id, hasil_kerja_id, rencana_id, personel_id, satker_id, jenis_output,
                                     judul, uraian, tanggal_hasil, status, skor_kelengkapan, skor_kesesuaian,
                                     skor_keabsahan, skor_total, verified_by, catatan_verifikator, verified_at)
                 VALUES ($1, $2, $3, $4, $5, $6::jenis_output, $7, $8, $9, $10::status_output,
                         $11::float8::numeric, $12::float8::numeric, $13::float8::numeric,
                         $14::float8::numeric, $15, $16, $17)",
            )
            .bind(uid(&format!("output-{hasil_id}")))
            .bind(hasil_id)
            .bind(rencana_id)
            .bind(personel_id)
            .bind(satker_id)
            .bind(if urutan == 1 { "laporan" } else { "berita_acara" })
            .bind(format!("Laporan Hasil Penyelidikan {urutan}"))
            .bind("Laporan hasil penyelidikan beserta rekomendasi tindak lanjut.")
            .bind(selesai)
            .bind(if (index + urutan).is_multiple_of(11) {
                "perlu_revisi"
            } else {
                "disetujui"
            })
            .bind(100.0_f64)
            .bind(skor_kesesuaian)
            .bind(skor_keabsahan)
            .bind(skor_output)
            .bind(atasan_id)
            .bind("Dokumen lengkap dan sah.")
            .bind(wita_to_utc(selesai, 15, 0))
            .execute(&pool)
            .await?;
        }
    }

    // ---------------------------------------- contoh rencana per status (Android)
    // Tiga dokumen untuk akun demo Android (NRP 85000001): draft, diajukan, perlu revisi.
    struct ContohTahapan {
        kegiatan: &'static str,
        uraian: &'static str,
        target: f64,
        satuan: &'static str,
        kualitas: &'static str,
        mulai: NaiveDate,
        selesai: NaiveDate,
    }

    struct ContohLampiran {
        nama: &'static str,
        mime: &'static str,
        ext: &'static str,
        isi: Vec<u8>,
    }

    struct ContohHasil {
        kode: &'static str,
        rencana_id: Uuid,
        personel_id: Uuid,
        uraian: &'static str,
        progres: f64,
        realisasi: Option<f64>,
        kendala: Option<&'static str>,
        tanggal: NaiveDate,
        status: &'static str,
        catatan: Option<&'static str>,
        verified_by: Option<Uuid>,
        verified_at: Option<DateTime<Utc>>,
        lampiran: Vec<ContohLampiran>,
    }

    struct ContohRencana {
        kode: &'static str,
        personel_id: Uuid,
        satker_id: Uuid,
        atasan_id: Option<Uuid>,
        program: &'static str,
        kegiatan: &'static str,
        uraian: &'static str,
        tujuan: &'static str,
        indikator: &'static str,
        jenis: &'static str,
        dasar_jenis: &'static str,
        dasar_nomor: &'static str,
        dasar_tanggal: NaiveDate,
        dasar_perihal: &'static str,
        target: f64,
        satuan: &'static str,
        kualitas: &'static str,
        mulai: NaiveDate,
        selesai: NaiveDate,
        status: &'static str,
        catatan: Option<&'static str>,
        diajukan: Option<DateTime<Utc>>,
        diputuskan: Option<DateTime<Utc>>,
        revisi: i32,
        tahapan: Vec<ContohTahapan>,
        lampiran: Vec<ContohLampiran>,
    }

    let personel_demo = personel_ids[0].0;
    let satker_demo = personel_ids[0].1;
    let atasan_demo = atasan_map.get(&satker_demo).copied();
    let atasan_personel = atasan_demo.expect("atasan demo tersedia");
    let personel_lain = personel_per_satker
        .get(&satker_demo)
        .and_then(|daftar| daftar.iter().copied().find(|id| *id != personel_demo))
        .expect("personel lain tersedia");

    let contoh_rencana = vec![
        ContohRencana {
            kode: "draft",
            personel_id: personel_demo,
            satker_id: satker_demo,
            atasan_id: Some(atasan_personel),
            program: "Penegakan Hukum",
            kegiatan: "Penyusunan laporan analisis gangguan siber",
            uraian: "Menyusun laporan analisis gangguan siber pada layanan publik daerah.",
            tujuan: "Menyediakan bahan rekomendasi mitigasi gangguan siber.",
            indikator: "Jumlah laporan analisis yang selesai tepat waktu",
            jenis: "mingguan",
            dasar_jenis: "sp",
            dasar_nomor: "SP/120/XI/2026",
            dasar_tanggal: awal_bulan + Delta::days(16),
            dasar_perihal: "Analisis gangguan siber layanan publik",
            target: 2.0,
            satuan: "laporan",
            kualitas: "100% lengkap",
            mulai: awal_bulan + Delta::days(17),
            selesai: awal_bulan + Delta::days(21),
            status: "draft",
            catatan: None,
            diajukan: None,
            diputuskan: None,
            revisi: 0,
            tahapan: vec![
                ContohTahapan {
                    kegiatan: "Pengumpulan data gangguan",
                    uraian: "Mengumpulkan log dan laporan gangguan dari unit terkait.",
                    target: 1.0,
                    satuan: "berkas",
                    kualitas: "Data lengkap",
                    mulai: awal_bulan + Delta::days(17),
                    selesai: awal_bulan + Delta::days(18),
                },
                ContohTahapan {
                    kegiatan: "Penyusunan laporan analisis",
                    uraian: "Menyusun laporan analisis dan rekomendasi mitigasi.",
                    target: 1.0,
                    satuan: "laporan",
                    kualitas: "100% lengkap",
                    mulai: awal_bulan + Delta::days(19),
                    selesai: awal_bulan + Delta::days(21),
                },
            ],
            lampiran: vec![],
        },
        ContohRencana {
            kode: "diajukan",
            personel_id: personel_demo,
            satker_id: satker_demo,
            atasan_id: Some(atasan_personel),
            program: "Pemeliharaan Keamanan",
            kegiatan: "Patroli siber media sosial",
            uraian: "Melakukan patroli siber pada media sosial untuk mendeteksi konten hoaks dan penipuan.",
            tujuan: "Menekan penyebaran konten hoaks dan penipuan daring.",
            indikator: "Jumlah temuan konten yang ditindaklanjuti",
            jenis: "harian",
            dasar_jenis: "sp",
            dasar_nomor: "SP/121/XI/2026",
            dasar_tanggal: awal_bulan + Delta::days(16),
            dasar_perihal: "Patroli siber media sosial",
            target: 5.0,
            satuan: "laporan",
            kualitas: "Tidak ada konten hoaks yang lolos",
            mulai: awal_bulan + Delta::days(17),
            selesai: awal_bulan + Delta::days(21),
            status: "diajukan",
            catatan: None,
            diajukan: Some(wita_to_utc(awal_bulan + Delta::days(16), 9, 0)),
            diputuskan: None,
            revisi: 0,
            tahapan: vec![
                ContohTahapan {
                    kegiatan: "Patroli pagi",
                    uraian: "Memantau media sosial pada pagi hari dan mencatat temuan.",
                    target: 2.0,
                    satuan: "laporan",
                    kualitas: "Lengkap",
                    mulai: awal_bulan + Delta::days(17),
                    selesai: awal_bulan + Delta::days(18),
                },
                ContohTahapan {
                    kegiatan: "Patroli sore",
                    uraian: "Memantau media sosial pada sore hari dan menindaklanjuti temuan.",
                    target: 3.0,
                    satuan: "laporan",
                    kualitas: "Lengkap",
                    mulai: awal_bulan + Delta::days(19),
                    selesai: awal_bulan + Delta::days(21),
                },
            ],
            lampiran: vec![
                ContohLampiran {
                    nama: "SP-121-Patroli-Siber.pdf",
                    mime: "application/pdf",
                    ext: "pdf",
                    isi: contoh_pdf(
                        "Surat Perintah Nomor SP/121/XI/2026",
                        "Patroli siber media sosial - Polda Bali",
                    ),
                },
                ContohLampiran {
                    nama: "Dokumentasi-Patroli-Siber.png",
                    mime: "image/png",
                    ext: "png",
                    isi: contoh_png(),
                },
            ],
        },
        ContohRencana {
            kode: "perlu-revisi",
            personel_id: personel_demo,
            satker_id: satker_demo,
            atasan_id: Some(atasan_personel),
            program: "Pembinaan Masyarakat",
            kegiatan: "Sosialisasi pencegahan penipuan daring",
            uraian: "Melaksanakan sosialisasi pencegahan penipuan daring kepada masyarakat.",
            tujuan: "Meningkatkan kesadaran masyarakat terhadap modus penipuan daring.",
            indikator: "Jumlah peserta sosialisasi dan materi tersampaikan",
            jenis: "bulanan",
            dasar_jenis: "tor",
            dasar_nomor: "TOR/045/XI/2026",
            dasar_tanggal: awal_bulan + Delta::days(10),
            dasar_perihal: "Sosialisasi pencegahan penipuan daring",
            target: 3.0,
            satuan: "kegiatan",
            kualitas: "Materi lengkap dan peserta terdata",
            mulai: awal_bulan + Delta::days(12),
            selesai: awal_bulan + Delta::days(20),
            status: "perlu_revisi",
            catatan: Some("Lengkapi target kualitas dan rincian waktu pelaksanaan."),
            diajukan: Some(wita_to_utc(awal_bulan + Delta::days(11), 9, 0)),
            diputuskan: Some(wita_to_utc(awal_bulan + Delta::days(12), 10, 0)),
            revisi: 1,
            tahapan: vec![
                ContohTahapan {
                    kegiatan: "Penyusunan materi",
                    uraian: "Menyusun materi sosialisasi dan daftar peserta.",
                    target: 1.0,
                    satuan: "dokumen",
                    kualitas: "Lengkap",
                    mulai: awal_bulan + Delta::days(12),
                    selesai: awal_bulan + Delta::days(14),
                },
                ContohTahapan {
                    kegiatan: "Pelaksanaan sosialisasi",
                    uraian: "Melaksanakan sosialisasi di lingkungan masyarakat.",
                    target: 2.0,
                    satuan: "kegiatan",
                    kualitas: "Peserta terdata",
                    mulai: awal_bulan + Delta::days(15),
                    selesai: awal_bulan + Delta::days(20),
                },
            ],
            lampiran: vec![ContohLampiran {
                nama: "TOR-045-Sosialisasi.pdf",
                mime: "application/pdf",
                ext: "pdf",
                isi: contoh_pdf(
                    "TOR Nomor TOR/045/XI/2026",
                    "Sosialisasi pencegahan penipuan daring",
                ),
            }],
        },
        // Atasan (000100) juga merupakan bawahan pimpinan (000002).
        ContohRencana {
            kode: "atasan-diajukan",
            personel_id: atasan_personel,
            satker_id: satker_demo,
            atasan_id: Some(pimpinan_polda_id),
            program: "Pengawasan dan Supervisi",
            kegiatan: "Supervisi penyelesaian perkara bawahan",
            uraian: "Melaksanakan supervisi berkas perkara yang ditangani personel di lingkungan satker.",
            tujuan: "Memastikan penyelesaian perkara sesuai prosedur dan tepat waktu.",
            indikator: "Jumlah berkas perkara yang disupervisi",
            jenis: "bulanan",
            dasar_jenis: "sp",
            dasar_nomor: "SP/122/XI/2026",
            dasar_tanggal: awal_bulan + Delta::days(17),
            dasar_perihal: "Supervisi penyelesaian perkara",
            target: 4.0,
            satuan: "berkas",
            kualitas: "Seluruh temuan ditindaklanjuti",
            mulai: awal_bulan + Delta::days(18),
            selesai: awal_bulan + Delta::days(22),
            status: "diajukan",
            catatan: None,
            diajukan: Some(wita_to_utc(awal_bulan + Delta::days(17), 9, 0)),
            diputuskan: None,
            revisi: 0,
            tahapan: vec![
                ContohTahapan {
                    kegiatan: "Pemeriksaan berkas",
                    uraian: "Memeriksa kelengkapan berkas perkara bawahan.",
                    target: 2.0,
                    satuan: "berkas",
                    kualitas: "Lengkap",
                    mulai: awal_bulan + Delta::days(18),
                    selesai: awal_bulan + Delta::days(20),
                },
                ContohTahapan {
                    kegiatan: "Tindak lanjut temuan",
                    uraian: "Menindaklanjuti temuan hasil supervisi.",
                    target: 2.0,
                    satuan: "berkas",
                    kualitas: "Tuntas",
                    mulai: awal_bulan + Delta::days(21),
                    selesai: awal_bulan + Delta::days(22),
                },
            ],
            lampiran: vec![ContohLampiran {
                nama: "SP-122-Supervisi.pdf",
                mime: "application/pdf",
                ext: "pdf",
                isi: contoh_pdf(
                    "Surat Perintah Nomor SP/122/XI/2026",
                    "Supervisi penyelesaian perkara bawahan",
                ),
            }],
        },
        // Personel lain di satker yang sama, juga mengajukan dengan lampiran.
        ContohRencana {
            kode: "personel-lain-diajukan",
            personel_id: personel_lain,
            satker_id: satker_demo,
            atasan_id: Some(atasan_personel),
            program: "Penegakan Hukum",
            kegiatan: "Pengungkapan jaringan penipuan daring",
            uraian: "Melakukan penyelidikan dan pengungkapan jaringan penipuan daring.",
            tujuan: "Mengungkap jaringan penipuan daring dan menyelamatkan korban.",
            indikator: "Jumlah tersangka dan barang bukti yang diamankan",
            jenis: "bulanan",
            dasar_jenis: "sp",
            dasar_nomor: "SP/123/XI/2026",
            dasar_tanggal: awal_bulan + Delta::days(15),
            dasar_perihal: "Pengungkapan jaringan penipuan daring",
            target: 1.0,
            satuan: "kasus",
            kualitas: "Berkas lengkap dan tersangka diamankan",
            mulai: awal_bulan + Delta::days(16),
            selesai: awal_bulan + Delta::days(20),
            status: "diajukan",
            catatan: None,
            diajukan: Some(wita_to_utc(awal_bulan + Delta::days(15), 10, 0)),
            diputuskan: None,
            revisi: 0,
            tahapan: vec![
                ContohTahapan {
                    kegiatan: "Penyelidikan awal",
                    uraian: "Menelusuri laporan korban dan bukti transaksi.",
                    target: 1.0,
                    satuan: "berkas",
                    kualitas: "Lengkap",
                    mulai: awal_bulan + Delta::days(16),
                    selesai: awal_bulan + Delta::days(18),
                },
                ContohTahapan {
                    kegiatan: "Pengungkapan kasus",
                    uraian: "Melakukan pengungkapan dan pengamanan barang bukti.",
                    target: 1.0,
                    satuan: "kasus",
                    kualitas: "Tuntas",
                    mulai: awal_bulan + Delta::days(19),
                    selesai: awal_bulan + Delta::days(20),
                },
            ],
            lampiran: vec![
                ContohLampiran {
                    nama: "SP-123-Pengungkapan.pdf",
                    mime: "application/pdf",
                    ext: "pdf",
                    isi: contoh_pdf(
                        "Surat Perintah Nomor SP/123/XI/2026",
                        "Pengungkapan jaringan penipuan daring",
                    ),
                },
                ContohLampiran {
                    nama: "Bukti-Transaksi.png",
                    mime: "image/png",
                    ext: "png",
                    isi: contoh_png(),
                },
            ],
        },
    ];

    for contoh in &contoh_rencana {
        let rencana_id = uid(&format!("rencana-demo-{personel_demo}-{}", contoh.kode));

        sqlx::query(
            "INSERT INTO rencana_kerja (
                 id, personel_id, satker_id, periode_id, program, kegiatan, uraian, tujuan,
                 indikator_kinerja, jenis, dasar_jenis, dasar_nomor, dasar_tanggal, dasar_perihal,
                 target_kuantitas, satuan, target_kualitas, target_waktu, tanggal_mulai,
                 tanggal_selesai, status, atasan_id, catatan_atasan, diajukan_at, diputuskan_at,
                 jumlah_revisi
             )
             VALUES (
                 $1, $2, $3, $4, $5, $6, $7, $8, $9, $10::jenis_rencana,
                 $11::dasar_tugas_jenis, $12, $13, $14, $15::float8::numeric, $16, $17, $18,
                 $19, $20, $21::status_rencana, $22, $23, $24, $25, $26
             )",
        )
        .bind(rencana_id)
        .bind(contoh.personel_id)
        .bind(contoh.satker_id)
        .bind(periode_id)
        .bind(contoh.program)
        .bind(contoh.kegiatan)
        .bind(contoh.uraian)
        .bind(contoh.tujuan)
        .bind(contoh.indikator)
        .bind(contoh.jenis)
        .bind(contoh.dasar_jenis)
        .bind(contoh.dasar_nomor)
        .bind(contoh.dasar_tanggal)
        .bind(contoh.dasar_perihal)
        .bind(contoh.target)
        .bind(contoh.satuan)
        .bind(contoh.kualitas)
        .bind(contoh.selesai)
        .bind(contoh.mulai)
        .bind(contoh.selesai)
        .bind(contoh.status)
        .bind(contoh.atasan_id)
        .bind(contoh.catatan)
        .bind(contoh.diajukan)
        .bind(contoh.diputuskan)
        .bind(contoh.revisi)
        .execute(&pool)
        .await?;

        for (index, tahap) in contoh.tahapan.iter().enumerate() {
            sqlx::query(
                "INSERT INTO rencana_tahapan (
                     id, rencana_id, urutan, uraian, target_kuantitas, satuan,
                     program, kegiatan, tujuan, indikator_kinerja, target_kualitas,
                     tanggal_mulai, tanggal_selesai
                 )
                 VALUES (
                     $1, $2, $3, $4, $5::float8::numeric, $6,
                     $7, $8, $9, $10, $11, $12, $13
                 )",
            )
            .bind(uid(&format!("tahap-demo-{rencana_id}-{index}")))
            .bind(rencana_id)
            .bind(index as i32 + 1)
            .bind(tahap.uraian)
            .bind(tahap.target)
            .bind(tahap.satuan)
            .bind(contoh.program)
            .bind(tahap.kegiatan)
            .bind(contoh.tujuan)
            .bind(contoh.indikator)
            .bind(tahap.kualitas)
            .bind(tahap.mulai)
            .bind(tahap.selesai)
            .execute(&pool)
            .await?;
        }

        for (index, lampiran) in contoh.lampiran.iter().enumerate() {
            let lampiran_id = uid(&format!("lampiran-demo-{rencana_id}-{index}"));
            let storage_key = format!(
                "lampiran/{}/rencana/{rencana_id}/{lampiran_id}.{}",
                contoh.satker_id, lampiran.ext
            );
            let path = settings.upload_dir.join(&storage_key);
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&path, &lampiran.isi).await?;

            sqlx::query(
                "INSERT INTO lampiran (id, owner_type, owner_id, kategori, file_name, mime,
                                       size_bytes, storage_key, uploaded_by)
                 VALUES ($1, 'rencana'::berkas_owner, $2, 'lampiran'::berkas_kategori,
                         $3, $4, $5, $6, $7)",
            )
            .bind(lampiran_id)
            .bind(rencana_id)
            .bind(lampiran.nama)
            .bind(lampiran.mime)
            .bind(lampiran.isi.len() as i64)
            .bind(&storage_key)
            .bind(contoh.personel_id)
            .execute(&pool)
            .await?;
        }
    }

    // Lampiran pada rencana yang sudah disetujui, untuk uji detail pasca-persetujuan.
    let rencana_disetujui = uid(&format!("rencana-{personel_demo}-1"));
    let lampiran_disetujui = uid(&format!("lampiran-demo-disetujui-{personel_demo}"));
    let isi_disetujui = contoh_pdf(
        "Laporan Pelaksanaan Rencana Kerja",
        "Penyelesaian berkas perkara tahap 1 - Polda Bali",
    );
    let kunci_disetujui =
        format!("lampiran/{satker_demo}/rencana/{rencana_disetujui}/{lampiran_disetujui}.pdf");
    let path_disetujui = settings.upload_dir.join(&kunci_disetujui);
    if let Some(parent) = path_disetujui.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&path_disetujui, &isi_disetujui).await?;
    sqlx::query(
        "INSERT INTO lampiran (id, owner_type, owner_id, kategori, file_name, mime,
                               size_bytes, storage_key, uploaded_by)
         VALUES ($1, 'rencana'::berkas_owner, $2, 'lampiran'::berkas_kategori,
                 $3, 'application/pdf', $4, $5, $6)",
    )
    .bind(lampiran_disetujui)
    .bind(rencana_disetujui)
    .bind("Laporan-Pelaksanaan-Tahap-1.pdf")
    .bind(isi_disetujui.len() as i64)
    .bind(&kunci_disetujui)
    .bind(personel_demo)
    .execute(&pool)
    .await?;

    // -------------------------------------------------- contoh hasil kerja
    let rencana_demo_1 = uid(&format!("rencana-{personel_demo}-1"));
    let rencana_demo_2 = uid(&format!("rencana-{personel_demo}-2"));
    let rencana_lain_1 = uid(&format!("rencana-{personel_lain}-1"));

    let contoh_hasil = vec![
        ContohHasil {
            kode: "draft",
            rencana_id: rencana_demo_1,
            personel_id: personel_demo,
            uraian: "Menyusun kelengkapan berkas perkara tahap 1.",
            progres: 40.0,
            realisasi: Some(3.0),
            kendala: None,
            tanggal: awal_bulan + Delta::days(18),
            status: "draft",
            catatan: None,
            verified_by: None,
            verified_at: None,
            lampiran: vec![],
        },
        ContohHasil {
            kode: "diajukan",
            rencana_id: rencana_demo_2,
            personel_id: personel_demo,
            uraian: "Melaksanakan penyelidikan dan menyusun berkas perkara tahap 2.",
            progres: 100.0,
            realisasi: Some(5.0),
            kendala: None,
            tanggal: awal_bulan + Delta::days(20),
            status: "diajukan",
            catatan: None,
            verified_by: None,
            verified_at: None,
            lampiran: vec![
                ContohLampiran {
                    nama: "Bukti-Penyelidikan.pdf",
                    mime: "application/pdf",
                    ext: "pdf",
                    isi: contoh_pdf(
                        "Bukti Pelaksanaan Hasil Kerja",
                        "Penyelidikan berkas perkara tahap 2",
                    ),
                },
                ContohLampiran {
                    nama: "Dokumentasi-Penyelidikan.png",
                    mime: "image/png",
                    ext: "png",
                    isi: contoh_png(),
                },
            ],
        },
        ContohHasil {
            kode: "perlu-revisi",
            rencana_id: rencana_demo_1,
            personel_id: personel_demo,
            uraian: "Menyelesaikan berkas perkara tahap 1.",
            progres: 100.0,
            realisasi: Some(8.0),
            kendala: Some("Sebagian dokumen masih dalam pengecekan."),
            tanggal: awal_bulan + Delta::days(19),
            status: "perlu_revisi",
            catatan: Some("Lengkapi bukti pelaksanaan dan tanda tangan berkas."),
            verified_by: Some(atasan_personel),
            verified_at: Some(wita_to_utc(awal_bulan + Delta::days(20), 14, 0)),
            lampiran: vec![ContohLampiran {
                nama: "Berkas-Perkara-Tahap-1.pdf",
                mime: "application/pdf",
                ext: "pdf",
                isi: contoh_pdf(
                    "Berkas Perkara Tahap 1",
                    "Ringkasan pelaksanaan dan bukti pendukung",
                ),
            }],
        },
        ContohHasil {
            kode: "personel-lain-diajukan",
            rencana_id: rencana_lain_1,
            personel_id: personel_lain,
            uraian: "Mengungkap jaringan penipuan daring dan mengamankan barang bukti.",
            progres: 100.0,
            realisasi: Some(1.0),
            kendala: None,
            tanggal: awal_bulan + Delta::days(19),
            status: "diajukan",
            catatan: None,
            verified_by: None,
            verified_at: None,
            lampiran: vec![
                ContohLampiran {
                    nama: "Bukti-Pengungkapan.pdf",
                    mime: "application/pdf",
                    ext: "pdf",
                    isi: contoh_pdf(
                        "Bukti Pengungkapan Kasus",
                        "Jaringan penipuan daring - Polda Bali",
                    ),
                },
                ContohLampiran {
                    nama: "Barang-Bukti.png",
                    mime: "image/png",
                    ext: "png",
                    isi: contoh_png(),
                },
            ],
        },
    ];

    for contoh in &contoh_hasil {
        let hasil_id = uid(&format!(
            "hasil-demo-{}-{}",
            contoh.personel_id, contoh.kode
        ));
        sqlx::query(
            "INSERT INTO hasil_kerja (id, rencana_id, personel_id, satker_id, uraian_pelaksanaan,
                                     progres_persen, jumlah_realisasi, kendala, tanggal_realisasi,
                                     status, verified_by, catatan_verifikator, verified_at)
             VALUES ($1, $2, $3, $4, $5, $6::float8::numeric, $7::float8::numeric, $8, $9,
                     $10::status_hasil, $11, $12, $13)",
        )
        .bind(hasil_id)
        .bind(contoh.rencana_id)
        .bind(contoh.personel_id)
        .bind(satker_demo)
        .bind(contoh.uraian)
        .bind(contoh.progres)
        .bind(contoh.realisasi)
        .bind(contoh.kendala)
        .bind(contoh.tanggal)
        .bind(contoh.status)
        .bind(contoh.verified_by)
        .bind(contoh.catatan)
        .bind(contoh.verified_at)
        .execute(&pool)
        .await?;

        for (index, lampiran) in contoh.lampiran.iter().enumerate() {
            simpan_lampiran_contoh(
                &pool,
                &settings.upload_dir,
                "hasil",
                "bukti",
                satker_demo,
                hasil_id,
                contoh.personel_id,
                uid(&format!("lampiran-hasil-demo-{hasil_id}-{index}")),
                lampiran.nama,
                lampiran.mime,
                lampiran.ext,
                &lampiran.isi,
            )
            .await?;
        }
    }

    // ------------------------------------------------------ contoh output
    struct ContohOutput {
        kode: &'static str,
        hasil_kerja_id: Uuid,
        rencana_id: Uuid,
        personel_id: Uuid,
        jenis: &'static str,
        judul: &'static str,
        uraian: &'static str,
        tanggal: NaiveDate,
        status: &'static str,
        catatan: Option<&'static str>,
        verified_by: Option<Uuid>,
        verified_at: Option<DateTime<Utc>>,
        lampiran: Vec<ContohLampiran>,
    }

    let contoh_output = vec![
        ContohOutput {
            kode: "draft",
            hasil_kerja_id: uid(&format!("hasil-{rencana_demo_1}")),
            rencana_id: rencana_demo_1,
            personel_id: personel_demo,
            jenis: "laporan",
            judul: "Draf Laporan Analisis Gangguan Siber",
            uraian: "Draf laporan analisis gangguan jaringan pada layanan publik.",
            tanggal: awal_bulan + Delta::days(20),
            status: "draft",
            catatan: None,
            verified_by: None,
            verified_at: None,
            lampiran: vec![],
        },
        ContohOutput {
            kode: "dalam-verifikasi",
            hasil_kerja_id: uid(&format!("hasil-{rencana_demo_2}")),
            rencana_id: rencana_demo_2,
            personel_id: personel_demo,
            jenis: "laporan",
            judul: "Laporan Hasil Analisis Gangguan Siber",
            uraian: "Laporan lengkap beserta rekomendasi tindak lanjut.",
            tanggal: awal_bulan + Delta::days(20),
            status: "dalam_verifikasi",
            catatan: None,
            verified_by: None,
            verified_at: None,
            lampiran: vec![
                ContohLampiran {
                    nama: "Laporan-Analisis-Gangguan.pdf",
                    mime: "application/pdf",
                    ext: "pdf",
                    isi: contoh_pdf(
                        "Laporan Analisis Gangguan Siber",
                        "Analisis gangguan jaringan dan rekomendasi tindak lanjut",
                    ),
                },
                ContohLampiran {
                    nama: "Dokumentasi-Analisis.png",
                    mime: "image/png",
                    ext: "png",
                    isi: contoh_png(),
                },
            ],
        },
        ContohOutput {
            kode: "perlu-revisi",
            hasil_kerja_id: uid(&format!("hasil-{rencana_demo_1}")),
            rencana_id: rencana_demo_1,
            personel_id: personel_demo,
            jenis: "dokumen_pendukung",
            judul: "Dokumen Pendukung Berkas Perkara",
            uraian: "Kelengkapan dokumen pendukung berkas perkara tahap 1.",
            tanggal: awal_bulan + Delta::days(18),
            status: "perlu_revisi",
            catatan: Some("Lengkapi tanda tangan pejabat dan unggah berkas terbaru."),
            verified_by: Some(atasan_personel),
            verified_at: Some(wita_to_utc(awal_bulan + Delta::days(19), 13, 0)),
            lampiran: vec![ContohLampiran {
                nama: "Dokumen-Pendukung-Perkara.pdf",
                mime: "application/pdf",
                ext: "pdf",
                isi: contoh_pdf(
                    "Dokumen Pendukung Berkas Perkara",
                    "Ringkasan dokumen pendukung tahap 1",
                ),
            }],
        },
        ContohOutput {
            kode: "personel-lain-dalam-verifikasi",
            hasil_kerja_id: uid(&format!("hasil-{rencana_lain_1}")),
            rencana_id: rencana_lain_1,
            personel_id: personel_lain,
            jenis: "foto",
            judul: "Dokumentasi Barang Bukti",
            uraian: "Foto barang bukti hasil pengungkapan kasus penipuan daring.",
            tanggal: awal_bulan + Delta::days(19),
            status: "dalam_verifikasi",
            catatan: None,
            verified_by: None,
            verified_at: None,
            lampiran: vec![ContohLampiran {
                nama: "Barang-Bukti-Output.png",
                mime: "image/png",
                ext: "png",
                isi: contoh_png(),
            }],
        },
    ];

    for contoh in &contoh_output {
        let output_id = uid(&format!(
            "output-demo-{}-{}",
            contoh.personel_id, contoh.kode
        ));
        let (kelengkapan, kesesuaian, keabsahan, total) = match contoh.status {
            "disetujui" => (100.0, 90.0, 92.0, 94.0),
            "perlu_revisi" => (100.0, 0.0, 0.0, 0.0),
            _ => (0.0, 0.0, 0.0, 0.0),
        };
        let (skor_kelengkapan, skor_kesesuaian, skor_keabsahan, skor_total) =
            if contoh.status == "draft" || contoh.status == "dalam_verifikasi" {
                (None, None, None, None)
            } else {
                (
                    Some(kelengkapan),
                    Some(kesesuaian),
                    Some(keabsahan),
                    Some(total),
                )
            };

        sqlx::query(
            "INSERT INTO output (id, hasil_kerja_id, rencana_id, personel_id, satker_id, jenis_output,
                                judul, uraian, tanggal_hasil, url_eksternal, status,
                                skor_kelengkapan, skor_kesesuaian, skor_keabsahan, skor_total,
                                verified_by, catatan_verifikator, verified_at)
             VALUES ($1, $2, $3, $4, $5, $6::jenis_output, $7, $8, $9, NULL, $10::status_output,
                     $11::float8::numeric, $12::float8::numeric, $13::float8::numeric,
                     $14::float8::numeric, $15, $16, $17)",
        )
        .bind(output_id)
        .bind(contoh.hasil_kerja_id)
        .bind(contoh.rencana_id)
        .bind(contoh.personel_id)
        .bind(satker_demo)
        .bind(contoh.jenis)
        .bind(contoh.judul)
        .bind(contoh.uraian)
        .bind(contoh.tanggal)
        .bind(contoh.status)
        .bind(skor_kelengkapan)
        .bind(skor_kesesuaian)
        .bind(skor_keabsahan)
        .bind(skor_total)
        .bind(contoh.verified_by)
        .bind(contoh.catatan)
        .bind(contoh.verified_at)
        .execute(&pool)
        .await?;

        for (index, lampiran) in contoh.lampiran.iter().enumerate() {
            simpan_lampiran_contoh(
                &pool,
                &settings.upload_dir,
                "output",
                "bukti",
                satker_demo,
                output_id,
                contoh.personel_id,
                uid(&format!("lampiran-output-demo-{output_id}-{index}")),
                lampiran.nama,
                lampiran.mime,
                lampiran.ext,
                &lampiran.isi,
            )
            .await?;
        }
    }

    // Foto profil contoh untuk akun demo Android.
    let foto_key = format!("foto/{personel_demo}/avatar.png");
    let foto_path = settings.upload_dir.join(&foto_key);
    if let Some(parent) = foto_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&foto_path, contoh_png()).await?;
    sqlx::query("UPDATE personel SET foto_path = $2 WHERE id = $1")
        .bind(personel_demo)
        .bind(&foto_key)
        .execute(&pool)
        .await?;

    let total_personel_akun: i64 =
        sqlx::query_scalar("SELECT count(*) FROM personel WHERE jenis_akun = 'personel'")
            .fetch_one(&pool)
            .await?;
    let total_admin: i64 =
        sqlx::query_scalar("SELECT count(*) FROM personel WHERE jenis_akun = 'admin'")
            .fetch_one(&pool)
            .await?;

    println!("\nSeed selesai.");
    println!("  Profil    : {}", profil.label());
    println!("  Polda     : 1");
    println!(
        "  Polres    : {}",
        if profil == ProfilSeed::Vps {
            0
        } else {
            POLRES.len()
        }
    );
    println!("  Satker    : {}", satker_nodes.len());
    println!(
        "  Personel  : {} ({} reguler + {} atasan + 4 pimpinan)",
        total_personel_akun,
        personel_ids.len(),
        satker_nodes.len()
    );
    println!("  Admin     : {total_admin} akun (login dashboard)");
    println!("  Periode   : {nama_periode} ({awal_bulan} s/d {akhir_bulan})");
    println!("  Absensi   : {awal_bulan} s/d {batas_absensi}");
    println!("  Jadwal PH : {}", jadwal_ph_rows.len());
    println!("  Hari libur: {libur_tetap_len}");
    println!("  Rencana demo: 1 draft, 1 diajukan, 1 perlu revisi (NRP 85000001)");
    println!("  Rencana atasan: 1 diajukan dari NRP 000100 ke pimpinan 000002");
    println!("  Rencana personel lain: 1 diajukan dengan lampiran PDF + PNG");
    println!("  Lampiran demo: PDF dan PNG ditulis ke folder upload");
    println!("  Hasil demo: 1 draft, 1 diajukan, 1 perlu revisi, 1 dari personel lain");
    println!(
        "  Status contoh: rencana draft/perlu revisi/disetujui, hasil draft/perlu revisi/terverifikasi, output draft/dalam verifikasi/disetujui"
    );
    println!("\nAkun demo:");
    println!("  Admin global        : superuser / Superuser#2026");
    println!("  Admin Polda         : admin_polda / AdminPolda#2026");
    println!("  Admin Satker Polda  : admin_satker_polda / AdminSatker#2026");
    if profil == ProfilSeed::Development {
        println!("  Admin Polres        : admin_polres / AdminPolres#2026");
        println!("  Admin Satker Polres : admin_satker_polres / AdminSatker#2026");
    }
    println!("  Pimpinan Polda      : NRP 000002 / Pimpinan#2026");
    if profil == ProfilSeed::Development {
        println!("  Pimpinan Polres     : NRP 000022 / PimpinanPolres#2026");
    }
    println!("  Pimpinan Satker     : NRP 000012 / PimpinanSatker#2026");
    if profil == ProfilSeed::Development {
        println!("  Pimpinan Satker Polres: NRP 000023 / PimpinanSatker#2026");
    }
    println!("  Atasan              : NRP 000100 / Atasan#2026");
    println!(
        "  Personel            : NRP {} / Polri#2026",
        personel_ids[0].2
    );

    Ok(())
}
