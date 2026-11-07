package vn.clevix.lichviet.people

import java.time.Instant
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import vn.clevix.lichviet.core.LvhvDataException

/**
 * YC-2026-07 và các hàm dùng chung của lõi (core2, #28) qua lớp [PeopleRepo]: gợi ý, viết hoa, giới tính, tệp đính kèm,
 * xóa sự kiện mồ côi, quy tắc mật khẩu, tháng nhuận, ghi nhận đã sao lưu, thẻ nhắc hôm nay.
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class Yc07Test {
    @Test
    fun coreSuggestionsPerKind() {
        val dead = PeopleRepo.danhXungList(true, null)
        val live = PeopleRepo.danhXungList(false, null)
        assertTrue(dead.isNotEmpty() && live.isNotEmpty())
        assertTrue("Cụ" in dead)
        assertFalse("Cụ" in live)
        assertTrue(PeopleRepo.quanHeList(true).isNotEmpty())
        // Từ hợp giới tính lên trước.
        val nu = PeopleRepo.danhXungList(true, "nu")
        assertTrue(nu.indexOf("Bà") < nu.indexOf("Ông"))
    }

    @Test
    fun displayCapitalize() {
        assertEquals("Ông nội (bên nội)", PeopleRepo.cap("ông nội (bên nội)"))
        assertEquals("", PeopleRepo.cap("  "))
    }

    @Test
    fun namesAreCapitalisedAndGenderKept() {
        val repo = Seed.newRepo()
        val m = Seed.living("anh", "Lê Văn Đức", "con trai", null).also { it["gioi_tinh"] = "nam" }
        val p = repo.person(repo.savePerson(m).id)!!
        assertEquals("Anh", p.danhXung)
        assertEquals("Con trai", p.quanHe)
        assertEquals("Anh Lê Văn Đức", p.displayName)
        assertEquals("nam", p.raw.sn("gioi_tinh"))
        assertEquals("Nam", p.gioiTinhText)
    }

    @Test
    fun attachmentsLifecycle() {
        val repo = Seed.newRepo()
        val ids = Seed.family(repo, listOf(Triple("giay_chung_tu", "application/pdf", "Giay chung tu.pdf" to Seed.PDF)))
        val d = repo.person(ids.d)!!
        val list = d.notice!!.objs("attachments")
        assertEquals(1, list.size)
        val fid = list[0].s("file_id")
        assertEquals("giay_chung_tu", list[0].s("kind"))
        assertTrue(repo.attachmentBytes(fid)!!.contentEquals(Seed.PDF))
        assertEquals("Giay chung tu.pdf", repo.attachmentMeta()[fid]!!.sn("file_name"))
        // Tệp chưa gắn: xóa được; tệp đang gắn: lõi từ chối (deleteAttachment nuốt lỗi, tệp còn).
        val loose = repo.addAttachment("application/pdf", null, Seed.PDF)
        repo.deleteAttachment(loose)
        assertNull(repo.attachmentBytes(loose))
        repo.deleteAttachment(fid)
        assertNotNull(repo.attachmentBytes(fid))
        // Gỡ khỏi danh sách rồi lưu → lõi tự xóa tệp.
        val raw = d.raw
        (raw.o("death")!!.o("notice")!!)["attachments"] = ArrayList<Any?>()
        repo.savePerson(raw)
        assertNull(repo.attachmentBytes(fid))
        // Không phải PDF → lỗi tiếng Việt.
        try {
            repo.addAttachment("application/pdf", null, "xin chao".toByteArray())
            org.junit.Assert.fail()
        } catch (e: LvhvDataException) {
            assertTrue(errText(e).isNotBlank())
        }
    }

    @Test
    fun deletePersonRemovesOrphanEventsOnly() {
        val repo = Seed.newRepo()
        val ids = Seed.family(repo)
        val removed = repo.deletePerson(ids.c)
        assertEquals(listOf(ids.evC), removed)
        assertTrue(repo.events().none { it.id == ids.evC })
    }

    @Test
    fun passwordRuleFromCore() {
        assertNotNull(PeopleRepo.passwordError("ngan", null))
        assertNull(PeopleRepo.passwordError("đủ tám ký tự", null))
        val nfc = "mật khẩu Hà Đông"
        val nfd = java.text.Normalizer.normalize(nfc, java.text.Normalizer.Form.NFD)
        assertNull("NFC/NFD là cùng mật khẩu", PeopleRepo.passwordError(nfc, nfd))
        assertNotNull(PeopleRepo.passwordError(nfc, nfc + "x"))
    }

    @Test
    fun leapMonthFromCore() {
        assertEquals(6, PeopleRepo.leapMonth(2025))
        assertEquals(2, PeopleRepo.leapMonth(2023))
        assertEquals(0, PeopleRepo.leapMonth(2024))
        assertEquals(0, PeopleRepo.leapMonth(5000))
    }

    @Test
    fun backupSavedOnlyWhenMarked() {
        val repo = Seed.newRepo()
        Seed.family(repo)
        repo.exportBackup("mật khẩu đủ dài", "0.0.0")
        assertNull("tạo byte chưa phải đã lưu", repo.settings().sn("last_backup_at"))
        repo.markBackupSaved(Instant.parse("2026-11-07T03:00:00Z"))
        assertNotNull(repo.settings().sn("last_backup_at"))
    }

    @Test
    fun dueTodayHasNamesAndMigrationOk() {
        val repo = Seed.newRepo()
        Seed.family(repo)
        val today = repo.todayReminders(Seed.TODAY, java.time.ZoneId.of("Asia/Ho_Chi_Minh"))
        assertTrue("bà B giỗ hôm nay", today.any { it.title.contains("Trần Thị B") })
        assertEquals("ok", repo.migration().state)
        assertFalse(repo.readOnly)
    }
}
