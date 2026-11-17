# PRD — Aplikasi Absensi & Penilaian Kinerja Personel Polda Bali

Versi: 1.1 · Status: MVP terimplementasi · Sumber kebutuhan: `aplikasi_nilai_kerja.pdf`

## 1. Latar belakang

Penilaian kinerja personel di lingkungan Ditreskrimsus Polda Bali masih dilakukan manual:
rekap kehadiran terpisah dari laporan rencana kerja, hasil kerja, dan bukti output,
sehingga penilaian tidak jarang bersifat subjektif dan sulit dipertanggungjawabkan.

Aplikasi ini menyatukan empat indikator (**Absensi 20% · Rencana Kerja 20% · Hasil Kerja 30% ·
Output 30%**) menjadi satu alur digital dengan verifikasi berjenjang dan dashboard pimpinan.

## 2. Tujuan & metrik keberhasilan

| Tujuan | Metrik |
|---|---|
| Kehadiran tercatat objektif | ≥ 95% personel absen melalui aplikasi; penyimpangan lokasi < 2% |
| Rencana kerja terdokumentasi | ≥ 90% personel memiliki rencana disetujui sebelum periode berjalan |
| Penilaian transparan | 100% nilai dapat ditelusuri ke komponen perhitungan (`penilaian_detail`) |
| Verifikasi cepat | Rata-rata waktu keputusan atasan < 2 hari kerja |
| Laporan siap pakai | Laporan A4 & CSV dihasilkan tanpa rekap manual |

## 3. Pengguna & hak akses

Struktur organisasi **3 level**: Polda → Polres → Satker. Satker dapat berada langsung di
bawah Polda maupun di bawah Polres. Peran admin dan pimpinan memiliki **cakupan satker**
(`scope_satker_id`) yang menentukan luas data yang boleh dikelola/dilihat.

Akun dibagi menjadi dua jenis:

- **Admin** — mengelola sistem, memakai **username** (tanpa NRP), hanya login lewat dashboard web,
  dan tidak memiliki kewajiban absensi. Halaman pengelolaannya terpisah: **Akun Admin**.
- **Personel** — pegawai yang wajib absen, memakai **NRP**, dapat login web dan Android.
  Pimpinan dan atasan termasuk personel. Halaman pengelolaannya: **Data Personel**.

| Peran | Cakupan data | Kemampuan utama |
|---|---|---|
| **Personel** | Data sendiri | Absen, PH, rencana kerja, hasil kerja, output, melihat nilai sendiri |
| **Atasan Langsung** | Satker + bawahan | Semua di atas untuk dirinya + memutuskan rencana, memverifikasi hasil/output/absensi/PH, mengelola titik dinas |
| **Admin Sistem** | Semua | Seluruh master, periode & bobot, hitung penilaian, audit log |
| **Admin Polda** | Polda + seluruh turunannya | Kelola satker/personel/PH di lingkungan Polda |
| **Admin Satker Polda** | Satu satker di level Polda | Kelola satker/personel/PH satkernya |
| **Admin Polres** | Polres + satker di bawahnya | Kelola satker/personel/PH di lingkungan Polres |
| **Admin Satker Polres** | Satu satker di level Polres | Kelola satker/personel/PH satkernya |
| **Pimpinan Polda** | Polda + seluruh turunannya | Dashboard, rekap, laporan, keputusan penilaian |
| **Pimpinan Satker Polda** | Satu satker di level Polda | Dashboard & laporan satkernya |
| **Pimpinan Polres** | Polres + satker di bawahnya | Dashboard & laporan lingkungan Polres |
| **Pimpinan Satker Polres** | Satu satker di level Polres | Dashboard & laporan satkernya |
| **Pimpinan** | Sesuai cakupan yang diberikan | Sama dengan pimpinan ber-scope |

Aturan teknis: setiap query difilter `satker_id = ANY(cakupan)`; cakupan dihitung dari
`personel_role.scope_satker_id` (rekursif ke turunan). Personel biasa hanya menerima baris
miliknya pada daftar absensi/penilaian dan tidak dapat mengakses dashboard pimpinan.

## 4. Ruang lingkup

