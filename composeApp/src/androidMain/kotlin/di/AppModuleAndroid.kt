package di

import android.annotation.SuppressLint
import org.koin.core.module.dsl.singleOf
import org.koin.dsl.bind
import org.koin.dsl.module
import platform.BottomLayout
import platform.BottomLayoutImpl

@SuppressLint("UnsafeOptInUsageError")
val appModuleAndroid = module {

    includes(appModule)
    singleOf(::BottomLayoutImpl) bind BottomLayout::class
}
