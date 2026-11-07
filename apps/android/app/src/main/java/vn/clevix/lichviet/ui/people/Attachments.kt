package vn.clevix.lichviet.ui.people

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.OpenableColumns
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
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
import androidx.core.content.FileProvider
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import vn.clevix.lichviet.people.JMap
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.Photos
import vn.clevix.lichviet.people.s
import vn.clevix.lichviet.people.sn

/** Loại tệp đính kèm báo tử (01 §3.2a, YC-2026-07). */
val ATT_KINDS = listOf("tin_bao_tu", "giay_chung_tu", "khac")
val ATT_KIND_TEXT = mapOf("tin_bao_tu" to "Tin báo tử", "giay_chung_tu" to "Giấy chứng tử", "khac" to "Tệp khác")
const val MAX_ATTACHMENTS = 6
const val MAX_PDF = 10L * 1024 * 1024

/** Tệp vừa chọn, chưa ghi vào kho: ảnh đã mã hóa lại (JPEG) hoặc PDF nguyên bản. */
class PendingAtt(val mime: String, val fileName: String?, val bytes: ByteArray)

/** Một phần tử danh sách `attachments`: [id] của phần tử, [fileId] của bản ghi Attachment (null khi còn [pending]). */
class AttItem(val id: String, val kind: String, val fileId: String?, val pending: PendingAtt?) {
    val isPdf: Boolean get() = pending?.mime == "application/pdf"
}

/** Tên tệp gốc (chỉ để hiện trên trang hồ sơ), tối đa 120 điểm mã (01 §3.8a). */
private fun displayName(ctx: Context, uri: Uri): String? = runCatching {
    ctx.contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { c ->
        if (c.moveToFirst()) c.getString(0) else null
    }
}.getOrNull()?.trim()?.takeIf { it.isNotEmpty() }?.let { n ->
    if (n.codePointCount(0, n.length) <= 120) n else n.substring(0, n.offsetByCodePoints(0, 120))
}

/**
 * Đọc PDF chọn qua bộ chọn tệp (SAF, không cần quyền bộ nhớ). Kiểm kích thước **trước khi** đọc (theo bộ chọn tệp), đọc
 * tối đa 10 MB + 1 byte, kiểm `%PDF-` (RA-SOAT-BAO-MAT-V02 §5.3 mục 2). Không bao giờ giải mã hay hiển thị PDF trong
 * ứng dụng. Byte lưu nguyên bản.
 */
fun readPdf(ctx: Context, uri: Uri): PendingAtt {
    val cr = ctx.contentResolver
    val size = cr.query(uri, arrayOf(OpenableColumns.SIZE), null, null, null)?.use { c ->
        if (c.moveToFirst() && !c.isNull(0)) c.getLong(0) else -1L
    } ?: -1L
    if (size > MAX_PDF) throw Photos.PhotoError("Tệp PDF lớn hơn 10 MB — hãy chọn tệp nhỏ hơn")
    val bytes = cr.openInputStream(uri)?.use { s ->
        val out = java.io.ByteArrayOutputStream()
        val buf = ByteArray(64 * 1024)
        var total = 0L
        while (true) {
            val n = s.read(buf)
            if (n < 0) break
            total += n
            if (total > MAX_PDF) throw Photos.PhotoError("Tệp PDF lớn hơn 10 MB — hãy chọn tệp nhỏ hơn")
            out.write(buf, 0, n)
        }
        out.toByteArray()
    } ?: throw Photos.PhotoError("Không đọc được tệp này")
    checkPdf(bytes)
    return PendingAtt("application/pdf", displayName(ctx, uri), bytes)
}

/**
 * Như `media::check_attachment` của lõi (lõi kiểm lại khi lưu): `%PDF-` ở đầu, `%%EOF` trong 1 024 byte cuối (tệp không
 * bị cụt — MA3C-24-03), 1 byte … 10 MB. Kiểm ngay khi chọn để báo lỗi sớm.
 */
fun checkPdf(bytes: ByteArray) {
    if (bytes.isEmpty() || bytes.size > MAX_PDF) throw Photos.PhotoError("Tệp PDF lớn hơn 10 MB — hãy chọn tệp nhỏ hơn")
    val head = String(bytes, 0, minOf(5, bytes.size), Charsets.ISO_8859_1)
    if (head != "%PDF-") throw Photos.PhotoError("Tệp này không phải PDF hợp lệ")
    val from = maxOf(0, bytes.size - PDF_EOF_WINDOW)
    val tail = String(bytes, from, bytes.size - from, Charsets.ISO_8859_1)
    if (!tail.contains("%%EOF")) throw Photos.PhotoError("Tệp PDF này bị thiếu phần cuối (có thể tải chưa xong) — hãy chọn lại tệp đầy đủ")
}

