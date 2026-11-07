package vn.clevix.lichviet.notify

import android.Manifest
import android.app.AlarmManager
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat
import java.io.File
import java.util.concurrent.Executors
import vn.clevix.lichviet.MainActivity
import vn.clevix.lichviet.R
import vn.clevix.lichviet.core.LvhvStore
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.ReminderMsg
import vn.clevix.lichviet.people.Zones

/** Kho người thân dùng chung trong tiến trình (một kết nối SQLite). */
object PeopleStore {
    @Volatile private var repo: PeopleRepo? = null
    @Volatile private var error: String? = null

    /** Thư mục dữ liệu: files/ (lvhv.sqlite + photos/, 01 §3.8, §4.2). Không vào sao lưu đám mây (S5). */
    fun dir(ctx: Context): File = ctx.applicationContext.filesDir

    /** Kho đã mở, hoặc null kèm [openError] (ví dụ dữ liệu của bản mới hơn). */
    fun get(ctx: Context): PeopleRepo? {
        repo?.let { return it }
        synchronized(this) {
            repo?.let { return it }
            return try {
                PeopleRepo(LvhvStore.open(dir(ctx).absolutePath)).also { repo = it; error = null }
            } catch (e: Exception) {
                error = vn.clevix.lichviet.people.errText(e)
                null
            }
        }
    }

    fun openError(): String? = error

    /** Chỉ cho kiểm thử: bỏ kho đã mở (mỗi kiểm thử Robolectric có thư mục dữ liệu riêng). */
    @androidx.annotation.VisibleForTesting
    fun reset() = synchronized(this) { repo = null; error = null }

    /** Đã từng có sổ người thân trên máy (để không tạo kho khi người dùng chưa mở mô-đun). */
    fun exists(ctx: Context): Boolean = File(dir(ctx), "lvhv.sqlite").exists()
}

/** Chuỗi báo thức Android (02 §7.3, O-6: báo thức không chính xác, không có SCHEDULE_EXACT_ALARM). */
object Reminders {
    const val ACTION_ALARM = "vn.clevix.lichviet.action.NHAC"
    const val EXTRA_OPEN = "vn.clevix.lichviet.extra.MO"
    const val CHANNEL = "nhac-ngay-gio"
    private const val PREFS = "nhac_viec"
    private const val KEY_LAST = "last_run_unix_s"
    private const val KEY_ASKED = "asked_post_notifications"

    private val worker = Executors.newSingleThreadExecutor { r -> Thread(r, "lvhv-nhac").apply { isDaemon = true } }

    /** Đặt lại chuỗi trên luồng nền (mở ứng dụng, lưu, nhập sao lưu). */
    fun rearmAsync(ctx: Context, done: (() -> Unit)? = null) {
        val app = ctx.applicationContext
        worker.execute {
            rearm(app)
            done?.invoke()
        }
    }

    /** Chỉ cho kiểm thử: chờ mọi bước đang xếp hàng chạy xong. */
    @androidx.annotation.VisibleForTesting
    fun awaitIdle() { worker.submit {}.get(30, java.util.concurrent.TimeUnit.SECONDS) }

    /** Thời gian chờ trước khi thử lại khi một bước lỗi (MA3N-24-04). */
    const val RETRY_S = 6 * 3600L
    private const val KEY_ERR = "last_error_at_unix_s"

    /**
     * Một bước của chuỗi (đồng bộ). Không có sổ người thân → không làm gì. Bước lỗi (kho không mở được, dữ liệu, múi giờ…)
     * **không** làm chuỗi dừng: ghi thời điểm lỗi (không ghi nội dung — có thể chứa dữ liệu người dùng), đặt báo thức thử
     * lại sau [RETRY_S] giây. Trả true nếu bước thành công.
     */
    fun rearm(ctx: Context, nowUnixS: Long = System.currentTimeMillis() / 1000): Boolean {
        if (!PeopleStore.exists(ctx)) return true
        return try {
            val repo = PeopleStore.get(ctx) ?: throw IllegalStateException("kho")
            AlarmChain.run(AndroidHost(ctx, repo), nowUnixS)
            prefs(ctx).edit().remove(KEY_ERR).apply()
            true
        } catch (e: Exception) {
            android.util.Log.w("LvhvNhac", "Đặt lời nhắc lỗi (${e.javaClass.simpleName}); thử lại sau ${RETRY_S / 3600} giờ")
            prefs(ctx).edit().putLong(KEY_ERR, nowUnixS).apply()
            scheduleAlarm(ctx, nowUnixS + RETRY_S)
            false
        }
    }

    /** Thời điểm lần đặt lời nhắc gần nhất bị lỗi (null = không lỗi). */
    fun lastError(ctx: Context): Long? = prefs(ctx).getLong(KEY_ERR, -1L).takeIf { it >= 0 }

