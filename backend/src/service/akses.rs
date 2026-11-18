//! Aturan cakupan organisasi dan role ber-scope.
//!
//! Polda adalah akar, Polres berada di bawah Polda, dan satker berada di bawah
//! Polda atau Polres. Role admin/pimpinan tertentu hanya berlaku pada satu
//! cabang pohon organisasi.

use std::collections::BTreeSet;

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::enums::{AppRole, JenisAkun, KanalLogin, SatkerLevel};
use crate::error::{AppError, AppResult};
use crate::repo::personel::{RoleScopeInput, RoleScopeRow};
use crate::repo::satker::{self, SatkerRow};

/// Menentukan apakah sebuah jenis akun boleh login pada kanal tertentu.
///
/// Admin adalah pengelola sistem dan hanya boleh masuk lewat dashboard web.
/// Android dan web personel hanya menerima akun personel (termasuk pimpinan/atasan).
pub fn kanal_login_diizinkan(jenis_akun: JenisAkun, kanal: KanalLogin) -> bool {
    match kanal {
        KanalLogin::Web => true,
        KanalLogin::Android | KanalLogin::PersonelWeb => jenis_akun == JenisAkun::Personel,
    }
}

/// Dashboard web hanya untuk akun yang memiliki minimal satu role admin/pimpinan.
/// Role atasan/personel tetap boleh ada pada akun yang sama, tetapi tidak membuka dashboard.
pub fn boleh_login_dashboard(roles: &[AppRole]) -> bool {
    roles
        .iter()
        .any(|role| role.admin_any() || role.pimpinan_any())
}

/// Refresh lewat cookie diizinkan untuk dashboard dan web personel.
/// Web personel menandai sesinya dengan `device_id = "web-personel"`.
pub fn refresh_cookie_diizinkan(roles: &[AppRole], device_id: Option<&str>) -> bool {
    device_id == Some("web-personel") || boleh_login_dashboard(roles)
}

/// Menghitung seluruh satker yang boleh diakses oleh seorang pengguna.
pub async fn hitung_satker_akses(
    pool: &PgPool,
    personel_id: Uuid,
    role_scopes: &[RoleScopeRow],
) -> AppResult<Vec<Uuid>> {
    let mut hasil = BTreeSet::new();

    let global = role_scopes
        .iter()
        .any(|r| r.role.admin_global() || r.role == AppRole::Pimpinan);
    if global {
        return semua_satker(pool).await;
    }

    for item in role_scopes {
        match item.role {
            AppRole::AdminPolda | AppRole::AdminPolres => {
                if let Some(scope) = item.scope_satker_id {
                    hasil.extend(sendiri_dan_anak_langsung(pool, scope).await?);
                }
            }
            AppRole::AdminSatkerPolda | AppRole::AdminSatkerPolres => {
                if let Some(scope) = item.scope_satker_id {
                    hasil.insert(scope);
                }
            }
            AppRole::PimpinanPolda | AppRole::PimpinanPolres => {
                if let Some(scope) = item.scope_satker_id {
                    hasil.extend(subtree(pool, scope).await?);
                }
            }
            AppRole::PimpinanSatkerPolda | AppRole::PimpinanSatkerPolres => {
                if let Some(scope) = item.scope_satker_id {
                    hasil.insert(scope);
                }
            }
            AppRole::Atasan | AppRole::Personel => {
                hasil.extend(satker_personel(pool, personel_id).await?);
            }
            AppRole::AdminSistem | AppRole::Pimpinan => {
                // Sudah ditangani oleh `global` di atas.
            }
        }
    }

    // Akun tanpa scope yang sah tetap boleh melihat satkernya sendiri.
    if hasil.is_empty() {
        hasil.extend(satker_personel(pool, personel_id).await?);
    }

    Ok(hasil.into_iter().collect())
}

async fn semua_satker(pool: &PgPool) -> AppResult<Vec<Uuid>> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM satker WHERE deleted_at IS NULL ORDER BY level, nama",
    )
    .fetch_all(pool)
    .await?)
}

