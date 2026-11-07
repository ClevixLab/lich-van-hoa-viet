package vn.clevix.lichviet.debug

import android.app.Activity
import android.content.Context
import android.graphics.Bitmap
import android.graphics.Color
import android.os.Bundle
import android.util.Log
import java.io.ByteArrayOutputStream
import java.io.File
import java.security.MessageDigest
import java.time.LocalDate
import vn.clevix.lichviet.notify.PeopleStore
import vn.clevix.lichviet.notify.Reminders
import vn.clevix.lichviet.people.DateValue
import vn.clevix.lichviet.people.JMap
import vn.clevix.lichviet.people.JsonW
import vn.clevix.lichviet.people.PeopleRepo
import vn.clevix.lichviet.people.Photos
import vn.clevix.lichviet.people.o
import vn.clevix.lichviet.people.objs
import vn.clevix.lichviet.people.parseObj
import vn.clevix.lichviet.people.s

/**
 * Tự kiểm nâng cấp R14 (docs/v0.2/04 Bổ sung B5) — CHỈ có trong bản debug. CI gọi:
 *   adb shell am start -n <gói>/vn.clevix.lichviet.debug.SelfCheckActivity --es op seed|verify|verify-prefs
 * Kết quả ghi vào files/upgrade-result.txt ("OK …" hoặc "FAIL …") và logcat thẻ "LvhvR14".
 *
 * - seed: tạo dữ liệu người thân đủ loại (người đã mất có ảnh chân dung + thông tin báo tử có ảnh giấy tờ, người đang
 *   sống, sự kiện, cài đặt, tùy chọn tên trên màn hình khóa), ghi "kỳ vọng" (toàn bộ dữ liệu + sha256 từng ảnh) vào
 *   files/upgrade-expected.json, rồi HỦY báo thức của chuỗi — sau khi cài đè, báo thức phải do ứng dụng đặt lại.
 * - verify: so dữ liệu hiện có với kỳ vọng (0 bản ghi mất, 0 trường khác), đọc lại từng ảnh, kiểm báo thức đã được đặt lại.
 * - verify-prefs: tùy chọn giao diện của bản 0.1.x (cỡ chữ, sáng/tối, "Để sau") còn nguyên — giá trị CI đã ghi trước khi
 *   cài đè (R14 (a)).
 * - verify-migrated: CI đã chép kho thật của bản 0.2-dev (bộ mẫu đóng băng `0.2-dev-before-attachments`, phiên bản 1) và
 *   tệp kỳ vọng `files/r14-expected.json` vào máy; mở bằng bản này → `migrated`, dữ liệu đúng từng trường như tệp kỳ
 *   vọng (cùng tệp bản Rust và bản web phải ra), từng tệp đính kèm đúng sha256, có bản sao dữ liệu cũ (R14 (c)).
 * - verify-readonly: bước chuyển 1 → 2 lỗi (CI thêm trigger chặn ghi bảng `meta`) → `read_only`: dữ liệu cũ xem được,
 *   mọi thao tác ghi bị từ chối; tệp sao lưu xuất ra mở lại được trọn vẹn và nhập vào kho trống cho đúng trạng thái kỳ
 *   vọng (B5 mục 3, MA3C-24-01). CI so thêm từng dòng mọi bảng và từng ảnh của kho trước/sau.
 */
class SelfCheckActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val op = intent.getStringExtra("op") ?: "verify"
        val result = try {
            when (op) {
                "seed" -> seed(this)
                "verify" -> verify(this)
                "verify-prefs" -> verifyPrefs(this)
                "verify-migrated" -> verifyMigrated(this)
                "verify-readonly" -> verifyReadOnly(this)
                else -> "FAIL thao tác lạ: $op"
            }
        } catch (e: Throwable) {
            "FAIL ${e.javaClass.simpleName}: ${e.message}"
        }
        Log.i(TAG, "$op → $result")
        File(filesDir, "upgrade-result.txt").writeText(result + "\n")
        finish()
    }

    companion object {
        const val TAG = "LvhvR14"

        private fun sha(b: ByteArray): String =
            MessageDigest.getInstance("SHA-256").digest(b).joinToString("") { "%02x".format(it) }

        private fun jpeg(w: Int, h: Int, color: Int): ByteArray {
            val bmp = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
            bmp.eraseColor(color)
            return ByteArrayOutputStream().also { bmp.compress(Bitmap.CompressFormat.JPEG, 90, it) }.toByteArray()
        }

        private fun rule(offsets: List<Int>): JMap = linkedMapOf<String, Any?>(
            "id" to PeopleRepo.uuid(), "enabled" to true, "offsets_days" to ArrayList(offsets), "time_local" to "07:00",
            "tien_thuong" to false,
        )

        /** Ảnh dữ liệu: toàn bộ kho (bỏ thời điểm xuất) + băm từng ảnh + tùy chọn chỉ có trên máy. */
        private fun snapshot(repo: PeopleRepo): JMap {
            val ds = parseObj(repo.datasetJson())
            ds.remove("exported_at")
            val photos = linkedMapOf<String, Any?>()
            val files = linkedMapOf<String, Any?>()
            for (p in repo.persons()) {
                p.photoId?.let { id -> photos[id] = repo.photo(id)?.let { sha(it) } ?: "MẤT" }
                for (a in (p.notice?.get("attachments") as? List<*>) ?: emptyList<Any?>()) {
                    val fid = (a as Map<*, *>)["file_id"] as String
                    files[fid] = repo.attachmentBytes(fid)?.let { sha(it) } ?: "MẤT"
                }
            }
            return linkedMapOf(
                "dataset" to ds, "photos" to photos, "attachments" to files, "lock_screen_names" to repo.lockScreenNames(),
            )
        }

        fun seed(ctx: Context): String {
            PeopleStore.reset()
            val repo = PeopleStore.get(ctx) ?: return "FAIL không mở được kho: ${PeopleStore.openError()}"
            if (repo.persons().isNotEmpty()) return "FAIL kho đã có dữ liệu"
            val today = LocalDate.now()
            val portrait = repo.addPhoto(Photos.encode(jpeg(300, 400, Color.rgb(150, 110, 80))))
            val cert = repo.addAttachment(
                "image/jpeg", "giay-chung-tu.jpg", Photos.encode(jpeg(1600, 1200, Color.rgb(230, 230, 220)), Photos.ATT_EDGE, Photos.ATT_STORED).bytes,
            )
            val pdf = repo.addAttachment(
                "application/pdf", "Tin bao tu.pdf",
                "%PDF-1.4\n1 0 obj<</Type/Catalog>>endobj\ntrailer<</Root 1 0 R>>\n%%EOF\n".toByteArray(Charsets.ISO_8859_1),
            )
            val a = repo.savePerson(
                linkedMapOf<String, Any?>(
                    "id" to "", "kind" to "deceased", "danh_xung" to "Ông", "ho_ten" to "Nguyễn Văn Ánh", "ten_hieu" to "Phúc Thiện",
                    "gioi_tinh" to "nam",
                    "quan_he" to "Ông nội",
                    "birth" to linkedMapOf<String, Any?>("input" to DateValue("solar", "day", 1940, 5, 1).toMap(), "noi_sinh" to "Hà Nội"),
                    "death" to linkedMapOf<String, Any?>(
                        "input" to DateValue("lunar", "day", 2025, 6, 15, true).toMap(),
                        "notice" to linkedMapOf<String, Any?>(
                            "reported_by" to "Chú Hải", "place" to "Bệnh viện Bạch Mai", "reason" to "Tuổi cao",
                            "notice_date" to DateValue("solar", "day", 2025, 8, 9).toMap(),
                            "attachments" to arrayListOf(
                                linkedMapOf<String, Any?>("id" to PeopleRepo.uuid(), "kind" to "giay_chung_tu", "file_id" to cert),
                                linkedMapOf<String, Any?>("id" to PeopleRepo.uuid(), "kind" to "tin_bao_tu", "file_id" to pdf),
                            ),
                            "note" to "Ghi chú báo tử",
                        ),
                    ),
                    "noi_an_tang" to "Nghĩa trang quê", "photo_id" to portrait,
                    "memorial" to linkedMapOf<String, Any?>(
                        "tray_style" to "truyen_thong", "incense" to true, "bia_tieu_de" to "", "loi_tuong_niem" to "Con cháu nhớ ông",
                        "gio_het_nam" to 3, "huong_duong_duoi" to 60,
                    ),
                    "reminders" to linkedMapOf<String, Any?>("gio" to rule(listOf(0, 3, 10)), "sinh_nhat" to null, "sinh_nhat_am" to null),
                    "sinh_nhat_am" to false, "feb29_rule" to "feb28", "ghi_chu" to "Dòng 1\nDòng 2",
                ),
            )
            val c = repo.savePerson(
                linkedMapOf<String, Any?>(
                    "id" to "", "kind" to "living", "danh_xung" to "", "ho_ten" to "Lê Văn Cường", "ten_hieu" to "", "quan_he" to "Con trai",
                    "birth" to linkedMapOf<String, Any?>("input" to DateValue("solar", "day", 1992, 2, 29).toMap(), "noi_sinh" to "Hà Đông"),
                    "death" to null, "noi_an_tang" to "", "photo_id" to null, "memorial" to null,
                    "reminders" to linkedMapOf<String, Any?>("gio" to null, "sinh_nhat" to rule(listOf(0, 1)), "sinh_nhat_am" to rule(listOf(0))),
                    "sinh_nhat_am" to true, "feb29_rule" to "mar1", "ghi_chu" to "",
                ),
            )
            val type = repo.saveEventType(
                linkedMapOf<String, Any?>("id" to "", "builtin" to null, "ten" to "Lên lão", "default_recurring" to true),
            )
            repo.saveEvent(
                linkedMapOf<String, Any?>(
                    "id" to "", "type_id" to type.id, "tieu_de" to "", "date" to DateValue("lunar", "month_day", null, 3, 10).toMap(),
                    "recurring" to true, "person_ids" to arrayListOf(c.id), "ghi_chu" to "", "reminder" to rule(listOf(0, 5)),
                ),
            )
            repo.saveEvent(
                linkedMapOf<String, Any?>(
                    "id" to "", "type_id" to "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0001", "tieu_de" to "Tốt nghiệp",
                    "date" to DateValue("solar", "day", today.year + 1, 6, 15).toMap(), "recurring" to false,
                    "person_ids" to arrayListOf(c.id, a.id), "ghi_chu" to "", "reminder" to null,
                ),
            )
            val s = repo.settings()
            s["backup_nag_days"] = 60
            s["default_time_local"] = "06:30"
            repo.putSettings(s)
            repo.setLockScreenNames(true)
            File(ctx.filesDir, "upgrade-expected.json").writeText(JsonW.write(snapshot(repo)))
            // Báo thức hiện có bị hủy: sau khi cài đè, báo thức phải do ứng dụng tự đặt lại (MY_PACKAGE_REPLACED).
            Reminders.cancelAlarm(ctx)
            if (Reminders.alarmArmed(ctx)) return "FAIL không hủy được báo thức"
            return "OK seed ${repo.persons().size} người, ${repo.events().size} sự kiện"
        }

        fun verify(ctx: Context): String {
            val f = File(ctx.filesDir, "upgrade-expected.json")
            if (!f.exists()) return "FAIL mất tệp kỳ vọng (thư mục dữ liệu bị xóa?)"
            PeopleStore.reset()
            val repo = PeopleStore.get(ctx) ?: return "FAIL không mở được kho: ${PeopleStore.openError()}"
            val expected = f.readText()
            val actual = JsonW.write(snapshot(repo))
            if (expected != actual) {
                val i = expected.zip(actual).indexOfFirst { (x, y) -> x != y }.let { if (it < 0) minOf(expected.length, actual.length) else it }
                return "FAIL dữ liệu khác kỳ vọng tại ký tự $i: kỳ vọng …${expected.substring(maxOf(0, i - 40), minOf(expected.length, i + 80))}… " +
                    "thực tế …${actual.substring(maxOf(0, i - 40), minOf(actual.length, i + 80))}…"
            }
            if (actual.contains("\"MẤT\"")) return "FAIL mất ảnh"
            if (!Reminders.alarmArmed(ctx)) return "FAIL chuỗi báo thức chưa được đặt lại sau cài đè"
            return "OK verify ${repo.persons().size} người, ${repo.events().size} sự kiện, báo thức đã đặt lại"
        }

        /** Bỏ trường không so được (thời điểm xuất, thông tin ứng dụng) khỏi tài liệu dữ liệu. */
        private fun stateOf(doc: JMap): JMap = doc.also { it.remove("app"); it.remove("exported_at") }

        /** Chỗ khác đầu tiên giữa hai giá trị JSON (đường dẫn), null khi giống hệt. */
        private fun diff(a: Any?, b: Any?, path: String = ""): String? {
            if (a is Map<*, *> && b is Map<*, *>) {
                for (k in (a.keys + b.keys).map { it.toString() }.toSortedSet()) diff(a[k], b[k], "$path/$k")?.let { return it }
                return null
            }
            if (a is List<*> && b is List<*>) {
                if (a.size != b.size) return "$path: ${a.size} phần tử ≠ ${b.size}"
                a.indices.forEach { i -> diff(a[i], b[i], "$path/$i")?.let { return it } }
                return null
            }
            if (a is Number && b is Number) return if (a.toDouble() == b.toDouble()) null else "$path: $a ≠ $b"
            return if (a == b) null else "$path: $a ≠ $b"
        }

        fun verifyMigrated(ctx: Context): String {
            val exp = File(ctx.filesDir, "r14-expected.json")
            if (!exp.exists()) return "FAIL thiếu tệp kỳ vọng"
            PeopleStore.reset()
            val repo = PeopleStore.get(ctx) ?: return "FAIL không mở được kho: ${PeopleStore.openError()}"
            val st = repo.migration()
            if (st.state != "migrated") return "FAIL trạng thái ${st.state} (${st.message}), cần migrated"
            if (st.fromVersion != 1L) return "FAIL from_version ${st.fromVersion}"
            val bak = st.backupPath?.let { File(it) }
            if (bak == null || !bak.exists()) return "FAIL không có bản sao dữ liệu cũ"
            val expected = parseObj(exp.readText()).o("state") ?: return "FAIL tệp kỳ vọng hỏng"
            val actual = stateOf(parseObj(repo.datasetJson()))
            diff(expected, actual)?.let { return "FAIL dữ liệu sau chuyển đổi khác kỳ vọng tại $it" }
            var n = 0
            for (a in actual.objs("attachments")) {
                val b = repo.attachmentBytes(a.s("id")) ?: return "FAIL mất tệp đính kèm ${a.s("id")}"
                if (sha(b) != a.s("sha256")) return "FAIL sha256 tệp đính kèm ${a.s("id")}"
                n++
            }
            for (p in actual.objs("photos")) {
                if (repo.photo(p.s("id")) == null) return "FAIL mất ảnh ${p.s("id")}"
            }
            return "OK migrated 1 → ${st.toVersion}: ${repo.persons().size} người, ${repo.events().size} sự kiện, $n tệp đính kèm, bản sao ${bak.name}"
        }

        fun verifyReadOnly(ctx: Context): String {
            PeopleStore.reset()
            val repo = PeopleStore.get(ctx) ?: return "FAIL không mở được kho: ${PeopleStore.openError()}"
            val st = repo.migration()
            if (st.state != "read_only") return "FAIL trạng thái ${st.state}, cần read_only"
            if (st.message.isNullOrBlank()) return "FAIL không có câu báo cho người dùng"
            val bak = st.backupPath?.let { File(it) }
            if (bak == null || !bak.exists()) return "FAIL không có bản sao dữ liệu cũ"
            val n = repo.persons().size
            if (n == 0) return "FAIL không xem được dữ liệu cũ"
            val write = runCatching { repo.putSettings(repo.settings()) }
            if (write.isSuccess) return "FAIL ghi được khi chỉ đọc"
            val photos = parseObj(repo.datasetJson()).objs("photos").size
            val bytes = runCatching { repo.exportBackup("mật khẩu thử nghiệm", vn.clevix.lichviet.BuildConfig.VERSION_NAME) }
            if (bytes.isFailure) return "FAIL không xuất được sao lưu: ${bytes.exceptionOrNull()?.message}"
            // Tệp xuất được giữ lại để CI giải mã thêm bằng công cụ của lõi trên máy dựng (chẩn đoán khi lỗi).
            File(ctx.filesDir, "r14-export.lvhv").writeBytes(bytes.getOrThrow())
            // MA3C-24-01: mở lại trọn vẹn tệp vừa xuất (giải mã + kiểm toàn bộ qua lõi), nhập vào một kho trống của bản này
            // rồi so từng hồ sơ, sự kiện, loại sự kiện với tệp kỳ vọng chung và từng tệp đính kèm theo sha256.
            val (v, pv) = runCatching { PeopleRepo.openBackupFile(bytes.getOrThrow(), "mật khẩu thử nghiệm") }
                .getOrElse { return "FAIL tệp sao lưu xuất từ chế độ chỉ đọc không mở được: ${it.message}" }
            if (pv.persons.toInt() != n || pv.photos.toInt() !in 1..photos) {
                v.destroy()
                return "FAIL tệp sao lưu có ${pv.persons} người, ${pv.photos} ảnh; kho có $n người, $photos ảnh"
            }
            val dir = File(ctx.cacheDir, "r14-import").apply { deleteRecursively(); mkdirs() }
            val fresh = PeopleRepo(vn.clevix.lichviet.core.LvhvStore.open(dir.absolutePath))
            runCatching { fresh.applyImport(v, true) }.also { v.destroy() }
                .onFailure { return "FAIL không nhập được tệp sao lưu vào kho trống: ${it.message}" }
            val exp = File(ctx.filesDir, "r14-expected.json")
            if (!exp.exists()) return "FAIL thiếu tệp kỳ vọng"
            val want = parseObj(exp.readText()).o("state") ?: return "FAIL tệp kỳ vọng hỏng"
            val got = parseObj(fresh.datasetJson())
            for (k in listOf("persons", "events", "event_types")) diff(want[k], got[k], "/$k")?.let { return "FAIL sao lưu → nhập khác kỳ vọng tại $it" }
            val wantSha = want.objs("attachments").map { it.s("sha256") }.sorted()
            val gotAtt = got.objs("attachments")
            if (gotAtt.map { it.s("sha256") }.sorted() != wantSha) return "FAIL tệp đính kèm sau nhập khác kỳ vọng"
            for (a in gotAtt) {
                val b = fresh.attachmentBytes(a.s("id")) ?: return "FAIL mất tệp đính kèm ${a.s("id")} sau nhập"
                if (sha(b) != a.s("sha256")) return "FAIL sha256 tệp đính kèm ${a.s("id")} sau nhập"
            }
            dir.deleteRecursively()
            return "OK read_only: $n người xem được, ghi bị từ chối; tệp sao lưu mở lại được ($n người, $photos ảnh), " +
                "nhập vào kho trống khớp kỳ vọng (${gotAtt.size} tệp đính kèm đúng sha256); ${st.message}"
        }

        /** Giá trị CI ghi vào shared_prefs/hien_thi.xml trước khi cài đè (cỡ chữ A++, giao diện tối, "Để sau"). */
        fun verifyPrefs(ctx: Context): String {
            val sp = ctx.getSharedPreferences("hien_thi", Context.MODE_PRIVATE)
            val size = sp.getInt("size", -1)
            val theme = sp.getInt("theme", -1)
            val snooze = sp.getString("nhac_cap_nhat_den", null)
            return if (size == 3 && theme == 2 && snooze == "2099-12-31") "OK prefs size=$size theme=$theme snooze=$snooze"
            else "FAIL prefs size=$size theme=$theme snooze=$snooze"
        }
    }
}
