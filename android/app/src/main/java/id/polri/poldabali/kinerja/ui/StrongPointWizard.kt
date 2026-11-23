package id.polri.poldabali.kinerja.ui

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.graphics.BitmapFactory
import android.content.Intent
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.provider.MediaStore
import android.util.Base64
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
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
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.core.content.FileProvider
import id.polri.poldabali.kinerja.data.JadwalPhSaya
import id.polri.poldabali.kinerja.lokasi.Geocoder
import id.polri.poldabali.kinerja.lokasi.Lokasi
import id.polri.poldabali.kinerja.ui.theme.Warna
import id.polri.poldabali.kinerja.util.Foto
import kotlinx.coroutines.launch
import java.io.File
import java.time.Instant
import java.util.Locale
import java.util.UUID

private const val LANGKAH_LOKASI = 1
private const val LANGKAH_DOKUMENTASI = 2
private const val LANGKAH_KIRIM = 3
private const val HIJAU_STRONG_POINT = 0xFF2E7D32.toInt()

@Composable
fun WizardStrongPoint(vm: KinerjaViewModel, onTutup: () -> Unit) {
    val konteks = LocalContext.current
    val skop = rememberCoroutineScope()
    val jadwal = pilihJadwalHariIni(vm.jadwalPh, vm.absensiPh, vm.antreanPh)

    var langkah by rememberSaveable { mutableIntStateOf(LANGKAH_LOKASI) }
    var jenis by rememberSaveable { mutableStateOf("masuk") }
    var clientUuid by rememberSaveable { mutableStateOf(UUID.randomUUID().toString()) }
    var capturedAt by rememberSaveable { mutableStateOf(Instant.now().toString()) }
    var offline by rememberSaveable { mutableStateOf(false) }
    var gagalKirim by rememberSaveable { mutableStateOf(false) }

    var lat by remember { mutableStateOf<Double?>(null) }
    var lng by remember { mutableStateOf<Double?>(null) }
    var akurasi by remember { mutableStateOf<Double?>(null) }
    var mock by remember { mutableStateOf(false) }
    var devOptions by remember { mutableStateOf(false) }
    var aplikasiMock by remember { mutableStateOf<String?>(null) }
    var mengambilLokasi by remember { mutableStateOf(false) }
    var daring by remember { mutableStateOf(adaInternet(konteks)) }

    var alamat by remember { mutableStateOf<String?>(null) }
    var memuatAlamat by remember { mutableStateOf(false) }

    var fotoBase64 by remember { mutableStateOf<String?>(null) }
    var berkasFoto by remember { mutableStateOf<File?>(null) }
    var lewatiFoto by rememberSaveable { mutableStateOf(false) }
    val adaKamera = remember {
        Intent(MediaStore.ACTION_IMAGE_CAPTURE)
            .resolveActivity(konteks.packageManager) != null
    }

    fun punyaIzin(izin: String): Boolean =
        ContextCompat.checkSelfPermission(konteks, izin) == PackageManager.PERMISSION_GRANTED

    fun ambilLokasiSekarang() {
        if (!punyaIzin(Manifest.permission.ACCESS_FINE_LOCATION) &&
            !punyaIzin(Manifest.permission.ACCESS_COARSE_LOCATION)
        ) {
            vm.pesanGalat("Izin lokasi diperlukan untuk strong point")
            return
        }
        skop.launch {
            mengambilLokasi = true
            runCatching { Lokasi.ambil(konteks) }
                .onSuccess { posisi ->
                    lat = posisi.latitude
                    lng = posisi.longitude
                    akurasi = if (posisi.hasAccuracy()) posisi.accuracy.toDouble() else null
                    val kondisi = Lokasi.kondisi(konteks, posisi)
                    mock = kondisi.mock
                    devOptions = kondisi.developerOptions
                    aplikasiMock = kondisi.aplikasiMock
                    daring = adaInternet(konteks)
                }
                .onFailure { vm.pesanGalat(it.message ?: "Gagal mengambil lokasi") }
            mengambilLokasi = false
        }
    }

    val mintaLokasi = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { hasil ->
        if (hasil.values.any { it }) {
            ambilLokasiSekarang()
        } else {
            vm.pesanGalat("Izin lokasi diperlukan untuk strong point")
        }
    }

    fun pastikanLokasi() {
        if (punyaIzin(Manifest.permission.ACCESS_FINE_LOCATION) ||
            punyaIzin(Manifest.permission.ACCESS_COARSE_LOCATION)
        ) {
            ambilLokasiSekarang()
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
            fotoBase64 = berkasFoto?.let { Foto.keBase64Jpeg(it) }
        }
    }

    fun bukaKamera() {
        val berkas = Foto.berkasSementara(File(konteks.cacheDir, "strong-point"))
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
        else vm.pesanGalat("Izin kamera diperlukan untuk dokumentasi strong point")
    }

    LaunchedEffect(Unit) { pastikanLokasi() }

    LaunchedEffect(lat, lng) {
        val la = lat ?: return@LaunchedEffect
        val lo = lng ?: return@LaunchedEffect
        memuatAlamat = true
        alamat = Geocoder.alamat(la, lo)
        memuatAlamat = false
    }

    val status = statusStrongPoint(jadwal?.id, vm.absensiPh, vm.antreanPh)
    val totalLangkah = if (jenis == "pulang") 2 else 3
    val nomorLangkah = if (jenis == "pulang" && langkah == LANGKAH_KIRIM) 2 else langkah
    val judul = judulLangkah(nomorLangkah, jenis)
    val deskripsi = deskripsiLangkah(nomorLangkah, jenis)

    fun mulai(jenisDipilih: String, modeOffline: Boolean) {
        if (jadwal == null) return
        vm.bersihkanPesan()
        jenis = jenisDipilih
        offline = modeOffline
        gagalKirim = false
        clientUuid = UUID.randomUUID().toString()
        capturedAt = Instant.now().toString()
        fotoBase64 = null
        berkasFoto = null
        lewatiFoto = false
        langkah = if (jenisDipilih == "masuk") LANGKAH_DOKUMENTASI else LANGKAH_KIRIM
    }

    fun kirim(modeOffline: Boolean) {
        val jadwalAktif = jadwal ?: return
        vm.kirimAbsensiPh(
            clientUuid = clientUuid,
            jadwalId = jadwalAktif.id,
            jenis = jenis,
            capturedAt = capturedAt,
            latitude = lat,
            longitude = lng,
            akurasi = akurasi,
            mock = mock,
            fotoBase64 = if (jenis == "masuk") fotoBase64 else null,
            offline = modeOffline
        ) { sukses, info ->
            if (sukses) {
                vm.pesanSukses(info)
                onTutup()
            } else {
                gagalKirim = true
            }
        }
    }

    fun kembali() {
        when (langkah) {
            LANGKAH_LOKASI -> onTutup()
            LANGKAH_DOKUMENTASI -> langkah = LANGKAH_LOKASI
            else -> langkah = if (jenis == "masuk") LANGKAH_DOKUMENTASI else LANGKAH_LOKASI
        }
    }

    Column(
        Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            TextButton(onClick = ::kembali) {
                Text(if (langkah == LANGKAH_LOKASI) "Batal" else "‹ Kembali", color = Warna.Navy)
            }
            Text(
                "Langkah $nomorLangkah dari $totalLangkah",
                style = MaterialTheme.typography.labelMedium,
                color = Warna.TeksLembut
            )
        }

        Column(Modifier.fillMaxWidth()) {
            Text(judul, style = MaterialTheme.typography.titleLarge, color = Warna.Teks)
            Text(deskripsi, style = MaterialTheme.typography.bodySmall, color = Warna.TeksLembut)
        }

        LinearProgressIndicator(
            progress = { nomorLangkah / totalLangkah.toFloat() },
            modifier = Modifier.fillMaxWidth().height(6.dp),
            color = Warna.Navy,
            trackColor = Warna.NavySoft
        )

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
        vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses) }

        when (langkah) {
            LANGKAH_LOKASI -> {
                Kartu {
                    PetaAbsensi(
                        latitude = lat,
                        longitude = lng,
                        akurasiM = akurasi,
                        titik = null,
                        modifier = Modifier.fillMaxWidth(),
                        warnaMarkerLokasi = HIJAU_STRONG_POINT
                    )
                    if (mengambilLokasi) {
                        Spacer(Modifier.height(8.dp))
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(18.dp),
                                strokeWidth = 2.dp,
                                color = Warna.Navy
                            )
                            Spacer(Modifier.size(8.dp))
                            Text(
                                "Mengambil lokasi terkini…",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                    }
                    Spacer(Modifier.height(8.dp))
                    BarisInfo("Koordinat saat ini", koordinatTeks(lat, lng))
                    BarisInfo("Akurasi GPS", akurasiTeks(akurasi))
                    BarisInfo(
                        "Alamat",
                        when {
                            memuatAlamat -> "Mencari alamat…"
                            !alamat.isNullOrBlank() -> alamat!!
                            else -> "Alamat tidak tersedia"
                        }
                    )
                    if (mock) {
                        Spacer(Modifier.height(8.dp))
                        KotakPesan(
                            "Terdeteksi lokasi tiruan (mock). Strong point dapat ditolak server atau ditandai untuk verifikasi atasan.",
                            JenisPesan.Galat
                        )
                    } else if (aplikasiMock != null) {
                        Spacer(Modifier.height(8.dp))
                        KotakPesan(
                            "Aplikasi lokasi tiruan terpilih: $aplikasiMock. Pastikan aplikasi tersebut tidak aktif.",
                            JenisPesan.Info
                        )
                    } else if (devOptions) {
                        Spacer(Modifier.height(8.dp))
                        KotakPesan(
                            "Developer Options aktif. GPS asli tetap dapat dipakai; lokasi tiruan akan ditolak server.",
                            JenisPesan.Info
                        )
                    }
                    Spacer(Modifier.height(8.dp))
                    TombolKedua(
                        teks = "Perbarui Lokasi",
                        aktif = !mengambilLokasi,
                        onClick = { pastikanLokasi() }
                    )
                }

                if (jadwal == null) {
                    KotakPesan(
                        "Tidak ada jadwal strong point untuk Anda hari ini. Hubungi admin satker bila jadwal belum tersedia.",
                        JenisPesan.Info
                    )
                    TombolUtama(teks = "Tutup", onClick = onTutup)
                } else {
                    Kartu {
                        JudulBagian(
                            "Jadwal Hari Ini",
                            keteranganJadwal(jadwal.regu, jadwal.strongPointNama)
                        )
                        Spacer(Modifier.height(6.dp))
                        BarisInfo(
                            "Sesi",
                            "${if (jadwal.sesi == "sore") "Sore" else "Pagi"} ${jadwal.jamMulai}–${jadwal.jamSelesai}"
                        )
                        BarisInfo("Status", statusHariIni(status))
                    }

                    if (!daring) {
                        KotakPesan(
                            "Tidak ada koneksi internet. Pilih check-in atau check-out; data disimpan di perangkat.",
                            JenisPesan.Info
                        )
                        Row(
                            Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            if (status.sesi.none { it.jenis == "masuk" }) {
                                TombolAbsenStrong(
                                    "CHECK-IN",
                                    "(Offline)",
                                    Warna.Hijau,
                                    Modifier.weight(1f)
                                ) { mulai("masuk", true) }
                            }
                            if (status.terbuka) {
                                TombolAbsenStrong(
                                    "CHECK-OUT",
                                    "(Offline)",
                                    Warna.Merah,
                                    Modifier.weight(1f)
                                ) { mulai("pulang", true) }
                            }
                        }
                    } else if (status.terbuka) {
                        TombolAbsenStrong(
                            "CHECK-OUT",
                            "(Check Out)",
                            Warna.Merah
                        ) { mulai("pulang", false) }
                    } else if (status.sesi.none { it.jenis == "masuk" }) {
                        TombolAbsenStrong(
                            "CHECK-IN",
                            "(Check In)",
                            Warna.Hijau
                        ) { mulai("masuk", false) }
                    } else {
                        KotakPesan("Sesi strong point ini sudah selesai.", JenisPesan.Sukses)
                    }
                }
            }

            LANGKAH_DOKUMENTASI -> {
                Kartu {
                    JudulBagian(
                        "Dokumentasi Check-in",
                        if (fotoBase64 == null) "Ambil foto sebagai bukti pelaksanaan."
                        else "Foto siap dikirim bersama check-in."
                    )
                    Spacer(Modifier.height(10.dp))
                    val bitmap = remember(fotoBase64) { bitmapDariBase64(fotoBase64) }
                    if (bitmap != null) {
                        Image(
                            bitmap = bitmap,
                            contentDescription = "Dokumentasi strong point",
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(220.dp)
                                .background(Warna.NavySoft, RoundedCornerShape(12.dp)),
                            contentScale = ContentScale.Crop
                        )
                    } else {
                        Box(
                            Modifier
                                .fillMaxWidth()
                                .height(160.dp)
                                .background(Warna.NavySoft, RoundedCornerShape(12.dp)),
                            contentAlignment = Alignment.Center
                        ) {
                            Text(
                                "Belum ada foto",
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                    }
                    Spacer(Modifier.height(10.dp))
                    if (adaKamera) {
                        TombolKedua(
                            teks = if (fotoBase64 == null) "Ambil Foto" else "Ganti Foto",
                            onClick = {
                                if (punyaIzin(Manifest.permission.CAMERA)) bukaKamera()
                                else mintaKamera.launch(Manifest.permission.CAMERA)
                            }
                        )
                        if (fotoBase64 == null) {
                            Spacer(Modifier.height(8.dp))
                            KotakPesan(
                                "Foto dokumentasi disarankan sebagai bukti pelaksanaan strong point.",
                                JenisPesan.Info
                            )
                            TextButton(onClick = { lewatiFoto = true }) {
                                Text("Lewati dokumentasi", color = Warna.TeksLembut)
                            }
                        }
                    } else {
                        KotakPesan(
                            "Kamera tidak tersedia di perangkat ini. Dokumentasi dapat dilewati.",
                            JenisPesan.Info
                        )
                    }
                }

                TombolUtama(
                    teks = "Lanjut ke Kirim",
                    aktif = fotoBase64 != null || lewatiFoto || !adaKamera,
                    onClick = { langkah = LANGKAH_KIRIM }
                )
            }

            else -> {
                Kartu {
                    JudulBagian(
                        labelJenis(jenis),
                        if (jenis == "masuk") "Check-in dengan dokumentasi"
                        else "Check-out tanpa dokumentasi"
                    )
                    Spacer(Modifier.height(8.dp))
                    BarisInfo(
                        "Waktu",
                        runCatching { jamWita(Instant.parse(capturedAt)) }.getOrDefault("-")
                    )
                    BarisInfo("Koordinat", koordinatTeks(lat, lng))
                    BarisInfo("Akurasi GPS", akurasiTeks(akurasi))
                    BarisInfo("Alamat", alamat?.takeIf { it.isNotBlank() } ?: "Alamat tidak tersedia")
                    if (jenis == "masuk" && fotoBase64 != null) {
                        Spacer(Modifier.height(10.dp))
                        val bitmap = remember(fotoBase64) { bitmapDariBase64(fotoBase64) }
                        if (bitmap != null) {
                            Image(
                                bitmap = bitmap,
                                contentDescription = "Dokumentasi strong point",
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .height(180.dp)
                                    .background(Warna.NavySoft, RoundedCornerShape(12.dp)),
                                contentScale = ContentScale.Crop
                            )
                        }
                    }
                    if (offline) {
                        Spacer(Modifier.height(8.dp))
                        KotakPesan(
                            "Mode offline: data strong point disimpan di perangkat lalu dikirim otomatis saat sinyal tersedia.",
                            JenisPesan.Info
                        )
                    }
                }

                TombolUtama(
                    teks = when {
                        offline -> "Kirim Strong Point (Offline)"
                        gagalKirim -> "Coba Kirim Lagi"
                        else -> "Kirim Strong Point"
                    },
                    memuat = vm.sedangMemproses,
                    aktif = lat != null && lng != null &&
                            (jenis != "masuk" || fotoBase64 != null || lewatiFoto || !adaKamera),
                    onClick = { kirim(offline) }
                )

                if (gagalKirim && !offline) {
                    TombolKedua(
                        teks = "Kirim Secara Offline",
                        aktif = !vm.sedangMemproses,
                        onClick = {
                            offline = true
                            gagalKirim = false
                            kirim(true)
                        }
                    )
                }
            }
        }
    }
}

