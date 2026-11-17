package id.polri.poldabali.kinerja.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import id.polri.poldabali.kinerja.R
import id.polri.poldabali.kinerja.ui.theme.Warna

@Composable
fun LoginScreen(
    vm: KinerjaViewModel,
    onBerhasil: () -> Unit
) {
    var nrp by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var server by remember { mutableStateOf(vm.alamatServer) }
    var tampilServer by remember { mutableStateOf(false) }

    Box(
        Modifier
            .fillMaxSize()
            .background(Warna.Navy)
            .imePadding()
    ) {
        Column(
            Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(24.dp),
            verticalArrangement = Arrangement.Center
        ) {
            Surface(
                color = Color.White,
                shape = androidx.compose.foundation.shape.RoundedCornerShape(16.dp),
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

            Spacer(Modifier.height(18.dp))
            Text(
                "Absensi & Penilaian Kinerja",
                style = MaterialTheme.typography.headlineSmall,
                color = androidx.compose.ui.graphics.Color.White
            )
            Text(
                "Personel Polda Bali dan Polres jajaran",
                style = MaterialTheme.typography.bodyMedium,
                color = Warna.NavySoft
            )

            Spacer(Modifier.height(28.dp))

            Surface(
                shape = androidx.compose.foundation.shape.RoundedCornerShape(18.dp),
                color = androidx.compose.ui.graphics.Color.White
            ) {
                Column(Modifier.padding(20.dp)) {
                    Text("Masuk", style = MaterialTheme.typography.titleLarge, color = Warna.Teks)
                    Text(
                        "Gunakan NRP dan password dari admin satker",
                        style = MaterialTheme.typography.bodySmall,
                        color = Warna.TeksLembut
                    )

                    Spacer(Modifier.height(16.dp))

                    OutlinedTextField(
                        value = nrp,
                        onValueChange = { nrp = it.filter { c -> c.isDigit() } },
                        label = { Text("NRP") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth(),
                        keyboardOptions = KeyboardOptions(
                            keyboardType = KeyboardType.Number,
                            imeAction = ImeAction.Next
                        )
                    )

                    Spacer(Modifier.height(12.dp))

                    OutlinedTextField(
                        value = password,
                        onValueChange = { password = it },
                        label = { Text("Password") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth(),
                        visualTransformation = PasswordVisualTransformation(),
                        keyboardOptions = KeyboardOptions(
                            keyboardType = KeyboardType.Password,
                            imeAction = ImeAction.Done
                        )
                    )

                    if (tampilServer) {
                        Spacer(Modifier.height(12.dp))
                        OutlinedTextField(
                            value = server,
                            onValueChange = { server = it },
                            label = { Text("Alamat server") },
                            supportingText = { Text("Contoh: http://192.168.1.10:8080/api/v1/") },
                            singleLine = true,
                            modifier = Modifier.fillMaxWidth()
                        )
                    }

                    Spacer(Modifier.height(6.dp))
                    TextButton(onClick = { tampilServer = !tampilServer }) {
                        Text(if (tampilServer) "Sembunyikan alamat server" else "Ubah alamat server")
                    }

                    vm.galat?.let {
                        KotakPesan(it, JenisPesan.Galat)
                        Spacer(Modifier.height(10.dp))
                    }

                    TombolUtama(
                        teks = "Masuk",
                        memuat = vm.sedangMemproses,
                        aktif = nrp.isNotBlank() && password.isNotBlank(),
                        onClick = {
                            if (tampilServer && server.isNotBlank()) vm.ubahAlamatServer(server)
                            vm.login(nrp, password, onBerhasil)
                        }
                    )
                }
            }

            Spacer(Modifier.height(18.dp))
            Text(
                "Data personel bersifat rahasia.\nGunakan perangkat dinas dan jangan bagikan password.",
                style = MaterialTheme.typography.bodySmall,
                color = Warna.NavySoft,
                textAlign = TextAlign.Center,
                modifier = Modifier.fillMaxWidth()
            )
        }
    }
}
