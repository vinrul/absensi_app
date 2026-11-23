package id.polri.poldabali.kinerja.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.DateRange
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import id.polri.poldabali.kinerja.data.AbsensiPh
import id.polri.poldabali.kinerja.ui.theme.Warna
import java.time.Instant

private const val HALAMAN_BERANDA = 0
private const val HALAMAN_WIZARD = 1
private const val HALAMAN_RIWAYAT = 2

@Composable
fun PhScreen(vm: KinerjaViewModel) {
    var halaman by rememberSaveable { mutableIntStateOf(HALAMAN_BERANDA) }

    LaunchedEffect(Unit) { vm.muatPh() }

    when (halaman) {
        HALAMAN_WIZARD -> WizardStrongPoint(vm) {
            halaman = HALAMAN_BERANDA
        }

        HALAMAN_RIWAYAT -> Column(
            Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
            vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses) }
            HalamanRiwayat(vm) {
                vm.bersihkanPesan()
                halaman = HALAMAN_BERANDA
            }
        }

        else -> Column(
            Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
            vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses) }
            HalamanBeranda(vm) {
                vm.bersihkanPesan()
                halaman = HALAMAN_WIZARD
            }
            TombolKedua(
                teks = "Riwayat Strong Point",
                onClick = {
                    vm.bersihkanPesan()
                    halaman = HALAMAN_RIWAYAT
                }
            )
        }
    }
}

@Composable
private fun HalamanBeranda(vm: KinerjaViewModel, onMulai: () -> Unit) {
    val hariIni = tanggalHariIni()
    val jadwal = pilihJadwalHariIni(vm.jadwalPh, vm.absensiPh, vm.antreanPh, hariIni)
    val status = statusStrongPoint(jadwal?.id, vm.absensiPh, vm.antreanPh)

    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .height(124.dp)
            .clickable(onClick = onMulai),
        shape = RoundedCornerShape(18.dp),
        color = Warna.Navy,
        shadowElevation = 3.dp
    ) {
        Column(
            Modifier.fillMaxSize().padding(20.dp),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Icon(
                Icons.Filled.DateRange,
                contentDescription = null,
                tint = Color.White,
                modifier = Modifier.size(38.dp)
            )
            Spacer(Modifier.height(6.dp))
            Text(
                "Strong Point",
                style = MaterialTheme.typography.headlineSmall,
                color = Color.White
            )
            Text(
                "Check-in / check-out dengan validasi lokasi",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.NavySoft
            )
        }
    }

    Kartu {
        val keterangan = when {
            jadwal == null -> "Tidak ada jadwal strong point hari ini"
            status.sesi.isEmpty() -> "Belum check-in hari ini"
            status.terbuka ->
                "Check-in pukul ${status.masukTerakhir?.jam.orEmpty()}" +
                        if (status.perluCheckInBaru) " (lebih dari 2 jam, belum check-out)" else ""

            else -> "Sudah check-out pukul ${status.pulangTerakhir?.jam.orEmpty()}"
        }
        JudulBagian("Status Hari Ini", keterangan)
        if (jadwal != null) {
            Spacer(Modifier.height(8.dp))
            BarisInfo("Jadwal", keteranganJadwal(jadwal.regu, jadwal.strongPointNama))
            BarisInfo(
                "Sesi",
                "${if (jadwal.sesi == "sore") "Sore" else "Pagi"} ${jadwal.jamMulai}–${jadwal.jamSelesai}"
            )
            BarisInfo("Sesi hari ini", "${status.sesi.size} catatan")
        }
        if (status.sesi.any { it.menungguKirim }) {
            Spacer(Modifier.height(8.dp))
            KotakPesan(
                "Sebagian sesi strong point masih menunggu dikirim. Data tersimpan di perangkat.",
                JenisPesan.Info
            )
        }
    }

    if (vm.antreanPh.isNotEmpty()) {
        Kartu {
            JudulBagian(
                "Antrean Strong Point",
                "${vm.antreanPh.size} data menunggu dikirim ke server"
            )
            Spacer(Modifier.height(8.dp))
            TombolKedua(
                teks = "Kirim Antrean Sekarang",
                aktif = !vm.sedangMemproses,
                onClick = { vm.sinkronkanSekarang() }
            )
        }
    }
}

