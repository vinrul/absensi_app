# Aplikasi Absensi & Penilaian Kinerja Personel — Polda Bali

Sistem terintegrasi untuk mencatat kehadiran berbasis geolokasi serta menilai kinerja
personel Polda Bali dan Polres jajaran berdasarkan empat indikator:
**Absensi 20% · Rencana Kerja 20% · Hasil Kerja 30% · Output 30%**.

Kebutuhan fungsional bersumber dari `aplikasi_nilai_kerja.pdf` (7 halaman: alur per
indikator, integrasi & bobot, Form 1–4, dan dashboard pimpinan).

Dokumen pendamping:

| Dokumen | Isi |
|---|---|
| [`docs/PRD.md`](docs/PRD.md) | Kebutuhan produk, peran, ruang lingkup, metrik |
| [`docs/HANDOFF.md`](docs/HANDOFF.md) | Status, keputusan teknis, jebakan, backlog (untuk agent AI berikutnya) |
| [`docs/CHECKLIST.md`](docs/CHECKLIST.md) | Checklist pekerjaan & sisa backlog |
| [`docs/penilaian.md`](docs/penilaian.md) | Rumus perhitungan nilai per indikator |
| [`docs/UAT.md`](docs/UAT.md) | Checklist uji terima per peran |
| [`docs/fake-gps.md`](docs/fake-gps.md) | Deteksi lokasi tiruan, kebijakan `flag`/`reject`, dan cara menguji di development |
| [`docs/deploy-vps.md`](docs/deploy-vps.md) | Deploy VPS: systemd, nginx mTLS, dashboard/personel, seed demo Ditreskrimsus |
| `AGENTS.md` | Aturan singkat untuk agent AI |

## Isi repositori

| Folder | Isi | Teknologi |
|---|---|---|
| `backend/` | REST API, autentikasi, geofencing, mesin penilaian | Rust (axum 0.8, sqlx 0.9, PostgreSQL 18) |
| `dashboard/` | Dashboard pimpinan & admin (SPA) | Svelte 5, SvelteKit 3, Tailwind CSS 4 — **Bun** |
| `personel/` | Aplikasi web personel & atasan langsung (SPA mobile-first) | Svelte 5, SvelteKit 3, Tailwind CSS 4, Leaflet — **Bun** |
| `android/` | Aplikasi personel (absensi GPS, offline queue) | Kotlin, Jetpack Compose, OkHttp |
| `deploy/` | Docker Compose, nginx, backup, contoh env | Docker |

## Menjalankan

### 1. Basis data + API

```powershell
cd F:\absensi_polda_bali\deploy
copy .env.example .env          # sesuaikan password & JWT_SECRET
docker compose up -d db         # PostgreSQL 18 di port 5433

cd ..\backend
copy .env.example .env          # APP__DATABASE_URL ke port 5433
cargo run --bin seed -- --reset # data demo + contoh rencana draft/diajukan/perlu revisi
cargo run                       # API di http://127.0.0.1:8080
```

Dokumentasi API interaktif: **http://127.0.0.1:8080/docs** (spesifikasi mentah di
`/api-docs/openapi.json`).

Akun demo (seed):

| Peran | Username / NRP | Password |
|---|---|---|
| Admin Sistem | `superuser` | `Superuser#2026` |
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

Admin memakai **username** dan hanya dapat login lewat dashboard web. Pimpinan memakai **NRP** dan
dapat login dashboard web maupun Android. **Atasan langsung dan personel** memakai Android atau
**aplikasi web personel** (`personel/`); dashboard menolak login mereka dengan 403.

Aplikasi web personel memakai kanal login `personel_web`: hanya akun personel (termasuk
atasan/pimpinan) yang diterima. Access token disimpan di memori dan refresh memakai cookie
httpOnly (`device_id = "web-personel"`).

Deteksi lokasi tiruan diatur server: `APP__MOCK_LOCATION_POLICY=flag` untuk development
(simpan + `perlu_verifikasi`) dan `reject` untuk produksi (HTTP 422). Panduan lengkap:
[`docs/fake-gps.md`](docs/fake-gps.md).

Data demo ada di database `kinerja_db`. Untuk uji otomatis gunakan database terpisah
`kinerja_test` (lihat `tools/uji/api-smoke.js`).

