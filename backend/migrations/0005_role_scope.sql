-- Role kini memiliki cakupan satker (scope). Role global memakai NULL.
ALTER TABLE personel_role
    ADD COLUMN IF NOT EXISTS scope_satker_id uuid REFERENCES satker (id) ON DELETE CASCADE;

ALTER TABLE personel_role
    DROP CONSTRAINT IF EXISTS personel_role_personel_id_role_key;

CREATE UNIQUE INDEX IF NOT EXISTS personel_role_unik
    ON personel_role (personel_id, role, scope_satker_id) NULLS NOT DISTINCT;

CREATE INDEX IF NOT EXISTS personel_role_scope_idx
    ON personel_role (scope_satker_id);

ALTER TABLE personel_role
    ADD CONSTRAINT personel_role_scope_check
    CHECK (
        role IN ('admin_sistem', 'pimpinan', 'personel', 'atasan')
        OR scope_satker_id IS NOT NULL
    );
