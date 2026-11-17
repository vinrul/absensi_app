-- Kalender hari libur.
-- satker_id NULL = libur nasional/berlaku semua satker.
-- satker_id terisi = libur khusus satker tersebut.

CREATE TYPE jenis_libur AS ENUM ('nasional', 'cuti_bersama', 'khusus');

CREATE TABLE hari_libur (
    id          uuid PRIMARY KEY,
    tanggal     date NOT NULL,
    nama        text NOT NULL,
    jenis       jenis_libur NOT NULL DEFAULT 'nasional',
    satker_id   uuid REFERENCES satker (id) ON DELETE CASCADE,
    is_active   boolean NOT NULL DEFAULT true,
    created_by  uuid REFERENCES personel (id) ON DELETE SET NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX hari_libur_unik
    ON hari_libur (tanggal, satker_id) NULLS NOT DISTINCT;

CREATE INDEX hari_libur_tanggal_idx ON hari_libur (tanggal, is_active);
CREATE INDEX hari_libur_satker_idx ON hari_libur (satker_id);

CREATE TRIGGER hari_libur_touch
    BEFORE UPDATE ON hari_libur
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
