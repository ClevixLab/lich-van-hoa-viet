package vn.clevix.lichviet.people

import java.time.LocalDate
import vn.clevix.lichviet.core.SolarDay
import vn.clevix.lichviet.data.JsonReader

/*
 * Mô hình hiển thị của mô-đun "Người thân". Dữ liệu thật là JSON đúng lược đồ
 * docs/v0.2/schema/backup-v1.schema.json (Person, Event, …) do lõi Rust (lvhv-ffi, src/v02.rs) đọc/ghi;
 * giao diện chỉ đọc trường để hiển thị và dựng JSON nhập liệu. Mọi tính toán (đổi lịch, ngày giỗ, tuổi, nhắc,
 * danh sách theo kỳ, .ics, sao lưu) ở lõi.
 */

typealias JMap = MutableMap<String, Any?>

/** Bộ ghi JSON tối giản (đi cùng JsonReader). Giữ thứ tự khóa. */
internal object JsonW {
    fun write(v: Any?): String = StringBuilder().also { w(it, v) }.toString()

    private fun w(b: StringBuilder, v: Any?) {
        when (v) {
            null -> b.append("null")
            is String -> str(b, v)
            is Boolean -> b.append(if (v) "true" else "false")
            is Int, is Long, is Short, is Byte -> b.append(v.toString())
            is Double -> if (v % 1.0 == 0.0 && kotlin.math.abs(v) < 1e15) b.append(v.toLong()) else b.append(v.toString())
            is Number -> b.append(v.toString())
            is Map<*, *> -> {
                b.append('{')
                var first = true
                for ((k, x) in v) {
                    if (!first) b.append(',')
                    first = false
                    str(b, k.toString())
                    b.append(':')
                    w(b, x)
                }
                b.append('}')
            }
            is Iterable<*> -> {
                b.append('[')
                var first = true
                for (x in v) {
                    if (!first) b.append(',')
                    first = false
                    w(b, x)
                }
                b.append(']')
            }
            else -> str(b, v.toString())
        }
    }

    private fun str(b: StringBuilder, s: String) {
        b.append('"')
        for (c in s) {
            when {
                c == '"' -> b.append("\\\"")
                c == '\\' -> b.append("\\\\")
                c == '\n' -> b.append("\\n")
                c == '\r' -> b.append("\\r")
                c == '\t' -> b.append("\\t")
                c < ' ' -> b.append("\\u%04x".format(c.code))
                else -> b.append(c)
            }
        }
        b.append('"')
    }
}

/** Đọc JSON thành cây có thể sửa (Map/List lồng nhau đều sửa được). */
@Suppress("UNCHECKED_CAST")
internal fun parseJ(text: String): Any? = deep(JsonReader.parse(text))

@Suppress("UNCHECKED_CAST")
internal fun parseObj(text: String): JMap = parseJ(text) as JMap

@Suppress("UNCHECKED_CAST")
internal fun parseArr(text: String): MutableList<Any?> = parseJ(text) as MutableList<Any?>

internal fun deep(v: Any?): Any? = when (v) {
    is Map<*, *> -> LinkedHashMap<String, Any?>().also { m -> v.forEach { (k, x) -> m[k.toString()] = deep(x) } }
    is List<*> -> ArrayList<Any?>().also { l -> v.forEach { l.add(deep(it)) } }
    else -> v
}

@Suppress("UNCHECKED_CAST")
internal fun Map<String, Any?>.copyJ(): JMap = deep(this) as JMap

@Suppress("UNCHECKED_CAST")
internal fun Map<String, Any?>.o(k: String): JMap? = this[k] as? JMap
internal fun Map<String, Any?>.s(k: String): String = this[k] as? String ?: ""
internal fun Map<String, Any?>.sn(k: String): String? = this[k] as? String
internal fun Map<String, Any?>.i(k: String): Int? = (this[k] as? Number)?.toInt()
internal fun Map<String, Any?>.b(k: String): Boolean = this[k] == true

