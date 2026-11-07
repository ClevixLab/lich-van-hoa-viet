// Bản nháp biểu mẫu (người đã mất, người đang sống, sự kiện, thông tin báo tử, quy tắc nhắc): giữ chữ người dùng
// gõ, kiểm tra theo câu báo lỗi của 08 §3.4–§3.7, dựng JSON đúng lược đồ (01 §3) để gửi cho lõi. Kiểm tra ngày có
// tồn tại hay không, đổi lịch, tuổi… đều hỏi lõi (save_death_json, derive_birth_json, derive_person_json).
import Foundation

// MARK: - Chuỗi (01 §1: NFC, cắt khoảng trắng, bỏ ký tự điều khiển)

enum Text1 {
    static func clean(_ s: String, multiline: Bool = false) -> String {
        let nfc = s.precomposedStringWithCanonicalMapping
        let filtered = String(String.UnicodeScalarView(nfc.unicodeScalars.filter { u in
            if multiline && u == "\n" { return true }
            let v = u.value
            return !(v <= 0x1F || (0x7F...0x9F).contains(v))
        }))
        return filtered.trimmingCharacters(in: .whitespacesAndNewlines)
    }
    static func count(_ s: String) -> Int { clean(s, multiline: true).unicodeScalars.count }
    /// Viết hoa chữ cái đầu, giữ nguyên phần còn lại (cách gọi, quan hệ — YC-2026-07).
    static func capFirst(_ s: String) -> String {
        guard let f = s.first else { return s }
        return String(f).uppercased() + s.dropFirst()
    }
}

/// Lỗi theo trường; giữ thứ tự để khung tóm tắt "Còn n chỗ cần sửa" liệt kê đúng thứ tự biểu mẫu.
struct FieldErrorItem: Equatable, Hashable {
    let key: String
    let message: String
}

struct FieldErrors: Equatable {
    private(set) var items: [FieldErrorItem] = []
    mutating func set(_ key: String, _ message: String) {
        if !items.contains(where: { $0.key == key }) { items.append(FieldErrorItem(key: key, message: message)) }
    }
    subscript(key: String) -> String? { items.first { $0.key == key }?.message }
    var isEmpty: Bool { items.isEmpty }
    var count: Int { items.count }
    mutating func merge(_ o: FieldErrors) { for i in o.items { set(i.key, i.message) } }
}

// MARK: - Ngày

enum CalKind: String, Equatable { case solar, lunar }

enum DatePurpose: Equatable {
    case death, birth, event, notice
}

/// Kết quả dựng DateValue: giá trị, hoặc câu báo lỗi nhập liệu.
enum DVResult: Equatable {
    case success(JV)
    case failure(String)
}

struct DateDraft: Equatable {
    var cal: CalKind = .solar
    var day = ""
    var month = ""
    var year = ""
    var leap = false
    /// Âm lịch không rõ năm (`month_day`).
    var unknownYear = false
    /// Chỉ biết năm (`precision = year`, ngày sinh).
    var yearOnly = false

    init() {}

    init(_ v: JV) {
        guard v.isObject else { return }
        cal = v["calendar"].str == "lunar" ? .lunar : .solar
        day = v["day"].int.map(String.init) ?? ""
        month = v["month"].int.map(String.init) ?? ""
        year = v["year"].int.map(String.init) ?? ""
        leap = v["leap"].isTrue
        unknownYear = v["precision"].str == "month_day"
        yearOnly = v["precision"].str == "year"
    }

    private static func t(_ s: String) -> String { s.trimmingCharacters(in: .whitespaces) }

    var isEmpty: Bool {
        if yearOnly { return Self.t(year).isEmpty }
        return Self.t(day).isEmpty && Self.t(month).isEmpty && (unknownYear || Self.t(year).isEmpty)
    }

    /// Năm đã nhập (số) — dùng để biết năm âm có tháng nhuận không.
    var yearNumber: Int? { unknownYear ? nil : Int(Self.t(year)) }

