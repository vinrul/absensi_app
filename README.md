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
| `AGENTS.md` | Aturan singkat untuk agent AI |

## Isi repositori

| Folder | Isi | Teknologi |
|---|---|---|
| `backend/` | REST API, autentikasi, geofencing, mesin penilaian | Rust (axum 0.8, sqlx 0.9, PostgreSQL 18) |
| `dashboard/` | Dashboard pimpinan & admin (SPA) | Svelte 5, SvelteKit 3, Tailwind CSS 4 — **Bun** |
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
cargo run --bin seed -- --reset # data demo: 1 Polda, 9 Polres, 21 satker, 145 personel + 5 admin
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

Admin memakai **username** dan hanya dapat login lewat dashboard web. Personel (termasuk
pimpinan/atasan) memakai **NRP** dan dapat login lewat web maupun Android.

Data demo ada di database `kinerja_db`. Untuk uji otomatis gunakan database terpisah
`kinerja_test` (lihat `tools/uji/api-smoke.js`).

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

### 3. Aplikasi Android

```powershell
cd F:\absensi_polda_bali\android
.\gradlew.bat :app:assembleDebug
# APK: app\build\outputs\apk\debug\app-debug.apk

# opsional: uji di emulator
adb install -r app\build\outputs\apk\debug\app-debug.apk
```

Alamat server default `http://10.0.2.2:8080/api/v1/` (emulator). Untuk perangkat
nyata, ubah lewat menu **Profil → Server** menjadi `http://<ip-server>:8080/api/v1/`.

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
- **PH / Strong Point terpisah**: strong point per satker, jadwal bulanan per personel,
  check-in **dan check-out wajib lokasi**, verifikasi, serta rekap PH sendiri.
- **Pengingat jadwal tanpa FCM**: notifikasi lokal Android (WorkManager) H-1 19:00 dan
  60 menit sebelum jadwal.
- **Rencana kerja**: dasar SP/TOR, target kuantitas/kualitas/waktu, tahapan, lampiran,
  alur persetujuan dengan catatan.
- **Hasil kerja & output**: realisasi + progres, verifikasi berskor, 8 jenis output,
  berkas bukti atau tautan.
- **Penilaian**: periode bulanan dibuat otomatis (penutupan manual), bobot per periode
  (default 20/20/30/30), snapshot komponen untuk audit, hari ber-jadwal PH dikeluarkan dari
  skor absensi harian.
- **Kesiapan periode**: setiap admin satker menandai satkernya siap ditutup; admin global
  melihat satker yang sudah/belum siap beserta jumlah personel yang belum dinilai.
- **Kalender hari libur**: libur nasional/cuti bersama dan libur khusus satker; tanggal libur
  otomatis dikeluarkan dari perhitungan absensi harian.
- **Dashboard pimpinan**: KPI, capaian per indikator, tren, sebaran kategori, rekap
  berpaginasi, catatan & keputusan, cetak A4, ekspor CSV.
- **Branding**: logo Polda Bali di login (terpusat), sidebar, kop laporan, aplikasi Android, dan ikon launcher. Password pada login Android dapat ditampilkan/disembunyikan lewat ikon mata.
- **Jam server palsu (development)**: `cargo run --bin fake -- DD-MM-YYYY HH:MM` mengubah jam yang dipakai backend untuk absensi; endpoint hanya aktif bila `APP__ALLOW_FAKE_TIME=true`.
- **Operasional**: audit log setiap mutasi, rate limit login 5/menit/IP, RBAC ber-scope,
  backup harian database + berkas.

## Aturan penilaian (ringkas)

| Indikator | Bobot | Komponen |
|---|---|---|
| Absensi | 20% | Rata-rata skor harian; telat/pulang cepat −1 per 10 menit (maks 20); tanpa keterangan 0; libur nasional/cuti/izin/sakit dan hari ber-jadwal PH netral |
| Rencana Kerja | 20% | Ketepatan 40% + kelengkapan 30% + kualitas 30% |
| Hasil Kerja | 30% | Kuantitas 40% + kualitas 30% + waktu 30% |
| Output | 30% | Kelengkapan 40% + kesesuaian 30% + keabsahan 30% |

Kategori: **Sangat Baik ≥ 95**, **Baik 90–94,99**, **Cukup 80–89,99**,
**Perlu Pembinaan < 80**. Detail rumus ada di `docs/penilaian.md`.

## Verifikasi yang sudah dilakukan

- `cargo test` → **30 test lulus** (geofence, scoring, deteksi berkas, kanal login).
- `bun tools/uji/api-smoke.js` → **48/48 cek lulus** pada database uji `kinerja_test`:
  RBAC ber-scope, CRUD satker/personel/admin, titik dinas (termasuk ubah jenis & koordinat),
  zona waktu & jam Sabtu, kanal login Android, tutup/buka periode, kesiapan satker,
  kalender libur, jadwal & absensi PH, jam server palsu, verifikasi, rekap.
- `bun tools/uji/openapi-check.js` → JSON valid, semua `$ref` ada (70 path).
- `bun run check` → 0 error · `bun run build` → sukses.
- `gradlew :app:assembleDebug` → APK debug terbentuk (7 tab, termasuk PH).
- Emulator Pixel 6a: login, absen GPS, dan navigasi modul utama.

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
