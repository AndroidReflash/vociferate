package ui

sealed class Routes(val route: String) {
    data object MainScreen: Routes("Main_Screen_Route")
    data object StartScreen: Routes("Start_Screen_Route")
    data object SettingsScreen: Routes("Settings_Route")
}