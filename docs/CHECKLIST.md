# Checklist Pekerjaan — Struktur Organisasi, Modul PH, dan Pengingat

Terakhir diperbarui: 17 November 2026.

Legenda: ✅ selesai & terverifikasi · 🟡 selesai, butuh uji manual di lapangan · ⬜ belum dikerjakan.

Dokumen ini memetakan seluruh rekomendasi audit awal dan permintaan restrukturisasi
ke kondisi kode saat ini. Rincian teknis ada di [`HANDOFF.md`](HANDOFF.md), aturan
nilai di [`penilaian.md`](penilaian.md), dan skenario uji di [`UAT.md`](UAT.md).

---

## 1. Logo & identitas

| # | Item | Status | Catatan |
|---|---|---|---|
| 1.1 | Logo Polda Bali di halaman login dashboard | ✅ | `dashboard/static/logo-polda-bali.png` |
| 1.2 | Logo di sidebar & header mobile dashboard | ✅ | `(app)/+layout.svelte` |
| 1.3 | Logo pada kop cetak laporan | ✅ | `laporan/+page.svelte` |
| 1.4 | Logo di layar login Android | ✅ | `LoginScreen.kt` → `R.drawable.logo_polda_bali` |
| 1.5 | Logo di top bar Android | ✅ | `MainActivity.kt` |
| 1.6 | Ikon launcher Android memakai logo Polda Bali | ✅ | `mipmap-*/ic_launcher_foreground.png` |
| 1.7 | Logo & header halaman login Android terpusat | ✅ | `LoginScreen.kt` |
| 1.8 | Tampil/sembunyikan password dengan ikon mata | ✅ | `IkonMata.kt`, `trailingIcon` pada field password |

## 2. Struktur organisasi & peran

| # | Item | Status | Catatan |
|---|---|---|---|
| 2.1 | Tiga level organisasi: Polda → Polres → Satker | ✅ | `satker_level` = `polda`, `polres`, `satker` |
| 2.2 | Satker dapat berada di bawah Polda maupun di bawah Polres | ✅ | `satker.parent_id` + validasi hierarki `service/akses.rs` |
| 2.3 | 12 peran: 5 admin, 4 pimpinan ber-scope, pimpinan, atasan, personel | ✅ | `domain/enums.rs` |
| 2.4 | Cakupan peran per satker (`scope_satker_id`) | ✅ | migrasi `0005_role_scope.sql`, unique `NULLS NOT DISTINCT` |
| 2.5 | Admin global tidak bisa dicabut dari admin global terakhir | ✅ | validasi `count_admin_global` |
| 2.6 | Admin Polda mengelola seluruh satker di bawah Polda | ✅ | resolusi cakupan rekursif |
| 2.7 | Admin Satker Polda hanya satkernya sendiri | ✅ | |
| 2.8 | Admin Polres mengelola satker di bawah Polres | ✅ | |
| 2.9 | Admin Satker Polres hanya satkernya sendiri | ✅ | |
| 2.10 | Pimpinan mengikuti cakupan yang sama (Polda/Polres/Satker) | ✅ | `is_pimpinan_any`, `require_satker` |
| 2.11 | Personel hanya melihat data sendiri | ✅ | daftar absensi, penilaian, lampiran |
| 2.12 | RBAC diterapkan di seluruh modul | ✅ | absensi, rekap, dashboard, penilaian, laporan, audit, lampiran, PH |
| 2.13 | Akun admin dan personel dipisahkan secara logis (`jenis_akun`) | ✅ | migrasi `0007_jenis_akun.sql` |
| 2.14 | Admin memakai username (tanpa NRP), personel memakai NRP | ✅ | constraint `personel_identitas_check` |
| 2.15 | Login Android hanya untuk akun personel (termasuk pimpinan) | ✅ | `client=android` + `KanalLogin`; admin 403 |

## 3. Dashboard admin (kekurangan yang dilaporkan)

