package vn.clevix.lichviet.ui.people

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.semantics.SemanticsProperties
import com.github.takahirom.roborazzi.captureRoboImage
import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import vn.clevix.lichviet.people.DateValue
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.Seed
import vn.clevix.lichviet.people.o
import vn.clevix.lichviet.ui.LichTheme
import vn.clevix.lichviet.ui.ThemeMode

/**
 * YC-2026-10 (08 §16.4): màn hình tưởng niệm không còn mâm vẽ. (1) người đã mất không có ảnh, gói ảnh rỗng → không có
 * phần tử mâm, có hai nến và bát hương; (4) Công giáo → nến, hoa, không hương. Ảnh chụp ở 360 dp và 1280 dp, sáng và tối.
 */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [34], qualifiers = "w360dp-h800dp-xhdpi")
class MemorialTrayTest {
    @get:Rule val rule = createComposeRule()

    private lateinit var repo: PeopleRepo
    private lateinit var ids: Seed.Ids
    private var mode by mutableStateOf(ThemeMode.Light)
    private val nav = PeopleNav()
    private val shotDir = File(System.getProperty("lvhv.screenshots.dir") ?: "build/screenshots")

    @Before
    fun setUp() {
        repo = Seed.newRepo()
        ids = Seed.family(repo)
    }

    /** Người đã mất Công giáo (tray_style = cong_giao). */
    private fun catholic(): String {
        val m = Seed.deceased("Ông", "Giuse Phạm Văn Long", "Ông ngoại", DateValue("solar", "day", 2020, 3, 9).toMap(), DateValue("solar", "day", 1935, 1, 2).toMap())
        (m.o("memorial")!!)["tray_style"] = "cong_giao"
        (m.o("memorial")!!)["incense"] = false
        return repo.savePerson(m).id
    }

    private fun show(id: String) {
        nav.tab(PRoute.Profiles)
        nav.go(PRoute.Memorial(id))
        rule.setContent {
            LichTheme(mode, 1f) {
                PeopleModule(repo, null, Seed.TODAY, nav, {}, reducedMotion = true)
            }
        }
        rule.waitForIdle()
    }

    /** Không phần tử nào mang nhãn có chữ "mâm" (nhãn trợ năng hay thẻ kiểm thử của mâm). */
    private fun assertNoTray() {
        val trayDesc = SemanticsMatcher("nhãn có chữ mâm") { n ->
            n.config.getOrElseNullable(SemanticsProperties.ContentDescription) { null }?.any { d -> d.contains("mâm", ignoreCase = true) } == true
        }
        assertTrue("không có phần tử mâm", rule.onAllNodes(trayDesc, useUnmergedTree = true).fetchSemanticsNodes().isEmpty())
        assertTrue(rule.onAllNodes(hasTestTag("tray"), useUnmergedTree = true).fetchSemanticsNodes().isEmpty())
        assertTrue(rule.onAllNodes(hasTestTag("tray-photo"), useUnmergedTree = true).fetchSemanticsNodes().isEmpty())
    }

    private fun shoot(name: String, width: String) {
        for (m in listOf(ThemeMode.Light, ThemeMode.Dark)) {
            mode = m
            rule.waitForIdle()
            rule.onRoot().captureRoboImage(File(shotDir, "and-yc10-$name-$width-${if (m == ThemeMode.Light) "light" else "dark"}.png").path)
        }
    }

    private fun case1(width: String) {
        show(ids.a)
        assertNoTray()
        val altar = rule.onAllNodes(hasTestTag("altar"), useUnmergedTree = true).fetchSemanticsNodes()
        assertEquals(1, altar.size)
        assertEquals(listOf(ALTAR_DESC), altar[0].config[SemanticsProperties.ContentDescription])
        shoot("1-khong-mam", width)
    }

    private fun case4(width: String) {
        show(catholic())
        assertNoTray()
        assertTrue(rule.onAllNodes(hasContentDescription(ALTAR_DESC_CATHOLIC), useUnmergedTree = true).fetchSemanticsNodes().size == 1)
        assertTrue("Công giáo không có hương", rule.onAllNodes(hasContentDescription("hương", substring = true), useUnmergedTree = true).fetchSemanticsNodes().isEmpty())
        shoot("4-cong-giao", width)
    }

    @Test fun noTray360() = case1("360")

    @Test @Config(qualifiers = "w1280dp-h900dp-mdpi") fun noTray1280() = case1("1280")

    @Test fun catholic360() = case4("360")

    @Test @Config(qualifiers = "w1280dp-h900dp-mdpi") fun catholic1280() = case4("1280")

