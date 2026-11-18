# Deploy VPS — `absensi.jagradewata.id` & `personel.jagradewata.id`

Panduan ini memasang aplikasi **tanpa Docker** di satu VPS:

- **API Rust** berjalan sebagai layanan systemd di `127.0.0.1:7085` (tidak terbuka ke internet).
- **Dashboard** (SPA) disajikan nginx dari `/var/www/absensi` pada `https://absensi.jagradewata.id`.
- **Web personel** (SPA) disajikan nginx dari `/var/www/personel` pada `https://personel.jagradewata.id`.
- Kedua domain memakai **mTLS Cloudflare** (`ssl_verify_client on`), sehingga origin hanya menerima
  koneksi yang membawa sertifikat klien Cloudflare.
- **PostgreSQL** berjalan lokal di VPS dan hanya mendengarkan `127.0.0.1`.

```
pengguna ──HTTPS──▶ Cloudflare ──mTLS──▶ nginx :443 ──/api,/files──▶ 127.0.0.1:7085 (API)
                                        └──/_app, halaman──▶ /var/www/{absensi,personel}
                                                                 │
                                                          PostgreSQL 127.0.0.1:5432
```

> Docker Compose di `deploy/` tetap dipakai untuk development lokal. Dokumen ini khusus VPS
> dengan nginx, systemd, dan sertifikat yang sudah disediakan.

---

## 1. Prasyarat

Contoh untuk Ubuntu/Debian:

```bash
sudo apt update
sudo apt install -y postgresql postgresql-client nginx rsync curl ca-certificates
```

Tambahan untuk **ekspor PDF** (dipakai tombol Export PDF di dashboard):

```bash
# Opsi A — Google Chrome stabil (disarankan; sandbox berjalan normal)
wget -O /tmp/google-chrome.deb https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb
sudo apt install -y /tmp/google-chrome.deb

# Opsi B — Chromium dari distro
sudo apt install -y chromium
```

Sertifikat yang sudah disiapkan pengguna:

| Berkas | Isi |
|---|---|
| `/etc/ssl/jagradewata_id_cert.pem` | Sertifikat origin `*.jagradewata.id` |
| `/etc/ssl/jagradewata_id_key.pem` | Kunci privat sertifikat origin |
| `/etc/ssl/cloudflare.crt` | CA sertifikat klien Cloudflare (Authenticated Origin Pulls) |

Di Cloudflare aktifkan **Authenticated Origin Pulls** untuk zona `jagradewata.id`, dan gunakan
mode SSL **Full (strict)**. Tanpa sertifikat klien Cloudflare, nginx akan menolak koneksi
(HTTP 400/495) — ini memang tujuannya.

Pastikan juga DNS `absensi` dan `personel` sudah **proxied** (awan oranye) ke IP VPS.

---

## 2. Struktur direktori

```bash
sudo useradd --system --create-home --home-dir /opt/absensi \
     --shell /usr/sbin/nologin absensi

sudo mkdir -p /opt/absensi \
              /etc/absensi \
              /var/lib/absensi/uploads \
              /var/www/absensi \
              /var/www/personel

sudo chown -R absensi:absensi /opt/absensi /var/lib/absensi
sudo chmod 750 /var/lib/absensi /var/lib/absensi/uploads
```

| Path | Fungsi |
|---|---|
| `/opt/absensi/kinerja-api` | Biner API |
| `/opt/absensi/seed` | Biner seed demo |
| `/etc/absensi/absensi.env` | Konfigurasi rahasia (root:absensi, 640) |
| `/var/lib/absensi/uploads` | Foto absensi/strong point, lampiran, bukti output |
| `/var/www/absensi` | Hasil `dashboard/build` |
| `/var/www/personel` | Hasil `personel/build` |

---

## 3. Build artefak

Build boleh dilakukan di mesin pengembang, lalu biner/isi build disalin ke VPS. Untuk VPS
berarsitektur Linux, build harus dilakukan di Linux (atau container Linux) agar biner kompatibel.

### 3.1 Backend

```bash
cd backend
cargo build --release --bin kinerja-api --bin seed
ls -lh target/release/kinerja-api target/release/seed
```

Salin ke VPS:

```bash
scp backend/target/release/kinerja-api backend/target/release/seed \
    user@vps:/tmp/
sudo install -m 755 -o absensi -g absensi /tmp/kinerja-api /tmp/seed /opt/absensi/
```

