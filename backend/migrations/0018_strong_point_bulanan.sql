-- ============================================================================
-- 0018_strong_point_bulanan.sql
--
-- Penyesuaian strong point:
--  * penempatan anggota menjadi tetap: satu anggota = satu titik (tanpa tanggal);
--  * pola mingguan dihapus karena jadwal disusun langsung per bulan;
--  * jadwal yang disalin otomatis dari bulan sebelumnya ditandai agar UI dapat
--    memperingatkan admin.
-- ============================================================================

ALTER TABLE jadwal_ph ADD COLUMN IF NOT EXISTS disalin_dari_bulan date;
CREATE INDEX IF NOT EXISTS jadwal_ph_salinan_idx
    ON jadwal_ph (satker_id, disalin_dari_bulan)
    WHERE disalin_dari_bulan IS NOT NULL;

-- Penempatan: pertahankan baris terbaru per anggota sebelum kolom tanggal hilang.
DELETE FROM penempatan_ph a
 USING penempatan_ph b
 WHERE a.personel_id = b.personel_id
   AND (
       a.berlaku_mulai < b.berlaku_mulai
       OR (a.berlaku_mulai = b.berlaku_mulai AND a.id < b.id)
   );

ALTER TABLE penempatan_ph
    DROP CONSTRAINT IF EXISTS penempatan_ph_personel_id_berlaku_mulai_key;
DROP INDEX IF EXISTS penempatan_ph_personel_idx;
ALTER TABLE penempatan_ph DROP COLUMN IF EXISTS berlaku_mulai;
ALTER TABLE penempatan_ph DROP COLUMN IF EXISTS berlaku_sampai;
ALTER TABLE penempatan_ph
    ADD CONSTRAINT penempatan_ph_personel_key UNIQUE (personel_id);

-- Pola mingguan tidak dipakai lagi; jadwal konkret tetap disimpan di jadwal_ph.
DROP TABLE IF EXISTS pola_ph_satker;