    /// DateValue (01 §2.1) hoặc câu báo lỗi nhập liệu (thiếu ô, ngoài khoảng). Chưa kiểm ngày có tồn tại.
    func value(_ purpose: DatePurpose) -> DVResult {
        let dS = Self.t(day), mS = Self.t(month), yS = Self.t(year)
        let noun: String
        switch purpose {
        case .death: noun = "Hãy nhập ngày mất"
        case .birth: noun = "Hãy nhập ngày sinh"
        case .event: noun = "Hãy nhập ngày"
        case .notice: noun = "Hãy nhập ngày nhận tin"
        }
        if isEmpty { return .failure(noun) }
        let yr: ClosedRange<Int>
        switch purpose {
        case .death: yr = cal == .solar ? 1900...3000 : 1000...3000
        case .birth: yr = 1000...3000
        case .event, .notice: yr = 1900...3000
        }
        let yrMsg = "Năm phải từ \(yr.lowerBound) đến \(yr.upperBound)"
        if yearOnly && purpose == .birth {
            guard let y = Int(yS) else { return .failure("Năm phải là số") }
            guard yr.contains(y) else { return .failure(yrMsg) }
            return .success(["calendar": .s(cal.rawValue), "precision": "year", "year": .i(y),
                             "month": nil, "day": nil, "leap": false])
        }
        let needYear = !(cal == .lunar && unknownYear)
        if dS.isEmpty || mS.isEmpty || (needYear && yS.isEmpty) {
            return .failure(needYear ? "Hãy nhập đủ ngày, tháng, năm" : "Hãy nhập đủ ngày và tháng âm")
        }
        guard let d = Int(dS), let m = Int(mS) else { return .failure("Ngày và tháng phải là số") }
        guard (1...12).contains(m) else { return .failure("Tháng phải từ 1 đến 12") }
        if cal == .lunar {
            guard (1...30).contains(d) else { return .failure("Ngày âm lịch phải từ 1 đến 30") }
        } else {
            guard (1...31).contains(d) else { return .failure("Ngày phải từ 1 đến 31") }
        }
        if needYear {
            guard let y = Int(yS) else { return .failure("Năm phải là số") }
            guard yr.contains(y) else { return .failure(yrMsg) }
            return .success(["calendar": .s(cal.rawValue), "precision": "day", "year": .i(y), "month": .i(m),
                             "day": .i(d), "leap": .bool(cal == .lunar && leap)])
        }
        return .success(["calendar": "lunar", "precision": "month_day", "year": nil, "month": .i(m), "day": .i(d),
                         "leap": .bool(leap)])
    }
}

/// Kết quả kiểm một ngày qua lõi: DateValue, phần dẫn xuất của lõi, hoặc câu báo lỗi.
struct CheckedDate: Equatable {
    var value: JV = .null
    var derived: JV = .null
    var error: String?
    var ok: Bool { error == nil && value.isObject }
}

enum DateCheck {
    private static var leapCache: [Int: Int?] = [:]

    /// Tháng nhuận của năm âm `y` (nil = không có, hoặc năm ngoài 1899–3000) — `lunar_leap_month` của lõi.
    static func leapMonth(ofLunarYear y: Int) -> Int? {
        if let c = leapCache[y] { return c }
        let r: Int? = (1899...3000).contains(y) ? ((try? lunarLeapMonth(year: Int32(y))) ?? nil).map { Int($0) } : nil
        leapCache[y] = r
        return r
    }

    /// Ngày mất (02 §1): đổi lịch, kiểm tồn tại, không sau hôm nay.
    static func death(_ d: DateDraft, today: CivilDate) -> CheckedDate {
        var r = CheckedDate()
        switch d.value(.death) {
        case .failure(let m): r.error = m; return r
        case .success(let v): r.value = v
        }
        do {
            r.derived = JV.parse(try saveDeathJson(dateValueJson: r.value.text)) ?? .null
        } catch {
            r.error = Core.message(error); return r
        }
        if let s = solar(r.derived["solar"]), s > today { r.error = "Ngày mất không được sau hôm nay" }
        return r
    }

    /// Ngày sinh (02 §4.1).
    static func birth(_ d: DateDraft, today: CivilDate) -> CheckedDate {
        var r = CheckedDate()
        switch d.value(.birth) {
        case .failure(let m): r.error = m; return r
        case .success(let v): r.value = v
        }
        do {
            r.derived = JV.parse(try deriveBirthJson(dateValueJson: r.value.text)) ?? .null
        } catch {
            r.error = Core.message(error); return r
        }
        if let s = solar(r.derived["solar"]), s > today { r.error = "Ngày sinh không được sau hôm nay" }
        else if d.yearOnly, let y = r.value["year"].int, d.cal == .solar, y > today.y { r.error = "Ngày sinh không được sau hôm nay" }
        return r
    }

