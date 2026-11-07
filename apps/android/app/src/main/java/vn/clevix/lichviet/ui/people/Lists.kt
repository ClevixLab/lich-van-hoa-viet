@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package vn.clevix.lichviet.ui.people

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import java.text.Collator
import java.time.LocalDate
import java.time.temporal.ChronoUnit
import java.util.Locale
import vn.clevix.lichviet.notify.Reminders
import vn.clevix.lichviet.people.DateValue
import vn.clevix.lichviet.people.Lunar
import vn.clevix.lichviet.people.Occ
import vn.clevix.lichviet.people.PFmt
import vn.clevix.lichviet.people.Person
import vn.clevix.lichviet.people.ReminderMsg
import vn.clevix.lichviet.people.errText
import vn.clevix.lichviet.people.o
import vn.clevix.lichviet.ui.LichIcons
import vn.clevix.lichviet.ui.LocalLich

/** Hồ sơ người của một lần xuất hiện / lời nhắc: id người, hoặc người đầu tiên của sự kiện. */
fun personOfEntity(x: PeopleCtx, entityId: String, personIds: List<String> = emptyList()): String? {
    personIds.firstOrNull()?.let { return it }
    if (x.repo.person(entityId) != null) return entityId
    return runCatching { x.repo.events().firstOrNull { it.id == entityId }?.personIds?.firstOrNull() }.getOrNull()
}

/** Khung chào khi chưa có hồ sơ nào (08 §3.1). */
@Composable
fun Welcome(x: PeopleCtx) {
    PCard {
        Body(
            "Chưa có ai trong sổ. Thêm ông bà, cha mẹ đã khuất để được nhắc ngày giỗ theo âm lịch; thêm người đang " +
                "sống để nhớ sinh nhật và ngày kỷ niệm.",
        )
        AddButtons(x)
    }
}

@Composable
fun AddButtons(x: PeopleCtx) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        PrimaryButton("+ Thêm người đã mất", { x.nav.go(PRoute.AddPerson(true)) }, Modifier.fillMaxWidth())
        SecondaryButton("+ Thêm người đang sống", { x.nav.go(PRoute.AddPerson(false)) }, Modifier.fillMaxWidth())
    }
}

