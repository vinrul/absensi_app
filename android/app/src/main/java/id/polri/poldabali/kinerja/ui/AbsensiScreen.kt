package id.polri.poldabali.kinerja.ui

import android.Manifest
import android.content.pm.PackageManager
import android.location.Location
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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.DateRange
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Place
import androidx.compose.material.icons.filled.Warning
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
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.core.content.FileProvider
import id.polri.poldabali.kinerja.data.Absensi
import id.polri.poldabali.kinerja.data.AbsensiPh
import id.polri.poldabali.kinerja.data.JadwalPhSaya
import id.polri.poldabali.kinerja.data.KonteksAbsensi
import id.polri.poldabali.kinerja.data.KonteksStrong
import id.polri.poldabali.kinerja.data.TitikDinas
import id.polri.poldabali.kinerja.lokasi.Lokasi
import id.polri.poldabali.kinerja.ui.theme.Warna
import id.polri.poldabali.kinerja.util.Foto
import kotlinx.coroutines.launch
import java.io.File
import java.time.Instant
import java.util.Locale
import java.util.UUID

private const val HALAMAN_BERANDA = 0
private const val HALAMAN_STATUS = 1
private const val HALAMAN_LOKASI = 2
private const val HALAMAN_FOTO = 3
private const val HALAMAN_RIWAYAT = 4

private data class OpsiStatus(
    val nilai: String,
    val label: String,
    val ikon: ImageVector,
    val keterangan: String? = null
)

private val STATUS_KEHADIRAN = listOf(
    OpsiStatus("dinas_kantor", "Dinas Kantor", Icons.Filled.Home, "Wajib di dalam radius kantor"),
    OpsiStatus("dinas_luar", "Dinas Luar", Icons.Filled.Place, "Catat lokasi, cukup check-in"),
    OpsiStatus("ph", "PH / Strong Point", Icons.Filled.Person, "Sesuai jadwal strong point"),
    OpsiStatus("piket", "Piket / Shift", Icons.Filled.Lock, "Wajib check-in dan check-out"),
    OpsiStatus("operasi", "Operasi / Pam", Icons.Filled.Warning, "Boleh di luar radius kantor"),
    OpsiStatus("cuti", "Cuti", Icons.Filled.DateRange, "Cukup satu catatan per hari"),
    OpsiStatus("izin", "Izin", Icons.AutoMirrored.Filled.List, "Cukup satu catatan per hari"),
    OpsiStatus("sakit", "Sakit", Icons.Filled.Favorite, "Cukup satu catatan per hari")
)

private data class SesiStrongRiwayat(
    val sesi: String,
    val jadwal: String,
    val titik: String,
    val masuk: String?,
    val pulang: String?
)

private data class StatusRiwayat(
    val label: String,
    val masuk: List<String>,
    val pulang: List<String>
)

private data class BarisRiwayatHarian(
    val tanggal: String,
    val status: List<StatusRiwayat>,
    val strong: List<SesiStrongRiwayat>
)

