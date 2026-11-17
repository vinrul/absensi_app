use chrono::{Datelike, NaiveDate, NaiveTime};

use crate::domain::enums::{IndikatorKinerja, KategoriKinerja, StatusKehadiran};

fn bulatkan(nilai: f64) -> f64 {
    (nilai * 100.0).round() / 100.0
}

fn clamp01(nilai: f64) -> f64 {
    if !nilai.is_finite() {
        return 0.0;
    }
    nilai.clamp(0.0, 100.0)
}

/// Jumlah hari kerja efektif dalam rentang tanggal (ISO: 1=Senin .. 7=Minggu).
pub fn hari_efektif(mulai: NaiveDate, selesai: NaiveDate, hari_kerja: &[i16]) -> i64 {
    let mut jumlah = 0;
    let mut tanggal = mulai;
    while tanggal <= selesai {
        let iso = tanggal.weekday().num_days_from_monday() as i16 + 1;
        if hari_kerja.contains(&iso) {
            jumlah += 1;
        }
        tanggal += chrono::Duration::days(1);
    }
    jumlah
}

fn menit_terlambat(aktual: NaiveTime, standar: NaiveTime) -> i64 {
    let selisih = aktual.signed_duration_since(standar).num_minutes();
    selisih.max(0)
}

/// Skor satu hari absensi.
///
/// `None` berarti hari tersebut netral (PH/cuti/izin/sakit) sehingga
/// dikeluarkan dari penyebut rata-rata.
pub fn skor_harian(
    status: StatusKehadiran,
    jam_masuk: Option<NaiveTime>,
    jam_pulang: Option<NaiveTime>,
    standar_masuk: NaiveTime,
    standar_pulang: NaiveTime,
) -> Option<f64> {
    if status == StatusKehadiran::TanpaKeterangan {
        return Some(0.0);
    }
    if status.netral() {
        return None;
    }
    if !status.dihitung_hadir() {
        return None;
    }

    let Some(masuk) = jam_masuk else {
        return Some(0.0);
    };

    let telat = menit_terlambat(masuk, standar_masuk);
    let penalti_telat = ((telat as f64) / 10.0).floor().min(20.0);

    let (dasar, penalti_pulang) = match jam_pulang {
        Some(pulang) => {
            let cepat = menit_terlambat(standar_pulang, pulang);
            let penalti = ((cepat as f64) / 10.0).floor().min(20.0);
            (100.0, penalti)
        }
        // Tidak absen pulang: hari dihitung separuh.
        None => (50.0, 0.0),
    };

    Some(bulatkan(clamp01(dasar - penalti_telat - penalti_pulang)))
}

/// Rata-rata skor harian; hari netral tidak masuk penyebut.
pub fn skor_absensi(harian: &[f64]) -> f64 {
    if harian.is_empty() {
        return 0.0;
    }
    let total: f64 = harian.iter().sum();
    bulatkan(total / harian.len() as f64)
}

/// Rencana kerja: ketepatan 40%, kelengkapan 30%, kualitas 30%.
pub fn skor_rencana(ketepatan: f64, kelengkapan: f64, kualitas: f64) -> f64 {
    bulatkan(clamp01(
        0.4 * ketepatan + 0.3 * kelengkapan + 0.3 * kualitas,
    ))
}

/// Ketepatan: 100 bila seluruh rencana disetujui sebelum tanggal mulai,
/// berkurang 10 poin per hari keterlambatan.
pub fn komponen_ketepatan(hari_terlambat: i64) -> f64 {
    clamp01(100.0 - (hari_terlambat.max(0) as f64) * 10.0)
}

/// Kualitas: 100 − 15 per revisi, maksimal turun ke 0.
pub fn komponen_kualitas(jumlah_revisi: i32, ada_ditolak: bool) -> f64 {
    let dasar = clamp01(100.0 - f64::from(jumlah_revisi) * 15.0);
    if ada_ditolak { dasar.min(60.0) } else { dasar }
}

pub fn komponen_kuantitas(realisasi: Option<f64>, target: Option<f64>) -> f64 {
    match (realisasi, target) {
        (Some(realisasi), Some(target)) if target > 0.0 => clamp01(realisasi / target * 100.0),
        _ => 0.0,
    }
}

