package id.polri.poldabali.kinerja.ui

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import id.polri.poldabali.kinerja.data.BerkasLampiran
import id.polri.poldabali.kinerja.data.Hasil
import id.polri.poldabali.kinerja.data.Lampiran
import id.polri.poldabali.kinerja.data.OutputRequest
import id.polri.poldabali.kinerja.data.UbahOutputRequest
import id.polri.poldabali.kinerja.ui.theme.Warna
import java.time.LocalDate

private val JUDUL_OUTPUT = mapOf(
    1 to "Pilih Hasil Kerja",
    2 to "Data Output",
    3 to "Berkas Bukti"
)

private val KETERANGAN_OUTPUT = mapOf(
    1 to "Output hanya dapat dibuat dari hasil kerja yang sudah diverifikasi atasan.",
    2 to "Isi jenis, judul, uraian singkat, dan tanggal hasil.",
    3 to "Unggah berkas bukti. Jenis Link/Sistem cukup menyertakan URL."
)

@Composable
fun WizardOutput(
    vm: KinerjaViewModel,
    idOutput: String? = null,
    onTutup: () -> Unit
) {
    val konteks = LocalContext.current
    val modeUbah = idOutput != null
    val daftarHasil = vm.hasil.filter { it.status == "terverifikasi" }

    var siap by remember(idOutput) { mutableStateOf(!modeUbah) }
    var statusAwal by remember(idOutput) { mutableStateOf("draft") }

    var langkah by rememberSaveable(idOutput) { mutableStateOf(1) }
    var hasilId by rememberSaveable(idOutput) {
        mutableStateOf(daftarHasil.firstOrNull()?.id.orEmpty())
    }
    var hasilLabel by rememberSaveable(idOutput) {
        mutableStateOf(daftarHasil.firstOrNull()?.rencanaKegiatan.orEmpty())
    }
    var jenis by rememberSaveable(idOutput) { mutableStateOf("laporan") }
    var judul by rememberSaveable(idOutput) { mutableStateOf("") }
    var uraian by rememberSaveable(idOutput) { mutableStateOf("") }
    var tanggal by rememberSaveable(idOutput) { mutableStateOf(tanggalHariIni()) }
    var url by rememberSaveable(idOutput) { mutableStateOf("") }

    val lampiranTersimpan = remember(idOutput) { mutableStateListOf<Lampiran>() }
    val idLampiranDihapus = remember(idOutput) { mutableStateListOf<String>() }
    val daftarBukti = remember(idOutput) { mutableStateListOf<BerkasLampiran>() }
    var lampiranDilihat by remember { mutableStateOf<Lampiran?>(null) }

    LaunchedEffect(Unit) {
        if (!modeUbah && daftarHasil.isEmpty()) vm.muatHasil()
    }

    LaunchedEffect(idOutput) {
        if (idOutput != null) {
            vm.muatDetailOutput(idOutput) { detail ->
                hasilId = detail.hasilKerjaId
                hasilLabel = detail.rencanaKegiatan.orEmpty()
                jenis = detail.jenisOutput
                judul = detail.judul
                uraian = detail.uraian.orEmpty()
                tanggal = detail.tanggalHasil
                url = detail.urlEksternal.orEmpty()
                statusAwal = detail.status
                lampiranTersimpan.clear()
                lampiranTersimpan.addAll(detail.lampiran)
                idLampiranDihapus.clear()
                siap = true
            }
        }
    }

    if (!siap) {
        Column(
            Modifier
                .fillMaxWidth()
                .padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            val galatMuat = vm.galat
            if (galatMuat != null) {
                KotakPesan(galatMuat, JenisPesan.Galat)
                TombolKedua(teks = "Kembali", onClick = onTutup)
            } else {
                CircularProgressIndicator(color = Warna.Navy)
                Text(
                    "Memuat output…",
                    style = MaterialTheme.typography.bodySmall,
                    color = Warna.TeksLembut
                )
            }
        }
        return
    }

    val pilihBukti = rememberLauncherForActivityResult(
        ActivityResultContracts.OpenMultipleDocuments()
    ) { daftarUri ->
        daftarUri.forEach { uri ->
            runCatching {
                val nama = namaBerkas(konteks, uri)
                val mime = konteks.contentResolver.getType(uri) ?: "application/octet-stream"
                val isi = konteks.contentResolver.openInputStream(uri)?.use { it.readBytes() }
                if (isi == null || isi.isEmpty()) {
                    vm.pesanGalat("Berkas $nama kosong")
                } else {
                    daftarBukti.add(BerkasLampiran(nama, mime, isi))
                }
            }.onFailure { vm.pesanGalat("Gagal membaca berkas: ${it.message}") }
        }
    }

    val jumlahBerkas = lampiranTersimpan.size + daftarBukti.size
    val tanggalValid = runCatching { LocalDate.parse(tanggal.trim()) }.isSuccess
    val urlValid = url.trim().startsWith("http://") || url.trim().startsWith("https://")
    val dataLengkap = judul.isNotBlank() &&
        tanggalValid &&
        (jenis != "link_sistem" || urlValid)
    val bisaLanjut = when (langkah) {
        1 -> hasilId.isNotBlank()
        2 -> dataLengkap
        else -> dataLengkap
    }
    val bisaKirim = dataLengkap && (jenis == "link_sistem" || jumlahBerkas > 0)

    fun kirim(ajukan: Boolean) {
        if (modeUbah) {
            vm.ubahOutput(
                id = idOutput,
                request = UbahOutputRequest(
                    jenisOutput = jenis,
                    judul = judul.trim(),
                    uraian = uraian.ifBlank { null },
                    tanggalHasil = tanggal,
                    urlEksternal = if (jenis == "link_sistem") url.trim() else null
                ),
                lampiranBaru = daftarBukti.toList(),
                lampiranDihapus = idLampiranDihapus.toList(),
                ajukan = ajukan,
                selesai = onTutup
            )
        } else {
            vm.simpanOutput(
                request = OutputRequest(
                    hasilKerjaId = hasilId,
                    jenisOutput = jenis,
                    judul = judul.trim(),
                    uraian = uraian.ifBlank { null },
                    tanggalHasil = tanggal,
                    urlEksternal = if (jenis == "link_sistem") url.trim() else null,
                    ajukan = ajukan
                ),
                lampiran = daftarBukti.toList(),
                selesai = onTutup
            )
        }
    }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            TextButton(onClick = { if (langkah > 1) langkah-- else onTutup() }) {
                Text(if (langkah > 1) "‹ Kembali" else "Batal", color = Warna.Navy)
            }
            Text(
                "Langkah $langkah dari 3",
                style = MaterialTheme.typography.labelMedium,
                color = Warna.TeksLembut
            )
        }

        Column(Modifier.fillMaxWidth()) {
            Text(
                if (modeUbah) {
                    "Ubah: ${JUDUL_OUTPUT[langkah].orEmpty()}"
                } else {
                    JUDUL_OUTPUT[langkah].orEmpty()
                },
                style = MaterialTheme.typography.titleLarge,
                color = Warna.Teks
            )
            Text(
                KETERANGAN_OUTPUT[langkah].orEmpty(),
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }

        LinearProgressIndicator(
            progress = { langkah / 3f },
            modifier = Modifier
                .fillMaxWidth()
                .height(6.dp),
            color = Warna.Navy,
            trackColor = Warna.NavySoft
        )

        if (modeUbah) {
            KotakPesan(
                "Mode ubah output. Perubahan hanya dapat disimpan selama status draf atau perlu revisi.",
                JenisPesan.Info
            )
        }

        when (langkah) {
            1 -> LangkahPilihHasil(
                daftar = daftarHasil,
                terpilih = hasilId,
                kunci = modeUbah,
                labelTerpilih = hasilLabel,
                onPilih = { hasil ->
                    hasilId = hasil.id
                    hasilLabel = hasil.rencanaKegiatan.orEmpty()
                }
            )
            2 -> LangkahDataOutput(
                jenis = jenis,
                onJenis = { jenis = it },
                judul = judul,
                onJudul = { judul = it },
                uraian = uraian,
                onUraian = { uraian = it },
                tanggal = tanggal,
                onTanggal = { tanggal = it },
                url = url,
                onUrl = { url = it }
            )
            else -> LangkahBerkasOutput(
                lama = lampiranTersimpan,
                baru = daftarBukti,
                jenisLink = jenis == "link_sistem",
                url = url,
                onPilih = {
                    pilihBukti.launch(arrayOf("application/pdf", "image/*", "video/mp4"))
                },
                onLihatLama = { lampiranDilihat = it },
                onHapusLama = { indeks ->
                    val item = lampiranTersimpan.removeAt(indeks)
                    idLampiranDihapus.add(item.id)
                },
                onHapusBaru = { indeks -> daftarBukti.removeAt(indeks) }
            )
        }

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }

        if (langkah < 3) {
            TombolUtama(
                teks = "Lanjut",
                aktif = bisaLanjut,
                onClick = { langkah++ }
            )
        } else {
            TombolUtama(
                teks = if (modeUbah && statusAwal == "perlu_revisi") {
                    "Kirim Ulang ke Atasan"
                } else {
                    "Kirim ke Atasan"
                },
                memuat = vm.sedangMemproses,
                aktif = bisaKirim && !vm.sedangMemproses,
                onClick = { kirim(true) }
            )
            TombolKedua(
                teks = if (modeUbah) "Simpan Perubahan" else "Simpan Draf",
                aktif = dataLengkap && !vm.sedangMemproses,
                onClick = { kirim(false) }
            )
            Text(
                if (modeUbah) {
                    "Perubahan disimpan pada output yang sama. Kirim ulang agar atasan memverifikasi."
                } else {
                    "Draf masih dapat diubah dan dikirim kapan saja dari daftar output."
                },
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }

        Spacer(Modifier.height(8.dp))
    }

    lampiranDilihat?.let { berkas ->
        DialogLihatLampiran(vm, berkas) { lampiranDilihat = null }
    }
}

