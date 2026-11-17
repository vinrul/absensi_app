use std::sync::RwLock;

use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, Utc};

pub const WITA_OFFSET_SECS: i32 = 8 * 3600;

/// Override waktu hanya untuk pengembangan. `None` berarti memakai jam sistem.
static OVERRIDE_UTC: RwLock<Option<DateTime<Utc>>> = RwLock::new(None);

fn baca_override() -> Option<DateTime<Utc>> {
    *OVERRIDE_UTC
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn sekarang_utc() -> DateTime<Utc> {
    baca_override().unwrap_or_else(Utc::now)
}

pub fn sekarang_wita() -> DateTime<FixedOffset> {
    ke_wita(sekarang_utc())
}

pub fn ke_wita(ts: DateTime<Utc>) -> DateTime<FixedOffset> {
    ts.with_timezone(&FixedOffset::east_opt(WITA_OFFSET_SECS).expect("offset WITA valid"))
}

pub fn hari_ini_wita() -> NaiveDate {
    sekarang_wita().date_naive()
}

/// Mengaktifkan/mematikan waktu palsu. Pemanggil wajib memastikan mode development.
pub fn set_override(nilai: Option<DateTime<Utc>>) {
    *OVERRIDE_UTC
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = nilai;
}

pub fn override_aktif() -> Option<DateTime<Utc>> {
    baca_override()
}

pub fn awal_bulan(tanggal: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(tanggal.year(), tanggal.month(), 1).unwrap_or(tanggal)
}

pub fn akhir_bulan(tanggal: NaiveDate) -> NaiveDate {
    let (tahun, bulan) = if tanggal.month() == 12 {
        (tanggal.year() + 1, 1)
    } else {
        (tanggal.year(), tanggal.month() + 1)
    };
    NaiveDate::from_ymd_opt(tahun, bulan, 1)
        .and_then(|awal_bulan_berikut| awal_bulan_berikut.pred_opt())
        .unwrap_or(tanggal)
}

/// Rentang tanggal penuh untuk bulan yang memuat `tanggal`.
pub fn rentang_bulan(tanggal: NaiveDate) -> (NaiveDate, NaiveDate) {
    (awal_bulan(tanggal), akhir_bulan(tanggal))
}

pub fn nama_bulan(bulan: u32) -> &'static str {
    match bulan {
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
    }
}
