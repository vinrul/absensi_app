-- Tambah level organisasi baru dan role ber-scope.
-- Nilai enum ditambahkan pada migrasi terpisah agar aman di PostgreSQL.
ALTER TYPE satker_level ADD VALUE IF NOT EXISTS 'satker';

ALTER TYPE app_role ADD VALUE IF NOT EXISTS 'admin_polda';
ALTER TYPE app_role ADD VALUE IF NOT EXISTS 'admin_satker_polda';
ALTER TYPE app_role ADD VALUE IF NOT EXISTS 'admin_polres';
ALTER TYPE app_role ADD VALUE IF NOT EXISTS 'admin_satker_polres';
ALTER TYPE app_role ADD VALUE IF NOT EXISTS 'pimpinan_polda';
ALTER TYPE app_role ADD VALUE IF NOT EXISTS 'pimpinan_satker_polda';
ALTER TYPE app_role ADD VALUE IF NOT EXISTS 'pimpinan_polres';
ALTER TYPE app_role ADD VALUE IF NOT EXISTS 'pimpinan_satker_polres';