    /// Ngày sự kiện (có thể ở tương lai) — kiểm tồn tại và đổi lịch bằng cùng hàm đổi của lõi.
    static func event(_ d: DateDraft) -> CheckedDate {
        var r = CheckedDate()
        switch d.value(.event) {
        case .failure(let m): r.error = m; return r
        case .success(let v): r.value = v
        }
        do {
            r.derived = JV.parse(try saveDeathJson(dateValueJson: r.value.text)) ?? .null
        } catch {
            r.error = Core.message(error)
        }
        return r
    }

    /// Ngày nhận tin báo tử: dương lịch, không sau hôm nay.
    static func notice(_ d: DateDraft, today: CivilDate) -> CheckedDate {
        var r = CheckedDate()
        var dd = d
        dd.cal = .solar; dd.unknownYear = false; dd.yearOnly = false
        switch dd.value(.notice) {
        case .failure(let m): r.error = m; return r
        case .success(let v): r.value = v
        }
        do {
            r.derived = JV.parse(try saveDeathJson(dateValueJson: r.value.text)) ?? .null
        } catch {
            r.error = Core.message(error); return r
        }
        if let s = solar(r.derived["solar"]), s > today { r.error = "Ngày nhận tin không được sau hôm nay" }
        return r
    }

    static func solar(_ s: JV) -> CivilDate? {
        guard let y = s["y"].int, let m = s["m"].int, let d = s["d"].int else { return nil }
        return CivilDate(y: y, m: m, d: d)
    }

    /// Dòng kết quả đổi lịch dưới ô ngày (08 §3.4, §3.7).
    static func resultLine(_ c: CheckedDate, purpose: DatePurpose, living: Bool = false, today: CivilDate) -> String? {
        guard c.ok else { return nil }
        let v = c.value, dv = c.derived
        switch purpose {
        case .death, .event, .notice:
            if v["calendar"].str == "solar" {
                if purpose == .notice { return nil }
                let l = dv["lunar"]
                guard let ly = l["year"].int else { return nil }
                return "Âm lịch: \(l["day"].int ?? 0)/\(l["month"].int ?? 0)\(l["leap"].isTrue ? " nhuận" : "") năm \(Lich.canChi(year: ly)) (\(ly))"
            }
            if v["precision"].str == "month_day" {
                if purpose == .event { return "Không rõ năm: nhắc hằng năm vào \(PFmt.lunarText(v))." }
                return "Không rõ năm: vẫn nhắc giỗ hằng năm vào \(PFmt.lunarText(v)), không ghi giỗ đầu, giỗ hết."
            }
            guard let s = solar(dv["solar"]) else { return "Năm này ngoài khoảng đổi lịch — không có ngày dương tương ứng" }
            return "Dương lịch: \(Fmt.weekday[s.weekdayCore]), \(s.dmy)"
        case .birth:
            guard let ly = dv["lunar_year"].int else { return nil }
            let cc = dv["can_chi"].str ?? Lich.canChi(year: ly)
            if living {
                var parts: [String] = []
                let l = dv["lunar"]
                if l.isObject, let ld = l["day"].int, let lm = l["month"].int {
                    parts.append("Âm lịch: \(ld)/\(lm)\(l["leap"].isTrue ? " nhuận" : "") năm \(cc) (\(ly))")
                } else {
                    parts.append(dv["lunar_year_approx"].isTrue ? "Khoảng năm âm \(ly) — \(cc)" : "Năm âm \(ly) — \(cc)")
                }
                let p: JV = ["kind": "living", "danh_xung": "", "ho_ten": "", "birth": derivedBirth(c), "death": nil,
                             "feb29_rule": "feb28"]
                if let j = try? personSummaryJson(personJson: p.text, today: today.sd),
                   let s = Core.decode(PersonSummary.self, j) {
                    if let t = s.tuoi_duong {
                        parts.append("Hôm nay \(t) tuổi" + (s.tuoi_am.map { " (tuổi âm \($0))" } ?? ""))
                    } else if let a = s.tuoi_am {
                        parts.append("Tuổi âm \(a)")
                    }
                }
                return parts.joined(separator: " · ")
            }
            if dv["lunar_year_approx"].isTrue {
                return "Khoảng năm âm \(ly) — \(cc) (nếu sinh trước Tết thì là năm trước)"
            }
            return "Năm âm \(ly) — \(cc)"
        }
    }

