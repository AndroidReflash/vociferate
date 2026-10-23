package repositories.interfaces

import dataClasses.DiscoverState
import dataClasses.DiscoveredDevice
import kotlinx.coroutines.flow.StateFlow

interface DiscoverRepo {
    val state: StateFlow<DiscoverState>
    suspend fun startServerKotlin()
    suspend fun connectToPeer(device: DiscoveredDevice)
    suspend fun disconnectPeer(device: DiscoveredDevice)
    suspend fun startDiscovery()
}