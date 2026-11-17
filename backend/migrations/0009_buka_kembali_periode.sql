-- Jejak pembukaan kembali periode yang sudah ditutup.
-- Penutupan tetap manual; admin global dapat membuka kembali bila ada data
-- yang belum dinilai atau perlu diperbaiki.

ALTER TABLE periode
    ADD COLUMN IF NOT EXISTS dibuka_kembali_oleh uuid
        REFERENCES personel (id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS dibuka_kembali_at timestamptz,
    ADD COLUMN IF NOT EXISTS alasan_buka_kembali text;
