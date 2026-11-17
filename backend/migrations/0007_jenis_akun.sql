-- Pisahkan akun admin (pengelola sistem) dari personel (wajib absen).
--
-- Admin memakai `username` dan tidak memiliki NRP. Personel tetap memakai NRP
-- dan tidak memiliki username. Keduanya masih memakai satu tabel identitas agar
-- seluruh foreign key audit/verifikasi/penilaian tetap valid.

ALTER TABLE personel
    ADD COLUMN IF NOT EXISTS jenis_akun text NOT NULL DEFAULT 'personel',
    ADD COLUMN IF NOT EXISTS username text;

ALTER TABLE personel DROP CONSTRAINT IF EXISTS personel_nrp_key;
ALTER TABLE personel ALTER COLUMN nrp DROP NOT NULL;

-- Akun lama yang memiliki peran admin dipindahkan menjadi akun admin.
-- Username sementara memakai prefix agar unik; seed demo akan menggantinya.
UPDATE personel p
   SET jenis_akun = 'admin',
       username = 'admin_' || p.nrp,
       nrp = NULL
 WHERE EXISTS (
       SELECT 1
         FROM personel_role pr
        WHERE pr.personel_id = p.id
          AND pr.role IN ('admin_sistem', 'admin_polda', 'admin_satker_polda',
                          'admin_polres', 'admin_satker_polres')
 );

-- Akun admin tidak boleh memegang peran personel, dan sebaliknya.
DELETE FROM personel_role pr
 USING personel p
 WHERE pr.personel_id = p.id
   AND p.jenis_akun = 'admin'
   AND pr.role IN ('personel', 'atasan', 'pimpinan', 'pimpinan_polda',
                   'pimpinan_satker_polda', 'pimpinan_polres',
                   'pimpinan_satker_polres');

DELETE FROM personel_role pr
 USING personel p
 WHERE pr.personel_id = p.id
   AND p.jenis_akun = 'personel'
   AND pr.role IN ('admin_sistem', 'admin_polda', 'admin_satker_polda',
                   'admin_polres', 'admin_satker_polres');

CREATE UNIQUE INDEX IF NOT EXISTS personel_nrp_unik
    ON personel (nrp) WHERE nrp IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS personel_username_unik
    ON personel (lower(username)) WHERE username IS NOT NULL;

CREATE INDEX IF NOT EXISTS personel_jenis_akun_idx ON personel (jenis_akun);

ALTER TABLE personel
    ADD CONSTRAINT personel_jenis_akun_check
    CHECK (jenis_akun IN ('admin', 'personel'));

ALTER TABLE personel
    ADD CONSTRAINT personel_identitas_check
    CHECK (
        (jenis_akun = 'admin' AND username IS NOT NULL AND nrp IS NULL)
        OR
        (jenis_akun = 'personel' AND nrp IS NOT NULL AND username IS NULL)
    );
