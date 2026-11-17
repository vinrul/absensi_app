package id.polri.poldabali.kinerja.ui

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.DateRange
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DatePicker
import androidx.compose.material3.DatePickerDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberDatePickerState
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import id.polri.poldabali.kinerja.data.BerkasLampiran
import id.polri.poldabali.kinerja.data.Lampiran
import id.polri.poldabali.kinerja.data.Rencana
import id.polri.poldabali.kinerja.data.RencanaRequest
import id.polri.poldabali.kinerja.data.Tahapan
import id.polri.poldabali.kinerja.data.TahapanRequest
import id.polri.poldabali.kinerja.data.UbahRencanaRequest
import id.polri.poldabali.kinerja.ui.theme.Warna
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset

private val JENIS_RENCANA = listOf(
    Triple("harian", "Harian", "Kegiatan rutin satu hari, misalnya apel atau patroli."),
    Triple("mingguan", "Mingguan", "Rencana untuk satu minggu kerja."),
    Triple("bulanan", "Bulanan", "Rencana kerja satu bulan berjalan."),
    Triple("triwulan", "Triwulanan/Semester", "Rencana untuk triwulan atau semester."),
    Triple("penugasan_khusus", "Penugasan Khusus", "Tugas khusus berdasarkan perintah atasan.")
)

private val DASAR_TUGAS = listOf(
    "sp" to "Surat Perintah",
    "tor" to "TOR",
    "perintah_lisan" to "Perintah Lisan"
)

private val JUDUL_LANGKAH = mapOf(
    1 to "Jenis Rencana Kerja",
    2 to "Dasar / Referensi Tugas",
    3 to "Rincian Rencana Kegiatan",
    4 to "Lampiran Pendukung"
)

private val KETERANGAN_LANGKAH = mapOf(
    1 to "Tentukan bentuk rencana yang akan disusun.",
    2 to "Isi dasar penugasan yang menjadi acuan kegiatan.",
    3 to "Tambahkan satu atau beberapa kegiatan beserta targetnya.",
    4 to "Lampirkan dokumen pendukung, lalu simpan atau ajukan."
)

/** Satu baris rincian kegiatan pada langkah 3 wizard. */
private data class KegiatanRencana(
    val program: String,
    val kegiatan: String,
    val uraian: String,
    val tujuan: String,
    val indikator: String,
    val targetKuantitas: String,
    val satuan: String,
    val targetKualitas: String,
    val mulai: String,
    val selesai: String
)

/**
 * Wizard rencana kerja 4 langkah.
 *
 * Bila [idRencana] diisi, wizard memuat detail rencana lalu bekerja dalam mode ubah
 * (hanya untuk status draft/perlu revisi, sesuai aturan server).
 */
