package vn.clevix.lichviet.notify

import android.app.Application
import android.content.Intent
import androidx.test.core.app.ApplicationProvider
import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import vn.clevix.lichviet.MainActivity
import vn.clevix.lichviet.ui.people.cleanShareCache
import vn.clevix.lichviet.ui.people.pwSame

/** BM2-19 (mã mở hồ sơ), BM2-16 (dọn cache/ics), BM2-23 (so mật khẩu sau NFC). */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class OpenTokenTest {
    private val ctx: Application = ApplicationProvider.getApplicationContext()

    private fun open(id: String, token: String?) = Intent(ctx, MainActivity::class.java).apply {
        putExtra(Reminders.EXTRA_OPEN, id)
        if (token != null) putExtra(Reminders.EXTRA_TOKEN, token)
    }

    @Test
    fun openRequiresAppToken() {
        val t = Reminders.openToken(ctx)
        assertEquals(32, t.length)
        assertEquals("mã ổn định giữa các lần gọi", t, Reminders.openToken(ctx))
        assertEquals("p1", Reminders.acceptOpen(ctx, open("p1", t)))
        assertNull("Intent của ứng dụng khác (không mã)", Reminders.acceptOpen(ctx, open("p1", null)))
        assertNull("mã sai", Reminders.acceptOpen(ctx, open("p1", "0".repeat(32))))
        assertNull("mã rỗng", Reminders.acceptOpen(ctx, open("p1", "")))
        assertNull(Reminders.acceptOpen(ctx, Intent(ctx, MainActivity::class.java)))
        assertNull(Reminders.acceptOpen(ctx, null))
    }

    @Test
    fun tokenIsRandomPerInstall() {
        val a = Reminders.openToken(ctx)
        ctx.getSharedPreferences("nhac_viec", 0).edit().clear().commit()
        val b = Reminders.openToken(ctx)
        assertNotEquals(a, b)
        assertNull("mã cũ hết hiệu lực", Reminders.acceptOpen(ctx, open("p1", a)))
    }

    @Test
    fun shareCacheIsCleaned() {
        val dir = File(ctx.cacheDir, "ics").apply { mkdirs() }
        val old = File(dir, "a.ics").apply { writeText("BEGIN:VCALENDAR"); setLastModified(1_000_000L) }
        val fresh = File(dir, "b.ics").apply { writeText("BEGIN:VCALENDAR"); setLastModified(1_090_000L) }
        cleanShareCache(ctx, 30_000L, nowMs = 1_100_000L)
        assertFalse("tệp cũ hơn 30 s bị xóa", old.exists())
        assertTrue("tệp vừa tạo (ứng dụng nhận có thể đang đọc) còn", fresh.exists())
        cleanShareCache(ctx, 0L)
        assertFalse("mở ứng dụng: xóa hết", fresh.exists())
        dir.deleteRecursively()
        cleanShareCache(ctx, 0L) // chưa có thư mục: không ném lỗi
    }

    @Test
    fun passwordsComparedAfterNfc() {
        val nfc = "mật khẩu Hà Đông" // dựng sẵn
        val nfd = java.text.Normalizer.normalize(nfc, java.text.Normalizer.Form.NFD)
        assertNotEquals(nfc, nfd)
        assertTrue(pwSame(nfc, nfd))
        assertFalse(pwSame(nfc, nfc + "1"))
    }
}
