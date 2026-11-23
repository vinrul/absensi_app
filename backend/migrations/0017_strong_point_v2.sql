-- ============================================================================
-- 0017_strong_point_v2.sql
--
-- Strong point dengan sesi pagi/sore, pola mingguan per satker, penempatan
-- anggota berdasarkan tanggal berlaku, serta pencatatan telat/pulang cepat.
--
-- Jadwal lama diarsipkan (bukan dihapus) agar riwayat absensi PH tetap utuh,
-- tetapi tidak lagi tampil di daftar/rekap baru.
-- ============================================================================

ALTER TABLE jadwal_ph ADD COLUMN IF NOT EXISTS sesi text;
ALTER TABLE jadwal_ph ADD COLUMN IF NOT EXISTS is_arsip boolean NOT NULL DEFAULT false;

ALTER TABLE jadwal_ph DROP CONSTRAINT IF EXISTS jadwal_ph_sesi_check;
ALTER TABLE jadwal_ph ADD CONSTRAINT jadwal_ph_sesi_check
    CHECK (sesi IS NULL OR sesi IN ('pagi', 'sore'));

UPDATE jadwal_ph SET is_arsip = true WHERE sesi IS NULL;

CREATE INDEX IF NOT EXISTS jadwal_ph_aktif_idx
    ON jadwal_ph (satker_id, tanggal, sesi) WHERE NOT is_arsip;

ALTER TABLE absensi_ph ADD COLUMN IF NOT EXISTS menit_telat integer NOT NULL DEFAULT 0;
ALTER TABLE absensi_ph ADD COLUMN IF NOT EXISTS menit_pulang_cepat integer NOT NULL DEFAULT 0;

-- Titik strong point sekarang dimiliki unit Polda/Polres, bukan satker.
-- Ganti nama titik yang akan bentrok setelah dipindah ke induk yang sama.
UPDATE strong_point sp
   SET nama = sp.nama || ' (' || s.nama || ')'
  FROM satker s
 WHERE sp.satker_id = s.id
   AND s.level = 'satker'
   AND s.parent_id IS NOT NULL
   AND EXISTS (
       SELECT 1
         FROM strong_point lain
        WHERE lain.satker_id = s.parent_id
          AND lain.nama = sp.nama
          AND lain.id <> sp.id
   );

UPDATE strong_point sp
   SET satker_id = s.parent_id
  FROM satker s
 WHERE sp.satker_id = s.id
   AND s.level = 'satker'
   AND s.parent_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS pola_ph_satker (
    id          uuid PRIMARY KEY,
    satker_id   uuid NOT NULL REFERENCES satker (id) ON DELETE CASCADE,
    hari        smallint NOT NULL CHECK (hari BETWEEN 1 AND 7),
    sesi        text NOT NULL CHECK (sesi IN ('pagi', 'sore')),
    jam_mulai   time NOT NULL,
    jam_selesai time NOT NULL,
    is_active   boolean NOT NULL DEFAULT true,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now(),
    UNIQUE (satker_id, hari, sesi),
    CHECK (jam_selesai > jam_mulai)
);
CREATE INDEX IF NOT EXISTS pola_ph_satker_idx ON pola_ph_satker (satker_id);

DROP TRIGGER IF EXISTS pola_ph_satker_touch ON pola_ph_satker;
CREATE TRIGGER pola_ph_satker_touch BEFORE UPDATE ON pola_ph_satker
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS penempatan_ph (
    id              uuid PRIMARY KEY,
    personel_id     uuid NOT NULL REFERENCES personel (id) ON DELETE CASCADE,
    strong_point_id uuid NOT NULL REFERENCES strong_point (id) ON DELETE RESTRICT,
    berlaku_mulai   date NOT NULL,
    berlaku_sampai  date,
    created_by      uuid REFERENCES personel (id) ON DELETE SET NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    UNIQUE (personel_id, berlaku_mulai),
    CHECK (berlaku_sampai IS NULL OR berlaku_sampai >= berlaku_mulai)
);
CREATE INDEX IF NOT EXISTS penempatan_ph_personel_idx
    ON penempatan_ph (personel_id, berlaku_mulai DESC);

DROP TRIGGER IF EXISTS penempatan_ph_touch ON penempatan_ph;
CREATE TRIGGER penempatan_ph_touch BEFORE UPDATE ON penempatan_ph
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
