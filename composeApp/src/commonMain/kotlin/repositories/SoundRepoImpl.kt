package repositories

import dataClasses.SoundState
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import repositories.interfaces.SettingsRepo
import repositories.interfaces.SoundRepo
import uniffi.vociferate_core.AudioConfig
import uniffi.vociferate_core.startListeningMicro
import uniffi.vociferate_core.startSpeakers
import uniffi.vociferate_core.stopListeningMicro
import uniffi.vociferate_core.stopSpeakers
import java.lang.Exception

class SoundRepoImpl(
    private val settings: SettingsRepo,
    private val scope: CoroutineScope
) : SoundRepo {
    private val defaultMircoConfig = AudioConfig(
        sampleRate = 48000.toUInt(),
        channels = 1.toUShort(),
        bufferSize = 1024.toUInt()
    )
    private val defaultSpeakersConfig = AudioConfig(
        sampleRate = 48000.toUInt(),
        channels = 1.toUShort(),
        bufferSize = 1024.toUInt()
    )

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
                    }else{
                        val inputDevice = settings.inputDevice.value
                        startListeningMicro(inputDevice, defaultMircoConfig)
                        _state.value = _state.value.copy(listening =  true)
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
                    }else{
                        val outputDevice = settings.outputDevice.value
                        startSpeakers(outputDevice, defaultSpeakersConfig)
                        _state.value = _state.value.copy(speakers =  true)
                    }
                }catch (e: Exception){
                    println("${e.message}")
                }
            }
        }
    }
}