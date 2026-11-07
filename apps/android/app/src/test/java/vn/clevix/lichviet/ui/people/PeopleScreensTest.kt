package vn.clevix.lichviet.ui.people

import android.Manifest
import android.app.Application
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasScrollAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performScrollToNode
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.dp
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.test.core.app.ApplicationProvider
import com.github.takahirom.roborazzi.ExperimentalRoborazziApi
import com.github.takahirom.roborazzi.captureRoboImage
import com.github.takahirom.roborazzi.captureScreenRoboImage
import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.Shadows.shadowOf
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.Seed
import vn.clevix.lichviet.ui.LichTheme
import vn.clevix.lichviet.ui.ThemeMode

/**
 * Kiểm thử giao diện mô-đun "Người thân" trên JVM (Robolectric, đồ họa gốc) với lõi Rust thật, và ảnh chụp mọi
 * màn hình mới: điện thoại 360 dp × 800 dp (xhdpi → ảnh 720 × 1600), sáng, tối và chữ lớn (cỡ chữ hệ thống 1,3).
 * Ảnh ghi vào apps/android/screenshots/and-*.png.
 */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [34], qualifiers = "w360dp-h800dp-xhdpi")
@OptIn(ExperimentalTestApi::class, ExperimentalRoborazziApi::class)
class PeopleScreensTest {
    @get:Rule val rule = createComposeRule()

    private enum class V(val suffix: String, val mode: ThemeMode, val scale: Float) {
        Light("light", ThemeMode.Light, 1f), Dark("dark", ThemeMode.Dark, 1f), Large("large", ThemeMode.Light, 1.3f),
        XLarge("xlarge", ThemeMode.Light, 2f),
    }

    private val std = listOf(V.Light, V.Dark, V.Large)
    private val withXl = listOf(V.Light, V.Dark, V.Large, V.XLarge)
    /** Số lần mô-đun báo "đã ghi" (→ đặt lại chuỗi báo thức trong ứng dụng thật). */
    private var changes = 0

    private lateinit var repo: PeopleRepo
    private lateinit var ids: Seed.Ids
    private val nav = PeopleNav()
    private var variant by mutableStateOf(V.Light)
    /** Activity chứa nội dung kiểm thử (để đọc cờ FLAG_SECURE của cửa sổ — BM2-04). */
    private var act: android.app.Activity? = null
    private val shotDir = File(System.getProperty("lvhv.screenshots.dir") ?: "build/screenshots")

    @Before
    fun setUp() {
        shadowOf(ApplicationProvider.getApplicationContext<Application>()).grantPermissions(Manifest.permission.POST_NOTIFICATIONS)
        repo = Seed.newRepo()
        ids = Seed.family(
            repo,
            listOf(
                Triple("giay_chung_tu", "image/jpeg", "giay-chung-tu.jpg" to certJpeg()),
                Triple("tin_bao_tu", "application/pdf", "Tin bao tu ba Dung.pdf" to Seed.PDF),
            ),
        )
    }

    /** Ảnh "giấy chứng tử" giả (JPEG không siêu dữ liệu, như sau Photos.encode). */
    private fun certJpeg(): ByteArray {
        val bmp = android.graphics.Bitmap.createBitmap(800, 560, android.graphics.Bitmap.Config.ARGB_8888)
        val cv = android.graphics.Canvas(bmp)
        cv.drawColor(android.graphics.Color.rgb(246, 243, 235))
        val pen = android.graphics.Paint().apply { color = android.graphics.Color.rgb(150, 140, 130); strokeWidth = 6f }
        cv.drawRect(30f, 30f, 770f, 530f, android.graphics.Paint().apply { style = android.graphics.Paint.Style.STROKE; color = android.graphics.Color.rgb(170, 60, 50); strokeWidth = 8f })
        for (i in 0 until 9) cv.drawLine(80f, 110f + i * 45f, if (i % 3 == 2) 480f else 720f, 110f + i * 45f, pen)
        val out = java.io.ByteArrayOutputStream()
        bmp.compress(android.graphics.Bitmap.CompressFormat.JPEG, 85, out)
        return vn.clevix.lichviet.people.Photos.encode(out.toByteArray(), vn.clevix.lichviet.people.Photos.ATT_EDGE, vn.clevix.lichviet.people.Photos.ATT_STORED).bytes
    }

    private fun show(r: PeopleRepo? = repo, start: PRoute = PRoute.Upcoming, reduced: Boolean = true) {
        nav.tab(start)
        rule.setContent {
            act = LocalContext.current.findActivity()
            val d = LocalDensity.current
            CompositionLocalProvider(LocalDensity provides Density(d.density, d.fontScale * variant.scale)) {
                LichTheme(variant.mode, 1f) {
                    PeopleModule(r, "Dữ liệu được tạo bởi phiên bản mới hơn — hãy cập nhật ứng dụng", Seed.TODAY, nav, { changes++ }, reducedMotion = reduced)
                }
            }
        }
        rule.waitForIdle()
    }

