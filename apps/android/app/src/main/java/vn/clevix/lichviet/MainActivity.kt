package vn.clevix.lichviet

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.provider.Settings
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import java.time.LocalDate
import kotlinx.coroutines.launch
import vn.clevix.lichviet.data.Lich
import vn.clevix.lichviet.ui.AboutScreen
import vn.clevix.lichviet.ui.ConvertScreen
import vn.clevix.lichviet.ui.GoodDaysScreen
import vn.clevix.lichviet.ui.LichIcons
import vn.clevix.lichviet.ui.LichTheme
import vn.clevix.lichviet.ui.LocalLich
import vn.clevix.lichviet.ui.MonthDialog
import vn.clevix.lichviet.ui.RulesScreen
import vn.clevix.lichviet.ui.SheetPager
import vn.clevix.lichviet.ui.ThemeMode
import vn.clevix.lichviet.ui.UpdateReminder
import vn.clevix.lichviet.update.Updates
import vn.clevix.lichviet.notify.PeopleStore
import vn.clevix.lichviet.notify.Reminders
import vn.clevix.lichviet.ui.people.PRoute
import vn.clevix.lichviet.ui.people.PeopleModule
import vn.clevix.lichviet.ui.people.PeopleNav
import vn.clevix.lichviet.ui.people.AttachmentOpener
import vn.clevix.lichviet.ui.people.cleanShareCache

/** Tùy chọn hiển thị: chỉ cỡ chữ và chế độ sáng/tối (giống PWA). Không lưu dữ liệu khác. */
private object Prefs {
    private const val FILE = "hien_thi"
    fun size(ctx: Context): Int = ctx.getSharedPreferences(FILE, Context.MODE_PRIVATE).getInt("size", 1)
    fun theme(ctx: Context): ThemeMode =
        ThemeMode.entries.getOrElse(ctx.getSharedPreferences(FILE, Context.MODE_PRIVATE).getInt("theme", 0)) { ThemeMode.System }
    fun save(ctx: Context, size: Int, theme: ThemeMode) {
        ctx.getSharedPreferences(FILE, Context.MODE_PRIVATE).edit().putInt("size", size).putInt("theme", theme.ordinal).apply()
    }
}

private enum class Tab(val label: String, val icon: ImageVector) {
    Sheet("Tờ lịch", LichIcons.Sheet),
    Good("Chọn ngày tốt", LichIcons.Star),
    Convert("Đổi ngày", LichIcons.Swap),
    People("Người thân", LichIcons.People),
    About("Giới thiệu", LichIcons.Info),
}

private val SCALES = listOf(1f, 1.15f, 1.32f)