    /// BirthInfo đầy đủ (input + phần dẫn xuất), `noi_sinh` trống.
    static func derivedBirth(_ c: CheckedDate, noiSinh: String = "") -> JV {
        var b: JV = ["input": c.value, "noi_sinh": .s(noiSinh)]
        if let o = c.derived.object { for (k, v) in o { b[k] = v } }
        return b
    }
}

// MARK: - Quy tắc nhắc (01 §3.6)

struct RuleDraft: Equatable {
    static let quick = [0, 1, 3, 5, 10, 15]
    var id: String
    var enabled = true
    var offsets: Set<Int> = [0, 3]
    var custom = ""
    var hour = "07"
    var minute = "00"
    var tienThuong = false

    init(id: String? = nil, defaults: JV = .null) {
        self.id = id ?? ((try? FFI.uuid()) ?? UUID().uuidString.lowercased())
        let d = defaults["default_offsets_days"].array.compactMap(\.int)
        if defaults.isObject { offsets = Set(d) }
        if let t = defaults["default_time_local"].str { (hour, minute) = Self.split(t) }
    }

    init(rule: JV, defaults: JV = .null) {
        self.init(id: rule["id"].str, defaults: defaults)
        if rule.isObject {
            enabled = rule["enabled"].isTrue
            offsets = Set(rule["offsets_days"].array.compactMap(\.int))
            (hour, minute) = Self.split(rule["time_local"].string)
            tienThuong = rule["tien_thuong"].isTrue
        }
    }

    static func split(_ t: String) -> (String, String) {
        let p = t.split(separator: ":")
        guard p.count == 2 else { return ("07", "00") }
        return (String(p[0]), String(p[1]))
    }

    /// Mốc nhắc cuối cùng (gồm ô tự đặt nếu hợp lệ).
    var finalOffsets: [Int] {
        var s = offsets
        if let c = Int(custom.trimmingCharacters(in: .whitespaces)), (1...60).contains(c) { s.insert(c) }
        return s.sorted()
    }

    var hhmm: String? {
        guard let h = Int(hour.trimmingCharacters(in: .whitespaces)), let m = Int(minute.trimmingCharacters(in: .whitespaces)),
              (0...23).contains(h), (0...59).contains(m) else { return nil }
        return String(format: "%02d:%02d", h, m)
    }

    func validate(prefix: String) -> FieldErrors {
        var e = FieldErrors()
        guard enabled else { return e }
        let c = custom.trimmingCharacters(in: .whitespaces)
        if !c.isEmpty, Int(c).map({ !(1...60).contains($0) }) ?? true {
            e.set(prefix + ".custom", "Số ngày tự đặt phải từ 1 đến 60")
        }
        if finalOffsets.count > 8 { e.set(prefix + ".offsets", "Tối đa 8 mốc nhắc") }
        if hhmm == nil { e.set(prefix + ".time", "Giờ nhắc phải từ 0 đến 23 giờ, phút từ 0 đến 59") }
        return e
    }

    /// Cảnh báo (vẫn lưu được).
    var warning: String? { enabled && finalOffsets.isEmpty ? "Chưa chọn mốc nhắc nào — sẽ không có lời nhắc" : nil }

    func json(tienThuongAllowed: Bool) -> JV {
        ["id": .s(id), "enabled": .bool(enabled), "offsets_days": .array(finalOffsets.map { JV.i($0) }),
         "time_local": .s(hhmm ?? "07:00"), "tien_thuong": .bool(tienThuongAllowed && tienThuong)]
    }

    /// Cùng cài đặt (bỏ qua id) — cho hộp "giống sinh nhật dương lịch" (TK-29).
    func sameSettings(as o: RuleDraft) -> Bool {
        enabled == o.enabled && finalOffsets == o.finalOffsets && hhmm == o.hhmm
    }
}

// MARK: - Thông tin báo tử (01 §3.2a) và tệp đính kèm (01 §3.8a, YC-2026-07)

