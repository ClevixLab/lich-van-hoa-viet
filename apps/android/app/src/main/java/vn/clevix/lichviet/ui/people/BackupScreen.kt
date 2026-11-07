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
import vn.clevix.lichviet.core.VerifiedBackup
import vn.clevix.lichviet.people.PFmt
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.errText
import vn.clevix.lichviet.people.i
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
    val ctx = LocalContext.current
    val scope = rememberCoroutineScope()
    val settings = remember(x.rev) { runCatching { x.repo.settings() }.getOrNull() }
    val ds = remember(x.rev) { runCatching { Triple(x.repo.persons().size, x.repo.events().size, x.repo.persons().count { it.photoId != null }) }.getOrDefault(Triple(0, 0, 0)) }

    // ---- A. .ics ----
    var ics by remember { mutableStateOf<IcsExport?>(null) }
    var icsErr by remember { mutableStateOf<String?>(null) }
    val today = x.today
    PCard(title = "Để lịch điện thoại cùng nhắc: xuất lịch (.ics)") {
        Body(
            "Ứng dụng đã tự nhắc bằng thông báo trên máy này. Nếu muốn ứng dụng Lịch của điện thoại (hoặc máy khác) " +
                "cũng nhắc, hãy xuất tệp lịch và mở bằng ứng dụng Lịch.",
        )
        PrimaryButton("Xuất tệp lịch (.ics)…", {
            icsErr = null
            runCatching { x.repo.ics(today) }.onSuccess { if (it.events == 0u) icsErr = "Chưa có hồ sơ nào bật nhắc." else ics = it }
                .onFailure { icsErr = errText(it) }
        }, Modifier.fillMaxWidth())
        icsErr?.let { Fine(it) }
    }
    ics?.let { e ->
        val prev = remember(e) { icsPreview(e.content) }
        val end = today.plusYears(10).minusDays(1)
        ConfirmDialog(
            title = "Xuất tệp lịch", confirm = "Chia sẻ tệp .ics",
            onDismiss = { ics = null },
            onConfirm = {
                ics = null
                runCatching { shareIcs(ctx, e) }.onSuccess { x.toast("Đã tạo tệp lịch. Chọn ứng dụng Lịch để nhập.") }
                    .onFailure { x.toast(errText(it)) }
            },
        ) {
            Body("Tệp này chứa tên, ngày giỗ, sinh nhật và sự kiện của ${e.persons} người. Bất kỳ ai có tệp đều đọc được.")
            Body("Gồm ${e.events} ngày trong 10 năm tới (từ ${PFmt.dmy(today)} đến ${PFmt.dmy(end)}), mỗi ngày có lời nhắc như đã đặt.")
            if (prev.isNotEmpty()) {
                Fine("Xem trước:")
                prev.forEach { Body("• $it") }
                if (e.events.toInt() > prev.size) Fine("… và ${e.events.toInt() - prev.size} ngày khác")
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
        if (uri == null) { busy = false; return@rememberLauncherForActivityResult }
        val pw = pw1
        scope.launch {
            val r = withContext(Dispatchers.Default) {
                runCatching {
                    val bytes = x.repo.exportBackup(pw, BuildConfig.VERSION_NAME)
                    ctx.contentResolver.openOutputStream(uri, "wt")?.use { it.write(bytes) } ?: error("Không ghi được tệp.")
                }
            }
            busy = false
            r.onSuccess {
                exportOpen = false; pw1 = ""; pw2 = ""; pwTouched = false
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
        PrimaryButton("Tạo tệp sao lưu…", { exportOpen = true; exportErr = null }, Modifier.fillMaxWidth())
    }
    if (exportOpen) {
        val e1 = if (pwTouched && pw1.length < 8) "Mật khẩu cần ít nhất 8 ký tự" else null
        val e2 = if (pwTouched && e1 == null && pw1 != pw2) "Hai lần nhập mật khẩu chưa giống nhau" else null
        ConfirmDialog(
            title = "Tạo tệp sao lưu", confirm = if (busy) "Đang mã hóa…" else "Tạo và lưu tệp",
            confirmEnabled = !busy, dismissEnabled = !busy,
            onDismiss = { exportOpen = false; pw1 = ""; pw2 = ""; pwTouched = false },
            onConfirm = {
                pwTouched = true
                if (pw1.length >= 8 && pw1 == pw2) {
                    busy = true
                    exportErr = null
                    createDoc.launch("LichVanHoaViet-saoluu-$today.lvhv")
                }
            },
        ) {
            Body("Tệp sẽ chứa ${ds.first} người, ${ds.second} sự kiện, ${ds.third} ảnh — đã mã hóa bằng mật khẩu.")
            Fine("Tệp gồm cả thông tin báo tử (người báo tin, nơi mất, lý do, ảnh giấy báo tử) — chỉ ai biết mật khẩu mới đọc được.")
            TextIn("Mật khẩu (ít nhất 8 ký tự)", pw1, { pw1 = it }, password = true, error = e1)
            TextIn("Nhập lại mật khẩu", pw2, { pw2 = it }, password = true, error = e2)
            NoticeBox("Lưu ý: quên mật khẩu là mất dữ liệu trong tệp — không ai lấy lại được, kể cả Clevix Lab.")
            Fine("Gợi ý: dùng một câu dễ nhớ, ví dụ \"nhà ở Hà Đông có cây bàng\".")
            if (busy) { LinearProgressIndicator(Modifier.fillMaxWidth()); Fine("Đang mã hóa… (có thể mất vài giây)") }
            exportErr?.let { ErrorText(it) }
        }
    }

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
                r.onSuccess { verified = it }.onFailure { openErr = errText(it) }
            }
        }, Modifier.fillMaxWidth(), enabled = !opening)
        if (opening) LinearProgressIndicator(Modifier.fillMaxWidth())
        openErr?.let { ErrorText(it) }
    }
    var replace by remember { mutableStateOf(false) }
    var confirmReplace by remember { mutableStateOf(false) }
    val doImport: () -> Unit = {
        val v = verified
        if (v != null) {
            try {
                x.repo.applyImport(v.first, replace)
                val prefs = ctx.getSharedPreferences(UI_PREFS, Context.MODE_PRIVATE).edit()
                if (replace) prefs.putString(KEY_RESTORED, PFmt.dateOfInstant(v.second.exportedAt)) else prefs.remove(KEY_RESTORED)
                prefs.apply()
                verified = null; picked = null; pickedName = null; pwIn = ""
                x.changed()
                x.toast("Đã khôi phục: ${v.second.persons} người, ${v.second.events} sự kiện.")
            } catch (e: Exception) {
                verified = null
                openErr = "Chưa khôi phục được — dữ liệu trên máy giữ nguyên. ${errText(e)}"
            }
        }
    }
    verified?.let { (_, pv) ->
        if (!confirmReplace) {
            ConfirmDialog(
                title = "Khôi phục dữ liệu", confirm = "Khôi phục",
                onDismiss = { verified = null },
                onConfirm = { if (replace) confirmReplace = true else doImport() },
            ) {
                Body("Tệp xuất ngày ${PFmt.dateOfInstant(pv.exportedAt)} từ ${platformName(pv.platform)}: ${pv.persons} người, ${pv.events} sự kiện, ${pv.photos} ảnh.")
                Fine("Cách nhập:")
                RadioRow("Gộp vào dữ liệu đang có (khuyên dùng) — giữ bản mới hơn của từng hồ sơ", !replace, { replace = false })
                RadioRow("Thay toàn bộ — xóa dữ liệu đang có trên máy rồi dùng dữ liệu trong tệp", replace, { replace = true })
            }
        } else {
            ConfirmDialog(
                title = "Thay toàn bộ?", confirm = "Thay toàn bộ", danger = true,
                onDismiss = { confirmReplace = false },
                onConfirm = { confirmReplace = false; doImport() },
            ) {
                Body("Dữ liệu hiện có trên máy (${ds.first} người) sẽ bị thay bằng dữ liệu trong tệp xuất ngày ${PFmt.dateOfInstant(pv.exportedAt)}. Không hoàn tác được.")
            }
        }
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

/** Đọc tệp sao lưu, từ chối trước khi đọc nếu quá giới hạn (S12) → null. */
private fun readLimited(ctx: Context, uri: Uri): ByteArray? {
    val size = ctx.contentResolver.query(uri, arrayOf(OpenableColumns.SIZE), null, null, null)?.use { c ->
        if (c.moveToFirst() && !c.isNull(0)) c.getLong(0) else -1L
    } ?: -1L
    if (size > BACKUP_MAX_BYTES) return null
    return ctx.contentResolver.openInputStream(uri)?.use { s ->
        val out = java.io.ByteArrayOutputStream()
        val buf = ByteArray(64 * 1024)
        var total = 0L
        while (true) {
            val n = s.read(buf)
            if (n < 0) break
            total += n
            if (total > BACKUP_MAX_BYTES) return null
            out.write(buf, 0, n)
        }
        out.toByteArray()
    }
}

/** Ghi .ics vào cache/ics/ (xóa tệp cũ) rồi mở bảng chia sẻ của hệ thống. */
private fun shareIcs(ctx: Context, e: IcsExport) {
    val dir = File(ctx.cacheDir, "ics").apply { mkdirs() }
    dir.listFiles()?.forEach { it.delete() }
    val f = File(dir, e.fileName)
    f.writeText(e.content, Charsets.UTF_8)
    val uri = FileProvider.getUriForFile(ctx, ctx.packageName + ".files", f)
    val send = Intent(Intent.ACTION_SEND)
        .setType("text/calendar")
        .putExtra(Intent.EXTRA_STREAM, uri)
        .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
    ctx.startActivity(Intent.createChooser(send, "Mở tệp lịch bằng…").addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
}
