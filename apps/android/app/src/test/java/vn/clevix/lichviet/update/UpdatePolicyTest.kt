package vn.clevix.lichviet.update

import java.time.LocalDate
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class UpdatePolicyTest {
    private val built = LocalDate.of(2026, 11, 6)

    @Test
    fun remindsOnlyAfter60Days() {
        assertFalse(UpdatePolicy.shouldRemind(built, built, null))
        assertFalse(UpdatePolicy.shouldRemind(built, built.plusDays(60), null))
        assertTrue(UpdatePolicy.shouldRemind(built, built.plusDays(61), null))
    }

    @Test
    fun snoozeHidesFor30Days() {
        val today = built.plusDays(70)
        val until = UpdatePolicy.snoozeUntil(today)
        assertEquals(today.plusDays(30), until)
        assertFalse(UpdatePolicy.shouldRemind(built, today.plusDays(29), until))
        assertTrue(UpdatePolicy.shouldRemind(built, today.plusDays(30), until))
    }

    @Test
    fun badBuildDateNeverReminds() {
        assertNull(UpdatePolicy.parseDate("không phải ngày"))
        assertFalse(UpdatePolicy.shouldRemind(null, built.plusDays(999), null))
        assertEquals(built, UpdatePolicy.parseDate("2026-11-06"))
    }

    @Test
    fun textsMatchPwaWording() {
        assertEquals("Bản quyền © 2026 của Clevix Lab, version 0.1.1", UpdatePolicy.copyright("0.1.1"))
        assertEquals("Phiên bản đang dùng: 0.1.1 (bản dựng 42)", UpdatePolicy.runningVersion("0.1.1", "42"))
        assertEquals("market://details?id=vn.clevix.lichviet", UpdatePolicy.MARKET_URL)
    }
}
