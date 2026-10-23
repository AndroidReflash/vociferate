package ui.mainScreen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.MusicNote
import androidx.compose.material.icons.outlined.Mic
import androidx.compose.material.icons.outlined.MicOff
import androidx.compose.material.icons.outlined.MusicOff
import androidx.compose.material3.Divider
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import platform.BottomLayout
import ui.Routes
import ui.mainScreen.extraElements.DeviceList
import ui.mainScreen.extraElements.ThemeButton
import viewModels.MainScreenModel

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun MainScreen(
    navController: NavController,
    viewModel: MainScreenModel,
    bottomLayout: BottomLayout
) {
    val elementColor by viewModel.elementColor.collectAsState()
    val elementColorActive by viewModel.elementColorActive.collectAsState()
    val backgroundColor by viewModel.backgroundColor.collectAsState()
    val textColor by viewModel.textColor.collectAsState()
    val discoverState by viewModel.discoverState.collectAsState()
    val soundState by viewModel.soundState.collectAsState()
    val inputDevice by viewModel.inputDevice.collectAsState()
    val outputDevice by viewModel.outputDevice.collectAsState()
    val deviceName by viewModel.deviceName.collectAsState()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(backgroundColor)
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Row (
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            verticalAlignment = Alignment.CenterVertically
        ){
            ThemeButton(
                false,
                {viewModel.setTheme()}
            )
            Column(
                Modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.Center
            ) {
                Text("Ввод: ${inputDevice ?: "default"}", color = textColor)
                Text("Вывод: ${outputDevice ?: "default"}", color = textColor)
            }
        }
        discoverState.errorMessage?.let {
            Text(
                text = it,
                color = MaterialTheme.colorScheme.error,
            )
        }
        DeviceList(
            color = textColor,
            devices = discoverState.devices,
            onConnect = { viewModel.connectToPeer(it) },
            onDisconnect = { viewModel.disconnectPeer(it) },
            elementColor = elementColor,
            elementColorActive = elementColorActive,
            modifier = Modifier.weight(1f),
        )
        Divider(modifier = Modifier.height(3.dp).padding(2.dp))

        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {

            Surface(
                onClick = {
                    viewModel.setListening()
                },
                shape = CircleShape,
                color = Color.Transparent,
                modifier = Modifier.size(44.dp),
            ){
                Icon(
                    imageVector = if(soundState.listening) Icons.Outlined.Mic else Icons.Outlined.MicOff,
                    contentDescription = "Микрофон",
                    tint = if(soundState.listening) elementColorActive else elementColor,
                    modifier = Modifier.padding(11.dp)
                )
            }

            Surface(
                onClick = {
                    viewModel.setAudio()
                },
                shape = CircleShape,
                color = Color.Transparent,
                modifier = Modifier.size(44.dp),
            ){
                Icon(
                    imageVector = if(soundState.speakers) Icons.Filled.MusicNote else Icons.Outlined.MusicOff,
                    contentDescription = "Динамик",
                    tint = if(soundState.speakers) elementColorActive else elementColor,
                    modifier = Modifier.padding(11.dp)
                )
            }
            // кнопка настроек
            bottomLayout.Content(
                elementColor = elementColor,
                onNavigate = { navController.navigate(Routes.SettingsScreen.route) },
            )
        }
        // имя устройства
        Text(
            text = deviceName ?: "Нет имени",
            style = MaterialTheme.typography.titleMedium,
            color = elementColorActive,
            fontSize = 20.sp
        )
    }
}