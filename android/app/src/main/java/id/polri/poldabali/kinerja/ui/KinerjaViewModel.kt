package id.polri.poldabali.kinerja.ui

import android.app.Application
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import id.polri.poldabali.kinerja.data.Absensi
import id.polri.poldabali.kinerja.data.AbsensiAntrean
import id.polri.poldabali.kinerja.data.AbsensiHariIni
import id.polri.poldabali.kinerja.data.AbsensiPh
import id.polri.poldabali.kinerja.data.AbsensiPhRequest
import id.polri.poldabali.kinerja.data.AbsensiRequest
import id.polri.poldabali.kinerja.data.ApiException
import id.polri.poldabali.kinerja.data.BerkasLampiran
import id.polri.poldabali.kinerja.data.Graf
import id.polri.poldabali.kinerja.data.Hasil
import id.polri.poldabali.kinerja.data.HasilRequest
import id.polri.poldabali.kinerja.data.JadwalPhSaya
import id.polri.poldabali.kinerja.data.KonteksAbsensi
import id.polri.poldabali.kinerja.data.Lampiran
import id.polri.poldabali.kinerja.data.Output
import id.polri.poldabali.kinerja.data.OutputRequest
import id.polri.poldabali.kinerja.data.Pengguna
import id.polri.poldabali.kinerja.data.PenilaianDetailResponse
import id.polri.poldabali.kinerja.data.Periode
import id.polri.poldabali.kinerja.data.Rencana
import id.polri.poldabali.kinerja.data.RencanaRequest
import id.polri.poldabali.kinerja.data.UbahHasilRequest
import id.polri.poldabali.kinerja.data.UbahOutputRequest
import id.polri.poldabali.kinerja.data.UbahProfilRequest
import id.polri.poldabali.kinerja.data.UbahRencanaRequest
import id.polri.poldabali.kinerja.data.VerifikasiHasilRequest
import id.polri.poldabali.kinerja.data.VerifikasiOutputRequest
import id.polri.poldabali.kinerja.sync.Pengingat
import id.polri.poldabali.kinerja.sync.Sinkronisasi
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter
import java.util.Locale
import java.util.UUID

class KinerjaViewModel(aplikasi: Application) : AndroidViewModel(aplikasi) {

    private val repo = Graf.repositori

    private var offsetJamServerMs = 0L
    private var epochServerTerakhir: Long? = null
    private var pekerjaanJamServer: Job? = null
    private val formatJamServer = DateTimeFormatter.ofPattern("HH:mm:ss")
    private val formatTanggalJamServer =
        DateTimeFormatter.ofPattern("d MMM yyyy", Locale.forLanguageTag("id-ID"))
    private val zonaWita: ZoneOffset = ZoneOffset.ofHours(8)

    var pengguna by mutableStateOf<Pengguna?>(null)
        private set

    var wizardIzinSelesai by mutableStateOf<Boolean?>(null)
        private set

    var memuatSesi by mutableStateOf(true)
        private set

    var sedangMemproses by mutableStateOf(false)
        private set

    var pesan by mutableStateOf<String?>(null)
        private set

    var galat by mutableStateOf<String?>(null)
        private set

    var riwayat by mutableStateOf<List<Absensi>>(emptyList())
        private set

    /** Rentang tanggal riwayat yang sedang ditampilkan (YYYY-MM-DD). */
    var riwayatDari by mutableStateOf("")
        private set

    var riwayatSampai by mutableStateOf("")
        private set

    /** Jadwal & absensi strong point khusus untuk layar riwayat. */
    var jadwalPhRiwayat by mutableStateOf<List<JadwalPhSaya>>(emptyList())
        private set

    var absensiPhRiwayat by mutableStateOf<List<AbsensiPh>>(emptyList())
        private set

    var absensiHariIni by mutableStateOf<AbsensiHariIni?>(null)
        private set

    var konteksAbsensi by mutableStateOf<KonteksAbsensi?>(null)
        private set

    var memuatKonteksAbsensi by mutableStateOf(false)
        private set

    var memuatAbsensiHariIni by mutableStateOf(false)
        private set

    /** True bila status absensi hari ini tidak dapat diambil dari server. */
    var absensiHariIniOffline by mutableStateOf(false)
        private set

    var antrean by mutableStateOf<List<AbsensiAntrean>>(emptyList())
        private set

    var antreanPh by mutableStateOf<List<AbsensiPhRequest>>(emptyList())
        private set

    var periode by mutableStateOf<Periode?>(null)
        private set

    var rencana by mutableStateOf<List<Rencana>>(emptyList())
        private set

