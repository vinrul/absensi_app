package id.polri.poldabali.kinerja.data

import android.os.Build
import kotlinx.serialization.builtins.ListSerializer

/**
 * Titik akses tunggal bagi lapisan UI ke server.
 *
 * Semua pemanggilan HTTP dan pengelolaan sesi/antrean offline ada di sini agar
 * layar Compose tetap ringan.
 */
class Repositori(
    private val sesi: PenyimpananSesi,
    private val antrean: AntreanAbsensi,
    private val antreanPh: AntreanPh,
    private val api: Api
) {
    data class HasilSinkron(val berhasil: Int, val ditolak: Int)

    private fun penolakanMock(error: Throwable): Boolean {
        if (error !is ApiException || error.status != 422) return false
        val pesan = error.message?.lowercase().orEmpty()
        return "tiruan" in pesan || "mock" in pesan
    }

    private val json = api.json

    val perangkat: String = "${Build.MANUFACTURER} ${Build.MODEL}".trim()

    // ------------------------------------------------------------- autentikasi

    suspend fun login(nrp: String, password: String): Pengguna {
        val body = json.encodeToString(
            LoginRequest.serializer(),
            LoginRequest(nrp = nrp.trim(), password = password, deviceId = perangkat)
        )
        val teks = api.panggil("POST", "auth/login", body, butuhToken = false)
        val hasil = json.decodeFromString(LoginResponse.serializer(), teks)
        sesi.simpanLogin(hasil.accessToken, hasil.refreshToken, hasil.user)
        return hasil.user
    }

    suspend fun penggunaTersimpan(): Pengguna? = sesi.pengguna()

    suspend fun profil(): Pengguna {
        val teks = api.panggil("GET", "auth/me")
        val pengguna = json.decodeFromString(Pengguna.serializer(), teks)
        sesi.simpanPengguna(pengguna)
        return pengguna
    }

    suspend fun ubahProfil(request: UbahProfilRequest): Pengguna {
        val body = json.encodeToString(UbahProfilRequest.serializer(), request)
        val teks = api.panggil("PATCH", "auth/profile", body)
        val pengguna = json.decodeFromString(Pengguna.serializer(), teks)
        sesi.simpanPengguna(pengguna)
        return pengguna
    }

    suspend fun unggahFotoProfil(nama: String, mime: String, byte: ByteArray): Pengguna {
        val teks = api.unggahFoto(nama, mime, byte)
        val pengguna = json.decodeFromString(Pengguna.serializer(), teks)
        sesi.simpanPengguna(pengguna)
        return pengguna
    }

    suspend fun unduhFotoProfil(fotoUrl: String): ByteArray = api.unduh(fotoUrl)

    /** Ganti password lalu hapus sesi lokal karena server mencabut seluruh refresh token. */
    suspend fun ubahPassword(oldPassword: String, newPassword: String) {
        val body = json.encodeToString(
            UbahPasswordRequest.serializer(),
            UbahPasswordRequest(oldPassword = oldPassword, newPassword = newPassword)
        )
        api.panggil("POST", "auth/change-password", body)
        sesi.hapusSesi()
        antrean.kosongkan()
    }

    suspend fun keluar() {
        runCatching { api.panggil("POST", "auth/logout") }
        sesi.hapusSesi()
        antrean.kosongkan()
    }

    suspend fun alamatServer(): String = sesi.alamatServer() ?: Api.ALAMAT_BAWAAN

    suspend fun ubahAlamatServer(url: String) = sesi.setAlamatServer(url.trim())

    suspend fun wizardIzinSelesai(): Boolean = sesi.wizardIzinSelesai()

    suspend fun tandaiWizardIzinSelesai() = sesi.setWizardIzinSelesai()

    // ---------------------------------------------------------------- absensi

    suspend fun konteksAbsensi(): KonteksAbsensi {
        val teks = api.panggil("GET", "absensi/konteks")
        return json.decodeFromString(KonteksAbsensi.serializer(), teks)
    }

    /** Jam server WITA untuk sinkronisasi tampilan jam di perangkat. */
    suspend fun waktuServer(): WaktuServer {
        val teks = api.panggil("GET", "waktu", butuhToken = false)
        return json.decodeFromString(WaktuServer.serializer(), teks)
    }

    suspend fun riwayatAbsensi(dari: String, sampai: String): List<Absensi> {
        val teks = api.panggil("GET", "absensi/saya?dari=$dari&sampai=$sampai")
        return json.decodeFromString(ListSerializer(Absensi.serializer()), teks)
    }

    /** Riwayat bulan berjalan menurut tanggal server (mendukung jam palsu development). */
    suspend fun riwayatAbsensiBulanIni(): List<Absensi> {
        val teks = api.panggil("GET", "absensi/saya")
        return json.decodeFromString(ListSerializer(Absensi.serializer()), teks)
    }

    suspend fun kirimAbsensi(request: AbsensiRequest): AbsensiResponse {
        val body = json.encodeToString(AbsensiRequest.serializer(), request)
        val teks = api.panggil("POST", "absensi", body)
        return json.decodeFromString(AbsensiResponse.serializer(), teks)
    }

    suspend fun absensiHariIni(): AbsensiHariIni {
        val teks = api.panggil("GET", "absensi/hari-ini")
        return json.decodeFromString(AbsensiHariIni.serializer(), teks)
    }

    suspend fun titikTerdekat(latitude: Double, longitude: Double, akurasi: Double): TitikTerdekatResponse {
        val teks = api.panggil(
            "GET",
            "absensi/titik-terdekat?latitude=$latitude&longitude=$longitude&akurasi_m=$akurasi"
        )
        return json.decodeFromString(TitikTerdekatResponse.serializer(), teks)
    }

    suspend fun antreAbsensi(request: AbsensiRequest) {
        antrean.tambah(
            AbsensiAntrean(
                clientUuid = request.clientUuid,
                jenis = request.jenis,
                statusKehadiran = request.statusKehadiran,
                capturedAt = request.capturedAt,
                latitude = request.latitude,
                longitude = request.longitude,
                akurasiM = request.akurasiM,
                isMockSuspect = request.isMockSuspect,
                fotoBase64 = request.fotoBase64
            )
        )
    }

    suspend fun daftarAntrean(): List<AbsensiAntrean> = antrean.daftar()

    suspend fun jumlahAntrean(): Int = antrean.jumlah()

    /** Mengirim seluruh antrean; mengembalikan jumlah berhasil dan ditolak karena lokasi tiruan. */
    suspend fun sinkronkanAntrean(): HasilSinkron {
        var berhasil = 0
        var ditolak = 0
        for (item in antrean.daftar()) {
            val request = AbsensiRequest(
                clientUuid = item.clientUuid,
                jenis = item.jenis,
                statusKehadiran = item.statusKehadiran,
                capturedAt = item.capturedAt,
                latitude = item.latitude,
                longitude = item.longitude,
                akurasiM = item.akurasiM,
                isMockSuspect = item.isMockSuspect,
                isOffline = true,
                deviceId = perangkat,
                fotoBase64 = item.fotoBase64
            )

            try {
                kirimAbsensi(request)
                antrean.hapus(item.clientUuid)
                berhasil++
            } catch (error: ApiException) {
                if (penolakanMock(error)) {
                    antrean.hapus(item.clientUuid)
                    ditolak++
                }
                // Kegagalan lain dipertahankan agar dapat diperiksa ulang.
            } catch (_: Exception) {
                // Gangguan jaringan/IO: antrean tetap disimpan.
            }
        }
        return HasilSinkron(berhasil, ditolak)
    }

    // ---------------------------------------------------------- rencana kerja

    suspend fun periodeAktif(): Periode? {
        val teks = api.panggil("GET", "periode")
        return json.decodeFromString(ListSerializer(Periode.serializer()), teks).firstOrNull()
    }

    suspend fun daftarRencana(personelId: String? = null): List<Rencana> {
        val filter = if (personelId != null) "&personel_id=$personelId" else ""
        val teks = api.panggil("GET", "rencana-kerja?per_page=50$filter")
        return json.decodeFromString(ListResponse.serializer(Rencana.serializer()), teks).data
    }

    suspend fun daftarRencanaDisetujui(): List<Rencana> =
        daftarRencana().filter { it.status == "disetujui" }

    suspend fun detailRencana(id: String): Rencana {
        val teks = api.panggil("GET", "rencana-kerja/$id")
        return json.decodeFromString(Rencana.serializer(), teks)
    }

    suspend fun buatRencana(request: RencanaRequest): Rencana {
        val body = json.encodeToString(RencanaRequest.serializer(), request)
        val teks = api.panggil("POST", "rencana-kerja", body)
        return json.decodeFromString(Rencana.serializer(), teks)
    }

    suspend fun ubahRencana(id: String, request: UbahRencanaRequest): Rencana {
        val body = json.encodeToString(UbahRencanaRequest.serializer(), request)
        val teks = api.panggil("PATCH", "rencana-kerja/$id", body)
        return json.decodeFromString(Rencana.serializer(), teks)
    }

    suspend fun ajukanRencana(id: String) {
        api.panggil("POST", "rencana-kerja/$id/ajukan")
    }

    suspend fun daftarPersetujuan(): List<Rencana> {
        val teks = api.panggil("GET", "rencana-kerja/persetujuan?per_page=50")
        return json.decodeFromString(ListResponse.serializer(Rencana.serializer()), teks).data
    }

    suspend fun putuskanRencana(id: String, status: String, catatan: String?) {
        val body = json.encodeToString(
            KeputusanRequest.serializer(),
            KeputusanRequest(status = status, catatan = catatan)
        )
        api.panggil("POST", "rencana-kerja/$id/keputusan", body)
    }

    suspend fun unggahLampiranRencana(rencanaId: String, berkas: BerkasLampiran) {
        api.unggah("rencana", rencanaId, "lampiran", berkas.nama, berkas.mime, berkas.isi)
    }

    suspend fun unduhLampiran(storageKey: String): ByteArray =
        api.unduh("lampiran/raw/$storageKey")

    suspend fun hapusLampiran(lampiranId: String) {
        api.panggil("DELETE", "lampiran/$lampiranId")
    }

    // ------------------------------------------------------------ hasil kerja

    suspend fun daftarHasil(personelId: String? = null): List<Hasil> {
        val filter = if (personelId != null) "&personel_id=$personelId" else ""
        val teks = api.panggil("GET", "hasil-kerja?per_page=50$filter")
        return json.decodeFromString(ListResponse.serializer(Hasil.serializer()), teks).data
    }

    suspend fun detailHasil(id: String): Hasil {
        val teks = api.panggil("GET", "hasil-kerja/$id")
        return json.decodeFromString(Hasil.serializer(), teks)
    }

    suspend fun buatHasil(request: HasilRequest): Hasil {
        val body = json.encodeToString(HasilRequest.serializer(), request)
        val teks = api.panggil("POST", "hasil-kerja", body)
        return json.decodeFromString(Hasil.serializer(), teks)
    }

    suspend fun ubahHasil(id: String, request: UbahHasilRequest): Hasil {
        val body = json.encodeToString(UbahHasilRequest.serializer(), request)
        val teks = api.panggil("PATCH", "hasil-kerja/$id", body)
        return json.decodeFromString(Hasil.serializer(), teks)
    }

    suspend fun ajukanHasil(id: String) {
        api.panggil("POST", "hasil-kerja/$id/ajukan")
    }

    suspend fun unggahBuktiHasil(hasilId: String, berkas: BerkasLampiran) {
        api.unggah("hasil", hasilId, "bukti", berkas.nama, berkas.mime, berkas.isi)
    }

    suspend fun daftarVerifikasiHasil(): List<Hasil> {
        val teks = api.panggil("GET", "hasil-kerja/verifikasi?per_page=50")
        return json.decodeFromString(ListResponse.serializer(Hasil.serializer()), teks).data
    }

    suspend fun verifikasiHasil(id: String, request: VerifikasiHasilRequest): Hasil {
        val body = json.encodeToString(VerifikasiHasilRequest.serializer(), request)
        val teks = api.panggil("POST", "hasil-kerja/$id/verifikasi", body)
        return json.decodeFromString(Hasil.serializer(), teks)
    }

    suspend fun daftarHasilTerverifikasi(): List<Hasil> =
        daftarHasil().filter { it.status == "terverifikasi" }

    // ------------------------------------------------------------------ output

    suspend fun daftarOutput(personelId: String? = null): List<Output> {
        val filter = if (personelId != null) "&personel_id=$personelId" else ""
        val teks = api.panggil("GET", "output?per_page=50$filter")
        return json.decodeFromString(ListResponse.serializer(Output.serializer()), teks).data
    }

    suspend fun detailOutput(id: String): Output {
        val teks = api.panggil("GET", "output/$id")
        return json.decodeFromString(Output.serializer(), teks)
    }

    suspend fun buatOutput(request: OutputRequest): Output {
        val body = json.encodeToString(OutputRequest.serializer(), request)
        val teks = api.panggil("POST", "output", body)
        return json.decodeFromString(Output.serializer(), teks)
    }

    suspend fun ubahOutput(id: String, request: UbahOutputRequest): Output {
        val body = json.encodeToString(UbahOutputRequest.serializer(), request)
        val teks = api.panggil("PATCH", "output/$id", body)
        return json.decodeFromString(Output.serializer(), teks)
    }

    suspend fun ajukanOutput(id: String) {
        api.panggil("POST", "output/$id/ajukan")
    }

    suspend fun unggahBuktiOutput(outputId: String, berkas: BerkasLampiran) {
        api.unggah("output", outputId, "bukti", berkas.nama, berkas.mime, berkas.isi)
    }

    suspend fun daftarVerifikasiOutput(): List<Output> {
        val teks = api.panggil("GET", "output/verifikasi?per_page=50")
        return json.decodeFromString(ListResponse.serializer(Output.serializer()), teks).data
    }

    suspend fun verifikasiOutput(id: String, request: VerifikasiOutputRequest): Output {
        val body = json.encodeToString(VerifikasiOutputRequest.serializer(), request)
        val teks = api.panggil("POST", "output/$id/verifikasi", body)
        return json.decodeFromString(Output.serializer(), teks)
    }

    // --------------------------------------------------------------- penilaian

    suspend fun nilaiSaya(periodeId: String, personelId: String): PenilaianDetailResponse? {
        val teks = runCatching {
            api.panggil("GET", "penilaian/personel/$personelId?periode_id=$periodeId")
        }.getOrNull() ?: return null

        return runCatching {
            json.decodeFromString(PenilaianDetailResponse.serializer(), teks)
        }.getOrNull()
    }

    // ---------------------------------------------------------------------- PH

    suspend fun jadwalPhSaya(dari: String, sampai: String): List<JadwalPhSaya> {
        val teks = api.panggil("GET", "ph/jadwal/saya?dari=$dari&sampai=$sampai")
        return json.decodeFromString(ListSerializer(JadwalPhSaya.serializer()), teks)
    }

    suspend fun absensiPhSaya(dari: String, sampai: String): List<AbsensiPh> {
        val teks = api.panggil("GET", "ph/absensi/saya?dari=$dari&sampai=$sampai")
        return json.decodeFromString(ListSerializer(AbsensiPh.serializer()), teks)
    }

    suspend fun kirimAbsensiPh(request: AbsensiPhRequest): AbsensiPh {
        val body = json.encodeToString(AbsensiPhRequest.serializer(), request)
        val teks = api.panggil("POST", "ph/absensi", body)
        return json.decodeFromString(AbsensiPh.serializer(), teks)
    }

    suspend fun antreAbsensiPh(request: AbsensiPhRequest) {
        antreanPh.tambah(request)
    }

    suspend fun daftarAntreanPh(): List<AbsensiPhRequest> = antreanPh.daftar()

    /** Mengirim antrean strong point berurutan; mock GPS dicatat sebagai ditolak. */
    suspend fun sinkronkanAntreanPh(): HasilSinkron {
        var berhasil = 0
        var ditolak = 0
        for (item in antreanPh.daftar()) {
            val hasil = runCatching { kirimAbsensiPh(item.copy(isOffline = true)) }
            if (hasil.isSuccess) {
                antreanPh.hapus(item.clientUuid)
                berhasil++
            } else {
                val error = hasil.exceptionOrNull()
                if (error != null && penolakanMock(error)) {
                    antreanPh.hapus(item.clientUuid)
                    ditolak++
                } else if (error is ApiException && error.status in 400..499 && error.status != 401) {
                    antreanPh.hapus(item.clientUuid)
                }
            }
        }
        return HasilSinkron(berhasil, ditolak)
    }
}
