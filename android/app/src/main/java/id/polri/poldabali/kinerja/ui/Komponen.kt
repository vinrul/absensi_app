package id.polri.poldabali.kinerja.ui

import android.graphics.BitmapFactory
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import id.polri.poldabali.kinerja.data.Absensi
import id.polri.poldabali.kinerja.ui.theme.Warna
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.util.Locale

/**
 * Avatar bulat dari byte foto profil.
 *
 * `tampilkanInisial = false` dipakai Beranda agar tidak menggambar placeholder
 * ketika personel belum mengunggah foto.
 */
@Composable
fun FotoProfil(
    foto: ByteArray?,
    nama: String? = null,
    ukuran: Dp = 48.dp,
    tampilkanInisial: Boolean = true
) {
    val bitmap = remember(foto) {
        foto?.let {
            runCatching {
                BitmapFactory.decodeByteArray(it, 0, it.size)?.asImageBitmap()
            }.getOrNull()
        }
    }
    if (bitmap == null && !tampilkanInisial) return

    Box(
        Modifier
            .size(ukuran)
            .clip(CircleShape)
            .background(Warna.NavySoft),
        contentAlignment = Alignment.Center
    ) {
        if (bitmap != null) {
            Image(
                bitmap = bitmap,
                contentDescription = "Foto profil",
                contentScale = ContentScale.Crop,
                modifier = Modifier.matchParentSize()
            )
        } else {
            Text(
                nama?.trim()?.firstOrNull()?.uppercase() ?: "?",
                style = MaterialTheme.typography.titleMedium,
                color = Warna.Navy,
                fontWeight = FontWeight.SemiBold
            )
        }
    }
}

@Composable
fun Kartu(
    modifier: Modifier = Modifier,
    isi: @Composable () -> Unit
) {
    Surface(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(14.dp),
        color = Color.White,
        tonalElevation = 0.dp,
        shadowElevation = 1.dp
    ) {
        Column(Modifier.padding(16.dp)) { isi() }
    }
}

@Composable
fun JudulBagian(teks: String, keterangan: String? = null, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth()) {
        Text(teks, style = MaterialTheme.typography.titleMedium, color = Warna.Teks)
        if (keterangan != null) {
            Text(
                keterangan,
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }
    }
}

@Composable
fun Lencana(teks: String, warna: Color, latar: Color = warna.copy(alpha = 0.12f)) {
    Box(
        Modifier
            .background(latar, RoundedCornerShape(50))
            .padding(horizontal = 10.dp, vertical = 3.dp)
    ) {
        Text(
            teks,
            style = MaterialTheme.typography.labelSmall,
            color = warna,
            fontWeight = FontWeight.SemiBold
        )
    }
}

@Composable
fun BarisInfo(label: String, nilai: String, warnaNilai: Color = Warna.Teks) {
    Row(
        Modifier.fillMaxWidth().padding(vertical = 3.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.Top
    ) {
        Text(label, style = MaterialTheme.typography.bodySmall, color = Warna.TeksLembut)
        Text(
            nilai,
            style = MaterialTheme.typography.bodySmall,
            color = warnaNilai,
            fontWeight = FontWeight.Medium,
            textAlign = TextAlign.End
        )
    }
}

@Composable
fun TombolUtama(
    teks: String,
    modifier: Modifier = Modifier,
    aktif: Boolean = true,
    memuat: Boolean = false,
    onClick: () -> Unit
) {
    Button(
        onClick = onClick,
        modifier = modifier.fillMaxWidth().height(48.dp),
        enabled = aktif && !memuat,
        shape = RoundedCornerShape(12.dp),
        colors = ButtonDefaults.buttonColors(
            containerColor = Warna.Navy,
            contentColor = Color.White,
            disabledContainerColor = Warna.Navy.copy(alpha = 0.35f)
        )
    ) {
        if (memuat) {
            CircularProgressIndicator(
                modifier = Modifier.size(18.dp),
                strokeWidth = 2.dp,
                color = Color.White
            )
        } else {
            Text(teks, fontWeight = FontWeight.SemiBold)
        }
    }
}

@Composable
fun TombolKedua(
    teks: String,
    modifier: Modifier = Modifier,
    aktif: Boolean = true,
    onClick: () -> Unit
) {
    OutlinedButton(
        onClick = onClick,
        modifier = modifier.fillMaxWidth().height(46.dp),
        enabled = aktif,
        shape = RoundedCornerShape(12.dp)
    ) {
        Text(teks, color = Warna.Navy, fontWeight = FontWeight.Medium)
    }
}

@Composable
fun KotakPesan(pesan: String, jenis: JenisPesan = JenisPesan.Info) {
    val warna = when (jenis) {
        JenisPesan.Info -> Warna.Biru
        JenisPesan.Sukses -> Warna.Hijau
        JenisPesan.Galat -> Warna.Merah
    }
    Box(
        Modifier
            .fillMaxWidth()
            .background(warna.copy(alpha = 0.10f), RoundedCornerShape(10.dp))
            .padding(12.dp)
    ) {
        Text(pesan, style = MaterialTheme.typography.bodySmall, color = warna)
    }
}

enum class JenisPesan { Info, Sukses, Galat }

@Composable
fun BarNilai(label: String, nilai: Double, warna: Color) {
    Column(Modifier.padding(vertical = 4.dp)) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text(label, style = MaterialTheme.typography.bodySmall, color = Warna.TeksLembut)
            Text(
                String.format(Locale.US, "%.1f", nilai),
                style = MaterialTheme.typography.bodySmall,
                color = warna,
                fontWeight = FontWeight.SemiBold
            )
        }
        LinearProgressIndicator(
            progress = { (nilai / 100.0).coerceIn(0.0, 1.0).toFloat() },
            modifier = Modifier.fillMaxWidth().height(6.dp).padding(top = 4.dp),
            color = warna,
            trackColor = warna.copy(alpha = 0.15f)
        )
    }
}