/** `%%EOF` phải nằm trong ngần này byte cuối (= `PDF_EOF_WINDOW` của lõi). */
const val PDF_EOF_WINDOW = 1024

/** Ảnh đính kèm: giải mã + xoay + thu cạnh dài ≤ 2048 + JPEG ≤ 2 MB, bỏ siêu dữ liệu (01 §3.8a, S9). */
fun readImageAtt(ctx: Context, uri: Uri): PendingAtt {
    val img = Photos.process(ctx, uri, Photos.ATT_EDGE, Photos.ATT_STORED)
    return PendingAtt("image/jpeg", displayName(ctx, uri), img.bytes)
}

/** Danh sách tệp đính kèm trong biểu mẫu thông tin báo tử. */
@Composable
fun AttachmentEditor(st: NoticeState, repo: PeopleRepo) {
    val ctx = LocalContext.current
    val scope = rememberCoroutineScope()
    var err by remember { mutableStateOf<String?>(null) }
    var busy by remember { mutableStateOf(false) }
    val add: (suspend () -> PendingAtt) -> Unit = { read ->
        busy = true
        scope.launch {
            val r = withContext(Dispatchers.IO) { runCatching { read() } }
            busy = false
            r.onSuccess { st.items.add(AttItem(PeopleRepo.uuid(), st.newKind, null, it)); err = null }
                .onFailure { err = it.message ?: "Không đọc được tệp này" }
        }
    }
    val pickImage = rememberLauncherForActivityResult(ActivityResultContracts.PickVisualMedia()) { uri ->
        if (uri != null) add { readImageAtt(ctx, uri) }
    }
    val pickPdf = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri != null) add { readPdf(ctx, uri) }
    }
    H3("Tệp đính kèm — tin báo tử, giấy chứng tử (tối đa $MAX_ATTACHMENTS)")
    if (st.items.isEmpty()) Fine("Chưa có tệp nào.")
    st.items.toList().forEachIndexed { i, it ->
        AttRow(it, i, repo) { st.items.remove(it) }
    }
    if (st.items.size < MAX_ATTACHMENTS) {
        Fine("Loại tệp sắp thêm")
        Choice(ATT_KINDS.map { ATT_KIND_TEXT.getValue(it) }, ATT_KINDS.indexOf(st.newKind).coerceAtLeast(0), { st.newKind = ATT_KINDS[it] })
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
            SecondaryButton("Thêm ảnh", {
                pickImage.launch(PickVisualMediaRequest(ActivityResultContracts.PickVisualMedia.ImageOnly))
            }, Modifier.weight(1f), enabled = !busy)
            SecondaryButton("Thêm tệp PDF", { pickPdf.launch(arrayOf("application/pdf")) }, Modifier.weight(1f), enabled = !busy)
        }
    } else {
        Fine("Đã đủ $MAX_ATTACHMENTS tệp.")
    }
    if (busy) Fine("Đang xử lý tệp…")
    err?.let { ErrorText(it) }
    Fine("Ảnh được nén lại và bỏ thông tin vị trí, máy chụp. Tệp PDF được lưu nguyên bản (có thể còn tên máy quét, phần mềm, ngày quét).")
}

@Composable
private fun AttRow(it: AttItem, i: Int, repo: PeopleRepo, onRemove: () -> Unit) {
    val label = "${ATT_KIND_TEXT[it.kind] ?: "Tệp"} ${i + 1}"
    val meta = remember(it.fileId) { it.fileId?.let { id -> repo.attachmentMeta()[id] } }
    val pdf = it.isPdf || meta?.s("mime") == "application/pdf"
    // Nhãn một dòng riêng (không bị ép hẹp bên cạnh ảnh); dưới là ảnh thu nhỏ / "PDF" và nút bỏ.
    Column(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Body("$label · ${if (pdf) "PDF" else "Ảnh"}")
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp), modifier = Modifier.fillMaxWidth()) {
            if (!pdf) {
                val bytes = remember(it) { it.pending?.bytes ?: it.fileId?.let { id -> repo.attachmentBytes(id) } }
                val img = remember(bytes) { bytes?.let { b -> Photos.bitmap(b, 512)?.asImageBitmap() } }
                if (img != null) Image(img, contentDescription = label, contentScale = ContentScale.Fit, modifier = Modifier.size(96.dp))
            } else {
                (it.pending?.fileName ?: meta?.sn("file_name"))?.let { n -> Fine(n, Modifier.weight(1f)) }
            }
            if (!pdf) androidx.compose.foundation.layout.Spacer(Modifier.weight(1f))
            SecondaryButton("Bỏ tệp", onRemove)
        }
    }
}

