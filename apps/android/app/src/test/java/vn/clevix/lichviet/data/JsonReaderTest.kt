package vn.clevix.lichviet.data

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Kiểm thử JVM thuần (không cần thư viện Rust). */
class JsonReaderTest {
    private val sample = """{"solar":{"y":2026,"m":11,"d":5,"weekday":4,"jdn":2461350},"lunar":{"y":2026,"m":9,"d":27,"leap":false,"month_len":30},"can_chi":{"year":"Bính Ngọ","month":"Mậu Tuất","day":"Quý Mùi","hour_ty":"Nhâm Tý"},"tiet_khi":"Sương Giáng","nap_am":"Dương Liễu Mộc","truc":"Thu","tu":"Tỉnh","than":{"name":"Chu Tước","hoang_dao":false},"gio_hoang_dao":[{"chi":"Dần","from":3,"to":5}],"ngoc_hap":{"sao_tot":["Thiên Quan"],"sao_xau":[],"nen":[],"ky":["Cưới hỏi"],"xung_tuoi":["Đinh Sửu"],"score":-6}}"""

    @Test
    fun parsesDayInfo() {
        val d = Lich.parseDayInfo(sample)
        assertEquals(27, d.lunarD)
        assertEquals("Quý Mùi", d.ccDay)
        assertEquals(false, d.hoangDao)
        assertEquals(listOf(Gio("Dần", 3, 5)), d.gio)
        assertEquals(-6, d.ngocHap.score)
        assertEquals(listOf("Cưới hỏi"), d.ngocHap.ky)
    }

    @Test
    fun escapes() {
        val v = JsonReader.parse("""["a\"b\\c\n", "à", -1.5e2, null, true]""").arr()
        assertEquals("a\"b\\c\n", v[0])
        assertEquals("à", v[1])
        assertEquals(-150.0, v[2])
        assertEquals(null, v[3])
        assertEquals(true, v[4])
    }

    @Test
    fun parseDmy() {
        assertEquals(java.time.LocalDate.of(2026, 2, 28), Fmt.parseDmy("28/02/2026"))
        assertEquals(null, Fmt.parseDmy("30/02/2026"))
        assertTrue(Fmt.parseDmy("x") == null)
    }
}
