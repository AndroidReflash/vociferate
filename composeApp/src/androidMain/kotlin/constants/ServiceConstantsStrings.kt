package constants

sealed class ServiceConstantsStrings(val value: String) {
    data object ChannelId : ServiceConstantsStrings("vociferate-foreground")
    data object ChannelName : ServiceConstantsStrings("vociferate")

    data object NotificationTitle : ServiceConstantsStrings("Передача голоса")
    data object NotificationText : ServiceConstantsStrings("слушаем входящие")
}
