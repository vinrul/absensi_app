# Publikasi SIAP Bali ke Google Play Store

Panduan ini merangkum kesiapan teknis, aset yang sudah dibuat, langkah di Play Console,
serta keputusan yang harus diambil sebelum unggahan pertama.

> Status terakhir: 25 November 2026. AAB release bertanda tangan sudah dibuat; aset store
> listing, kebijakan privasi, dan akun peninjau produksi sudah disiapkan.

---

## 1. Identitas aplikasi

| Item | Nilai |
|---|---|
| Nama tampilan | **SIAP Bali** |
| Application ID | `id.polri.poldabali.kinerja` |
| Version code / name | `1` / `1.0.0` |
| minSdk / targetSdk | 26 / 36 |
| Kanal login | Android (`client=android`) |
| Server produksi | `https://absensi.jagradewata.id/api/v1/` |
| AAB siap unggah | `android/app/build/outputs/bundle/release/app-release.aab` (±10 MB) |
| APK uji release | `android/app/build/outputs/apk/release/app-release.apk` |
| Sertifikat upload | `E:/new_disposisi/siopa.jks` (lewat `android/keystore.properties`) |

Build ulang kapan saja:

```powershell
cd F:\absensi_polda_bali\android
.\build-release.bat
```

AAB sudah diverifikasi: `application-label='SIAP Bali'`, `versionCode=1`, `targetSdk=36`,
dan ditandatangani dengan sertifikat upload.

---

## 2. Keputusan penting sebelum unggah

### 2.1 Nama paket `id.polri.poldabali.kinerja`

Nama paket **permanen** setelah aplikasi pertama kali diunggah ke Play Console. Karena memakai
domain `polri`, Google Play dapat meminta bukti kewenangan (kebijakan impersonation/aplikasi
pemerintah). Pilih salah satu:

- **A. Aplikasi resmi Polda Bali** — siapkan surat tugas/keterangan dari Polda Bali yang
  menyatakan aplikasi dan logo digunakan untuk absensi personel. Gunakan akun developer
  organisasi bila tersedia. Cantumkan dokumen saat Play Console meminta.
- **B. Belum ada surat resmi** — ganti `applicationId` **sebelum unggahan pertama** ke domain
  yang Anda kendalikan, misalnya `id.jagradewata.siapbali`. Ubah di
  `android/app/build.gradle.kts` (`applicationId`), lalu build ulang AAB. `namespace` internal
  boleh tetap. Setelah diunggah, nama paket tidak dapat diubah.

Jangan mengunggah AAB sebelum keputusan ini final.

### 2.2 Tipe akun developer

- **Akun personal baru** — Google mewajibkan **closed testing** dengan minimal **12 tester
  selama 14 hari** sebelum dapat mengajukan produksi (kebijakan dapat berubah; ikuti petunjuk
  di Play Console → *Testing requirements*).
- **Akun organisasi** — umumnya tidak terkena kewajiban tersebut, tetapi perlu verifikasi
  organisasi.

### 2.3 Akun peninjau (sudah dibuat di produksi)

Reviewer Google Play harus dapat masuk ke aplikasi. Akun uji khusus sudah dibuat di server
produksi:

```
NRP     : 99999999
Password: Tinjau2026
Nama    : Peninjau Google Play
Satker  : DITRESKRIMSUS
```

Akun ini **hanya untuk peninjauan**. Nonaktifkan atau ganti password secara berkala setelah
aplikasi disetujui. Jangan memakai akun personel nyata.

### 2.4 Kebijakan privasi

Halaman sudah dibuat:

- `dashboard/static/privacy-policy.html` → `https://absensi.jagradewata.id/privacy-policy.html`
- `dashboard/static/hapus-data.html` → `https://absensi.jagradewata.id/hapus-data.html`

Keduanya ikut masuk ke `dashboard/build/` saat `bun run build`. **Belum ter-deploy**; saat ini
URL tersebut masih mengembalikan fallback SPA. Deploy dashboard ke VPS lalu pastikan halaman
benar-benar tampil (lihat §8).

