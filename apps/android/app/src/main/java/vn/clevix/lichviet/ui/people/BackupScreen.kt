@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package vn.clevix.lichviet.ui.people

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.OpenableColumns
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.core.content.FileProvider
import java.io.File
import java.time.LocalDate
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import vn.clevix.lichviet.BuildConfig
import vn.clevix.lichviet.core.BackupPreview
import vn.clevix.lichviet.core.IcsExport
import vn.clevix.lichviet.core.IcsSummary
import vn.clevix.lichviet.core.VerifiedBackup
import vn.clevix.lichviet.people.PFmt
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.errText
import vn.clevix.lichviet.people.i
import vn.clevix.lichviet.people.local
import vn.clevix.lichviet.people.sn

/** Giới hạn kích thước tệp sao lưu trước khi đọc (S12, 03 §2): 72 + 128 MiB + 16 byte. */
const val BACKUP_MAX_BYTES = 72L + 128L * 1024 * 1024 + 16

/** Ghi chú giao diện (không thuộc dữ liệu sao lưu, TK-16): ngày của bản sao lưu vừa "Thay toàn bộ". */
private const val UI_PREFS = "nguoi_than_ui"
private const val KEY_RESTORED = "restored_from"

/** Xem trước tệp .ics: 2 mục đầu theo DTSTART (chỉ đọc lại tệp lõi đã sinh để hiển thị). */
fun icsPreview(content: String, n: Int = 2): List<String> {
    val out = ArrayList<Pair<String, String>>()
    var start: String? = null
    var summary: String? = null
    for (raw in content.split("\r\n", "\n")) {
        when {
            raw.startsWith("BEGIN:VEVENT") -> { start = null; summary = null }
            raw.startsWith("DTSTART") -> start = raw.substringAfter(':').take(8)
            raw.startsWith("SUMMARY:") -> summary = raw.substringAfter(':').replace("\\,", ",").replace("\\;", ";")
            raw.startsWith("END:VEVENT") -> if (start != null && summary != null) out.add(start!! to summary!!)
        }
    }
    return out.sortedBy { it.first }.take(n).map { (d, s) ->
        "${d.substring(6, 8)}/${d.substring(4, 6)}/${d.substring(0, 4)} — $s"
    }
}

