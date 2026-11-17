# Catatan Serah Terima (untuk agent AI / pengembang berikutnya)

Dokumen ini merangkum kondisi proyek, keputusan teknis, konvensi, dan pekerjaan yang tersisa.
Baca ini dulu sebelum mengubah kode.

Terakhir diverifikasi: 17 November 2026 (backend unit test + 48 cek API pada DB uji;
dashboard `bun run check`/`build`; APK debug; `openapi.json` 83 path).

Checklist pekerjaan & sisa backlog: [`CHECKLIST.md`](CHECKLIST.md).

---

## 1. Status

**Selesai & terverifikasi**

| Area | Bukti |
|---|---|
| Skema DB + seed demo (1 Polda, 9 Polres, 21 Satker, 145 personel + 5 admin) | `cargo run --bin seed -- --reset`; lampiran contoh PDF/PNG ikut ditulis ke `APP__UPLOAD_DIR` |
| Struktur 3 level (polda → polres → satker) + 12 peran ber-scope | `service/akses.rs`, migrasi `0003`/`0005` |
| Akun admin terpisah secara logis (`jenis_akun=admin`, username, tanpa NRP) | migrasi `0007`, halaman **Akun Admin** |
| Login berkanal: admin hanya web, Android hanya akun personel | `routes/auth.rs`, `KanalLogin` |
| CRUD satker + jam kerja + hari kerja (zona waktu & Sabtu setengah hari) | halaman **Satker & Jam Kerja** |
| Titik dinas dengan peta OpenStreetMap interaktif (tambah/edit, radius, koordinat) | `PetaPicker.svelte` (Leaflet), halaman **Titik Dinas** |
| Periode bulan berjalan dibuat otomatis + dapat dibuka kembali | `repo/periode.rs::pastikan_bulan_ini` / `buka`; alasan tersimpan |
| Kesiapan satker per periode (admin satker menandai, admin global memantau) | tabel `periode_satker_kesiapan`, halaman **Kesiapan Periode** |
| Kalender hari libur nasional & khusus satker | tabel `hari_libur`, halaman **Hari Libur**, otomatis netral di penilaian |
| CRUD personel + atasan + peran/cakupan + reset password | halaman **Data Personel**; form tanpa unit/email, pangkat combobox 21 tingkat |
| Pagination seragam di seluruh modul daftar | komponen `Pagination.svelte`; lihat CHECKLIST §3.24 |
| CRUD akun admin + username + cakupan + reset password | halaman **Akun Admin** (`/admin`) |
| Modul PH / Strong Point terpisah | migrasi `0006`, `/ph/*`, halaman **PH & Strong Point** |
| Jam server palsu development (`fake`) | `/dev/waktu` + `cargo run --bin fake --`; hanya aktif dengan `APP__ALLOW_FAKE_TIME=true` |
| Absensi Android bertahap + peta OSM | tombol besar → validasi GPS/peta → status → kirim/offline; `PetaAbsensi.kt` |
| Wizard izin pertama kali sebelum login | `WizardIzinScreen.kt` + DataStore `wizard_izin_selesai` |
| Absensi PH check-in/out wajib lokasi + rekap PH | e2e: GPS kosong 422, di luar radius 422 |
| Pengingat jadwal PH tanpa FCM (WorkManager) | `sync/Pengingat.kt`, H-1 19:00 & 60 menit sebelum |
| Logo Polda Bali di dashboard, cetak, Android (login terpusat), ikon launcher | login/sidebar/top bar/launcher; password dapat ditampilkan/disembunyikan |
| Auth (JWT + refresh rotasi), RBAC per satker | cek e2e: personel hanya melihat data sendiri, dashboard 403 |
| Absensi + geofence + antrean offline | e2e: dalam radius 200, luar radius 422, duplikat 409 |
| Rencana kerja + persetujuan + lampiran | e2e alur penuh + wizard Android 4 langkah (jenis → dasar → rincian kegiatan → lampiran); seed menyediakan contoh draft/diajukan/perlu revisi |
| Hasil kerja + verifikasi | e2e API + wizard Android 3 langkah (pilih rencana → realisasi → bukti), draf, kirim, perbaikan; progres memakai slider 0–100 |
| Output + verifikasi | e2e API + wizard Android 3 langkah (pilih hasil terverifikasi → data → berkas), draf, kirim, revisi; status `draft` lewat migrasi `0014` |
| Mesin penilaian 20/20/30/30 + kategori | unit test `scoring.rs` + hari PH dikecualikan |
| Dashboard pimpinan + cetak + CSV | UI diverifikasi di browser; Rencana/Hasil/Output memiliki halaman detail + lampiran, filter periode/status/satker/personel, dan **Export PDF** lewat Chrome/Edge headless |
| Rekap Absensi bertab + Export PDF | `/absensi/rekap` memiliki tab **Rekap Kehadiran**, **Verifikasi**, dan **Detail per Orang** dengan filter terpisah; absen masuk/pulang digabung per tanggal lewat `GET /absensi/harian` (beserta menit telat Dinas Kantor); filter subdit/bagian dihapus; PDF `/absensi/rekap/ekspor`, `/absensi/verifikasi/ekspor`, `/absensi/detail/ekspor` (uji API) |
| Aplikasi Android (login, absensi GPS, offline, wizard rencana/hasil/output, persetujuan & verifikasi atasan, profil + foto + ganti password, penampil lampiran) | APK debug terbentuk |
| Audit log mutasi | `GET /audit` mencatat `PATCH /api/v1/satker/<uuid>` |
| Rate limit | login ke-4 dalam 1 menit → 429 |
| Dokumentasi API | `/docs` (83 path), `/api-docs/openapi.json` |
| Backup script | `deploy/backup.sh` (belum dijadwalkan di server) |

