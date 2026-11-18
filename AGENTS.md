# Panduan untuk agent AI

Berkas ini adalah pintu masuk singkat. **Baca `docs/HANDOFF.md` sebelum mengubah kode** —
di sana ada status, keputusan teknis, jebakan, dan backlog berprioritas.

## Aturan wajib

1. **Dashboard dan web personel memakai Bun.** `cd dashboard && bun install && bun run dev|check|build`;
   `cd personel && bun install && bun run dev|check|build`. Jangan membuat `package-lock.json`;
   lockfile resmi adalah `bun.lock`.
2. **Jangan mengubah versi inti tanpa memeriksa kompatibilitas tiga arah**:
   AGP ↔ Gradle ↔ compileSdk (lihat HANDOFF §3.7), dan SvelteKit 3 ↔ Vite 8 ↔ Tailwind 4.
3. **SQL harus literal `'static`** (sqlx 0.9 `SqlSafeStr`). Filter opsional memakai
   `($n::tipe IS NULL OR kolom = $n)`.
4. **Semua mutasi lewat `/api/v1`** otomatis diaudit middleware; jangan menambah penulisan
   `audit_log` manual kecuali butuh `before/after`.
5. **Rumus penilaian** hanya di `backend/src/service/scoring.rs`. Setiap perubahan wajib disertai
   unit test dan pembaruan `docs/penilaian.md`.
6. **Tambahkan endpoint baru** ke `router.rs`, `openapi.json`, dan bila relevan `docs/PRD.md`.
7. **Android**: jangan menambah Hilt/Room/KSP tanpa memigrasikan `Graf`/`AntreanAbsensi`; alasan
   ada di HANDOFF §3.9.
8. **Web personel** memakai kanal login `personel_web` dan cookie refresh httpOnly dengan
   `device_id = "web-personel"`. Jangan menaruh access/refresh token di `localStorage`.
9. **Fake GPS** diputuskan server lewat `APP__MOCK_LOCATION_POLICY` (`flag` untuk development,
   `reject` untuk produksi). Klien hanya mengirim `is_mock_suspect` dan peringatan Developer
   Options; jangan hardcode penolakan di aplikasi. Panduan: `docs/fake-gps.md`.
10. **Rahasia**: jangan commit `backend/.env`, `deploy/.env`, `deploy/data/`, atau
    `android/keystore.properties` (beserta `*.jks`/`*.keystore`).

## Perintah verifikasi cepat

```powershell
cd backend;  cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test
cd dashboard; bun run check; bun run build
cd personel;  bun run check; bun run build
cd android;  .\gradlew.bat :app:assembleDebug
# release bertanda tangan (butuh android\keystore.properties):
cd android;  .\build-release.bat
```

## Deploy VPS

Panduan lengkap: `docs/deploy-vps.md`. Seed demo server (hanya Ditreskrimsus) memakai
`cargo run --bin seed -- --reset --vps`; **jangan** menjalankan `--reset` pada database
produksi yang sudah berisi data. Profil tanpa `--vps` tetap dipakai untuk development.

## Bahasa

Kode, komentar, pesan error API, dan dokumentasi memakai **Bahasa Indonesia** agar konsisten
dengan pengguna akhir (personel Polri). Nama variabel/tipe tetap bahasa Inggris atau Indonesia
sesuai konvensi yang sudah ada di masing-masing file — ikuti file sekitarnya.