@Composable
fun WizardRencana(
    vm: KinerjaViewModel,
    idRencana: String? = null,
    onTutup: () -> Unit
) {
    val konteks = LocalContext.current
    val modeUbah = idRencana != null

    var siap by remember(idRencana) { mutableStateOf(!modeUbah) }
    var statusAwal by remember(idRencana) { mutableStateOf("draft") }
    var catatanAtasan by remember(idRencana) { mutableStateOf<String?>(null) }

    var langkah by rememberSaveable(idRencana) { mutableStateOf(1) }
    var jenis by rememberSaveable(idRencana) { mutableStateOf("bulanan") }
    var dasarJenis by rememberSaveable(idRencana) { mutableStateOf("sp") }
    var dasarNomor by rememberSaveable(idRencana) { mutableStateOf("") }
    var dasarTanggal by rememberSaveable(idRencana) { mutableStateOf(tanggalHariIni()) }
    var dasarPerihal by rememberSaveable(idRencana) { mutableStateOf("") }

    val daftarKegiatan = remember(idRencana) { mutableStateListOf<KegiatanRencana>() }
    val lampiranTersimpan = remember(idRencana) { mutableStateListOf<Lampiran>() }
    val idLampiranDihapus = remember(idRencana) { mutableStateListOf<String>() }
    val daftarLampiran = remember(idRencana) { mutableStateListOf<BerkasLampiran>() }

    var bukaEditor by remember { mutableStateOf(false) }
    var indeksSunting by remember { mutableStateOf<Int?>(null) }
    var lampiranDilihat by remember { mutableStateOf<Lampiran?>(null) }

    LaunchedEffect(idRencana) {
        if (idRencana != null) {
            vm.muatDetailRencana(idRencana) { detail ->
                jenis = detail.jenis
                dasarJenis = detail.dasarJenis ?: "sp"
                dasarNomor = detail.dasarNomor.orEmpty()
                dasarTanggal = detail.dasarTanggal ?: tanggalHariIni()
                dasarPerihal = detail.dasarPerihal.orEmpty()
                statusAwal = detail.status
                catatanAtasan = detail.catatanAtasan

                daftarKegiatan.clear()
                detail.tahapan.sortedBy { it.urutan }.forEach { tahap ->
                    daftarKegiatan.add(kegiatanDari(tahap, detail))
                }

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
                    "Memuat rencana kerja…",
                    style = MaterialTheme.typography.bodySmall,
                    color = Warna.TeksLembut
                )
            }
        }
        return
    }

    val pilihLampiran = rememberLauncherForActivityResult(
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
                    daftarLampiran.add(BerkasLampiran(nama, mime, isi))
                }
            }.onFailure { vm.pesanGalat("Gagal membaca berkas: ${it.message}") }
        }
    }

    val dasarValid = dasarJenis == "perintah_lisan" || dasarNomor.isNotBlank()
    val tanggalDasarValid = dasarTanggal.isBlank() ||
        validasiRentang(dasarTanggal, dasarTanggal) == null
    val bisaLanjut = when (langkah) {
        1 -> true
        2 -> dasarValid && tanggalDasarValid
        3 -> daftarKegiatan.isNotEmpty()
        else -> daftarKegiatan.isNotEmpty()
    }

    fun kirim(ajukan: Boolean) {
        val pertama = daftarKegiatan.firstOrNull() ?: return
        if (ajukan && vm.pengguna?.atasanId == null) {
            vm.pesanGalat("Atasan langsung belum ditetapkan untuk akun Anda")
            return
        }

        val mulai = daftarKegiatan.minOf { it.mulai }
        val selesai = daftarKegiatan.maxOf { it.selesai }
        val jumlahLain = daftarKegiatan.size - 1
        val kegiatanInduk = if (jumlahLain == 0) {
            pertama.kegiatan
        } else {
            "${pertama.kegiatan} (+$jumlahLain kegiatan)"
        }
        val tahapan = daftarKegiatan.map { item ->
            TahapanRequest(
                uraian = item.uraian,
                targetKuantitas = item.targetKuantitas.toDoubleOrNull(),
                satuan = item.satuan.ifBlank { null },
                program = item.program.ifBlank { null },
                kegiatan = item.kegiatan.ifBlank { null },
                tujuan = item.tujuan.ifBlank { null },
                indikatorKinerja = item.indikator.ifBlank { null },
                targetKualitas = item.targetKualitas.ifBlank { null },
                tanggalMulai = item.mulai,
                tanggalSelesai = item.selesai
            )
        }

        if (idRencana != null) {
            vm.ubahRencana(
                id = idRencana,
                request = UbahRencanaRequest(
                    program = pertama.program,
                    kegiatan = kegiatanInduk,
                    uraian = pertama.uraian,
                    tujuan = pertama.tujuan.ifBlank { null },
                    indikatorKinerja = pertama.indikator.ifBlank { null },
                    jenis = jenis,
                    dasarJenis = dasarJenis,
                    dasarNomor = dasarNomor.ifBlank { null },
                    dasarTanggal = dasarTanggal.ifBlank { null },
                    dasarPerihal = dasarPerihal.ifBlank { null },
                    targetKuantitas = pertama.targetKuantitas.toDoubleOrNull(),
                    satuan = pertama.satuan.ifBlank { null },
                    targetKualitas = pertama.targetKualitas.ifBlank { null },
                    targetWaktu = selesai,
                    tanggalMulai = mulai,
                    tanggalSelesai = selesai,
                    tahapan = tahapan
                ),
                lampiranBaru = daftarLampiran.toList(),
                lampiranDihapus = idLampiranDihapus.toList(),
                ajukan = ajukan,
                selesai = onTutup
            )
        } else {
            vm.simpanRencana(
                request = RencanaRequest(
                    periodeId = vm.periode?.id,
                    program = pertama.program,
                    kegiatan = kegiatanInduk,
                    uraian = pertama.uraian,
                    tujuan = pertama.tujuan.ifBlank { null },
                    indikatorKinerja = pertama.indikator.ifBlank { null },
                    jenis = jenis,
                    dasarJenis = dasarJenis,
                    dasarNomor = dasarNomor.ifBlank { null },
                    dasarTanggal = dasarTanggal.ifBlank { null },
                    dasarPerihal = dasarPerihal.ifBlank { null },
                    targetKuantitas = pertama.targetKuantitas.toDoubleOrNull(),
                    satuan = pertama.satuan.ifBlank { null },
                    targetKualitas = pertama.targetKualitas.ifBlank { null },
                    targetWaktu = selesai,
                    tanggalMulai = mulai,
                    tanggalSelesai = selesai,
                    tahapan = tahapan,
                    ajukan = ajukan
                ),
                lampiran = daftarLampiran.toList(),
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
                "Langkah $langkah dari 4",
                style = MaterialTheme.typography.labelMedium,
                color = Warna.TeksLembut
            )
        }

        Column(Modifier.fillMaxWidth()) {
            Text(
                if (modeUbah) "Ubah: ${JUDUL_LANGKAH[langkah].orEmpty()}" else JUDUL_LANGKAH[langkah].orEmpty(),
                style = MaterialTheme.typography.titleLarge,
                color = Warna.Teks
            )
            Text(
                KETERANGAN_LANGKAH[langkah].orEmpty(),
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }

        LinearProgressIndicator(
            progress = { langkah / 4f },
            modifier = Modifier
                .fillMaxWidth()
                .height(6.dp),
            color = Warna.Navy,
            trackColor = Warna.NavySoft
        )

        if (modeUbah) {
            KotakPesan(
                "Mode ubah rencana. Perubahan hanya dapat disimpan selama status draft atau perlu revisi.",
                JenisPesan.Info
            )
            catatanAtasan?.let { KotakPesan("Catatan atasan: $it", JenisPesan.Info) }
        }

        when (langkah) {
            1 -> LangkahJenis(jenis) { jenis = it }
            2 -> LangkahDasar(
                dasarJenis = dasarJenis,
                onJenis = { dasarJenis = it },
                nomor = dasarNomor,
                onNomor = { dasarNomor = it },
                tanggal = dasarTanggal,
                onTanggal = { dasarTanggal = it },
                perihal = dasarPerihal,
                onPerihal = { dasarPerihal = it }
            )
            3 -> LangkahKegiatan(
                daftar = daftarKegiatan,
                onTambah = {
                    indeksSunting = null
                    bukaEditor = true
                },
                onUbah = { indeks ->
                    indeksSunting = indeks
                    bukaEditor = true
                },
                onHapus = { indeks -> daftarKegiatan.removeAt(indeks) }
            )
            else -> LangkahLampiran(
                lama = lampiranTersimpan,
                baru = daftarLampiran,
                onPilih = {
                    pilihLampiran.launch(
                        arrayOf("application/pdf", "image/*", "video/mp4")
                    )
                },
                onHapusLama = { indeks ->
                    val item = lampiranTersimpan.removeAt(indeks)
                    idLampiranDihapus.add(item.id)
                },
                onLihatLama = { lampiranDilihat = it },
                onHapusBaru = { indeks -> daftarLampiran.removeAt(indeks) }
            )
        }

        vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }

        if (langkah < 4) {
            TombolUtama(
                teks = "Lanjut",
                aktif = bisaLanjut,
                onClick = { langkah++ }
            )
        } else {
            val labelAjukan = if (modeUbah && statusAwal == "perlu_revisi") {
                "Ajukan Ulang ke Atasan"
            } else {
                "Ajukan ke Atasan"
            }
            TombolUtama(
                teks = labelAjukan,
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
                    "Perubahan disimpan pada rencana yang sama. Ajukan ulang agar atasan meninjau perbaikan."
                } else {
                    "Draf masih dapat diubah dan diajukan kapan saja dari daftar rencana kerja."
                },
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }

        Spacer(Modifier.height(8.dp))
    }

    if (bukaEditor) {
        val sunting = indeksSunting
        DialogKegiatan(
            awal = sunting?.let { daftarKegiatan.getOrNull(it) },
            onTutup = { bukaEditor = false },
            onSimpan = { hasil ->
                if (sunting != null && sunting in daftarKegiatan.indices) {
                    daftarKegiatan[sunting] = hasil
                } else {
                    daftarKegiatan.add(hasil)
                }
                bukaEditor = false
            }
        )
    }

    lampiranDilihat?.let { berkas ->
        DialogLihatLampiran(vm, berkas) { lampiranDilihat = null }
    }
}

