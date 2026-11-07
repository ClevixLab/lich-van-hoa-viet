package vn.clevix.lichviet.notify

import vn.clevix.lichviet.people.JMap
import vn.clevix.lichviet.people.ReminderMsg
import vn.clevix.lichviet.people.i
import vn.clevix.lichviet.people.objs
import vn.clevix.lichviet.people.parseObj

/**
 * Phần không phụ thuộc Android của chuỗi báo thức (02 §7.3): đọc kết quả `android_alarm_step_json` của lõi,
 * đăng thông báo đến hạn, đặt **một** báo thức cho lời nhắc kế tiếp, ghi lại thời điểm chạy.
 * Phần Android (AlarmManager, NotificationManager, SharedPreferences) nằm sau giao diện [Host] để kiểm thử
 * trên JVM.
 */
object AlarmChain {

    /** Một bước của lõi: lời nhắc cần đăng ngay + báo thức kế tiếp (null = không còn lời nhắc tương lai). */
    data class Step(val post: List<ReminderMsg>, val droppedLate: Int, val nextAtUnixS: Long?)

    interface Host {
        /** Lần chạy trước (giây Unix) hoặc null. */
        fun lastRun(): Long?
        fun setLastRun(unixS: Long)
        /** Gọi lõi: `LvhvStore.android_alarm_step_json(now, last_run, zone)`. */
        fun step(nowUnixS: Long, lastRunUnixS: Long?): String
        /** Có được đăng thông báo không (quyền POST_NOTIFICATIONS, người dùng không tắt kênh). */
        fun canPost(): Boolean
        fun post(r: ReminderMsg)
        /** Đặt báo thức duy nhất của chuỗi (thay báo thức cũ). */
        fun schedule(atUnixS: Long)
        /** Hủy báo thức của chuỗi. */
        fun cancel()
    }

    fun parse(json: String): Step {
        val o: JMap = parseObj(json)
        val next = (o["next_at_unix_s"] as? Number)?.toLong()
        return Step(o.objs("post").map { ReminderMsg.of(it) }, o.i("dropped_late") ?: 0, next)
    }

    /**
     * Chạy một bước. Gọi khi báo thức chạy, khởi động lại, đổi giờ/múi giờ, cập nhật ứng dụng, mở ứng dụng,
     * lưu dữ liệu, nhập sao lưu. Trả bước đã chạy (để kiểm thử).
     */
    fun run(host: Host, nowUnixS: Long): Step {
        val last = host.lastRun()
        // Đồng hồ máy bị lùi (TIME_SET về quá khứ): coi như vừa chạy lúc này — không đăng lại các thông báo của 24 giờ
        // trước mà người dùng đã gạt (MA3N-24-05).
        val lastOk = if (last != null && last > nowUnixS) nowUnixS else last
        val step = parse(host.step(nowUnixS, lastOk))
        // Chỉ ghi nhận đã chạy khi đăng được: quyền bị tắt rồi bật lại trong 24 giờ thì lời nhắc vẫn được đăng bù
        // (02 §7.3; lõi tự giới hạn 24 giờ) — MA3N-24-06.
        if (host.canPost()) {
            step.post.forEach { host.post(it) }
            host.setLastRun(nowUnixS)
        }
        val next = step.nextAtUnixS
        if (next != null) host.schedule(next) else host.cancel()
        return step
    }

    /** Mã thông báo ổn định từ id lời nhắc ("lvhv:<uuid>:…"): đăng lại cùng lời nhắc thì thay, không nhân đôi. */
    fun notificationId(reminderId: String): Int {
        // FNV-1a 32 bit — ổn định giữa các lần chạy (không dùng String.hashCode để không phụ thuộc cài đặt JVM).
        var h = 0x811C9DC5.toInt()
        for (b in reminderId.toByteArray(Charsets.UTF_8)) {
            h = h xor (b.toInt() and 0xFF)
            h *= 0x01000193
        }
        return h and 0x7FFFFFFF
    }
}
