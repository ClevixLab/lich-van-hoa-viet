package vn.clevix.lichviet.people

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.ImageDecoder
import android.graphics.Matrix
import android.media.ExifInterface
import android.net.Uri
import android.os.Build
import android.provider.OpenableColumns
import java.io.ByteArrayOutputStream
import kotlin.math.max
import kotlin.math.roundToInt

/** Ảnh đã giải mã và mã hóa lại JPEG: byte không còn EXIF/GPS/XMP/IPTC (01 §3.8, S9). */
class EncodedImage(val bytes: ByteArray, val width: Int, val height: Int)

/**
 * Xử lý ảnh chọn qua Photo Picker (không cần quyền đọc bộ nhớ): giải mã (xoay theo EXIF), thu cạnh dài ≤ 1024 px
 * (không phóng to), vẽ lại và mã hóa JPEG. `Bitmap.compress` chỉ ghi điểm ảnh — không chép siêu dữ liệu nào của tệp
 * gốc, nên byte lưu không có đoạn APP1 (EXIF/XMP) hay APP13 (IPTC).
 */
object Photos {
    const val MAX_INPUT = 25L * 1024 * 1024
    const val MAX_STORED = 1024 * 1024
    const val MAX_EDGE = 1024

    class PhotoError(msg: String) : Exception(msg)

    fun process(ctx: Context, uri: Uri): EncodedImage {
        val cr = ctx.contentResolver
        val size = cr.query(uri, arrayOf(OpenableColumns.SIZE), null, null, null)?.use { c ->
            if (c.moveToFirst() && !c.isNull(0)) c.getLong(0) else -1L
        } ?: -1L
        if (size > MAX_INPUT) throw PhotoError("Ảnh lớn hơn 25 MB — hãy chọn ảnh nhỏ hơn")
        val raw = cr.openInputStream(uri)?.use { s ->
            val buf = ByteArrayOutputStream()
            val tmp = ByteArray(64 * 1024)
            var total = 0L
            while (true) {
                val n = s.read(tmp)
                if (n < 0) break
                total += n
                if (total > MAX_INPUT) throw PhotoError("Ảnh lớn hơn 25 MB — hãy chọn ảnh nhỏ hơn")
                buf.write(tmp, 0, n)
            }
            buf.toByteArray()
        } ?: throw PhotoError("Không đọc được ảnh này (định dạng không hỗ trợ)")
        return encode(raw)
    }

    /** Giải mã + xoay + thu nhỏ + mã hóa lại. */
    fun encode(raw: ByteArray): EncodedImage {
        // Trần điểm ảnh trước khi giải mã (T-D3): đọc kích thước từ phần đầu tệp.
        val b = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        runCatching { BitmapFactory.decodeByteArray(raw, 0, raw.size, b) }
        if (b.outWidth > MAX_DECLARED_EDGE || b.outHeight > MAX_DECLARED_EDGE ||
            b.outWidth.toLong() * b.outHeight > MAX_DECLARED_PIXELS
        ) throw PhotoError("Ảnh quá lớn (${b.outWidth} × ${b.outHeight} điểm ảnh) — hãy chọn ảnh nhỏ hơn")
        val bmp = decode(raw) ?: throw PhotoError("Không đọc được ảnh này (định dạng không hỗ trợ)")
        val scale = MAX_EDGE.toFloat() / max(bmp.width, bmp.height)
        var w = bmp.width
        var h = bmp.height
        var src = bmp
        if (scale < 1f) {
            w = (bmp.width * scale).roundToInt().coerceAtLeast(1)
            h = (bmp.height * scale).roundToInt().coerceAtLeast(1)
            src = Bitmap.createScaledBitmap(bmp, w, h, true)
        }
        // Vẽ lại lên nền trắng (ảnh PNG có kênh trong suốt) rồi mã hóa JPEG.
        val out = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
        android.graphics.Canvas(out).apply {
            drawColor(android.graphics.Color.WHITE)
            drawBitmap(src, 0f, 0f, null)
        }
        for (q in intArrayOf(88, 80, 70, 60, 50)) {
            val bos = ByteArrayOutputStream()
            out.compress(Bitmap.CompressFormat.JPEG, q, bos)
            if (bos.size() <= MAX_STORED) return EncodedImage(bos.toByteArray(), w, h)
        }
        throw PhotoError("Ảnh quá chi tiết, chưa nén được dưới 1 MB")
    }

