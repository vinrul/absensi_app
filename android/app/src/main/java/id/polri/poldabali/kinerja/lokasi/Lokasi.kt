package id.polri.poldabali.kinerja.lokasi

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.location.Location
import android.provider.Settings
import com.google.android.gms.location.LocationServices
import com.google.android.gms.location.Priority
import com.google.android.gms.tasks.CancellationTokenSource
import androidx.core.content.ContextCompat
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withTimeout
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

object Lokasi {

    fun punyaIzin(context: Context): Boolean {
        val halus = ContextCompat.checkSelfPermission(context, Manifest.permission.ACCESS_FINE_LOCATION)
        val kasar = ContextCompat.checkSelfPermission(context, Manifest.permission.ACCESS_COARSE_LOCATION)
        return halus == PackageManager.PERMISSION_GRANTED || kasar == PackageManager.PERMISSION_GRANTED
    }

    /**
     * Mengambil posisi terkini berakurasi tinggi.
     *
     * Bila perangkat tidak memberi posisi baru dalam batas waktu, dipakai posisi
     * terakhir yang diketahui supaya personel tetap dapat absen di area minim sinyal.
     */
    suspend fun ambil(context: Context, batasMs: Long = 20_000): Location {
        if (!punyaIzin(context)) {
            throw IllegalStateException("Izin lokasi belum diberikan")
        }

        val klien = LocationServices.getFusedLocationProviderClient(context)

        val baru = runCatching {
            withTimeout(batasMs) {
                suspendCancellableCoroutine { lanjut ->
                    val pembatal = CancellationTokenSource()
                    klien.getCurrentLocation(Priority.PRIORITY_HIGH_ACCURACY, pembatal.token)
                        .addOnSuccessListener { lokasi ->
                            if (lokasi != null) lanjut.resume(lokasi)
                            else lanjut.resumeWithException(IllegalStateException("Lokasi tidak tersedia"))
                        }
                        .addOnFailureListener { galat -> lanjut.resumeWithException(galat) }
                    lanjut.invokeOnCancellation { pembatal.cancel() }
                }
            }
        }.getOrNull()

        if (baru != null) return baru

        return withTimeout(5_000) {
            suspendCancellableCoroutine { lanjut ->
                klien.lastLocation
                    .addOnSuccessListener { lokasi ->
                        if (lokasi != null) lanjut.resume(lokasi)
                        else lanjut.resumeWithException(
                            IllegalStateException("Lokasi terakhir tidak tersedia, aktifkan GPS lalu coba lagi")
                        )
                    }
                    .addOnFailureListener { lanjut.resumeWithException(it) }
            }
        }
    }

    fun jarakMeter(lat1: Double, lng1: Double, lat2: Double, lng2: Double): Double {
        val hasil = FloatArray(1)
        Location.distanceBetween(lat1, lng1, lat2, lng2, hasil)
        return hasil[0].toDouble()
    }

    /** Deteksi lokasi tiruan; `isMock` baru tersedia mulai Android 12. */
    @Suppress("DEPRECATION")
    fun adalahMock(lokasi: Location): Boolean =
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.S) {
            lokasi.isMock
        } else {
            lokasi.isFromMockProvider
        }

    data class KondisiLokasi(
        val mock: Boolean,
        val developerOptions: Boolean,
        val aplikasiMock: String?
    )

    /**
     * Pemeriksaan tambahan untuk development/verifikasi:
     * Developer Options dan aplikasi mock yang sedang dipilih. Keduanya hanya
     * menjadi peringatan; keputusan menolak tetap di server.
     */
    fun kondisi(context: Context, lokasi: Location): KondisiLokasi =
        KondisiLokasi(
            mock = adalahMock(lokasi),
            developerOptions = developerOptionsAktif(context),
            aplikasiMock = aplikasiMockTerpilih(context)
        )

    fun developerOptionsAktif(context: Context): Boolean =
        runCatching {
            Settings.Global.getInt(
                context.contentResolver,
                Settings.Global.DEVELOPMENT_SETTINGS_ENABLED,
                0
            ) == 1
        }.getOrDefault(false)

    /** Nama paket aplikasi mock yang dipilih di Developer Options, bila tersedia. */
    fun aplikasiMockTerpilih(context: Context): String? =
        runCatching {
            Settings.Secure.getString(context.contentResolver, "mock_location")
        }.getOrNull()?.takeIf { it.isNotBlank() && it != "null" }
}
