package vn.clevix.lichviet.ui.people

import java.io.ByteArrayInputStream
import java.io.InputStream
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/** S12, MA3N-24-03: đọc tệp sao lưu với bộ nhớ có giới hạn; mật khẩu đếm theo điểm mã sau NFC. */
class BackupReadTest {
    /** Luồng đếm số byte đã đọc (để chứng minh không đọc tệp quá giới hạn). */
    private class Counting(val inner: InputStream) : InputStream() {
        var n = 0L
        override fun read(): Int = inner.read().also { if (it >= 0) n++ }
        override fun read(b: ByteArray, off: Int, len: Int): Int = inner.read(b, off, len).also { if (it > 0) n += it }
    }

    @Test
    fun exactSizeReadsOnce() {
        val data = ByteArray(1000) { it.toByte() }
        assertArrayEquals(data, readBackupBytes(ByteArrayInputStream(data), 1000))
    }

    @Test
    fun overLimitRejectedBeforeReading() {
        val s = Counting(ByteArrayInputStream(ByteArray(10)))
        assertNull(readBackupBytes(s, BACKUP_MAX_BYTES + 1))
        assertEquals(0L, s.n)
    }

    @Test
    fun lengthMismatchRejected() {
        assertNull(readBackupBytes(ByteArrayInputStream(ByteArray(11)), 10)) // dài hơn
        assertNull(readBackupBytes(ByteArrayInputStream(ByteArray(9)), 10)) // ngắn hơn
    }

    @Test
    fun unknownSizeStopsAtLimit() {
        // Luồng vô tận: phải dừng ngay sau giới hạn, không đọc mãi.
        val endless = object : InputStream() {
            var n = 0L
            override fun read(): Int { n++; return 0 }
            override fun read(b: ByteArray, off: Int, len: Int): Int { n += len; return len }
        }
        assertNull(readBackupBytes(endless, -1))
        assertEquals(true, endless.n <= BACKUP_MAX_BYTES + 64 * 1024)
    }

    @Test
    fun passwordLengthCountsCodePointsAfterNfc() {
        assertEquals(4, pwLen("😀😀😀😀")) // 8 đơn vị UTF-16 nhưng 4 điểm mã → không đủ 8
        assertEquals(1, pwLen("é")) // NFD → NFC "é"
        assertEquals(8, pwLen("Hà Đông1"))
    }
}
