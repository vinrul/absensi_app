package id.polri.poldabali.kinerja

import android.app.Application
import id.polri.poldabali.kinerja.data.Graf
import id.polri.poldabali.kinerja.sync.Pengingat
import id.polri.poldabali.kinerja.sync.Sinkronisasi

class App : Application() {
    override fun onCreate() {
        super.onCreate()
        Graf.init(this)
        Pengingat.buatChannel(this)
        Sinkronisasi.jadwalkan(this)
    }
}