/// Loại tệp đính kèm.
enum AttKind: String, CaseIterable {
    case tinBaoTu = "tin_bao_tu", giayChungTu = "giay_chung_tu", khac = "khac"
    var title: String {
        switch self {
        case .tinBaoTu: return "Tin báo tử"
        case .giayChungTu: return "Giấy chứng tử"
        case .khac: return "Tệp khác"
        }
    }
    static func title(_ code: String) -> String { AttKind(rawValue: code)?.title ?? "Tệp khác" }
}

/// Tệp mới chọn, chưa ghi vào kho: ảnh đã mã hóa lại (S9) hoặc PDF nguyên bản.
struct NewAttachment: Equatable {
    let data: Data
    let mime: String
    let fileName: String?
    var isPdf: Bool { mime == "application/pdf" }
}

/// Một mục trong `death.notice.attachments`.
struct AttDraft: Equatable, Identifiable {
    var id: String
    var kind: String
    /// Id tệp trong kho (đã có, hoặc vừa ghi).
    var fileId: String?
    /// Tệp mới chưa ghi.
    var new: NewAttachment?

    static func make(_ n: NewAttachment, kind: AttKind) -> AttDraft {
        AttDraft(id: (try? FFI.uuid()) ?? UUID().uuidString.lowercased(), kind: kind.rawValue, fileId: nil, new: n)
    }
}

struct NoticeDraft: Equatable {
    static let maxAttachments = 6
    var reportedBy = ""
    var place = ""
    var reason = ""
    var note = ""
    var date = DateDraft()
    var attachments: [AttDraft] = []

    init() {}
    init(_ n: JV) {
        guard n.isObject else { return }
        reportedBy = n["reported_by"].string
        place = n["place"].string
        reason = n["reason"].string
        note = n["note"].string
        date = DateDraft(n["notice_date"])
        attachments = n["attachments"].array.compactMap { a in
            guard let id = a["id"].str, let f = a["file_id"].str else { return nil }
            return AttDraft(id: id, kind: a["kind"].str ?? "khac", fileId: f, new: nil)
        }
    }

    var attachmentCount: Int { attachments.count }

    var isEmpty: Bool {
        Text1.clean(reportedBy).isEmpty && Text1.clean(place).isEmpty && Text1.clean(reason).isEmpty
            && Text1.clean(note).isEmpty && date.isEmpty && attachments.isEmpty
    }

    func validate(today: CivilDate) -> FieldErrors {
        var e = FieldErrors()
        if Text1.count(reportedBy) > 200 { e.set("notice.reported_by", "Người báo tin tối đa 200 ký tự") }
        if Text1.count(place) > 200 { e.set("notice.place", "Nơi mất tối đa 200 ký tự") }
        if !date.isEmpty, let m = DateCheck.notice(date, today: today).error { e.set("notice.date", m) }
        if Text1.count(reason) > 500 { e.set("notice.reason", "Lý do tối đa 500 ký tự") }
        if attachments.count > Self.maxAttachments { e.set("notice.attachments", "Tối đa 6 tệp đính kèm") }
        if Text1.count(note) > 2000 { e.set("notice.note", "Ghi chú tối đa 2000 ký tự") }
        return e
    }

    /// JSON `DeathNotice` (chỉ các tệp đã có trong kho), hoặc nil khi không có gì.
    func json(today: CivilDate) -> JV? {
        if isEmpty { return nil }
        let opt = { (s: String, ml: Bool) -> JV in let c = Text1.clean(s, multiline: ml); return c.isEmpty ? .null : .s(c) }
        let nd = date.isEmpty ? JV.null : DateCheck.notice(date, today: today).value
        let atts: [JV] = attachments.compactMap { a in
            guard let f = a.fileId else { return nil }
            return ["id": .s(a.id), "kind": .s(a.kind), "file_id": .s(f)]
        }
        return ["reported_by": opt(reportedBy, false), "place": opt(place, false), "reason": opt(reason, true),
                "notice_date": nd, "attachments": .array(atts), "note": opt(note, true)]
    }

    /// Mô tả ngắn cho hộp xác nhận (không đọc ra nội dung).
    var summaryLine: String? {
        isEmpty ? nil : "Lưu thông tin báo tử\(attachments.isEmpty ? "" : " (gồm \(attachments.count) tệp đính kèm)") — chỉ hiện trên trang hồ sơ."
    }
}

// MARK: - Hồ sơ người