### 3.2 Dashboard

```bash
cd dashboard
bun install --frozen-lockfile
bun run check
bun run build
```

### 3.3 Web personel

```bash
cd personel
bun install --frozen-lockfile
bun run check
bun run build
```

Salin hasil build (dari mesin pengembang):

```bash
rsync -az --delete dashboard/build/ user@vps:/tmp/dashboard-build/
rsync -az --delete personel/build/  user@vps:/tmp/personel-build/

# di VPS
sudo rsync -a --delete /tmp/dashboard-build/ /var/www/absensi/
sudo rsync -a --delete /tmp/personel-build/  /var/www/personel/
sudo chown -R www-data:www-data /var/www/absensi /var/www/personel
sudo find /var/www/absensi /var/www/personel -type d -exec chmod 755 {} +
sudo find /var/www/absensi /var/www/personel -type f -exec chmod 644 {} +
```

---

## 4. Database PostgreSQL

```bash
sudo -u postgres psql
```

```sql
CREATE ROLE absensi LOGIN PASSWORD 'GANTI_PASSWORD_KUAT';
CREATE DATABASE absensi OWNER absensi;
\q
```

Pastikan PostgreSQL hanya mendengarkan localhost (bawaan Ubuntu):

```bash
sudo grep -E '^\s*listen_addresses' /etc/postgresql/*/main/postgresql.conf
# listen_addresses = 'localhost'
```

Skema dibuat otomatis oleh seed/migrasi. Tidak perlu menjalankan `sqlx migrate` manual.

---

## 5. Konfigurasi API

Buat `/etc/absensi/absensi.env`:

```bash
sudo tee /etc/absensi/absensi.env >/dev/null <<'EOF'
APP__DATABASE_URL=postgres://absensi:GANTI_PASSWORD_KUAT@127.0.0.1:5432/absensi
APP__HOST=127.0.0.1
APP__PORT=7085
APP__JWT_SECRET=GANTI_DENGAN_openssl_rand_hex_32
APP__ACCESS_TOKEN_TTL_MINUTES=15
APP__REFRESH_TOKEN_TTL_DAYS=30
APP__UPLOAD_DIR=/var/lib/absensi/uploads
APP__RUN_MIGRATIONS=true
APP__COOKIE_SECURE=true
APP__ALLOW_FAKE_TIME=false
APP__MOCK_LOCATION_POLICY=reject
APP__CORS_ORIGINS=https://absensi.jagradewata.id,https://personel.jagradewata.id
APP__CHROME_PATH=/usr/bin/google-chrome
EOF

sudo chown root:absensi /etc/absensi/absensi.env
sudo chmod 640 /etc/absensi/absensi.env
```

Membuat JWT secret:

```bash
openssl rand -hex 32
```

> Gunakan password database yang aman untuk URL (mis. `openssl rand -hex 24`). Bila password
> mengandung karakter khusus, URL-encode atau pilih karakter `A-Z a-z 0-9 - _` agar dapat
> dibaca systemd dan `pg_dump` tanpa escaping tambahan.

Catatan:

- `APP__COOKIE_SECURE=true` wajib karena seluruh akses memakai HTTPS.
- `APP__MOCK_LOCATION_POLICY=reject` menolak absensi/strong point yang terdeteksi memakai
  lokasi tiruan. Rincian: [`fake-gps.md`](fake-gps.md).
- `APP__CORS_ORIGINS` tidak dipakai browser pada deployment ini karena SPA memanggil `/api`
  satu domain lewat nginx; nilai ini berguna bila nanti ada klien lintas origin.
- `APP__CHROME_PATH` hanya diperlukan bila lokasi Chrome tidak terdeteksi otomatis.

---

## 6. Seed demo VPS

Seed VPS **hanya** mengisi Polda Bali dan **Ditreskrimsus**, berbeda dari seed development yang
mengisi seluruh Polres. Jalankan sekali setelah database dibuat:

```bash
sudo systemctl stop absensi-api 2>/dev/null || true
sudo -u absensi bash -c 'set -a; . /etc/absensi/absensi.env; set +a; /opt/absensi/seed --reset --vps'
```

Yang dihasilkan profil `--vps`:

