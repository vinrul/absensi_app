package id.polri.poldabali.kinerja.ui

import android.Manifest
import android.content.pm.PackageManager
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.DateRange
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Place
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.core.content.FileProvider
import id.polri.poldabali.kinerja.lokasi.Lokasi
import id.polri.poldabali.kinerja.ui.theme.Warna
import id.polri.poldabali.kinerja.util.Foto
import kotlinx.coroutines.launch
import java.io.File
import java.util.Locale

private const val HALAMAN_BERANDA = 0
private const val HALAMAN_VALIDASI = 1
private const val HALAMAN_STATUS = 2
private const val HALAMAN_RIWAYAT = 3

private data class OpsiStatus(
    val nilai: String,
    val label: String,
    val ikon: ImageVector,
    val keterangan: String? = null
)

private val STATUS_KEHADIRAN = listOf(
    OpsiStatus("dinas_kantor", "Dinas Kantor", Icons.Filled.Home),
    OpsiStatus("dinas_luar", "Dinas Luar", Icons.Filled.Place),
    OpsiStatus("ph", "PH", Icons.Filled.Person, "Perintah Hadir"),
    OpsiStatus("piket", "Piket / Shift", Icons.Filled.Lock),
    OpsiStatus("cuti", "Cuti", Icons.Filled.DateRange),
    OpsiStatus("izin", "Izin", Icons.AutoMirrored.Filled.List),
    OpsiStatus("sakit", "Sakit", Icons.Filled.Favorite),
    OpsiStatus("tanpa_keterangan", "Tanpa Keterangan", Icons.Filled.Warning)
)

