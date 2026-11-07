@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package vn.clevix.lichviet.ui.people

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import java.time.LocalDate
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import vn.clevix.lichviet.notify.Reminders
import vn.clevix.lichviet.people.DateValue
import vn.clevix.lichviet.people.EncodedImage
import vn.clevix.lichviet.people.JMap
import vn.clevix.lichviet.people.Lunar
import vn.clevix.lichviet.people.PFmt
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.Person
import vn.clevix.lichviet.people.Photos
import vn.clevix.lichviet.people.b
import vn.clevix.lichviet.people.copyJ
import vn.clevix.lichviet.people.errText
import vn.clevix.lichviet.people.i
import vn.clevix.lichviet.people.o
import vn.clevix.lichviet.people.objs
import vn.clevix.lichviet.people.s
import vn.clevix.lichviet.people.sn
import vn.clevix.lichviet.people.strs
import vn.clevix.lichviet.ui.LocalLich

/** Độ dài theo điểm mã (01 §1). */
fun cp(s: String): Int = s.trim().codePointCount(0, s.trim().length)

fun lenErr(s: String, max: Int, what: String): String? = if (cp(s) > max) "$what tối đa $max ký tự" else null

/** Ô chữ kèm hàng gợi ý (thay cho datalist). */
@Composable
fun SuggestIn(label: String, value: String, onChange: (String) -> Unit, options: List<String>, error: String?, help: String? = null) {
    var open by remember { mutableStateOf(false) }
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        TextIn(label, value, onChange, error = error, help = help)
        LinkButton(if (open) "Ẩn gợi ý ▴" else "Gợi ý ▾") { open = !open }
        if (open) Flow { options.forEach { o -> Chip(o, value == o, { onChange(o); open = false }) } }
    }
}

/** Thông tin báo tử (01 §3.2a) đang nhập. */
@Stable
class NoticeState(init: Map<String, Any?>?) {
    var reportedBy by mutableStateOf(init?.sn("reported_by") ?: "")
    var place by mutableStateOf(init?.sn("place") ?: "")
    var reason by mutableStateOf(init?.sn("reason") ?: "")
    var note by mutableStateOf(init?.sn("note") ?: "")
    val date = DateInputState(DateValue.of(init?.o("notice_date")), allowLunar = false)
    /** Tệp đính kèm (01 §3.2a, YC-2026-07): phần tử đã lưu (có file_id) và tệp vừa chọn (chưa ghi vào kho). */
    val items = mutableStateListOf<AttItem>().apply {
        addAll((init?.objs("attachments") ?: emptyList()).map { AttItem(it.s("id"), it.s("kind"), it.s("file_id"), null) })
    }
    /** Loại của tệp sắp thêm. */
    var newKind by mutableStateOf("tin_bao_tu")
    val hadNotice = init != null

    fun hasContent(): Boolean =
        reportedBy.isNotBlank() || place.isNotBlank() || reason.isNotBlank() || note.isNotBlank() || !date.empty ||
            items.isNotEmpty()

    fun errors(today: LocalDate): List<String> {
        val e = ArrayList<String>()
        lenErr(reportedBy, 200, "Người báo tin")?.let(e::add)
        lenErr(place, 200, "Nơi mất")?.let(e::add)
        lenErr(reason, 500, "Lý do")?.let(e::add)
        lenErr(note, 2000, "Ghi chú")?.let(e::add)
        if (!date.empty) {
            val fe = date.formatError("")
            if (fe != null) e.add(fe)
            else date.value()?.let { v -> if (LocalDate.of(v.year!!, v.month!!, v.day!!).isAfter(today)) e.add("Ngày nhận tin không được sau hôm nay") }
        }
        return e
    }

    /**
     * Ghi các tệp vừa chọn vào kho (`add_attachment`); trả map id phần tử → file_id. Lỗi giữa chừng → xóa các tệp đã ghi
     * ở lần này rồi ném lại (không để tệp mồ côi).
     */
    fun storePending(repo: PeopleRepo): Map<String, String> {
        val out = LinkedHashMap<String, String>()
        try {
            for (it in items) it.pending?.let { p -> out[it.id] = repo.addAttachment(p.mime, p.fileName, p.bytes) }
        } catch (e: Exception) {
            out.values.forEach { repo.deleteAttachment(it) }
            throw e
        }
        return out
    }

