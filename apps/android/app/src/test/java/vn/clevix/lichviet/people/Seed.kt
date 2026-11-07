package vn.clevix.lichviet.people

import java.io.File
import java.nio.file.Files
import java.time.LocalDate
import vn.clevix.lichviet.core.LvhvStore
import vn.clevix.lichviet.core.solarToLunar

/** Dữ liệu mẫu cố định cho kiểm thử giao diện và ảnh chụp (hôm nay = 07/11/2026). */
object Seed {
    val TODAY: LocalDate = LocalDate.of(2026, 11, 7)

    /** Thư mục kho của lần newRepo() gần nhất. */
    lateinit var lastDir: File

    fun newRepo(): PeopleRepo {
        val dir: File = Files.createTempDirectory("lvhv-and-test").toFile()
        lastDir = dir
        return PeopleRepo(LvhvStore.open(dir.absolutePath))
    }

    private fun dv(cal: String, y: Int?, m: Int, d: Int, leap: Boolean = false, precision: String = "day"): JMap =
        DateValue(cal, precision, y, m, d, leap).toMap()

    private fun rule(offsets: List<Int> = listOf(0, 3), tt: Boolean = false): JMap = linkedMapOf<String, Any?>(
        "id" to PeopleRepo.uuid(), "enabled" to true, "offsets_days" to ArrayList(offsets), "time_local" to "07:00", "tien_thuong" to tt,
    )

    fun deceased(
        danhXung: String, hoTen: String, quanHe: String, death: JMap, birth: JMap? = null,
        tenHieu: String = "", loi: String = "", notice: JMap? = null,
    ): JMap {
        val d = linkedMapOf<String, Any?>("input" to death)
        if (notice != null) d["notice"] = notice
        return linkedMapOf<String, Any?>(
            "id" to "", "kind" to "deceased", "danh_xung" to danhXung, "ho_ten" to hoTen, "ten_hieu" to tenHieu, "quan_he" to quanHe,
            "birth" to birth?.let { linkedMapOf<String, Any?>("input" to it, "noi_sinh" to "Hà Nội") },
            "death" to d, "noi_an_tang" to "Nghĩa trang quê", "photo_id" to null,
            "memorial" to linkedMapOf<String, Any?>(
                "tray_style" to "truyen_thong", "incense" to true, "bia_tieu_de" to "", "loi_tuong_niem" to loi,
                "gio_het_nam" to 2, "huong_duong_duoi" to 60,
            ),
            "reminders" to linkedMapOf<String, Any?>("gio" to rule(), "sinh_nhat" to null, "sinh_nhat_am" to null),
            "sinh_nhat_am" to false, "feb29_rule" to "feb28", "ghi_chu" to "",
        )
    }

    fun living(danhXung: String, hoTen: String, quanHe: String, birth: JMap?): JMap = linkedMapOf<String, Any?>(
        "id" to "", "kind" to "living", "danh_xung" to danhXung, "ho_ten" to hoTen, "ten_hieu" to "", "quan_he" to quanHe,
        "birth" to birth?.let { linkedMapOf<String, Any?>("input" to it, "noi_sinh" to "Hà Đông") },
        "death" to null, "noi_an_tang" to "", "photo_id" to null, "memorial" to null,
        "reminders" to linkedMapOf<String, Any?>("gio" to null, "sinh_nhat" to rule(), "sinh_nhat_am" to null),
        "sinh_nhat_am" to false, "feb29_rule" to "feb28", "ghi_chu" to "",
    )

    class Ids(val a: String, val b: String, val c: String, val m: String, val d: String, val evC: String)

    /** Gia đình mẫu: ông A (mất 08/08/2025), bà B (giỗ đúng hôm nay), cụ bà D (có thông báo tử), C và M đang sống. */
    /** Một tệp PDF tối thiểu hợp lệ cho lõi (`%PDF-` … `%%EOF`). */
    val PDF: ByteArray = "%PDF-1.4\n1 0 obj<</Type/Catalog>>endobj\ntrailer<</Root 1 0 R>>\n%%EOF\n".toByteArray(Charsets.ISO_8859_1)

    /**
     * [attachments]: (loại, mime, tên tệp, byte) gắn vào thông tin báo tử của cụ bà D — ảnh phải là JPEG không siêu dữ liệu
     * (kiểm thử giao diện tạo bằng `Bitmap.compress`).
     */
    fun family(repo: PeopleRepo, attachments: List<Triple<String, String, Pair<String?, ByteArray>>> = emptyList()): Ids {
        val att = ArrayList<Any?>(attachments.map { (kind, mime, f) ->
            val fid = repo.addAttachment(mime, f.first, f.second)
            linkedMapOf<String, Any?>("id" to PeopleRepo.uuid(), "kind" to kind, "file_id" to fid)
        })
        val a = repo.savePerson(
            deceased(
                "Ông", "Nguyễn Văn A", "ông nội (bên nội)", dv("solar", 2025, 8, 8), dv("solar", 1940, 5, 1),
                tenHieu = "Phúc Thiện", loi = "Con cháu luôn nhớ ông.",
            ),
        ).id
        // Bà B: ngày mất âm lịch = ngày âm của hôm nay (năm 2019) → giỗ đúng hôm nay.
        val l = solarToLunar(TODAY.year, TODAY.monthValue, TODAY.dayOfMonth)
        val b = repo.savePerson(deceased("Bà", "Trần Thị B", "bà nội", dv("lunar", 2019, l.m, l.d), dv("lunar", 1938, 2, 10))).id
        val d = repo.savePerson(
            deceased(
                "Cụ bà", "Phạm Thị Dung", "Cụ bà", dv("solar", 2024, 7, 17), dv("solar", 1931, 3, 2),
                notice = linkedMapOf<String, Any?>(
                    "reported_by" to "Chú Phạm Văn Hải", "place" to "Bệnh viện Bạch Mai", "reason" to "Tuổi cao, bệnh nặng",
                    "notice_date" to dv("solar", 2024, 7, 18), "attachments" to att, "note" to "Gia đình đã báo họ hàng.",
                ),
            ),
        ).id
        val c = repo.savePerson(living("", "Lê Văn Cường", "Con trai", dv("solar", 1990, 1, 20))).id
        val m = repo.savePerson(living("Chú", "Đỗ Văn Minh", "Chú", dv("solar", 1958, 11, 25))).id
        val ev = repo.saveEvent(
            linkedMapOf<String, Any?>(
                "id" to "", "type_id" to "6f1c2a1e-0b6e-4c55-9a51-0d1f2b7c0001", "tieu_de" to "Tốt nghiệp đại học",
                "date" to dv("solar", 2026, 11, 20), "recurring" to false, "person_ids" to arrayListOf(c), "ghi_chu" to "",
                "reminder" to rule(listOf(1)),
            ),
        ).id
        return Ids(a, b, c, m, d, ev)
    }
}
