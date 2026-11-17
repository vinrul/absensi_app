-- Unit kerja (subdit/bagian/satuan) di dalam satker, dipakai untuk filter dashboard.
ALTER TABLE personel ADD COLUMN unit text;

CREATE INDEX personel_unit_idx ON personel (satker_id, unit);