    /** JSON `DeathNotice` (lõi cắt khoảng trắng, chuẩn hóa NFC, rỗng → null). [stored]: kết quả [storePending]. */
    fun toMap(stored: Map<String, String>): JMap = linkedMapOf<String, Any?>(
        "reported_by" to reportedBy,
        "place" to place,
        "reason" to reason,
        "notice_date" to date.value()?.toMap(),
        "attachments" to ArrayList(items.mapNotNull { a ->
            val fid = a.fileId ?: stored[a.id] ?: return@mapNotNull null
            linkedMapOf<String, Any?>("id" to a.id, "kind" to a.kind, "file_id" to fid)
        }),
        "note" to note,
    )
}

/** Khối "Thông tin báo tử (không bắt buộc)" (§12g). */
@Composable
fun NoticeEditor(st: NoticeState, startOpen: Boolean, showErrors: Boolean, today: LocalDate, repo: PeopleRepo) {
    var open by remember { mutableStateOf(startOpen) }
    PCard {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            H2("Thông tin báo tử (không bắt buộc)", Modifier.weight(1f))
            LinkButton(if (open) "Thu gọn ▴" else "Mở ▾") { open = !open }
        }
        if (open) {
            Fine(
                "Chỉ hiện trên trang hồ sơ này. Không đưa vào lời nhắc, tệp lịch (.ics) hay trang tưởng niệm; có trong tệp " +
                    "sao lưu (đã mã hóa).",
            )
            TextIn("Người báo tin", st.reportedBy, { st.reportedBy = it }, error = if (showErrors) lenErr(st.reportedBy, 200, "Người báo tin") else null)
            TextIn("Mất tại", st.place, { st.place = it }, error = if (showErrors) lenErr(st.place, 200, "Nơi mất") else null)
            Fine("Ngày nhận tin (dương lịch)")
            val dErr = if (st.date.empty) null else st.date.formatError("")
            DateInput(st.date, null, dErr)
            TextIn("Lý do", st.reason, { st.reason = it }, multiline = true, error = if (showErrors) lenErr(st.reason, 500, "Lý do") else null)
            AttachmentEditor(st, repo)
            TextIn("Ghi chú", st.note, { st.note = it }, multiline = true, error = if (showErrors) lenErr(st.note, 2000, "Ghi chú") else null)
            if (showErrors) st.errors(today).filter { it.startsWith("Ngày nhận") }.forEach { ErrorText(it) }
        }
    }
}

/** Khung tóm tắt lỗi "Còn n chỗ cần sửa" (08 §2). */
@Composable
fun ErrorSummary(errors: List<String>) {
    if (errors.isEmpty()) return
    NoticeBox("Còn ${errors.size} chỗ cần sửa:\n" + errors.joinToString("\n") { "• $it" }, warn = true)
}

// -------------------------------------------------------------------------------------------------
// Thêm / sửa người đã mất (08 §3.4) và người đang sống (§3.5)
// -------------------------------------------------------------------------------------------------