@Suppress("UNCHECKED_CAST")
internal fun Map<String, Any?>.list(k: String): List<Any?> = this[k] as? List<Any?> ?: emptyList()
internal fun Map<String, Any?>.strs(k: String): List<String> = list(k).mapNotNull { it as? String }

@Suppress("UNCHECKED_CAST")
internal fun Map<String, Any?>.objs(k: String): List<JMap> = list(k).mapNotNull { it as? JMap }

fun LocalDate.solar(): SolarDay = SolarDay(year, monthValue, dayOfMonth)
fun SolarDay.local(): LocalDate = LocalDate.of(y, m, d)

/** "YYYY-MM-DD" → LocalDate. */
fun isoDate(s: String): LocalDate = LocalDate.parse(s)

/** `DateValue` (01 §2.1) — một ngày người dùng nhập. */
data class DateValue(
    val calendar: String = "solar",
    val precision: String = "day",
    val year: Int? = null,
    val month: Int? = null,
    val day: Int? = null,
    val leap: Boolean = false,
) {
    val lunar: Boolean get() = calendar == "lunar"

    fun toMap(): JMap = linkedMapOf<String, Any?>(
        "calendar" to calendar,
        "precision" to precision,
        "year" to year,
        "month" to month,
        "day" to day,
        "leap" to (lunar && leap),
    )

    fun json(): String = JsonW.write(toMap())

    companion object {
        fun of(m: Map<String, Any?>?): DateValue? = m?.let {
            DateValue(it.s("calendar"), it.s("precision"), it.i("year"), it.i("month"), it.i("day"), it.b("leap"))
        }
    }
}

/** Ngày âm hiển thị. */
data class Lunar(val year: Int?, val month: Int, val day: Int, val leap: Boolean) {
    /** "15/6 nhuận" / "7/10". */
    val dm: String get() = "$day/$month" + if (leap) " nhuận" else ""

    companion object {
        /** Lược đồ đầu ra FFI: `{y,m,d,leap}`; lược đồ sao lưu: `{year,month,day,leap}`. */
        fun of(m: Map<String, Any?>?): Lunar? = m?.let {
            val y = it.i("y") ?: it.i("year")
            val mo = it.i("m") ?: it.i("month") ?: return null
            val d = it.i("d") ?: it.i("day") ?: return null
            Lunar(y, mo, d, it.b("leap"))
        }
    }
}

/** Một lần xuất hiện (giỗ, sinh nhật, sự kiện…) do lõi tính (02 §7.2, §7.6, Phụ lục A.6). */
data class Occ(
    val date: LocalDate,
    val lunar: Lunar?,
    val cls: String,
    val classText: String,
    val entityId: String,
    val personIds: List<String>,
    val name: String,
    val key: String,
    val n: Int?,
    val label: String?,
    val headline: String,
    val dateLine: String,
    val days: Int?,
    val daysText: String?,
) {
    val isGio: Boolean get() = cls == "gio" || cls == "tien_thuong"

    companion object {
        fun of(m: Map<String, Any?>): Occ = Occ(
            date = isoDate(m.s("date")),
            lunar = Lunar.of(m.o("lunar")),
            cls = m.s("class"),
            classText = m.s("class_text"),
            entityId = m.s("entity_id"),
            personIds = m.strs("person_ids"),
            name = m.s("name"),
            key = m.s("key"),
            n = m.i("n"),
            label = m.sn("label"),
            headline = m.s("headline"),
            dateLine = m.s("date_line"),
            days = m.i("days"),
            daysText = m.sn("days_text"),
        )
    }
}

/** Một lời nhắc (02 §7.1). Nội dung thông báo chỉ lấy từ [title]/[body] (O-9). */
data class ReminderMsg(
    val id: String,
    val atUnixS: Long,
    val fireLocal: String,
    val type: String,
    val title: String,
    val body: String,
    val entityId: String,
    val cls: String,
    val date: LocalDate,
) {
    companion object {
        fun of(m: Map<String, Any?>): ReminderMsg {
            val occ = m.o("occ") ?: linkedMapOf<String, Any?>()
            return ReminderMsg(
                id = m.s("id"),
                atUnixS = (m["at_unix_s"] as Number).toLong(),
                fireLocal = m.s("fire_local"),
                type = m.s("type"),
                title = m.s("title"),
                body = m.s("body"),
                entityId = occ.s("entity_id"),
                cls = occ.s("class"),
                date = isoDate(occ.s("date")),
            )
        }
    }
}