/** Thẻ nhắc hôm nay (08 §5.5): tiêu đề + dòng phụ của lõi; "Xem", "Đã biết"; đúng ngày giỗ có "Mở trang tưởng niệm". */
@Composable
fun ReminderCard(x: PeopleCtx, r: ReminderMsg, onDone: () -> Unit) {
    val c = LocalLich.current
    PCard {
        Text(r.title, style = MaterialTheme.typography.titleMedium, color = c.ink)
        Text(r.body, style = MaterialTheme.typography.bodyLarge, color = c.ink2)
        val pid = personOfEntity(x, r.entityId)
        if (r.cls == "gio" && r.date == x.today && pid != null) {
            PrimaryButton("Mở trang tưởng niệm", { x.nav.go(PRoute.Memorial(pid)) }, Modifier.fillMaxWidth())
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            if (pid != null) SecondaryButton("Xem", { x.nav.go(PRoute.PersonPage(pid)) })
            SecondaryButton("Đã biết", onDone)
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Sắp tới — 08 §3.1
// -------------------------------------------------------------------------------------------------

@Composable
fun UpcomingScreen(x: PeopleCtx) {
    val ctx = LocalContext.current
    val data = remember(x.rev, x.today) {
        runCatching { Triple(x.repo.persons(), x.repo.upcoming(x.today, 30), x.repo.todayReminders(x.today)) }
    }
    val nag = remember(x.rev) { runCatching { x.repo.backupDue() }.getOrDefault(false) }
    val hidden = remember { mutableStateListOf<String>() }
    data.onFailure { NoticeBox("Không mở được dữ liệu trên máy: ${errText(it)}. Lịch vẫn dùng được bình thường.", warn = true); return }
    val (persons, items, reminders) = data.getOrThrow()
    if (persons.isEmpty()) { Welcome(x); return }

    val shown = reminders.filter { it.id !in hidden }
    shown.take(3).forEach { r -> ReminderCard(x, r) { hidden.add(r.id) } }
    if (shown.size > 3) Fine("… và ${shown.size - 3} lời nhắc khác")

    if (nag) {
        NoticeBox(
            "Dữ liệu người thân đã thay đổi lâu mà chưa sao lưu. Nếu mất máy hoặc cài lại ứng dụng, sẽ không lấy lại được.",
            action = { PrimaryButton("Sao lưu ngay", { x.nav.tab(PRoute.Backup) }) },
        )
    }
    val allowed = remember(x.rev) { Reminders.notificationsAllowed(ctx) }
    if (!allowed) {
        NoticeBox(
            "Thông báo của ứng dụng đang tắt — điện thoại sẽ không tự nhắc. Lời nhắc vẫn hiện ở đây khi bạn mở ứng dụng.",
            action = { SecondaryButton("Cài đặt nhắc", { x.nav.go(PRoute.Settings) }) },
        )
    }

    PCard(title = "30 ngày tới") {
        if (items.isEmpty()) {
            Body("Trong 30 ngày tới không có ngày giỗ, sinh nhật hay sự kiện nào.")
            LinkButton("Xem theo tháng, quý, năm") { x.nav.tab(PRoute.Period) }
        } else {
            items.forEach { o ->
                OccRow(o, x.today) { personOfEntity(x, o.entityId, o.personIds)?.let { x.nav.go(PRoute.PersonPage(it)) } }
                if (o.cls == "gio" && o.date == x.today) {
                    val pid = personOfEntity(x, o.entityId, o.personIds)
                    if (pid != null) SecondaryButton("Mở trang tưởng niệm", { x.nav.go(PRoute.Memorial(pid)) }, Modifier.fillMaxWidth())
                }
            }
        }
    }
    AddButtons(x)
}

// -------------------------------------------------------------------------------------------------
// Theo kỳ — 08 §3.2 (R13)
// -------------------------------------------------------------------------------------------------

private val KINDS = listOf("tuan", "thang", "quy", "nam")
private val FILTERS = listOf("tat_ca", "da_mat", "dang_song")
private val FILTER_LABELS = listOf("Tất cả", "Người đã mất", "Người đang sống")

@Composable
fun PeriodScreen(x: PeopleCtx) {
    val c = LocalLich.current
    var kind by rememberSaveable { mutableStateOf("thang") }
    var filter by rememberSaveable { mutableStateOf("tat_ca") }
    var anchorEpoch by rememberSaveable { mutableStateOf(x.today.toEpochDay()) }
    val anchor = LocalDate.ofEpochDay(anchorEpoch)
    val persons = remember(x.rev) { runCatching { x.repo.persons() }.getOrDefault(emptyList()) }
    if (persons.isEmpty()) { Welcome(x); return }
    val res = remember(x.rev, kind, anchorEpoch, filter) { runCatching { x.repo.period(kind, anchor, filter) } }

    Choice(listOf("Tuần", "Tháng", "Quý", "Năm"), KINDS.indexOf(kind), { kind = KINDS[it] })
    Choice(FILTER_LABELS, FILTERS.indexOf(filter), { filter = FILTERS[it] })
    when (filter) {
        "da_mat" -> Fine("Chỉ có ngày giỗ (và tiên thường nếu bật nhắc). Không nhắc sinh nhật và sự kiện của người đã mất.")
        "dang_song" -> Fine("Sinh nhật và sự kiện của người đang sống.")
    }
    res.onFailure { NoticeBox(errText(it), warn = true); return }
    val (from, to, items) = res.getOrThrow()

    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        RoundBtn(LichIcons.Prev, "Kỳ trước") { anchorEpoch = vn.clevix.lichviet.people.PeopleRepo.prev(kind, anchor).toEpochDay() }
        Text(
            PFmt.periodTitle(kind, from, to),
            style = MaterialTheme.typography.titleMedium, color = c.ink, textAlign = TextAlign.Center,
            modifier = Modifier.weight(1f).padding(horizontal = 6.dp).semantics { liveRegion = LiveRegionMode.Polite },
        )
        RoundBtn(LichIcons.Next, "Kỳ sau") { anchorEpoch = vn.clevix.lichviet.people.PeopleRepo.next(kind, anchor).toEpochDay() }
    }
    val hasToday = !x.today.isBefore(from) && !x.today.isAfter(to)
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
        Text("${items.size} mục", style = MaterialTheme.typography.bodyLarge, color = c.ink2, modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite })
        if (hasToday) Fine("Đang xem kỳ có hôm nay") else SecondaryButton("Hôm nay", { anchorEpoch = x.today.toEpochDay() })
    }
    if (items.isEmpty()) {
        val f = if (filter != "tat_ca") " (đang lọc: ${FILTER_LABELS[FILTERS.indexOf(filter)]})" else ""
        Body("Không có ngày giỗ, sinh nhật hay sự kiện nào trong ${PFmt.periodThis(kind)}.$f")
        return
    }
    val open: (Occ) -> Unit = { o -> personOfEntity(x, o.entityId, o.personIds)?.let { x.nav.go(PRoute.PersonPage(it)) } }
    if (kind == "quy" || kind == "nam") {
        // R-UX-2: chia nhóm theo tháng; tháng không có mục thì không có tiêu đề.
        items.groupBy { it.date.withDayOfMonth(1) }.forEach { (m, list) ->
            H3("Tháng ${m.monthValue}/${m.year}")
            list.forEach { o -> OccRow(o, x.today) { open(o) } }
        }
    } else {
        items.forEach { o -> OccRow(o, x.today) { open(o) } }
    }
}