/// Waktu: 100 bila selesai ≤ target, −5 per hari keterlambatan.
pub fn komponen_waktu(tanggal_realisasi: NaiveDate, target: Option<NaiveDate>) -> f64 {
    match target {
        Some(target) => {
            let terlambat = (tanggal_realisasi - target).num_days().max(0);
            clamp01(100.0 - terlambat as f64 * 5.0)
        }
        None => 100.0,
    }
}

/// Hasil kerja: kuantitas 40%, kualitas 30%, waktu 30%.
pub fn skor_hasil(kuantitas: f64, kualitas: f64, waktu: f64) -> (f64, f64, f64, f64) {
    let kuantitas = clamp01(kuantitas);
    let kualitas = clamp01(kualitas);
    let waktu = clamp01(waktu);
    let total = bulatkan(0.4 * kuantitas + 0.3 * kualitas + 0.3 * waktu);
    (
        bulatkan(kuantitas),
        bulatkan(kualitas),
        bulatkan(waktu),
        total,
    )
}

/// Output: kelengkapan 40%, kesesuaian 30%, keabsahan 30%.
pub fn skor_output(kelengkapan: f64, kesesuaian: f64, keabsahan: f64) -> (f64, f64, f64, f64) {
    let kelengkapan = clamp01(kelengkapan);
    let kesesuaian = clamp01(kesesuaian);
    let keabsahan = clamp01(keabsahan);
    let total = bulatkan(0.4 * kelengkapan + 0.3 * kesesuaian + 0.3 * keabsahan);
    (
        bulatkan(kelengkapan),
        bulatkan(kesesuaian),
        bulatkan(keabsahan),
        total,
    )
}

/// Nilai akhir berbobot. Bobot dianggap persen (total 100).
pub fn nilai_akhir(bobot: &[(IndikatorKinerja, f64)], skor: &[(IndikatorKinerja, f64)]) -> f64 {
    let total_bobot: f64 = bobot.iter().map(|(_, b)| *b).sum();
    if total_bobot <= 0.0 {
        return 0.0;
    }

    let mut jumlah = 0.0;
    for (indikator, beban) in bobot {
        let nilai = skor
            .iter()
            .find(|(lain, _)| lain == indikator)
            .map(|(_, nilai)| *nilai)
            .unwrap_or(0.0);
        jumlah += (beban / total_bobot) * nilai;
    }

    bulatkan(jumlah)
}