---

## 3. Aset store listing (siap pakai)

Semua ada di `store/play/`:

| Berkas | Ukuran | Kegunaan |
|---|---|---|
| `icon-512.png` | 512×512 | Ikon aplikasi Play Store |
| `feature-graphic-1024x500.png` | 1024×500 | Feature graphic |
| `screenshot-1-beranda.png` | 1840×2208 | Screenshot Beranda |
| `screenshot-2-absensi.png` | 1840×2208 | Screenshot menu Absensi |
| `screenshot-3-validasi-gps.png` | 1840×2208 | Screenshot validasi GPS |
| `screenshot-4-riwayat.png` | 1840×2208 | Screenshot riwayat absensi |
| `icon.html`, `feature-graphic.html` | — | Sumber untuk membuat ulang gambar |

Persyaratan Play: ikon 512×512 PNG 32-bit, feature graphic 1024×500 PNG/JPEG, screenshot
minimal 2 (disarankan 4–8) dengan sisi 320–3840 px. Semua aset di atas memenuhi.

---

## 4. Copy-paste store listing

### Nama aplikasi (maks. 30 karakter)

```
SIAP Bali
```

### Deskripsi singkat (maks. 80 karakter)

```
Absensi GPS, strong point, dan rekap kehadiran personel Polda Bali.
```

### Deskripsi lengkap (maks. 4.000 karakter)

```
SIAP Bali (Sistem Informasi Absensi Personel Bali) adalah aplikasi absensi dan strong point untuk personel Polda Bali dan Polres jajaran. Aplikasi membantu pencatatan kehadiran berbasis lokasi, pelaksanaan strong point, serta pemantauan rekap kehadiran secara tertib dan transparan.

Fitur utama:
• Absensi berbasis GPS — check-in dan check-out dengan validasi lokasi dan radius titik dinas.
• Strong Point — jadwal pagi/sore, check-in dan check-out per sesi, serta pemantauan realisasi.
• Riwayat kehadiran — melihat absensi dan sesi strong point per tanggal.
• Foto dokumentasi — lampiran foto absensi/strong point (opsional).
• Bekerja offline — absensi tersimpan di perangkat lalu dikirim otomatis saat jaringan tersedia.
• Keamanan — sesi login aman dan data hanya dapat diakses personel serta pengelola yang berwenang.

Aplikasi ini diperuntukkan bagi personel Polda Bali yang telah memiliki akun dari admin satker. Akun tidak dapat didaftarkan sendiri melalui aplikasi.

Catatan: beberapa fitur memerlukan izin lokasi, kamera, dan notifikasi. Izin lokasi hanya digunakan saat proses absensi/strong point, bukan untuk pelacakan terus-menerus.

Kebijakan privasi: https://absensi.jagradewata.id/privacy-policy.html
```

### Catatan rilis pertama (maks. 500 karakter)

```
Rilis perdana SIAP Bali: absensi GPS, strong point, riwayat kehadiran, mode offline, dan pengingat jadwal strong point.
```

### Kategori & tag

- Kategori: **Business**
- Tag yang disarankan: *Attendance*, *Workforce management*, *Business operations*
- Email kontak: `vinrul.id@gmail.com`
- Website: `https://absensi.jagradewata.id`
- Kebijakan privasi: `https://absensi.jagradewata.id/privacy-policy.html`

---

## 5. Langkah di Play Console

1. **Create app**
   - App name: `SIAP Bali`
   - Default language: Indonesian (Indonesia)
   - App or game: **App**
   - Free or paid: **Free**
   - Setujui declaration yang diminta.

2. **Dashboard → Store listing**
   - Isi nama, deskripsi singkat, deskripsi lengkap dari §4.
   - Unggah ikon, feature graphic, dan 4 screenshot.
   - Pilih kategori **Business**, isi email/website.

