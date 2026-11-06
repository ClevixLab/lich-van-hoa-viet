package vn.clevix.lichviet.ui

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PagerState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import java.time.LocalDate
import vn.clevix.lichviet.data.DayInfo
import vn.clevix.lichviet.data.Fmt
import vn.clevix.lichviet.data.Lich

/** Tờ lịch: lật trái/phải bằng HorizontalPager, mỗi trang là một ngày 2000–3000. */
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun SheetPager(pager: PagerState, today: LocalDate, fontScale: Float, onOpenMonth: () -> Unit) {
    HorizontalPager(
        state = pager,
        beyondViewportPageCount = 1,
        modifier = Modifier.fillMaxSize(),
        key = { it },
    ) { page ->
        val date = Lich.dateOfPage(page)
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
            Column(
                Modifier
                    .widthIn(max = 560.dp)
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = 16.dp, vertical = 8.dp),
            ) {
                DaySheet(date, date == today, fontScale, onOpenMonth)
                Spacer(Modifier.height(24.dp))
            }
        }
    }
}

@Composable
fun DaySheet(date: LocalDate, isToday: Boolean, fontScale: Float, onOpenMonth: () -> Unit) {
    val c = LocalLich.current
    val info = remember(date) { Lich.dayInfo(date) }
    val cadao = remember(date) { Lich.cadao(date) }
    val wd = info.weekday
    val sunday = wd == 0
    val bigColor = if (sunday) c.red else c.ink

    Box {
        Column(
            Modifier
                .padding(top = 9.dp)
                .clip(RoundedCornerShape(6.dp))
                .background(c.paper),
        ) {
            // Dải đầu tờ: "Tháng 11 2026" + mép răng cưa.
            Box(
                Modifier
                    .fillMaxWidth()
                    .background(c.redDeep)
                    .clickable(role = Role.Button, onClickLabel = "Mở bảng tháng") { onOpenMonth() }
                    .semantics { contentDescription = "Tháng ${date.monthValue} năm ${date.year}. Mở bảng tháng để chọn ngày khác" }
                    .padding(vertical = 18.dp),
                contentAlignment = Alignment.Center,
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text("Tháng ${date.monthValue}", color = c.bandInk, style = MaterialTheme.typography.headlineSmall)
                    Spacer(Modifier.width(10.dp))
                    Text("${date.year}", color = c.bandInk.copy(alpha = .82f), style = MaterialTheme.typography.headlineSmall.copy(fontWeight = FontWeight.Normal))
                    Spacer(Modifier.width(10.dp))
                    Icon(LichIcons.Grid, null, tint = c.bandInk.copy(alpha = .8f), modifier = Modifier.size(22.dp))
                }
            }
            Scallop(c.redDeep)

            Column(Modifier.padding(horizontal = 18.dp)) {
                // Ngày dương
                Column(
                    Modifier
                        .fillMaxWidth()
                        .padding(top = 12.dp, bottom = 8.dp)
                        .semantics(mergeDescendants = true) {
                            heading()
                            contentDescription = "${Fmt.WEEKDAY[wd]}, ngày ${date.dayOfMonth} tháng ${date.monthValue} năm ${date.year}. " +
                                "Âm lịch ${info.lunarD} tháng ${info.lunarM}${if (info.leap) " nhuận" else ""} năm ${info.ccYear}"
                        },
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Text(Fmt.WEEKDAY[wd], style = MaterialTheme.typography.headlineMedium, color = bigColor)
                    Text(
                        "${date.dayOfMonth}",
                        fontFamily = Serif,
                        fontWeight = FontWeight.SemiBold,
                        fontSize = (150 * fontScale).sp,
                        lineHeight = (160 * fontScale).sp,
                        color = bigColor,
                    )
                    if (isToday) {
                        Text(
                            "Hôm nay",
                            color = c.redInk,
                            style = MaterialTheme.typography.labelLarge,
                            modifier = Modifier
                                .clip(RoundedCornerShape(50))
                                .background(c.redTint)
                                .padding(horizontal = 12.dp, vertical = 3.dp),
                        )
                    }
                }
                HorizontalDivider(color = c.rule)
                LunarBlock(info)
                HorizontalDivider(color = c.rule)
                Labels(info)
                Section("Giờ hoàng đạo") { HoursGrid(info) }
                DoDont(info)
                if (cadao != null) {
                    Column(
                        Modifier
                            .padding(vertical = 14.dp)
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(12.dp))
                            .background(c.goldTint)
                            .padding(16.dp)
                            .semantics(mergeDescendants = true) { contentDescription = "Ca dao trong ngày: ${cadao.text}" },
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        cadao.lines.forEach { line ->
                            Text(
                                line,
                                fontFamily = Serif,
                                fontSize = 18.sp * fontScale,
                                lineHeight = 27.sp * fontScale,
                                color = c.ink,
                                textAlign = TextAlign.Center,
                            )
                        }
                        Spacer(Modifier.height(6.dp))
                        Text(cadao.type, style = MaterialTheme.typography.labelMedium, color = c.gold)
                    }
                }
                Details(info)
                Text(
                    "Theo Ngọc Hạp Thông Thư; tham khảo",
                    style = MaterialTheme.typography.bodySmall,
                    color = c.ink3,
                    modifier = Modifier.padding(bottom = 16.dp),
                )
            }
        }
        // Hai khoen treo tờ lịch
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceEvenly) {
            repeat(2) {
                Box(
                    Modifier
                        .size(width = 14.dp, height = 22.dp)
                        .clip(RoundedCornerShape(8.dp))
                        .background(Brush.verticalGradient(listOf(Color(0xFFD8D2CE), Color(0xFF9D9592)))),
                )
            }
        }
    }
}

