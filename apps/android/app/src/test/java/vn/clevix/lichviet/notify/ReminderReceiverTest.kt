package vn.clevix.lichviet.notify

import android.Manifest
import android.app.AlarmManager
import android.app.Application
import android.content.Intent
import android.os.Looper
import androidx.test.core.app.ApplicationProvider
import java.io.File
import java.time.ZoneId
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.Shadows.shadowOf
import org.robolectric.annotation.Config
import vn.clevix.lichviet.people.Seed
import vn.clevix.lichviet.people.Zones
import vn.clevix.lichviet.people.parseObj

/**
 * ReminderReceiver và các lối vào đặt lại chuỗi (02 §7.3, MA3N-24-07): đủ năm action → đúng một báo thức tại lời nhắc kế
 * tiếp mà lõi tính; bước lỗi → báo thức thử lại sau 6 giờ (MA3N-24-04).
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class ReminderReceiverTest {
    private val app: Application = ApplicationProvider.getApplicationContext()
    private val am get() = shadowOf(app.getSystemService(AlarmManager::class.java))

    @Before
    fun setUp() {
        PeopleStore.reset()
        shadowOf(app).grantPermissions(Manifest.permission.POST_NOTIFICATIONS)
    }

    @After
    fun tearDown() = PeopleStore.reset()

    private fun expectedNext(): Long {
        val repo = PeopleStore.get(app)!!
        val now = System.currentTimeMillis() / 1000
        val o = parseObj(repo.store.androidAlarmStepJson(now, now, Zones.device(ZoneId.systemDefault())))
        return (o["next_at_unix_s"] as Number).toLong()
    }

    private fun send(action: String) {
        app.sendBroadcast(Intent(app, ReminderReceiver::class.java).setAction(action))
        shadowOf(Looper.getMainLooper()).idle()
        Reminders.awaitIdle()
        shadowOf(Looper.getMainLooper()).idle()
    }

    @Test
    fun everyActionArmsExactlyOneAlarmAtNextReminder() {
        Seed.family(PeopleStore.get(app)!!)
        val actions = listOf(
            Reminders.ACTION_ALARM,
            Intent.ACTION_BOOT_COMPLETED,
            Intent.ACTION_MY_PACKAGE_REPLACED,
            Intent.ACTION_TIME_CHANGED,
            Intent.ACTION_TIMEZONE_CHANGED,
        )
        for (a in actions) {
            app.getSystemService(AlarmManager::class.java).cancel(
                android.app.PendingIntent.getBroadcast(
                    app, 0, Intent(app, ReminderReceiver::class.java).setAction(Reminders.ACTION_ALARM),
                    android.app.PendingIntent.FLAG_UPDATE_CURRENT or android.app.PendingIntent.FLAG_IMMUTABLE,
                ),
            )
            assertNull("$a: còn báo thức cũ", am.nextScheduledAlarm)
            send(a)
            val alarms = am.scheduledAlarms
            assertEquals("$a: đúng một báo thức", 1, alarms.size)
            assertEquals("$a: loại RTC_WAKEUP", AlarmManager.RTC_WAKEUP, alarms[0].type)
            assertEquals("$a: thời điểm = lời nhắc kế tiếp của lõi", expectedNext() * 1000, alarms[0].triggerAtMs)
        }
    }

    @Test
    fun unrelatedActionDoesNothing() {
        Seed.family(PeopleStore.get(app)!!)
        send("vn.clevix.lichviet.action.KHAC")
        assertNull(am.nextScheduledAlarm)
    }

    @Test
    fun noPeopleStoreNoAlarm() {
        // Người chưa từng mở mô-đun: không tạo kho, không đặt báo thức.
        File(app.filesDir, "lvhv.sqlite").delete()
        send(Intent.ACTION_BOOT_COMPLETED)
        assertNull(am.nextScheduledAlarm)
        assertFalse(File(app.filesDir, "lvhv.sqlite").exists())
    }

    @Test
    fun failingStepSchedulesRetryAndRecordsError() {
        // Tệp kho hỏng → không mở được → bước lỗi: phải đặt báo thức thử lại, không dừng lặng lẽ.
        File(app.filesDir, "lvhv.sqlite").writeText("không phải SQLite")
        val now = 1_790_000_000L
        assertFalse(Reminders.rearm(app, now))
        val a = am.nextScheduledAlarm
        assertNotNull(a)
        assertEquals((now + Reminders.RETRY_S) * 1000, a!!.triggerAtMs)
        assertEquals(now, Reminders.lastError(app))
        // Kho sửa được → bước thành công xóa dấu lỗi.
        File(app.filesDir, "lvhv.sqlite").delete()
        PeopleStore.reset()
        Seed.family(PeopleStore.get(app)!!)
        assertTrue(Reminders.rearm(app))
        assertNull(Reminders.lastError(app))
    }
}
