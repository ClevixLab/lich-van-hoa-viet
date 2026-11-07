@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package vn.clevix.lichviet.ui.people

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.GenericShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.PI
import kotlin.math.sin
import kotlinx.coroutines.delay
import vn.clevix.lichviet.people.DateValue
import vn.clevix.lichviet.people.Lunar
import vn.clevix.lichviet.people.PFmt
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.i
import vn.clevix.lichviet.people.o
import vn.clevix.lichviet.people.s
import vn.clevix.lichviet.people.sn
import vn.clevix.lichviet.ui.Serif

// Cùng một kiểu ở sáng và tối (nền tối ấm để bia nổi), 08 §3.9.
private val BG = Color(0xFF1E1714)
private val STONE = Color(0xFF2B2422)
private val STONE_EDGE = Color(0xFF4A3B33)
private val GOLD = Color(0xFFE8C67A)
private val GOLD_DIM = Color(0xFFC9A961)
private val WOOD = Color(0xFF5A3A24)
private val WOOD_DARK = Color(0xFF3E2716)
private val BRASS = Color(0xFFB0874A)
private val INK_SOFT = Color(0xFFE9DDD0)

/** Thời gian khói chạy mỗi lần thắp (R8: tự dừng sau 2 phút). */
const val SMOKE_MS = 120_000L

/**
 * Màn hình tưởng niệm (08 §3.9, R8, R9): bia, ảnh, mâm lễ đơn giản, ba nén hương có khói nhẹ. Không âm thanh,
 * không chia sẻ. Không đọc thông tin báo tử (R-PRIV-2). Giảm chuyển động → khói tĩnh, không có nút.
 */
@Composable
fun MemorialScreen(x: PeopleCtx, id: String) {
    val p = remember(x.rev, id) { x.repo.person(id) }
    Box(Modifier.fillMaxSize().background(BG)) {
        Column(
            Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Box(Modifier.fillMaxWidth()) {
                TextButton(onClick = { x.nav.back() }) { Text("‹ Về hồ sơ", color = GOLD, style = MaterialTheme.typography.labelLarge) }
            }
            if (p == null || !p.deceased) {
                Text("Không tìm thấy hồ sơ này (có thể đã bị xóa).", color = INK_SOFT)
                return@Column
            }
            val gio = remember(x.rev, x.today) {
                runCatching { x.repo.upcoming(x.today, 400).firstOrNull { it.cls == "gio" && (id in it.personIds || it.entityId == id) } }.getOrNull()
            }
            val sum = remember(x.rev) { runCatching { x.repo.summary(p, x.today) }.getOrNull() }
            val img = rememberPhoto(remember(p.photoId) { p.photoId?.let { x.repo.photo(it) } })
            val catholic = p.memorial?.s("tray_style") == "cong_giao"
            val incense = !catholic && (p.memorial?.get("incense") != false)

            // Dòng ngày giỗ.
            val dateLine = when {
                gio == null -> null
                gio.date == x.today -> "${occTitle(gio).substringBefore(" · ")} · hôm nay ${gio.lunar?.dm ?: ""} âm lịch"
                gio.date.year == x.today.year -> "Ngày giỗ năm nay: ${gio.lunar?.dm ?: ""} âm lịch, ${PFmt.dmy(gio.date)}"
                else -> "Ngày giỗ sắp tới: ${gio.lunar?.dm ?: ""} âm lịch, ${PFmt.dmy(gio.date)}"
            }
            dateLine?.let { Text(it, color = GOLD, style = MaterialTheme.typography.titleMedium, textAlign = TextAlign.Center) }

            // Khói: chạy ≤ 2 phút mỗi lần thắp; giảm chuyển động → tĩnh, không có nút.
            var lit by remember { mutableStateOf(!x.reducedMotion) }
            var round by remember { mutableIntStateOf(0) }
            LaunchedEffect(lit, round) {
                if (lit) { delay(SMOKE_MS); lit = false }
            }
            if (incense && !x.reducedMotion) {
                OutlinedButton(
                    onClick = { if (lit) lit = false else { lit = true; round++ } },
                    colors = ButtonDefaults.outlinedButtonColors(contentColor = GOLD),
                    border = androidx.compose.foundation.BorderStroke(1.dp, GOLD_DIM),
                ) { Text(if (lit) "Dừng khói" else "Thắp lại") }
            }

            OvalPhoto(img, p.hoTen, 132, 168, frame = BRASS, fill = STONE, initialColor = GOLD_DIM)
            Stele(p.quanHe, p.hoTen, p.displayName, p.tenHieu, yearsLine(p), sum?.tuoiMatText)
            Altar(incense = incense, catholic = catholic, animate = lit && !x.reducedMotion)

            p.memorial?.sn("loi_tuong_niem")?.takeIf { it.isNotBlank() }?.let {
                Text(
                    "“$it”", color = INK_SOFT, style = MaterialTheme.typography.bodyLarge, fontStyle = FontStyle.Italic,
                    textAlign = TextAlign.Center, modifier = Modifier.widthIn(max = 480.dp),
                )
            }
        }
    }
}