Seed menyertakan contoh rencana kerja untuk akun `85000001` pada tiga status:
**Draft** (dapat diubah/diajukan), **Diajukan** (menunggu persetujuan atasan), dan
**Perlu Revisi** (dengan catatan atasan), di samping dua rencana yang sudah disetujui.
Ada juga satu rencana `diajukan` milik atasan `000100` yang ditujukan ke pimpinan `000002`,
untuk menguji rantai dual-role (atasan sekaligus bawahan).

Contoh hasil kerja dan output juga tersedia: hasil `draft`, `diajukan`, `perlu_revisi`, dan
`terverifikasi`, serta output `draft`, `dalam_verifikasi` (PDF + PNG), `perlu_revisi`, dan
`disetujui` — semuanya dapat dibuka dari tab Hasil/Output Android tanpa membuat data baru.

Pengajuan contoh juga dilengkapi lampiran nyata di folder upload:

- `85000001` — Patroli siber media sosial: **PDF + PNG**.
- `85210022` (personel lain) — Pengungkapan jaringan penipuan daring: **PDF + PNG**.
- `000100` — Supervisi penyelesaian perkara: **PDF**.
- `85000001` — Sosialisasi pencegahan penipuan daring: **PDF** (status perlu revisi).
- `85000001` — Penyelesaian berkas perkara tahap 1: **PDF** (status disetujui, untuk uji detail pasca-persetujuan).

### 2. Dashboard (Bun)

```powershell
cd F:\absensi_polda_bali\dashboard
bun install                    # wajib Bun; jangan memakai npm/package-lock.json
bun run dev                    # http://localhost:5173 (proxy /api → :8080)
bun run check                  # type-check Svelte/TS
bun run build                  # hasil statis di dashboard/build
```

> Catatan: shell mesin pengembangan ini menetapkan `NODE_ENV=production`, sehingga
> `npm install` akan melewati devDependencies. Bun tidak terpengaruh.
>
> **Ekspor PDF dashboard** (mis. Rencana Kerja) dirender server memakai **Chrome/Edge
> headless**. Pastikan salah satunya terpasang; lokasi dideteksi otomatis, atau set
> `APP__CHROME_PATH` di `backend/.env` (contoh ada di `backend/.env.example`).

### 3. Aplikasi web personel (Bun)

```powershell
cd F:\absensi_polda_bali\personel
bun install                    # lockfile resmi: bun.lock
bun run dev                    # http://localhost:5174 (proxy /api → :8080)
bun run check
bun run build                  # hasil statis di personel/build
```

Aplikasi ini memakai desain mobile-first dengan bottom navigation dan alur yang sama dengan
Android: Beranda, Absensi, Rencana, Hasil, Output, dan Strong Point. Untuk produksi, sajikan
isi `personel/build` pada host/port terpisah. Contoh: salin ke `deploy/data/web-personel/`
lalu jalankan `docker compose up -d web-personel` (port default `8089`).

> Web personel **wajib memakai HTTPS** di produksi (kecuali `localhost`) agar browser
> mengizinkan geolocation dan kamera.

### 4. Aplikasi Android

```powershell
cd F:\absensi_polda_bali\android
.\gradlew.bat :app:assembleDebug
# APK: app\build\outputs\apk\debug\app-debug.apk

# opsional: uji di emulator
adb install -r app\build\outputs\apk\debug\app-debug.apk
```

Alamat server default `http://10.0.2.2:8080/api/v1/` (emulator). Untuk perangkat
nyata, ubah lewat menu **Profil → Server** menjadi `http://<ip-server>:8080/api/v1/`.

#### Build release (APK + AAB)

Signing release dibaca dari `android/keystore.properties` yang **tidak di-commit**.
Salin contohnya lalu isi nilainya:

```powershell
cd F:\absensi_polda_bali\android
copy keystore.properties.example keystore.properties
notepad keystore.properties
```

```properties
storeFile=E:/new_disposisi/siopa.jks
storePassword=...
keyAlias=siopa
keyPassword=...
```

Build keduanya sekaligus:

```powershell
.\build-release.bat
# APK  : app\build\outputs\apk\release\app-release.apk
# AAB  : app\build\outputs\bundle\release\app-release.aab
```

