# Checklist UAT — Absensi & Strong Point Personel Polda Bali

> **Catatan (23 November 2026):** Fokus aplikasi kini **absensi + strong point**. Skenario
> Rencana/Hasil/Output/Penilaian/Periode di dokumen ini **tidak berlaku untuk UI baru** (modul
> dorman). Aturan strong point “2 jam” diganti jendela sesi: check-in dibuka 30 menit sebelum
> jam mulai dan ditutup di jam selesai; check-out maksimal 60 menit setelah jam selesai.
> Satu jadwal = satu sesi pagi/sore. Lihat `HANDOFF.md` §3.17 dan `CHECKLIST.md` §4.

Status: ✅ = sudah diverifikasi otomatis (API/e2e) · ⬜ = perlu diuji manual di lapangan.

Cara pakai: jalankan `cargo run --bin seed -- --reset` untuk data demo di `kinerja_db`,
lalu ikuti skenario. Untuk uji otomatis gunakan database terpisah `kinerja_test`
(`bun tools/uji/api-smoke.js`). Catat temuan pada kolom Catatan.

Akun demo: lihat tabel di `docs/CHECKLIST.md` §6 (admin global `superuser / Superuser#2026`).

## A. Admin Sistem (`superuser` / `Superuser#2026`)

| # | Skenario | Hasil yang diharapkan | Status | Catatan |
|---|---|---|---|---|
| A1 | Buka `/docs` | Daftar seluruh operasi (96 path) tampil, filter tag berfungsi | ✅ | |
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
| A33 | Edit titik dinas: ubah nama, jenis, koordinat/penanda, radius, status | Perubahan tersimpan dan tampil di tabel | ✅ | API ✅; UI perlu uji visual |
| A34 | Rencana Kerja → filter Satker lalu Personel | Daftar menyempit sesuai pilihan; pilihan personel mengikuti satker terpilih | ✅ | API + UI |
| A35 | Rencana Kerja → **Export PDF** dengan filter | Berkas `rencana-kerja-YYYYMMDD.pdf` terunduh dari Chrome headless; kop, ringkasan filter, dan tabel sesuai data | ✅ | API |
| A36 | Buka detail rencana dari tabel | Halaman `/rencana-kerja/{id}` menampilkan program, kegiatan, uraian, dasar tugas, target, seluruh rincian tahapan, dan lampiran yang dapat dibuka | ✅ | UI + API |
| A37 | Buka detail hasil kerja | `/hasil-kerja/{id}` menampilkan rencana, personel, progres, realisasi, kendala, komponen nilai, catatan atasan, dan lampiran bukti | ✅ | UI + API |
| A38 | Buka detail output | `/output/{id}` menampilkan jenis, judul, uraian, tautan, komponen nilai, catatan, dan berkas bukti | ✅ | UI + API |
| A39 | Hasil Kerja → filter + **Export PDF** | Tabel dan PDF mengikuti periode/status/satker/personel; PDF memuat kolom personel, satker, rencana, progres, realisasi, nilai, status | ✅ | API |
| A40 | Output → filter + **Export PDF** | Tabel dan PDF mengikuti filter; PDF memuat personel, satker, judul/uraian, jenis, tanggal, nilai, status | ✅ | API |
| A41 | Rekap Absensi → tab **Rekap Kehadiran** | Filter hanya periode + satker (subdit/bagian dihapus); tabel memuat hadir, telat (hari), **menit telat**, DL, piket, cuti, izin, sakit, PH, TK, menunggu | ✅ | API + UI |
| A42 | Tab **Verifikasi** → tanggal dengan absen masuk & pulang | Kedua absen tampil dalam **satu baris** (kolom Masuk/Pulang), menit telat tampil bila Dinas Kantor terlambat; Setujui memverifikasi semua catatan pada tanggal itu | ✅ | API |
| A43 | Tab **Detail per Orang** → pilih personel | Riwayat harian digabung per tanggal: Masuk, Pulang, Status, Telat, Radius, Kondisi, Verifikasi, Catatan | ✅ | API |
| A44 | **Export PDF** pada tiap tab | `rekap-absensi`, `verifikasi-absensi`, dan `detail-absensi` terunduh sesuai filter; rekap memuat kolom **Menit** + jumlah, verifikasi/detail memakai baris per tanggal gabungan | ✅ | API |
| A45 | Tab Detail tanpa memilih personel | Pesan “Pilih personel terlebih dahulu”; tombol Export PDF menampilkan pesan yang sama tanpa memanggil server | ⬜ | UI |
| A46 | Sidebar dashboard bergrup | Menu tampil sesuai peran; submenu **Detail Rekap**/**Verifikasi Offline** membuka tab yang sesuai, **Jadwal/Rekap Strong Point** membuka tab PH terkait, grup aktif terbuka otomatis, dan judul header mengikuti submenu | ✅ | UI (Chrome headless) |
| A47 | Buka `/ph?tab=strong` | Dialihkan ke `?tab=jadwal`; tombol master **Strong Point** tidak tampil; tab hanya Jadwal Strong Point & Rekap Strong Point | ✅ | UI (Chrome headless) |
| A48 | Hari kerja dengan jadwal strong point; personel check-in strong point lalu absen kantor 08:30 | Menit telat absensi kantor pada hari itu **0** (dikecualikan) dan detail menandai **Strong point** | ✅ | API + seed |
| A49 | Personel punya jadwal strong point tetapi tidak check-in, lalu absen kantor 08:30 | Menit telat tetap dihitung (60 menit bila jam masuk 07:30) | ✅ | API + seed |
| A50 | Rekap Strong Point (rentang tanggal) | Kolom Check-in, **Hanya Check-in**, **Tidak Strong Point**, total telat, total pulang cepat; jadwal setelah hari ini tidak dihitung | ✅ | API + UI |
| A51 | Tambah/Edit Jadwal Strong Point | Form memuat satker, tanggal/rentang, **sesi pagi/sore**, jam mulai/selesai, regu, personel; **tanpa pilihan titik**; titik diambil otomatis dari penempatan anggota | ✅ | UI + API |
| A52 | Salin jadwal bulan sebelumnya | `POST /ph/jadwal/salin` menyalin jadwal dengan pemetaan hari + urutan minggu; jadwal tujuan tanpa absensi diganti jika `timpa=true` | ✅ | API + UI |
| A53 | Buka tab Jadwal Strong Point | Daftar jadwal berpaginasi (Tanggal/Sesi/Jam/Titik/Regu/Anggota), bukan roster bulanan | ✅ | UI + API |
| A54 | **Export PDF** pada tab Jadwal Strong Point | PDF tabel jadwal sesuai rentang & satker | ✅ | API + Chrome headless |
| A55 | **Export PDF** pada tab Rekap Strong Point | PDF rekap per rentang; memuat telat & pulang cepat; perhitungan dibatasi sampai hari ini | ✅ | API |
| A55a | Bulan berjalan belum punya jadwal (fake time Desember) | Sistem menyalin jadwal November secara otomatis dan menandai `disalin_dari_bulan` di dashboard | ✅ | API + smoke |
| A55b | Penempatan anggota: satker (termasuk Polda/Polres) → titik → checkbox anggota (tanpa level) | Simpan membuat satu baris per anggota; simpan ulang tidak menggandakan | ✅ | API + UI |
| A55c | Menyusun jadwal untuk anggota yang belum ditempatkan | Server menolak 422 dan meminta anggota ditempatkan lebih dulu | ✅ | API |
| A55d | Buka aplikasi Android | Hanya ada tab **Beranda** dan **Absensi**; wizard absensi berurutan status → GPS → foto → kirim | ✅ | build + emulator |
| A55e | Check-in Dinas Kantor lalu tekan Absensi lagi | Scan kedua otomatis menjadi check-out tanpa memilih status; konteks berubah menjadi `selesai` | ✅ | API + emulator |
| A55f | Piket lintas hari: check-in 23:00, scan besok 22:00 | Scan dibaca sebagai check-out siklus piket; selisih > 25 jam menjadi check-in baru | ✅ | API + fake time |
| A55g | Hari dengan jadwal strong point | Pilihan Dinas Kantor dinonaktifkan; check-in/check-out strong point dicatat juga sebagai absensi `ph` dan masuk rekap kehadiran | ✅ | API + emulator |
| A55h | Sesi pagi lupa check-out, jendela sesi sore dibuka | Scan sore menjadi check-in sesi sore; sesi pagi tetap tanpa check-out dan tampil terpisah | ✅ | API |
| A56 | Tombol **Lihat** dokumentasi di Rekap Absensi | Galeri foto absensi per personel tampil (tanggal, masuk/pulang, jam, status); hanya pengelola sesuai cakupan yang dapat membuka | ✅ | API + build |
| A57 | Dokumentasi di **Detail per Orang** | Tombol **Masuk** dan **Pulang** tampil pada tanggal yang memiliki foto; modal menampilkan gambar penuh | ✅ | API + build |
| A58 | Tombol **Lihat** dokumentasi di Rekap Strong Point | Galeri foto check-in strong point per personel tampil; foto tanpa token ditolak 401, di luar cakupan 403 | ✅ | API |
| A59 | Login dashboard dengan akun **atasan langsung** atau **personel** saja | Ditolak 403 dengan pesan dashboard hanya untuk pimpinan/admin; diarahkan memakai Android | ✅ | API |
| A60 | Login dashboard akun **pimpinan** yang juga atasan langsung + personel | Tetap berhasil karena memiliki role pimpinan; dashboard memakai gabungan kemampuan role | ✅ | API |
| A61 | Jam server di dashboard & web personel | Jam WITA berdetak tiap detik, tanggal tampil di dashboard, indikator sinkron hijau; saat jam palsu aktif muncul badge **PALSU** | ✅ | build + API |
| A62 | Rekap Kehadiran → **Matriks Harian** | Satu baris per personel, kolom per tanggal (maks. 31 hari); sel menampilkan status, jam check-in/out, telat (**T**), dan pulang cepat (**PC**); rentang >31 hari ditolak | ✅ | API + Chrome headless |
| A63 | Klik sel **PH / Dinas Luar / Cuti / Izin / Sakit / Operasi** | Modal menampilkan personel, tanggal, status, jam, telat/pulang cepat, foto bila ada, alamat hasil reverse geocode, dan peta Leaflet bertanda lokasi; status tanpa koordinat menampilkan “Lokasi tidak dicatat” | ✅ | Chrome headless |
| A64 | Tombol **PDF Ringkasan** dan **PDF Matriks Harian** | Masing-masing mengunduh PDF sesuai tampilan; PDF matriks dibagi per minggu, memuat status/jam/telat/pulang cepat, maks. 31 hari; rentang lebih panjang ditolak | ✅ | API + build |
| A61 | Sidebar pimpinan murni | Grup **Master Data** tidak tampil; akses langsung `/satker`, `/personel`, `/libur`, `/absensi/titik-dinas` dialihkan ke dashboard | ✅ | UI build |
| A62 | Aksi Kinerja pimpinan murni | Tombol **Buat Rencana**, **Tambah Hasil**, **Tambah Output**, dan **Ajukan** rencana tidak tampil; admin tetap melihat | ✅ | UI build |
| A63 | Tab **Verifikasi Offline** pimpinan murni | Submenu **Verifikasi Offline** dan tab **Verifikasi** disembunyikan; akses `?tab=verifikasi` dialihkan ke Rekap Kehadiran | ✅ | UI build |
| A64 | Buka **Profil Saya** lewat avatar/nama di sidebar atau header | Halaman `/profil` menampilkan nama, pangkat, jabatan, NRP/username, satker, dan badge peran; judul header menjadi **Profil Saya** | ✅ | build + API |
| A65 | Profil → unggah foto JPG/PNG ≤ 5 MB | Pratinjau tampil, unggahan berhasil, avatar berubah di profil, sidebar, dan header; berkas PDF atau > 5 MB ditolak dengan pesan | ✅ | build + endpoint (Android) |
| A66 | Profil → **Ganti Password** dengan ikon mata | Ketiga kolom dapat ditampilkan/disembunyikan; validasi minimal 8 karakter, huruf + angka, dan konfirmasi sama; setelah sukses semua sesi dicabut dan pengguna diarahkan ke `/login` | ✅ | build + endpoint (Android) |
| A67 | Hari kerja tanpa absensi | Sel matriks menampilkan **TA** “Tidak absen”, termasuk hari ini sebelum check-in; hari libur, akhir pekan di luar hari kerja, jadwal mendatang, dan personel nonaktif tidak ditandai | ✅ | API + build |
| A68 | Check-in tanpa check-out / check-out tanpa check-in | Badge status dan **TC** tampil terpisah (mis. **DK TC**, **PH TC**); **TM** untuk check-out tanpa check-in; pada hari berjalan check-in tanpa check-out hanya menampilkan status dan jam masuk sampai tanggal berganti | ✅ | API + build |
| A69 | Legenda dan PDF matriks | Tabel, modal detail, dan PDF memuat penanda TA/TC/TM, sesi strong point, dan keterangannya | ✅ | build + PDF |
| A70 | Jadwal strong point pagi/sore | Sel menampilkan sesi Pagi dan Sore beserta jam jadwal/realisasi; jadwal tanpa absensi tampil **PH + TA**; jadwal dengan status Izin/Sakit/Dinas Luar/Cuti tetap menampilkan status tersebut bersama penanda **Jadwal PH** | ✅ | API + Chrome headless |
| A71 | Tab **Rekap Harian** | Tanggal default hari ini dan dapat dipilih; filter satker; kartu Tidak Absen, Jadwal PH, Sudah PH, DK, DL, Piket, Operasi, Cuti, Izin, Sakit, TK, dan Libur menampilkan jumlah personel; klik kartu membuka daftar orang dengan status/jam/sesi dan tombol Detail | ✅ | Chrome headless |

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
| C9 | Bottom navigation Android | Hanya **Beranda** dan **Absensi**; tab Strong dihapus | ✅ | emulator |
| C9a | Wizard lokasi strong point | Peta OSM tanpa titik dinas, marker hijau lokasi, koordinat, akurasi GPS, dan alamat hasil geocode `geocode.vinrul.my.id` tampil | ✅ | emulator |
| C9b | Check-in → dokumentasi → kirim | Progress "Langkah 2 dari 3", foto/documentasi (dapat dilewati bila kamera tidak tersedia), lalu kirim; status berubah menjadi Check-in pukul HH:mm | ✅ | emulator |
| C9c | Check-out | Langsung ke "Langkah 2 dari 2" tanpa dokumentasi; status menjadi Sudah check-out pukul HH:mm | ✅ | emulator |
| C9d | Offline (mode pesawat) | Dua tombol CHECK-IN/CHECK-OUT (Offline) muncul; kirim menampilkan “Kirim Strong Point (Offline)”, antrean tersimpan, lalu terkirim otomatis saat jaringan kembali | ✅ | emulator + API |
| C9e | Setelah check-out sesi pagi | Sesi pagi terkunci (check-in/out hanya sekali); sesi sore tampil sebagai jadwal terpisah dengan tombol check-in sendiri | ✅ | emulator |
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
| C25 | Riwayat absensi per tanggal | Satu baris per tanggal: status, check-in, check-out; jadwal dan realisasi strong point tampil pada tanggal yang sama; filter **Dari/Sampai** default tanggal 1 s.d. akhir bulan berjalan | ✅ | emulator |
| C26 | Jam server palsu `20-11-2026 07:45` | Absensi tercatat tanggal 20 Nov 2026 pukul 07:45; reset mengembalikan jam sistem | ✅ | API + emulator |
| C27 | Pilih status **PH (Perintah Hadir)** | Tersimpan sebagai status `ph` dan dihitung hadir | ✅ | API |
| C28 | Buka tab Rencana → **+ Baru** | Wizard langkah 1 (jenis rencana) tampil; bottom nav menampilkan 6 tab termasuk Strong | ✅ | emulator |
| C29 | Pilih jenis, isi dasar SP, tambah 2 kegiatan | Langkah 3 menampilkan daftar rincian dengan ubah/hapus; field target & tanggal tervalidasi | ✅ | emulator |
| C30 | Tambah 2 lampiran PDF sekaligus | Langkah 4 menampilkan dua berkas; dapat dihapus sebelum simpan | ✅ | emulator |
| C31 | Tekan **Simpan Draf** | Kembali ke daftar, kartu berstatus Draft, lampiran terunggah, pesan sukses | ✅ | emulator + DB |
| C32 | Tekan **Ajukan ke Atasan** | Kartu berubah menjadi Diajukan dan notifikasi terkirim ke atasan | ✅ | emulator + API |
| C33 | Buka rencana `perlu_revisi` → **Perbaiki Rencana** | Wizard terisi jenis, dasar tugas, rincian kegiatan, lampiran lama, dan catatan atasan | ✅ | emulator |
| C34 | Pilih tanggal lewat kalender | Date picker tampil; tanggal dasar dan tanggal kegiatan tersimpan sesuai pilihan | ✅ | emulator + DB |
| C35 | **Simpan Perubahan** lalu **Ajukan Ulang ke Atasan** | Perubahan tersimpan (status tetap `perlu_revisi`), lalu berubah `diajukan` setelah diajukan ulang | ✅ | emulator + API |
| C36 | Jalankan `seed --reset` lalu buka tab Rencana | Tampil 5 dokumen: 2 disetujui, 1 draft, 1 diajukan, 1 perlu revisi dengan catatan atasan | ✅ | seed + emulator |
| C37 | Buka **Lihat Detail Rencana** pada rencana disetujui | Dialog menampilkan status Disetujui, rincian kegiatan, lampiran PDF, dan catatan; lampiran dapat dibuka | ✅ | emulator |
| C38 | Tab Hasil → **+ Baru** → pilih rencana → isi realisasi | Slider progres 0–100 tampil dengan default **100%**; tanggal dipilih lewat kalender | ✅ | emulator |
| C39 | **Simpan Draf** pada wizard hasil kerja | Kartu baru berstatus Draft dan dapat diubah/dikirim kapan saja | ✅ | emulator |
| C40 | Tab Output → **+ Baru** → pilih hasil terverifikasi → data output | Jenis 8 pilihan, judul/uraian/tanggal/URL; tombol **Simpan Draf** dan **Kirim ke Atasan** tersedia | ✅ | emulator |
| C41 | Simpan draf output → Ubah → tambah PDF → **Kirim ke Atasan** | Berkas terunggah, status menjadi Dalam Verifikasi, pesan sukses tampil | ✅ | emulator + API |
| C42 | Profil → **Ganti Foto Profil** → pilih gambar | Avatar bulat berubah sesuai gambar; foto disimpan ke `upload_dir/foto/<id>/` | ✅ | emulator + API |
| C43 | Profil → **Edit Profil** → ubah pangkat lewat combobox & jabatan → Simpan | Combobox berisi 22 pangkat Polri (termasuk Brigpol) + 17 pangkat ASN dari Juru Muda I/a sampai Pembina Utama IV/e; pilihan baru tampil di kartu profil (`BRIPTU · … Madya`), telepon/email tetap tersimpan | ✅ | emulator + API |
| C44 | Profil → **Ganti Password** → kosongkan lalu Simpan | Validasi “Password lama wajib diisi”; konfirmasi tidak sama juga ditolak | ✅ | emulator |
| C45 | Ganti password dengan data benar | Semua sesi dicabut, aplikasi kembali ke layar login; password lama tidak berlaku | ✅ | API |
| C46 | Tekan ikon mata pada kolom password | Password berubah dari titik-titik menjadi teks biasa, lalu kembali tersembunyi saat ditekan lagi | ✅ | emulator |
| C47 | Beranda personel dengan foto profil | Avatar bulat tampil di kiri salam; akun tanpa foto (mis. `000100`) tidak menampilkan avatar/placeholder | ✅ | emulator |
| C48 | Server `APP__MOCK_LOCATION_POLICY=flag`, absen memakai Fake GPS | API 200; data tersimpan dengan `is_mock_suspect=true` dan `perlu_verifikasi=true`; aplikasi menampilkan peringatan | ✅ | API (smoke) |
| C49 | Server `APP__MOCK_LOCATION_POLICY=reject`, absen/strong point memakai Fake GPS | API 422 dengan pesan lokasi tiruan; tidak ada baris absensi/strong point baru | ✅ | API |
| C50 | Developer Options aktif tanpa aplikasi mock; absensi memakai GPS asli | Absensi tetap 200; hanya muncul peringatan informatif | ⬜ | emulator |
| C51 | Aplikasi mock dipilih tetapi tidak aktif | Peringatan “aplikasi lokasi tiruan terpilih”; absensi GPS asli tetap boleh dikirim | ⬜ | emulator |
| C52 | Antrean offline berisi absensi/strong point mock, lalu dikirim saat server `reject` | Item dihapus dari antrean, jumlah “ditolak” tampil, tidak ada data baru di server | ✅ | build (logika) |
| C53 | Jam server di header Android | Jam WITA berdetak tiap detik di bawah judul; badge **PALSU** saat fake time; reset fake time kembali ke jam sistem tanpa restart | ✅ | emulator |

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
| D11 | Login Android sebagai atasan (`000100`) | Kartu **Persetujuan Rencana** tampil di Beranda; tombol **Tinjau Persetujuan** membuka daftar bawahan langsung berstatus `diajukan` di tab Rencana | ✅ | emulator |
| D12 | Buka detail lalu **Minta Revisi** tanpa catatan | Validasi wajib catatan; setelah diisi status `perlu_revisi` dan `jumlah_revisi` bertambah | ✅ | emulator + DB |
| D13 | Dual role: buka tab Rencana | Rencana milik `000100` tetap tampil, sekaligus tombol persetujuan; rencana `85000001` hanya tampil setelah tombol ditekan | ✅ | emulator |
| D14 | Login pimpinan `000002` | Rencana `000100` tampil di daftar persetujuan (pimpinan sebagai atasan langsung) | ✅ | emulator + API |
| D15 | Buka detail pengajuan dengan lampiran | Lampiran PDF dan PNG tampil dengan nama & ukuran; kedua berkas dapat diunduh | ✅ | emulator + API |
| D16 | Persetujuan dari personel berbeda | Daftar memuat rencana `85000001` dan `85210022`; keduanya memiliki lampiran PDF+PNG | ✅ | emulator + API |
| D17 | Tekan **Lihat** pada lampiran PDF/gambar | PDF tampil per halaman di dalam aplikasi; gambar tampil penuh dan dapat di-zoom | ✅ | emulator |
| D18 | Kartu **Verifikasi Hasil Kerja** di Beranda | Menampilkan jumlah hasil `diajukan` dari bawahan langsung; **Tinjau Verifikasi** membuka layar di tab Hasil | ✅ | emulator |
| D19 | Kartu **Verifikasi Output** di Beranda | Menampilkan jumlah output `dalam_verifikasi`; **Tinjau Verifikasi Output** membuka layar di tab Output | ✅ | emulator |
| D20 | Verifikasi output: tolak tanpa catatan, lalu setujui | Tanpa catatan ditolak dengan pesan validasi; dengan skor 90/92 status menjadi `disetujui` dan nilai tampil | ✅ | emulator + API |
| D21 | Geser slider skor kesesuaian & keabsahan | Dua slider 0–100 tampil; nilai berubah (mis. 54 dan 82) sebelum disimpan | ✅ | emulator |
| D22 | Verifikasi hasil kerja | Slider skor kualitas 0–100 tampil dengan default 85 dan dapat digeser | ✅ | emulator |

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
| F1 | `cargo test` | 40 unit test lulus | ✅ | |
| F2 | `cargo fmt --check` + clippy `-D warnings` | Bersih | ✅ | |
| F3 | `bun run check` | 0 error, 0 warning (dashboard & personel) | ✅ | |
| F4 | `bun run build` | SPA statis terbentuk di `dashboard/build` dan `personel/build` | ✅ | |
| F5 | `.gradlew :app:assembleDebug` | APK debug terbentuk | ✅ | |
| F6 | `bun tools/uji/api-smoke.js` | 89/89 cek lulus (DB `kinerja_test`) | ✅ | |
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

## I. Aplikasi web personel & atasan (`personel/`)

| # | Skenario | Hasil yang diharapkan | Status | Catatan |
|---|---|---|---|---|
| W1 | Login `85000001` / `000100` di `localhost:5174` | Berhasil masuk ke Beranda; tombol mata pada password berfungsi | ✅ | API + Chrome headless (sesi uji) |
| W2 | Login akun admin (`superuser`) di web personel | Ditolak 403 dengan pesan kanal login | ✅ | API |
| W3 | Buka aplikasi pertama kali setelah login | Wizard izin browser tampil (lokasi, kamera, notifikasi, antrean), dapat dilewati | ✅ | build |
| W4 | Beranda | Salam + avatar, status masuk/pulang hari ini, kartu strong point, antrean offline, bottom navigation **Beranda/Absensi/Strong** | ✅ | Chrome headless |
| W5 | Absensi → ambil lokasi → pilih status → kirim | Peta OSM, koordinat/akurasi/jarak/titik dinas tampil; Dinas Kantor/Piket ditolak di luar radius; foto opsional | ⬜ | perlu GPS perangkat |
| W6 | Absensi saat server tidak terjangkau | Kedua tombol Masuk/Pulang tampil; data dapat disimpan offline dan dikirim lewat antrean | ✅ | build + API antrean |
| W7 | Strong Point tanpa jadwal hari ini | Pesan “tidak ada jadwal” tampil; wizard tidak mengirim data | ✅ | Chrome headless |
| W8 | Strong Point: sesi sudah check-out | Tombol check-in tidak tampil untuk jadwal itu; sesi lain (sore) dapat dipilih dari daftar; check-out tanpa dokumentasi | ✅ | build (logika) |
| W9 | Wizard Rencana 4 langkah | Jenis → dasar SP/TOR/lisan → rincian kegiatan (multi) → lampiran; simpan draft, ajukan, dan ubah/perbaiki | ✅ | Chrome headless (tampilan) + endpoint Android |
| W10 | Wizard Hasil 3 langkah & Output 3 langkah | Pilih rencana disetujui/hasil terverifikasi, isi data, unggah bukti, simpan draft/ajukan | ✅ | build + endpoint Android |
| W11 | Atasan membuka Persetujuan Rencana | Dua rencana tampil; detail + lampiran; Setujui/Minta Revisi/Tolak dengan catatan wajib untuk non-setuju | ✅ | Chrome headless (daftar) + endpoint Android |
| W12 | Atasan membuka Verifikasi Hasil/Output | Slider skor tampil; keputusan valid/revisi tersimpan | ✅ | build + endpoint Android |
| W13 | Profil: unggah foto, edit profil, ganti password | Avatar berubah; pangkat dari 22 tingkat Polri + 17 pangkat ASN; tiga kolom password punya tombol mata; sukses ganti password kembali ke login | ✅ | build + endpoint Android |
| W14 | Tampilan pada lebar 500 px | Tidak ada elemen terpotong; bottom navigation tetap terlihat; kartu responsif | ✅ | Chrome headless |