@Composable
fun PersonForm(x: PeopleCtx, id: String?, deceasedNew: Boolean?, focusTray: Boolean = false) {
    val existing: Person? = remember(id) { id?.let { x.repo.person(it) } }
    if (id != null && existing == null) {
        BackLink("Về danh sách hồ sơ") { x.nav.back() }
        Body("Không tìm thấy hồ sơ này (có thể đã bị xóa).")
        return
    }
    val deceased = existing?.deceased ?: (deceasedNew == true)
    val settings = remember { runCatching { x.repo.settings() }.getOrNull() }
    val f = remember(id) { PersonFormState(existing, deceased, settings) }
    val ctx = LocalContext.current
    val scope = rememberCoroutineScope()
    // Bộ chọn ảnh mâm lễ ở cấp biểu mẫu (P46-01): không mất khi khối "Tưởng niệm" thu gọn.
    val trayPick = rememberTrayPicker(f)
    var showErrors by remember { mutableStateOf(false) }
    var saveError by remember { mutableStateOf<String?>(null) }
    var confirmCancel by remember { mutableStateOf(false) }
    var busy by remember { mutableStateOf(false) }
    var photoErr by remember { mutableStateOf<String?>(null) }
    val picker = rememberLauncherForActivityResult(ActivityResultContracts.PickVisualMedia()) { uri ->
        if (uri != null) scope.launch {
            val r = withContext(Dispatchers.Default) { runCatching { Photos.process(ctx, uri) } }
            r.onSuccess { f.photo = it; f.photoRemoved = false; photoErr = null }.onFailure { photoErr = it.message }
        }
    }

    val cancel: () -> Unit = {
        if (f.dirty()) {
            confirmCancel = true
        } else {
            x.nav.back()
        }
    }
    BackLink(if (existing != null) "Về hồ sơ" else "Quay lại") { cancel() }
    H1(if (existing != null) "Sửa hồ sơ" else if (deceased) "Thêm người đã mất" else "Thêm người đang sống")
    if (existing != null) Fine(existing.displayName)

    val errors = f.errors(x.today)
    if (showErrors) ErrorSummary(errors)
    saveError?.let { NoticeBox(it, warn = true) }

    // Nhóm đầu.
    PCard(title = if (deceased) "Người đã mất" else "Người đang sống") {
        SuggestIn("Cách gọi (đứng trước tên)", f.danhXung, { f.danhXung = it }, remember(deceased, f.gioiTinh) { PeopleRepo.danhXungList(deceased, f.gioiTinh) }, if (showErrors) lenErr(f.danhXung, 40, "Cách gọi") else null)
        TextIn("Họ và tên *", f.hoTen, { f.hoTen = it }, error = if (showErrors) f.nameError() else null)
        Fine("Giới tính")
        Choice(listOf("Nam", "Nữ", "Không ghi"), when (f.gioiTinh) { "nam" -> 0; "nu" -> 1; else -> 2 }, {
            f.gioiTinh = when (it) { 0 -> "nam"; 1 -> "nu"; else -> null }
        })
        if (deceased) TextIn("Tên hiệu / pháp danh", f.tenHieu, { f.tenHieu = it }, error = if (showErrors) lenErr(f.tenHieu, 120, "Tên hiệu") else null)
        SuggestIn(
            "Quan hệ với bạn", f.quanHe, { f.quanHe = it }, remember(deceased) { PeopleRepo.quanHeList(deceased) },
            if (showErrors) lenErr(f.quanHe, 120, "Quan hệ") else null,
            help = if (deceased) "Trang tưởng niệm ghi quan hệ này phía trên tên, ví dụ “Ông nội”, “Mẹ”. Để trống thì chỉ ghi tên." else null,
        )
        TextIn("Ghi chú", f.ghiChu, { f.ghiChu = it }, multiline = true, error = if (showErrors) lenErr(f.ghiChu, 2000, "Ghi chú") else null)
    }

    if (deceased) {
        PCard(title = "Ngày mất *") {
            val (res, err) = f.deathPreview(x.today)
            DateInput(f.death, res, err, unknownYearLabel = "Không rõ năm mất")
        }
    }

    PCard(title = if (deceased) "Ngày sinh và nơi sinh" else "Ngày sinh") {
        val (res, err) = f.birthPreview(x.today)
        DateInput(
            f.birth, res, err,
            yearOnlyLabels = listOf("Biết đủ ngày", "Chỉ biết năm"),
        )
        TextIn("Nơi sinh", f.noiSinh, { f.noiSinh = it }, error = if (showErrors) lenErr(f.noiSinh, 300, "Nơi sinh") else null,
            help = "Nơi sinh được lưu cùng ngày sinh.")
        if (!deceased && f.isFeb29()) {
            Fine("Sinh ngày 29/02 — năm không nhuận mừng sinh nhật vào:")
            RadioRow("Năm không nhuận: mừng 28/02", f.feb29 == "feb28", { f.feb29 = "feb28" })
            RadioRow("Năm không nhuận: mừng 01/03", f.feb29 == "mar1", { f.feb29 = "mar1" })
        }
    }

    if (deceased) {
        PCard(title = "Tuổi") {
            val t = f.agePreview(x.today)
            if (!f.editAge) {
                Body(t ?: "Chưa đủ năm sinh để tính tuổi")
                SecondaryButton("Sửa tuổi", { f.editAge = true })
            } else {
                NumIn("Tuổi (1–150)", f.manualAge, { f.manualAge = it }, Modifier.fillMaxWidth(0.5f), maxLen = 3)
                if (showErrors) f.ageError()?.let { ErrorText(it) }
                SecondaryButton("Tính lại", { f.editAge = false; f.manualAge = "" })
            }
        }
        PCard(title = "Nơi an táng và ảnh") {
            TextIn("Nơi an táng", f.noiAnTang, { f.noiAnTang = it }, error = if (showErrors) lenErr(f.noiAnTang, 300, "Nơi an táng") else null)
            val stored = remember(existing?.photoId) { existing?.photoId?.let { x.repo.photo(it) } }
            val img = remember(f.photo, f.photoRemoved, stored) {
                when {
                    f.photo != null -> Photos.bitmap(f.photo!!.bytes)?.asImageBitmap()
                    f.photoRemoved -> null
                    else -> stored?.let { Photos.bitmap(it)?.asImageBitmap() }
                }
            }
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                OvalPhoto(img, f.hoTen, 96, 120)
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    SecondaryButton(if (img == null) "Chọn ảnh" else "Đổi ảnh", {
                        picker.launch(PickVisualMediaRequest(ActivityResultContracts.PickVisualMedia.ImageOnly))
                    })
                    if (img != null) SecondaryButton("Bỏ ảnh", { f.photo = null; f.photoRemoved = true })
                }
            }
            photoErr?.let { ErrorText(it) }
            Fine("Ảnh chỉ lưu trong ứng dụng trên máy này, đã bỏ thông tin vị trí và máy chụp.")
        }
        SecureScreen() // BM2-04: khối thông tin báo tử
        NoticeEditor(f.notice, startOpen = f.notice.hadNotice, showErrors = showErrors, today = x.today, repo = x.repo)
        PCard(title = "Nhắc ngày giỗ") {
            RuleEditor(f.gio, "Nhắc ngày giỗ", showTienThuong = true, showErrors = showErrors)
        }
        PCard {
            var open by remember { mutableStateOf(focusTray) }
            Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                H2("Tưởng niệm", Modifier.weight(1f))
                LinkButton(if (open) "Thu gọn ▴" else "Mở ▾") { open = !open }
            }
            if (open) {
                Fine("Giỗ hết (giỗ lần cuối của thời gian để tang) vào")
                RadioRow("Năm thứ 2 sau năm mất", f.gioHet == 2, { f.gioHet = 2 })
                RadioRow("Năm thứ 3 sau năm mất", f.gioHet == 3, { f.gioHet = 3 })
                TextIn("Lời tưởng niệm", f.loi, { f.loi = it }, multiline = true, error = if (showErrors) lenErr(f.loi, 2000, "Lời tưởng niệm") else null)
                TraySection(f, x.repo, focusTray, trayPick)
                TextIn(
                    "Mất trước tuổi này thì bia ghi “hưởng dương” (mất sớm) thay cho “hưởng thọ”", f.huongDuong,
                    { f.huongDuong = it.filter(Char::isDigit).take(3) }, number = true,
                    error = if (showErrors) f.huongDuongError() else null,
                )
            }
        }
    } else {
        PCard(title = "Nhắc sinh nhật") {
            RuleEditor(f.sn, "Nhắc sinh nhật", showTienThuong = false, showErrors = showErrors)
            val full = f.birth.value()?.precision == "day"
            CheckRow("Nhắc cả sinh nhật âm lịch hằng năm", f.sinhNhatAm && full, { f.sinhNhatAm = it }, enabled = full)
            if (!full) Fine("Cần biết đủ ngày sinh")
            if (f.sinhNhatAm && full) {
                CheckRow("Nhắc sinh nhật âm lịch giống nhắc sinh nhật dương lịch", f.sameAsSolar, { f.sameAsSolar = it })
                if (!f.sameAsSolar) {
                    H3("Nhắc sinh nhật âm lịch")
                    RuleEditor(f.snAm, "Nhắc sinh nhật âm lịch", showTienThuong = false, showErrors = showErrors)
                }
            }
        }
    }

    Row(horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
        SecondaryButton("Hủy", { cancel() }, Modifier.weight(1f))
        PrimaryButton(if (busy) "Đang lưu…" else "Lưu", {
            showErrors = true
            f.touchAll()
            saveError = null
            if (f.errors(x.today).isEmpty() && !busy) {
                busy = true
                try {
                    val saved = f.save(x.repo)
                    x.changed()
                    if (existing != null) x.nav.back() else x.nav.replace(PRoute.PersonPage(saved.id))
                    x.toast("Đã lưu hồ sơ.")
                    if (f.anyReminderOn() && Reminders.needsPermission(ctx) && !Reminders.askedBefore(ctx)) x.askNotify()
                } catch (e: Exception) {
                    saveError = "Chưa lưu được: ${errText(e)}. Dữ liệu đang nhập vẫn còn trên màn hình."
                } finally {
                    busy = false
                }
            }
        }, Modifier.weight(1f), enabled = !busy && !f.trayBusy)
    }

    if (confirmCancel) {
        ConfirmDialog(
            title = "Bỏ các thay đổi chưa lưu?", confirm = "Bỏ thay đổi", dismiss = "Tiếp tục sửa",
            onConfirm = { confirmCancel = false; x.nav.back() }, onDismiss = { confirmCancel = false },
        ) { Body("Những gì bạn vừa nhập sẽ không được lưu.") }
    }
}

