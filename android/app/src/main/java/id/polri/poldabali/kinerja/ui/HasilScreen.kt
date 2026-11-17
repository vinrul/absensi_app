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
import id.polri.poldabali.kinerja.data.Hasil
import id.polri.poldabali.kinerja.data.Lampiran
import id.polri.poldabali.kinerja.ui.theme.Warna
import java.util.Locale

@Composable
fun HasilScreen(
    vm: KinerjaViewModel,
    mintaBukaVerifikasi: Boolean = false,
    onVerifikasiDibuka: () -> Unit = {}
) {
    var bukaForm by remember { mutableStateOf(false) }
    var idSunting by remember { mutableStateOf<String?>(null) }
    var bukaVerifikasi by remember { mutableStateOf(false) }
    var detailId by remember { mutableStateOf<String?>(null) }
    var detail by remember { mutableStateOf<Hasil?>(null) }
    var lampiranDilihat by remember { mutableStateOf<Lampiran?>(null) }

    LaunchedEffect(vm.pengguna?.id) {
        vm.muatHasil()
        if (vm.rencana.isEmpty()) vm.muatRencana()
        if (vm.bisaMemverifikasiHasil) vm.muatVerifikasiHasil()
    }

    LaunchedEffect(mintaBukaVerifikasi) {
        if (mintaBukaVerifikasi) {
            bukaVerifikasi = true
            onVerifikasiDibuka()
        }
    }

    if (bukaForm) {
        WizardHasil(vm, idHasil = idSunting) {
            bukaForm = false
            idSunting = null
        }
        return
    }

    if (bukaVerifikasi) {
        VerifikasiHasilScreen(vm) { bukaVerifikasi = false }
        return
    }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        if (vm.bisaMemverifikasiHasil) {
            Kartu {
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.Top
                ) {
                    Column(Modifier.weight(1f)) {
                        Text(
                            "Verifikasi Hasil Kerja",
                            style = MaterialTheme.typography.titleMedium,
                            color = Warna.Teks
                        )
                        Text(
                            "Hasil kerja bawahan langsung yang menunggu verifikasi.",
                            style = MaterialTheme.typography.bodySmall,
                            color = Warna.TeksLembut
                        )
                    }
                    Lencana(
                        "${vm.hasilMenungguVerifikasi.size}",
                        if (vm.hasilMenungguVerifikasi.isEmpty()) Warna.TeksLembut else Warna.Kuning
                    )
                }
                Spacer(Modifier.height(8.dp))
                Text(
                    if (vm.hasilMenungguVerifikasi.isEmpty()) {
                        "Tidak ada hasil kerja yang menunggu verifikasi."
                    } else {
                        "${vm.hasilMenungguVerifikasi.size} hasil kerja menunggu verifikasi Anda."
                    },
                    style = MaterialTheme.typography.bodySmall,
                    color = Warna.Teks
                )
                Spacer(Modifier.height(10.dp))
                TombolUtama(
                    teks = if (vm.hasilMenungguVerifikasi.isEmpty()) {
                        "Buka Verifikasi"
                    } else {
                        "Tinjau Verifikasi (${vm.hasilMenungguVerifikasi.size})"
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
                    "Hasil Kerja",
                    "${vm.hasil.size} laporan realisasi",
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
                            labelStatusHasil(item.status),
                            when (item.status) {
                                "terverifikasi" -> Warna.Hijau
                                "perlu_revisi" -> Warna.Kuning
                                else -> Warna.Biru
                            }
                        )
                    }

                    Spacer(Modifier.height(8.dp))
                    HorizontalDivider(color = Warna.NavySoft)
                    Spacer(Modifier.height(8.dp))

                    BarisInfo("Progres", "${item.progresPersen.toInt()}%")
                    BarisInfo("Jumlah realisasi", item.jumlahRealisasi?.toString() ?: "-")
                    item.skorTotal?.let {
                        BarisInfo(
                            "Nilai hasil kerja",
                            String.format(Locale.US, "%.2f", it),
                            Warna.Hijau
                        )
                    }
                    item.kendala?.let { BarisInfo("Kendala", it) }
                    item.catatanVerifikator?.let { BarisInfo("Catatan atasan", it, Warna.Kuning) }

                    Spacer(Modifier.height(10.dp))
                    if (item.status == "draft" || item.status == "perlu_revisi") {
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
                            onClick = { vm.ajukanHasil(item.id) }
                        )
                    } else {
                        TombolKedua(
                            teks = "Lihat Detail Hasil",
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

    lampiranDilihat?.let { berkas ->
        DialogLihatLampiran(vm, berkas) { lampiranDilihat = null }
    }
}
