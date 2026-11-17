# Aturan Perhitungan Nilai Kinerja

Dokumen ini merinci implementasi `backend/src/service/scoring.rs` dan
`backend/src/service/penilaian.rs`. Semua bobot dibaca dari tabel
`bobot_indikator` per periode (default 20/20/30/30).

## 1. Absensi (bobot 20%)

Perhitungan dilakukan per hari kerja efektif dalam periode. Akun dengan
`jenis_akun='admin'` tidak dihitung karena admin adalah pengelola sistem, bukan
personel yang dinilai.

| Kondisi | Skor harian |
|---|---|
| Hadir (dinas kantor / dinas luar / PH / piket) dalam radius | 100 |
| Telat | −1 per 10 menit (maksimal −20) |
| Pulang cepat | −1 per 10 menit (maksimal −20) |
| Tidak absen pulang | 50 |
| Tanpa keterangan | 0 |
| Libur nasional, cuti, izin, sakit | netral (dikeluarkan dari penyebut) |

- Hari kerja diambil dari `satker.hari_kerja` (default Senin–Jumat).
- **Hari libur pada kalender `hari_libur` dikeluarkan dari perhitungan** (global maupun
  khusus satker), tanpa perlu absensi berstatus `libur_nasional`. Jumlahnya disimpan pada
  `detail.hari_libur`.
- **Sabtu setengah hari**: bila Sabtu termasuk hari kerja dan `satker.jam_pulang_sabtu` terisi,
  jam pulang standar hari Sabtu memakai nilai tersebut (mis. 10:00/12:00). Hari lain memakai
  `satker.jam_pulang`.
- **Hari yang memiliki jadwal PH untuk personel tersebut dikeluarkan dari perhitungan absensi
  harian** (netral), karena kehadirannya dinilai pada rekap PH terpisah. Detail perhitungan
  menyimpan jumlah hari tersebut pada `detail.hari_ph`.
- Hari kerja tanpa catatan absensi dihitung **0** (dianggap tanpa keterangan).
- Periode berjalan hanya menghitung hari sampai kemarin agar personel tidak
  dihukum untuk hari yang belum terjadi.
- `skor_absensi = rata-rata skor harian` (dua desimal).

## 2. Rencana Kerja (bobot 20%)

```
skor = 0,4 × ketepatan + 0,3 × kelengkapan + 0,3 × kualitas
```

- **Ketepatan**: 100 bila seluruh rencana disetujui sebelum `tanggal_mulai`,
  berkurang 10 poin per hari keterlambatan (minimum 0).
- **Kelengkapan**: persentase rencana yang memiliki dasar tugas (jenis + nomor),
  target (kuantitas + waktu), dan minimal satu lampiran.
- **Kualitas**: `100 − 15 × jumlah_revisi` (minimum 0); jika ada rencana
  `ditolak`, nilai dibatasi maksimal 60.

Bila personel tidak memiliki rencana pada periode tersebut, skor = 0.

## 3. Hasil Kerja (bobot 30%)

```
skor = 0,4 × kuantitas + 0,3 × kualitas + 0,3 × waktu
```

- **Kuantitas**: `min(100, realisasi / target × 100)`.
- **Kualitas**: skor verifikator 0–100 (default 80 bila tidak diisi).
- **Waktu**: 100 bila `tanggal_realisasi ≤ target_waktu`, selain itu
  `max(0, 100 − 5 × hari_terlambat)`.
- Hanya hasil berstatus `terverifikasi` yang dihitung; nilai akhir indikator
  adalah rata-rata `skor_total` hasil kerja terverifikasi.

## 4. Output (bobot 30%)

```
skor_output = 0,4 × kelengkapan + 0,3 × kesesuaian + 0,3 × keabsahan
```

- **Kelengkapan**: 100 bila berkas bukti ada (atau URL terisi untuk jenis
  `link_sistem`), 0 bila tidak.
- **Kesesuaian** dan **keabsahan**: penilaian verifikator 0–100.
- Nilai indikator = rata-rata `skor_total` output `disetujui`,
  dikurangi 20 poin untuk setiap output `perlu_revisi` (minimum 0).

## 5. Nilai akhir

```
nilai = Σ (bobot_indikator / total_bobot × skor_indikator)
```

Kategori:

| Rentang | Kategori |
|---|---|
| ≥ 95 | Sangat Baik |
| 90 – 94,99 | Baik |
| 80 – 89,99 | Cukup |
| < 80 | Perlu Pembinaan |

## 6. Jejak audit

Setiap perhitungan menyimpan:

- `penilaian` — skor per indikator, nilai akhir, kategori, waktu hitung.
- `penilaian_detail` — komponen mentah per indikator (jumlah hari tercatat,
  `hari_ph`, total rencana, hari terlambat, daftar skor hasil kerja, dan sebagainya)
  dalam bentuk `jsonb`, sehingga hasil bisa dipertanggungjawabkan saat
  diperiksa.

Menghitung ulang periode yang sama akan memperbarui baris yang ada
(`ON CONFLICT (personel_id, periode_id)`), bukan membuat duplikat.