/** Trạng thái biểu mẫu người. Dựng JSON `Person` cho `save_person_json` của lõi. */
@Stable
class PersonFormState(val existing: Person?, val deceased: Boolean, settings: JMap?) {
    private val raw = existing?.raw
    var danhXung by mutableStateOf(existing?.danhXung ?: "")
    var hoTen by mutableStateOf(existing?.hoTen ?: "")
    var tenHieu by mutableStateOf(existing?.tenHieu ?: "")
    var quanHe by mutableStateOf(existing?.quanHe ?: "")
    /** Giới tính (YC-2026-07): "nam" | "nu" | null = Không ghi. */
    var gioiTinh by mutableStateOf(raw?.sn("gioi_tinh"))
    var ghiChu by mutableStateOf(existing?.ghiChu ?: "")
    var noiSinh by mutableStateOf(existing?.birth?.s("noi_sinh") ?: "")
    var noiAnTang by mutableStateOf(existing?.noiAnTang ?: "")
    val death = DateInputState(DateValue.of(existing?.death?.o("input")), allowUnknownYear = true)
    val birth = DateInputState(DateValue.of(existing?.birth?.o("input")), allowYearOnly = true, minYear = 1000)
    var feb29 by mutableStateOf(raw?.s("feb29_rule")?.takeIf { it.isNotEmpty() } ?: "feb28")
    private val tuoi = existing?.death?.o("tuoi")
    var editAge by mutableStateOf(tuoi?.b("manual") == true)
    var manualAge by mutableStateOf(if (tuoi?.b("manual") == true) tuoi.i("value")?.toString() ?: "" else "")
    private val mem = existing?.memorial
    var gioHet by mutableStateOf(mem?.i("gio_het_nam") ?: 2)
    var loi by mutableStateOf(mem?.s("loi_tuong_niem") ?: "")
    var huongDuong by mutableStateOf((mem?.i("huong_duong_duoi") ?: 60).toString())
    var photo by mutableStateOf<EncodedImage?>(null)
    var photoRemoved by mutableStateOf(false)
    /** Ảnh mâm lễ của gia đình (YC-2026-10, 08 §16.2): ảnh mới chọn (chưa ghi vào kho), bỏ ảnh, đang xử lý. */
    var tray by mutableStateOf<EncodedImage?>(null)
    var trayRemoved by mutableStateOf(false)
    var trayBusy by mutableStateOf(false)
    var trayError by mutableStateOf<String?>(null)
    val trayStoredId: String? = existing?.memorial?.sn("tray_photo_id")
    val notice = NoticeState(existing?.notice)
    val gio = RuleState(existing?.reminders?.o("gio"), settings)
    val sn = RuleState(existing?.reminders?.o("sinh_nhat"), settings)
    private val snAmInit = existing?.reminders?.o("sinh_nhat_am")
    val snAm = RuleState(snAmInit, settings)
    var sinhNhatAm by mutableStateOf(raw?.b("sinh_nhat_am") == true)
    var sameAsSolar by mutableStateOf(snAmInit == null || RuleState(snAmInit, settings).sameAs(sn))
    private val previewId = existing?.id ?: PeopleRepo.uuid()
    private val snapshot = fingerprint()

