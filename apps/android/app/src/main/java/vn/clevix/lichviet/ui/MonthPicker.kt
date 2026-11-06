package vn.clevix.lichviet.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import java.time.LocalDate
import java.time.YearMonth
import vn.clevix.lichviet.data.Fmt
import vn.clevix.lichviet.data.Lich

/** Hộp thoại "Chọn ngày": bảng tháng (Thứ Hai đầu tuần), ngày dương lớn, ngày âm nhỏ. */
@Composable
fun MonthDialog(selected: LocalDate, today: LocalDate, onPick: (LocalDate) -> Unit, onDismiss: () -> Unit) {
    val c = LocalLich.current
    var view by remember { mutableStateOf(YearMonth.of(selected.year, selected.monthValue)) }
    Dialog(onDismissRequest = onDismiss, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        Column(
            Modifier
                .padding(16.dp)
                .fillMaxWidth()
                .clip(RoundedCornerShape(24.dp))
                .background(c.paper)
                .verticalScroll(rememberScrollState())
                .padding(20.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("Chọn ngày", style = MaterialTheme.typography.headlineMedium, color = c.ink, modifier = Modifier.weight(1f))
                IconButton(onClick = onDismiss) { Icon(LichIcons.Close, "Đóng", tint = c.ink) }
            }
            Spacer(Modifier.height(12.dp))
            MonthGrid(view, selected, today, onView = { view = it }, onPick = onPick)
            Spacer(Modifier.height(16.dp))
            Button(
                onClick = { onPick(today) },
                modifier = Modifier.align(Alignment.CenterHorizontally),
                colors = ButtonDefaults.buttonColors(containerColor = c.redDeep, contentColor = Color.White),
            ) { Text("Về hôm nay", style = MaterialTheme.typography.titleMedium) }
        }
    }
}

private fun clampMonth(v: YearMonth): YearMonth = when {
    v.year < Lich.MIN_YEAR -> YearMonth.of(Lich.MIN_YEAR, 1)
    v.year > Lich.MAX_YEAR -> YearMonth.of(Lich.MAX_YEAR, 12)
    else -> v
}

