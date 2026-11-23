use base64::Engine as _;
use chrono::NaiveDate;

/// Escape teks dinamis sebelum dimasukkan ke HTML laporan.
pub fn esc(teks: &str) -> String {
    teks.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn format_tanggal(tanggal: NaiveDate) -> String {
    tanggal.format("%d-%m-%Y").to_string()
}

pub struct Kolom<'a> {
    pub judul: &'a str,
    /// Lebar relatif dalam persen; total kolom sebaiknya 100.
    pub lebar: u8,
}

pub struct Laporan<'a> {
    pub judul: &'a str,
    pub periode: &'a str,
    pub satker: &'a str,
    pub personel: &'a str,
    pub status: &'a str,
    pub dicetak_oleh: &'a str,
    pub dicetak_pada: &'a str,
    pub kolom: Vec<Kolom<'a>>,
    /// Potongan `<tr>…</tr>` yang sudah di-escape.
    pub baris: String,
    pub jumlah: usize,
    pub total: i64,
    /// Catatan tambahan di bawah tabel; bila `None`, dipakai ringkasan jumlah.
    pub catatan: Option<String>,
}

/// Merangkai HTML laporan A4 landscape lengkap dengan kop, tabel, dan footer.
pub fn html(laporan: &Laporan<'_>) -> String {
    html_dengan_css(laporan, "")
}

/// Laporan dengan isi HTML kustom (mis. matriks multi-tabel).
pub struct LaporanKustom<'a> {
    pub judul: &'a str,
    pub periode: &'a str,
    pub satker: &'a str,
    pub personel: &'a str,
    pub status: &'a str,
    pub dicetak_oleh: &'a str,
    pub dicetak_pada: &'a str,
    /// Potongan HTML mentah yang sudah di-escape pemanggil.
    pub isi: String,
    pub catatan: Option<String>,
    pub css_tambahan: &'a str,
}

pub fn html_kustom(laporan: &LaporanKustom<'_>) -> String {
    let logo = base64::engine::general_purpose::STANDARD.encode(include_bytes!(
        "../../../dashboard/static/logo-polda-bali.png"
    ));
    let catatan = laporan.catatan.clone().unwrap_or_default();

    format!(
        r#"<!doctype html>
<html lang="id">
<head><meta charset="utf-8"><style>{css}{css_tambahan}</style></head>
<body>
  <table class="kop"><tr>
    <td class="logo"><img src="data:image/png;base64,{logo}" alt="Logo SIAP Bali"></td>
    <td class="identitas">
      <div class="instansi">Kepolisian Negara Republik Indonesia</div>
      <div class="instansi">Daerah Bali · Direktorat Reserse Kriminal Khusus</div>
      <h1>{judul}</h1>
    </td>
  </tr></table>

  <div class="meta">
    <span><strong>Periode:</strong> {periode}</span>
    <span>Dicetak: {dicetak_pada}</span>
  </div>
  <div class="filters">
    <span><strong>Satker:</strong> {satker}</span>
    <span><strong>Personel:</strong> {personel}</span>
    <span><strong>Status:</strong> {status}</span>
  </div>

  {isi}

  <p class="catatan">{catatan}</p>
  <div class="footer">
    <div>Dicetak oleh: <strong>{dicetak_oleh}</strong></div>
    <div class="ttd">Bali, {dicetak_pada}<br><br><br><strong>{dicetak_oleh}</strong></div>
  </div>
</body>
</html>"#,
        css = CSS_LAPORAN,
        css_tambahan = laporan.css_tambahan,
        logo = logo,
        judul = esc(laporan.judul),
        periode = esc(laporan.periode),
        satker = esc(laporan.satker),
        personel = esc(laporan.personel),
        status = esc(laporan.status),
        dicetak_pada = esc(laporan.dicetak_pada),
        dicetak_oleh = esc(laporan.dicetak_oleh),
        isi = laporan.isi,
        catatan = esc(&catatan),
    )
}

