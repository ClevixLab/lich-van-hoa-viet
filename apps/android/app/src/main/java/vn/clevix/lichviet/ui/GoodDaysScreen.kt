package vn.clevix.lichviet.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import java.time.LocalDate
import java.time.temporal.ChronoUnit
import vn.clevix.lichviet.data.Fmt
import vn.clevix.lichviet.data.GoodDay
import vn.clevix.lichviet.data.Lich
import vn.clevix.lichviet.data.Res

private enum class RangeKey(val label: String) { Week("Tuần này"), Month("Tháng này"), Three("3 tháng tới"), Custom("Tự chọn") }

private const val PAGE = 15

/** Chọn ngày tốt. Năm sinh chỉ giữ trong bộ nhớ khi màn hình đang mở; không ghi xuống máy. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun GoodDaysScreen(today: LocalDate, openDay: (LocalDate) -> Unit) {
    val c = LocalLich.current
    var act by rememberSaveable { mutableStateOf<String?>(null) }
    var range by rememberSaveable { mutableStateOf(RangeKey.Month) }
    var fromText by rememberSaveable { mutableStateOf(Fmt.dmy(today)) }
    var toText by rememberSaveable { mutableStateOf(Fmt.dmy(today.plusDays(30))) }
    // Không dùng rememberSaveable cho năm sinh: không lưu vào trạng thái hệ thống.
    var birth by remember { mutableStateOf("") }
    var shown by remember(act, range, fromText, toText, birth) { mutableIntStateOf(PAGE) }

    val birthYear = birth.toIntOrNull()?.takeIf { it in 1900..today.year }
    val result: Pair<String, List<GoodDay>>? = remember(act, range, fromText, toText, birthYear) {
        val a = act ?: return@remember null
        val r: Pair<LocalDate, LocalDate>? = when (range) {
            RangeKey.Week -> today to today.plusDays(((7 - Fmt.weekday(today)) % 7).toLong())
            RangeKey.Month -> today to today.withDayOfMonth(today.lengthOfMonth())
            RangeKey.Three -> today to today.plusDays(89)
            RangeKey.Custom -> {
                val f = Fmt.parseDmy(fromText)
                val t = Fmt.parseDmy(toText)
                if (f == null || t == null) null else f to t
            }
        }
        if (r == null) return@remember "Hãy nhập đủ ngày bắt đầu và ngày kết thúc (dạng ngày/tháng/năm)." to emptyList()
        val (f, t) = r
        if (!Lich.inRange(f) || !Lich.inRange(t)) return@remember "Lịch hỗ trợ từ năm ${Lich.MIN_YEAR} đến năm ${Lich.MAX_YEAR}." to emptyList()
        val span = ChronoUnit.DAYS.between(f, t)
        if (span < 0) return@remember "Ngày kết thúc cần sau ngày bắt đầu." to emptyList()
        if (span > 366) return@remember "Khoảng tìm tối đa là một năm. Hãy thu hẹp lại." to emptyList()
        when (val res = Lich.findGoodDays(a, f, t, birthYear)) {
            is Res.Err -> res.message to emptyList()
            is Res.Ok -> {
                val name = Lich.activities.firstOrNull { it.id == a }?.name ?: ""
                val tuoi = birthYear?.let { ", tuổi ${Lich.yearCanChiOf(it)}" } ?: ""
                val n = res.value.size
                "$name, từ ${Fmt.dmy(f)} đến ${Fmt.dmy(t)}$tuoi: ${if (n > 0) "$n ngày phù hợp" else "chưa có ngày phù hợp"}." to res.value
            }
        }
    }

    LazyColumn(
        Modifier.fillMaxWidth().widthIn(max = 720.dp),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        item {
            Text("Chọn ngày tốt", style = MaterialTheme.typography.headlineLarge, color = c.ink, modifier = Modifier.semantics { heading() })
            Text("Chọn việc định làm và khoảng thời gian. Ngày được xếp từ tốt nhất.", style = MaterialTheme.typography.bodyLarge, color = c.ink2)
        }
        item {
            Panel {
                Text("Việc định làm", style = MaterialTheme.typography.titleLarge, color = c.ink, modifier = Modifier.semantics { heading() })
                Spacer(Modifier.height(8.dp))
                FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp), maxItemsInEachRow = 2) {
                    Lich.activities.forEach { a ->
                        val sel = act == a.id
                        Text(
                            a.name,
                            textAlign = TextAlign.Center,
                            color = if (sel) Color.White else c.ink,
                            fontWeight = if (sel) FontWeight.SemiBold else FontWeight.Normal,
                            modifier = Modifier
                                .weight(1f)
                                .clip(RoundedCornerShape(12.dp))
                                .background(if (sel) c.redDeep else c.paper2)
                                .clickable(role = Role.RadioButton) { act = a.id }
                                .semantics { this.contentDescription = a.name + if (sel) ", đang chọn" else "" }
                                .padding(vertical = 12.dp, horizontal = 8.dp),
                        )
                    }
                }
                Spacer(Modifier.height(14.dp))
                Text("Trong khoảng", style = MaterialTheme.typography.titleLarge, color = c.ink, modifier = Modifier.semantics { heading() })
                Spacer(Modifier.height(8.dp))
                SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
                    RangeKey.entries.forEachIndexed { i, k ->
                        SegmentedButton(
                            selected = range == k,
                            onClick = { range = k },
                            shape = SegmentedButtonDefaults.itemShape(i, RangeKey.entries.size),
                            label = { Text(k.label, fontSize = 13.sp, maxLines = 2, textAlign = TextAlign.Center) },
                        )
                    }
                }
                if (range == RangeKey.Custom) {
                    Spacer(Modifier.height(8.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedTextField(fromText, { fromText = it }, label = { Text("Từ ngày (ngày/tháng/năm)") }, singleLine = true, modifier = Modifier.weight(1f))
                        OutlinedTextField(toText, { toText = it }, label = { Text("Đến ngày") }, singleLine = true, modifier = Modifier.weight(1f))
                    }
                }
                Spacer(Modifier.height(14.dp))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    OutlinedTextField(
                        value = birth,
                        onValueChange = { birth = it.filter(Char::isDigit).take(4) },
                        label = { Text("Năm sinh âm lịch (không bắt buộc)") },
                        placeholder = { Text("Ví dụ 1976") },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                        modifier = Modifier.weight(1f),
                    )
                    Spacer(Modifier.width(12.dp))
                    Text(birthYear?.let { "Tuổi ${Lich.yearCanChiOf(it)}" } ?: "", color = c.gold, fontFamily = Serif, fontSize = 18.sp)
                }
                Text("Nhập năm âm lịch của ngày sinh (năm tuổi). Ai sinh trước Tết Nguyên đán thì lấy năm trước, ví dụ sinh 20/01/1977 dương lịch (trước Tết Đinh Tỵ) là năm Bính Thìn 1976.", style = MaterialTheme.typography.bodySmall, color = c.ink3)
                Text("Năm sinh chỉ dùng để tránh ngày xung tuổi, không được lưu lại.", style = MaterialTheme.typography.bodySmall, color = c.ink3)
            }
        }
        item {
            Text("Kết quả", style = MaterialTheme.typography.headlineSmall, color = c.ink, modifier = Modifier.semantics { heading() })
            val msg = when {
                act == null -> "Chọn một việc ở trên để xem những ngày phù hợp."
                result == null -> ""
                else -> result.first
            }
            Text(msg, style = MaterialTheme.typography.bodyLarge, color = c.ink2, modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite })
            if (act != null && result != null && result.second.isEmpty() && result.first.contains("chưa có")) {
                Text("Hãy thử mở rộng khoảng thời gian, chẳng hạn 3 tháng tới.", style = MaterialTheme.typography.bodyMedium, color = c.ink3)
            }
        }
        val list = result?.second.orEmpty()
        items(list.take(shown), key = { it.date.toEpochDay() }) { g -> GoodDayItem(g) { openDay(g.date) } }
        if (list.size > shown) {
            item {
                OutlinedButton(onClick = { shown += PAGE }, modifier = Modifier.fillMaxWidth()) {
                    Text("Xem thêm ${minOf(PAGE, list.size - shown)} ngày", color = c.redInk)
                }
            }
        }
    }
}

@Composable
private fun GoodDayItem(g: GoodDay, onClick: () -> Unit) {
    val c = LocalLich.current
    val s = g.date
    val l = remember(s) { Lich.lunar(s) }
    val info = remember(s) { Lich.dayInfo(s) }
    val wd = Fmt.weekday(s)
    val first = info.gio.firstOrNull()
    Row(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(c.paper)
            .border(1.dp, c.rule, RoundedCornerShape(14.dp))
            .clickable(role = Role.Button, onClickLabel = "Mở tờ lịch ngày này", onClick = onClick)
            .semantics(mergeDescendants = true) {
                contentDescription = "${Fmt.WEEKDAY[wd]} ${Fmt.dmy(s)}, âm lịch ${l.d}/${l.m}${if (l.leap) " nhuận" else ""}, điểm ${g.score}"
            }
            .padding(12.dp),
        verticalAlignment = Alignment.Top,
    ) {
        Column(Modifier.width(54.dp), horizontalAlignment = Alignment.CenterHorizontally) {
            Text("${s.dayOfMonth}", fontFamily = Serif, fontWeight = FontWeight.Bold, fontSize = 30.sp, color = if (wd == 0) c.red else c.ink)
            Text("/${s.monthValue}", fontSize = 12.sp, color = c.ink3)
        }
        Column(Modifier.weight(1f).padding(horizontal = 8.dp)) {
            Text("${Fmt.WEEKDAY[wd]}, ${Fmt.dmy(s)}", fontWeight = FontWeight.SemiBold, color = c.ink, style = MaterialTheme.typography.bodyLarge)
            Text("Âm lịch ${l.d}/${l.m}${if (l.leap) " nhuận" else ""}, ngày ${info.ccDay}", style = MaterialTheme.typography.bodyMedium, color = c.ink2)
            if (first != null) Text("Giờ tốt đầu tiên: ${first.chi} (${first.from}–${first.to} giờ)", style = MaterialTheme.typography.bodyMedium, color = c.ink2)
            Text(g.reasons.joinToString("; "), style = MaterialTheme.typography.bodySmall, color = c.green)
        }
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text("${g.score}", fontFamily = Serif, fontWeight = FontWeight.Bold, fontSize = 24.sp, color = c.gold)
            Text("điểm", fontSize = 11.sp, color = c.ink3)
        }
    }
}

@Composable
fun Panel(content: @Composable () -> Unit) {
    val c = LocalLich.current
    Column(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .background(c.paper)
            .padding(16.dp),
    ) { content() }
}