@Composable
private fun HalamanRiwayat(vm: KinerjaViewModel, onKembali: () -> Unit) {
    BarisKembaliStrong("Riwayat Strong Point", onKembali)

    if (vm.antreanPh.isNotEmpty()) {
        Kartu {
            JudulBagian("Menunggu Dikirim", "${vm.antreanPh.size} catatan offline")
            Spacer(Modifier.height(8.dp))
            vm.antreanPh.takeLast(10).reversed().forEach { item ->
                val waktu = runCatching { Instant.parse(item.capturedAt) }.getOrNull()
                BarisInfo(
                    label = "${tanggalIndo(item.capturedAt.take(10))} · ${labelJenis(item.jenis)}",
                    nilai = "${waktu?.let(::jamWita) ?: "-"} · offline",
                    warnaNilai = Warna.Kuning
                )
            }
            Spacer(Modifier.height(8.dp))
            TombolKedua(
                teks = "Kirim Semua Antrean",
                aktif = !vm.sedangMemproses,
                onClick = { vm.sinkronkanSekarang() }
            )
        }
    }

    Kartu {
        JudulBagian("Riwayat Strong Point", "${vm.absensiPh.size} catatan 30 hari terakhir")
        Spacer(Modifier.height(8.dp))
        if (vm.absensiPh.isEmpty()) {
            LayarKosong("Belum ada absensi strong point pada 30 hari terakhir.")
        } else {
            vm.absensiPh.sortedByDescending { it.capturedAt }.forEach { item ->
                BarisInfo(
                    label = "${tanggalIndo(item.capturedAt.take(10))} · ${labelJenis(item.jenis)}",
                    nilai = "${item.jam} · ${jarakTeks(item)}",
                    warnaNilai = when {
                        !item.dalamRadius -> Warna.Merah
                        item.perluVerifikasi && item.verifiedAt == null -> Warna.Kuning
                        else -> Warna.Hijau
                    }
                )
                Text(
                    item.strongPointNama,
                    style = MaterialTheme.typography.bodySmall,
                    color = Warna.TeksLembut
                )
                if (item.perluVerifikasi && item.verifiedAt == null) {
                    Text(
                        "Menunggu verifikasi atasan",
                        style = MaterialTheme.typography.labelSmall,
                        color = Warna.Kuning
                    )
                }
                Spacer(Modifier.height(6.dp))
            }
        }
    }

    Kartu {
        JudulBagian("Jadwal 30 Hari", "${vm.jadwalPh.size} jadwal")
        Spacer(Modifier.height(8.dp))
        if (vm.jadwalPh.isEmpty()) {
            LayarKosong("Belum ada jadwal strong point pada 30 hari terakhir/berikutnya.")
        } else {
            vm.jadwalPh.forEach { item ->
                BarisInfo(
                    label = tanggalIndo(item.tanggal),
                    nilai = "${if (item.sesi == "sore") "Sore" else "Pagi"} ${item.jamMulai}–${item.jamSelesai} · " +
                            keteranganJadwal(item.regu, item.strongPointNama)
                )
            }
        }
    }
}

@Composable
private fun BarisKembaliStrong(judul: String, onKembali: () -> Unit) {
    Row(
        Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically
    ) {
        TextButton(onClick = onKembali) {
            Text("‹ Kembali", color = Warna.Navy)
        }
        Text(
            judul,
            style = MaterialTheme.typography.titleMedium,
            color = Warna.Teks,
            fontWeight = FontWeight.SemiBold
        )
    }
}

internal fun keteranganJadwal(regu: String?, strongPoint: String): String =
    listOfNotNull(strongPoint, regu?.takeIf { it.isNotBlank() }).joinToString(" · ")

internal fun labelJenis(jenis: String): String =
    if (jenis == "masuk") "Check-in" else "Check-out"

internal fun jarakTeks(item: AbsensiPh): String =
    item.jarakM?.let { String.format(java.util.Locale.US, "%.0f m", it) } ?: "-"
