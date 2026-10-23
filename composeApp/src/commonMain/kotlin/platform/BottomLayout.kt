package platform

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

interface BottomLayout {
    @Composable
    fun Content(
        elementColor: Color,
        onNavigate: () -> Unit
    )
}