    /** Đặt báo thức duy nhất của chuỗi. O-6: không chính xác, không bao giờ sớm hơn t; Android 12+ giao trong khoảng 1 giờ. */
    fun scheduleAlarm(ctx: Context, atUnixS: Long) {
        val am = ctx.getSystemService(AlarmManager::class.java) ?: return
        am.setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, atUnixS * 1000, alarmIntent(ctx))
    }

    /** Tiêu đề không tên dùng cho bản hiện trên màn hình khóa khi người dùng bật "hiện tên" (MA3N-24-06). */
    fun genericTitle(cls: String): String = when (cls) {
        "gio", "tien_thuong" -> "Sắp đến ngày giỗ"
        "sinh_nhat", "sinh_nhat_am" -> "Sắp đến sinh nhật"
        else -> "Sắp đến sự kiện" // cùng chữ với lõi (O-9)
    }

    fun ensureChannel(ctx: Context) {
        val nm = ctx.getSystemService(NotificationManager::class.java) ?: return
        if (nm.getNotificationChannel(CHANNEL) == null) {
            val ch = NotificationChannel(CHANNEL, "Nhắc ngày giỗ, sinh nhật", NotificationManager.IMPORTANCE_DEFAULT).apply {
                description = "Lời nhắc ngày giỗ, sinh nhật và sự kiện của gia đình"
                // O-9: nội dung đã không có tên người (trừ khi người dùng bật); màn hình khóa vẫn hiện nội dung đó.
                lockscreenVisibility = android.app.Notification.VISIBILITY_PUBLIC
            }
            nm.createNotificationChannel(ch)
        }
    }

    fun notificationsAllowed(ctx: Context): Boolean {
        if (Build.VERSION.SDK_INT >= 33 &&
            ContextCompat.checkSelfPermission(ctx, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED
        ) return false
        return NotificationManagerCompat.from(ctx).areNotificationsEnabled()
    }

    /** Cần hỏi quyền thông báo (Android 13+, chưa cấp). */
    fun needsPermission(ctx: Context): Boolean =
        Build.VERSION.SDK_INT >= 33 &&
            ContextCompat.checkSelfPermission(ctx, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED

    fun askedBefore(ctx: Context): Boolean = prefs(ctx).getBoolean(KEY_ASKED, false)
    fun markAsked(ctx: Context) = prefs(ctx).edit().putBoolean(KEY_ASKED, true).apply()

    private fun prefs(ctx: Context) = ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    private fun alarmIntent(ctx: Context): PendingIntent = PendingIntent.getBroadcast(
        ctx, 0,
        Intent(ctx, ReminderReceiver::class.java).setAction(ACTION_ALARM),
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
    )

    private class AndroidHost(val ctx: Context, val repo: PeopleRepo) : AlarmChain.Host {
        override fun lastRun(): Long? = prefs(ctx).getLong(KEY_LAST, -1L).takeIf { it >= 0 }
        override fun setLastRun(unixS: Long) { prefs(ctx).edit().putLong(KEY_LAST, unixS).apply() }
        override fun step(nowUnixS: Long, lastRunUnixS: Long?): String =
            repo.store.androidAlarmStepJson(nowUnixS, lastRunUnixS, Zones.device())

        override fun canPost(): Boolean = notificationsAllowed(ctx)

        // Đã kiểm quyền ở notificationsAllowed() ngay trên; lint không thấy được qua lời gọi hàm.
        @android.annotation.SuppressLint("MissingPermission")
        override fun post(r: ReminderMsg) {
            if (!canPost()) return
            ensureChannel(ctx)
            val open = Intent(ctx, MainActivity::class.java)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)
                .putExtra(EXTRA_OPEN, r.entityId)
            val pi = PendingIntent.getActivity(
                ctx, AlarmChain.notificationId(r.id), open,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
            )
            val names = runCatching { repo.lockScreenNames() }.getOrDefault(false)
            val b = NotificationCompat.Builder(ctx, CHANNEL)
                .setSmallIcon(R.drawable.ic_stat_nhac)
                .setContentTitle(r.title)
                .setContentText(r.body)
                .setStyle(NotificationCompat.BigTextStyle().bigText(r.body))
                .setCategory(NotificationCompat.CATEGORY_REMINDER)
                .setAutoCancel(true)
                .setContentIntent(pi)
            if (names) {
                // Có tên: PRIVATE + bản công khai không tên, để cài đặt "Ẩn nội dung nhạy cảm" của máy vẫn có hiệu lực.
                val pub = NotificationCompat.Builder(ctx, CHANNEL)
                    .setSmallIcon(R.drawable.ic_stat_nhac)
                    .setContentTitle(genericTitle(r.cls))
                    .setCategory(NotificationCompat.CATEGORY_REMINDER)
                    .build()
                b.setVisibility(NotificationCompat.VISIBILITY_PRIVATE).setPublicVersion(pub)
            } else {
                // Mặc định O-9: nội dung đã không có tên.
                b.setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
            }
            val n = b.build()
            try {
                NotificationManagerCompat.from(ctx).notify(AlarmChain.notificationId(r.id), n)
            } catch (_: SecurityException) {
                // Quyền bị thu hồi giữa chừng: bỏ qua, "Sắp tới" vẫn hiển thị.
            }
        }

        override fun schedule(atUnixS: Long) = scheduleAlarm(ctx, atUnixS)

        override fun cancel() {
            ctx.getSystemService(AlarmManager::class.java)?.cancel(alarmIntent(ctx))
        }
    }
}
