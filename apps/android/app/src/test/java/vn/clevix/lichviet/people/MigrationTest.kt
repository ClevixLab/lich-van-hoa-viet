package vn.clevix.lichviet.people

import java.io.File
import java.nio.file.Files
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import vn.clevix.lichviet.core.LvhvStore

/**
 * B5 / R14 (c) qua lớp Android: kho SQLite thật của bản 0.2-dev (bộ mẫu đóng băng, phiên bản 1) mở bằng bản này →
 * `migrated`, tệp đính kèm thay `certificate_photo_ids`; kho có hồ sơ hỏng → `read_only`, ghi bị từ chối, vẫn xuất
 * sao lưu được. (Trên trình giả lập: job `upgrade`, `ci/upgrade-test.sh`.)
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class MigrationTest {
    companion object {
        /** Thư mục bộ mẫu, tính từ thư mục mô-đun `apps/android/app` (thư mục làm việc của kiểm thử Gradle). */
        val FIX = File("../../../docs/v0.2/schema/test-vectors/released/0.2-dev-before-attachments/store")

        fun copyFixture(): File {
            assumeTrue("thiếu bộ mẫu ${FIX.absolutePath}", FIX.isDirectory)
            val d = Files.createTempDirectory("lvhv-r14").toFile()
            FIX.copyRecursively(d, overwrite = true)
            return d
        }

        /**
         * Làm hỏng một hồ sơ trong kho phiên bản 1 như store/tests/released.rs: chuỗi chưa cắt khoảng trắng → bước kiểm
         * 01 §6.2 sau chuyển đổi thất bại (cần công cụ sqlite3 của máy dựng).
         */
        fun breakOnePerson(dir: File) {
            val p = runCatching {
                ProcessBuilder(
                    "sqlite3", File(dir, "lvhv.sqlite").path,
                    "UPDATE person SET json = json_set(json, '$.death.notice.place', '  Quê nhà') WHERE id = '0b8e4f52-3c1a-4d2e-9f60-1a2b3c4d5e02';",
                ).redirectErrorStream(true).start()
            }.getOrNull()
            assumeTrue("cần công cụ sqlite3", p != null && p.waitFor() == 0)
        }
    }

    @Test
    fun releasedStoreIsMigrated() {
        val repo = PeopleRepo(LvhvStore.open(copyFixture().absolutePath))
        val st = repo.migration()
        assertEquals("migrated", st.state)
        assertEquals(1L, st.fromVersion)
        assertTrue(File(st.backupPath!!).exists())
        val ps = repo.persons()
        assertEquals(3, ps.size)
        val b = ps.first { it.id.endsWith("5e02") }
        val att = b.notice!!.objs("attachments")
        assertEquals(listOf("giay_chung_tu", "giay_chung_tu"), att.map { it.s("kind") })
        for (a in att) assertNotNull(repo.attachmentBytes(a.s("file_id")))
        assertFalse(b.notice!!.containsKey("certificate_photo_ids"))
        assertFalse(repo.readOnly)
    }

    @Test
    fun failedMigrationIsReadOnly() {
        val dir = copyFixture()
        breakOnePerson(dir)
        val repo = PeopleRepo(LvhvStore.open(dir.absolutePath))
        val st = repo.migration()
        assertEquals("read_only", st.state)
        assertTrue(repo.readOnly)
        assertTrue(!st.message.isNullOrBlank())
        assertEquals(3, repo.persons().size)
        assertTrue("ghi phải bị từ chối", runCatching { repo.putSettings(repo.settings()) }.isFailure)
        // Vẫn tạo được tệp sao lưu (phần đầu hợp lệ). Như store/tests/released.rs: hồ sơ hỏng vẫn bị kiểm khi nhập, nên
        // mở lại tệp này báo lỗi cho tới khi dữ liệu được sửa.
        val bytes = repo.exportBackup("mật khẩu thử nghiệm", "0.0.0")
        PeopleRepo.checkBackupHeader(bytes.copyOf(72), bytes.size.toLong())
        assertTrue(runCatching { PeopleRepo.openBackupFile(bytes, "mật khẩu thử nghiệm") }.isFailure)
    }
}
