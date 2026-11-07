@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package vn.clevix.lichviet.ui.people

import android.Manifest
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.ui.platform.LocalContext
import vn.clevix.lichviet.notify.Reminders
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import java.time.LocalDate
import kotlinx.coroutines.launch
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.ui.LocalLich

/** Các trang của mô-đun (08 §1.2). Không trang nào mang tên hay ngày của người (S4'). */
sealed interface PRoute {
    data object Upcoming : PRoute
    data object Period : PRoute
    data object Profiles : PRoute
    data object Backup : PRoute
    data object Settings : PRoute
    data class PersonPage(val id: String) : PRoute
    data class AddPerson(val deceased: Boolean) : PRoute
    data class EditPerson(val id: String) : PRoute
    data class EventEdit(val personId: String, val eventId: String?) : PRoute
    data class MarkDeceased(val id: String) : PRoute
    data class Memorial(val id: String) : PRoute
}

/** Ngăn xếp trang của mô-đun. */
@Stable
class PeopleNav(initial: PRoute = PRoute.Upcoming) {
    val stack = mutableStateListOf(initial)
    val top: PRoute get() = stack.last()
    fun go(r: PRoute) { stack.add(r) }
    fun tab(r: PRoute) { stack.clear(); stack.add(r) }
    fun back(): Boolean = if (stack.size > 1) { stack.removeAt(stack.lastIndex); true } else false
    /** Thay trang hiện tại (sau khi lưu: biểu mẫu → hồ sơ). */
    fun replace(r: PRoute) { stack[stack.lastIndex] = r }
    /** Về một trang đã có trong ngăn xếp (hoặc mở mới). */
    fun backTo(r: PRoute) {
        val i = stack.lastIndexOf(r)
        if (i >= 0) { while (stack.size > i + 1) stack.removeAt(stack.lastIndex) } else replace(r)
    }
}

/** Ngữ cảnh chung của các trang. [rev] tăng sau mỗi lần ghi để các trang đọc lại dữ liệu. */
class PeopleCtx(
    val repo: PeopleRepo,
    val today: LocalDate,
    val nav: PeopleNav,
    val rev: Int,
    val toast: (String) -> Unit,
    /** Gọi sau mỗi lần ghi (lưu, xóa, nhập sao lưu): tải lại + đặt lại chuỗi nhắc. */
    val changed: () -> Unit,
    val reducedMotion: Boolean,
    /** Hỏi quyền thông báo (Android 13+) — gọi khi người dùng bật nhắc lần đầu. */
    val askNotify: () -> Unit = {},
)

private val SEGMENTS = listOf(PRoute.Upcoming, PRoute.Period, PRoute.Profiles, PRoute.Backup)
private val SEGMENT_LABELS = listOf("Sắp tới", "Theo kỳ", "Hồ sơ", "Sao lưu")

/**
 * Mô-đun "Người thân" (thẻ thứ tư của thanh mục, 08 §1.1). [repo] = null khi không mở được kho ([openError]).
 */
@Composable
fun PeopleModule(
    repo: PeopleRepo?,
    openError: String?,
    today: LocalDate,
    nav: PeopleNav,
    onChanged: () -> Unit,
    reducedMotion: Boolean,
    snackbar: SnackbarHostState = remember { SnackbarHostState() },
) {
    var rev by remember { mutableIntStateOf(0) }
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    val permLauncher = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) { _ ->
        rev++
        onChanged()
    }
    val askNotify: () -> Unit = {
        if (Reminders.needsPermission(context)) {
            Reminders.markAsked(context)
            permLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
        }
    }
    BackHandler(enabled = nav.stack.size > 1) { nav.back() }

    Box(Modifier.fillMaxSize()) {
        if (repo == null) {
            Page {
                Header(nav)
                NoticeBox(
                    "Không mở được dữ liệu trên máy: ${openError ?: "lỗi không rõ"}. Lịch vẫn dùng được bình thường.",
                    warn = true,
                )
            }
        } else {
            val ctx = PeopleCtx(
                repo = repo, today = today, nav = nav, rev = rev,
                toast = { msg -> scope.launch { snackbar.currentSnackbarData?.dismiss(); snackbar.showSnackbar(msg) } },
                changed = { rev++; onChanged() },
                reducedMotion = reducedMotion,
                askNotify = askNotify,
            )
            when (val r = nav.top) {
                PRoute.Upcoming -> Page { Header(nav); UpcomingScreen(ctx) }
                PRoute.Period -> Page { Header(nav); PeriodScreen(ctx) }
                PRoute.Profiles -> Page { Header(nav); ProfilesScreen(ctx) }
                PRoute.Backup -> Page { Header(nav); BackupScreen(ctx) }
                PRoute.Settings -> Page { SettingsScreen(ctx) }
                is PRoute.PersonPage -> Page { PersonScreen(ctx, r.id) }
                is PRoute.AddPerson -> Page { PersonForm(ctx, null, r.deceased) }
                is PRoute.EditPerson -> Page { PersonForm(ctx, r.id, null) }
                is PRoute.EventEdit -> Page { EventForm(ctx, r.personId, r.eventId) }
                is PRoute.MarkDeceased -> Page { MarkDeceasedForm(ctx, r.id) }
                is PRoute.Memorial -> MemorialScreen(ctx, r.id)
            }
        }
        SnackbarHost(snackbar, Modifier.align(Alignment.BottomCenter).padding(12.dp))
    }
}

/** Khung trang: một cột cuộn, lề 16 dp, rộng tối đa 720 dp, căn giữa. */
@Composable
fun Page(content: @Composable ColumnScope.() -> Unit) {
    val scroll = rememberScrollState()
    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
        Column(
            Modifier.widthIn(max = 720.dp).fillMaxWidth().verticalScroll(scroll).padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
            content = content,
        )
    }
}

/** Tiêu đề mô-đun + thanh phân đoạn (Sắp tới · Theo kỳ · Hồ sơ · Sao lưu) + lối vào cài đặt nhắc. */
@Composable
private fun Header(nav: PeopleNav) {
    val c = LocalLich.current
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        H1("Người thân", Modifier.weight(1f))
        LinkButton("Cài đặt nhắc") { nav.go(PRoute.Settings) }
    }
    Text(
        "Ngày giỗ, sinh nhật và sự kiện của gia đình. Dữ liệu chỉ nằm trên máy này.",
        style = MaterialTheme.typography.bodyLarge, color = c.ink2,
    )
    val sel = SEGMENTS.indexOf(nav.top).coerceAtLeast(0)
    Choice(SEGMENT_LABELS, sel, { nav.tab(SEGMENTS[it]) })
}

/** Lối quay lại theo đích (TK-21): "‹ Về hồ sơ", "‹ Về danh sách hồ sơ"… */
@Composable
fun BackLink(text: String, onClick: () -> Unit) {
    LinkButton("‹ $text", onClick)
}