/** Hồ sơ một người (đọc từ JSON `Person`). */
class Person(val raw: JMap) {
    val id: String get() = raw.s("id")
    val deceased: Boolean get() = raw.s("kind") == "deceased"
    /** Hiển thị luôn viết hoa chữ đầu (YC-2026-07), kể cả dữ liệu cũ. */
    val danhXung: String get() = PeopleRepo.cap(raw.s("danh_xung"))
    val hoTen: String get() = raw.s("ho_ten")
    val tenHieu: String get() = raw.s("ten_hieu")
    val quanHe: String get() = PeopleRepo.cap(raw.s("quan_he"))
    /** "Nam" / "Nữ" / null (không ghi). */
    val gioiTinhText: String? get() = when (raw.sn("gioi_tinh")) { "nam" -> "Nam"; "nu" -> "Nữ"; else -> null }
    val ghiChu: String get() = raw.s("ghi_chu")
    val noiAnTang: String get() = raw.s("noi_an_tang")
    val photoId: String? get() = raw.sn("photo_id")
    val birth: JMap? get() = raw.o("birth")
    val death: JMap? get() = raw.o("death")
    val notice: JMap? get() = death?.o("notice")
    val memorial: JMap? get() = raw.o("memorial")
    val reminders: JMap? get() = raw.o("reminders")

    /** Tên hiển thị = cách gọi + " " + họ tên (02 §7.1). */
    val displayName: String get() = if (danhXung.isBlank()) hoTen else "$danhXung $hoTen"

    /** Chữ để xếp theo tên (chữ cuối của họ tên), rồi cả họ tên (TK-30). */
    val sortKey: String get() = (hoTen.trim().substringAfterLast(' ') + " " + hoTen).lowercase()
}

/** Sự kiện. */
class EventRec(val raw: JMap) {
    val id: String get() = raw.s("id")
    val typeId: String get() = raw.s("type_id")
    val tieuDe: String get() = raw.s("tieu_de")
    val date: DateValue? get() = DateValue.of(raw.o("date"))
    val recurring: Boolean get() = raw.b("recurring")
    val personIds: List<String> get() = raw.strs("person_ids")
    val reminder: JMap? get() = raw.o("reminder")
    val ghiChu: String get() = raw.s("ghi_chu")
}

class EventType(val raw: JMap) {
    val id: String get() = raw.s("id")
    val ten: String get() = raw.s("ten")
    val defaultRecurring: Boolean get() = raw.b("default_recurring")
    val builtin: String? get() = raw.sn("builtin")
}

/** Tóm tắt hiển thị (`person_summary_json`). */
data class Summary(
    val displayName: String,
    val canChi: String?,
    val lunarYear: Int?,
    val tuoiDuong: Int?,
    val tuoiAm: Int?,
    val tuoiMatText: String?,
    val deathLunarText: String?,
) {
    companion object {
        fun of(m: Map<String, Any?>) = Summary(
            m.s("display_name"), m.sn("can_chi"), m.i("lunar_year"), m.i("tuoi_duong"), m.i("tuoi_am"),
            m.sn("tuoi_mat_text"), m.sn("death_lunar_text"),
        )
    }
}

/** Định dạng hiển thị (không tính toán lịch). */
object PFmt {
    private val WD_FULL = listOf("Thứ Hai", "Thứ Ba", "Thứ Tư", "Thứ Năm", "Thứ Sáu", "Thứ Bảy", "Chủ nhật")
    private val WD_SHORT = listOf("T2", "T3", "T4", "T5", "T6", "T7", "CN")

