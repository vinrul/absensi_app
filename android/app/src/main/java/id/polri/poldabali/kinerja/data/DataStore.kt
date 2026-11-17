package id.polri.poldabali.kinerja.data

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.preferencesDataStore

/** Satu DataStore untuk sesi, konfigurasi server, dan antrean offline. */
internal val Context.dataStore: DataStore<Preferences> by preferencesDataStore(name = "kinerja")
