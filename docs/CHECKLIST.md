# Checklist Pekerjaan — Struktur Organisasi, Modul PH, dan Pengingat

Terakhir diperbarui: 23 November 2026.

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
| 3.7 | Tambah personel (NRP, nama, pangkat, jabatan, satker, atasan) | ✅ | pangkat/satker/atasan memakai **Combobox ber-pencarian** (22 tingkat Polri + 17 pangkat ASN, ketik untuk mencari tanpa scroll); unit & email tidak lagi diminta |
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
| 3.18 | Periode bulan berjalan dibuat otomatis | 🟡 | kode & API tetap ada, **menu dinonaktifkan**; fokus aplikasi kini absensi + strong point |
| 3.19 | Admin global dapat membuka kembali periode tertutup | 🟡 | API tetap ada, menu Periode tidak ditampilkan |
| 3.20 | Halaman **Kesiapan Periode** per satker | 🟡 | halaman & API tetap ada, menu dinonaktifkan |
| 3.21 | Admin satker menandai siap; admin global memantau semua | 🟡 | lihat 3.20 |
| 3.22 | Halaman **Hari Libur** + kalender nasional/khusus satker | ✅ | tabel `hari_libur`, `/libur` |
| 3.23 | Libur otomatis netral di penilaian absensi | ✅ | `service/penilaian.rs` + detail `hari_libur` |
| 3.24 | Pagination seragam di semua modul daftar | ✅ | komponen `Pagination.svelte`; server-side: dashboard, penilaian, rencana, hasil, output, personel, admin, persetujuan, verifikasi & detail absensi; client-side: satker, libur, titik dinas, kesiapan, PH, rekap kehadiran |
| 3.25 | Peta interaktif OpenStreetMap untuk tambah/edit titik dinas | ✅ | `PetaPicker.svelte` (Leaflet): tinggi `h-72 sm:h-96`, tombol **Peta/Satelit** (ArcGIS World Imagery), pencarian alamat **Nominatim** dengan debounce 700 ms (min. 3 huruf, cache, 1 req/detik), klik hasil → peta & koordinat terisi; modal titik dinas `max-w-5xl` |
| 3.26 | Jam server palsu untuk development | ✅ | `cargo run --bin fake -- 20-11-2026 07:45`; endpoint `/dev/waktu` hanya aktif bila `APP__ALLOW_FAKE_TIME=true`; produksi memakai jam sistem |
| 3.27 | Alur absensi Android bertahap | ✅ | tombol besar → validasi peta/GPS → pilihan status → kirim/simpan offline |
| 3.28 | Tombol absen adaptif status hari ini | ✅ | hanya Masuk bila belum masuk; hanya Pulang bila sudah masuk; dua tombol bila status server tidak dapat diambil |
| 3.29 | Peta absensi Android (OSM) + radius & titik terdekat | ✅ | `PetaAbsensi.kt` (osmdroid), marker lokasi, titik dinas, radius, akurasi |
| 3.30 | Wizard izin pertama kali sebelum login | ✅ | `WizardIzinScreen.kt`: lokasi, kamera, notifikasi, WorkManager/penyimpanan + alasan tiap izin |
| 3.31 | Riwayat absensi di halaman terpisah | ✅ | tombol **Riwayat Absensi** pada halaman awal tab Absensi |
| 3.32 | Wizard input rencana kerja Android 4 langkah | ✅ | `WizardRencana.kt`: jenis → dasar tugas → rincian kegiatan → lampiran; **Simpan Draf** & **Ajukan ke Atasan** |
| 3.33 | Rincian kegiatan multi-entri dengan field lengkap | ✅ | program, kegiatan, uraian, tujuan, indikator, target kuantitas/kualitas, tanggal mulai/selesai; migrasi `0013` menambah kolom pada `rencana_tahapan`; detail dashboard menampilkan seluruh field |
| 3.34 | Lampiran lebih dari satu berkas pada wizard | ✅ | pemilih dokumen multi-select; tiap berkas diunggah ke `POST /lampiran` setelah rencana tersimpan |
| 3.35 | Pengaturan tab bottom nav | ✅ | **Beranda / Absensi / Strong** (3 tab); menu Rencana/Hasil/Output dihapus dari navigasi Android |
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
| 3.47 | Sidebar dashboard bergrup dengan submenu | ✅ | Dashboard; Master Data; Absensi (Rekap Kehadiran/Detail Rekap/Verifikasi Offline); Strong Point (Jadwal/Rekap/Penempatan Anggota/Titik); Akun Admin. Grup **Kinerja/Laporan/Pengaturan** dan menu Penilaian/Print Laporan/Kesiapan/Periode dihapus; route lama dialihkan ke `/dashboard` |
| 3.48 | Galeri dokumentasi foto absensi | ✅ | Tombol **Lihat** di rekap membuka galeri foto per personel; detail per orang menampilkan tombol **Masuk/Pulang**; berkas disajikan `/api/v1/files/...` dengan cek kepemilikan/cakupan |
| 3.49 | Pembatasan peran dashboard | ✅ | Login web hanya role admin/pimpinan; atasan/personel ditolak 403. Pimpinan murni tidak melihat Master Data, **Verifikasi Offline**, dan tombol Buat Rencana/Tambah Hasil/Tambah Output/Ajukan; akun multi-role tetap masuk lewat role pimpinan/admin |
| 3.50 | Profil Saya di dashboard | ✅ | `/profil`: unggah foto (JPG/PNG maks. 5 MB, avatar di sidebar/header) dan ganti password dengan tombol mata; ganti password mencabut semua sesi lalu diarahkan ke login |
| 3.51 | Aplikasi web personel terpisah | ✅ | `personel/` (Bun, Svelte 5, SvelteKit 3 SPA, Tailwind 4, Leaflet); desain mobile-first, bottom navigation **Beranda/Absensi/Strong**, port dev 5174 |
| 3.52 | Login web personel `personel_web` | ✅ | Hanya akun personel (termasuk atasan/pimpinan); admin 403; access token di memori + refresh cookie httpOnly `device_id=web-personel`; API 200 |
| 3.53 | Absensi & Strong Point web personel | ✅ | Peta OSM, radius/titik dinas, 8 status, foto via kamera/file, antrean offline browser; strong point marker hijau tanpa titik dinas, geocode, daftar sesi hari ini, check-in/out per jadwal |
| 3.54 | Wizard & persetujuan web personel | 🟡 | halaman Rencana/Hasil/Output/Persetujuan masih ada di disk, tetapi **tidak dijangkau navigasi**; fokus baru absensi + strong point |
| 3.55 | Profil & nilai web personel | ✅ | Foto, edit nama/pangkat 22 tingkat Polri + 17 pangkat ASN/jabatan/telepon/email, ganti password dengan tombol mata; kartu nilai periode di Beranda **dihapus** |
| 3.56 | Kebijakan lokasi tiruan server | ✅ | `APP__MOCK_LOCATION_POLICY=flag|reject`; `flag` menyimpan + `perlu_verifikasi`, `reject` HTTP 422 untuk absensi dan strong point |
| 3.57 | Deteksi & antrean fake GPS Android | ✅ | `Location.isMock`/`isFromMockProvider` + peringatan Developer Options/aplikasi mock; antrean menghitung item `ditolak` saat 422 mock |
| 3.58 | Dokumentasi fake GPS | ✅ | `docs/fake-gps.md`: policy, ADB/Fake GPS, uji endpoint, antrean, batasan, matriks uji |
| 3.59 | Combobox ber-pencarian di modal personel | ✅ | `Combobox.svelte`: input pencarian, navigasi keyboard, opsi pangkat/satker/atasan; data lama tetap tampil |
| 3.60 | Peta titik dinas: perbesar, satelit, cari alamat | ✅ | tinggi peta bertambah, basemap ArcGIS World Imagery, pencarian Nominatim debounce 700 ms tanpa tombol Cari; modal `max-w-5xl` |
| 3.61 | Dashboard fokus absensi + strong point | ✅ | filter tanggal + satker; kartu total/hadir/belum/telat/sesi strong point/cuti-izin-sakit; tabel strong point & absensi hari itu |
| 3.62 | Rekap absensi berbasis rentang tanggal | ✅ | filter `dari`/`sampai` per tab; tidak lagi bergantung pada periode bulan |
| 3.63 | Modul kinerja dinonaktifkan dari UI | ✅ | menu & route diarahkan ke `/dashboard`; API/tabel tetap (lihat HANDOFF §3.30) |

