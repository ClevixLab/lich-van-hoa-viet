package vn.clevix.lichviet.ui.people

import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onFirst
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import com.github.takahirom.roborazzi.ExperimentalRoborazziApi
import com.github.takahirom.roborazzi.captureScreenRoboImage
import java.io.File
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import vn.clevix.lichviet.MainActivity

/**
 * Trang "Giới thiệu" theo nội dung bản web 0.2.1 (PR #73), chỉnh cho Android: thông báo thật (không cần .ics), không có
 * mẹo "Thêm vào Màn hình chính", dữ liệu trong bộ nhớ riêng của ứng dụng. Ảnh chụp sáng/tối.
 */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [34], qualifiers = "w360dp-h800dp-xhdpi")
@OptIn(ExperimentalRoborazziApi::class)
class AboutScreenTest {
    @get:Rule val rule = createAndroidComposeRule<MainActivity>()
    private val shotDir = File(System.getProperty("lvhv.screenshots.dir") ?: "build/screenshots")

    private fun has(t: String, sub: Boolean = false) =
        assertTrue(t, rule.onAllNodesWithText(t, substring = sub).fetchSemanticsNodes().isNotEmpty())

    private fun shoot(theme: String) {
        rule.onNodeWithText("Về ứng dụng").performScrollTo()
        rule.waitForIdle()
        captureScreenRoboImage(File(shotDir, "and-gioi-thieu-1-ve-ung-dung-$theme.png").path)
        rule.onNodeWithText("Bắt đầu trong một phút").performScrollTo()
        rule.waitForIdle()
        captureScreenRoboImage(File(shotDir, "and-gioi-thieu-2-bat-dau-$theme.png").path)
        rule.onNodeWithText("Quyền riêng tư").performScrollTo()
        rule.waitForIdle()
        captureScreenRoboImage(File(shotDir, "and-gioi-thieu-3-rieng-tu-$theme.png").path)
    }

    @Test
    fun aboutContentAndScreenshots() {
        rule.onAllNodesWithText("Giới thiệu").onFirst().performClick()
        rule.waitForIdle()
        for (t in listOf(
            "Về ứng dụng", "Ứng dụng giúp bạn", "Bắt đầu trong một phút", "Mẹo để không lỡ ngày nào", "Cách tính lịch",
            "Quyền riêng tư", "Nguồn quy tắc xem ngày", "Nguồn ca dao, tục ngữ", "Phông chữ",
        )) has(t)
        for (t in listOf(
            "Tờ lịch mỗi ngày", "Chọn ngày tốt", "Đổi ngày", "Người thân", "Nhắc trước", "Xem cả nhà có việc gì",
            "Trang tưởng niệm ngày giỗ", "Thông tin báo tử",
        )) has(t, sub = true)
        has("cả khi ứng dụng đang đóng", sub = true)
        has("bộ nhớ riêng của ứng dụng trên điện thoại này", sub = true)
        has("gỡ cài đặt ứng dụng", sub = true)
        // Không có mẹo chỉ đúng cho bản web.
        assertTrue(rule.onAllNodesWithText("Thêm vào Màn hình chính", substring = true).fetchSemanticsNodes().isEmpty())
        assertTrue(rule.onAllNodesWithText("trình duyệt", substring = true).fetchSemanticsNodes().isEmpty())
        shoot("light")
        rule.onNodeWithContentDescription("Giao diện theo máy. Chạm để chuyển sang tối").performClick()
        rule.waitForIdle()
        shoot("dark")
    }
}
