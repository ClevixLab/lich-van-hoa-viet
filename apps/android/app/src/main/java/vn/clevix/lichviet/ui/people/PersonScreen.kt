@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package vn.clevix.lichviet.ui.people

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import vn.clevix.lichviet.people.DateValue
import vn.clevix.lichviet.people.EventRec
import vn.clevix.lichviet.people.Lunar
import vn.clevix.lichviet.people.PFmt
import vn.clevix.lichviet.people.Person
import vn.clevix.lichviet.people.b
import vn.clevix.lichviet.people.errText
import vn.clevix.lichviet.people.i
import vn.clevix.lichviet.people.list
import vn.clevix.lichviet.people.o
import vn.clevix.lichviet.people.objs
import vn.clevix.lichviet.people.s
import vn.clevix.lichviet.people.sn
import vn.clevix.lichviet.people.strs
import vn.clevix.lichviet.ui.LocalLich

/** Hồ sơ một người — 08 §3.6 (+ §12f B4, §12g thông tin báo tử). */
@Composable
fun PersonScreen(x: PeopleCtx, id: String) {
    val c = LocalLich.current
    val data = remember(x.rev, id) {
        runCatching {
            val p = x.repo.person(id)
            if (p == null) null else {
                val evs = x.repo.events().filter { id in it.personIds }
                val types = x.repo.eventTypes().associate { it.id to it.ten }
                val next = x.repo.upcoming(x.today, 366).filter { id in it.personIds || it.entityId == id }.take(5)
                val sum = x.repo.summary(p, x.today)
                PersonData(p, evs, types, next, sum)
            }
        }
    }
    val idx = remember(x.rev, x.today) { occIndex(x) }
    BackLink("Về danh sách hồ sơ") { x.nav.backTo(PRoute.Profiles) }
    data.onFailure { NoticeBox(errText(it), warn = true); return }
    val d = data.getOrNull()
    if (d == null) {
        Body("Không tìm thấy hồ sơ này (có thể đã bị xóa).")
        LinkButton("Về Hồ sơ") { x.nav.tab(PRoute.Profiles) }
        return
    }
    val p = d.p
    var confirmDelete by rememberSaveable { mutableStateOf(false) }
    var delEvent by rememberSaveable { mutableStateOf<String?>(null) }

    // Đầu trang: ảnh, tên, quan hệ, tên hiệu, nút.
    Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        if (p.deceased) {
            val img = rememberPhoto(remember(p.photoId) { p.photoId?.let { x.repo.photo(it) } })
            OvalPhoto(img, p.hoTen)
        }
        H1(p.displayName)
        if (p.quanHe.isNotBlank()) Text(p.quanHe, style = MaterialTheme.typography.bodyLarge, color = c.ink2)
        if (p.tenHieu.isNotBlank()) Fine("Tên hiệu / pháp danh: ${p.tenHieu}")
    }
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
        if (p.deceased) {
            PrimaryButton("Tưởng niệm", { x.nav.go(PRoute.Memorial(p.id)) }, Modifier.weight(1f))
            if (!x.readOnly) SecondaryButton("Sửa", { x.nav.go(PRoute.EditPerson(p.id)) }, Modifier.weight(1f))
        } else if (!x.readOnly) {
            // "Sửa" trước, "Cập nhật: đã mất" sau — thống nhất mọi nền tảng (TKN-X-01).
            SecondaryButton("Sửa", { x.nav.go(PRoute.EditPerson(p.id)) }, Modifier.weight(1f))
            SecondaryButton("Cập nhật: đã mất", { x.nav.go(PRoute.MarkDeceased(p.id)) }, Modifier.weight(1f))
        }
    }

    PCard(title = "Lần tới") {
        if (d.next.isEmpty()) Body("Không có lần nào trong 12 tháng tới.")
        d.next.forEach { OccRow(it, x.today, null, idx) }
    }

    PCard(title = "Thông tin") { InfoTable(p, d.sum) }

    // Thông tin báo tử: chỉ ở đây (R-PRIV-2), thu gọn.
    val n = p.notice
    // BM2-04: thông tin báo tử trên màn hình → che ảnh đa nhiệm.
    SecureScreen(p.deceased && n != null && noticeHasContent(n))
    if (p.deceased && n != null && noticeHasContent(n)) NoticeView(x, n)

    PCard(title = "Sự kiện") {
        if (!x.readOnly) SecondaryButton("+ Thêm sự kiện", { x.nav.go(PRoute.EventEdit(p.id, null)) })
        if (d.events.isEmpty()) Body("Chưa có sự kiện nào.")
        d.events.forEach { e -> EventLine(e, d.types[e.typeId] ?: "Sự kiện", { x.nav.go(PRoute.EventEdit(p.id, e.id)) }, { delEvent = e.id }, !x.readOnly) }
        if (p.deceased) Fine("Không nhắc sinh nhật và sự kiện của người đã mất — chỉ nhắc ngày giỗ. Thông tin vẫn được lưu và sao lưu.")
    }

    PCard(title = "Nhắc") {
        val rem = p.reminders
        if (p.deceased) {
            Body(ruleText(rem?.o("gio"), true))
        } else {
            Body("Sinh nhật: " + ruleText(rem?.o("sinh_nhat"), false))
            if (p.raw.b("sinh_nhat_am")) Body("Sinh nhật âm lịch: " + ruleText(rem?.o("sinh_nhat_am"), false))
        }
    }

    if (!x.readOnly) SecondaryButton("Xóa hồ sơ", { confirmDelete = true }, Modifier.fillMaxWidth(), danger = true)

    if (confirmDelete) {
        ConfirmDialog(
            title = "Xóa hồ sơ của ${p.displayName}?",
            confirm = "Xóa hồ sơ", dismiss = "Giữ lại", danger = true,
            onDismiss = { confirmDelete = false },
            onConfirm = {
                confirmDelete = false
                try {
                    x.repo.deletePerson(p.id)
                    x.changed()
                    x.nav.tab(PRoute.Profiles)
                    x.toast("Đã xóa hồ sơ.")
                } catch (e: Exception) {
                    x.toast("Chưa xóa được: ${errText(e)}")
                }
            },
        ) {
            Body(
                (if (p.deceased) "Ngày giỗ, lời nhắc và ảnh của người này sẽ bị xóa khỏi máy. " else "Lời nhắc của người này sẽ bị xóa khỏi máy. ") +
                    "Sự kiện gắn với người này vẫn còn nếu có người khác cùng tham gia. Không hoàn tác được.",
            )
        }
    }
    delEvent?.let { eid ->
        val e = d.events.firstOrNull { it.id == eid }
        if (e == null) { delEvent = null; return@let }
        val name = e.tieuDe.ifBlank { d.types[e.typeId] ?: "Sự kiện" }
        ConfirmDialog(
            title = "Xóa sự kiện \"$name\"?",
            confirm = "Xóa sự kiện", dismiss = "Giữ lại", danger = true,
            onDismiss = { delEvent = null },
            onConfirm = {
                delEvent = null
                try { x.repo.deleteEvent(eid); x.changed(); x.toast("Đã xóa sự kiện.") } catch (ex: Exception) { x.toast(errText(ex)) }
            },
        ) { Body("Không hoàn tác được.") }
    }
}