@Composable
fun AbsensiScreen(vm: KinerjaViewModel) {
    val konteks = LocalContext.current
    val skop = rememberCoroutineScope()

    var halaman by rememberSaveable { mutableIntStateOf(HALAMAN_BERANDA) }
    var jenis by rememberSaveable { mutableStateOf("masuk") }
    var status by rememberSaveable { mutableStateOf<String?>(null) }
    var offline by rememberSaveable { mutableStateOf(false) }

    var lat by remember { mutableStateOf<Double?>(null) }
    var lng by remember { mutableStateOf<Double?>(null) }
    var akurasi by remember { mutableStateOf<Double?>(null) }
    var mock by remember { mutableStateOf(false) }
    var devOptions by remember { mutableStateOf(false) }
    var aplikasiMock by remember { mutableStateOf<String?>(null) }
    var mengambilLokasi by remember { mutableStateOf(false) }

    var berkasFoto by remember { mutableStateOf<File?>(null) }
    var fotoBase64 by remember { mutableStateOf<String?>(null) }

    val hariIni = vm.absensiHariIni
    val dalamRadius = vm.titikTerdekat?.terdekat?.dalamRadius == true
    val titikTerdekat = vm.titikTerdekat?.titik?.minByOrNull { it.jarakM ?: Double.MAX_VALUE }

    fun punya(izin: String) =
        ContextCompat.checkSelfPermission(konteks, izin) == PackageManager.PERMISSION_GRANTED

    suspend fun ambilLokasi() {
        mengambilLokasi = true
        try {
            val lokasi = Lokasi.ambil(konteks)
            lat = lokasi.latitude
            lng = lokasi.longitude
            akurasi = lokasi.accuracy.toDouble()
            val kondisi = Lokasi.kondisi(konteks, lokasi)
            mock = kondisi.mock
            devOptions = kondisi.developerOptions
            aplikasiMock = kondisi.aplikasiMock
            vm.cekLokasi(lokasi.latitude, lokasi.longitude, lokasi.accuracy.toDouble())
        } catch (galat: Exception) {
            vm.pesanGalat(galat.message ?: "Gagal mengambil lokasi")
        } finally {
            mengambilLokasi = false
        }
    }

    val mintaLokasi = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { hasil ->
        if (hasil.values.any { it }) {
            skop.launch { ambilLokasi() }
        } else {
            vm.pesanGalat("Izin lokasi diperlukan untuk absensi berbasis radius")
        }
    }

    fun mintaAtauAmbilLokasi() {
        if (punya(Manifest.permission.ACCESS_FINE_LOCATION) ||
            punya(Manifest.permission.ACCESS_COARSE_LOCATION)
        ) {
            skop.launch { ambilLokasi() }
        } else {
            mintaLokasi.launch(
                arrayOf(
                    Manifest.permission.ACCESS_FINE_LOCATION,
                    Manifest.permission.ACCESS_COARSE_LOCATION
                )
            )
        }
    }

    fun mulaiValidasi() {
        vm.bersihkanPesan()
        vm.muatAbsensiHariIni()
        halaman = HALAMAN_VALIDASI
        if (lat == null) mintaAtauAmbilLokasi()
    }

    fun lanjutPilihStatus(jenisDipilih: String) {
        jenis = jenisDipilih
        status = if (dalamRadius) "dinas_kantor" else null
        offline = vm.absensiHariIniOffline
        vm.bersihkanPesan()
        halaman = HALAMAN_STATUS
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

    LaunchedEffect(Unit) {
        vm.muatAbsensiHariIni()
    }

    Column(
        Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
        vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses) }

        when (halaman) {
            HALAMAN_BERANDA -> {
                Surface(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(124.dp)
                        .clickable { mulaiValidasi() },
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
                            Icons.Filled.Place,
                            contentDescription = null,
                            tint = Color.White,
                            modifier = Modifier.size(38.dp)
                        )
                        Spacer(Modifier.height(6.dp))
                        Text(
                            "Absensi",
                            style = MaterialTheme.typography.headlineSmall,
                            color = Color.White
                        )
                        Text(
                            "Absen dengan validasi lokasi GPS",
                            style = MaterialTheme.typography.bodySmall,
                            color = Warna.NavySoft
                        )
                    }
                }

                Kartu {
                    val keterangan = when {
                        vm.memuatAbsensiHariIni && hariIni == null -> "Memeriksa absensi hari ini…"
                        vm.absensiHariIniOffline -> "Tidak dapat memeriksa absensi hari ini (offline)"
                        hariIni?.masuk != null && hariIni?.pulang != null ->
                            "Absensi hari ini sudah lengkap"

                        hariIni?.masuk != null ->
                            "Sudah absen masuk pukul ${hariIni?.masuk?.jam}"

                        else -> "Belum absen masuk hari ini"
                    }
                    JudulBagian("Status Hari Ini", keterangan)
                    if (hariIni?.pulang != null) {
                        Spacer(Modifier.height(4.dp))
                        Text(
                            "Sudah absen pulang pukul ${hariIni?.pulang?.jam}",
                            style = MaterialTheme.typography.bodySmall,
                            color = Warna.TeksLembut
                        )
                    }
                    if (vm.absensiHariIniOffline) {
                        Spacer(Modifier.height(8.dp))
                        KotakPesan(
                            "Status tidak dapat diambil dari server. Anda tetap dapat memilih absen masuk/pulang dan menyimpannya secara offline.",
                            JenisPesan.Info
                        )
                    }
                }

                if (vm.antrean.isNotEmpty()) {
                    Kartu {
                        JudulBagian(
                            "Antrean Offline",
                            "${vm.antrean.size} absensi menunggu dikirim ke server"
                        )
                        Spacer(Modifier.height(8.dp))
                        TombolKedua(
                            teks = "Kirim Antrean Sekarang",
                            aktif = !vm.sedangMemproses,
                            onClick = { vm.sinkronkanSekarang() }
                        )
                    }
                }

                TombolKedua(
                    teks = "Riwayat Absensi",
                    onClick = {
                        vm.bersihkanPesan()
                        halaman = HALAMAN_RIWAYAT
                    }
                )
            }

            HALAMAN_VALIDASI -> {
                BarisKembali("Validasi Lokasi (GPS)") {
                    vm.bersihkanPesan()
                    halaman = HALAMAN_BERANDA
                }

                Kartu {
                    PetaAbsensi(
                        latitude = lat,
                        longitude = lng,
                        akurasiM = akurasi,
                        titik = titikTerdekat,
                        modifier = Modifier.fillMaxWidth()
                    )

                    if (mengambilLokasi) {
                        Spacer(Modifier.height(8.dp))
                        Row(
                            Modifier.fillMaxWidth(),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(18.dp),
                                strokeWidth = 2.dp,
                                color = Warna.Navy
                            )
                            Spacer(Modifier.width(8.dp))
                            Text(
                                "Mengambil lokasi…",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                    }

                    Spacer(Modifier.height(8.dp))
                    TombolKedua(
                        teks = when {
                            lat == null -> "Ambil Lokasi"
                            else -> "Perbarui Lokasi"
                        },
                        aktif = !mengambilLokasi,
                        onClick = { mintaAtauAmbilLokasi() }
                    )
                }

                if (lat != null) {
                    val warna = when {
                        dalamRadius -> Warna.Hijau
                        vm.absensiHariIniOffline -> Warna.Kuning
                        else -> Warna.Merah
                    }
                    Surface(
                        shape = RoundedCornerShape(14.dp),
                        color = warna.copy(alpha = 0.08f),
                        border = BorderStroke(1.dp, warna.copy(alpha = 0.35f)),
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Column(Modifier.padding(14.dp)) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Icon(
                                    if (dalamRadius) Icons.Filled.CheckCircle else Icons.Filled.Warning,
                                    contentDescription = null,
                                    tint = warna,
                                    modifier = Modifier.size(24.dp)
                                )
                                Spacer(Modifier.width(8.dp))
                                Text(
                                    when {
                                        dalamRadius -> "Lokasi Terverifikasi"
                                        vm.absensiHariIniOffline -> "Lokasi Belum Terverifikasi"
                                        else -> "Di Luar Radius"
                                    },
                                    style = MaterialTheme.typography.titleMedium,
                                    color = warna,
                                    fontWeight = FontWeight.Bold
                                )
                            }
                            Spacer(Modifier.height(4.dp))
                            Text(
                                when {
                                    dalamRadius ->
                                        "Anda berada dalam radius ${vm.titikTerdekat?.terdekat?.radiusM ?: 100} meter dari titik dinas."

                                    vm.absensiHariIniOffline ->
                                        "Titik dinas belum dapat diverifikasi karena server tidak terjangkau. Absensi akan disimpan offline dan diverifikasi saat sinkronisasi."

                                    else ->
                                        "Anda berada di luar radius titik dinas. Status Dinas Kantor/Piket tidak dapat dipilih."
                                },
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                            Spacer(Modifier.height(8.dp))
                            BarisInfo(
                                "Koordinat Anda",
                                if (lat != null && lng != null) {
                                    String.format(Locale.US, "%.6f, %.6f", lat, lng)
                                } else {
                                    "-"
                                }
                            )
                            BarisInfo(
                                "Jarak ke Titik Dinas",
                                vm.titikTerdekat?.terdekat?.jarakM?.let {
                                    String.format(Locale.US, "%.0f meter", it)
                                } ?: "Menghitung…"
                            )
                            BarisInfo(
                                "Akurasi GPS",
                                akurasi?.let { String.format(Locale.US, "± %.0f meter", it) } ?: "-"
                            )
                            BarisInfo(
                                "Titik Dinas",
                                titikTerdekat?.nama ?: vm.titikTerdekat?.terdekat?.titikNama ?: "-"
                            )
                        }
                    }

                    if (mock) {
                        KotakPesan(
                            "Terdeteksi lokasi tiruan (mock). Absensi dapat ditolak server atau ditandai untuk verifikasi atasan.",
                            JenisPesan.Galat
                        )
                    } else if (aplikasiMock != null) {
                        KotakPesan(
                            "Aplikasi lokasi tiruan terpilih: $aplikasiMock. Pastikan aplikasi tersebut tidak aktif; lokasi asli tetap dapat dikirim.",
                            JenisPesan.Info
                        )
                    } else if (devOptions) {
                        KotakPesan(
                            "Developer Options aktif. Selama tidak memakai aplikasi lokasi tiruan, absensi dengan GPS asli tetap dapat dikirim.",
                            JenisPesan.Info
                        )
                    }

                    when {
                        vm.absensiHariIniOffline -> {
                            KotakPesan(
                                "Status absensi hari ini tidak dapat diambil dari server. Pilih jenis absen yang sesuai; data akan disimpan offline.",
                                JenisPesan.Info
                            )
                            Row(
                                Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.spacedBy(8.dp)
                            ) {
                                TombolAbsen(
                                    "ABSEN MASUK",
                                    "(Check In)",
                                    Warna.Hijau,
                                    Modifier.weight(1f)
                                ) { lanjutPilihStatus("masuk") }
                                TombolAbsen(
                                    "ABSEN PULANG",
                                    "(Check Out)",
                                    Warna.Merah,
                                    Modifier.weight(1f)
                                ) { lanjutPilihStatus("pulang") }
                            }
                        }

                        vm.memuatAbsensiHariIni && hariIni == null -> {
                            Row(
                                Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.Center,
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                CircularProgressIndicator(
                                    modifier = Modifier.size(18.dp),
                                    strokeWidth = 2.dp,
                                    color = Warna.Navy
                                )
                                Spacer(Modifier.width(8.dp))
                                Text(
                                    "Memeriksa absensi hari ini…",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = Warna.TeksLembut
                                )
                            }
                        }

                        hariIni?.masuk == null -> {
                            TombolAbsen(
                                "ABSEN MASUK",
                                "(Check In)",
                                Warna.Hijau,
                                Modifier.fillMaxWidth()
                            ) { lanjutPilihStatus("masuk") }
                        }

                        hariIni?.pulang == null -> {
                            TombolAbsen(
                                "ABSEN PULANG",
                                "(Check Out)",
                                Warna.Merah,
                                Modifier.fillMaxWidth()
                            ) { lanjutPilihStatus("pulang") }
                        }

                        else -> {
                            KotakPesan(
                                "Absensi hari ini sudah lengkap (masuk dan pulang).",
                                JenisPesan.Sukses
                            )
                        }
                    }
                } else {
                    KotakPesan(
                        "Ambil lokasi terlebih dahulu untuk melanjutkan absensi.",
                        JenisPesan.Info
                    )
                }
            }

            HALAMAN_STATUS -> {
                BarisKembali("Pilih Status Kehadiran") {
                    vm.bersihkanPesan()
                    halaman = HALAMAN_VALIDASI
                }

                Kartu {
                    JudulBagian(
                        "Status Kehadiran",
                        "Pilih alasan kehadiran Anda hari ini"
                    )
                    Spacer(Modifier.height(8.dp))

                    STATUS_KEHADIRAN.chunked(2).forEach { baris ->
                        Row(
                            Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            baris.forEach { opsi ->
                                KartuStatus(
                                    opsi = opsi,
                                    terpilih = status == opsi.nilai,
                                    modifier = Modifier.weight(1f)
                                ) {
                                    status = opsi.nilai
                                }
                            }
                            if (baris.size == 1) Spacer(Modifier.weight(1f))
                        }
                        Spacer(Modifier.height(8.dp))
                    }

                    val butuhRadius = status == "dinas_kantor" || status == "piket"
                    if (butuhRadius && !dalamRadius && !offline && !vm.absensiHariIniOffline) {
                        KotakPesan(
                            "Status Dinas Kantor/Piket membutuhkan lokasi di dalam radius titik dinas. Pilih status lain atau perbarui lokasi.",
                            JenisPesan.Galat
                        )
                    }
                }

                Kartu {
                    JudulBagian(
                        "Dokumentasi",
                        "Foto bukti bersifat opsional, maksimal 5 MB"
                    )
                    Spacer(Modifier.height(8.dp))
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

                if (vm.absensiHariIniOffline) {
                    Kartu {
                        Row(
                            Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Column(Modifier.weight(1f)) {
                                Text(
                                    "Mode offline",
                                    style = MaterialTheme.typography.titleMedium,
                                    color = Warna.Teks
                                )
                                Text(
                                    "Absensi disimpan di perangkat lalu dikirim otomatis saat ada sinyal",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = Warna.TeksLembut
                                )
                            }
                            Switch(checked = offline, onCheckedChange = { offline = it })
                        }
                    }
                }

                val butuhRadius = status == "dinas_kantor" || status == "piket"
                val bolehKirim =
                    status != null &&
                            (offline || vm.absensiHariIniOffline || !(butuhRadius && !dalamRadius))

                TombolUtama(
                    teks = if (offline) "Simpan Absensi (Offline)" else "Kirim Absensi",
                    memuat = vm.sedangMemproses,
                    aktif = bolehKirim,
                    onClick = {
                        val statusFinal = status ?: return@TombolUtama
                        vm.kirimAbsensi(
                            jenis = jenis,
                            statusKehadiran = statusFinal,
                            latitude = lat,
                            longitude = lng,
                            akurasi = akurasi,
                            mock = mock,
                            fotoBase64 = fotoBase64,
                            offline = offline
                        ) { info ->
                            vm.pesanSukses(info)
                            fotoBase64 = null
                            berkasFoto = null
                            halaman = HALAMAN_BERANDA
                        }
                    }
                )
            }

            else -> {
                BarisKembali("Riwayat Absensi") {
                    vm.bersihkanPesan()
                    halaman = HALAMAN_BERANDA
                }

                Kartu {
                    Row(
                        Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column(Modifier.weight(1f)) {
                            Text(
                                "Bulan Ini",
                                style = MaterialTheme.typography.titleMedium,
                                color = Warna.Teks
                            )
                            Text(
                                "${vm.riwayat.size} catatan",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                        TextButton(onClick = { vm.muatRiwayatBulanIni() }) {
                            Text("Muat ulang", color = Warna.Navy)
                        }
                    }
                    Spacer(Modifier.height(6.dp))

                    if (vm.riwayat.isEmpty()) {
                        LayarKosong("Belum ada absensi pada bulan ini.")
                    } else {
                        vm.riwayat.forEach { item ->
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
                                                append(String.format(Locale.US, " · %.0f m", it))
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

                if (vm.antrean.isNotEmpty()) {
                    Kartu {
                        JudulBagian(
                            "Antrean Offline",
                            "${vm.antrean.size} absensi menunggu dikirim"
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
        }
    }
}

@Composable
private fun BarisKembali(judul: String, onKembali: () -> Unit) {
    Row(
        Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically
    ) {
        TextButton(onClick = onKembali) {
            Text("‹ Kembali", color = Warna.Navy)
        }
        Spacer(Modifier.width(4.dp))
        Text(
            judul,
            style = MaterialTheme.typography.titleMedium,
            color = Warna.Teks
        )
    }
}

@Composable
private fun TombolAbsen(
    teks: String,
    subteks: String,
    warna: Color,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    Button(
        onClick = onClick,
        modifier = modifier.height(64.dp),
        shape = RoundedCornerShape(12.dp),
        colors = ButtonDefaults.buttonColors(
            containerColor = warna,
            contentColor = Color.White
        )
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(teks, fontWeight = FontWeight.Bold, style = MaterialTheme.typography.bodyMedium)
            Text(subteks, style = MaterialTheme.typography.labelSmall, color = Color.White.copy(alpha = 0.8f))
        }
    }
}

@Composable
private fun KartuStatus(
    opsi: OpsiStatus,
    terpilih: Boolean,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    val warna = if (terpilih) Warna.Hijau else Warna.TeksLembut
    Surface(
        modifier = modifier.clickable { onClick() },
        shape = RoundedCornerShape(12.dp),
        color = if (terpilih) Warna.Hijau.copy(alpha = 0.08f) else Color.White,
        border = BorderStroke(1.dp, if (terpilih) Warna.Hijau else Color(0xFFCBD5E1))
    ) {
        Row(
            Modifier.padding(horizontal = 8.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            RadioButton(selected = terpilih, onClick = onClick)
            Icon(
                opsi.ikon,
                contentDescription = null,
                tint = if (terpilih) Warna.Hijau else Warna.Navy,
                modifier = Modifier.size(20.dp)
            )
            Spacer(Modifier.width(6.dp))
            Column {
                Text(
                    opsi.label,
                    style = MaterialTheme.typography.bodySmall,
                    color = if (terpilih) Warna.Hijau else Warna.Teks,
                    fontWeight = FontWeight.Medium
                )
                opsi.keterangan?.let {
                    Text(
                        it,
                        style = MaterialTheme.typography.labelSmall,
                        color = warna
                    )
                }
            }
        }
    }
}
