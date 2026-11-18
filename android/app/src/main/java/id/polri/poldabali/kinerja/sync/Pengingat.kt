package id.polri.poldabali.kinerja.sync

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat
import androidx.work.Data
import androidx.work.ExistingWorkPolicy
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.Worker
import androidx.work.WorkerParameters
import id.polri.poldabali.kinerja.MainActivity
import id.polri.poldabali.kinerja.data.JadwalPhSaya
import java.time.LocalDate
import java.time.LocalTime
import java.time.ZoneId
import java.time.ZonedDateTime
import java.util.concurrent.TimeUnit

/**
 * Pengingat jadwal PH tanpa FCM.
 *
 * Jadwal bulanan disinkronkan ke perangkat, lalu pengingat lokal dijadwalkan
 * dengan WorkManager. Tidak ada push dari server; pengingat baru berubah
 * setelah aplikasi melakukan sinkronisasi jadwal.
 */
object Pengingat {
    const val CHANNEL_ID = "jadwal_ph"
    private const val TAG = "pengingat_ph"

    fun buatChannel(context: Context) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val manager = context.getSystemService(NotificationManager::class.java) ?: return
        val channel = NotificationChannel(
            CHANNEL_ID,
            "Pengingat Strong Point",
            NotificationManager.IMPORTANCE_DEFAULT
        ).apply {
            description = "Pengingat jadwal strong point"
        }
        manager.createNotificationChannel(channel)
    }

    fun jadwalkan(context: Context, jadwal: List<JadwalPhSaya>) {
        buatChannel(context)
        val work = WorkManager.getInstance(context)
        work.cancelAllWorkByTag(TAG)

        val sekarang = ZonedDateTime.now()
        for (item in jadwal) {
            val tanggal = runCatching { LocalDate.parse(item.tanggal) }.getOrNull() ?: continue
            val semuaHari = item.jamMulai == "00:00" && item.jamSelesai == "23:59"
            val mulai = runCatching { LocalTime.parse(item.jamMulai) }.getOrNull()
            val lokasi = buildString {
                append(item.strongPointNama)
                item.regu?.takeIf { it.isNotBlank() }?.let { append(" ($it)") }
            }

            val hMinusSatu = tanggal.minusDays(1).atTime(19, 0)
                .atZone(ZoneId.systemDefault())
            if (hMinusSatu.isAfter(sekarang)) {
                val judul = "Jadwal strong point besok"
                val isi = if (semuaHari) {
                    "Besok $tanggal di $lokasi. Lakukan check-in di lokasi strong point."
                } else {
                    "Besok $tanggal di $lokasi, pukul ${item.jamMulai}–${item.jamSelesai}."
                }
                jadwalkanSatu(context, work, "${item.id}-h1", hMinusSatu, judul, isi)
            }

            if (semuaHari) {
                val pagi = tanggal.atTime(7, 0).atZone(ZoneId.systemDefault())
                if (pagi.isAfter(sekarang)) {
                    jadwalkanSatu(
                        context,
                        work,
                        "${item.id}-pagi",
                        pagi,
                        "Jadwal strong point hari ini",
                        "Hari ini di $lokasi. Lakukan check-in di lokasi strong point."
                    )
                }
            } else if (mulai != null) {
                val mulaiZoned = ZonedDateTime.of(tanggal, mulai, ZoneId.systemDefault())
                val satuJamSebelum = mulaiZoned.minusHours(1)
                if (satuJamSebelum.isAfter(sekarang)) {
                    val judul = "Jadwal strong point 1 jam lagi"
                    val isi = "Di $lokasi pukul ${item.jamMulai}. Lakukan check-in di lokasi."
                    jadwalkanSatu(context, work, "${item.id}-m60", satuJamSebelum, judul, isi)
                }
            }
        }
    }

    private fun jadwalkanSatu(
        context: Context,
        work: WorkManager,
        kunci: String,
        waktu: ZonedDateTime,
        judul: String,
        isi: String
    ) {
        val delay = waktu.toInstant().toEpochMilli() - System.currentTimeMillis()
        if (delay <= 0) return
        val data = Data.Builder()
            .putString(PengingatWorker.KEY_JUDUL, judul)
            .putString(PengingatWorker.KEY_ISI, isi)
            .build()
        val permintaan = OneTimeWorkRequestBuilder<PengingatWorker>()
            .setInitialDelay(delay, TimeUnit.MILLISECONDS)
            .setInputData(data)
            .addTag(TAG)
            .build()
        work.enqueueUniqueWork("$TAG-$kunci", ExistingWorkPolicy.REPLACE, permintaan)
    }
}

class PengingatWorker(context: Context, params: WorkerParameters) : Worker(context, params) {
    override fun doWork(): Result {
        val judul = inputData.getString(KEY_JUDUL) ?: "Jadwal Strong Point"
        val isi = inputData.getString(KEY_ISI) ?: "Ada jadwal strong point."

        if (Build.VERSION.SDK_INT >= 33 &&
            ContextCompat.checkSelfPermission(
                applicationContext,
                Manifest.permission.POST_NOTIFICATIONS
            ) != PackageManager.PERMISSION_GRANTED
        ) {
            return Result.success()
        }

        Pengingat.buatChannel(applicationContext)
        val intent = Intent(applicationContext, MainActivity::class.java)
        val pending = PendingIntent.getActivity(
            applicationContext,
            0,
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        val notifikasi = NotificationCompat.Builder(applicationContext, Pengingat.CHANNEL_ID)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentTitle(judul)
            .setContentText(isi)
            .setStyle(NotificationCompat.BigTextStyle().bigText(isi))
            .setContentIntent(pending)
            .setAutoCancel(true)
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .build()

        return runCatching {
            NotificationManagerCompat.from(applicationContext).notify(judul.hashCode(), notifikasi)
            Result.success()
        }.getOrElse { Result.retry() }
    }

    companion object {
        const val KEY_JUDUL = "judul"
        const val KEY_ISI = "isi"
    }
}