/** Sao lưu và lịch điện thoại — 08 §3.8 (bản Android: chia sẻ .ics, lưu/mở tệp qua Storage Access Framework). */
@Composable
fun BackupScreen(x: PeopleCtx) {
    SecureScreen() // BM2-04: ô mật khẩu sao lưu, xem trước tệp sao lưu
    val ctx = LocalContext.current
    val scope = rememberCoroutineScope()
    val settings = remember(x.rev) { runCatching { x.repo.settings() }.getOrNull() }
    val ds = remember(x.rev) { runCatching { Triple(x.repo.persons().size, x.repo.events().size, x.repo.persons().count { it.photoId != null }) }.getOrDefault(Triple(0, 0, 0)) }

    // ---- A. .ics ----
    var ics by remember { mutableStateOf<IcsSummary?>(null) }
    var icsErr by remember { mutableStateOf<String?>(null) }
    val today = x.today
    PCard(title = "Để lịch điện thoại cùng nhắc: xuất lịch (.ics)") {
        Body(
            "Ứng dụng đã tự nhắc bằng thông báo trên máy này. Nếu muốn ứng dụng Lịch của điện thoại (hoặc máy khác) " +
                "cũng nhắc, hãy xuất tệp lịch và mở bằng ứng dụng Lịch.",
        )
        PrimaryButton("Xuất tệp lịch (.ics)…", {
            icsErr = null
            runCatching { x.repo.icsSummary(today) }.onSuccess { if (it.events == 0u) icsErr = "Chưa có hồ sơ nào bật nhắc." else ics = it }
                .onFailure { icsErr = errText(it) }
        }, Modifier.fillMaxWidth())
        icsErr?.let { Fine(it) }
    }
    ics?.let { e ->
        // Hộp xác nhận S10 theo `ics_summary` của lõi; tệp chỉ được sinh khi người dùng bấm chia sẻ.
        ConfirmDialog(
            title = "Xuất tệp lịch", confirm = "Chia sẻ tệp .ics",
            onDismiss = { ics = null },
            onConfirm = {
                ics = null
                runCatching { shareIcs(ctx, x.repo.ics(today)) }.onSuccess { x.toast("Đã tạo tệp lịch. Chọn ứng dụng Lịch để nhập.") }
                    .onFailure { x.toast(errText(it)) }
            },
        ) {
            Body(e.privacyText)
            Body("Gồm ${e.events} ngày trong 10 năm tới (từ ${PFmt.dmy(e.from.local())} đến ${PFmt.dmy(e.to.local())}), mỗi ngày có lời nhắc như đã đặt.")
            if (e.preview.isNotEmpty()) {
                Fine("Xem trước:")
                e.preview.forEach { Body("• $it") }
                if (e.events.toInt() > e.preview.size) Fine("… và ${e.events.toInt() - e.preview.size} ngày khác")
            }
            Fine("Không gồm thông tin báo tử. Mở tệp bằng ứng dụng Lịch của điện thoại hoặc máy tính để nhập.")
        }
    }

    // ---- B. Tạo tệp sao lưu ----
    var exportOpen by remember { mutableStateOf(false) }
    var pw1 by remember { mutableStateOf("") }
    var pw2 by remember { mutableStateOf("") }
    var pwTouched by remember { mutableStateOf(false) }
    var busy by remember { mutableStateOf(false) }
    var exportErr by remember { mutableStateOf<String?>(null) }
    val createDoc = rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("application/octet-stream")) { uri ->
        // BM2-17: xóa mật khẩu ở mọi nhánh (hủy chọn nơi lưu, lỗi, thành công) — người dùng nhập lại khi thử lại.
        if (uri == null) { busy = false; pw1 = ""; pw2 = ""; pwTouched = false; return@rememberLauncherForActivityResult }
        val pw = pw1
        scope.launch {
            val r = withContext(Dispatchers.Default) {
                runCatching {
                    val bytes = x.repo.exportBackup(pw, BuildConfig.VERSION_NAME)
                    ctx.contentResolver.openOutputStream(uri, "wt")?.use { it.write(bytes) } ?: error("Không ghi được tệp.")
                }
            }
            busy = false
            pw1 = ""; pw2 = ""; pwTouched = false
            r.onSuccess {
                exportOpen = false
                // 03 §2.4: chỉ khi hệ điều hành báo ghi xong mới ghi nhận "đã sao lưu" (hủy, lỗi ghi → không gọi).
                runCatching { x.repo.markBackupSaved() }
                x.changed()
                x.toast("Đã tạo tệp sao lưu. Hãy cất tệp ở nơi an toàn (ổ đám mây, USB) — tệp không tự gửi đi đâu.")
            }.onFailure { exportErr = "Chưa tạo được tệp sao lưu: ${errText(it)}" }
        }
    }
    val restoredFrom = remember(x.rev) { ctx.getSharedPreferences(UI_PREFS, Context.MODE_PRIVATE).getString(KEY_RESTORED, null) }
    PCard(title = "Sao lưu dữ liệu") {
        val last = settings?.sn("last_backup_at")
        Body(if (last != null) "Lần sao lưu gần nhất: ${PFmt.dateOfInstant(last)}" else if (restoredFrom != null) "Chưa sao lưu lần nào trên máy này" else "Chưa sao lưu lần nào")
        restoredFrom?.let { Fine("Dữ liệu được khôi phục từ bản sao lưu ngày $it") }
        if (!exportOpen) {
            PrimaryButton("Tạo tệp sao lưu…", { exportOpen = true; exportErr = null }, Modifier.fillMaxWidth())
        } else {
            // Bản Android: các ô mật khẩu mở ngay trong khung (không đặt ô nhập trong hộp thoại) — 08 §13.
            // Quy tắc mật khẩu của lõi (`backup_password_error`, 03 §2.4; BM2-23): NFC, ≥ 8 điểm mã, hai lần giống nhau.
            val e1 = if (pwTouched) PeopleRepo.passwordError(pw1, null) else null
            val e2 = if (pwTouched && e1 == null) PeopleRepo.passwordError(pw1, pw2) else null
            H3("Tạo tệp sao lưu")
            Body("Tệp sẽ chứa ${ds.first} người, ${ds.second} sự kiện, ${ds.third} ảnh — đã mã hóa bằng mật khẩu.")
            Fine("Tệp gồm cả thông tin báo tử (người báo tin, nơi mất, lý do, ảnh giấy báo tử) — chỉ ai biết mật khẩu mới đọc được.")
            TextIn("Mật khẩu (ít nhất 8 ký tự)", pw1, { pw1 = it }, password = true, error = e1)
            TextIn("Nhập lại mật khẩu", pw2, { pw2 = it }, password = true, error = e2)
            NoticeBox("Lưu ý: quên mật khẩu là mất dữ liệu trong tệp — không ai lấy lại được, kể cả Clevix Lab.")
            Fine("Gợi ý: dùng một câu dễ nhớ, ví dụ \"nhà ở Hà Đông có cây bàng\".")
            if (busy) { LinearProgressIndicator(Modifier.fillMaxWidth()); Fine("Đang mã hóa… (có thể mất vài giây)") }
            exportErr?.let { ErrorText(it) }
            PrimaryButton(if (busy) "Đang mã hóa…" else "Tạo và lưu tệp", {
                pwTouched = true
                if (PeopleRepo.passwordError(pw1, pw2) == null) {
                    busy = true
                    exportErr = null
                    createDoc.launch("LichVanHoaViet-saoluu-$today.lvhv")
                }
            }, Modifier.fillMaxWidth(), enabled = !busy)
            SecondaryButton("Hủy", { exportOpen = false; pw1 = ""; pw2 = ""; pwTouched = false }, Modifier.fillMaxWidth(), enabled = !busy)
        }
    }

    // Chế độ chỉ đọc (B5): chỉ xuất .ics và tạo tệp sao lưu; không khôi phục, không đổi cài đặt.
    if (x.readOnly) return
    // ---- C. Khôi phục ----
    var picked by remember { mutableStateOf<Uri?>(null) }
    var pickedName by remember { mutableStateOf<String?>(null) }
    var pwIn by remember { mutableStateOf("") }
    var openErr by remember { mutableStateOf<String?>(null) }
    var opening by remember { mutableStateOf(false) }
    var verified by remember { mutableStateOf<Pair<VerifiedBackup, BackupPreview>?>(null) }
    val openDoc = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri != null) { picked = uri; pickedName = displayName(ctx, uri); openErr = null }
    }
    PCard(title = "Khôi phục từ tệp sao lưu") {
        SecondaryButton("Chọn tệp sao lưu…", { openDoc.launch(arrayOf("*/*")) }, Modifier.fillMaxWidth())
        Fine(pickedName?.let { "Đã chọn: $it" } ?: "Chưa chọn tệp.")
        TextIn("Mật khẩu của tệp", pwIn, { pwIn = it }, password = true)
        SecondaryButton(if (opening) "Đang mở…" else "Mở tệp", {
            val uri = picked
            if (uri == null) { openErr = "Hãy chọn tệp sao lưu"; return@SecondaryButton }
            opening = true; openErr = null
            val pw = pwIn
            scope.launch {
                val r = withContext(Dispatchers.Default) {
                    runCatching {
                        val bytes = readLimited(ctx, uri) ?: throw IllegalStateException("Sai mật khẩu hoặc tệp bị hỏng")
                        PeopleRepo.openBackupFile(bytes, pw)
                    }
                }
                opening = false
                // BM2-17: mật khẩu xóa ngay cả khi sai mật khẩu / tệp hỏng; khi đúng, xóa lúc đóng tấm xem trước.
                r.onSuccess { verified = it }.onFailure { pwIn = ""; openErr = errText(it) }
            }
        }, Modifier.fillMaxWidth(), enabled = !opening)
        if (opening) LinearProgressIndicator(Modifier.fillMaxWidth())
        openErr?.let { ErrorText(it) }
    }
    var replace by remember { mutableStateOf(false) }
    var confirmReplace by remember { mutableStateOf(false) }
    var importing by remember { mutableStateOf(false) }
    // Giải phóng ngay bản rõ ở phía Rust (không chờ GC) và xóa mật khẩu khỏi ô — MA3N-24-02.
    val dropVerified: () -> Unit = {
        verified?.first?.let { runCatching { it.destroy() } }
        verified = null
        pwIn = ""
        confirmReplace = false
        replace = false
    }
    val doImport: () -> Unit = {
        val v = verified
        if (v != null && !importing) {
            importing = true
            val rep = replace
            scope.launch {
                // Ghi giao dịch + tệp ảnh ngoài luồng giao diện (MA3N-24-01).
                val r = withContext(Dispatchers.IO) { runCatching { x.repo.applyImport(v.first, rep) } }
                importing = false
                r.onSuccess {
                    val prefs = ctx.getSharedPreferences(UI_PREFS, Context.MODE_PRIVATE).edit()
                    if (rep) prefs.putString(KEY_RESTORED, PFmt.dateOfInstant(v.second.exportedAt)) else prefs.remove(KEY_RESTORED)
                    prefs.apply()
                    picked = null; pickedName = null
                    x.changed()
                    x.toast("Đã khôi phục: ${v.second.persons} người, ${v.second.events} sự kiện.")
                }.onFailure { e -> openErr = "Chưa khôi phục được — dữ liệu trên máy giữ nguyên. ${errText(e)}" }
                dropVerified()
            }
        }
    }
    // Rời màn hình khi tấm xem trước còn mở (nút Quay lại, đổi thẻ): vẫn thả bản rõ ở phía Rust (BM2-17).
    val latestVerified = androidx.compose.runtime.rememberUpdatedState(verified)
    androidx.compose.runtime.DisposableEffect(Unit) {
        onDispose { latestVerified.value?.first?.let { runCatching { it.destroy() } } }
    }
    verified?.let { (_, pv) ->
        RestoreDialogs(
            pv = pv, currentPersons = ds.first, replace = replace, confirmReplace = confirmReplace, busy = importing,
            onReplace = { replace = it },
            onDismiss = { if (confirmReplace) confirmReplace = false else dropVerified() },
            onConfirm = { if (replace && !confirmReplace) confirmReplace = true else doImport() },
        )
    }

    // ---- D. Nhắc sao lưu ----
    PCard(title = "Nhắc sao lưu") {
        Fine("Nhắc sao lưu khi có thay đổi chưa sao lưu sau:")
        val cur = settings?.i("backup_nag_days") ?: 30
        val opts = listOf(30, 60, 90, 0)
        opts.forEach { d ->
            RadioRow(if (d == 0) "Không nhắc" else "$d ngày", cur == d, {
                settings?.let { s ->
                    s["backup_nag_days"] = d
                    runCatching { x.repo.putSettings(s) }.onSuccess { x.changed() }.onFailure { x.toast(errText(it)) }
                }
            })
        }
    }
}