@Composable
fun LayarKosong(pesan: String) {
    Box(
        Modifier.fillMaxWidth().padding(vertical = 32.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(
            pesan,
            style = MaterialTheme.typography.bodyMedium,
            color = Warna.TeksLembut,
            textAlign = TextAlign.Center
        )
    }
}

// --------------------------------------------------------------- util format

val FORMAT_TANGGAL: DateTimeFormatter = DateTimeFormatter.ofPattern("d MMM yyyy", Locale.forLanguageTag("id-ID"))

fun tanggalHariIni(): String = LocalDate.now().toString()

fun awalBulan(): String = LocalDate.now().withDayOfMonth(1).toString()

fun tanggalIndo(iso: String): String = runCatching {
    LocalDate.parse(iso).format(FORMAT_TANGGAL)
}.getOrDefault(iso)

fun labelStatusKehadiran(status: String): String = when (status) {
    "dinas_kantor" -> "Dinas Kantor"
    "dinas_luar" -> "Dinas Luar"
    "ph" -> "PH (Perintah Hadir)"
    "piket" -> "Piket/Shift"
    "libur_nasional" -> "Libur Nasional"
    "cuti" -> "Cuti"
    "izin" -> "Izin"
    "sakit" -> "Sakit"
    "tanpa_keterangan" -> "Tanpa Keterangan"
    else -> status
}

fun labelStatusRencana(status: String): String = when (status) {
    "draft" -> "Draft"
    "diajukan" -> "Diajukan"
    "disetujui" -> "Disetujui"
    "perlu_revisi" -> "Perlu Revisi"
    "ditolak" -> "Ditolak"
    else -> status
}

fun labelJenisRencana(jenis: String): String = when (jenis) {
    "harian" -> "Harian"
    "mingguan" -> "Mingguan"
    "bulanan" -> "Bulanan"
    "triwulan" -> "Triwulanan/Semester"
    "penugasan_khusus" -> "Penugasan Khusus"
    else -> jenis
}

fun labelStatusOutput(status: String): String = when (status) {
    "draft" -> "Draft"
    "menunggu_verifikasi" -> "Menunggu Verifikasi"
    "dalam_verifikasi" -> "Dalam Verifikasi"
    "disetujui" -> "Disetujui"
    "perlu_revisi" -> "Perlu Revisi"
    else -> status
}

val JENIS_OUTPUT: List<Pair<String, String>> = listOf(
    "laporan" to "Laporan",
    "dokumen_pendukung" to "Dokumen Pendukung",
    "foto" to "Foto Kegiatan",
    "video" to "Video Kegiatan",
    "data_diagram" to "Data/Diagram",
    "berita_acara" to "Berita Acara",
    "surat_nota_dinas" to "Surat/Nota Dinas",
    "link_sistem" to "Link/Sistem"
)

fun labelJenisOutput(jenis: String): String =
    JENIS_OUTPUT.firstOrNull { it.first == jenis }?.second ?: jenis

/** Pangkat Polri dari terendah ke tertinggi (termasuk Brigpol). */
val PANGKAT_POLRI: List<Pair<String, String>> = listOf(
    "BHARADA" to "Bhayangkara Dua (Bharada)",
    "BHARATU" to "Bhayangkara Satu (Bharatu)",
    "BHARAKA" to "Bhayangkara Kepala (Bharaka)",
    "ABRIPDA" to "Ajun Brigadir Polisi Dua (Abripda)",
    "ABRIPTU" to "Ajun Brigadir Polisi Satu (Abriptu)",
    "ABRIPKA" to "Ajun Brigadir Polisi Kepala (Abripka)",
    "BRIPDA" to "Brigadir Polisi Dua (Bripda)",
    "BRIPTU" to "Brigadir Polisi Satu (Briptu)",
    "BRIGPOL" to "Brigadir Polisi (Brigpol)",
    "BRIPKA" to "Brigadir Polisi Kepala (Bripka)",
    "AIPDA" to "Ajun Inspektur Polisi Dua (Aipda)",
    "AIPTU" to "Ajun Inspektur Polisi Satu (Aiptu)",
    "IPDA" to "Inspektur Polisi Dua (Ipda)",
    "IPTU" to "Inspektur Polisi Satu (Iptu)",
    "AKP" to "Ajun Komisaris Polisi (AKP)",
    "KOMPOL" to "Komisaris Polisi (Kompol)",
    "AKBP" to "Ajun Komisaris Besar Polisi (AKBP)",
    "KOMBES POL" to "Komisaris Besar Polisi (Kombes Pol)",
    "BRIGJEN POL" to "Brigadir Jenderal Polisi (Brigjen Pol)",
    "IRJEN POL" to "Inspektur Jenderal Polisi (Irjen Pol)",
    "KOMJEN POL" to "Komisaris Jenderal Polisi (Komjen Pol)",
    "JENDERAL POL" to "Jenderal Polisi"
)

/** Pangkat PNS/ASN dari golongan terendah (I/a) sampai tertinggi (IV/e). */
val PANGKAT_ASN: List<Pair<String, String>> = listOf(
    "JURU MUDA (I/A)" to "Juru Muda (Gol. I/a)",
    "JURU MUDA TINGKAT I (I/B)" to "Juru Muda Tingkat I (Gol. I/b)",
    "JURU (I/C)" to "Juru (Gol. I/c)",
    "JURU TINGKAT I (I/D)" to "Juru Tingkat I (Gol. I/d)",
    "PENGATUR MUDA (II/A)" to "Pengatur Muda (Gol. II/a)",
    "PENGATUR MUDA TINGKAT I (II/B)" to "Pengatur Muda Tingkat I (Gol. II/b)",
    "PENGATUR (II/C)" to "Pengatur (Gol. II/c)",
    "PENGATUR TINGKAT I (II/D)" to "Pengatur Tingkat I (Gol. II/d)",
    "PENATA MUDA (III/A)" to "Penata Muda (Gol. III/a)",
    "PENATA MUDA TINGKAT I (III/B)" to "Penata Muda Tingkat I (Gol. III/b)",
    "PENATA (III/C)" to "Penata (Gol. III/c)",
    "PENATA TINGKAT I (III/D)" to "Penata Tingkat I (Gol. III/d)",
    "PEMBINA (IV/A)" to "Pembina (Gol. IV/a)",
    "PEMBINA TINGKAT I (IV/B)" to "Pembina Tingkat I (Gol. IV/b)",
    "PEMBINA UTAMA MUDA (IV/C)" to "Pembina Utama Muda (Gol. IV/c)",
    "PEMBINA UTAMA MADYA (IV/D)" to "Pembina Utama Madya (Gol. IV/d)",
    "PEMBINA UTAMA (IV/E)" to "Pembina Utama (Gol. IV/e)"
)

val PANGKAT_SEMUA: List<Pair<String, String>> = PANGKAT_POLRI + PANGKAT_ASN

fun labelPangkat(pangkat: String): String =
    PANGKAT_SEMUA.firstOrNull { it.first == pangkat }?.second ?: pangkat

fun labelStatusHasil(status: String): String = when (status) {
    "draft" -> "Draft"
    "diajukan" -> "Menunggu Verifikasi"
    "terverifikasi" -> "Terverifikasi"
    "perlu_revisi" -> "Perlu Revisi"
    else -> status
}

fun labelKategori(kategori: String): String = when (kategori) {
    "sangat_baik" -> "Sangat Baik"
    "baik" -> "Baik"
    "cukup" -> "Cukup"
    "perlu_pembinaan" -> "Perlu Pembinaan"
    else -> kategori
}

fun warnaKategori(kategori: String): Color = when (kategori) {
    "sangat_baik" -> Warna.Hijau
    "baik" -> Warna.Biru
    "cukup" -> Warna.Kuning
    else -> Warna.Merah
}

fun warnaStatusKehadiran(status: String): Color = when (status) {
    "dinas_kantor", "piket", "dinas_luar", "ph" -> Warna.Hijau
    "tanpa_keterangan" -> Warna.Merah
    "libur_nasional" -> Warna.TeksLembut
    else -> Warna.Kuning
}

fun ringkasAbsensi(item: Absensi): String =
    "${item.jam} · ${labelStatusKehadiran(item.statusKehadiran)}"