    // ---------------------------------------------------------------------------------------------
    // Bước 2: ảnh mâm lễ của gia đình (memorial.tray_photo_id) và gói ảnh Clevix
    // ---------------------------------------------------------------------------------------------

    /** JPEG có đoạn APP1 Exif chứa GPS (như ảnh chụp bằng điện thoại). */
    private fun exifJpeg(w: Int, h: Int): ByteArray {
        val bmp = android.graphics.Bitmap.createBitmap(w, h, android.graphics.Bitmap.Config.ARGB_8888)
        val cv = android.graphics.Canvas(bmp)
        cv.drawColor(android.graphics.Color.rgb(120, 70, 40))
        val p = android.graphics.Paint()
        for ((k, c) in listOf(0xFFE0B13A, 0xFFF5F0E6, 0xFFC0463A, 0xFF7FA34A).withIndex()) {
            p.color = c.toInt()
            cv.drawCircle(w * (0.2f + k * 0.2f), h * 0.5f, h * 0.15f, p)
        }
        val bos = java.io.ByteArrayOutputStream()
        bmp.compress(android.graphics.Bitmap.CompressFormat.JPEG, 90, bos)
        val plain = bos.toByteArray()
        val gps = "Exif\u0000\u0000II*\u0000\u0008\u0000\u0000\u0000GPS 21.0285N 105.8542E".toByteArray(Charsets.ISO_8859_1)
        val len = gps.size + 2
        val app1 = byteArrayOf(0xFF.toByte(), 0xE1.toByte(), (len shr 8).toByte(), len.toByte()) + gps
        return plain.copyOfRange(0, 2) + app1 + plain.copyOfRange(2, plain.size)
    }

    private fun hasApp1(jpeg: ByteArray): Boolean {
        var i = 2
        while (i + 4 <= jpeg.size && jpeg[i] == 0xFF.toByte()) {
            val m = jpeg[i + 1].toInt() and 0xFF
            if (m == 0xE1 || m == 0xED) return true
            if (m == 0xDA) return false
            i += 2 + (((jpeg[i + 2].toInt() and 0xFF) shl 8) or (jpeg[i + 3].toInt() and 0xFF))
        }
        return false
    }

    /** Gắn ảnh mâm lễ cho ông A (như biểu mẫu: mã hóa lại ≤ 1600 px, ≤ 1 MB, rồi lưu hồ sơ). */
    private fun addFamilyTray(): String {
        val src = exifJpeg(2400, 1800)
        assertTrue("ảnh mẫu phải có EXIF", hasApp1(src))
        val enc = vn.clevix.lichviet.people.Photos.encode(src, TRAY_EDGE, TRAY_STORED)
        val pid = repo.addPhoto(enc)
        val m = repo.person(ids.a)!!.raw
        m.o("memorial")!!["tray_photo_id"] = pid
        repo.savePerson(m)
        return pid
    }

    private fun trayNodes(desc: String) =
        rule.onAllNodes(hasTestTag("tray-photo") and hasContentDescription(desc), useUnmergedTree = true).fetchSemanticsNodes()

    private fun case2(width: String) {
        val pid = addFamilyTray()
        val stored = repo.photo(pid)!!
        assertTrue("ảnh lưu không còn EXIF/IPTC", !hasApp1(stored))
        assertTrue(!String(stored, Charsets.ISO_8859_1).contains("GPS"))
        val bo = android.graphics.BitmapFactory.Options().apply { inJustDecodeBounds = true }
        android.graphics.BitmapFactory.decodeByteArray(stored, 0, stored.size, bo)
        assertEquals(TRAY_EDGE, maxOf(bo.outWidth, bo.outHeight))
        assertTrue(stored.size <= TRAY_STORED)
        show(ids.a)
        assertEquals(1, trayNodes("Ảnh mâm lễ của gia đình").size)
        assertTrue("có ảnh gia đình → không có nút thêm", rule.onAllNodes(androidx.compose.ui.test.hasText("Thêm ảnh mâm lễ của gia đình")).fetchSemanticsNodes().isEmpty())
        shoot("2-anh-gia-dinh", width)
    }

    @Test fun familyTray360() = case2("360")

    @Test @Config(qualifiers = "w1280dp-h900dp-mdpi") fun familyTray1280() = case2("1280")

    /** (3) bỏ ảnh → về (1), không còn tệp (lõi xóa ảnh cũ trong cùng giao dịch). */
    @Test
    fun removeTrayGoesBackToNone() {
        val pid = addFamilyTray()
        val m = repo.person(ids.a)!!.raw
        m.o("memorial")!!.remove("tray_photo_id")
        repo.savePerson(m)
        org.junit.Assert.assertNull(repo.photo(pid))
        assertTrue(File(Seed.lastDir, "photos").listFiles().orEmpty().none { it.name.startsWith(pid) })
        show(ids.a)
        assertNoTray()
        assertEquals(1, rule.onAllNodes(androidx.compose.ui.test.hasText("Thêm ảnh mâm lễ của gia đình")).fetchSemanticsNodes().size)
    }

