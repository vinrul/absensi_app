-- Output dapat disimpan sebagai draf sebelum diajukan ke atasan langsung.
ALTER TYPE status_output ADD VALUE IF NOT EXISTS 'draft' BEFORE 'menunggu_verifikasi';
