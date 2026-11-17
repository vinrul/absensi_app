package id.polri.poldabali.kinerja.data

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonNamingStrategy
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.MultipartBody
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.io.IOException
import java.util.concurrent.TimeUnit

class ApiException(val status: Int, message: String) : Exception(message)

/**
 * Klien HTTP di atas OkHttp.
 *
 * Menangani bearer token, percobaan ulang setelah refresh, unggahan multipart,
 * dan pemetaan pesan kesalahan dari server.
 */
@OptIn(ExperimentalSerializationApi::class)
class Api(private val sesi: PenyimpananSesi) {

    val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
        explicitNulls = false
        namingStrategy = JsonNamingStrategy.SnakeCase
    }

    private val klien = OkHttpClient.Builder()
        .connectTimeout(20, TimeUnit.SECONDS)
        .readTimeout(60, TimeUnit.SECONDS)
        .writeTimeout(60, TimeUnit.SECONDS)
        .build()

    private val mediaJson = "application/json; charset=utf-8".toMediaType()

    private suspend fun alamat(): String {
        val dasar = sesi.alamatServer()?.trim().orEmpty().ifEmpty { ALAMAT_BAWAAN }
        return if (dasar.endsWith("/")) dasar else "$dasar/"
    }

    suspend fun panggil(
        metode: String,
        path: String,
        bodyJson: String? = null,
        butuhToken: Boolean = true
    ): String {
        val url = alamat() + path.removePrefix("/")
        var hasil = kirim(metode, url, bodyJson, butuhToken)

        if (hasil.first == 401 && butuhToken && segarkan()) {
            hasil = kirim(metode, url, bodyJson, butuhToken)
        }

        val (status, teks) = hasil
        if (status !in 200..299) {
            throw ApiException(status, pesanError(teks, status))
        }
        return teks
    }

    private fun pesanError(teks: String, status: Int): String {
        val dariServer = runCatching {
            json.decodeFromString(ApiErrorBody.serializer(), teks).error.message
        }.getOrNull()

        return dariServer?.takeIf { it.isNotBlank() } ?: when (status) {
            401 -> "Sesi berakhir, silakan login kembali"
            403 -> "Anda tidak memiliki akses"
            404 -> "Data tidak ditemukan"
            else -> "Permintaan gagal ($status)"
        }
    }

    private suspend fun kirim(
        metode: String,
        url: String,
        bodyJson: String?,
        butuhToken: Boolean
    ): Pair<Int, String> = withContext(Dispatchers.IO) {
        val pembangun = Request.Builder().url(url)

        when (metode) {
            "GET" -> pembangun.get()
            "POST" -> pembangun.post((bodyJson ?: "").toRequestBody(mediaJson))
            "PUT" -> pembangun.put((bodyJson ?: "").toRequestBody(mediaJson))
            "PATCH" -> pembangun.patch((bodyJson ?: "").toRequestBody(mediaJson))
            "DELETE" -> pembangun.delete()
            else -> pembangun.method(metode, bodyJson?.toRequestBody(mediaJson))
        }

        if (butuhToken) {
            sesi.aksesToken()?.let { pembangun.header("Authorization", "Bearer $it") }
        }

        try {
            klien.newCall(pembangun.build()).execute().use { respons ->
                respons.code to (respons.body?.string().orEmpty())
            }
        } catch (galat: IOException) {
            throw ApiException(0, "Tidak dapat terhubung ke server. Periksa jaringan atau alamat server.")
        }
    }

    private suspend fun segarkan(): Boolean {
        val segar = sesi.segarToken() ?: return false

        return runCatching {
            val tokenJson = json.encodeToString(String.serializer(), segar)
            val hasil = kirim(
                "POST",
                alamat() + "auth/refresh",
                """{"refresh_token":$tokenJson}""",
                butuhToken = false
            )
            if (hasil.first !in 200..299) return@runCatching false

            val token = json.decodeFromString(TokenResponse.serializer(), hasil.second)
            sesi.perbaruiToken(token.accessToken, token.refreshToken)
            true
        }.getOrDefault(false)
    }

    /** Unggah lampiran/bukti (multipart) sesuai kontrak `POST /lampiran`. */
    suspend fun unggah(
        ownerType: String,
        ownerId: String,
        kategori: String,
        namaBerkas: String,
        mime: String,
        byte: ByteArray
    ): String = withContext(Dispatchers.IO) {
        val isi = MultipartBody.Builder()
            .setType(MultipartBody.FORM)
            .addFormDataPart("owner_type", ownerType)
            .addFormDataPart("owner_id", ownerId)
            .addFormDataPart("kategori", kategori)
            .addFormDataPart("file", namaBerkas, byte.toRequestBody(mime.toMediaType()))
            .build()

        val kirimSekali = suspend {
            val pembangun = Request.Builder()
                .url(alamat() + "lampiran")
                .post(isi)
            sesi.aksesToken()?.let { pembangun.header("Authorization", "Bearer $it") }
            klien.newCall(pembangun.build()).execute().use { respons ->
                respons.code to (respons.body?.string().orEmpty())
            }
        }

        var hasil = kirimSekali()
        if (hasil.first == 401 && segarkan()) hasil = kirimSekali()

        if (hasil.first !in 200..299) {
            throw ApiException(hasil.first, pesanError(hasil.second, hasil.first))
        }
        hasil.second
    }

    /** Unggah foto profil (multipart) ke `POST /auth/foto`. */
    suspend fun unggahFoto(namaBerkas: String, mime: String, byte: ByteArray): String =
        withContext(Dispatchers.IO) {
            val isi = MultipartBody.Builder()
                .setType(MultipartBody.FORM)
                .addFormDataPart("file", namaBerkas, byte.toRequestBody(mime.toMediaType()))
                .build()

            val kirimSekali = suspend {
                val pembangun = Request.Builder()
                    .url(alamat() + "auth/foto")
                    .post(isi)
                sesi.aksesToken()?.let { pembangun.header("Authorization", "Bearer $it") }
                klien.newCall(pembangun.build()).execute().use { respons ->
                    respons.code to (respons.body?.string().orEmpty())
                }
            }

            var hasil = kirimSekali()
            if (hasil.first == 401 && segarkan()) hasil = kirimSekali()

            if (hasil.first !in 200..299) {
                throw ApiException(hasil.first, pesanError(hasil.second, hasil.first))
            }
            hasil.second
        }

    /** Mengunduh berkas biner (mis. lampiran) dengan bearer token + refresh otomatis. */
    suspend fun unduh(path: String): ByteArray {
        val url = alamat() + path.removePrefix("/")
        var hasil = unduhSekali(url)

        if (hasil.first == 401 && segarkan()) {
            hasil = unduhSekali(url)
        }

        val (status, isi) = hasil
        if (status !in 200..299) {
            throw ApiException(status, pesanError(isi.toString(Charsets.UTF_8), status))
        }
        return isi
    }

    private suspend fun unduhSekali(url: String): Pair<Int, ByteArray> = withContext(Dispatchers.IO) {
        val pembangun = Request.Builder().url(url).get()
        sesi.aksesToken()?.let { pembangun.header("Authorization", "Bearer $it") }

        try {
            klien.newCall(pembangun.build()).execute().use { respons ->
                respons.code to (respons.body?.bytes() ?: ByteArray(0))
            }
        } catch (galat: IOException) {
            throw ApiException(0, "Tidak dapat terhubung ke server. Periksa jaringan atau alamat server.")
        }
    }

    companion object {
        /** Emulator: 10.0.2.2 menuju localhost mesin pengembang. */
        const val ALAMAT_BAWAAN = "http://10.0.2.2:8080/api/v1/"
    }
}
