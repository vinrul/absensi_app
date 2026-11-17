package id.polri.poldabali.kinerja.data

/** Berkas pilihan pengguna yang akan diunggah sebagai lampiran. */
data class BerkasLampiran(
    val nama: String,
    val mime: String,
    val isi: ByteArray
)