## 4. Modul Strong Point v2 (terpisah dari absensi harian)

| # | Item | Status | Catatan |
|---|---|---|---|
| 4.1 | Tabel inti PH/strong point | ✅ | `strong_point`, `penempatan_ph`, `jadwal_ph`, `jadwal_ph_personel`, `absensi_ph`; migrasi `0006` + `0017` + `0018` |
| 4.2 | Titik hanya dimiliki Polda/Polres | ✅ | `POST/PATCH /ph/strong-point` menolak level `satker`; satker di bawahnya otomatis memakai titik induk (`induk_titik_ids`) |
| 4.3 | Jadwal disusun per bulan | ✅ | Form jadwal: satker, tanggal/rentang, sesi, jam, regu, anggota; titik ditentukan otomatis dari penempatan anggota; filter bulan (tanggal 1–akhir bulan) |
| 4.4 | Salin jadwal bulan sebelumnya | ✅ | `POST /ph/jadwal/salin`; tanggal dipetakan berdasarkan hari + urutan minggu; `timpa=true` mengganti jadwal tanpa absensi |
| 4.5 | Penempatan anggota tetap | ✅ | `GET/POST /ph/penempatan`: unik per anggota, satu anggota = satu titik; modal langsung satker (termasuk unit Polda/Polres) → titik → checkbox anggota |
| 4.6 | Sesi pagi & sore dengan jam per sesi | ✅ | Satu jadwal = satu sesi + satu titik; jam default form 07:00–09:00 / 16:00–18:00, dapat diubah |
| 4.7 | Check-in/out wajib lokasi & radius | ✅ | Haversine sisi server; di luar radius 422 |
| 4.8 | Satu check-in & satu check-out per jadwal | ✅ | Ganda 409; check-in dibuka −30 menit dan ditutup di `jam_selesai`; check-out maks. `jam_selesai + 60 menit` |
| 4.9 | Telat & pulang cepat per sesi | ✅ | `absensi_ph.menit_telat` dan `menit_pulang_cepat`; tampil di rekap |
| 4.10 | Rekap PH per rentang tanggal | ✅ | `GET /ph/rekap?dari&sampai&satker_id`; total telat/pulang cepat; dibatasi sampai hari ini |
| 4.11 | Rekap menandai tidak hadir / hanya check-in | ✅ | `tanggal_tidak_hadir`, `tanggal_hanya_check_in`, `menunggu_verifikasi` |
| 4.12 | Deteksi bentrok & validasi sesi | ✅ | Bentrok jam 409; sesi selain pagi/sore 422; jam selesai ≤ mulai 422 |
| 4.13 | Export PDF jadwal & rekap | ✅ | `/ph/jadwal/ekspor` (tabel Tanggal/Sesi/Jam/Titik/Regu/Anggota) dan `/ph/rekap/ekspor` |
| 4.14 | Tab Strong Point di dashboard | ✅ | `?tab=` jadwal, rekap, penempatan, titik; filter **bulan** untuk jadwal dan rentang tanggal untuk rekap; tombol **Salin Bulan Lalu** |
| 4.15 | Titik strong point dengan peta | ✅ | `PetaPicker` satelit + pencarian alamat; modal lebar |
| 4.16 | Tab **Strong** Android | ✅ | Daftar sesi hari ini, status per sesi, jadwal berikutnya, wizard 3 langkah, antrean offline |
| 4.17 | Web personel Strong | ✅ | `/strong` daftar sesi + tombol per jadwal; riwayat 30 hari dengan telat/pulang cepat |
| 4.18 | Hari libur tidak mengubah strong point | ✅ | Jadwal mengikuti jadwal yang disusun; kalender libur hanya memengaruhi penilaian absensi kantor |
| 4.19 | Penilaian tetap memakai strong point | ✅ | Telat kantor diabaikan bila check-in strong point; pulang cepat tetap dihitung (`scoring.rs`) |
| 4.20 | Antrean offline absensi PH | ✅ | `AntreanPh` Android; web personel memakai antrean browser |
| 4.21 | Ekspor CSV rekap PH | ⬜ | baru JSON + PDF |
| 4.22 | Seed penempatan semua personel | ✅ | `seed --reset` menempatkan seluruh akun personel aktif (termasuk atasan/pimpinan) ke titik induk satkernya |
| 4.23 | Arsip jadwal lama | ✅ | Migrasi `0017` menandai jadwal `sesi IS NULL` sebagai `is_arsip`; query baru mengabaikannya |
| 4.24 | Fallback & pengingat jadwal bulanan | ✅ | Bulan berjalan tanpa jadwal otomatis memakai bulan terakhir yang tersedia (`disalin_dari_bulan`); `GET /ph/jadwal/status` memberi peringatan susun bulan depan |
| 4.25 | Daftar penempatan & ubah titik | ✅ | Tabel anggota + satker + titik; tombol **Ubah/Tempatkan** membuka modal satker/titik/anggota tanpa pilihan level |

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
| 6.3 | Data contoh jadwal PH | ✅ | seed development: 102 jadwal (sesi pagi/sore) + penempatan tetap seluruh personel |
| 6.4 | Database demo `kinerja_db` direset ke skema terbaru | ✅ | migrasi 1–17 |
| 6.5 | Database uji terpisah `kinerja_test` | ✅ | jangan dicampur dengan data demo |
| 6.6 | Skrip uji API otomatis memakai DB uji | ✅ | `bun tools/uji/api-smoke.js` (73 cek) |
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
| 8.1 | `openapi.json` memuat seluruh endpoint baru | ✅ | 92 path; termasuk `/ph/penempatan`, `/ph/jadwal/salin`, `/ph/jadwal/status`, `/dashboard/kehadiran`, filter `dari`/`sampai` |
| 8.2 | `docs/HANDOFF.md` diperbarui | ✅ | struktur, peran, PH, akun, DB uji |
| 8.3 | `docs/PRD.md` diperbarui | ✅ | peran ber-scope, FR PH, notifikasi lokal |
| 8.4 | `docs/penilaian.md` diperbarui | ✅ | hari PH dikecualikan |
| 8.5 | `docs/UAT.md` diperbarui | ✅ | skenario admin satker/personel/PH |
| 8.6 | `README.md` diperbarui | ✅ | akun, fitur, jumlah data |
| 8.7 | Dokumen checklist ini | ✅ | `docs/CHECKLIST.md` |
| 8.8 | Dokumentasi fake GPS | ✅ | `docs/fake-gps.md` + tautan di README/HANDOFF/AGENTS |