struct PersonDraft: Equatable {
    var kind: String
    var base: JV
    var danhXung = ""
    var hoTen = ""
    var tenHieu = ""
    var quanHe = ""
    var ghiChu = ""
    var death = DateDraft()
    var birth = DateDraft()
    var noiSinh = ""
    var noiAnTang = ""
    var tuoiManual = false
    var tuoiText = ""
    var gioHetNam = 2
    var loiTuongNiem = ""
    var nguong = "60"
    var gio: RuleDraft
    var sinhNhat: RuleDraft
    var sinhNhatAm: RuleDraft
    var sinhNhatAmOn = false
    var amSame = true
    var feb29 = "feb28"
    var photoId: String?
    var newPhoto: ProcessedPhoto?
    var notice = NoticeDraft()
    /// Giới tính (YC-2026-07): "nam" | "nu" | nil (Không ghi).
    var gioiTinh: String?

    var deceased: Bool { kind == "deceased" }
    var isNew: Bool { base["id"].str == nil }

    init(kind: String, settings: JV) {
        self.kind = kind
        base = .null
        gio = RuleDraft(defaults: settings)
        sinhNhat = RuleDraft(defaults: settings)
        sinhNhatAm = RuleDraft(defaults: settings)
    }

    init(person p: JV, settings: JV) {
        kind = p["kind"].str ?? "living"
        base = p
        danhXung = p["danh_xung"].string
        hoTen = p["ho_ten"].string
        tenHieu = p["ten_hieu"].string
        quanHe = p["quan_he"].string
        ghiChu = p["ghi_chu"].string
        death = DateDraft(p["death"]["input"])
        birth = DateDraft(p["birth"]["input"])
        noiSinh = p["birth"]["noi_sinh"].string
        noiAnTang = p["noi_an_tang"].string
        tuoiManual = p["death"]["tuoi"]["manual"].isTrue
        tuoiText = p["death"]["tuoi"]["value"].int.map(String.init) ?? ""
        gioHetNam = p["memorial"]["gio_het_nam"].int ?? 2
        loiTuongNiem = p["memorial"]["loi_tuong_niem"].string
        nguong = String(p["memorial"]["huong_duong_duoi"].int ?? 60)
        gio = RuleDraft(rule: p["reminders"]["gio"], defaults: settings)
        sinhNhat = RuleDraft(rule: p["reminders"]["sinh_nhat"], defaults: settings)
        sinhNhatAm = RuleDraft(rule: p["reminders"]["sinh_nhat_am"], defaults: settings)
        sinhNhatAmOn = p["sinh_nhat_am"].isTrue
        amSame = !p["reminders"]["sinh_nhat_am"].isObject || sinhNhatAm.sameSettings(as: sinhNhat)
        feb29 = p["feb29_rule"].str ?? "feb28"
        photoId = p["photo_id"].str
        notice = NoticeDraft(p["death"]["notice"])
        gioiTinh = p["gioi_tinh"].str
    }

    var birthFullDay: Bool { !birth.isEmpty && !birth.yearOnly }
    var isFeb29: Bool {
        birth.cal == .solar && !birth.yearOnly && Int(birth.day.trimmingCharacters(in: .whitespaces)) == 29
            && Int(birth.month.trimmingCharacters(in: .whitespaces)) == 2
    }