**Termasuk (MVP):**
- Autentikasi NRP (personel) atau username (admin) + password, refresh token, ganti password
- Master organisasi 3 level (Polda → Polres → Satker) + jam kerja & hari kerja per satker,
  zona waktu (WIB/WITA/WIT) dan jam pulang Sabtu untuk hari kerja setengah hari
- **Kalender hari libur** nasional/cuti bersama dan libur khusus satker; tanggal libur
  otomatis dikeluarkan dari perhitungan absensi
- 12 peran dengan cakupan satker (admin & pimpinan ber-scope)
- Titik dinas dengan radius (default 100 m)
- Absensi masuk/pulang: 8 status kehadiran (termasuk `libur_nasional`), validasi radius sisi server,
  foto opsional, antrean offline di perangkat, verifikasi atasan untuk data offline/mock
- **Modul PH/Strong Point terpisah**: strong point per satker, jadwal bulanan per personel,
  check-in/check-out **wajib lokasi**, verifikasi, dan rekap PH
- **Pengingat jadwal PH tanpa FCM** (notifikasi lokal WorkManager di Android)
- Rencana kerja: dasar tugas (SP/TOR), target kuantitas/kualitas/waktu, tahapan, lampiran,
  alur persetujuan (disetujui / perlu revisi / ditolak)
- Hasil kerja: realisasi, progres, kendala, bukti, verifikasi + skor komponen
- Output: 8 jenis, berkas atau tautan, verifikasi (disetujui / perlu revisi)
- Periode bulanan + bobot indikator yang dapat diubah (total 100)
- Perhitungan nilai otomatis + kategori + snapshot komponen
- Dashboard pimpinan (KPI, capaian, tren, sebaran, rekap) + cetak A4 + CSV
- Audit log mutasi + pembatasan laju permintaan
- Dokumentasi API (`/docs`, `/api-docs/openapi.json`)

**Belum termasuk:**
- Integrasi SSO/AD Polri, absensi via mesin sidik jari
- Push daring (FCM) — pengingat lokal sudah tersedia; pusat notifikasi web belum ada UI
- Antrean offline khusus absensi PH
- Peta interaktif pemilihan titik dinas/strong point
- Multi-instansi/Redis untuk scaling horizontal
- Tanda tangan digital pada laporan

## 5. Kebutuhan fungsional

### FR-01 Autentikasi
- Login admin memakai **username**; login personel memakai **NRP**. Pesan gagal seragam.
- Parameter `client` (`web`/`android`) menentukan kanal. Akun admin hanya boleh `web`;
  `client=android` hanya menerima akun personel (pimpinan/atasan termasuk).
- Access token 15 menit; refresh token 30 hari dengan rotasi; logout mencabut token.
- Password minimal 8 karakter, kombinasi huruf & angka; `must_change_password` diset saat akun dibuat.
- Pembatasan 5 percobaan login/menit/IP → HTTP 429 + `Retry-After`.

### FR-02 Absensi harian
- Status: dinas kantor, dinas luar, PH (Perintah Hadir), piket/shift, libur nasional, cuti, izin,
  sakit, tanpa keterangan.
- Aplikasi Android menampilkan alur bertahap: tombol **Absensi** → validasi lokasi (peta OSM,
  titik dinas terdekat, radius, jarak, akurasi) → pilihan status → kirim atau simpan offline.
- Tombol absen mengikuti status server hari ini: hanya Masuk bila belum masuk, hanya Pulang bila
  sudah masuk; bila status server tidak dapat diambil, kedua tombol ditampilkan dan mode offline aktif.
- Status **PH** pada absensi harian berbeda dari modul PH/Strong Point yang memiliki jadwal,
  strong point, dan absensi sendiri.
- Status yang dihitung hadir wajib menyertakan koordinat GPS; server menghitung jarak ke titik
  dinas terdekat (Haversine) dengan toleransi akurasi GPS.
- Di luar radius → HTTP 422 dengan pesan jarak aktual dan radius.
- Data ditandai `perlu_verifikasi` bila: offline, mock location, akurasi > 100 m, atau status
  tanpa keterangan.