**Belum / pekerjaan lanjutan** — lihat §9 backlog.

---

## 2. Struktur & perintah

```
backend/    Rust 1.99, axum 0.8.9, sqlx 0.9, PostgreSQL 18
dashboard/  Svelte 5.57, SvelteKit 3.2, Tailwind 4.3, Vite 8.3, Leaflet 1.9 (peta OSM) — kelola dengan **bun**
android/    Kotlin 2.4.21, AGP 9.0.1, Gradle 9.3, Compose BOM 2025.06, OkHttp 4.12
deploy/     docker compose (db :5433), nginx, backup.sh
docs/       PRD.md, penilaian.md, UAT.md, CHECKLIST.md, HANDOFF.md (dokumen ini)
tools/uji/  api-smoke.js (e2e DB uji), openapi-check.js
```

```powershell
# Database + API (demo)
cd deploy; podman compose up -d db           # atau: docker compose up -d db
cd ..\backend; cargo run --bin seed -- --reset; cargo run

# Jam server palsu (development; .env harus APP__ALLOW_FAKE_TIME=true)
cargo run --bin fake -- 20-11-2026 07:45
cargo run --bin fake -- status
cargo run --bin fake -- reset

# Database uji terpisah (untuk e2e/unit test)
# DATABASE_URL=postgres://kinerja:kinerja_dev@127.0.0.1:5433/kinerja_test
cd backend; $env:DATABASE_URL="postgres://kinerja:kinerja_dev@127.0.0.1:5433/kinerja_test"; sqlx migrate run
cd ..; bun tools/uji/api-smoke.js              # spawn API di :8091

# Dashboard (Bun)
cd dashboard; bun install; bun run dev      # http://localhost:5173
bun run check; bun run build                # type-check + build statis ke build/

# Android
cd android; .\gradlew.bat :app:assembleDebug
adb install -r app\build\outputs\apk\debug\app-debug.apk

# Test backend
cd backend; cargo test; cargo clippy --all-targets -- -D warnings
```

Akun seed:

| Peran | Username / NRP | Password |
|---|---|---|
| Admin global | `superuser` | `Superuser#2026` |
| Admin Polda | `admin_polda` | `AdminPolda#2026` |
| Admin Satker Polda | `admin_satker_polda` | `AdminSatker#2026` |
| Admin Polres | `admin_polres` | `AdminPolres#2026` |
| Admin Satker Polres | `admin_satker_polres` | `AdminSatker#2026` |
| Pimpinan Polda | `000002` | `Pimpinan#2026` |
| Pimpinan Polres | `000022` | `PimpinanPolres#2026` |
| Pimpinan Satker Polda | `000012` | `PimpinanSatker#2026` |
| Pimpinan Satker Polres | `000023` | `PimpinanSatker#2026` |
| Atasan | `000100` | `Atasan#2026` |
| Personel | `85000001` | `Polri#2026` |

Admin hanya dapat login lewat dashboard (`client=web`). Android hanya menerima akun
personel (termasuk pimpinan/atasan); akun admin ditolak dengan HTTP 403.

---

## 3. Keputusan teknis & alasan (jangan diubah tanpa alasan kuat)

1. **sqlx 0.9 butuh SQL literal `'static`.** API baru `SqlSafeStr` menolak `&String`/`format!`.
   Semua query ditulis literal; filter opsional memakai pola `($n::tipe IS NULL OR kolom = $n)`.
2. **Numeric → f64.** Kolom `numeric` di-cast `::float8` saat SELECT dan di-bind `$n::float8::numeric`
   saat INSERT. Jangan menambah `rust_decimal` tanpa memigrasikan seluruh repo/penilaian.
3. **Refresh token juga dikirim di body.** Browser memakai cookie httpOnly; Android menyimpan
   `refresh_token` di DataStore dan mengirimnya di body `POST /auth/refresh`.
4. **SvelteKit 3**: `svelte.config.js` **tidak dipakai**; adapter static dikonfigurasi di
   `vite.config.ts` lewat `sveltekit({ adapter })`. Alias `$lib` dihapus → gunakan `#lib/...`
   (subpath import di `package.json`, plus `paths` di `tsconfig.json` agar type-check lolos).
5. **Tailwind 4** tanpa `tailwind.config.js`: token warna di `src/app.css` (`@theme`).
6. **Chart dibuat manual (SVG)** di `#lib/components/{Donut,TrendChart,CategoryPie}.svelte` —
   sengaja tanpa pustaka chart agar tidak ada dependensi rapuh.
