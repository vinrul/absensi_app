package id.polri.poldabali.kinerja.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

/** Warna instansi: navy kepolisian dengan aksen emas. */
object Warna {
    val Navy = Color(0xFF0B2545)
    val NavyLight = Color(0xFF12355A)
    val NavySoft = Color(0xFFD6E2F0)
    val Gold = Color(0xFFD4A017)
    val GoldSoft = Color(0xFFFBEFD0)
    val Hijau = Color(0xFF059669)
    val Kuning = Color(0xFFF59E0B)
    val Merah = Color(0xFFE11D48)
    val Biru = Color(0xFF0284C7)
    val Latar = Color(0xFFF1F5F9)
    val Teks = Color(0xFF1E293B)
    val TeksLembut = Color(0xFF64748B)
}

private val Skema = lightColorScheme(
    primary = Warna.Navy,
    onPrimary = Color.White,
    primaryContainer = Warna.NavySoft,
    onPrimaryContainer = Warna.Navy,
    secondary = Warna.Gold,
    onSecondary = Warna.Navy,
    secondaryContainer = Warna.GoldSoft,
    onSecondaryContainer = Color(0xFF5C4300),
    background = Warna.Latar,
    onBackground = Warna.Teks,
    surface = Color.White,
    onSurface = Warna.Teks,
    surfaceVariant = Color(0xFFE2E8F0),
    onSurfaceVariant = Warna.TeksLembut,
    error = Warna.Merah,
    outline = Color(0xFFCBD5E1)
)

private val Tipografi = Typography(
    headlineSmall = TextStyle(fontSize = 22.sp, fontWeight = FontWeight.Bold),
    titleLarge = TextStyle(fontSize = 18.sp, fontWeight = FontWeight.SemiBold),
    titleMedium = TextStyle(fontSize = 15.sp, fontWeight = FontWeight.SemiBold),
    bodyLarge = TextStyle(fontSize = 15.sp),
    bodyMedium = TextStyle(fontSize = 13.5.sp),
    bodySmall = TextStyle(fontSize = 12.sp),
    labelSmall = TextStyle(fontSize = 10.5.sp, fontWeight = FontWeight.Medium, letterSpacing = 0.4.sp)
)

@Composable
fun KinerjaTheme(content: @Composable () -> Unit) {
    MaterialTheme(colorScheme = Skema, typography = Tipografi, content = content)
}