| # | Item | Status | Catatan |
|---|---|---|---|
| 3.1 | Menu & halaman **Satker & Jam Kerja** | ✅ | `dashboard/src/routes/(app)/satker/+page.svelte` |
| 3.2 | Tambah satker baru (pilih induk & level) | ✅ | level `polda` hanya akar tunggal; kode unik |
| 3.3 | Ubah satker, aktif/nonaktif | ✅ | |
| 3.4 | Jam masuk & jam pulang per satker | ✅ | format `HH:MM`, divalidasi server |
| 3.5 | Hari kerja per satker | ✅ | checkbox 1–7 (Senin–Minggu) |
| 3.6 | Menu & halaman **Data Personel** | ✅ | `(app)/personel/+page.svelte`, hanya akun personel |
| 3.7 | Tambah personel (NRP, nama, pangkat, jabatan, satker, atasan) | ✅ | pangkat memakai combobox 21 tingkat Polri (terendah→tertinggi); unit & email tidak lagi diminta |
| 3.8 | Ubah personel & nonaktifkan | ✅ | |
| 3.9 | Atur peran + cakupan satker dari UI | ✅ | `GET|POST /personel/{id}/roles` |
| 3.10 | Reset password personel | ✅ | `POST /personel/{id}/reset-password` |
| 3.11 | Pencarian & filter satker | ✅ | filter unit dihapus dari form personel |
| 3.12 | Halaman **Akun Admin** terpisah (username, peran, cakupan) | ✅ | `(app)/admin/+page.svelte`, `/admin` |
| 3.13 | Admin Polda hanya dapat membuat admin_satker_polda | ✅ | validasi `boleh_membuat_role` |
| 3.14 | Admin Polres hanya dapat membuat admin_satker_polres | ✅ | |
| 3.15 | Reset password & nonaktifkan akun admin | ✅ | `POST /admin/{id}/reset-password` |
| 3.16 | Zona waktu memakai combo box 3 opsi | ✅ | WIB/WITA/WIT divalidasi server |
| 3.17 | Jam pulang Sabtu setengah hari (10:00/12:00) | ✅ | `satker.jam_pulang_sabtu`, dipakai penilaian |
| 3.18 | Periode bulan berjalan dibuat otomatis | ✅ | `find_or_current` + `pastikan_bulan_ini` |
| 3.19 | Admin global dapat membuka kembali periode tertutup | ✅ | `POST /periode/{id}/buka` + alasan audit |
| 3.20 | Halaman **Kesiapan Periode** per satker | ✅ | jumlah personel, dinilai, belum, status |
| 3.21 | Admin satker menandai siap; admin global memantau semua | ✅ | tabel `periode_satker_kesiapan` |
| 3.22 | Halaman **Hari Libur** + kalender nasional/khusus satker | ✅ | tabel `hari_libur`, `/libur` |
| 3.23 | Libur otomatis netral di penilaian absensi | ✅ | `service/penilaian.rs` + detail `hari_libur` |
| 3.24 | Pagination seragam di semua modul daftar | ✅ | komponen `Pagination.svelte`; server-side: dashboard, penilaian, rencana, hasil, output, personel, admin, persetujuan, verifikasi & detail absensi; client-side: satker, libur, titik dinas, kesiapan, PH, rekap kehadiran |
| 3.25 | Peta interaktif OpenStreetMap untuk tambah/edit titik dinas | ✅ | `PetaPicker.svelte` (Leaflet): klik peta/geser penanda mengisi lat/lng, lingkaran radius, input manual tetap bisa; edit nama/jenis/koordinat/radius/status |
| 3.26 | Jam server palsu untuk development | ✅ | `cargo run --bin fake -- 20-11-2026 07:45`; endpoint `/dev/waktu` hanya aktif bila `APP__ALLOW_FAKE_TIME=true`; produksi memakai jam sistem |
| 3.27 | Alur absensi Android bertahap | ✅ | tombol besar → validasi peta/GPS → pilihan status → kirim/simpan offline |
| 3.28 | Tombol absen adaptif status hari ini | ✅ | hanya Masuk bila belum masuk; hanya Pulang bila sudah masuk; dua tombol bila status server tidak dapat diambil |
| 3.29 | Peta absensi Android (OSM) + radius & titik terdekat | ✅ | `PetaAbsensi.kt` (osmdroid), marker lokasi, titik dinas, radius, akurasi |
| 3.30 | Wizard izin pertama kali sebelum login | ✅ | `WizardIzinScreen.kt`: lokasi, kamera, notifikasi, WorkManager/penyimpanan + alasan tiap izin |
| 3.31 | Riwayat absensi di halaman terpisah | ✅ | tombol **Riwayat Absensi** pada halaman awal tab Absensi |
| 3.32 | Wizard input rencana kerja Android 4 langkah | ✅ | `WizardRencana.kt`: jenis → dasar tugas → rincian kegiatan → lampiran; **Simpan Draf** & **Ajukan ke Atasan** |
| 3.33 | Rincian kegiatan multi-entri dengan field lengkap | ✅ | program, kegiatan, uraian, tujuan, indikator, target kuantitas/kualitas, tanggal mulai/selesai; migrasi `0013` menambah kolom pada `rencana_tahapan`; detail dashboard menampilkan seluruh field |
| 3.34 | Lampiran lebih dari satu berkas pada wizard | ✅ | pemilih dokumen multi-select; tiap berkas diunggah ke `POST /lampiran` setelah rencana tersimpan |
| 3.35 | Pengaturan tab bottom nav | ✅ | **Nilai** tetap diringkas di Beranda; **Strong Point** menjadi tab ke-6 (Beranda, Absensi, Rencana, Hasil, Output, Strong) |
| 3.36 | Ubah draft / perbaiki rencana yang diminta revisi | ✅ | tombol **Ubah Rencana** / **Perbaiki Rencana** membuka wizard terisi data lama, catatan atasan, rincian, dan lampiran; `PATCH /rencana-kerja/{id}` mengganti tahapan secara transaksional |
| 3.37 | Date picker untuk seluruh tanggal rencana | ✅ | `FieldTanggal` + `DatePickerDialog` Material 3: tanggal dasar, tanggal mulai/selesai kegiatan; tanpa input teks |
| 3.38 | Persetujuan rencana di Android untuk atasan/pimpinan | ✅ | kartu **Persetujuan Rencana** di Beranda + tombol **Tinjau Persetujuan** di tab Rencana; daftar `GET /rencana-kerja/persetujuan`, detail rincian + lampiran, keputusan Setujui / Minta Revisi / Tolak |
| 3.39 | Dual role atasan sekaligus bawahan | ✅ | tab Rencana memuat rencana sendiri (`personel_id`) sekaligus tombol persetujuan bawahan; Beranda menampilkan jumlah yang menunggu; seed: `000100` → `85000001` dan `000100` → `000002` |
| 3.40 | Pimpinan dapat menjadi atasan langsung | ✅ | gate `list_persetujuan`/`keputusan` menerima `pimpinan_any`; menu Persetujuan dashboard tampil untuk pimpinan |
| 3.41 | Filter satker/personel + Export PDF rencana kerja | ✅ | filter dikirim ke `/rencana-kerja` dan `/rencana-kerja/ekspor`; PDF A4 landscape dibuat Chrome/Edge headless |
| 3.42 | Detail & PDF Hasil Kerja | ✅ | `/hasil-kerja/{id}` (lampiran bukti) + filter periode/status/satker/personel dan `/hasil-kerja/ekspor` |
| 3.43 | Detail & PDF Output | ✅ | `/output/{id}` (berkas bukti) + filter periode/status/satker/personel dan `/output/ekspor` |
| 3.41 | Penampil lampiran PDF & gambar di Android | ✅ | `LihatLampiran.kt`: unduh via `GET /lampiran/raw`, PDF dirender `PdfRenderer` (maks. 10 halaman), gambar dengan pinch-zoom; tombol **Lihat** pada detail persetujuan dan lampiran wizard |
| 3.42 | Detail rencana untuk semua status | ✅ | tombol **Lihat Detail Rencana** pada kartu disetujui/diajukan/ditolak; **Lihat Detail** pada draft/revisi; dialog menampilkan status, rincian, lampiran (dapat dibuka), dan catatan atasan |
| 3.44 | Tab Rekap Absensi: kehadiran, verifikasi, detail per orang | ✅ | `absensi/rekap/+page.svelte`; tiap tab punya filter sendiri; filter subdit/bagian dihapus dari rekap; kolom **Menit** menampilkan total menit telat Dinas Kantor |
| 3.45 | Export PDF rekap, verifikasi, dan detail absensi | ✅ | `/absensi/rekap/ekspor`, `/absensi/verifikasi/ekspor`, `/absensi/detail/ekspor` (Chrome/Edge headless); rekap memuat baris jumlah + menit telat |
| 3.46 | Gabung absen masuk & pulang per tanggal | ✅ | `GET /absensi/harian` (dipakai tab Verifikasi & Detail + PDF); menit telat dihitung untuk status Dinas Kantor yang melewati jam masuk satker |
| 3.47 | Sidebar dashboard bergrup dengan submenu | ✅ | Dashboard; Master Data; Absensi (Rekap Kehadiran/Detail Rekap/Verifikasi Offline via `?tab=`); Strong Point (Jadwal/Rekap via `?tab=`); Kinerja; Laporan (Penilaian/Print Laporan); Pengaturan (Akun Admin/Kesiapan Periode/Periode); grup aktif terbuka otomatis |

