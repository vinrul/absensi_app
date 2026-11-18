package id.polri.poldabali.kinerja.sync

import android.content.Context
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import id.polri.poldabali.kinerja.data.Graf
import java.util.concurrent.TimeUnit

/** Mengirim antrean absensi offline secara berkala maupun saat diminta. */
class SyncWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        return try {
            Graf.repositori.sinkronkanAntrean()
            Graf.repositori.sinkronkanAntreanPh()
            Result.success()
        } catch (error: Exception) {
            if (runAttemptCount < 5) Result.retry() else Result.failure()
        }
    }
}

object Sinkronisasi {
    private const val NAMA_BERKALA = "sinkron-absensi-berkala"
    private const val NAMA_SEGERA = "sinkron-absensi-segera"

    private val syarat = Constraints.Builder()
        .setRequiredNetworkType(NetworkType.CONNECTED)
        .build()

    fun jadwalkan(context: Context) {
        val permintaan = PeriodicWorkRequestBuilder<SyncWorker>(30, TimeUnit.MINUTES)
            .setConstraints(syarat)
            .build()

        WorkManager.getInstance(context).enqueueUniquePeriodicWork(
            NAMA_BERKALA,
            ExistingPeriodicWorkPolicy.KEEP,
            permintaan
        )
    }

    fun segera(context: Context) {
        val permintaan = OneTimeWorkRequestBuilder<SyncWorker>()
            .setConstraints(syarat)
            .build()

        WorkManager.getInstance(context).enqueueUniqueWork(
            NAMA_SEGERA,
            androidx.work.ExistingWorkPolicy.REPLACE,
            permintaan
        )
    }
}