    private fun decode(raw: ByteArray): Bitmap? = try {
        if (Build.VERSION.SDK_INT >= 28) {
            // ImageDecoder tự xoay theo EXIF; giới hạn kích thước khi giải mã để không tốn bộ nhớ.
            val src = ImageDecoder.createSource(java.nio.ByteBuffer.wrap(raw))
            ImageDecoder.decodeBitmap(src) { dec, info, _ ->
                val s = info.size
                val f = MAX_EDGE.toFloat() / max(s.width, s.height)
                if (f < 1f) dec.setTargetSize((s.width * f).roundToInt().coerceAtLeast(1), (s.height * f).roundToInt().coerceAtLeast(1))
                dec.allocator = ImageDecoder.ALLOCATOR_SOFTWARE
            }
        } else {
            val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
            BitmapFactory.decodeByteArray(raw, 0, raw.size, bounds)
            var sample = 1
            while (max(bounds.outWidth, bounds.outHeight) / (sample * 2) >= MAX_EDGE) sample *= 2
            val bmp = BitmapFactory.decodeByteArray(raw, 0, raw.size, BitmapFactory.Options().apply { inSampleSize = sample })
            bmp?.let { rotate(it, orientation(raw)) }
        }
    } catch (_: Exception) {
        null
    }

    private fun orientation(raw: ByteArray): Int = try {
        ExifInterface(raw.inputStream()).getAttributeInt(ExifInterface.TAG_ORIENTATION, ExifInterface.ORIENTATION_NORMAL)
    } catch (_: Exception) {
        ExifInterface.ORIENTATION_NORMAL
    }

    private fun rotate(b: Bitmap, o: Int): Bitmap {
        val m = Matrix()
        when (o) {
            ExifInterface.ORIENTATION_ROTATE_90 -> m.postRotate(90f)
            ExifInterface.ORIENTATION_ROTATE_180 -> m.postRotate(180f)
            ExifInterface.ORIENTATION_ROTATE_270 -> m.postRotate(270f)
            ExifInterface.ORIENTATION_FLIP_HORIZONTAL -> m.postScale(-1f, 1f)
            ExifInterface.ORIENTATION_FLIP_VERTICAL -> m.postScale(1f, -1f)
            ExifInterface.ORIENTATION_TRANSPOSE -> { m.postRotate(90f); m.postScale(-1f, 1f) }
            ExifInterface.ORIENTATION_TRANSVERSE -> { m.postRotate(270f); m.postScale(-1f, 1f) }
            else -> return b
        }
        return Bitmap.createBitmap(b, 0, 0, b.width, b.height, m, true)
    }

    /** Cạnh dài tối đa của ảnh giải mã để hiển thị (ảnh lưu ≤ 1024 px, tệp đính kèm ≤ 2048 px — 01 §3.8, §3.8a). */
    const val DISPLAY_EDGE = 2048
    /** Trần kích thước khai báo trong phần đầu tệp: vượt → không giải mã (bom giải nén, BM2-10, B6 mục 3). */
    const val MAX_DECLARED_EDGE = 12_000
    const val MAX_DECLARED_PIXELS = 50_000_000L

    /**
     * Giải mã byte đã lưu để hiển thị, có giới hạn bộ nhớ (BM2-10): đọc kích thước từ phần đầu tệp trước
     * (`inJustDecodeBounds`), từ chối kích thước bất thường, lấy mẫu xuống (`inSampleSize`) để cạnh dài ≤ [maxEdge]. Mọi
     * đường hiển thị ảnh (chân dung, ảnh giấy tờ, màn hình tưởng niệm, xem trước biểu mẫu) đều qua hàm này. Lỗi → null
     * (giao diện hiện khung trống), không bao giờ sập.
     */
    fun bitmap(bytes: ByteArray, maxEdge: Int = DISPLAY_EDGE): Bitmap? = runCatching {
        val b = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        BitmapFactory.decodeByteArray(bytes, 0, bytes.size, b)
        val w = b.outWidth
        val h = b.outHeight
        if (w <= 0 || h <= 0 || w > MAX_DECLARED_EDGE || h > MAX_DECLARED_EDGE || w.toLong() * h > MAX_DECLARED_PIXELS) {
            return@runCatching null
        }
        BitmapFactory.decodeByteArray(bytes, 0, bytes.size, BitmapFactory.Options().apply { inSampleSize = sampleFor(w, h, maxEdge) })
    }.getOrNull()

    /** Hệ số lấy mẫu (lũy thừa của 2) để cạnh dài ≤ [maxEdge]. */
    fun sampleFor(w: Int, h: Int, maxEdge: Int): Int {
        var s = 1
        while (max(w, h) / s > maxEdge) s *= 2
        return s
    }
}
