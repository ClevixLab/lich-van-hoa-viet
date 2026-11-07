package vn.clevix.lichviet.people

import android.content.Context
import java.security.MessageDigest

/** Một ảnh mâm lễ Clevix đã duyệt (mục `approved` do scripts/check-tray-pack.py sinh lúc dựng). */
class TrayItem(val meta: JMap, val bytes: ByteArray) {
    val style: String get() = meta.s("tray_style")
    /** Dòng ghi công "Ảnh: <tác giả> · <giấy phép>" khi ảnh cần ghi công (08 §16.1), ngược lại null. */
    val credit: String? get() = if (meta["ghi_cong"] == null) null else "Ảnh: ${meta.sn("tac_gia") ?: ""} · ${meta.sn("giay_phep") ?: ""}"
}

/**
 * Gói ảnh mâm lễ Clevix nhúng trong ứng dụng (YC-2026-10, ADR-V02-01, 08 §16.3): `assets/trays/approved.json` + các tệp
 * ảnh. Không tải gì qua mạng. Bước dựng đã kiểm sha256, siêu dữ liệu, giới hạn; ở đây kiểm lại sha256 khi đọc (tệp hỏng →
 * bỏ qua). Gói rỗng → [forStyle] luôn null → không hiện mâm (R15 (a)).
 */
object TrayPack {
    /** Chỉ cho kiểm thử (08 §16.4 ca (5)): thay gói nhúng bằng danh sách cho sẵn. */
    @Volatile var override: List<TrayItem>? = null

    @Volatile private var cache: List<TrayItem>? = null

    fun items(ctx: Context): List<TrayItem> {
        override?.let { return it }
        cache?.let { return it }
        val list = runCatching {
            val doc = parseObj(ctx.assets.open("trays/approved.json").use { it.readBytes().toString(Charsets.UTF_8) })
            doc.objs("approved").mapNotNull { m ->
                val b = runCatching { ctx.assets.open("trays/" + m.s("file")).use { it.readBytes() } }.getOrNull() ?: return@mapNotNull null
                if (sha256(b) != m.s("sha256")) null else TrayItem(m, b)
            }
        }.getOrDefault(emptyList())
        cache = list
        return list
    }

    /** Ảnh Clevix đầu tiên cùng [style], hoặc null. */
    fun forStyle(ctx: Context, style: String?): TrayItem? = items(ctx).firstOrNull { it.style == (style ?: "truyen_thong") }

    fun sha256(b: ByteArray): String = MessageDigest.getInstance("SHA-256").digest(b).joinToString("") { "%02x".format(it) }
}
