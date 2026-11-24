package id.polri.poldabali.kinerja

import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Place
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.lifecycle.viewmodel.compose.viewModel
import id.polri.poldabali.kinerja.R
import id.polri.poldabali.kinerja.ui.AbsensiScreen
import id.polri.poldabali.kinerja.ui.BerandaScreen
import id.polri.poldabali.kinerja.ui.KinerjaViewModel
import id.polri.poldabali.kinerja.ui.LoginScreen
import id.polri.poldabali.kinerja.ui.PhScreen
import id.polri.poldabali.kinerja.ui.ProfilScreen
import id.polri.poldabali.kinerja.ui.WizardIzinScreen
import id.polri.poldabali.kinerja.ui.theme.KinerjaTheme
import id.polri.poldabali.kinerja.ui.theme.Warna

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            KinerjaTheme { Aplikasi() }
        }
    }
}

private data class Tab(val label: String, val judul: String, val ikon: ImageVector)

private val DAFTAR_TAB = listOf(
    Tab("Beranda", "Absensi & Strong Point", Icons.Filled.Home),
    Tab("Absensi", "Absensi", Icons.Filled.Place)
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun Aplikasi(vm: KinerjaViewModel = viewModel()) {
    val konteks = LocalContext.current
    val izinNotifikasi = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { }
    LaunchedEffect(Unit) {
        if (Build.VERSION.SDK_INT >= 33 &&
            ContextCompat.checkSelfPermission(
                konteks,
                Manifest.permission.POST_NOTIFICATIONS
            ) != PackageManager.PERMISSION_GRANTED
        ) {
            izinNotifikasi.launch(Manifest.permission.POST_NOTIFICATIONS)
        }
    }

    if (vm.memuatSesi || vm.wizardIzinSelesai == null) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            CircularProgressIndicator(color = Warna.Navy)
        }
        return
    }

    if (vm.wizardIzinSelesai == false) {
        WizardIzinScreen(onSelesai = vm::tandaiWizardIzinSelesai)
        return
    }

    var tabAktif by rememberSaveable { mutableStateOf(0) }
    var bukaProfil by rememberSaveable { mutableStateOf(false) }

    if (vm.pengguna == null) {
        LoginScreen(vm) { bukaProfil = false }
        return
    }

    if (bukaProfil) {
        Scaffold(
            topBar = {
                TopAppBar(
                    title = { Text("Profil & Pengaturan") },
                    colors = TopAppBarDefaults.topAppBarColors(
                        containerColor = Warna.Navy,
                        titleContentColor = Color.White
                    ),
                    navigationIcon = {
                        IconButton(onClick = { bukaProfil = false }) {
                            Text(
                                "‹",
                                color = Color.White,
                                style = androidx.compose.material3.MaterialTheme.typography.headlineSmall
                            )
                        }
                    }
                )
            }
        ) { padding ->
            Box(Modifier.padding(padding)) {
                ProfilScreen(vm) {
                    bukaProfil = false
                    tabAktif = 0
                }
            }
        }
        return
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Image(
                            painter = painterResource(R.drawable.logo_polda_bali),
                            contentDescription = "Logo SIAP Bali",
                            modifier = Modifier.height(38.dp),
                            contentScale = ContentScale.Fit
                        )
                        Column(Modifier.weight(1f).padding(start = 10.dp)) {
                            Text(
                                DAFTAR_TAB[tabAktif].judul,
                                style = MaterialTheme.typography.titleMedium,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                            Text(
                                buildString {
                                    append(vm.jamServerTeks)
                                    append(" WITA")
                                    if (vm.jamServerPalsu) append(" · PALSU")
                                },
                                style = MaterialTheme.typography.labelSmall,
                                color = Color.White.copy(alpha = 0.78f),
                                maxLines = 1
                            )
                        }
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = Warna.Navy,
                    titleContentColor = Color.White
                ),
                actions = {
                    IconButton(onClick = { bukaProfil = true }) {
                        Icon(Icons.Filled.Person, contentDescription = "Profil", tint = Color.White)
                    }
                }
            )
        },
        bottomBar = {
            NavigationBar(containerColor = Color.White) {
                DAFTAR_TAB.forEachIndexed { indeks, tab ->
                    NavigationBarItem(
                        selected = tabAktif == indeks,
                        onClick = {
                            tabAktif = indeks
                        },
                        icon = { Icon(tab.ikon, contentDescription = tab.label) },
                        label = { Text(tab.label, maxLines = 1) }
                    )
                }
            }
        }
    ) { padding ->
        Box(Modifier.padding(padding)) {
            when (tabAktif) {
                0 -> BerandaScreen(
                    vm = vm,
                    onBukaAbsensi = { tabAktif = 1 }
                )

                1 -> AbsensiScreen(vm)
            }
        }
    }
}