APK release memakai `applicationId` `id.polri.poldabali.kinerja` (tanpa `.debug`), sehingga
dapat terpasang berdampingan dengan build debug. Untuk produksi, server memakai HTTPS dan
`src/main/res/xml/network_security_config.xml` menetapkan `cleartextTrafficPermitted="false"`.
Build debug memakai override `src/debug/res/xml/network_security_config.xml` yang masih
mengizinkan HTTP untuk emulator (`http://10.0.2.2:8080`).

#### Menjalankan emulator & memasang APK lewat command line

Pastikan backend sudah berjalan di `http://127.0.0.1:8080` (emulator mengaksesnya lewat
`10.0.2.2`). Jalankan dua terminal terpisah: satu untuk emulator, satu untuk `adb`.

```powershell
# 1. Lihat AVD yang tersedia
emulator -list-avds

# 2. Jalankan emulator (nama AVD contoh dari mesin pengembangan ini)
emulator -avd Pixel_3a_API_34 -gpu swiftshader_indirect

# 3. Terminal lain: tunggu emulator siap
adb wait-for-device
adb shell getprop sys.boot_completed   # keluaran "1" berarti sudah siap
adb devices                            # harus tampil "emulator-5554  device"
```

Jika `emulator`/`adb` tidak ada di `PATH`, panggil langsung dari Android SDK:

```powershell
$sdk = "$env:LOCALAPPDATA\Android\Sdk"
& "$sdk\emulator\emulator.exe" -avd Pixel_3a_API_34 -gpu swiftshader_indirect
& "$sdk\platform-tools\adb.exe" devices
```

Setelah emulator siap, build dan pasang APK dari root proyek:

```powershell
cd F:\absensi_polda_bali\android
.\gradlew.bat :app:assembleDebug

# APK debug memakai applicationId id.polri.poldabali.kinerja.debug
adb install -r app\build\outputs\apk\debug\app-debug.apk

# Buka aplikasi tanpa menyentuh layar
adb shell am start -n id.polri.poldabali.kinerja.debug/id.polri.poldabali.kinerja.MainActivity
```

Perintah pemeliharaan yang sering dipakai:

```powershell
# Hapus data aplikasi (sesi login, wizard izin, pengaturan server kembali awal)
adb shell pm clear id.polri.poldabali.kinerja.debug

# Copot aplikasi dari emulator
adb uninstall id.polri.poldabali.kinerja.debug

# Matikan emulator
exit   # pada terminal emulator, atau:
adb -s emulator-5554 emu kill
```

Catatan:

- `seed --reset` mengganti id personel sehingga **sesi Android lama dapat gagal**; login ulang
  dengan akun demo (mis. `85000001` / `Polri#2026`).
- Jika `adb install` menolak karena versi lama berbeda tanda tangan, jalankan
  `adb uninstall id.polri.poldabali.kinerja.debug` lalu pasang ulang.
- Emulator mesin ini memerlukan `-gpu swiftshader_indirect` agar tidak crash.

### 4. Fake server / jam server palsu (development)

Backend dapat memakai tanggal dan jam palsu untuk menguji absensi tanpa mengubah jam
komputer. Fitur ini **hanya untuk development** dan hanya aktif bila
`APP__ALLOW_FAKE_TIME=true` pada `backend/.env`.

```env
# backend/.env
APP__ALLOW_FAKE_TIME=true
```

Jalankan API seperti biasa, lalu gunakan perintah `fake` dari terminal lain:

```powershell
cd F:\absensi_polda_bali\backend

# aktifkan jam palsu: 20 November 2026 pukul 07:45 WITA
cargo run --bin fake -- 20-11-2026 07:45

# cek jam yang sedang dipakai
cargo run --bin fake -- status

# kembalikan ke jam sistem
cargo run --bin fake -- reset
```

Format tanggal `DD-MM-YYYY` dan jam `HH:MM`. Zona waktu default `Asia/Makassar` (WITA);
opsional gunakan `--zona Asia/Jakarta` atau `--zona Asia/Jayapura`:

```powershell
cargo run --bin fake -- 20-11-2026 07:45 --zona Asia/Jakarta
```