- `client_uuid` menjamin idempotensi pengiriman ulang antrean offline.
- Satu absen masuk dan satu absen pulang per personel per hari.

### FR-03 Rencana kerja
- Wajib: program, kegiatan, uraian, tanggal mulai/selesai, periode.
- Opsional: tujuan, indikator, dasar tugas (jenis/nomor/tanggal/perihal), target, tahapan, lampiran.
- Hanya pemilik dapat mengubah, dan hanya saat status `draft`/`perlu_revisi`.
- Atasan langsung (atau admin) memutuskan; permintaan revisi wajib menyertakan catatan.
- Notifikasi dikirim ke atasan saat diajukan dan ke personel saat diputuskan.

### FR-04 Hasil kerja
- Hanya dari rencana berstatus `disetujui`; progres 0–100.
- Verifikasi menghitung skor kuantitas (realisasi/target), kualitas (input verifikator), dan waktu
  (keterlambatan) — lihat `docs/penilaian.md`.
- Verifikator dapat meminta revisi dengan catatan wajib.

### FR-05 Output
- Hanya dari hasil kerja `terverifikasi`.
- Jenis: laporan, dokumen pendukung, foto, video, data/diagram, berita acara, surat/nota dinas,
  link/sistem.
- Jenis selain `link_sistem` wajib berkas; `link_sistem` wajib URL `http(s)://`.
- Status: menunggu verifikasi → dalam verifikasi → disetujui / perlu revisi.
- Kelengkapan dinilai dari keberadaan berkas/tautan saat verifikasi.

### FR-06 Periode & penilaian
- Periode bulanan dengan tanggal mulai/selesai; bobot default 20/20/30/30; total bobot harus 100.
- Perhitungan dapat diulang (idempoten per personel+periode) dan menyimpan komponen mentah.
- Periode tertutup menolak perubahan data baru.

### FR-07 Dashboard & laporan
- Filter periode dan satker.
- KPI: total personel, hadir hari ini, rencana, hasil, output, rata-rata nilai.
- Grafik: capaian per indikator, tren antar periode, sebaran kategori.
- Tabel rekap berpaginasi + rincian komponen per personel.
- Semua tabel daftar berpaginasi (10 baris/halaman); modul besar memakai pagination server-side
  (`page`/`per_page` pada respons daftar).
- Pimpinan dapat menyimpan catatan & keputusan; laporan dapat dicetak A4 atau diunduh CSV.

### FR-08 Audit & operasional
- Setiap POST/PUT/PATCH/DELETE dicatat (aktor, aksi, entitas, entitas_id, IP, user agent, status).
- Admin global dapat menelusuri audit berdasarkan entitas, aktor, dan rentang tanggal.
- Backup harian database + berkas unggahan dengan retensi 30 hari (`deploy/backup.sh`).

### FR-09 PH / Strong Point
- Admin satker mengelola strong point (nama, alamat, koordinat, radius).
- Admin satker menyusun jadwal PH per bulan; satu jadwal berisi satu atau lebih personel
  beserta peran (mis. ketua/anggota).
- Server menolak jadwal yang jamnya beririsan untuk personel yang sama (HTTP 422).
- Personel hanya dapat check-in/check-out pada jadwal miliknya dan pada tanggal jadwal.
- Check-in **dan check-out wajib menyertakan koordinat**; server memvalidasi radius strong point.
  Check-out tidak boleh dilakukan dari lokasi lain (mis. rumah).
- Absensi PH memiliki `client_uuid` untuk idempotensi dan dapat diverifikasi atasan/admin.
- Rekap PH per bulan menampilkan jumlah jadwal, check-in, check-out, dan status.
- Hari yang memiliki jadwal PH **dikeluarkan dari skor absensi harian** dan tidak mengubah
  bobot 20/20/30/30; PH dilaporkan pada rekap terpisah.

### FR-10 Pengingat jadwal (tanpa FCM)
- Aplikasi Android menjadwalkan notifikasi lokal melalui WorkManager: H-1 pukul 19:00 dan
  60 menit sebelum jam mulai jadwal.
