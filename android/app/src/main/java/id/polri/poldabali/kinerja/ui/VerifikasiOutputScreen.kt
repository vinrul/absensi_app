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
import id.polri.poldabali.kinerja.data.Lampiran
import id.polri.poldabali.kinerja.data.Output
import id.polri.poldabali.kinerja.ui.theme.Warna
import kotlin.math.roundToInt

@Composable
fun VerifikasiOutputScreen(vm: KinerjaViewModel, onKembali: () -> Unit) {
    var detailId by remember { mutableStateOf<String?>(null) }
    var detail by remember { mutableStateOf<Output?>(null) }
    var lampiranDilihat by remember { mutableStateOf<Lampiran?>(null) }
    var dipilih by remember { mutableStateOf<Output?>(null) }

    LaunchedEffect(vm.pengguna?.id) { vm.muatVerifikasiOutput() }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        TextButton(onClick = onKembali) {
            Text("‹ Kembali ke Output", color = Warna.Navy)
        }

        Kartu {
            JudulBagian(
                "Verifikasi Output",
                "${vm.outputMenungguVerifikasi.size} output menunggu verifikasi"
            )
            Spacer(Modifier.height(6.dp))
            Text(
                "Nilai kelengkapan, kesesuaian, dan keabsahan bukti sebelum menyetujui atau meminta revisi.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
        vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses) }

        if (vm.outputMenungguVerifikasi.isEmpty()) {
            Kartu {
                LayarKosong(
                    if (vm.sedangMemproses) "Memuat…" else "Tidak ada output menunggu verifikasi."
                )
            }
        } else {
            vm.outputMenungguVerifikasi.forEach { item ->
                Kartu {
                    Row(
                        Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.Top
                    ) {
                        Column(Modifier.weight(1f)) {
                            Text(
                                item.judul,
                                style = MaterialTheme.typography.titleMedium,
                                color = Warna.Teks
                            )
                            Text(
                                "${item.personelNama ?: "Personel"} · ${item.personelNrp ?: "-"}",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.Gold
                            )
                        }
                        Lencana("Dalam Verifikasi", Warna.Biru)
                    }

                    Spacer(Modifier.height(8.dp))
                    HorizontalDivider(color = Warna.NavySoft)
                    Spacer(Modifier.height(8.dp))

                    BarisInfo("Jenis", labelJenisOutput(item.jenisOutput))
                    BarisInfo("Tanggal hasil", tanggalIndo(item.tanggalHasil))
                    item.rencanaKegiatan?.let { BarisInfo("Rencana kerja", it) }
                    item.uraian?.let {
                        Spacer(Modifier.height(4.dp))
                        Text(it, style = MaterialTheme.typography.bodySmall, color = Warna.Teks)
                    }
                    item.urlEksternal?.let { BarisInfo("Tautan", it) }

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
                vm.muatDetailOutput(id) { detail = it }
            }
        }
        DialogDetailOutput(
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
        DialogVerifikasiOutput(item, vm) { dipilih = null }
    }

    lampiranDilihat?.let { berkas ->
        DialogLihatLampiran(vm, berkas) { lampiranDilihat = null }
    }
}

@Composable
private fun SliderSkorOutput(
    label: String,
    nilai: Float,
    onNilai: (Float) -> Unit
) {
    Text(
        label,
        style = MaterialTheme.typography.labelLarge,
        color = Warna.TeksLembut
    )
    Row(verticalAlignment = Alignment.CenterVertically) {
        Slider(
            value = nilai,
            onValueChange = { onNilai(it.roundToInt().toFloat()) },
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
            "${nilai.roundToInt()}",
            style = MaterialTheme.typography.titleMedium,
            fontWeight = FontWeight.SemiBold,
            color = Warna.Navy
        )
    }
}

@Composable
private fun DialogVerifikasiOutput(
    item: Output,
    vm: KinerjaViewModel,
    onTutup: () -> Unit
) {
    var setuju by remember { mutableStateOf(true) }
    var skorKesesuaian by remember { mutableStateOf(90f) }
    var skorKeabsahan by remember { mutableStateOf(92f) }
    var catatan by remember { mutableStateOf("") }
    var galat by remember { mutableStateOf<String?>(null) }

    fun simpan() {
        if (!setuju && catatan.isBlank()) {
            galat = "Catatan wajib diisi saat meminta revisi"
            return
        }
        galat = null

        vm.putuskanVerifikasiOutput(
            id = item.id,
            setuju = setuju,
            skorKesesuaian = if (setuju) skorKesesuaian.toDouble() else null,
            skorKeabsahan = if (setuju) skorKeabsahan.toDouble() else null,
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
                    "Verifikasi Output",
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
                                item.judul,
                                style = MaterialTheme.typography.bodyMedium,
                                color = Warna.Teks,
                                fontWeight = FontWeight.Medium
                            )
                            Text(
                                "${labelJenisOutput(item.jenisOutput)} · " +
                                    (item.personelNama ?: "Personel"),
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                    }

                    Row(
                        Modifier
                            .fillMaxWidth()
                            .clickable {
                                setuju = true
                                galat = null
                            }
                            .padding(vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        RadioButton(selected = setuju, onClick = {
                            setuju = true
                            galat = null
                        })
                        Column(Modifier.padding(start = 4.dp)) {
                            Text(
                                "Disetujui (Valid)",
                                style = MaterialTheme.typography.bodyMedium,
                                color = Warna.Teks,
                                fontWeight = if (setuju) FontWeight.SemiBold else FontWeight.Normal
                            )
                            Text(
                                "Output lengkap dan sah sebagai bukti capaian kinerja.",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                    }
                    Row(
                        Modifier
                            .fillMaxWidth()
                            .clickable {
                                setuju = false
                                galat = null
                            }
                            .padding(vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        RadioButton(selected = !setuju, onClick = {
                            setuju = false
                            galat = null
                        })
                        Column(Modifier.padding(start = 4.dp)) {
                            Text(
                                "Kembalikan dan Revisi",
                                style = MaterialTheme.typography.bodyMedium,
                                color = Warna.Teks,
                                fontWeight = if (!setuju) FontWeight.SemiBold else FontWeight.Normal
                            )
                            Text(
                                "Personel harus melengkapi atau memperbaiki output sesuai catatan.",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                    }

                    if (setuju) {
                        SliderSkorOutput(
                            label = "Skor kesesuaian (0–100)",
                            nilai = skorKesesuaian,
                            onNilai = { skorKesesuaian = it }
                        )
                        SliderSkorOutput(
                            label = "Skor keabsahan (0–100)",
                            nilai = skorKeabsahan,
                            onNilai = { skorKeabsahan = it }
                        )
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
