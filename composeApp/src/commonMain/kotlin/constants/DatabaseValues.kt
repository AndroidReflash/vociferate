package constants

sealed class DatabaseValues(val value: String) {
    data object Theme: DatabaseValues(value = "theme")
    data object InputDevice: DatabaseValues(value = "input_device")
    data object OutputDevice: DatabaseValues(value = "output_device")
}