## 4. Modul PH / Strong Point (terpisah dari absensi harian)

| # | Item | Status | Catatan |
|---|---|---|---|
| 4.1 | Tabel `strong_point`, `jadwal_ph`, `jadwal_ph_personel`, `absensi_ph` | ✅ | migrasi `0006_ph.sql` |
| 4.2 | CRUD strong point per satker (titik + radius) | ✅ | API + modal tetap ada; tab master **disembunyikan sementara** karena strong point belum punya titik tetap |
| 4.3 | Jadwal PH per satker per bulan | ✅ | filter `?bulan=YYYY-MM&satker_id=`; form dashboard hanya satker, tanggal, catatan, personel (tanpa strong point & jam) |
| 4.4 | Satu jadwal berisi banyak personel + peran | ✅ | mis. ketua/anggota |
| 4.5 | Deteksi bentrok jadwal personel | ✅ | `repo/ph.rs::ada_bentrok` (jam beririsan) |
| 4.6 | Tiap personel dapat jadwal berbeda tiap bulan | ✅ | seed: 90 jadwal contoh (Ditreskrimsus Polda Bali dua regu, tanggal 1–akhir bulan) |
| 4.7 | Check-in PH **wajib lokasi** | ✅ | HTTP 422 bila GPS kosong |
| 4.8 | Check-out PH **wajib lokasi** | ✅ | lokasi diambil ulang saat check-out |
| 4.9 | Validasi radius strong point sisi server | ✅ | jarak dihitung Haversine |
| 4.10 | Idempotensi `client_uuid` | ✅ | kirim ulang tidak menggandakan |
| 4.11 | Rekap PH terpisah (check-in/out per orang) | ✅ | `GET /ph/rekap` + tab **Rekap PH** |
| 4.12 | Hari strong point tetap dinilai absensi kantor | ✅ | Telat kantor diabaikan bila check-in strong point; bila tidak strong point, telat dihitung; pulang cepat tetap dihitung (`skor_harian_dengan_strong_point`) |
| 4.13 | Absensi PH tidak mengubah bobot 20/20/30/30 | ✅ | hanya rekap PH terpisah |
| 4.14 | Verifikasi absensi PH oleh atasan/admin | ✅ | `POST /ph/absensi/{id}/verifikasi` |
| 4.15 | Tab **Strong Point** di aplikasi Android | ✅ | Tab ke-6 di bottom nav (`PhScreen`): jadwal hari ini di atas, regu, radius, check-in/check-out ber-lokasi, riwayat bulan ini; pengingat disesuaikan untuk jadwal tanpa jam |
| 4.16 | Antrean offline untuk absensi PH | ⬜ | absensi PH saat ini butuh koneksi |
| 4.17 | Ekspor CSV rekap PH | ⬜ | baru JSON + tampilan dashboard |
| 4.18 | Rekap menandai tidak strong point & hanya check-in | ✅ | `hanya_check_in`, `tanggal_tidak_hadir`, `tanggal_hanya_check_in`; badge + tanggal di tab **Rekap Strong Point** |
| 4.19 | Sesi strong point pagi & sore | ⬜ | menyusul di laporan dan Android |
| 4.20 | Form jadwal tanpa strong point & jam | ✅ | Combo strong point dan jam mulai/selesai dihapus dari modal tambah/edit; backend memakai default (titik aktif pertama satker, 00:00–23:59) agar API lama tetap kompatibel |
| 4.21 | Roster bulanan strong point ala spreadsheet | ✅ | Tab **Jadwal Strong Point**: kolom Anggota/Pangkat × tanggal 1–akhir bulan, blok biru berlabel regu, akhir pekan merah; daftar kelola jadwal tetap tersedia |
| 4.22 | Input rentang tanggal jadwal | ✅ | `tanggal_selesai` membuat satu jadwal per tanggal (bukan satu baris rentang); semua tanggal divalidasi bentrok sebelum disimpan |
| 4.23 | Ekspor PDF jadwal & rekap strong point | ✅ | `/ph/jadwal/ekspor` dan `/ph/rekap/ekspor` (Chrome/Edge headless); rekap tanpa kolom telat; tombol Export PDF di tab Jadwal/Rekap |
| 4.24 | Seed roster sebulan Ditreskrimsus | ✅ | Dua regu: Regu 1 tanggal 1–15 dan Regu 2 tanggal 16–akhir bulan (3 anggota per regu); satker lain tetap contoh tanggal 3–5 |

