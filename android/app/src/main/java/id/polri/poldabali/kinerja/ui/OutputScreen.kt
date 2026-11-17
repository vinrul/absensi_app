package id.polri.poldabali.kinerja.ui

import android.net.Uri
import android.provider.OpenableColumns
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import id.polri.poldabali.kinerja.data.OutputRequest
import id.polri.poldabali.kinerja.ui.theme.Warna

private val JENIS_OUTPUT = listOf(
    "laporan" to "Laporan",
    "dokumen_pendukung" to "Dokumen Pendukung",
    "foto" to "Foto Kegiatan",
    "video" to "Video Kegiatan",
    "data_diagram" to "Data/Diagram",
    "berita_acara" to "Berita Acara",
    "surat_nota_dinas" to "Surat/Nota Dinas",
    "link_sistem" to "Link/Sistem"
)

@Composable
fun OutputScreen(vm: KinerjaViewModel) {
    var bukaForm by remember { mutableStateOf(false) }

    LaunchedEffect(vm.pengguna?.id) {
        vm.muatOutput()
        if (vm.hasil.isEmpty()) vm.muatHasil()
    }

    if (bukaForm) {
        FormOutput(vm) { bukaForm = false }
        return
    }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Kartu {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                JudulBagian("Output", "${vm.output.size} bukti hasil kerja")
                TextButton(onClick = { bukaForm = true }) { Text("+ Baru", color = Warna.Navy) }
            }
            Spacer(Modifier.height(6.dp))
            Text(
                "Output hanya dapat dibuat dari hasil kerja yang sudah diverifikasi atasan.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }
        vm.pesan?.let { KotakPesan(it, JenisPesan.Sukses) }

        if (vm.output.isEmpty()) {
            Kartu { LayarKosong(if (vm.sedangMemproses) "Memuat…" else "Belum ada output.") }
        } else {
            vm.output.forEach { item ->
                Kartu {
                    Row(
                        Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.Top
                    ) {
                        Column(Modifier.weight(1f)) {
                            Text(item.judul, style = MaterialTheme.typography.titleMedium, color = Warna.Teks)
                            Text(
                                JENIS_OUTPUT.firstOrNull { it.first == item.jenisOutput }?.second
                                    ?: item.jenisOutput,
                                style = MaterialTheme.typography.bodySmall,
                                color = Warna.TeksLembut
                            )
                        }
                        Lencana(
                            labelStatusOutput(item.status),
                            when (item.status) {
                                "disetujui" -> Warna.Hijau
                                "perlu_revisi" -> Warna.Kuning
                                "dalam_verifikasi" -> Warna.Biru
                                else -> Warna.TeksLembut
                            }
                        )
                    }

                    Spacer(Modifier.height(8.dp))
                    BarisInfo("Tanggal hasil", tanggalIndo(item.tanggalHasil))
                    item.skorTotal?.let {
                        BarisInfo(
                            "Nilai output",
                            String.format(java.util.Locale.US, "%.2f", it),
                            Warna.Hijau
                        )
                    }
                    item.uraian?.let { BarisInfo("Uraian", it) }
                    item.urlEksternal?.let { BarisInfo("Tautan", it) }
                    item.catatanVerifikator?.let { BarisInfo("Catatan atasan", it, Warna.Kuning) }
                }
            }
        }
    }
}