- Notifikasi dijadwalkan ulang setiap kali daftar jadwal PH dimuat dari server.
- Izin `POST_NOTIFICATIONS` diminta pada Android 13+; tanpa izin, jadwal tetap tampil di aplikasi.
- Tidak diperlukan layanan push eksternal maupun koneksi permanen.

### FR-11 Pengelolaan akun admin
- Akun admin memakai username unik (3–32 karakter) dan tidak memiliki NRP.
- Halaman **Akun Admin** terpisah dari **Data Personel**.
- Admin global dapat mengelola seluruh akun admin.
- Admin Polda hanya dapat membuat/mengelola `admin_satker_polda` dalam cakupannya.
- Admin Polres hanya dapat membuat/mengelola `admin_satker_polres` dalam cakupannya.
- Admin Satker tidak memiliki akses ke halaman Akun Admin.
- Admin global terakhir tidak dapat dinonaktifkan atau diturunkan perannya.

### FR-13 Periode otomatis
- Saat dashboard/penilaian/laporan dibuka dan belum ada periode yang memuat tanggal hari ini,
  sistem otomatis membuat periode bulan berjalan dengan bobot default 20/20/30/30.
- Admin dapat membuat periode tambahan (mis. bulan depan) dan mengubah bobot selama periode
  belum ditutup.
- Penutupan periode tetap **manual** oleh admin global setelah data bulan tersebut lengkap.
- Admin global dapat **membuka kembali** periode yang sudah ditutup bila ada data yang belum
  dinilai; alasan pembukaan disimpan untuk audit.
- Periode yang sudah ditutup tetap tersimpan sebagai histori dan dipakai untuk tren.

### FR-14 Kesiapan satker sebelum penutupan
- Setiap satker (level `satker`) dapat ditandai **siap ditutup** pada periode berjalan.
- Admin satker hanya dapat menandai satkernya; admin Polda/Polres dapat menandai satker
  dalam cakupannya; admin global melihat seluruh satker.
- Halaman **Kesiapan Periode** menampilkan jumlah personel, jumlah yang sudah dinilai,
  jumlah yang belum dinilai, status siap, penanda, waktu, dan catatan.
- Tanda siap hanya dapat diubah saat periode `open`; saat periode ditutup tombol dikunci.
- Saat periode dibuka kembali, seluruh tanda siap direset karena data dapat berubah.
- Admin global tetap dapat menutup periode walau masih ada satker belum siap, tetapi
  dashboard memberi peringatan jumlah satker yang belum siap.

### FR-15 Kalender hari libur
- Admin global mengelola libur nasional dan cuti bersama yang berlaku untuk semua satker.
- Admin dengan cakupan satker dapat mengelola libur khusus satkernya.
- Setiap entri memiliki tanggal, nama, jenis, cakupan, dan status aktif.
- Tanggal libur aktif otomatis dikeluarkan dari penyebut skor absensi harian, tanpa perlu
  setiap personel membuat absensi berstatus `libur_nasional`.
- Absensi pada hari libur tetap boleh dicatat (mis. piket), tetapi tidak menambah/mengurangi
  skor indikator absensi.
- Kalender dapat dilihat semua pengguna; perubahan hanya oleh admin sesuai cakupan.

### FR-12 Jam kerja, zona waktu, dan Sabtu setengah hari
- Zona waktu satker dipilih dari `Asia/Jakarta`, `Asia/Makassar`, atau `Asia/Jayapura`.
- Jam masuk dan jam pulang berlaku untuk hari kerja biasa.
- Bila Sabtu termasuk hari kerja, admin dapat mengisi `jam_pulang_sabtu` (mis. 10:00 atau 12:00)
  untuk Sabtu setengah hari. Bila kosong, Sabtu memakai jam pulang biasa.
- Nilai jam pulang Sabtu harus setelah jam masuk dan sebelum jam pulang biasa.

### FR-16 Data personel
- Admin mengisi NRP, nama, pangkat, jabatan, satker, atasan langsung, telepon, dan kata sandi awal.
- Pangkat dipilih dari combobox 21 tingkat Polri, urut dari Bharada sampai Jenderal Pol.
- Unit dan email tidak lagi diminta pada form personel; data lama tetap ditampilkan bila ada.
- Atasan langsung wajib berada pada satker yang sama dan berstatus aktif.
- Peran admin tidak dapat diberikan lewat halaman Data Personel; gunakan halaman Akun Admin.