@Composable
fun RoundBtn(icon: androidx.compose.ui.graphics.vector.ImageVector, label: String, onClick: () -> Unit) {
    val c = LocalLich.current
    FilledIconButton(
        onClick = onClick, shape = CircleShape, modifier = Modifier.size(52.dp),
        colors = IconButtonDefaults.filledIconButtonColors(containerColor = c.paper, contentColor = c.ink),
    ) { Icon(icon, label) }
}

// -------------------------------------------------------------------------------------------------
// Hồ sơ (danh sách) — 08 §3.3
// -------------------------------------------------------------------------------------------------

private val VI: Collator = Collator.getInstance(Locale.forLanguageTag("vi"))

@Composable
fun ProfilesScreen(x: PeopleCtx) {
    val data = remember(x.rev, x.today) {
        runCatching {
            val ps = x.repo.persons().sortedWith { a, b -> VI.compare(a.sortKey, b.sortKey) }
            val next = x.repo.upcoming(x.today, 400).filter { it.cls == "gio" }
            Pair(ps, next)
        }
    }
    data.onFailure { NoticeBox(errText(it), warn = true); return }
    val (ps, next) = data.getOrThrow()
    val dead = ps.filter { it.deceased }
    val live = ps.filter { !it.deceased }
    PCard(title = "Người đã mất (${dead.size})") {
        SecondaryButton("+ Thêm", { x.nav.go(PRoute.AddPerson(true)) })
        if (dead.isEmpty()) Body("Chưa có ai.")
        dead.forEach { p -> PersonRow(x, p, deceasedLine(p, next.firstOrNull { p.id in it.personIds || it.entityId == p.id }, x.today)) }
    }
    PCard(title = "Người đang sống (${live.size})") {
        SecondaryButton("+ Thêm", { x.nav.go(PRoute.AddPerson(false)) })
        if (live.isEmpty()) Body("Chưa có ai.")
        live.forEach { p -> PersonRow(x, p, livingLine(x, p)) }
    }
}

private fun deceasedLine(p: Person, next: Occ?, today: LocalDate): String {
    val l = Lunar.of(p.death?.o("lunar"))
    val unknownYear = DateValue.of(p.death?.o("input"))?.precision == "month_day"
    val sb = StringBuilder()
    if (l != null) sb.append("Giỗ ${l.dm} âm lịch")
    if (next != null) {
        val d = ChronoUnit.DAYS.between(today, next.date)
        sb.append(" · lần tới ${PFmt.dmy(next.date)} (").append(if (d == 0L) "hôm nay" else "còn $d ngày").append(")")
    }
    if (unknownYear) sb.append(" (không rõ năm mất)")
    return sb.toString()
}

private fun livingLine(x: PeopleCtx, p: Person): String {
    val v = DateValue.of(p.birth?.o("input")) ?: return ""
    if (v.precision == "year") return "Sinh năm ${v.year}"
    val age = runCatching { x.repo.summary(p, x.today).tuoiDuong }.getOrNull()
    return "Sinh ${PFmt.dateValue(v)}" + (age?.let { " · $it tuổi" } ?: "")
}

@Composable
private fun PersonRow(x: PeopleCtx, p: Person, line: String) {
    val c = LocalLich.current
    Row(
        Modifier.fillMaxWidth().heightIn(min = 64.dp)
            .clickable(role = Role.Button) { x.nav.go(PRoute.PersonPage(p.id)) }
            .padding(vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Initial(p.hoTen)
        Spacer(Modifier.width(12.dp))
        Column(Modifier.weight(1f)) {
            Text(p.displayName, style = MaterialTheme.typography.titleMedium, color = c.ink)
            if (p.quanHe.isNotBlank()) Text(p.quanHe, style = MaterialTheme.typography.bodyMedium, color = c.ink2)
            if (line.isNotBlank()) Text(line, style = MaterialTheme.typography.bodyMedium, color = c.ink3)
        }
    }
}