    private fun fingerprint(): String = listOf(
        danhXung, hoTen, tenHieu, quanHe, ghiChu, noiSinh, noiAnTang, death.d, death.m, death.y, death.lunar, death.leap,
        birth.d, birth.m, birth.y, birth.lunar, manualAge, loi, huongDuong, gioHet, photo != null, photoRemoved, tray != null, trayRemoved,
        notice.reportedBy, notice.place, notice.reason, notice.note, notice.items.map { it.id + it.kind }, gioiTinh,
        gio.allOffsets(), gio.enabled, gio.timeText(), sn.allOffsets(), sn.enabled, sinhNhatAm,
    ).joinToString("|")

    fun dirty(): Boolean = fingerprint() != snapshot

    fun touchAll() { death.touched = true; birth.touched = true; notice.date.touched = true }

    fun isFeb29(): Boolean = birth.value()?.let { !it.lunar && it.precision == "day" && it.month == 2 && it.day == 29 } == true

    fun nameError(): String? = when {
        hoTen.isBlank() -> "Họ tên không được để trống"
        cp(hoTen) > 120 -> "Họ tên tối đa 120 ký tự"
        else -> null
    }

    fun ageError(): String? {
        if (!editAge) return null
        val v = manualAge.toIntOrNull()
        return if (v != null && v in 1..150) null else "Tuổi phải từ 1 đến 150"
    }

