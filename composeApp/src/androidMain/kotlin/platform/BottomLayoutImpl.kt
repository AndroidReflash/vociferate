package platform

import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

class BottomLayoutImpl: BottomLayout {
    @Composable
    override fun Content(
        elementColor: Color,
        onNavigate: () -> Unit,
    ) {
//        Surface(
//            onClick = { onNavigate() },
//            shape = CircleShape,
//            color = Color.Transparent,
//            modifier = Modifier.size(44.dp),
//        ){
//            // кнопка настроек
//            Icon(
//                imageVector = Icons.Outlined.Settings,
//                contentDescription = "Настройки",
//                tint = elementColor,
//            )
//        }
    }
}