7. **AGP 9.0.1 + Gradle 9.3 + compileSdk 36**, dengan opt-out `android.newDsl=false` dan
   `android.builtInKotlin=false`. AGP 9.5 menuntut Gradle > 9.3; AGP 9.1+ menuntut compileSdk 37
   yang belum tersedia di SDK manager. Jangan naikkan salah satu tanpa mengecek ketiganya.
8. **OkHttp, bukan Ktor.** `ktor-client-core-jvm:3.2.0` berisi nama kelas berspasi → gagal di-dex
   untuk `minSdk 26`. `data/Api.kt` memakai OkHttp + kotlinx.serialization (`SnakeCase`).
9. **Android tanpa Hilt/Room/KSP.** DI manual (`data/Graf.kt`), antrean offline di DataStore
   (JSON) + WorkManager. Ini disengaja untuk menekan risiko versi; lihat backlog bila ingin Room.
10. **Bun untuk dashboard.** `bun install` tidak terpengaruh `NODE_ENV=production` seperti npm
    (yang melewati devDependencies). Jangan kembalikan `package-lock.json`.
11. **PostgreSQL 18**: mount volume harus di `/var/lib/postgresql` (bukan `.../data`).
12. **Audit log berbasis middleware**, bukan dipanggil per handler: setiap POST/PUT/PATCH/DELETE
    dicatat `{metode} {path}` + status. Detail semantik (before/after) belum diisi.
13. **Rate limit** berbasis memori per IP (login 5/menit, umum 120/menit). Untuk >1 instans API,
    ganti dengan Redis.
14. **Nilai indikator tanpa data = 0.** Absensi memakai hari kerja yang sudah lewat saja
    (periode berjalan tidak menghukum hari yang belum terjadi).
15. **Organisasi 3 level.** `satker_level` = `polda` → `polres` → `satker`; satker boleh
    berada langsung di bawah polda atau di bawah polres. Hierarki divalidasi di
    `service/akses.rs::hierarki_sah`; jangan membuat satker melompati level.
16. **Peran ber-scope, bukan peran datar.** `personel_role.scope_satker_id` menentukan
    cakupan admin/pimpinan. `AuthUser` memuat `role_scopes` + `satker_akses` yang dihitung
    per request; jangan menggantinya dengan pengecekan `roles.contains()` saja.
17. **PH/Strong Point terpisah dari absensi harian.** Hari yang punya jadwal PH dikeluarkan
    dari skor absensi (`service/penilaian.rs`), tetapi tetap masuk rekap PH. Check-in dan
    **check-out** sama-sama wajib lokasi & radius agar tidak bisa absen pulang dari rumah.
18. **Pengingat jadwal tanpa FCM.** Android memakai WorkManager + notifikasi lokal
    (H-1 19:00 dan 60 menit sebelum). Tidak ada ketergantungan layanan push eksternal.
19. **Database uji terpisah (`kinerja_test`).** Seed demo ada di `kinerja_db`; skrip e2e
    men-spawn API sendiri di port 8091 dengan `APP__DATABASE_URL` ke `kinerja_test`.
20. **Satu tabel identitas, dua jenis akun.** `personel.jenis_akun` membedakan `admin` dan
    `personel`. Admin memakai `username` (tanpa NRP), personel memakai NRP (tanpa username).
    Ini menjaga seluruh foreign key audit/verifikasi tetap valid tanpa tabel polimorfik.
21. **Kanal login ditegakkan di server.** `client=android` hanya menerima `jenis_akun=personel`
    (termasuk pimpinan/atasan); akun admin ditolak 403. Jangan hanya menyembunyikan tombol
    di aplikasi.
22. **Halaman admin dan personel dipisah.** `/personel` hanya menangani akun personel; peran
    admin ditolak. Akun admin dikelola di `/admin` oleh admin global/Polda/Polres saja.
    Admin Polda hanya boleh membuat `admin_satker_polda`; admin Polres hanya
    `admin_satker_polres`.
23. **Zona waktu dibatasi tiga pilihan** (`Asia/Jakarta`, `Asia/Makassar`, `Asia/Jayapura`)
    dan divalidasi di server, bukan hanya di combo box dashboard.
24. **Sabtu setengah hari memakai `satker.jam_pulang_sabtu`.** Bila hari kerja memuat Sabtu
    dan kolom ini terisi (mis. 10:00/12:00), penilaian absensi hari Sabtu memakai jam pulang
    tersebut. Kolom NULL berarti Sabtu memakai jam pulang biasa.
25. **Periode bulan berjalan dibuat otomatis.** `find_or_current` membuat periode bulan ini
    (bobot default 20/20/30/30) bila belum ada. Penutupan **tetap manual** oleh admin global
    karena harus menunggu verifikasi rencana/hasil/absensi selesai.
26. **Periode yang sudah ditutup dapat dibuka kembali oleh admin global.**
    `POST /periode/{id}/buka` mengembalikan status ke `open` dan menyimpan
    `dibuka_kembali_oleh`, `dibuka_kembali_at`, serta `alasan_buka_kembali` agar perubahan
    status dapat diaudit. Jangan membuka kembali tanpa alasan yang jelas.
