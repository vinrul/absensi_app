package id.polri.poldabali.kinerja.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import id.polri.poldabali.kinerja.data.Absensi
import id.polri.poldabali.kinerja.data.JadwalPhSaya
import id.polri.poldabali.kinerja.ui.theme.Warna

@Composable
fun BerandaScreen(
    vm: KinerjaViewModel,
    onBukaAbsensi: () -> Unit
) {
    val pengguna = vm.pengguna
    val statusServer = vm.absensiHariIni
    val tanggalTampil = statusServer?.tanggal ?: tanggalHariIni()
    val dariRiwayat = vm.riwayat.filter { it.tanggal == tanggalTampil }
    val masuk = statusServer?.masuk ?: dariRiwayat.firstOrNull { it.jenis == "masuk" }
    val pulang = statusServer?.pulang ?: dariRiwayat.firstOrNull { it.jenis == "pulang" }
    val jadwalHariIni = vm.jadwalPh.filter { it.tanggal == tanggalTampil }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            val foto = vm.fotoProfil
            if (foto != null) {
                FotoProfil(
                    foto = foto,
                    nama = pengguna?.nama,
                    ukuran = 56.dp,
                    tampilkanInisial = false
                )
                Spacer(Modifier.width(12.dp))
            }
            Column(Modifier.weight(1f)) {
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

        Kartu {
            JudulBagian(
                "Strong Point Hari Ini",
                if (jadwalHariIni.isEmpty()) "Tidak ada jadwal" else "${jadwalHariIni.size} sesi"
            )
            Spacer(Modifier.height(8.dp))
            if (jadwalHariIni.isEmpty()) {
                Text(
                    "Tidak ada jadwal strong point pada tanggal ini.",
                    style = MaterialTheme.typography.bodySmall,
                    color = Warna.TeksLembut
                )
            } else {
                jadwalHariIni.forEach { item ->
                    BarisInfo(
                        label = labelSesi(item),
                        nilai = item.strongPointNama
                    )
                }
            }
            Spacer(Modifier.height(10.dp))
            TombolKedua(
                teks = if (jadwalHariIni.isEmpty()) "Buka Absensi" else "Absen Strong Point",
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

        if (vm.antreanPh.isNotEmpty()) {
            Kartu {
                JudulBagian(
                    "Antrean Strong Point",
                    "${vm.antreanPh.size} data menunggu dikirim"
                )
                Spacer(Modifier.height(10.dp))
                TombolKedua(
                    teks = if (vm.sedangMemproses) "Mengirim…" else "Kirim sekarang",
                    aktif = !vm.sedangMemproses,
                    onClick = { vm.sinkronkanSekarang() }
                )
            }
        }

        Spacer(Modifier.height(4.dp))
        Text(
            "Absensi utama dilakukan melalui aplikasi ini. Bila tidak ada sinyal, data disimpan di perangkat dan dikirim otomatis.",
            style = MaterialTheme.typography.bodySmall,
            color = Warna.TeksLembut
        )
    }
}

private fun labelSesi(item: JadwalPhSaya): String {
    val sesi = if (item.sesi == "sore") "Sore" else "Pagi"
    return "$sesi ${item.jamMulai}–${item.jamSelesai}"
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
