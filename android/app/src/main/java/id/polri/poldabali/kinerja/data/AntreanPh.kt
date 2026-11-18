package id.polri.poldabali.kinerja.data

import android.content.Context
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.first
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.json.Json

/** Antrean absensi strong point offline; dikirim oleh WorkManager saat sinyal tersedia. */
@OptIn(ExperimentalSerializationApi::class)
class AntreanPh(private val context: Context) {

    private val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
        explicitNulls = false
        namingStrategy = kotlinx.serialization.json.JsonNamingStrategy.SnakeCase
    }

    private val kunci = stringPreferencesKey("antrean_ph")

    suspend fun daftar(): List<AbsensiPhRequest> {
        val teks = context.dataStore.data.first()[kunci] ?: return emptyList()
        return runCatching {
            json.decodeFromString(ListSerializer(AbsensiPhRequest.serializer()), teks)
        }.getOrDefault(emptyList())
    }

    suspend fun tambah(item: AbsensiPhRequest) {
        val sekarang = daftar().filterNot { it.clientUuid == item.clientUuid } + item
        simpan(sekarang)
    }

    suspend fun hapus(clientUuid: String) {
        simpan(daftar().filterNot { it.clientUuid == clientUuid })
    }

    suspend fun kosongkan() = simpan(emptyList())

    private suspend fun simpan(items: List<AbsensiPhRequest>) {
        val teks = json.encodeToString(ListSerializer(AbsensiPhRequest.serializer()), items)
        context.dataStore.edit { it[kunci] = teks }
    }
}
