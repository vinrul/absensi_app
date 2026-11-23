# PRD — Aplikasi Absensi & Strong Point Personel Polda Bali

Versi: 1.2 · Status: MVP terimplementasi · Fokus aktif: **absensi kehadiran + strong point**
(modul kinerja/periode tetap ada di kode tetapi dinonaktifkan dari UI)

## 1. Latar belakang

Penilaian kinerja personel di lingkungan Ditreskrimsus Polda Bali masih dilakukan manual:
rekap kehadiran terpisah dari laporan rencana kerja, hasil kerja, dan bukti output,
sehingga penilaian tidak jarang bersifat subjektif dan sulit dipertanggungjawabkan.

Aplikasi ini menyatukan **absensi kehadiran** dan **strong point** (perintah hadir) dalam satu
alur digital: titik per satker, jadwal bulanan per sesi, penempatan anggota tetap, serta rekap
telat/pulang cepat per sesi. Modul penilaian kinerja (bobot
20/20/30/30, rencana/hasil/output/periode) masih tersimpan di kode dan database, tetapi
**menu serta navigasinya dinonaktifkan** sampai ada keputusan produk berikutnya.

Laporan absensi dan strong point kini memakai **rentang tanggal** (`dari`–`sampai`), bukan
periode bulanan.

## 2. Tujuan & metrik keberhasilan

| Tujuan | Metrik |
|---|---|
| Kehadiran tercatat objektif | ≥ 95% personel absen melalui aplikasi; penyimpangan lokasi < 2% |
| Strong point terlaksana sesuai jadwal | ≥ 95% sesi pagi/sore memiliki check-in & check-out dalam jendela waktu |
| Kepatuhan jam sesi terukur | 100% check-in/check-out menghasilkan `menit_telat`/`menit_pulang_cepat` yang dapat ditelusuri |
| Laporan siap pakai | Rekap absensi & strong point per rentang tanggal dapat diekspor PDF tanpa rekap manual |
| (Dorman) Penilaian transparan | 100% nilai dapat ditelusuri ke `penilaian_detail` bila modul diaktifkan kembali |

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
| **Personel** | Data sendiri | Absen, strong point (check-in/out per sesi), riwayat; modul kinerja dorman |
| **Atasan Langsung** | Satker + bawahan | Semua di atas untuk dirinya + verifikasi absensi/PH, mengelola titik dinas; modul kinerja dorman |
| **Admin Sistem** | Semua | Seluruh master, akun admin, audit log; periode/penilaian dorman |
| **Admin Polda** | Polda + seluruh turunannya | Kelola satker/personel, titik strong point level Polda/Polres, jadwal & penempatan |
| **Admin Satker Polda** | Satu satker di level Polda | Kelola personel, jadwal & penempatan satkernya |
| **Admin Polres** | Polres + satker di bawahnya | Kelola satker/personel, titik strong point level Polres, jadwal & penempatan |
| **Admin Satker Polres** | Satu satker di level Polres | Kelola personel, jadwal & penempatan satkernya |
| **Pimpinan Polda** | Polda + seluruh turunannya | Dashboard kehadiran, rekap, laporan |
| **Pimpinan Satker Polda** | Satu satker di level Polda | Dashboard & laporan satkernya |
| **Pimpinan Polres** | Polres + satker di bawahnya | Dashboard & laporan lingkungan Polres |
| **Pimpinan Satker Polres** | Satu satker di level Polres | Dashboard & laporan satkernya |
| **Pimpinan** | Sesuai cakupan yang diberikan | Sama dengan pimpinan ber-scope |

Aturan teknis: setiap query difilter `satker_id = ANY(cakupan)`; cakupan dihitung dari
`personel_role.scope_satker_id` (rekursif ke turunan). Personel biasa hanya menerima baris
miliknya pada daftar absensi/penilaian dan tidak dapat mengakses dashboard pimpinan.

## 4. Ruang lingkup

