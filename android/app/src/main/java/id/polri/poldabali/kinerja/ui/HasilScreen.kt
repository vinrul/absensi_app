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
import androidx.compose.material3.FilterChip
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
import id.polri.poldabali.kinerja.data.HasilRequest
import id.polri.poldabali.kinerja.ui.theme.Warna

@Composable
fun HasilScreen(vm: KinerjaViewModel) {
    var bukaForm by remember { mutableStateOf(false) }

    LaunchedEffect(vm.pengguna?.id) {
        vm.muatHasil()
        if (vm.rencana.isEmpty()) vm.muatRencana()
    }

    if (bukaForm) {
        FormHasil(vm) { bukaForm = false }
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
                JudulBagian("Hasil Kerja", "${vm.hasil.size} laporan realisasi")
                TextButton(onClick = { bukaForm = true }) { Text("+ Baru", color = Warna.Navy) }
            }
            Spacer(Modifier.height(6.dp))
            Text(
                "Hasil kerja hanya dapat diisi dari rencana yang sudah disetujui atasan.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
        vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses) }

        if (vm.hasil.isEmpty()) {
            Kartu { LayarKosong(if (vm.sedangMemproses) "Memuat…" else "Belum ada hasil kerja.") }
        } else {
            vm.hasil.forEach { item ->
                Kartu {
                    Row(
                        Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.Top
                    ) {
                        Column(Modifier.weight(1f)) {
                            Text(
                                item.rencanaKegiatan ?: "Realisasi",
                                style = MaterialTheme.typography.titleMedium,
                                color = Warna.Teks
                            )
                            Text(
                                tanggalIndo(item.tanggalRealisasi),
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                        Lencana(
                            when (item.status) {
                                "terverifikasi" -> "Terverifikasi"
                                "perlu_revisi" -> "Perlu Revisi"
                                else -> "Menunggu Verifikasi"
                            },
                            when (item.status) {
                                "terverifikasi" -> Warna.Hijau
                                "perlu_revisi" -> Warna.Kuning
                                else -> Warna.Biru
                            }
                        )
                    }

                    Spacer(Modifier.height(8.dp))
                    BarisInfo("Progres", "${item.progresPersen.toInt()}%")
                    BarisInfo("Jumlah realisasi", item.jumlahRealisasi?.toString() ?: "-")
                    item.skorTotal?.let {
                        BarisInfo(
                            "Nilai hasil kerja",
                            String.format(java.util.Locale.US, "%.2f", it),
                            Warna.Hijau
                        )
                    }
                    item.kendala?.let { BarisInfo("Kendala", it) }
                    item.catatanVerifikator?.let { BarisInfo("Catatan atasan", it, Warna.Kuning) }
                }
            }
        }
    }
}

@Composable
private fun FormHasil(vm: KinerjaViewModel, onTutup: () -> Unit) {
    val daftarRencana = vm.rencanaDisetujui()
    var rencanaId by remember { mutableStateOf(daftarRencana.firstOrNull()?.id ?: "") }
    var uraian by remember { mutableStateOf("") }
    var progres by remember { mutableStateOf("100") }
    var realisasi by remember { mutableStateOf("") }
    var kendala by remember { mutableStateOf("") }
    var tanggal by remember { mutableStateOf(tanggalHariIni()) }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        Kartu {
            JudulBagian("Lapor Hasil Kerja", "Pilih rencana yang sudah disetujui")
            Spacer(Modifier.height(10.dp))

            if (daftarRencana.isEmpty()) {
                KotakPesan("Belum ada rencana kerja yang disetujui atasan.", JenisPesan.Info)
            } else {
                Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    daftarRencana.take(8).forEach { rencana ->
                        FilterChip(
                            selected = rencanaId == rencana.id,
                            onClick = { rencanaId = rencana.id },
                            label = { Text(rencana.kegiatan) }
                        )
                    }
                }

                Spacer(Modifier.height(10.dp))
                OutlinedTextField(
                    value = uraian,
                    onValueChange = { uraian = it },
                    label = { Text("Uraian pelaksanaan") },
                    minLines = 3,
                    modifier = Modifier.fillMaxWidth()
                )
                Spacer(Modifier.height(8.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    OutlinedTextField(
                        value = progres,
                        onValueChange = { progres = it.filter { c -> c.isDigit() } },
                        label = { Text("Progres %") },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                        modifier = Modifier.weight(1f)
                    )
                    OutlinedTextField(
                        value = realisasi,
                        onValueChange = { realisasi = it.filter { c -> c.isDigit() || c == '.' } },
                        label = { Text("Realisasi") },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                        modifier = Modifier.weight(1f)
                    )
                }
                Spacer(Modifier.height(8.dp))
                OutlinedTextField(
                    value = kendala,
                    onValueChange = { kendala = it },
                    label = { Text("Kendala (opsional)") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
                Spacer(Modifier.height(8.dp))
                OutlinedTextField(
                    value = tanggal,
                    onValueChange = { tanggal = it },
                    label = { Text("Tanggal realisasi") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
            }
        }

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }

        TombolUtama(
            teks = "Kirim Hasil Kerja",
            memuat = vm.sedangMemproses,
            aktif = rencanaId.isNotBlank() && uraian.isNotBlank(),
            onClick = {
                vm.buatHasil(
                    HasilRequest(
                        rencanaId = rencanaId,
                        uraianPelaksanaan = uraian,
                        progresPersen = progres.toDoubleOrNull() ?: 100.0,
                        jumlahRealisasi = realisasi.toDoubleOrNull(),
                        kendala = kendala.ifBlank { null },
                        tanggalRealisasi = tanggal
                    )
                ) { onTutup() }
            }
        )

        TombolKedua(teks = "Batal", onClick = onTutup)
        Spacer(Modifier.height(8.dp))
    }
}