@Composable
private fun LangkahJenis(jenis: String, onPilih: (String) -> Unit) {
    Kartu {
        JudulBagian("Jenis rencana", "Menentukan periode dan bentuk rencana kerja")
        Spacer(Modifier.height(8.dp))

        JENIS_RENCANA.forEach { (nilai, label, keterangan) ->
            Row(
                Modifier
                    .fillMaxWidth()
                    .clickable { onPilih(nilai) }
                    .padding(vertical = 6.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                RadioButton(selected = jenis == nilai, onClick = { onPilih(nilai) })
                Column(Modifier.padding(start = 4.dp)) {
                    Text(
                        label,
                        style = MaterialTheme.typography.bodyMedium,
                        color = Warna.Teks,
                        fontWeight = if (jenis == nilai) FontWeight.SemiBold else FontWeight.Normal
                    )
                    Text(
                        keterangan,
                        style = MaterialTheme.typography.bodySmall,
                        color = Warna.TeksLembut
                    )
                }
            }
        }
    }
}

@Composable
private fun LangkahDasar(
    dasarJenis: String,
    onJenis: (String) -> Unit,
    nomor: String,
    onNomor: (String) -> Unit,
    tanggal: String,
    onTanggal: (String) -> Unit,
    perihal: String,
    onPerihal: (String) -> Unit
) {
    Kartu {
        JudulBagian("Dasar tugas", "Surat perintah atau perintah lisan yang menjadi acuan")
        Spacer(Modifier.height(10.dp))

        Text("Jenis dasar tugas", style = MaterialTheme.typography.bodySmall, color = Warna.TeksLembut)
        Spacer(Modifier.height(4.dp))
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            DASAR_TUGAS.forEach { (nilai, label) ->
                FilterChip(
                    selected = dasarJenis == nilai,
                    onClick = { onJenis(nilai) },
                    label = { Text(label) }
                )
            }
        }

        Spacer(Modifier.height(10.dp))
        OutlinedTextField(
            value = nomor,
            onValueChange = onNomor,
            label = { Text(if (dasarJenis == "perintah_lisan") "Nomor (opsional)" else "Nomor SP / TOR") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth()
        )
        Spacer(Modifier.height(8.dp))
        FieldTanggal(
            label = "Tanggal dasar",
            nilai = tanggal,
            onPilih = onTanggal
        )
        Spacer(Modifier.height(8.dp))
        OutlinedTextField(
            value = perihal,
            onValueChange = onPerihal,
            label = { Text("Perihal") },
            minLines = 2,
            modifier = Modifier.fillMaxWidth()
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
private fun LangkahKegiatan(
    daftar: List<KegiatanRencana>,
    onTambah: () -> Unit,
    onUbah: (Int) -> Unit,
    onHapus: (Int) -> Unit
) {
    Kartu {
        JudulBagian("Rincian kegiatan", "${daftar.size} kegiatan ditambahkan")
        Spacer(Modifier.height(8.dp))

        if (daftar.isEmpty()) {
            Text(
                "Belum ada kegiatan. Tambahkan minimal satu rincian rencana.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        } else {
            daftar.forEachIndexed { indeks, item ->
                if (indeks > 0) {
                    Spacer(Modifier.height(10.dp))
                    HorizontalDivider(color = Warna.NavySoft)
                    Spacer(Modifier.height(10.dp))
                }

                Text(
                    "${indeks + 1}. ${item.kegiatan}",
                    style = MaterialTheme.typography.titleSmall,
                    color = Warna.Teks,
                    fontWeight = FontWeight.SemiBold
                )
                Text(
                    item.program,
                    style = MaterialTheme.typography.bodySmall,
                    color = Warna.TeksLembut
                )
                Spacer(Modifier.height(4.dp))
                Text(item.uraian, style = MaterialTheme.typography.bodySmall, color = Warna.Teks)
                Spacer(Modifier.height(4.dp))
                Text(
                    "Target: ${item.targetKuantitas.ifBlank { "-" }} ${item.satuan}".trim(),
                    style = MaterialTheme.typography.bodySmall,
                    color = Warna.TeksLembut
                )
                if (item.targetKualitas.isNotBlank()) {
                    Text(
                        "Kualitas: ${item.targetKualitas}",
                        style = MaterialTheme.typography.bodySmall,
                        color = Warna.TeksLembut
                    )
                }
                Text(
                    "Waktu: ${tanggalIndo(item.mulai)} – ${tanggalIndo(item.selesai)}",
                    style = MaterialTheme.typography.bodySmall,
                    color = Warna.TeksLembut
                )

                Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    TextButton(onClick = { onUbah(indeks) }) {
                        Text("Ubah", color = Warna.Navy)
                    }
                    TextButton(onClick = { onHapus(indeks) }) {
                        Text("Hapus", color = Warna.Merah)
                    }
                }
            }
        }

        Spacer(Modifier.height(10.dp))
        TombolKedua(teks = "+ Tambah Kegiatan", onClick = onTambah)
    }
}

@Composable
private fun LangkahLampiran(
    lama: List<Lampiran>,
    baru: List<BerkasLampiran>,
    onPilih: () -> Unit,
    onHapusLama: (Int) -> Unit,
    onLihatLama: (Lampiran) -> Unit,
    onHapusBaru: (Int) -> Unit
) {
    val jumlah = lama.size + baru.size

    Kartu {
        JudulBagian("Lampiran", "$jumlah dokumen dipilih")
        Spacer(Modifier.height(6.dp))
        Text(
            "Dokumen pendukung seperti SP, TOR, atau bukti lain. Dapat lebih dari satu berkas.",
            style = MaterialTheme.typography.bodySmall,
            color = Warna.TeksLembut
        )
        Spacer(Modifier.height(8.dp))

        if (jumlah == 0) {
            Text(
                "Belum ada lampiran dipilih.",
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
                        "${ukuranTeks(berkas.sizeBytes)} · tersimpan",
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
                        "${ukuranTeks(berkas.isi.size.toLong())} · baru",
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
        TombolKedua(teks = "Tambah Lampiran", onClick = onPilih)
        Spacer(Modifier.height(6.dp))
        Text(
            "Format PDF, JPG, PNG, atau MP4. Berkas baru diunggah setelah rencana tersimpan.",
            style = MaterialTheme.typography.bodySmall,
            color = Warna.TeksLembut
        )
    }
}

@Composable
private fun DialogKegiatan(
    awal: KegiatanRencana?,
    onTutup: () -> Unit,
    onSimpan: (KegiatanRencana) -> Unit
) {
    var program by remember { mutableStateOf(awal?.program ?: "") }
    var kegiatan by remember { mutableStateOf(awal?.kegiatan ?: "") }
    var uraian by remember { mutableStateOf(awal?.uraian ?: "") }
    var tujuan by remember { mutableStateOf(awal?.tujuan ?: "") }
    var indikator by remember { mutableStateOf(awal?.indikator ?: "") }
    var targetKuantitas by remember { mutableStateOf(awal?.targetKuantitas ?: "") }
    var satuan by remember { mutableStateOf(awal?.satuan ?: "laporan") }
    var targetKualitas by remember { mutableStateOf(awal?.targetKualitas ?: "") }
    var mulai by remember { mutableStateOf(awal?.mulai ?: tanggalHariIni()) }
    var selesai by remember { mutableStateOf(awal?.selesai ?: tanggalHariIni()) }
    var galat by remember { mutableStateOf<String?>(null) }

    fun simpan() {
        val pesan = when {
            program.isBlank() -> "Program kerja wajib diisi"
            kegiatan.isBlank() -> "Kegiatan utama wajib diisi"
            uraian.isBlank() -> "Uraian rencana wajib diisi"
            else -> validasiRentang(mulai, selesai)
        }
        galat = pesan
        if (pesan != null) return

        onSimpan(
            KegiatanRencana(
                program = program.trim(),
                kegiatan = kegiatan.trim(),
                uraian = uraian.trim(),
                tujuan = tujuan.trim(),
                indikator = indikator.trim(),
                targetKuantitas = targetKuantitas.trim(),
                satuan = satuan.trim(),
                targetKualitas = targetKualitas.trim(),
                mulai = mulai.trim(),
                selesai = selesai.trim()
            )
        )
    }

    Dialog(onDismissRequest = onTutup) {
        Surface(shape = RoundedCornerShape(16.dp), color = Color.White) {
            Column(
                Modifier
                    .heightIn(max = 620.dp)
                    .imePadding()
                    .padding(16.dp)
            ) {
                Text(
                    if (awal == null) "Tambah Kegiatan" else "Ubah Kegiatan",
                    style = MaterialTheme.typography.titleMedium,
                    color = Warna.Teks
                )
                Spacer(Modifier.height(8.dp))

                Column(
                    Modifier
                        .weight(1f, fill = false)
                        .verticalScroll(rememberScrollState()),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    OutlinedTextField(
                        value = program,
                        onValueChange = { program = it },
                        label = { Text("Program kerja") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth()
                    )
                    OutlinedTextField(
                        value = kegiatan,
                        onValueChange = { kegiatan = it },
                        label = { Text("Kegiatan utama") },
                        modifier = Modifier.fillMaxWidth()
                    )
                    OutlinedTextField(
                        value = uraian,
                        onValueChange = { uraian = it },
                        label = { Text("Uraian rencana") },
                        minLines = 2,
                        modifier = Modifier.fillMaxWidth()
                    )
                    OutlinedTextField(
                        value = tujuan,
                        onValueChange = { tujuan = it },
                        label = { Text("Tujuan (opsional)") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth()
                    )
                    OutlinedTextField(
                        value = indikator,
                        onValueChange = { indikator = it },
                        label = { Text("Indikator kinerja (opsional)") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth()
                    )

                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedTextField(
                            value = targetKuantitas,
                            onValueChange = { targetKuantitas = it.filter { c -> c.isDigit() || c == '.' } },
                            label = { Text("Target kuantitas") },
                            singleLine = true,
                            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                            modifier = Modifier.weight(1f)
                        )
                        OutlinedTextField(
                            value = satuan,
                            onValueChange = { satuan = it },
                            label = { Text("Satuan") },
                            singleLine = true,
                            modifier = Modifier.weight(1f)
                        )
                    }
                    OutlinedTextField(
                        value = targetKualitas,
                        onValueChange = { targetKualitas = it },
                        label = { Text("Target kualitas (opsional)") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth()
                    )

                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        FieldTanggal(
                            label = "Tanggal mulai",
                            nilai = mulai,
                            modifier = Modifier.weight(1f),
                            onPilih = { mulai = it }
                        )
                        FieldTanggal(
                            label = "Tanggal selesai",
                            nilai = selesai,
                            modifier = Modifier.weight(1f),
                            onPilih = { selesai = it }
                        )
                    }

                    galat?.let { KotakPesan(it, JenisPesan.Galat) }
                }

                Spacer(Modifier.height(10.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    TombolKedua(
                        teks = "Batal",
                        modifier = Modifier.weight(1f),
                        onClick = onTutup
                    )
                    TombolUtama(
                        teks = "Simpan",
                        modifier = Modifier.weight(1f),
                        onClick = { simpan() }
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun FieldTanggal(
    label: String,
    nilai: String,
    modifier: Modifier = Modifier,
    onPilih: (String) -> Unit
) {
    var buka by remember { mutableStateOf(false) }
    val tanggal = runCatching { LocalDate.parse(nilai) }.getOrNull()

    Box(modifier.fillMaxWidth()) {
        OutlinedTextField(
            value = tanggal?.format(FORMAT_TANGGAL) ?: "Pilih tanggal",
            onValueChange = {},
            readOnly = true,
            label = { Text(label) },
            trailingIcon = {
                Icon(Icons.Filled.DateRange, contentDescription = "Pilih tanggal", tint = Warna.Navy)
            },
            modifier = Modifier.fillMaxWidth()
        )
        Box(
            Modifier
                .matchParentSize()
                .clickable { buka = true }
        )
    }

    if (buka) {
        val tanggalPicker = rememberDatePickerState(
            initialSelectedDateMillis = tanggal
                ?.atStartOfDay(ZoneOffset.UTC)
                ?.toInstant()
                ?.toEpochMilli()
        )
        DatePickerDialog(
            onDismissRequest = { buka = false },
            confirmButton = {
                TextButton(
                    enabled = tanggalPicker.selectedDateMillis != null,
                    onClick = {
                        tanggalPicker.selectedDateMillis?.let { millis ->
                            onPilih(millisKeTanggal(millis))
                        }
                        buka = false
                    }
                ) {
                    Text("Pilih")
                }
            },
            dismissButton = {
                TextButton(onClick = { buka = false }) {
                    Text("Batal")
                }
            }
        ) {
            DatePicker(state = tanggalPicker)
        }
    }
}

private fun kegiatanDari(tahap: Tahapan, induk: Rencana): KegiatanRencana = KegiatanRencana(
    program = tahap.program?.takeIf { it.isNotBlank() } ?: induk.program,
    kegiatan = tahap.kegiatan?.takeIf { it.isNotBlank() } ?: tahap.uraian,
    uraian = tahap.uraian,
    tujuan = tahap.tujuan ?: induk.tujuan.orEmpty(),
    indikator = tahap.indikatorKinerja ?: induk.indikatorKinerja.orEmpty(),
    targetKuantitas = angkaKeTeks(tahap.targetKuantitas ?: induk.targetKuantitas),
    satuan = tahap.satuan ?: induk.satuan ?: "laporan",
    targetKualitas = tahap.targetKualitas ?: induk.targetKualitas.orEmpty(),
    mulai = tahap.tanggalMulai ?: induk.tanggalMulai,
    selesai = tahap.tanggalSelesai ?: induk.tanggalSelesai
)

private fun angkaKeTeks(nilai: Double?): String = when {
    nilai == null -> ""
    nilai % 1.0 == 0.0 -> nilai.toLong().toString()
    else -> nilai.toString()
}

private fun millisKeTanggal(millis: Long): String =
    Instant.ofEpochMilli(millis).atZone(ZoneOffset.UTC).toLocalDate().toString()

private fun ukuranTeks(bytes: Long): String {
    val kb = bytes / 1024
    return if (kb < 1) "<1 KB" else "$kb KB"
}

private fun validasiRentang(mulai: String, selesai: String): String? {
    val tanggalMulai = runCatching { LocalDate.parse(mulai.trim()) }.getOrNull()
        ?: return "Tanggal mulai harus dipilih"
    val tanggalSelesai = runCatching { LocalDate.parse(selesai.trim()) }.getOrNull()
        ?: return "Tanggal selesai harus dipilih"
    if (tanggalSelesai < tanggalMulai) {
        return "Tanggal selesai tidak boleh sebelum tanggal mulai"
    }
    return null
}

private fun namaBerkas(konteks: Context, uri: Uri): String {
    konteks.contentResolver.query(uri, null, null, null, null)?.use { kursor ->
        val indeks = kursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
        if (indeks >= 0 && kursor.moveToFirst()) {
            return kursor.getString(indeks) ?: uri.lastPathSegment ?: "lampiran"
        }
    }
    return uri.lastPathSegment ?: "lampiran"
}