**Termasuk (MVP aktif):**
- Autentikasi NRP (personel) atau username (admin) + password, refresh token, ganti password
- Master organisasi 3 level (Polda → Polres → Satker) + jam kerja & hari kerja per satker,
  zona waktu (WIB/WITA/WIT) dan jam pulang Sabtu untuk hari kerja setengah hari
- **Kalender hari libur** nasional/cuti bersama dan libur khusus satker; tanggal libur
  otomatis dikeluarkan dari perhitungan absensi (tidak mengubah jadwal strong point)
- 12 peran dengan cakupan satker (admin & pimpinan ber-scope)
- Titik dinas dengan radius (default 100 m), peta OSM + mode satelit + pencarian alamat
- Absensi masuk/pulang: 8 status kehadiran, validasi radius sisi server, foto opsional,
  antrean offline, verifikasi atasan untuk data offline/mock; laporan per rentang tanggal
- **Strong Point v2**: titik milik Polda/Polres, **penempatan tetap satu anggota = satu titik**,
  jadwal disusun **per bulan** (sesi pagi/sore), **salin/fallback jadwal bulan sebelumnya**,
  check-in/out wajib lokasi, telat & pulang cepat, rekap + PDF per rentang tanggal
- **Pengingat jadwal tanpa FCM** (notifikasi lokal WorkManager di Android)
- Dashboard kehadiran harian (filter tanggal + satker) dan rekap absensi/strong point
- Audit log mutasi + pembatasan laju permintaan
- Dokumentasi API (`/docs`, `/api-docs/openapi.json`)

**Dorman (kode & data dipertahankan, menu dinonaktifkan):**
- Rencana kerja, hasil kerja, output, persetujuan/verifikasi kinerja
- Periode bulanan, bobot indikator, perhitungan nilai, kesiapan satker, laporan penilaian

**Belum termasuk:**
- Integrasi SSO/AD Polri, absensi via mesin sidik jari
- Push daring (FCM) — pengingat lokal sudah tersedia; pusat notifikasi web belum ada UI
- Ekspor CSV rekap strong point
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

> **Dorman:** halaman, API, dan tabel tetap ada, tetapi menu Kinerja dihapus dari navigasi
dashboard/Android/web personel. Jangan menambah fitur baru tanpa keputusan produk.

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

> **Dorman** (lihat catatan FR-03).

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

> **Dorman** (lihat catatan FR-03).

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

> **Dorman:** periode bulanan, bobot, dan perhitungan nilai tidak lagi ditampilkan di UI.
> Rekap absensi/strong point memakai rentang tanggal. API & tabel dipertahankan.

- Periode bulanan dengan tanggal mulai/selesai; bobot default 20/20/30/30; total bobot harus 100.
- Perhitungan dapat diulang (idempoten per personel+periode) dan menyimpan komponen mentah.
- Periode tertutup menolak perubahan data baru.

### FR-07 Dashboard & laporan
- Sidebar dashboard memakai menu bergrup: **Dashboard**; **Master Data** (Satker & Jam Kerja,
  Titik Dinas, Hari Libur, Data Personel); **Absensi** (Rekap Kehadiran, Detail Rekap,
  Verifikasi Offline); **Strong Point** (Jadwal, Rekap, Penempatan Anggota, Titik); **Akun Admin**.
  Grup Kinerja/Laporan/Pengaturan serta menu Penilaian, Print Laporan, Kesiapan Periode, dan
  Periode dihapus; route lama dialihkan ke `/dashboard`.
- Dashboard baru: filter **tanggal** + satker, kartu total personel, hadir, belum absensi,
  telat, sesi strong point, serta cuti/izin/sakit; tabel strong point hari itu dan absensi
  masuk/pulang.
