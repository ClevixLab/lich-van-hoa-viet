package vn.clevix.lichviet.ui.people

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.MediaStore
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.core.content.FileProvider
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.Photos

/** Ảnh mâm lễ (01 §3.7, 08 §16.2): cạnh dài ≤ 1600 px, ≤ 1 MB, mã hóa lại (không siêu dữ liệu). */
const val TRAY_EDGE = 1600
const val TRAY_STORED = 1024 * 1024

/** Câu lỗi của mục ảnh mâm lễ (08 §16.2, như ảnh chân dung). */
fun trayError(e: Throwable): String =
    if (e.message?.contains("25 MB") == true) "Ảnh quá lớn (tối đa 25 MB)." else "Không đọc được ảnh này. Hãy chọn một ảnh khác."

/**
 * Ảnh vừa chụp bằng ứng dụng máy ảnh của hệ thống: tệp tạm `cache/capture/mam.jpg` (FileProvider, quyền ghi chỉ cho
 * đúng Intent chụp). Ứng dụng không xin quyền CAMERA — `ACTION_IMAGE_CAPTURE` do ứng dụng máy ảnh làm.
 */
object TrayCapture {
    fun file(ctx: Context) = File(File(ctx.cacheDir, "capture").apply { mkdirs() }, "mam.jpg")
    fun uri(ctx: Context): Uri = FileProvider.getUriForFile(ctx, ctx.packageName + ".files", file(ctx))
    fun cleanup(ctx: Context) {
        runCatching {
            val f = File(File(ctx.cacheDir, "capture"), "mam.jpg")
            if (f.exists()) {
                runCatching { ctx.revokeUriPermission(uri(ctx), Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION) }
                f.delete()
            }
        }
    }

    /** Hộp chọn của hệ thống: thư viện ảnh (`GET_CONTENT`, mọi loại ảnh, không cần quyền bộ nhớ) + chụp ảnh. */
    fun chooser(ctx: Context): Intent {
        cleanup(ctx)
        val out = uri(ctx)
        val pick = Intent(Intent.ACTION_GET_CONTENT).setType("image/*").addCategory(Intent.CATEGORY_OPENABLE)
        val shoot = Intent(MediaStore.ACTION_IMAGE_CAPTURE)
            .putExtra(MediaStore.EXTRA_OUTPUT, out)
            .addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION or Intent.FLAG_GRANT_READ_URI_PERMISSION)
        return Intent.createChooser(pick, "Chọn ảnh mâm lễ").putExtra(Intent.EXTRA_INITIAL_INTENTS, arrayOf(shoot))
    }
}

/** Mục "Ảnh mâm lễ ngày giỗ" (08 §16.2) — đặt ngay sau ô "Lời tưởng niệm". [focus]: cuộn tới mục này khi mở. */
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
@Composable
fun TraySection(f: PersonFormState, repo: PeopleRepo, focus: Boolean) {
    val ctx = LocalContext.current
    val scope = rememberCoroutineScope()
    var err by remember { mutableStateOf<String?>(null) }
    val bring = remember { BringIntoViewRequester() }
    LaunchedEffect(focus) { if (focus) bring.bringIntoView() }
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.StartActivityForResult()) { r ->
        val captured = TrayCapture.file(ctx)
        val uri: Uri? = r.data?.data ?: if (captured.exists() && captured.length() > 0) Uri.fromFile(captured) else null
        if (uri == null) { TrayCapture.cleanup(ctx); return@rememberLauncherForActivityResult }
        f.trayBusy = true
        err = null
        scope.launch {
            val res = withContext(Dispatchers.Default) {
                runCatching {
                    if (uri.scheme == "file") Photos.encode(captured.readBytes().also { if (it.size > Photos.MAX_INPUT) throw Photos.PhotoError("Ảnh lớn hơn 25 MB") }, TRAY_EDGE, TRAY_STORED)
                    else Photos.process(ctx, uri, TRAY_EDGE, TRAY_STORED)
                }
            }
            TrayCapture.cleanup(ctx)
            f.trayBusy = false
            res.onSuccess { f.tray = it; f.trayRemoved = false }.onFailure { err = trayError(it) }
        }
    }
    val stored = remember(f.trayStoredId) { f.trayStoredId?.let { repo.photo(it) } }
    val bytes = f.tray?.bytes ?: if (f.trayRemoved) null else stored
    val img = remember(bytes) { bytes?.let { Photos.bitmap(it, 480)?.asImageBitmap() } }
    Column(Modifier.bringIntoViewRequester(bring).testTag("tray-form"), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        H3("Ảnh mâm lễ ngày giỗ")
        when {
            f.trayBusy -> Body("Đang xử lý ảnh…")
            img == null -> {
                Body("Chưa có ảnh. Ngày giỗ chỉ hiện bát hương và nến.")
                SecondaryButton("Chọn ảnh mâm lễ…", { launcher.launch(TrayCapture.chooser(ctx)) })
            }
            else -> Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Image(img, contentDescription = "Ảnh mâm lễ của gia đình", contentScale = ContentScale.Crop, modifier = Modifier.size(120.dp, 90.dp))
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    SecondaryButton("Đổi ảnh", { launcher.launch(TrayCapture.chooser(ctx)) })
                    SecondaryButton("Bỏ ảnh", { f.tray = null; f.trayRemoved = true })
                }
            }
        }
        err?.let { ErrorText(it) }
        Fine("Ảnh chỉ lưu trên máy này và trong tệp sao lưu mã hóa. Ứng dụng tự bỏ thông tin vị trí và máy chụp trong ảnh.")
    }
}
