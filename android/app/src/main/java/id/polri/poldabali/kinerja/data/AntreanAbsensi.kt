package id.polri.poldabali.kinerja.data

import android.content.Context
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.first
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.json.Json

/** Antrean absensi offline yang disimpan lokal dan dikirim oleh WorkManager. */
@OptIn(ExperimentalSerializationApi::class)
class AntreanAbsensi(private val context: Context) {

    private val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
        explicitNulls = false
        namingStrategy = kotlinx.serialization.json.JsonNamingStrategy.SnakeCase
    }

    private val kunci = stringPreferencesKey("antrean_absensi")

    suspend fun daftar(): List<AbsensiAntrean> {
        val teks = context.dataStore.data.first()[kunci] ?: return emptyList()
        return runCatching {
            json.decodeFromString(kotlinx.serialization.builtins.ListSerializer(AbsensiAntrean.serializer()), teks)
        }.getOrDefault(emptyList())
    }

    suspend fun jumlah(): Int = daftar().size

    suspend fun tambah(item: AbsensiAntrean) {
        val sekarang = daftar().filterNot { it.clientUuid == item.clientUuid } + item
        simpan(sekarang)
    }

    suspend fun hapus(clientUuid: String) {
        simpan(daftar().filterNot { it.clientUuid == clientUuid })
    }

    suspend fun kosongkan() = simpan(emptyList())

    private suspend fun simpan(items: List<AbsensiAntrean>) {
        val teks = json.encodeToString(kotlinx.serialization.builtins.ListSerializer(AbsensiAntrean.serializer()), items)
        context.dataStore.edit { it[kunci] = teks }
    }
}