- Halaman **Rekap Absensi** (`/absensi/rekap`) memiliki tiga tab dengan filter **rentang tanggal**:
  - **Rekap Kehadiran**: `dari`–`sampai` + satker; kolom hadir, telat (hari), menit telat,
    dinas luar, piket, cuti, izin, sakit, PH, tanpa keterangan, dan menunggu verifikasi.
  - **Verifikasi**: `dari`–`sampai` + satker + personel; absen masuk dan pulang pada tanggal yang
    sama digabung menjadi satu baris, menampilkan kedua jam dan menit telat.
  - **Detail per Orang**: `dari`–`sampai` + satker + personel; riwayat harian dengan pasangan
    jam masuk/pulang, status, menit telat, status radius, kondisi, dan catatan verifikator.
  Setiap tab menyediakan **Export PDF** sesuai filter aktif (kop Polda Bali, A4 landscape)
  memakai Chrome/Edge headless di server.
- Semua tabel daftar berpaginasi; modul besar memakai pagination server-side
  (`page`/`per_page` pada respons daftar).

### FR-08 Audit & operasional
- Setiap POST/PUT/PATCH/DELETE dicatat (aktor, aksi, entitas, entitas_id, IP, user agent, status).
- Admin global dapat menelusuri audit berdasarkan entitas, aktor, dan rentang tanggal.
- Backup harian database + berkas unggahan dengan retensi 30 hari (`deploy/backup.sh`).

### FR-09 PH / Strong Point
- **Titik strong point** hanya dikelola pada level **Polda/Polres** (admin sesuai cakupan);
  satker di bawahnya otomatis memakai titik induk. Setiap titik memiliki nama, alamat,
  koordinat, radius (default 100 m), dan status aktif.
- **Pola mingguan tidak dipakai lagi.** Jadwal disusun langsung per bulan di tab **Jadwal**
  (tanggal/rentang, sesi, jam, regu, anggota). **Titik strong point tidak dipilih di form jadwal**;
  server menentukan titik dari penempatan anggota dan membuat satu jadwal per titik. Filter jadwal
  memakai **bulan berjalan** dengan rentang tanggal 1–akhir bulan; admin dapat memilih bulan lain
  untuk menyusun lebih awal.
- **Penempatan anggota bersifat tetap**: satu anggota hanya memiliki satu titik strong point
  (`unique personel_id`). Admin memilih **satker → titik → checkbox anggota** (tanpa pilihan
  level, termasuk unit Polda/Polres); perubahan titik memakai upsert tanpa menyimpan riwayat
  tanggal. Hanya akun personel aktif yang dapat ditempatkan dan titik harus milik induk
  Polda/Polres satker.
- **Salin & fallback bulanan**: `POST /ph/jadwal/salin` menyalin jadwal bulan sebelumnya ke bulan
  tujuan dengan memetakan hari + urutan minggu. Jika admin lupa menyusun jadwal bulan berjalan,
  sistem otomatis memakai jadwal bulan terakhir yang tersedia dan menandainya
  `disalin_dari_bulan`; bulan yang belum berjalan tidak di-fallback. `GET /ph/jadwal/status`
  memberi peringatan agar jadwal bulan depan disusun sebelum tanggal 1.
- **Satu jadwal = satu sesi dan satu titik.** Check-in dibuka `jam_mulai − 30 menit` dan ditolak
  setelah `jam_selesai`; check-out sesi dapat dilakukan sampai pukul **23:59** hari yang sama.
- **Absensi harian satu pintu**: status Dinas Kantor, Dinas Luar, PH/Strong Point, Piket/Shift,
  Operasi/Pam, Cuti, Izin, dan Sakit. Server menentukan check-in/check-out otomatis dari catatan
  terbuka (scan kedua = check-out tanpa memilih status). Piket/Operasi tanpa jadwal memakai
  toleransi check-out 24 jam + 1 jam; lewat itu siklus lama ditutup tanpa check-out dan scan baru
  menjadi check-in. Dinas Kantor dibuka 30 menit sebelum jam masuk dan ditolak setelah jam pulang;
  Piket wajib radius kantor, Operasi/Pam boleh di luar radius dengan lokasi tetap dicatat.