/** Độ dài mật khẩu như lõi đếm: điểm mã sau NFC (backup/src/lib.rs) — không phải đơn vị UTF-16 (MA3N-24-08). */
fun pwLen(pw: String): Int {
    val n = java.text.Normalizer.normalize(pw, java.text.Normalizer.Form.NFC)
    return n.codePointCount(0, n.length)
}

/** Hai lần nhập giống nhau sau NFC (BM2-23): cùng mật khẩu gõ bằng hai bộ gõ (NFC/NFD) không bị báo khác. */
fun pwSame(a: String, b: String): Boolean =
    java.text.Normalizer.normalize(a, java.text.Normalizer.Form.NFC) == java.text.Normalizer.normalize(b, java.text.Normalizer.Form.NFC)

/** Hộp xem trước khôi phục (Gộp / Thay toàn bộ) và hộp xác nhận thứ hai "Thay toàn bộ" (08 §3.8 C). */
@Composable
fun RestoreDialogs(
    pv: BackupPreview,
    currentPersons: Int,
    replace: Boolean,
    confirmReplace: Boolean,
    busy: Boolean,
    onReplace: (Boolean) -> Unit,
    onDismiss: () -> Unit,
    onConfirm: () -> Unit,
) {
    if (!confirmReplace) {
        ConfirmDialog(
            title = "Khôi phục dữ liệu", confirm = if (busy) "Đang khôi phục…" else "Khôi phục",
            confirmEnabled = !busy, dismissEnabled = !busy,
            onDismiss = onDismiss, onConfirm = onConfirm,
        ) {
            Body("Tệp xuất ngày ${PFmt.dateOfInstant(pv.exportedAt)} từ ${platformName(pv.platform)}: ${pv.persons} người, ${pv.events} sự kiện, ${pv.photos} ảnh.")
            Fine("Cách nhập:")
            RadioRow("Gộp vào dữ liệu đang có (khuyên dùng) — giữ bản mới hơn của từng hồ sơ", !replace, { if (!busy) onReplace(false) })
            RadioRow("Thay toàn bộ — xóa dữ liệu đang có trên máy rồi dùng dữ liệu trong tệp", replace, { if (!busy) onReplace(true) })
            if (busy) LinearProgressIndicator(Modifier.fillMaxWidth())
        }
    } else {
        ConfirmDialog(
            title = "Thay toàn bộ?", confirm = if (busy) "Đang khôi phục…" else "Thay toàn bộ", danger = true,
            confirmEnabled = !busy, dismissEnabled = !busy,
            onDismiss = onDismiss, onConfirm = onConfirm,
        ) {
            Body("Dữ liệu hiện có trên máy ($currentPersons người) sẽ bị thay bằng dữ liệu trong tệp xuất ngày ${PFmt.dateOfInstant(pv.exportedAt)}. Không hoàn tác được.")
            if (busy) LinearProgressIndicator(Modifier.fillMaxWidth())
        }
    }
}