async fn sendiri_dan_anak_langsung(pool: &PgPool, root: Uuid) -> AppResult<Vec<Uuid>> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM satker
          WHERE deleted_at IS NULL
            AND (id = $1 OR (parent_id = $1 AND level = 'satker'))
          ORDER BY nama",
    )
    .bind(root)
    .fetch_all(pool)
    .await?)
}

async fn subtree(pool: &PgPool, root: Uuid) -> AppResult<Vec<Uuid>> {
    const SQL: &str = r#"
        WITH RECURSIVE pohon AS (
            SELECT id FROM satker WHERE id = $1 AND deleted_at IS NULL
            UNION ALL
            SELECT s.id FROM satker s
              JOIN pohon p ON s.parent_id = p.id
             WHERE s.deleted_at IS NULL
        )
        SELECT id FROM pohon
    "#;
    Ok(sqlx::query_scalar::<_, Uuid>(SQL)
        .bind(root)
        .fetch_all(pool)
        .await?)
}

async fn satker_personel(pool: &PgPool, personel_id: Uuid) -> AppResult<Vec<Uuid>> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        "SELECT satker_id FROM personel WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(personel_id)
    .fetch_all(pool)
    .await?)
}

/// Aturan hierarki saat membuat node organisasi baru.
pub fn hierarki_sah(level: SatkerLevel, parent: Option<&SatkerRow>) -> bool {
    match (level, parent) {
        (SatkerLevel::Polda, None) => true,
        (SatkerLevel::Polres, Some(induk)) => induk.level == SatkerLevel::Polda,
        (SatkerLevel::Satker, Some(induk)) => {
            matches!(induk.level, SatkerLevel::Polda | SatkerLevel::Polres)
        }
        _ => false,
    }
}

pub fn boleh_buat_satker(
    role_scopes: &[RoleScopeRow],
    level: SatkerLevel,
    parent: Option<&SatkerRow>,
) -> bool {
    if role_scopes.iter().any(|r| r.role.admin_global()) {
        return hierarki_sah(level, parent);
    }
    if !hierarki_sah(level, parent) {
        return false;
    }
    let Some(induk) = parent else {
        return false;
    };
    match level {
        SatkerLevel::Satker => role_scopes.iter().any(|r| {
            matches!(r.role, AppRole::AdminPolda | AppRole::AdminPolres)
                && r.scope_satker_id == Some(induk.id)
        }),
        _ => false,
    }
}

pub fn boleh_kelola_satker(role_scopes: &[RoleScopeRow], target: &SatkerRow) -> bool {
    role_scopes.iter().any(|r| {
        if r.role.admin_global() {
            return true;
        }
        let Some(scope) = r.scope_satker_id else {
            return false;
        };
        match r.role {
            AppRole::AdminPolda | AppRole::AdminPolres => {
                target.id == scope
                    || (target.level == SatkerLevel::Satker && target.parent_id == Some(scope))
            }
            AppRole::AdminSatkerPolda | AppRole::AdminSatkerPolres => target.id == scope,
            _ => false,
        }
    })
}

/// Personel boleh dikelola jika satkernya berada di cabang yang dikelola.
pub fn boleh_kelola_personel(role_scopes: &[RoleScopeRow], target: &SatkerRow) -> bool {
    role_scopes.iter().any(|r| {
        if r.role.admin_global() {
            return true;
        }
        let Some(scope) = r.scope_satker_id else {
            return false;
        };
        match r.role {
            AppRole::AdminPolda | AppRole::AdminPolres => {
                target.id == scope
                    || (target.level == SatkerLevel::Satker && target.parent_id == Some(scope))
            }
            AppRole::AdminSatkerPolda | AppRole::AdminSatkerPolres => target.id == scope,
            _ => false,
        }
    })
}

