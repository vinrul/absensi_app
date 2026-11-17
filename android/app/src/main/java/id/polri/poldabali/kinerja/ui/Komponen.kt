package id.polri.poldabali.kinerja.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import id.polri.poldabali.kinerja.data.Absensi
import id.polri.poldabali.kinerja.ui.theme.Warna
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.util.Locale

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
fun JudulBagian(teks: String, keterangan: String? = null) {
    Column(Modifier.fillMaxWidth()) {
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

fun labelStatusOutput(status: String): String = when (status) {
    "menunggu_verifikasi" -> "Menunggu Verifikasi"
    "dalam_verifikasi" -> "Dalam Verifikasi"
    "disetujui" -> "Disetujui"
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
    "dinas_kantor", "piket", "dinas_luar" -> Warna.Hijau
    "tanpa_keterangan" -> Warna.Merah
    "libur_nasional" -> Warna.TeksLembut
    else -> Warna.Kuning
}

fun ringkasAbsensi(item: Absensi): String =
    "${item.jam} · ${labelStatusKehadiran(item.statusKehadiran)}"
