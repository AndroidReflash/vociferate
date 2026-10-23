package constants

sealed class DiscoveryConstantsInts(val value: Int) {
    data object BrowseTimeoutMs : DiscoveryConstantsInts(3000)
}