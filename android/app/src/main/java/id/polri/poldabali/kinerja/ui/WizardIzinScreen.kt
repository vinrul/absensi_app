package id.polri.poldabali.kinerja.ui

import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Notifications
import androidx.compose.material.icons.filled.Place
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import id.polri.poldabali.kinerja.R
import id.polri.poldabali.kinerja.ui.theme.Warna

private data class HalamanIzin(
    val judul: String,
    val deskripsi: String,
    val alasan: String,
    val izin: List<String>,
    val ikon: ImageVector,
    val label: String
)

@Composable
fun WizardIzinScreen(onSelesai: () -> Unit) {
    val konteks = LocalContext.current
    var langkah by rememberSaveable { mutableStateOf(0) }
    var versiIzin by remember { mutableIntStateOf(0) }

    val notifikasi = if (Build.VERSION.SDK_INT >= 33) {
        listOf(Manifest.permission.POST_NOTIFICATIONS)
    } else {
        emptyList()
    }

    val halaman = listOf(
        HalamanIzin(
            judul = "Selamat datang di Absensi Kinerja",
            deskripsi = "Sebelum masuk, aplikasi perlu menjelaskan izin yang digunakan. " +
                "Izin hanya diminta sesuai kebutuhan dan tidak digunakan untuk melacak Anda di luar absensi.",
            alasan = "Wizard ini muncul sekali setelah aplikasi dipasang. Anda dapat mengubah izin kapan saja lewat Pengaturan Android.",
            izin = emptyList(),
            ikon = Icons.Filled.CheckCircle,
            label = "Mulai"
        ),
        HalamanIzin(
            judul = "Akses Lokasi",
            deskripsi = "Lokasi dipakai saat Anda menekan tombol absen untuk menghitung jarak ke titik dinas dan memvalidasi radius.",
            alasan = "Aplikasi tidak melacak lokasi di latar belakang. Lokasi diambil hanya ketika Anda membuka halaman absensi atau menekan Ambil Lokasi.",
            izin = listOf(
                Manifest.permission.ACCESS_FINE_LOCATION,
                Manifest.permission.ACCESS_COARSE_LOCATION
            ),
            ikon = Icons.Filled.Place,
            label = "Izinkan Lokasi"
        ),
        HalamanIzin(
            judul = "Kamera",
            deskripsi = "Kamera dipakai untuk mengambil foto bukti kehadiran. Foto bersifat opsional dan dikirim hanya saat Anda menyimpan absensi.",
            alasan = "Aplikasi hanya membuka kamera ketika Anda menekan tombol Ambil Foto. Tidak ada pengambilan foto diam-diam.",
            izin = listOf(Manifest.permission.CAMERA),
            ikon = Icons.Filled.Add,
            label = "Izinkan Kamera"
        ),
        HalamanIzin(
            judul = "Notifikasi",
            deskripsi = "Notifikasi mengingatkan jadwal strong point: sehari sebelumnya pukul 19.00 dan pagi hari saat jadwal berlangsung.",
            alasan = if (notifikasi.isEmpty()) {
                "Versi Android ini tidak memerlukan izin notifikasi terpisah. Pengingat akan muncul otomatis."
            } else {
                "Tanpa izin ini, pengingat jadwal tetap tersimpan di aplikasi tetapi tidak muncul sebagai notifikasi."
            },
            izin = notifikasi,
            ikon = Icons.Filled.Notifications,
            label = "Aktifkan Notifikasi"
        ),
        HalamanIzin(
            judul = "Latar Belakang & Penyimpanan",
            deskripsi = "WorkManager mengirim antrean absensi offline saat sinyal tersedia dan menjadwalkan pengingat lokal.",
            alasan = "Tidak ada layanan lokasi latar belakang. Foto bukti disimpan di cache privat aplikasi, sehingga aplikasi tidak meminta izin penyimpanan luas.",
            izin = emptyList(),
            ikon = Icons.Filled.Lock,
            label = "Mengerti"
        ),
        HalamanIzin(
            judul = "Siap Digunakan",
            deskripsi = "Semua penjelasan selesai. Anda dapat mengubah izin kapan saja melalui Pengaturan Android.",
            alasan = "Jika izin lokasi belum diberikan, absensi berbasis radius tidak dapat dilakukan sampai izin diaktifkan.",
            izin = emptyList(),
            ikon = Icons.Filled.CheckCircle,
            label = "Selesai"
        )
    )

    val aktif = halaman[langkah.coerceIn(0, halaman.lastIndex)]
    val izinDiberikan = remember(langkah, versiIzin) {
        aktif.izin.all {
            ContextCompat.checkSelfPermission(konteks, it) == PackageManager.PERMISSION_GRANTED
        }
    }

    val mintaIzin = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) {
        versiIzin++
    }

    fun lanjut() {
        if (langkah >= halaman.lastIndex) onSelesai() else langkah++
    }

    Box(
        Modifier
            .fillMaxSize()
            .background(Warna.Navy)
    ) {
        Column(
            Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Spacer(Modifier.height(20.dp))
            Surface(
                color = Color.White,
                shape = RoundedCornerShape(16.dp),
                shadowElevation = 4.dp
            ) {
                Image(
                    painter = painterResource(R.drawable.logo_polda_bali),
                    contentDescription = "Logo Polda Bali",
                    modifier = Modifier
                        .height(88.dp)
                        .padding(horizontal = 12.dp, vertical = 8.dp),
                    contentScale = ContentScale.Fit
                )
            }
            Spacer(Modifier.height(16.dp))
            Text(
                "Izin Aplikasi",
                style = MaterialTheme.typography.headlineSmall,
                color = Color.White,
                textAlign = TextAlign.Center,
                modifier = Modifier.fillMaxWidth()
            )
            Text(
                "Penjelasan sebelum Anda masuk",
                style = MaterialTheme.typography.bodyMedium,
                color = Warna.NavySoft,
                textAlign = TextAlign.Center,
                modifier = Modifier.fillMaxWidth()
            )

            Spacer(Modifier.height(22.dp))

            Surface(
                shape = RoundedCornerShape(18.dp),
                color = Color.White,
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(
                    Modifier.padding(20.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Icon(
                        aktif.ikon,
                        contentDescription = null,
                        tint = Warna.Navy,
                        modifier = Modifier.size(40.dp)
                    )
                    Text(aktif.judul, style = MaterialTheme.typography.titleLarge, color = Warna.Teks)
                    Text(
                        aktif.deskripsi,
                        style = MaterialTheme.typography.bodyMedium,
                        color = Warna.TeksLembut
                    )

                    Surface(
                        shape = RoundedCornerShape(12.dp),
                        color = Warna.Navy.copy(alpha = 0.06f)
                    ) {
                        Column(Modifier.padding(12.dp)) {
                            Text(
                                "Mengapa izin ini dipakai?",
                                style = MaterialTheme.typography.labelLarge,
                                color = Warna.Navy,
                                fontWeight = FontWeight.SemiBold
                            )
                            Spacer(Modifier.height(4.dp))
                            Text(
                                aktif.alasan,
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                    }

                    if (aktif.izin.isNotEmpty()) {
                        Text(
                            if (izinDiberikan) "Izin sudah diberikan." else "Izin belum diberikan.",
                            style = MaterialTheme.typography.bodySmall,
                            color = if (izinDiberikan) Warna.Hijau else Warna.Kuning,
                            fontWeight = FontWeight.Medium
                        )
                    }

                    Spacer(Modifier.height(2.dp))

                    TombolUtama(
                        teks = when {
                            aktif.izin.isEmpty() -> aktif.label
                            izinDiberikan -> "Lanjut"
                            else -> aktif.label
                        },
                        onClick = {
                            when {
                                aktif.izin.isEmpty() || izinDiberikan -> lanjut()
                                else -> mintaIzin.launch(aktif.izin.toTypedArray())
                            }
                        }
                    )

                    if (aktif.izin.isNotEmpty() && !izinDiberikan) {
                        TombolKedua(teks = "Lewati dulu", onClick = { lanjut() })
                    }
                }
            }

            Spacer(Modifier.height(16.dp))
            Text(
                "Langkah ${langkah + 1} dari ${halaman.size}",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.NavySoft,
                textAlign = TextAlign.Center,
                modifier = Modifier.fillMaxWidth()
            )
        }
    }
}
