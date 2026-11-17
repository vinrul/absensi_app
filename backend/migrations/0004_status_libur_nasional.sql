-- Nama lama `ph` bermakna hari libur. Nama itu dibebaskan untuk modul
-- PH/Strong Point yang terpisah dari absensi harian.
ALTER TYPE status_kehadiran RENAME VALUE 'ph' TO 'libur_nasional';