27. **Kesiapan satker bersifat manual dan per periode.** Admin melihat jumlah personel,
    yang sudah dinilai, dan status siap ditutup. Admin satker hanya dapat menandai satkernya;
    admin Polda/Polres dapat menandai satker dalam cakupannya; admin global melihat semua.
    Saat periode dibuka kembali, seluruh tanda siap direset agar tidak menyesatkan.
28. **Hari libur adalah kalender, bukan hanya status absensi.** `hari_libur` menyimpan
    tanggal + nama + jenis (`nasional`, `cuti_bersama`, `khusus`). `satker_id NULL` berarti
    berlaku semua satker; terisi berarti khusus satker. Tanggal libur aktif otomatis
    dikeluarkan dari penyebut skor absensi. Status absensi `libur_nasional` tetap ada untuk
    data manual/legacy, tetapi penilaian tidak lagi bergantung pada status itu.

---

## 4. Konvensi kode

**Backend**
- Lapisan: `routes/` (HTTP + validasi ringan) → `repo/` (SQL) → `service/` (aturan bisnis murni).
- Error: `AppError` di `error.rs`; pesan untuk pengguna berbahasa Indonesia, detail internal
  hanya masuk log (`Self::Internal` tidak membocorkan isi).
- Query ke DB selalu lewat `repo/`; route tidak menulis SQL langsung (kecuali agregat dashboard).
- Respons daftar: `{ data, meta: { page, per_page, total } }`. DTO enum memakai `snake_case`.
- Tanggal/waktu: simpan `timestamptz` (UTC), tampilkan WITA (`service/waktu.rs`).
- Tambah endpoint baru: daftarkan di `routes/<modul>.rs` → `router.rs` → `openapi.json`.

**Dashboard**
- Svelte 5 runes (`$state`, `$derived`, `$props`); event memakai `onclick={...}`.
- Komponen di `src/lib/components`, data & tipe di `src/lib/{api,types.ts}`, sesi di
  `src/lib/stores/session.svelte.ts`.
- Semua panggilan API lewat `#lib/api/client` (auto-refresh 401 sekali).
- Gaya visual: kartu putih, navy `#0b2545`, emas `#d4a017`, teks slate.
- `lib/components/Modal.svelte` mengunci tinggi maksimal pada viewport, membuat header/tombol tutup
  tetap terlihat, dan menggulir isi saja. Tombol aksi diletakkan pada snippet `footer`
  (contoh modal **Buat Rencana Kerja**) agar selalu tampil tanpa perlu menggulir.
- Item sidebar **Absensi Saya** dan **Persetujuan** sementara disembunyikan (baris dikomentari di
  `(app)/+layout.svelte`); halaman dan API-nya tetap aktif bila diakses langsung.
- **Ekspor PDF** dashboard: halaman Rencana Kerja, Hasil Kerja, Output, dan **Rekap Absensi** mengirim filter ke
  `GET /{modul}/ekspor` (absensi: `rekap/ekspor`, `verifikasi/ekspor`, `detail/ekspor`) dengan bearer token
  lewat `apiBlob` (`#lib/api/client`), lalu mengunduh blob.
  Backend menyusun HTML lengkap (kop + tabel + footer) via `service/laporan.rs` dan mencetaknya
  dengan `service/pdf.rs` memakai Chrome/Edge `--headless --print-to-pdf`; lokasi browser
  auto-detect atau `APP__CHROME_PATH`.
- **Rekap Absensi** (`/absensi/rekap`) memakai tab lokal tanpa route baru: **Rekap Kehadiran**
  (periode + satker), **Verifikasi** (periode + satker + personel, aksi Setujui + foto), dan
  **Detail per Orang** (periode + satker + personel wajib). Tiap tab punya filter sendiri dan
  memuat data saat dibuka. Tab Verifikasi/Detail memakai `GET /absensi/harian` (satu baris per
  personel per tanggal: masuk & pulang digabung, `menit_telat` hanya untuk Dinas Kantor), rekap
  memakai `GET /absensi/rekap` yang juga mengembalikan total `menit_telat` per personel.
  Penyetujuan dari tab Verifikasi mengulang `POST /absensi/{id}/verifikasi` untuk setiap
  `perlu_verifikasi_ids` pada baris tanggal tersebut.
- **Halaman detail** dashboard: `/rencana-kerja/{id}`, `/hasil-kerja/{id}`, dan `/output/{id}`
  memuat data lengkap + lampiran (`apiBlobUrl` → object URL, dibuka di tab baru). Tautan berasal
  dari judul baris tabel maupun tombol **Detail**.

**Android**
- Satu `KinerjaViewModel` menangani semua state & pemanggilan repo; layar Compose hanya
  memanggil method VM.
- Navigasi: bottom bar 5 tab (Beranda, Absensi, Rencana, Hasil, Output) + Profil lewat ikon
  di top bar (tanpa Navigation-Compose). Tab **Nilai** dan **PH** disembunyikan sementara;
  `NilaiScreen`/`PhScreen` masih ada di kode dan dapat diaktifkan kembali.