Catatan penting:

- API harus **sudah berjalan** (`cargo run`) karena perintah `fake` mengirim ke endpoint
  development `/api/v1/dev/waktu`; tidak perlu restart API saat mengganti jam.
- Gunakan `--bin fake --`, bukan `-p fake`; proyek ini bukan workspace Cargo.
- Jam palsu memengaruhi tanggal/jam absensi, `GET /absensi/hari-ini`, validasi tanggal,
  dan periode berjalan. Produksi tetap memakai jam sistem karena
  `APP__ALLOW_FAKE_TIME` default `false` dan route `/dev/*` tidak dipasang.
- Override hilang saat proses API di-restart.
- Data seed demo sudah memiliki absensi tanggal 1–16 November 2026. Untuk menguji
  "belum absen" dari nol, pilih tanggal setelahnya, misalnya **20-11-2026**;
  tanggal **11-11-2026** sudah berisi data contoh.

## Fitur utama

- **Organisasi 3 level**: Polda → Polres → Satker, dengan 12 peran ber-scope
  (admin/pimpinan Polda, Polres, dan Satker).
- **Akun admin terpisah**: admin memakai username (tanpa NRP), hanya login dashboard web,
  dan dikelola di halaman **Akun Admin**; personel (termasuk pimpinan) memakai NRP dan
  dikelola di halaman **Data Personel**. Form personel meminta NRP, nama, pangkat (combobox
  21 tingkat Polri), jabatan, satker, dan atasan — tanpa unit/email.
- **Pagination seragam**: semua tabel daftar di dashboard memakai komponen `Pagination`
  (10 baris per halaman); modul besar memakai pagination server-side.
- **Master & jam kerja**: tambah/ubah satker, zona waktu (WIB/WITA/WIT), jam masuk/pulang,
  hari kerja, dan jam pulang Sabtu untuk hari kerja setengah hari (mis. 10:00/12:00).
- **Titik dinas dengan peta interaktif**: tambah/edit titik langsung di peta OpenStreetMap
  (klik atau geser penanda), lingkaran radius, dan input koordinat manual tetap tersedia.
- **Absensi**: GPS radius 100 m (validasi server), 8 status kehadiran (termasuk PH/Perintah Hadir
  pada absensi harian), foto opsional, deteksi mock location, antrean offline + sinkronisasi otomatis.
- **Alur absensi Android**: tombol besar **Absensi** → validasi lokasi dengan peta OpenStreetMap,
  titik dinas terdekat, radius, jarak, dan akurasi → pilihan status kehadiran → kirim/simpan offline.
  Tombol Masuk/Pulang mengikuti absensi hari ini dari server; bila server tidak terjangkau,
  keduanya ditampilkan dan data dapat disimpan offline.
- **Wizard izin**: penjelasan lokasi, kamera, notifikasi, WorkManager, dan penyimpanan sebelum login. 
- **Kebijakan fake GPS**: Android mendeteksi `Location.isMock`/`isFromMockProvider`, menampilkan
  peringatan bila Developer Options atau aplikasi mock dipilih, lalu mengirim `is_mock_suspect`.
  Server memakai `APP__MOCK_LOCATION_POLICY=flag` (development: simpan + `perlu_verifikasi`) atau
  `reject` (produksi: HTTP 422). Detail dan cara uji ada di [`docs/fake-gps.md`](docs/fake-gps.md).
- **PH / Strong Point terpisah**: strong point per satker, jadwal bulanan per personel,
  check-in **dan check-out wajib lokasi**, verifikasi, serta rekap PH sendiri. Jadwal memakai
  **roster bulanan** (anggota × tanggal 1–akhir bulan) dengan blok regu, dan input tanggal
  mendukung rentang (disimpan satu jadwal per tanggal). Rekap menandai personel yang
  **tidak strong point** dan yang **hanya check-in** beserta tanggalnya; absensi kantor tetap
  wajib pada hari strong point, dan telat kantor tidak dihitung bila sudah check-in strong point.
  Jadwal dan rekap strong point dapat diekspor PDF; PDF jadwal berbentuk roster (anggota × tanggal)
  dan rekap tidak menampilkan kolom telat serta hanya menghitung sampai hari ini karena jadwal
  tidak memakai jam mulai/selesai. Tab master strong point disembunyikan sementara karena
  titiknya belum tetap. Aplikasi Android memiliki tab **Strong** dengan tombol besar menuju wizard
  check-in/check-out: peta lokasi (marker hijau) tanpa titik dinas, koordinat, akurasi GPS, alamat
  reverse-geocode, dokumentasi foto saat check-in, dan antrean offline untuk kondisi tanpa sinyal.
  Foto dokumentasi absensi dan strong point dapat dibuka kembali dari dashboard (galeri rekap dan
  detail per orang) dengan kontrol akses pemilik/cakupan satker.