| Data | Isi |
|---|---|
| Organisasi | Polda Bali + **Ditreskrimsus** saja (tanpa Polres) |
| Personel | 40 personel + atasan `000100` + pimpinan `000002`/`000012`; 3 akun admin |
| Absensi kantor | tanggal 1–19 (tidak melewati hari ini), Senin–Jumat |
| Jadwal strong point | tanggal 1–akhir bulan; **Regu 1 tanggal 1–15**, **Regu 2 tanggal 16–akhir bulan** |
| Rekap strong point | tanggal 1–19 (tidak melewati hari ini), termasuk kasus hanya check-in |
| Rencana kerja | ada `draft`, `disetujui`, `perlu_revisi` (+ contoh `diajukan`) |
| Hasil kerja | ada `draft`, `terverifikasi`, `perlu_revisi` (+ contoh `diajukan`) |
| Output | ada `draft`, `disetujui`, `perlu_revisi` (+ contoh `dalam_verifikasi`) |
| Lampiran | PDF + PNG contoh ditulis ke `/var/lib/absensi/uploads` |

> `--reset` menghapus seluruh data lama. Jangan jalankan pada database yang sudah berisi data
> produksi. Setelah seed, seluruh sesi Android/web lama tidak berlaku.

Profil development tidak berubah:

```bash
# hanya untuk database lokal/uji
cargo run --bin seed -- --reset
```

### Akun demo VPS

| Peran | Username / NRP | Password | Kanal |
|---|---|---|---|
| Admin Sistem | `superuser` | `Superuser#2026` | Dashboard |
| Admin Polda | `admin_polda` | `AdminPolda#2026` | Dashboard |
| Admin Satker Polda | `admin_satker_polda` | `AdminSatker#2026` | Dashboard |
| Pimpinan Polda | `000002` | `Pimpinan#2026` | Dashboard/Android |
| Pimpinan Satker Polda | `000012` | `PimpinanSatker#2026` | Dashboard/Android |
| Atasan | `000100` | `Atasan#2026` | Web personel/Android |
| Personel | `85000001` | `Polri#2026` | Web personel/Android |
| Personel lain | `85000002` … `85000040` | `Polri#2026` | Web personel/Android |

Akun Polres (`admin_polres`, `000022`, `000023`) **tidak dibuat** pada profil VPS.

**Ganti seluruh password demo sebelum dipakai sungguhan.**

---

## 7. Layanan systemd

Buat `/etc/systemd/system/absensi-api.service`:

```ini
[Unit]
Description=API Absensi & Penilaian Kinerja Polda Bali
After=network-online.target postgresql.service
Wants=network-online.target

[Service]
Type=simple
User=absensi
Group=absensi
WorkingDirectory=/opt/absensi
Environment=HOME=/var/lib/absensi
EnvironmentFile=/etc/absensi/absensi.env
ExecStart=/opt/absensi/kinerja-api
Restart=always
RestartSec=3
TimeoutStopSec=20
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/absensi
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

> `Environment=HOME` diperlukan Chrome headless untuk menulis cache. `ReadWritePaths` mencakup
> folder unggahan.

Aktifkan:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now absensi-api
sudo systemctl status absensi-api --no-pager
```

Uji dari VPS (tanpa nginx/mTLS):

```bash
curl -fsS http://127.0.0.1:7085/health
# {"status":"ok", ...}
```

Lihat log:

```bash
sudo journalctl -u absensi-api -f
```

---

## 8. nginx + mTLS Cloudflare

### 8.1 Snippet mTLS

Buat `/etc/nginx/snippets/jagradewata-mtls.conf`:

```nginx
ssl_certificate         /etc/ssl/jagradewata_id_cert.pem;
ssl_certificate_key     /etc/ssl/jagradewata_id_key.pem;
ssl_client_certificate  /etc/ssl/cloudflare.crt;
ssl_verify_client       on;
ssl_protocols           TLSv1.2 TLSv1.3;
ssl_session_cache       shared:SSL:10m;
ssl_session_timeout     10m;
```

### 8.2 Dashboard — `absensi.jagradewata.id`

Buat `/etc/nginx/sites-available/absensi`:

