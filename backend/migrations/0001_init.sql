-- ============================================================================
-- 0001_init.sql — Skema awal aplikasi absensi & penilaian kinerja personel
-- ============================================================================

-- ---------------------------------------------------------------- ENUM TYPES
CREATE TYPE satker_level        AS ENUM ('polda', 'polres');
CREATE TYPE app_role            AS ENUM ('admin_sistem', 'personel', 'atasan', 'pimpinan');
CREATE TYPE absensi_jenis       AS ENUM ('masuk', 'pulang');
CREATE TYPE status_kehadiran    AS ENUM ('dinas_kantor', 'dinas_luar', 'piket', 'ph', 'cuti', 'izin', 'sakit', 'tanpa_keterangan');
CREATE TYPE titik_dinas_jenis   AS ENUM ('kantor', 'luar');
CREATE TYPE jenis_rencana       AS ENUM ('harian', 'mingguan', 'bulanan', 'triwulan', 'penugasan_khusus');
CREATE TYPE dasar_tugas_jenis   AS ENUM ('sp', 'tor', 'perintah_lisan');
CREATE TYPE status_rencana      AS ENUM ('draft', 'diajukan', 'disetujui', 'perlu_revisi', 'ditolak');
CREATE TYPE status_hasil        AS ENUM ('draft', 'diajukan', 'terverifikasi', 'perlu_revisi');
CREATE TYPE jenis_output        AS ENUM ('laporan', 'dokumen_pendukung', 'foto', 'video', 'data_diagram', 'berita_acara', 'surat_nota_dinas', 'link_sistem');
CREATE TYPE status_output       AS ENUM ('menunggu_verifikasi', 'dalam_verifikasi', 'disetujui', 'perlu_revisi');
CREATE TYPE berkas_owner        AS ENUM ('rencana', 'hasil', 'output', 'absensi');
CREATE TYPE berkas_kategori     AS ENUM ('lampiran', 'bukti', 'foto');
CREATE TYPE periode_status      AS ENUM ('open', 'closed');
CREATE TYPE indikator_kinerja   AS ENUM ('absensi', 'rencana_kerja', 'hasil_kerja', 'output');
CREATE TYPE kategori_kinerja    AS ENUM ('sangat_baik', 'baik', 'cukup', 'perlu_pembinaan');
CREATE TYPE keputusan_penilaian AS ENUM ('disetujui', 'perlu_perbaikan');

