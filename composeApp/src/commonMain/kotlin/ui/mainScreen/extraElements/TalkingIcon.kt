import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp

@Composable
fun TalkingIcon(
    elementColorActive: Color,
    modifier: Modifier = Modifier
) {
    val infiniteTransition = rememberInfiniteTransition(label = "talking_animation")

    val minRadiusDp = 10.dp
    val maxRadiusDp = 20.dp

    val currentRadiusDp by infiniteTransition.animateFloat(
        initialValue = minRadiusDp.value,
        targetValue = maxRadiusDp.value,
        animationSpec = infiniteRepeatable(
            animation = tween(1200, easing = FastOutSlowInEasing),
            repeatMode = RepeatMode.Restart
        ),
        label = "waveRadius"
    )

    val waveAlpha by infiniteTransition.animateFloat(
        initialValue = 1f,
        targetValue = 0f,
        animationSpec = infiniteRepeatable(
            animation = tween(1200, easing = FastOutSlowInEasing),
            repeatMode = RepeatMode.Restart
        ),
        label = "waveAlpha"
    )

    Canvas(modifier = modifier.size(72.dp)) {
        val waveRadius = currentRadiusDp.dp.toPx()

        val centerX = size.width / 2
        val centerY = size.height / 2

        val headCenter = Offset(centerX, centerY - 4.dp.toPx())
        val headRadius = 4.dp.toPx()
        val strokeWidth = 2.5.dp.toPx()

        drawArc(
            color = elementColorActive.copy(alpha = waveAlpha),
            startAngle = 135f,
            sweepAngle = 90f,
            useCenter = false,
            topLeft = Offset(headCenter.x - waveRadius, headCenter.y - waveRadius),
            size = Size(waveRadius * 2, waveRadius * 2),
            style = Stroke(width = strokeWidth, cap = StrokeCap.Round)
        )

        drawArc(
            color = elementColorActive.copy(alpha = waveAlpha),
            startAngle = 315f,
            sweepAngle = 90f,
            useCenter = false,
            topLeft = Offset(headCenter.x - waveRadius, headCenter.y - waveRadius),
            size = Size(waveRadius * 2, waveRadius * 2),
            style = Stroke(width = strokeWidth, cap = StrokeCap.Round)
        )

        drawCircle(
            color = elementColorActive,
            radius = headRadius,
            center = headCenter
        )
    }
}