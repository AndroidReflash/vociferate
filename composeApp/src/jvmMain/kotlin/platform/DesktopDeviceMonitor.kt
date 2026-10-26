package platform

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

class DesktopDeviceMonitor() : DeviceMonitor {
    private val _inputDeviceChanged = MutableStateFlow("")
    override val inputDeviceChanged: StateFlow<String> = _inputDeviceChanged

    private val _outputDeviceChanged = MutableStateFlow("")
    override val outputDeviceChanged: StateFlow<String> = _outputDeviceChanged
    override fun start() {

    }

    override fun stop() {
    }
}