    /** Chụp trang hiện tại ở ba biến thể. [prep] chạy sau mỗi lần đổi biến thể (mở hộp thoại, cuộn…). */
    private fun shoot(name: String, screen: Boolean = false, variants: List<V> = std, prep: () -> Unit = {}) {
        for (v in variants) {
            variant = v
            rule.waitForIdle()
            prep()
            rule.waitForIdle()
            val f = File(shotDir, "and-$name-${v.suffix}.png").path
            if (screen) captureScreenRoboImage(f) else rule.onRoot().captureRoboImage(f)
        }
    }

    private fun go(r: PRoute) {
        rule.runOnIdle { nav.tab(PRoute.Profiles); if (r != PRoute.Profiles) nav.go(r) }
        rule.waitForIdle()
    }

    /** B5 mục 3: chuyển đổi lỗi → thẻ đỏ chỉ đọc trên mọi trang, không có nút thêm/sửa/xóa, vẫn tạo được tệp sao lưu. */
    @Test
    fun readOnlyAfterFailedMigration() {
        val dir = vn.clevix.lichviet.people.MigrationTest.copyFixture()
        vn.clevix.lichviet.people.MigrationTest.breakMigration(dir)
        val ro = PeopleRepo(vn.clevix.lichviet.core.LvhvStore.open(dir.absolutePath))
        assertTrue(ro.readOnly)
        show(r = ro, start = PRoute.Profiles)
        rule.onNodeWithText("chế độ chỉ đọc", substring = true).assertExists()
        assertTrue(rule.onAllNodesWithText("+ Thêm").fetchSemanticsNodes().isEmpty())
        // Dữ liệu cũ vẫn xem được.
        assertTrue(rule.onAllNodesWithText("Nguyễn Văn A", substring = true).fetchSemanticsNodes().isNotEmpty())
        // TKN3-A-01: lời thường; mã kỹ thuật chỉ trong mục thu gọn "Chi tiết kỹ thuật".
        assertTrue(rule.onAllNodesWithText("R14 kiem dinh", substring = true).fetchSemanticsNodes().isEmpty())
        shoot("41-chi-doc", variants = withXl)
        rule.onNodeWithText("Chi tiết kỹ thuật ▾").performClick()
        rule.onNodeWithText("R14 kiem dinh", substring = true).assertExists()
        rule.onNodeWithText("Chi tiết kỹ thuật ▴").performClick()
        rule.onNodeWithText("Đi tới Sao lưu").performScrollTo().performClick()
        rule.waitForIdle()
        assertTrue(rule.onAllNodesWithText("Khôi phục từ tệp sao lưu").fetchSemanticsNodes().isEmpty())
        shoot("42-chi-doc-sao-luu")
        val p = ro.persons().first { it.deceased }
        go(PRoute.PersonPage(p.id))
        assertTrue(rule.onAllNodesWithText("Xóa hồ sơ").fetchSemanticsNodes().isEmpty())
        rule.runOnIdle { nav.go(PRoute.EditPerson(p.id)) }
        rule.waitForIdle()
        assertTrue("không mở biểu mẫu khi chỉ đọc", rule.onAllNodesWithText("Lưu").fetchSemanticsNodes().isEmpty())
    }

    /** BM2-04: cửa sổ đặt FLAG_SECURE (ảnh đa nhiệm trống) đúng ở các màn hình nhạy cảm, gỡ khi rời đi. */
    @Test
    fun flagSecureOnlyOnSensitiveScreens() {
        show()
        fun secure(): Boolean { rule.waitForIdle(); return SecureWindow.isSecure(act!!) }
        assertFalse("Sắp tới không che", secure())
        go(PRoute.Backup)
        assertTrue("Sao lưu (ô mật khẩu) phải che", secure())
        go(PRoute.Profiles)
        assertFalse("rời Sao lưu → gỡ che", secure())
        go(PRoute.PersonPage(ids.d))
        assertTrue("hồ sơ có thông tin báo tử phải che", secure())
        go(PRoute.PersonPage(ids.b))
        assertFalse("hồ sơ người đã mất không có báo tử không che", secure())
        go(PRoute.PersonPage(ids.c))
        assertFalse("hồ sơ người đang sống không che", secure())
        go(PRoute.MarkDeceased(ids.m))
        assertTrue("biểu mẫu báo tử phải che", secure())
        go(PRoute.EditPerson(ids.d))
        assertTrue("sửa hồ sơ người đã mất (khối báo tử) phải che", secure())
        go(PRoute.Upcoming)
        assertFalse("về danh sách → gỡ che", secure())
        // Hai màn hình che chồng nhau (Sao lưu → hồ sơ có báo tử → quay lại): không hở khi một màn hình rời đi.
        rule.runOnIdle { nav.tab(PRoute.Backup); nav.go(PRoute.PersonPage(ids.d)) }
        assertTrue(secure())
        rule.runOnIdle { nav.back() }
        assertTrue("quay lại Sao lưu vẫn che", secure())
    }

