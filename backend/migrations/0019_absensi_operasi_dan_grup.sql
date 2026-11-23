-- ============================================================================
-- 0019_absensi_operasi_dan_grup.sql
--
-- Absensi harian menjadi siklus check-in/check-out:
--  * status baru Operasi/Pam;
--  * satu check-in dan satu check-out dihubungkan lewat `grup_id`, sehingga
--    piket/operasi boleh memiliki beberapa siklus dalam satu hari;
--  * batasan unik lama (satu masuk + satu pulang per tanggal) dihapus.
-- ============================================================================

ALTER TYPE status_kehadiran ADD VALUE IF NOT EXISTS 'operasi';

ALTER TABLE absensi ADD COLUMN IF NOT EXISTS grup_id uuid;

-- Baris lama: check-in memakai id sendiri, check-out memakai grup yang sama.
UPDATE absensi SET grup_id = id WHERE grup_id IS NULL;

UPDATE absensi AS pulang
   SET grup_id = masuk.grup_id
  FROM absensi AS masuk
 WHERE pulang.jenis = 'pulang'
   AND masuk.jenis = 'masuk'
   AND pulang.personel_id = masuk.personel_id
   AND pulang.tanggal = masuk.tanggal
   AND pulang.status_kehadiran = masuk.status_kehadiran;

ALTER TABLE absensi ALTER COLUMN grup_id SET NOT NULL;

ALTER TABLE absensi
    DROP CONSTRAINT IF EXISTS absensi_personel_id_tanggal_jenis_key;

CREATE INDEX IF NOT EXISTS absensi_grup_idx ON absensi (grup_id);
CREATE INDEX IF NOT EXISTS absensi_personel_jenis_idx ON absensi (personel_id, jenis);
