package vn.clevix.lichviet.update

import java.time.LocalDate
import java.time.temporal.ChronoUnit

/**
 * Nhắc cập nhật KHÔNG dùng mạng: ứng dụng không biết bản mới nhất là bản nào, chỉ biết ngày dựng của
 * chính nó (ghi vào BuildConfig lúc dựng). Bản đã dựng hơn [REMIND_AFTER_DAYS] ngày thì hiện thẻ nhắc
 * nhẹ; bấm "Để sau" thì ẩn [SNOOZE_DAYS] ngày. Mã thuần Kotlin để kiểm thử trên JVM.
 */
object UpdatePolicy {
    const val REMIND_AFTER_DAYS = 60L
    const val SNOOZE_DAYS = 30L

    const val PACKAGE = "vn.clevix.lichviet"
    const val MARKET_URL = "market://details?id=$PACKAGE"
    const val PLAY_WEB_URL = "https://play.google.com/store/apps/details?id=$PACKAGE"
    const val DOWNLOAD_PAGE = "https://clevixlab.github.io/lich-van-hoa-viet/"

    const val BUTTON = "Kiểm tra cập nhật / Cập nhật"
    const val EXPLAIN = "Ứng dụng không tự kết nối mạng để bảo vệ quyền riêng tư. Bấm nút để mở cửa hàng ứng dụng hoặc trang tải về và cập nhật lên phiên bản mới nhất."
    const val REMINDER = "Phiên bản này đã phát hành hơn 2 tháng. Hãy kiểm tra cập nhật để có dữ liệu và tính năng mới nhất."

    /** Dòng bản quyền (dòng 1 chân trang Giới thiệu); ký hiệu © được vẽ nhỏ và nhẹ hơn chữ. */
    const val COPYRIGHT_SIGN = "©"
    const val COPYRIGHT_HOLDER = "\u00a02026 Clevix Lab"
    const val COPYRIGHT = COPYRIGHT_SIGN + COPYRIGHT_HOLDER

    /** Dòng 2 chân trang Giới thiệu: số phiên bản, chữ nhỏ và nhạt. */
    fun versionLine(version: String): String = "Phiên bản $version"

    fun runningVersion(version: String, build: String): String = "Phiên bản đang dùng: $version (bản dựng $build)"

    /** Ngày dựng dạng YYYY-MM-DD; sai định dạng → null (khi đó không nhắc). */
    fun parseDate(s: String): LocalDate? = runCatching { LocalDate.parse(s) }.getOrNull()

    fun shouldRemind(buildDate: LocalDate?, today: LocalDate, snoozedUntil: LocalDate?): Boolean {
        if (buildDate == null) return false
        if (ChronoUnit.DAYS.between(buildDate, today) <= REMIND_AFTER_DAYS) return false
        return snoozedUntil == null || !today.isBefore(snoozedUntil)
    }

    fun snoozeUntil(today: LocalDate): LocalDate = today.plusDays(SNOOZE_DAYS)
}
