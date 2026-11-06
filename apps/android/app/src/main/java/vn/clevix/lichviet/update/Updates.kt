package vn.clevix.lichviet.update

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import java.time.LocalDate
import vn.clevix.lichviet.BuildConfig

/**
 * Mở cửa hàng/trang tải về QUA HỆ ĐIỀU HÀNH (Intent ACTION_VIEW). Ứng dụng không có quyền INTERNET và
 * không tự kết nối mạng; Google Play hoặc trình duyệt là ứng dụng khác, do người dùng chủ động mở.
 */
object Updates {
    val versionName: String get() = BuildConfig.VERSION_NAME
    val buildNumber: String get() = BuildConfig.BUILD_NUMBER
    val buildDate: LocalDate? get() = UpdatePolicy.parseDate(BuildConfig.BUILD_DATE)

    /** Thứ tự: market:// (ứng dụng Google Play) → trang Play trên web → trang tải về của dự án. */
    fun openStore(ctx: Context): Boolean {
        for (url in listOf(UpdatePolicy.MARKET_URL, UpdatePolicy.PLAY_WEB_URL, UpdatePolicy.DOWNLOAD_PAGE)) {
            val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            try {
                ctx.startActivity(intent)
                return true
            } catch (e: ActivityNotFoundException) {
                // thử địa chỉ kế tiếp
            } catch (e: SecurityException) {
                // thử địa chỉ kế tiếp
            }
        }
        return false
    }

    /** true nếu ứng dụng được cài từ Google Play (khi đó Play tự cập nhật). Lỗi bất kỳ → false. */
    fun installedFromPlay(ctx: Context): Boolean = try {
        val installer = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            ctx.packageManager.getInstallSourceInfo(ctx.packageName).installingPackageName
        } else {
            @Suppress("DEPRECATION")
            ctx.packageManager.getInstallerPackageName(ctx.packageName)
        }
        installer == "com.android.vending"
    } catch (e: Exception) {
        false
    }

    // Chỉ lưu một cờ giao diện: ngày hết hạn "Để sau" của thẻ nhắc cập nhật (cùng tệp với cỡ chữ/giao diện).
    private const val FILE = "hien_thi"
    private const val KEY = "nhac_cap_nhat_den"

    fun snoozedUntil(ctx: Context): LocalDate? =
        ctx.getSharedPreferences(FILE, Context.MODE_PRIVATE).getString(KEY, null)?.let(UpdatePolicy::parseDate)

    fun snooze(ctx: Context, today: LocalDate) {
        ctx.getSharedPreferences(FILE, Context.MODE_PRIVATE).edit()
            .putString(KEY, UpdatePolicy.snoozeUntil(today).toString()).apply()
    }

    fun shouldRemind(ctx: Context, today: LocalDate): Boolean =
        UpdatePolicy.shouldRemind(buildDate, today, snoozedUntil(ctx))
}
