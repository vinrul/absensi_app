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
- **Personel** — pegawai yang wajib absen, memakai **NRP**. Atasan langsung dan personel hanya
  memakai **Android**; pimpinan memakai Android dan dashboard web. Halaman pengelolaannya:
  **Data Personel**.

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
- Dashboard web hanya menerima akun yang memiliki **role admin atau pimpinan**. Akun dengan
  role atasan/personel saja ditolak (403) dan diarahkan memakai aplikasi Android. Bila satu akun
  memiliki beberapa role, akses dashboard terbuka karena adanya role pimpinan/admin; kemampuan
  di dalam dashboard mengikuti gabungan role tersebut (mis. pimpinan yang juga atasan langsung
  tetap dapat memutuskan rencana bawahannya).
- Access token 15 menit; refresh token 30 hari dengan rotasi; logout mencabut token.
- Password minimal 8 karakter, kombinasi huruf & angka; `must_change_password` diset saat akun dibuat.
- Pembatasan 5 percobaan login/menit/IP → HTTP 429 + `Retry-After`.
- Profil personel dapat memuat **foto** (JPG/PNG maks. 5 MB) dan diubah sendiri untuk nama,
  **pangkat** (combobox 22 tingkat Polri termasuk Brigpol, atau 17 pangkat ASN/PNS dari
  Juru Muda I/a sampai Pembina Utama IV/e), **jabatan**, telepon,
  serta email. Satker, NRP, jenis akun, dan peran tetap hanya dapat diubah admin.
- Halaman **Profil & Pengaturan** Android menyediakan ganti foto, edit profil, dan ganti password.
  Kolom password memiliki ikon mata untuk menampilkan/menyembunyikan isian. Ganti password
  mencabut seluruh sesi sehingga pengguna harus login kembali.
- Halaman **Profil Saya** di dashboard (`/profil`, untuk admin & pimpinan) menyediakan unggah
  foto profil (JPG/PNG maks. 5 MB) dan ganti password dengan tombol mata pada tiap kolom.
  Avatar tampil di sidebar dan header; setelah ganti password seluruh sesi dicabut dan pengguna
  diarahkan ke halaman login.
- Foto profil yang sudah diunggah ditampilkan sebagai avatar di Beranda; bila belum ada, Beranda
  tidak menampilkan avatar maupun placeholder.

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
- Server menerapkan kebijakan lokasi tiruan `APP__MOCK_LOCATION_POLICY`: `flag` menyimpan data
  dengan `perlu_verifikasi=true`, `reject` menolak dengan HTTP 422. Default development `flag`,
  produksi disarankan `reject`.
- `client_uuid` menjamin idempotensi pengiriman ulang antrean offline.
- Satu absen masuk dan satu absen pulang per personel per hari.
- Foto bukti absensi yang diunggah dari Android dapat dilihat di dashboard: galeri pada rekap
  absensi dan tombol Masuk/Pulang pada detail per orang. Berkas hanya dapat diakses pemilik atau
  pengelola dalam cakupan satker.

### FR-03 Rencana kerja
- Wajib: program, kegiatan, uraian, tanggal mulai/selesai, periode.
- Opsional: tujuan, indikator, dasar tugas (jenis/nomor/tanggal/perihal), target, tahapan, lampiran.
- Android menyediakan wizard 4 langkah: jenis rencana → dasar tugas → rincian kegiatan → lampiran.
  Rincian kegiatan dapat ditambah lebih dari satu; tiap entri memuat program, kegiatan, uraian,
  tujuan, indikator, target kuantitas/kualitas, dan tanggal mulai/selesai (tabel
  `rencana_tahapan`). Satu wizard menghasilkan satu rencana sehingga satu alur persetujuan.
- Wizard dapat menyimpan **draf** (masih dapat diubah) atau langsung **mengajukan ke atasan**;
  lampiran multi-berkas diunggah setelah rencana tersimpan.
- Rencana berstatus `draft`/`perlu_revisi` dapat dibuka kembali di Android (**Ubah/Perbaiki Rencana**).
  Wizard terisi data lama beserta catatan atasan; perubahan dikirim lewat `PATCH /rencana-kerja/{id}`
  yang juga mengganti daftar rincian kegiatan, lalu dapat langsung diajukan ulang.
- Dashboard **Rencana Kerja** menyediakan filter periode, status, **satker**, dan **personel**,
  halaman **detail** lengkap (uraian, dasar tugas, rincian tahapan, lampiran), serta tombol
  **Export PDF** sesuai filter (kop Polda Bali, A4 landscape) memakai Chrome/Edge headless di server.
  Untuk role pimpinan, tombol **Buat Rencana** dan aksi **Ajukan** disembunyikan (pimpinan hanya
  memantau/memutuskan, bukan menyusun rencana).