private class PersonData(
    val p: Person,
    val events: List<EventRec>,
    val types: Map<String, String>,
    val next: List<vn.clevix.lichviet.people.Occ>,
    val sum: vn.clevix.lichviet.people.Summary,
)

@Composable
private fun InfoTable(p: Person, sum: vn.clevix.lichviet.people.Summary) {
    val death = p.death
    if (p.deceased && death != null) {
        val input = DateValue.of(death.o("input"))
        if (input != null) InfoLine("Ngày mất (đã nhập)", PFmt.dateValue(input))
        val l = Lunar.of(death.o("lunar"))
        if (l != null) {
            val src = if (death.s("lunar_source") == "converted") "đổi từ ngày dương" else "nhập theo âm lịch"
            val yr = l.year?.let { " ${vn.clevix.lichviet.people.PeopleRepo.canChi(it)} ($it)" } ?: ""
            InfoLine("Âm lịch", "${l.dm}$yr ($src)")
        }
        // Dòng "Dương lịch" chỉ khi nhập theo âm lịch (§12 mục 2).
        val sol = death.o("solar")
        if (input?.lunar == true && sol != null) {
            val dd = java.time.LocalDate.of(sol.i("y") ?: 2000, sol.i("m") ?: 1, sol.i("d") ?: 1)
            InfoLine("Dương lịch", PFmt.full(dd))
        }
        sum.tuoiMatText?.let { InfoLine("Tuổi", it) }
    }
    p.gioiTinhText?.let { InfoLine("Giới tính", it) }
    val b = p.birth
    val bi = DateValue.of(b?.o("input"))
    if (bi != null) InfoLine("Ngày sinh", PFmt.dateValue(bi))
    val canChi = b?.sn("can_chi")
    val ly = b?.i("lunar_year")
    if (canChi != null && ly != null) InfoLine("Năm âm sinh", "$ly — $canChi" + if (b?.b("lunar_year_approx") == true) " (khoảng)" else "")
    if (!p.deceased && sum.tuoiDuong != null) InfoLine("Tuổi", "${sum.tuoiDuong}" + (sum.tuoiAm?.let { " (tuổi âm $it)" } ?: ""))
    b?.s("noi_sinh")?.takeIf { it.isNotBlank() }?.let { InfoLine("Nơi sinh", it) }
    if (p.deceased && p.noiAnTang.isNotBlank()) InfoLine("Nơi an táng", p.noiAnTang)
    // 08 §16.2: "Ảnh mâm lễ: Đã có" / "Ảnh mâm lễ: Chưa có".
    if (p.deceased) InfoLine("Ảnh mâm lễ", if (p.memorial?.sn("tray_photo_id") != null) "Đã có" else "Chưa có")
    if (p.ghiChu.isNotBlank()) InfoLine("Ghi chú", p.ghiChu)
}