    @Test
    fun screenshotsLists() {
        show()
        rule.onNodeWithText("30 ngày tới").assertExists()
        shoot("01-sap-toi")
        go(PRoute.Period)
        rule.runOnIdle { nav.tab(PRoute.Period) }
        shoot("02-theo-ky-thang", variants = withXl)
        rule.onNodeWithText("Quý").performClick()
        shoot("03-theo-ky-quy", variants = withXl)
        rule.onNodeWithText("Người đã mất").performClick()
        shoot("04-theo-ky-da-mat", variants = withXl)
        rule.runOnIdle { nav.tab(PRoute.Profiles) }
        shoot("05-ho-so", variants = withXl)
        rule.runOnIdle { nav.tab(PRoute.Backup) }
        shoot("06-sao-luu")
        rule.runOnIdle { nav.go(PRoute.Settings) }
        shoot("07-cai-dat-nhac")
    }

    @Test
    fun screenshotsPerson() {
        show(start = PRoute.Profiles)
        go(PRoute.PersonPage(ids.a))
        rule.onNodeWithText("Lần tới").assertExists()
        shoot("08-ho-so-da-mat")
        go(PRoute.PersonPage(ids.c))
        shoot("09-ho-so-dang-song")
        go(PRoute.PersonPage(ids.d))
        shoot("10-ho-so-bao-tu") {
            if (rule.onAllNodesWithText("Xem ▾").fetchSemanticsNodes().isNotEmpty()) rule.onNodeWithText("Xem ▾").performScrollTo().performClick()
            rule.onNode(hasScrollAction()).performScrollToNode(hasText("Người báo tin"))
        }
        shoot("10b-ho-so-tep-dinh-kem") {
            if (rule.onAllNodesWithText("Xem ▾").fetchSemanticsNodes().isNotEmpty()) rule.onNodeWithText("Xem ▾").performScrollTo().performClick()
            rule.onNode(hasScrollAction()).performScrollToNode(hasText("Mở bằng ứng dụng khác…"))
        }
        // Mở PDF: hộp xác nhận trước khi đưa tệp ra ứng dụng khác (§5.3, S10).
        shoot("10c-hop-mo-pdf", screen = true) {
            if (rule.onAllNodesWithText("Mở tệp").fetchSemanticsNodes().isEmpty()) {
                if (rule.onAllNodesWithText("Xem ▾").fetchSemanticsNodes().isNotEmpty()) rule.onNodeWithText("Xem ▾").performScrollTo().performClick()
                rule.onNodeWithText("Mở bằng ứng dụng khác…").performScrollTo().performClick()
            }
        }
        rule.onNodeWithText("Hủy").performClick()
        go(PRoute.Memorial(ids.a))
        shoot("11-tuong-niem")
        go(PRoute.Memorial(ids.b))
        shoot("12-tuong-niem-hom-nay")
    }

    @Test
    fun screenshotsForms() {
        show(start = PRoute.Profiles)
        go(PRoute.AddPerson(true))
        shoot("13-them-da-mat")
        go(PRoute.EditPerson(ids.a))
        shoot("14-sua-da-mat")
        shoot("15-sua-da-mat-cuoi") { rule.onNode(hasScrollAction()).performScrollToNode(hasText("Nhắc ngày giỗ")) }
        go(PRoute.AddPerson(false))
        shoot("16-them-dang-song")
        go(PRoute.EventEdit(ids.c, null))
        shoot("17-them-su-kien")
        go(PRoute.EventEdit(ids.a, null))
        shoot("18-su-kien-nguoi-da-mat") { rule.onNode(hasScrollAction()).performScrollToNode(hasText("Không nhắc sự kiện của người đã mất", substring = true)) }
        go(PRoute.MarkDeceased(ids.m))
        shoot("19-cap-nhat-da-mat")
        go(PRoute.EditPerson(ids.d))
        shoot("19b-sua-tep-dinh-kem") {
            rule.onNode(hasScrollAction()).performScrollToNode(hasText("Thêm tệp PDF"))
        }
    }

    @Test
    fun screenshotsDialogsAndStates() {
        show(start = PRoute.Profiles)
        go(PRoute.PersonPage(ids.c))
        shoot("20-hop-xoa", screen = true) {
            if (rule.onAllNodesWithText("Giữ lại").fetchSemanticsNodes().isEmpty()) {
                rule.onNode(hasScrollAction()).performScrollToNode(hasText("Xóa hồ sơ"))
                rule.onNodeWithText("Xóa hồ sơ").performClick()
            }
        }
        rule.onNodeWithText("Giữ lại").performClick()
        rule.runOnIdle { nav.tab(PRoute.Backup) }
        shoot("21-hop-ics", screen = true) {
            if (rule.onAllNodesWithText("Chia sẻ tệp .ics").fetchSemanticsNodes().isEmpty()) rule.onNodeWithText("Xuất tệp lịch (.ics)…").performClick()
        }
        rule.onNodeWithText("Hủy").performClick()
        shoot("22-tao-sao-luu") {
            if (rule.onAllNodesWithText("Tạo và lưu tệp").fetchSemanticsNodes().isEmpty()) {
                rule.onNodeWithText("Tạo tệp sao lưu…").performScrollTo().performClick()
            }
            rule.onNode(hasScrollAction()).performScrollToNode(androidx.compose.ui.test.hasContentDescription("Nhập lại mật khẩu"))
        }
        // Lỗi mật khẩu hiện ngay dưới ô (TK-14).
        field("Mật khẩu (ít nhất 8 ký tự)").performTextInput("ngan")
        rule.onNodeWithText("Tạo và lưu tệp").performScrollTo().performClick()
        rule.onNodeWithText("Mật khẩu cần ít nhất 8 ký tự").assertExists()
    }

