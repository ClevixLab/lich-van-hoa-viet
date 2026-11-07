@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package vn.clevix.lichviet.ui.people

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import java.time.LocalDate
import vn.clevix.lichviet.people.DateValue
import vn.clevix.lichviet.people.JMap
import vn.clevix.lichviet.people.Lunar
import vn.clevix.lichviet.people.PFmt
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.copyJ
import vn.clevix.lichviet.people.errText
import vn.clevix.lichviet.people.i
import vn.clevix.lichviet.people.o

private const val OTHER = "__khac__"

/** Thêm / sửa sự kiện của một người — 08 §3.7 (+ B4: sự kiện của người đã mất không có khối nhắc). */
@Composable
fun EventForm(x: PeopleCtx, personId: String, eventId: String?) {
    val person = remember(personId) { x.repo.person(personId) }
    val existing = remember(eventId) { eventId?.let { id -> x.repo.events().firstOrNull { it.id == id } } }
    val types = remember(x.rev) { x.repo.eventTypes() }
    val settings = remember { runCatching { x.repo.settings() }.getOrNull() }
    if (person == null || (eventId != null && existing == null)) {
        BackLink("Về hồ sơ") { x.nav.back() }
        Body("Không tìm thấy hồ sơ hoặc sự kiện này (có thể đã bị xóa).")
        return
    }
    val deceased = person.deceased
    var typeId by remember { mutableStateOf(existing?.typeId ?: types.firstOrNull()?.id ?: OTHER) }
    var newType by remember { mutableStateOf("") }
    var tieuDe by remember { mutableStateOf(existing?.tieuDe ?: "") }
    var ghiChu by remember { mutableStateOf(existing?.ghiChu ?: "") }
    var recurring by remember { mutableStateOf(existing?.recurring ?: (types.firstOrNull { it.id == typeId }?.defaultRecurring ?: false)) }
    val date = remember { DateInputState(existing?.date, allowUnknownYear = true) }
    val rule = remember { RuleState(existing?.reminder, settings) }
    var ruleOn by remember { mutableStateOf(existing?.let { it.reminder != null } ?: true) }
    var showErrors by remember { mutableStateOf(false) }
    var saveError by remember { mutableStateOf<String?>(null) }
    var menu by remember { mutableStateOf(false) }
    var confirmCancel by remember { mutableStateOf(false) }
    val start = remember { listOf(typeId, tieuDe, ghiChu, recurring, date.d, date.m, date.y, date.lunar).joinToString("|") }
    fun dirty() = listOf(typeId, tieuDe, ghiChu, recurring, date.d, date.m, date.y, date.lunar).joinToString("|") != start
    val cancel: () -> Unit = {
        if (dirty()) {
            confirmCancel = true
        } else {
            x.nav.back()
        }
    }

    // Kết quả đổi lịch + lỗi ngày.
    fun preview(): Pair<String?, String?> {
        val fe = date.formatError("Hãy nhập ngày")
        if (fe != null) return null to fe
        val v = date.value() ?: return null to null
        if (v.precision == "month_day") {
            if (!recurring) return null to "Sự kiện không rõ năm phải lặp hằng năm"
            return "Lặp hằng năm vào ${v.day}/${v.month}${if (v.leap) " nhuận" else ""} âm lịch" to null
        }
        val r = runCatching { PeopleRepo.deriveDeath(v) }
        r.exceptionOrNull()?.let { return null to errText(it) }
        val d = r.getOrThrow()
        val l = Lunar.of(d.o("lunar"))
        val sol = d.o("solar")?.let { LocalDate.of(it.i("y")!!, it.i("m")!!, it.i("d")!!) }
        return when {
            !v.lunar && l != null -> "Âm lịch: ${lunarYearText(l)}" to null
            sol != null -> "Dương lịch: ${PFmt.full(sol)}" to null
            else -> null to null
        }
    }

    fun errors(): List<String> {
        val e = ArrayList<String>()
        if (typeId == OTHER) {
            if (newType.isBlank()) e.add("Hãy đặt tên cho loại sự kiện")
            lenErr(newType, 60, "Tên loại")?.let(e::add)
        }
        lenErr(tieuDe, 120, "Tiêu đề")?.let(e::add)
        lenErr(ghiChu, 2000, "Ghi chú")?.let(e::add)
        preview().second?.let { e.add("Ngày: $it") }
        if (!deceased && ruleOn) rule.errors().forEach { e.add("Nhắc: $it") }
        return e
    }

    BackLink("Về hồ sơ") { cancel() }
    H1(if (existing != null) "Sửa sự kiện" else "Thêm sự kiện")
    Fine(person.displayName)
    if (showErrors) ErrorSummary(errors())
    saveError?.let { NoticeBox(it, warn = true) }

    PCard(title = "Sự kiện") {
        val label = if (typeId == OTHER) "Loại khác…" else types.firstOrNull { it.id == typeId }?.ten ?: "Chọn loại"
        Fine("Loại")
        Box {
            SecondaryButton("$label ▾", { menu = true }, Modifier.fillMaxWidth())
            DropdownMenu(menu, { menu = false }) {
                types.forEach { t ->
                    DropdownMenuItem(text = { Text(t.ten) }, onClick = {
                        typeId = t.id; menu = false
                        if (existing == null) recurring = t.defaultRecurring
                    })
                }
                DropdownMenuItem(text = { Text("Loại khác…") }, onClick = { typeId = OTHER; menu = false })
            }
        }
        if (typeId == OTHER) {
            TextIn("Tên loại mới", newType, { newType = it }, error = if (showErrors && newType.isBlank()) "Hãy đặt tên cho loại sự kiện" else null)
        }
        TextIn("Tiêu đề", tieuDe, { tieuDe = it }, help = "Để trống sẽ dùng tên loại", error = if (showErrors) lenErr(tieuDe, 120, "Tiêu đề") else null)
        TextIn("Ghi chú", ghiChu, { ghiChu = it }, multiline = true, error = if (showErrors) lenErr(ghiChu, 2000, "Ghi chú") else null)
    }

    PCard(title = "Ngày") {
        val (res, err) = preview()
        DateInput(
            date, res, err,
            extra = { CheckRow("Lặp lại hằng năm", recurring, { recurring = it }) },
        )
    }

    PCard(title = "Nhắc") {
        if (deceased) {
            Body(
                "Không nhắc sự kiện của người đã mất — với người đã mất, ứng dụng chỉ nhắc ngày giỗ. Sự kiện vẫn được " +
                    "lưu và hiện trên hồ sơ.",
            )
        } else {
            CheckRow("Bật nhắc", ruleOn, { ruleOn = it })
            if (ruleOn) RuleEditor(rule, "Nhắc sự kiện này", showTienThuong = false, showErrors = showErrors)
        }
    }

    Row(horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        SecondaryButton("Hủy", cancel, Modifier.weight(1f))
        PrimaryButton("Lưu", {
            showErrors = true
            date.touched = true
            saveError = null
            if (errors().isEmpty()) {
                try {
                    var tid = typeId
                    if (tid == OTHER) {
                        val name = newType.trim()
                        val same = types.firstOrNull { it.ten.trim().lowercase() == name.lowercase() }
                        tid = same?.id ?: x.repo.saveEventType(
                            linkedMapOf<String, Any?>("id" to "", "builtin" to null, "ten" to name, "default_recurring" to recurring),
                        ).id
                    }
                    val m: JMap = existing?.raw?.copyJ() ?: linkedMapOf<String, Any?>("id" to "")
                    m["type_id"] = tid
                    m["tieu_de"] = tieuDe
                    m["date"] = date.value()!!.toMap()
                    m["recurring"] = recurring
                    if (existing == null) m["person_ids"] = arrayListOf(personId)
                    m["ghi_chu"] = ghiChu
                    // B4: người đã mất — giữ nguyên quy tắc nhắc đang có (nếu đến từ sao lưu), không tạo mới.
                    if (!deceased) m["reminder"] = if (ruleOn) rule.toMap(false) else null
                    else if (existing == null) m["reminder"] = null
                    x.repo.saveEvent(m)
                    x.changed()
                    x.nav.back()
                    x.toast("Đã lưu sự kiện.")
                } catch (e: Exception) {
                    saveError = "Chưa lưu được: ${errText(e)}. Dữ liệu đang nhập vẫn còn trên màn hình."
                }
            }
        }, Modifier.weight(1f))
    }
    if (confirmCancel) {
        ConfirmDialog(
            title = "Bỏ các thay đổi chưa lưu?", confirm = "Bỏ thay đổi", dismiss = "Tiếp tục sửa",
            onConfirm = { confirmCancel = false; x.nav.back() }, onDismiss = { confirmCancel = false },
        ) { Body("Những gì bạn vừa nhập sẽ không được lưu.") }
    }
}