## 5. Pengingat jadwal tanpa FCM

| # | Item | Status | Catatan |
|---|---|---|---|
| 5.1 | Notifikasi lokal Android (WorkManager) | ✅ | `sync/Pengingat.kt` |
| 5.2 | Pengingat H-1 pukul 19:00 waktu perangkat | ✅ | dijadwalkan per jadwal |
| 5.3 | Pengingat 60 menit sebelum jam mulai | ✅ | |
| 5.4 | Penjadwalan ulang otomatis saat jadwal dimuat | ✅ | `KinerjaViewModel.muatPh()` |
| 5.5 | Izin `POST_NOTIFICATIONS` Android 13+ | ✅ | diminta di `MainActivity` |
| 5.6 | Channel notifikasi terdaftar saat aplikasi dibuat | ✅ | `App.kt` |
| 5.7 | Pusat notifikasi di dashboard web | ⬜ | API `/notifikasi` ada, UI belum |
| 5.8 | Push daring (FCM) | ⬜ | tidak diperlukan untuk pengingat lokal |

## 6. Reset data demo & database uji

| # | Item | Status | Catatan |
|---|---|---|---|
| 6.1 | Seed baru: 1 Polda, 9 Polres, 21 Satker, 145 personel + 5 admin | ✅ | `cargo run --bin seed -- --reset` |
| 6.2 | Akun admin per level + pimpinan + atasan + personel | ✅ | lihat tabel di bawah |
| 6.3 | Data contoh jadwal PH (63 jadwal) | ✅ | |
| 6.4 | Database demo `kinerja_db` direset ke skema terbaru | ✅ | migrasi 1–13 |
| 6.5 | Database uji terpisah `kinerja_test` | ✅ | jangan dicampur dengan data demo |
| 6.6 | Skrip uji API otomatis memakai DB uji | ✅ | `bun tools/uji/api-smoke.js` (48 cek) |
| 6.7 | Validator OpenAPI | ✅ | `bun tools/uji/openapi-check.js` |
| 6.8 | Pembuatan periode otomatis diuji | ✅ | `bun tools/uji/periode-auto-check.js` (menghapus periode DB uji dahulu) |
| 6.9 | Contoh rencana draft, diajukan, dan perlu revisi untuk akun Android | ✅ | seed `--reset`: 3 dokumen + tahapan rincian + catatan revisi untuk NRP `85000001` |
| 6.10 | Contoh rantai dual-role atasan → pimpinan | ✅ | seed: rencana `000100` berstatus `diajukan` ke atasan `000002` |
| 6.11 | Lampiran contoh PDF & gambar pada pengajuan | ✅ | seed menulis file ke `APP__UPLOAD_DIR`: PDF+PNG untuk `85000001` dan personel lain `85210022`, PDF untuk `000100` dan rencana revisi |