```nginx
server {
    listen 80;
    listen [::]:80;
    server_name absensi.jagradewata.id;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    listen [::]:443 ssl;
    server_name absensi.jagradewata.id;

    include snippets/jagradewata-mtls.conf;

    root /var/www/absensi;
    index index.html;
    client_max_body_size 60m;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api/ {
        proxy_pass http://127.0.0.1:7085;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
        proxy_read_timeout 120s;
        proxy_request_buffering off;
    }

    # Berkas unggahan yang diminta lewat jalur /files/...
    location /files/ {
        proxy_pass http://127.0.0.1:7085/api/v1/files/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-Proto https;
    }

    location /_app/immutable/ {
        expires 1y;
        add_header Cache-Control "public, immutable";
        try_files $uri =404;
    }

    gzip on;
    gzip_types text/css application/javascript application/json image/svg+xml;
}
```

### 8.3 Web personel — `personel.jagradewata.id`

Buat `/etc/nginx/sites-available/personel` dengan isi yang sama, ganti:

- `server_name` menjadi `personel.jagradewata.id`
- `root` menjadi `/var/www/personel`

### 8.4 Aktifkan

```bash
sudo ln -s /etc/nginx/sites-available/absensi  /etc/nginx/sites-enabled/absensi
sudo ln -s /etc/nginx/sites-available/personel /etc/nginx/sites-enabled/personel
sudo rm -f /etc/nginx/sites-enabled/default
sudo nginx -t
sudo systemctl reload nginx
```

> `http2` sengaja tidak diaktifkan agar kompatibel dengan nginx 1.24 (Ubuntu 24.04).
> Bila memakai nginx ≥ 1.25.1, tambahkan `http2 on;` di dalam kedua `server` 443.

### 8.5 Uji

Lewat browser: `https://absensi.jagradewata.id` dan `https://personel.jagradewata.id`.

Uji bahwa origin menolak koneksi tanpa sertifikat klien Cloudflare:

```bash
curl -vk https://absensi.jagradewata.id/ 2>&1 | grep -i 'alert\|certificate'
```

Pesan `No required SSL certificate was sent` (HTTP 400) adalah tanda mTLS bekerja.
Untuk diagnosis sementara, buat server block uji di port lain dengan
`ssl_verify_client optional_no_ca;`, lalu hapus setelah selesai.

---

## 9. Firewall

```bash
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable
sudo ufw status verbose
```

- Port **7085** dan **5432** tidak dibuka ke publik; keduanya hanya `127.0.0.1`.
- Port 22 tetap sesuai kebijakan akses server.

---

## 10. Backup & restore

Buat `/usr/local/bin/absensi-backup.sh`:

```bash
#!/bin/sh
set -eu

AKAR=/var/lib/absensi
TUJUAN="${TUJUAN:-$AKAR/backup}"
RETENSI_HARI="${RETENSI_HARI:-30}"
STAMP="$(date +%Y%m%d-%H%M%S)"

set -a
. /etc/absensi/absensi.env
set +a

mkdir -p "$TUJUAN"

pg_dump --dbname="$APP__DATABASE_URL" --clean --if-exists \
    | gzip -9 > "$TUJUAN/absensi-$STAMP.sql.gz"

tar -czf "$TUJUAN/unggahan-$STAMP.tar.gz" -C "$AKAR" uploads

find "$TUJUAN" -maxdepth 1 -name 'absensi-*.sql.gz'   -mtime "+$RETENSI_HARI" -delete
find "$TUJUAN" -maxdepth 1 -name 'unggahan-*.tar.gz' -mtime "+$RETENSI_HARI" -delete

echo "[$(date '+%F %T')] backup selesai: $TUJUAN/absensi-$STAMP.sql.gz"
```

```bash
sudo chmod 750 /usr/local/bin/absensi-backup.sh
sudo /usr/local/bin/absensi-backup.sh
```

Jadwalkan harian pukul 02:30:

```cron
30 2 * * * /usr/local/bin/absensi-backup.sh >> /var/lib/absensi/backup/backup.log 2>&1
```

Restore:

```bash
sudo systemctl stop absensi-api
sudo -u absensi bash -c 'set -a; . /etc/absensi/absensi.env; set +a; gunzip -c /var/lib/absensi/backup/absensi-YYYYMMDD-HHMMSS.sql.gz | psql "$APP__DATABASE_URL"'
sudo tar -xzf /var/lib/absensi/backup/unggahan-YYYYMMDD-HHMMSS.tar.gz -C /var/lib/absensi
sudo chown -R absensi:absensi /var/lib/absensi/uploads
sudo systemctl start absensi-api
```