    var persetujuan by mutableStateOf<List<Rencana>>(emptyList())
        private set

    /** Atasan langsung atau pimpinan yang ditunjuk sebagai atasan dapat memutuskan rencana. */
    val bisaMenyetujuiRencana: Boolean
        get() = pengguna?.roles?.any { it == "atasan" || it.startsWith("pimpinan") } == true

    var hasil by mutableStateOf<List<Hasil>>(emptyList())
        private set

    var hasilMenungguVerifikasi by mutableStateOf<List<Hasil>>(emptyList())
        private set

    /** Atasan langsung atau pimpinan yang ditunjuk sebagai atasan dapat memverifikasi hasil. */
    val bisaMemverifikasiHasil: Boolean
        get() = bisaMenyetujuiRencana

    var output by mutableStateOf<List<Output>>(emptyList())
        private set

    var outputMenungguVerifikasi by mutableStateOf<List<Output>>(emptyList())
        private set

    /** Foto profil terakhir (byte gambar); null berarti belum diunggah/gagal dimuat. */
    var fotoProfil by mutableStateOf<ByteArray?>(null)
        private set

    /** Atasan langsung atau pimpinan yang ditunjuk sebagai atasan dapat memverifikasi output. */
    val bisaMemverifikasiOutput: Boolean
        get() = bisaMenyetujuiRencana

    var nilai by mutableStateOf<PenilaianDetailResponse?>(null)
        private set

    var jadwalPh by mutableStateOf<List<JadwalPhSaya>>(emptyList())
        private set

    var absensiPh by mutableStateOf<List<AbsensiPh>>(emptyList())
        private set

    var titikTerdekat by mutableStateOf<id.polri.poldabali.kinerja.data.TitikTerdekatResponse?>(null)
        private set

    var alamatServer by mutableStateOf("")
        private set

    /** Jam server WITA yang berdetak lokal; disinkronkan ulang berkala. */
    var jamServerTeks by mutableStateOf("--:--:--")
        private set

    var jamServerTanggal by mutableStateOf("")
        private set

    var jamServerPalsu by mutableStateOf(false)
        private set

    init {
        mulaiJamServer()
        viewModelScope.launch {
            alamatServer = repo.alamatServer()
            pengguna = repo.penggunaTersimpan()
            wizardIzinSelesai = runCatching { repo.wizardIzinSelesai() }.getOrDefault(false)
            memuatSesi = false
            if (pengguna != null) muatSemua()
        }
    }

    /**
     * Sinkronkan offset sekali, lalu berdetak lokal tiap detik. Resync 5 menit
     * sekali (15 detik saat jam server palsu) agar beban server tetap ringan.
     */
    private fun mulaiJamServer() {
        if (pekerjaanJamServer?.isActive == true) return
        pekerjaanJamServer = viewModelScope.launch {
            perbaruiJamServer()
            var berikutnyaSinkron = 0L
            while (isActive) {
                val sekarang = System.currentTimeMillis()
                if (sekarang >= berikutnyaSinkron) {
                    runCatching { sinkronJamServer() }
                    berikutnyaSinkron =
                        sekarang + if (jamServerPalsu) 15_000L else 5 * 60_000L
                }
                perbaruiJamServer()
                delay(1_000L)
            }
        }
    }

    private suspend fun sinkronJamServer() {
        val mulai = System.currentTimeMillis()
        val waktu = repo.waktuServer()
        val selesai = System.currentTimeMillis()
        // Saat jam palsu tidak berubah, pertahankan offset agar jam tetap berdetak
        // alih-alih melompat kembali ke nilai fake yang dibekukan.
        val epochBerubah = epochServerTerakhir == null || waktu.epochMs != epochServerTerakhir
        if (!waktu.palsu || epochBerubah) {
            // Asumsi latensi simetris: waktu server berada di tengah rentang permintaan.
            offsetJamServerMs = waktu.epochMs - (mulai + selesai) / 2
            epochServerTerakhir = waktu.epochMs
        }
        jamServerPalsu = waktu.palsu
        perbaruiJamServer()
    }

    private fun perbaruiJamServer() {
        val wita = Instant.ofEpochMilli(System.currentTimeMillis() + offsetJamServerMs)
            .atOffset(zonaWita)
        jamServerTeks = wita.format(formatJamServer)
        jamServerTanggal = wita.format(formatTanggalJamServer)
    }

    // ------------------------------------------------------------------ util

