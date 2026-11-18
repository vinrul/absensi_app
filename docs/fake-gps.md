# Deteksi Lokasi Tiruan (Fake GPS)

Dokumen ini menjelaskan cara aplikasi mendeteksi lokasi tiruan, kebijakan server
`flag`/`reject`, cara mengujinya saat development, serta batasannya.

## 1. Ringkasan

| Lapisan | Yang diperiksa | Perilaku |
|---|---|---|
| Android | `Location.isMock` (Android 12+) / `isFromMockProvider` | Mengirim `is_mock_suspect=true` dan menampilkan peringatan |
| Android | Developer Options aktif | Peringatan informatif; **tidak** menolak |
| Android | Aplikasi mock yang dipilih di Developer Options | Peringatan informatif; **tidak** menolak |
| Server | `APP__MOCK_LOCATION_POLICY` | `flag` = simpan + `perlu_verifikasi`; `reject` = HTTP 422 |

Keputusan akhir selalu di **server**, bukan di aplikasi. Aplikasi hanya melaporkan
kecurigaan; klien yang dimodifikasi tetap dapat mengirim `is_mock_suspect=false`.

## 2. Konfigurasi server

```env
# development (default)
APP__MOCK_LOCATION_POLICY=flag

# produksi
APP__MOCK_LOCATION_POLICY=reject
```

Nilai yang diterima: `flag`/`tandai` dan `reject`/`tolak`.

### `flag`

- Absensi dan strong point tetap disimpan.
- `perlu_verifikasi` diset `true`.
- Data masuk daftar verifikasi atasan/admin.
- Dipakai agar development dengan emulator/Fake GPS tetap bisa menguji alur.

### `reject`

- Permintaan dengan `is_mock_suspect=true` langsung ditolak **HTTP 422**.
- Pesan: “Lokasi tiruan (mock GPS) terdeteksi. Matikan aplikasi lokasi tiruan/Developer Options lalu coba lagi.”
- Tidak ada baris absensi/strong point yang dibuat dan tidak ada foto yang disimpan.

Berlaku untuk:

- `POST /api/v1/absensi`
- `POST /api/v1/ph/absensi`
- pengiriman ulang antrean offline (endpoint yang sama).

## 3. Deteksi di Android

Kode utama: `android/app/src/main/java/id/polri/poldabali/kinerja/lokasi/Lokasi.kt`.

```kotlin
fun adalahMock(lokasi: Location): Boolean =
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        lokasi.isMock
    } else {
        lokasi.isFromMockProvider
    }
```

Tambahan pemeriksaan untuk development/verifikasi:

- `developerOptionsAktif(context)` membaca `Settings.Global.DEVELOPMENT_SETTINGS_ENABLED`.
- `aplikasiMockTerpilih(context)` membaca setting `mock_location` (best-effort; tidak
  semua OEM menyediakannya).

Keduanya hanya menampilkan peringatan di layar absensi/strong point. Developer Options
yang aktif **tidak otomatis** dianggap fake GPS, karena banyak teknisi menyalakannya
untuk keperluan lain.

Lokasi diambil lewat **FusedLocationProvider** dengan prioritas akurasi tinggi. Bila
tidak ada posisi baru dalam 20 detik, aplikasi memakai `lastLocation`; ini bukan fake
GPS, tetapi bisa menghasilkan lokasi lama.

## 4. Antrean offline saat ditolak

- Absensi offline yang ditolak karena mock GPS **dihapus dari antrean** dan dihitung
  sebagai `ditolak`; aplikasi menampilkan “X ditolak karena lokasi tiruan”.
- Absensi offline yang gagal karena sebab lain (mis. di luar radius) tetap disimpan
  agar dapat diperiksa ulang.
- Strong point mengikuti aturan yang sama; kegagalan 4xx lain dibuang seperti sebelumnya.
- Web personel memakai aturan yang sama di `personel/src/lib/offline.ts`.

## 5. Menguji saat development

### 5.1 Menyiapkan Lokasi Tiruan di emulator

```bash
adb shell settings put global development_settings_enabled 1
adb shell appops set com.example.fakegps android:mock_location allow
adb shell appops get com.example.fakegps android:mock_location
```

1. Pasang aplikasi Fake GPS di emulator.
2. Buka **Developer Options → Select mock location app**, pilih aplikasi tersebut.
3. Jalankan Fake GPS di dalam radius titik dinas.
4. Buka tab **Absensi** Android.

Hasil yang diharapkan:

| Policy server | Hasil API | Tampilan aplikasi |
|---|---|---|
| `flag` | 200, `perlu_verifikasi=true` | Peringatan merah “Terdeteksi lokasi tiruan” |
| `reject` | 422 | Peringatan merah + pesan penolakan |