### Akun demo (seed)

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

> Untuk uji manual, gunakan `kinerja_db`. Untuk unit/e2e otomatis, gunakan `kinerja_test`
> agar data demo tidak tercemar.

## 7. Keamanan & audit

| # | Item | Status | Catatan |
|---|---|---|---|
| 7.1 | Setiap mutasi `/api/v1` masuk `audit_log` | ✅ | middleware |
| 7.2 | Audit hanya admin global | ✅ | |
| 7.3 | Lampiran mentah dicek kepemilikan/cakupan | ✅ | `GET /lampiran/raw/{key}` |
| 7.4 | Hapus lampiran dicek pemilik/admin | ✅ | |
| 7.5 | Password di-hash Argon2id | ✅ | |
| 7.6 | Rate limit login | ✅ | 5/menit/IP |
| 7.7 | UI audit log di dashboard | ⬜ | masih via API |
| 7.8 | HTTPS + cookie `Secure` di produksi | ⬜ | lihat backlog P1 |

## 8. Dokumentasi

| # | Item | Status | Catatan |
|---|---|---|---|
| 8.1 | `openapi.json` memuat seluruh endpoint baru | ✅ | 83 path; `/admin`, `/ph/*`, kesiapan, libur, buka periode, `/absensi/hari-ini`, `/absensi/harian`, `/dev/waktu`, hasil/output draf + verifikasi, profil/foto, ekspor PDF rencana/hasil/output/rekap/verifikasi/detail absensi |
| 8.2 | `docs/HANDOFF.md` diperbarui | ✅ | struktur, peran, PH, akun, DB uji |
| 8.3 | `docs/PRD.md` diperbarui | ✅ | peran ber-scope, FR PH, notifikasi lokal |
| 8.4 | `docs/penilaian.md` diperbarui | ✅ | hari PH dikecualikan |
| 8.5 | `docs/UAT.md` diperbarui | ✅ | skenario admin satker/personel/PH |
| 8.6 | `README.md` diperbarui | ✅ | akun, fitur, jumlah data |
| 8.7 | Dokumen checklist ini | ✅ | `docs/CHECKLIST.md` |

