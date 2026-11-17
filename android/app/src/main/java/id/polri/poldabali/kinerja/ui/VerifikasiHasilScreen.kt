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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import id.polri.poldabali.kinerja.data.Hasil
import id.polri.poldabali.kinerja.data.Lampiran
import id.polri.poldabali.kinerja.ui.theme.Warna
import kotlin.math.roundToInt

@Composable
fun VerifikasiHasilScreen(vm: KinerjaViewModel, onKembali: () -> Unit) {
    var detailId by remember { mutableStateOf<String?>(null) }
    var detail by remember { mutableStateOf<Hasil?>(null) }
    var lampiranDilihat by remember { mutableStateOf<Lampiran?>(null) }
    var dipilih by remember { mutableStateOf<Hasil?>(null) }

    LaunchedEffect(vm.pengguna?.id) { vm.muatVerifikasiHasil() }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        TextButton(onClick = onKembali) {
            Text("‹ Kembali ke Hasil Kerja", color = Warna.Navy)
        }

        Kartu {
            JudulBagian(
                "Verifikasi Hasil Kerja",
                "${vm.hasilMenungguVerifikasi.size} dokumen menunggu verifikasi"
            )
            Spacer(Modifier.height(6.dp))
            Text(
                "Periksa realisasi dan bukti pendukung sebelum memverifikasi atau meminta revisi.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
        vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses) }

        if (vm.hasilMenungguVerifikasi.isEmpty()) {
            Kartu {
                LayarKosong(
                    if (vm.sedangMemproses) "Memuat…" else "Tidak ada hasil kerja menunggu verifikasi."
                )
            }
        } else {
            vm.hasilMenungguVerifikasi.forEach { item ->
                Kartu {
                    Row(
                        Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.Top
                    ) {
                        Column(Modifier.weight(1f)) {
                            Text(
                                item.rencanaKegiatan ?: "Hasil Kerja",
                                style = MaterialTheme.typography.titleMedium,
                                color = Warna.Teks
                            )
                            Text(
                                "${item.personelNama ?: "Personel"} · ${item.personelNrp ?: "-"}",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.Gold
                            )
                        }
                        Lencana("Menunggu Verifikasi", Warna.Biru)
                    }

                    Spacer(Modifier.height(8.dp))
                    HorizontalDivider(color = Warna.NavySoft)
                    Spacer(Modifier.height(8.dp))

                    BarisInfo("Tanggal realisasi", tanggalIndo(item.tanggalRealisasi))
                    BarisInfo("Progres", "${item.progresPersen.toInt()}%")
                    BarisInfo("Jumlah realisasi", item.jumlahRealisasi?.toString() ?: "-")
                    item.kendala?.let { BarisInfo("Kendala", it) }
                    Spacer(Modifier.height(4.dp))
                    Text(
                        item.uraianPelaksanaan,
                        style = MaterialTheme.typography.bodySmall,
                        color = Warna.Teks
                    )

                    Spacer(Modifier.height(10.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        TombolKedua(
                            teks = "Detail",
                            modifier = Modifier.weight(1f),
                            aktif = !vm.sedangMemproses,
                            onClick = {
                                vm.bersihkanPesan()
                                detail = null
                                detailId = item.id
                            }
                        )
                        TombolUtama(
                            teks = "Verifikasi",
                            modifier = Modifier.weight(1f),
                            memuat = vm.sedangMemproses,
                            aktif = !vm.sedangMemproses,
                            onClick = {
                                vm.bersihkanPesan()
                                dipilih = item
                            }
                        )
                    }
                }
            }
        }
    }

    if (detailId != null) {
        LaunchedEffect(detailId) {
            val id = detailId
            if (id != null) {
                vm.muatDetailHasil(id) { detail = it }
            }
        }
        DialogDetailHasil(
            detail = detail,
            galat = vm.galat,
            onLihatLampiran = { lampiranDilihat = it },
            onTutup = {
                detailId = null
                detail = null
                vm.bersihkanPesan()
            }
        )
    }

    dipilih?.let { item ->
        DialogVerifikasiHasil(item, vm) { dipilih = null }
    }

    lampiranDilihat?.let { berkas ->
        DialogLihatLampiran(vm, berkas) { lampiranDilihat = null }
    }
}