    private fun jalankan(
        bersihkanPesan: Boolean = true,
        blok: suspend () -> Unit
    ) {
        viewModelScope.launch {
            sedangMemproses = true
            if (bersihkanPesan) {
                pesan = null
                galat = null
            }
            try {
                blok()
            } catch (error: ApiException) {
                if (error.status == 401 && pengguna != null) {
                    pengguna = null
                    galat = "Sesi berakhir, silakan login kembali"
                } else {
                    galat = error.message
                }
            } catch (error: Exception) {
                galat = error.message ?: "Terjadi kesalahan"
            } finally {
                sedangMemproses = false
            }
        }
    }

    fun bersihkanPesan() {
        pesan = null
        galat = null
    }

    fun pesanSukses(teks: String) {
        pesan = teks
        galat = null
    }

    fun pesanGalat(teks: String) {
        galat = teks
    }

    /** Memeriksa jarak ke titik dinas terdekat setelah lokasi didapat. */
    fun cekLokasi(latitude: Double, longitude: Double, akurasi: Double) {
        viewModelScope.launch {
            runCatching { repo.titikTerdekat(latitude, longitude, akurasi) }
                .onSuccess { titikTerdekat = it }
        }
    }

    // ---------------------------------------------------------- autentikasi

    fun login(nrp: String, password: String, selesai: () -> Unit) {
        jalankan {
            pengguna = repo.login(nrp, password)
            sinkronJamSekarang()
            muatSemuaLangsung()
            selesai()
        }
    }

    fun ubahAlamatServer(url: String) {
        jalankan {
            repo.ubahAlamatServer(url)
            alamatServer = repo.alamatServer()
            sinkronJamSekarang()
            pesan = "Alamat server disimpan"
        }
    }

    /** Sinkronisasi jam segera setelah login atau alamat server berubah. */
    private fun sinkronJamSekarang() {
        viewModelScope.launch { runCatching { sinkronJamServer() } }
    }

    fun keluar(selesai: () -> Unit) {
        viewModelScope.launch {
            repo.keluar()
            bersihkanDataSesi()
            selesai()
        }
    }

    private fun bersihkanDataSesi() {
        pengguna = null
        fotoProfil = null
        riwayat = emptyList()
        riwayatDari = ""
        riwayatSampai = ""
        jadwalPhRiwayat = emptyList()
        absensiPhRiwayat = emptyList()
        rencana = emptyList()
        persetujuan = emptyList()
        hasil = emptyList()
        hasilMenungguVerifikasi = emptyList()
        output = emptyList()
        outputMenungguVerifikasi = emptyList()
        nilai = null
        jadwalPh = emptyList()
        absensiPh = emptyList()
        antrean = emptyList()
        antreanPh = emptyList()
    }

    fun segarkanProfil() {
        jalankan {
            pengguna = repo.profil()
            muatFotoProfilLangsung()
        }
    }

    fun simpanProfil(request: UbahProfilRequest, selesai: () -> Unit) {
        jalankan {
            pengguna = repo.ubahProfil(request)
            pesan = "Profil diperbarui"
            selesai()
        }
    }

    fun unggahFotoProfil(nama: String, mime: String, byte: ByteArray) {
        jalankan {
            pengguna = repo.unggahFotoProfil(nama, mime, byte)
            fotoProfil = byte
            pesan = "Foto profil diperbarui"
        }
    }

    fun gantiPassword(lama: String, baru: String, selesai: () -> Unit) {
        jalankan {
            repo.ubahPassword(lama, baru)
            bersihkanDataSesi()
            pesan = "Password berhasil diubah, silakan login kembali"
            selesai()
        }
    }

    fun muatFotoProfil() {
        viewModelScope.launch { muatFotoProfilLangsung() }
    }

    private suspend fun muatFotoProfilLangsung() {
        val url = pengguna?.fotoUrl
        if (url == null) {
            fotoProfil = null
            return
        }
        runCatching { fotoProfil = repo.unduhFotoProfil(url) }
    }

    // --------------------------------------------------------------- absensi

    fun muatSemua() {
        jalankan { muatSemuaLangsung() }
    }

    fun tandaiWizardIzinSelesai() {
        viewModelScope.launch {
            repo.tandaiWizardIzinSelesai()
            wizardIzinSelesai = true
        }
    }

    /** Memuat status absensi hari ini dari server (tanggal server, bukan jam perangkat). */
    fun muatAbsensiHariIni() {
        viewModelScope.launch {
            memuatAbsensiHariIni = true
            try {
                absensiHariIni = repo.absensiHariIni()
                absensiHariIniOffline = false
            } catch (error: ApiException) {
                absensiHariIni = null
                if (error.status == 401) {
                    pengguna = null
                    galat = "Sesi berakhir, silakan login kembali"
                } else {
                    absensiHariIniOffline = true
                }
            } catch (_: Exception) {
                absensiHariIni = null
                absensiHariIniOffline = true
            } finally {
                memuatAbsensiHariIni = false
            }
        }
    }