- **Persetujuan di Android** (`ui/PersetujuanScreen.kt`) dibuka dari kartu **Persetujuan Rencana**
  di Beranda atau tombol **Tinjau Persetujuan** pada tab Rencana (tanpa tab terpisah).
  Daftar `GET /rencana-kerja/persetujuan` hanya memuat bawahan langsung berstatus `diajukan`,
  dilengkapi dialog detail rincian + lampiran, lalu keputusan Setujui / Minta Revisi / Tolak
  lewat `POST /rencana-kerja/{id}/keputusan`. Catatan wajib saat meminta revisi.
- **Dual role**: tab Rencana memakai filter `personel_id` milik akun yang login, sedangkan tombol
  persetujuan menampilkan bawahan langsung. Dengan begitu akun seperti `000100` tetap dapat
  mengajukan rencana ke `000002` sekaligus memutuskan rencana `85000001`. Pimpinan yang ditunjuk
  sebagai `atasan_id` juga boleh memutuskan (gate backend + menu dashboard diperluas).
- **Penampil lampiran** (`ui/LihatLampiran.kt`): unduh biner lewat `Api.unduh`/`GET /lampiran/raw`
  dengan bearer token, lalu render PDF memakai `PdfRenderer` (maks. 10 halaman) atau tampilkan
  gambar dengan pinch-zoom. Dipakai dari detail persetujuan dan daftar lampiran wizard.
- **Detail rencana** (`DialogDetailRencana`) dapat dibuka dari setiap kartu Rencana, termasuk
  status `disetujui`/`diajukan`/`ditolak` lewat **Lihat Detail Rencana**, serta dari
  draft/revisi lewat **Lihat Detail**.
- **Wizard rencana kerja** (`ui/WizardRencana.kt`): langkah jenis → dasar tugas → rincian kegiatan
  (bisa banyak entri, lihat `rencana_tahapan`) → lampiran multi-berkas. Tombol **Simpan Draf**
  (`ajukan=false`) dan **Ajukan ke Atasan** (`ajukan=true`); lampiran diunggah setelah rencana
  tersimpan karena `POST /lampiran` membutuhkan `owner_id`.
- **Mode ubah** dipakai untuk status `draft`/`perlu_revisi`: detail dimuat lewat
  `GET /rencana-kerja/{id}`, wizard terisi otomatis (termasuk catatan atasan), lalu
  `PATCH /rencana-kerja/{id}` mengganti rincian kegiatan dalam satu transaksi. Tombol
  **Simpan Perubahan** dan **Ajukan Ulang ke Atasan**. Lampiran lama dapat dihapus (dihapus saat
  simpan) dan berkas baru ditambahkan.
- **Wizard hasil kerja** (`ui/WizardHasil.kt`): langkah pilih rencana disetujui → realisasi
  (uraian, slider progres 0–100 dengan default 100%, jumlah realisasi, kendala, tanggal lewat
  kalender) → bukti multi-berkas. **Simpan Draf** menyimpan `ajukan=false`; **Kirim ke Atasan**
  mengunggah bukti lalu `POST /hasil-kerja/{id}/ajukan`. Mode ubah tersedia untuk
  `draft`/`perlu_revisi`.
- **Verifikasi hasil kerja** (`ui/VerifikasiHasilScreen.kt`): daftar bawahan langsung berstatus
  `diajukan`, dialog detail + lampiran, keputusan **Terverifikasi** (skor kualitas lewat slider
  0–100, default 85) atau **Perlu Revisi** (catatan wajib). Dibuka dari kartu Beranda atau tab Hasil.
- **Wizard output** (`ui/WizardOutput.kt`): langkah pilih hasil kerja `terverifikasi` → data output
  (8 jenis, judul, uraian, tanggal, URL untuk `link_sistem`) → berkas bukti. Sama seperti hasil,
  draf disimpan lebih dulu (`ajukan=false`), berkas diunggah, lalu diajukan; jika unggahan gagal
  status tetap draf agar dapat diperbaiki. Hanya `GET /output/{id}` yang mengembalikan `lampiran`.
- **Verifikasi output** (`ui/VerifikasiOutputScreen.kt`): daftar `dalam_verifikasi` dari bawahan
  langsung, detail + lampiran, keputusan **Disetujui (Valid)** dengan skor kesesuaian & keabsahan
  (slider 0–100, default 90/92) atau **Kembalikan dan Revisi** dengan catatan wajib. Kelengkapan
  dihitung server dari berkas/URL.
- **Detail output** (`ui/DetailOutput.kt`) menampilkan status, jenis, tanggal, uraian, tautan,
  komponen nilai, catatan atasan, dan lampiran yang dapat dibuka di penampil internal.
- Status output `draft` ditambahkan lewat migrasi `0014`; seed menyediakan contoh draft, dalam
  verifikasi, perlu revisi, dan disetujui untuk akun demo.
