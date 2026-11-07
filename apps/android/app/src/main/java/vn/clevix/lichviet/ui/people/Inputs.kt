@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package vn.clevix.lichviet.ui.people

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import java.time.YearMonth
import vn.clevix.lichviet.people.DateValue
import vn.clevix.lichviet.people.JMap
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.b
import vn.clevix.lichviet.people.list
import vn.clevix.lichviet.people.s
import vn.clevix.lichviet.ui.LocalLich

/**
 * Trạng thái ô nhập ngày (08 §2 "Ô nhập ngày"): ba ô Ngày / Tháng / Năm, Dương/Âm, tháng nhuận, "Không rõ năm" /
 * "Chỉ biết năm". Chỉ kiểm định dạng; ngày có tồn tại không (âm lịch) do lõi trả lời.
 */
@Stable
class DateInputState(
    init: DateValue?,
    val allowLunar: Boolean = true,
    val allowUnknownYear: Boolean = false,
    val allowYearOnly: Boolean = false,
    val minYear: Int = 1900,
    val maxYear: Int = 3000,
) {
    var lunar by mutableStateOf(init?.lunar == true)
    var yearOnly by mutableStateOf(init?.precision == "year")
    var unknownYear by mutableStateOf(init?.precision == "month_day")
    var d by mutableStateOf(init?.day?.toString() ?: "")
    var m by mutableStateOf(init?.month?.toString() ?: "")
    var y by mutableStateOf(init?.year?.toString() ?: "")
    var leap by mutableStateOf(init?.leap == true)
    /** Đã rời ô (hoặc bấm Lưu): từ đây báo lỗi theo từng lần gõ (TK-03). */
    var touched by mutableStateOf(init != null)

    val empty: Boolean get() = d.isBlank() && m.isBlank() && y.isBlank()

    /** Tháng nhuận của năm âm đang nhập (0 = không có; null = chưa có năm hợp lệ). */
    fun leapMonthOfYear(): Int? {
        val yy = y.toIntOrNull() ?: return null
        if (!lunar || unknownYear || yy !in 1899..3000) return null
        return PeopleRepo.leapMonth(yy)
    }

    /** Lỗi định dạng (null = hợp lệ hoặc trống). */
    fun formatError(missing: String): String? {
        if (empty) return missing
        if (yearOnly) {
            val yy = y.toIntOrNull() ?: return "Hãy nhập năm"
            if (yy < minYear || yy > maxYear) return "Năm phải từ $minYear đến $maxYear"
            return null
        }
        val noYear = lunar && unknownYear
        val full = if (noYear) "Hãy nhập đủ ngày và tháng âm" else "Hãy nhập đủ ngày, tháng, năm"
        val dd: Int = d.toIntOrNull() ?: return full
        val mm: Int = m.toIntOrNull() ?: return full
        val yy: Int = if (noYear) 0 else (y.toIntOrNull() ?: return full)
        if (mm < 1 || mm > 12) return "Tháng phải từ 1 đến 12"
        if (!noYear && (yy < minYear || yy > maxYear)) return "Năm phải từ $minYear đến $maxYear"
        if (lunar) {
            if (dd < 1 || dd > 30) return "Ngày âm lịch phải từ 1 đến 30"
            val lm = leapMonthOfYear()
            // Năm không có tháng nhuận: hộp "Tháng nhuận" ẩn, coi như không đánh dấu (R-UX-1).
            if (leap && lm != null && lm != 0 && lm != mm) return "Năm $yy không có tháng $mm nhuận"
        } else {
            val len = YearMonth.of(yy, mm).lengthOfMonth()
            if (dd < 1 || dd > 31) return "Ngày phải từ 1 đến 31"
            if (dd > len) return "Tháng $mm năm $yy chỉ có $len ngày"
        }
        return null
    }

    /** Giá trị nếu đủ định dạng; null nếu trống hoặc sai. */
    fun value(): DateValue? {
        if (formatError("") != null) return null
        val cal = if (lunar) "lunar" else "solar"
        return when {
            yearOnly -> DateValue(cal, "year", y.toInt(), null, null, false)
            lunar && unknownYear -> DateValue(cal, "month_day", null, m.toInt(), d.toInt(), leap)
            else -> {
                val lm = leapMonthOfYear()
                DateValue(cal, "day", y.toInt(), m.toInt(), d.toInt(), lunar && leap && (lm == null || lm == m.toInt()))
            }
        }
    }
}