    fun huongDuongError(): String? {
        val v = huongDuong.toIntOrNull()
        return if (v != null && v in 1..150) null else "Hãy nhập số từ 1 đến 150"
    }

    /** Kết quả đổi lịch của ngày mất (dòng trực tiếp) và lỗi. */
    fun deathPreview(today: LocalDate): Pair<String?, String?> {
        val fe = death.formatError("Hãy nhập ngày mất")
        if (fe != null) return null to fe
        val v = death.value() ?: return null to null
        val r = runCatching { PeopleRepo.deriveDeath(v) }
        r.exceptionOrNull()?.let { return null to errText(it) }
        val d = r.getOrThrow()
        val sol = d.o("solar")?.let { LocalDate.of(it.i("y")!!, it.i("m")!!, it.i("d")!!) }
        if (sol != null && sol.isAfter(today)) return null to "Ngày mất không được sau hôm nay"
        val l = Lunar.of(d.o("lunar"))
        return when {
            v.precision == "month_day" ->
                "Không rõ năm: vẫn nhắc giỗ hằng năm vào ${v.day}/${v.month}${if (v.leap) " nhuận" else ""} âm lịch, không ghi giỗ đầu, giỗ hết." to null
            !v.lunar && l != null -> "Âm lịch: ${l.dm} năm ${PeopleRepo.canChi(l.year ?: 0)} (${l.year})" to null
            sol != null -> "Dương lịch: ${PFmt.full(sol)}" to null
            else -> "Năm này ngoài khoảng đổi lịch — không có ngày dương tương ứng" to null
        }
    }