    /** Memuat konteks check-in/check-out dari server. */
    fun muatKonteksAbsensi() {
        viewModelScope.launch {
            memuatKonteksAbsensi = true
            try {
                konteksAbsensi = repo.konteksAbsensi()
                absensiHariIniOffline = false
            } catch (error: ApiException) {
                konteksAbsensi = null
                if (error.status == 401) {
                    pengguna = null
                    galat = "Sesi berakhir, silakan login kembali"
                } else {
                    absensiHariIniOffline = true
                }
            } catch (_: Exception) {
                konteksAbsensi = null
                absensiHariIniOffline = true
            } finally {
                memuatKonteksAbsensi = false
            }
        }
    }

    private suspend fun muatSemuaLangsung() {
        periode = repo.periodeAktif()
        antrean = repo.daftarAntrean()
        antreanPh = repo.daftarAntreanPh()
        runCatching { absensiHariIni = repo.absensiHariIni() }
            .onSuccess { absensiHariIniOffline = false }
            .onFailure { absensiHariIniOffline = true }
        runCatching { konteksAbsensi = repo.konteksAbsensi() }
        muatRiwayatRentangLangsung()

        // Ringkasan di Beranda butuh jumlah dokumen & nilai terbaru.
        runCatching { rencana = daftarRencanaSaya() }
        if (bisaMenyetujuiRencana) {
            runCatching { persetujuan = repo.daftarPersetujuan() }
        }
        runCatching { hasil = repo.daftarHasil(pengguna?.id) }
        if (bisaMemverifikasiHasil) {
            runCatching { hasilMenungguVerifikasi = repo.daftarVerifikasiHasil() }
        }
        runCatching { output = repo.daftarOutput(pengguna?.id) }
        if (bisaMemverifikasiOutput) {
            runCatching { outputMenungguVerifikasi = repo.daftarVerifikasiOutput() }
        }
        runCatching { muatFotoProfilLangsung() }
        runCatching { muatPhLangsung() }

        val penggunaSaatIni = pengguna
        val periodeSaatIni = periode
        if (penggunaSaatIni != null && periodeSaatIni != null) {
            nilai = repo.nilaiSaya(periodeSaatIni.id, penggunaSaatIni.id)
        }
    }

    fun muatRiwayat(dari: String, sampai: String) {
        jalankan { muatRiwayatRentangLangsung(dari, sampai) }
    }

    fun muatRiwayatBulanIni() {
        jalankan {
            runCatching { konteksAbsensi = repo.konteksAbsensi() }
            val acuan = tanggalAcuan()
            muatRiwayatRentangLangsung(
                dari = acuan.withDayOfMonth(1).toString(),
                sampai = acuan.withDayOfMonth(acuan.lengthOfMonth()).toString()
            )
        }
    }

    /** Memuat absensi dan sesi strong point untuk rentang yang sedang aktif. */
    private suspend fun muatRiwayatRentangLangsung(dari: String? = null, sampai: String? = null) {
        val acuan = tanggalAcuan()
        val awal = dari ?: riwayatDari.ifBlank { acuan.withDayOfMonth(1).toString() }
        val akhir = sampai ?: riwayatSampai.ifBlank {
            acuan.withDayOfMonth(acuan.lengthOfMonth()).toString()
        }
        riwayatDari = awal
        riwayatSampai = akhir
        riwayat = repo.riwayatAbsensi(awal, akhir)
        jadwalPhRiwayat = repo.jadwalPhSaya(awal, akhir)
        absensiPhRiwayat = repo.absensiPhSaya(awal, akhir)
    }

    /** Tanggal server bila tersedia agar riwayat tetap benar saat jam server dipalsukan. */
    private fun tanggalAcuan(): LocalDate {
        val teks = konteksAbsensi?.tanggal ?: absensiHariIni?.tanggal
        return teks?.let { runCatching { LocalDate.parse(it) }.getOrNull() } ?: LocalDate.now()
    }