/**
 * Ô nhập ngày. [result]: dòng kết quả đổi lịch trực tiếp (aria-live) do màn hình gọi lõi tính.
 * [error]: lỗi đang hiện (định dạng hoặc của lõi).
 */
@Composable
fun DateInput(
    st: DateInputState,
    result: String?,
    error: String?,
    calLabels: List<String> = listOf("Nhập theo dương lịch", "Nhập theo âm lịch"),
    unknownYearLabel: String = "Không rõ năm",
    yearOnlyLabels: List<String>? = null,
    extra: (@Composable () -> Unit)? = null,
) {
    val c = LocalLich.current
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (st.allowLunar) {
            Choice(calLabels, if (st.lunar) 1 else 0, { st.lunar = it == 1; if (!st.lunar) st.unknownYear = false; st.touched = true })
        }
        if (yearOnlyLabels != null) {
            Choice(yearOnlyLabels, if (st.yearOnly) 1 else 0, { st.yearOnly = it == 1 })
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            val err = error != null && st.touched
            if (!st.yearOnly) {
                NumIn("Ngày", st.d, { st.d = it }, Modifier.weight(1f), err, 2)
                NumIn("Tháng", st.m, { st.m = it }, Modifier.weight(1f), err, 2)
            }
            if (!(st.lunar && st.unknownYear)) {
                NumIn("Năm", st.y, { st.y = it }, Modifier.weight(1.4f), err, 4)
            }
        }
        if (st.lunar && !st.yearOnly) {
            val lm = st.leapMonthOfYear()
            when {
                lm == 0 -> Fine("Năm ${st.y} không có tháng nhuận")
                lm != null -> CheckRow("Tháng nhuận (năm ${st.y} nhuận tháng $lm)", st.leap, { st.leap = it })
                else -> CheckRow("Tháng nhuận", st.leap, { st.leap = it })
            }
        }
        extra?.invoke()
        if (st.lunar && st.allowUnknownYear && !st.yearOnly) {
            CheckRow(unknownYearLabel, st.unknownYear, { st.unknownYear = it })
        }
        if (error != null && st.touched) ErrorText(error)
        else if (result != null) {
            Text(
                result, style = MaterialTheme.typography.bodyLarge, color = c.green,
                modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
            )
        }
    }
}

/** Trạng thái quy tắc nhắc (01 §3.6). */
@Stable
class RuleState(init: Map<String, Any?>?, defaults: Map<String, Any?>?) {
    val id: String = init?.s("id")?.takeIf { it.isNotEmpty() } ?: PeopleRepo.uuid()
    var enabled by mutableStateOf(init?.let { it.b("enabled") } ?: true)
    val offsets = mutableStateListOf<Int>().apply {
        val src = init?.list("offsets_days") ?: defaults?.list("default_offsets_days") ?: listOf(0L, 3L)
        addAll(src.mapNotNull { (it as? Number)?.toInt() })
    }
    var custom by mutableStateOf(offsets.firstOrNull { it !in PRESETS }?.toString() ?: "")
    private val time = (init?.s("time_local")?.takeIf { it.isNotEmpty() } ?: defaults?.s("default_time_local")?.takeIf { it.isNotEmpty() } ?: "07:00")
    var hh by mutableStateOf(time.substringBefore(':'))
    var mi by mutableStateOf(time.substringAfter(':'))
    var tienThuong by mutableStateOf(init?.b("tien_thuong") == true)

