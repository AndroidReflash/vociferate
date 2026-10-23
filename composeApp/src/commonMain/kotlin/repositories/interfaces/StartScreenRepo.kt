package repositories.interfaces

import dataClasses.StartScreenState
import kotlinx.coroutines.flow.StateFlow

interface StartScreenRepo {
    val state: StateFlow<StartScreenState>
    val deviceName : StateFlow<String?>
    fun loadDeviceName()
    fun saveDeviceName(name: String)
}
