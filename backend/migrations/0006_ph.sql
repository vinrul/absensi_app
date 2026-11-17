-- ============================================================================
-- 0006_ph.sql — Modul PH / Strong Point terpisah dari absensi harian
-- ============================================================================

CREATE TABLE strong_point (
    id          uuid PRIMARY KEY,
    satker_id   uuid NOT NULL REFERENCES satker (id) ON DELETE CASCADE,
    nama        text NOT NULL,
    alamat      text,
    latitude    numeric(10, 7) NOT NULL,
    longitude   numeric(10, 7) NOT NULL,
    radius_m    integer NOT NULL DEFAULT 100 CHECK (radius_m BETWEEN 10 AND 5000),
    is_active   boolean NOT NULL DEFAULT true,
    deleted_at  timestamptz,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now(),
    UNIQUE (satker_id, nama)
);
CREATE INDEX strong_point_satker_idx ON strong_point (satker_id);
CREATE TRIGGER strong_point_touch BEFORE UPDATE ON strong_point
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE jadwal_ph (
    id              uuid PRIMARY KEY,
    satker_id       uuid NOT NULL REFERENCES satker (id) ON DELETE CASCADE,
    strong_point_id uuid NOT NULL REFERENCES strong_point (id) ON DELETE RESTRICT,
    tanggal         date NOT NULL,
    jam_mulai       time NOT NULL,
    jam_selesai     time NOT NULL,
    catatan         text,
    created_by      uuid REFERENCES personel (id) ON DELETE SET NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    CHECK (jam_selesai <> jam_mulai)
);
CREATE INDEX jadwal_ph_satker_tanggal_idx ON jadwal_ph (satker_id, tanggal);
CREATE INDEX jadwal_ph_tanggal_idx ON jadwal_ph (tanggal);
CREATE TRIGGER jadwal_ph_touch BEFORE UPDATE ON jadwal_ph
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE jadwal_ph_personel (
    jadwal_ph_id uuid NOT NULL REFERENCES jadwal_ph (id) ON DELETE CASCADE,
    personel_id  uuid NOT NULL REFERENCES personel (id) ON DELETE CASCADE,
    peran        text,
    created_at   timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (jadwal_ph_id, personel_id)
);
CREATE INDEX jadwal_ph_personel_personel_idx ON jadwal_ph_personel (personel_id);

CREATE TABLE absensi_ph (
    id                  uuid PRIMARY KEY,
    client_uuid         uuid NOT NULL UNIQUE,
    jadwal_ph_id        uuid NOT NULL REFERENCES jadwal_ph (id) ON DELETE CASCADE,
    personel_id         uuid NOT NULL REFERENCES personel (id) ON DELETE CASCADE,
    strong_point_id     uuid NOT NULL REFERENCES strong_point (id) ON DELETE RESTRICT,
    satker_id           uuid NOT NULL REFERENCES satker (id) ON DELETE RESTRICT,
    jenis               absensi_jenis NOT NULL,
    captured_at         timestamptz NOT NULL,
    jam                 time NOT NULL,
    latitude            numeric(10, 7),
    longitude           numeric(10, 7),
    akurasi_m           numeric(8, 2),
    jarak_m             numeric(10, 2),
    dalam_radius        boolean NOT NULL DEFAULT false,
    foto_path           text,
    is_offline          boolean NOT NULL DEFAULT false,
    is_mock_suspect     boolean NOT NULL DEFAULT false,
    perlu_verifikasi    boolean NOT NULL DEFAULT false,
    verified_by         uuid REFERENCES personel (id) ON DELETE SET NULL,
    verified_at         timestamptz,
    catatan_verifikator text,
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now(),
    UNIQUE (jadwal_ph_id, personel_id, jenis)
);
CREATE INDEX absensi_ph_jadwal_idx ON absensi_ph (jadwal_ph_id);
CREATE INDEX absensi_ph_personel_idx ON absensi_ph (personel_id, jam);
CREATE INDEX absensi_ph_satker_idx ON absensi_ph (satker_id, captured_at);
CREATE TRIGGER absensi_ph_touch BEFORE UPDATE ON absensi_ph
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
