package vn.clevix.lichviet.people

import java.time.Duration
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import vn.clevix.lichviet.core.BackupImportMode
import vn.clevix.lichviet.core.BackupPreview
import vn.clevix.lichviet.core.DeviceZone
import vn.clevix.lichviet.core.IcsExport
import vn.clevix.lichviet.core.LvhvDataException
import vn.clevix.lichviet.core.LvhvStore
import vn.clevix.lichviet.core.VerifiedBackup
import vn.clevix.lichviet.core.ZoneTransition
import vn.clevix.lichviet.core.deriveBirthJson
import vn.clevix.lichviet.core.derivePersonJson
import vn.clevix.lichviet.core.icsExport
import vn.clevix.lichviet.core.newUuid
import vn.clevix.lichviet.core.openBackup
import vn.clevix.lichviet.core.periodBounds
import vn.clevix.lichviet.core.periodNext
import vn.clevix.lichviet.core.periodPrev
import vn.clevix.lichviet.core.personSummaryJson
import vn.clevix.lichviet.core.remindersJson
import vn.clevix.lichviet.core.saveDeathJson
import vn.clevix.lichviet.core.yearCanChi

/** Thông điệp tiếng Việt của một lỗi từ lõi (LvhvDataException mang sẵn câu hiển thị được). */
fun errText(e: Throwable): String = when (e) {
    is LvhvDataException -> e.message ?: "Có lỗi khi xử lý dữ liệu."
    else -> e.message ?: e.toString()
}

/** Múi giờ thiết bị cho lõi (DeviceZone): độ lệch hiện tại + các lần chuyển giờ trong 401 ngày tới. */
object Zones {
    fun device(zone: ZoneId = ZoneId.systemDefault(), now: Instant = Instant.now()): DeviceZone {
        val rules = zone.rules
        val list = ArrayList<ZoneTransition>()
        val end = now.plus(Duration.ofDays(401))
        var t = rules.nextTransition(now)
        while (t != null && t.instant.isBefore(end) && list.size < 64) {
            list.add(ZoneTransition(t.instant.epochSecond, t.offsetAfter.totalSeconds / 60))
            t = rules.nextTransition(t.instant)
        }
        return DeviceZone(rules.getOffset(now).totalSeconds / 60, list)
    }
}

/**
 * Lớp truy cập dữ liệu người thân. Bọc [LvhvStore] của lõi; mọi tính toán ở lõi.
 * Hàm có thể ném [LvhvDataException] (câu tiếng Việt, xem [errText]).
 */
class PeopleRepo(val store: LvhvStore) {

    // ---------- Đọc ----------

    fun settings(): JMap = parseObj(store.settingsJson())
    fun putSettings(m: Map<String, Any?>) = store.putSettingsJson(JsonW.write(m))

    fun persons(): List<Person> = parseArr(store.personsJson()).map { Person(it as JMap) }
    fun person(id: String): Person? = try {
        Person(parseObj(store.personJson(id)))
    } catch (_: LvhvDataException.NotFound) {
        null
    }

    fun events(): List<EventRec> = parseArr(store.eventsJson()).map { EventRec(it as JMap) }
    fun eventTypes(): List<EventType> = parseArr(store.eventTypesJson()).map { EventType(it as JMap) }

    fun photo(id: String): ByteArray? = runCatching { store.photoBytes(id) }.getOrNull()

    fun upcoming(today: LocalDate, days: Int): List<Occ> =
        parseArr(store.upcomingJson(today.solar(), days.toUInt())).map { Occ.of(it as JMap) }

    /** Danh sách theo kỳ (02 §7.6, R13): kind ∈ tuan/thang/quy/nam, filter ∈ tat_ca/da_mat/dang_song. */
    fun period(kind: String, anchor: LocalDate, filter: String): Triple<LocalDate, LocalDate, List<Occ>> {
        val o = parseObj(store.eventsInPeriodJson(kind, anchor.solar(), filter))
        return Triple(isoDate(o.s("from")), isoDate(o.s("to")), o.objs("items").map { Occ.of(it) })
    }

    fun summary(p: Person, today: LocalDate): Summary =
        Summary.of(parseObj(personSummaryJson(JsonW.write(p.raw), today.solar())))

    fun lockScreenNames(): Boolean = store.lockScreenNames()
    fun setLockScreenNames(v: Boolean) = store.setLockScreenNames(v)
    fun backupDue(): Boolean = store.backupReminderDue()
    fun datasetJson(): String = store.datasetJson()

    /** Lời nhắc có ngày giờ máy = [today] (thẻ nhắc trong ứng dụng; trong ứng dụng **có** tên, 08 §5.5). */
    fun todayReminders(today: LocalDate, zone: ZoneId = ZoneId.systemDefault()): List<ReminderMsg> {
        val start = today.atStartOfDay(zone).toInstant().epochSecond - 1
        val z = Zones.device(zone, Instant.ofEpochSecond(start))
        val json = remindersJson(store.datasetJson(), start, z, 2u, true)
        val day = today.toString()
        return parseArr(json).map { ReminderMsg.of(it as JMap) }.filter { it.fireLocal.startsWith(day) }
    }

    // ---------- Ghi ----------

    fun savePerson(p: Map<String, Any?>): Person = Person(parseObj(store.savePersonJson(JsonW.write(p))))

    fun markDeceased(id: String, death: DateValue, notice: Map<String, Any?>?, today: LocalDate): Person =
        Person(parseObj(store.markDeceasedJson(id, death.json(), notice?.let { JsonW.write(it) }, today.solar())))

