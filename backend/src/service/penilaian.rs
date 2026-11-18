use std::collections::{HashMap, HashSet};

use chrono::{NaiveDate, NaiveTime};
use uuid::Uuid;

use crate::domain::enums::{
    BerkasOwner, IndikatorKinerja, KategoriKinerja, StatusHasil, StatusOutput, StatusRencana,
};
use crate::error::AppResult;
use crate::repo::{absensi, hasil, lampiran, libur, penilaian, periode, personel, rencana, satker};
use crate::service::scoring;
use crate::service::waktu::ke_wita;
use crate::state::AppState;

#[derive(Debug, serde::Serialize)]
pub struct HasilHitung {
    pub periode_id: Uuid,
    pub jumlah_personel: usize,
    pub rata_rata_nilai: f64,
}

struct StandarSatker {
    hari_kerja: Vec<i16>,
    jam_masuk: NaiveTime,
    jam_pulang: NaiveTime,
    jam_pulang_sabtu: Option<NaiveTime>,
}

/// (jam masuk, jam pulang, status kehadiran) untuk satu hari.
type CatatanHarian = (
    Option<NaiveTime>,
    Option<NaiveTime>,
    Option<crate::domain::enums::StatusKehadiran>,
);

/// Menghitung (atau menghitung ulang) penilaian seluruh personel pada satu periode.
pub async fn hitung_periode(
    state: &AppState,
    periode_id: Uuid,
    satker_ids: &[Uuid],
    unit: Option<&str>,
    _oleh: Uuid,
) -> AppResult<HasilHitung> {
    let periode_row = periode::find(&state.pool, periode_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Periode tidak ditemukan".into()))?;

    let bobot = periode::bobot(&state.pool, periode_id).await?;

    let satker_rows = satker::list(&state.pool).await?;
    let standar: HashMap<Uuid, StandarSatker> = satker_rows
        .into_iter()
        .map(|row| {
            (
                row.id,
                StandarSatker {
                    hari_kerja: row.hari_kerja,
                    jam_masuk: row.jam_masuk,
                    jam_pulang: row.jam_pulang,
                    jam_pulang_sabtu: row.jam_pulang_sabtu,
                },
            )
        })
        .collect();

    // Libur nasional berlaku untuk semua satker; libur khusus hanya satkernya.
    let libur_rows = libur::tanggal_aktif(
        &state.pool,
        periode_row.tanggal_mulai,
        periode_row.tanggal_selesai,
    )
    .await?;
    let libur_global: HashSet<NaiveDate> = libur_rows
        .iter()
        .filter(|row| row.satker_id.is_none())
        .map(|row| row.tanggal)
        .collect();
    let mut libur_satker: HashMap<Uuid, HashSet<NaiveDate>> = HashMap::new();
    for row in &libur_rows {
        if let Some(satker_id) = row.satker_id {
            libur_satker
                .entry(satker_id)
                .or_default()
                .insert(row.tanggal);
        }
    }

    let (personel_rows, _) =
        personel::list(&state.pool, satker_ids, unit, None, None, 1000, 0).await?;

    let mut jumlah = 0usize;
    let mut total_nilai = 0.0f64;

    for p in &personel_rows {
        let standar_satker = standar.get(&p.satker_id);
        let jam_masuk = standar_satker
            .map(|s| s.jam_masuk)
            .unwrap_or_else(|| NaiveTime::from_hms_opt(8, 0, 0).unwrap_or_default());
        let jam_pulang = standar_satker
            .map(|s| s.jam_pulang)
            .unwrap_or_else(|| NaiveTime::from_hms_opt(16, 0, 0).unwrap_or_default());
        let jam_pulang_sabtu = standar_satker.and_then(|s| s.jam_pulang_sabtu);

        // ---------------------------------------------------------- absensi
        let absensi_rows = absensi::list_personel(
            &state.pool,
            p.id,
            periode_row.tanggal_mulai,
            periode_row.tanggal_selesai,
        )
        .await?;

        // Hari ber-jadwal strong point tetap dihitung sebagai absensi kantor. Keterlambatan
        // kantor hanya diabaikan bila personel benar-benar check-in strong point pada hari itu.
        let ph_jadwal: HashSet<chrono::NaiveDate> = crate::repo::ph::tanggal_personel(
            &state.pool,
            p.id,
            periode_row.tanggal_mulai,
            periode_row.tanggal_selesai,
        )
        .await?
        .into_iter()
        .collect();
        let ph_dilakukan: HashSet<chrono::NaiveDate> = crate::repo::ph::tanggal_personel_hadir(
            &state.pool,
            p.id,
            periode_row.tanggal_mulai,
            periode_row.tanggal_selesai,
        )
        .await?
        .into_iter()
        .collect();

        let mut per_hari: HashMap<chrono::NaiveDate, CatatanHarian> = HashMap::new();

        for row in &absensi_rows {
            let entry = per_hari.entry(row.tanggal).or_insert((None, None, None));
            match row.jenis {
                crate::domain::enums::AbsensiJenis::Masuk => {
                    entry.0 = Some(row.jam);
                    entry.2 = Some(row.status_kehadiran);
                }
                crate::domain::enums::AbsensiJenis::Pulang => entry.1 = Some(row.jam),
            }
        }

        let mut skor_harian = Vec::new();
        let mut hari_tercatat = 0i64;
        let mut hari_libur = 0i64;
        let libur_satker_ini = libur_satker.get(&p.satker_id);

        // Hari kerja efektif yang sudah lewat: periode tertutup dihitung penuh,
        // periode berjalan hanya sampai kemarin agar tidak menghukum hari yang
        // belum terjadi. Hari kerja tanpa catatan absensi dihitung 0 (tanpa keterangan).
        let batas = if periode_row.status == crate::domain::enums::PeriodeStatus::Closed {
            periode_row.tanggal_selesai
        } else {
            let kemarin = crate::service::waktu::hari_ini_wita() - chrono::Duration::days(1);
            kemarin.min(periode_row.tanggal_selesai)
        };

        let hari_kerja = standar_satker
            .map(|s| s.hari_kerja.clone())
            .unwrap_or_else(|| vec![1, 2, 3, 4, 5]);

        let mut tanggal = periode_row.tanggal_mulai;
        while tanggal <= batas {
            let iso = chrono::Datelike::weekday(&tanggal).num_days_from_monday() as i16 + 1;
            let hari_kerja_ini = hari_kerja.contains(&iso);

            if hari_kerja_ini
                && (libur_global.contains(&tanggal)
                    || libur_satker_ini.is_some_and(|set| set.contains(&tanggal)))
            {
                hari_libur += 1;
                tanggal += chrono::Duration::days(1);
                continue;
            }

            if hari_kerja_ini {
                match per_hari.get(&tanggal) {
                    Some((masuk, pulang, Some(status))) => {
                        hari_tercatat += 1;
                        let standar_pulang_hari = if iso == 6 {
                            jam_pulang_sabtu.unwrap_or(jam_pulang)
                        } else {
                            jam_pulang
                        };
                        let nilai = if ph_dilakukan.contains(&tanggal) {
                            scoring::skor_harian_dengan_strong_point(
                                *status,
                                *masuk,
                                *pulang,
                                jam_masuk,
                                standar_pulang_hari,
                            )
                        } else {
                            scoring::skor_harian(
                                *status,
                                *masuk,
                                *pulang,
                                jam_masuk,
                                standar_pulang_hari,
                            )
                        };
                        if let Some(nilai) = nilai {
                            skor_harian.push(nilai);
                        }
                    }
                    _ => {
                        // Tidak ada catatan pada hari kerja: dianggap tanpa keterangan.
                        skor_harian.push(0.0);
                    }
                }
            }

            tanggal += chrono::Duration::days(1);
        }

        let skor_absensi = scoring::skor_absensi(&skor_harian);

        // ------------------------------------------------------ rencana kerja
        let rencana_rows = rencana::by_personel(&state.pool, p.id, periode_id).await?;

        let total_rencana = rencana_rows.len();
        let mut lengkap = 0usize;
        let mut total_revisi = 0i32;
        let mut ada_ditolak = false;
        let mut hari_terlambat = 0i64;

        for r in &rencana_rows {
            if r.status == StatusRencana::Ditolak {
                ada_ditolak = true;
            }
            total_revisi += r.jumlah_revisi;

            if r.status == StatusRencana::Disetujui
                && let Some(diputuskan) = r.diputuskan_at
            {
                let tanggal = ke_wita(diputuskan).date_naive();
                let terlambat = (tanggal - r.tanggal_mulai).num_days();
                hari_terlambat = hari_terlambat.max(terlambat.max(0));
            }

            let punya_dasar = r.dasar_jenis.is_some() && r.dasar_nomor.is_some();
            let punya_target = r.target_kuantitas.is_some() && r.target_waktu.is_some();
            let jumlah_lampiran = lampiran::count(&state.pool, BerkasOwner::Rencana, r.id).await?;

            if punya_dasar && punya_target && jumlah_lampiran > 0 {
                lengkap += 1;
            }
        }

        let kelengkapan = if total_rencana > 0 {
            (lengkap as f64 / total_rencana as f64) * 100.0
        } else {
            0.0
        };
        let ketepatan = scoring::komponen_ketepatan(hari_terlambat);
        let kualitas = scoring::komponen_kualitas(total_revisi, ada_ditolak);
        let skor_rencana = if total_rencana > 0 {
            scoring::skor_rencana(ketepatan, kelengkapan, kualitas)
        } else {
            0.0
        };

        // --------------------------------------------------------- hasil kerja
        let hasil_rows = hasil::by_personel(&state.pool, p.id, periode_id).await?;
        let terverifikasi: Vec<f64> = hasil_rows
            .iter()
            .filter(|row| row.status == StatusHasil::Terverifikasi)
            .filter_map(|row| row.skor_total)
            .collect();

        let skor_hasil = if terverifikasi.is_empty() {
            0.0
        } else {
            let total: f64 = terverifikasi.iter().sum();
            (total / terverifikasi.len() as f64 * 100.0).round() / 100.0
        };

        // -------------------------------------------------------------- output
        let output_rows = crate::repo::output::by_personel(&state.pool, p.id, periode_id).await?;

        let disetujui: Vec<f64> = output_rows
            .iter()
            .filter(|row| row.status == StatusOutput::Disetujui)
            .filter_map(|row| row.skor_total)
            .collect();

        let perlu_revisi = output_rows
            .iter()
            .filter(|row| row.status == StatusOutput::PerluRevisi)
            .count() as f64;

        let dasar_output = if disetujui.is_empty() {
            0.0
        } else {
            disetujui.iter().sum::<f64>() / disetujui.len() as f64
        };

        let skor_output = (dasar_output - perlu_revisi * 20.0).max(0.0);
        let skor_output = (skor_output * 100.0).round() / 100.0;

        // ------------------------------------------------------------- akhir
        let skor_indikator = [
            (IndikatorKinerja::Absensi, skor_absensi),
            (IndikatorKinerja::RencanaKerja, skor_rencana),
            (IndikatorKinerja::HasilKerja, skor_hasil),
            (IndikatorKinerja::Output, skor_output),
        ];

        let nilai = scoring::nilai_akhir(&bobot, &skor_indikator);
        let kategori = KategoriKinerja::dari_nilai(nilai);

        let penilaian_id = penilaian::upsert(
            &state.pool,
            Uuid::now_v7(),
            p.id,
            periode_id,
            p.satker_id,
            skor_absensi,
            skor_rencana,
            skor_hasil,
            skor_output,
            nilai,
            kategori,
        )
        .await?;

        penilaian::simpan_detail(
            &state.pool,
            penilaian_id,
            IndikatorKinerja::Absensi,
            skor_absensi,
            serde_json::json!({
                "hari_tercatat": hari_tercatat,
                "hari_dihitung": skor_harian.len(),
                "hari_ph": ph_jadwal.len(),
                "hari_ph_dilakukan": ph_dilakukan.len(),
                "hari_libur": hari_libur,
                "standar_masuk": jam_masuk.format("%H:%M").to_string(),
                "standar_pulang": jam_pulang.format("%H:%M").to_string(),
                "standar_pulang_sabtu": jam_pulang_sabtu
                    .map(|jam| jam.format("%H:%M").to_string()),
            }),
        )
        .await?;

        penilaian::simpan_detail(
            &state.pool,
            penilaian_id,
            IndikatorKinerja::RencanaKerja,
            skor_rencana,
            serde_json::json!({
                "total_rencana": total_rencana,
                "lengkap": lengkap,
                "hari_terlambat": hari_terlambat,
                "total_revisi": total_revisi,
                "ada_ditolak": ada_ditolak,
                "ketepatan": ketepatan,
                "kelengkapan": kelengkapan,
                "kualitas": kualitas,
            }),
        )
        .await?;

        penilaian::simpan_detail(
            &state.pool,
            penilaian_id,
            IndikatorKinerja::HasilKerja,
            skor_hasil,
            serde_json::json!({
                "jumlah_terverifikasi": terverifikasi.len(),
                "skor": terverifikasi,
            }),
        )
        .await?;

        penilaian::simpan_detail(
            &state.pool,
            penilaian_id,
            IndikatorKinerja::Output,
            skor_output,
            serde_json::json!({
                "jumlah_disetujui": disetujui.len(),
                "perlu_revisi": perlu_revisi,
            }),
        )
        .await?;

        jumlah += 1;
        total_nilai += nilai;
    }

    let rata_rata = if jumlah > 0 {
        (total_nilai / jumlah as f64 * 100.0).round() / 100.0
    } else {
        0.0
    };

    Ok(HasilHitung {
        periode_id,
        jumlah_personel: jumlah,
        rata_rata_nilai: rata_rata,
    })
}
