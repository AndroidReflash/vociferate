package viewModels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dataClasses.DiscoverState
import dataClasses.DiscoveredDevice
import dataClasses.SoundState
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import repositories.interfaces.DiscoverRepo
import repositories.interfaces.SettingsRepo
import repositories.interfaces.SoundRepo
import repositories.interfaces.StartScreenRepo

class MainScreenModel(
    private val discoverRepo: DiscoverRepo,
    private val settingsRepo: SettingsRepo,
    private val soundRepo: SoundRepo,
    private val startScreenRepo: StartScreenRepo
): ViewModel() {
    val deviceName = startScreenRepo.deviceName
    fun setTheme(){
        settingsRepo.setTheme()
    }
    val elementColor = settingsRepo.elementColor
    val elementColorActive = settingsRepo.elementColorActive
    val backgroundColor = settingsRepo.backgroundColor
    val textColor = settingsRepo.textColor
    val discoverState: StateFlow<DiscoverState> = discoverRepo.state
    val soundState: StateFlow<SoundState> = soundRepo.state
    val inputDevice: StateFlow<String?> = settingsRepo.inputDevice
    val outputDevice: StateFlow<String?> = settingsRepo.outputDevice


    init {
        viewModelScope.launch {
            viewModelScope.launch {
                deviceName.first { it != null }
                discoverRepo.startDiscovery()
                discoverRepo.startServerKotlin()
            }
        }
    }
    fun connectToPeer(device: DiscoveredDevice) = viewModelScope.launch (Dispatchers.IO) {
        discoverRepo.connectToPeer(device)
    }
    fun disconnectPeer(device: DiscoveredDevice) = viewModelScope.launch (Dispatchers.IO) {
        discoverRepo.disconnectPeer(device)
    }
    fun setListening(){
        soundRepo.setListening()
    }
    fun setAudio(){
        soundRepo.setAudio()
    }
}