pub fn kategori(nilai: f64) -> KategoriKinerja {
    KategoriKinerja::dari_nilai(nilai)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(h: u32, m: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, 0).unwrap()
    }

    fn tanggal(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn hadir_tepat_waktu_dapat_seratus() {
        let skor = skor_harian(
            StatusKehadiran::DinasKantor,
            Some(t(7, 50)),
            Some(t(16, 5)),
            t(8, 0),
            t(16, 0),
        );
        assert_eq!(skor, Some(100.0));
    }

    #[test]
    fn telat_25_menit_berkurang_2_poin() {
        let skor = skor_harian(
            StatusKehadiran::DinasKantor,
            Some(t(8, 25)),
            Some(t(16, 5)),
            t(8, 0),
            t(16, 0),
        );
        assert_eq!(skor, Some(98.0));
    }

    #[test]
    fn pulang_cepat_15_menit_berkurang_1_poin() {
        let skor = skor_harian(
            StatusKehadiran::Piket,
            Some(t(7, 55)),
            Some(t(15, 45)),
            t(8, 0),
            t(16, 0),
        );
        assert_eq!(skor, Some(99.0));
    }

    #[test]
    fn tidak_absen_pulang_setengah() {
        let skor = skor_harian(
            StatusKehadiran::DinasKantor,
            Some(t(7, 55)),
            None,
            t(8, 0),
            t(16, 0),
        );
        assert_eq!(skor, Some(50.0));
    }

    #[test]
    fn status_netral_dikeluarkan() {
        assert_eq!(
            skor_harian(StatusKehadiran::Cuti, None, None, t(8, 0), t(16, 0)),
            None
        );
        assert_eq!(
            skor_harian(
                StatusKehadiran::LiburNasional,
                None,
                None,
                t(8, 0),
                t(16, 0)
            ),
            None
        );
    }

    #[test]
    fn tanpa_keterangan_nol() {
        assert_eq!(
            skor_harian(
                StatusKehadiran::TanpaKeterangan,
                None,
                None,
                t(8, 0),
                t(16, 0)
            ),
            Some(0.0)
        );
    }

    #[test]
    fn penalti_telat_dibatasi_20() {
        // 08:00 -> 11:40 = 220 menit telat => 22 poin, dibatasi 20.
        let skor = skor_harian(
            StatusKehadiran::DinasKantor,
            Some(t(11, 40)),
            Some(t(16, 5)),
            t(8, 0),
            t(16, 0),
        );
        assert_eq!(skor, Some(80.0));
    }

    #[test]
    fn telat_tiga_jam_penalti_18() {
        // 08:00 -> 11:00 = 180 menit => 18 poin.
        let skor = skor_harian(
            StatusKehadiran::DinasKantor,
            Some(t(11, 0)),
            Some(t(16, 5)),
            t(8, 0),
            t(16, 0),
        );
        assert_eq!(skor, Some(82.0));
    }

    #[test]
    fn rata_rata_mengabaikan_hari_netral() {
        assert_eq!(skor_absensi(&[100.0, 90.0, 0.0]), 63.33);
        assert_eq!(skor_absensi(&[]), 0.0);
    }

    #[test]
    fn rencana_berbobot() {
        // 0.4*100 + 0.3*100 + 0.3*85 = 95.5
        assert_eq!(skor_rencana(100.0, 100.0, 85.0), 95.5);
    }

    #[test]
    fn kualitas_turun_per_revisi() {
        assert_eq!(komponen_kualitas(1, false), 85.0);
        assert_eq!(komponen_kualitas(3, false), 55.0);
        assert_eq!(komponen_kualitas(0, true), 60.0);
        assert_eq!(komponen_kualitas(10, false), 0.0);
    }

    #[test]
    fn kuantitas_melebihi_target_dibatasi_seratus() {
        assert_eq!(komponen_kuantitas(Some(12.0), Some(10.0)), 100.0);
        assert_eq!(komponen_kuantitas(Some(5.0), Some(10.0)), 50.0);
        assert_eq!(komponen_kuantitas(None, Some(10.0)), 0.0);
    }

    #[test]
    fn waktu_turun_lima_per_hari() {
        assert_eq!(
            komponen_waktu(tanggal(2026, 11, 10), Some(tanggal(2026, 11, 10))),
            100.0
        );
        assert_eq!(
            komponen_waktu(tanggal(2026, 11, 13), Some(tanggal(2026, 11, 10))),
            85.0
        );
        assert_eq!(komponen_waktu(tanggal(2026, 11, 10), None), 100.0);
    }

    #[test]
    fn hasil_kerja_berbobot() {
        // 0.4*90 + 0.3*80 + 0.3*100 = 90.0
        assert_eq!(skor_hasil(90.0, 80.0, 100.0).3, 90.0);
    }

    #[test]
    fn output_berbobot() {
        // 0.4*100 + 0.3*90 + 0.3*80 = 91.0
        assert_eq!(skor_output(100.0, 90.0, 80.0).3, 91.0);
    }

    #[test]
    fn nilai_akhir_sesuai_bobot() {
        let bobot = [
            (IndikatorKinerja::Absensi, 20.0),
            (IndikatorKinerja::RencanaKerja, 20.0),
            (IndikatorKinerja::HasilKerja, 30.0),
            (IndikatorKinerja::Output, 30.0),
        ];
        let skor = [
            (IndikatorKinerja::Absensi, 100.0),
            (IndikatorKinerja::RencanaKerja, 100.0),
            (IndikatorKinerja::HasilKerja, 95.0),
            (IndikatorKinerja::Output, 95.0),
        ];
        assert_eq!(nilai_akhir(&bobot, &skor), 97.0);
    }

    #[test]
    fn kategori_sesuai_ambang() {
        assert_eq!(kategori(95.0), KategoriKinerja::SangatBaik);
        assert_eq!(kategori(94.99), KategoriKinerja::Baik);
        assert_eq!(kategori(90.0), KategoriKinerja::Baik);
        assert_eq!(kategori(89.99), KategoriKinerja::Cukup);
        assert_eq!(kategori(80.0), KategoriKinerja::Cukup);
        assert_eq!(kategori(79.99), KategoriKinerja::PerluPembinaan);
    }

    #[test]
    fn hari_efektif_senin_jumat() {
        // November 2026: 1 Nov Minggu, 30 Nov Senin
        assert_eq!(
            hari_efektif(
                tanggal(2026, 11, 1),
                tanggal(2026, 11, 30),
                &[1, 2, 3, 4, 5]
            ),
            21
        );
    }
}
