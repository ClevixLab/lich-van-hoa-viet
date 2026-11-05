package vn.clevix.lichviet.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import vn.clevix.lichviet.R

/** Bảng màu tờ lịch — giống pwa/src/style.css (:root và chế độ tối). */
@Immutable
data class LichColors(
    val wall: Color,
    val paper: Color,
    val paper2: Color,
    val rule: Color,
    val ink: Color,
    val ink2: Color,
    val ink3: Color,
    val red: Color,
    val redDeep: Color,
    val redInk: Color,
    val redTint: Color,
    val gold: Color,
    val goldTint: Color,
    val green: Color,
    val greenTint: Color,
    val bandInk: Color,
)

val LightLich = LichColors(
    wall = Color(0xFFE8E3E1), paper = Color(0xFFFDFCFA), paper2 = Color(0xFFF6F2EE), rule = Color(0xFFE4DBD5),
    ink = Color(0xFF231A1A), ink2 = Color(0xFF5E5150), ink3 = Color(0xFF7A6B69),
    red = Color(0xFFC4161C), redDeep = Color(0xFF8F1416), redInk = Color(0xFFB3151B), redTint = Color(0xFFFBECEB),
    gold = Color(0xFF8A5E0C), goldTint = Color(0xFFF8EFDC), green = Color(0xFF1F6B45), greenTint = Color(0xFFE7F2EC),
    bandInk = Color(0xFFFFF8F4),
)

val DarkLich = LichColors(
    wall = Color(0xFF141011), paper = Color(0xFF211A1B), paper2 = Color(0xFF2A2223), rule = Color(0xFF3A3031),
    ink = Color(0xFFF1E9E6), ink2 = Color(0xFFCDBFBB), ink3 = Color(0xFFA99A96),
    red = Color(0xFFFF6B63), redDeep = Color(0xFF7A1214), redInk = Color(0xFFFF8A83), redTint = Color(0xFF3A1D1E),
    gold = Color(0xFFE3B45C), goldTint = Color(0xFF352A18), green = Color(0xFF7FD1A2), greenTint = Color(0xFF1B2E24),
    bandInk = Color(0xFFFFF1EC),
)

val LocalLich = staticCompositionLocalOf { LightLich }

/** Phông tiêu đề và chữ số ("LVHV Serif" = Lora rút gọn, OFL 1.1). */
val Serif = FontFamily(
    Font(R.font.lvhv_serif_regular, FontWeight.Normal),
    Font(R.font.lvhv_serif_semibold, FontWeight.SemiBold),
    Font(R.font.lvhv_serif_bold, FontWeight.Bold),
)

enum class ThemeMode { System, Light, Dark }

@Composable
fun LichTheme(mode: ThemeMode, fontScale: Float, content: @Composable () -> Unit) {
    val dark = when (mode) {
        ThemeMode.System -> isSystemInDarkTheme()
        ThemeMode.Light -> false
        ThemeMode.Dark -> true
    }
    val c = if (dark) DarkLich else LightLich
    val scheme = if (dark) {
        darkColorScheme(
            primary = c.red, onPrimary = Color.White, secondary = c.gold, background = c.wall, onBackground = c.ink,
            surface = c.paper, onSurface = c.ink, surfaceVariant = c.paper2, onSurfaceVariant = c.ink2, outline = c.rule,
            error = c.red, primaryContainer = c.redDeep, onPrimaryContainer = c.bandInk,
        )
    } else {
        lightColorScheme(
            primary = c.redDeep, onPrimary = Color.White, secondary = c.gold, background = c.wall, onBackground = c.ink,
            surface = c.paper, onSurface = c.ink, surfaceVariant = c.paper2, onSurfaceVariant = c.ink2, outline = c.rule,
            error = c.red, primaryContainer = c.redTint, onPrimaryContainer = c.redDeep,
        )
    }
    val base = Typography()
    fun TextStyle.s() = copy(fontSize = fontSize * fontScale, lineHeight = lineHeight * fontScale)
    val typo = Typography(
        displayLarge = base.displayLarge.copy(fontFamily = Serif).s(),
        headlineLarge = base.headlineLarge.copy(fontFamily = Serif, fontWeight = FontWeight.SemiBold).s(),
        headlineMedium = base.headlineMedium.copy(fontFamily = Serif, fontWeight = FontWeight.SemiBold).s(),
        headlineSmall = base.headlineSmall.copy(fontFamily = Serif, fontWeight = FontWeight.SemiBold).s(),
        titleLarge = base.titleLarge.copy(fontFamily = Serif, fontWeight = FontWeight.SemiBold).s(),
        titleMedium = base.titleMedium.copy(fontWeight = FontWeight.SemiBold).s(),
        titleSmall = base.titleSmall.s(),
        bodyLarge = base.bodyLarge.copy(fontSize = 17.sp, lineHeight = 26.sp).s(),
        bodyMedium = base.bodyMedium.copy(fontSize = 15.sp, lineHeight = 22.sp).s(),
        bodySmall = base.bodySmall.s(),
        labelLarge = base.labelLarge.copy(fontSize = 15.sp).s(),
        labelMedium = base.labelMedium.s(),
        labelSmall = base.labelSmall.s(),
    )
    CompositionLocalProvider(LocalLich provides c) {
        MaterialTheme(colorScheme = scheme, typography = typo, content = content)
    }
}
