package di

import android.app.Application
import org.koin.android.ext.koin.androidContext
import org.koin.core.context.GlobalContext.startKoin
import uniffi.vociferate_core.initDatabase

class KoinApplication: Application() {
    override fun onCreate() {
        super.onCreate()
        System.loadLibrary("vociferate_core")
        val dbPath = "${filesDir.absolutePath}/device_settings.db"
        initDatabase(dbPath)
        startKoin {
            androidContext(this@KoinApplication.applicationContext)
            modules(appModuleAndroid)
        }
    }
}