### 5.2 GPS asli/simulasi emulator (bukan mock)

```bash
adb emu geo fix 115.22144 -8.65629
```

Gunakan ini untuk menguji jalur normal. `adb emu geo fix` tidak dianggap mock provider,
sehingga cocok untuk memastikan absensi tetap diterima saat policy `reject`.

### 5.3 Uji endpoint tanpa aplikasi

Dengan token personel:

```bash
curl -X POST http://127.0.0.1:8080/api/v1/absensi \
  -H "authorization: Bearer <token>" \
  -H "content-type: application/json" \
  -d '{
    "client_uuid": "<uuid>",
    "jenis": "masuk",
    "status_kehadiran": "dinas_luar",
    "captured_at": "2026-11-18T08:00:00Z",
    "latitude": -8.65629,
    "longitude": 115.22144,
    "akurasi_m": 5,
    "is_mock_suspect": true
  }'
```

- Server `flag` → 200 dan `perlu_verifikasi=true`.
- Server `reject` → 422 dengan pesan lokasi tiruan.

### 5.4 Uji antrean offline

1. Set server `reject`.
2. Aktifkan Fake GPS, matikan jaringan, lalu simpan absensi offline.
3. Nyalakan jaringan → kirim antrean.
4. Pastikan item hilang dari antrean, jumlah “ditolak” muncul, dan tidak ada baris
   absensi baru di server.

### 5.5 Uji Developer Options

- Nyalakan Developer Options **tanpa** memilih aplikasi mock:
  - `flag` → absensi GPS asli tetap 200.
  - `reject` → absensi GPS asli tetap 200, hanya ada peringatan informasi.
- Pilih aplikasi mock tetapi jangan jalankan:
  - aplikasi menampilkan peringatan “aplikasi lokasi tiruan terpilih”;
  - absensi dengan GPS asli tetap boleh dikirim.

## 6. Batasan

- `isMock` hanya mendeteksi mock provider standar. Perangkat **root**, custom ROM,
  LSPosed/Xposed, atau spoofing level sistem dapat memalsukan lokasi tanpa terdeteksi.
- Developer Options aktif bukan bukti fake GPS; karena itu tidak dijadikan penolakan.
- `lastLocation` bisa berupa posisi lama saat GPS belum mendapat fix.
- **Web personel tidak dapat mendeteksi fake GPS**; browser tidak menyediakan flag
  tersebut. Gunakan foto dokumentasi + verifikasi atasan.
- Play Integrity **belum diterapkan**. Attestation hanya membuktikan APK/perangkat,
  bukan keaslian koordinat. Rencana penguatan ada di backlog.
- Validasi plausibilitas server (lompatan koordinat/kecepatan) belum diterapkan.

## 7. Development vs produksi

| Lingkungan | Policy | Alasan |
|---|---|---|
| Development/emulator | `flag` | Alur absensi tetap bisa diuji dengan Fake GPS |
| Staging | `flag` atau `reject` | Uji pesan penolakan dan antrean |
| Produksi | `reject` | Cegah absensi dengan lokasi tiruan |

Jangan memblokir berdasarkan `BuildConfig.DEBUG`, `device_id`, atau nama perangkat di
server — nilai tersebut dapat dipalsukan. Gunakan environment server dan/atau akun uji
yang eksplisit.

## 8. Matriks uji

| Skenario | Policy `flag` | Policy `reject` |
|---|---|---|
| GPS asli dalam radius | 200 | 200 |
| Fake GPS aktif | 200 + perlu verifikasi | 422 |
| Fake GPS + antrean offline | Tersimpan, lalu diverifikasi | Ditolak saat sinkronisasi |
| Developer Options aktif, GPS asli | 200 | 200 |
| Aplikasi mock dipilih, tidak aktif | 200 + peringatan | 200 + peringatan |
| Akurasi > 100 m | 200 + perlu verifikasi | 200 + perlu verifikasi |
| Di luar radius | 422 | 422 |

## 9. Berkas terkait

- `android/.../lokasi/Lokasi.kt` — deteksi mock, Developer Options, aplikasi mock.
- `android/.../ui/AbsensiScreen.kt` — peringatan dan pengiriman `is_mock_suspect`.
- `android/.../ui/StrongPointWizard.kt` — peringatan strong point.
- `android/.../data/Repositori.kt` — antrean dan hitungan ditolak.
- `backend/src/config.rs` — `MockLocationPolicy`.
- `backend/src/routes/absensi.rs` — penolakan absensi mock.
- `backend/src/routes/ph.rs` — penolakan strong point mock.
- `personel/src/lib/offline.ts` — antrean web personel.
