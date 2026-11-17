package id.polri.poldabali.kinerja.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.size
import androidx.compose.material3.LocalContentColor
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp

/**
 * Ikon mata untuk tombol tampilkan/sembunyikan password.
 * Dibuat manual agar tidak menambah dependensi material-icons-extended.
 */
@Composable
fun IkonMata(
    terbuka: Boolean,
    modifier: Modifier = Modifier,
    warna: Color = LocalContentColor.current
) {
    Canvas(modifier = modifier.size(24.dp)) {
        val lebar = size.width
        val tinggi = size.height
        val tebal = lebar * 0.07f

        val bentukMata = Path().apply {
            moveTo(lebar * 0.08f, tinggi * 0.5f)
            cubicTo(
                lebar * 0.30f, tinggi * 0.18f,
                lebar * 0.70f, tinggi * 0.18f,
                lebar * 0.92f, tinggi * 0.5f
            )
            cubicTo(
                lebar * 0.70f, tinggi * 0.82f,
                lebar * 0.30f, tinggi * 0.82f,
                lebar * 0.08f, tinggi * 0.5f
            )
            close()
        }

        drawPath(bentukMata, color = warna, style = Stroke(width = tebal))
        drawCircle(
            color = warna,
            radius = lebar * 0.13f,
            center = Offset(lebar * 0.5f, tinggi * 0.5f),
            style = Stroke(width = tebal)
        )

        if (!terbuka) {
            drawLine(
                color = warna,
                start = Offset(lebar * 0.17f, tinggi * 0.83f),
                end = Offset(lebar * 0.83f, tinggi * 0.17f),
                strokeWidth = tebal,
                cap = StrokeCap.Round
            )
        }
    }
}
