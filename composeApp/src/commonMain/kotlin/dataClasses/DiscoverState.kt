package dataClasses

data class  DiscoverState(
    val devicesList: Map<String, DiscoveredDevice> = emptyMap(),
    val isDiscovering: Boolean = false,
    val errorMessage: String? = null,
){
    val devices: List<DiscoveredDevice> get() = devicesList.values.toList()
}