    @Test
    fun screenshotsEmptyAndError() {
        show(r = Seed.newRepo())
        rule.onNodeWithText("Chưa có ai trong sổ", substring = true).assertExists()
        shoot("23-trong")
    }

    @Test
    fun screenshotStoreError() {
        show(r = null)
        rule.onNodeWithText("Không mở được dữ liệu trên máy", substring = true).assertExists()
        shoot("24-loi-kho")
    }

    // ---------------------------------------------------------------------------------------------
    // Luồng (08 §4) — kiểm trên giao diện, so với dữ liệu trong kho của lõi.
    // ---------------------------------------------------------------------------------------------

    /** L1: thêm người đã mất theo ngày dương → dòng đổi lịch "15/6 nhuận", hồ sơ lưu ngày âm đã quy đổi. */
    @Test
    fun flowAddDeceasedSolar() {
        val r = Seed.newRepo()
        show(r = r, start = PRoute.Profiles)
        go(PRoute.AddPerson(true))
        field("Họ và tên *").performTextInput("Hoàng Văn E")
        field("Ngày").performTextInput("8")
        field("Tháng").performTextInput("8")
        field("Năm").performTextInput("2025")
        rule.onNodeWithText("15/6 nhuận", substring = true).assertExists()
        rule.onNode(hasScrollAction()).performScrollToNode(hasText("Lưu"))
        rule.onNodeWithText("Lưu").performClick()
        rule.waitForIdle()
        val p = r.persons().single()
        assertEquals("Hoàng Văn E", p.hoTen)
        assertEquals("converted", p.death!!["lunar_source"])
        assertTrue(nav.top is PRoute.PersonPage)
        rule.onNodeWithText("Lần tới").assertExists()
    }

    /** Lưu khi trống → khung tóm tắt lỗi, không ghi gì. */
    @Test
    fun flowEmptyFormShowsErrors() {
        val r = Seed.newRepo()
        show(r = r, start = PRoute.Profiles)
        go(PRoute.AddPerson(true))
        rule.onNode(hasScrollAction()).performScrollToNode(hasText("Lưu"))
        rule.onNodeWithText("Lưu").performClick()
        rule.onNode(hasScrollAction()).performScrollToNode(hasText("chỗ cần sửa", substring = true))
        // Lỗi ở khung tóm tắt và ngay dưới ô.
        assertEquals(2, rule.onAllNodesWithText("Họ tên không được để trống", substring = true).fetchSemanticsNodes().size)
        assertTrue(r.persons().isEmpty())
    }

    /** US-M-07: "Cập nhật: đã mất" → cùng id, rev tăng, không còn nhắc sinh nhật (B4); thông tin báo tử chỉ trong hồ sơ. */
    @Test
    fun flowMarkDeceased() {
        show(start = PRoute.Profiles)
        go(PRoute.MarkDeceased(ids.m))
        field("Ngày").performTextInput("1")
        field("Tháng").performTextInput("11")
        field("Năm").performTextInput("2026")
        field("Người báo tin").performTextInput("Anh Tuấn")
        rule.onNode(hasScrollAction()).performScrollToNode(hasText("Lưu"))
        rule.onNodeWithText("Lưu").performClick()
        rule.onNodeWithText("Lưu thay đổi").performClick()
        rule.waitForIdle()
        val p = repo.person(ids.m)!!
        assertTrue(p.deceased)
        assertEquals(2L, p.raw["rev"])
        assertNull(p.reminders!!["sinh_nhat"])
        assertEquals("Anh Tuấn", p.notice!!["reported_by"])
        // Thông tin báo tử không có trong danh sách, lời nhắc, .ics (R-PRIV-2).
        val ics = repo.ics(Seed.TODAY).content
        assertFalse(ics.contains("Anh Tuấn"))
        assertFalse(repo.upcoming(Seed.TODAY, 400).any { it.headline.contains("Anh Tuấn") })
    }

