package platform

import kotlinx.coroutines.flow.StateFlow

interface DeviceMonitor {
    val inputDeviceChanged: StateFlow<String>
    val outputDeviceChanged: StateFlow<String>
    fun start()
    fun stop()
}