/** Năm trên bia: "Canh Thìn 1940 — Ất Tỵ 2025"; thiếu năm sinh → "Mất năm Ất Tỵ (2025)"; không rõ năm mất → "Giỗ ngày 10/3 âm lịch". */
private fun yearsLine(p: vn.clevix.lichviet.people.Person): String? {
    val death = p.death ?: return null
    val dl = Lunar.of(death.o("lunar"))
    if (DateValue.of(death.o("input"))?.precision == "month_day" || dl?.year == null) {
        return dl?.let { "Giỗ ngày ${it.dm} âm lịch" }
    }
    val dy = dl.year
    val by = p.birth?.i("lunar_year")
    val bcc = p.birth?.sn("can_chi")
    val dcc = PeopleRepo.canChi(dy)
    return if (by != null && bcc != null) "$bcc $by — $dcc $dy" else "Mất năm $dcc ($dy)"
}

private val SteleShape = GenericShape { size, _ ->
    val r = size.width / 2f
    val top = r * 0.55f
    moveTo(0f, size.height)
    lineTo(0f, top)
    cubicTo(0f, 0f, size.width, 0f, size.width, top)
    lineTo(size.width, size.height)
    close()
}

@Composable
private fun Stele(quanHe: String, hoTen: String, displayName: String, tenHieu: String, years: String?, age: String?) {
    val rel = PFmt.relationLine(quanHe)
    val name = if (rel.isNotEmpty()) hoTen else displayName
    val long = name.length > 28
    Box(
        Modifier.fillMaxWidth(0.8f).widthIn(max = 360.dp).clip(SteleShape).background(STONE)
            .border(2.dp, STONE_EDGE, SteleShape)
            .semantics(mergeDescendants = true) { contentDescription = listOfNotNull(rel.ifEmpty { null }, name, years, age).joinToString(". ") },
    ) {
        Column(
            Modifier.fillMaxWidth().padding(start = 18.dp, end = 18.dp, top = 48.dp, bottom = 22.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            if (rel.isNotEmpty()) Text(rel, color = GOLD, fontFamily = Serif, fontSize = 20.sp, textAlign = TextAlign.Center)
            Text(
                name, color = GOLD, fontFamily = Serif, fontSize = if (long) 20.sp else 26.sp, lineHeight = if (long) 26.sp else 32.sp,
                textAlign = TextAlign.Center, modifier = Modifier.semantics { heading() },
            )
            if (tenHieu.isNotBlank()) Text(tenHieu, color = GOLD_DIM, fontFamily = Serif, fontSize = 15.sp, textAlign = TextAlign.Center)
            years?.let { Text(it, color = GOLD_DIM, fontFamily = Serif, fontSize = 16.sp, textAlign = TextAlign.Center) }
            age?.let { Text(it, color = GOLD, fontFamily = Serif, fontSize = 17.sp, textAlign = TextAlign.Center) }
        }
    }
}

/** Bàn thờ: ba nén hương + khói (hoặc nến, hoa khi Công giáo), bát hương, mâm ngũ quả, hai nến. Vẽ trong mã. */
@Composable
private fun Altar(incense: Boolean, catholic: Boolean, animate: Boolean) {
    val phase = if (animate) {
        val t = rememberInfiniteTransition(label = "khoi")
        t.animateFloat(0f, 1f, infiniteRepeatable(tween(4000, easing = LinearEasing), RepeatMode.Restart), label = "pha").value
    } else 0.35f
    val desc = if (catholic) "Bàn thờ với hai cây nến và bình hoa" else "Bàn thờ với bát hương ba nén hương, mâm ngũ quả và hai cây nến"
    Canvas(Modifier.fillMaxWidth().widthIn(max = 420.dp).height(220.dp).semantics { contentDescription = desc }) {
        val w = size.width
        val h = size.height
        val cx = w / 2f
        val tableY = h * 0.78f
        // Mặt bàn gỗ.
        drawRoundRect(WOOD, Offset(w * 0.04f, tableY), Size(w * 0.92f, h * 0.09f), CornerRadius(6f, 6f))
        drawRect(WOOD_DARK, Offset(w * 0.1f, tableY + h * 0.09f), Size(w * 0.8f, h * 0.06f))
        // Hai cây nến.
        for (fx in listOf(0.12f, 0.88f)) candle(Offset(w * fx, tableY), h)
        if (catholic) {
            // Bình hoa.
            drawRoundRect(BRASS, Offset(cx - 18f, tableY - 46f), Size(36f, 46f), CornerRadius(8f, 8f))
            for ((i, c) in listOf(Color(0xFFE8A0A0), Color(0xFFF3E2B5), Color(0xFFE8A0A0)).withIndex()) {
                drawCircle(c, 12f, Offset(cx - 20f + i * 20f, tableY - 62f - (if (i == 1) 10f else 0f)))
            }
            return@Canvas
        }
        // Mâm ngũ quả bên phải.
        val tx = w * 0.68f
        drawOval(BRASS, Offset(tx - 52f, tableY - 14f), Size(104f, 18f))
        val fruits = listOf(Color(0xFFD9822B), Color(0xFF7FA34A), Color(0xFFE0B13A), Color(0xFFC0463A), Color(0xFF9DBB55))
        fruits.forEachIndexed { i, c -> drawCircle(c, 15f, Offset(tx - 34f + i * 17f, tableY - 22f - (if (i % 2 == 1) 10f else 0f))) }
        drawCircle(Color(0xFFE0B13A), 15f, Offset(tx, tableY - 44f))
        // Bát hương ở giữa.
        val bowlTop = tableY - 40f
        drawRoundRect(BRASS, Offset(cx - 40f, bowlTop), Size(80f, 40f), CornerRadius(14f, 14f))
        drawRect(Color(0xFF8A6838), Offset(cx - 44f, bowlTop - 4f), Size(88f, 8f))
        if (!incense) return@Canvas
        // Ba nén hương.
        val stickTop = bowlTop - h * 0.34f
        val xs = listOf(cx - 16f, cx, cx + 16f)
        xs.forEachIndexed { i, sx ->
            val tilt = (i - 1) * 6f
            drawLine(Color(0xFFB5523B), Offset(sx, bowlTop), Offset(sx + tilt, stickTop + 10f), 4f, StrokeCap.Round)
            drawLine(Color(0xFF8B3A2A), Offset(sx, bowlTop), Offset(sx + tilt * 0.3f, bowlTop - 18f), 4f, StrokeCap.Round)
            drawCircle(Color(0xFFFF9A4D), 3.5f, Offset(sx + tilt, stickTop + 8f))
            // Khói nằm trong vùng vẽ (không đè lên bia).
            smoke(Offset(sx + tilt, stickTop + 4f), (stickTop - 4f).coerceAtLeast(10f), (phase + i * 0.33f) % 1f)
        }
    }
}

private fun DrawScope.candle(base: Offset, h: Float) {
    val cw = 16f
    val ch = h * 0.28f
    drawRect(Color(0xFFC0392B), Offset(base.x - cw / 2, base.y - ch), Size(cw, ch))
    drawRect(BRASS, Offset(base.x - cw, base.y - 6f), Size(cw * 2, 6f))
    val flameY = base.y - ch - 10f
    drawOval(Brush.radialGradient(listOf(Color(0xFFFFE08A), Color(0x00FFB347)), Offset(base.x, flameY), 18f), Offset(base.x - 12f, flameY - 16f), Size(24f, 32f))
    drawOval(Color(0xFFFFD36B), Offset(base.x - 4f, flameY - 8f), Size(8f, 14f))
}

/** Một vệt khói: đường cong lượn, nhạt dần lên trên. [phase] 0..1 dịch vệt lên (tĩnh khi không đổi). */
private fun DrawScope.smoke(from: Offset, height: Float, phase: Float) {
    val path = Path()
    val steps = 24
    path.moveTo(from.x, from.y)
    for (k in 1..steps) {
        val f = k / steps.toFloat()
        val y = from.y - f * height
        val x = from.x + sin((f * 2.2f + phase) * 2 * PI).toFloat() * (4f + f * 10f)
        path.lineTo(x, y)
    }
    drawPath(
        path,
        Brush.verticalGradient(listOf(Color(0x00D9D2CC), Color(0x88D9D2CC)), startY = from.y - height, endY = from.y),
        style = Stroke(width = 3f, cap = StrokeCap.Round),
    )
}