    /** L5 / R13: bộ lọc "Người đã mất" chỉ có giỗ (và tiên thường); "đã mất" ∪ "đang sống" = "tất cả". */
    @Test
    fun periodFilters() {
        val all = repo.period("nam", Seed.TODAY, "tat_ca").third
        val dead = repo.period("nam", Seed.TODAY, "da_mat").third
        val live = repo.period("nam", Seed.TODAY, "dang_song").third
        assertTrue(dead.all { it.isGio })
        assertEquals(all.size, dead.size + live.size)
        show(start = PRoute.Period)
        rule.onNodeWithText("Năm").performClick()
        rule.onNodeWithText("Năm 2026 · 01/01 – 31/12").assertExists()
        rule.onNodeWithText("${all.size} mục").assertExists()
        rule.onNodeWithText("Người đã mất").performClick()
        rule.onNodeWithText("${dead.size} mục").assertExists()
    }

    /** B4: sự kiện của người đã mất không có khối nhắc. */
    @Test
    fun deceasedEventHasNoReminder() {
        show(start = PRoute.Profiles)
        go(PRoute.EventEdit(ids.a, null))
        rule.onNode(hasScrollAction()).performScrollToNode(hasText("Không nhắc sự kiện của người đã mất", substring = true))
        assertTrue(rule.onAllNodesWithText("Nhắc sự kiện này").fetchSemanticsNodes().isEmpty())
    }

    /** Màn hình tưởng niệm: dòng quan hệ (bỏ ngoặc) trên tên, tên không lặp cách gọi, "Hưởng thọ"; không có thông tin báo tử. */
    @Test
    fun memorialContent() {
        show(start = PRoute.Profiles)
        go(PRoute.Memorial(ids.a))
        rule.onNodeWithText("Ông nội").assertExists()
        rule.onNodeWithText("Nguyễn Văn A").assertExists()
        rule.onNodeWithText("Hưởng thọ", substring = true).assertExists()
        // Chuyển sang bia khác: không giữ dữ liệu của trang trước.
        go(PRoute.Memorial(ids.b))
        rule.onNodeWithText("Bà nội").assertExists()
        rule.onNodeWithText("Trần Thị B").assertExists()
        assertTrue(rule.onAllNodesWithText("Nguyễn Văn A").fetchSemanticsNodes().isEmpty())
        go(PRoute.Memorial(ids.d))
        assertTrue(rule.onAllNodesWithText("Bạch Mai", substring = true).fetchSemanticsNodes().isEmpty())
        assertNotNull(repo.person(ids.d)!!.notice)
    }

    private fun assertNull(v: Any?) = org.junit.Assert.assertNull(v)

    // ---------------------------------------------------------------------------------------------
    // Vòng duyệt thiết kế native (TKN-A-*) và duyệt mã (MA3N-24-*)
    // ---------------------------------------------------------------------------------------------

    private fun click(text: String, index: Int = 0) {
        val n = rule.onAllNodesWithText(text)[index]
        runCatching { n.performScrollTo() }
        n.performClick()
        rule.waitForIdle()
    }

    /** TKN-A-02: ô ngày âm, tháng nhuận, kết quả đổi lịch — chụp **cả** biểu mẫu (khung nhìn cao). */
    @Test
    @Config(qualifiers = "w360dp-h3200dp-xhdpi")
    fun screenshotsFullFormsLunar() {
        show(start = PRoute.Profiles)
        go(PRoute.AddPerson(true))
        field("Họ và tên *").performTextInput("Trần Thị Bình")
        click("Nhập theo âm lịch", 0)
        field("Ngày").performTextInput("15")
        field("Tháng").performTextInput("6")
        field("Năm").performTextInput("2025")
        click("Tháng nhuận (năm 2025 nhuận tháng 6)")
        rule.onNodeWithText("Dương lịch: Thứ Sáu, 08/08/2025").assertExists()
        shoot("25-them-da-mat-am-nhuan")
        // Năm không có tháng nhuận: hộp ẩn, có câu giải thích; "Không rõ năm mất".
        field("Năm").performTextClearance()
        field("Năm").performTextInput("2026")
        rule.onNodeWithText("Năm 2026 không có tháng nhuận").assertExists()
        shoot("26-am-khong-nhuan")
        click("Không rõ năm mất")
        rule.onNodeWithText("Không rõ năm: vẫn nhắc giỗ hằng năm", substring = true).assertExists()
        shoot("27-am-khong-ro-nam")
    }

    @Test
    @Config(qualifiers = "w360dp-h3200dp-xhdpi")
    fun screenshotsFullFormsSolarAndErrors() {
        show(start = PRoute.Profiles)
        go(PRoute.AddPerson(true))
        field("Ngày").performTextInput("8")
        field("Tháng").performTextInput("8")
        field("Năm").performTextInput("2025")
        rule.onNodeWithText("Âm lịch: 15/6 nhuận năm Ất Tỵ (2025)").assertExists()
        shoot("28-them-da-mat-duong-ket-qua")
        // Lỗi: ngày không tồn tại + họ tên trống → chữ đỏ dưới ô và khung "Còn n chỗ cần sửa".
        field("Ngày").performTextClearance(); field("Ngày").performTextInput("30")
        field("Tháng").performTextClearance(); field("Tháng").performTextInput("2")
        field("Năm").performTextClearance(); field("Năm").performTextInput("2026")
        click("Lưu")
        rule.onNodeWithText("Còn 2 chỗ cần sửa", substring = true).assertExists()
        assertEquals(2, rule.onAllNodesWithText("Tháng 2 năm 2026 chỉ có 28 ngày", substring = true).fetchSemanticsNodes().size)
        shoot("29-them-da-mat-loi") { rule.onNode(hasScrollAction()).performScrollToNode(hasText("Còn 2 chỗ cần sửa", substring = true)) }
    }

