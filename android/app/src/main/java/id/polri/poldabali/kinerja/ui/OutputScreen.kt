package id.polri.poldabali.kinerja.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
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
import androidx.compose.ui.unit.dp
import id.polri.poldabali.kinerja.data.Lampiran
import id.polri.poldabali.kinerja.data.Output
import id.polri.poldabali.kinerja.ui.theme.Warna
import java.util.Locale

@Composable
fun OutputScreen(
    vm: KinerjaViewModel,
    mintaBukaVerifikasi: Boolean = false,
    onVerifikasiDibuka: () -> Unit = {}
) {
    var bukaForm by remember { mutableStateOf(false) }
    var idSunting by remember { mutableStateOf<String?>(null) }
    var bukaVerifikasi by remember { mutableStateOf(false) }
    var detailId by remember { mutableStateOf<String?>(null) }
    var detail by remember { mutableStateOf<Output?>(null) }
    var lampiranDilihat by remember { mutableStateOf<Lampiran?>(null) }

    LaunchedEffect(vm.pengguna?.id) {
        vm.muatOutput()
        if (vm.hasil.isEmpty()) vm.muatHasil()
        if (vm.bisaMemverifikasiOutput) vm.muatVerifikasiOutput()
    }

    LaunchedEffect(mintaBukaVerifikasi) {
        if (mintaBukaVerifikasi) {
            bukaVerifikasi = true
            onVerifikasiDibuka()
        }
    }

    if (bukaForm) {
        WizardOutput(vm, idOutput = idSunting) {
            bukaForm = false
            idSunting = null
        }
        return
    }

    if (bukaVerifikasi) {
        VerifikasiOutputScreen(vm) { bukaVerifikasi = false }
        return
    }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        if (vm.bisaMemverifikasiOutput) {
            Kartu {
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.Top
                ) {
                    Column(Modifier.weight(1f)) {
                        Text(
                            "Verifikasi Output",
                            style = MaterialTheme.typography.titleMedium,
                            color = Warna.Teks
                        )
                        Text(
                            "Output bawahan langsung yang menunggu verifikasi.",
                            style = MaterialTheme.typography.bodySmall,
                            color = Warna.TeksLembut
                        )
                    }
                    Lencana(
                        "${vm.outputMenungguVerifikasi.size}",
                        if (vm.outputMenungguVerifikasi.isEmpty()) Warna.TeksLembut else Warna.Kuning
                    )
                }
                Spacer(Modifier.height(8.dp))
                Text(
                    if (vm.outputMenungguVerifikasi.isEmpty()) {
                        "Tidak ada output yang menunggu verifikasi."
                    } else {
                        "${vm.outputMenungguVerifikasi.size} output menunggu verifikasi Anda."
                    },
                    style = MaterialTheme.typography.bodySmall,
                    color = Warna.Teks
                )
                Spacer(Modifier.height(10.dp))
                TombolUtama(
                    teks = if (vm.outputMenungguVerifikasi.isEmpty()) {
                        "Buka Verifikasi Output"
                    } else {
                        "Tinjau Verifikasi (${vm.outputMenungguVerifikasi.size})"
                    },
                    aktif = !vm.sedangMemproses,
                    onClick = {
                        vm.bersihkanPesan()
                        bukaVerifikasi = true
                    }
                )
            }
        }

        Kartu {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                JudulBagian(
                    "Output",
                    "${vm.output.size} bukti hasil kerja",
                    Modifier.weight(1f)
                )
                TextButton(onClick = {
                    vm.bersihkanPesan()
                    idSunting = null
                    bukaForm = true
                }) {
                    Text("+ Baru", color = Warna.Navy)
                }
            }
            Spacer(Modifier.height(6.dp))
            Text(
                "Output hanya dapat dibuat dari hasil kerja yang sudah diverifikasi atasan.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
        vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses) }

        if (vm.output.isEmpty()) {
            Kartu { LayarKosong(if (vm.sedangMemproses) "Memuat…" else "Belum ada output.") }
        } else {
            vm.output.forEach { item ->
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
                                "${labelJenisOutput(item.jenisOutput)} · ${tanggalIndo(item.tanggalHasil)}",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                        Lencana(
                            labelStatusOutput(item.status),
                            when (item.status) {
                                "disetujui" -> Warna.Hijau
                                "perlu_revisi" -> Warna.Kuning
                                "draft" -> Warna.TeksLembut
                                else -> Warna.Biru
                            }
                        )
                    }

                    Spacer(Modifier.height(8.dp))
                    HorizontalDivider(color = Warna.NavySoft)
                    Spacer(Modifier.height(8.dp))

                    item.rencanaKegiatan?.let { BarisInfo("Rencana kerja", it) }
                    item.uraian?.let { BarisInfo("Uraian", it) }
                    item.urlEksternal?.let { BarisInfo("Tautan", it) }
                    item.skorTotal?.let {
                        BarisInfo(
                            "Nilai output",
                            String.format(Locale.US, "%.2f", it),
                            Warna.Hijau
                        )
                    }
                    item.catatanVerifikator?.let { BarisInfo("Catatan atasan", it, Warna.Kuning) }

                    Spacer(Modifier.height(10.dp))
                    val bisaUbah = item.status == "draft" || item.status == "perlu_revisi"
                    val bisaKirim = bisaUbah || item.status == "menunggu_verifikasi"

                    if (bisaUbah) {
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            TombolKedua(
                                teks = "Lihat Detail",
                                modifier = Modifier.weight(1f),
                                aktif = !vm.sedangMemproses,
                                onClick = {
                                    vm.bersihkanPesan()
                                    detail = null
                                    detailId = item.id
                                }
                            )
                            TombolKedua(
                                teks = if (item.status == "perlu_revisi") "Perbaiki" else "Ubah",
                                modifier = Modifier.weight(1f),
                                aktif = !vm.sedangMemproses,
                                onClick = {
                                    vm.bersihkanPesan()
                                    idSunting = item.id
                                    bukaForm = true
                                }
                            )
                        }
                        Spacer(Modifier.height(8.dp))
                        TombolUtama(
                            teks = if (item.status == "perlu_revisi") {
                                "Kirim Ulang ke Atasan"
                            } else {
                                "Kirim ke Atasan"
                            },
                            memuat = vm.sedangMemproses,
                            aktif = !vm.sedangMemproses,
                            onClick = { vm.ajukanOutput(item.id) }
                        )
                    } else if (bisaKirim) {
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            TombolKedua(
                                teks = "Lihat Detail",
                                modifier = Modifier.weight(1f),
                                aktif = !vm.sedangMemproses,
                                onClick = {
                                    vm.bersihkanPesan()
                                    detail = null
                                    detailId = item.id
                                }
                            )
                            TombolUtama(
                                teks = "Kirim ke Atasan",
                                modifier = Modifier.weight(1f),
                                memuat = vm.sedangMemproses,
                                aktif = !vm.sedangMemproses,
                                onClick = { vm.ajukanOutput(item.id) }
                            )
                        }
                    } else {
                        TombolKedua(
                            teks = "Lihat Detail Output",
                            aktif = !vm.sedangMemproses,
                            onClick = {
                                vm.bersihkanPesan()
                                detail = null
                                detailId = item.id
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

    lampiranDilihat?.let { berkas ->
        DialogLihatLampiran(vm, berkas) { lampiranDilihat = null }
    }
}
