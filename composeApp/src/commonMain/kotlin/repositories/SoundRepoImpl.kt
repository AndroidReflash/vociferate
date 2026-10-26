package repositories

import constants.DatabaseValues
import dataClasses.SoundState
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import platform.DeviceMonitor
import repositories.interfaces.SettingsRepo
import repositories.interfaces.SoundRepo
import uniffi.vociferate_core.AudioConfig
import uniffi.vociferate_core.getBoolSetting
import uniffi.vociferate_core.startListeningMicro
import uniffi.vociferate_core.startSpeakers
import uniffi.vociferate_core.stopListeningMicro
import uniffi.vociferate_core.stopSpeakers
import java.lang.Exception

class SoundRepoImpl(
    private val settings: SettingsRepo,
    private val deviceMonitor: DeviceMonitor,
    private val scope: CoroutineScope
) : SoundRepo {
    override fun setDefaultInOut(){
        val speakers = getBoolSetting(DatabaseValues.OutputDeviceListen.value) ?: false
        val listening = getBoolSetting(DatabaseValues.InputDeviceListen.value) ?: false
        if (listening){
            val inputDevice = settings.inputDevice.value
            try {
                startListeningMicro(inputDevice,  settings.defaultMircoConfig)
            }catch (e: Exception){
                println("$inputDevice плохое устройство")
            }
        }
        if (speakers){
            val outputDevice = settings.outputDevice.value
            try {
                startSpeakers(outputDevice, settings.defaultSpeakersConfig)
            }catch (e: Exception){
                println("$outputDevice плохое устройство")
            }

        }
        _state.value = _state.value.copy(listening = listening, speakers = speakers)
    }

    private val _state = MutableStateFlow(
        SoundState(
            listening = false,
            speakers = false
        )
    )
    override val state: StateFlow<SoundState> = _state

    override fun setListening() {
        scope.launch {
            withContext(Dispatchers.IO){
                try {
                    if(state.value.listening){
                        stopListeningMicro()
                        _state.value = _state.value.copy(listening =  false)
                        settings.setMicroListening(false)
                    }else{
                        val inputDevice = settings.inputDevice.value
                        startListeningMicro(inputDevice, settings.defaultMircoConfig)
                        _state.value = _state.value.copy(listening =  true)
                        settings.setMicroListening(true)
                    }
                }catch (e: Exception){
                    println("${e.message}")
                }
            }
        }
    }

    override fun setAudio(){
        scope.launch {
            withContext(Dispatchers.IO){
                try {
                    if(state.value.speakers){
                        stopSpeakers()
                        _state.value = _state.value.copy(speakers =  false)
                        settings.setAudioListening(false)
                    }else{
                        val outputDevice = settings.outputDevice.value
                        startSpeakers(outputDevice, settings.defaultSpeakersConfig)
                        _state.value = _state.value.copy(speakers =  true)
                        settings.setAudioListening(true)
                    }
                }catch (e: Exception){
                    println("${e.message}")
                }
            }
        }
    }
    init {
        scope.launch {
            // для мобилок т.к. там выбор устройств автоматический
            launch {
                withContext(Dispatchers.IO){
                    deviceMonitor.start()
                }
            }
            // для мобилок т.к. там выбор устройств автоматический
            launch {
                withContext(Dispatchers.IO){
                    deviceMonitor.inputDeviceChanged.collect {
                        println("ввод: $it")
                        if(state.value.listening) {
                            delay(1000)
                            startListeningMicro(
                                settings.inputDevice.value,
                                settings.defaultMircoConfig
                            )
                        }
                    }
                }
            }
            // для мобилок т.к. там выбор устройств автоматический
            launch {
                withContext(Dispatchers.IO){
                    deviceMonitor.outputDeviceChanged.collect{
                        println("вывод: $it")
                        if(state.value.speakers) {
                            delay(1000)
                            startSpeakers(
                                settings.outputDevice.value,
                                settings.defaultSpeakersConfig
                            )
                        }
                    }
                }
            }
        }
    }
}