package ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import viewModels.SettingsViewModel
import ui.Routes
import ui.settings.extraElements.DeviceSection

@Composable
fun SettingsScreen(
    navController: NavController,
    viewModel: SettingsViewModel
) {
    val backgroundColor by viewModel.backgroundColor.collectAsState()
    val textColor by viewModel.textColor.collectAsState()
    val elementColor by viewModel.elementColor.collectAsState()
    val inputDevice by viewModel.inputDevice.collectAsState()
    val outputDevice by viewModel.outputDevice.collectAsState()
    val inputDevices by viewModel.inputDevices.collectAsState()
    val outputDevices by viewModel.outputDevices.collectAsState()

    val onBackClicked = { navController.navigate(Routes.MainScreen.route) }
    val scroll = rememberScrollState()
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp)
            .background(backgroundColor)
            .verticalScroll(scroll),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(15.dp),
            modifier = Modifier.fillMaxWidth().padding(5.dp)
        ) {
            Icon(
                imageVector = Icons.Filled.ArrowBack,
                contentDescription = "Назад",
                tint = elementColor,
                modifier = Modifier.clickable{onBackClicked()}
            )
            Column {
                Text("Ввод: ${inputDevice ?: "default"}", color = textColor)
                Text("Вывод: ${outputDevice ?: "default"}", color = textColor)
            }
        }

        Spacer(modifier = Modifier.height(8.dp))

        DeviceSection(
            textColor = textColor,
            color = elementColor,
            selected = inputDevice,
            title = "Устройства ввода",
            devices = inputDevices,
            onDeviceClick = {
                viewModel.setInputDevice(it)
            }
        )

        Divider(modifier = Modifier.height(8.dp).padding(16.dp))

        DeviceSection(
            textColor = textColor,
            color = elementColor,
            selected = outputDevice,
            title = "Устройства вывода",
            devices = outputDevices,
            onDeviceClick = {
                viewModel.setOutputDevice(it)
            }
        )
    }
}