    @Test
    @Config(qualifiers = "w360dp-h3200dp-xhdpi")
    fun screenshotsLivingLunarBirthday() {
        show(start = PRoute.Profiles)
        go(PRoute.AddPerson(false))
        field("Họ và tên *").performTextInput("Lê Văn Cường")
        field("Ngày").performTextInput("20")
        field("Tháng").performTextInput("1")
        field("Năm").performTextInput("1990")
        rule.onNodeWithText("Âm lịch: 24/12 năm Kỷ Tỵ (1989)", substring = true).assertExists()
        click("Nhắc cả sinh nhật âm lịch hằng năm")
        click("Nhắc sinh nhật âm lịch giống nhắc sinh nhật dương lịch")
        assertTrue(rule.onAllNodesWithText("Nhắc sinh nhật âm lịch", useUnmergedTree = true).fetchSemanticsNodes().isNotEmpty())
        shoot("30-them-dang-song-sinh-nhat-am")
        // Tất cả biểu mẫu khác, chụp trọn: sửa người đã mất, sự kiện, cập nhật đã mất.
        go(PRoute.EditPerson(ids.a))
        shoot("31-sua-da-mat-tron")
        go(PRoute.EventEdit(ids.c, null))
        shoot("32-them-su-kien-tron")
        go(PRoute.MarkDeceased(ids.m))
        shoot("33-cap-nhat-da-mat-tron")
    }

    /** TKN-A-02: thẻ "thông báo đang tắt" ở Sắp tới. */
    @Test
    fun screenshotNotificationsOff() {
        shadowOf(ApplicationProvider.getApplicationContext<Application>()).denyPermissions(Manifest.permission.POST_NOTIFICATIONS)
        show()
        rule.onNodeWithText("Thông báo của ứng dụng đang tắt", substring = true).assertExists()
        shoot("34-sap-toi-thong-bao-tat") {
            rule.onNode(hasScrollAction()).performScrollToNode(hasText("Thông báo của ứng dụng đang tắt", substring = true))
        }
    }

    /** TKN-A-02: nhắc sao lưu (dữ liệu đổi từ lâu mà chưa sao lưu). */
    @Test
    fun screenshotBackupNag() {
        // Lùi mốc "chờ sao lưu" của kho về đầu năm (bảng meta của lvhv-store) → quá 30 ngày.
        // (SQLite của Robolectric không mở được tệp của kho Rust; dùng công cụ sqlite3 của máy dựng.)
        val p = runCatching {
            ProcessBuilder(
                "sqlite3", File(Seed.lastDir, "lvhv.sqlite").path,
                "UPDATE meta SET value = '2026-01-01T00:00:00Z' WHERE key = 'backup_pending_since';",
            ).redirectErrorStream(true).start()
        }.getOrNull()
        org.junit.Assume.assumeTrue("cần công cụ sqlite3", p != null && p.waitFor() == 0)
        assertTrue(repo.backupDue())
        show()
        rule.onNodeWithText("Sao lưu ngay").assertExists()
        shoot("35-nhac-sao-luu") { rule.onNode(hasScrollAction()).performScrollToNode(hasText("Sao lưu ngay")) }
    }

    /** TKN-A-02: hộp xem trước khôi phục và hộp xác nhận thứ hai "Thay toàn bộ". */
    @Test
    fun screenshotRestoreDialogs() {
        var replace by mutableStateOf(false)
        var second by mutableStateOf(false)
        val pv = vn.clevix.lichviet.core.BackupPreview(5u, 1u, 0u, "2026-11-01T03:00:00Z", "pwa")
        rule.setContent {
            val d = LocalDensity.current
            CompositionLocalProvider(LocalDensity provides Density(d.density, d.fontScale * variant.scale)) {
                LichTheme(variant.mode, 1f) {
                    androidx.compose.foundation.layout.Box(
                        androidx.compose.ui.Modifier.fillMaxSize().background(vn.clevix.lichviet.ui.LocalLich.current.wall),
                    ) {
                        RestoreDialogs(pv, 5, replace, second, false, { replace = it }, { second = false }, {
                            if (replace) second = true
                        })
                    }
                }
            }
        }
        rule.onNodeWithText("Tệp xuất ngày 01/11/2026 từ ứng dụng web", substring = true).assertExists()
        shoot("36-hop-khoi-phuc", screen = true)
        rule.onNodeWithText("Thay toàn bộ — xóa dữ liệu", substring = true).performClick()
        rule.onNodeWithText("Khôi phục").performClick()
        rule.onNodeWithText("Không hoàn tác được", substring = true).assertExists()
        shoot("37-hop-thay-toan-bo", screen = true)
    }