class MainActivity : ComponentActivity() {
    /** Hồ sơ cần mở khi chạm vào thông báo (id hồ sơ hoặc sự kiện). */
    private val openRequest = mutableStateOf<String?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        val ctx = applicationContext
        // BM2-19: chỉ nhận yêu cầu mở hồ sơ kèm mã của chính ứng dụng (từ thông báo nhắc).
        if (savedInstanceState == null) openRequest.value = Reminders.acceptOpen(ctx, intent)
        // BM2-16: tệp .ics tạm của lần chia sẻ trước (có tên, ngày giỗ) không nằm lại trong cache.
        cleanShareCache(ctx, 0L)
        AttachmentOpener.cleanup(ctx)
        setContent {
            var size by rememberSaveable { mutableIntStateOf(Prefs.size(ctx).coerceIn(1, 3)) }
            var theme by rememberSaveable { mutableStateOf(Prefs.theme(ctx)) }
            LaunchedEffect(size, theme) { Prefs.save(ctx, size, theme) }
            LichTheme(theme, SCALES[size - 1]) {
                App(
                    size = size,
                    onSize = { size = it },
                    onToggleTheme = {
                        theme = when (theme) {
                            ThemeMode.System -> ThemeMode.Dark
                            ThemeMode.Dark -> ThemeMode.Light
                            ThemeMode.Light -> ThemeMode.System
                        }
                    },
                    theme = theme,
                    openRequest = openRequest.value,
                    onOpenHandled = { openRequest.value = null },
                )
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        Reminders.acceptOpen(applicationContext, intent)?.let { openRequest.value = it }
    }

    override fun onResume() {
        super.onResume()
        // Mở ứng dụng: đặt lại chuỗi báo thức nhắc việc (02 §7.3). Chưa có sổ người thân → không làm gì.
        Reminders.rearmAsync(this)
        // Quay lại sau khi chia sẻ .ics: ứng dụng nhận đã đọc xong → xóa tệp tạm (chừa 30 s cho ứng dụng đọc chậm).
        cleanShareCache(applicationContext, 30_000L)
        // Quay lại sau khi mở PDF đính kèm bằng ứng dụng khác: thu hồi quyền đọc, xóa bản tạm (RA-SOAT-BAO-MAT-V02 §5.3).
        AttachmentOpener.cleanup(applicationContext)
    }
}

/** Người dùng bật "Giảm chuyển động" (tắt hiệu ứng) trong cài đặt Android. */
private fun reducedMotion(ctx: Context): Boolean =
    Settings.Global.getFloat(ctx.contentResolver, Settings.Global.ANIMATOR_DURATION_SCALE, 1f) == 0f

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun App(
    size: Int,
    onSize: (Int) -> Unit,
    onToggleTheme: () -> Unit,
    theme: ThemeMode,
    openRequest: String? = null,
    onOpenHandled: () -> Unit = {},
) {
    val c = LocalLich.current
    var today by remember { mutableStateOf(LocalDate.now()) }
    // Cập nhật "hôm nay" khi người dùng quay lại ứng dụng (qua nửa đêm).
    val owner = LocalLifecycleOwner.current
    DisposableEffect(owner) {
        val obs = LifecycleEventObserver { _, e -> if (e == Lifecycle.Event.ON_RESUME) today = LocalDate.now() }
        owner.lifecycle.addObserver(obs)
        onDispose { owner.lifecycle.removeObserver(obs) }
    }
    val ctx = LocalContext.current
    // Nhắc cập nhật (không mạng): bản dựng hơn 60 ngày và chưa bấm "Để sau" trong 30 ngày qua.
    var remind by remember(today) { mutableStateOf(Updates.shouldRemind(ctx, today)) }
    var tab by rememberSaveable { mutableStateOf(Tab.Sheet) }
    var rules by rememberSaveable { mutableStateOf(false) }
    var monthOpen by rememberSaveable { mutableStateOf(false) }
    val pager = rememberPagerState(initialPage = Lich.pageOf(today)) { Lich.PAGE_COUNT }
    val scope = rememberCoroutineScope()
    val current = Lich.dateOfPage(pager.currentPage)

    fun openDay(d: LocalDate) {
        tab = Tab.Sheet
        rules = false
        scope.launch { pager.scrollToPage(Lich.pageOf(d)) }
    }

    // Mô-đun Người thân: mở kho khi người dùng vào thẻ (hoặc khi đã có sổ trên máy).
    val peopleNav = remember { PeopleNav() }
    var peopleOpened by rememberSaveable { mutableStateOf(PeopleStore.exists(ctx)) }
    LaunchedEffect(tab) { if (tab == Tab.People) peopleOpened = true }
    val peopleRepo = remember(peopleOpened) { if (peopleOpened) PeopleStore.get(ctx) else null }
    LaunchedEffect(openRequest) {
        val id = openRequest ?: return@LaunchedEffect
        val repo = PeopleStore.get(ctx)
        val pid = repo?.let { r ->
            if (r.person(id) != null) id else runCatching { r.events().firstOrNull { it.id == id }?.personIds?.firstOrNull() }.getOrNull()
        }
        tab = Tab.People
        peopleOpened = true
        peopleNav.tab(PRoute.Upcoming)
        if (pid != null) peopleNav.go(PRoute.PersonPage(pid))
        onOpenHandled()
    }

    BackHandler(enabled = rules || tab != Tab.Sheet) {
        if (rules) rules = false else tab = Tab.Sheet
    }

    Scaffold(
        containerColor = c.wall,
        topBar = {
            Column(Modifier.background(c.wall).statusBarsPadding().padding(top = 4.dp)) {
                Row(
                    Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 6.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text("Lịch Việt", style = MaterialTheme.typography.titleLarge, color = c.ink, modifier = Modifier.weight(1f))
                    SingleChoiceSegmentedButtonRow {
                        listOf("A", "A+", "A++").forEachIndexed { i, t ->
                            SegmentedButton(
                                selected = size == i + 1,
                                onClick = { onSize(i + 1) },
                                shape = SegmentedButtonDefaults.itemShape(i, 3),
                                icon = {},
                                label = { Text(t, fontSize = 14.sp) },
                            )
                        }
                    }
                    Spacer(Modifier.width(6.dp))
                    FilledIconButton(
                        onClick = onToggleTheme,
                        colors = IconButtonDefaults.filledIconButtonColors(containerColor = c.paper, contentColor = c.ink),
                    ) {
                        Icon(
                            LichIcons.Contrast,
                            when (theme) {
                                ThemeMode.System -> "Giao diện theo máy. Chạm để chuyển sang tối"
                                ThemeMode.Dark -> "Giao diện tối. Chạm để chuyển sang sáng"
                                ThemeMode.Light -> "Giao diện sáng. Chạm để theo máy"
                            },
                        )
                    }
                }
                if (tab == Tab.Sheet) {
                    Row(
                        Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.SpaceBetween,
                    ) {
                        RoundNav(LichIcons.Prev, "Ngày trước", pager.currentPage > 0) {
                            scope.launch { pager.animateScrollToPage(pager.currentPage - 1) }
                        }
                        Button(
                            onClick = { scope.launch { pager.animateScrollToPage(Lich.pageOf(today)) } },
                            enabled = current != today,
                            colors = ButtonDefaults.buttonColors(containerColor = c.redDeep, contentColor = Color.White),
                        ) { Text("Hôm nay", style = MaterialTheme.typography.titleMedium) }
                        RoundNav(LichIcons.Next, "Ngày sau", pager.currentPage < Lich.PAGE_COUNT - 1) {
                            scope.launch { pager.animateScrollToPage(pager.currentPage + 1) }
                        }
                    }
                }
            }
        },
        bottomBar = {
            NavigationBar(containerColor = c.paper) {
                Tab.entries.forEach { t ->
                    NavigationBarItem(
                        selected = tab == t,
                        onClick = { tab = t; rules = false },
                        icon = { Icon(t.icon, null, modifier = Modifier.size(26.dp)) },
                        label = { Text(t.label, textAlign = TextAlign.Center, maxLines = 2) },
                        colors = NavigationBarItemDefaults.colors(
                            selectedIconColor = c.redInk,
                            selectedTextColor = c.redInk,
                            indicatorColor = c.redTint,
                            unselectedIconColor = c.ink2,
                            unselectedTextColor = c.ink2,
                        ),
                    )
                }
            }
        },
    ) { pad ->
        Box(Modifier.padding(pad).fillMaxSize()) {
            when (tab) {
                Tab.Sheet -> Column(Modifier.fillMaxSize()) {
                    if (remind) {
                        UpdateReminder(
                            onCheck = { Updates.openStore(ctx) },
                            onLater = { Updates.snooze(ctx, today); remind = false },
                        )
                    }
                    Box(Modifier.weight(1f)) { SheetPager(pager, today, SCALES[size - 1]) { monthOpen = true } }
                }
                Tab.Good -> GoodDaysScreen(today, ::openDay)
                Tab.Convert -> ConvertScreen(current, today, ::openDay)
                Tab.People -> PeopleModule(
                    repo = peopleRepo,
                    openError = PeopleStore.openError(),
                    today = today,
                    nav = peopleNav,
                    onChanged = { Reminders.rearmAsync(ctx) },
                    reducedMotion = reducedMotion(ctx),
                )
                Tab.About -> if (rules) RulesScreen { rules = false } else AboutScreen { rules = true }
            }
        }
    }

    if (monthOpen) {
        MonthDialog(
            selected = current,
            today = today,
            onPick = { d -> monthOpen = false; openDay(d) },
            onDismiss = { monthOpen = false },
        )
    }
}

@Composable
private fun RoundNav(icon: ImageVector, label: String, enabled: Boolean, onClick: () -> Unit) {
    val c = LocalLich.current
    FilledIconButton(
        onClick = onClick,
        enabled = enabled,
        shape = CircleShape,
        modifier = Modifier.size(52.dp),
        colors = IconButtonDefaults.filledIconButtonColors(containerColor = c.paper, contentColor = c.ink),
    ) { Icon(icon, label) }
}
