package id.polri.poldabali.kinerja.data

import android.content.Context

/** Penyedia dependensi sederhana (tanpa pustaka DI tambahan). */
object Graf {
    lateinit var appContext: Context
        private set

    lateinit var sesi: PenyimpananSesi
        private set

    lateinit var antrean: AntreanAbsensi
        private set

    lateinit var api: Api
        private set

    lateinit var repositori: Repositori
        private set

    fun init(context: Context) {
        if (::repositori.isInitialized) return
        appContext = context.applicationContext
        sesi = PenyimpananSesi(appContext)
        antrean = AntreanAbsensi(appContext)
        api = Api(sesi)
        repositori = Repositori(sesi, antrean, api)
    }
}