- **Profil & Pengaturan** (`ui/ProfilScreen.kt`): avatar bulat dari `GET /auth/foto/{id}`
  (dimuat ke `vm.fotoProfil`), **Ganti Foto Profil** lewat pemilih gambar sistem (JPG/PNG maks. 5 MB,
  `POST /auth/foto`), **Edit Profil** (nama, pangkat lewat combobox 21 tingkat Polri terendah→tertinggi,
  jabatan, telepon, email via `PATCH /auth/profile`), dan **Ganti Password**
  (`POST /auth/change-password`). Tiap kolom password punya ikon mata tampil/sembunyi (`IkonMata`,
  tanpa dependensi tambahan). Satker/NRP/peran tetap hanya admin. Ganti password memicu
  `bersihkanDataSesi()` sehingga aplikasi kembali ke layar login.
- **Avatar di Beranda** memakai komponen bersama `FotoProfil` dan hanya digambar bila foto berhasil
  diunggah/dimuat (`tampilkanInisial = false`); tanpa foto, salam langsung dimulai dari teks.
- Semua tanggal memakai `FieldTanggal` + `DatePickerDialog` Material 3 (tidak ada input tanggal manual).
- Satu pengajuan wizard = satu baris `rencana_kerja`; rincian kegiatan disimpan sebagai
  `rencana_tahapan` dengan kolom tambahan (migrasi `0013`) agar tetap satu alur persetujuan.
- Filter `/rencana-kerja/persetujuan`: `atasan_id` dan status `diajukan` sekarang digabung dengan
  `AND` (sebelumnya `OR` membuat atasan melihat semua rencana dalam cakupan).
- DTO @Serializable dengan `JsonNamingStrategy.SnakeCase` — jangan menulis `@SerialName` manual.
- Tanggal ISO (`yyyy-MM-dd`), waktu kirim `Instant.now().toString()`.

---

## 5. Peta API (ringkas)

Base `/api/v1`. Semua butuh `Authorization: Bearer` kecuali login/refresh.
Login menerima `username` (admin), `nrp` (personel), dan `client` (`web`/`android`).

```
POST   /auth/login | /auth/refresh | /auth/logout | /auth/change-password
GET    /auth/me          PATCH /auth/profile     POST /auth/foto     GET /auth/foto/{id}
GET    /satker            POST /satker            GET|PATCH /satker/{id}
GET    /admin             POST /admin             GET|PATCH /admin/{id}   POST /admin/{id}/reset-password
GET    /personel          POST /personel          GET|PATCH /personel/{id}
GET|POST /personel/{id}/roles    POST /personel/{id}/reset-password
GET    /titik-dinas       POST /titik-dinas       PATCH|DELETE /titik-dinas/{id}
GET|POST /ph/strong-point PATCH|DELETE /ph/strong-point/{id}
GET|POST /ph/jadwal       GET /ph/jadwal/saya     GET|PUT|DELETE /ph/jadwal/{id}
POST   /ph/absensi        GET /ph/absensi/saya    GET /ph/absensi/jadwal/{id}
POST   /ph/absensi/{id}/verifikasi                 GET /ph/rekap
POST   /absensi           GET  /absensi?dari&sampai&personel_id&perlu_verifikasi   GET /absensi/saya
GET    /absensi/harian?dari&sampai&personel_id&perlu_verifikasi   # masuk+pulang digabung
GET    /absensi/rekap     GET  /absensi/rekap/ekspor      GET /absensi/verifikasi/ekspor
GET    /absensi/detail/ekspor   GET /absensi/titik-terdekat   POST /absensi/{id}/verifikasi
GET    /rencana-kerja     POST /rencana-kerja     GET /rencana-kerja/persetujuan
GET    /rencana-kerja/ekspor?periode_id&satker_id&personel_id&status   # PDF
GET|PATCH /rencana-kerja/{id}     POST /rencana-kerja/{id}/ajukan | /keputusan
GET    /hasil-kerja       POST /hasil-kerja       GET /hasil-kerja/{id}   POST /hasil-kerja/{id}/verifikasi
GET    /hasil-kerja/ekspor?periode_id&satker_id&personel_id&status     # PDF
GET    /output            POST /output            GET|PATCH /output/{id}   GET /output/verifikasi
GET    /output/ekspor?periode_id&satker_id&personel_id&status           # PDF
POST   /output/{id}/ajukan | /verifikasi
POST   /lampiran          GET  /lampiran/raw/{key} DELETE /lampiran/{id}
GET    /periode           POST /periode           PUT /periode/{id}/bobot
POST   /periode/{id}/tutup   POST /periode/{id}/buka
GET    /periode/{id}/kesiapan     POST /periode/{id}/kesiapan/{satker_id}
GET    /libur              POST /libur             PATCH|DELETE /libur/{id}
GET    /penilaian         POST /penilaian/{periode_id}/hitung
GET    /penilaian/personel/{personel_id}          POST /penilaian/{id}/keputusan
GET    /dashboard/{ringkasan,capaian-indikator,tren,sebaran-kategori,rekap}
GET    /laporan/penilaian[?format=csv]
GET    /notifikasi        POST /notifikasi/{id}/baca
GET    /audit             # admin
GET    /health            # di luar /api/v1
GET    /docs              # di luar /api/v1
```

