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
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
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
import id.polri.poldabali.kinerja.data.Rencana
import id.polri.poldabali.kinerja.data.Lampiran
import id.polri.poldabali.kinerja.ui.theme.Warna

private val PILIHAN_KEPUTUSAN = listOf(
    Triple("disetujui", "Setujui", "Rencana menjadi aktif dan dapat dilaksanakan."),
    Triple("perlu_revisi", "Minta Revisi", "Personel harus memperbaiki rencana sesuai catatan."),
    Triple("ditolak", "Tolak", "Rencana tidak dilanjutkan.")
)

@Composable
fun PersetujuanScreen(vm: KinerjaViewModel, onKembali: () -> Unit) {
    var dipilih by remember { mutableStateOf<Rencana?>(null) }
    var detailId by remember { mutableStateOf<String?>(null) }
    var detail by remember { mutableStateOf<Rencana?>(null) }
    var lampiranDilihat by remember { mutableStateOf<Lampiran?>(null) }

    LaunchedEffect(vm.pengguna?.id) { vm.muatPersetujuan() }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        TextButton(onClick = onKembali) {
            Text("‹ Kembali ke Rencana", color = Warna.Navy)
        }
        Kartu {
            JudulBagian(
                "Persetujuan Rencana",
                "${vm.persetujuan.size} dokumen menunggu keputusan"
            )
            Spacer(Modifier.height(6.dp))
            Text(
                "Tinjau rincian kegiatan sebelum menyetujui, meminta revisi, atau menolak.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
        vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses) }

        if (vm.persetujuan.isEmpty()) {
            Kartu {
                LayarKosong(
                    if (vm.sedangMemproses) "Memuat…" else "Tidak ada rencana menunggu persetujuan."
                )
            }
        } else {
            vm.persetujuan.forEach { item ->
                Kartu {
                    Row(
                        Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.Top
                    ) {
                        Column(Modifier.weight(1f)) {
                            Text(
                                item.kegiatan,
                                style = MaterialTheme.typography.titleMedium,
                                color = Warna.Teks
                            )
                            Text(
                                item.program,
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                            Text(
                                "${item.personelNama ?: "Personel"} · ${item.personelNrp ?: "-"}",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.Gold
                            )
                        }
                        Lencana("Diajukan", Warna.Biru)
                    }

                    Spacer(Modifier.height(8.dp))
                    HorizontalDivider(color = Warna.NavySoft)
                    Spacer(Modifier.height(8.dp))

                    BarisInfo("Jenis", labelJenisRencana(item.jenis))
                    BarisInfo(
                        "Periode",
                        "${tanggalIndo(item.tanggalMulai)} – ${tanggalIndo(item.tanggalSelesai)}"
                    )
                    BarisInfo("Target", "${item.targetKuantitas ?: "-"} ${item.satuan ?: ""}")
                    item.targetKualitas?.let { BarisInfo("Kualitas", it) }
                    item.dasarNomor?.let { BarisInfo("Dasar tugas", it) }
                    if (item.jumlahRevisi > 0) {
                        BarisInfo("Revisi", "${item.jumlahRevisi}×", Warna.Kuning)
                    }

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
                            teks = "Putuskan",
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
                vm.muatDetailRencana(id) { detail = it }
            }
        }
        DialogDetailRencana(
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
        DialogKeputusan(item, vm) { dipilih = null }
    }

    lampiranDilihat?.let { berkas ->
        DialogLihatLampiran(vm, berkas) { lampiranDilihat = null }
    }
}

@Composable
internal fun DialogDetailRencana(
    detail: Rencana?,
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
                    "Detail Rencana Kerja",
                    style = MaterialTheme.typography.titleMedium,
                    color = Warna.Teks
                )
                Spacer(Modifier.height(8.dp))

                Column(
                    Modifier
                        .weight(1f, fill = false)
                        .verticalScroll(rememberScrollState()),
                    verticalArrangement = Arrangement.spacedBy(6.dp)
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
                            detail.kegiatan,
                            style = MaterialTheme.typography.titleSmall,
                            color = Warna.Teks,
                            fontWeight = FontWeight.SemiBold
                        )
                        Text(
                            "${detail.personelNama ?: "Personel"} · ${detail.personelNrp ?: "-"}" +
                                (detail.unit?.let { " · $it" } ?: ""),
                            style = MaterialTheme.typography.bodySmall,
                            color = Warna.TeksLembut
                        )

                        Spacer(Modifier.height(4.dp))
                        HorizontalDivider(color = Warna.NavySoft)
                        Spacer(Modifier.height(4.dp))

                        BarisInfo("Program", detail.program)
                        BarisInfo("Jenis", labelJenisRencana(detail.jenis))
                        BarisInfo("Status", labelStatusRencana(detail.status))
                        BarisInfo(
                            "Periode",
                            "${tanggalIndo(detail.tanggalMulai)} – ${tanggalIndo(detail.tanggalSelesai)}"
                        )
                        BarisInfo("Target", "${detail.targetKuantitas ?: "-"} ${detail.satuan ?: ""}")
                        detail.targetKualitas?.let { BarisInfo("Kualitas", it) }
                        detail.tujuan?.let { BarisInfo("Tujuan", it) }
                        detail.indikatorKinerja?.let { BarisInfo("Indikator", it) }
                        detail.dasarNomor?.let { BarisInfo("Dasar tugas", it) }
                        detail.dasarPerihal?.let { BarisInfo("Perihal", it) }
                        if (detail.jumlahRevisi > 0) {
                            BarisInfo("Revisi", "${detail.jumlahRevisi}×", Warna.Kuning)
                        }
                        detail.catatanAtasan?.let {
                            KotakPesan("Catatan atasan: $it", JenisPesan.Info)
                        }

                        if (detail.tahapan.isNotEmpty()) {
                            Spacer(Modifier.height(6.dp))
                            Text(
                                "Rincian Kegiatan",
                                style = MaterialTheme.typography.labelMedium,
                                color = Warna.TeksLembut,
                                fontWeight = FontWeight.SemiBold
                            )
                            detail.tahapan.sortedBy { it.urutan }.forEach { tahap ->
                                Column(Modifier.padding(vertical = 4.dp)) {
                                    Text(
                                        "${tahap.urutan}. ${tahap.kegiatan ?: tahap.uraian}",
                                        style = MaterialTheme.typography.bodyMedium,
                                        color = Warna.Teks,
                                        fontWeight = FontWeight.Medium
                                    )
                                    Text(
                                        tahap.uraian,
                                        style = MaterialTheme.typography.bodySmall,
                                        color = Warna.Teks
                                    )
                                    Text(
                                        "Target ${tahap.targetKuantitas ?: "-"} ${tahap.satuan ?: ""}" +
                                            (tahap.targetKualitas?.let { " · $it" } ?: "") +
                                            (tahap.tanggalMulai?.let {
                                                " · ${tanggalIndo(it)} – ${tanggalIndo(tahap.tanggalSelesai ?: it)}"
                                            } ?: ""),
                                        style = MaterialTheme.typography.bodySmall,
                                        color = Warna.TeksLembut
                                    )
                                }
                            }
                        }

                        Spacer(Modifier.height(6.dp))
                        Text(
                            "Lampiran",
                            style = MaterialTheme.typography.labelMedium,
                            color = Warna.TeksLembut,
                            fontWeight = FontWeight.SemiBold
                        )
                        if (detail.lampiran.isEmpty()) {
                            Text(
                                "Belum ada lampiran.",
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
                                        "• ${berkas.fileName} (${ukuranBerkas(berkas.sizeBytes)})",
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

@Composable
private fun DialogKeputusan(
    item: Rencana,
    vm: KinerjaViewModel,
    onTutup: () -> Unit
) {
    var aksi by remember { mutableStateOf("disetujui") }
    var catatan by remember { mutableStateOf("") }
    var galat by remember { mutableStateOf<String?>(null) }

    fun simpan() {
        val pesan = if (aksi == "perlu_revisi" && catatan.isBlank()) {
            "Catatan wajib diisi saat meminta revisi"
        } else {
            null
        }
        galat = pesan
        if (pesan != null) return

        vm.putuskanRencana(item.id, aksi, catatan.ifBlank { null }) { onTutup() }
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
                    "Keputusan Rencana Kerja",
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
                    Surface(
                        shape = RoundedCornerShape(10.dp),
                        color = Warna.NavySoft
                    ) {
                        Column(Modifier.padding(10.dp)) {
                            Text(
                                item.kegiatan,
                                style = MaterialTheme.typography.bodyMedium,
                                color = Warna.Teks,
                                fontWeight = FontWeight.Medium
                            )
                            Text(
                                "${item.personelNama ?: "Personel"} · ${item.personelNrp ?: "-"}",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                    }

                    PILIHAN_KEPUTUSAN.forEach { (nilai, label, keterangan) ->
                        Row(
                            Modifier
                                .fillMaxWidth()
                                .clickable { aksi = nilai }
                                .padding(vertical = 4.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            RadioButton(selected = aksi == nilai, onClick = { aksi = nilai })
                            Column(Modifier.padding(start = 4.dp)) {
                                Text(
                                    label,
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = Warna.Teks,
                                    fontWeight = if (aksi == nilai) {
                                        FontWeight.SemiBold
                                    } else {
                                        FontWeight.Normal
                                    }
                                )
                                Text(
                                    keterangan,
                                    style = MaterialTheme.typography.bodySmall,
                                    color = Warna.TeksLembut
                                )
                            }
                        }
                    }

                    OutlinedTextField(
                        value = catatan,
                        onValueChange = { catatan = it },
                        label = {
                            Text(
                                if (aksi == "perlu_revisi") {
                                    "Catatan atasan (wajib)"
                                } else {
                                    "Catatan atasan (opsional)"
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

private fun ukuranBerkas(bytes: Long): String {
    val kb = bytes / 1024
    return if (kb < 1) "<1 KB" else "$kb KB"
}
