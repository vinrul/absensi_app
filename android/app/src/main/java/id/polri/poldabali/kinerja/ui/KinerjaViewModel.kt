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
import id.polri.poldabali.kinerja.data.Lampiran
import id.polri.poldabali.kinerja.data.Output
import id.polri.poldabali.kinerja.data.OutputRequest
import id.polri.poldabali.kinerja.data.Pengguna
import id.polri.poldabali.kinerja.data.PenilaianDetailResponse
import id.polri.poldabali.kinerja.data.Periode
import id.polri.poldabali.kinerja.data.Rencana
import id.polri.poldabali.kinerja.data.RencanaRequest
import id.polri.poldabali.kinerja.data.UbahRencanaRequest
import id.polri.poldabali.kinerja.sync.Pengingat
import id.polri.poldabali.kinerja.sync.Sinkronisasi
import kotlinx.coroutines.launch
import java.time.Instant
import java.util.UUID

class KinerjaViewModel(aplikasi: Application) : AndroidViewModel(aplikasi) {

    private val repo = Graf.repositori

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

        	var absensiHariIni by mutableStateOf<AbsensiHariIni?>(null)
        		private set

        	var memuatAbsensiHariIni by mutableStateOf(false)
        		private set

        	/** True bila status absensi hari ini tidak dapat diambil dari server. */
        	var absensiHariIniOffline by mutableStateOf(false)
        		private set

    var antrean by mutableStateOf<List<AbsensiAntrean>>(emptyList())
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

    var output by mutableStateOf<List<Output>>(emptyList())
        private set

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

    init {
        		viewModelScope.launch {
        			alamatServer = repo.alamatServer()
        			pengguna = repo.penggunaTersimpan()
        			wizardIzinSelesai = runCatching { repo.wizardIzinSelesai() }.getOrDefault(false)
        			memuatSesi = false
        			if (pengguna != null) muatSemua()
        		}
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
            muatSemuaLangsung()
            selesai()
        }
    }

    fun ubahAlamatServer(url: String) {
        jalankan {
            repo.ubahAlamatServer(url)
            alamatServer = repo.alamatServer()
            pesan = "Alamat server disimpan"
        }
    }

    fun keluar(selesai: () -> Unit) {
        viewModelScope.launch {
            repo.keluar()
            pengguna = null
            riwayat = emptyList()
            rencana = emptyList()
            persetujuan = emptyList()
            hasil = emptyList()
            output = emptyList()
            nilai = null
            jadwalPh = emptyList()
            absensiPh = emptyList()
            antrean = emptyList()
            selesai()
        }
    }

    fun segarkanProfil() {
        jalankan {
            pengguna = repo.profil()
        }
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

    private suspend fun muatSemuaLangsung() {
        periode = repo.periodeAktif()
        antrean = repo.daftarAntrean()
        riwayat = repo.riwayatAbsensiBulanIni()
        runCatching { absensiHariIni = repo.absensiHariIni() }
            .onSuccess { absensiHariIniOffline = false }
            .onFailure { absensiHariIniOffline = true }

        // Ringkasan di Beranda butuh jumlah dokumen & nilai terbaru.
        runCatching { rencana = daftarRencanaSaya() }
        if (bisaMenyetujuiRencana) {
            runCatching { persetujuan = repo.daftarPersetujuan() }
        }
        runCatching { hasil = repo.daftarHasil() }
        runCatching { output = repo.daftarOutput() }
        runCatching { muatPhLangsung() }

        val penggunaSaatIni = pengguna
        val periodeSaatIni = periode
        if (penggunaSaatIni != null && periodeSaatIni != null) {
            nilai = repo.nilaiSaya(periodeSaatIni.id, penggunaSaatIni.id)
        }
    }

    fun muatRiwayat(dari: String, sampai: String) {
        jalankan { riwayat = repo.riwayatAbsensi(dari, sampai) }
    }

    fun muatRiwayatBulanIni() {
        jalankan { riwayat = repo.riwayatAbsensiBulanIni() }
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
                riwayat = repo.riwayatAbsensiBulanIni()
                runCatching { absensiHariIni = repo.absensiHariIni() }
                    .onSuccess { absensiHariIniOffline = false }
                val info = buildString {
                    append("Absen ${request.jenis} tercatat pukul ${hasilKirim.data.jam}")
                    val jarak = hasilKirim.geofence.jarakM
                    if (jarak != null) append(String.format(java.util.Locale.US, " (%.0f m dari titik dinas)", jarak))
                }
                selesai(info)
            }
        }
    }

    fun sinkronkanSekarang() {
        jalankan {
            val jumlah = repo.sinkronkanAntrean()
            antrean = repo.daftarAntrean()
            riwayat = repo.riwayatAbsensiBulanIni()
            runCatching { absensiHariIni = repo.absensiHariIni() }
                .onSuccess { absensiHariIniOffline = false }
            pesan = if (jumlah > 0) "$jumlah absensi berhasil dikirim" else "Tidak ada antrean yang dapat dikirim"
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
        jalankan { hasil = repo.daftarHasil() }
    }

    fun buatHasil(request: HasilRequest, selesai: () -> Unit) {
        jalankan {
            repo.buatHasil(request)
            hasil = repo.daftarHasil()
            pesan = "Hasil kerja terkirim untuk verifikasi"
            selesai()
        }
    }

    // ----------------------------------------------------------------- output

    fun muatOutput() {
        jalankan { output = repo.daftarOutput() }
    }

    fun buatOutput(
        request: OutputRequest,
        namaBerkas: String?,
        mime: String?,
        byte: ByteArray?,
        selesai: () -> Unit
    ) {
        jalankan {
            val dibuat = repo.buatOutput(request)
            if (namaBerkas != null && mime != null && byte != null) {
                repo.unggahBukti(dibuat.id, namaBerkas, mime, byte)
            }
            runCatching { repo.ajukanOutput(dibuat.id) }
            output = repo.daftarOutput()
            pesan = "Output tersimpan dan diajukan verifikasi"
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
        val bulan = awalBulan().take(7)
        jadwalPh = repo.jadwalPhSaya(bulan)
        absensiPh = repo.absensiPhSaya(bulan)
        Pengingat.jadwalkan(getApplication(), jadwalPh)
    }

    fun muatPh() {
        jalankan { muatPhLangsung() }
    }

    fun kirimAbsensiPh(
        jadwalId: String,
        jenis: String,
        latitude: Double?,
        longitude: Double?,
        akurasi: Double?,
        mock: Boolean,
        fotoBase64: String?,
        selesai: (String) -> Unit
    ) {
        jalankan {
            val hasil = repo.kirimAbsensiPh(
                AbsensiPhRequest(
                    clientUuid = UUID.randomUUID().toString(),
                    jadwalPhId = jadwalId,
                    jenis = jenis,
                    capturedAt = Instant.now().toString(),
                    latitude = latitude,
                    longitude = longitude,
                    akurasiM = akurasi,
                    isMockSuspect = mock,
                    isOffline = false,
                    fotoBase64 = fotoBase64
                )
            )
            muatPhLangsung()
            val jarak = hasil.jarakM
            selesai(
                buildString {
                    append("Absensi PH ${hasil.jenis} tercatat pukul ${hasil.jam}")
                    if (jarak != null) {
                        append(String.format(java.util.Locale.US, " (%.0f m dari strong point)", jarak))
                    }
                }
            )
        }
    }
}