    /** Xóa hồ sơ; xóa luôn sự kiện không còn ai sau khi gỡ (08 §3.10). */
    fun deletePerson(id: String) {
        val mine = events().filter { id in it.personIds }
        store.deletePerson(id)
        for (e in mine) if (e.personIds.all { it == id }) runCatching { store.deleteEvent(e.id) }
    }

    fun saveEvent(e: Map<String, Any?>): EventRec = EventRec(parseObj(store.saveEventJson(JsonW.write(e))))
    fun deleteEvent(id: String) = store.deleteEvent(id)
    fun saveEventType(t: Map<String, Any?>): EventType = EventType(parseObj(store.saveEventTypeJson(JsonW.write(t))))

    /** Ghi ảnh đã mã hóa lại (JPEG, không siêu dữ liệu); trả id. */
    fun addPhoto(img: EncodedImage): String {
        val id = newUuid()
        store.addPhoto(id, "image/jpeg", img.width.toUInt(), img.height.toUInt(), img.bytes)
        return id
    }

    // ---------- Sao lưu, .ics ----------

    fun ics(today: LocalDate, now: Instant = Instant.now()): IcsExport =
        icsExport(store.datasetJson(), today.solar(), now.epochSecond, null)

    fun exportBackup(password: String, appVersion: String): ByteArray =
        store.exportBackup(password, appVersion, "android")

    fun applyImport(b: VerifiedBackup, replace: Boolean) =
        store.applyImport(b, if (replace) BackupImportMode.REPLACE else BackupImportMode.MERGE)

    companion object {
        fun uuid(): String = newUuid()

        /**
         * Lõi đã nhận trường `gioi_tinh` (YC-2026-07, 01 §3.1) chưa. Lược đồ của lõi đóng (trường lạ → lỗi V1), nên giao
         * diện chỉ hiện ô Giới tính khi lõi kiểm hợp lệ một hồ sơ mẫu có trường này.
         */
        val supportsGender: Boolean by lazy { runCatching { derivePerson(genderProbe(true)) }.isSuccess }

        /** Hồ sơ mẫu để dò khả năng của lõi (kiểm thử khẳng định bản không có giới tính luôn hợp lệ). */
        internal fun genderProbe(withGender: Boolean): JMap = linkedMapOf<String, Any?>(
            "id" to newUuid(), "kind" to "living", "danh_xung" to "", "ho_ten" to "Mẫu", "ten_hieu" to "",
            "quan_he" to "", "birth" to null, "death" to null, "noi_an_tang" to "",
            "photo_id" to null, "memorial" to null,
            "reminders" to linkedMapOf<String, Any?>("gio" to null, "sinh_nhat" to null, "sinh_nhat_am" to null),
            "sinh_nhat_am" to false, "feb29_rule" to "feb28", "ghi_chu" to "",
        ).also { if (withGender) it["gioi_tinh"] = "nam" }

        /** `save_death` (02 §1): ngày mất → {lunar, solar, lunar_source, method}. */
        fun deriveDeath(v: DateValue): JMap = parseObj(saveDeathJson(v.json()))

        /** Ngày sinh (02 §4.1) → {lunar_year, lunar_year_approx, can_chi, lunar, solar}. */
        fun deriveBirth(v: DateValue): JMap = parseObj(deriveBirthJson(v.json()))

        fun derivePerson(p: Map<String, Any?>): JMap = parseObj(derivePersonJson(JsonW.write(p)))

        fun summaryOf(p: Map<String, Any?>, today: LocalDate): Summary =
            Summary.of(parseObj(personSummaryJson(JsonW.write(p), today.solar())))

        fun canChi(year: Int): String = runCatching { yearCanChi(year) }.getOrDefault("")

        fun bounds(kind: String, anchor: LocalDate): Pair<LocalDate, LocalDate> =
            periodBounds(kind, anchor.solar()).let { it.from.local() to it.to.local() }

        fun prev(kind: String, anchor: LocalDate): LocalDate = periodPrev(kind, anchor.solar()).from.local()
        fun next(kind: String, anchor: LocalDate): LocalDate = periodNext(kind, anchor.solar()).from.local()

        /** Tháng nhuận của năm âm [year] (0 = không có), dò bằng `save_death` của lõi. */
        private val leapCache = HashMap<Int, Int>()
        fun leapMonth(year: Int): Int = synchronized(leapCache) {
            leapCache.getOrPut(year) {
                (1..12).firstOrNull { m ->
                    runCatching { deriveDeath(DateValue("lunar", "day", year, m, 1, true)) }.isSuccess
                } ?: 0
            }
        }

        fun openBackupFile(bytes: ByteArray, password: String): Pair<VerifiedBackup, BackupPreview> {
            val b = openBackup(bytes, password)
            return b to b.preview()
        }

        /** Quy tắc nhắc mới (01 §3.6) theo cài đặt chung. */
        fun newRule(settings: Map<String, Any?>?, tienThuong: Boolean = false): JMap = linkedMapOf<String, Any?>(
            "id" to newUuid(),
            "enabled" to true,
            "offsets_days" to ArrayList((settings?.list("default_offsets_days") ?: listOf(0L, 3L)).map { (it as Number).toInt() }),
            "time_local" to (settings?.sn("default_time_local") ?: "07:00"),
            "tien_thuong" to tienThuong,
        )
    }
}
