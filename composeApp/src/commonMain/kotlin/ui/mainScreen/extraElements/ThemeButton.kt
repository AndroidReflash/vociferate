package ui.mainScreen.extraElements

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathOperation
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.unit.dp
import constants.ButtonConstantsLongs
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin

@Composable
fun ThemeButton(dark: Boolean, onClick: () -> Unit, modifier: Modifier = Modifier) {
    val ink = Color(
        if (dark) ButtonConstantsLongs.DarkInk.value else ButtonConstantsLongs.LightInk.value
    )

    DefButton(
        onClick = onClick,
        modifier = modifier,
    ) {
        drawMoon(ink)
    }
}

@Composable
private fun DefButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    icon: DrawScope.() -> Unit,
) {
    Surface(
        onClick = onClick,
        shape = CircleShape,
        color = Color.Transparent,
        modifier = modifier.size(44.dp),
    ) {
        Canvas(Modifier.fillMaxSize().padding(11.dp)) { icon() }
    }
}

private fun DrawScope.drawMoon(color: Color) {
    val r = size.minDimension / 2f
    val disc = Path().apply { addOval(Rect(center, r)) }
    // The crescent is the disc minus a second one shifted up and to the right.
    val bite = Path().apply { addOval(Rect(center + Offset(r * 0.55f, -r * 0.35f), r * 0.95f)) }
    drawPath(Path.combine(PathOperation.Difference, disc, bite), color)
}