- **Pengingat jadwal tanpa FCM**: notifikasi lokal Android (WorkManager) H-1 19:00 dan
  60 menit sebelum jadwal.
- **Rencana kerja**: wizard 4 langkah di Android (jenis → dasar SP/TOR → rincian kegiatan →
  lampiran), target kuantitas/kualitas/waktu, beberapa rincian kegiatan per rencana,
  lampiran multi-berkas, date picker untuk semua tanggal, serta simpan draf, ajukan ke atasan,
  dan perbaiki rencana yang diminta revisi dengan catatan persetujuan.
- **Rekap, detail & PDF dashboard**: Rencana Kerja, Hasil Kerja, dan Output punya halaman detail
  lengkap beserta lampiran; tiap daftar mendukung filter periode/status/satker/personel dan tombol
  **Export PDF** sesuai filter (kop Polda Bali, tabel A4 landscape) melalui Chrome/Edge headless.
  **Rekap Kehadiran** dipecah menjadi tab **Rekap Kehadiran**, **Verifikasi**, dan **Detail per Orang**
  dengan filter terpisah dan **Export PDF** per tab; filter subdit/bagian dihapus dari rekap,
  absen masuk/pulang pada tanggal yang sama digabung satu baris, dan keterlambatan Dinas Kantor
  ditampilkan dalam menit.
- **Persetujuan rencana**: atasan langsung — termasuk pimpinan yang ditunjuk sebagai `atasan_id` —
  dapat menyetujui, meminta revisi, atau menolak dari dashboard web maupun dari kartu
  **Persetujuan Rencana** di Beranda Android/tombol **Tinjau Persetujuan** pada tab Rencana.
  Lampiran PDF dirender langsung di aplikasi dan gambar dapat dibuka dengan pinch-zoom.
  Akun dual-role (atasan sekaligus bawahan) melihat rencana sendiri di tab Rencana dan rencana
  bawahannya lewat tombol persetujuan; rencana sendiri tetap diajukan ke atasannya.
- **Hasil kerja & output**: realisasi + progres (slider 0–100), wizard 3 langkah dengan simpan
  draf dan kirim ke atasan, verifikasi berskor (slider 0–100), 8 jenis output, berkas bukti atau
  tautan, serta layar verifikasi untuk atasan langsung.
- **Profil & pengaturan Android**: foto profil (JPG/PNG), edit nama/pangkat (combobox 21 tingkat
  Polri)/jabatan/telepon/email, ganti password, pengaturan alamat server, dan antrean offline.
- **Profil Saya di dashboard**: halaman `/profil` untuk admin & pimpinan dengan unggah foto profil
  (JPG/PNG maks. 5 MB) dan ganti password; setiap kolom password punya ikon mata
  tampil/sembunyikan. Avatar tampil di sidebar dan header, dan ganti password mencabut seluruh
  sesi sehingga pengguna kembali ke halaman login.
- **Aplikasi web personel & atasan** (`personel/`): SPA mobile-first dengan bottom navigation
  (Beranda, Absensi, Rencana, Hasil, Output, Strong) dan alur yang sama dengan Android. Mencakup
  absensi GPS + peta OSM, status kehadiran, dokumentasi foto, antrean offline di browser, wizard
  strong point (marker hijau, koordinat, akurasi, reverse-geocode, aturan 2 jam), wizard rencana
  4 langkah, hasil/output 3 langkah, persetujuan rencana, verifikasi hasil/output berslider,
  nilai kinerja di Beranda, serta profil (foto, edit, ganti password dengan tombol mata).
