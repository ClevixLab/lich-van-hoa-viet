@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package vn.clevix.lichviet.ui.people

import android.Manifest
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.ui.platform.LocalContext
import vn.clevix.lichviet.notify.Reminders
import androidx.compose.foundation.background
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
    /** B5/R14: nâng cấp dữ liệu lỗi → chế độ chỉ đọc (chỉ xem và xuất sao lưu); câu của lõi trong [readOnlyText]. */
    val readOnlyText: String? = null,
) {
    val readOnly: Boolean get() = readOnlyText != null
}

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

    Box(Modifier.fillMaxSize().background(LocalLich.current.wall)) {
        if (repo == null) {
            Page {
                Header(nav)
                NoticeBox(
                    "Không mở được dữ liệu trên máy: ${openError ?: "lỗi không rõ"}. Lịch vẫn dùng được bình thường.\n\n" +
                        STORE_SAFE_TEXT,
                    warn = true,
                )
            }
        } else {
            val mig = remember(repo) { runCatching { repo.migration() }.getOrNull() }
            val ro = if (mig?.state == "read_only") (mig.message ?: "Chưa nâng cấp được dữ liệu. Ứng dụng đang ở chế độ chỉ đọc — hãy xuất sao lưu.") else null
            val ctx = PeopleCtx(
                repo = repo, today = today, nav = nav, rev = rev, readOnlyText = ro,
                toast = { msg -> scope.launch { snackbar.currentSnackbarData?.dismiss(); snackbar.showSnackbar(msg) } },
                changed = { rev++; onChanged() },
                reducedMotion = reducedMotion,
                askNotify = askNotify,
            )
            // Mỗi trang (theo vị trí trong ngăn xếp) có trạng thái riêng: chuyển hồ sơ A → B không giữ dữ liệu của A.
            val r = nav.top
            androidx.compose.runtime.key(nav.stack.size, r) { when (r) {
                PRoute.Upcoming -> Page { Header(nav); ReadOnlyBanner(ctx); UpcomingScreen(ctx) }
                PRoute.Period -> Page { Header(nav); ReadOnlyBanner(ctx); PeriodScreen(ctx) }
                PRoute.Profiles -> Page { Header(nav); ReadOnlyBanner(ctx); ProfilesScreen(ctx) }
                PRoute.Backup -> Page { Header(nav); ReadOnlyBanner(ctx); BackupScreen(ctx) }
                PRoute.Settings -> Page { ReadOnlyBanner(ctx); SettingsScreen(ctx) }
                is PRoute.PersonPage -> Page { ReadOnlyBanner(ctx); PersonScreen(ctx, r.id) }
                is PRoute.AddPerson -> Page { if (ctx.readOnly) ReadOnlyForm(ctx) else PersonForm(ctx, null, r.deceased) }
                is PRoute.EditPerson -> Page { if (ctx.readOnly) ReadOnlyForm(ctx) else PersonForm(ctx, r.id, null) }
                is PRoute.EventEdit -> Page { if (ctx.readOnly) ReadOnlyForm(ctx) else EventForm(ctx, r.personId, r.eventId) }
                is PRoute.MarkDeceased -> Page { if (ctx.readOnly) ReadOnlyForm(ctx) else MarkDeceasedForm(ctx, r.id) }
                is PRoute.Memorial -> MemorialScreen(ctx, r.id)
            } }
        }
        SnackbarHost(snackbar, Modifier.align(Alignment.BottomCenter).padding(12.dp))
    }
}

/**
 * B5 (R14) — khi kho không mở được (ví dụ dữ liệu của bản mới hơn): dữ liệu không bị xóa. Ứng dụng không ghi gì vào thư
 * mục dữ liệu khi lỗi; cập nhật Android (`install -r`) không xóa dữ liệu; allowBackup=false. Chuyển đổi lỗi thì lõi mở
 * kho ở chế độ chỉ đọc ([ReadOnlyBanner]) — vẫn xem và xuất sao lưu được.
 */
const val STORE_SAFE_TEXT =
    "Dữ liệu người thân của bạn vẫn còn nguyên trên máy — ứng dụng không xóa gì. Đừng gỡ cài đặt ứng dụng (gỡ cài đặt sẽ " +
        "xóa dữ liệu). Hãy cập nhật ứng dụng lên bản mới nhất; nếu vẫn lỗi, hãy liên hệ Clevix Lab."

/** Thẻ đỏ chế độ chỉ đọc (B5 mục 3, `migration_status`): câu của lõi + lối tới "Tạo tệp sao lưu". */
@Composable
fun ReadOnlyBanner(x: PeopleCtx) {
    val t = x.readOnlyText ?: return
    val toBackup: @Composable () -> Unit = { PrimaryButton("Tạo tệp sao lưu…", { x.nav.tab(PRoute.Backup) }) }
    NoticeBox(
        "$t\n\nDữ liệu người thân vẫn còn nguyên trên máy. Trong lúc này chỉ xem được và tạo được tệp sao lưu; chưa thêm, " +
            "sửa hay xóa được. Đừng gỡ cài đặt ứng dụng.",
        warn = true,
        action = if (x.nav.top != PRoute.Backup) toBackup else null,
    )
}

/** Biểu mẫu khi đang chỉ đọc: không mở biểu mẫu, chỉ thẻ đỏ và lối quay lại. */
@Composable
private fun ReadOnlyForm(x: PeopleCtx) {
    BackLink("Quay lại") { x.nav.back() }
    ReadOnlyBanner(x)
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
    if (largeText()) {
        H1("Người thân")
        LinkButton("Cài đặt nhắc") { nav.go(PRoute.Settings) }
    } else {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            H1("Người thân", Modifier.weight(1f))
            LinkButton("Cài đặt nhắc") { nav.go(PRoute.Settings) }
        }
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
