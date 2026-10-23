package dataClasses

data class  DiscoverState(
    val devices: List<DiscoveredDevice> = emptyList(),
    val isDiscovering: Boolean = false,
    val errorMessage: String? = null,
)