    /** TKN-A-02: khói đang chạy, nút "Dừng khói" (hoạt ảnh bật — đồng hồ kiểm thử chạy tay). */
    @Test
    fun screenshotSmokeRunning() {
        // Đồng hồ khung hình chạy tay từ đầu (hoạt ảnh vô tận không bao giờ "nghỉ").
        rule.mainClock.autoAdvance = false
        show(start = PRoute.Memorial(ids.b), reduced = false)
        rule.mainClock.advanceTimeByFrame()
        rule.mainClock.advanceTimeBy(1_500)
        rule.onNodeWithText("Dừng khói").assertExists()
        for (v in std) {
            variant = v
            rule.mainClock.advanceTimeBy(700)
            rule.onRoot().captureRoboImage(File(shotDir, "and-38-tuong-niem-khoi-chay-${v.suffix}.png").path)
        }
        // Dừng → "Thắp lại"; khói tự dừng sau 2 phút.
        rule.onNodeWithText("Dừng khói").performClick()
        rule.mainClock.advanceTimeBy(100)
        rule.onNodeWithText("Thắp lại").assertExists()
        rule.onNodeWithText("Thắp lại").performClick()
        rule.mainClock.advanceTimeBy(SMOKE_MS + 1_000)
        rule.onNodeWithText("Thắp lại").assertExists()
    }

    /**
     * TKN-A-02: nội dung thông báo. Robolectric không vẽ được khay thông báo của hệ thống, nên ảnh là **mô phỏng** bố cục
     * thông báo với **đúng văn bản** lõi trả về (`title`/`body` của `android_alarm_step`), mặc định không tên (O-9) và khi bật tên.
     */
    @Test
    fun screenshotNotificationContent() {
        val zone = java.time.ZoneId.of("Asia/Ho_Chi_Minh")
        val now0 = Seed.TODAY.atStartOfDay(zone).toEpochSecond()
        fun msgs(names: Boolean) = vn.clevix.lichviet.people.parseArr(
            vn.clevix.lichviet.core.remindersJson(repo.datasetJson(), now0, vn.clevix.lichviet.people.Zones.device(zone), 30u, names),
        ).take(3).map { vn.clevix.lichviet.people.ReminderMsg.of(it as vn.clevix.lichviet.people.JMap) }
        val hidden = msgs(false)
        val shown = msgs(true)
        assertTrue(hidden.isNotEmpty())
        // Mặc định: không tên người nào.
        val names = repo.persons().map { it.hoTen }
        assertFalse(hidden.any { m -> names.any { m.title.contains(it) || m.body.contains(it) } })
        rule.setContent {
            val d = LocalDensity.current
            CompositionLocalProvider(LocalDensity provides Density(d.density, d.fontScale * variant.scale)) {
                LichTheme(variant.mode, 1f) {
                    androidx.compose.foundation.layout.Column(
                        androidx.compose.ui.Modifier.fillMaxSize().background(androidx.compose.ui.graphics.Color(0xFF101418))
                            .padding(16.dp),
                        verticalArrangement = androidx.compose.foundation.layout.Arrangement.spacedBy(10.dp),
                    ) {
                        NotifLabel("Mô phỏng thông báo — văn bản thật của lõi. Mặc định (không hiện tên):")
                        hidden.forEach { NotifCard(it.title, it.body) }
                        NotifLabel("Khi bật “Hiện tên người trên màn hình khóa”:")
                        shown.forEach { NotifCard(it.title, it.body) }
                    }
                }
            }
        }
        shoot("40-thong-bao")
    }

    @androidx.compose.runtime.Composable
    private fun NotifLabel(t: String) =
        androidx.compose.material3.Text(t, color = androidx.compose.ui.graphics.Color(0xFFCFD8DC), style = androidx.compose.material3.MaterialTheme.typography.bodyMedium)

    @androidx.compose.runtime.Composable
    private fun NotifCard(title: String, body: String) {
        androidx.compose.material3.Surface(
            shape = androidx.compose.foundation.shape.RoundedCornerShape(20.dp),
            color = androidx.compose.ui.graphics.Color(0xFFECEFF1),
            modifier = androidx.compose.ui.Modifier.fillMaxWidth(),
        ) {
            androidx.compose.foundation.layout.Column(androidx.compose.ui.Modifier.padding(14.dp)) {
                androidx.compose.material3.Text(
                    "Lịch và Văn hóa Việt · bây giờ", color = androidx.compose.ui.graphics.Color(0xFF455A64),
                    style = androidx.compose.material3.MaterialTheme.typography.labelMedium,
                )
                androidx.compose.material3.Text(title, color = androidx.compose.ui.graphics.Color(0xFF111111), style = androidx.compose.material3.MaterialTheme.typography.titleMedium)
                if (body.isNotEmpty()) androidx.compose.material3.Text(body, color = androidx.compose.ui.graphics.Color(0xFF263238), style = androidx.compose.material3.MaterialTheme.typography.bodyMedium)
            }
        }
    }