    fun dmy(d: LocalDate): String = "%02d/%02d/%d".format(d.dayOfMonth, d.monthValue, d.year)
    fun dm(d: LocalDate): String = "%02d/%02d".format(d.dayOfMonth, d.monthValue)
    fun weekday(d: LocalDate): String = WD_FULL[d.dayOfWeek.value - 1]
    fun weekdayShort(d: LocalDate): String = WD_SHORT[d.dayOfWeek.value - 1]

    /** "thg 11" — không viết "Th 11" (dễ đọc thành Thứ Bảy, TK-07). */
    fun monthShort(d: LocalDate): String = "thg ${d.monthValue}"

    fun roman(q: Int): String = listOf("I", "II", "III", "IV")[q - 1]

    /** "07/11/2026" từ RFC 3339 UTC, theo giờ của máy. */
    fun dateOfInstant(rfc3339: String, zone: java.time.ZoneId = java.time.ZoneId.systemDefault()): String =
        runCatching { dmy(java.time.Instant.parse(rfc3339).atZone(zone).toLocalDate()) }.getOrDefault(rfc3339)

    /** "Chủ nhật, 08/08/2025". */
    fun full(d: LocalDate): String = "${weekday(d)}, ${dmy(d)}"

    /** Tiêu đề kỳ (08 §3.2, định dạng cố định). */
    fun periodTitle(kind: String, from: LocalDate, to: LocalDate): String = when (kind) {
        "tuan" -> if (from.year == to.year) "Tuần ${dm(from)} – ${dmy(to)}" else "Tuần ${dmy(from)} – ${dmy(to)}"
        "thang" -> "Tháng ${from.monthValue}/${from.year} · ${dm(from)} – ${dm(to)}"
        "quy" -> "Quý ${roman((from.monthValue - 1) / 3 + 1)}/${from.year} · ${dm(from)} – ${dm(to)}"
        else -> "Năm ${from.year} · ${dm(from)} – ${dm(to)}"
    }

    /** "tuần này / tháng này / quý này / năm này". */
    fun periodThis(kind: String): String = when (kind) {
        "tuan" -> "tuần này"
        "thang" -> "tháng này"
        "quy" -> "quý này"
        else -> "năm này"
    }

    /** Mô tả mốc nhắc: "Đúng ngày, 3 ngày trước". */
    fun offsets(list: List<Int>): String =
        if (list.isEmpty()) "Chưa chọn mốc nhắc nào"
        else list.joinToString(", ") { if (it == 0) "Đúng ngày" else "$it ngày trước" }

    /** "dd/mm/yyyy" của DateValue (dương) hoặc "d/m âm lịch (năm)" (âm). */
    fun dateValue(v: DateValue): String = when {
        v.precision == "year" -> "Năm ${v.year}" + if (v.lunar) " (âm lịch)" else ""
        !v.lunar -> "%02d/%02d/%d".format(v.day ?: 0, v.month ?: 0, v.year ?: 0)
        v.precision == "month_day" -> "${v.day}/${v.month}${if (v.leap) " nhuận" else ""} âm lịch (không rõ năm)"
        else -> "${v.day}/${v.month}${if (v.leap) " nhuận" else ""}/${v.year} âm lịch"
    }

    /** Viết hoa chữ đầu, bỏ phần trong ngoặc: "ông nội (bên nội)" → "Ông nội" (08 §3.9). */
    fun relationLine(quanHe: String): String {
        val t = quanHe.replace(Regex("\\s*\\([^)]*\\)"), "").trim()
        return t.replaceFirstChar { it.uppercase() }
    }
}

/** Viết hoa chữ cái đầu (NFC, cắt khoảng trắng đầu), phần còn lại giữ nguyên như người dùng gõ (YC-2026-07, 01 §3.1). */
fun capFirst(s: String): String {
    val t = java.text.Normalizer.normalize(s.trimStart(), java.text.Normalizer.Form.NFC)
    if (t.isEmpty()) return t
    val cp = t.codePointAt(0)
    val n = Character.charCount(cp)
    return String(Character.toChars(Character.toUpperCase(cp))) + t.substring(n)
}
