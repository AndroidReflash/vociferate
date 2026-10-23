package dataClasses

import uniffi.vociferate_core.AudioDevice


data class SoundDevices(
    val input: AudioDevice?,
    val output: AudioDevice?
)
