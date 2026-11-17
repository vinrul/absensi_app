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
import id.polri.poldabali.kinerja.data.Rencana
import id.polri.poldabali.kinerja.ui.theme.Warna

@Composable
fun RencanaScreen(
    vm: KinerjaViewModel,
    mintaBukaPersetujuan: Boolean = false,
    onPersetujuanDibuka: () -> Unit = {}
) {
    var bukaForm by remember { mutableStateOf(false) }
    var idSunting by remember { mutableStateOf<String?>(null) }
    var bukaPersetujuan by remember { mutableStateOf(false) }
    var detailId by remember { mutableStateOf<String?>(null) }
    var detail by remember { mutableStateOf<Rencana?>(null) }
    var lampiranDilihat by remember { mutableStateOf<Lampiran?>(null) }

    LaunchedEffect(vm.pengguna?.id) {
        vm.muatRencana()
        if (vm.bisaMenyetujuiRencana) vm.muatPersetujuan()
    }

    LaunchedEffect(mintaBukaPersetujuan) {
        if (mintaBukaPersetujuan) {
            bukaPersetujuan = true
            onPersetujuanDibuka()
        }
    }

    if (bukaForm) {
        WizardRencana(vm, idRencana = idSunting) {
            bukaForm = false
            idSunting = null
        }
        return
    }

    if (bukaPersetujuan) {
        PersetujuanScreen(vm) { bukaPersetujuan = false }
        return
    }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        if (vm.bisaMenyetujuiRencana) {
            Kartu {
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.Top
                ) {
                    Column(Modifier.weight(1f)) {
                        Text(
                            "Persetujuan Rencana",
                            style = MaterialTheme.typography.titleMedium,
                            color = Warna.Teks
                        )
                        Text(
                            "Rencana kerja bawahan langsung yang menunggu keputusan Anda.",
                            style = MaterialTheme.typography.bodySmall,
                            color = Warna.TeksLembut
                        )
                    }
                    Lencana(
                        "${vm.persetujuan.size}",
                        if (vm.persetujuan.isEmpty()) Warna.TeksLembut else Warna.Kuning
                    )
                }
                Spacer(Modifier.height(8.dp))
                Text(
                    if (vm.persetujuan.isEmpty()) {
                        "Tidak ada rencana yang menunggu keputusan."
                    } else {
                        "${vm.persetujuan.size} rencana menunggu keputusan Anda."
                    },
                    style = MaterialTheme.typography.bodySmall,
                    color = Warna.Teks
                )
                Spacer(Modifier.height(10.dp))
                TombolUtama(
                    teks = if (vm.persetujuan.isEmpty()) {
                        "Buka Persetujuan"
                    } else {
                        "Tinjau Persetujuan (${vm.persetujuan.size})"
                    },
                    aktif = !vm.sedangMemproses,
                    onClick = {
                        vm.bersihkanPesan()
                        bukaPersetujuan = true
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
                    "Rencana Kerja",
                    "${vm.rencana.size} dokumen · ${vm.periode?.nama ?: ""}",
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

                    BarisInfo("Jenis", labelJenisRencana(item.jenis))
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
                            teks = "Ajukan ke Atasan",
                            memuat = vm.sedangMemproses,
                            aktif = !vm.sedangMemproses,
                            onClick = { vm.ajukanRencana(item.id) }
                        )
                    } else {
                        Spacer(Modifier.height(10.dp))
                        TombolKedua(
                            teks = "Lihat Detail Rencana",
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

    lampiranDilihat?.let { berkas ->
        DialogLihatLampiran(vm, berkas) { lampiranDilihat = null }
    }
}