private fun platformName(p: String): String = when (p) {
    "pwa" -> "ứng dụng web"
    "android" -> "điện thoại Android"
    "ios" -> "iPhone/iPad"
    "macos" -> "máy Mac"
    "windows" -> "máy Windows"
    else -> p
}

private fun displayName(ctx: Context, uri: Uri): String? = runCatching {
    ctx.contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { c ->
        if (c.moveToFirst()) c.getString(0) else null
    }
}.getOrNull()

/**
 * Đọc tệp sao lưu: biết kích thước → kiểm 72 byte đầu và kích thước bằng `backup_header_check` của lõi **trước khi** đọc
 * cả tệp (03 §2.2; lỗi → ném câu của lõi); quá giới hạn (S12) → null.
 */
private fun readLimited(ctx: Context, uri: Uri): ByteArray? {
    val size = ctx.contentResolver.query(uri, arrayOf(OpenableColumns.SIZE), null, null, null)?.use { c ->
        if (c.moveToFirst() && !c.isNull(0)) c.getLong(0) else -1L
    } ?: -1L
    if (size > BACKUP_MAX_BYTES) return null
    if (size >= 0) {
        val head = ctx.contentResolver.openInputStream(uri)?.use { s ->
            val h = ByteArray(minOf(72L, size).toInt())
            var off = 0
            while (off < h.size) { val n = s.read(h, off, h.size - off); if (n < 0) break; off += n }
            h.copyOf(off)
        } ?: return null
        PeopleRepo.checkBackupHeader(head, size)
    }
    return ctx.contentResolver.openInputStream(uri)?.use { readBackupBytes(it, size) }
}

