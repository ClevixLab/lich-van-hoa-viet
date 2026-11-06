package vn.clevix.lichviet.data

import java.time.LocalDate
import vn.clevix.lichviet.core.LunarDay
import vn.clevix.lichviet.core.LvhvException
import vn.clevix.lichviet.core.SolarDay
import vn.clevix.lichviet.core.activitiesJson
import vn.clevix.lichviet.core.cadaoAttribution
import vn.clevix.lichviet.core.cadaoFor
import vn.clevix.lichviet.core.cadaoSourcesBlocksJson
import vn.clevix.lichviet.core.cadaoCount
import vn.clevix.lichviet.core.lunarMethod
import vn.clevix.lichviet.core.ngochapSourcesBlocksJson
import vn.clevix.lichviet.core.dayInfoJson
import vn.clevix.lichviet.core.dayRating
import vn.clevix.lichviet.core.findGoodDaysJson
import vn.clevix.lichviet.core.lunarToSolar
import vn.clevix.lichviet.core.monthLunarDays
import vn.clevix.lichviet.core.solarToLunar
import vn.clevix.lichviet.core.yearCanChi

// ---------- Mô hình dữ liệu (khớp pwa/src/types.ts) ----------

data class Gio(val chi: String, val from: Int, val to: Int)

data class NgocHap(
    val saoTot: List<String>,
    val saoXau: List<String>,
    val nen: List<String>,
    val ky: List<String>,
    val xungTuoi: List<String>,
    val score: Int,
)

data class DayInfo(
    val date: LocalDate,
    val weekday: Int,
    val jdn: Long,
    val lunarY: Int,
    val lunarM: Int,
    val lunarD: Int,
    val leap: Boolean,
    val monthLen: Int,
    val ccYear: String,
    val ccMonth: String,
    val ccDay: String,
    val ccHourTy: String,
    val tietKhi: String,
    val napAm: String,
    val truc: String,
    val tu: String,
    val thanName: String,
    val hoangDao: Boolean,
    val gio: List<Gio>,
    val ngocHap: NgocHap,
)

data class Activity(val id: String, val name: String)

data class GoodDay(val date: LocalDate, val score: Int, val reasons: List<String>)

data class Cadao(val text: String, val lines: List<String>, val type: String, val sources: List<String>)

data class Rating(val label: String, val level: Int)

/** Khối văn bản nguồn: kind = "h" | "p" | "li". */
data class TextBlock(val kind: String, val text: String)

/** Kết quả gọi lõi: giá trị hoặc thông điệp lỗi tiếng Việt. */
sealed class Res<out T> {
    data class Ok<T>(val value: T) : Res<T>()
    data class Err(val message: String) : Res<Nothing>()
}

// ---------- Hằng và tiện ích ----------

object Fmt {
    val WEEKDAY = listOf("Chủ nhật", "Thứ Hai", "Thứ Ba", "Thứ Tư", "Thứ Năm", "Thứ Sáu", "Thứ Bảy")
    val WEEKDAY_SHORT = listOf("CN", "T2", "T3", "T4", "T5", "T6", "T7")
    val LUNAR_MONTH = listOf("", "Giêng", "Hai", "Ba", "Tư", "Năm", "Sáu", "Bảy", "Tám", "Chín", "Mười", "Mười Một", "Chạp")

    /** 0 = Chủ nhật … 6 = Thứ Bảy (như lõi). */
    fun weekday(d: LocalDate): Int = d.dayOfWeek.value % 7

    fun dmy(d: LocalDate): String = "%02d/%02d/%d".format(d.dayOfMonth, d.monthValue, d.year)

    /** Đọc "dd/mm/yyyy" (chấp nhận cả '-' và '.'); null nếu sai. */
    fun parseDmy(s: String): LocalDate? {
        val p = s.trim().split('/', '-', '.')
        if (p.size != 3) return null
        val d = p[0].toIntOrNull() ?: return null
        val m = p[1].toIntOrNull() ?: return null
        val y = p[2].toIntOrNull() ?: return null
        return try { LocalDate.of(y, m, d) } catch (_: Exception) { null }
    }

    fun hourRange(g: Gio): String = "${g.from}–${g.to} giờ"

    fun lunarDayLabel(d: Int): String = if (d <= 10) "Mùng $d" else "Ngày $d"
}

/** Lớp truy cập lõi lịch. Mọi màn hình chỉ gọi lõi qua đối tượng này. */
object Lich {
    const val MIN_YEAR = 2000
    const val MAX_YEAR = 3000
    val MIN_DATE: LocalDate = LocalDate.of(MIN_YEAR, 1, 1)
    val MAX_DATE: LocalDate = LocalDate.of(MAX_YEAR, 12, 31)

    /** Số trang (ngày) của lịch: 2000-01-01 … 3000-12-31. */
    val PAGE_COUNT: Int = (MAX_DATE.toEpochDay() - MIN_DATE.toEpochDay() + 1).toInt()

    fun pageOf(d: LocalDate): Int = (clamp(d).toEpochDay() - MIN_DATE.toEpochDay()).toInt()
    fun dateOfPage(p: Int): LocalDate = MIN_DATE.plusDays(p.toLong())

