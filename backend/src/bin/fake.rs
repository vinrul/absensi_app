//! CLI bantu pengembangan untuk mengatur jam server palsu.
//!
//! Contoh:
//!   cargo run --bin fake -- 11-11-2026 07:45
//!   cargo run --bin fake -- 2026-11-11 07:45 --zona Asia/Jakarta
//!   cargo run --bin fake -- status
//!   cargo run --bin fake -- reset
//!
//! API harus sudah berjalan dan `APP__ALLOW_FAKE_TIME=true` harus aktif.

use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::ExitCode;

use chrono::NaiveDate;
use serde_json::{Value, json};

const ZONA_BAWAAN: &str = "Asia/Makassar";

fn main() -> ExitCode {
    match jalankan() {
        Ok(()) => ExitCode::SUCCESS,
        Err(pesan) => {
            eprintln!("Gagal: {pesan}");
            ExitCode::FAILURE
        }
    }
}

fn jalankan() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        cetak_bantuan();
        return Err("argumen belum diisi".into());
    }

    let host = env::var("APP__FAKE_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = env::var("APP__PORT").unwrap_or_else(|_| "8080".into());

    if args[0] == "status" {
        let (kode, isi) = kirim(&host, &port, "GET", "/api/v1/dev/waktu", None)?;
        return cetak_hasil(kode, &isi);
    }

    if args[0] == "reset" || args[0] == "--reset" {
        let (kode, isi) = kirim(&host, &port, "POST", "/api/v1/dev/waktu/reset", None)?;
        return cetak_hasil(kode, &isi);
    }

    if args.len() < 2 {
        cetak_bantuan();
        return Err("format: fake <DD-MM-YYYY> <HH:MM> [--zona Asia/Jakarta]".into());
    }

    let tanggal = normalisasi_tanggal(&args[0])?;
    let jam = normalisasi_jam(&args[1])?;
    let zona = if args.len() >= 4 && args[2] == "--zona" {
        args[3].clone()
    } else {
        ZONA_BAWAAN.to_string()
    };

    let body = json!({
        "tanggal": tanggal,
        "jam": jam,
        "zona": zona,
    })
    .to_string();

    let (kode, isi) = kirim(&host, &port, "POST", "/api/v1/dev/waktu", Some(&body))?;
    cetak_hasil(kode, &isi)
}

fn normalisasi_tanggal(input: &str) -> Result<String, String> {
    NaiveDate::parse_from_str(input, "%d-%m-%Y")
        .or_else(|_| NaiveDate::parse_from_str(input, "%Y-%m-%d"))
        .map(|tanggal| tanggal.format("%Y-%m-%d").to_string())
        .map_err(|_| format!("tanggal '{input}' tidak valid; gunakan DD-MM-YYYY"))
}

fn normalisasi_jam(input: &str) -> Result<String, String> {
    let bagian: Vec<&str> = input.split(':').collect();
    if bagian.len() != 2 {
        return Err(format!("jam '{input}' tidak valid; gunakan HH:MM"));
    }
    let jam: u32 = bagian[0]
        .parse()
        .map_err(|_| format!("jam '{input}' tidak valid"))?;
    let menit: u32 = bagian[1]
        .parse()
        .map_err(|_| format!("jam '{input}' tidak valid"))?;
    if jam > 23 || menit > 59 {
        return Err(format!("jam '{input}' di luar rentang 00:00–23:59"));
    }
    Ok(format!("{jam:02}:{menit:02}"))
}

fn kirim(
    host: &str,
    port: &str,
    metode: &str,
    path: &str,
    body: Option<&str>,
) -> Result<(u16, String), String> {
    let alamat = format!("{host}:{port}");
    let mut stream = TcpStream::connect(&alamat).map_err(|_| {
        format!(
            "tidak dapat terhubung ke API di {alamat}. Jalankan backend lebih dulu \
             dan pastikan APP__ALLOW_FAKE_TIME=true."
        )
    })?;

    let isi = body.unwrap_or("");
    let permintaan = format!(
        "{metode} {path} HTTP/1.1\r\nHost: {alamat}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{isi}",
        isi.len()
    );
    stream
        .write_all(permintaan.as_bytes())
        .map_err(|err| format!("gagal mengirim permintaan: {err}"))?;

    let mut mentah = String::new();
    stream
        .read_to_string(&mut mentah)
        .map_err(|err| format!("gagal membaca respons: {err}"))?;

    let (kepala, badan) = mentah
        .split_once("\r\n\r\n")
        .ok_or_else(|| "respons HTTP tidak lengkap".to_string())?;
    let kode = kepala
        .lines()
        .next()
        .and_then(|baris| baris.split_whitespace().nth(1))
        .and_then(|nilai| nilai.parse::<u16>().ok())
        .ok_or_else(|| "status HTTP tidak dapat dibaca".to_string())?;

    Ok((kode, badan.trim().to_string()))
}

fn cetak_hasil(kode: u16, isi: &str) -> Result<(), String> {
    if let Ok(json) = serde_json::from_str::<Value>(isi) {
        if kode == 200 {
            println!(
                "{}",
                serde_json::to_string_pretty(&json).unwrap_or(isi.into())
            );
            return Ok(());
        }
        let pesan = json
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(Value::as_str)
            .unwrap_or(isi);
        return Err(format!("HTTP {kode}: {pesan}"));
    }
    Err(format!("HTTP {kode}: {isi}"))
}

fn cetak_bantuan() {
    println!(
        "Penggunaan:\n  \
         cargo run --bin fake -- <DD-MM-YYYY> <HH:MM> [--zona Asia/Jakarta]\n  \
         cargo run --bin fake -- status\n  \
         cargo run --bin fake -- reset\n\n\
         Contoh:\n  \
         cargo run --bin fake -- 11-11-2026 07:45\n\n\
         Syarat: backend berjalan dan APP__ALLOW_FAKE_TIME=true."
    );
}