3. **App content**
   - **Privacy policy**: isi URL `https://absensi.jagradewata.id/privacy-policy.html`
     setelah halaman ter-deploy dan terverifikasi.
   - **App access**: pilih *All functionality is available with login* lalu isi akun peninjau
     dan langkah akses (lihat §7).
   - **Ads**: pilih **No ads**.
   - **Content rating**: isi kuesioner (lihat §6).
   - **Target audience**: pilih **18+**; aplikasi bukan untuk anak.
   - **News app**: No.
   - **Data safety**: isi sesuai §6.2.
   - **Government apps**: jawab sesuai fakta; unggah surat kewenangan bila ada.
   - **Financial features / Health**: No.
   - **App category**: Business.

4. **Testing → Internal testing**
   - Buat release, unggah `app-release.aab`.
   - Tambahkan email Anda sebagai tester.
   - Bagikan link opt-in, pasang dari Play Store, lalu uji login + absensi + riwayat.

5. **Closed testing (bila akun personal baru)**
   - Tambahkan minimal 12 tester, jalankan 14 hari, kumpulkan umpan balik.
   - Jika akun organisasi/terverifikasi, tahap ini bisa dilewati sesuai petunjuk Play.

6. **Production**
   - Promote release dari testing, pilih negara **Indonesia**, isi catatan rilis.
   - Kirim untuk review. Waktu review biasanya beberapa hari; pantau email Play Console.

---

## 6. Formulir Play Console

### 6.1 App access (salin-tempel)

```
Aplikasi memerlukan akun personel yang dibuat oleh admin satker. Tidak ada pendaftaran mandiri.

Akun uji:
NRP: 99999999
Password: Tinjau2026

Langkah:
1. Buka aplikasi dan selesaikan/lewati wizard izin (lokasi, kamera, notifikasi).
2. Masukkan NRP dan password di atas, lalu tekan Masuk.
3. Beranda menampilkan kehadiran hari ini dan jadwal strong point.
4. Buka tab Absensi → Mulai Absensi → pilih Dinas Luar (tidak wajib berada dalam radius) → Perbarui Lokasi → kirim.
5. Buka Riwayat Absensi untuk melihat daftar kehadiran bulan berjalan.

Catatan: akun ini hanya untuk peninjauan. Data absensi yang dikirim akan tercatat pada akun uji.
```

### 6.2 Data safety

| Pertanyaan / jenis data | Jawaban |
|---|---|
| Apakah aplikasi mengumpulkan atau membagikan data pengguna? | **Ya** |
| Nama | Dikumpulkan; tidak dibagikan; fungsi aplikasi & manajemen akun; wajib (dari admin) |
| ID pengguna (NRP) | Dikumpulkan; tidak dibagikan; fungsi aplikasi & manajemen akun; wajib |
| Surel | Dikumpulkan bila diisi; tidak dibagikan; profil; opsional |
| Nomor telepon | Dikumpulkan bila diisi; tidak dibagikan; profil; opsional |
| Foto | Dikumpulkan; tidak dibagikan; bukti absensi/strong point & foto profil; opsional |
| Lokasi perkiraan | Dikumpulkan; tidak dibagikan; fungsi absensi; wajib untuk absensi |
| Lokasi tepat | Dikumpulkan; tidak dibagikan; fungsi absensi; wajib untuk absensi |
| Aktivitas dalam aplikasi | Dikumpulkan; tidak dibagikan; fungsi absensi, strong point, rekap; wajib |
| ID perangkat atau lainnya | Dikumpulkan; tidak dibagikan; keamanan sesi/login; wajib |
| Log crash / diagnostik | Tidak dikumpulkan |
| ID iklan | Tidak dikumpulkan |
| Data dienkripsi saat transit | **Ya** (HTTPS/TLS) |
| Pengguna dapat meminta penghapusan data | **Ya** — `https://absensi.jagradewata.id/hapus-data.html` |
| Akun dibuat di aplikasi? | Tidak; akun dibuat admin satker |

