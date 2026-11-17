-- Status kehadiran "PH (Perintah Hadir)" pada absensi harian.
-- Berbeda dari modul PH/Strong Point yang memiliki jadwal dan absensi sendiri.
ALTER TYPE status_kehadiran ADD VALUE IF NOT EXISTS 'ph';
