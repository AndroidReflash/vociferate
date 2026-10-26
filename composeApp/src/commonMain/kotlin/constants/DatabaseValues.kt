package constants

sealed class DatabaseValues(val value: String) {
    data object Theme: DatabaseValues(value = "theme")
    data object InputDevice: DatabaseValues(value = "input_device")
    data object OutputDevice: DatabaseValues(value = "output_device")
    data object OutputDeviceListen: DatabaseValues(value = "output_device_listen")
    data object InputDeviceListen: DatabaseValues(value = "input_device_listen")
}