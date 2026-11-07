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

        /** Trigger chặn ghi bảng `meta` (như Windows `upgrade.rs`): bước chuyển 1 → 2 lỗi ở câu SQL cuối giao dịch. */
        const val FAIL_TRIGGER =
            "CREATE TRIGGER kiem_dinh_loi_chuyen_doi BEFORE INSERT ON meta BEGIN SELECT RAISE(ABORT, 'R14 kiem dinh: loi chuyen doi'); END;"

        /**
         * Làm bước chuyển đổi lỗi thật mà dữ liệu cũ vẫn hợp lệ (`force_migration_failure` của lõi không có qua FFI) —
         * cần công cụ sqlite3 của máy dựng.
         */
        fun breakMigration(dir: File) = sqlite(dir, FAIL_TRIGGER)

        fun sqlite(dir: File, sql: String): String {
            val p = runCatching { ProcessBuilder("sqlite3", File(dir, "lvhv.sqlite").path, sql).redirectErrorStream(true).start() }.getOrNull()
            val out = p?.inputStream?.bufferedReader()?.readText() ?: ""
            assumeTrue("cần công cụ sqlite3", p != null && p.waitFor() == 0)
            return out
        }

        /** Bản sao JSON với khóa đối tượng được sắp (so sánh không phụ thuộc thứ tự khóa); số → Double. */
        fun sorted(v: Any?): Any? = when (v) {
            is Map<*, *> -> java.util.TreeMap<String, Any?>().apply { v.forEach { (k, x) -> put(k.toString(), sorted(x)) } }
            is List<*> -> v.map { sorted(it) }
            is Number -> v.toDouble()
            else -> v
        }

        /** Nội dung mọi bảng dữ liệu của kho (như `db_content` của Windows / `.dump` của các bảng). */
        fun content(dir: File): String =
            listOf("settings", "photo", "person", "event_type", "event", "event_person", "meta").joinToString("\n") { t ->
                "## $t\n" + sqlite(dir, "SELECT * FROM $t ORDER BY 1;")
            } + "\nuser_version=" + sqlite(dir, "PRAGMA user_version;")
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
        breakMigration(dir)
        val before = content(dir)
        val photosBefore = File(dir, "photos").listFiles()!!.associate { it.name to it.readBytes().contentHashCode() }
        val repo = PeopleRepo(LvhvStore.open(dir.absolutePath))
        val st = repo.migration()
        assertEquals("read_only", st.state)
        assertTrue(repo.readOnly)
        assertTrue(!st.message.isNullOrBlank())
        assertEquals(3, repo.persons().size)
        assertTrue("ghi phải bị từ chối", runCatching { repo.putSettings(repo.settings()) }.isFailure)
        // Kho cũ giữ nguyên từng dòng mọi bảng, từng ảnh; không có tệp đính kèm mồ côi.
        assertEquals(before, content(dir))
        assertEquals(photosBefore, File(dir, "photos").listFiles()!!.associate { it.name to it.readBytes().contentHashCode() })
        assertTrue(File(dir, "attachments").listFiles().isNullOrEmpty())
        // Tệp sao lưu xuất từ chế độ chỉ đọc mở lại được trọn vẹn; nhập vào kho trống của bản mới → đúng trạng thái kỳ vọng.
        val bytes = repo.exportBackup("mật khẩu thử nghiệm", "0.0.0")
        val (v, pv) = PeopleRepo.openBackupFile(bytes, "mật khẩu thử nghiệm")
        assertEquals(3, pv.persons.toInt())
        assertTrue(pv.photos.toInt() >= 1) // hai ảnh của bộ mẫu cùng byte (5e09, 5e0c)
        val fresh = PeopleRepo(LvhvStore.open(Files.createTempDirectory("lvhv-r14-import").toFile().absolutePath))
        fresh.applyImport(v, replace = true)
        v.destroy()
        val exp = parseObj(File(FIX, "../../expected/0.2-dev-before-attachments.json").readText()).o("state")!!
        val got = parseObj(fresh.datasetJson())
        for (k in listOf("persons", "events", "event_types")) assertEquals(k, JsonW.write(sorted(exp[k])), JsonW.write(sorted(got[k])))
        val att = got.objs("attachments")
        assertEquals(exp.objs("attachments").map { it.s("sha256") }.sorted(), att.map { it.s("sha256") }.sorted())
        for (a in att) {
            val b = fresh.attachmentBytes(a.s("id"))!!
            val sha = java.security.MessageDigest.getInstance("SHA-256").digest(b).joinToString("") { "%02x".format(it) }
            assertEquals(a.s("sha256"), sha)
        }
    }

    @Test
    fun truncatedPdfIsRejected() {
        val repo = Seed.newRepo()
        val cut = Seed.PDF.copyOf(Seed.PDF.size - 7) // mất "%%EOF\n"
        assertTrue(runCatching { vn.clevix.lichviet.ui.people.checkPdf(cut) }.isFailure)
        vn.clevix.lichviet.ui.people.checkPdf(Seed.PDF)
        // Lõi cũng từ chối khi lưu (media::check_attachment).
        assertTrue(runCatching { repo.addAttachment("application/pdf", null, cut) }.isFailure)
    }
}
