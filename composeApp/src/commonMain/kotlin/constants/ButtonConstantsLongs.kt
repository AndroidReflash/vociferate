package constants

sealed class ButtonConstantsLongs(val value: Long) {
    data object LightFace : ButtonConstantsLongs(0xFFFDFCFA)
    data object DarkFace : ButtonConstantsLongs(0xFF20242B)
    data object LightInk : ButtonConstantsLongs(0xFF3A3F4A)
    data object DarkInk : ButtonConstantsLongs(0xFFF2C14E)
}
