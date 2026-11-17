package id.polri.poldabali.kinerja.ui

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
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
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import id.polri.poldabali.kinerja.data.BerkasLampiran
import id.polri.poldabali.kinerja.data.Hasil
import id.polri.poldabali.kinerja.data.HasilRequest
import id.polri.poldabali.kinerja.data.Lampiran
import id.polri.poldabali.kinerja.data.Rencana
import id.polri.poldabali.kinerja.data.UbahHasilRequest
import id.polri.poldabali.kinerja.ui.theme.Warna
import java.time.LocalDate
import kotlin.math.roundToInt

private val JUDUL_HASIL = mapOf(
    1 to "Pilih Rencana Kerja",
    2 to "Input Hasil Kerja",
    3 to "Bukti Pendukung"
)

private val KETERANGAN_HASIL = mapOf(
    1 to "Hasil kerja hanya dapat dilaporkan dari rencana yang sudah disetujui.",
    2 to "Isi uraian pelaksanaan, progres, dan realisasi capaian.",
    3 to "Lampirkan dokumen, foto, atau laporan pendukung."
)

@Composable
fun WizardHasil(
    vm: KinerjaViewModel,
    idHasil: String? = null,
    onTutup: () -> Unit
) {
    val konteks = LocalContext.current
    val modeUbah = idHasil != null
    val daftarRencana = vm.rencanaDisetujui()

    var siap by remember(idHasil) { mutableStateOf(!modeUbah) }
    var statusAwal by remember(idHasil) { mutableStateOf("draft") }

    var langkah by rememberSaveable(idHasil) { mutableStateOf(1) }
    var rencanaId by rememberSaveable(idHasil) {
        mutableStateOf(daftarRencana.firstOrNull()?.id.orEmpty())
    }
    var rencanaLabel by rememberSaveable(idHasil) {
        mutableStateOf(daftarRencana.firstOrNull()?.kegiatan.orEmpty())
    }
    var uraian by rememberSaveable(idHasil) { mutableStateOf("") }
    var progres by rememberSaveable(idHasil) { mutableStateOf("100") }
    var realisasi by rememberSaveable(idHasil) { mutableStateOf("") }
    var kendala by rememberSaveable(idHasil) { mutableStateOf("") }
    var tanggal by rememberSaveable(idHasil) { mutableStateOf(tanggalHariIni()) }

    val lampiranTersimpan = remember(idHasil) { mutableStateListOf<Lampiran>() }
    val idLampiranDihapus = remember(idHasil) { mutableStateListOf<String>() }
    val daftarBukti = remember(idHasil) { mutableStateListOf<BerkasLampiran>() }
    var lampiranDilihat by remember { mutableStateOf<Lampiran?>(null) }

    LaunchedEffect(idHasil) {
        if (idHasil != null) {
            vm.muatDetailHasil(idHasil) { detail ->
                rencanaId = detail.rencanaId
                rencanaLabel = detail.rencanaKegiatan.orEmpty()
                uraian = detail.uraianPelaksanaan
                progres = angkaKeTeksHasil(detail.progresPersen)
                realisasi = angkaKeTeksHasil(detail.jumlahRealisasi)
                kendala = detail.kendala.orEmpty()
                tanggal = detail.tanggalRealisasi
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
                    "Memuat hasil kerja…",
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

    val progresAngka = progres.toDoubleOrNull()
    val tanggalValid = validasiTanggalHasil(tanggal) == null
    val bisaLanjut = when (langkah) {
        1 -> rencanaId.isNotBlank()
        2 -> uraian.isNotBlank() &&
            progresAngka != null &&
            progresAngka in 0.0..100.0 &&
            tanggalValid
        else -> uraian.isNotBlank()
    }

    fun kirim(ajukan: Boolean) {
        val progresNilai = progres.toDoubleOrNull() ?: return
        if (modeUbah) {
            vm.ubahHasil(
                id = idHasil,
                request = UbahHasilRequest(
                    uraianPelaksanaan = uraian.trim(),
                    progresPersen = progresNilai,
                    jumlahRealisasi = realisasi.toDoubleOrNull(),
                    kendala = kendala.ifBlank { null },
                    tanggalRealisasi = tanggal
                ),
                lampiranBaru = daftarBukti.toList(),
                lampiranDihapus = idLampiranDihapus.toList(),
                ajukan = ajukan,
                selesai = onTutup
            )
        } else {
            vm.simpanHasil(
                request = HasilRequest(
                    rencanaId = rencanaId,
                    uraianPelaksanaan = uraian.trim(),
                    progresPersen = progresNilai,
                    jumlahRealisasi = realisasi.toDoubleOrNull(),
                    kendala = kendala.ifBlank { null },
                    tanggalRealisasi = tanggal,
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
                if (modeUbah) "Ubah: ${JUDUL_HASIL[langkah].orEmpty()}" else JUDUL_HASIL[langkah].orEmpty(),
                style = MaterialTheme.typography.titleLarge,
                color = Warna.Teks
            )
            Text(
                KETERANGAN_HASIL[langkah].orEmpty(),
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
                "Mode ubah hasil kerja. Perubahan hanya dapat disimpan selama status draft atau perlu revisi.",
                JenisPesan.Info
            )
        }

        when (langkah) {
            1 -> LangkahPilihRencana(
                daftar = daftarRencana,
                terpilih = rencanaId,
                kunci = modeUbah,
                labelTerpilih = rencanaLabel,
                onPilih = { rencana ->
                    rencanaId = rencana.id
                    rencanaLabel = rencana.kegiatan
                }
            )
            2 -> LangkahRealisasi(
                uraian = uraian,
                onUraian = { uraian = it },
                progres = progres,
                onProgres = { progres = it.filter { c -> c.isDigit() } },
                realisasi = realisasi,
                onRealisasi = { realisasi = it.filter { c -> c.isDigit() || c == '.' } },
                kendala = kendala,
                onKendala = { kendala = it },
                tanggal = tanggal,
                onTanggal = { tanggal = it }
            )
            else -> LangkahBukti(
                lama = lampiranTersimpan,
                baru = daftarBukti,
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
                aktif = bisaLanjut && !vm.sedangMemproses,
                onClick = { kirim(true) }
            )
            TombolKedua(
                teks = if (modeUbah) "Simpan Perubahan" else "Simpan Draf",
                aktif = bisaLanjut && !vm.sedangMemproses,
                onClick = { kirim(false) }
            )
            Text(
                if (modeUbah) {
                    "Perubahan disimpan pada hasil kerja yang sama. Kirim ulang agar atasan memverifikasi."
                } else {
                    "Draf masih dapat diubah dan dikirim kapan saja dari daftar hasil kerja."
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
private fun LangkahPilihRencana(
    daftar: List<Rencana>,
    terpilih: String,
    kunci: Boolean,
    labelTerpilih: String,
    onPilih: (Rencana) -> Unit
) {
    Kartu {
        JudulBagian("Rencana kerja disetujui", "Pilih kegiatan yang akan dilaporkan hasilnya")
        Spacer(Modifier.height(8.dp))

        if (kunci) {
            Text(
                labelTerpilih,
                style = MaterialTheme.typography.bodyMedium,
                color = Warna.Teks,
                fontWeight = FontWeight.SemiBold
            )
            Text(
                "Rencana tidak dapat diubah saat memperbaiki hasil kerja.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        } else if (daftar.isEmpty()) {
            KotakPesan(
                "Belum ada rencana kerja yang disetujui atasan.",
                JenisPesan.Info
            )
        } else {
            daftar.forEach { rencana ->
                Row(
                    Modifier
                        .fillMaxWidth()
                        .padding(vertical = 5.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    RadioButton(
                        selected = terpilih == rencana.id,
                        onClick = { onPilih(rencana) }
                    )
                    Column(Modifier.padding(start = 4.dp)) {
                        Text(
                            rencana.kegiatan,
                            style = MaterialTheme.typography.bodyMedium,
                            color = Warna.Teks,
                            fontWeight = if (terpilih == rencana.id) {
                                FontWeight.SemiBold
                            } else {
                                FontWeight.Normal
                            }
                        )
                        Text(
                            "Target ${rencana.targetKuantitas ?: "-"} ${rencana.satuan ?: ""} · " +
                                "${tanggalIndo(rencana.tanggalMulai)} – ${tanggalIndo(rencana.tanggalSelesai)}",
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
private fun LangkahRealisasi(
    uraian: String,
    onUraian: (String) -> Unit,
    progres: String,
    onProgres: (String) -> Unit,
    realisasi: String,
    onRealisasi: (String) -> Unit,
    kendala: String,
    onKendala: (String) -> Unit,
    tanggal: String,
    onTanggal: (String) -> Unit
) {
    val nilaiProgres = progres.toDoubleOrNull()?.toFloat()?.coerceIn(0f, 100f) ?: 100f

    Kartu {
        JudulBagian("Realisasi pelaksanaan", "Isi capaian sesuai pelaksanaan tugas")
        Spacer(Modifier.height(10.dp))

        OutlinedTextField(
            value = uraian,
            onValueChange = onUraian,
            label = { Text("Uraian pelaksanaan") },
            minLines = 3,
            modifier = Modifier.fillMaxWidth()
        )
        Spacer(Modifier.height(8.dp))
        Text(
            "Progres pelaksanaan",
            style = MaterialTheme.typography.labelLarge,
            color = Warna.TeksLembut
        )
        Row(verticalAlignment = Alignment.CenterVertically) {
            Slider(
                value = nilaiProgres,
                onValueChange = { onProgres(it.roundToInt().toString()) },
                valueRange = 0f..100f,
                colors = SliderDefaults.colors(
                    thumbColor = Warna.Navy,
                    activeTrackColor = Warna.Navy,
                    inactiveTrackColor = Warna.NavySoft
                ),
                modifier = Modifier.weight(1f)
            )
            Spacer(Modifier.width(12.dp))
            Text(
                "${nilaiProgres.roundToInt()}%",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = Warna.Navy
            )
        }
        if (progres.toDoubleOrNull()?.let { it !in 0.0..100.0 } == true) {
            Spacer(Modifier.height(4.dp))
            Text(
                "Progres harus antara 0 dan 100.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.Merah
            )
        }
        Spacer(Modifier.height(8.dp))
        OutlinedTextField(
            value = realisasi,
            onValueChange = onRealisasi,
            label = { Text("Jumlah realisasi") },
            singleLine = true,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
            modifier = Modifier.fillMaxWidth()
        )
        Spacer(Modifier.height(8.dp))
        OutlinedTextField(
            value = kendala,
            onValueChange = onKendala,
            label = { Text("Keterangan / kendala (opsional)") },
            minLines = 2,
            modifier = Modifier.fillMaxWidth()
        )
        Spacer(Modifier.height(8.dp))
        FieldTanggal(
            label = "Tanggal realisasi",
            nilai = tanggal,
            onPilih = onTanggal
        )
        Spacer(Modifier.height(6.dp))
        Text(
            "Tanggal dipilih melalui kalender.",
            style = MaterialTheme.typography.bodySmall,
            color = Warna.TeksLembut
        )
    }
}

@Composable
private fun LangkahBukti(
    lama: List<Lampiran>,
    baru: List<BerkasLampiran>,
    onPilih: () -> Unit,
    onLihatLama: (Lampiran) -> Unit,
    onHapusLama: (Int) -> Unit,
    onHapusBaru: (Int) -> Unit
) {
    val jumlah = lama.size + baru.size

    Kartu {
        JudulBagian("Bukti pendukung", "$jumlah berkas dipilih")
        Spacer(Modifier.height(6.dp))
        Text(
            "Unggah dokumen, foto, atau laporan pendukung. Dapat lebih dari satu berkas.",
            style = MaterialTheme.typography.bodySmall,
            color = Warna.TeksLembut
        )
        Spacer(Modifier.height(8.dp))

        if (jumlah == 0) {
            Text(
                "Belum ada bukti pendukung.",
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
                        "${ukuranBerkasHasil(berkas.sizeBytes)} · tersimpan",
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
                        "${ukuranBerkasHasil(berkas.isi.size.toLong())} · baru",
                        style = MaterialTheme.typography.bodySmall,
                        color = Warna.TeksLembut
                    )
                }
                TextButton(onClick = { onHapusBaru(indeks) }) {
                    Text("Hapus", color = Warna.Merah)
                }
            }
        }

        Spacer(Modifier.height(8.dp))
        TombolKedua(teks = "Tambah Bukti", onClick = onPilih)
        Spacer(Modifier.height(6.dp))
        Text(
            "Format PDF, JPG, PNG, atau MP4. Bukti baru diunggah setelah hasil kerja tersimpan.",
            style = MaterialTheme.typography.bodySmall,
            color = Warna.TeksLembut
        )
    }
}

private fun validasiTanggalHasil(tanggal: String): String? =
    if (runCatching { LocalDate.parse(tanggal.trim()) }.isSuccess) {
        null
    } else {
        "Tanggal realisasi harus dipilih"
    }

private fun angkaKeTeksHasil(nilai: Double?): String = when {
    nilai == null -> ""
    nilai % 1.0 == 0.0 -> nilai.toLong().toString()
    else -> nilai.toString()
}

private fun ukuranBerkasHasil(bytes: Long): String {
    val kb = bytes / 1024
    return if (kb < 1) "<1 KB" else "$kb KB"
}