---

## 6. Skema database (ringkas)

`satker` (3 level, +`jam_pulang_sabtu`) · `personel` (+`atasan_id`, `jenis_akun`, `username`; kolom `unit`/`email` lama, tidak diisi UI baru) ·
`personel_role` (+`scope_satker_id`) · `refresh_token` · `titik_dinas` ·
`absensi` (unik personel+tanggal+jenis, `client_uuid` unik) · `strong_point` · `jadwal_ph` ·
`jadwal_ph_personel` · `absensi_ph` · `rencana_kerja` · `rencana_tahapan` · `hasil_kerja` ·
`output` · `lampiran` · `periode` · `bobot_indikator` · `periode_satker_kesiapan` ·
`hari_libur` · `penilaian` (unik personel+periode) · `penilaian_detail` (jsonb komponen) ·
`notifikasi` · `audit_log`.

`personel.nrp` hanya wajib untuk `jenis_akun='personel'`; `personel.username` hanya wajib
untuk `jenis_akun='admin'` (constraint `personel_identitas_check`).

Migrasi: `backend/migrations/0001_init.sql` … `0013_rencana_tahapan_rincian.sql`. Status absensi
`libur_nasional` menggantikan nama lama `ph` (nama `ph` sekarang khusus modul PH).
Tambah migrasi baru = file `000N_nama.sql` lalu `sqlx migrate run` + `cargo sqlx prepare` bila
memakai `query!` (saat ini semua runtime query, jadi `.sqlx` belum diperlukan).

---

## 7. Pengujian

| Lapis | Perintah | Cakupan |
|---|---|---|
| Unit | `cd backend; cargo test` | geofence, scoring, deteksi berkas, kanal login |
| Lint Rust | `cargo clippy --all-targets -- -D warnings` | — |
| API e2e | `bun tools/uji/api-smoke.js` (DB `kinerja_test`, API :8091) | RBAC ber-scope, CRUD satker/personel/admin/titik dinas, kesiapan, libur, PH, jam palsu 48 cek |
| OpenAPI | `bun tools/uji/openapi-check.js` | JSON valid + seluruh `$ref` ada |
| Periode otomatis | `bun tools/uji/periode-auto-check.js` | Hapus periode DB uji → dashboard memicu pembuatan bulan berjalan |
| Type-check UI | `cd dashboard; bun run check` | 0 error |
| Build UI | `bun run build` | SPA statis |
| Android | `.\gradlew.bat :app:assembleDebug` | APK debug |

Jangan menjalankan skrip e2e pada `kinerja_db` (data demo untuk uji manual). Skrip men-spawn
API sendiri di port 8091; pastikan tidak ada `kinerja-api.exe` lain yang mengunci `target/debug`.

---

## 8. Jebakan yang sudah ditemui (hemat waktu)

- `npm install` melewati devDependencies karena `NODE_ENV=production` di shell ini → gunakan
  `bun install` (atau `npm install --include=dev`). `bun.exe` hanya ada di Windows.
- `sqlx::query_as` menolak `String` dinamis (SqlSafeStr) → SQL harus literal.
- `ALTER TYPE ... ADD VALUE` tidak boleh satu transaksi dengan pemakaian nilai baru → pisahkan
  ke migrasi berbeda (contoh `0003` menambah nilai, `0004` me-rename `ph` → `libur_nasional`).
- Unique constraint lama `personel_role` harus di-drop sebelum indeks `NULLS NOT DISTINCT`;
  jika tidak, peran global + ber-scope bentrok.
- CLI `sqlx` membaca `DATABASE_URL`, bukan `APP__DATABASE_URL`; seed/API membaca `APP__*`.
- DB default `.env.example` bernama `kinerja`, padahal container ini memakai `kinerja_db` dan
  DB uji `kinerja_test`.
- `cargo build` gagal bila `kinerja-api.exe` sedang berjalan → matikan dulu proses API.
- Server yang dijalankan lewat `run_in_background` sering tidak menangkap stdout Rust;
  jalankan `cargo build` lalu `target\debug\kinerja-api.exe` langsung. Proses `nohup ... &`
  mati antar tool call → skrip uji memakai `Bun.spawn`.
- Emulator: `adb shell input text` menggandakan karakter pada input tertentu (mis. `#`) —
  ketik bertahap (`Polri`, `'#'`, `2026`) alih-alih satu string.
- Emulator GPS: `adb emu geo fix <longitude> <latitude>` (urutan longitude dulu), lalu
  `pm grant ... ACCESS_FINE_LOCATION`.
- `emulator` butuh `-gpu swiftshader_indirect` di mesin ini agar tidak crash.
- Postgres 18: jangan mount ke `/var/lib/postgresql/data`.
- SvelteKit 3: `svelte.config.js` yang tersisa akan menggagalkan build (`config_file_unsupported`).
- `asset()` dari `$app/paths` menolak path berawalan `/`; logo dashboard memakai `src` absolut.
- `personel.nrp` kini **nullable** untuk akun admin. Setiap query yang menampilkan personel
  wajib menambahkan `jenis_akun = 'personel'`, jika tidak akun admin ikut muncul.