@Composable
fun AbsensiScreen(vm: KinerjaViewModel) {
    val konteksApp = LocalContext.current
    val skop = rememberCoroutineScope()

    var halaman by rememberSaveable { mutableIntStateOf(HALAMAN_BERANDA) }
    var status by rememberSaveable { mutableStateOf<String?>(null) }
    var jenis by rememberSaveable { mutableStateOf("masuk") }
    var jadwalId by rememberSaveable { mutableStateOf<String?>(null) }
    var checkout by rememberSaveable { mutableStateOf(false) }
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
    val muatKonteks = vm.memuatKonteksAbsensi
    val sesiTerpilih = vm.konteksAbsensi?.jadwalStrong?.firstOrNull { it.jadwalPhId == jadwalId }
    val titikPeta = titikUntukPeta(vm, status, sesiTerpilih)
    val jarakStrong = if (lat != null && lng != null && sesiTerpilih != null) {
        jarakMeter(lat!!, lng!!, sesiTerpilih.latitude, sesiTerpilih.longitude)
    } else {
        null
    }
    val dalamRadiusKantor = vm.titikTerdekat?.terdekat?.dalamRadius == true
    val dalamRadiusStrong = jarakStrong != null && jarakStrong <= (sesiTerpilih?.radiusM ?: 0)
    val dalamRadius = if (status == "ph") dalamRadiusStrong else dalamRadiusKantor
    val butuhRadius = status == "dinas_kantor" || status == "piket" || status == "ph"
    val butuhLokasi = status == null || status !in listOf("cuti", "izin", "sakit")
    val jarakTampil = if (status == "ph") jarakStrong else vm.titikTerdekat?.terdekat?.jarakM
    val radiusTampil = if (status == "ph") sesiTerpilih?.radiusM else vm.titikTerdekat?.terdekat?.radiusM
    val namaTitik = if (status == "ph") sesiTerpilih?.strongPointNama else titikPeta?.nama

    fun punya(izin: String) =
        ContextCompat.checkSelfPermission(konteksApp, izin) == PackageManager.PERMISSION_GRANTED

    suspend fun ambilLokasi() {
        mengambilLokasi = true
        try {
            val lokasi = Lokasi.ambil(konteksApp)
            lat = lokasi.latitude
            lng = lokasi.longitude
            akurasi = lokasi.accuracy.toDouble()
            val kondisi = Lokasi.kondisi(konteksApp, lokasi)
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

    val ambilFoto = rememberLauncherForActivityResult(
        ActivityResultContracts.TakePicture()
    ) { sukses ->
        if (sukses) {
            fotoBase64 = Foto.keBase64Jpeg(berkasFoto!!)
        }
    }

    fun bukaKamera() {
        val berkas = Foto.berkasSementara(File(konteksApp.cacheDir, "absensi"))
        berkasFoto = berkas
        val uri = FileProvider.getUriForFile(
            konteksApp,
            "${konteksApp.packageName}.fileprovider",
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

    fun resetKeBeranda() {
        vm.muatKonteksAbsensi()
        vm.muatAbsensiHariIni()
        status = null
        jenis = "masuk"
        jadwalId = null
        checkout = false
        fotoBase64 = null
        berkasFoto = null
        halaman = HALAMAN_BERANDA
    }

    fun sesiMasukBerikutnya(k: KonteksAbsensi): KonteksStrong? =
        k.jadwalStrong.firstOrNull { it.bisaMasuk }

    fun sesiTerbuka(k: KonteksAbsensi): KonteksStrong? =
        k.jadwalStrong.firstOrNull { it.sudahMasuk && !it.sudahPulang }

    fun mulai() {
        vm.bersihkanPesan()
        vm.muatKonteksAbsensi()
        val k = vm.konteksAbsensi
        if (k == null) {
            vm.pesanGalat("Konteks absensi belum tersedia. Periksa koneksi lalu coba lagi.")
            return
        }
        if (k.aksi == "selesai") {
            vm.pesanGalat("Absensi hari ini sudah lengkap.")
            return
        }
        if (k.aksi == "checkout") {
            checkout = true
            jenis = "pulang"
            status = k.statusTerkunci
            jadwalId = null
            halaman = HALAMAN_LOKASI
            if (lat == null) mintaAtauAmbilLokasi()
            return
        }

        if (k.strongTerbuka) {
            val baru = sesiMasukBerikutnya(k)
            val terbuka = sesiTerbuka(k)
            if (baru != null) {
                checkout = false
                jenis = "masuk"
                status = "ph"
                jadwalId = baru.jadwalPhId
                vm.pesanSukses(
                    "Check-in sesi ${labelSesi(baru)}. Sesi sebelumnya yang belum check-out ditandai tidak check-out."
                )
                halaman = HALAMAN_LOKASI
                if (lat == null) mintaAtauAmbilLokasi()
                return
            }
            if (terbuka != null) {
                checkout = true
                jenis = "pulang"
                status = "ph"
                jadwalId = terbuka.jadwalPhId
                halaman = HALAMAN_LOKASI
                if (lat == null) mintaAtauAmbilLokasi()
                return
            }
        }

        status = null
        jenis = "masuk"
        jadwalId = null
        checkout = false
        halaman = HALAMAN_STATUS
    }

    fun pilihStatus(nilai: String) {
        if (!opsiAktif(nilai, vm.konteksAbsensi)) return
        status = nilai
        checkout = false
        jenis = "masuk"
        jadwalId = if (nilai == "ph") {
            vm.konteksAbsensi?.jadwalStrong?.firstOrNull { it.bisaMasuk }?.jadwalPhId
        } else {
            null
        }
        if (nilai == "ph" && jadwalId == null) {
            vm.pesanGalat("Tidak ada sesi strong point yang bisa check-in sekarang.")
            return
        }
        halaman = HALAMAN_LOKASI
        if (lat == null) mintaAtauAmbilLokasi()
    }

    LaunchedEffect(Unit) {
        vm.muatAbsensiHariIni()
        vm.muatKonteksAbsensi()
    }

    LaunchedEffect(halaman) {
        if (halaman == HALAMAN_RIWAYAT) vm.muatRiwayatBulanIni()
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
                        .height(124.dp),
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
                            "Pilih status, ambil lokasi, lalu kirim",
                            style = MaterialTheme.typography.bodySmall,
                            color = Warna.NavySoft
                        )
                    }
                }

                Kartu {
                    JudulBagian("Status Absensi", ringkasanKonteks(vm.konteksAbsensi, muatKonteks))
                    vm.konteksAbsensi?.catatan?.let {
                        Spacer(Modifier.height(6.dp))
                        Text(it, style = MaterialTheme.typography.bodySmall, color = Warna.TeksLembut)
                    }
                    vm.konteksAbsensi?.let { k ->
                        if (k.adaJadwalStrong) {
                            Spacer(Modifier.height(8.dp))
                            k.jadwalStrong.forEach { sesi ->
                                BarisInfo(
                                    labelSesi(sesi),
                                    buildString {
                                        append(sesi.strongPointNama)
                                        when {
                                            sesi.sudahPulang -> append(" · selesai")
                                            sesi.sudahMasuk -> append(" · sudah check-in")
                                            sesi.bisaMasuk -> append(" · siap check-in")
                                            else -> append(" · di luar jendela")
                                        }
                                    }
                                )
                            }
                        }
                        Spacer(Modifier.height(10.dp))
                        when (k.aksi) {
                            "selesai" -> KotakPesan(
                                "Absensi hari ini sudah lengkap. Scan berikutnya baru dapat dilakukan besok.",
                                JenisPesan.Sukses
                            )

                            "checkout" -> {
                                TombolUtama("Check-out Sekarang", memuat = muatKonteks) { mulai() }
                                Text(
                                    "Check-out tidak perlu memilih status lagi.",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = Warna.TeksLembut,
                                    modifier = Modifier.padding(top = 6.dp)
                                )
                            }

                            else -> {
                                if (k.strongTerbuka && sesiMasukBerikutnya(k) == null) {
                                    KotakPesan(
                                        "Sesi strong point belum check-out. Check-out tidak perlu memilih status lagi.",
                                        JenisPesan.Info
                                    )
                                    Spacer(Modifier.height(8.dp))
                                    TombolUtama("Check-out Strong Point", memuat = muatKonteks) {
                                        mulai()
                                    }
                                } else {
                                    if (k.strongTerbuka && sesiMasukBerikutnya(k) != null) {
                                        KotakPesan(
                                            "Sesi strong point berikutnya sudah dibuka. Sesi sebelumnya yang belum check-out akan ditandai tidak check-out.",
                                            JenisPesan.Info
                                        )
                                        Spacer(Modifier.height(8.dp))
                                    }
                                    TombolUtama("Mulai Absensi", memuat = muatKonteks) { mulai() }
                                }
                            }
                        }
                    }
                    if (vm.konteksAbsensi == null && !muatKonteks) {
                        Text(
                            "Konteks absensi tidak dapat dimuat. Periksa koneksi lalu muat ulang.",
                            style = MaterialTheme.typography.bodySmall,
                            color = Warna.Merah
                        )
                        Spacer(Modifier.height(8.dp))
                        TombolKedua("Muat Ulang") { vm.muatKonteksAbsensi() }
                    }
                }

                Kartu {
                    val h = hariIni
                    val keterangan = when {
                        vm.memuatAbsensiHariIni && h == null -> "Memeriksa absensi hari ini…"
                        vm.absensiHariIniOffline -> "Tidak dapat memeriksa absensi hari ini (offline)"
                        h != null && h.masuk != null && h.pulang != null -> "Absensi hari ini sudah lengkap"
                        h?.masuk != null -> "Sudah absen masuk pukul ${h.masuk.jam}"
                        else -> "Belum ada absensi hari ini"
                    }
                    JudulBagian("Status Hari Ini", keterangan)
                    if (vm.absensiHariIniOffline) {
                        Spacer(Modifier.height(8.dp))
                        KotakPesan(
                            "Status tidak dapat diambil dari server. Absensi dapat disimpan offline dan dikirim otomatis.",
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

            HALAMAN_STATUS -> {
                BarisKembali("Pilih Status Kehadiran") {
                    vm.bersihkanPesan()
                    halaman = HALAMAN_BERANDA
                }

                Kartu {
                    JudulBagian(
                        "Status Kehadiran",
                        "Pilih status sebelum mengambil lokasi dan foto"
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
                                    aktif = opsiAktif(opsi.nilai, vm.konteksAbsensi),
                                    alasan = alasanTidakAktif(opsi.nilai, vm.konteksAbsensi),
                                    modifier = Modifier.weight(1f)
                                ) {
                                    pilihStatus(opsi.nilai)
                                }
                            }
                            if (baris.size == 1) Spacer(Modifier.weight(1f))
                        }
                        Spacer(Modifier.height(8.dp))
                    }

                    if (vm.absensiHariIniOffline && vm.konteksAbsensi == null) {
                        Spacer(Modifier.height(4.dp))
                        KotakPesan(
                            "Server tidak terjangkau. Pilih status dan tentukan check-in/check-out secara manual.",
                            JenisPesan.Info
                        )
                        Spacer(Modifier.height(8.dp))
                        Row(
                            Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text(
                                if (checkout) "Mode check-out" else "Mode check-in",
                                style = MaterialTheme.typography.bodyMedium,
                                color = Warna.Teks
                            )
                            Switch(checked = checkout, onCheckedChange = { checkout = it })
                        }
                    }
                }
            }

            HALAMAN_LOKASI -> {
                BarisKembali("Validasi Lokasi (GPS)") {
                    vm.bersihkanPesan()
                    halaman = if (checkout || vm.konteksAbsensi?.aksi == "checkout") {
                        HALAMAN_BERANDA
                    } else {
                        HALAMAN_STATUS
                    }
                }

                Kartu {
                    PetaAbsensi(
                        latitude = lat,
                        longitude = lng,
                        akurasiM = akurasi,
                        titik = titikPeta,
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
                        teks = if (lat == null) "Ambil Lokasi" else "Perbarui Lokasi",
                        aktif = !mengambilLokasi,
                        onClick = { mintaAtauAmbilLokasi() }
                    )
                }

                if (lat != null) {
                    val warna = when {
                        !butuhRadius -> Warna.Biru
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
                                    if (!butuhRadius || dalamRadius) {
                                        Icons.Filled.CheckCircle
                                    } else {
                                        Icons.Filled.Warning
                                    },
                                    contentDescription = null,
                                    tint = warna,
                                    modifier = Modifier.size(24.dp)
                                )
                                Spacer(Modifier.width(8.dp))
                                Text(
                                    when {
                                        !butuhRadius -> "Lokasi Tercatat"
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
                                    !butuhRadius ->
                                        "Status ini tidak mewajibkan absensi di dalam radius kantor."

                                    dalamRadius ->
                                        "Anda berada dalam radius ${radiusTampil ?: 100} meter dari ${namaTitik ?: "titik absensi"}."

                                    vm.absensiHariIniOffline ->
                                        "Lokasi belum dapat diverifikasi karena server tidak terjangkau."

                                    else ->
                                        "Absensi wajib di dalam radius ${namaTitik ?: "titik absensi"}. Dekati lokasi lalu perbarui."
                                },
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                            Spacer(Modifier.height(8.dp))
                            BarisInfo(
                                "Status",
                                if (checkout) {
                                    "Check-out ${labelStatusKehadiran(status ?: "-")}"
                                } else {
                                    "Check-in ${labelStatusKehadiran(status ?: "-")}"
                                }
                            )
                            if (status == "ph" && sesiTerpilih != null) {
                                BarisInfo("Sesi", labelSesi(sesiTerpilih))
                            }
                            BarisInfo(
                                "Koordinat Anda",
                                if (lat != null && lng != null) {
                                    String.format(Locale.US, "%.6f, %.6f", lat, lng)
                                } else {
                                    "-"
                                }
                            )
                            BarisInfo(
                                "Jarak",
                                jarakTampil?.let { String.format(Locale.US, "%.0f meter", it) }
                                    ?: "Menghitung…"
                            )
                            BarisInfo(
                                "Akurasi GPS",
                                akurasi?.let { String.format(Locale.US, "± %.0f meter", it) } ?: "-"
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
                } else {
                    KotakPesan(
                        "Ambil lokasi terlebih dahulu untuk melanjutkan absensi.",
                        JenisPesan.Info
                    )
                }

                val bolehLanjut = !mengambilLokasi &&
                        (lat != null || !butuhLokasi) &&
                        (!butuhRadius || dalamRadius || vm.absensiHariIniOffline) &&
                        !(status == "ph" && jadwalId == null)
                TombolUtama(
                    teks = "Lanjut ke Foto",
                    aktif = bolehLanjut,
                    onClick = { halaman = HALAMAN_FOTO }
                )
            }

            HALAMAN_FOTO -> {
                BarisKembali("Foto & Kirim") {
                    vm.bersihkanPesan()
                    halaman = HALAMAN_LOKASI
                }

                Kartu {
                    JudulBagian(
                        if (checkout) "Konfirmasi Check-out" else "Konfirmasi Check-in",
                        "Periksa data sebelum mengirim"
                    )
                    Spacer(Modifier.height(8.dp))
                    BarisInfo("Aksi", if (checkout) "Check-out" else "Check-in")
                    BarisInfo("Status", labelStatusKehadiran(status ?: "-"))
                    if (status == "ph" && sesiTerpilih != null) {
                        BarisInfo("Sesi", labelSesi(sesiTerpilih))
                        BarisInfo("Titik", sesiTerpilih.strongPointNama)
                    } else if (namaTitik != null) {
                        BarisInfo("Titik", namaTitik)
                    }
                    BarisInfo(
                        "Lokasi",
                        if (lat != null && lng != null) {
                            String.format(Locale.US, "%.6f, %.6f", lat, lng)
                        } else {
                            "Tidak dilampirkan"
                        }
                    )
                }

                Kartu {
                    JudulBagian("Dokumentasi", "Foto bukti, maksimal 5 MB")
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

                if (vm.absensiHariIniOffline && vm.konteksAbsensi == null) {
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
                                    "Absensi disimpan di perangkat lalu dikirim otomatis",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = Warna.TeksLembut
                                )
                            }
                            Switch(checked = offline, onCheckedChange = { offline = it })
                        }
                    }
                }

                val bolehKirim = status != null &&
                        (checkout || status != "ph" || jadwalId != null) &&
                        (lat != null || !butuhLokasi) &&
                        (!butuhRadius || dalamRadius || vm.absensiHariIniOffline) &&
                        !vm.sedangMemproses

                TombolUtama(
                    teks = if (offline) "Simpan Absensi (Offline)" else "Kirim Absensi",
                    memuat = vm.sedangMemproses,
                    aktif = bolehKirim,
                    onClick = {
                        val finalStatus = status ?: return@TombolUtama
                        val finalJenis = if (checkout) "pulang" else "masuk"
                        if (finalStatus == "ph") {
                            val idJadwal = jadwalId
                            if (idJadwal == null) {
                                vm.pesanGalat("Sesi strong point tidak ditemukan.")
                                return@TombolUtama
                            }
                            vm.kirimAbsensiPh(
                                clientUuid = UUID.randomUUID().toString(),
                                jadwalId = idJadwal,
                                jenis = finalJenis,
                                capturedAt = Instant.now().toString(),
                                latitude = lat,
                                longitude = lng,
                                akurasi = akurasi,
                                mock = mock,
                                fotoBase64 = fotoBase64,
                                offline = false
                            ) { sukses, info ->
                                if (sukses) {
                                    vm.pesanSukses(info)
                                    resetKeBeranda()
                                } else {
                                    vm.pesanGalat(info)
                                }
                            }
                        } else {
                            vm.kirimAbsensi(
                                jenis = finalJenis,
                                statusKehadiran = finalStatus,
                                latitude = lat,
                                longitude = lng,
                                akurasi = akurasi,
                                mock = mock,
                                fotoBase64 = fotoBase64,
                                offline = offline
                            ) { info ->
                                vm.pesanSukses(info)
                                resetKeBeranda()
                            }
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
                    val barisRiwayat = remember(vm.riwayat, vm.jadwalPh, vm.absensiPh) {
                        riwayatHarian(vm.riwayat, vm.jadwalPh, vm.absensiPh)
                    }
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
                                "${barisRiwayat.size} tanggal",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                        TextButton(onClick = { vm.muatRiwayatBulanIni() }) {
                            Text("Muat ulang", color = Warna.Navy)
                        }
                    }
                    Spacer(Modifier.height(6.dp))

                    if (barisRiwayat.isEmpty()) {
                        LayarKosong("Belum ada absensi atau jadwal strong point pada bulan ini.")
                    } else {
                        barisRiwayat.forEachIndexed { indeks, item ->
                            Column(
                                Modifier.fillMaxWidth().padding(vertical = 9.dp),
                                verticalArrangement = Arrangement.spacedBy(3.dp)
                            ) {
                                Text(
                                    tanggalIndo(item.tanggal),
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = Warna.Teks,
                                    fontWeight = FontWeight.Medium
                                )
                                item.status.forEach { status ->
                                    Text(
                                        status.label,
                                        style = MaterialTheme.typography.bodySmall,
                                        color = Warna.TeksLembut
                                    )
                                    if (status.masuk.isNotEmpty() || status.pulang.isNotEmpty()) {
                                        Text(
                                            buildString {
                                                append("Check-in: ")
                                                append(status.masuk.joinToString(", ").ifEmpty { "-" })
                                                append("   ·   Check-out: ")
                                                append(status.pulang.joinToString(", ").ifEmpty { "-" })
                                            },
                                            style = MaterialTheme.typography.bodySmall,
                                            color = Warna.Teks
                                        )
                                    }
                                }
                                item.strong.forEach { sesi ->
                                    Text(
                                        buildString {
                                            append("Strong Point ${sesi.sesi} ${sesi.jadwal}")
                                            append(" · ${sesi.titik}")
                                            append(" · ")
                                            append(
                                                when {
                                                    sesi.masuk == null -> "belum absen"
                                                    sesi.pulang == null ->
                                                        "${sesi.masuk} → belum check-out"

                                                    else -> "${sesi.masuk} → ${sesi.pulang}"
                                                }
                                            )
                                        },
                                        style = MaterialTheme.typography.bodySmall,
                                        color = if (sesi.masuk != null) Warna.Hijau else Warna.TeksLembut
                                    )
                                }
                                if (indeks < barisRiwayat.lastIndex) {
                                    Spacer(Modifier.height(2.dp))
                                    Box(
                                        Modifier
                                            .fillMaxWidth()
                                            .height(1.dp)
                                            .background(Color(0xFFE2E8F0))
                                    )
                                }
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

private fun riwayatHarian(
    riwayat: List<Absensi>,
    jadwalPh: List<JadwalPhSaya>,
    absensiPh: List<AbsensiPh>
): List<BarisRiwayatHarian> {
    val absensiPerTanggal = riwayat.distinctBy { it.id }.groupBy { it.tanggal }
    val jadwalUnik = jadwalPh.distinctBy { it.id }
    val jadwalPerTanggal = jadwalUnik.groupBy { it.tanggal }
    val jadwalById = jadwalUnik.associateBy { it.id }
    val absensiPhPerJadwal = absensiPh.distinctBy { it.id }.groupBy { it.jadwalPhId }
    val semuaTanggal = sortedSetOf<String>()
    semuaTanggal.addAll(absensiPerTanggal.keys)
    semuaTanggal.addAll(jadwalPerTanggal.keys)
    for (catatan in absensiPh) {
        jadwalById[catatan.jadwalPhId]?.let { semuaTanggal.add(it.tanggal) }
    }

    return semuaTanggal.sortedDescending().map { tanggal ->
        val catatan = absensiPerTanggal[tanggal].orEmpty()
        val jadwal = jadwalPerTanggal[tanggal].orEmpty()
        val adaStrong = jadwal.isNotEmpty()
        // Cermin absensi strong point tidak diulang sebagai absensi kantor.
        val utama = catatan.filter { !adaStrong || it.statusKehadiran != "ph" }
        val status = utama
            .groupBy { it.statusKehadiran }
            .map { (kode, daftar) ->
                StatusRiwayat(
                    label = labelStatusKehadiran(kode),
                    masuk = daftar.filter { it.jenis == "masuk" }.map { it.jam }.distinct().sorted(),
                    pulang = daftar.filter { it.jenis == "pulang" }.map { it.jam }.distinct().sorted()
                )
            }
            .toMutableList()
        if (adaStrong) status.add(StatusRiwayat("PH / Strong Point", emptyList(), emptyList()))
        val strong = jadwal
            .sortedWith(compareBy<JadwalPhSaya>({ it.jamMulai }, { it.sesi.orEmpty() }))
            .map { item ->
                val sesi = absensiPhPerJadwal[item.id].orEmpty()
                SesiStrongRiwayat(
                    sesi = when (item.sesi) {
                        "pagi" -> "Pagi"
                        "sore" -> "Sore"
                        else -> "Sesi"
                    },
                    jadwal = "${item.jamMulai}–${item.jamSelesai}",
                    titik = item.strongPointNama,
                    masuk = sesi.filter { it.jenis == "masuk" }.minByOrNull { it.jam }?.jam,
                    pulang = sesi.filter { it.jenis == "pulang" }.maxByOrNull { it.jam }?.jam
                )
            }
        BarisRiwayatHarian(
            tanggal = tanggal,
            status = status,
            strong = strong
        )
    }
}

private fun ringkasanKonteks(konteks: KonteksAbsensi?, memuat: Boolean): String = when {
    memuat && konteks == null -> "Memeriksa status absensi…"
    konteks == null -> "Konteks absensi tidak tersedia"
    konteks.aksi == "selesai" -> "Absensi hari ini sudah lengkap"
    konteks.aksi == "checkout" -> "Siap check-out ${labelStatusKehadiran(konteks.statusTerkunci ?: "-")}"
    konteks.strongTerbuka && konteks.jadwalStrong.any { it.bisaMasuk } ->
        "Sesi strong point berikutnya siap check-in"

    konteks.strongTerbuka -> "Sesi strong point belum check-out"
    konteks.adaJadwalStrong -> "Hari ini ada jadwal strong point"
    else -> "Siap check-in"
}

private fun opsiAktif(nilai: String, konteks: KonteksAbsensi?): Boolean {
    if (konteks == null) return nilai != "ph"
    if (konteks.aksi == "selesai") return false
    return when (nilai) {
        "dinas_kantor" -> !konteks.adaJadwalStrong && dalamJamKantor(konteks)
        "ph" -> konteks.jadwalStrong.any { it.bisaMasuk }
        else -> true
    }
}

private fun alasanTidakAktif(nilai: String, konteks: KonteksAbsensi?): String? {
    if (konteks == null) return if (nilai == "ph") "Butuh koneksi ke server" else null
    if (konteks.aksi == "selesai") return "Absensi hari ini sudah lengkap"
    return when (nilai) {
        "dinas_kantor" -> when {
            konteks.adaJadwalStrong -> "Hari ini wajib strong point"
            !dalamJamKantor(konteks) -> "Di luar jam ${konteks.bukaKantor}–${konteks.tutupKantor}"
            else -> null
        }

        "ph" -> if (konteks.jadwalStrong.any { it.bisaMasuk }) null else "Belum ada sesi yang dibuka"
        else -> null
    }
}

private fun dalamJamKantor(konteks: KonteksAbsensi): Boolean =
    konteks.jam >= konteks.bukaKantor && konteks.jam <= konteks.tutupKantor

private fun titikUntukPeta(
    vm: KinerjaViewModel,
    status: String?,
    sesi: KonteksStrong?
): TitikDinas? {
    if (status == "ph" && sesi != null) {
        return TitikDinas(
            id = sesi.jadwalPhId,
            nama = sesi.strongPointNama,
            jenis = "kantor",
            latitude = sesi.latitude,
            longitude = sesi.longitude,
            radiusM = sesi.radiusM
        )
    }
    return vm.titikTerdekat?.titik?.minByOrNull { it.jarakM ?: Double.MAX_VALUE }
}

private fun labelSesi(sesi: KonteksStrong): String {
    val nama = when (sesi.sesi) {
        "sore" -> "Sore"
        "pagi" -> "Pagi"
        else -> "Sesi"
    }
    return "$nama ${sesi.jamMulai}–${sesi.jamSelesai}"
}

private fun jarakMeter(lat1: Double, lng1: Double, lat2: Double, lng2: Double): Double {
    val hasil = FloatArray(1)
    Location.distanceBetween(lat1, lng1, lat2, lng2, hasil)
    return hasil[0].toDouble()
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
private fun KartuStatus(
    opsi: OpsiStatus,
    terpilih: Boolean,
    aktif: Boolean,
    alasan: String?,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    val warna = when {
        terpilih -> Warna.Hijau
        !aktif -> Warna.TeksLembut
        else -> Warna.TeksLembut
    }
    Surface(
        modifier = modifier.clickable(enabled = aktif) { onClick() },
        shape = RoundedCornerShape(12.dp),
        color = when {
            terpilih -> Warna.Hijau.copy(alpha = 0.08f)
            !aktif -> Color(0xFFF1F5F9)
            else -> Color.White
        },
        border = BorderStroke(
            1.dp,
            when {
                terpilih -> Warna.Hijau
                !aktif -> Color(0xFFE2E8F0)
                else -> Color(0xFFCBD5E1)
            }
        )
    ) {
        Row(
            Modifier.padding(horizontal = 8.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            RadioButton(selected = terpilih, onClick = if (aktif) onClick else null, enabled = aktif)
            Icon(
                opsi.ikon,
                contentDescription = null,
                tint = when {
                    terpilih -> Warna.Hijau
                    !aktif -> Color(0xFF94A3B8)
                    else -> Warna.Navy
                },
                modifier = Modifier.size(20.dp)
            )
            Spacer(Modifier.width(6.dp))
            Column {
                Text(
                    opsi.label,
                    style = MaterialTheme.typography.bodySmall,
                    color = when {
                        terpilih -> Warna.Hijau
                        !aktif -> Color(0xFF94A3B8)
                        else -> Warna.Teks
                    },
                    fontWeight = FontWeight.Medium
                )
                (alasan ?: opsi.keterangan)?.let {
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