    /** (5) gói thử có một ảnh đã duyệt → ảnh Clevix khi chưa có ảnh gia đình; ảnh gia đình thắng khi có. */
    @Test
    fun clevixPackThenFamilyWins() {
        val bytes = vn.clevix.lichviet.people.Photos.encode(exifJpeg(1200, 900), TRAY_EDGE, 400 * 1024).bytes
        val meta = linkedMapOf<String, Any?>(
            "id" to "truyen-thong-thu", "tray_style" to "truyen_thong", "file" to "t.jpg", "mime" to "image/jpeg",
            "sha256" to vn.clevix.lichviet.people.TrayPack.sha256(bytes), "tac_gia" to "Nguyễn Thử", "giay_phep" to "CC BY 4.0",
            "ghi_cong" to "Nguyễn Thử, CC BY 4.0", "trang_thai" to "duyet",
        )
        vn.clevix.lichviet.people.TrayPack.override = listOf(vn.clevix.lichviet.people.TrayItem(meta, bytes))
        try {
            show(ids.a)
            assertEquals(1, trayNodes("Ảnh mâm lễ").size)
            rule.onNode(androidx.compose.ui.test.hasText("Ảnh: Nguyễn Thử · CC BY 4.0")).assertExists()
            assertEquals(1, rule.onAllNodes(androidx.compose.ui.test.hasText("Thêm ảnh mâm lễ của gia đình")).fetchSemanticsNodes().size)
            // Ảnh gia đình thắng.
            addFamilyTray()
            rule.runOnIdle { nav.tab(PRoute.Profiles); nav.go(PRoute.Memorial(ids.a)) }
            rule.waitForIdle()
            assertEquals(1, trayNodes("Ảnh mâm lễ của gia đình").size)
            assertTrue(rule.onAllNodes(androidx.compose.ui.test.hasText("Ảnh: Nguyễn Thử", substring = true)).fetchSemanticsNodes().isEmpty())
        } finally {
            vn.clevix.lichviet.people.TrayPack.override = null
        }
    }

    /** Biểu mẫu (08 §16.2): câu chữ chính xác; nút "Thêm ảnh mâm lễ của gia đình" mở mục này; hồ sơ có dòng "Ảnh mâm lễ". */
    @Test
    fun formSectionAndProfileLine() {
        show(ids.a)
        rule.onNode(androidx.compose.ui.test.hasText("Thêm ảnh mâm lễ của gia đình")).performScrollTo().performClick()
        rule.waitForIdle()
        rule.onNode(androidx.compose.ui.test.hasText("Ảnh mâm lễ ngày giỗ")).assertExists()
        rule.onNode(androidx.compose.ui.test.hasText("Chưa có ảnh. Ngày giỗ chỉ hiện bát hương và nến.")).assertExists()
        rule.onNode(androidx.compose.ui.test.hasText("Chọn ảnh mâm lễ…")).assertExists()
        rule.onNode(androidx.compose.ui.test.hasText("Ảnh chỉ lưu trên máy này và trong tệp sao lưu mã hóa. Ứng dụng tự bỏ thông tin vị trí và máy chụp trong ảnh.")).assertExists()
        rule.onRoot().captureRoboImage(File(shotDir, "and-yc10-form-chua-co-anh-360-light.png").path)
        rule.runOnIdle { nav.tab(PRoute.Profiles); nav.go(PRoute.PersonPage(ids.a)) }
        rule.waitForIdle()
        rule.onNode(androidx.compose.ui.test.hasText("Chưa có")).assertExists()
        addFamilyTray()
        rule.runOnIdle { nav.tab(PRoute.Profiles); nav.go(PRoute.EditPerson(ids.a, tray = true)) }
        rule.waitForIdle()
        rule.onNode(androidx.compose.ui.test.hasText("Đổi ảnh")).assertExists()
        rule.onNode(androidx.compose.ui.test.hasText("Bỏ ảnh")).assertExists()
        rule.onRoot().captureRoboImage(File(shotDir, "and-yc10-form-co-anh-360-light.png").path)
        assertEquals("Không đọc được ảnh này. Hãy chọn một ảnh khác.", trayError(Exception("x")))
        assertEquals("Ảnh quá lớn (tối đa 25 MB).", trayError(vn.clevix.lichviet.people.Photos.PhotoError("Ảnh lớn hơn 25 MB — hãy chọn ảnh nhỏ hơn")))
    }
}
