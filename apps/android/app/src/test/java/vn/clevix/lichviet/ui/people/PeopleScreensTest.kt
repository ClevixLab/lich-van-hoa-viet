package vn.clevix.lichviet.ui.people

import android.Manifest
import android.app.Application
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasScrollAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToNode
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.unit.Density
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
        Light("light", ThemeMode.Light, 1f), Dark("dark", ThemeMode.Dark, 1f), Large("large", ThemeMode.Light, 1.3f)
    }

    private lateinit var repo: PeopleRepo
    private lateinit var ids: Seed.Ids
    private val nav = PeopleNav()
    private var variant by mutableStateOf(V.Light)
    private val shotDir = File(System.getProperty("lvhv.screenshots.dir") ?: "build/screenshots")

    @Before
    fun setUp() {
        shadowOf(ApplicationProvider.getApplicationContext<Application>()).grantPermissions(Manifest.permission.POST_NOTIFICATIONS)
        repo = Seed.newRepo()
        ids = Seed.family(repo)
    }

    private fun show(r: PeopleRepo? = repo, start: PRoute = PRoute.Upcoming) {
        nav.tab(start)
        rule.setContent {
            val d = LocalDensity.current
            CompositionLocalProvider(LocalDensity provides Density(d.density, d.fontScale * variant.scale)) {
                LichTheme(variant.mode, 1f) {
                    PeopleModule(r, "Dữ liệu được tạo bởi phiên bản mới hơn — hãy cập nhật ứng dụng", Seed.TODAY, nav, {}, reducedMotion = true)
                }
            }
        }
        rule.waitForIdle()
    }

    /** Chụp trang hiện tại ở ba biến thể. [prep] chạy sau mỗi lần đổi biến thể (mở hộp thoại, cuộn…). */
    private fun shoot(name: String, screen: Boolean = false, prep: () -> Unit = {}) {
        for (v in V.entries) {
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

    @Test
    fun screenshotsLists() {
        show()
        rule.onNodeWithText("30 ngày tới").assertExists()
        shoot("01-sap-toi")
        go(PRoute.Period)
        rule.runOnIdle { nav.tab(PRoute.Period) }
        shoot("02-theo-ky-thang")
        rule.onNodeWithText("Quý").performClick()
        shoot("03-theo-ky-quy")
        rule.onNodeWithText("Người đã mất").performClick()
        shoot("04-theo-ky-da-mat")
        rule.runOnIdle { nav.tab(PRoute.Profiles) }
        shoot("05-ho-so")
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
            runCatching { rule.onNodeWithText("Xem ▸").performClick() }
            rule.onNode(hasScrollAction()).performScrollToNode(hasText("Người báo tin"))
        }
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
    }

    @Test
    fun screenshotsDialogsAndStates() {
        show(start = PRoute.Profiles)
        go(PRoute.PersonPage(ids.c))
        shoot("20-hop-xoa", screen = true) {
            rule.onNode(hasScrollAction()).performScrollToNode(hasText("Xóa hồ sơ"))
            if (rule.onAllNodesWithText("Giữ lại").fetchSemanticsNodes().isEmpty()) rule.onNodeWithText("Xóa hồ sơ").performClick()
        }
        rule.onNodeWithText("Giữ lại").performClick()
        rule.runOnIdle { nav.tab(PRoute.Backup) }
        shoot("21-hop-ics", screen = true) {
            if (rule.onAllNodesWithText("Chia sẻ tệp .ics").fetchSemanticsNodes().isEmpty()) rule.onNodeWithText("Xuất tệp lịch (.ics)…").performClick()
        }
        rule.onNodeWithText("Hủy").performClick()
        shoot("22-hop-sao-luu", screen = true) {
            if (rule.onAllNodesWithText("Tạo và lưu tệp").fetchSemanticsNodes().isEmpty()) rule.onNodeWithText("Tạo tệp sao lưu…").performClick()
        }
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
        rule.onNodeWithText("Họ và tên *").performTextInput("Hoàng Văn E")
        rule.onAllNodesWithText("Ngày")[0].performTextInput("8")
        rule.onAllNodesWithText("Tháng")[0].performTextInput("8")
        rule.onAllNodesWithText("Năm")[0].performTextInput("2025")
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
        rule.onNodeWithText("Họ tên không được để trống", substring = true).assertExists()
        assertTrue(r.persons().isEmpty())
    }

    /** US-M-07: "Cập nhật: đã mất" → cùng id, rev tăng, không còn nhắc sinh nhật (B4); thông tin báo tử chỉ trong hồ sơ. */
    @Test
    fun flowMarkDeceased() {
        show(start = PRoute.Profiles)
        go(PRoute.MarkDeceased(ids.m))
        rule.onAllNodesWithText("Ngày")[0].performTextInput("1")
        rule.onAllNodesWithText("Tháng")[0].performTextInput("11")
        rule.onAllNodesWithText("Năm")[0].performTextInput("2026")
        rule.onNodeWithText("Người báo tin").performTextInput("Anh Tuấn")
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
        go(PRoute.Memorial(ids.d))
        assertTrue(rule.onAllNodesWithText("Bạch Mai", substring = true).fetchSemanticsNodes().isEmpty())
        assertNotNull(repo.person(ids.d)!!.notice)
    }

    private fun assertNull(v: Any?) = org.junit.Assert.assertNull(v)
}
