package vn.clevix.lichviet.people

import android.graphics.Bitmap
import android.graphics.Color
import java.io.ByteArrayOutputStream
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/**
 * S9 (MA3N-24-07 mục 4): ảnh có EXIF (APP1, hướng xoay 90°, GPS) và IPTC (APP13) → byte lưu không còn APP1/APP13,
 * ảnh đã xoay đúng, cạnh dài ≤ 1024, ≤ 1 MB.
 */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [34])
class PhotosTest {

    /** Các đoạn (marker) của JPEG trước SOS. */
    private fun markers(jpeg: ByteArray): List<Int> {
        val out = ArrayList<Int>()
        var i = 2
        while (i + 4 <= jpeg.size && jpeg[i] == 0xFF.toByte()) {
            val m = jpeg[i + 1].toInt() and 0xFF
            out.add(m)
            if (m == 0xDA) break
            val len = ((jpeg[i + 2].toInt() and 0xFF) shl 8) or (jpeg[i + 3].toInt() and 0xFF)
            i += 2 + len
        }
        return out
    }

    private fun seg(marker: Int, payload: ByteArray): ByteArray {
        val len = payload.size + 2
        return byteArrayOf(0xFF.toByte(), marker.toByte(), (len shr 8).toByte(), len.toByte()) + payload
    }

    /** APP1 EXIF (little endian): IFD0 có Orientation = 6 (xoay 90°) và con trỏ GPS IFD; GPS IFD có GPSLatitudeRef "N". */
    private fun exifApp1(): ByteArray {
        val tiff = java.nio.ByteBuffer.allocate(64).order(java.nio.ByteOrder.LITTLE_ENDIAN)
        tiff.put('I'.code.toByte()).put('I'.code.toByte()).putShort(42).putInt(8)
        tiff.putShort(2) // IFD0: 2 mục
        tiff.putShort(0x0112).putShort(3).putInt(1).putShort(6).putShort(0) // Orientation = 6
        tiff.putShort(0x8825.toShort()).putShort(4).putInt(1).putInt(38) // GPS IFD tại 38
        tiff.putInt(0)
        tiff.putShort(1) // GPS IFD: 1 mục (vị trí 38)
        tiff.putShort(1).putShort(2).putInt(2).put('N'.code.toByte()).put(0).putShort(0)
        tiff.putInt(0)
        val t = tiff.array().copyOf(tiff.position())
        return seg(0xE1, "Exif\u0000\u0000".toByteArray(Charsets.ISO_8859_1) + t)
    }

    private fun iptcApp13(): ByteArray = seg(0xED, "Photoshop 3.0\u0000IPTC vi tri".toByteArray(Charsets.ISO_8859_1))

    private fun sampleJpeg(w: Int, h: Int): ByteArray {
        val bmp = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
        bmp.eraseColor(Color.rgb(200, 120, 40))
        val bos = ByteArrayOutputStream()
        bmp.compress(Bitmap.CompressFormat.JPEG, 90, bos)
        val plain = bos.toByteArray()
        // Chèn APP1 + APP13 ngay sau SOI.
        return plain.copyOfRange(0, 2) + exifApp1() + iptcApp13() + plain.copyOfRange(2, plain.size)
    }

    @Test
    fun reencodeStripsMetadataAndAppliesOrientation() {
        val src = sampleJpeg(400, 200)
        assertTrue("ảnh mẫu phải có APP1", 0xE1 in markers(src))
        assertTrue("ảnh mẫu phải có APP13", 0xED in markers(src))
        val out = Photos.encode(src)
        val m = markers(out.bytes)
        assertFalse("còn APP1 (EXIF/XMP)", 0xE1 in m)
        assertFalse("còn APP13 (IPTC)", 0xED in m)
        assertFalse("còn chữ Exif", String(out.bytes, Charsets.ISO_8859_1).contains("Exif"))
        assertFalse("còn Photoshop/IPTC", String(out.bytes, Charsets.ISO_8859_1).contains("Photoshop"))
        // Orientation = 6 → xoay 90°: 400 × 200 → 200 × 400.
        assertEquals(200, out.width)
        assertEquals(400, out.height)
        assertTrue(out.bytes.size <= Photos.MAX_STORED)
    }

    @Test
    fun largeImageIsDownscaledToMaxEdge() {
        val out = Photos.encode(sampleJpeg(3000, 1500))
        assertEquals(Photos.MAX_EDGE, maxOf(out.width, out.height))
    }

    @Test(expected = Photos.PhotoError::class)
    fun garbageIsRejected() {
        Photos.encode(ByteArray(100) { it.toByte() })
    }
}
