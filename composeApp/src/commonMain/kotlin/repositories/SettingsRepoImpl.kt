package repositories

import androidx.compose.ui.graphics.Color
import constants.DatabaseValues
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import repositories.interfaces.SettingsRepo
import uniffi.vociferate_core.AudioConfig
import uniffi.vociferate_core.AudioDevice
import uniffi.vociferate_core.getBoolSetting
import uniffi.vociferate_core.getStringSetting
import uniffi.vociferate_core.listAudioInputDevices
import uniffi.vociferate_core.listAudioOutputDevices
import uniffi.vociferate_core.setBoolSetting
import uniffi.vociferate_core.setStringSetting
import uniffi.vociferate_core.startListeningMicro
import uniffi.vociferate_core.startSpeakers

class SettingsRepoImpl(
    private val scope: CoroutineScope,
): SettingsRepo {
    override val defaultMircoConfig = AudioConfig(
        sampleRate = 48000.toUInt(),
        channels = 1.toUShort(),
        bufferSize = 1024.toUInt()
    )
    override val defaultSpeakersConfig = AudioConfig(
        sampleRate = 48000.toUInt(),
        channels = 1.toUShort(),
        bufferSize = 1024.toUInt()
    )
    private val _theme = MutableStateFlow(true)
    private val theme : StateFlow<Boolean> = _theme
    private val _elementColor = MutableStateFlow(Color.Black)
    override val elementColor : StateFlow<Color> = _elementColor
    private val _elementColorActive = MutableStateFlow(Color.Red)
    override val elementColorActive : StateFlow<Color> = _elementColorActive
    private val _backgroundColor = MutableStateFlow(Color.White)
    override val backgroundColor : StateFlow<Color> = _backgroundColor
    private val _textColor = MutableStateFlow(Color.Black)
    override val textColor : StateFlow<Color> = _textColor

    private val _inputDevice = MutableStateFlow<String?>(null)
    override val inputDevice : StateFlow<String?> = _inputDevice

    private val _outputDevice = MutableStateFlow<String?>(null)
    override val outputDevice : StateFlow<String?> = _outputDevice

    override fun setTheme(){
        _theme.value = !_theme.value
        setBoolSetting(DatabaseValues.Theme.value, _theme.value)
    }

    override fun setInputDevice(device: AudioDevice){
        _inputDevice.value = device.id
        setStringSetting(DatabaseValues.InputDevice.value, device.id)
        try {
            startListeningMicro(device.id, defaultMircoConfig)
        }catch (e: Exception){
            println("плохое устройство")
        }
    }

    override fun setOutputDevice(device: AudioDevice){
        _outputDevice.value = device.id
        setStringSetting(DatabaseValues.OutputDevice.value, device.id)
        try {
            startSpeakers(device.id, defaultSpeakersConfig)
        }catch (e: Exception){
            println("плохое устройство")
        }
    }
    private val _inputDevices = MutableStateFlow(emptyList<AudioDevice>())
    override val inputDevices: StateFlow<List<AudioDevice>> = _inputDevices

    private val _outputDevices = MutableStateFlow(emptyList<AudioDevice>())
    override val outputDevices : StateFlow<List<AudioDevice>> = _outputDevices

    private fun getDevices(){
        _inputDevices.value = listAudioInputDevices()
        _outputDevices.value = listAudioOutputDevices()
    }

    override fun setAudioListening(boolean: Boolean){
        setBoolSetting(DatabaseValues.OutputDeviceListen.value, boolean)
    }
    override fun setMicroListening(boolean: Boolean){
        setBoolSetting(DatabaseValues.InputDeviceListen.value, boolean)
    }

    init {
        scope.launch {
            launch {
                withContext(Dispatchers.IO){
                    _theme.value = getBoolSetting(DatabaseValues.Theme.value) ?: true
                    _inputDevice.value = getStringSetting(DatabaseValues.InputDevice.value)
                    _outputDevice.value = getStringSetting(DatabaseValues.OutputDevice.value)
                }
            }
            launch {
                withContext(Dispatchers.IO) {
                    getDevices()
                }
            }
            launch {
                withContext(Dispatchers.IO){
                    theme.collect {
                        if(!it){
                            _textColor.value = Color.White
                            _elementColorActive.value = Color.Cyan
                            _elementColor.value = Color.White
                            _backgroundColor.value = Color.Black
                        }else{
                            _textColor.value = Color.Black
                            _elementColorActive.value = Color.Red
                            _elementColor.value = Color.Black
                            _backgroundColor.value = Color.White
                        }
                    }
                }
            }
        }
    }
}