    /** TKN-A-08: mục sự kiện ghi tên người (người đầu tiên của person_ids). */
    @Test
    fun eventItemShowsPersonName() {
        show()
        rule.onNodeWithText("Tốt nghiệp đại học · Lê Văn Cường", useUnmergedTree = true).assertExists()
        go(PRoute.Period)
        rule.runOnIdle { nav.tab(PRoute.Period) }
        rule.onNodeWithText("Tốt nghiệp đại học · Lê Văn Cường", useUnmergedTree = true).assertExists()
    }

    /** MA3N-24-07 (1): mọi lần ghi từ giao diện đều báo "đã ghi" → ứng dụng đặt lại chuỗi báo thức. */
    @Test
    fun writesTriggerRearm() {
        show(start = PRoute.Profiles)
        // Lưu (sửa) hồ sơ.
        go(PRoute.EditPerson(ids.c))
        val c0 = changes
        click("Lưu")
        assertEquals(c0 + 1, changes)
        // Đổi cài đặt nhắc.
        rule.runOnIdle { nav.tab(PRoute.Upcoming); nav.go(PRoute.Settings) }
        rule.waitForIdle()
        val c1 = changes
        click("Hiện tên người trên màn hình khóa")
        assertEquals(c1 + 1, changes)
        assertTrue(repo.lockScreenNames())
        // Xóa hồ sơ.
        go(PRoute.PersonPage(ids.m))
        val c2 = changes
        click("Xóa hồ sơ")
        rule.onNode(hasText("Xóa hồ sơ") and androidx.compose.ui.test.hasAnyAncestor(androidx.compose.ui.test.isDialog())).performClick()
        rule.waitForIdle()
        assertEquals(c2 + 1, changes)
        assertNull(repo.person(ids.m))
    }

    /** TKN-A-01: không chữ nào của mô-đun bị cắt / tràn ở cỡ chữ 1,3 và 2,0. */
    @Test
    fun noTextOverflowAtLargeFonts() {
        show(start = PRoute.Profiles)
        val routes = listOf(
            PRoute.Upcoming, PRoute.Period, PRoute.Profiles, PRoute.Backup, PRoute.Settings,
            PRoute.PersonPage(ids.a), PRoute.PersonPage(ids.c), PRoute.PersonPage(ids.d),
            PRoute.AddPerson(true), PRoute.AddPerson(false), PRoute.EventEdit(ids.c, null),
            PRoute.MarkDeceased(ids.m), PRoute.Memorial(ids.a),
        )
        for (v in listOf(V.Large, V.XLarge)) {
            variant = v
            for (r in routes) {
                go(r)
                val nodes = rule.onAllNodes(
                    androidx.compose.ui.test.SemanticsMatcher.keyIsDefined(androidx.compose.ui.semantics.SemanticsActions.GetTextLayoutResult)
                        and !androidx.compose.ui.test.hasSetTextAction(),
                    useUnmergedTree = true,
                ).fetchSemanticsNodes()
                assertTrue("$r: không thấy chữ nào", nodes.isNotEmpty())
                for (n in nodes) {
                    val out = ArrayList<androidx.compose.ui.text.TextLayoutResult>()
                    n.config[androidx.compose.ui.semantics.SemanticsActions.GetTextLayoutResult].action?.invoke(out)
                    for (t in out) {
                        assertFalse(
                            "${v.suffix} $r: chữ bị cắt/tràn: \"${t.layoutInput.text}\" (rộng ${t.didOverflowWidth}, cao " +
                                "${t.didOverflowHeight}, ${t.size}, ${t.layoutInput.constraints}, maxLines ${t.layoutInput.maxLines})",
                            // Bỏ qua sai số làm tròn < 1 px (đoạn chữ rộng 300,4 px trong ô 300 px) — chỉ bắt tràn/cắt thật.
                            // Đo theo từng dòng (bề rộng nét chữ, đáy dòng cuối) — bề rộng đoạn (paragraph) có thể là bề rộng
                            // tối đa của ràng buộc chứ không phải bề rộng chữ.
                            t.didOverflowHeight || t.multiParagraph.didExceedMaxLines ||
                                (t.didOverflowWidth && (0 until t.lineCount).any { t.getLineRight(it) - t.getLineLeft(it) - t.size.width > 1f }),
                        )
                    }
                }
            }
        }
    }

    /** Ô số đầu tiên có nhãn [label] (Ngày / Tháng / Năm của ô ngày đầu trang). */
    private fun field(label: String) =
        rule.onAllNodes(androidx.compose.ui.test.hasContentDescription(label) and androidx.compose.ui.test.hasSetTextAction())[0]
}
