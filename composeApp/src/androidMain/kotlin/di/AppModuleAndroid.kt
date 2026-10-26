package di

import android.annotation.SuppressLint
import org.koin.core.module.dsl.singleOf
import org.koin.dsl.bind
import org.koin.dsl.module
import platform.AudioDeviceMonitor
import platform.BottomLayout
import platform.BottomLayoutImpl
import platform.DeviceMonitor

@SuppressLint("UnsafeOptInUsageError")
val appModuleAndroid = module {

    includes(appModule)
    singleOf(::BottomLayoutImpl) bind BottomLayout::class
    singleOf(::AudioDeviceMonitor) bind DeviceMonitor::class
}
