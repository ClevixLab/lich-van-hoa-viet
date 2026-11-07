package vn.clevix.lichviet.ui.people

import android.app.Application
import android.net.Uri
import androidx.test.core.app.ApplicationProvider
import java.io.File
import java.io.RandomAccessFile
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import vn.clevix.lichviet.people.EncodedImage
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.Photos
import vn.clevix.lichviet.people.Seed
import vn.clevix.lichviet.people.o
import vn.clevix.lichviet.people.parseObj
import vn.clevix.lichviet.people.objs

/**
 * PR #46 duyệt mã: P46-03 — đường biểu mẫu thật (`PersonFormState.build()/save()`) cho "Đổi ảnh", "Bỏ ảnh", lưu lỗi;
 * P46-01/02/06 — xử lý ảnh vừa chọn luôn dọn dẹp (cả khi bị hủy), từ chối > 25 MB trước khi đọc.
 */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [34])
class TrayFormStateTest {
    private val ctx: Application = ApplicationProvider.getApplicationContext()
    private lateinit var repo: PeopleRepo
    private lateinit var ids: Seed.Ids

    @Before
    fun setUp() {
        repo = Seed.newRepo()
        ids = Seed.family(repo)
    }

    /** JPEG có APP1 Exif chứa chuỗi GPS. */
    private fun exifJpeg(w: Int = 2000, h: Int = 1500): ByteArray {
        val bmp = android.graphics.Bitmap.createBitmap(w, h, android.graphics.Bitmap.Config.ARGB_8888)
        bmp.eraseColor(android.graphics.Color.rgb(140, 90, 50))
        val bos = java.io.ByteArrayOutputStream()
        bmp.compress(android.graphics.Bitmap.CompressFormat.JPEG, 85, bos)
        val plain = bos.toByteArray()
        val gps = "Exif\u0000\u0000II*\u0000\u0008\u0000\u0000\u0000GPS 21.0285N 105.8542E".toByteArray(Charsets.ISO_8859_1)
        val len = gps.size + 2
        return plain.copyOfRange(0, 2) + byteArrayOf(0xFF.toByte(), 0xE1.toByte(), (len shr 8).toByte(), len.toByte()) + gps +
            plain.copyOfRange(2, plain.size)
    }

    private fun tray(): EncodedImage = Photos.encode(exifJpeg(), TRAY_EDGE, TRAY_STORED)

    private fun photoCount(): Int = parseObj(repo.datasetJson()).objs("photos").size
    private fun photoFiles(): Int = File(Seed.lastDir, "photos").listFiles().orEmpty().size

    private fun form(id: String = ids.a) = PersonFormState(repo.person(id)!!, true, repo.settings())

    private fun trayId(id: String = ids.a): String? = repo.person(id)!!.memorial?.get("tray_photo_id") as String?

    /** Gắn ảnh mâm lễ qua biểu mẫu thật. */
    private fun setTrayViaForm(): String {
        val f = form()
        f.tray = tray()
        f.save(repo)
        return trayId()!!
    }

    @Test
    fun chooseThenChangeReplacesAndDeletesOld() {
        val n0 = photoCount()
        val first = setTrayViaForm()
        assertEquals(n0 + 1, photoCount())
        // "Đổi ảnh" + Lưu → khóa mới; ảnh cũ bị lõi xóa (bản ghi và tệp), không mồ côi.
        val f = form()
        f.tray = tray()
        f.save(repo)
        val second = trayId()!!
        assertNotEquals(first, second)
        assertNull("ảnh cũ bị xóa", repo.photo(first))
        assertNotNull(repo.photo(second))
        assertEquals(n0 + 1, photoCount())
        assertEquals(photoCount(), photoFiles())
    }

    @Test
    fun removeThenSaveLeavesNoKeyNoFile() {
        val n0 = photoCount()
        val pid = setTrayViaForm()
        val f = form()
        f.tray = null
        f.trayRemoved = true // "Bỏ ảnh"
        f.save(repo)
        assertNull(trayId())
        assertFalse("không còn khóa", repo.person(ids.a)!!.memorial!!.containsKey("tray_photo_id"))
        assertNull(repo.photo(pid))
        assertEquals(n0, photoCount())
        assertEquals(photoCount(), photoFiles())
    }

    @Test
    fun failedSaveDeletesJustAddedTrayPhoto() {
        val n0 = photoCount()
        val files0 = photoFiles()
        val f = form()
        f.tray = tray()
        f.hoTen = "" // hồ sơ sai → lõi từ chối khi lưu
        assertTrue(runCatching { f.save(repo) }.isFailure)
        assertEquals("ảnh mâm lễ vừa ghi bị xóa", n0, photoCount())
        assertEquals(files0, photoFiles())
        assertNull(trayId())
    }

    @Test
    fun pickThenRemoveOnPersonWithoutTray() {
        val n0 = photoCount()
        val f = form()
        f.tray = tray()
        f.tray = null
        f.trayRemoved = true
        f.save(repo)
        assertNull(trayId())
        assertEquals(n0, photoCount())
    }

    private fun capturedFile(bytes: ByteArray): File = TrayCapture.file(ctx).apply { writeBytes(bytes) }

    @Test
    fun processingCleansUpEvenWhenCancelled() = runBlocking {
        val f = form()
        val cap = capturedFile(exifJpeg(4000, 3000))
        val job = launch(Dispatchers.Default, start = CoroutineStart.UNDISPATCHED) { processTrayPick(ctx, f, Uri.fromFile(cap), cap) }
        assertTrue(f.trayBusy)
        job.cancel() // khối "Tưởng niệm" thu gọn / rời biểu mẫu
        job.join()
        assertFalse("Lưu không bị khóa", f.trayBusy)
        assertFalse("tệp chụp tạm (còn EXIF) đã xóa", cap.exists())
    }

    @Test
    fun processingSuccessAndOversizeRejectedBeforeRead() = runBlocking {
        val f = form()
        val cap = capturedFile(exifJpeg())
        assertTrue(processTrayPick(ctx, f, Uri.fromFile(cap), cap).isSuccess)
        assertNotNull(f.tray)
        assertFalse(f.trayBusy)
        assertFalse(cap.exists())
        // > 25 MB: từ chối theo độ dài tệp, không đọc (tệp thưa, không phải ảnh).
        val big = TrayCapture.file(ctx)
        RandomAccessFile(big, "rw").use { it.setLength(Photos.MAX_INPUT + 1) }
        val f2 = form()
        assertTrue(processTrayPick(ctx, f2, Uri.fromFile(big), big).isFailure)
        assertEquals("Ảnh quá lớn (tối đa 25 MB).", f2.trayError)
        assertNull(f2.tray)
        assertFalse(big.exists())
    }
}
