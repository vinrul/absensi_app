-- ============================================================================
-- 0020_absensi_ph_link.sql
--
-- Absensi strong point juga dicatat pada tabel `absensi` (status `ph`) agar
-- rekap kehadiran kantor menghitung kehadiran strong point dan dapat memakai
-- jadwal sebagai acuan telat/pulang cepat.
-- ============================================================================

ALTER TABLE absensi
    ADD COLUMN IF NOT EXISTS jadwal_ph_id uuid REFERENCES jadwal_ph (id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS absensi_jadwal_ph_idx ON absensi (jadwal_ph_id);
