package id.polri.poldabali.kinerja.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import id.polri.poldabali.kinerja.data.Hasil
import id.polri.poldabali.kinerja.data.Lampiran
import id.polri.poldabali.kinerja.ui.theme.Warna
import java.util.Locale

@Composable
internal fun DialogDetailHasil(
    detail: Hasil?,
    galat: String?,
    onLihatLampiran: (Lampiran) -> Unit,
    onTutup: () -> Unit
) {
    Dialog(onDismissRequest = onTutup) {
        Surface(shape = RoundedCornerShape(16.dp), color = Color.White) {
            Column(
                Modifier
                    .heightIn(max = 620.dp)
                    .imePadding()
                    .padding(16.dp)
            ) {
                Text(
                    "Detail Hasil Kerja",
                    style = MaterialTheme.typography.titleMedium,
                    color = Warna.Teks
                )
                Spacer(Modifier.height(8.dp))

                Column(
                    Modifier
                        .weight(1f, fill = false)
                        .verticalScroll(rememberScrollState()),
                    verticalArrangement = Arrangement.spacedBy(5.dp)
                ) {
                    if (detail == null) {
                        if (galat != null) {
                            KotakPesan(galat, JenisPesan.Galat)
                        } else {
                            Row(
                                Modifier
                                    .fillMaxWidth()
                                    .padding(vertical = 16.dp),
                                horizontalArrangement = Arrangement.Center,
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                CircularProgressIndicator(
                                    modifier = Modifier.size(20.dp),
                                    strokeWidth = 2.dp,
                                    color = Warna.Navy
                                )
                                Spacer(Modifier.size(10.dp))
                                Text(
                                    "Memuat detail…",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = Warna.TeksLembut
                                )
                            }
                        }
                    } else {
                        Text(
                            detail.rencanaKegiatan ?: "Hasil Kerja",
                            style = MaterialTheme.typography.titleSmall,
                            color = Warna.Teks,
                            fontWeight = FontWeight.SemiBold
                        )
                        detail.personelNama?.let { nama ->
                            Text(
                                "$nama · ${detail.personelNrp ?: "-"}",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }

                        Spacer(Modifier.height(4.dp))
                        HorizontalDivider(color = Warna.NavySoft)
                        Spacer(Modifier.height(4.dp))

                        BarisInfo("Status", labelStatusHasil(detail.status))
                        BarisInfo("Tanggal realisasi", tanggalIndo(detail.tanggalRealisasi))
                        BarisInfo("Progres", "${detail.progresPersen.toInt()}%")
                        BarisInfo("Jumlah realisasi", detail.jumlahRealisasi?.toString() ?: "-")
                        detail.kendala?.let { BarisInfo("Kendala", it) }

                        Spacer(Modifier.height(4.dp))
                        Text(
                            "Uraian Pelaksanaan",
                            style = MaterialTheme.typography.labelMedium,
                            color = Warna.TeksLembut,
                            fontWeight = FontWeight.SemiBold
                        )
                        Text(
                            detail.uraianPelaksanaan,
                            style = MaterialTheme.typography.bodySmall,
                            color = Warna.Teks
                        )

                        if (detail.skorTotal != null) {
                            Spacer(Modifier.height(4.dp))
                            Text(
                                "Penilaian",
                                style = MaterialTheme.typography.labelMedium,
                                color = Warna.TeksLembut,
                                fontWeight = FontWeight.SemiBold
                            )
                            detail.skorKuantitas?.let { BarisInfo("Kuantitas", angkaDua(it)) }
                            detail.skorKualitas?.let { BarisInfo("Kualitas", angkaDua(it)) }
                            detail.skorWaktu?.let { BarisInfo("Ketepatan waktu", angkaDua(it)) }
                            BarisInfo("Nilai total", angkaDua(detail.skorTotal), Warna.Hijau)
                        }

                        detail.catatanVerifikator?.let {
                            KotakPesan("Catatan atasan: $it", JenisPesan.Info)
                        }

                        Spacer(Modifier.height(4.dp))
                        Text(
                            "Bukti Pendukung",
                            style = MaterialTheme.typography.labelMedium,
                            color = Warna.TeksLembut,
                            fontWeight = FontWeight.SemiBold
                        )
                        if (detail.lampiran.isEmpty()) {
                            Text(
                                "Belum ada bukti pendukung.",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        } else {
                            detail.lampiran.forEach { berkas ->
                                Row(
                                    Modifier
                                        .fillMaxWidth()
                                        .clickable { onLihatLampiran(berkas) }
                                        .padding(vertical = 4.dp),
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                                    Text(
                                        "• ${berkas.fileName} (${ukuranBerkasDetail(berkas.sizeBytes)})",
                                        style = MaterialTheme.typography.bodySmall,
                                        color = Warna.Teks,
                                        modifier = Modifier.weight(1f)
                                    )
                                    Text(
                                        "Lihat",
                                        style = MaterialTheme.typography.labelMedium,
                                        color = Warna.Navy,
                                        fontWeight = FontWeight.SemiBold
                                    )
                                }
                            }
                        }
                    }
                }

                Spacer(Modifier.height(10.dp))
                TombolKedua(teks = "Tutup", onClick = onTutup)
            }
        }
    }
}

private fun angkaDua(nilai: Double): String = String.format(Locale.US, "%.2f", nilai)

private fun ukuranBerkasDetail(bytes: Long): String {
    val kb = bytes / 1024
    return if (kb < 1) "<1 KB" else "$kb KB"
}
