#!/bin/sh
# Backup harian PostgreSQL + berkas unggahan untuk aplikasi Absensi & Penilaian Kinerja.
#
# Pemakaian:
#   ./backup.sh                 # backup ke ./data/backup
#   RETENSI_HARI=14 ./backup.sh # ubah masa simpan
#
# Jadwalkan lewat cron host, contoh (02:30 setiap hari):
#   30 2 * * * cd /opt/kinerja/deploy && ./backup.sh >> data/backup/backup.log 2>&1

set -eu

AKAR="$(cd "$(dirname "$0")" && pwd)"
TUJUAN="${TUJUAN:-$AKAR/data/backup}"
RETENSI_HARI="${RETENSI_HARI:-30}"
STAMP="$(date +%Y%m%d-%H%M%S)"
BERKAS="$TUJUAN/kinerja-$STAMP.sql.gz"

mkdir -p "$TUJUAN"

echo "[$(date '+%Y-%m-%d %H:%M:%S')] mulai backup"

# pg_dump dijalankan di dalam kontainer agar versi klien selalu sama dengan server.
docker compose -f "$AKAR/docker-compose.yml" exec -T db \
    pg_dump -U "${POSTGRES_USER:-kinerja}" -d "${POSTGRES_DB:-kinerja}" --clean --if-exists \
    | gzip -9 > "$BERKAS"

if [ ! -s "$BERKAS" ]; then
    echo "GAGAL: berkas backup kosong" >&2
    rm -f "$BERKAS"
    exit 1
fi

echo "[$(date '+%Y-%m-%d %H:%M:%S')] database -> $BERKAS ($(du -h "$BERKAS" | cut -f1))"

# Arsipkan berkas unggahan (foto absensi, lampiran, bukti output).
UNGGAHAN="$AKAR/data/uploads"
if [ -d "$UNGGAHAN" ] && [ "$(ls -A "$UNGGAHAN" 2>/dev/null || true)" ]; then
    ARSIP="$TUJUAN/unggahan-$STAMP.tar.gz"
    tar -czf "$ARSIP" -C "$AKAR/data" uploads
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] unggahan -> $ARSIP ($(du -h "$ARSIP" | cut -f1))"
fi

# Hapus backup lebih lama dari RETENSI_HARI.
find "$TUJUAN" -maxdepth 1 -name 'kinerja-*.sql.gz' -mtime "+$RETENSI_HARI" -print -delete
find "$TUJUAN" -maxdepth 1 -name 'unggahan-*.tar.gz' -mtime "+$RETENSI_HARI" -print -delete

echo "[$(date '+%Y-%m-%d %H:%M:%S')] selesai (retensi ${RETENSI_HARI} hari)"

# ------------------------------------------------------------------ restore
# 1. Hentikan API:            docker compose stop api
# 2. Pulihkan database:
#    gunzip -c data/backup/kinerja-YYYYMMDD-HHMMSS.sql.gz \
#      | docker compose exec -T db psql -U kinerja -d kinerja
# 3. Pulihkan berkas:         tar -xzf data/backup/unggahan-*.tar.gz -C data
# 4. Nyalakan kembali:        docker compose up -d api
