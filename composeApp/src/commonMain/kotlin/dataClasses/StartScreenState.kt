package dataClasses

data class StartScreenState(
    val deviceName: String = "",
    val isLoading: Boolean = false,
    val errorMessage: String? = null,
    val isNameSaved: Boolean = false,
)