fun noticeHasContent(n: Map<String, Any?>): Boolean =
    listOf("reported_by", "place", "reason", "note").any { !n.sn(it).isNullOrBlank() } ||
        n.o("notice_date") != null || n.objs("attachments").isNotEmpty()

/** Khối thu gọn "Thông tin báo tử" (§12g): chỉ những dòng có dữ liệu; ảnh không cắt. */
@Composable
private fun NoticeView(x: PeopleCtx, n: Map<String, Any?>) {
    var open by rememberSaveable { mutableStateOf(false) }
    PCard {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            H2("Thông tin báo tử", Modifier.weight(1f))
            LinkButton(if (open) "Thu gọn ▴" else "Xem ▾") { open = !open }
        }
        if (open) {
            n.sn("reported_by")?.let { InfoLine("Người báo tin", it) }
            n.sn("place")?.let { InfoLine("Mất tại", it) }
            DateValue.of(n.o("notice_date"))?.let { InfoLine("Ngày nhận tin", PFmt.dateValue(it)) }
            n.sn("reason")?.let { InfoLine("Lý do", it) }
            n.sn("note")?.let { InfoLine("Ghi chú", it) }
            AttachmentView(x.repo, n.objs("attachments"))
        }
    }
}

@Composable
private fun EventLine(e: EventRec, typeName: String, onEdit: () -> Unit, onDelete: () -> Unit, editable: Boolean = true) {
    val title = e.tieuDe.ifBlank { typeName }
    val date = e.date?.let { PFmt.dateValue(it) } ?: ""
    Column(Modifier.fillMaxWidth()) {
        Body("$title · $date · ${if (e.recurring) "hằng năm" else "một lần"}")
        if (editable) Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            SecondaryButton("Sửa", onEdit)
            SecondaryButton("Xóa", onDelete, danger = true)
        }
    }
}

/** "Đúng ngày, 3 ngày trước lúc 07:00 · Tiên thường: tắt". */
fun ruleText(r: Map<String, Any?>?, gio: Boolean): String {
    if (r == null || !r.b("enabled")) return "Không nhắc"
    val offs = r.list("offsets_days").mapNotNull { (it as? Number)?.toInt() }
    val base = PFmt.offsets(offs) + " lúc " + r.s("time_local")
    return if (gio) base + " · Tiên thường (lễ cúng hôm trước ngày giỗ): " + (if (r.b("tien_thuong")) "bật" else "tắt") else base
}
