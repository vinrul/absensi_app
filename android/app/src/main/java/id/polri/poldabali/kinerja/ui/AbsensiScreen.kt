package id.polri.poldabali.kinerja.ui

import android.Manifest
import android.content.pm.PackageManager
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.core.content.FileProvider
import id.polri.poldabali.kinerja.lokasi.Lokasi
import id.polri.poldabali.kinerja.ui.theme.Warna
import id.polri.poldabali.kinerja.util.Foto
import kotlinx.coroutines.launch
import java.io.File

private val STATUS_KEHADIRAN = listOf(
    "dinas_kantor" to "Dinas Kantor",
    "dinas_luar" to "Dinas Luar",
    "piket" to "Piket/Shift",
    "libur_nasional" to "Libur Nasional",
    "cuti" to "Cuti",
    "izin" to "Izin",
    "sakit" to "Sakit",
    "tanpa_keterangan" to "Tanpa Keterangan"
)

@Composable
fun AbsensiScreen(vm: KinerjaViewModel) {
    val konteks = LocalContext.current
    val skop = rememberCoroutineScope()

    var status by remember { mutableStateOf("dinas_kantor") }
    var jenis by remember { mutableStateOf(if (vm.riwayat.any { it.tanggal == tanggalHariIni() && it.jenis == "masuk" }) "pulang" else "masuk") }
    var offline by remember { mutableStateOf(false) }
    var lat by remember { mutableStateOf<Double?>(null) }
    var lng by remember { mutableStateOf<Double?>(null) }
    var akurasi by remember { mutableStateOf<Double?>(null) }
    var mock by remember { mutableStateOf(false) }
    var berkasFoto by remember { mutableStateOf<File?>(null) }
    var fotoBase64 by remember { mutableStateOf<String?>(null) }
    var mengambilLokasi by remember { mutableStateOf(false) }

    fun punya(izin: String) =
        ContextCompat.checkSelfPermission(konteks, izin) == PackageManager.PERMISSION_GRANTED

    val mintaLokasi = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { hasil ->
        if (hasil.values.any { it }) {
            skop.launch {
                mengambilLokasi = true
                runCatching { Lokasi.ambil(konteks) }
                    .onSuccess { lokasi ->
                        lat = lokasi.latitude
                        lng = lokasi.longitude
                        akurasi = lokasi.accuracy.toDouble()
                        mock = Lokasi.adalahMock(lokasi)
                        vm.cekLokasi(lokasi.latitude, lokasi.longitude, lokasi.accuracy.toDouble())
                    }
                    .onFailure { vm.pesanGalat(it.message ?: "Gagal mengambil lokasi") }
                mengambilLokasi = false
            }
        }
    }

    val ambilFoto = rememberLauncherForActivityResult(
        ActivityResultContracts.TakePicture()
    ) { sukses ->
        if (sukses) {
            fotoBase64 = Foto.keBase64Jpeg(berkasFoto!!)
        }
    }

    fun bukaKamera() {
        val berkas = Foto.berkasSementara(File(konteks.cacheDir, "absensi"))
        berkasFoto = berkas
        val uri = FileProvider.getUriForFile(
            konteks,
            "${konteks.packageName}.fileprovider",
            berkas
        )
        ambilFoto.launch(uri)
    }

    val mintaKamera = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { diberikan ->
        if (diberikan) bukaKamera()
        else vm.pesanGalat("Izin kamera diperlukan untuk mengambil foto bukti")
    }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Kartu {
            JudulBagian("Absensi", "Pencatatan kehadiran dengan validasi lokasi")
            Spacer(Modifier.height(10.dp))

            Text("Jenis Absensi", style = MaterialTheme.typography.bodySmall, color = Warna.TeksLembut)
            Spacer(Modifier.height(6.dp))
            FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                listOf("masuk" to "Masuk", "pulang" to "Pulang").forEach { (nilai, label) ->
                    FilterChip(
                        selected = jenis == nilai,
                        onClick = { jenis = nilai },
                        label = { Text(label) }
                    )
                }
            }

            Spacer(Modifier.height(12.dp))
            Text("Status Kehadiran", style = MaterialTheme.typography.bodySmall, color = Warna.TeksLembut)
            Spacer(Modifier.height(6.dp))
            FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                STATUS_KEHADIRAN.forEach { (nilai, label) ->
                    FilterChip(
                        selected = status == nilai,
                        onClick = { status = nilai },
                        label = { Text(label) }
                    )
                }
            }
        }

        Kartu {
            JudulBagian("Lokasi", "Radius titik dinas divalidasi server")
            Spacer(Modifier.height(10.dp))

            if (lat == null) {
                Text("Lokasi belum diambil.", style = MaterialTheme.typography.bodySmall, color = Warna.TeksLembut)
            } else {
                BarisInfo("Koordinat", String.format(java.util.Locale.US, "%.6f, %.6f", lat, lng))
                BarisInfo("Akurasi GPS", String.format(java.util.Locale.US, "%.0f m", akurasi ?: 0.0))
                val geofence = vm.titikTerdekat?.terdekat
                if (geofence?.jarakM != null) {
                    BarisInfo(
                        "Jarak ke titik dinas",
                        String.format(java.util.Locale.US, "%.0f m / radius %d m", geofence.jarakM, geofence.radiusM ?: 100),
                        if (geofence.dalamRadius) Warna.Hijau else Warna.Merah
                    )
                    BarisInfo("Titik", geofence.titikNama ?: "-")
                }
                if (mock) {
                    KotakPesan("Terdeteksi lokasi tiruan (mock). Absensi akan ditandai untuk verifikasi atasan.", JenisPesan.Galat)
                }
            }

            Spacer(Modifier.height(10.dp))
            TombolKedua(
                teks = when {
                    mengambilLokasi -> "Mengambil lokasi…"
                    lat == null -> "Ambil Lokasi"
                    else -> "Perbarui Lokasi"
                },
                aktif = !mengambilLokasi,
                onClick = {
                    if (punya(Manifest.permission.ACCESS_FINE_LOCATION)) {
                        skop.launch {
                            mengambilLokasi = true
                            runCatching { Lokasi.ambil(konteks) }
                                .onSuccess { lokasi ->
                                    lat = lokasi.latitude
                                    lng = lokasi.longitude
                                    akurasi = lokasi.accuracy.toDouble()
                                    mock = Lokasi.adalahMock(lokasi)
                                    vm.cekLokasi(lokasi.latitude, lokasi.longitude, lokasi.accuracy.toDouble())
                                }
                                .onFailure { vm.pesanGalat(it.message ?: "Gagal mengambil lokasi") }
                            mengambilLokasi = false
                        }
                    } else {
                        mintaLokasi.launch(
                            arrayOf(
                                Manifest.permission.ACCESS_FINE_LOCATION,
                                Manifest.permission.ACCESS_COARSE_LOCATION
                            )
                        )
                    }
                }
            )
        }

        Kartu {
            JudulBagian("Foto Bukti (opsional)", "Maksimal 5 MB, otomatis dikompres")
            Spacer(Modifier.height(10.dp))
            Text(
                if (fotoBase64 != null) "Foto siap diunggah." else "Belum ada foto.",
                style = MaterialTheme.typography.bodySmall,
                color = if (fotoBase64 != null) Warna.Hijau else Warna.TeksLembut
            )
            Spacer(Modifier.height(8.dp))
            TombolKedua(
                teks = if (fotoBase64 == null) "Ambil Foto" else "Ganti Foto",
                			onClick = {
                				if (punya(Manifest.permission.CAMERA)) bukaKamera()
                				else mintaKamera.launch(Manifest.permission.CAMERA)
                			}
            )
        }

        Kartu {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(Modifier.weight(1f)) {
                    Text("Mode offline", style = MaterialTheme.typography.titleMedium, color = Warna.Teks)
                    Text(
                        "Simpan di perangkat lalu kirim otomatis saat ada sinyal",
                        style = MaterialTheme.typography.bodySmall,
                        color = Warna.TeksLembut
                    )
                }
                Switch(checked = offline, onCheckedChange = { offline = it })
            }

            Spacer(Modifier.height(10.dp))

            vm.galat?.let { KotakPesan(it, JenisPesan.Galat); Spacer(Modifier.height(8.dp)) }
            vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses); Spacer(Modifier.height(8.dp)) }

            TombolUtama(
                teks = if (offline) "Simpan Absensi (Offline)" else "Kirim Absensi",
                memuat = vm.sedangMemproses,
                onClick = {
                    vm.kirimAbsensi(
                        jenis = jenis,
                        statusKehadiran = status,
                        latitude = lat,
                        longitude = lng,
                        akurasi = akurasi,
                        mock = mock,
                        fotoBase64 = fotoBase64,
                        offline = offline
                    ) { info ->
                        if (offline) {
                            fotoBase64 = null
                            berkasFoto = null
                        }
                        vm.pesanSukses(info)
                    }
                }
            )
        }

        Kartu {
            JudulBagian("Riwayat Bulan Ini", "${vm.riwayat.size} catatan")
            Spacer(Modifier.height(8.dp))

            if (vm.riwayat.isEmpty()) {
                LayarKosong("Belum ada absensi pada bulan ini.")
            } else {
                vm.riwayat.take(20).forEach { item ->
                    Row(
                        Modifier.fillMaxWidth().padding(vertical = 6.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column(Modifier.weight(1f)) {
                            Text(
                                "${tanggalIndo(item.tanggal)} · ${item.jenis}",
                                style = MaterialTheme.typography.bodyMedium,
                                color = Warna.Teks
                            )
                            Text(
                                buildString {
                                    append(labelStatusKehadiran(item.statusKehadiran))
                                    item.jarakM?.let {
                                        append(String.format(java.util.Locale.US, " · %.0f m", it))
                                    }
                                    if (item.perluVerifikasi) append(" · menunggu verifikasi")
                                },
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                        Lencana(item.jam, warnaStatusKehadiran(item.statusKehadiran))
                    }
                }
            }
        }
    }
}
