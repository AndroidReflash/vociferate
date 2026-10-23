package ui.mainScreen.extraElements

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import dataClasses.DiscoveredDevice

@Composable
fun DeviceRow(
    color: Color,
    device: DiscoveredDevice,
    onConnect: (DiscoveredDevice) -> Unit,
    onDisconnect: (DiscoveredDevice) -> Unit,
    elementColor: Color,
    elementColorActive: Color
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .border(3.dp, if(device.isSelected) elementColorActive else elementColor)
            .fillMaxWidth()
            .clickable {
                if(!device.isSelected){
                    onConnect(device)
                }else{
                    onDisconnect(device)
                }
            },
    ){
        Column(
            modifier = Modifier.padding(12.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Text(
                text = device.name,
                style = MaterialTheme.typography.titleMedium,
                color = color
            )
            Text(
                text = "${device.address}:${device.port}",
                style = MaterialTheme.typography.bodyMedium,
                color = color
            )
        }
    }
}
