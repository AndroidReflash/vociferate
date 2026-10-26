package di

import org.koin.core.module.dsl.singleOf
import org.koin.dsl.bind
import org.koin.dsl.module
import platform.BottomLayout
import platform.BottomLayoutImpl
import platform.DesktopDeviceMonitor
import platform.DeviceMonitor

/** The desktop half: three stubs that do not pretend, on top of the common module. */
val platformModule = module {
    includes(appModule)

    singleOf(::BottomLayoutImpl) bind BottomLayout::class
    singleOf(::DesktopDeviceMonitor) bind DeviceMonitor::class
}
