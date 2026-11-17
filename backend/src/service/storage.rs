use std::path::PathBuf;

use chrono::{Datelike, NaiveDate};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

pub const MAX_FOTO_BYTES: usize = 5 * 1024 * 1024;
pub const MAX_LAMPIRAN_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 50 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct LocalStorage {
    root: PathBuf,
}

impl LocalStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub async fn save(&self, key: &str, bytes: &[u8]) -> AppResult<()> {
        let path = self.root.join(key);

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&path, bytes).await?;
        Ok(())
    }

    pub async fn read(&self, key: &str) -> AppResult<Vec<u8>> {
        let path = self.root.join(key);
        tokio::fs::read(&path)
            .await
            .map_err(|_| AppError::NotFound("Berkas tidak ditemukan".into()))
    }

    pub async fn remove(&self, key: &str) -> AppResult<()> {
        let path = self.root.join(key);
        if tokio::fs::try_exists(&path).await.unwrap_or(false) {
            tokio::fs::remove_file(path).await?;
        }
        Ok(())
    }
}

/// Deteksi tipe berkas dari magic bytes (tidak mempercayai ekstensi/MIME klien).
pub fn deteksi_berkas(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(("jpg", "image/jpeg"));
    }
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some(("png", "image/png"));
    }
    if bytes.starts_with(b"%PDF") {
        return Some(("pdf", "application/pdf"));
    }
    if bytes.len() > 12 && &bytes[4..8] == b"ftyp" {
        return Some(("mp4", "video/mp4"));
    }
    None
}

pub fn kunci_absensi(satker_id: Uuid, tanggal: NaiveDate, id: Uuid, ext: &str) -> String {
    format!(
        "absensi/{satker_id}/{:04}/{:02}/{id}.{ext}",
        tanggal.year(),
        tanggal.month()
    )
}

pub fn kunci_ph(satker_id: Uuid, tanggal: NaiveDate, id: Uuid, ext: &str) -> String {
    format!(
        "ph/{satker_id}/{:04}/{:02}/{id}.{ext}",
        tanggal.year(),
        tanggal.month()
    )
}

pub fn kunci_lampiran(satker_id: Uuid, owner: &str, owner_id: Uuid, id: Uuid, ext: &str) -> String {
    format!("lampiran/{satker_id}/{owner}/{owner_id}/{id}.{ext}")
}

/// Batas ukuran per jenis berkas.
pub fn batas_ukuran(kategori: &str) -> usize {
    match kategori {
        "absensi" => MAX_FOTO_BYTES,
        "output" => MAX_OUTPUT_BYTES,
        _ => MAX_LAMPIRAN_BYTES,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deteksi_jpeg() {
        let (ext, mime) = deteksi_berkas(&[0xFF, 0xD8, 0xFF, 0xE0]).unwrap();
        assert_eq!(ext, "jpg");
        assert_eq!(mime, "image/jpeg");
    }

    #[test]
    fn deteksi_pdf() {
        let (ext, _) = deteksi_berkas(b"%PDF-1.7").unwrap();
        assert_eq!(ext, "pdf");
    }

    #[test]
    fn tolak_berkas_tidak_dikenal() {
        assert!(deteksi_berkas(b"halo dunia").is_none());
    }
}
