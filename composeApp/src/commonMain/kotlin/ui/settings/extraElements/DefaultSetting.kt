package ui.settings.extraElements

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

@Composable
fun DefaultSetting(
    textColor: Color,
    color: Color,
    selected: String?,
    onClick: () -> Unit
) {
    Box(Modifier
        .wrapContentHeight()
        .fillMaxWidth()
        .clickable{ onClick() }
        .border(3.dp, if(selected==null) color else Color.Transparent),
        contentAlignment = Alignment.Center
    ){
        Text(
            modifier = Modifier.padding(10.dp),
            text = "По умолчанию",
            style = MaterialTheme.typography.bodyMedium,
            overflow = TextOverflow.Ellipsis,
            color = textColor
        )
    }
}