    func validate(today: CivilDate) -> FieldErrors {
        var e = FieldErrors()
        if Text1.count(danhXung) > 40 { e.set("danh_xung", "Cách gọi tối đa 40 ký tự") }
        let ten = Text1.clean(hoTen)
        if ten.isEmpty { e.set("ho_ten", "Họ tên không được để trống") }
        else if ten.unicodeScalars.count > 120 { e.set("ho_ten", "Họ tên tối đa 120 ký tự") }
        if Text1.count(tenHieu) > 120 { e.set("ten_hieu", "Tên hiệu tối đa 120 ký tự") }
        if Text1.count(quanHe) > 120 { e.set("quan_he", "Quan hệ tối đa 120 ký tự") }
        if Text1.count(ghiChu) > 2000 { e.set("ghi_chu", "Ghi chú tối đa 2000 ký tự") }
        var deathC = CheckedDate()
        if deceased {
            deathC = DateCheck.death(death, today: today)
            if let m = deathC.error { e.set("death", m) }
        }
        var birthC = CheckedDate()
        if !birth.isEmpty {
            birthC = DateCheck.birth(birth, today: today)
            if let m = birthC.error { e.set("birth", m) }
        }
        if Text1.count(noiSinh) > 300 { e.set("noi_sinh", "Nơi sinh tối đa 300 ký tự") }
        if deceased && deathC.ok && birthC.ok,
           let dy = deathC.derived["lunar"]["year"].int, let by = birthC.derived["lunar_year"].int {
            if dy < by { e.set("death", "Ngày mất trước ngày sinh") }
            else if dy - by + 1 > 150 { e.set("death", "Tuổi tính ra lớn hơn 150 — hãy kiểm tra năm sinh và năm mất") }
        }
        if deceased {
            if tuoiManual {
                if let v = Int(tuoiText.trimmingCharacters(in: .whitespaces)), (1...150).contains(v) {} else {
                    e.set("tuoi", "Tuổi phải từ 1 đến 150")
                }
            }
            if Text1.count(noiAnTang) > 300 { e.set("noi_an_tang", "Nơi an táng tối đa 300 ký tự") }
            e.merge(notice.validate(today: today))
            e.merge(gio.validate(prefix: "gio"))
            if Text1.count(loiTuongNiem) > 2000 { e.set("loi_tuong_niem", "Lời tưởng niệm tối đa 2000 ký tự") }
            if let n = Int(nguong.trimmingCharacters(in: .whitespaces)), (1...150).contains(n) {} else {
                e.set("nguong", "Số tuổi phải từ 1 đến 150")
            }
        } else {
            e.merge(sinhNhat.validate(prefix: "sinh_nhat"))
            if sinhNhatAmOn && !amSame { e.merge(sinhNhatAm.validate(prefix: "sinh_nhat_am")) }
        }
        return e
    }

    /// Person JSON để gửi `save_person_json` (lõi tính phần dẫn xuất, gắn id/rev).
    func build(photoId pid: String?, today: CivilDate) -> JV {
        var p = base.isObject ? base : JV.object([:])
        p["kind"] = .s(kind)
        p["danh_xung"] = .s(Text1.capFirst(Text1.clean(danhXung)))
        p["ho_ten"] = .s(Text1.clean(hoTen))
        p["ten_hieu"] = .s(Text1.clean(tenHieu))
        p["quan_he"] = .s(Text1.capFirst(Text1.clean(quanHe)))
        p["ghi_chu"] = .s(Text1.clean(ghiChu, multiline: true))
        p["photo_id"] = .opt(pid)
        p["gioi_tinh"] = .opt(gioiTinh)
        p["feb29_rule"] = .s(isFeb29 ? feb29 : (base["feb29_rule"].str ?? "feb28"))
        if birth.isEmpty {
            p["birth"] = .null
        } else {
            let c = DateCheck.birth(birth, today: today)
            p["birth"] = ["input": c.value, "noi_sinh": .s(Text1.clean(noiSinh))]
        }
        if deceased {
            let c = DateCheck.death(death, today: today)
            var d: JV = ["input": c.value]
            if tuoiManual, let v = Int(tuoiText.trimmingCharacters(in: .whitespaces)) {
                let ng = Int(nguong) ?? 60
                d["tuoi"] = ["value": .i(v), "label": .s(v < ng ? "huong_duong" : "huong_tho"), "manual": true, "approx": false]
            }
            if let n = notice.json(today: today) { d["notice"] = n }
            p["death"] = d
            p["noi_an_tang"] = .s(Text1.clean(noiAnTang))
            var mem: JV = base["memorial"].isObject ? base["memorial"]
                : ["tray_style": "truyen_thong", "incense": true, "bia_tieu_de": ""]
            mem["loi_tuong_niem"] = .s(Text1.clean(loiTuongNiem, multiline: true))
            mem["gio_het_nam"] = .i(gioHetNam)
            mem["huong_duong_duoi"] = .i(Int(nguong.trimmingCharacters(in: .whitespaces)) ?? 60)
            p["memorial"] = mem
            p["reminders"] = ["gio": gio.json(tienThuongAllowed: true), "sinh_nhat": nil, "sinh_nhat_am": nil]
            p["sinh_nhat_am"] = false
        } else {
            p["death"] = .null
            p["memorial"] = .null
            p["noi_an_tang"] = ""
            let amOn = sinhNhatAmOn && birthFullDay
            var am = sinhNhatAm
            if amSame { am.enabled = sinhNhat.enabled; am.offsets = Set(sinhNhat.finalOffsets); am.custom = ""
                am.hour = sinhNhat.hour; am.minute = sinhNhat.minute }
            p["reminders"] = ["gio": nil, "sinh_nhat": sinhNhat.json(tienThuongAllowed: false),
                              "sinh_nhat_am": amOn ? am.json(tienThuongAllowed: false) : nil]
            p["sinh_nhat_am"] = .bool(amOn)
        }
        return p
    }

