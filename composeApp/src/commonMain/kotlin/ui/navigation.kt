package ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import org.koin.compose.koinInject
import org.koin.compose.viewmodel.koinViewModel
import platform.BottomLayout
import ui.mainScreen.MainScreen
import ui.settings.SettingsScreen
import ui.startScreen.StartScreen
import viewModels.MainScreenModel
import viewModels.SettingsViewModel
import viewModels.StartScreenModel

@Composable
fun Navigation() {
    val navController = rememberNavController()
    val startScreenModel: StartScreenModel = koinViewModel()
    val settingsScreenModel: SettingsViewModel = koinViewModel()
    val bottomLayout : BottomLayout = koinInject()
    val mainScreenModel: MainScreenModel = koinViewModel()
    val backgroundColor by mainScreenModel.backgroundColor.collectAsState()
    val deviceName by startScreenModel.deviceName.collectAsState()
    val topPadding = WindowInsets.statusBars.asPaddingValues().calculateTopPadding()
    val bottomPadding = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()

    Box(Modifier.fillMaxSize().background(backgroundColor)){
        Box(Modifier.fillMaxSize().padding(top = topPadding, bottom = bottomPadding)){
            NavHost(
                navController = navController,
                startDestination = if(deviceName == null) Routes.StartScreen.route else Routes.MainScreen.route
            ) {
                composable(Routes.MainScreen.route) {
                    MainScreen(navController, mainScreenModel, bottomLayout)
                }
                composable(Routes.StartScreen.route) {
                    StartScreen(
                        navController = navController,
                        viewModel = startScreenModel
                    )
                }
                composable(Routes.SettingsScreen.route) {
                    SettingsScreen(navController, settingsScreenModel)
                }
            }
        }
    }
}
