use std::env;
use std::path::{Path, PathBuf};

use tokio::process::Command;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Lokasi Chrome/Edge. Bisa diatur lewat `APP__CHROME_PATH` atau `CHROME_PATH`.
pub fn chrome_path() -> String {
    for kunci in ["APP__CHROME_PATH", "CHROME_PATH", "LAPORAN_CHROME"] {
        if let Ok(nilai) = env::var(kunci) {
            let nilai = nilai.trim().to_string();
            if !nilai.is_empty() {
                return nilai;
            }
        }
    }

    for path in [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
    ] {
        if Path::new(path).exists() {
            return path.to_string();
        }
    }

    if cfg!(windows) {
        "chrome.exe".to_string()
    } else {
        "google-chrome".to_string()
    }
}

fn file_url(path: &Path) -> String {
    let teks = path.to_string_lossy().replace('\\', "/");
    if teks.starts_with('/') {
        format!("file://{teks}")
    } else {
        format!("file:///{teks}")
    }
}

/// Merender HTML menjadi PDF memakai Chrome/Edge headless.
///
/// Berkas HTML, profil Chrome, dan PDF sementara dibersihkan setelah selesai.
pub async fn render_pdf(html: &str) -> AppResult<Vec<u8>> {
    let dir: PathBuf = env::temp_dir().join(format!("kinerja-pdf-{}", Uuid::now_v7()));
    tokio::fs::create_dir_all(&dir).await?;

    let html_path = dir.join("laporan.html");
    let pdf_path = dir.join("laporan.pdf");
    let chrome_dir = dir.join("chrome");
    tokio::fs::write(&html_path, html).await?;

    let hasil = Command::new(chrome_path())
        .arg("--headless")
        .arg("--disable-gpu")
        .arg("--disable-extensions")
        .arg("--hide-scrollbars")
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg(format!("--user-data-dir={}", chrome_dir.display()))
        .arg("--no-pdf-header-footer")
        .arg("--run-all-compositor-stages-before-draw")
        .arg("--virtual-time-budget=3000")
        .arg(format!("--print-to-pdf={}", pdf_path.display()))
        .arg(file_url(&html_path))
        .kill_on_drop(true)
        .output()
        .await;

    let keluaran = match hasil {
        Ok(keluaran) if keluaran.status.success() => keluaran,
        Ok(keluaran) => {
            let stderr = String::from_utf8_lossy(&keluaran.stderr);
            let _ = tokio::fs::remove_dir_all(&dir).await;
            return Err(AppError::Internal(format!(
                "PDF gagal dibuat oleh Chrome/Edge: {}",
                stderr.trim()
            )));
        }
        Err(error) => {
            let _ = tokio::fs::remove_dir_all(&dir).await;
            return Err(AppError::Internal(format!(
                "Chrome/Edge tidak dapat dijalankan ({error}). Atur APP__CHROME_PATH ke chrome.exe atau msedge.exe."
            )));
        }
    };
    drop(keluaran);

    let bytes = match tokio::fs::read(&pdf_path).await {
        Ok(bytes) => bytes,
        Err(_) => {
            let _ = tokio::fs::remove_dir_all(&dir).await;
            return Err(AppError::Internal(
                "Chrome tidak menghasilkan berkas PDF".into(),
            ));
        }
    };
    let _ = tokio::fs::remove_dir_all(&dir).await;
    Ok(bytes)
}
