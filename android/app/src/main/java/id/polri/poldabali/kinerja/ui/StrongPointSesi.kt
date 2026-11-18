package id.polri.poldabali.kinerja.ui

import id.polri.poldabali.kinerja.data.AbsensiPh
import id.polri.poldabali.kinerja.data.AbsensiPhRequest
import java.time.Duration
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter

internal data class SesiStrongPoint(
    val jenis: String,
    val waktu: Instant,
    val jam: String,
    val menungguKirim: Boolean
)

internal data class StatusStrongPoint(
    val sesi: List<SesiStrongPoint>
) {
    val masukTerakhir: SesiStrongPoint? = sesi.lastOrNull { it.jenis == "masuk" }
    val pulangTerakhir: SesiStrongPoint? = sesi.lastOrNull { it.jenis == "pulang" }

    val terbuka: Boolean =
        masukTerakhir != null &&
            (pulangTerakhir == null || masukTerakhir.waktu.isAfter(pulangTerakhir.waktu))

    val durasiTerbuka: Duration? =
        if (terbuka && masukTerakhir != null) {
            Duration.between(masukTerakhir.waktu, Instant.now())
        } else {
            null
        }

    /** Check-in terbuka lebih dari 2 jam dianggap sesi pagi yang tidak check-out. */
    val perluCheckInBaru: Boolean =
        terbuka && (durasiTerbuka?.toMinutes() ?: 0) >= 120
}

private val ZONA_WITA: ZoneId = ZoneId.of("Asia/Makassar")
private val FORMAT_JAM: DateTimeFormatter = DateTimeFormatter.ofPattern("HH:mm")

internal fun jamWita(waktu: Instant): String =
    waktu.atZone(ZONA_WITA).format(FORMAT_JAM)

/**
 * Menggabungkan absensi server dan antrean offline menjadi urutan sesi strong point.
 * Data antrean tetap dihitung agar tombol check-in/check-out konsisten saat offline.
 */
internal fun statusStrongPoint(
    jadwalId: String?,
    absensi: List<AbsensiPh>,
    antrean: List<AbsensiPhRequest>
): StatusStrongPoint {
    if (jadwalId == null) return StatusStrongPoint(emptyList())

    val dariServer = absensi
        .filter { it.jadwalPhId == jadwalId }
        .mapNotNull { item ->
            runCatching {
                val waktu = Instant.parse(item.capturedAt)
                SesiStrongPoint(item.jenis, waktu, jamWita(waktu), false)
            }.getOrNull()
        }

    val dariAntrean = antrean
        .filter { it.jadwalPhId == jadwalId }
        .mapNotNull { item ->
            runCatching {
                val waktu = Instant.parse(item.capturedAt)
                SesiStrongPoint(item.jenis, waktu, jamWita(waktu), true)
            }.getOrNull()
        }

    return StatusStrongPoint((dariServer + dariAntrean).sortedBy { it.waktu })
}
