package vn.clevix.lichviet.ui.people

import android.Manifest
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.performClick
import com.github.takahirom.roborazzi.ExperimentalRoborazziApi
import com.github.takahirom.roborazzi.captureScreenRoboImage
import java.io.File
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.Shadows.shadowOf
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import vn.clevix.lichviet.MainActivity
import vn.clevix.lichviet.notify.PeopleStore
import vn.clevix.lichviet.people.Seed

/** TKN-A-02: ảnh toàn màn hình của ứng dụng thật (MainActivity) có thanh điều hướng đáy với thẻ "Người thân". */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [34], qualifiers = "w360dp-h800dp-xhdpi")
@OptIn(ExperimentalRoborazziApi::class)
class AppNavTest {
    @get:Rule val rule = createAndroidComposeRule<MainActivity>()
    private val shotDir = File(System.getProperty("lvhv.screenshots.dir") ?: "build/screenshots")

    @Test
    fun bottomNavWithPeopleTab() {
        shadowOf(rule.activity.application).grantPermissions(Manifest.permission.POST_NOTIFICATIONS)
        PeopleStore.reset()
        Seed.family(PeopleStore.get(rule.activity)!!)
        // Thanh đáy có đủ năm mục, "Người thân" là mục thứ tư.
        for (t in listOf("Tờ lịch", "Chọn ngày tốt", "Đổi ngày", "Người thân", "Giới thiệu")) {
            assertTrue(t, rule.onAllNodesWithText(t).fetchSemanticsNodes().isNotEmpty())
        }
        rule.onNodeWithText("Người thân").performClick()
        rule.waitForIdle()
        rule.onNodeWithText("30 ngày tới").assertExists()
        captureScreenRoboImage(File(shotDir, "and-39-thanh-dieu-huong-light.png").path)
        // Giao diện theo máy → tối.
        rule.onNodeWithContentDescription("Giao diện theo máy. Chạm để chuyển sang tối").performClick()
        rule.waitForIdle()
        captureScreenRoboImage(File(shotDir, "and-39-thanh-dieu-huong-dark.png").path)
        PeopleStore.reset()
    }
}