    fun kirimAbsensi(
        jenis: String,
        statusKehadiran: String,
        latitude: Double?,
        longitude: Double?,
        akurasi: Double?,
        mock: Boolean,
        fotoBase64: String?,
        offline: Boolean,
        selesai: (String) -> Unit
    ) {
        jalankan {
            val request = AbsensiRequest(
                clientUuid = UUID.randomUUID().toString(),
                jenis = jenis,
                statusKehadiran = statusKehadiran,
                capturedAt = Instant.now().toString(),
                latitude = latitude,
                longitude = longitude,
                akurasiM = akurasi,
                isMockSuspect = mock,
                isOffline = offline,
                deviceId = repo.perangkat,
                fotoBase64 = fotoBase64
            )

            if (offline) {
                repo.antreAbsensi(request)
                antrean = repo.daftarAntrean()
                Sinkronisasi.segera(getApplication())
                selesai("Absensi disimpan di perangkat dan akan dikirim otomatis")
            } else {
                val hasilKirim = repo.kirimAbsensi(request)
                muatRiwayatRentangLangsung()
                runCatching { absensiHariIni = repo.absensiHariIni() }
                    .onSuccess { absensiHariIniOffline = false }
                runCatching { konteksAbsensi = repo.konteksAbsensi() }
                val aksi = if (hasilKirim.aksi == "checkout") "Check-out" else "Check-in"
                val info = buildString {
                    append("$aksi tercatat pukul ${hasilKirim.data.jam}")
                    val jarak = hasilKirim.geofence.jarakM
                    if (jarak != null) append(String.format(java.util.Locale.US, " (%.0f m dari titik dinas)", jarak))
                }
                selesai(info)
            }
        }
    }

    fun sinkronkanSekarang() {
        jalankan {
            val hasilAbsensi = repo.sinkronkanAntrean()
            val hasilPh = repo.sinkronkanAntreanPh()
            antrean = repo.daftarAntrean()
            antreanPh = repo.daftarAntreanPh()
            muatRiwayatRentangLangsung()
            runCatching { muatPhLangsung() }
            runCatching { absensiHariIni = repo.absensiHariIni() }
                .onSuccess { absensiHariIniOffline = false }
            val total = hasilAbsensi.berhasil + hasilPh.berhasil
            val ditolak = hasilAbsensi.ditolak + hasilPh.ditolak
            pesan = when {
                ditolak > 0 && total > 0 ->
                    "$total data terkirim, $ditolak ditolak karena lokasi tiruan (mock GPS)"

                ditolak > 0 -> "$ditolak data ditolak karena lokasi tiruan (mock GPS)"
                total > 0 -> "$total data berhasil dikirim"
                else -> "Tidak ada antrean yang dapat dikirim"
            }
        }
    }

    // --------------------------------------------------------- rencana kerja

    fun muatRencana() {
        jalankan { rencana = daftarRencanaSaya() }
    }

    private suspend fun daftarRencanaSaya(): List<Rencana> = repo.daftarRencana(pengguna?.id)

    fun simpanRencana(
        request: RencanaRequest,
        lampiran: List<BerkasLampiran>,
        selesai: () -> Unit
    ) {
        jalankan {
            val dibuat = repo.buatRencana(request)

            var gagalUnggah = 0
            lampiran.forEach { berkas ->
                runCatching { repo.unggahLampiranRencana(dibuat.id, berkas) }
                    .onFailure { gagalUnggah++ }
            }

            rencana = daftarRencanaSaya()
            pesan = if (request.ajukan) {
                "Rencana kerja diajukan ke atasan"
            } else {
                "Rencana kerja disimpan sebagai draft"
            }
            if (gagalUnggah > 0) {
                galat = "$gagalUnggah dari ${lampiran.size} lampiran gagal diunggah"
            } else if (lampiran.isNotEmpty()) {
                pesan = "$pesan · ${lampiran.size} lampiran terunggah"
            }
            selesai()
        }
    }

    /** Memuat detail rencana (termasuk tahapan & lampiran) untuk mode ubah. */
    fun muatDetailRencana(id: String, selesai: (Rencana) -> Unit) {
        viewModelScope.launch {
            sedangMemproses = true
            galat = null
            try {
                selesai(repo.detailRencana(id))
            } catch (error: ApiException) {
                if (error.status == 401) {
                    pengguna = null
                    galat = "Sesi berakhir, silakan login kembali"
                } else {
                    galat = error.message
                }
            } catch (error: Exception) {
                galat = error.message ?: "Terjadi kesalahan"
            } finally {
                sedangMemproses = false
            }
        }
    }

