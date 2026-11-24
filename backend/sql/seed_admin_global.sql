-- ============================================================================
-- seed_admin_global.sql — Akun admin global (Admin Sistem)
--
-- Akun  : superuser
-- Sandi : Superuser#2026
-- Peran : admin_sistem (cakupan NULL = seluruh organisasi)
--
-- Jalankan pada database yang sudah dimigrasi:
--   sudo -u postgres psql -d absensi -f backend/sql/seed_admin_global.sql
--   psql "$APP__DATABASE_URL" -f backend/sql/seed_admin_global.sql
--
-- Idempoten: bila akun sudah ada, hash sandi, status aktif, satker, dan role
-- admin_sistem disetel ulang tanpa menghapus data lain.
--
-- PERINGATAN: ganti sandi setelah login pertama.
-- ============================================================================

BEGIN;

DO $$
DECLARE
    v_admin_id  uuid;
    v_satker_id uuid;
BEGIN
    -- 1. Satker Polda sebagai induk organisasi. Dibuat bila belum ada.
    SELECT id INTO v_satker_id
      FROM satker
     WHERE level = 'polda' AND deleted_at IS NULL
     ORDER BY created_at
     LIMIT 1;

    IF v_satker_id IS NULL THEN
        INSERT INTO satker (id, parent_id, level, nama, kode)
        VALUES (gen_random_uuid(), NULL, 'polda', 'Kepolisian Daerah Bali', 'POLDA-BALI')
        ON CONFLICT (kode) DO NOTHING
        RETURNING id INTO v_satker_id;
    END IF;

    IF v_satker_id IS NULL THEN
        SELECT id INTO v_satker_id
          FROM satker
         WHERE kode = 'POLDA-BALI' AND deleted_at IS NULL;
    END IF;

    IF v_satker_id IS NULL THEN
        RAISE EXCEPTION 'Satker level polda tidak ditemukan dan gagal dibuat';
    END IF;

    -- 2. Akun admin global (tanpa NRP, memakai username).
    SELECT id INTO v_admin_id
      FROM personel
     WHERE jenis_akun = 'admin' AND lower(username) = 'superuser';

    IF v_admin_id IS NULL THEN
        INSERT INTO personel (
            id, jenis_akun, username, nrp, nama, pangkat, jabatan,
            satker_id, atasan_id, password_hash, must_change_password, is_active
        ) VALUES (
            gen_random_uuid(), 'admin', 'superuser', NULL, 'Super User', 'AKP',
            'Administrator Sistem', v_satker_id, NULL,
            '$argon2id$v=19$m=19456,t=2,p=1$DOc8uMXGwMxvE0iwM02J3A$5PwSDkRH20/6IIOMYxHssEwmPr5uphnPNi1TO6GD55c',
            false, true
        )
        RETURNING id INTO v_admin_id;
    ELSE
        UPDATE personel
           SET jenis_akun           = 'admin',
               username             = 'superuser',
               nrp                  = NULL,
               nama                 = 'Super User',
               pangkat              = 'AKP',
               jabatan              = 'Administrator Sistem',
               satker_id            = v_satker_id,
               atasan_id            = NULL,
               password_hash        = '$argon2id$v=19$m=19456,t=2,p=1$DOc8uMXGwMxvE0iwM02J3A$5PwSDkRH20/6IIOMYxHssEwmPr5uphnPNi1TO6GD55c',
               must_change_password = false,
               is_active            = true,
               deleted_at           = NULL
         WHERE id = v_admin_id;
    END IF;

    -- 3. Role admin_sistem dengan cakupan global (NULL).
    DELETE FROM personel_role
     WHERE personel_id = v_admin_id AND role = 'admin_sistem';

    INSERT INTO personel_role (id, personel_id, role, scope_satker_id)
    VALUES (gen_random_uuid(), v_admin_id, 'admin_sistem', NULL);

    RAISE NOTICE 'Akun admin global siap: superuser';
END $$;

COMMIT;