@Composable
private fun DialogVerifikasiHasil(
    item: Hasil,
    vm: KinerjaViewModel,
    onTutup: () -> Unit
) {
    var setuju by remember { mutableStateOf(true) }
    var skorKualitas by remember { mutableStateOf(85f) }
    var catatan by remember { mutableStateOf("") }
    var galat by remember { mutableStateOf<String?>(null) }

    fun simpan() {
        if (!setuju && catatan.isBlank()) {
            galat = "Catatan wajib diisi saat meminta revisi"
            return
        }
        galat = null

        vm.putuskanVerifikasiHasil(
            id = item.id,
            setuju = setuju,
            skorKualitas = if (setuju) skorKualitas.toDouble() else null,
            catatan = catatan.ifBlank { null }
        ) { onTutup() }
    }

    Dialog(onDismissRequest = onTutup) {
        Surface(shape = RoundedCornerShape(16.dp), color = Color.White) {
            Column(
                Modifier
                    .heightIn(max = 620.dp)
                    .imePadding()
                    .padding(16.dp)
            ) {
                Text(
                    "Verifikasi Hasil Kerja",
                    style = MaterialTheme.typography.titleMedium,
                    color = Warna.Teks
                )
                Spacer(Modifier.height(8.dp))

                Column(
                    Modifier
                        .weight(1f, fill = false)
                        .verticalScroll(rememberScrollState()),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    Surface(shape = RoundedCornerShape(10.dp), color = Warna.NavySoft) {
                        Column(Modifier.padding(10.dp)) {
                            Text(
                                item.rencanaKegiatan ?: "Hasil Kerja",
                                style = MaterialTheme.typography.bodyMedium,
                                color = Warna.Teks,
                                fontWeight = FontWeight.Medium
                            )
                            Text(
                                "${item.personelNama ?: "Personel"} · realisasi " +
                                    (item.jumlahRealisasi?.toString() ?: "-"),
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                    }

                    Row(
                        Modifier
                            .fillMaxWidth()
                            .clickable { setuju = true }
                            .padding(vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        RadioButton(selected = setuju, onClick = { setuju = true })
                        Column(Modifier.padding(start = 4.dp)) {
                            Text(
                                "Terverifikasi",
                                style = MaterialTheme.typography.bodyMedium,
                                color = Warna.Teks,
                                fontWeight = if (setuju) FontWeight.SemiBold else FontWeight.Normal
                            )
                            Text(
                                "Nilai dihitung otomatis dari kuantitas, kualitas, dan waktu.",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                    }
                    Row(
                        Modifier
                            .fillMaxWidth()
                            .clickable { setuju = false }
                            .padding(vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        RadioButton(selected = !setuju, onClick = { setuju = false })
                        Column(Modifier.padding(start = 4.dp)) {
                            Text(
                                "Perlu Revisi",
                                style = MaterialTheme.typography.bodyMedium,
                                color = Warna.Teks,
                                fontWeight = if (!setuju) FontWeight.SemiBold else FontWeight.Normal
                            )
                            Text(
                                "Personel harus memperbaiki realisasi atau bukti sesuai catatan.",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                    }

                    if (setuju) {
                        Text(
                            "Skor kualitas (0–100)",
                            style = MaterialTheme.typography.labelLarge,
                            color = Warna.TeksLembut
                        )
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Slider(
                                value = skorKualitas,
                                onValueChange = { skorKualitas = it.roundToInt().toFloat() },
                                valueRange = 0f..100f,
                                colors = SliderDefaults.colors(
                                    thumbColor = Warna.Navy,
                                    activeTrackColor = Warna.Navy,
                                    inactiveTrackColor = Warna.NavySoft
                                ),
                                modifier = Modifier.weight(1f)
                            )
                            Spacer(Modifier.width(12.dp))
                            Text(
                                "${skorKualitas.roundToInt()}",
                                style = MaterialTheme.typography.titleMedium,
                                fontWeight = FontWeight.SemiBold,
                                color = Warna.Navy
                            )
                        }
                    }

                    OutlinedTextField(
                        value = catatan,
                        onValueChange = { catatan = it },
                        label = {
                            Text(
                                if (setuju) {
                                    "Catatan verifikator (opsional)"
                                } else {
                                    "Catatan verifikator (wajib)"
                                }
                            )
                        },
                        minLines = 3,
                        modifier = Modifier.fillMaxWidth()
                    )

                    galat?.let { KotakPesan(it, JenisPesan.Galat) }
                    vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
                }

                Spacer(Modifier.height(10.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    TombolKedua(
                        teks = "Batal",
                        modifier = Modifier.weight(1f),
                        onClick = onTutup
                    )
                    TombolUtama(
                        teks = "Simpan",
                        modifier = Modifier.weight(1f),
                        memuat = vm.sedangMemproses,
                        aktif = !vm.sedangMemproses,
                        onClick = { simpan() }
                    )
                }
            }
        }
    }
}
