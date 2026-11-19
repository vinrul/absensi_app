package id.polri.poldabali.kinerja.ui

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.ExposedDropdownMenuDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuAnchorType
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import id.polri.poldabali.kinerja.data.Pengguna
import id.polri.poldabali.kinerja.data.UbahProfilRequest
import id.polri.poldabali.kinerja.ui.theme.Warna

@Composable
fun ProfilScreen(vm: KinerjaViewModel, onKeluar: () -> Unit) {
    val pengguna = vm.pengguna
    val konteks = LocalContext.current
    var server by remember { mutableStateOf(vm.alamatServer) }
    var editProfil by remember { mutableStateOf(false) }
    var ubahPassword by remember { mutableStateOf(false) }

    LaunchedEffect(pengguna?.id) { vm.muatFotoProfil() }

    val pilihFoto = rememberLauncherForActivityResult(
        ActivityResultContracts.GetContent()
    ) { uri ->
        if (uri != null) {
            runCatching {
                val nama = namaBerkas(konteks, uri)
                val mime = konteks.contentResolver.getType(uri) ?: "image/jpeg"
                val isi = konteks.contentResolver.openInputStream(uri)?.use { it.readBytes() }
                if (isi == null || isi.isEmpty()) {
                    vm.pesanGalat("Berkas foto kosong")
                } else {
                    vm.unggahFotoProfil(nama, mime, isi)
                }
            }.onFailure { vm.pesanGalat("Gagal membaca foto: ${it.message}") }
        }
    }

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Kartu {
            Row(verticalAlignment = Alignment.CenterVertically) {
                FotoProfil(vm.fotoProfil, pengguna?.nama, 84.dp)
                Spacer(Modifier.width(14.dp))
                Column(Modifier.weight(1f)) {
                    Text(
                        pengguna?.nama ?: "-",
                        style = MaterialTheme.typography.titleMedium,
                        color = Warna.Teks,
                        fontWeight = FontWeight.SemiBold
                    )
                    Text(
                        listOfNotNull(pengguna?.pangkat, pengguna?.jabatan)
                            .joinToString(" · "),
                        style = MaterialTheme.typography.bodySmall,
                        color = Warna.TeksLembut
                    )
                    Text(
                        pengguna?.satkerNama ?: "-",
                        style = MaterialTheme.typography.bodySmall,
                        color = Warna.Gold
                    )
                }
            }

            Spacer(Modifier.height(12.dp))
            TombolKedua(
                teks = "Ganti Foto Profil",
                aktif = !vm.sedangMemproses,
                onClick = { pilihFoto.launch("image/*") }
            )
            Spacer(Modifier.height(8.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                TombolKedua(
                    teks = "Edit Profil",
                    modifier = Modifier.weight(1f),
                    aktif = !vm.sedangMemproses,
                    onClick = {
                        vm.bersihkanPesan()
                        editProfil = true
                    }
                )
                TombolKedua(
                    teks = "Ganti Password",
                    modifier = Modifier.weight(1f),
                    aktif = !vm.sedangMemproses,
                    onClick = {
                        vm.bersihkanPesan()
                        ubahPassword = true
                    }
                )
            }
            Spacer(Modifier.height(6.dp))
            Text(
                "Foto JPG/PNG maksimal 5 MB. Satker, NRP, dan peran hanya dapat diubah admin.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.TeksLembut
            )
        }

        Kartu {
            JudulBagian("Profil Personel")
            Spacer(Modifier.height(10.dp))
            BarisInfo("Nama", pengguna?.nama ?: "-")
            BarisInfo("NRP", pengguna?.nrp ?: "-")
            BarisInfo("Pangkat", pengguna?.pangkat ?: "-")
            BarisInfo("Jabatan", pengguna?.jabatan ?: "-")
            BarisInfo("Satker", pengguna?.satkerNama ?: "-")
            BarisInfo("Peran", pengguna?.peranUtama ?: "-")
            BarisInfo("Telepon", pengguna?.phone ?: "-")
            BarisInfo("Email", pengguna?.email ?: "-")
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

    if (editProfil) {
        DialogEditProfil(vm, pengguna) { editProfil = false }
    }

    if (ubahPassword) {
        DialogGantiPassword(vm) { ubahPassword = false }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun DialogEditProfil(
    vm: KinerjaViewModel,
    pengguna: Pengguna?,
    onTutup: () -> Unit
) {
    var nama by remember { mutableStateOf(pengguna?.nama.orEmpty()) }
    var pangkat by remember { mutableStateOf(pengguna?.pangkat.orEmpty()) }
    var jabatan by remember { mutableStateOf(pengguna?.jabatan.orEmpty()) }
    var phone by remember { mutableStateOf(pengguna?.phone.orEmpty()) }
    var email by remember { mutableStateOf(pengguna?.email.orEmpty()) }
    var pilihPangkat by remember { mutableStateOf(false) }
    var galat by remember { mutableStateOf<String?>(null) }

    fun simpan() {
        val namaBersih = nama.trim()
        val jabatanBersih = jabatan.trim()
        val pesan = when {
            namaBersih.length < 3 -> "Nama minimal 3 karakter"
            PANGKAT_SEMUA.none { it.first == pangkat } -> "Pilih pangkat dari daftar"
            jabatanBersih.length < 3 -> "Jabatan minimal 3 karakter"
            email.isNotBlank() && !email.contains('@') -> "Alamat email tidak valid"
            else -> null
        }
        galat = pesan
        if (pesan != null) return

        vm.simpanProfil(
            UbahProfilRequest(
                nama = namaBersih,
                pangkat = pangkat,
                jabatan = jabatanBersih,
                phone = phone.trim().ifBlank { null },
                email = email.trim().ifBlank { null }
            )
        ) { onTutup() }
    }

    Dialog(onDismissRequest = onTutup) {
        Surface(shape = RoundedCornerShape(16.dp), color = Color.White) {
            Column(
                Modifier
                    .heightIn(max = 620.dp)
                    .imePadding()
                    .padding(16.dp)
                    .verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                Text(
                    "Edit Profil",
                    style = MaterialTheme.typography.titleMedium,
                    color = Warna.Teks
                )

                OutlinedTextField(
                    value = nama,
                    onValueChange = { nama = it },
                    label = { Text("Nama lengkap") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )

                ExposedDropdownMenuBox(
                    expanded = pilihPangkat,
                    onExpandedChange = { pilihPangkat = it }
                ) {
                    OutlinedTextField(
                        value = labelPangkat(pangkat),
                        onValueChange = {},
                        readOnly = true,
                        label = { Text("Pangkat") },
                        trailingIcon = {
                            ExposedDropdownMenuDefaults.TrailingIcon(expanded = pilihPangkat)
                        },
                        modifier = Modifier
                            .menuAnchor(MenuAnchorType.PrimaryNotEditable)
                            .fillMaxWidth()
                    )
                    ExposedDropdownMenu(
                        expanded = pilihPangkat,
                        onDismissRequest = { pilihPangkat = false },
                        modifier = Modifier.heightIn(max = 400.dp)
                    ) {
                        PANGKAT_POLRI.forEach { (nilai, label) ->
                            DropdownMenuItem(
                                text = { Text(label) },
                                onClick = {
                                    pangkat = nilai
                                    pilihPangkat = false
                                }
                            )
                        }
                        HorizontalDivider(color = Warna.NavySoft)
                        DropdownMenuItem(
                            text = {
                                Text(
                                    "ASN (PNS)",
                                    style = MaterialTheme.typography.labelSmall,
                                    color = Warna.TeksLembut
                                )
                            },
                            onClick = {},
                            enabled = false
                        )
                        PANGKAT_ASN.forEach { (nilai, label) ->
                            DropdownMenuItem(
                                text = { Text(label) },
                                onClick = {
                                    pangkat = nilai
                                    pilihPangkat = false
                                }
                            )
                        }
                    }
                }

                OutlinedTextField(
                    value = jabatan,
                    onValueChange = { jabatan = it },
                    label = { Text("Jabatan") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
                OutlinedTextField(
                    value = phone,
                    onValueChange = { phone = it },
                    label = { Text("Nomor telepon") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Phone),
                    modifier = Modifier.fillMaxWidth()
                )
                OutlinedTextField(
                    value = email,
                    onValueChange = { email = it },
                    label = { Text("Email") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
                    modifier = Modifier.fillMaxWidth()
                )

                galat?.let { KotakPesan(it, JenisPesan.Galat) }
                vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }

                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    TombolKedua(
                        teks = "Batal",
                        modifier = Modifier.weight(1f),
                        onClick = onTutup
                    )
                    TombolUtama(
                        teks = "Simpan",
                        modifier = Modifier.weight(1f),
                        memuat = vm.sedangMemproses,
                        aktif = !vm.sedangMemproses,
                        onClick = { simpan() }
                    )
                }
            }
        }
    }
}

@Composable
private fun DialogGantiPassword(
    vm: KinerjaViewModel,
    onTutup: () -> Unit
) {
    var lama by remember { mutableStateOf("") }
    var baru by remember { mutableStateOf("") }
    var ulangi by remember { mutableStateOf("") }
    var tampilLama by remember { mutableStateOf(false) }
    var tampilBaru by remember { mutableStateOf(false) }
    var tampilUlangi by remember { mutableStateOf(false) }
    var galat by remember { mutableStateOf<String?>(null) }

    fun simpan() {
        val pesan = when {
            lama.isBlank() -> "Password lama wajib diisi"
            baru.length < 8 -> "Password baru minimal 8 karakter"
            baru != ulangi -> "Konfirmasi password tidak sama"
            else -> null
        }
        galat = pesan
        if (pesan != null) return

        vm.gantiPassword(lama, baru) { onTutup() }
    }

    Dialog(onDismissRequest = onTutup) {
        Surface(shape = RoundedCornerShape(16.dp), color = Color.White) {
            Column(
                Modifier
                    .heightIn(max = 620.dp)
                    .imePadding()
                    .padding(16.dp)
                    .verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                Text(
                    "Ganti Password",
                    style = MaterialTheme.typography.titleMedium,
                    color = Warna.Teks
                )
                Text(
                    "Setelah password diubah, semua sesi berakhir dan Anda harus login kembali.",
                    style = MaterialTheme.typography.bodySmall,
                    color = Warna.TeksLembut
                )

                OutlinedTextField(
                    value = lama,
                    onValueChange = { lama = it },
                    label = { Text("Password lama") },
                    singleLine = true,
                    visualTransformation = if (tampilLama) {
                        VisualTransformation.None
                    } else {
                        PasswordVisualTransformation()
                    },
                    trailingIcon = {
                        TombolMata(
                            terbuka = tampilLama,
                            onClick = { tampilLama = !tampilLama }
                        )
                    },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                    modifier = Modifier.fillMaxWidth()
                )
                OutlinedTextField(
                    value = baru,
                    onValueChange = { baru = it },
                    label = { Text("Password baru (min. 8 karakter)") },
                    singleLine = true,
                    visualTransformation = if (tampilBaru) {
                        VisualTransformation.None
                    } else {
                        PasswordVisualTransformation()
                    },
                    trailingIcon = {
                        TombolMata(
                            terbuka = tampilBaru,
                            onClick = { tampilBaru = !tampilBaru }
                        )
                    },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                    modifier = Modifier.fillMaxWidth()
                )
                OutlinedTextField(
                    value = ulangi,
                    onValueChange = { ulangi = it },
                    label = { Text("Ulangi password baru") },
                    singleLine = true,
                    visualTransformation = if (tampilUlangi) {
                        VisualTransformation.None
                    } else {
                        PasswordVisualTransformation()
                    },
                    trailingIcon = {
                        TombolMata(
                            terbuka = tampilUlangi,
                            onClick = { tampilUlangi = !tampilUlangi }
                        )
                    },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                    modifier = Modifier.fillMaxWidth()
                )

                galat?.let { KotakPesan(it, JenisPesan.Galat) }
                vm.galat?.let { KotakPesan(it, JenisPesan.Galat) }

                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    TombolKedua(
                        teks = "Batal",
                        modifier = Modifier.weight(1f),
                        onClick = onTutup
                    )
                    TombolUtama(
                        teks = "Simpan",
                        modifier = Modifier.weight(1f),
                        memuat = vm.sedangMemproses,
                        aktif = !vm.sedangMemproses,
                        onClick = { simpan() }
                    )
                }
            }
        }
    }
}

@Composable
private fun TombolMata(terbuka: Boolean, onClick: () -> Unit) {
    IconButton(
        onClick = onClick,
        modifier = Modifier.semantics {
            contentDescription = if (terbuka) "Sembunyikan password" else "Tampilkan password"
        }
    ) {
        IkonMata(terbuka = terbuka, warna = Warna.TeksLembut)
    }
}
