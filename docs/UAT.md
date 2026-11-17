# Checklist UAT — Absensi & Penilaian Kinerja Personel Polda Bali

Status: ✅ = sudah diverifikasi otomatis (API/e2e) · ⬜ = perlu diuji manual di lapangan.

Cara pakai: jalankan `cargo run --bin seed -- --reset` untuk data demo di `kinerja_db`,
lalu ikuti skenario. Untuk uji otomatis gunakan database terpisah `kinerja_test`
(`bun tools/uji/api-smoke.js`). Catat temuan pada kolom Catatan.

Akun demo: lihat tabel di `docs/CHECKLIST.md` §6 (admin global `superuser / Superuser#2026`).

## A. Admin Sistem (`superuser` / `Superuser#2026`)

| # | Skenario | Hasil yang diharapkan | Status | Catatan |
|---|---|---|---|---|
| A1 | Buka `/docs` | Daftar 70 endpoint tampil, filter tag berfungsi | ✅ | |
| A2 | Login memakai username `superuser` | Berhasil; `jenis_akun=admin`, `nrp=null` | ✅ | API |
| A3 | Login memakai NRP `000001` | Gagal (admin tidak punya NRP) | ✅ | API |
| A4 | Login `superuser` dengan `client=android` | 403 "Akun admin hanya dapat masuk melalui dashboard web" | ✅ | API |
| A5 | Tambah satker baru level `satker` di bawah Polres | Satker muncul di daftar, kode unik | ✅ | API |
| A6 | Tambah satker dengan kode duplikat | 409 "Kode satker sudah dipakai" | ✅ | API |
| A7 | Tambah personel NRP baru + atasan satu satker | Personel muncul, `must_change_password=true`; pangkat dipilih dari combobox, tanpa kolom unit/email | ✅ | UI + API |
| A8 | Tambah personel dengan NRP duplikat | 409 "NRP … sudah terdaftar" | ✅ | API |
| A9 | Tambah personel dengan atasan beda satker | 422 (atasan harus satu satker) | ✅ | API |
| A10 | Beri peran admin lewat halaman Data Personel | 422 (peran admin hanya lewat Akun Admin) | ⬜ | |
| A11 | Ubah jam masuk/pulang & hari kerja satker | Tersimpan; absensi memakai jam baru | ⬜ | UI |
| A12 | Tambah akun admin di halaman **Akun Admin** | Akun muncul di daftar admin, tidak di Data Personel | ✅ | API |
| A13 | Cabut peran admin global terakhir | 422 ditolak | ⬜ | |
| A14 | Reset password akun admin | Password baru berlaku, sesi lama dicabut | ⬜ | |
| A15 | Ubah bobot periode menjadi 25/25/25/25 | Tersimpan; hitung ulang mengubah nilai akhir | ⬜ | |
| A16 | Simpan bobot total 90 | 422 "Total bobot harus 100" | ⬜ | |
| A17 | Buka `GET /audit` setelah beberapa perubahan | Baris audit memuat aktor, aksi, IP | ✅ | API |
| A18 | Tutup periode | Status `closed`; input baru untuk periode itu ditolak | ⬜ | |
| A19 | Login salah 6× dalam 1 menit | Percobaan ke-4 dst → 429 + Retry-After | ✅ | API |
| A20 | Pilih zona waktu dari combo box (WIB/WITA/WIT) | Tersimpan; zona di luar daftar ditolak 422 | ✅ | API |
| A21 | Set Sabtu sebagai hari kerja + jam pulang 10:00/12:00 | Tersimpan; penilaian Sabtu memakai jam pulang tersebut | ✅ | API |
| A22 | Buka dashboard saat periode bulan ini belum ada | Periode bulan berjalan dibuat otomatis, status `open` | ✅ | API |
| A23 | Tutup periode lalu buka kembali dengan alasan | Status `closed` → `open`; alasan tersimpan | ✅ | API |
| A24 | Admin Polda mencoba menutup/membuka periode | 403 (hanya admin global) | ✅ | API |
| A25 | Buka halaman Kesiapan Periode | 21 satker tampil dengan jumlah personel & status | ✅ | API |
| A26 | Tandai satu satker siap, lalu buka daftar lagi | Status siap + penanda/waktu tercatat | ✅ | API |
| A27 | Tutup periode saat masih ada satker belum siap | Konfirmasi memperingatkan jumlah satker belum siap | ⬜ | UI |
| A28 | Buka kembali periode | Semua tanda siap direset | ⬜ | UI |
| A29 | Buka halaman Hari Libur | 5 libur nasional seed tampil | ✅ | API |
| A30 | Tambah libur nasional + duplikat | Tersimpan; tanggal sama 409 | ✅ | API |
| A31 | Buka daftar satker/libur/titik dinas/kesiapan/PH/absensi/rekap/persetujuan | Tiap tabel menampilkan “Menampilkan X–Y dari N” dan tombol Sebelumnya/Berikutnya; maks. 10 baris | ⬜ | UI |
| A32 | Tambah titik dinas, klik peta OpenStreetMap | Koordinat terisi otomatis, penanda & lingkaran radius muncul; simpan berhasil | ⬜ | UI |
| A33 | Edit titik dinas: ubah nama, jenis, koordinat/penanda, radius, status | Perubahan tersimpan dan tampil di tabel | ⬜ | API ✅; UI perlu uji visual |