@Composable
private fun FormOutput(vm: KinerjaViewModel, onTutup: () -> Unit) {
    val konteks = LocalContext.current
    val daftarHasil = vm.hasil.filter { it.status == "terverifikasi" }

    var hasilId by remember { mutableStateOf(daftarHasil.firstOrNull()?.id ?: "") }
    var jenis by remember { mutableStateOf("laporan") }
    var judul by remember { mutableStateOf("") }
    var uraian by remember { mutableStateOf("") }
    var tanggal by remember { mutableStateOf(tanggalHariIni()) }
    var url by remember { mutableStateOf("") }

    var namaBerkas by remember { mutableStateOf<String?>(null) }
    var mimeBerkas by remember { mutableStateOf<String?>(null) }
    var isiBerkas by remember { mutableStateOf<ByteArray?>(null) }

    fun namaDari(uri: Uri): String {
        val kursor = konteks.contentResolver.query(uri, null, null, null, null)
        kursor?.use {
            val indeks = it.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            if (indeks >= 0 && it.moveToFirst()) return it.getString(indeks)
        }
        return uri.lastPathSegment ?: "bukti"
    }

    val pilihBerkas = rememberLauncherForActivityResult(
        ActivityResultContracts.OpenDocument()
    ) { uri ->
        if (uri != null) {
            runCatching {
                namaBerkas = namaDari(uri)
                mimeBerkas = konteks.contentResolver.getType(uri) ?: "application/octet-stream"
                isiBerkas = konteks.contentResolver.openInputStream(uri)?.use { it.readBytes() }
            }.onFailure { vm.pesanGalat("Gagal membaca berkas: ${it.message}") }
        }
    }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        Kartu {
            JudulBagian("Tambah Output", "Bukti hasil kerja yang sah")
            Spacer(Modifier.height(10.dp))

            if (daftarHasil.isEmpty()) {
                KotakPesan("Belum ada hasil kerja terverifikasi.", JenisPesan.Info)
            } else {
                Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    daftarHasil.take(6).forEach { hasil ->
                        FilterChip(
                            selected = hasilId == hasil.id,
                            onClick = { hasilId = hasil.id },
                            label = { Text(hasil.rencanaKegiatan ?: hasil.tanggalRealisasi) }
                        )
                    }
                }
            }

            Spacer(Modifier.height(10.dp))
            Text("Jenis Output", style = MaterialTheme.typography.bodySmall, color = Warna.TeksLembut)
            Spacer(Modifier.height(4.dp))
            Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                JENIS_OUTPUT.chunked(2).forEach { baris ->
                    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        baris.forEach { (nilai, label) ->
                            FilterChip(
                                selected = jenis == nilai,
                                onClick = { jenis = nilai },
                                label = { Text(label) },
                                modifier = Modifier.weight(1f)
                            )
                        }
                        if (baris.size == 1) Spacer(Modifier.weight(1f))
                    }
                }
            }

            Spacer(Modifier.height(10.dp))
            OutlinedTextField(
                value = judul,
                onValueChange = { judul = it },
                label = { Text("Judul output") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = uraian,
                onValueChange = { uraian = it },
                label = { Text("Uraian singkat") },
                minLines = 2,
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = tanggal,
                onValueChange = { tanggal = it },
                label = { Text("Tanggal hasil (YYYY-MM-DD)") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )

            if (jenis == "link_sistem") {
                Spacer(Modifier.height(8.dp))
                OutlinedTextField(
                    value = url,
                    onValueChange = { url = it },
                    label = { Text("URL / tautan sistem") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
            } else {
                Spacer(Modifier.height(10.dp))
                Text(
                    namaBerkas?.let { "Berkas: $it" } ?: "Belum ada berkas dipilih",
                    style = MaterialTheme.typography.bodySmall,
                    color = if (namaBerkas != null) Warna.Hijau else Warna.TeksLembut
                )
                Spacer(Modifier.height(6.dp))
                TombolKedua(
                    teks = if (namaBerkas == null) "Pilih Berkas Bukti" else "Ganti Berkas",
                    onClick = { pilihBerkas.launch(arrayOf("application/pdf", "image/*", "video/mp4")) }
                )
            }
        }

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }

        TombolUtama(
            teks = "Simpan & Ajukan",
            memuat = vm.sedangMemproses,
            aktif = hasilId.isNotBlank() && judul.isNotBlank() &&
                (jenis == "link_sistem" && url.isNotBlank() || jenis != "link_sistem" && isiBerkas != null),
            onClick = {
                vm.buatOutput(
                    OutputRequest(
                        hasilKerjaId = hasilId,
                        jenisOutput = jenis,
                        judul = judul,
                        uraian = uraian.ifBlank { null },
                        tanggalHasil = tanggal,
                        urlEksternal = if (jenis == "link_sistem") url else null
                    ),
                    namaBerkas = if (jenis == "link_sistem") null else namaBerkas,
                    mime = if (jenis == "link_sistem") null else mimeBerkas,
                    byte = if (jenis == "link_sistem") null else isiBerkas
                ) { onTutup() }
            }
        )

        TombolKedua(teks = "Batal", onClick = onTutup)
        Spacer(Modifier.height(8.dp))
    }
}
