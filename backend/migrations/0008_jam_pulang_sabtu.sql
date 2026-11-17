-- Jam pulang khusus Sabtu untuk satker yang bekerja setengah hari.
-- NULL berarti Sabtu memakai jam pulang biasa (jika Sabtu termasuk hari kerja).

ALTER TABLE satker
    ADD COLUMN IF NOT EXISTS jam_pulang_sabtu time;

ALTER TABLE satker
    ADD CONSTRAINT satker_jam_pulang_sabtu_check
    CHECK (jam_pulang_sabtu IS NULL OR jam_pulang_sabtu > jam_masuk);