/** Mép răng cưa dưới dải đỏ (giống .sheet-band::after của PWA). */
@Composable
private fun Scallop(color: Color) {
    Box(
        Modifier
            .fillMaxWidth()
            .height(6.dp)
            .drawBehind {
                val step = 10.dp.toPx()
                val r = 3.4.dp.toPx()
                var x = step / 2
                while (x < size.width + step) {
                    drawCircle(color, r, Offset(x, 0f))
                    x += step
                }
            },
    )
}

@Composable
private fun LunarBlock(info: DayInfo) {
    val c = LocalLich.current
    val key = info.lunarD == 1 || info.lunarD == 15
    Row(
        Modifier
            .fillMaxWidth()
            .height(IntrinsicSize.Min)
            .padding(vertical = 14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(0.42f), horizontalAlignment = Alignment.CenterHorizontally) {
            Text("Âm lịch", style = MaterialTheme.typography.bodyMedium, color = c.ink3)
            Text(
                "${info.lunarD}",
                fontFamily = Serif,
                fontWeight = FontWeight.Bold,
                fontSize = 56.sp,
                lineHeight = 60.sp,
                color = if (key) c.red else c.gold,
            )
            Text(
                "tháng ${Fmt.LUNAR_MONTH[info.lunarM]}${if (info.leap) " (nhuận)" else ""}",
                fontFamily = Serif,
                fontSize = 19.sp,
                color = c.ink,
                textAlign = TextAlign.Center,
            )
            Text(if (info.monthLen == 30) "tháng đủ" else "tháng thiếu", style = MaterialTheme.typography.bodySmall, color = c.ink3)
        }
        VerticalDivider(Modifier.fillMaxHeight().padding(vertical = 4.dp), color = c.rule)
        Column(Modifier.weight(0.58f).padding(start = 16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            KV("Năm", info.ccYear)
            KV("Tháng", info.ccMonth)
            KV("Ngày", info.ccDay)
            KV("Tiết", info.tietKhi)
        }
    }
}

@Composable
private fun KV(k: String, v: String) {
    val c = LocalLich.current
    Row(verticalAlignment = Alignment.Bottom) {
        Text(k, style = MaterialTheme.typography.bodyMedium, color = c.ink3, modifier = Modifier.width(64.dp))
        Text(v, fontFamily = Serif, fontSize = 19.sp, color = c.ink)
    }
}

@Composable
private fun Labels(info: DayInfo) {
    val c = LocalLich.current
    val r = Lich.rating(info.ngocHap.score)
    val (rb, rf) = when (r.level) {
        3 -> c.greenTint to c.green
        2 -> c.greenTint to c.green
        1 -> c.paper2 to c.ink
        else -> c.redTint to c.redInk
    }
    Column(Modifier.padding(vertical = 14.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        LabelBox(
            if (info.hoangDao) "Hoàng đạo" else "Hắc đạo",
            info.thanName,
            if (info.hoangDao) c.goldTint else c.paper2,
            if (info.hoangDao) c.gold else c.ink,
            Modifier.fillMaxWidth(),
        )
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            LabelBox("Trực", info.truc, c.paper2, c.ink, Modifier.weight(1f))
            LabelBox("Đánh giá", r.label, rb, rf, Modifier.weight(1f))
        }
    }
}

@Composable
private fun LabelBox(k: String, v: String, bg: Color, fg: Color, modifier: Modifier) {
    val c = LocalLich.current
    Column(
        modifier
            .clip(RoundedCornerShape(12.dp))
            .background(bg)
            .padding(vertical = 10.dp, horizontal = 12.dp)
            .semantics(mergeDescendants = true) {},
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(k, style = MaterialTheme.typography.bodyMedium, color = c.ink3)
        Text(v, style = MaterialTheme.typography.titleMedium, color = fg)
    }
}

@Composable
fun Section(title: String, content: @Composable () -> Unit) {
    val c = LocalLich.current
    Column(Modifier.padding(vertical = 10.dp)) {
        Text(title, style = MaterialTheme.typography.titleLarge, color = c.ink, modifier = Modifier.semantics { heading() })
        Spacer(Modifier.height(8.dp))
        content()
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun HoursGrid(info: DayInfo) {
    val c = LocalLich.current
    FlowRow(
        Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
        maxItemsInEachRow = 3,
    ) {
        info.gio.forEach { g ->
            Column(
                Modifier
                    .weight(1f)
                    .clip(RoundedCornerShape(10.dp))
                    .border(1.dp, c.rule, RoundedCornerShape(10.dp))
                    .padding(vertical = 8.dp)
                    .semantics(mergeDescendants = true) { contentDescription = "Giờ ${g.chi}, ${Fmt.hourRange(g)}" },
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Text(g.chi, fontFamily = Serif, fontWeight = FontWeight.SemiBold, fontSize = 18.sp, color = c.ink)
                Text(Fmt.hourRange(g), style = MaterialTheme.typography.bodySmall, color = c.ink3)
            }
        }
    }
}

private const val MAX_LIST = 4

@Composable
private fun DoDont(info: DayInfo) {
    Row(Modifier.fillMaxWidth().padding(vertical = 6.dp), horizontalArrangement = Arrangement.spacedBy(14.dp)) {
        Column(Modifier.weight(1f)) { ExpandableList("Nên làm", info.ngocHap.nen, LocalLich.current.green) }
        Column(Modifier.weight(1f)) { ExpandableList("Kỵ", info.ngocHap.ky, LocalLich.current.redInk) }
    }
}

@Composable
private fun ExpandableList(title: String, items: List<String>, accent: Color) {
    val c = LocalLich.current
    var open by rememberSaveable(items) { mutableStateOf(false) }
    Column(Modifier.animateContentSize()) {
        Text(title, style = MaterialTheme.typography.titleLarge, color = accent, modifier = Modifier.semantics { heading() })
        Spacer(Modifier.height(4.dp))
        if (items.isEmpty()) {
            Text("Không có ghi nhận", style = MaterialTheme.typography.bodyMedium, color = c.ink3)
        } else {
            val shown = if (open) items else items.take(MAX_LIST)
            shown.forEach { Text("• $it", style = MaterialTheme.typography.bodyMedium, color = c.ink, modifier = Modifier.padding(vertical = 2.dp)) }
            if (items.size > MAX_LIST) {
                TextButton(onClick = { open = !open }, contentPadding = PaddingValues(0.dp)) {
                    Text(if (open) "Thu gọn" else "Xem thêm ${items.size - MAX_LIST} việc", color = c.redInk)
                }
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun Chips(items: List<String>, bg: Color, fg: Color) {
    val c = LocalLich.current
    if (items.isEmpty()) {
        Text("Không có", style = MaterialTheme.typography.bodyMedium, color = c.ink3)
        return
    }
    FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        items.forEach {
            Text(
                it,
                color = fg,
                style = MaterialTheme.typography.bodyMedium,
                modifier = Modifier
                    .clip(RoundedCornerShape(50))
                    .background(bg)
                    .padding(horizontal = 10.dp, vertical = 3.dp),
            )
        }
    }
}

@Composable
private fun Details(info: DayInfo) {
    val c = LocalLich.current
    var open by rememberSaveable { mutableStateOf(false) }
    Column(
        Modifier
            .padding(vertical = 8.dp)
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .border(1.dp, c.rule, RoundedCornerShape(12.dp))
            .animateContentSize(),
    ) {
        Row(
            Modifier
                .fillMaxWidth()
                .clickable(role = Role.Button, onClickLabel = if (open) "Thu gọn chi tiết" else "Mở chi tiết") { open = !open }
                .padding(14.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("Chi tiết", style = MaterialTheme.typography.titleLarge, color = c.ink, modifier = Modifier.weight(1f))
            Icon(LichIcons.Down, null, tint = c.ink2, modifier = Modifier.size(22.dp).rotate(if (open) 180f else 0f))
        }
        if (open) {
            Column(Modifier.padding(start = 14.dp, end = 14.dp, bottom = 14.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text("Sao tốt", style = MaterialTheme.typography.titleMedium, color = c.ink)
                Chips(info.ngocHap.saoTot, c.greenTint, c.green)
                Text("Sao xấu", style = MaterialTheme.typography.titleMedium, color = c.ink)
                Chips(info.ngocHap.saoXau, c.redTint, c.redInk)
                FactRow("Nhị thập bát tú", "Sao ${info.tu}")
                FactRow("Nạp âm", info.napAm)
                FactRow("Giờ Tý đầu ngày", info.ccHourTy)
                FactRow("Tuổi xung", info.ngocHap.xungTuoi.joinToString(", ").ifEmpty { "Không có" })
            }
        }
    }
}

@Composable
private fun FactRow(k: String, v: String) {
    val c = LocalLich.current
    Row(Modifier.fillMaxWidth().semantics(mergeDescendants = true) {}) {
        Text(k, style = MaterialTheme.typography.bodyMedium, color = c.ink3, modifier = Modifier.weight(0.45f))
        Text(v, style = MaterialTheme.typography.bodyLarge, color = c.ink, modifier = Modifier.weight(0.55f))
    }
}
