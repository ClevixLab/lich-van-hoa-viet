@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package vn.clevix.lichviet.ui.people

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import java.time.LocalDate
import vn.clevix.lichviet.people.Lunar
import vn.clevix.lichviet.people.PFmt
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.errText
import vn.clevix.lichviet.people.i
import vn.clevix.lichviet.people.list
import vn.clevix.lichviet.people.o
import vn.clevix.lichviet.people.sn

/**
 * "Cập nhật: đã mất" (08 §12g, US-M-07, 01 §3.2a): cập nhật **cùng** hồ sơ người đang sống thành người đã mất, kèm
 * thông tin báo tử tùy chọn. Lõi (`mark_deceased_json`) giữ id, tên, ảnh, ngày sinh; bỏ nhắc sinh nhật (B4).
 */
@Composable
fun MarkDeceasedForm(x: PeopleCtx, id: String) {
    SecureScreen() // BM2-04: biểu mẫu thông tin báo tử
    val p = remember(id) { x.repo.person(id) }
    if (p == null) {
        BackLink("Về hồ sơ") { x.nav.back() }
        Body("Không tìm thấy hồ sơ này (có thể đã bị xóa).")
        return
    }
    val death = remember { DateInputState(null, allowUnknownYear = true) }
    val notice = remember { NoticeState(null) }
    val settings = remember { runCatching { x.repo.settings() }.getOrNull() }
    val hasEvents = remember(x.rev) { runCatching { x.repo.events().any { id in it.personIds } }.getOrDefault(false) }
    var showErrors by remember { mutableStateOf(false) }
    var confirm by remember { mutableStateOf(false) }
    var saveError by remember { mutableStateOf<String?>(null) }

    fun preview(): Triple<String?, String?, Lunar?> {
        val fe = death.formatError("Hãy nhập ngày mất")
        if (fe != null) return Triple(null, fe, null)
        val v = death.value() ?: return Triple(null, null, null)
        val r = runCatching { PeopleRepo.deriveDeath(v) }
        r.exceptionOrNull()?.let { return Triple(null, errText(it), null) }
        val d = r.getOrThrow()
        val l = Lunar.of(d.o("lunar"))
        val sol = d.o("solar")?.let { LocalDate.of(it.i("y")!!, it.i("m")!!, it.i("d")!!) }
        if (sol != null && sol.isAfter(x.today)) return Triple(null, "Ngày mất không được sau hôm nay", null)
        val text = when {
            v.precision == "month_day" ->
                "Không rõ năm: vẫn nhắc giỗ hằng năm vào ${v.day}/${v.month}${if (v.leap) " nhuận" else ""} âm lịch, không ghi giỗ đầu, giỗ hết."
            !v.lunar && l != null -> "Âm lịch: ${lunarYearText(l)}"
            sol != null -> "Dương lịch: ${PFmt.full(sol)}"
            else -> null
        }
        return Triple(text, null, l)
    }

    fun errors(): List<String> = buildList {
        preview().second?.let { add("Ngày mất: $it") }
        addAll(notice.errors(x.today))
    }

    BackLink("Về hồ sơ") { x.nav.back() }
    H1("Cập nhật: đã mất")
    H2(p.displayName)
    Body("Ghi lại ngày mất của ${p.displayName}. Hồ sơ, ảnh và ngày sinh được giữ nguyên; ứng dụng sẽ nhắc ngày giỗ hằng năm.")
    if (showErrors) ErrorSummary(errors())
    saveError?.let { NoticeBox(it, warn = true) }
    PCard(title = "Ngày mất *") {
        val (res, err, _) = preview()
        DateInput(death, res, err, unknownYearLabel = "Không rõ năm mất")
    }
    NoticeEditor(notice, startOpen = true, showErrors = showErrors, today = x.today, repo = x.repo)
    Row(horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        SecondaryButton("Hủy", { x.nav.back() }, Modifier.weight(1f))
        PrimaryButton("Lưu", {
            showErrors = true
            death.touched = true
            if (errors().isEmpty()) confirm = true
        }, Modifier.weight(1f))
    }

    if (confirm) {
        val l = preview().third
        val offs = (settings?.list("default_offsets_days") ?: listOf(0L, 3L)).mapNotNull { (it as? Number)?.toInt() }
        val time = settings?.sn("default_time_local") ?: "07:00"
        val day = l?.let { "${it.dm} âm lịch" } ?: "ngày đã nhập"
        ConfirmDialog(
            title = "Cập nhật ${p.displayName} là người đã mất?",
            confirm = "Lưu thay đổi", dismiss = "Quay lại",
            onDismiss = { confirm = false },
            onConfirm = {
                confirm = false
                var att: Map<String, String> = emptyMap()
                try {
                    att = notice.storePending(x.repo)
                    val n = if (notice.hasContent()) notice.toMap(att) else null
                    x.repo.markDeceased(p.id, death.value()!!, n, x.today)
                    x.changed()
                    x.nav.back()
                    x.toast("Đã lưu hồ sơ.")
                } catch (e: Exception) {
                    att.values.forEach { x.repo.deleteAttachment(it) } // không để tệp mồ côi
                    saveError = "Chưa lưu được: ${errText(e)}. Dữ liệu đang nhập vẫn còn trên màn hình."
                }
            },
        ) {
            Body("• Sẽ không còn nhắc sinh nhật" + (if (hasEvents) " và sự kiện" else "") + " của ${p.displayName}.")
            Body("• Bắt đầu nhắc ngày giỗ hằng năm vào $day (${PFmt.offsets(offs).lowercase()}, lúc $time).")
            Body("• Họ tên, ảnh, ngày sinh và sự kiện vẫn được giữ trong hồ sơ.")
            if (notice.hasContent()) Body("• Thông tin báo tử chỉ hiện trên trang hồ sơ này.")
        }
    }
}
