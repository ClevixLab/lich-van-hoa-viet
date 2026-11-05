package vn.clevix.lichviet.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Checkbox
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import java.time.LocalDate
import vn.clevix.lichviet.data.Fmt
import vn.clevix.lichviet.data.Lich

/** Đổi ngày dương ↔ âm. */
@Composable
fun ConvertScreen(current: LocalDate, today: LocalDate, openDay: (LocalDate) -> Unit) {
    val c = LocalLich.current
    Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.TopCenter) {
        Column(
            Modifier.widthIn(max = 720.dp).fillMaxWidth().verticalScroll(rememberScrollState()).padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text("Đổi ngày", style = MaterialTheme.typography.headlineLarge, color = c.ink, modifier = Modifier.semantics { heading() })
            Text("Đổi qua lại giữa dương lịch và âm lịch, từ năm ${Lich.MIN_YEAR} đến năm ${Lich.MAX_YEAR}.", style = MaterialTheme.typography.bodyLarge, color = c.ink2)

            // Dương → âm
            Panel {
                Text("Dương lịch sang âm lịch", style = MaterialTheme.typography.titleLarge, color = c.ink, modifier = Modifier.semantics { heading() })
                Spacer(Modifier.height(8.dp))
                var sText by rememberSaveable { mutableStateOf(Fmt.dmy(current)) }
                OutlinedTextField(sText, { sText = it }, label = { Text("Ngày dương lịch (ngày/tháng/năm)") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                val s = Fmt.parseDmy(sText)
                Column(Modifier.padding(top = 8.dp).semantics { liveRegion = LiveRegionMode.Polite }) {
                    if (s == null || !Lich.inRange(s)) {
                        Text("Hãy nhập một ngày dương lịch từ năm ${Lich.MIN_YEAR} đến năm ${Lich.MAX_YEAR}.", color = c.redInk)
                    } else {
                        val info = remember(s) { Lich.dayInfo(s) }
                        Text("${info.lunarD} tháng ${Fmt.LUNAR_MONTH[info.lunarM]}${if (info.leap) " (nhuận)" else ""}", fontFamily = Serif, fontSize = 26.sp, color = c.ink)
                        Text("Năm ${info.ccYear} · ngày ${info.ccDay} · tháng ${info.ccMonth}", style = MaterialTheme.typography.bodyLarge, color = c.ink2)
                        TextButton(onClick = { openDay(s) }) { Text("Mở tờ lịch ${Fmt.WEEKDAY[info.weekday]}, ${Fmt.dmy(s)}", color = c.redInk) }
                    }
                }
            }

            // Âm → dương
            Panel {
                Text("Âm lịch sang dương lịch", style = MaterialTheme.typography.titleLarge, color = c.ink, modifier = Modifier.semantics { heading() })
                Spacer(Modifier.height(8.dp))
                val l0 = remember { Lich.lunar(Lich.clamp(current)) }
                var ld by rememberSaveable { mutableIntStateOf(l0.d) }
                var lm by rememberSaveable { mutableIntStateOf(l0.m) }
                var lyText by rememberSaveable { mutableStateOf(l0.y.toString()) }
                var leap by rememberSaveable { mutableStateOf(l0.leap) }
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Picker("Ngày", Fmt.lunarDayLabel(ld), (1..30).map { it to Fmt.lunarDayLabel(it) }, { ld = it }, Modifier.weight(1f))
                    Picker("Tháng", "Tháng $lm", (1..12).map { it to "Tháng $it (${Fmt.LUNAR_MONTH[it]})" }, { lm = it }, Modifier.weight(1.2f))
                    OutlinedTextField(
                        lyText,
                        { lyText = it.filter(Char::isDigit).take(4) },
                        label = { Text("Năm") },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                        modifier = Modifier.weight(1f),
                    )
                }
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Checkbox(leap, { leap = it })
                    Text("Tháng nhuận", color = c.ink)
                }
                val ly = lyText.toIntOrNull()
                Column(Modifier.semantics { liveRegion = LiveRegionMode.Polite }) {
                    if (ly == null || ly !in Lich.MIN_YEAR..Lich.MAX_YEAR) {
                        Text("Hãy nhập năm âm lịch từ ${Lich.MIN_YEAR} đến ${Lich.MAX_YEAR}.", color = c.redInk)
                    } else {
                        val s = remember(ly, lm, ld, leap) { Lich.toSolar(ly, lm, ld, leap) }
                        if (s == null) {
                            val plain = if (leap) Lich.toSolar(ly, lm, ld, false) else null
                            Text(
                                if (leap) "Năm $ly không có tháng $lm nhuận${if (plain != null) "" else ", hoặc tháng này không có ngày $ld"}. Hãy bỏ chọn \"Tháng nhuận\" hoặc chọn tháng khác."
                                else "Tháng $lm âm lịch năm $ly không có ngày $ld (tháng thiếu chỉ có 29 ngày).",
                                color = c.redInk,
                            )
                        } else {
                            Text("${Fmt.WEEKDAY[Fmt.weekday(s)]}, ${Fmt.dmy(s)}", fontFamily = Serif, fontSize = 26.sp, color = c.ink)
                            TextButton(onClick = { openDay(s) }) { Text("Mở tờ lịch ngày này", color = c.redInk) }
                        }
                    }
                }
            }
            Text("Hôm nay theo máy của bạn: ${Fmt.dmy(today)}.", style = MaterialTheme.typography.bodySmall, color = c.ink3)
        }
    }
}

@Composable
private fun Picker(label: String, value: String, options: List<Pair<Int, String>>, onPick: (Int) -> Unit, modifier: Modifier) {
    val c = LocalLich.current
    var open by remember { mutableStateOf(false) }
    Column(modifier) {
        Text(label, style = MaterialTheme.typography.labelMedium, color = c.ink3)
        Box {
            OutlinedButton(onClick = { open = true }, modifier = Modifier.fillMaxWidth()) { Text(value, color = c.ink, maxLines = 1) }
            DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
                options.forEach { (v, t) -> DropdownMenuItem(text = { Text(t) }, onClick = { open = false; onPick(v) }) }
            }
        }
    }
}