    fun toggle(k: Int) {
        if (k in offsets) offsets.remove(k) else offsets.add(k)
    }

    /** Mốc cuối cùng (có cả ô "tự đặt"), tăng dần, không trùng. */
    fun allOffsets(): List<Int> {
        val set = offsets.filter { it in PRESETS }.toMutableSet()
        custom.toIntOrNull()?.let { if (it in 1..60) set.add(it) }
        return set.sorted()
    }

    fun timeText(): String = "%02d:%02d".format(hh.toIntOrNull() ?: 0, mi.toIntOrNull() ?: 0)

    fun errors(): List<String> {
        val e = ArrayList<String>()
        if (!enabled) return e
        if (custom.isNotBlank() && (custom.toIntOrNull() ?: 0) !in 1..60) e.add("Số ngày tự đặt phải từ 1 đến 60")
        if (allOffsets().size > 8) e.add("Tối đa 8 mốc nhắc")
        val h = hh.toIntOrNull()
        val m = mi.toIntOrNull()
        if (h == null || m == null || h !in 0..23 || m !in 0..59) e.add("Giờ nhắc phải từ 0 đến 23 giờ, phút từ 0 đến 59")
        return e
    }

    fun toMap(withTienThuong: Boolean): JMap = linkedMapOf<String, Any?>(
        "id" to id,
        "enabled" to enabled,
        "offsets_days" to ArrayList(allOffsets()),
        "time_local" to timeText(),
        "tien_thuong" to (withTienThuong && tienThuong),
    )

    fun sameAs(o: RuleState): Boolean =
        enabled == o.enabled && allOffsets() == o.allOffsets() && timeText() == o.timeText()

    fun copyFrom(o: RuleState) {
        enabled = o.enabled
        offsets.clear(); offsets.addAll(o.offsets)
        custom = o.custom
        hh = o.hh; mi = o.mi
    }

    companion object {
        val PRESETS = listOf(0, 1, 3, 5, 10, 15)
    }
}

/** Khối nhắc: bật/tắt, mốc nhắc trước, giờ nhắc 24 giờ (TK-28), tiên thường (chỉ giỗ). */
@Composable
fun RuleEditor(r: RuleState, switchLabel: String, showTienThuong: Boolean, showErrors: Boolean) {
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        SwitchRow(switchLabel, r.enabled, { r.enabled = it })
        if (r.enabled) {
            Fine("Nhắc trước")
            Flow {
                RuleState.PRESETS.forEach { k ->
                    Chip(if (k == 0) "Đúng ngày" else "$k ngày", k in r.offsets, { r.toggle(k) })
                }
            }
            Row(verticalAlignment = Alignment.CenterVertically) {
                NumIn("Tự đặt (1–60 ngày)", r.custom, { r.custom = it }, Modifier.width(200.dp), maxLen = 2)
            }
            if (r.allOffsets().isEmpty()) Fine("Chưa chọn mốc nhắc nào — sẽ không có lời nhắc")
            Fine("Giờ nhắc (24 giờ)")
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                NumIn("Giờ", r.hh, { r.hh = it }, Modifier.width(96.dp), maxLen = 2)
                Text(":", style = MaterialTheme.typography.titleLarge)
                NumIn("Phút", r.mi, { r.mi = it }, Modifier.width(96.dp), maxLen = 2)
            }
            if (showTienThuong) {
                CheckRow("Nhắc thêm ngày tiên thường (lễ cúng hôm trước ngày giỗ)", r.tienThuong, { r.tienThuong = it })
            }
            if (showErrors) r.errors().forEach { ErrorText(it) }
        }
    }
}

/** Đường kẻ mảnh giữa các khối. */
@Composable
fun Rule() {
    val c = LocalLich.current
    androidx.compose.material3.HorizontalDivider(Modifier.fillMaxWidth(), color = c.rule)
}