    fun ubahRencana(
        id: String,
        request: UbahRencanaRequest,
        lampiranBaru: List<BerkasLampiran>,
        lampiranDihapus: List<String>,
        ajukan: Boolean,
        selesai: () -> Unit
    ) {
        jalankan {
            repo.ubahRencana(id, request)

            if (ajukan) repo.ajukanRencana(id)

            var gagalLampiran = 0
            lampiranDihapus.forEach { lampiranId ->
                runCatching { repo.hapusLampiran(lampiranId) }
                    .onFailure { gagalLampiran++ }
            }
            lampiranBaru.forEach { berkas ->
                runCatching { repo.unggahLampiranRencana(id, berkas) }
                    .onFailure { gagalLampiran++ }
            }

            rencana = daftarRencanaSaya()
            pesan = if (ajukan) {
                "Rencana kerja diajukan ulang ke atasan"
            } else {
                "Perubahan rencana kerja disimpan"
            }
            if (gagalLampiran > 0) {
                galat = "$gagalLampiran lampiran gagal diproses"
            }
            selesai()
        }
    }

    fun ajukanRencana(id: String) {
        jalankan {
            repo.ajukanRencana(id)
            rencana = daftarRencanaSaya()
            pesan = "Rencana kerja diajukan ke atasan"
        }
    }

    // ------------------------------------------------- persetujuan rencana

    fun muatPersetujuan() {
        if (!bisaMenyetujuiRencana) return
        jalankan { persetujuan = repo.daftarPersetujuan() }
    }

    fun putuskanRencana(
        id: String,
        status: String,
        catatan: String?,
        selesai: () -> Unit
    ) {
        jalankan {
            repo.putuskanRencana(id, status, catatan)
            persetujuan = repo.daftarPersetujuan()
            rencana = daftarRencanaSaya()
            pesan = when (status) {
                "disetujui" -> "Rencana kerja disetujui"
                "perlu_revisi" -> "Rencana dikembalikan untuk revisi"
                else -> "Rencana kerja ditolak"
            }
            selesai()
        }
    }

    fun unduhLampiran(
        lampiran: Lampiran,
        selesai: (ByteArray) -> Unit,
        gagal: (String) -> Unit
    ) {
        viewModelScope.launch {
            try {
                selesai(repo.unduhLampiran(lampiran.storageKey))
            } catch (error: ApiException) {
                if (error.status == 401) pengguna = null
                gagal(error.message ?: "Gagal mengunduh lampiran")
            } catch (error: Exception) {
                gagal(error.message ?: "Gagal mengunduh lampiran")
            }
        }
    }

    fun rencanaDisetujui(): List<Rencana> = rencana.filter { it.status == "disetujui" }

    // ------------------------------------------------------------ hasil kerja

    fun muatHasil() {
        jalankan { hasil = repo.daftarHasil(pengguna?.id) }
    }

    fun simpanHasil(
        request: HasilRequest,
        lampiran: List<BerkasLampiran>,
        selesai: () -> Unit
    ) {
        jalankan {
            val dibuat = repo.buatHasil(request)

            var gagalUnggah = 0
            lampiran.forEach { berkas ->
                runCatching { repo.unggahBuktiHasil(dibuat.id, berkas) }
                    .onFailure { gagalUnggah++ }
            }

            hasil = repo.daftarHasil(pengguna?.id)
            pesan = if (request.ajukan) {
                "Hasil kerja dikirim ke atasan"
            } else {
                "Hasil kerja disimpan sebagai draft"
            }
            if (gagalUnggah > 0) {
                galat = "$gagalUnggah dari ${lampiran.size} bukti gagal diunggah"
            } else if (lampiran.isNotEmpty()) {
                pesan = "$pesan · ${lampiran.size} bukti terunggah"
            }
            selesai()
        }
    }

    fun ubahHasil(
        id: String,
        request: UbahHasilRequest,
        lampiranBaru: List<BerkasLampiran>,
        lampiranDihapus: List<String>,
        ajukan: Boolean,
        selesai: () -> Unit
    ) {
        jalankan {
            repo.ubahHasil(id, request)
            if (ajukan) repo.ajukanHasil(id)

            var gagalLampiran = 0
            lampiranDihapus.forEach { lampiranId ->
                runCatching { repo.hapusLampiran(lampiranId) }
                    .onFailure { gagalLampiran++ }
            }
            lampiranBaru.forEach { berkas ->
                runCatching { repo.unggahBuktiHasil(id, berkas) }
                    .onFailure { gagalLampiran++ }
            }

            hasil = repo.daftarHasil(pengguna?.id)
            pesan = if (ajukan) {
                "Hasil kerja dikirim ulang ke atasan"
            } else {
                "Perubahan hasil kerja disimpan"
            }
            if (gagalLampiran > 0) {
                galat = "$gagalLampiran bukti gagal diproses"
            }
            selesai()
        }
    }

