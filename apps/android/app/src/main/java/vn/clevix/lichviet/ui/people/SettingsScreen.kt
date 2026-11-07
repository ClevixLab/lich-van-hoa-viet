@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package vn.clevix.lichviet.ui.people

import android.content.Intent
import android.provider.Settings
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import vn.clevix.lichviet.notify.Reminders
import vn.clevix.lichviet.people.b
import vn.clevix.lichviet.people.errText

/** Cài đặt nhắc (Android): công tắc chung, hiện tên trên màn hình khóa (O-9), quyền thông báo. */
@Composable
fun SettingsScreen(x: PeopleCtx) {
    val ctx = LocalContext.current
    val settings = remember(x.rev) { runCatching { x.repo.settings() }.getOrNull() }
    val names = remember(x.rev) { runCatching { x.repo.lockScreenNames() }.getOrDefault(false) }
    val allowed = remember(x.rev) { Reminders.notificationsAllowed(ctx) }
    BackLink("Về Người thân") { x.nav.back() }
    H1("Cài đặt nhắc")

    PCard(title = "Thông báo") {
        if (allowed) {
            Body("Thông báo đang bật. Điện thoại sẽ nhắc ngày giỗ, sinh nhật và sự kiện theo giờ bạn đặt, kể cả khi không mở ứng dụng.")
        } else {
            Body("Thông báo của ứng dụng đang tắt — điện thoại sẽ không tự nhắc. Lời nhắc vẫn hiện ở mục Sắp tới khi bạn mở ứng dụng.")
            PrimaryButton("Cho phép thông báo", {
                if (Reminders.needsPermission(ctx) && !Reminders.askedBefore(ctx)) {
                    x.askNotify()
                } else {
                    // Đã từ chối trước đó (hệ thống không hỏi lại) hoặc người dùng tắt trong cài đặt: mở trang cài đặt thông báo.
                    val i = Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS)
                        .putExtra(Settings.EXTRA_APP_PACKAGE, ctx.packageName)
                        .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    runCatching { ctx.startActivity(i) }
                }
            }, Modifier.fillMaxWidth())
        }
        Fine(
            "Thông báo dùng báo thức tiết kiệm pin của Android: có thể đến muộn tới khoảng một giờ so với giờ đã đặt, " +
                "không bao giờ đến sớm. Sau khi khởi động lại máy, lời nhắc được đặt lại tự động.",
        )
    }

    PCard(title = "Lời nhắc") {
        SwitchRow(
            "Nhắc ngày giỗ, sinh nhật và sự kiện", settings?.b("reminders_enabled") ?: true,
            { on ->
                settings?.let { s ->
                    s["reminders_enabled"] = on
                    runCatching { x.repo.putSettings(s) }.onSuccess { x.changed() }.onFailure { x.toast(errText(it)) }
                    if (on) x.askNotify()
                }
            },
            sub = "Công tắc chung cho mọi hồ sơ trên máy này.",
        )
        SwitchRow(
            "Hiện tên người trên màn hình khóa", names,
            { on -> runCatching { x.repo.setLockScreenNames(on) }.onSuccess { x.changed() }.onFailure { x.toast(errText(it)) } },
            sub = if (names) "Thông báo ghi rõ tên, ví dụ \"Còn 3 ngày: Giỗ Ông Nguyễn Văn A\". Ai nhìn màn hình khóa cũng đọc được."
            else "Mặc định: thông báo chỉ ghi \"Sắp đến ngày giỗ\" / \"Sắp đến sinh nhật\", không có tên. Mở ứng dụng để xem là ai.",
        )
        Fine("Tùy chọn này chỉ lưu trên máy này, không có trong tệp sao lưu. Thông tin báo tử không bao giờ có trong thông báo.")
    }
}
