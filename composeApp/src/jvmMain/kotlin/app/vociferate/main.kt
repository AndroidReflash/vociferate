package app.vociferate

import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import di.platformModule
import org.koin.core.context.startKoin
import uniffi.vociferate_core.initDatabase
import uniffi.vociferate_core.initRustLogging
import java.io.File

fun main() = application {
    initRustLogging()
    startKoin { modules(platformModule) }

    val dbDir = File("database")
    if (!dbDir.exists()) {
        dbDir.mkdirs()
    }
    val dbFile = File(dbDir, "vociferate.db")
    initDatabase(dbFile.absolutePath)
    println("Database initialized at: ${dbFile.absolutePath}")

    Window(
        onCloseRequest = ::exitApplication,
        title = "Vociferate",
    ) {
        App()
    }
}