    fun ajukanHasil(id: String) {
        jalankan {
            repo.ajukanHasil(id)
            hasil = repo.daftarHasil(pengguna?.id)
            pesan = "Hasil kerja dikirim ke atasan"
        }
    }

    fun muatDetailHasil(id: String, selesai: (Hasil) -> Unit) {
        viewModelScope.launch {
            sedangMemproses = true
            galat = null
            try {
                selesai(repo.detailHasil(id))
            } catch (error: ApiException) {
                if (error.status == 401) {
                    pengguna = null
                    galat = "Sesi berakhir, silakan login kembali"
                } else {
                    galat = error.message
                }
            } catch (error: Exception) {
                galat = error.message ?: "Terjadi kesalahan"
            } finally {
                sedangMemproses = false
            }
        }
    }

    // ------------------------------------------------ verifikasi hasil kerja

    fun muatVerifikasiHasil() {
        if (!bisaMemverifikasiHasil) return
        jalankan { hasilMenungguVerifikasi = repo.daftarVerifikasiHasil() }
    }

    fun putuskanVerifikasiHasil(
        id: String,
        setuju: Boolean,
        skorKualitas: Double?,
        catatan: String?,
        selesai: () -> Unit
    ) {
        jalankan {
            repo.verifikasiHasil(
                id,
                VerifikasiHasilRequest(
                    setuju = setuju,
                    skorKualitas = if (setuju) skorKualitas else null,
                    catatan = catatan
                )
            )
            hasilMenungguVerifikasi = repo.daftarVerifikasiHasil()
            hasil = repo.daftarHasil(pengguna?.id)
            pesan = if (setuju) "Hasil kerja terverifikasi" else "Hasil kerja dikembalikan untuk revisi"
            selesai()
        }
    }

    // ----------------------------------------------------------------- output

    fun muatOutput() {
        jalankan { output = repo.daftarOutput(pengguna?.id) }
    }

    fun simpanOutput(
        request: OutputRequest,
        lampiran: List<BerkasLampiran>,
        selesai: () -> Unit
    ) {
        jalankan {
            val dibuat = repo.buatOutput(request.copy(ajukan = false))

            var gagalUnggah = 0
            lampiran.forEach { berkas ->
                runCatching { repo.unggahBuktiOutput(dibuat.id, berkas) }
                    .onFailure { gagalUnggah++ }
            }

            val dikirim = request.ajukan && gagalUnggah == 0
            if (dikirim) repo.ajukanOutput(dibuat.id)

            output = repo.daftarOutput(pengguna?.id)
            pesan = when {
                dikirim -> "Output dikirim ke atasan"
                request.ajukan -> "Output disimpan sebagai draf karena unggahan gagal"
                else -> "Output disimpan sebagai draf"
            }
            if (gagalUnggah > 0) {
                galat = "$gagalUnggah dari ${lampiran.size} berkas gagal diunggah"
            } else if (lampiran.isNotEmpty()) {
                pesan = "$pesan · ${lampiran.size} berkas terunggah"
            }
            selesai()
        }
    }

    fun ubahOutput(
        id: String,
        request: UbahOutputRequest,
        lampiranBaru: List<BerkasLampiran>,
        lampiranDihapus: List<String>,
        ajukan: Boolean,
        selesai: () -> Unit
    ) {
        jalankan {
            repo.ubahOutput(id, request)

            var gagalLampiran = 0
            lampiranDihapus.forEach { lampiranId ->
                runCatching { repo.hapusLampiran(lampiranId) }
                    .onFailure { gagalLampiran++ }
            }
            lampiranBaru.forEach { berkas ->
                runCatching { repo.unggahBuktiOutput(id, berkas) }
                    .onFailure { gagalLampiran++ }
            }

            val dikirim = ajukan && gagalLampiran == 0
            if (dikirim) repo.ajukanOutput(id)

            output = repo.daftarOutput(pengguna?.id)
            pesan = when {
                dikirim -> "Output dikirim ulang ke atasan"
                ajukan -> "Perubahan disimpan, pengiriman dibatalkan karena unggahan gagal"
                else -> "Perubahan output disimpan"
            }
            if (gagalLampiran > 0) {
                galat = "$gagalLampiran berkas gagal diproses"
            }
            selesai()
        }
    }

    fun ajukanOutput(id: String) {
        jalankan {
            repo.ajukanOutput(id)
            output = repo.daftarOutput(pengguna?.id)
            pesan = "Output dikirim ke atasan"
        }
    }

