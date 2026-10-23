package ui.startScreen

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import dataClasses.StartScreenState
import org.koin.compose.viewmodel.koinViewModel
import ui.Routes
import viewModels.StartScreenModel

@Composable
fun StartScreen(
    navController: NavController,
    viewModel: StartScreenModel
) {
    val elementColor by viewModel.elementColor.collectAsState()
    val elementColorActive by viewModel.elementColorActive.collectAsState()
    val backgroundColor by viewModel.backgroundColor.collectAsState()
    val textColor by viewModel.textColor.collectAsState()
    StartScreenContent(
        backgroundColor = backgroundColor,
        textColor = textColor,
        elementColor = elementColor,
        elementColorActive = elementColorActive,
        onSaveClick = {
            name -> viewModel.saveDeviceName(name)
            navController.navigate(Routes.MainScreen.route) {
                popUpTo(Routes.StartScreen.route) { inclusive = true }
            }
        }
    )
}

@Composable
private fun StartScreenContent(
    backgroundColor: Color,
    textColor: Color,
    elementColor: Color,
    elementColorActive: Color,
    onSaveClick: (String) -> Unit,
) {
    var name by remember { mutableStateOf("") }
    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(backgroundColor)
            .padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(
            text = "Vociferate",
            fontSize = 28.sp,
            style = MaterialTheme.typography.headlineMedium,
            color = textColor,
        )
        Text(
            text = "Введи имя устройства",
            style = MaterialTheme.typography.bodyLarge,
            color = textColor,
        )
        OutlinedTextField(
            value = name,
            onValueChange = { name = it },
            label = { Text("Имя устройства") },
            placeholder = { Text("Например: Мой телефон") },
            modifier = Modifier.fillMaxWidth(),
            singleLine = true,
            colors = OutlinedTextFieldDefaults.colors(
                // Цвет текста, который вводит пользователь
                focusedTextColor = textColor,
                unfocusedTextColor = textColor.copy(alpha = 0.5f),

                // Цвет рамки (outline)
                focusedBorderColor = elementColorActive,
                unfocusedBorderColor = elementColor,

                // Цвет лейбла (метки)
                focusedLabelColor = elementColorActive,
                unfocusedLabelColor = elementColor,

                // Цвет плейсхолдера
                focusedPlaceholderColor =  textColor.copy(alpha = 0.5f),
                unfocusedPlaceholderColor = textColor.copy(alpha = 0.5f),
            )
        )
        Box(Modifier.fillMaxWidth().border(3.dp, elementColor).clickable{
            if(name != ""){
                onSaveClick(name)
            }
        }.wrapContentHeight(), contentAlignment = Alignment.Center){
            Text(
                text = "Сохранить и продолжить",
                fontSize = 16.sp,
                color = textColor,
                style = MaterialTheme.typography.labelLarge,
                modifier = Modifier.padding(15.dp)
            )
        }
    }
}
