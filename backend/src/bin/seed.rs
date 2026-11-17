//! Seed data demo: struktur Polda → Polres → Satker, role ber-scope,
//! Strong Point, jadwal PH, serta aktivitas satu bulan.
//!
//! Pemakaian:
//!   cargo run --bin seed              # isi jika database masih kosong
//!   cargo run --bin seed -- --reset   # hapus semua data lalu isi ulang

use std::collections::{HashMap, HashSet};
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
    "BRIPDA", "BRIPTU", "BRIPKA", "AIPDA", "AIPTU", "IPDA", "IPTU", "AKP",
];

const JUMLAH_PERSONEL: usize = 120;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let settings = Settings::from_env().map_err(|err| format!("konfigurasi: {err}"))?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&settings.database_url)
        .await?;

    let reset = std::env::args().any(|arg| arg == "--reset");

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
        .bind(uid(node.kode))
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
    for index in 0..JUMLAH_PERSONEL {
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
        for (slot, hari) in [5u32, 15, 25].into_iter().enumerate() {
            if hari > akhir_bulan.day() {
                continue;
            }
            let tanggal = NaiveDate::from_ymd_opt(awal_bulan.year(), awal_bulan.month(), hari)
                .expect("tanggal PH valid");
            let id = uid(&format!("jadwal-ph-{satker_id}-{hari}"));
            let jam_mulai = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
            let jam_selesai = NaiveTime::from_hms_opt(16, 0, 0).unwrap();
            sqlx::query(
                "INSERT INTO jadwal_ph (id, satker_id, strong_point_id, tanggal, jam_mulai,
                                        jam_selesai, catatan, created_by)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(id)
            .bind(satker_id)
            .bind(strong_point_id)
            .bind(tanggal)
            .bind(jam_mulai)
            .bind(jam_selesai)
            .bind(format!("Jadwal PH {}", node.nama))
            .bind(uid("admin-polda"))
            .execute(&pool)
            .await?;

            for offset in 0..2 {
                let personel_id = orang[(satker_index + slot + offset) % orang.len()];
                sqlx::query(
                    "INSERT INTO jadwal_ph_personel (jadwal_ph_id, personel_id, peran)
                     VALUES ($1, $2, $3)",
                )
                .bind(id)
                .bind(personel_id)
                .bind(if offset == 0 { "Komandan" } else { "Anggota" })
                .execute(&pool)
                .await?;
                ph_dates.entry(personel_id).or_default().insert(tanggal);
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

    // -------------------------------------------------------------- absensi
    let mut tanggal = awal_bulan;
    while tanggal <= akhir_bulan.min(today - Delta::days(1)) {
        let iso_weekday = tanggal.weekday().num_days_from_monday() + 1;
        if iso_weekday <= 5 {
            for (index, (personel_id, satker_id, _nrp)) in personel_ids.iter().enumerate() {
                if ph_dates
                    .get(personel_id)
                    .is_some_and(|set| set.contains(&tanggal))
                {
                    continue;
                }
                let pola = (index * 13 + tanggal.day() as usize * 7) % 100;
                let (status, jam_masuk) = if pola < 80 {
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

    // Absensi PH untuk jadwal yang sudah lewat.
    for (jadwal_id, satker_id, strong_point_id, tanggal, jam_mulai, jam_selesai) in &jadwal_ph_rows
    {
        if *tanggal >= today {
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
            for (jenis, jam) in [
                ("masuk", *jam_mulai + Delta::minutes(5)),
                ("pulang", *jam_selesai + Delta::minutes(5)),
            ] {
                let jam = NaiveTime::from_num_seconds_from_midnight_opt(
                    jam.num_seconds_from_midnight(),
                    0,
                )
                .unwrap_or(*jam_mulai);
                let captured = wita_to_utc(*tanggal, jam.hour(), jam.minute());
                sqlx::query(
                    "INSERT INTO absensi_ph (
                         id, client_uuid, jadwal_ph_id, personel_id, strong_point_id, satker_id,
                         jenis, captured_at, jam, latitude, longitude, akurasi_m, jarak_m,
                         dalam_radius, is_offline, is_mock_suspect, perlu_verifikasi
                     )
                     VALUES ($1, $2, $3, $4, $5, $6, $7::absensi_jenis, $8, $9,
                             $10::float8::numeric, $11::float8::numeric, 8, 2, true, false, false, false)",
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

    let total_personel_akun: i64 =
        sqlx::query_scalar("SELECT count(*) FROM personel WHERE jenis_akun = 'personel'")
            .fetch_one(&pool)
            .await?;
    let total_admin: i64 =
        sqlx::query_scalar("SELECT count(*) FROM personel WHERE jenis_akun = 'admin'")
            .fetch_one(&pool)
            .await?;

    println!("\nSeed selesai.");
    println!("  Polda     : 1");
    println!("  Polres    : {}", POLRES.len());
    println!("  Satker    : {}", satker_nodes.len());
    println!(
        "  Personel  : {} ({} reguler + {} atasan + 4 pimpinan)",
        total_personel_akun,
        personel_ids.len(),
        satker_nodes.len()
    );
    println!("  Admin     : {total_admin} akun (login dashboard)");
    println!("  Periode   : {nama_periode} ({awal_bulan} s/d {akhir_bulan})");
    println!("  Jadwal PH : {}", jadwal_ph_rows.len());
    println!("  Hari libur: {libur_tetap_len}");
    println!("\nAkun demo:");
    println!("  Admin global        : superuser / Superuser#2026");
    println!("  Admin Polda         : admin_polda / AdminPolda#2026");
    println!("  Admin Satker Polda  : admin_satker_polda / AdminSatker#2026");
    println!("  Admin Polres        : admin_polres / AdminPolres#2026");
    println!("  Admin Satker Polres : admin_satker_polres / AdminSatker#2026");
    println!("  Pimpinan Polda      : NRP 000002 / Pimpinan#2026");
    println!("  Pimpinan Polres     : NRP 000022 / PimpinanPolres#2026");
    println!("  Pimpinan Satker     : NRP 000012 / PimpinanSatker#2026");
    println!("  Pimpinan Satker Polres: NRP 000023 / PimpinanSatker#2026");
    println!("  Atasan              : NRP 000100 / Atasan#2026");
    println!(
        "  Personel            : NRP {} / Polri#2026",
        personel_ids[0].2
    );

    Ok(())
}