    fun muatDetailOutput(id: String, selesai: (Output) -> Unit) {
        viewModelScope.launch {
            sedangMemproses = true
            galat = null
            try {
                selesai(repo.detailOutput(id))
            } catch (error: ApiException) {
                if (error.status == 401) {
                    pengguna = null
                    galat = "Sesi berakhir, silakan login kembali"
                } else {
                    galat = error.message
                }
            } catch (error: Exception) {
                galat = error.message ?: "Terjadi kesalahan"
            } finally {
                sedangMemproses = false
            }
        }
    }

    // ------------------------------------------------- verifikasi output

    fun muatVerifikasiOutput() {
        if (!bisaMemverifikasiOutput) return
        jalankan { outputMenungguVerifikasi = repo.daftarVerifikasiOutput() }
    }

    fun putuskanVerifikasiOutput(
        id: String,
        setuju: Boolean,
        skorKesesuaian: Double?,
        skorKeabsahan: Double?,
        catatan: String?,
        selesai: () -> Unit
    ) {
        jalankan {
            repo.verifikasiOutput(
                id,
                VerifikasiOutputRequest(
                    setuju = setuju,
                    skorKesesuaian = if (setuju) skorKesesuaian else null,
                    skorKeabsahan = if (setuju) skorKeabsahan else null,
                    catatan = catatan
                )
            )
            outputMenungguVerifikasi = repo.daftarVerifikasiOutput()
            output = repo.daftarOutput(pengguna?.id)
            pesan = if (setuju) "Output disetujui" else "Output dikembalikan untuk revisi"
            selesai()
        }
    }

    // -------------------------------------------------------------- penilaian

    fun muatNilai() {
        val penggunaSaatIni = pengguna ?: return
        val periodeSaatIni = periode ?: return
        jalankan {
            nilai = repo.nilaiSaya(periodeSaatIni.id, penggunaSaatIni.id)
        }
    }

    // ---------------------------------------------------------------------- PH

    private suspend fun muatPhLangsung() {
        val acuan = tanggalAcuan()
        val dari = acuan.minusDays(7).toString()
        val sampai = acuan.plusDays(30).toString()
        jadwalPh = repo.jadwalPhSaya(dari, sampai)
        absensiPh = repo.absensiPhSaya(dari, sampai)
        antreanPh = repo.daftarAntreanPh()
        Pengingat.jadwalkan(getApplication(), jadwalPh)
    }

    fun muatPh() {
        jalankan { muatPhLangsung() }
    }

    fun kirimAbsensiPh(
        clientUuid: String,
        jadwalId: String,
        jenis: String,
        capturedAt: String,
        latitude: Double?,
        longitude: Double?,
        akurasi: Double?,
        mock: Boolean,
        fotoBase64: String?,
        offline: Boolean,
        selesai: (Boolean, String) -> Unit
    ) {
        viewModelScope.launch {
            sedangMemproses = true
            pesan = null
            galat = null
            val request = AbsensiPhRequest(
                clientUuid = clientUuid,
                jadwalPhId = jadwalId,
                jenis = jenis,
                capturedAt = capturedAt,
                latitude = latitude,
                longitude = longitude,
                akurasiM = akurasi,
                isMockSuspect = mock,
                isOffline = offline,
                fotoBase64 = fotoBase64
            )
            try {
                if (offline) {
                    repo.antreAbsensiPh(request)
                    antreanPh = repo.daftarAntreanPh()
                    Sinkronisasi.segera(getApplication())
                    selesai(true, "Strong point disimpan di perangkat dan akan dikirim otomatis")
                } else {
                    val hasil = repo.kirimAbsensiPh(request)
                    runCatching { muatPhLangsung() }
                    val jarak = hasil.jarakM
                    selesai(
                        true,
                        buildString {
                            append("Strong point ${hasil.jenis} tercatat pukul ${hasil.jam}")
                            if (jarak != null) {
                                append(String.format(java.util.Locale.US, " (%.0f m dari strong point)", jarak))
                            }
                        }
                    )
                }
            } catch (error: ApiException) {
                if (error.status == 401 && pengguna != null) {
                    pengguna = null
                    galat = "Sesi berakhir, silakan login kembali"
                } else {
                    galat = error.message
                }
                selesai(false, error.message ?: "Gagal mengirim strong point")
            } catch (error: Exception) {
                val teks = error.message ?: "Gagal mengirim strong point"
                galat = teks
                selesai(false, teks)
            } finally {
                sedangMemproses = false
            }
        }
    }
}
