package repositories.interfaces

import dataClasses.SoundState
import kotlinx.coroutines.flow.StateFlow

interface SoundRepo {
    val state: StateFlow<SoundState>
    fun setListening()
    fun setAudio()
}