-- Kesiapan satker sebelum admin global menutup periode.
-- Satu baris per periode + satker. Baris dibuat saat satker ditandai;
-- satker yang belum ditandai dianggap belum siap (left join di query).

CREATE TABLE IF NOT EXISTS periode_satker_kesiapan (
    periode_id      uuid NOT NULL REFERENCES periode (id) ON DELETE CASCADE,
    satker_id       uuid NOT NULL REFERENCES satker (id) ON DELETE CASCADE,
    siap_ditutup    boolean NOT NULL DEFAULT false,
    ditandai_oleh   uuid REFERENCES personel (id) ON DELETE SET NULL,
    ditandai_at     timestamptz,
    catatan         text,
    updated_at      timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (periode_id, satker_id)
);

CREATE INDEX IF NOT EXISTS periode_satker_kesiapan_satker_idx
    ON periode_satker_kesiapan (satker_id);