    fun birthPreview(today: LocalDate): Pair<String?, String?> {
        if (birth.empty) return null to null
        val fe = birth.formatError("")
        if (fe != null) return null to fe
        val v = birth.value() ?: return null to null
        val r = runCatching { PeopleRepo.deriveBirth(v) }
        r.exceptionOrNull()?.let { return null to errText(it) }
        val b = r.getOrThrow()
        val sol = b.o("solar")?.let { LocalDate.of(it.i("y")!!, it.i("m")!!, it.i("d")!!) }
        if (sol != null && sol.isAfter(today)) return null to "Ngày sinh không được sau hôm nay"
        val ly = b.i("lunar_year")
        val cc = b.sn("can_chi") ?: ""
        if (deceased || v.precision == "year") {
            if (ly == null) return null to null
            return (if (b.b("lunar_year_approx")) "khoảng năm âm $ly — $cc (nếu sinh trước Tết thì là năm trước)" else "Năm âm $ly — $cc") to null
        }
        val l = Lunar.of(b.o("lunar"))
        val stub: JMap = linkedMapOf<String, Any?>("kind" to "living", "ho_ten" to "", "danh_xung" to "", "birth" to b, "feb29_rule" to feb29)
        val s = runCatching { PeopleRepo.summaryOf(stub, today) }.getOrNull()
        val am = s?.tuoiAm?.let { a -> " (tuổi âm $a)" } ?: ""
        val age = s?.tuoiDuong?.let { " · Hôm nay $it tuổi$am" } ?: ""
        val lunarTxt = if (v.lunar) sol?.let { "Dương lịch: ${PFmt.full(it)}" } else l?.let { "Âm lịch: ${it.dm} năm $cc (${it.year})" }
        return ((lunarTxt ?: "Năm âm $ly — $cc") + age) to null
    }

    /** "Hưởng thọ 86 tuổi" (lõi tính theo 02 §4.2 với ngưỡng của hồ sơ). */
    fun agePreview(today: LocalDate): String? {
        if (death.value() == null) return null
        return runCatching { PeopleRepo.summaryOf(PeopleRepo.derivePerson(build(emptyMap(), null, preview = true)), today).tuoiMatText }.getOrNull()
    }

    fun errors(today: LocalDate): List<String> {
        val e = ArrayList<String>()
        nameError()?.let(e::add)
        lenErr(danhXung, 40, "Cách gọi")?.let(e::add)
        lenErr(tenHieu, 120, "Tên hiệu")?.let(e::add)
        lenErr(quanHe, 120, "Quan hệ")?.let(e::add)
        lenErr(ghiChu, 2000, "Ghi chú")?.let(e::add)
        lenErr(noiSinh, 300, "Nơi sinh")?.let(e::add)
        if (deceased) {
            deathPreview(today).second?.let { e.add("Ngày mất: $it") }
            lenErr(noiAnTang, 300, "Nơi an táng")?.let(e::add)
            lenErr(loi, 2000, "Lời tưởng niệm")?.let(e::add)
            ageError()?.let(e::add)
            huongDuongError()?.let(e::add)
            e.addAll(notice.errors(today))
            gio.errors().forEach { e.add("Nhắc giỗ: $it") }
        } else {
            sn.errors().forEach { e.add("Nhắc sinh nhật: $it") }
            if (sinhNhatAm && !sameAsSolar) snAm.errors().forEach { e.add("Nhắc sinh nhật âm lịch: $it") }
        }
        birthPreview(today).second?.let { e.add("Ngày sinh: $it") }
        return e
    }

    fun anyReminderOn(): Boolean = if (deceased) gio.enabled && gio.allOffsets().isNotEmpty() else sn.enabled

