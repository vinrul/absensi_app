package id.polri.poldabali.kinerja.data

import android.content.Context
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.first
import kotlinx.serialization.json.Json

/**
 * Penyimpanan sesi & konfigurasi.
 *
 * Catatan keamanan: token disimpan di penyimpanan privat aplikasi (DataStore).
 * Untuk produksi, aktifkan enkripsi berbasis Android Keystore.
 */
class PenyimpananSesi(private val context: Context) {

    private val json = Json { ignoreUnknownKeys = true; encodeDefaults = true }

    private object Kunci {
        val akses = stringPreferencesKey("token_akses")
        val segar = stringPreferencesKey("token_segar")
        val pengguna = stringPreferencesKey("pengguna")
        		val server = stringPreferencesKey("alamat_server")
        		val wizardIzin = booleanPreferencesKey("wizard_izin_selesai")
    }

    suspend fun simpanLogin(akses: String, segar: String, pengguna: Pengguna) {
        context.dataStore.edit { pref ->
            pref[Kunci.akses] = akses
            pref[Kunci.segar] = segar
            pref[Kunci.pengguna] = json.encodeToString(Pengguna.serializer(), pengguna)
        }
    }

    suspend fun simpanPengguna(pengguna: Pengguna) {
        context.dataStore.edit { pref ->
            pref[Kunci.pengguna] = json.encodeToString(Pengguna.serializer(), pengguna)
        }
    }

    suspend fun pengguna(): Pengguna? {
        val teks = context.dataStore.data.first()[Kunci.pengguna] ?: return null
        return runCatching { json.decodeFromString(Pengguna.serializer(), teks) }.getOrNull()
    }

    suspend fun aksesToken(): String? = context.dataStore.data.first()[Kunci.akses]

    suspend fun segarToken(): String? = context.dataStore.data.first()[Kunci.segar]

    suspend fun perbaruiAkses(token: String) {
        context.dataStore.edit { it[Kunci.akses] = token }
    }

    suspend fun perbaruiToken(akses: String, segar: String) {
        context.dataStore.edit { pref ->
            pref[Kunci.akses] = akses
            pref[Kunci.segar] = segar
        }
    }

    suspend fun alamatServer(): String? = context.dataStore.data.first()[Kunci.server]

    	suspend fun setAlamatServer(url: String) {
    		context.dataStore.edit { it[Kunci.server] = url }
    	}

    	suspend fun wizardIzinSelesai(): Boolean =
    		context.dataStore.data.first()[Kunci.wizardIzin] ?: false

    	suspend fun setWizardIzinSelesai() {
    		context.dataStore.edit { it[Kunci.wizardIzin] = true }
    	}

    suspend fun hapusSesi() {
        context.dataStore.edit { pref ->
            pref.remove(Kunci.akses)
            pref.remove(Kunci.segar)
            pref.remove(Kunci.pengguna)
        }
    }
}
