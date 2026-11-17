package id.polri.poldabali.kinerja.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
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
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import id.polri.poldabali.kinerja.data.RencanaRequest
import id.polri.poldabali.kinerja.data.TahapanRequest
import id.polri.poldabali.kinerja.ui.theme.Warna

@Composable
fun RencanaScreen(vm: KinerjaViewModel) {
    var bukaForm by remember { mutableStateOf(false) }

    LaunchedEffect(vm.pengguna?.id) { vm.muatRencana() }

    if (bukaForm) {
        FormRencana(vm) { bukaForm = false }
        return
    }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Kartu {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                JudulBagian("Rencana Kerja", "${vm.rencana.size} dokumen · ${vm.periode?.nama ?: ""}")
                TextButton(onClick = { bukaForm = true }) {
                    Text("+ Baru", color = Warna.Navy)
                }
            }
            Spacer(Modifier.height(6.dp))
            Text(
                "Ajukan rencana sebelum periode berjalan agar dinilai tepat waktu.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
        vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses) }

        if (vm.rencana.isEmpty()) {
            Kartu { LayarKosong(if (vm.sedangMemproses) "Memuat…" else "Belum ada rencana kerja.") }
        } else {
            vm.rencana.forEach { item ->
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
                        }
                        Lencana(
                            labelStatusRencana(item.status),
                            when (item.status) {
                                "disetujui" -> Warna.Hijau
                                "perlu_revisi" -> Warna.Kuning
                                "ditolak" -> Warna.Merah
                                else -> Warna.Biru
                            }
                        )
                    }

                    Spacer(Modifier.height(8.dp))
                    HorizontalDivider(color = Warna.NavySoft)
                    Spacer(Modifier.height(8.dp))

                    BarisInfo("Periode", "${tanggalIndo(item.tanggalMulai)} – ${tanggalIndo(item.tanggalSelesai)}")
                    BarisInfo("Target", "${item.targetKuantitas ?: "-"} ${item.satuan ?: ""}")
                    item.dasarNomor?.let { BarisInfo("Dasar tugas", it) }
                    if (item.jumlahRevisi > 0) {
                        BarisInfo("Revisi", "${item.jumlahRevisi}×", Warna.Kuning)
                    }
                    item.catatanAtasan?.let {
                        Spacer(Modifier.height(6.dp))
                        KotakPesan("Catatan atasan: $it", JenisPesan.Info)
                    }

                    if (item.status == "draft" || item.status == "perlu_revisi") {
                        Spacer(Modifier.height(10.dp))
                        TombolKedua(
                            teks = "Ajukan ke Atasan",
                            aktif = !vm.sedangMemproses,
                            onClick = { vm.ajukanRencana(item.id) }
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun FormRencana(vm: KinerjaViewModel, onTutup: () -> Unit) {
    var program by remember { mutableStateOf("Penegakan Hukum") }
    var kegiatan by remember { mutableStateOf("") }
    var uraian by remember { mutableStateOf("") }
    var tujuan by remember { mutableStateOf("") }
    var indikator by remember { mutableStateOf("") }
    var dasarNomor by remember { mutableStateOf("") }
    var target by remember { mutableStateOf("") }
    var satuan by remember { mutableStateOf("laporan") }
    var mulai by remember { mutableStateOf(tanggalHariIni()) }
    var selesai by remember { mutableStateOf(tanggalHariIni()) }
    var tahapan by remember { mutableStateOf("") }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        Kartu {
            JudulBagian("Rencana Kerja Baru", "Isi target dan dasar tugas")
            Spacer(Modifier.height(10.dp))

            OutlinedTextField(
                value = program,
                onValueChange = { program = it },
                label = { Text("Program") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = kegiatan,
                onValueChange = { kegiatan = it },
                label = { Text("Kegiatan Utama") },
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = uraian,
                onValueChange = { uraian = it },
                label = { Text("Uraian Tugas") },
                minLines = 3,
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = tujuan,
                onValueChange = { tujuan = it },
                label = { Text("Tujuan (opsional)") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = indikator,
                onValueChange = { indikator = it },
                label = { Text("Indikator Kinerja (opsional)") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = dasarNomor,
                onValueChange = { dasarNomor = it },
                label = { Text("Nomor SP / TOR") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(Modifier.height(8.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(
                    value = target,
                    onValueChange = { target = it.filter { c -> c.isDigit() || c == '.' } },
                    label = { Text("Target") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                    modifier = Modifier.weight(1f)
                )
                OutlinedTextField(
                    value = satuan,
                    onValueChange = { satuan = it },
                    label = { Text("Satuan") },
                    singleLine = true,
                    modifier = Modifier.weight(1f)
                )
            }
            Spacer(Modifier.height(8.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(
                    value = mulai,
                    onValueChange = { mulai = it },
                    label = { Text("Mulai") },
                    singleLine = true,
                    modifier = Modifier.weight(1f)
                )
                OutlinedTextField(
                    value = selesai,
                    onValueChange = { selesai = it },
                    label = { Text("Selesai") },
                    singleLine = true,
                    modifier = Modifier.weight(1f)
                )
            }
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = tahapan,
                onValueChange = { tahapan = it },
                label = { Text("Tahapan (satu per baris, opsional)") },
                minLines = 2,
                modifier = Modifier.fillMaxWidth()
            )
        }

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }

        TombolUtama(
            teks = "Simpan & Ajukan",
            memuat = vm.sedangMemproses,
            aktif = kegiatan.isNotBlank() && uraian.isNotBlank() && mulai.isNotBlank() && selesai.isNotBlank(),
            onClick = {
                val daftarTahapan = tahapan.split("\n")
                    .map { it.trim() }
                    .filter { it.isNotEmpty() }
                    .map { TahapanRequest(uraian = it) }

                vm.buatRencana(
                    RencanaRequest(
                        periodeId = vm.periode?.id,
                        program = program,
                        kegiatan = kegiatan,
                        uraian = uraian,
                        tujuan = tujuan.ifBlank { null },
                        indikatorKinerja = indikator.ifBlank { null },
                        dasarJenis = if (dasarNomor.isBlank()) null else "sp",
                        dasarNomor = dasarNomor.ifBlank { null },
                        targetKuantitas = target.toDoubleOrNull(),
                        satuan = satuan.ifBlank { null },
                        targetWaktu = selesai,
                        tanggalMulai = mulai,
                        tanggalSelesai = selesai,
                        tahapan = daftarTahapan,
                        ajukan = true
                    )
                ) { onTutup() }
            }
        )

        TombolKedua(teks = "Batal", onClick = onTutup)
        Spacer(Modifier.height(8.dp))
    }
}