@Composable
fun MonthGrid(
    view: YearMonth,
    selected: LocalDate,
    today: LocalDate,
    onView: (YearMonth) -> Unit,
    onPick: (LocalDate) -> Unit,
) {
    val c = LocalLich.current
    val lunar = remember(view) { Lich.monthLunar(view.year, view.monthValue) }
    val first = view.atDay(1)
    val lead = (Fmt.weekday(first) + 6) % 7 // Thứ Hai đầu tuần
    val n = view.lengthOfMonth()

    // Đầu bảng: ‹  [Tháng ▾] [Năm]  ›
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        IconButton(
            onClick = { onView(clampMonth(view.minusMonths(1))) },
            enabled = !(view.year == Lich.MIN_YEAR && view.monthValue == 1),
        ) { Icon(LichIcons.Prev, "Tháng trước", tint = c.ink) }
        var menu by remember { mutableStateOf(false) }
        Box(Modifier.weight(1f)) {
            OutlinedButton(onClick = { menu = true }, modifier = Modifier.fillMaxWidth()) {
                Text("Tháng ${view.monthValue}", color = c.ink)
            }
            DropdownMenu(expanded = menu, onDismissRequest = { menu = false }) {
                for (m in 1..12) {
                    DropdownMenuItem(text = { Text("Tháng $m") }, onClick = { menu = false; onView(YearMonth.of(view.year, m)) })
                }
            }
        }
        var yearText by remember(view.year) { mutableStateOf(view.year.toString()) }
        val yearValid = yearText.toIntOrNull()?.let { it in Lich.MIN_YEAR..Lich.MAX_YEAR } == true
        OutlinedTextField(
            value = yearText,
            onValueChange = { t ->
                val digits = t.filter { it.isDigit() }.take(4)
                yearText = digits
                digits.toIntOrNull()?.takeIf { it in Lich.MIN_YEAR..Lich.MAX_YEAR }?.let { onView(YearMonth.of(it, view.monthValue)) }
            },
            isError = !yearValid,
            singleLine = true,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
            modifier = Modifier.weight(1f).semantics { contentDescription = "Năm" },
            textStyle = MaterialTheme.typography.titleMedium.copy(textAlign = TextAlign.Center),
        )
        IconButton(
            onClick = { onView(clampMonth(view.plusMonths(1))) },
            enabled = !(view.year == Lich.MAX_YEAR && view.monthValue == 12),
        ) { Icon(LichIcons.Next, "Tháng sau", tint = c.ink) }
    }
    Spacer(Modifier.height(10.dp))
    Row(Modifier.fillMaxWidth()) {
        listOf(1, 2, 3, 4, 5, 6, 0).forEach { w ->
            Text(
                Fmt.WEEKDAY_SHORT[w],
                Modifier.weight(1f).semantics { contentDescription = Fmt.WEEKDAY[w] },
                textAlign = TextAlign.Center,
                fontWeight = FontWeight.SemiBold,
                color = if (w == 0) c.red else c.ink3,
                style = MaterialTheme.typography.labelLarge,
            )
        }
    }
    Spacer(Modifier.height(6.dp))
    val cells = lead + n
    val rows = (cells + 6) / 7
    for (r in 0 until rows) {
        Row(Modifier.fillMaxWidth()) {
            for (col in 0 until 7) {
                val idx = r * 7 + col - lead
                Box(Modifier.weight(1f).aspectRatio(0.92f).padding(2.dp)) {
                    if (idx in 0 until n) {
                        val d = idx + 1
                        val date = view.atDay(d)
                        val l = lunar[idx]
                        val key = l.d == 1 || l.d == 15
                        val isSel = date == selected
                        val isToday = date == today
                        val sunday = col == 6
                        val bg = when {
                            isSel -> c.redDeep
                            key -> c.goldTint
                            else -> Color.Transparent
                        }
                        val main = when {
                            isSel -> Color.White
                            sunday -> c.red
                            else -> c.ink
                        }
                        val sub = when {
                            isSel -> Color.White
                            key -> c.redInk
                            else -> c.ink3
                        }
                        val lunarTxt = if (l.d == 1) "${l.d}/${l.m}${if (l.leap) "n" else ""}" else "${l.d}"
                        Column(
                            Modifier
                                .fillMaxSize()
                                .clip(RoundedCornerShape(10.dp))
                                .background(bg)
                                .then(if (isToday && !isSel) Modifier.border(1.5.dp, c.red, RoundedCornerShape(10.dp)) else Modifier)
                                .clickable(role = Role.Button) { onPick(date) }
                                .semantics(mergeDescendants = true) {
                                    this.selected = isSel
                                    contentDescription = "${Fmt.WEEKDAY[(col + 1) % 7]} $d tháng ${view.monthValue}, âm lịch ${l.d} tháng ${l.m}" +
                                        (if (l.leap) " nhuận" else "") + (if (isToday) ", hôm nay" else "")
                                },
                            horizontalAlignment = Alignment.CenterHorizontally,
                            verticalArrangement = Arrangement.Center,
                        ) {
                            Text("$d", fontFamily = Serif, fontWeight = FontWeight.SemiBold, fontSize = 20.sp, color = main)
                            Text(lunarTxt, fontSize = 12.sp, fontWeight = if (key) FontWeight.Bold else FontWeight.Normal, color = sub)
                        }
                    }
                }
            }
        }
    }
    Spacer(Modifier.height(8.dp))
    Row(verticalAlignment = Alignment.CenterVertically) {
        Box(Modifier.size(14.dp).clip(RoundedCornerShape(3.dp)).background(c.goldTint).border(1.dp, c.gold, RoundedCornerShape(3.dp)))
        Spacer(Modifier.width(8.dp))
        Text("Mùng 1 và ngày rằm âm lịch", style = MaterialTheme.typography.bodySmall, color = c.ink3)
    }
}