## B. Admin ber-scope

| # | Skenario | Hasil yang diharapkan | Status | Catatan |
|---|---|---|---|---|
| B1 | Admin Polda (`admin_polda`) melihat daftar satker | Seluruh satker di bawah Polda tampil | ✅ | API |
| B2 | Admin Polda membuat satker di luar Polda | 403 | ✅ | API |
| B3 | Admin Satker Polda (`admin_satker_polda`) melihat personel | Hanya personel satkernya | ✅ | API |
| B4 | Admin Satker Polda mengubah personel satker lain | 403 | ✅ | API |
| B5 | Admin Polres (`admin_polres`) mengelola satker di bawahnya | Berhasil untuk satker turunan | ✅ | API |
| B6 | Admin Satker Polres (`admin_satker_polres`) mengelola satkernya | Berhasil hanya untuk satkernya | ✅ | API |
| B7 | Admin ber-scope membuka `/audit` | 403 (audit khusus admin global) | ✅ | API |
| B8 | Admin Polda membuat `admin_satker_polda` | Berhasil; akun hanya muncul di Akun Admin | ✅ | API |
| B9 | Admin Polda membuat `admin_sistem` lewat `/admin` | 403 | ✅ | API |
| B10 | Admin Satker membuka halaman Akun Admin | Menu tidak tampil / API 403 | ✅ | API |
| B11 | Admin Satker menandai kesiapan satkernya | Berhasil; hanya satker sendiri yang tampil | ✅ | API |
| B12 | Admin Satker menandai satker lain | 403 | ✅ | API |
| B13 | Admin Satker menambah libur khusus satkernya | Berhasil; satker lain 403 | ✅ | API |

## C. Personel (`85000001` / `Polri#2026`)

