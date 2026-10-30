package viewModels

import androidx.lifecycle.ViewModel
import repositories.interfaces.SettingsRepo
import uniffi.vociferate_core.AudioDevice

class SettingsViewModel(
    private val repo: SettingsRepo
): ViewModel() {
    val outputDevice = repo.outputDevice
    val inputDevice = repo.inputDevice
    val textColor = repo.textColor
    val backgroundColor = repo.backgroundColor
    val inputDevices = repo.inputDevices
    val outputDevices = repo.outputDevices

    fun setInputDevice(device: AudioDevice){
        repo.setInputDevice(device)
    }

    fun setOutputDevice(device: AudioDevice){
        repo.setOutputDevice(device)
    }

    fun setOutputDefault(){
        repo.setOutputDefault()
    }

    fun setInputDefault(){
        repo.setInputDefault()
    }

    val elementColor = repo.elementColor
}