## 9. Verifikasi teknis (dijalankan ulang 17 November 2026)

| # | Perintah | Status | Hasil |
|---|---|---|---|
| 9.1 | `cargo fmt --check` | ✅ | bersih |
| 9.2 | `cargo clippy --all-targets -- -D warnings` | ✅ | tanpa peringatan |
| 9.3 | `cargo test` | ✅ | 30 test lulus |
| 9.4 | `bun run check` (dashboard) | ✅ | 0 error |
| 9.5 | `bun run build` (dashboard) | ✅ | SPA statis terbentuk |
| 9.6 | `gradlew :app:assembleDebug` | ✅ | APK debug terbentuk |
| 9.7 | `bun tools/uji/api-smoke.js` | ✅ | 48/48 cek lulus |
| 9.8 | `bun tools/uji/openapi-check.js` | ✅ | JSON & `$ref` valid (79 path) |
| 9.9 | Pagination & form personel (combobox pangkat) | ✅ | `bun run check` 0 error, `bun run build` sukses; verifikasi visual di browser tetap disarankan |
| 9.10 | Uji API titik dinas (tambah/ubah jenis & koordinat/hapus) | ✅ | 3 cek tambahan pada `api-smoke.js` |
| 9.11 | Uji jam palsu development | ✅ | 6 cek tambahan: aktifkan, tanggal `hari-ini`, absen tanggal/jam palsu, deteksi masuk, status PH, reset |
| 9.12 | Alur absensi Android + peta + wizard izin | ✅ | diuji di emulator API 34: wizard, peta/radius, tombol adaptif, status otomatis, offline, riwayat |
| 9.13 | Wizard rencana kerja Android | ✅ | diuji di emulator API 34: 2 kegiatan, 2 lampiran PDF, simpan draf, ajukan; data tersimpan di `rencana_tahapan` + `lampiran` |
| 9.14 | Persetujuan rencana Android + dual-role | ✅ | emulator: `000100` melihat kartu persetujuan di Beranda dan tombol di tab Rencana; hanya rencana `85000001` yang tampil, 1 rencana sendiri tetap di daftar; revisi tanpa catatan ditolak, dengan catatan tersimpan; `000002` juga dapat membuka persetujuan |
| 9.15 | Lampiran contoh PDF/PNG dapat diunduh | ✅ | `GET /lampiran/raw/{key}` → 200, `application/pdf`/`image/png`, signature `%PDF`/PNG valid; detail persetujuan Android menampilkan kedua berkas |
| 9.16 | Penampil lampiran Android | ✅ | emulator: PDF `SP-123-Pengungkapan.pdf` dirender satu halaman; PNG `Bukti-Transaksi.png` tampil penuh, pinch-zoom berfungsi |
| 9.17 | Detail rencana berstatus disetujui | ✅ | emulator: kartu disetujui menampilkan **Lihat Detail Rencana**; dialog memuat status Disetujui, rincian, dan lampiran `Laporan-Pelaksanaan-Tahap-1.pdf` yang dapat dibuka |
| 9.18 | Wizard hasil kerja Android + slider progres | ✅ | emulator: 3 langkah, slider 0–100 default 100%, **Simpan Draf** dan **Kirim ke Atasan**; draft tampil di daftar dan dapat dikirim ulang |
| 9.19 | Verifikasi hasil kerja Android | ✅ | emulator `000100`: kartu Beranda + tab Hasil, detail lampiran, Terverifikasi (skor kualitas) dan Perlu Revisi (catatan wajib) |
| 9.20 | Wizard output Android + verifikasi atasan | ✅ | emulator: draf tanpa berkas, mode ubah menambah PDF, kirim ke atasan; atasan melihat 3 output, detail + PDF viewer, tolak tanpa catatan 422, setujui dengan skor 90/92 |
| 9.21 | Data demo output | ✅ | seed: draft, dalam verifikasi (PDF+PNG), perlu revisi (catatan + PDF), dan disetujui untuk `85000001`/`85210022` |
| 9.22 | Profil Android: foto, edit, ganti password | ✅ | emulator: avatar seed tampil, unggah PNG baru berhasil, edit nama/pangkat (combobox 21 tingkat)/jabatan/telepon/email tersimpan, validasi password wajib; API: ganti password mencabut sesi lama |
| 9.23 | Slider skor verifikasi | ✅ | emulator: skor kualitas 85→62, kesesuaian 90→54 dan keabsahan 92→82 pada dialog output |
| 9.24 | Tampil/sembunyi password & avatar Beranda | ✅ | emulator: ikon mata mengubah titik-titik menjadi `Polri#2026` dan kembali; avatar tampil di Beranda `85000001`, tidak tampil untuk `000100` |
| 9.25 | Validasi pangkat/jabatan profil | ✅ | API: pangkat `BRIPKA`/`JENDERAL POL` diterima, `KOLONEL` ditolak 422, jabatan <3 karakter ditolak 422 |
| 9.26 | Modal dashboard responsif | ✅ | `Modal.svelte`: tinggi maksimal `100vh-2rem`, header + tombol tutup selalu terlihat, isi digulir, footer aksi tetap tampil; modal Rencana Kerja memakai slot `footer` untuk Batal/Simpan Draft/Ajukan ke Atasan |
| 9.27 | Filter & PDF Rencana Kerja | ✅ | API: filter satker+personel menghasilkan 2 baris; PDF 200 `application/pdf` ~89 KB (~2,2 s), semua satker ~1,0 MB dengan kolom Satker; UI: `bun run check` 0 error |
| 9.28 | Detail & PDF Hasil/Output | ✅ | API: detail rencana (2 tahapan, 1 lampiran), detail hasil & output (lampiran terbaca); PDF hasil/output 200 `application/pdf` ~100 KB; UI: `bun run check` 0 error, `bun run build` sukses |