@Composable
private fun LangkahPilihHasil(
    daftar: List<Hasil>,
    terpilih: String,
    kunci: Boolean,
    labelTerpilih: String,
    onPilih: (Hasil) -> Unit
) {
    Kartu {
        JudulBagian("Hasil kerja terverifikasi", "Pilih hasil kerja yang menjadi dasar output")
        Spacer(Modifier.height(8.dp))

        if (kunci) {
            Text(
                labelTerpilih.ifBlank { "Hasil kerja" },
                style = MaterialTheme.typography.bodyMedium,
                color = Warna.Teks,
                fontWeight = FontWeight.SemiBold
            )
            Text(
                "Hasil kerja tidak dapat diubah saat memperbaiki output.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        } else if (daftar.isEmpty()) {
            KotakPesan(
                "Belum ada hasil kerja terverifikasi. Selesaikan verifikasi hasil kerja terlebih dahulu.",
                JenisPesan.Info
            )
        } else {
            daftar.forEach { hasil ->
                Row(
                    Modifier
                        .fillMaxWidth()
                        .clickable { onPilih(hasil) }
                        .padding(vertical = 5.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    RadioButton(
                        selected = terpilih == hasil.id,
                        onClick = { onPilih(hasil) }
                    )
                    Column(Modifier.padding(start = 4.dp)) {
                        Text(
                            hasil.rencanaKegiatan ?: "Hasil kerja",
                            style = MaterialTheme.typography.bodyMedium,
                            color = Warna.Teks,
                            fontWeight = if (terpilih == hasil.id) {
                                FontWeight.SemiBold
                            } else {
                                FontWeight.Normal
                            }
                        )
                        Text(
                            "Terverifikasi · ${tanggalIndo(hasil.tanggalRealisasi)} · " +
                                "progres ${hasil.progresPersen.toInt()}%",
                            style = MaterialTheme.typography.bodySmall,
                            color = Warna.TeksLembut
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun LangkahDataOutput(
    jenis: String,
    onJenis: (String) -> Unit,
    judul: String,
    onJudul: (String) -> Unit,
    uraian: String,
    onUraian: (String) -> Unit,
    tanggal: String,
    onTanggal: (String) -> Unit,
    url: String,
    onUrl: (String) -> Unit
) {
    Kartu {
        JudulBagian("Data output", "Jenis, judul, dan uraian bukti hasil kerja")
        Spacer(Modifier.height(10.dp))

        Text(
            "Jenis output",
            style = MaterialTheme.typography.labelLarge,
            color = Warna.TeksLembut
        )
        Spacer(Modifier.height(4.dp))
        Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
            JENIS_OUTPUT.chunked(2).forEach { baris ->
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    baris.forEach { (nilai, label) ->
                        FilterChip(
                            selected = jenis == nilai,
                            onClick = { onJenis(nilai) },
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
            onValueChange = onJudul,
            label = { Text("Judul output") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth()
        )
        Spacer(Modifier.height(8.dp))
        OutlinedTextField(
            value = uraian,
            onValueChange = onUraian,
            label = { Text("Uraian singkat") },
            minLines = 3,
            modifier = Modifier.fillMaxWidth()
        )
        Spacer(Modifier.height(8.dp))
        FieldTanggal(
            label = "Tanggal hasil",
            nilai = tanggal,
            onPilih = onTanggal
        )

        if (jenis == "link_sistem") {
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = url,
                onValueChange = onUrl,
                label = { Text("URL / tautan sistem") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )
            Text(
                "Wajib diawali http:// atau https://.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }
    }
}

@Composable
private fun LangkahBerkasOutput(
    lama: List<Lampiran>,
    baru: List<BerkasLampiran>,
    jenisLink: Boolean,
    url: String,
    onPilih: () -> Unit,
    onLihatLama: (Lampiran) -> Unit,
    onHapusLama: (Int) -> Unit,
    onHapusBaru: (Int) -> Unit
) {
    val jumlah = lama.size + baru.size

    Kartu {
        JudulBagian("Berkas bukti", "$jumlah berkas dipilih")
        Spacer(Modifier.height(6.dp))

        if (jenisLink) {
            KotakPesan(
                if (url.isBlank()) {
                    "Jenis Link/Sistem tidak memerlukan berkas, tetapi URL wajib diisi."
                } else {
                    "Tautan sistem: $url"
                },
                JenisPesan.Info
            )
        } else {
            Text(
                "Unggah dokumen, foto, atau laporan pendukung. Dapat lebih dari satu berkas.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }
        Spacer(Modifier.height(8.dp))

        if (jumlah == 0 && !jenisLink) {
            Text(
                "Belum ada berkas bukti.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }

        lama.forEachIndexed { indeks, berkas ->
            Row(
                Modifier
                    .fillMaxWidth()
                    .padding(vertical = 4.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(Modifier.weight(1f)) {
                    Text(
                        berkas.fileName,
                        style = MaterialTheme.typography.bodyMedium,
                        color = Warna.Teks
                    )
                    Text(
                        "${ukuranBerkasOutput(berkas.sizeBytes)} · tersimpan",
                        style = MaterialTheme.typography.bodySmall,
                        color = Warna.TeksLembut
                    )
                }
                Row(verticalAlignment = Alignment.CenterVertically) {
                    TextButton(onClick = { onLihatLama(berkas) }) {
                        Text("Lihat", color = Warna.Navy)
                    }
                    TextButton(onClick = { onHapusLama(indeks) }) {
                        Text("Hapus", color = Warna.Merah)
                    }
                }
            }
        }

        baru.forEachIndexed { indeks, berkas ->
            Row(
                Modifier
                    .fillMaxWidth()
                    .padding(vertical = 4.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(Modifier.weight(1f)) {
                    Text(
                        berkas.nama,
                        style = MaterialTheme.typography.bodyMedium,
                        color = Warna.Teks
                    )
                    Text(
                        "${ukuranBerkasOutput(berkas.isi.size.toLong())} · baru",
                        style = MaterialTheme.typography.bodySmall,
                        color = Warna.TeksLembut
                    )
                }
                TextButton(onClick = { onHapusBaru(indeks) }) {
                    Text("Hapus", color = Warna.Merah)
                }
            }
        }

        if (!jenisLink) {
            Spacer(Modifier.height(8.dp))
            TombolKedua(teks = "Tambah Berkas", onClick = onPilih)
            Spacer(Modifier.height(6.dp))
            Text(
                "Format PDF, JPG, PNG, atau MP4. Berkas baru diunggah setelah output tersimpan.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }
    }
}

private fun ukuranBerkasOutput(bytes: Long): String {
    val kb = bytes / 1024
    return if (kb < 1) "<1 KB" else "$kb KB"
}
