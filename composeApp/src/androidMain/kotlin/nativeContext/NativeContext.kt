package nativeContext

import android.content.Context

object NativeContext {

    private var installed = false

    fun install(context: Context): Boolean {
        if (!installed) {
            System.loadLibrary("vociferate_core")
            installed = initialize(context.applicationContext)
        }
        return installed
    }

    private external fun initialize(context: Context): Boolean
}