pub async fn boleh_kelola_personel_id(
    pool: &PgPool,
    role_scopes: &[RoleScopeRow],
    target_satker: Uuid,
) -> AppResult<bool> {
    let target = satker::find(pool, target_satker)
        .await?
        .ok_or_else(|| AppError::Unprocessable("Satker personel tidak ditemukan".into()))?;
    Ok(boleh_kelola_personel(role_scopes, &target))
}

pub fn boleh_melihat_personel(
    role_scopes: &[RoleScopeRow],
    satker_akses: &[Uuid],
    aktor_id: Uuid,
    target_id: Uuid,
    target_satker: Uuid,
) -> bool {
    if aktor_id == target_id {
        return true;
    }
    if !satker_akses.contains(&target_satker) {
        return false;
    }
    role_scopes
        .iter()
        .any(|r| r.role.admin_any() || r.role.pimpinan_any() || r.role == AppRole::Atasan)
}

pub async fn boleh_kelola_ph(
    pool: &PgPool,
    role_scopes: &[RoleScopeRow],
    target_satker: Uuid,
) -> AppResult<bool> {
    boleh_kelola_personel_id(pool, role_scopes, target_satker).await
}

pub async fn scope_mencakup(pool: &PgPool, scope: Uuid, target: Uuid) -> AppResult<bool> {
    let ids = subtree(pool, scope).await?;
    Ok(ids.contains(&target))
}

