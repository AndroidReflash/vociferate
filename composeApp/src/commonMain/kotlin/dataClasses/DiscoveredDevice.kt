package dataClasses

data class DiscoveredDevice(
    val id: String,
    val name: String,
    val address: String,
    val port: Int,
    val isSelected: Boolean = false,
)