## 10. Yang belum dikerjakan (backlog)

| # | Item | Prioritas | Alasan |
|---|---|---|---|
| 10.1 | Antrean offline absensi PH | P1 | PH biasanya di lokasi dengan sinyal; perlu keputusan produk |
| 10.2 | Pusat notifikasi & pengingat di dashboard | P2 | API `/notifikasi` sudah tersedia |
| 10.3 | UI audit log di Pengaturan | P2 | API `GET /audit` sudah tersedia |
| 10.4 | Peta pemilihan titik dinas/strong point | P2 | saat ini input koordinat manual |
| 10.5 | Ekspor PDF server-side (kop + QR) | P2 | cetak A4 browser sudah jalan |
| 10.6 | Ekspor CSV rekap PH | P2 | |
| 10.7 | Jadwalkan backup + uji restore | P1 | skrip ada, cron belum |
| 10.8 | HTTPS produksi + `cleartextTrafficPermitted=false` | P1 | |
| 10.9 | Uji perangkat nyata (Android 8 & 13) | P1 | emulator sudah |
| 10.10 | Push FCM untuk notifikasi daring | P3 | pengingat lokal sudah memenuhi kebutuhan |
| 10.11 | Utoipa/OpenAPI otomatis | P3 | `openapi.json` manual + validator |
| 10.12 | Cache titik dinas untuk absensi offline sejak awal | P2 | saat offline total, radius belum diketahui sampai ada sinyal |
