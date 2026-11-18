package id.polri.poldabali.kinerja.lokasi

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import okhttp3.OkHttpClient
import okhttp3.Request
import java.util.concurrent.TimeUnit

/** Reverse geocoding memakai layanan Nominatim mandiri. */
object Geocoder {
    private const val ALAMAT_DASAR = "https://geocode.vinrul.my.id"

    private val klien = OkHttpClient.Builder()
        .connectTimeout(8, TimeUnit.SECONDS)
        .readTimeout(8, TimeUnit.SECONDS)
        .build()

    private val json = Json { ignoreUnknownKeys = true }

    suspend fun alamat(latitude: Double, longitude: Double): String? = withContext(Dispatchers.IO) {
        val url = "$ALAMAT_DASAR/reverse?format=jsonv2&lat=$latitude&lon=$longitude" +
            "&zoom=18&addressdetails=1&accept-language=id"
        val permintaan = Request.Builder()
            .url(url)
            .header("User-Agent", "AbsensiKinerjaPoldaBali/1.0")
            .header("Accept", "application/json")
            .build()

        runCatching {
            klien.newCall(permintaan).execute().use { respons ->
                if (!respons.isSuccessful) return@use null
                val teks = respons.body?.string() ?: return@use null
                json.parseToJsonElement(teks)
                    .jsonObject["display_name"]
                    ?.jsonPrimitive
                    ?.contentOrNull
            }
        }.getOrNull()
    }
}