    fun clamp(d: LocalDate): LocalDate = when {
        d.isBefore(MIN_DATE) -> MIN_DATE
        d.isAfter(MAX_DATE) -> MAX_DATE
        else -> d
    }

    fun inRange(d: LocalDate): Boolean = !d.isBefore(MIN_DATE) && !d.isAfter(MAX_DATE)

    private fun SolarDay.toLocal(): LocalDate = LocalDate.of(y, m, d)
    private fun LocalDate.toSolar(): SolarDay = SolarDay(year, monthValue, dayOfMonth)

    private val cache = object : LinkedHashMap<LocalDate, DayInfo>(64, 0.75f, true) {
        override fun removeEldestEntry(eldest: MutableMap.MutableEntry<LocalDate, DayInfo>?): Boolean = size > 64
    }

    fun dayInfo(d: LocalDate): DayInfo = synchronized(cache) {
        cache.getOrPut(d) { parseDayInfo(dayInfoJson(d.year, d.monthValue, d.dayOfMonth)) }
    }

    internal fun parseDayInfo(json: String): DayInfo {
        val o = JsonReader.parse(json).obj()
        val s = o["solar"].obj()
        val l = o["lunar"].obj()
        val cc = o["can_chi"].obj()
        val th = o["than"].obj()
        val nh = o["ngoc_hap"].obj()
        return DayInfo(
            date = LocalDate.of(s.int("y"), s.int("m"), s.int("d")),
            weekday = s.int("weekday"),
            jdn = (s["jdn"] as Number).toLong(),
            lunarY = l.int("y"),
            lunarM = l.int("m"),
            lunarD = l.int("d"),
            leap = l.bool("leap"),
            monthLen = l.int("month_len"),
            ccYear = cc.str("year"),
            ccMonth = cc.str("month"),
            ccDay = cc.str("day"),
            ccHourTy = cc.str("hour_ty"),
            tietKhi = o.str("tiet_khi"),
            napAm = o.str("nap_am"),
            truc = o.str("truc"),
            tu = o.str("tu"),
            thanName = th.str("name"),
            hoangDao = th.bool("hoang_dao"),
            gio = o["gio_hoang_dao"].arr().map { val g = it.obj(); Gio(g.str("chi"), g.int("from"), g.int("to")) },
            ngocHap = NgocHap(
                saoTot = nh.strList("sao_tot"),
                saoXau = nh.strList("sao_xau"),
                nen = nh.strList("nen"),
                ky = nh.strList("ky"),
                xungTuoi = nh.strList("xung_tuoi"),
                score = nh.int("score"),
            ),
        )
    }

    fun rating(score: Int): Rating = dayRating(score).let { Rating(it.label, it.level.toInt()) }

    fun lunar(d: LocalDate): LunarDay = solarToLunar(d.year, d.monthValue, d.dayOfMonth)

    /** Âm lịch của cả tháng dương (phần tử i là ngày i+1). */
    fun monthLunar(y: Int, m: Int): List<LunarDay> = monthLunarDays(y, m)

    fun toSolar(ly: Int, lm: Int, ld: Int, leap: Boolean): LocalDate? =
        lunarToSolar(ly, lm, ld, leap)?.toLocal()

    fun yearCanChiOf(y: Int): String = yearCanChi(y)

    val activities: List<Activity> by lazy {
        JsonReader.parse(activitiesJson()).arr().map { val a = it.obj(); Activity(a.str("id"), a.str("name")) }
    }

    /** [birthLunarYear]: năm ÂM lịch của ngày sinh (năm tuổi), không phải năm dương. */
    fun findGoodDays(activity: String, from: LocalDate, to: LocalDate, birthLunarYear: Int?): Res<List<GoodDay>> = try {
        val list = JsonReader.parse(findGoodDaysJson(activity, from.toSolar(), to.toSolar(), birthLunarYear)).arr().map {
            val g = it.obj()
            val s = g["solar"].obj()
            GoodDay(LocalDate.of(s.int("y"), s.int("m"), s.int("d")), g.int("score"), g.strList("reasons"))
        }
        Res.Ok(list)
    } catch (e: LvhvException) {
        Res.Err(e.message ?: "Không tìm được ngày.")
    }

    fun cadao(d: LocalDate): Cadao? {
        val v = JsonReader.parse(cadaoFor(d.year, d.monthValue, d.dayOfMonth)) ?: return null
        val o = v.obj()
        val type = o.str("type").replaceFirstChar { it.uppercase() }
        val lines = o.strList("lines").ifEmpty { listOf(o.str("text")) }
        val sources = o["sources"].arr().map { it.obj().str("title") }
        return Cadao(o.str("text"), lines, type, sources)
    }

    val attribution: String by lazy { cadaoAttribution() }
    val cadaoTotal: Int by lazy { cadaoCount().toInt() }
    val method: String by lazy { lunarMethod() }

    private fun blocks(json: String): List<TextBlock> =
        JsonReader.parse(json).arr().map { val b = it.obj(); TextBlock(b.str("kind"), b.str("text")) }

    val cadaoSources: List<TextBlock> by lazy { blocks(cadaoSourcesBlocksJson()) }
    val ruleSources: List<TextBlock> by lazy { blocks(ngochapSourcesBlocksJson()) }
}
