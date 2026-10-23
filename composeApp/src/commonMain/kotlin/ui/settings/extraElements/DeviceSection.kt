package ui.settings.extraElements

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import uniffi.vociferate_core.AudioDevice

@Composable
fun DeviceSection(
    textColor: Color,
    color: Color,
    selected: String?,
    title: String,
    devices: List<AudioDevice>,
    onDeviceClick: (AudioDevice) -> Unit
) {
    val scroll = rememberScrollState()
    var expanded by remember { mutableStateOf(false) }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(40.dp)
            .border(3.dp, color)
            .padding(bottom = 4.dp)
            .clickable { expanded = !expanded },
        verticalAlignment = Alignment.CenterVertically
    ) {
        Spacer(Modifier.padding(start = 5.dp))
        Text(
            text = title,
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier.weight(1f),
            color = textColor
        )
        Icon(
            imageVector = if (expanded) Icons.Default.KeyboardArrowDown else Icons.Default.KeyboardArrowUp,
            contentDescription = "Развернуть/Свернуть",
            modifier = Modifier.size(20.dp),
            tint = color
        )
        Spacer(Modifier.padding(end = 5.dp))
    }

    if (expanded) {
        if (devices.isEmpty()) {
            Text(
                text = "Устройств не найдено",
                color = textColor,
                style = MaterialTheme.typography.bodyMedium,
                modifier = Modifier.padding(8.dp)
            )
        } else {
            Column(
                modifier = Modifier
                    .wrapContentWidth()
                    .padding(16.dp)
                    .heightIn(max = 300.dp)
                    .verticalScroll(scroll),
                horizontalAlignment = Alignment.CenterHorizontally
            ){
                devices.forEach { device ->
                    DeviceItem(
                        textColor = textColor,
                        color = color,
                        selected = selected,
                        device = device,
                        onClick = onDeviceClick
                    )
                }
            }
        }
    }
}