- **Penilaian**: periode bulanan dibuat otomatis (penutupan manual), bobot per periode
  (default 20/20/30/30), snapshot komponen untuk audit, hari ber-jadwal PH dikeluarkan dari
  skor absensi harian.
- **Kesiapan periode**: setiap admin satker menandai satkernya siap ditutup; admin global
  melihat satker yang sudah/belum siap beserta jumlah personel yang belum dinilai.
- **Kalender hari libur**: libur nasional/cuti bersama dan libur khusus satker; tanggal libur
  otomatis dikeluarkan dari perhitungan absensi harian.
- **Dashboard pimpinan**: KPI, capaian per indikator, tren, sebaran kategori, rekap
  berpaginasi, catatan & keputusan, cetak A4, ekspor CSV. Sidebar memakai menu bergrup
  (Master Data, Absensi, Strong Point, Kinerja, Laporan, Pengaturan) dengan submenu yang dapat
  dilipat dan otomatis terbuka saat halamannya aktif. Untuk pimpinan murni, grup **Master Data**
  disembunyikan, submenu/tab **Verifikasi Offline** tidak ditampilkan, dan tombol
  **Buat Rencana/Tambah Hasil/Tambah Output/Ajukan** tidak ditampilkan.
- **Branding**: logo Polda Bali di login (terpusat), sidebar, kop laporan, aplikasi Android, dan ikon launcher. Password pada login Android dapat ditampilkan/disembunyikan lewat ikon mata.
- **Jam server palsu (development)**: `cargo run --bin fake -- DD-MM-YYYY HH:MM` mengubah jam yang dipakai backend untuk absensi; endpoint hanya aktif bila `APP__ALLOW_FAKE_TIME=true`.
- **Operasional**: audit log setiap mutasi, rate limit login 5/menit/IP, RBAC ber-scope,
  backup harian database + berkas.

## Aturan penilaian (ringkas)

| Indikator | Bobot | Komponen |
|---|---|---|
| Absensi | 20% | Rata-rata skor harian; telat/pulang cepat −1 per 10 menit (maks 20); tanpa keterangan 0; libur nasional/cuti/izin/sakit netral; pada hari strong point telat kantor diabaikan bila check-in strong point |
| Rencana Kerja | 20% | Ketepatan 40% + kelengkapan 30% + kualitas 30% |
| Hasil Kerja | 30% | Kuantitas 40% + kualitas 30% + waktu 30% |
| Output | 30% | Kelengkapan 40% + kesesuaian 30% + keabsahan 30% |

Kategori: **Sangat Baik ≥ 95**, **Baik 90–94,99**, **Cukup 80–89,99**,
**Perlu Pembinaan < 80**. Detail rumus ada di `docs/penilaian.md`.

## Verifikasi yang sudah dilakukan

- `cargo test` → **37 test lulus** (geofence, scoring, deteksi berkas, kanal login, policy mock GPS).
- `bun tools/uji/api-smoke.js` → **60/60 cek lulus** pada database uji `kinerja_test`:
  RBAC ber-scope, pembatasan login dashboard (pimpinan/admin vs atasan/personel), kanal
  `personel_web` (personel & atasan 200, admin 403), CRUD satker/personel/admin, titik dinas
  (termasuk ubah jenis & koordinat), zona waktu & jam Sabtu, kanal login Android, tutup/buka
  periode, kesiapan satker, kalender libur, jadwal & absensi PH, jam server palsu, verifikasi,
  foto dokumentasi, rekap, dan penandaan mock GPS pada policy `flag`.
- `bun tools/uji/openapi-check.js` → JSON valid, semua `$ref` ada (88 path).
- `bun run check` → 0 error · `bun run build` → sukses (dashboard dan `personel/`).
- Login `personel_web` diuji pada API: atasan & personel 200, admin 403, refresh cookie 200.
- `gradlew :app:assembleDebug` → APK debug terbentuk (7 tab, termasuk PH).
- Emulator Pixel 6a: login, absen GPS, dan navigasi modul utama.
- Chrome headless: login, Beranda, Absensi, Strong, wizard Rencana, Persetujuan, dan Profil
  aplikasi `personel/` tampil tanpa error pada lebar 500 px.

## Catatan teknis penting