## 9. Verifikasi teknis (dijalankan ulang 23 November 2026)

| # | Perintah | Status | Hasil |
|---|---|---|---|
| 9.1 | `cargo fmt --check` | ✅ | bersih |
| 9.2 | `cargo clippy --all-targets -- -D warnings` | ✅ | tanpa peringatan |
| 9.3 | `cargo test` | ✅ | 37 test lulus |
| 9.4 | `bun run check` (dashboard) | ✅ | 0 error |
| 9.5 | `bun run build` (dashboard) | ✅ | SPA statis terbentuk |
| 9.6 | `bun run check` + `bun run build` (personel) | ✅ | 0 error; build statis terbentuk |
| 9.7 | `gradlew :app:assembleDebug` | ✅ | APK debug terbentuk |
| 9.8 | `bun tools/uji/api-smoke.js` | ✅ | 74/74 cek lulus |
| 9.9 | `bun tools/uji/openapi-check.js` | ✅ | JSON & `$ref` valid (92 path) |
| 9.10 | Migrasi `0017_strong_point_v2.sql` + `0018_strong_point_bulanan.sql` | ✅ | terpasang di `kinerja_db` & `kinerja_test` |
| 9.11 | Uji E2E strong point v2 | ✅ | penempatan tetap (upsert), salin bulan sebelumnya, status/pengingat, fallback otomatis (fake time Desember), check-in telat 25 menit, pulang cepat 30 menit, jendela sesi, radius, PDF |
| 9.12 | Uji API titik dinas | ✅ | tambah/ubah jenis & koordinat/hapus |
| 9.13 | Uji jam palsu development | ✅ | aktifkan, tanggal `hari-ini`, absen tanggal/jam palsu, deteksi masuk, status PH, reset |
| 9.14 | Alur absensi Android + peta + wizard izin | ✅ | emulator API 34: wizard, peta/radius, tombol adaptif, status otomatis, offline, riwayat |
| 9.15 | Modul kinerja (Rencana/Hasil/Output/Persetujuan) | 🟡 | API & data tetap; halaman tidak dijangkau navigasi baru |
| 9.16 | UI baru dashboard & web personel | ✅ | `bun run check` 0 error, `bun run build` sukses; uji visual browser tetap disarankan |
| 9.17 | Modal dashboard responsif | ✅ | `Modal.svelte`: tinggi maksimal `100vh-2rem`, header + tombol tutup selalu terlihat, isi digulir, footer aksi tetap tampil |
| 9.18 | PDF modul kinerja (dorman) | ✅ | rencana/hasil/output masih dapat diekspor; tidak dijangkau UI baru |
| 9.19 | Aplikasi web personel | ✅ | `personel/`: `bun run check` 0 error, `bun run build` sukses; Chrome headless 500 px: login, Beranda, Absensi, Strong, Profil tampil |
| 9.20 | Kanal login `personel_web` + refresh cookie | ✅ | API: atasan/personel login 200, admin 403, refresh cookie 200; `cargo test` 37 lulus |
| 9.21 | Mock GPS policy `flag` | ✅ | API smoke: absensi `is_mock_suspect=true` → 200, `perlu_verifikasi=true` |
| 9.22 | Mock GPS policy `reject` | ✅ | Instance sementara `APP__MOCK_LOCATION_POLICY=reject` → absensi mock 422 tanpa menyimpan data |
| 9.23 | Android deteksi mock + build | ✅ | `:app:assembleDebug` sukses; peringatan Developer Options/mock app dan hitungan `ditolak` terkompilasi |