    /** Dựng JSON `Person` (trường dẫn xuất để lõi tính). */
    fun build(newNoticeIds: Map<String, String>, newPhotoId: String?, preview: Boolean = false, newTrayId: String? = null): JMap {
        val m: JMap = raw?.copyJ() ?: linkedMapOf<String, Any?>()
        if (preview && m["id"] == null) m["id"] = previewId
        m["kind"] = if (deceased) "deceased" else "living"
        m["danh_xung"] = danhXung.trim() // lõi viết hoa chữ đầu khi lưu (save_person_json)
        m["ho_ten"] = if (preview && hoTen.isBlank()) "?" else hoTen
        m["ten_hieu"] = if (deceased) tenHieu else (raw?.s("ten_hieu") ?: "")
        m["quan_he"] = quanHe.trim()
        m["gioi_tinh"] = gioiTinh
        m["ghi_chu"] = ghiChu
        m["birth"] = birth.value()?.let { linkedMapOf<String, Any?>("input" to it.toMap(), "noi_sinh" to noiSinh) }
        m["feb29_rule"] = feb29
        if (deceased) {
            val dm: JMap = linkedMapOf<String, Any?>("input" to death.value()?.toMap())
            if (editAge) {
                val v = manualAge.toIntOrNull()
                val th = huongDuong.toIntOrNull() ?: 60
                val label = if (v != null && v < th) "huong_duong" else "huong_tho"
                dm["tuoi"] = linkedMapOf<String, Any?>("value" to v, "label" to label, "manual" to true, "approx" to false)
            }
            if (!preview) {
                if (notice.hasContent()) dm["notice"] = notice.toMap(newNoticeIds)
                else if (notice.hadNotice) dm["notice"] = null
            }
            m["death"] = dm
            m["noi_an_tang"] = noiAnTang
            m["photo_id"] = when {
                newPhotoId != null -> newPhotoId
                photoRemoved -> null
                else -> raw?.sn("photo_id")
            }
            val memo: JMap = existing?.memorial?.copyJ() ?: linkedMapOf<String, Any?>(
                "tray_style" to "truyen_thong", "incense" to true, "bia_tieu_de" to "",
            )
            memo["loi_tuong_niem"] = loi
            memo["gio_het_nam"] = gioHet
            memo["huong_duong_duoi"] = huongDuong.toIntOrNull() ?: 60
            // 01 §3.7: null → không ghi khóa. Đổi/bỏ ảnh có hiệu lực khi lưu; lõi xóa ảnh cũ trong cùng giao dịch.
            when {
                newTrayId != null -> memo["tray_photo_id"] = newTrayId
                trayRemoved -> memo.remove("tray_photo_id")
            }
            if (memo["tray_photo_id"] == null) memo.remove("tray_photo_id")
            m["memorial"] = memo
            m["reminders"] = linkedMapOf<String, Any?>("gio" to gio.toMap(true), "sinh_nhat" to null, "sinh_nhat_am" to null)
            m["sinh_nhat_am"] = false
        } else {
            m["death"] = null
            m["memorial"] = null
            m["noi_an_tang"] = ""
            m["photo_id"] = raw?.sn("photo_id")
            val full = birth.value()?.precision == "day"
            val am = sinhNhatAm && full
            if (am && sameAsSolar) snAm.copyFrom(sn)
            m["reminders"] = linkedMapOf<String, Any?>(
                "gio" to null,
                "sinh_nhat" to sn.toMap(false),
                "sinh_nhat_am" to if (am) snAm.toMap(false) else null,
            )
            m["sinh_nhat_am"] = am
        }
        if (raw == null) m.putIfAbsent("id", "")
        return m
    }

    /** Ghi ảnh (nếu có) rồi hồ sơ. */
    /** Ghi ảnh + tệp đính kèm mới rồi lưu hồ sơ; lưu lỗi → xóa ảnh/tệp vừa ghi (`delete_photo`, `delete_attachment`). */
    fun save(repo: PeopleRepo): Person {
        val photoId = photo?.let { repo.addPhoto(it) }
        var att: Map<String, String> = emptyMap()
        var trayId: String? = null
        try {
            trayId = if (deceased) tray?.let { repo.addPhoto(it) } else null
            att = if (deceased) notice.storePending(repo) else emptyMap()
            return repo.savePerson(build(att, photoId, newTrayId = trayId))
        } catch (e: Exception) {
            photoId?.let { repo.deletePhoto(it) }
            trayId?.let { repo.deletePhoto(it) }
            att.values.forEach { repo.deleteAttachment(it) }
            throw e
        }
    }
}

/** Chữ hiển thị giá trị một ngày âm có năm: "15/6 nhuận năm Ất Tỵ (2025)". */
fun lunarYearText(l: Lunar): String = l.dm + (l.year?.let { " năm ${PeopleRepo.canChi(it)} ($it)" } ?: "")