- **Versi dipin & diverifikasi**: axum 0.8.9, sqlx 0.9.0 (butuh Rust ≥ 1.94), tower-http 0.7.1,
  Svelte 5.57, SvelteKit 3.2, Tailwind 4.3, Vite 8.3, Bun 1.4, AGP 9.0.1, Gradle 9.3,
  Compose BOM 2025.06.01, Kotlin 2.4.21, OkHttp 4.12.
- **SvelteKit 3** tidak lagi memakai `svelte.config.js`; konfigurasi (adapter static) berada di
  `vite.config.ts`, dan alias `$lib` diganti subpath `#lib` (didefinisikan di `package.json`).
- **AGP 9** mengaktifkan DSL baru + Kotlin built-in. Proyek ini memakai opt-out resmi
  (`android.newDsl=false`, `android.builtInKotlin=false`) agar DSL klasik tetap dipakai.
  AGP 9.5 memerlukan Gradle lebih baru dari 9.3, dan AGP 9.1+ menuntut `compileSdk 37`
  (platform 37 belum tersedia di SDK manager), sehingga dipin ke **AGP 9.0.1 + Gradle 9.3 +
  compileSdk 36**.
- **HTTP client Android memakai OkHttp, bukan Ktor**: `ktor-client-core-jvm` 3.2.0 mengandung
  nama kelas dengan spasi sehingga gagal di-dex untuk `minSdk 26`.
- **Absensi offline** disimpan di DataStore (JSON antrean) dan dikirim `WorkManager`.
  Deviasi dari rencana awal (Room) diambil agar tidak menambah ketergantungan KSP; sifatnya
  tetap durable dan idempoten karena memakai `client_uuid`.
- **Token**: access token di memori, refresh token di DataStore privat. Backend juga menerima
  `refresh_token` di body agar klien mobile bisa memperbarui sesi tanpa cookie.
- **Berkas**: foto absensi maks 5 MB (JPG/PNG, dikompres di klien), lampiran rencana 10 MB,
  bukti output 50 MB. Validasi tipe memakai magic bytes, bukan ekstensi.

## Keamanan & operasional

- RBAC ber-scope: personel hanya melihat datanya, atasan melihat satuan kerjanya,
  admin/pimpinan melihat sesuai cakupan satker (Polda/Polres/Satker), admin global penuh.
- Setiap mutasi (`POST/PUT/PATCH/DELETE`) otomatis dicatat ke `audit_log`
  (aktor, aksi, entitas, IP, user agent, status) dan dapat ditelusuri admin via `GET /audit`.
- Rate limit: login 5/menit/IP, endpoint lain 120/menit/IP (HTTP 429 + `Retry-After`).
- Untuk produksi: aktifkan HTTPS (`APP__COOKIE_SECURE=true`), ganti `JWT_SECRET`,
  ganti seluruh password demo, dan set `cleartextTrafficPermitted=false` pada
  `network_security_config.xml`.
- Backup harian:

  ```bash
  cd deploy
  chmod +x backup.sh      # sekali saja di server Linux
  ./backup.sh             # database + unggahan, retensi 30 hari
  ```

  Cara restore ada di komentar bagian bawah `deploy/backup.sh`.

## Deploy VPS

Panduan lengkap (systemd, nginx mTLS Cloudflare, dashboard di `/var/www/absensi`, web
personel di `/var/www/personel`, backup, dan pemecahan masalah) ada di
[`docs/deploy-vps.md`](docs/deploy-vps.md).

Ringkasnya, API berjalan di `127.0.0.1:7085` sebagai layanan systemd, kedua SPA disajikan
nginx pada `absensi.jagradewata.id` dan `personel.jagradewata.id`, dan database diisi dengan
profil khusus server:

```bash
sudo -u absensi bash -c 'set -a; . /etc/absensi/absensi.env; set +a; /opt/absensi/seed --reset --vps'
```

Profil `--vps` hanya mengisi Polda Bali + Ditreskrimsus, absensi & rekap strong point tanggal
1–19, jadwal strong point tanggal 1–akhir bulan (Regu 1/Regu 2), serta contoh rencana, hasil,
dan output pada status draft/revisi/disetujui. Seed development tanpa `--vps` tetap mengisi
seluruh satker.