- Semua tanggal (dasar tugas, mulai/selesai kegiatan) dipilih lewat **date picker**, bukan input teks.
- Lampiran yang sudah tersimpan dapat dilihat langsung di Android: PDF dirender per halaman dan
  gambar ditampilkan penuh dengan pinch-zoom; tidak memerlukan aplikasi eksternal.
- Hanya pemilik dapat mengubah, dan hanya saat status `draft`/`perlu_revisi`.
- Atasan langsung (atau admin) memutuskan; permintaan revisi wajib menyertakan catatan.
- Persetujuan dapat dilakukan lewat dashboard web maupun tombol **Persetujuan Rencana** di
  Android (kartu di Beranda dan tombol di tab Rencana). Yang berhak adalah pemilik `atasan_id`
  pada rencana tersebut; pimpinan yang ditunjuk sebagai atasan langsung juga dapat memutuskan.
- Akun dual-role (atasan sekaligus bawahan) tetap dapat mengajukan rencana sendiri ke atasannya
  dan memutuskan rencana bawahannya; daftar rencana pribadi dipisahkan dari daftar persetujuan.
- Notifikasi dikirim ke atasan saat diajukan dan ke personel saat diputuskan.

### FR-04 Hasil kerja
- Hanya dari rencana berstatus `disetujui`; progres 0–100 memakai slider (default 100%).
- Wizard Android 3 langkah: pilih rencana → realisasi (uraian, progres, realisasi, kendala,
  tanggal) → bukti pendukung. Tersedia **Simpan Draf** dan **Kirim ke Atasan**.
- Draf dan `perlu_revisi` dapat diubah lalu dikirim ulang; detail tersedia untuk semua status.
- Verifikasi menghitung skor kuantitas (realisasi/target), kualitas (input verifikator), dan waktu
  (keterlambatan) — lihat `docs/penilaian.md`.
- Verifikator dapat meminta revisi dengan catatan wajib. Skor kualitas dipilih memakai slider 0–100
  pada layar verifikasi Android. Layar verifikasi dibuka dari kartu di Beranda atau tab Hasil Kerja.
- Dashboard **Hasil Kerja** menyediakan filter periode, status, satker, dan personel, halaman detail
  beserta lampiran bukti, serta **Export PDF** sesuai filter (Chrome/Edge headless). Tombol
  **Tambah Hasil** disembunyikan untuk role pimpinan.

### FR-05 Output
- Hanya dari hasil kerja `terverifikasi`.
- Jenis: laporan, dokumen pendukung, foto, video, data/diagram, berita acara, surat/nota dinas,
  link/sistem.
- Jenis selain `link_sistem` wajib berkas; `link_sistem` wajib URL `http(s)://`.
- Wizard Android 3 langkah: pilih hasil kerja → data output (jenis, judul, uraian, tanggal, URL) →
  berkas bukti. Tersedia **Simpan Draf** dan **Kirim ke Atasan**; draf/revisi dapat diubah.
- Status: `draft` → dalam verifikasi → `disetujui` / `perlu_revisi`.
- Kelengkapan dinilai dari keberadaan berkas/tautan saat verifikasi; kesesuaian dan keabsahan
  diisi atasan memakai slider 0–100 (bobot 40/30/30). Permintaan revisi wajib menyertakan catatan.
- Layar verifikasi atasan menampilkan output bawahan langsung yang berstatus dalam verifikasi,
  lengkap dengan detail lampiran.
- Dashboard **Output** menyediakan filter periode, status, satker, dan personel, halaman detail
  (uraian, tautan, komponen nilai, catatan, lampiran), serta **Export PDF** sesuai filter
  (Chrome/Edge headless). Tombol **Tambah Output** disembunyikan untuk role pimpinan.

### FR-06 Periode & penilaian
- Periode bulanan dengan tanggal mulai/selesai; bobot default 20/20/30/30; total bobot harus 100.
- Perhitungan dapat diulang (idempoten per personel+periode) dan menyimpan komponen mentah.
- Periode tertutup menolak perubahan data baru.

### FR-07 Dashboard & laporan
- Sidebar dashboard memakai menu bergrup yang dapat dilipat: **Dashboard**; **Master Data**
  (Satker & Jam Kerja, Titik Dinas, Hari Libur, Data Personel); **Absensi** (Rekap Kehadiran,
  Detail Rekap, Verifikasi Offline); **Strong Point** (Jadwal Strong Point, Rekap Strong Point);
  **Kinerja** (Rencana Kerja, Hasil Kerja, Output); **Laporan** (Penilaian, Print Laporan);
  **Pengaturan** (Akun Admin, Kesiapan Periode, Periode). Grup **Master Data** disembunyikan untuk
  role pimpinan (tanpa admin), termasuk akses langsung ke halaman master. Submenu **Verifikasi
  Offline** dan tab verifikasinya juga disembunyikan untuk pimpinan; akses `?tab=verifikasi`
  dialihkan ke Rekap Kehadiran.