### FR-17 Titik dinas berbasis peta
- Admin dan atasan mengelola titik dinas per satker (nama, jenis kantor/luar, koordinat, radius).
- Form tambah/edit menampilkan peta OpenStreetMap (Leaflet). Koordinat dapat diketik manual atau
  dipilih dengan klik peta dan digeser melalui penanda.
- Lingkaran radius tampil di peta dan ikut berubah saat radius diubah; nilai valid 10–5000 meter.
- Saat edit, satker tidak dipindah; nama, jenis, koordinat, radius, dan status aktif dapat diubah.

### FR-18 Jam server palsu (development)
- Backend dapat memakai jam palsu untuk pengujian (mis. `cargo run --bin fake -- 20-11-2026 07:45`).
- Jam palsu memengaruhi tanggal/jam absensi, endpoint `/absensi/hari-ini`, dan validasi tanggal.
- Fitur hanya aktif bila `APP__ALLOW_FAKE_TIME=true`; produksi wajib memakai jam sistem.
- Perintah `fake status` dan `fake reset` tersedia; tidak ada perubahan permanen pada data selain absensi uji.

### FR-19 Wizard izin Android
- Wizard tampil sekali sebelum login setelah aplikasi dipasang/di-reset.
- Menjelaskan izin lokasi (validasi radius, tanpa pelacakan latar belakang), kamera (foto bukti opsional),
  notifikasi (pengingat PH), serta WorkManager/penyimpanan (antrean offline, cache privat).
- Setiap izin dapat diberikan, ditolak, atau dilewati; status dapat diubah lewat Pengaturan Android.

## 6. Kebutuhan non-fungsional

| Aspek | Target |
|---|---|
| Waktu respons API | < 300 ms (p95) untuk endpoint daftar pada 145 personel/periode |
| Perhitungan periode | < 60 detik untuk 200 personel |
| Keamanan | Argon2id, JWT HS256, RBAC per satker, audit log, rate limit, HTTPS di produksi |
| Privasi | Tidak ada token di localStorage browser; berkas diunduh dengan bearer token |
| Perangkat | Android 8.0+ (minSdk 26), GPS + kamera opsional; dashboard Chrome/Edge/Firefox terbaru |
| Offline | Absensi tersimpan lokal dan tersinkron otomatis (WorkManager, 30 menit / manual) |
| Kapasitas awal | 1 Polda + 9 Polres, ±200 personel, 1 periode aktif |

## 7. Batasan & asumsi

- Satu server on-prem (LAN) tanpa autoscaling; pembatas laju berbasis memori satu instans.
- Radius titik dinas dan jam kerja ditetapkan per satker (default 100 m, 08:00–16:00 WITA).
- Bobot indikator dapat berubah antar periode, tetapi tidak dalam periode tertutup.
- Foto absensi maksimal 5 MB (dikompres di perangkat), lampiran 10 MB, bukti output 50 MB.

## 8. Risiko produk

| Risiko | Mitigasi |
|---|---|
| GPS melenceng di dalam gedung | toleransi akurasi + eskalasi verifikasi atasan |
| Titik dinas belum diatur | API menolak dengan pesan jelas; admin wajib menyiapkan titik sebelum go-live |
| Personel tidak mengisi rencana | skor rencana 0; dashboard menampilkan capaian rendah sebagai sinyal |
| Periode dihitung sebelum data lengkap | periode berjalan hanya menghitung hari yang sudah lewat; bisa dihitung ulang |

## 9. Roadmap lanjutan

1. Pusat notifikasi web + pengingat jadwal PH di dashboard; FCM bila perlu lintas perangkat.
2. Antrean offline absensi PH + ekspor CSV rekap PH.
3. Peta pemilihan titik dinas/strong point di dashboard.
4. Ekspor PDF server-side dengan kop resmi & QR verifikasi.
5. Redis untuk rate limit/session bila API di-scale.
6. Integrasi SI SDM Polri untuk sinkronisasi data personel.