Salin hasil backup ke luar VPS (mis. object storage) secara berkala.

---

## 11. Pembaruan aplikasi

```bash
# backend
sudo systemctl stop absensi-api
sudo install -m 755 -o absensi -g absensi kinerja-api /opt/absensi/kinerja-api
sudo systemctl start absensi-api

# frontend (dari mesin pengembang)
rsync -az --delete dashboard/build/ user@vps:/tmp/dashboard-build/
rsync -az --delete personel/build/  user@vps:/tmp/personel-build/
```

```bash
# di VPS
sudo rsync -a --delete /tmp/dashboard-build/ /var/www/absensi/
sudo rsync -a --delete /tmp/personel-build/  /var/www/personel/
sudo chown -R www-data:www-data /var/www/absensi /var/www/personel
```

Migrasi baru dijalankan otomatis saat API start (`APP__RUN_MIGRATIONS=true`). Jangan jalankan
seed `--reset` pada database produksi.

---

## 12. Aplikasi Android

- Alamat server: `https://absensi.jagradewata.id/api/v1/` (isi lewat **Profil → Server** pada
  build debug, atau ubah `ALAMAT_BAWAAN` di `android/app/src/main/java/id/polri/poldabali/kinerja/data/Api.kt`
  untuk build rilis).
- Konfigurasi jaringan sudah dipisah: `src/main/res/xml/network_security_config.xml` menetapkan
  `cleartextTrafficPermitted="false"` untuk rilis, sedangkan `src/debug/res/xml/` mengizinkan
  HTTP untuk emulator. Buat APK/AAB dengan `android\build-release.bat` (signing dari
  `android/keystore.properties`).
- Android terhubung ke Cloudflare, bukan langsung ke origin; mTLS origin tidak mengganggu
  aplikasi selama DNS diproksikan Cloudflare.

---

## 13. Pemecahan masalah

| Gejala | Penyebab & solusi |
|---|---|
| `curl https://...` → HTTP 400/495 | mTLS aktif dan klien tidak membawa sertifikat Cloudflare. Uji lewat browser/Cloudflare, atau aktifkan **Authenticated Origin Pulls**. |
| Halaman 404 saat refresh | `try_files ... /index.html;` hilang atau `root` salah. |
| API 502 | `systemctl status absensi-api`; pastikan API mendengarkan `127.0.0.1:7085`. |
| Login sukses tapi sesi hilang | `APP__COOKIE_SECURE=true` tanpa HTTPS, atau jam server tidak sinkron (`timedatectl`). |
| Upload 413 | Naikkan `client_max_body_size` (maks. bukti output 50 MB). |
| Export PDF 500 | Chrome/Chromium belum terpasang atau `APP__CHROME_PATH` salah. Uji `google-chrome --headless --print-to-pdf=/tmp/x.pdf about:blank` sebagai user `absensi`. |
| Chrome: `No usable sandbox` | Gunakan paket Google Chrome `.deb`, atau aktifkan user namespace pada distro. |
| PDF kosong / font kotak | Pasang font: `sudo apt install -y fonts-dejavu fonts-liberation`. |
| Foto tidak muncul | Pastikan `ReadWritePaths` mencakup `/var/lib/absensi` dan file dapat dibaca user `absensi`. |
| `APP__JWT_SECRET minimal 32 karakter` | Isi dengan `openssl rand -hex 32`. |
| Seed gagal FK | Jangan mencampur profil: `--vps` membutuhkan database kosong/`--reset`. |

---

## 14. Daftar periksa setelah deploy

- [ ] `systemctl is-active absensi-api` → `active`
- [ ] `curl -fsS http://127.0.0.1:7085/health` → `{"status":"ok"}`
- [ ] `sudo nginx -t` → `syntax is ok`
- [ ] Login dashboard `superuser` dan web personel `85000001`
- [ ] Rekap absensi menampilkan data 1–19 (sesuai tanggal deploy)
- [ ] Jadwal strong point menampilkan Regu 1 (1–15) dan Regu 2 (16–akhir bulan)
- [ ] Export PDF rekap absensi, jadwal strong point, dan rekap strong point berhasil
- [ ] Unggah foto profil dan foto absensi berhasil, tampil di galeri
- [ ] Password demo sudah diganti; `JWT_SECRET` bukan contoh
- [ ] Backup harian terjadwal dan diuji restore ke database lain
