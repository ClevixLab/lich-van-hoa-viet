package vn.clevix.lichviet.notify

import java.time.ZoneId
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import vn.clevix.lichviet.core.remindersJson
import vn.clevix.lichviet.people.DateValue
import vn.clevix.lichviet.people.JMap
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.ReminderMsg
import vn.clevix.lichviet.people.Seed
import vn.clevix.lichviet.people.Zones
import vn.clevix.lichviet.people.parseArr
import vn.clevix.lichviet.people.parseObj

/** Phần "keo" của chuỗi báo thức Android (02 §7.3) — đăng, đặt báo thức kế tiếp, nhớ lần chạy. */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class AlarmChainTest {

    private class FakeHost(val repo: PeopleRepo?, var stepJson: String? = null, var allowed: Boolean = true) : AlarmChain.Host {
        var last: Long? = null
        val posted = ArrayList<ReminderMsg>()
        var scheduled: Long? = null
        var cancelled = 0
        var stepArgs: Pair<Long, Long?>? = null
        override fun lastRun() = last
        override fun setLastRun(unixS: Long) { last = unixS }
        override fun step(nowUnixS: Long, lastRunUnixS: Long?): String {
            stepArgs = nowUnixS to lastRunUnixS
            return stepJson ?: repo!!.store.androidAlarmStepJson(nowUnixS, lastRunUnixS, Zones.device(ZoneId.of("Asia/Ho_Chi_Minh")))
        }
        override fun canPost() = allowed
        override fun post(r: ReminderMsg) { posted.add(r) }
        override fun schedule(atUnixS: Long) { scheduled = atUnixS }
        override fun cancel() { cancelled++ }
    }

    private val sample = """{"post":[{"id":"lvhv:0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e11:2026:k=3","at_unix_s":1790000000,""" +
        """"fire_local":"2026-09-21T07:00","type":"k=3","title":"Sắp đến ngày giỗ","body":"Còn 3 ngày","occ":{"entity_id":"e1",""" +
        """"class":"gio","date":"2026-09-24","key":"2026"}}],"dropped_late":1,"next_at_unix_s":1790500000,"next_fire_local":"2026-09-27T07:00"}"""

    @Test
    fun parsesStepAndSchedulesNext() {
        val h = FakeHost(null, sample)
        val s = AlarmChain.run(h, 1_790_000_100)
        assertEquals(1, s.post.size)
        assertEquals(1, s.droppedLate)
        assertEquals("Sắp đến ngày giỗ", h.posted.single().title)
        assertEquals("e1", h.posted.single().entityId)
        assertEquals(1_790_500_000L, h.scheduled)
        assertEquals(1_790_000_100L, h.last)
    }

    @Test
    fun noFutureReminderCancelsAlarm() {
        val h = FakeHost(null, """{"post":[],"dropped_late":0,"next_at_unix_s":null,"next_fire_local":null}""")
        AlarmChain.run(h, 1_790_000_000)
        assertNull(h.scheduled)
        assertEquals(1, h.cancelled)
    }

    @Test
    fun withoutPermissionNothingPostedButChainContinues() {
        val h = FakeHost(null, sample, allowed = false)
        AlarmChain.run(h, 1_790_000_100)
        assertTrue(h.posted.isEmpty())
        assertEquals(1_790_500_000L, h.scheduled)
        // MA3N-24-06: không "tiêu" lời nhắc khi chưa đăng được — lần chạy chưa được ghi nhận.
        assertNull(h.last)
    }

    @Test
    fun clockMovedBackUsesNowNotNull() {
        // MA3N-24-05: last_run ở tương lai (đồng hồ lùi) → truyền now, không truyền null (null = đăng lại 24 giờ trước).
        val h = FakeHost(null, sample)
        h.last = 2_000_000_000
        AlarmChain.run(h, 1_790_000_100)
        assertEquals(1_790_000_100L, h.stepArgs!!.second)
    }

    @Test
    fun firstRunPassesNull() {
        val h = FakeHost(null, sample)
        AlarmChain.run(h, 1_790_000_100)
        assertNull(h.stepArgs!!.second)
    }

    @Test
    fun notificationIdIsStable() {
        val a = AlarmChain.notificationId("lvhv:abc:2026:k=0")
        assertEquals(a, AlarmChain.notificationId("lvhv:abc:2026:k=0"))
        assertTrue(a >= 0)
        assertFalse(a == AlarmChain.notificationId("lvhv:abc:2026:k=3"))
    }

    private val vnZone = ZoneId.of("Asia/Ho_Chi_Minh")
    private fun reminders(repo: PeopleRepo, from: Long, names: Boolean): List<ReminderMsg> =
        parseArr(remindersJson(repo.datasetJson(), from, Zones.device(vnZone), 400u, names)).map { ReminderMsg.of(it as JMap) }

    /** Lõi thật: báo thức chạy sau thời điểm nhắc → đăng đúng lời nhắc đó, đặt báo thức đúng lời nhắc kế tiếp; chạy lại không đăng lại. */
    @Test
    fun realCoreChain() {
        val repo = Seed.newRepo()
        Seed.family(repo)
        val now0 = Seed.TODAY.atStartOfDay(vnZone).toEpochSecond()
        val all = reminders(repo, now0, names = false)
        val first = all.first()
        val due = all.filter { it.atUnixS == first.atUnixS }
        val second = all.first { it.atUnixS > first.atUnixS }
        val h = FakeHost(repo)
        h.last = first.atUnixS - 60
        val s = AlarmChain.run(h, first.atUnixS + 60)
        // Đăng đúng các lời nhắc đến hạn trong (last, now], không hơn không kém.
        assertEquals(due.map { it.id }.toSet(), s.post.map { it.id }.toSet())
        assertEquals(due.map { it.id }.toSet(), h.posted.map { it.id }.toSet())
        // Báo thức kế tiếp = lời nhắc kế tiếp của lõi.
        assertEquals(second.atUnixS, h.scheduled)
        // O-9 (mặc định): tiêu đề và nội dung không có tên người nào trong sổ.
        val names = repo.persons().flatMap { listOf(it.hoTen, it.displayName) }
        assertFalse(h.posted.any { r -> names.any { r.title.contains(it) || r.body.contains(it) } })
        h.posted.clear()
        AlarmChain.run(h, first.atUnixS + 60)
        assertTrue(h.posted.isEmpty())
    }

    /** R-PRIV-2: thông tin báo tử không có trong lời nhắc, kể cả khi bật hiện tên (chỗ duy nhất có thể lộ). */
    @Test
    fun noticeNeverInRemindersEvenWithNames() {
        val repo = Seed.newRepo()
        Seed.family(repo)
        val now0 = Seed.TODAY.atStartOfDay(vnZone).toEpochSecond()
        val json = remindersJson(repo.datasetJson(), now0, Zones.device(vnZone), 400u, true)
        for (secret in listOf("Bạch Mai", "Phạm Văn Hải", "Tuổi cao", "báo họ hàng")) assertFalse(secret, json.contains(secret))
        // Có tên thật khi bật (để chắc phép kiểm trên không rỗng).
        assertTrue(json.contains("Phạm Thị Dung"))
    }

    /** B4 qua chuỗi báo thức: người đang sống được cập nhật đã mất → bước kế tiếp không còn lời nhắc sinh nhật của người đó. */
    @Test
    fun b4MarkDeceasedRemovesBirthdayFromChain() {
        val repo = Seed.newRepo()
        val ids = Seed.family(repo)
        val now0 = Seed.TODAY.atStartOfDay(vnZone).toEpochSecond()
        val bday = reminders(repo, now0, names = true).first { it.entityId == ids.m && it.cls == "sinh_nhat" }
        val h = FakeHost(repo)
        h.last = bday.atUnixS - 60
        // Trước khi cập nhật: bước chạy ngay sau thời điểm nhắc có lời nhắc này.
        val before = parse(repo.store.androidAlarmStepJson(bday.atUnixS + 60, bday.atUnixS - 60, Zones.device(vnZone)))
        assertTrue(before.post.any { it.id == bday.id })
        repo.markDeceased(ids.m, DateValue("solar", "day", 2026, 11, 1), null, Seed.TODAY)
        val s = AlarmChain.run(h, bday.atUnixS + 60)
        assertFalse(s.post.any { it.id == bday.id })
        assertFalse(reminders(repo, now0, names = true).any { it.entityId == ids.m && it.cls.startsWith("sinh_nhat") })
        // Chuỗi vẫn tiếp tục.
        assertNotNull(h.scheduled)
    }

    private fun parse(json: String) = AlarmChain.parse(json)
}