/// Validasi penugasan role + scope. Aturan ini sengaja konservatif:
/// admin ber-scope tidak boleh membuat admin global atau memperluas cakupannya
/// sendiri di luar cabang organisasinya.
pub async fn validasi_role_assignments(
    pool: &PgPool,
    aktor_scopes: &[RoleScopeRow],
    target_satker: Uuid,
    assignments: &[RoleScopeInput],
) -> AppResult<()> {
    if assignments.is_empty() {
        return Err(AppError::Unprocessable("Minimal satu peran".into()));
    }

    let super_admin = aktor_scopes.iter().any(|r| r.role.admin_global());
    let target_node = satker::find(pool, target_satker)
        .await?
        .ok_or_else(|| AppError::Unprocessable("Satker personel tidak ditemukan".into()))?;

    for item in assignments {
        let scope = item.scope_satker_id;
        if item.role.butuh_scope() && scope.is_none() {
            return Err(AppError::Unprocessable(format!(
                "Peran {:?} membutuhkan cakupan satker",
                item.role
            )));
        }
        if !item.role.butuh_scope() && scope.is_some() {
            return Err(AppError::Unprocessable(format!(
                "Peran {:?} tidak menggunakan cakupan satker",
                item.role
            )));
        }

        let scope_node =
            match scope {
                Some(id) => Some(satker::find(pool, id).await?.ok_or_else(|| {
                    AppError::Unprocessable("Satker cakupan tidak ditemukan".into())
                })?),
                None => None,
            };

        if let Some(id) = scope
            && !scope_mencakup(pool, id, target_satker).await?
        {
            return Err(AppError::Unprocessable(
                "Satker cakupan tidak mencakup satker personel".into(),
            ));
        }

        let parent_node = match scope_node.as_ref().and_then(|n| n.parent_id) {
            Some(id) => satker::find(pool, id).await?,
            None => None,
        };

        let scope_ok = match item.role {
            AppRole::AdminSistem | AppRole::Pimpinan => super_admin,
            AppRole::AdminPolda | AppRole::PimpinanPolda => {
                scope_node
                    .as_ref()
                    .is_some_and(|n| n.level == SatkerLevel::Polda)
                    && (super_admin
                        || aktor_scopes.iter().any(|r| {
                            matches!(r.role, AppRole::AdminPolda) && r.scope_satker_id == scope
                        }))
            }
            AppRole::AdminPolres | AppRole::PimpinanPolres => {
                scope_node
                    .as_ref()
                    .is_some_and(|n| n.level == SatkerLevel::Polres)
                    && (super_admin
                        || aktor_scopes.iter().any(|r| {
                            matches!(r.role, AppRole::AdminPolres) && r.scope_satker_id == scope
                        }))
            }
            AppRole::AdminSatkerPolda | AppRole::PimpinanSatkerPolda => {
                scope_node
                    .as_ref()
                    .is_some_and(|n| n.level == SatkerLevel::Satker)
                    && parent_node
                        .as_ref()
                        .is_some_and(|p| p.level == SatkerLevel::Polda)
                    && (super_admin
                        || aktor_scopes.iter().any(|r| {
                            matches!(r.role, AppRole::AdminPolda)
                                && r.scope_satker_id == parent_node.as_ref().map(|p| p.id)
                        })
                        || aktor_scopes.iter().any(|r| {
                            matches!(r.role, AppRole::AdminSatkerPolda)
                                && r.scope_satker_id == scope
                        }))
            }
            AppRole::AdminSatkerPolres | AppRole::PimpinanSatkerPolres => {
                scope_node
                    .as_ref()
                    .is_some_and(|n| n.level == SatkerLevel::Satker)
                    && parent_node
                        .as_ref()
                        .is_some_and(|p| p.level == SatkerLevel::Polres)
                    && (super_admin
                        || aktor_scopes.iter().any(|r| {
                            matches!(r.role, AppRole::AdminPolres)
                                && r.scope_satker_id == parent_node.as_ref().map(|p| p.id)
                        })
                        || aktor_scopes.iter().any(|r| {
                            matches!(r.role, AppRole::AdminSatkerPolres)
                                && r.scope_satker_id == scope
                        }))
            }
            AppRole::Atasan | AppRole::Personel => {
                boleh_kelola_personel(aktor_scopes, &target_node)
            }
        };

        if !scope_ok {
            return Err(AppError::Forbidden(
                "Anda tidak berhak menetapkan peran/cakupan tersebut".into(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_boleh_login_web() {
        assert!(kanal_login_diizinkan(JenisAkun::Admin, KanalLogin::Web));
    }

    #[test]
    fn admin_ditolak_login_android() {
        assert!(!kanal_login_diizinkan(
            JenisAkun::Admin,
            KanalLogin::Android
        ));
    }

    #[test]
    fn personel_boleh_login_web_dan_android() {
        assert!(kanal_login_diizinkan(JenisAkun::Personel, KanalLogin::Web));
        assert!(kanal_login_diizinkan(
            JenisAkun::Personel,
            KanalLogin::Android
        ));
        assert!(kanal_login_diizinkan(
            JenisAkun::Personel,
            KanalLogin::PersonelWeb
        ));
    }

    #[test]
    fn admin_ditolak_login_personel_web() {
        assert!(!kanal_login_diizinkan(
            JenisAkun::Admin,
            KanalLogin::PersonelWeb
        ));
    }

    #[test]
    fn refresh_cookie_hanya_dashboard_atau_web_personel() {
        assert!(refresh_cookie_diizinkan(
            &[AppRole::Pimpinan],
            Some("dashboard")
        ));
        assert!(refresh_cookie_diizinkan(
            &[AppRole::Atasan, AppRole::Personel],
            Some("web-personel")
        ));
        assert!(!refresh_cookie_diizinkan(
            &[AppRole::Atasan, AppRole::Personel],
            Some("dashboard")
        ));
        assert!(!refresh_cookie_diizinkan(&[AppRole::Personel], None));
    }

    #[test]
    fn dashboard_hanya_untuk_admin_atau_pimpinan() {
        assert!(boleh_login_dashboard(&[AppRole::AdminSatkerPolda]));
        assert!(boleh_login_dashboard(&[AppRole::PimpinanSatkerPolda]));
        assert!(boleh_login_dashboard(&[
            AppRole::Atasan,
            AppRole::Personel,
            AppRole::Pimpinan
        ]));
        assert!(!boleh_login_dashboard(&[
            AppRole::Atasan,
            AppRole::Personel
        ]));
    }
}
