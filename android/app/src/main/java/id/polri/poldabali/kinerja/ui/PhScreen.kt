package id.polri.poldabali.kinerja.ui

import android.Manifest
import android.content.pm.PackageManager
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import id.polri.poldabali.kinerja.data.AbsensiPh
import id.polri.poldabali.kinerja.data.JadwalPhSaya
import id.polri.poldabali.kinerja.lokasi.Lokasi
import id.polri.poldabali.kinerja.ui.theme.Warna
import kotlinx.coroutines.launch

@Composable
fun PhScreen(vm: KinerjaViewModel) {
    val konteks = LocalContext.current
    val skop = rememberCoroutineScope()
    var menunggu by remember { mutableStateOf<Pair<JadwalPhSaya, String>?>(null) }
    var mengambil by remember { mutableStateOf(false) }

    fun punyaLokasi(): Boolean =
        ContextCompat.checkSelfPermission(konteks, Manifest.permission.ACCESS_FINE_LOCATION) ==
            PackageManager.PERMISSION_GRANTED

    fun kirimDenganLokasi(item: JadwalPhSaya, jenis: String) {
        skop.launch {
            mengambil = true
            runCatching { Lokasi.ambil(konteks) }
                .onSuccess { lokasi ->
                    vm.kirimAbsensiPh(
                        jadwalId = item.id,
                        jenis = jenis,
                        latitude = lokasi.latitude,
                        longitude = lokasi.longitude,
                        akurasi = lokasi.accuracy.toDouble(),
                        mock = Lokasi.adalahMock(lokasi),
                        fotoBase64 = null
                    ) { info -> vm.pesanSukses(info) }
                }
                .onFailure { vm.pesanGalat(it.message ?: "Gagal mengambil lokasi") }
            mengambil = false
        }
    }

    val mintaLokasi = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { hasil ->
        val item = menunggu
        menunggu = null
        if (hasil.values.any { it } && item != null) {
            kirimDenganLokasi(item.first, item.second)
        } else if (item != null) {
            vm.pesanGalat("Izin lokasi diperlukan untuk absensi PH")
        }
    }

    fun proses(item: JadwalPhSaya, jenis: String) {
        if (punyaLokasi()) {
            kirimDenganLokasi(item, jenis)
        } else {
            menunggu = item to jenis
            mintaLokasi.launch(
                arrayOf(
                    Manifest.permission.ACCESS_FINE_LOCATION,
                    Manifest.permission.ACCESS_COARSE_LOCATION
                )
            )
        }
    }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Text(
            "PH & Strong Point",
            style = MaterialTheme.typography.titleLarge,
            color = Warna.Teks
        )
        Text(
            "Check-in dan check-out wajib di lokasi strong point. Lokasi diambil setiap kali absen.",
            style = MaterialTheme.typography.bodySmall,
            color = Warna.TeksLembut
        )

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
        vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses) }

        if (vm.jadwalPh.isEmpty()) {
            Kartu {
                JudulBagian("Jadwal PH", "Belum ada jadwal PH bulan ini")
                Spacer(Modifier.height(8.dp))
                Text(
                    "Admin satker belum menyusun jadwal PH untuk Anda pada bulan ini.",
                    style = MaterialTheme.typography.bodySmall,
                    color = Warna.TeksLembut
                )
            }
        } else {
            			vm.jadwalPh.forEach { item ->
            				val masuk = vm.absensiPh.firstOrNull {
            					it.jadwalPhId == item.id && it.jenis == "masuk"
            				}
            				val pulang = vm.absensiPh.firstOrNull {
            					it.jadwalPhId == item.id && it.jenis == "pulang"
            				}
            				val hariIni = item.tanggal == tanggalHariIni()
                Kartu {
                    Row(
                        Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Column(Modifier.weight(1f)) {
                            Text(
                                tanggalIndo(item.tanggal),
                                style = MaterialTheme.typography.titleMedium,
                                color = Warna.Teks
                            )
                            Text(
                                "${item.jamMulai}–${item.jamSelesai} · ${item.strongPointNama}",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                            if (!item.peran.isNullOrBlank()) {
                                Text(
                                    "Peran: ${item.peran}",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = Warna.Gold
                                )
                            }
                            if (!item.catatan.isNullOrBlank()) {
                                Text(
                                    item.catatan,
                                    style = MaterialTheme.typography.bodySmall,
                                    color = Warna.TeksLembut
                                )
                            }
                        }
                        val selesai = masuk != null && pulang != null
                        Lencana(
                            when {
                                selesai -> "Selesai"
                                masuk != null -> "Check-in"
                                else -> "Belum"
                            },
                            when {
                                selesai -> Warna.Hijau
                                masuk != null -> Warna.Biru
                                else -> Warna.Kuning
                            }
                        )
                    }
                    Spacer(Modifier.height(8.dp))
                    BarisInfo("Radius", "${item.radiusM} m")
                    if (masuk != null) {
                        BarisInfo("Check-in", "${masuk.jam} · ${jarakTeks(masuk)}", Warna.Hijau)
                    }
                    if (pulang != null) {
                        BarisInfo("Check-out", "${pulang.jam} · ${jarakTeks(pulang)}", Warna.Hijau)
                    }
                    					Spacer(Modifier.height(8.dp))
                    					if (!hariIni) {
                    						Text(
                    							if (item.tanggal < tanggalHariIni()) "Jadwal PH sudah lewat." else "Check-in dapat dilakukan pada tanggal jadwal.",
                    							style = MaterialTheme.typography.bodySmall,
                    							color = Warna.TeksLembut
                    						)
                    					} else if (masuk == null) {
                        TombolUtama(
                            teks = if (mengambil) "Mengambil lokasi…" else "Check-in PH",
                            aktif = !mengambil,
                            onClick = { proses(item, "masuk") }
                        )
                    } else if (pulang == null) {
                        TombolUtama(
                            teks = if (mengambil) "Mengambil lokasi…" else "Check-out PH",
                            aktif = !mengambil,
                            onClick = { proses(item, "pulang") }
                        )
                    } else {
                        Text(
                            "Absensi PH untuk jadwal ini sudah lengkap.",
                            style = MaterialTheme.typography.bodySmall,
                            color = Warna.Hijau
                        )
                    }
                }
            }
        }

        if (vm.absensiPh.isNotEmpty()) {
            Kartu {
                JudulBagian("Riwayat PH Bulan Ini", "${vm.absensiPh.size} catatan")
                Spacer(Modifier.height(8.dp))
                vm.absensiPh.takeLast(10).reversed().forEach { item ->
                    BarisInfo(
                        label = "${item.spKodeTanggal()} · ${item.jenis}",
                        nilai = "${item.jam} · ${jarakTeks(item)}",
                        warnaNilai = if (item.dalamRadius) Warna.Hijau else Warna.Merah
                    )
                }
            }
        }
    }
}

private fun jarakTeks(item: AbsensiPh): String =
    item.jarakM?.let { String.format(java.util.Locale.US, "%.0f m", it) } ?: "-"

private fun AbsensiPh.spKodeTanggal(): String =
    "${tanggalIndo(capturedAt.take(10))} · $strongPointNama"
