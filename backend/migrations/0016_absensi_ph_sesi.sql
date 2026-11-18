-- Sesi strong point: satu jadwal dapat memiliki lebih dari satu check-in dalam
-- sehari (mis. pagi dan sore). Check-out dipasangkan ke check-in terakhir yang
-- masih terbuka; aturan jarak minimal 2 jam divalidasi aplikasi.
ALTER TABLE absensi_ph
    DROP CONSTRAINT IF EXISTS absensi_ph_jadwal_ph_id_personel_id_jenis_key;

CREATE INDEX IF NOT EXISTS absensi_ph_sesi_idx
    ON absensi_ph (jadwal_ph_id, personel_id, captured_at);
