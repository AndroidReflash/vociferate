package platform

import android.content.Context
import android.media.AudioDeviceCallback
import android.media.AudioDeviceInfo
import android.media.AudioManager
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

class AudioDeviceMonitor(context: Context): DeviceMonitor {
    private val priorityMap = listOf(
        AudioDeviceInfo.TYPE_BLUETOOTH_A2DP,
        AudioDeviceInfo.TYPE_BLUETOOTH_SCO,
        AudioDeviceInfo.TYPE_USB_HEADSET,
        AudioDeviceInfo.TYPE_USB_DEVICE,
        AudioDeviceInfo.TYPE_WIRED_HEADSET,
        AudioDeviceInfo.TYPE_WIRED_HEADPHONES,
        AudioDeviceInfo.TYPE_BUILTIN_SPEAKER
    ).withIndex().associate { it.value to it.index }
    private val _inputDeviceChanged = MutableStateFlow("")
    override val inputDeviceChanged: StateFlow<String> = _inputDeviceChanged

    private val _outputDeviceChanged = MutableStateFlow("")
    override val outputDeviceChanged: StateFlow<String> = _outputDeviceChanged

    private val audioManager = context.getSystemService(Context.AUDIO_SERVICE) as AudioManager

    private val deviceCallback = object : AudioDeviceCallback() {
        override fun onAudioDevicesAdded(addedDevices: Array<out AudioDeviceInfo>) {
            evaluateActiveDevices()
        }

        override fun onAudioDevicesRemoved(removedDevices: Array<out AudioDeviceInfo>) {
            evaluateActiveDevices()
        }
    }

    override fun start() {
        evaluateActiveDevices()
        audioManager.registerAudioDeviceCallback(deviceCallback, null)
    }

    override fun stop() {
        audioManager.unregisterAudioDeviceCallback(deviceCallback)
    }

    private fun evaluateActiveDevices() {
        val outputs = audioManager.getDevices(AudioManager.GET_DEVICES_OUTPUTS)
        val inputs = audioManager.getDevices(AudioManager.GET_DEVICES_INPUTS)

        val activeOutput = selectDefaultOutput(outputs)
        val activeInput = selectDefaultInput(inputs)

        _outputDeviceChanged.value = activeOutput?.productName.toString()
        _inputDeviceChanged.value = activeInput?.productName.toString()
    }

    private fun selectDefaultOutput(devices: Array<AudioDeviceInfo>): AudioDeviceInfo? {
        return devices.minByOrNull { priorityMap[it.type] ?: Int.MAX_VALUE }
    }

    private fun selectDefaultInput(devices: Array<AudioDeviceInfo>): AudioDeviceInfo? {
        val priority = listOf(
            AudioDeviceInfo.TYPE_BLUETOOTH_SCO,
            AudioDeviceInfo.TYPE_BLE_HEADSET,
            AudioDeviceInfo.TYPE_USB_HEADSET,
            AudioDeviceInfo.TYPE_USB_DEVICE,
            AudioDeviceInfo.TYPE_WIRED_HEADSET,
            AudioDeviceInfo.TYPE_BUILTIN_MIC
        )
        for (type in priority) {
            devices.find { it.type == type }?.let { return it }
        }
        return devices.firstOrNull()
    }
}