Catatan: penyedia tile peta (OpenStreetMap/Esri) menerima permintaan teknis untuk menampilkan
peta. Hal ini dijelaskan di kebijakan privasi. Jika tim Play meminta klarifikasi, jelaskan bahwa
penyedia peta hanya memproses permintaan teknis (service provider), bukan menerima data lokasi
terstruktur.

### 6.3 Content rating (IARC)

- Kategori kuesioner: **Utility, Productivity, Communication, or Other**
- Kekerasan, seksualitas, bahasa kasar, narkoba, perjudian: **Tidak**
- Konten buatan pengguna yang dibagikan publik: **Tidak**
- Membagikan lokasi ke pengguna lain: **Ya, terbatas ke atasan/admin internal** (jelaskan bila
  kuesioner menanyakan; bukan lokasi publik)
- Pembelian dalam aplikasi: **Tidak**
- Iklan: **Tidak**
- Akses web tanpa batas: **Tidak**
- Perkiraan rating: **Everyone / 3+**

---

## 7. Checklist sebelum menekan Submit

- [ ] Keputusan nama paket final (`id.polri...` atau ganti ke domain sendiri).
- [ ] Surat kewenangan Polda Bali siap (bila memakai `id.polri`).
- [ ] Kebijakan privasi & halaman hapus data sudah ter-deploy dan **isinya benar** (bukan
      fallback SPA).
- [ ] Akun peninjau `99999999 / Tinjau2026` aktif dan bisa login di APK release.
- [ ] AAB `app-release.aab` diunggah (bukan APK) dan ditandatangani.
- [ ] `versionCode` dinaikkan untuk setiap unggahan berikutnya.
- [ ] Keystore `E:/new_disposisi/siopa.jks` dan `keystore.properties` di-backup aman.
- [ ] Ikon, feature graphic, dan minimal 2 screenshot terunggah.
- [ ] Data safety, content rating, ads, target audience, app access terisi.
- [ ] Internal testing lolos (login, absensi Dinas Luar, riwayat).
- [ ] Catatan rilis pertama diisi.

---

## 8. Deploy kebijakan privasi ke VPS

Dari mesin pengembang:

```bash
cd dashboard
bun run build
rsync -az --delete build/ user@vps:/tmp/dashboard-build/
```

Di VPS:

```bash
sudo rsync -a --delete /tmp/dashboard-build/ /var/www/absensi/
sudo chown -R www-data:www-data /var/www/absensi
```

Verifikasi (harus menampilkan judul halaman, bukan halaman login SPA):

```bash
curl -s https://absensi.jagradewata.id/privacy-policy.html | grep -q "Kebijakan Privasi SIAP Bali" && echo "privacy OK"
curl -s https://absensi.jagradewata.id/hapus-data.html | grep -q "Permintaan Penghapusan Data" && echo "hapus-data OK"
```

Jika `curl` masih mengembalikan HTML SPA (bukan teks kebijakan), berarti berkas belum tersalin.

---

## 9. Setelah tayang

- Pantau **Play Console → Quality → Android vitals** untuk crash/ANR.
- Naikkan `versionCode` dan `versionName` di `android/app/build.gradle.kts` untuk setiap rilis.
- Jangan pernah menghapus/kehilangan keystore upload; jika hilang, ajukan reset upload key
  melalui Play Console.
- Perbarui kebijakan privasi bila data yang dikumpulkan berubah.
- Ganti password akun peninjau secara berkala; nonaktifkan bila tidak lagi diperlukan.
- Simpan salinan AAB dan mapping ProGuard (bila minify diaktifkan) untuk setiap rilis.

---

## 10. Berkas terkait

- `android/app/build.gradle.kts` — applicationId, version, signing
- `android/build-release.bat` — build APK + AAB release
- `android/keystore.properties` — kredensial keystore (jangan di-commit)
- `dashboard/static/privacy-policy.html` — kebijakan privasi
- `dashboard/static/hapus-data.html` — permintaan penghapusan data
- `store/play/` — ikon, feature graphic, screenshot, sumber HTML
- `docs/deploy-vps.md` — deployment server
