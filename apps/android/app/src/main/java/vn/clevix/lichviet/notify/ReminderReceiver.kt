package vn.clevix.lichviet.notify

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/**
 * Báo thức của chuỗi nhắc + các sự kiện hệ thống buộc đặt lại chuỗi (02 §7.3): BOOT_COMPLETED (mọi báo thức bị
 * hủy khi tắt máy), MY_PACKAGE_REPLACED, TIME_SET, TIMEZONE_CHANGED.
 */
class ReminderReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        when (intent.action) {
            Reminders.ACTION_ALARM,
            Intent.ACTION_BOOT_COMPLETED,
            Intent.ACTION_MY_PACKAGE_REPLACED,
            Intent.ACTION_TIME_CHANGED,
            Intent.ACTION_TIMEZONE_CHANGED -> {
                val pending: PendingResult? = goAsync()
                Reminders.rearmAsync(context) { pending?.finish() }
            }
        }
    }
}
