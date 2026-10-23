package app.maps.constants

sealed class ServiceConstantsInts(val value: Int) {
    data object NotificationId : ServiceConstantsInts(1)
}