## 10. Yang belum dikerjakan (backlog)

| # | Item | Prioritas | Alasan |
|---|---|---|---|
| 10.1 | Ekspor CSV rekap PH | P2 | JSON + PDF sudah ada |
| 10.2 | Pusat notifikasi & pengingat di dashboard | P2 | API `/notifikasi` sudah tersedia |
| 10.3 | UI audit log di Pengaturan | P2 | API `GET /audit` sudah tersedia |
| 10.4 | Uji lapangan penempatan tetap, salin bulanan, fallback & toleransi jam | P1 | pastikan −30/+60 menit dan pemetaan minggu sesuai kondisi lapangan |
| 10.5 | Ekspor PDF server-side (kop + QR) | P2 | PDF sudah memakai Chrome/Edge headless |
| 10.6 | Bersihkan halaman kinerja dorman | P3 | API/tabel sengaja dipertahankan |
| 10.7 | Jadwalkan backup + uji restore | P1 | skrip ada, cron belum |
| 10.8 | HTTPS produksi + `cleartextTrafficPermitted=false` | P1 | |
| 10.9 | Uji perangkat nyata (Android 8 & 13) | P1 | emulator sudah |
| 10.10 | Push FCM untuk notifikasi daring | P3 | pengingat lokal sudah memenuhi kebutuhan |
| 10.11 | Utoipa/OpenAPI otomatis | P3 | `openapi.json` manual + validator |
| 10.12 | Cache titik dinas untuk absensi offline sejak awal | P2 | saat offline total, radius belum diketahui sampai ada sinyal |

## 11. Deploy VPS

| # | Item | Status | Catatan |
|---|---|---|---|
| 11.1 | Seed demo VPS khusus Ditreskrimsus | ✅ | `seed --vps`: Polda + Ditreskrimsus, 40 personel, penempatan titik tetap, jadwal PH bulan berjalan (pagi/sore), absensi/rekap PH sampai hari ini (maks. 19), rencana/hasil/output draft–revisi–disetujui |
| 11.2 | Dokumentasi deploy VPS native | ✅ | `docs/deploy-vps.md`: PostgreSQL, systemd `absensi-api`, nginx mTLS Cloudflare, `/var/www/absensi` & `/var/www/personel`, backup, pemecahan masalah |
| 11.3 | Eksekusi deploy di VPS produksi | ⬜ | Ikuti `docs/deploy-vps.md`; ganti password demo, `JWT_SECRET`, dan aktifkan Authenticated Origin Pulls |
