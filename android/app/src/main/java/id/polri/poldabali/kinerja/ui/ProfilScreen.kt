package id.polri.poldabali.kinerja.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import id.polri.poldabali.kinerja.ui.theme.Warna

@Composable
fun ProfilScreen(vm: KinerjaViewModel, onKeluar: () -> Unit) {
    val pengguna = vm.pengguna
    var server by remember { mutableStateOf(vm.alamatServer) }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Kartu {
            JudulBagian("Profil Personel")
            Spacer(Modifier.height(10.dp))
            BarisInfo("Nama", pengguna?.nama ?: "-")
            BarisInfo("NRP", pengguna?.nrp ?: "-")
            BarisInfo("Pangkat", pengguna?.pangkat ?: "-")
            BarisInfo("Jabatan", pengguna?.jabatan ?: "-")
            BarisInfo("Satker", pengguna?.satkerNama ?: "-")
            BarisInfo("Peran", pengguna?.peranUtama ?: "-")
            Spacer(Modifier.height(10.dp))
            HorizontalDivider(color = Warna.NavySoft)
            Spacer(Modifier.height(10.dp))
            TombolKedua(teks = "Muat ulang profil", onClick = { vm.segarkanProfil() })
        }

        Kartu {
            JudulBagian("Server", "Alamat API backend (berakhir dengan /api/v1/)")
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = server,
                onValueChange = { server = it },
                label = { Text("Alamat server") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(Modifier.height(8.dp))
            Text(
                "Contoh emulator: http://10.0.2.2:8080/api/v1/\nContoh perangkat: http://192.168.1.10:8080/api/v1/",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
            Spacer(Modifier.height(8.dp))
            TombolKedua(
                teks = "Simpan Alamat",
                aktif = server.isNotBlank(),
                onClick = { vm.ubahAlamatServer(server) }
            )
        }

        Kartu {
            JudulBagian("Perangkat")
            Spacer(Modifier.height(8.dp))
            BarisInfo("Model", "${android.os.Build.MANUFACTURER} ${android.os.Build.MODEL}")
            BarisInfo("Android", android.os.Build.VERSION.RELEASE ?: "-")
            BarisInfo("Antrean offline", "${vm.antrean.size} absensi")
            Spacer(Modifier.height(8.dp))
            TombolKedua(
                teks = "Kirim antrean sekarang",
                aktif = vm.antrean.isNotEmpty() && !vm.sedangMemproses,
                onClick = { vm.sinkronkanSekarang() }
            )
        }

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
        vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses) }

        TombolUtama(teks = "Keluar", onClick = { vm.keluar(onKeluar) })

        Text(
            "Aplikasi Absensi & Penilaian Kinerja Personel · Polda Bali",
            style = MaterialTheme.typography.bodySmall,
            color = Warna.TeksLembut,
            modifier = Modifier.padding(top = 8.dp)
        )
    }
}
