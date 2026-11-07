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
import vn.clevix.lichviet.people.JsonW
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
        assertEquals(1_790_000_100L, h.last)
    }

    @Test
    fun clockMovedBackTreatsLastRunAsUnknown() {
        val h = FakeHost(null, sample)
        h.last = 2_000_000_000
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

    /** Với lõi thật: báo thức chạy sau thời điểm nhắc → đăng đúng lời nhắc đó, đặt báo thức kế tiếp; chạy lại không đăng lại. */
    @Test
    fun realCoreChain() {
        val repo = Seed.newRepo()
        Seed.family(repo)
        val zone = Zones.device(ZoneId.of("Asia/Ho_Chi_Minh"))
        val now0 = Seed.TODAY.atStartOfDay(ZoneId.of("Asia/Ho_Chi_Minh")).toEpochSecond()
        val first = parseArr(repo.store.remindersJson(now0, zone, 400u)).first().let { ReminderMsg.of(parseObj(JsonW.write(it))) }
        val h = FakeHost(repo)
        h.last = first.atUnixS - 60
        val s = AlarmChain.run(h, first.atUnixS + 60)
        assertTrue(s.post.any { it.id == first.id })
        assertNotNull(h.scheduled)
        assertTrue(h.scheduled!! > first.atUnixS + 60)
        // O-9 (mặc định): thông báo không có tên người.
        assertFalse(h.posted.any { it.title.contains("Nguyễn") || it.body.contains("Nguyễn") || it.title.contains("Trần") })
        // Thông tin báo tử không bao giờ ở thông báo.
        assertFalse(h.posted.any { it.body.contains("Bạch Mai") })
        h.posted.clear()
        AlarmChain.run(h, first.atUnixS + 60)
        assertTrue(h.posted.isEmpty())
    }
}
