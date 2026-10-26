package repositories.interfaces

import dataClasses.SoundState
import kotlinx.coroutines.flow.StateFlow
import uniffi.vociferate_core.AudioConfig

interface SoundRepo {
    val state: StateFlow<SoundState>
    fun setListening()
    fun setAudio()
    fun setDefaultInOut()
}