| # | Skenario | Hasil yang diharapkan | Status | Catatan |
|---|---|---|---|---|
| C1 | Login di Android | Logo & header terpusat, ikon mata menampilkan/menyembunyikan password, masuk ke Beranda | ✅ | emulator |
| C2 | Absen masuk di dalam radius | Tercatat, "0 m dari titik dinas" | ✅ | emulator |
| C3 | Absen masuk di luar radius (status Dinas Kantor) | Ditolak 422 dengan jarak & radius | ✅ | API |
| C4 | Absen dua kali jenis sama di hari sama | 409 "Absen masuk sudah tercatat" | ✅ | API |
| C5 | Mode pesawat → absen | Tersimpan lokal, muncul badge antrean | ⬜ | |
| C6 | Nyalakan data → kirim antrean | Terkirim sekali (tidak duplikat), antrean kosong | ⬜ | |
| C7 | Lihat daftar absensi satker | Hanya data sendiri | ✅ | API |
| C8 | Buka dashboard pimpinan | 403 (tidak berhak) | ✅ | API |
| C9 | Buka tab **PH** | Jadwal PH bulan ini tampil | ✅ | UI |
| C10 | Check-in PH tanpa GPS | 422 "Lokasi GPS wajib diambil untuk absensi PH" | ✅ | API |
| C11 | Check-in PH di luar radius strong point | 422 dengan jarak aktual | ✅ | API |
| C12 | Check-out PH dari lokasi berbeda (mis. rumah) | Ditolak 422 (wajib lokasi & radius) | ✅ | API |
| C13 | Check-out PH pada tanggal bukan jadwal | Ditolak 422 | ✅ | API |
| C14 | Check-in/out PH dua kali | 409 untuk jenis yang sama | ✅ | API |
| C15 | Lihat rekap PH sendiri di dashboard | Check-in/out dengan lokasi tercatat | ⬜ | UI |
| C16 | Izin notifikasi Android 13+ | Diminta saat pertama buka aplikasi | ⬜ | perangkat |
| C17 | Pengingat H-1 19:00 & 60 menit sebelum | Notifikasi muncul sesuai jadwal | ⬜ | perangkat |
| C18 | Login memakai akun admin di Android | Ditolak 403 dengan pesan jelas | ✅ | API |
| C19 | Buka aplikasi pertama kali setelah install | Wizard izin tampil sebelum login; lokasi, kamera, notifikasi, WorkManager/penyimpanan dijelaskan | ✅ | emulator |
| C20 | Tekan tombol besar **Absensi** | Halaman validasi peta OpenStreetMap dengan titik dinas, radius, lokasi, jarak, dan akurasi | ✅ | emulator |
| C21 | Belum absen masuk hari ini | Hanya tombol **Absen Masuk** yang tampil | ✅ | emulator |
| C22 | Sudah absen masuk hari ini | Hanya tombol **Absen Pulang** yang tampil | ✅ | emulator |
| C23 | Di dalam radius | Status Dinas Kantor otomatis terpilih; di luar radius tidak ada yang terpilih | ✅ | emulator |
| C24 | Server tidak terjangkau saat cek absensi hari ini | Dua tombol tampil, mode offline aktif, absensi tersimpan di perangkat | ✅ | emulator |
| C25 | Riwayat absensi | Terbuka di halaman terpisah dan memuat riwayat bulan berjalan | ✅ | emulator |
| C26 | Jam server palsu `20-11-2026 07:45` | Absensi tercatat tanggal 20 Nov 2026 pukul 07:45; reset mengembalikan jam sistem | ✅ | API + emulator |
| C27 | Pilih status **PH (Perintah Hadir)** | Tersimpan sebagai status `ph` dan dihitung hadir | ✅ | API |

## D. Atasan Langsung (`000100` / `Atasan#2026`)

| # | Skenario | Hasil yang diharapkan | Status | Catatan |
|---|---|---|---|---|
| D1 | Buka menu Persetujuan | Rencana `diajukan` dari bawahan tampil | ✅ | UI |
| D2 | Setujui rencana | Status `disetujui`, personel dapat notifikasi | ✅ | e2e |
| D3 | Minta revisi tanpa catatan | 422 "Catatan wajib diisi" | ⬜ | |
| D4 | Verifikasi hasil kerja (kualitas 90) | Skor kuantitas/kualitas/waktu + total tersimpan | ✅ | e2e |
| D5 | Verifikasi output dengan berkas bukti | Status `disetujui`, kelengkapan 100 | ✅ | e2e |
| D6 | Verifikasi absensi offline/mock | `perlu_verifikasi=false`, catatan tersimpan | ⬜ | |
| D7 | Verifikasi absensi PH | Status terverifikasi + catatan | ✅ | API |
| D8 | Verifikasi absensi PH satker lain | 403 | ✅ | API |
| D9 | Menyusun jadwal PH | Hanya admin satker; atasan ditolak | ✅ | API |
| D10 | Kelola titik dinas satker | Titik aktif/nonaktif berubah, radius valid 10–5000 m | ✅ | UI |

