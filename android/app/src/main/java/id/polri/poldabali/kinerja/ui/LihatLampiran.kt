package id.polri.poldabali.kinerja.ui

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.pdf.PdfRenderer
import android.os.ParcelFileDescriptor
import androidx.compose.foundation.Image
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import id.polri.poldabali.kinerja.data.Lampiran
import java.io.File

private data class HasilPdf(val halaman: List<Bitmap>, val total: Int)

/** Penampil lampiran dalam aplikasi: gambar dengan pinch-zoom, PDF dirender per halaman. */
@Composable
fun DialogLihatLampiran(
    vm: KinerjaViewModel,
    lampiran: Lampiran,
    onTutup: () -> Unit
) {
    var isi by remember(lampiran.id) { mutableStateOf<ByteArray?>(null) }
    var galat by remember(lampiran.id) { mutableStateOf<String?>(null) }

    LaunchedEffect(lampiran.id) {
        vm.unduhLampiran(lampiran, { isi = it }, { galat = it })
    }

    Dialog(
        onDismissRequest = onTutup,
        properties = DialogProperties(usePlatformDefaultWidth = false)
    ) {
        Surface(Modifier.fillMaxSize(), color = Color(0xFF10141B)) {
            Column(Modifier.fillMaxSize()) {
                Row(
                    Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 12.dp, vertical = 8.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column(Modifier.weight(1f)) {
                        Text(
                            lampiran.fileName,
                            style = MaterialTheme.typography.titleSmall,
                            color = Color.White,
                            fontWeight = FontWeight.SemiBold,
                            maxLines = 1
                        )
                        Text(
                            "${lampiran.mime} · ${ukuranBerkasLampiran(lampiran.sizeBytes)}",
                            style = MaterialTheme.typography.bodySmall,
                            color = Color.White.copy(alpha = 0.65f)
                        )
                    }
                    TextButton(onClick = onTutup) {
                        Text("Tutup", color = Color.White)
                    }
                }

                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    val berkas = isi
                    when {
                        galat != null -> Text(
                            galat.orEmpty(),
                            color = Color(0xFFFF8A80),
                            textAlign = TextAlign.Center,
                            modifier = Modifier.padding(24.dp)
                        )
                        berkas == null -> CircularProgressIndicator(color = Color.White)
                        lampiran.mime.startsWith("image/") -> TampilanGambar(berkas)
                        lampiran.mime == "application/pdf" ||
                            lampiran.fileName.endsWith(".pdf", ignoreCase = true) -> TampilanPdf(berkas)
                        else -> Text(
                            "Tipe berkas ini belum dapat ditampilkan di aplikasi.",
                            color = Color.White,
                            textAlign = TextAlign.Center,
                            modifier = Modifier.padding(24.dp)
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun TampilanGambar(bytes: ByteArray) {
    val bitmap = remember(bytes) { BitmapFactory.decodeByteArray(bytes, 0, bytes.size) }
    if (bitmap == null) {
        Text("Gambar tidak dapat dibaca.", color = Color.White)
        return
    }

    var skala by remember { mutableFloatStateOf(1f) }
    var geser by remember { mutableStateOf(Offset.Zero) }

    Box(
        Modifier
            .fillMaxSize()
            .clipToBounds()
            .pointerInput(Unit) {
                detectTransformGestures { _, pan, zoom, _ ->
                    skala = (skala * zoom).coerceIn(1f, 5f)
                    geser = if (skala > 1f) geser + pan else Offset.Zero
                }
            },
        contentAlignment = Alignment.Center
    ) {
        Image(
            bitmap = bitmap.asImageBitmap(),
            contentDescription = "Lampiran gambar",
            modifier = Modifier
                .fillMaxSize()
                .graphicsLayer {
                    scaleX = skala
                    scaleY = skala
                    translationX = geser.x
                    translationY = geser.y
                },
            contentScale = ContentScale.Fit
        )
    }
}

@Composable
private fun TampilanPdf(bytes: ByteArray) {
    val konteks = LocalContext.current
    val hasil = remember(bytes) { runCatching { renderPdf(konteks, bytes) } }
    val pdf = hasil.getOrNull()

    if (pdf == null) {
        Text(
            "PDF tidak dapat ditampilkan: ${hasil.exceptionOrNull()?.message ?: "format tidak dikenal"}",
            color = Color(0xFFFF8A80),
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(24.dp)
        )
        return
    }
    if (pdf.halaman.isEmpty()) {
        Text("PDF tidak memiliki halaman.", color = Color.White)
        return
    }

    Column(Modifier.fillMaxSize()) {
        LazyColumn(
            Modifier
                .weight(1f)
                .fillMaxWidth(),
            contentPadding = PaddingValues(8.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            items(pdf.halaman) { halaman ->
                Image(
                    bitmap = halaman.asImageBitmap(),
                    contentDescription = "Halaman PDF",
                    modifier = Modifier.fillMaxWidth(),
                    contentScale = ContentScale.FillWidth
                )
            }
        }
        if (pdf.total > pdf.halaman.size) {
            Text(
                "Menampilkan ${pdf.halaman.size} dari ${pdf.total} halaman.",
                style = MaterialTheme.typography.bodySmall,
                color = Color.White.copy(alpha = 0.65f),
                modifier = Modifier.padding(8.dp)
            )
        }
    }
}

private fun renderPdf(context: Context, bytes: ByteArray): HasilPdf {
    val file = File.createTempFile("lampiran-", ".pdf", context.cacheDir)
    try {
        file.writeBytes(bytes)
        ParcelFileDescriptor.open(file, ParcelFileDescriptor.MODE_READ_ONLY).use { descriptor ->
            val renderer = PdfRenderer(descriptor)
            try {
                val total = renderer.pageCount
                val batas = minOf(total, 10)
                val lebar = context.resources.displayMetrics.widthPixels.coerceAtLeast(720)
                val halaman = (0 until batas).map { index ->
                    val page = renderer.openPage(index)
                    try {
                        val tinggi = (lebar.toFloat() * page.height / page.width)
                            .toInt()
                            .coerceAtLeast(1)
                        val bitmap = Bitmap.createBitmap(lebar, tinggi, Bitmap.Config.ARGB_8888)
                        bitmap.eraseColor(android.graphics.Color.WHITE)
                        page.render(bitmap, null, null, PdfRenderer.Page.RENDER_MODE_FOR_DISPLAY)
                        bitmap
                    } finally {
                        page.close()
                    }
                }
                return HasilPdf(halaman, total)
            } finally {
                renderer.close()
            }
        }
    } finally {
        file.delete()
    }
}

private fun ukuranBerkasLampiran(bytes: Long): String {
    val kb = bytes / 1024
    return if (kb < 1) "<1 KB" else "$kb KB"
}