/** Tệp đính kèm trên trang hồ sơ (chỉ ở đây, cùng thông tin báo tử — R-PRIV-2). Ảnh hiện trong trang; PDF chỉ có nhãn. */
@Composable
fun AttachmentView(repo: PeopleRepo, list: List<JMap>) {
    if (list.isEmpty()) return
    val ctx = LocalContext.current
    val meta = remember(list) { repo.attachmentMeta() }
    var confirmPdf by remember { mutableStateOf<String?>(null) }
    var err by remember { mutableStateOf<String?>(null) }
    H3("Tệp đính kèm")
    list.forEachIndexed { i, a ->
        val fid = a.s("file_id")
        val m = meta[fid]
        val label = "${ATT_KIND_TEXT[a.s("kind")] ?: "Tệp"} ${i + 1}"
        if (m?.s("mime") == "application/pdf") {
            Column(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Body("$label · PDF")
                m?.sn("file_name")?.let { Fine(it) }
                SecondaryButton("Mở bằng ứng dụng khác…", { confirmPdf = fid })
            }
        } else {
            val img = rememberPhoto(remember(fid) { repo.attachmentBytes(fid) })
            Body(label)
            m?.sn("file_name")?.let { Fine(it) }
            if (img != null) {
                Image(img, contentDescription = label, contentScale = ContentScale.Fit, modifier = Modifier.fillMaxWidth().heightIn(max = 480.dp))
            } else {
                Fine("Không hiện được ảnh này.")
            }
        }
    }
    err?.let { ErrorText(it) }
    confirmPdf?.let { fid ->
        ConfirmDialog(
            title = "Mở tệp PDF bằng ứng dụng khác?", confirm = "Mở tệp",
            onDismiss = { confirmPdf = null },
            onConfirm = {
                confirmPdf = null
                err = runCatching { AttachmentOpener.openPdf(ctx, repo.attachmentBytes(fid) ?: error("Không đọc được tệp")) }
                    .exceptionOrNull()?.let { e -> if (e is ActivityNotFoundException) "Máy chưa có ứng dụng đọc PDF." else "Chưa mở được tệp: ${e.message}" }
            },
        ) {
            Body("Tệp sẽ được mở bằng một ứng dụng đọc PDF trên máy. Ứng dụng đó đọc được nội dung tệp (thông tin báo tử).")
            Fine("Lịch và Văn hóa Việt chỉ cho ứng dụng đó quyền đọc tệp này một lần, dùng tên chung TepDinhKem.pdf, và xóa bản tạm khi bạn quay lại.")
        }
    }
}

/**
 * Mở PDF bằng ứng dụng ngoài theo RA-SOAT-BAO-MAT-V02 §5.3 mục 3: bản tạm trong `cache/att/` với tên chung
 * `TepDinhKem.pdf`, `ACTION_VIEW` + `FLAG_GRANT_READ_URI_PERMISSION` chỉ cho Intent đó (không `grantUriPermission` cho
 * gói nào), xóa tệp và thu hồi quyền khi quay lại ứng dụng ([cleanup], gọi ở `onResume`) và khi mở ứng dụng.
 */
object AttachmentOpener {
    const val FILE_NAME = "TepDinhKem.pdf"

    private fun dir(ctx: Context) = File(ctx.cacheDir, "att")

    fun openPdf(ctx: Context, bytes: ByteArray) {
        checkPdf(bytes)
        cleanup(ctx)
        val d = dir(ctx).apply { mkdirs() }
        val f = File(d, FILE_NAME)
        f.writeBytes(bytes)
        val uri = FileProvider.getUriForFile(ctx, ctx.packageName + ".files", f)
        val view = Intent(Intent.ACTION_VIEW)
            .setDataAndType(uri, "application/pdf")
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        if (ctx !is android.app.Activity) view.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        try {
            ctx.startActivity(view)
        } catch (e: ActivityNotFoundException) {
            cleanup(ctx)
            throw e
        }
    }

    /** Thu hồi quyền đọc và xóa bản tạm (không bao giờ ném lỗi). */
    fun cleanup(ctx: Context) {
        runCatching {
            val f = File(dir(ctx), FILE_NAME)
            if (f.exists()) {
                runCatching {
                    val uri = FileProvider.getUriForFile(ctx, ctx.packageName + ".files", f)
                    ctx.revokeUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
                }
            }
            dir(ctx).listFiles()?.forEach { it.delete() }
        }
    }
}
