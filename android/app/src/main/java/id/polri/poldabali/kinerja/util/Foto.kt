package id.polri.poldabali.kinerja.util

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.util.Base64
import java.io.ByteArrayOutputStream
import java.io.File
import kotlin.math.max

object Foto {

    /** Mengecilkan foto kamera lalu mengubahnya menjadi base64 JPEG (target < 500 KB). */
    fun keBase64Jpeg(berkas: File, sisiMaks: Int = 1280, kualitas: Int = 82): String? {
        if (!berkas.exists()) return null

        val kasar = BitmapFactory.decodeFile(berkas.absolutePath) ?: return null
        val skala = max(1, max(kasar.width, kasar.height) / sisiMaks)
        val kecil = if (skala > 1) {
            Bitmap.createScaledBitmap(kasar, kasar.width / skala, kasar.height / skala, true)
        } else {
            kasar
        }

        val keluaran = ByteArrayOutputStream()
        kecil.compress(Bitmap.CompressFormat.JPEG, kualitas, keluaran)
        if (kecil !== kasar) kecil.recycle()
        kasar.recycle()

        return Base64.encodeToString(keluaran.toByteArray(), Base64.NO_WRAP)
    }

    fun berkasSementara(direktori: File, nama: String = "absen-${System.currentTimeMillis()}.jpg"): File {
        if (!direktori.exists()) direktori.mkdirs()
        return File(direktori, nama)
    }
}
