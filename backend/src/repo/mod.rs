pub mod absensi;
pub mod audit;
pub mod hasil;
pub mod lampiran;
pub mod libur;
pub mod notifikasi;
pub mod output;
pub mod penilaian;
pub mod periode;
pub mod personel;
pub mod ph;
pub mod rencana;
pub mod satker;
pub mod titik_dinas;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PageMeta {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub meta: PageMeta,
}

impl<T> ListResponse<T> {
    pub fn new(data: Vec<T>, page: i64, per_page: i64, total: i64) -> Self {
        Self {
            data,
            meta: PageMeta {
                page,
                per_page,
                total,
            },
        }
    }
}

/// Pembatas pagination agar klien tidak meminta halaman tak terbatas.
pub fn pagination(page: Option<i64>, per_page: Option<i64>) -> (i64, i64, i64) {
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(20).clamp(1, 200);
    let offset = (page - 1) * per_page;
    (page, per_page, offset)
}
