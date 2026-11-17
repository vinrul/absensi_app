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
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import id.polri.poldabali.kinerja.ui.theme.Warna

@Composable
fun NilaiScreen(vm: KinerjaViewModel) {
    LaunchedEffect(vm.pengguna?.id, vm.periode?.id) {
        if (vm.nilai == null) vm.muatNilai()
    }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        val nilai = vm.nilai?.data

        Kartu {
            JudulBagian("Nilai Kinerja", vm.periode?.nama ?: "Periode berjalan")

            if (nilai == null) {
                Spacer(Modifier.height(8.dp))
                LayarKosong(
                    if (vm.sedangMemproses) "Memuat nilai…"
                    else "Nilai belum dihitung pada periode ini."
                )
                TombolKedua(teks = "Muat ulang", onClick = { vm.muatNilai() })
            } else {
                Spacer(Modifier.height(12.dp))
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column {
                        Text(
                            String.format(java.util.Locale.US, "%.2f", nilai.nilaiAkhir),
                            style = MaterialTheme.typography.headlineSmall,
                            color = Warna.Teks,
                            fontWeight = FontWeight.Bold
                        )
                        Text(
                            "Nilai akhir berbobot",
                            style = MaterialTheme.typography.bodySmall,
                            color = Warna.TeksLembut
                        )
                    }
                    Lencana(labelKategori(nilai.kategori), warnaKategori(nilai.kategori))
                }

                Spacer(Modifier.height(12.dp))
                HorizontalDivider(color = Warna.NavySoft)
                Spacer(Modifier.height(8.dp))

                BarNilai("Absensi (20%)", nilai.skorAbsensi, Warna.Hijau)
                BarNilai("Rencana Kerja (20%)", nilai.skorRencanaKerja, Warna.Biru)
                BarNilai("Hasil Kerja (30%)", nilai.skorHasilKerja, Warna.Navy)
                BarNilai("Output (30%)", nilai.skorOutput, Warna.Gold)

                nilai.catatanPimpinan?.let {
                    Spacer(Modifier.height(10.dp))
                    KotakPesan("Catatan pimpinan: $it", JenisPesan.Info)
                }

                nilai.keputusan?.let {
                    Spacer(Modifier.height(8.dp))
                    BarisInfo(
                        "Keputusan",
                        if (it == "disetujui") "Disetujui" else "Perlu Perbaikan",
                        if (it == "disetujui") Warna.Hijau else Warna.Kuning
                    )
                }
            }
        }

        if (vm.nilai?.komponen?.isNotEmpty() == true) {
            Kartu {
                JudulBagian("Rincian Komponen", "Dasar perhitungan tiap indikator")
                Spacer(Modifier.height(8.dp))
                vm.nilai?.komponen?.forEach { komponen ->
                    BarisInfo(
                        label = komponen.indikator.replace('_', ' ').replaceFirstChar { it.uppercase() },
                        nilai = String.format(java.util.Locale.US, "%.2f", komponen.skor)
                    )
                }
            }
        }

        Kartu {
            JudulBagian("Cara Penilaian")
            Spacer(Modifier.height(8.dp))
            Text(
                "Nilai akhir = 20% Absensi + 20% Rencana Kerja + 30% Hasil Kerja + 30% Output.\n" +
                    "Kategori: Sangat Baik ≥ 95, Baik 90–94,99, Cukup 80–89,99, Perlu Pembinaan < 80.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }
    }
}