## E. Pimpinan (`000002` / `Pimpinan#2026` dan akun ber-scope)

| # | Skenario | Hasil yang diharapkan | Status | Catatan |
|---|---|---|---|---|
| E1 | Pimpinan Polda buka dashboard tanpa filter | KPI total personel 145, capaian 4 indikator, tren, sebaran | ✅ | UI |
| E2 | Pimpinan Polres (`000022`) buka dashboard | Hanya data lingkungan Polres | ✅ | API |
| E3 | Pimpinan Satker Polda (`000012`) buka dashboard | Hanya data satkernya | ✅ | API |
| E4 | Pimpinan Satker Polres (`000023`) buka dashboard | Hanya data satkernya | ⬜ | |
| E5 | Simpan catatan + keputusan "Disetujui" | Tersimpan, tampil di rincian penilaian | ✅ | UI |
| E6 | Cetak laporan (A4) | Kop + logo, tabel, rata-rata, kolom tanda tangan rapi | ✅ | UI |
| E7 | Unduh CSV | Header + 145 baris personel | ✅ | API |
| E8 | Hitung ulang penilaian | Nilai diperbarui, tidak membuat duplikat | ✅ | e2e |
| E9 | Hari ber-jadwal PH | Tidak dihitung sebagai absensi harian (`hari_ph` terisi) | ✅ | unit test |

## F. Non-fungsional

| # | Skenario | Hasil yang diharapkan | Status | Catatan |
|---|---|---|---|---|
| F1 | `cargo test` | 30 unit test lulus | ✅ | |
| F2 | `cargo fmt --check` + clippy `-D warnings` | Bersih | ✅ | |
| F3 | `bun run check` | 0 error, 0 warning | ✅ | |
| F4 | `bun run build` | SPA statis terbentuk di `dashboard/build` | ✅ | |
| F5 | `.gradlew :app:assembleDebug` | APK debug terbentuk | ✅ | |
| F6 | `bun tools/uji/api-smoke.js` | 48/48 cek lulus (DB `kinerja_test`) | ✅ | |
| F7 | `bun tools/uji/openapi-check.js` | JSON valid, semua `$ref` ada | ✅ | |
| F8 | Restore backup di lingkungan uji | Data & berkas kembali utuh | ⬜ | |
| F9 | Uji 1 bulan data riil | Hitung periode < 60 detik | ⬜ | |
| F10 | Akses via HTTPS internal | Cookie `Secure`, tidak ada peringatan mixed content | ⬜ | |
| F11 | Uji dua perangkat sekaligus (Android 8 & 13) | GPS, kamera, notifikasi, antrean offline berjalan | ⬜ | |

## G. Kriteria lulus

Rilis dinyatakan layak bila:

1. Semua item **P1** pada `docs/HANDOFF.md` §9 selesai.
2. Tidak ada item ✅ yang gagal, dan seluruh item ⬜ pada bagian A–E sudah diuji minimal sekali.
3. Backup & restore terbukti (F8).
4. Password demo sudah diganti dan data demo dibersihkan pada server produksi.

## H. Format laporan temuan

```
ID: C5
Perangkat/tanggal: Pixel 6a / 2026-11-18
Langkah: aktifkan mode pesawat → Absen Masuk
Hasil: ...
Diharapkan: ...
Bukti: screenshot / log
```
