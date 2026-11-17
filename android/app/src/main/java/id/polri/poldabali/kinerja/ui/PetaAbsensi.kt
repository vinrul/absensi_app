package id.polri.poldabali.kinerja.ui

import android.content.Context
import android.graphics.Paint
import android.graphics.drawable.GradientDrawable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import id.polri.poldabali.kinerja.data.TitikDinas
import id.polri.poldabali.kinerja.lokasi.Lokasi
import id.polri.poldabali.kinerja.ui.theme.Warna
import org.osmdroid.config.Configuration
import org.osmdroid.tileprovider.tilesource.TileSourceFactory
import org.osmdroid.util.BoundingBox
import org.osmdroid.util.GeoPoint
import org.osmdroid.views.MapView
import org.osmdroid.views.overlay.Marker
import org.osmdroid.views.overlay.Polygon
import java.io.File

private fun siapkanKonfigurasi(context: Context) {
    Configuration.getInstance().apply {
        load(context, context.getSharedPreferences("osmdroid", Context.MODE_PRIVATE))
        userAgentValue = context.packageName
        osmdroidBasePath = File(context.cacheDir, "osmdroid")
        osmdroidTileCache = File(context.cacheDir, "osmdroid/tiles")
    }
}

private fun ikonBulat(context: Context, warna: Int, ukuranDp: Int): GradientDrawable {
    val ukuran = (ukuranDp * context.resources.displayMetrics.density).toInt()
    return GradientDrawable().apply {
        shape = GradientDrawable.OVAL
        setColor(warna)
        setStroke((2 * context.resources.displayMetrics.density).toInt(), android.graphics.Color.WHITE)
        setSize(ukuran, ukuran)
    }
}

private fun lingkaran(
    peta: MapView,
    pusat: GeoPoint,
    radiusMeter: Double,
    warnaGaris: Int,
    warnaIsi: Int
): Polygon = Polygon(peta).apply {
    points = Polygon.pointsAsCircle(pusat, radiusMeter)
    outlinePaint.color = warnaGaris
    outlinePaint.strokeWidth = 3f
    outlinePaint.style = Paint.Style.STROKE
    fillColor = warnaIsi
}

/**
 * Peta OpenStreetMap untuk halaman validasi absensi.
 * Menampilkan lokasi saat ini, titik dinas terdekat, dan radius titik dinas.
 */
@Composable
fun PetaAbsensi(
    latitude: Double?,
    longitude: Double?,
    akurasiM: Double?,
    titik: TitikDinas?,
    modifier: Modifier = Modifier
) {
    val petaRef = remember { mutableStateOf<MapView?>(null) }
    var kunciTerakhir by remember { mutableStateOf("") }

    Column(modifier) {
        AndroidView(
modifier = Modifier.fillMaxWidth().height(260.dp).clip(RoundedCornerShape(12.dp)),
            factory = { context ->
                siapkanKonfigurasi(context)
                MapView(context).apply {
                    setTileSource(TileSourceFactory.MAPNIK)
                    setMultiTouchControls(true)
                    setTilesScaledToDpi(true)
                    controller.setZoom(16.5)
                    val awal = if (latitude != null && longitude != null) {
                        GeoPoint(latitude, longitude)
                    } else {
                        GeoPoint(-8.65629, 115.22144)
                    }
                    controller.setCenter(awal)
                    onResume()
                    petaRef.value = this
                }
            },
            update = { peta ->
                peta.overlays.clear()

                titik?.let { lokasiTitik ->
                    val pusat = GeoPoint(lokasiTitik.latitude, lokasiTitik.longitude)
                    peta.overlays.add(
                        lingkaran(
                            peta = peta,
                            pusat = pusat,
                            radiusMeter = lokasiTitik.radiusM.toDouble(),
                            warnaGaris = android.graphics.Color.parseColor("#2E7D32"),
                            warnaIsi = android.graphics.Color.argb(45, 46, 125, 50)
                        )
                    )
                    peta.overlays.add(
                        Marker(peta).apply {
                            position = pusat
                            setAnchor(Marker.ANCHOR_CENTER, Marker.ANCHOR_CENTER)
                            icon = ikonBulat(peta.context, android.graphics.Color.parseColor("#C62828"), 22)
                            title = lokasiTitik.nama
                            snippet = "Radius ${lokasiTitik.radiusM} m"
                        }
                    )
                }

                if (latitude != null && longitude != null) {
                    val posisi = GeoPoint(latitude, longitude)
                    if (akurasiM != null && akurasiM > 0) {
                        peta.overlays.add(
                            lingkaran(
                                peta = peta,
                                pusat = posisi,
                                radiusMeter = akurasiM,
                                warnaGaris = android.graphics.Color.parseColor("#1565C0"),
                                warnaIsi = android.graphics.Color.argb(35, 21, 101, 192)
                            )
                        )
                    }
                    peta.overlays.add(
                        Marker(peta).apply {
                            position = posisi
                            setAnchor(Marker.ANCHOR_CENTER, Marker.ANCHOR_CENTER)
                            icon = ikonBulat(peta.context, android.graphics.Color.parseColor("#1565C0"), 18)
                            title = "Lokasi Anda"
                        }
                    )
                }

                peta.invalidate()

                val kunci = "$latitude,$longitude,${titik?.id}"
                if (kunci != kunciTerakhir) {
                    kunciTerakhir = kunci
                    peta.post {
                        val sekarang = if (latitude != null && longitude != null) {
                            GeoPoint(latitude, longitude)
                        } else {
                            null
                        }
                        val pusatTitik = titik?.let { GeoPoint(it.latitude, it.longitude) }

                        when {
                            sekarang != null && pusatTitik != null -> {
                                val jarak = Lokasi.jarakMeter(
                                    sekarang.latitude,
                                    sekarang.longitude,
                                    pusatTitik.latitude,
                                    pusatTitik.longitude
                                )
                                if (jarak > 350) {
                                    peta.zoomToBoundingBox(
                                        BoundingBox.fromGeoPoints(listOf(sekarang, pusatTitik)),
                                        true,
                                        96
                                    )
                                } else {
                                    peta.controller.setCenter(sekarang)
                                    peta.controller.setZoom(17.0)
                                }
                            }
                            sekarang != null -> {
                                peta.controller.setCenter(sekarang)
                                peta.controller.setZoom(17.0)
                            }
                            pusatTitik != null -> {
                                peta.controller.setCenter(pusatTitik)
                                peta.controller.setZoom(17.0)
                            }
                        }
                    }
                }
            }
        )

        		Text(
        			"© OpenStreetMap contributors",
        			style = MaterialTheme.typography.bodySmall,
        			color = Warna.TeksLembut,
        			modifier = Modifier.fillMaxWidth().padding(top = 4.dp)
        		)
        	}

        	DisposableEffect(Unit) {
        		onDispose {
        			petaRef.value?.onPause()
        			petaRef.value?.onDetach()
        			petaRef.value = null
        		}
        	}
        }