- **Strong point menggantikan absensi kantor**: bila hari itu ada jadwal strong point, Dinas Kantor
  tidak dapat dipilih. Kehadiran strong point dicerminkan ke absensi harian (`ph`) sehingga
  dihitung hadir, telat, dan pulang cepat pada rekap kehadiran. Jika sesi sore sudah dibuka saat
  sesi pagi belum check-out, check-in sore diprioritaskan dan sesi pagi ditandai tidak check-out.
- **Android hanya memiliki tab Beranda dan Absensi** dengan urutan wizard status → GPS → foto →
  kirim. Tab Strong dihapus; sesi strong point ditampilkan di Beranda dan diproses lewat Absensi.
- Personel hanya dapat check-in/check-out pada jadwal miliknya dan pada tanggal jadwal.
- Check-in **dan check-out wajib menyertakan koordinat**; server memvalidasi radius titik.
- Server mencatat `menit_telat` (check-in setelah jam mulai) dan `menit_pulang_cepat`
  (check-out sebelum jam selesai); rekap menjumlahkan keduanya per personel.
- Absensi PH memiliki `client_uuid` untuk idempotensi dan dapat diverifikasi atasan/admin.
- Rekap PH memakai **rentang tanggal** dan menampilkan jumlah jadwal, check-in, hanya check-in,
  tidak hadir beserta tanggal, menunggu verifikasi, total telat, dan total pulang cepat.
  Perhitungan dibatasi sampai hari ini.
- Jadwal dan rekap dapat diekspor **PDF** (A4 landscape); PDF jadwal berupa tabel
  Tanggal/Sesi/Jam/Titik/Regu/Anggota.
- **Hari libur nasional tidak mengubah jadwal strong point**; jadwal mengikuti tanggal yang
  ditetapkan admin satker.
- Strong point dan absensi kantor tetap terpisah namun terkait penilaian: hari ber-jadwal strong
  point tetap wajib absensi kantor; telat kantor diabaikan bila check-in strong point, pulang
  cepat tetap dihitung.
- Di aplikasi Android & web personel, halaman **Strong** menampilkan sesi hari ini dengan status
  per sesi dan tombol check-in/check-out kondisional; antrean offline tersedia. Dokumentasi foto
  diminta saat check-in (dapat dilewati bila kamera tidak tersedia).
- Absensi PH memakai kebijakan lokasi tiruan yang sama dengan absensi harian: `flag` menandai
  `perlu_verifikasi`, `reject` menolak HTTP 422.

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
- Pangkat, satker, dan atasan langsung dipilih lewat **combobox ber-pencarian** (ketik untuk
  menyaring tanpa scroll); pangkat memuat 22 tingkat Polri atau 17 pangkat ASN/PNS.
- Unit dan email tidak lagi diminta pada form personel; data lama tetap ditampilkan bila ada.
- Atasan langsung wajib berada pada satker yang sama dan berstatus aktif.
- Peran admin tidak dapat diberikan lewat halaman Data Personel; gunakan halaman Akun Admin.

### FR-17 Titik dinas berbasis peta
- Admin dan atasan mengelola titik dinas per satker (nama, jenis kantor/luar, koordinat, radius).
- Form tambah/edit menampilkan peta yang **lebih tinggi** dengan dua mode: **Peta** (OpenStreetMap)
  dan **Satelit** (ArcGIS World Imagery).
- Pencarian alamat memakai **Nominatim** tanpa tombol Cari: debounce 700 ms, minimal 3 karakter,
  cache hasil, dan maksimal 1 permintaan/detik; memilih hasil menggeser peta dan mengisi koordinat.
- Koordinat dapat diketik manual atau dipilih dengan klik peta dan digeser melalui penanda.
- Lingkaran radius tampil di peta dan ikut berubah saat radius diubah; nilai valid 10–5000 meter.
- Modal titik dinas diperlebar agar peta dan hasil pencarian nyaman dipakai.
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
- Navigasi bawah **3 tab**: Beranda, Absensi, Strong. Desain mobile-first dan responsif,
  berbeda dari dashboard (tanpa sidebar desktop). Halaman Rencana/Hasil/Output/Persetujuan masih
  ada di disk tetapi tidak dijangkau navigasi.
