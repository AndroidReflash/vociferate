package repositories.interfaces

import androidx.compose.ui.graphics.Color
import kotlinx.coroutines.flow.StateFlow
import uniffi.vociferate_core.AudioConfig
import uniffi.vociferate_core.AudioDevice

interface SettingsRepo {
    val inputDevices: StateFlow<List<AudioDevice>>
    val outputDevices : StateFlow<List<AudioDevice>>

    val elementColor : StateFlow<Color>
    val elementColorActive : StateFlow<Color>
    val backgroundColor : StateFlow<Color>
    val textColor : StateFlow<Color>
    val inputDevice : StateFlow<String?>
    val outputDevice : StateFlow<String?>

    val defaultSpeakersConfig: AudioConfig
    val defaultMircoConfig: AudioConfig
    fun setAudioListening(boolean: Boolean)
    fun setMicroListening(boolean: Boolean)
    fun setInputDevice(device: AudioDevice)
    fun setOutputDevice(device: AudioDevice)
    fun setTheme()
}