package vn.clevix.lichviet.ui.people

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.view.WindowManager
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.ui.platform.LocalContext

/**
 * BM2-04 (RA-SOAT-BAO-MAT-V02): màn hình có thông tin báo tử / ảnh giấy tờ / ô mật khẩu sao lưu đặt `FLAG_SECURE` —
 * ảnh đa nhiệm (Recents) chỉ là khung trống, không ai vuốt đa nhiệm đọc được. Chỉ áp cho các màn hình đó (không toàn ứng
 * dụng, để người dùng vẫn chụp được tờ lịch). Đếm số màn hình đang yêu cầu để chuyển giữa hai màn hình cần che không làm
 * hở một khung hình.
 */
object SecureWindow {
    private val holders = HashMap<Activity, Int>()

    fun acquire(a: Activity) {
        val n = (holders[a] ?: 0) + 1
        holders[a] = n
        if (n == 1) a.window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)
    }

    fun release(a: Activity) {
        val n = (holders[a] ?: 1) - 1
        if (n <= 0) {
            holders.remove(a)
            a.window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
        } else {
            holders[a] = n
        }
    }

    fun isSecure(a: Activity): Boolean =
        (a.window.attributes.flags and WindowManager.LayoutParams.FLAG_SECURE) != 0
}

fun Context.findActivity(): Activity? {
    var c: Context? = this
    while (c is ContextWrapper) {
        if (c is Activity) return c
        c = c.baseContext
    }
    return null
}

/** Đặt `FLAG_SECURE` khi [enabled] và màn hình đang hiện; gỡ khi rời màn hình. */
@Composable
fun SecureScreen(enabled: Boolean = true) {
    val a = LocalContext.current.findActivity() ?: return
    DisposableEffect(a, enabled) {
        if (enabled) SecureWindow.acquire(a)
        onDispose { if (enabled) SecureWindow.release(a) }
    }
}
