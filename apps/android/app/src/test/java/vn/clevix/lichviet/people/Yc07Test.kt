package vn.clevix.lichviet.people

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import vn.clevix.lichviet.ui.people.DANH_XUNG_DANG_SONG
import vn.clevix.lichviet.ui.people.DANH_XUNG_DA_MAT

/** YC-2026-07: viết hoa cách gọi / quan hệ, gợi ý theo người sống/mất, dò trường giới tính của lõi. */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class Yc07Test {
    @Test
    fun capitaliseFirstLetterOnly() {
        assertEquals("Ông", capFirst("ông"))
        assertEquals("Ông nội (bên nội)", capFirst("ông nội (bên nội)"))
        assertEquals("Bà Cô Tổ họ", capFirst("bà Cô Tổ họ"))
        assertEquals("Ông", capFirst(java.text.Normalizer.normalize("ông", java.text.Normalizer.Form.NFD)))
        assertEquals("", capFirst("  "))
    }

    @Test
    fun suggestionsPerKind() {
        assertEquals(
            listOf(
                "Ông", "Bà", "Cụ", "Kỵ", "Bác", "Cô", "Chú", "Dì", "Cậu", "Mợ", "Thím", "Anh", "Chị", "Em", "Cháu", "Con", "Bạn",
                "Bố", "Mẹ", "Ông Tổ họ", "Bà Cô Tổ họ", "Bé đỏ",
            ),
            DANH_XUNG_DA_MAT,
        )
        for (x in listOf("Cụ", "Kỵ", "Ông Tổ họ", "Bà Cô Tổ họ", "Bé đỏ")) assertFalse(x, x in DANH_XUNG_DANG_SONG)
        assertEquals(DANH_XUNG_DA_MAT.size - 5, DANH_XUNG_DANG_SONG.size)
        assertTrue((DANH_XUNG_DA_MAT + DANH_XUNG_DANG_SONG).all { it == capFirst(it) })
    }

    @Test
    fun genderProbeIsOtherwiseValid() {
        // Hồ sơ mẫu không có giới tính phải hợp lệ — để "lõi chưa nhận gioi_tinh" không bị nhầm với "mẫu sai".
        PeopleRepo.derivePerson(PeopleRepo.genderProbe(false))
    }

    @Test
    fun namesAreCapitalisedForDisplay() {
        val repo = Seed.newRepo()
        val id = repo.savePerson(Seed.living("anh", "Lê Văn Đức", "con trai", null)).id
        val p = repo.person(id)!!
        assertEquals("Anh", p.danhXung)
        assertEquals("Con trai", p.quanHe)
        assertEquals("Anh Lê Văn Đức", p.displayName)
    }
}
