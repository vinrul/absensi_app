package id.polri.poldabali.kinerja.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import id.polri.poldabali.kinerja.data.Absensi
import id.polri.poldabali.kinerja.ui.theme.Warna

@Composable
fun BerandaScreen(
    vm: KinerjaViewModel,
    onBukaAbsensi: () -> Unit,
    onBukaNilai: () -> Unit
) {
    	val pengguna = vm.pengguna
    	val statusServer = vm.absensiHariIni
    	val tanggalTampil = statusServer?.tanggal ?: tanggalHariIni()
    	val dariRiwayat = vm.riwayat.filter { it.tanggal == tanggalTampil }
    	val masuk = statusServer?.masuk ?: dariRiwayat.firstOrNull { it.jenis == "masuk" }
    	val pulang = statusServer?.pulang ?: dariRiwayat.firstOrNull { it.jenis == "pulang" }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Column {
            Text(
                "Selamat bertugas,",
                style = MaterialTheme.typography.bodyMedium,
                color = Warna.TeksLembut
            )
            Text(
                pengguna?.nama ?: "-",
                style = MaterialTheme.typography.titleLarge,
                color = Warna.Teks
            )
            Text(
                "${pengguna?.pangkat} · ${pengguna?.jabatan}",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
            Text(
                "${pengguna?.satkerNama} · ${pengguna?.peranUtama}",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.Gold
            )
        }

        Kartu {
            			JudulBagian("Kehadiran Hari Ini", tanggalIndo(tanggalTampil))
            Spacer(Modifier.height(10.dp))

            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                StatusKehadiranKotak("Masuk", masuk)
                StatusKehadiranKotak("Pulang", pulang)
            }

            Spacer(Modifier.height(12.dp))
            TombolUtama(
                teks = if (masuk == null) "Absen Sekarang" else "Lihat / Absen Pulang",
                onClick = onBukaAbsensi
            )
        }

        if (vm.antrean.isNotEmpty()) {
            Kartu {
                JudulBagian(
                    "Antrean Offline",
                    "${vm.antrean.size} absensi menunggu dikirim"
                )
                Spacer(Modifier.height(8.dp))
                vm.antrean.take(3).forEach { item ->
                    BarisInfo(
                        label = "${item.jenis} · ${labelStatusKehadiran(item.statusKehadiran)}",
                        nilai = item.capturedAt.take(16).replace('T', ' '),
                        warnaNilai = Warna.Kuning
                    )
                }
                Spacer(Modifier.height(10.dp))
                TombolKedua(
                    teks = if (vm.sedangMemproses) "Mengirim…" else "Kirim sekarang",
                    aktif = !vm.sedangMemproses,
                    onClick = { vm.sinkronkanSekarang() }
                )
            }
        }

        Kartu {
            JudulBagian("Nilai Kinerja Saya", vm.periode?.nama ?: "Periode berjalan")
            Spacer(Modifier.height(10.dp))

            val nilai = vm.nilai?.data
            if (nilai == null) {
                Text(
                    "Nilai belum dihitung pada periode ini.",
                    style = MaterialTheme.typography.bodySmall,
                    color = Warna.TeksLembut
                )
                Spacer(Modifier.height(8.dp))
                TombolKedua(teks = "Muat nilai", aktif = !vm.sedangMemproses, onClick = { vm.muatNilai() })
            } else {
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(
                        String.format(java.util.Locale.US, "%.2f", nilai.nilaiAkhir),
                        style = MaterialTheme.typography.headlineSmall,
                        color = Warna.Teks,
                        fontWeight = FontWeight.Bold
                    )
                    Lencana(labelKategori(nilai.kategori), warnaKategori(nilai.kategori))
                }
                Spacer(Modifier.height(8.dp))
                BarNilai("Absensi", nilai.skorAbsensi, Warna.Hijau)
                BarNilai("Rencana Kerja", nilai.skorRencanaKerja, Warna.Biru)
                BarNilai("Hasil Kerja", nilai.skorHasilKerja, Warna.Navy)
                BarNilai("Output", nilai.skorOutput, Warna.Gold)
                Spacer(Modifier.height(6.dp))
                TombolKedua(teks = "Lihat rincian", onClick = onBukaNilai)
            }
        }

        Kartu {
            JudulBagian("Ringkasan Periode", vm.periode?.nama ?: "-")
            Spacer(Modifier.height(8.dp))
            BarisInfo("Rencana kerja saya", "${vm.rencana.size} dokumen")
            BarisInfo("Hasil kerja", "${vm.hasil.size} laporan")
            BarisInfo("Output", "${vm.output.size} bukti")
        }

        Spacer(Modifier.height(4.dp))
        Text(
            "Absensi utama dilakukan melalui aplikasi ini. Bila tidak ada sinyal, data disimpan di perangkat dan dikirim otomatis.",
            style = MaterialTheme.typography.bodySmall,
            color = Warna.TeksLembut
        )
    }
}

@Composable
private fun StatusKehadiranKotak(label: String, item: Absensi?) {
    Column {
        Text(label, style = MaterialTheme.typography.bodySmall, color = Warna.TeksLembut)
        Spacer(Modifier.height(2.dp))
        if (item == null) {
            Text("Belum", style = MaterialTheme.typography.titleMedium, color = Warna.TeksLembut)
        } else {
            Text(item.jam, style = MaterialTheme.typography.titleLarge, color = Warna.Teks)
            Lencana(
                labelStatusKehadiran(item.statusKehadiran),
                warnaStatusKehadiran(item.statusKehadiran)
            )
        }
    }
}

internal val Transparan: Color = Color.Transparent