@Composable
private fun TombolAbsenStrong(
    teks: String,
    subteks: String,
    warna: Color,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    Button(
        onClick = onClick,
        modifier = modifier.fillMaxWidth().height(64.dp),
        shape = RoundedCornerShape(14.dp),
        colors = ButtonDefaults.buttonColors(containerColor = warna)
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(teks, fontWeight = FontWeight.Bold, style = MaterialTheme.typography.bodyMedium)
            Text(
                subteks,
                style = MaterialTheme.typography.labelSmall,
                color = Color.White.copy(alpha = 0.8f)
            )
        }
    }
}

private fun judulLangkah(nomor: Int, jenis: String): String = when (nomor) {
    1 -> "Lokasi Strong Point"
    2 -> if (jenis == "masuk") "Dokumentasi Strong Point" else "Kirim Strong Point"
    else -> "Kirim Strong Point"
}

private fun deskripsiLangkah(nomor: Int, jenis: String): String = when (nomor) {
    1 -> "Pastikan titik GPS Anda sudah tepat sebelum check-in atau check-out."
    2 -> if (jenis == "masuk") {
        "Ambil foto sebagai bukti pelaksanaan strong point."
    } else {
        "Periksa kembali data check-out lalu kirim."
    }

    else -> "Periksa kembali data strong point lalu kirim."
}

private fun statusHariIni(status: StatusStrongPoint): String = when {
    status.sesi.isEmpty() -> "Belum check-in hari ini"
    status.terbuka -> "Check-in pukul ${status.masukTerakhir?.jam.orEmpty()}"
    else -> "Sudah check-out pukul ${status.pulangTerakhir?.jam.orEmpty()}"
}

private fun adaInternet(context: Context): Boolean {
    val manajer = context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
    val jaringan = manajer.activeNetwork ?: return false
    val kemampuan = manajer.getNetworkCapabilities(jaringan) ?: return false
    return kemampuan.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
}

private fun koordinatTeks(lat: Double?, lng: Double?): String =
    if (lat == null || lng == null) {
        "-"
    } else {
        String.format(Locale.US, "%.6f, %.6f", lat, lng)
    }

private fun akurasiTeks(nilai: Double?): String =
    nilai?.let { String.format(Locale.US, "±%.0f m", it) } ?: "-"

private fun bitmapDariBase64(teks: String?) =
    teks?.let {
        runCatching {
            val bytes = Base64.decode(it, Base64.DEFAULT)
            BitmapFactory.decodeByteArray(bytes, 0, bytes.size)?.asImageBitmap()
        }.getOrNull()
    }
