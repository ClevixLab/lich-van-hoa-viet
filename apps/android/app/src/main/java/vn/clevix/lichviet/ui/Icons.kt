package vn.clevix.lichviet.ui

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.unit.dp

/**
 * Biểu tượng nét, chép từ pwa/src/icons.ts (khung 24×24, nét 1.8). Không dùng thư viện biểu tượng
 * ngoài (material-icons-extended rất nặng).
 */
object LichIcons {
    private fun icon(name: String, vararg paths: String, filled: String? = null): ImageVector {
        val b = ImageVector.Builder(name, 24.dp, 24.dp, 24f, 24f)
        for (p in paths) {
            b.addPath(
                pathData = PathParser().parsePathString(p).toNodes(),
                fill = null,
                stroke = SolidColor(Color.Black),
                strokeLineWidth = 1.8f,
                strokeLineCap = StrokeCap.Round,
                strokeLineJoin = StrokeJoin.Round,
            )
        }
        if (filled != null) {
            b.addPath(pathData = PathParser().parsePathString(filled).toNodes(), fill = SolidColor(Color.Black))
        }
        return b.build()
    }

    // Hình tròn / chữ nhật bo góc viết dưới dạng path để dùng chung một bộ phân tích.
    private const val CIRCLE_8_5 = "M3.5,12a8.5,8.5 0 1,0 17,0a8.5,8.5 0 1,0 -17,0z"

    val Prev = icon("prev", "M15 5l-7 7 7 7")
    val Next = icon("next", "M9 5l7 7-7 7")
    val Sheet = icon(
        "sheet",
        "M6.5,4h11a2.5,2.5 0 0 1 2.5,2.5v12a2.5,2.5 0 0 1 -2.5,2.5h-11a2.5,2.5 0 0 1 -2.5,-2.5v-12a2.5,2.5 0 0 1 2.5,-2.5z",
        "M4 9h16M9 2.5v3M15 2.5v3", "M10.5 13.5h3v4",
    )
    val Star = icon("star", "M12 3.5l2.5 5.2 5.6.7-4.1 3.9 1 5.6-5-2.7-5 2.7 1-5.6-4.1-3.9 5.6-.7z")
    val Swap = icon("swap", "M4 8h13l-3-3M20 16H7l3 3")
    val Info = icon("info", CIRCLE_8_5, "M12 11v5.5M12 7.6v.2")
    val Grid = icon(
        "grid",
        "M5.5,5h13a2,2 0 0 1 2,2v11a2,2 0 0 1 -2,2h-13a2,2 0 0 1 -2,-2v-11a2,2 0 0 1 2,-2z",
        "M3.5 10h17M8 5v-2M16 5v-2M8.5 14h1M11.5 14h1M14.5 14h1M8.5 17h1M11.5 17h1",
    )
    val Close = icon("close", "M6 6l12 12M18 6 6 18")
    val Down = icon("down", "M6 9l6 6 6-6")
    val Contrast = icon("auto", CIRCLE_8_5, filled = "M12 3.5v17a8.5 8.5 0 0 0 0-17z")
}