    /// Dòng "Hưởng thọ 86 tuổi" xem trước (lõi tính), hoặc lời nhắc thiếu năm sinh.
    func ageLine(today: CivilDate) -> String? {
        guard deceased else { return nil }
        let d = DateCheck.death(death, today: today)
        guard d.ok else { return nil }
        let p = build(photoId: nil, today: today)
        var noNotice = p
        noNotice["death"] = p["death"].removing("notice")
        guard let j = try? derivePersonJson(json: noNotice.text), let dp = JV.parse(j),
              let sj = try? personSummaryJson(personJson: dp.text, today: today.sd),
              let s = Core.decode(PersonSummary.self, sj) else { return nil }
        return s.tuoi_mat_text ?? "Chưa đủ năm sinh để tính tuổi"
    }
}

// MARK: - Sự kiện (01 §3.4)

struct EventDraft: Equatable {
    static let newType = "__new__"
    var base: JV
    var personId: String
    var typeId: String
    var newTypeName = ""
    var title = ""
    var date = DateDraft()
    var recurring = false
    var rule: RuleDraft
    var note = ""

    init(personId: String, types: [JV], settings: JV) {
        base = .null
        self.personId = personId
        typeId = types.first?["id"].str ?? Self.newType
        recurring = types.first?["default_recurring"].isTrue ?? false
        rule = RuleDraft(defaults: settings)
    }

    init(event e: JV, settings: JV) {
        base = e
        personId = e["person_ids"][0].str ?? ""
        typeId = e["type_id"].string
        title = e["tieu_de"].string
        date = DateDraft(e["date"])
        recurring = e["recurring"].isTrue
        rule = RuleDraft(rule: e["reminder"], defaults: settings)
        if !e["reminder"].isObject { rule.enabled = false }
        note = e["ghi_chu"].string
    }

    func validate(personDeceased: Bool) -> FieldErrors {
        var e = FieldErrors()
        if typeId == Self.newType {
            let n = Text1.clean(newTypeName)
            if n.isEmpty { e.set("type", "Hãy đặt tên cho loại sự kiện") }
            else if n.unicodeScalars.count > 60 { e.set("type", "Tên loại tối đa 60 ký tự") }
        }
        if Text1.count(title) > 120 { e.set("tieu_de", "Tiêu đề tối đa 120 ký tự") }
        var dd = date
        if !(dd.cal == .lunar && recurring) { dd.unknownYear = false }
        let c = DateCheck.event(dd)
        if let m = c.error { e.set("date", m) }
        if date.cal == .lunar && date.unknownYear && !recurring { e.set("date", "Sự kiện không rõ năm phải lặp hằng năm") }
        if !personDeceased { e.merge(rule.validate(prefix: "rule")) }
        if Text1.count(note) > 2000 { e.set("ghi_chu", "Ghi chú tối đa 2000 ký tự") }
        return e
    }

    func build(typeId tid: String, personDeceased: Bool) -> JV {
        var e = base.isObject ? base : JV.object([:])
        var dd = date
        if !(dd.cal == .lunar && recurring) { dd.unknownYear = false }
        e["type_id"] = .s(tid)
        e["tieu_de"] = .s(Text1.clean(title))
        e["date"] = DateCheck.event(dd).value
        e["recurring"] = .bool(recurring || (dd.cal == .lunar && dd.unknownYear))
        if !base.isObject { e["person_ids"] = [.s(personId)] }
        e["ghi_chu"] = .s(Text1.clean(note, multiline: true))
        if personDeceased {
            // B4: không nhắc sự kiện của người đã mất; quy tắc đang lưu (từ sao lưu) giữ nguyên.
            if !base.isObject { e["reminder"] = nil }
        } else {
            e["reminder"] = rule.enabled ? rule.json(tienThuongAllowed: false) : nil
        }
        return e
    }
}