- Filter periode dan satker.
- KPI: total personel, hadir hari ini, rencana, hasil, output, rata-rata nilai.
- Grafik: capaian per indikator, tren antar periode, sebaran kategori.
- Tabel rekap berpaginasi + rincian komponen per personel.
- Halaman **Rekap Absensi** (`/absensi/rekap`) memiliki tiga tab dengan filter terpisah:
  - **Rekap Kehadiran**: periode + satker; kolom hadir, telat (hari), menit telat (khusus
    Dinas Kantor), dinas luar, piket, cuti, izin, sakit, PH, tanpa keterangan, dan menunggu
    verifikasi. Filter subdit/bagian dihapus.
  - **Verifikasi**: periode + satker + personel; absen masuk dan pulang pada tanggal yang sama
    digabung menjadi satu baris, menampilkan kedua jam dan menit telat; foto absensi dan catatan
    verifikator tersedia, dan penyetujuan memproses seluruh catatan pada tanggal tersebut.
  - **Detail per Orang**: periode + satker + personel (wajib dipilih); riwayat harian dengan
    pasangan jam masuk/pulang, status, menit telat, status radius, kondisi, dan catatan verifikator.
  Setiap tab menyediakan **Export PDF** sesuai filter aktif (kop Polda Bali, A4 landscape)
  memakai Chrome/Edge headless di server; PDF rekap memuat baris jumlah termasuk menit telat,
  sedangkan PDF verifikasi/detail memakai baris per tanggal yang sudah digabung.
- Semua tabel daftar berpaginasi (10 baris/halaman); modul besar memakai pagination server-side
  (`page`/`per_page` pada respons daftar).
- Pimpinan dapat menyimpan catatan & keputusan; laporan dapat dicetak A4 atau diunduh CSV.

### FR-08 Audit & operasional
- Setiap POST/PUT/PATCH/DELETE dicatat (aktor, aksi, entitas, entitas_id, IP, user agent, status).
- Admin global dapat menelusuri audit berdasarkan entitas, aktor, dan rentang tanggal.
- Backup harian database + berkas unggahan dengan retensi 30 hari (`deploy/backup.sh`).

### FR-09 PH / Strong Point
- Admin satker mengelola strong point (nama, alamat, koordinat, radius). Tab master strong point di
dashboard **disembunyikan sementara** karena strong point belum memiliki titik tetap; API-nya tetap
aktif dan dapat diaktifkan kembali.
- Admin satker menyusun jadwal PH per bulan; satu jadwal berisi satu atau lebih personel
  beserta peran (mis. ketua/anggota). Sesi pagi/sore belum dipisah (menyusul di laporan & Android).
- Form tambah/edit jadwal di dashboard hanya meminta **satker, regu, tanggal, catatan, dan
  personel**; combo strong point serta jam mulai/selesai tidak ditampilkan. Backend memakai strong
  point aktif pertama milik satker (kompatibilitas radius lama) dan waktu bawaan `00:00–23:59`.
- Input tanggal mendukung **rentang** (`tanggal` s.d. `tanggal_selesai`); backend membuat satu
  jadwal per tanggal dalam satu kali simpan, setelah memvalidasi bentrok pada seluruh tanggal.
- Tab **Jadwal Strong Point** menampilkan **roster bulanan** seperti spreadsheet: kolom No,
  Anggota, Pangkat, dan tanggal 1–akhir bulan; tanggal yang dijadwalkan regu ditandai blok biru
  berlabel regu (di-merge vertikal/horizontal bila seluruh anggota regu bertugas).
- Server menolak jadwal yang jamnya beririsan untuk personel yang sama (HTTP 422).
- Personel hanya dapat check-in/check-out pada jadwal miliknya dan pada tanggal jadwal.
- Check-in **dan check-out wajib menyertakan koordinat**; server memvalidasi radius strong point.
  Check-out tidak boleh dilakukan dari lokasi lain (mis. rumah).
- Absensi PH memiliki `client_uuid` untuk idempotensi dan dapat diverifikasi atasan/admin.
- Rekap PH per bulan menampilkan jumlah jadwal, check-in, **hanya check-in**, dan
  **tidak strong point** beserta tanggalnya, sehingga personel yang hanya check-in atau tidak
  menjalankan jadwal langsung terlihat (kolom telat tidak ditampilkan karena jadwal strong point
  tidak lagi memakai jam mulai/selesai). Perhitungan rekap dibatasi **sampai hari ini**; tanggal
  yang belum berjalan tidak dianggap tidak strong point.