/// Merangkai HTML laporan dengan tambahan CSS khusus (mis. roster tanggal).
pub fn html_dengan_css(laporan: &Laporan<'_>, css_tambahan: &str) -> String {
    let kepala = laporan
        .kolom
        .iter()
        .map(|kolom| {
            format!(
                "<th style=\"width:{}%\">{}</th>",
                kolom.lebar,
                esc(kolom.judul)
            )
        })
        .collect::<String>();

    let baris = if laporan.jumlah == 0 {
        format!(
            "<tr><td colspan=\"{}\" class=\"empty\">Tidak ada data sesuai filter.</td></tr>",
            laporan.kolom.len()
        )
    } else {
        laporan.baris.clone()
    };

    let catatan = laporan.catatan.clone().unwrap_or_else(|| {
        if laporan.total > laporan.jumlah as i64 {
            format!(
                "Menampilkan {} dari {} data (batas cetak 5000). Persempit filter untuk hasil lengkap.",
                laporan.jumlah, laporan.total
            )
        } else {
            format!("Total {} data.", laporan.jumlah)
        }
    });

    let isi = format!(
        "<table class=\"data\"><thead><tr>{kepala}</tr></thead><tbody>{baris}</tbody></table>"
    );

    html_kustom(&LaporanKustom {
        judul: laporan.judul,
        periode: laporan.periode,
        satker: laporan.satker,
        personel: laporan.personel,
        status: laporan.status,
        dicetak_oleh: laporan.dicetak_oleh,
        dicetak_pada: laporan.dicetak_pada,
        isi,
        catatan: Some(catatan),
        css_tambahan,
    })
}

pub const CSS_LAPORAN: &str = r#"
@page { size: A4 landscape; margin: 10mm 12mm; }
* { box-sizing: border-box; }
body { font-family: "Segoe UI", Arial, sans-serif; font-size: 9pt; color: #1e293b; margin: 0; }
.kop { width: 100%; border-bottom: 2px solid #0b2545; padding-bottom: 8px; margin-bottom: 10px; }
.kop td { vertical-align: middle; }
.logo { width: 76px; }
.logo img { width: 66px; height: auto; }
.identitas { text-align: center; }
.identitas .instansi { font-size: 9pt; letter-spacing: .06em; text-transform: uppercase; color: #334155; }
.identitas h1 { font-size: 14pt; margin: 2px 0 0; color: #0b2545; text-transform: uppercase; letter-spacing: .03em; }
.meta { display: flex; justify-content: space-between; font-size: 8.5pt; color: #475569; margin-bottom: 6px; }
.filters { font-size: 8.5pt; color: #334155; margin-bottom: 8px; }
.filters span { margin-right: 16px; }
table.data { width: 100%; border-collapse: collapse; }
table.data th { background: #0b2545; color: #fff; font-size: 8pt; text-align: left; padding: 5px 6px; }
table.data td { border-bottom: 1px solid #cbd5e1; padding: 4px 6px; vertical-align: top; }
table.data tr:nth-child(even) td { background: #f8fafc; }
thead { display: table-header-group; }
tr { page-break-inside: avoid; }
.badge { display: inline-block; padding: 1px 6px; border-radius: 8px; font-size: 7.5pt; white-space: nowrap; }
.badge-draft { background: #e2e8f0; color: #334155; }
.badge-diajukan { background: #dbeafe; color: #1d4ed8; }
.badge-disetujui { background: #d1fae5; color: #047857; }
.badge-terverifikasi { background: #d1fae5; color: #047857; }
.badge-dalam_verifikasi { background: #dbeafe; color: #1d4ed8; }
.badge-menunggu_verifikasi { background: #e0f2fe; color: #0369a1; }
.badge-perlu_revisi { background: #fef3c7; color: #b45309; }
.badge-ditolak { background: #ffe4e6; color: #be123c; }
.footer { display: flex; justify-content: space-between; margin-top: 14px; font-size: 8.5pt; color: #475569; }
.ttd { text-align: right; min-width: 220px; }
.ttd strong { color: #1e293b; }
.catatan { margin-top: 6px; font-size: 8pt; color: #64748b; }
.empty { text-align: center; color: #64748b; padding: 16px; }
"#;
