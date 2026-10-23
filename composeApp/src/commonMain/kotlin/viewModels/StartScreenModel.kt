package viewModels

import androidx.lifecycle.ViewModel
import dataClasses.StartScreenState
import kotlinx.coroutines.flow.StateFlow
import repositories.interfaces.SettingsRepo
import repositories.interfaces.StartScreenRepo

class StartScreenModel(
    private val startScreenRepo: StartScreenRepo,
    private val settingsRepo: SettingsRepo,
): ViewModel() {
    val elementColor = settingsRepo.elementColor
    val elementColorActive = settingsRepo.elementColorActive
    val backgroundColor = settingsRepo.backgroundColor
    val textColor = settingsRepo.textColor

    val deviceName = startScreenRepo.deviceName

    init {
        startScreenRepo.loadDeviceName()
    }

    fun saveDeviceName(name: String) = startScreenRepo.saveDeviceName(name)
}