- Jadwal dan rekap strong point dapat diekspor ke **PDF** (A4 landscape) sesuai bulan dan filter
  satker. PDF jadwal berbentuk roster (kolom anggota/pangkat × tanggal 1–akhir bulan dengan blok
  regu), sedangkan PDF rekap memakai batas tanggal yang sama dengan dashboard.
- Strong point dan absensi kantor tetap terpisah namun saling terkait: hari ber-jadwal strong point
  **tetap wajib absensi kantor**. Keterlambatan absensi kantor **tidak dihitung** bila personel
  check-in strong point pada hari itu; bila jadwal strong point tidak dijalankan, keterlambatan
  kantor dihitung normal. Pulang cepat tetap dihitung. Detail penilaian menyimpan `hari_ph`
  (jadwal) dan `hari_ph_dilakukan` (check-in).
- Satu jadwal dapat memiliki **beberapa sesi check-in** (mis. pagi dan sore). Check-in baru ditolak
  bila sesi sebelumnya masih terbuka dan belum berumur **2 jam**; check-out dipasangkan ke check-in
  terakhir yang terbuka.
- Di aplikasi Android, tab **Strong** menyediakan tombol besar menuju wizard: peta lokasi tanpa
  titik dinas (marker hijau), koordinat, akurasi GPS, alamat reverse-geocode, lalu check-in/check-out.
  Check-in dilanjutkan ke dokumentasi foto dan halaman kirim; check-out langsung ke halaman kirim.
  Bila tidak ada sinyal, kedua tombol check-in/check-out (offline) tersedia dan data masuk antrean
  untuk dikirim otomatis.
- Foto dokumentasi check-in dapat dilihat pada rekap strong point dashboard melalui tombol
  **Lihat** (galeri per personel).
- Absensi PH memakai kebijakan lokasi tiruan yang sama dengan absensi harian: `flag` menandai
  `perlu_verifikasi`, `reject` menolak HTTP 422. Aplikasi menampilkan peringatan bila
  `Location.isMock`, Developer Options aktif, atau aplikasi mock dipilih.

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
- Pangkat dipilih dari combobox 22 tingkat Polri (Bharada sampai Jenderal Pol, termasuk Brigpol)
  atau 17 pangkat ASN/PNS (Juru Muda I/a sampai Pembina Utama IV/e).
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

### FR-20 Aplikasi web personel & atasan
- Aplikasi terpisah di folder `personel/` memakai Svelte 5, SvelteKit 3 SPA, Tailwind 4, dan Bun.
  Login memakai NRP + password dengan tombol tampil/sembunyikan password.
- Kanal login server `personel_web` hanya menerima akun personel (termasuk atasan/pimpinan);
  akun admin ditolak. Access token berada di memori dan refresh memakai cookie httpOnly.
- Navigasi bawah 6 tab: Beranda, Absensi, Rencana, Hasil, Output, Strong Point. Desain
  mobile-first dan responsif, berbeda dari dashboard (tanpa sidebar desktop).
- Wizard izin browser menjelaskan lokasi, kamera, notifikasi, dan antrean offline sekali
  setelah login pertama.
- Absensi memakai GPS perangkat, peta OpenStreetMap, validasi radius ke titik dinas terdekat,
  8 status kehadiran, foto opsional, serta antrean offline di penyimpanan browser.
- Strong Point memakai marker hijau tanpa titik dinas, koordinat, akurasi, reverse-geocode
  `geocode.vinrul.my.id`, dokumentasi hanya saat check-in, aturan sesi baru setelah 2 jam,
  dan antrean offline.
- Wizard rencana (4 langkah), hasil (3 langkah), dan output (3 langkah) mendukung simpan draft,
  lampiran/bukti, serta ajukan ke atasan. Atasan dapat menyetujui/merevisi/menolak rencana dan
  memverifikasi hasil/output dengan slider skor.
- Profil menyediakan foto, edit nama/pangkat/jabatan/telepon/email, ganti password dengan
  tombol mata, dan logout. Beranda menampilkan nilai kinerja periode berjalan.
- Keterbatasan dibanding Android: tidak ada pengingat notifikasi latar (WorkManager) dan alamat
  server mengikuti origin aplikasi, bukan dapat diubah pengguna.

## 6. Kebutuhan non-fungsional

| Aspek | Target |
|---|---|
| Waktu respons API | < 300 ms (p95) untuk endpoint daftar pada 145 personel/periode |
| Perhitungan periode | < 60 detik untuk 200 personel |
| Keamanan | Argon2id, JWT HS256, RBAC per satker, audit log, rate limit, HTTPS di produksi |
| Privasi | Tidak ada token di localStorage browser; berkas diunduh dengan bearer token |
| Perangkat | Android 8.0+ (minSdk 26); web personel pada Chrome/Edge/Firefox/Safari mobile dan desktop; dashboard Chrome/Edge/Firefox terbaru |
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
