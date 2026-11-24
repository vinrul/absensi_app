# Catatan Serah Terima (untuk agent AI / pengembang berikutnya)

Dokumen ini merangkum kondisi proyek, keputusan teknis, konvensi, dan pekerjaan yang tersisa.
Baca ini dulu sebelum mengubah kode.

Terakhir diverifikasi: 24 November 2026 (40 unit test backend; 89 cek API pada DB uji;
seed `--reset` development menghasilkan 145 personel, 102 jadwal PH, penempatan tetap, dan
fallback jadwal bulan sebelumnya; dashboard & web personel `bun run check`/`build`; APK debug;
`openapi.json` 96 path; peta dashboard diuji Chrome headless).

Checklist pekerjaan & sisa backlog: [`CHECKLIST.md`](CHECKLIST.md).

---

## 1. Status

**Selesai & terverifikasi**

| Area | Bukti |
|---|---|
| Skema DB + seed demo (1 Polda, 9 Polres, 21 Satker, 145 personel + 5 admin) | `cargo run --bin seed -- --reset`; lampiran contoh PDF/PNG ikut ditulis ke `APP__UPLOAD_DIR` |
| Seed demo VPS khusus Ditreskrimsus + panduan deploy native | `cargo run --bin seed -- --reset --vps`: Polda + Ditreskrimsus, absensi/rekap PH 1–19 (tidak melewati hari ini), jadwal PH 1–akhir bulan (Regu 1 1–15, Regu 2 16–akhir), status rencana/hasil/output lengkap; systemd + nginx mTLS di `docs/deploy-vps.md` |
| Struktur 3 level (polda → polres → satker) + 12 peran ber-scope | `service/akses.rs`, migrasi `0003`/`0005` |
| Akun admin terpisah secara logis (`jenis_akun=admin`, username, tanpa NRP) | migrasi `0007`, halaman **Akun Admin** |
| Login berkanal: admin hanya web, Android hanya akun personel | `routes/auth.rs`, `KanalLogin` |
| CRUD satker + jam kerja + hari kerja (zona waktu & Sabtu setengah hari) | halaman **Satker & Jam Kerja** |
| Titik dinas dengan peta OpenStreetMap interaktif (tambah/edit, radius, koordinat) | `PetaPicker.svelte` (Leaflet), halaman **Titik Dinas** |
| Periode bulan berjalan dibuat otomatis + dapat dibuka kembali | Kode & API tetap ada, tetapi **menu/navigasi dinonaktifkan**; laporan kini memakai rentang tanggal |
| Kesiapan satker per periode | Kode & API tetap ada, tetapi **menu dinonaktifkan** |
| Kalender hari libur nasional & khusus satker | tabel `hari_libur`, halaman **Hari Libur**, otomatis netral di penilaian |
| CRUD personel + atasan + peran/cakupan + reset password | halaman **Data Personel**; form tanpa unit/email, pangkat/satker/atasan memakai **Combobox ber-pencarian** (22 tingkat Polri + 17 pangkat ASN) |
| **Filter satker/personel ber-pencarian** | `Combobox.svelte` dipakai di Rekap Kehadiran, Rekap Harian, Verifikasi, Detail per Orang, filter Strong Point, serta modal Jadwal/Penempatan; anggota dapat dicari lewat nama/NRP |
| Pagination seragam di seluruh modul daftar | komponen `Pagination.svelte`; lihat CHECKLIST §3.24 |
| CRUD akun admin + username + cakupan + reset password | halaman **Akun Admin** (`/admin`) |
| Modul PH / Strong Point terpisah | migrasi `0006` + `0017` + `0018`; `/ph/*`; halaman **Strong Point** dengan tab Jadwal, Rekap, Penempatan Anggota, Titik |
| **Strong Point v2** | Titik hanya milik Polda/Polres (satker memakai titik induk); **satu anggota = satu titik**; jadwal disusun per bulan; sesi pagi/sore dengan jam per sesi; telat & pulang cepat per sesi |
| **Jadwal bulanan + fallback** | Filter jadwal per bulan (tanggal 1–akhir bulan); pengingat menyusun bulan depan; `POST /ph/jadwal/salin`; jika lupa, sistem menyalin jadwal bulan sebelumnya secara otomatis (ditandai di UI) |
| Laporan absensi & strong point berbasis **rentang tanggal** | `dari`/`sampai` di `/absensi/*`, `/ph/*`, `/dashboard/kehadiran`; UI tidak lagi memakai periode bulan |
| **Modul kinerja/periode dinonaktifkan dari UI** | Menu Kinerja, Penilaian, Print Laporan, Kesiapan, Periode dihapus dari navigasi; route lama diarahkan ke `/dashboard`; tabel & API dibiarkan dorman |
| Jam server palsu development (`fake`) | `/dev/waktu` + `cargo run --bin fake --`; hanya aktif dengan `APP__ALLOW_FAKE_TIME=true` |
| **Jam server realtime** | `GET /api/v1/waktu` tanpa akses DB; dashboard, web personel, dan Android menghitung offset lalu berdetak lokal tiap detik; resync 5 menit (15 detik saat jam palsu) dan saat tab/aplikasi aktif kembali |
| Absensi Android satu alur + peta OSM | status kehadiran → validasi GPS/peta → foto → kirim/offline; riwayat disusun **satu baris per tanggal**; `PetaAbsensi.kt` |
| Wizard izin pertama kali sebelum login | `WizardIzinScreen.kt` + DataStore `wizard_izin_selesai` |
| Absensi PH check-in/out wajib lokasi + rekap PH | e2e: GPS kosong 422, di luar radius 422; check-in/check-out ganda 409; rekap menandai tidak strong point / hanya check-in serta total telat & pulang cepat |
| Aplikasi Android dua tab + riwayat harian | bottom nav **Beranda / Absensi** (tab Strong dihapus); wizard status → lokasi → foto → kirim; riwayat bulan berjalan memakai **filter tanggal mulai/selesai** (default tanggal 1 s.d. akhir bulan) dan menggabungkan absensi masuk/pulang, status, serta sesi strong point (jadwal + realisasi) dalam satu baris per tanggal |
| Dokumentasi foto absensi & strong point | Unggahan base64 Android disimpan sebagai `foto_path`; galeri dashboard lewat `/absensi/foto` dan `/ph/foto`; berkas `/api/v1/files/{key}` dengan cek pemilik/cakupan satker |
| Pengingat jadwal PH tanpa FCM (WorkManager) | `sync/Pengingat.kt`, H-1 19:00; jadwal tanpa jam diingatkan pagi hari, jadwal berjam tetap 60 menit sebelum |
| Logo **SIAP Bali** di dashboard, cetak/PDF, web personel, Android (login terpusat), favicon, dan ikon launcher | logo transparan tanpa kotak putih di login/sidebar/top bar; judul **SIAP Bali** dengan subjudul **Sistem Informasi Absensi Personel Bali**; password dapat ditampilkan/disembunyikan; nama aplikasi Android `SIAP Bali` (`app_name`); alamat server Android otomatis per build (debug `10.0.2.2`, rilis `absensi.jagradewata.id`) |
| Auth (JWT + refresh rotasi), RBAC per satker | cek e2e: personel hanya melihat data sendiri, dashboard 403; login dashboard hanya role admin/pimpinan (atasan/personel ditolak 403) |
| Absensi + geofence + antrean offline | e2e: dalam radius 200, luar radius 422, duplikat 409 |
| Rencana kerja + persetujuan + lampiran | e2e alur penuh + wizard Android 4 langkah (jenis → dasar → rincian kegiatan → lampiran); seed menyediakan contoh draft/diajukan/perlu revisi |
| Hasil kerja + verifikasi | e2e API + wizard Android 3 langkah (pilih rencana → realisasi → bukti), draf, kirim, perbaikan; progres memakai slider 0–100 |
| Output + verifikasi | e2e API + wizard Android 3 langkah (pilih hasil terverifikasi → data → berkas), draf, kirim, revisi; status `draft` lewat migrasi `0014` |
| Mesin penilaian 20/20/30/30 + kategori | unit test `scoring.rs`; hari strong point tetap dinilai, telat kantor diabaikan bila check-in |
| Dashboard pimpinan + cetak + CSV | Dashboard baru fokus kehadiran & strong point hari ini (filter tanggal + satker); laporan kinerja tetap ada di kode tetapi tidak dijangkau UI |
| **Peta absensi di dashboard** | `PetaDashboard.svelte` menggambar marker absensi berwarna status, marker check-in strong point (lengkap/TC), titik dinas + radius, dan titik strong point + radius; lapisan dapat dimatikan per kategori, basemap Jalan/Satelit, popup detail. Memakai endpoint yang ada (`/absensi/harian`, `/ph/rekap/harian`, `/titik-dinas`, `/ph/strong-point`) tanpa endpoint baru |
| Rekap Absensi bertab + Export PDF | `/absensi/rekap` memiliki tab **Rekap Kehadiran**, **Rekap Harian**, **Verifikasi**, dan **Detail per Orang** dengan filter terpisah; rekap kehadiran memakai **matriks harian** (satu baris per personel, satu kolom per tanggal, maks. 31 hari) berisi status, jam masuk/pulang, telat, pulang cepat; badge status dan **TA/TC/TM** terpisah (mis. DK TC); sesi strong point pagi/sore beserta realisasinya; jadwal PH tanpa absensi tampil PH + TA; sel dapat diklik untuk modal alamat (Nominatim) + peta Leaflet dan rincian sesi. Kartu ringkasan total periode tampil di atas mode Matriks maupun Ringkasan (klik kartu membuka rincian per personel). Tersedia **Export PDF Ringkasan** dan **Export PDF Matriks Harian** (PDF matriks dibagi per minggu); PDF verifikasi/detail tetap ada |
| **Rekap Harian ringkas** | Tab **Rekap Harian** dengan filter tanggal (default hari ini) + satker; sub-tab **Ringkasan** berisi kartu **Tidak Absen**, **Jadwal PH** (rincian **Sudah PH**/**Belum PH**), DK, DL, Piket, Operasi, Cuti, Izin, dan Sakit; sub-tab **Peta** menampilkan marker check-in berwarna status, titik dinas + radius, strong point + radius, basemap Jalan/Satelit, dan legenda. Klik kartu membuka daftar personel + tombol Detail; memakai matriks satu hari dan endpoint titik yang ada, tanpa endpoint baru |
| Profil Saya di dashboard | `/profil` (admin & pimpinan): unggah foto JPG/PNG maks. 5 MB (`POST /auth/foto`), avatar bersama `Avatar.svelte` di sidebar + header, ganti password (`POST /auth/change-password`) dengan tombol mata; sukses memicu logout dan redirect `/login` |
| Aplikasi web personel & atasan | Folder `personel/` (SvelteKit SPA, Tailwind, Bun, port 5174): Beranda, Absensi GPS + peta OSM + antrean offline, Strong Point (daftar sesi hari ini + wizard), riwayat; halaman kinerja masih ada di disk tetapi tidak dijangkau navigasi; login memakai kanal `personel_web` |
| Deteksi lokasi tiruan (fake GPS) | Android memeriksa `Location.isMock`/`isFromMockProvider` + peringatan Developer Options/mock app; server `APP__MOCK_LOCATION_POLICY=flag|reject` (default `flag`, produksi `reject`); antrean mencatat penolakan; panduan `docs/fake-gps.md` |
| Strong Point: rekap + Export PDF | Rekap per rentang tanggal (default awal bulan s.d. hari ini, maks. 31 hari) dengan sub-tab **Ringkasan** (per personel) dan **Matriks Harian** (baris personel berjadwal, kolom tanggal; sel memuat sesi/jam/titik, melakukan, telat, pulang cepat, TC, TA); tab **Rekap Harian** (satu tanggal + satker) berisi kartu ringkasan (personel, sesi, titik, hadir, belum, TC, telat, pulang cepat; klik kartu → daftar personel) dan **Peta** (titik strong point + radius, marker check-in lengkap/TC, Jalan/Satelit); klik nama personel membuka **detail per personel** (`?tab=rekap&personel_id=`, kartu ringkasan + rincian per jadwal + tombol **Export PDF Detail** via `/ph/rekap/personel/{id}/ekspor`); tombol **Export PDF Matriks** memakai `/ph/rekap/harian/ekspor` (A4 landscape, dibagi per minggu, hanya personel yang berjadwal di minggu tsb); PDF `/ph/jadwal/ekspor` berupa tabel Tanggal/Sesi/Jam/Titik/Regu/Anggota dan `/ph/rekap/ekspor` (uji API + Chrome headless) |
| Aplikasi Android (login, absensi GPS, offline, wizard rencana/hasil/output, persetujuan & verifikasi atasan, profil + foto + ganti password, penampil lampiran) | APK debug terbentuk |
| Audit log mutasi | `GET /audit` (admin global & admin Polda) + halaman **Pengaturan → Audit Log** di dashboard; mencatat mis. `PATCH /api/v1/satker/<uuid>` dengan status respons |
| Rate limit | login ke-4 dalam 1 menit → 429 |
| Dokumentasi API | `/docs` (96 path), `/api-docs/openapi.json` |
| Backup script | `deploy/backup.sh` (belum dijadwalkan di server) |

**Belum / pekerjaan lanjutan** — lihat §9 backlog.

---

## 2. Struktur & perintah

```
backend/    Rust 1.99, axum 0.8.9, sqlx 0.9, PostgreSQL 18
dashboard/  Svelte 5.57, SvelteKit 3.2, Tailwind 4.3, Vite 8.3, Leaflet 1.9 (peta OSM) — kelola dengan **bun**
personel/   Aplikasi web personel & atasan (SPA mobile-first) — Svelte 5, SvelteKit 3, Tailwind 4, Leaflet, **bun**
android/    Kotlin 2.4.21, AGP 9.0.1, Gradle 9.3, Compose BOM 2025.06, OkHttp 4.12
deploy/     docker compose (db :5433, dashboard :8088, personel :8089), nginx, backup.sh
docs/       PRD.md, penilaian.md, UAT.md, CHECKLIST.md, HANDOFF.md (dokumen ini), deploy-vps.md
tools/uji/  api-smoke.js (e2e DB uji), openapi-check.js
```

```powershell
# Database + API (demo)
cd deploy; podman compose up -d db           # atau: docker compose up -d db
cd ..\backend; cargo run --bin seed -- --reset; cargo run
# profil VPS (hanya Ditreskrimsus; jangan pada DB produksi berisi data):
# cargo run --bin seed -- --reset --vps

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

# Aplikasi web personel (Bun)
cd personel; bun install; bun run dev       # http://localhost:5174 (proxy /api → :8080)
bun run check; bun run build                # build statis ke personel/build

# Android
cd android; .\gradlew.bat :app:assembleDebug
adb install -r app\build\outputs\apk\debug\app-debug.apk

# Android release bertanda tangan (APK + AAB; butuh android/keystore.properties)
cd android; .\build-release.bat

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

Untuk database produksi tanpa data demo, buat hanya akun admin global lewat
`backend/sql/seed_admin_global.sql` (idempoten, menyetel sandi `Superuser#2026` dan role
`admin_sistem`).

Admin hanya dapat login lewat dashboard (`client=web`). Android dan web personel hanya menerima
akun personel (termasuk pimpinan/atasan); akun admin ditolak dengan HTTP 403. Web personel
mengirim `client=personel_web` dan menyegarkan sesi lewat cookie httpOnly.

---

## 3. Keputusan teknis & alasan (jangan diubah tanpa alasan kuat)

1. **sqlx 0.9 butuh SQL literal `'static`.** API baru `SqlSafeStr` menolak `&String`/`format!`.
   Semua query ditulis literal; filter opsional memakai pola `($n::tipe IS NULL OR kolom = $n)`.
2. **Numeric → f64.** Kolom `numeric` di-cast `::float8` saat SELECT dan di-bind `$n::float8::numeric`
   saat INSERT. Jangan menambah `rust_decimal` tanpa memigrasikan seluruh repo/penilaian.
3. **Refresh token juga dikirim di body.** Browser dashboard memakai cookie httpOnly; Android
   menyimpan `refresh_token` di DataStore dan mengirimnya di body `POST /auth/refresh`. Web
   personel memakai cookie httpOnly dengan `device_id = "web-personel"`; access token hanya
   berada di memori browser.
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
17. **Strong Point v2 dipisah dari absensi kantor, tetapi tetap terkait penilaian.**
    - Titik strong point hanya boleh dimiliki satker level **Polda/Polres**. Satker di bawahnya
      otomatis memakai titik induk lewat `repo/ph.rs::induk_titik_ids` (recursive CTE);
      `POST /ph/strong-point` menolak pemilik level `satker`.
    - **Jadwal disusun langsung per bulan**, tanpa pola mingguan. Admin memilih bulan lalu
      menambah jadwal (tanggal/rentang, sesi, jam, regu, anggota). **Titik tidak dipilih di
      form**; server mengelompokkan anggota berdasarkan `penempatan_ph` dan membuat satu jadwal
      per titik (422 bila ada anggota belum ditempatkan). Tombol **Salin Bulan Lalu**
      (`POST /ph/jadwal/salin`) menyalin jadwal bulan sebelumnya dengan memetakan tanggal
      berdasarkan hari dan urutan minggu (Senin ke-1, Selasa ke-2, dst.).
    - **Penempatan anggota bersifat tetap**: `penempatan_ph(personel_id, strong_point_id)`
      dengan unique `personel_id`. Satu anggota hanya punya satu titik; perpindahan titik
      dilakukan lewat upsert (tanpa riwayat tanggal). Modal penempatan langsung memilih
      **satker → titik → checkbox anggota** (tanpa pilihan level; satker Polda/Polres ikut
      ditampilkan karena dapat memiliki personel sendiri). Hanya akun `jenis_akun='personel'`
      yang dapat ditempatkan; titik harus milik Polda/Polres induk satker.
    - **Fallback bulan sebelumnya**: bila bulan yang sudah berjalan belum memiliki jadwal,
      `pastikan_jadwal_scope` menyalin jadwal bulan terakhir yang tersedia ke bulan tersebut
      (maks. satu kali per satker, dilindungi advisory lock). Baris hasil salinan ditandai
      `disalin_dari_bulan` agar dashboard memperingatkan admin. Bulan yang belum berjalan
      tidak di-fallback supaya admin masih bisa menyusun lebih awal.
    - **Pengingat**: `GET /ph/jadwal/status?bulan=&satker_id=` melaporkan jumlah jadwal bulan
      terpilih, jumlah jadwal bulan depan, dan `perlu_susun_bulan_depan`. Dashboard menampilkan
      peringatan agar jadwal bulan depan disusun sebelum tanggal 1.
    - **Satu jadwal = satu sesi dan satu titik.** Check-in dibuka `jam_mulai − 30 menit` dan
      ditolak setelah `jam_selesai`; check-out sesi sampai pukul **23:59** hari yang sama.
      Aturan sesi berulang/2 jam dari versi lama **dihapus**.
    - **Absensi harian memakai state machine check-in/check-out.** `POST /absensi` menentukan
      `checkin`/`checkout` dari catatan terbuka, bukan dari field `jenis` klien. Dinas Kantor
      dibuka `jam_masuk − 30 menit` s.d. `jam_pulang`; Piket/Shift dan Operasi/Pam dicatat per
      siklus tanpa jadwal (check-out otomatis bila selisih ≤ 24 jam + 1 jam); Dinas Luar/Cuti/Izin/
      Sakit hanya check-in sekali per hari. `GET /absensi/konteks` memberi tahu aksi dan sesi strong
      point yang aktif.
    - **Hari strong point menggantikan Dinas Kantor.** Bila ada jadwal strong point, Dinas Kantor
      ditolak; check-in/check-out strong point dicerminkan ke `absensi` berstatus `ph` sehingga
      hadir, telat, dan pulang cepat ikut rekap kehadiran. Jika sesi baru sudah dibuka sedangkan
      sesi lama belum check-out, aplikasi memprioritaskan check-in sesi baru dan sesi lama tetap
      tercatat tanpa check-out (bukan pulang cepat).
    - `absensi_ph.menit_telat` dihitung bila check-in melewati `jam_mulai`;
      `menit_pulang_cepat` dihitung bila check-out sebelum `jam_selesai`. Rekap menjumlahkan
      keduanya.
    - Laporan memakai **rentang tanggal** `dari`–`sampai` (default 30 hari, maks. 366), bukan
      `bulan`. Rekap PH dibatasi sampai hari ini agar tanggal yang belum berjalan tidak
      dihitung tidak hadir.
    - **Hari libur nasional tidak mengubah jadwal strong point**; jadwal mengikuti jadwal yang
      ditetapkan admin satker.
    - Jam kerja kantor & penilaian tetap: telat kantor diabaikan bila check-in strong point;
      pulang cepat tetap dihitung (`service/scoring.rs`).
    - Tab **Jadwal** berupa roster bulanan + daftar jadwal berpaginasi (Tanggal/Sesi/Jam/Titik/Regu/Anggota),
      dengan filter bulan. Seed memberi Ditreskrimsus jadwal Senin–Jumat pagi+sore dan satker lain
      contoh tanggal 3–5; seluruh personel ditempatkan ke titik induknya.
    - Aplikasi Android kini hanya memiliki tab **Beranda** dan **Absensi**. Wizard absensi:
      pilih status → GPS/radius → foto → kirim. Saat masih ada check-in terbuka, tombol langsung
      menjadi check-out tanpa memilih status. Tab **Strong** dihapus; jadwal strong point tampil di
      Beranda/Absensi. Web personel tetap memiliki halaman Strong terpisah.
    - Dokumentasi foto diminta saat check-in (dapat dilewati bila kamera tidak tersedia).
    - **Penyajian foto:** `foto_url` memakai `/api/v1/files/{key}`; endpoint `/files/{*key}`
      memeriksa pemilik atau cakupan satker. Galeri memakai `/absensi/foto` dan `/ph/foto`
      (filter `dari`/`sampai`); detail per orang memakai `foto_masuk_url`/`foto_pulang_url`
      dari `GET /absensi/harian`.
18. **Pengingat jadwal tanpa FCM.** Android memakai WorkManager + notifikasi lokal
    (H-1 19:00; jadwal tanpa jam diingatkan pagi hari, jadwal berjam tetap 60 menit sebelum).
    Tidak ada ketergantungan layanan push eksternal.
19. **Database uji terpisah (`kinerja_test`).** Seed demo ada di `kinerja_db`; skrip e2e
    men-spawn API sendiri di port 8091 dengan `APP__DATABASE_URL` ke `kinerja_test`.
20. **Satu tabel identitas, dua jenis akun.** `personel.jenis_akun` membedakan `admin` dan
    `personel`. Admin memakai `username` (tanpa NRP), personel memakai NRP (tanpa username).
    Ini menjaga seluruh foreign key audit/verifikasi tetap valid tanpa tabel polimorfik.
21. **Kanal login ditegakkan di server.** `client=android` hanya menerima `jenis_akun=personel`
    (termasuk pimpinan/atasan); akun admin ditolak 403. Jangan hanya menyembunyikan tombol
    di aplikasi. Untuk `client=web`, dashboard hanya menerima akun dengan role **admin atau
    pimpinan**; role atasan/personel saja ditolak 403 (gunakan Android). Jika satu akun memiliki
    beberapa role (mis. pimpinan + atasan + personel), login tetap diizinkan karena ada role
    pimpinan; kemampuan di dashboard mengikuti **gabungan role** tersebut. Role admin tetap
    melihat Master Data meski juga pimpinan; sembunyikan Master Data hanya untuk pimpinan murni
    (`adalahPimpinanSaja`). Jika ingin pemisahan ketat, gunakan akun terpisah atau tambahkan
    pemilih mode aktif saat login.
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
29. **Kebijakan lokasi tiruan ditentukan server, bukan klien.** Android mengirim
    `is_mock_suspect` dari `Location.isMock`/`isFromMockProvider`; backend membaca
    `APP__MOCK_LOCATION_POLICY` (`flag` default development, `reject` produksi). Jangan
    memblokir berdasarkan `BuildConfig.DEBUG`/`device_id` karena bisa dipalsukan. Developer
    Options/aplikasi mock terpilih hanya peringatan UI, bukan penolakan. Detail:
    `docs/fake-gps.md`.
30. **Modul kinerja/periode dinonaktifkan dari UI, bukan dihapus.** Aplikasi kini fokus pada
    **absensi kehadiran + strong point**. Menu Kinerja, Penilaian, Print Laporan, Kesiapan,
    dan Periode dihapus dari navigasi dashboard; route lama diarahkan ke `/dashboard`.
    Tabel dan endpoint (`/rencana-kerja`, `/hasil-kerja`, `/output`, `/periode`, `/penilaian`,
    `/laporan`) tetap ada agar data lama tidak hilang. Laporan absensi/strong point memakai
    **rentang tanggal**, bukan periode bulanan. Jangan menambah fitur baru ke modul dorman
    ini tanpa keputusan produk; jangan `DROP TABLE`.
31. **Jam server realtime tanpa socket.** `GET /api/v1/waktu` membaca `service::waktu`
    (termasuk jam palsu) tanpa akses database dan mengembalikan `epoch_ms` + tanggal/jam WITA.
    Klien menghitung `offset = epoch_ms_server − tengah(latensi request)`, lalu berdetak lokal
    tiap detik. Resync tiap 5 menit; saat `palsu=true` resync tiap 15 detik dan epoch fake yang
    tidak berubah tidak menggeser offset (agar jam tetap berdetak). Jangan ganti dengan polling
    per detik, WebSocket, atau SSE: untuk jam, itu membebani server dan baterai tanpa manfaat.
    Sinkronisasi ulang juga dipicu saat tab browser kembali terlihat/fokus.
32. **Rekap kehadiran berbentuk matriks per tanggal.** `GET /api/v1/absensi/matriks`
    mengembalikan satu baris per personel dengan peta tanggal → detail (status, jam, menit telat,
    menit pulang cepat, koordinat, foto). Rentang **maksimal 31 hari** divalidasi di server dan
    klien. Hari strong point memakai jam/koordinat PH (bukan absensi kantor) bila keduanya ada.
    Alamat pada modal diambil lewat reverse geocode Nominatim di browser (antrean 1 req/detik,
    cache), bukan disimpan di database. Matriks memuat semua personel aktif dalam cakupan,
    ditambah personel nonaktif yang masih punya data. PDF `/absensi/matriks/ekspor` membagi
    tanggal menjadi blok 7 hari per bagian dengan kop/tabel berulang agar tetap terbaca; karena
    bisa panjang saat seluruh satker dipilih, filter satker/personel disarankan untuk cetak.
    Hari kerja efektif tanpa catatan absensi diberi flag `tidak_absen` (TA), termasuk hari
    ini sebelum check-in; hari libur, akhir pekan di luar hari kerja, dan tanggal mendatang
    tidak ditandai. Badge status dan penanda ditampilkan terpisah (mis. **DK TC**), sehingga
    status DK/PH tetap terbaca meski check-out belum ada. TC hanya aktif setelah tanggal
    berganti (toleransi check-out sampai 23.59; Piket/Operasi 25 jam), jadi pada hari
    berjalan check-in tanpa check-out hanya menampilkan status + jam masuk.
    Jadwal strong point dimuat per sesi (`sesi_strong`): pagi/sore, jam jadwal, realisasi
    check-in/check-out, telat/pulang cepat, serta flag TC/TA per sesi. Jika ada jadwal tetapi
    tidak ada absensi, sel menampilkan PH + TA; jika statusnya Izin/Sakit/Dinas Luar/Cuti,
    status tersebut tetap tampil bersama penanda Jadwal PH.
33. **Rekap Harian memakai ulang matriks satu hari.** Tab **Rekap Harian** memanggil
    `GET /absensi/matriks?dari=<tanggal>&sampai=<tanggal>` (bukan endpoint baru), lalu
    menghitung kartu di klien: Tidak Absen (`tidak_absen`), Jadwal PH (`jadwal_strong` dengan
    rincian Sudah PH dari `ph_absen` dan Belum PH), serta status DK/DL/Piket/Operasi/Cuti/Izin/
    Sakit dari `statusEfektif`. Sub-tab **Peta** memakai data matriks yang sama (koordinat
    check-in), ditambah `GET /titik-dinas` dan `GET /ph/strong-point` dengan filter `satker_id`;
    hanya titik aktif yang digambar, radius memakai `radius_m`, marker personel diwarnai per
    status, dan basemap dapat diganti Jalan/Satelit. Kartu dapat diklik untuk modal daftar
    personel dan tombol **Detail** menuju modal absensi penuh. Jangan menduplikasi query agregat
    di server kecuali kartu perlu dihitung untuk periode yang lebih besar.
34. **Peta dashboard memakai ulang endpoint yang ada.** `PetaDashboard.svelte` di `/dashboard`
    memanggil `/absensi/harian`, `/ph/rekap/harian`, `/titik-dinas`, dan `/ph/strong-point`
    dengan filter tanggal + satker aktif. Marker absensi menyembunyikan baris `ph_hadir` agar
    tidak bertumpuk dengan marker check-in strong point; hanya titik aktif yang digambar.
    Kegagalan endpoint titik/rekap PH diabaikan (peta tetap tampil) supaya dashboard tidak
    ikut gagal saat salah satu layanan bermasalah.

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
- Item sidebar **Absensi Saya** dan **Persetujuan** tidak dimasukkan ke struktur menu baru; halaman
  dan API-nya tetap aktif bila diakses langsung.
- **Sidebar bergrup** dengan submenu yang dapat dilipat (`(app)/+layout.svelte`):
  **Dashboard**; **Master Data** (Satker & Jam Kerja, Titik Dinas, Hari Libur, Data Personel);
  **Absensi** (Rekap Kehadiran, Rekap Harian, Detail Rekap, Verifikasi Offline); **Strong Point**
  (Jadwal Strong Point, Rekap Strong Point); **Kinerja** (Rencana Kerja, Hasil Kerja, Output);
  **Laporan** (Penilaian, Print Laporan); **Pengaturan** (Akun Admin, Kesiapan Periode, Periode).
  Grup **Master Data** disembunyikan untuk pimpinan murni dan akses langsung ke
  `/satker`, `/personel`, `/libur`, `/absensi/titik-dinas` dialihkan ke dashboard. Submenu dan tab
  **Verifikasi Offline** juga disembunyikan (`?tab=verifikasi` dialihkan ke rekap). Tombol
  **Buat Rencana**, **Tambah Hasil**, **Tambah Output**, dan **Ajukan** rencana disembunyikan
  untuk pimpinan murni; admin tetap melihatnya.
  Grup otomatis terbuka saat salah satu submenunya aktif; submenu Absensi memakai
  `/absensi/rekap?tab=rekap|detail|verifikasi` dan Strong Point memakai
  `/ph?tab=jadwal|rekap` sehingga sidebar dan tab di halaman tetap sinkron.
- **Ekspor PDF** dashboard: halaman Rencana Kerja, Hasil Kerja, Output, dan **Rekap Absensi** mengirim filter ke
  `GET /{modul}/ekspor` (absensi: `rekap/ekspor`, `verifikasi/ekspor`, `detail/ekspor`) dengan bearer token
  lewat `apiBlob` (`#lib/api/client`), lalu mengunduh blob.
  Backend menyusun HTML lengkap (kop + tabel + footer) via `service/laporan.rs` dan mencetaknya
  dengan `service/pdf.rs` memakai Chrome/Edge `--headless --print-to-pdf`; lokasi browser
  auto-detect atau `APP__CHROME_PATH`.
- **Rekap Absensi** (`/absensi/rekap`) memakai tab lokal tanpa route baru: **Rekap Kehadiran**
  (periode + satker), **Rekap Harian** (satu tanggal + satker, kartu ringkasan), **Verifikasi**
  (periode + satker + personel, aksi Setujui + foto), dan **Detail per Orang** (periode + satker
  + personel wajib). Tiap tab punya filter sendiri dan memuat data saat dibuka. Filter satker dan
  personel memakai `Combobox.svelte` (cari nama/NRP) — jangan diganti `<select>` biasa. Tab
  Verifikasi/Detail memakai `GET /absensi/harian` (satu baris per personel per tanggal: masuk &
  pulang digabung, `menit_telat` hanya untuk Dinas Kantor), rekap memakai `GET /absensi/rekap`
  yang juga mengembalikan total `menit_telat` per personel, sedangkan Rekap Harian memakai
  `GET /absensi/matriks` satu hari dan menghitung kartu di klien.
  Penyetujuan dari tab Verifikasi mengulang `POST /absensi/{id}/verifikasi` untuk setiap
  `perlu_verifikasi_ids` pada baris tanggal tersebut.
- **Halaman detail** dashboard: `/rencana-kerja/{id}`, `/hasil-kerja/{id}`, dan `/output/{id}`
  memuat data lengkap + lampiran (`apiBlobUrl` → object URL, dibuka di tab baru). Tautan berasal
  dari judul baris tabel maupun tombol **Detail**.

**Web personel (`personel/`)**
- SPA SvelteKit dengan `ssr=false`; `+layout.ts` root, CSS global diimpor dari
  `src/routes/+layout.svelte` (jangan lupa, kalau tidak Tailwind tidak termuat).
- Desain sengaja berbeda dari dashboard: mobile-first, header ringkas, **bottom navigation**
  6 tab (Beranda, Absensi, Rencana, Hasil, Output, Strong), kartu membulat, aksen teal
  `brand-*`, bukan sidebar navy/gold.
- Sesi: access token hanya di memori; refresh memakai **cookie httpOnly** hasil login.
  Backend mengizinkan refresh cookie bila `device_id = "web-personel"` atau role
  admin/pimpinan. `#lib/api/client` memancarkan event `personel:sesi-berakhir` saat refresh
  gagal; layout mendengarkan event itu lalu redirect ke `/login`.
- Wizard izin browser tampil sekali setelah login (`localStorage['personel.izin_selesai']`),
  menjelaskan lokasi, kamera, notifikasi, dan antrean offline.
- Antrean offline: `#lib/offline.ts` (kunci per pengguna). Absensi yang gagal tetap disimpan;
  antrean PH membuang kegagalan permanen 4xx (kecuali 401) seperti Android.
- Absensi: `GET /absensi/hari-ini` menentukan tombol Masuk/Pulang; bila gagal, kedua tombol
  tampil dan mode offline aktif. Status Dinas Kantor/Piket memerlukan radius kecuali offline.
- Strong Point: `#lib/strong.ts` menggabungkan sesi server + antrean; check-in terbuka > 2 jam
  dianggap sesi baru. Dokumentasi foto hanya untuk check-in; check-out tanpa dokumentasi.
- Reverse geocode memakai `https://geocode.vinrul.my.id/reverse` (sama dengan Android).
- Wizard memakai komponen `WizardHeader` (judul, deskripsi, “Langkah X dari Y”, progress bar)
  seperti tampilan wizard Android.
- Foto disimpan sebagai base64 JPEG (kanvas, sisi maks. 1280, kualitas 0.82) agar cocok dengan
  field `foto_base64` backend.
- Keterbatasan web dibanding Android: tidak ada WorkManager/notifikasi latar (pengingat hanya
  dapat dibuat saat tab aktif, belum diimplementasikan), dan alamat server mengikuti origin
  aplikasi (proxy `/api`), jadi tidak ada pengaturan alamat server.

**Android**
- Satu `KinerjaViewModel` menangani semua state & pemanggilan repo; layar Compose hanya
  memanggil method VM.
- Navigasi: bottom bar 6 tab (Beranda, Absensi, Rencana, Hasil, Output, Strong Point) + Profil
  lewat ikon di top bar (tanpa Navigation-Compose). Tab **Nilai** tetap diringkas di Beranda;
  `NilaiScreen` masih ada di kode dan dapat diaktifkan kembali.
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
  `POST /auth/foto`), **Edit Profil** (nama, pangkat lewat combobox 22 tingkat Polri + 17 pangkat ASN terendah→tertinggi,
  jabatan, telepon, email via `PATCH /auth/profile`), dan **Ganti Password**
  (`POST /auth/change-password`). Tiap kolom password punya ikon mata tampil/sembunyi (`IkonMata`,
  tanpa dependensi tambahan). Satker/NRP/peran tetap hanya admin. Ganti password memicu
  `bersihkanDataSesi()` sehingga aplikasi kembali ke layar login.
- **Avatar di Beranda** memakai komponen bersama `FotoProfil` dan hanya digambar bila foto berhasil
  diunggah/dimuat (`tampilkanInisial = false`); tanpa foto, salam langsung dimulai dari teks.
- **Profil dashboard** (`(app)/profil/+page.svelte`): avatar memakai komponen bersama
  `Avatar.svelte` (menerima `pengguna` + `versi` untuk memuat ulang setelah unggah), unggah foto
  lewat `POST /auth/foto` (FormData field `file`), ganti password lewat
  `POST /auth/change-password`. `apiBlobUrl('auth/foto/{id}')` dipetakan ke
  `/api/v1/auth/foto/{id}`. Ganti password mencabut seluruh sesi → panggil `logout()` lalu
  `goto('/login?ubah=password')`; jangan hanya menampilkan pesan sukses.
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
GET|POST /ph/strong-point?satker_id=   PATCH|DELETE /ph/strong-point/{id}
GET|POST /ph/penempatan?satker_id=        # satu anggota = satu titik
GET|POST /ph/jadwal?dari&sampai&satker_id= POST /ph/jadwal/salin
GET    /ph/jadwal/status?bulan&satker_id=  # pengingat bulan depan + fallback
GET    /ph/jadwal/saya?dari&sampai=        GET|PUT|DELETE /ph/jadwal/{id}
GET    /ph/jadwal/ekspor?dari&sampai&satker_id=      # PDF tabel jadwal
POST   /ph/absensi        GET /ph/absensi/saya?dari&sampai=   GET /ph/absensi/jadwal/{id}
POST   /ph/absensi/{id}/verifikasi                 GET /ph/rekap?dari&sampai&satker_id=
GET    /ph/rekap/harian?dari&sampai&satker_id=     GET /ph/rekap/harian/ekspor?dari&sampai&satker_id=
GET    /ph/rekap/personel/{id}/ekspor?dari&sampai&satker_id=   GET /ph/rekap/ekspor?dari&sampai&satker_id=      GET /ph/foto?dari&sampai&personel_id=
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
GET    /penilaian          POST /penilaian/{periode_id}/hitung      # dorman (UI dinonaktifkan)
GET    /penilaian/personel/{personel_id}          POST /penilaian/{id}/keputusan
GET    /dashboard/{ringkasan,capaian-indikator,tren,sebaran-kategori,rekap}   # dorman
GET    /dashboard/kehadiran?tanggal=&satker_id=    # dipakai UI baru
GET    /laporan/penilaian[?format=csv]
GET    /notifikasi        POST /notifikasi/{id}/baca
GET    /audit             # admin global & admin Polda
GET    /health            # di luar /api/v1
GET    /docs              # di luar /api/v1
```

---

## 6. Skema database (ringkas)

`satker` (3 level, +`jam_pulang_sabtu`) · `personel` (+`atasan_id`, `jenis_akun`, `username`; kolom `unit`/`email` lama, tidak diisi UI baru) ·
`personel_role` (+`scope_satker_id`) · `refresh_token` · `titik_dinas` ·
`absensi` (unik personel+tanggal+jenis, `client_uuid` unik) · `strong_point` (+`satker_id` induk
Polda/Polres) · `penempatan_ph` (unik `personel_id`; satu anggota = satu titik) ·
`jadwal_ph` (+`sesi` pagi/sore, `is_arsip`, `disalin_dari_bulan`) ·
`jadwal_ph_personel` · `absensi_ph` (+`menit_telat`, `menit_pulang_cepat`) · `rencana_kerja` ·
`rencana_tahapan` · `hasil_kerja` · `output` · `lampiran` · `periode` · `bobot_indikator` ·
`periode_satker_kesiapan` · `hari_libur` · `penilaian` (unik personel+periode) ·
`penilaian_detail` (jsonb komponen) · `notifikasi` · `audit_log`.

`personel.nrp` hanya wajib untuk `jenis_akun='personel'`; `personel.username` hanya wajib
untuk `jenis_akun='admin'` (constraint `personel_identitas_check`).

Migrasi: `backend/migrations/0001_init.sql` … `0020_absensi_ph_link.sql` (penempatan tetap,
fallback bulanan, hapus pola mingguan). Status absensi `libur_nasional` menggantikan nama lama
`ph` (nama `ph` sekarang khusus modul PH).
Tambah migrasi baru = file `000N_nama.sql` lalu `sqlx migrate run` + `cargo sqlx prepare` bila
memakai `query!` (saat ini semua runtime query, jadi `.sqlx` belum diperlukan).

---

## 7. Pengujian

| Lapis | Perintah | Cakupan |
|---|---|---|
| Unit | `cd backend; cargo test` | geofence, scoring, deteksi berkas, kanal login, policy mock GPS |
| Lint Rust | `cargo clippy --all-targets -- -D warnings` | — |
| API e2e | `bun tools/uji/api-smoke.js` (DB `kinerja_test`, API :8091) | RBAC ber-scope, CRUD satker/personel/admin/titik dinas, kesiapan, libur, **penempatan tetap**, **salin jadwal bulanan**, **status & fallback jadwal bulan sebelumnya**, sesi, check-in telat/pulang cepat, rekap rentang, PDF, jam palsu, mock GPS flag, **73 cek** |
| OpenAPI | `bun tools/uji/openapi-check.js` | JSON valid + seluruh `$ref` ada |
| Periode otomatis | `bun tools/uji/periode-auto-check.js` | Hapus periode DB uji → dashboard memicu pembuatan bulan berjalan |
| Type-check UI | `cd dashboard; bun run check` | 0 error |
| Type-check web personel | `cd personel; bun run check` | 0 error |
| Build UI | `bun run build` | SPA statis |
| Android | `.\gradlew.bat :app:assembleDebug` | APK debug |
| Seed VPS | `cargo run --bin seed -- --reset --vps` (DB uji) | 40 personel, penempatan titik tetap, jadwal PH bulan berjalan, absensi & rekap PH sampai hari ini (maks. 19), status rencana/hasil/output lengkap |

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
- Signing Android dibaca dari `android/keystore.properties` (gitignored). Jangan menaruh
  password keystore di `build.gradle.kts` atau berkas yang di-commit; `storeFile` harus ada
  saat `assembleRelease`/`bundleRelease`, sedangkan build debug tetap berjalan tanpa berkas itu.
- `network_security_config.xml` dipisah per build: `src/main` = `cleartextTrafficPermitted=false`
  (release/HTTPS) dan `src/debug` = `true` (emulator HTTP). Jangan membalik keduanya.
- Seed `--vps` hanya membuat Polda Bali + Ditreskrimsus dan **tidak** membuat akun Polres
  (`admin_polres`, `000022`, `000023`); jangan mencampurnya dengan database yang sudah berisi
  data development. `--reset` tetap menghapus seluruh data.
- Deploy native: proxy nginx `/files/` harus diarahkan ke `/api/v1/files/` (bukan
  `lampiran/raw/`) agar galeri foto absensi/strong point bekerja.
- Chrome headless di systemd membutuhkan `HOME` yang dapat ditulis (mis. `/var/lib/absensi`)
  dan `ReadWritePaths` yang mencakup folder tersebut; tanpa itu ekspor PDF gagal 500.
- Dengan `ssl_verify_client on`, `curl` langsung ke domain akan ditolak; uji lewat Cloudflare
  (Authenticated Origin Pulls) atau server block diagnostik `optional_no_ca`.
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
- **Kanal login `personel_web` baru**; API yang masih berjalan dari biner lama akan menolak
  login web personel dengan 400/422. Jalankan ulang `cargo run` setelah mengubah enum.
- Web personel berjalan di **:5174** (dashboard :5173). Jangan tertukar saat menguji; proxy
  `/api` hanya ada di dev server. Produksi wajib HTTPS agar geolocation/kamera tidak diblokir
  browser (kecuali `localhost`).
- Aplikasi web personel memakai cookie refresh httpOnly dengan penanda `device_id = "web-personel"`.
  Jangan mengubah penanda itu tanpa memperbarui `akses::refresh_cookie_diizinkan`; tanpa penanda,
  refresh cookie untuk role atasan/personel akan ditolak 403.
- `Pesan`/`WizardHeader`/`Sheet` adalah komponen khusus `personel/`; jangan memakai komponen
  dashboard karena gaya dan alias impornya berbeda.
- Ekspor PDF butuh Chrome/Edge terpasang. Di mesin ini `chrome.exe` dan `msedge.exe` terdeteksi;
  bila tidak, set `APP__CHROME_PATH`. Chrome dijalankan dengan profil sementara di folder temp
  dan seluruh berkas sementara dibersihkan setelah PDF terbentuk.
- `kinerja-api.exe` yang sedang berjalan mengunci `target/debug` → matikan dulu sebelum
  `cargo build` (lihat §8).
- **Strong point v2:** `strong_point` hanya boleh dimiliki level Polda/Polres; saat menambah
  `JOIN satker`, kualifikasi semua kolom (mis. `sp.id`, `sp.nama`) — `id`/`nama` di kedua tabel
  membuat PostgreSQL menolak dengan "column reference is ambiguous".
- `personel.nrp` NULL untuk akun admin; query daftar penempatan/penjadwalan yang menampilkan
  personel wajib menambahkan `jenis_akun = 'personel'`, kalau tidak decoding `nrp: String` gagal 500.
- Rekap PH sengaja dibatasi `sampai = min(sampai_diminta, hari_ini_wita())`; saat menguji dengan
  tanggal masa depan, rekap akan kosong (bukan bug). Gunakan tanggal hari ini/lampau.
- **Fallback jadwal** hanya berjalan untuk bulan yang **sudah berjalan** (`awal_bulan <= awal bulan
  hari ini`). Bulan depan tetap kosong sampai disusun/salin agar admin bisa menyiapkan lebih awal.
  Baris hasil fallback punya `disalin_dari_bulan`; salinan manual tidak diberi tanda.
- `salin_jadwal` memetakan tanggal berdasarkan **hari + urutan minggu**; tanggal ke-5 yang tidak
  ada di bulan tujuan akan dilewati. `timpa=true` hanya menghapus jadwal tanpa absensi.
- `penempatan_ph` sekarang unik per `personel_id`; simpan penempatan bersifat upsert. Jangan
  menambahkan kolom tanggal kembali tanpa memigrasikan semua query/repo.
- **Titik jadwal berasal dari penempatan**: `JadwalRequest` tidak menerima `strong_point_id`.
  Jangan mengembalikan field itu ke form jadwal; `create_jadwal`/`update_jadwal` memakai
  `kelompokkan_titik` dan dapat membuat beberapa jadwal (satu per titik) dari satu permintaan.
- `status_jadwal` membaca jadwal **mentah** dan tidak memicu fallback; jangan menambahkan
  `pastikan_jadwal_*` di endpoint status agar peringatan tetap muncul.
- Saat menguji jam palsu, pilih tanggal yang tidak punya absensi seed (mis. 1 November atau
  akhir pekan); seed development mengisi absensi kantor semua hari kerja sampai kemarin.
- **Fake GPS:** `APP__MOCK_LOCATION_POLICY=flag` untuk development agar emulator + Fake GPS
  tetap bisa menguji alur; `reject` untuk produksi. Antrean absensi/PH yang ditolak mock
  dihapus dan dihitung `ditolak`, sedangkan kegagalan lain dipertahankan. `adb emu geo fix`
  bukan mock provider dan cocok untuk jalur normal. Jangan memblokir hanya karena
  Developer Options aktif.

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
7. Pusat notifikasi + pengingat jadwal strong point di dashboard web.
8. Ekspor CSV rekap PH (JSON + PDF sudah ada).
9. **Uji lapangan strong point**: penempatan tetap, salin jadwal bulanan, fallback, dan toleransi
   jendela sesi (−30/+60 menit).
10. **Bersihkan modul kinerja dorman** (halaman `/rencana-kerja`, `/hasil-kerja`, `/output`,
    `/penilaian`, `/laporan`, `/kesiapan`, `/pengaturan`) bila produk memastikan tidak dipakai;
    API dan tabel sengaja dipertahankan sampai keputusan itu.
11. **Pengingat Android per sesi**: saat ini WorkManager memakai jadwal (satu baris per sesi),
    tetapi label notifikasi belum membedakan pagi/sore secara eksplisit.
12. Ekspor PDF server-side dengan kop & QR verifikasi.
13. Rate limit berbasis Redis bila API di-scale.
14. **Play Integrity** (`APP__PLAY_INTEGRITY_MODE=off|log|enforce`) untuk memastikan APK asli;
    bukan pengganti deteksi koordinat.
15. **Validasi plausibilitas server**: lompatan koordinat, kecepatan tidak wajar, dan jarak/waktu
    antarabsensi.
16. Deteksi lokasi tiruan di web personel tidak tersedia di browser; pertimbangkan kebijakan
    foto wajib + verifikasi atasan.

**P3 — teknis**
18. Pindahkan antrean offline Android ke Room (butuh KSP) agar query lebih kaya.
19. CI (GitHub Actions self-hosted) untuk `api-smoke.js` + `openapi-check.js`.
20. Utoipa/OpenAPI otomatis sebagai pengganti `openapi.json` manual.
21. Uji beban: 200 personel × 1 bulan, target hitung < 60 detik.
22. Push FCM bila kelak butuh notifikasi daring lintas perangkat.

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