- Wizard izin browser menjelaskan lokasi, kamera, notifikasi, dan antrean offline sekali
  setelah login pertama.
- Absensi memakai GPS perangkat, peta OpenStreetMap, validasi radius ke titik dinas terdekat,
  8 status kehadiran, foto opsional, serta antrean offline di penyimpanan browser.
- Strong Point menampilkan daftar sesi hari ini (pagi/sore) dan tombol per jadwal, riwayat 30 hari
  dengan telat/pulang cepat, peta lokasi tanpa titik dinas, koordinat, akurasi, reverse-geocode
  `geocode.vinrul.my.id`, dokumentasi hanya saat check-in, dan antrean offline.
- Profil menyediakan foto, edit nama/pangkat/jabatan/telepon/email, ganti password dengan
  tombol mata, dan logout. Kartu nilai kinerja di Beranda dihapus.
- Atasan masih dapat memakai halaman persetujuan/verifikasi yang lama bila dibuka langsung, tetapi
  fitur tersebut tidak lagi menjadi fokus aplikasi.
- Keterbatasan dibanding Android: tidak ada pengingat notifikasi latar (WorkManager) dan alamat
  server mengikuti origin aplikasi, bukan dapat diubah pengguna.

### FR-21 Rekap kehadiran matriks per tanggal
- Tab **Rekap Kehadiran** menampilkan matriks: satu baris per personel dan satu kolom per tanggal.
- Rentang tanggal maksimal **31 hari**; rentang lebih panjang ditolak di klien dan server.
- Setiap sel memuat status, jam check-in/check-out, menit telat, dan menit pulang cepat.
- Badge status dan penanda ketidaklengkapan ditampilkan terpisah, misalnya **DK TC** atau
  **PH TC**. **TC** hanya muncul setelah tanggal berganti (check-out masih dapat dilakukan
  sampai 23.59); shift Piket/Operasi memakai toleransi 25 jam. Pada hari berjalan, check-in
  tanpa check-out cukup menampilkan status dan jam check-in.
- Sel hari kerja efektif tanpa catatan absensi ditandai **TA** (tidak absen), termasuk hari
  ini sebelum check-in. Hari libur, akhir pekan di luar hari kerja, dan tanggal mendatang
  tidak ditandai TA.
- Sesi strong point **pagi dan sore** ditampilkan terpisah beserta jam jadwal dan realisasi
  check-in/check-out. Bila ada jadwal strong point tetapi tidak ada check-in/check-out,
  sel menampilkan badge **PH** + **TA**. Bila status absensi Izin/Sakit/Dinas Luar/Cuti,
  badge status tetap tampil bersama penanda **Jadwal PH**.
- Sel yang memiliki absensi dapat diklik untuk membuka modal detail berisi status, jam,
  telat/pulang cepat, foto (jika ada), alamat hasil reverse geocode, dan peta lokasi.
- Hari strong point memakai jam dan koordinat PH.
- Tersedia **Export PDF Ringkasan** (agregat per personel) dan **Export PDF Matriks Harian**
  (maks. 31 hari, dibagi per minggu agar terbaca).

### FR-22 Jam server realtime
- Endpoint publik `GET /api/v1/waktu` mengembalikan `epoch_ms`, tanggal/jam WITA, zona, dan status
  jam palsu; endpoint ini tidak mengakses database.
- Dashboard, web personel, dan Android menampilkan jam server WITA dengan detik dan berdetak lokal.
- Klien menghitung offset dari satu permintaan, lalu resync setiap 5 menit (15 detik saat jam palsu)
  serta saat tab/aplikasi kembali aktif. Tidak memakai WebSocket/SSE maupun polling per detik.

### FR-23 Rekap harian ringkas
- Tab **Rekap Harian** menampilkan ringkasan satu tanggal dengan filter **tanggal** (default hari
  ini, tombol **Hari Ini**) dan **satker**.
