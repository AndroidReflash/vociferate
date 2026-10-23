package ui.mainScreen.extraElements

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import dataClasses.DiscoveredDevice

@Composable
fun DeviceList(
    color: Color,
    devices: List<DiscoveredDevice>,
    onConnect: (DiscoveredDevice) -> Unit,
    onDisconnect: (DiscoveredDevice) -> Unit,
    elementColor: Color,
    elementColorActive: Color,
    modifier: Modifier = Modifier,
) {
    if (devices.isEmpty()) {
        Text(
            text = "Ничего не найдено",
            modifier = modifier,
            color = color
        )
        return
    }
    LazyColumn(
        modifier = modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        items(items = devices) { device ->
            DeviceRow(
                color = color,
                device = device,
                onConnect = { onConnect(it) },
                onDisconnect = { onDisconnect(it) },
                elementColor = elementColor,
                elementColorActive = elementColorActive
            )
        }
    }
}