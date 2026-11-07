package vn.clevix.lichviet.ui.people

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
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
}