/**
 * Đọc thân tệp sao lưu với bộ nhớ có giới hạn (MA3N-24-03, S12): biết kích thước → cấp **đúng** một mảng và đọc đầy
 * (không nới bộ đệm, không chép lần hai); không biết → đọc theo khối, dừng ngay khi vượt giới hạn. Quá giới hạn, hoặc tệp
 * dài hơn kích thước đã báo → null (không cấp thêm bộ nhớ).
 */
fun readBackupBytes(input: java.io.InputStream, size: Long): ByteArray? {
    if (size > BACKUP_MAX_BYTES) return null
    if (size >= 0) {
        val buf = ByteArray(size.toInt())
        var off = 0
        while (off < buf.size) {
            val n = input.read(buf, off, buf.size - off)
            if (n < 0) return null // ngắn hơn kích thước đã báo
            off += n
        }
        if (input.read() >= 0) return null // dài hơn kích thước đã báo
        return buf
    }
    val out = java.io.ByteArrayOutputStream(1 shl 20)
    val chunk = ByteArray(64 * 1024)
    var total = 0L
    while (true) {
        val n = input.read(chunk)
        if (n < 0) break
        total += n
        if (total > BACKUP_MAX_BYTES) return null
        out.write(chunk, 0, n)
    }
    return out.toByteArray()
}

/**
 * Xóa tệp tạm để chia sẻ (`cache/ics/`, có tên và ngày giỗ) cũ hơn [olderThanMs] — BM2-16. Gọi khi mở ứng dụng (0: xóa
 * hết) và mỗi lần quay lại ứng dụng (ứng dụng nhận đã đọc xong). Không bao giờ ném lỗi.
 */
fun cleanShareCache(ctx: Context, olderThanMs: Long, nowMs: Long = System.currentTimeMillis()) {
    runCatching {
        File(ctx.cacheDir, "ics").listFiles()?.forEach { f ->
            if (olderThanMs <= 0L || nowMs - f.lastModified() >= olderThanMs) f.delete()
        }
    }
}

/** Ghi .ics vào cache/ics/ (xóa tệp cũ) rồi mở bảng chia sẻ của hệ thống. */
private fun shareIcs(ctx: Context, e: IcsExport) {
    val dir = File(ctx.cacheDir, "ics").apply { mkdirs() }
    cleanShareCache(ctx, 0L)
    val f = File(dir, e.fileName)
    f.writeText(e.content, Charsets.UTF_8)
    val uri = FileProvider.getUriForFile(ctx, ctx.packageName + ".files", f)
    val send = Intent(Intent.ACTION_SEND)
        .setType("text/calendar")
        .putExtra(Intent.EXTRA_STREAM, uri)
        .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
    ctx.startActivity(Intent.createChooser(send, "Mở tệp lịch bằng…").addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
}
