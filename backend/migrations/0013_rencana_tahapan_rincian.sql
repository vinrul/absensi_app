-- Rincian kegiatan pada wizard rencana kerja Android.
-- Kolom baru opsional agar tahapan lama (satu uraian per baris) tetap valid.
ALTER TABLE rencana_tahapan
    ADD COLUMN program text,
    ADD COLUMN kegiatan text,
    ADD COLUMN tujuan text,
    ADD COLUMN indikator_kinerja text,
    ADD COLUMN target_kualitas text,
    ADD COLUMN tanggal_mulai date,
    ADD COLUMN tanggal_selesai date;
