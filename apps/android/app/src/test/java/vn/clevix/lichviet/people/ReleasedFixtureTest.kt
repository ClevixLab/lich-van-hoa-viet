package vn.clevix.lichviet.people

import java.io.File
import java.nio.file.Files
import java.security.MessageDigest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import vn.clevix.lichviet.core.LvhvStore
import vn.clevix.lichviet.people.MigrationTest.Companion.sorted

/**
 * R14 / 08-QUY-TRINH §10 qua lớp Android: bộ mẫu đóng băng `released/0.2.1/android` (bản Android đầu tiên có mô-đun Người
 * thân) — từng tệp đúng sha256 của `manifest.json`; bản sao kho mở bằng bản này → `ok`, dữ liệu đúng tệp kỳ vọng chung;
 * tệp sao lưu của bộ mẫu nhập vào kho trống cho cùng dữ liệu. (Trên trình giả lập: `ci/upgrade-test.sh`, op
 * `verify-released`.)
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class ReleasedFixtureTest {
    private val fix = File("../../../docs/v0.2/schema/test-vectors/released/0.2.1/android")
    private val expected = File("../../../docs/v0.2/schema/test-vectors/released/expected/0.2.1-android.json")

    private fun sha(b: ByteArray) = MessageDigest.getInstance("SHA-256").digest(b).joinToString("") { "%02x".format(it) }

    @Test
    fun fixtureIsFrozen() {
        assumeTrue("thiếu bộ mẫu ${fix.absolutePath}", fix.isDirectory)
        val files = parseObj(File(fix, "manifest.json").readText()).o("files")!!
        assertTrue(files.size >= 10)
        for ((f, want) in files) assertEquals("$f đã bị sửa — bộ mẫu đóng băng không được sửa", want, sha(File(fix, f).readBytes()))
    }

    @Test
    fun releasedStoreOpensUnchanged() {
        assumeTrue("thiếu bộ mẫu ${fix.absolutePath}", fix.isDirectory)
        val dir = Files.createTempDirectory("lvhv-r14-021").toFile()
        File(fix, "store").copyRecursively(dir, overwrite = true)
        val repo = PeopleRepo(LvhvStore.open(dir.absolutePath))
        assertEquals("ok", repo.migration().state)
        assertFalse(repo.readOnly)
        val exp = parseObj(expected.readText())
        val want = exp.o("state")!!
        val got = parseObj(repo.datasetJson()).also { it.remove("app"); it.remove("exported_at") }
        assertEquals(JsonW.write(sorted(want)), JsonW.write(sorted(got)))
        assertEquals(exp["lock_screen_names"], repo.lockScreenNames())
        for ((id, s) in exp.o("photo_sha256")!!) assertEquals(s, sha(repo.photo(id)!!))
        for ((id, s) in exp.o("attachment_sha256")!!) assertEquals(s, sha(repo.attachmentBytes(id)!!))
        val a = repo.persons().first { it.id.endsWith("00000000000a") }
        assertEquals(2, a.notice!!.objs("attachments").size)

        val (v, pv) = PeopleRepo.openBackupFile(File(fix, "backup.lvhv").readBytes(), "mật khẩu thử nghiệm")
        assertEquals(3, pv.persons.toInt())
        val fresh = PeopleRepo(LvhvStore.open(Files.createTempDirectory("lvhv-r14-021-import").toFile().absolutePath))
        fresh.applyImport(v, replace = true)
        v.destroy()
        val imp = parseObj(fresh.datasetJson())
        for (k in listOf("persons", "events", "event_types", "photos", "attachments", "settings")) {
            assertEquals(k, JsonW.write(sorted(want[k])), JsonW.write(sorted(imp[k])))
        }
    }
}