- Kartu ringkasan: **Tidak Absen**, **Jadwal PH** (dengan rincian **Sudah PH** dan **Belum PH**),
  Dinas Kantor, Dinas Luar, Piket/Shift, Operasi/Pam, Cuti, Izin, dan Sakit.
- Jumlah pada kartu dihitung dari data matriks satu hari; kartu Jadwal PH menampilkan total
  personel yang punya jadwal sekaligus berapa yang sudah dan belum check-in PH.
- Klik kartu membuka modal berisi daftar personel pada kategori tersebut, lengkap dengan status,
  jam, sesi strong point, penanda TA/TC, dan tombol **Detail** ke modal absensi penuh.
- Rekap Harian memiliki sub-tab **Ringkasan** dan **Peta**. Peta menampilkan marker lokasi
  **check-in** setiap personel dengan warna sesuai status, titik dinas beserta radius, dan strong
  point beserta radius. Basemap dapat diganti **Jalan/Satelit**, dengan legenda warna dan popup
  detail (nama, NRP, satker, status, jam check-in, sesi strong point). Titik yang ditampilkan
  hanya yang aktif dan sesuai filter satker.

## 6. Kebutuhan non-fungsional

| Aspek | Target |
|---|---|
| Waktu respons API | < 300 ms (p95) untuk endpoint daftar pada 145 personel/rentang tanggal |
| Perhitungan laporan | < 60 detik untuk 200 personel per rentang tanggal |
| Keamanan | Argon2id, JWT HS256, RBAC per satker, audit log, rate limit, HTTPS di produksi |
| Privasi | Tidak ada token di localStorage browser; berkas diunduh dengan bearer token |
| Perangkat | Android 8.0+ (minSdk 26); web personel pada Chrome/Edge/Firefox/Safari mobile dan desktop; dashboard Chrome/Edge/Firefox terbaru |
| Offline | Absensi tersimpan lokal dan tersinkron otomatis (WorkManager, 30 menit / manual) |
| Kapasitas awal | 1 Polda + 9 Polres, ±200 personel, jadwal strong point bulanan per satker |

## 7. Batasan & asumsi

- Satu server on-prem (LAN) tanpa autoscaling; pembatas laju berbasis memori satu instans.
- Radius titik dinas dan jam kerja ditetapkan per satker (default 100 m, 08:00–16:00 WITA).
- Jadwal strong point disusun per bulan; perubahan hanya memengaruhi jadwal yang dibuat setelahnya,
  bukan jadwal/absensi lama.
- Bobot indikator & periode bulanan tetap tersimpan, tetapi tidak dipakai UI sampai modul kinerja
  diaktifkan kembali.
- Foto absensi maksimal 5 MB (dikompres di perangkat), lampiran 10 MB, bukti output 50 MB.

## 8. Risiko produk

| Risiko | Mitigasi |
|---|---|
| GPS melenceng di dalam gedung | toleransi akurasi + eskalasi verifikasi atasan |
| Titik dinas belum diatur | API menolak dengan pesan jelas; admin wajib menyiapkan titik sebelum go-live |
| Personel tidak menjalankan strong point | rekap menandai tidak hadir/hanya check-in + total telat/pulang cepat |
| Periode/penilaian dihitung sebelum data lengkap | modul dinonaktifkan dari UI; API tetap idempoten bila diaktifkan kembali |

## 9. Roadmap lanjutan

1. Pusat notifikasi web + pengingat jadwal strong point di dashboard; FCM bila perlu lintas perangkat.
2. Ekspor CSV rekap strong point.
3. Uji lapangan penempatan tetap, salin jadwal bulanan, fallback, dan toleransi jendela sesi
   (−30/+60 menit).
4. Ekspor PDF server-side dengan kop resmi & QR verifikasi.
5. Redis untuk rate limit/session bila API di-scale.
6. Integrasi SI SDM Polri untuk sinkronisasi data personel.
7. Keputusan produk atas modul kinerja/periode yang kini dorman (aktifkan kembali atau bersihkan).
