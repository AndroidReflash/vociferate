package di


import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import org.koin.core.module.dsl.singleOf
import org.koin.core.module.dsl.viewModelOf
import org.koin.dsl.bind
import org.koin.dsl.module
import repositories.DiscoverRepoImpl
import repositories.SoundRepoImpl
import repositories.StartScreenRepoImpl
import repositories.interfaces.DiscoverRepo
import repositories.interfaces.SettingsRepo
import repositories.SettingsRepoImpl
import repositories.interfaces.SoundRepo
import repositories.interfaces.StartScreenRepo
import viewModels.MainScreenModel
import viewModels.SettingsViewModel
import viewModels.StartScreenModel

val appModule = module {
    single { CoroutineScope(SupervisorJob() + Dispatchers.Default) }

    viewModelOf(::MainScreenModel)
    viewModelOf(::StartScreenModel)
    viewModelOf(::SettingsViewModel)

    singleOf(::DiscoverRepoImpl) bind DiscoverRepo::class
    singleOf(::StartScreenRepoImpl) bind StartScreenRepo::class
    singleOf(::SoundRepoImpl) bind SoundRepo::class
    singleOf(::SettingsRepoImpl) bind SettingsRepo::class
}