- Jangan mengisi `nrp` untuk akun admin atau `username` untuk personel;
  constraint `personel_identitas_check` akan menolak.
- Personel tanpa `atasan_id` tidak bisa mengajukan rencana — form tambah personel wajib
  memilih atasan satu satka.
- `seed` sekarang menjalankan migrasi sebelum menulis data; jangan hapus langkah itu —
  nilai enum baru (mis. `status_output='draft'`) gagal disisipkan bila skema DB uji tertinggal.
- Alur output Android sengaja membuat baris `draft` lebih dulu, mengunggah berkas, baru
  `POST /output/{id}/ajukan`. Jangan mengubah urutan ini: jika unggahan gagal, draf tetap dapat
  diperbaiki. Daftar verifikasi atasan hanya memuat `dalam_verifikasi`.
- Emulator: jangan menekan BACK untuk menyembunyikan keyboard bila keyboard sudah tertutup —
  aktivitas ikut tertutup. Pindah field dengan `keyevent 61` (TAB), lalu satu BACK saat keyboard pasti tampil.
- Emulator: foto yang di-`adb push` tidak muncul di pemilih foto Android 14 sampai dipindai:
  `adb shell am broadcast -a android.intent.action.MEDIA_SCANNER_SCAN_FILE -d file:///sdcard/Download/<nama>.png`.
- Foto profil disimpan di `upload_dir/foto/<personel_id>/<uuid>.<ext>` dan `personel.foto_path`
  hanya menyimpan kunci; jangan menaruh biner di database.
- Ekspor PDF butuh Chrome/Edge terpasang. Di mesin ini `chrome.exe` dan `msedge.exe` terdeteksi;
  bila tidak, set `APP__CHROME_PATH`. Chrome dijalankan dengan profil sementara di folder temp
  dan seluruh berkas sementara dibersihkan setelah PDF terbentuk.
- `kinerja-api.exe` yang sedang berjalan mengunci `target/debug` → matikan dulu sebelum
  `cargo build` (lihat §8).

---

## 9. Backlog berprioritas

**P1 — sebelum go-live**
1. **Jadwalkan backup** di server + uji restore (`deploy/backup.sh`).
2. **Isi titik dinas & jam kerja riil** untuk seluruh satker; uji radius di lapangan.
3. **HTTPS**: pasang sertifikat, set `APP__COOKIE_SECURE=true`, dan
   `cleartextTrafficPermitted=false` di `network_security_config.xml`.
4. **Ganti password default** seluruh akun seed/demo; hapus data demo bila perlu
   (`--reset` membuat ulang).
5. **Uji perangkat nyata** (minimal 1 Android 8 dan 1 Android 13) untuk GPS + kamera.

**P2 — peningkatan kualitas**
6. Audit log menyimpan `before`/`after` untuk entitas kritikal (personel, periode, bobot).
7. Pusat notifikasi + pengingat jadwal PH di dashboard web.
8. Antrean offline untuk absensi PH.
9. Ekspor CSV rekap PH.
10. Peta pemilihan titik dinas/strong point di dashboard (Leaflet/MapLibre).
11. Ekspor PDF server-side dengan kop & QR verifikasi.
12. `GET /audit` UI di halaman Pengaturan (saat ini hanya API).
13. Rate limit berbasis Redis bila API di-scale.

**P3 — teknis**
14. Pindahkan antrean offline Android ke Room (butuh KSP) agar query lebih kaya.
15. CI (GitHub Actions self-hosted) untuk `api-smoke.js` + `openapi-check.js`.
16. Utoipa/OpenAPI otomatis sebagai pengganti `openapi.json` manual.
17. Uji beban: 200 personel × 1 bulan, target hitung < 60 detik.
18. Push FCM bila kelak butuh notifikasi daring lintas perangkat.

---

## 10. Cara cepat memverifikasi ulang

```powershell
# 1. siapkan data & server
cd deploy; podman compose up -d db        # atau docker compose
cd ..\backend; cargo run --bin seed -- --reset
cargo build; .\target\debug\kinerja-api.exe   # terminal terpisah

# 2. cek cepat
curl http://127.0.0.1:8080/health
curl -X POST http://127.0.0.1:8080/api/v1/auth/login -H "content-type: application/json" -d '{"username":"superuser","password":"Superuser#2026","client":"web"}'
# buka http://127.0.0.1:8080/docs

# 3. UI
cd ..\dashboard; bun install; bun run dev     # login superuser / Superuser#2026

# 4. uji otomatis pada DB terpisah (kinerja_test)
cd ..; bun tools/uji/api-smoke.js
bun tools/uji/openapi-check.js
```
```

Bila mengubah rumus penilaian: perbarui `backend/src/service/scoring.rs`, **tambah unit test**,
lalu sesuaikan `docs/penilaian.md`. Jangan mengubah `nilai_akhir` tanpa memperbarui kategori dan
`penilaian_detail`.

Bila menambah endpoint: perbarui `openapi.json` dan (bila perlu) dokumentasi di `docs/PRD.md`.