-- --------------------------------------------------------------- updated_at
CREATE FUNCTION set_updated_at() RETURNS trigger AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ---------------------------------------------------------------------- SATKER
CREATE TABLE satker (
    id          uuid PRIMARY KEY,
    parent_id   uuid REFERENCES satker (id) ON DELETE RESTRICT,
    level       satker_level NOT NULL,
    nama        text NOT NULL,
    kode        text NOT NULL UNIQUE,
    alamat      text,
    timezone    text NOT NULL DEFAULT 'Asia/Makassar',
    hari_kerja  smallint[] NOT NULL DEFAULT '{1,2,3,4,5}', -- ISO 1=Senin .. 7=Minggu
    jam_masuk   time NOT NULL DEFAULT '08:00',
    jam_pulang  time NOT NULL DEFAULT '16:00',
    is_active   boolean NOT NULL DEFAULT true,
    deleted_at  timestamptz,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX satker_parent_idx ON satker (parent_id);
CREATE TRIGGER satker_touch BEFORE UPDATE ON satker FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- -------------------------------------------------------------------- PERSONEL
CREATE TABLE personel (
    id                   uuid PRIMARY KEY,
    nrp                  text NOT NULL UNIQUE,
    nama                 text NOT NULL,
    pangkat              text NOT NULL,
    jabatan              text NOT NULL,
    satker_id            uuid NOT NULL REFERENCES satker (id) ON DELETE RESTRICT,
    atasan_id            uuid REFERENCES personel (id) ON DELETE SET NULL,
    email                text,
    phone                text,
    foto_path            text,
    password_hash        text NOT NULL,
    must_change_password boolean NOT NULL DEFAULT true,
    is_active            boolean NOT NULL DEFAULT true,
    last_login_at        timestamptz,
    deleted_at           timestamptz,
    created_at           timestamptz NOT NULL DEFAULT now(),
    updated_at           timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX personel_satker_idx ON personel (satker_id);
CREATE INDEX personel_atasan_idx ON personel (atasan_id);
CREATE TRIGGER personel_touch BEFORE UPDATE ON personel FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE personel_role (
    id          uuid PRIMARY KEY,
    personel_id uuid NOT NULL REFERENCES personel (id) ON DELETE CASCADE,
    role        app_role NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    UNIQUE (personel_id, role)
);

CREATE TABLE refresh_token (
    id          uuid PRIMARY KEY,
    personel_id uuid NOT NULL REFERENCES personel (id) ON DELETE CASCADE,
    token_hash  text NOT NULL UNIQUE,
    device_id   text,
    user_agent  text,
    expires_at  timestamptz NOT NULL,
    revoked_at  timestamptz,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX refresh_token_personel_idx ON refresh_token (personel_id);

-- ----------------------------------------------------------------- TITIK DINAS
CREATE TABLE titik_dinas (
    id         uuid PRIMARY KEY,
    satker_id  uuid NOT NULL REFERENCES satker (id) ON DELETE CASCADE,
    nama       text NOT NULL,
    jenis      titik_dinas_jenis NOT NULL DEFAULT 'kantor',
    latitude   numeric(10, 7) NOT NULL,
    longitude  numeric(10, 7) NOT NULL,
    radius_m   integer NOT NULL DEFAULT 100 CHECK (radius_m BETWEEN 10 AND 5000),
    is_active  boolean NOT NULL DEFAULT true,
    deleted_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX titik_dinas_satker_idx ON titik_dinas (satker_id);
CREATE TRIGGER titik_dinas_touch BEFORE UPDATE ON titik_dinas FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- -------------------------------------------------------------------- PERIODE
CREATE TABLE periode (
    id             uuid PRIMARY KEY,
    nama           text NOT NULL,
    tipe           text NOT NULL DEFAULT 'bulanan' CHECK (tipe IN ('bulanan')),
    tanggal_mulai  date NOT NULL,
    tanggal_selesai date NOT NULL,
    status         periode_status NOT NULL DEFAULT 'open',
    ditutup_oleh   uuid REFERENCES personel (id) ON DELETE SET NULL,
    ditutup_at     timestamptz,
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    CHECK (tanggal_selesai >= tanggal_mulai),
    UNIQUE (tanggal_mulai)
);
CREATE TRIGGER periode_touch BEFORE UPDATE ON periode FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE bobot_indikator (
    id          uuid PRIMARY KEY,
    periode_id  uuid NOT NULL REFERENCES periode (id) ON DELETE CASCADE,
    indikator   indikator_kinerja NOT NULL,
    bobot       numeric(5, 2) NOT NULL CHECK (bobot >= 0 AND bobot <= 100),
    UNIQUE (periode_id, indikator)
);

-- -------------------------------------------------------------------- ABSENSI
CREATE TABLE absensi (
    id                 uuid PRIMARY KEY,
    client_uuid        uuid NOT NULL UNIQUE,
    personel_id        uuid NOT NULL REFERENCES personel (id) ON DELETE CASCADE,
    satker_id          uuid NOT NULL REFERENCES satker (id) ON DELETE RESTRICT,
    tanggal            date NOT NULL,
    jenis              absensi_jenis NOT NULL,
    status_kehadiran   status_kehadiran NOT NULL,
    captured_at        timestamptz NOT NULL,
    jam                time NOT NULL,
    latitude           numeric(10, 7),
    longitude          numeric(10, 7),
    akurasi_m          numeric(8, 2),
    jarak_m            numeric(10, 2),
    dalam_radius       boolean NOT NULL DEFAULT false,
    is_mock_suspect    boolean NOT NULL DEFAULT false,
    is_offline         boolean NOT NULL DEFAULT false,
    foto_path          text,
    device_id          text,
    perlu_verifikasi   boolean NOT NULL DEFAULT false,
    verified_by        uuid REFERENCES personel (id) ON DELETE SET NULL,
    verified_at        timestamptz,
    catatan_verifikator text,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    UNIQUE (personel_id, tanggal, jenis)
);
CREATE INDEX absensi_satker_tanggal_idx ON absensi (satker_id, tanggal);
CREATE INDEX absensi_personel_tanggal_idx ON absensi (personel_id, tanggal);
CREATE INDEX absensi_perlu_verifikasi_idx ON absensi (perlu_verifikasi) WHERE perlu_verifikasi;
CREATE TRIGGER absensi_touch BEFORE UPDATE ON absensi FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- --------------------------------------------------------------- RENCANA KERJA
CREATE TABLE rencana_kerja (
    id                uuid PRIMARY KEY,
    personel_id       uuid NOT NULL REFERENCES personel (id) ON DELETE CASCADE,
    satker_id         uuid NOT NULL REFERENCES satker (id) ON DELETE RESTRICT,
    periode_id        uuid NOT NULL REFERENCES periode (id) ON DELETE RESTRICT,
    program           text NOT NULL,
    kegiatan          text NOT NULL,
    uraian            text NOT NULL,
    tujuan            text,
    indikator_kinerja text,
    jenis             jenis_rencana NOT NULL,
    dasar_jenis       dasar_tugas_jenis,
    dasar_nomor       text,
    dasar_tanggal     date,
    dasar_perihal     text,
    target_kuantitas  numeric(12, 2),
    satuan            text,
    target_kualitas   text,
    target_waktu      date,
    tanggal_mulai     date NOT NULL,
    tanggal_selesai   date NOT NULL,
    status            status_rencana NOT NULL DEFAULT 'draft',
    atasan_id         uuid REFERENCES personel (id) ON DELETE SET NULL,
    catatan_atasan    text,
    diajukan_at       timestamptz,
    diputuskan_at     timestamptz,
    jumlah_revisi     integer NOT NULL DEFAULT 0,
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now(),
    CHECK (tanggal_selesai >= tanggal_mulai)
);
CREATE INDEX rencana_personel_periode_idx ON rencana_kerja (personel_id, periode_id);
CREATE INDEX rencana_status_idx ON rencana_kerja (status);
CREATE INDEX rencana_atasan_idx ON rencana_kerja (atasan_id);
CREATE TRIGGER rencana_touch BEFORE UPDATE ON rencana_kerja FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE rencana_tahapan (
    id              uuid PRIMARY KEY,
    rencana_id      uuid NOT NULL REFERENCES rencana_kerja (id) ON DELETE CASCADE,
    urutan          integer NOT NULL,
    uraian          text NOT NULL,
    target_kuantitas numeric(12, 2),
    satuan          text,
    bobot           numeric(5, 2),
    UNIQUE (rencana_id, urutan)
);

-- ----------------------------------------------------------------- HASIL KERJA
CREATE TABLE hasil_kerja (
    id                 uuid PRIMARY KEY,
    rencana_id         uuid NOT NULL REFERENCES rencana_kerja (id) ON DELETE CASCADE,
    personel_id        uuid NOT NULL REFERENCES personel (id) ON DELETE CASCADE,
    satker_id          uuid NOT NULL REFERENCES satker (id) ON DELETE RESTRICT,
    uraian_pelaksanaan text NOT NULL,
    progres_persen     numeric(5, 2) NOT NULL DEFAULT 0 CHECK (progres_persen BETWEEN 0 AND 100),
    jumlah_realisasi   numeric(12, 2),
    kendala            text,
    tanggal_realisasi  date NOT NULL,
    status             status_hasil NOT NULL DEFAULT 'draft',
    skor_kuantitas     numeric(5, 2),
    skor_kualitas      numeric(5, 2),
    skor_waktu         numeric(5, 2),
    skor_total         numeric(5, 2),
    verified_by        uuid REFERENCES personel (id) ON DELETE SET NULL,
    catatan_verifikator text,
    verified_at        timestamptz,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX hasil_rencana_idx ON hasil_kerja (rencana_id);
CREATE INDEX hasil_personel_idx ON hasil_kerja (personel_id);
CREATE TRIGGER hasil_touch BEFORE UPDATE ON hasil_kerja FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ---------------------------------------------------------------------- OUTPUT
CREATE TABLE output (
    id                  uuid PRIMARY KEY,
    hasil_kerja_id      uuid NOT NULL REFERENCES hasil_kerja (id) ON DELETE CASCADE,
    rencana_id          uuid NOT NULL REFERENCES rencana_kerja (id) ON DELETE CASCADE,
    personel_id         uuid NOT NULL REFERENCES personel (id) ON DELETE CASCADE,
    satker_id           uuid NOT NULL REFERENCES satker (id) ON DELETE RESTRICT,
    jenis_output        jenis_output NOT NULL,
    judul               text NOT NULL,
    uraian              text,
    tanggal_hasil       date NOT NULL,
    url_eksternal       text,
    status              status_output NOT NULL DEFAULT 'menunggu_verifikasi',
    skor_kelengkapan    numeric(5, 2),
    skor_kesesuaian     numeric(5, 2),
    skor_keabsahan      numeric(5, 2),
    skor_total          numeric(5, 2),
    verified_by         uuid REFERENCES personel (id) ON DELETE SET NULL,
    catatan_verifikator text,
    verified_at         timestamptz,
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX output_personel_idx ON output (personel_id);
CREATE INDEX output_hasil_idx ON output (hasil_kerja_id);
CREATE INDEX output_status_idx ON output (status);
CREATE TRIGGER output_touch BEFORE UPDATE ON output FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- -------------------------------------------------------------------- LAMPIRAN
CREATE TABLE lampiran (
    id          uuid PRIMARY KEY,
    owner_type  berkas_owner NOT NULL,
    owner_id    uuid NOT NULL,
    kategori    berkas_kategori NOT NULL DEFAULT 'lampiran',
    file_name   text NOT NULL,
    mime        text NOT NULL,
    size_bytes  bigint NOT NULL,
    storage_key text NOT NULL,
    uploaded_by uuid NOT NULL REFERENCES personel (id) ON DELETE CASCADE,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX lampiran_owner_idx ON lampiran (owner_type, owner_id);

-- ------------------------------------------------------------------ PENILAIAN
CREATE TABLE penilaian (
    id                  uuid PRIMARY KEY,
    personel_id         uuid NOT NULL REFERENCES personel (id) ON DELETE CASCADE,
    periode_id          uuid NOT NULL REFERENCES periode (id) ON DELETE CASCADE,
    satker_id           uuid NOT NULL REFERENCES satker (id) ON DELETE RESTRICT,
    skor_absensi        numeric(5, 2) NOT NULL DEFAULT 0,
    skor_rencana_kerja  numeric(5, 2) NOT NULL DEFAULT 0,
    skor_hasil_kerja    numeric(5, 2) NOT NULL DEFAULT 0,
    skor_output         numeric(5, 2) NOT NULL DEFAULT 0,
    nilai_akhir         numeric(5, 2) NOT NULL DEFAULT 0,
    kategori            kategori_kinerja NOT NULL,
    catatan_pimpinan    text,
    keputusan           keputusan_penilaian,
    dinilai_oleh        uuid REFERENCES personel (id) ON DELETE SET NULL,
    computed_at         timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now(),
    UNIQUE (personel_id, periode_id)
);
CREATE INDEX penilaian_periode_satker_idx ON penilaian (periode_id, satker_id);
CREATE TRIGGER penilaian_touch BEFORE UPDATE ON penilaian FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE penilaian_detail (
    id           uuid PRIMARY KEY,
    penilaian_id uuid NOT NULL REFERENCES penilaian (id) ON DELETE CASCADE,
    indikator    indikator_kinerja NOT NULL,
    skor         numeric(5, 2) NOT NULL,
    komponen     jsonb NOT NULL DEFAULT '{}'::jsonb
);
CREATE INDEX penilaian_detail_idx ON penilaian_detail (penilaian_id);

-- ---------------------------------------------------------------- NOTIFIKASI
CREATE TABLE notifikasi (
    id          uuid PRIMARY KEY,
    personel_id uuid NOT NULL REFERENCES personel (id) ON DELETE CASCADE,
    tipe        text NOT NULL,
    judul       text NOT NULL,
    body        text NOT NULL,
    ref_type    text,
    ref_id      uuid,
    read_at     timestamptz,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX notifikasi_personel_idx ON notifikasi (personel_id, read_at);

-- ------------------------------------------------------------------ AUDIT LOG
CREATE TABLE audit_log (
    id          uuid PRIMARY KEY,
    actor_id    uuid REFERENCES personel (id) ON DELETE SET NULL,
    aksi        text NOT NULL,
    entitas     text NOT NULL,
    entitas_id  uuid,
    before_data jsonb,
    after_data  jsonb,
    ip          text,
    user_agent  text,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX audit_log_entitas_idx ON audit_log (entitas, entitas_id);
CREATE INDEX audit_log_created_idx ON audit_log (created_at DESC);
