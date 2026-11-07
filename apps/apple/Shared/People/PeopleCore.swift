// Lớp nối giữa giao diện "Người thân" và lõi Rust (UniFFI, Generated/LvhvCore.swift).
// Mọi tính toán (đổi lịch, lần xuất hiện, nhắc, .ics, sao lưu) do lõi làm; tệp này chỉ gọi lõi, giữ trạng thái
// để hiển thị và lập lịch lại thông báo sau mỗi lần ghi (02 §7.4).
import Foundation
import SwiftUI

// MARK: - Hàm lõi gọi qua tên riêng (tránh trùng tên với thành viên của kiểu)

enum FFI {
    static func weekdayOf(_ d: CivilDate) -> Int? {
        (try? weekday(y: Int32(d.y), m: Int32(d.m), d: Int32(d.d))).map { Int($0) }
    }
    static func ics(datasetJson: String, today: CivilDate, now: Date) throws -> IcsExport {
        try icsExport(datasetJson: datasetJson, today: today.sd, nowUnixS: Int64(now.timeIntervalSince1970), onlyIds: nil)
    }
    static func uuid() throws -> String { try newUuid() }
}

// MARK: - Ngày

extension CivilDate {
    init(_ s: SolarDay) { self.init(y: Int(s.y), m: Int(s.m), d: Int(s.d)) }
    var sd: SolarDay { SolarDay(y: Int32(y), m: Int32(m), d: Int32(d)) }
    /// "YYYY-MM-DD".
    var iso: String { String(format: "%04d-%02d-%02d", y, m, d) }
    init?(iso: String) {
        let p = iso.prefix(10).split(separator: "-").compactMap { Int($0) }
        guard p.count == 3 else { return nil }
        self.init(y: p[0], m: p[1], d: p[2])
    }
    /// Thứ trong tuần theo lõi (0 = Chủ nhật); lõi ngoài miền → công thức JDN.
    var weekdayCore: Int {
        FFI.weekdayOf(self) ?? weekday
    }
}

/// "Hôm nay" của mô-đun. Bản Debug cho phép kiểm thử giao diện đặt ngày cố định (`-lvhvToday YYYY-MM-DD`).
enum PeopleClock {
    static func today() -> CivilDate {
        #if DEBUG
        if let s = UserDefaults.standard.string(forKey: "lvhvToday"), let d = CivilDate(iso: s) { return d }
        #endif
        return CivilDate.today()
    }
    static func now() -> Date {
        #if DEBUG
        if let s = UserDefaults.standard.string(forKey: "lvhvToday"), let d = CivilDate(iso: s) {
            var c = DateComponents(); c.year = d.y; c.month = d.m; c.day = d.d; c.hour = 6; c.minute = 0
            return Calendar(identifier: .gregorian).date(from: c) ?? Date()
        }
        #endif
        return Date()
    }
}

// MARK: - Thư mục dữ liệu (01 §4.2, S5)

enum PeoplePaths {
    /// `Application Support/LichVanHoaViet/` (trong hộp cát của ứng dụng).
    static func storeDir() -> URL {
        #if DEBUG
        if let s = UserDefaults.standard.string(forKey: "lvhvStoreDir"), !s.isEmpty {
            return URL(fileURLWithPath: s, isDirectory: true)
        }
        #endif
        let base = (try? FileManager.default.url(for: .applicationSupportDirectory, in: .userDomainMask,
                                                  appropriateFor: nil, create: true))
            ?? URL(fileURLWithPath: NSTemporaryDirectory(), isDirectory: true)
        return base.appendingPathComponent("LichVanHoaViet", isDirectory: true)
    }

    static var databaseURL: URL { storeDir().appendingPathComponent("lvhv.sqlite") }
    static var storeExists: Bool { FileManager.default.fileExists(atPath: databaseURL.path) }

    /// Đánh dấu không sao lưu tự động lên đám mây (iCloud / bản sao lưu máy, Time Machine): thư mục kho, tệp
    /// SQLite (kể cả -wal, -shm) và thư mục ảnh (01 §4.2, S5). Trả danh sách đường dẫn đã đánh dấu.
    @discardableResult
    static func excludeFromBackup(_ dir: URL) -> [URL] {
        var done: [URL] = []
        let names = ["", "lvhv.sqlite", "lvhv.sqlite-wal", "lvhv.sqlite-shm", "photos"]
        for n in names {
            var u = n.isEmpty ? dir : dir.appendingPathComponent(n)
            guard FileManager.default.fileExists(atPath: u.path) else { continue }
            var v = URLResourceValues()
            v.isExcludedFromBackup = true
            if (try? u.setResourceValues(v)) != nil { done.append(u) }
        }
        protectPhotos(dir)
        return done
    }

    /// Ảnh (và sau này tệp đính kèm) chỉ đọc khi ứng dụng đang mở → lớp bảo vệ `.complete` (BM2-20). Giữ mặc định cho
    /// `lvhv.sqlite` vì lập lịch lại thông báo có thể chạy khi máy còn khóa.
    static func protectPhotos(_ dir: URL) {
        #if os(iOS)
        let fm = FileManager.default
        for sub in ["photos", "attachments"] {
            let d = dir.appendingPathComponent(sub, isDirectory: true)
            guard fm.fileExists(atPath: d.path) else { continue }
            try? fm.setAttributes([.protectionKey: FileProtectionType.complete], ofItemAtPath: d.path)
            for f in (try? fm.contentsOfDirectory(atPath: d.path)) ?? [] {
                try? fm.setAttributes([.protectionKey: FileProtectionType.complete],
                                      ofItemAtPath: d.appendingPathComponent(f).path)
            }
        }
        #endif
    }
}

// MARK: - Dữ liệu hiển thị (Decodable từ JSON của lõi, docs/v0.2/schema/ffi-v02-output.schema.json)

struct LunarYMD: Decodable, Hashable { let y: Int; let m: Int; let d: Int; let leap: Bool }

/// Một lần xuất hiện (Sắp tới, Theo kỳ, Lần tới).
struct OccItem: Decodable, Hashable, Identifiable {
    let date: String
    let lunar: LunarYMD?
    let `class`: String
    let class_text: String
    let entity_id: String
    let person_ids: [String]
    let name: String
    let key: String
    let n: Int?
    let label: String?
    let headline: String
    let date_line: String
    let days: Int?
    let days_text: String?

    var id: String { "\(self.class)|\(entity_id)|\(key)|\(date)" }
    var civil: CivilDate { CivilDate(iso: date) ?? CivilDate(y: 2000, m: 1, d: 1) }
    var isGio: Bool { self.class == "gio" || self.class == "tien_thuong" }
    /// Dòng 1 của mục: "Giỗ đầu · Ông Nguyễn Văn A" (08 §2) — chèn "·" trước tên trong tiêu đề của lõi.
    var line1: String {
        guard !isEvent, !name.isEmpty, let r = headline.range(of: " " + name) else { return headline }
        return headline.replacingCharacters(in: r, with: " · " + name)
    }
    var isEvent: Bool { self.class.hasPrefix("su_kien") }
    /// Hồ sơ mở ra khi chạm (sự kiện → người đầu tiên).
    var personId: String? { isEvent ? person_ids.first : entity_id }
    /// "26/11/2026 · 7/10 âm lịch".
    var dateText: String {
        var s = civil.dmy
        if let l = lunar { s += " · \(l.d)/\(l.m)\(l.leap ? " nhuận" : "") âm lịch" }
        return s
    }
}

struct PeriodResult: Decodable {
    let kind: String
    let anchor: String
    let filter: String
    let from: String
    let to: String
    let items: [OccItem]
}

struct PersonSummary: Decodable, Hashable {
    let display_name: String
    let can_chi: String?
    let lunar_year: Int?
    let tuoi_duong: Int?
    let tuoi_am: Int?
    let tuoi_mat_text: String?
    let death_lunar_text: String?
}

/// Lời nhắc trong ứng dụng (thẻ "hôm nay"), có tên (02 §7.1; O-9 chỉ áp cho màn hình khóa).
struct InAppReminder: Decodable, Hashable, Identifiable {
    let id: String
    let at_unix_s: Int64
    let fire_local: String
    let type: String
    let title: String
    let body: String
    let occ: OccItem
}

enum Core {
    static let decoder = JSONDecoder()
    static func decode<T: Decodable>(_ t: T.Type, _ json: String) -> T? {
        try? decoder.decode(t, from: Data(json.utf8))
    }
    /// Thông điệp tiếng Việt của lỗi lõi.
    static func message(_ e: Error) -> String {
        if let d = e as? LvhvDataError {
            switch d {
            case .Invalid(let m), .NotFound(let m), .Storage(let m), .Backup(let m): return m
            }
        }
        if let d = e as? LvhvError { return Lich.message(d) }
        return "Đã có lỗi: \(e.localizedDescription)"
    }
    static var appVersion: String {
        (Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String) ?? "0.0.0"
    }
    static var platform: String {
        #if os(macOS)
        return "macos"
        #else
        return "ios"
        #endif
    }
}

// MARK: - Định dạng (chỉ ghép chữ từ giá trị lõi đã tính)

enum PFmt {
    static let roman = ["", "I", "II", "III", "IV"]

    /// "Tháng 11/2026 · 01/11 – 30/11" … (08 §3.2).
    static func periodTitle(kind: String, from: CivilDate, to: CivilDate) -> String {
        let dm = { (d: CivilDate) in String(format: "%02d/%02d", d.d, d.m) }
        switch kind {
        case "tuan":
            if from.y == to.y { return "Tuần \(dm(from)) – \(dm(to))/\(to.y)" }
            return "Tuần \(from.dmy) – \(to.dmy)"
        case "thang": return "Tháng \(from.m)/\(from.y) · \(dm(from)) – \(dm(to))"
        case "quy": return "Quý \(roman[(from.m - 1) / 3 + 1])/\(from.y) · \(dm(from)) – \(dm(to))"
        default: return "Năm \(from.y) · \(dm(from)) – \(dm(to))"
        }
    }

    static func periodNoun(_ kind: String) -> String {
        switch kind {
        case "tuan": return "tuần này"
        case "thang": return "tháng này"
        case "quy": return "quý này"
        default: return "năm này"
        }
    }

    /// Hôm nay · Ngày mai · Còn n ngày · Hôm qua · Đã qua n ngày (08 §5.1).
    static func distance(_ days: Int) -> String {
        switch days {
        case 0: return "Hôm nay"
        case 1: return "Ngày mai"
        case -1: return "Hôm qua"
        case let n where n > 1: return "Còn \(n) ngày"
        default: return "Đã qua \(-days) ngày"
        }
    }

    /// Ngày giỗ hằng năm: không ghi "nhuận" vì giỗ cúng vào tháng thường (O-4, TKN-I-04).
    static func gioText(_ l: JV) -> String {
        guard let m = l["month"].int, let d = l["day"].int else { return "" }
        return "\(d)/\(m) âm lịch"
    }

    static func lunarText(_ l: JV) -> String {
        guard let m = l["month"].int, let d = l["day"].int else { return "" }
        return "\(d)/\(m)\(l["leap"].isTrue ? " nhuận" : "") âm lịch"
    }

    /// Ngày (DateValue) như người dùng nhập: "08/08/2025", "15/6 nhuận âm lịch năm 2025", "Năm 1940".
    static func dateValue(_ v: JV) -> String {
        let cal = v["calendar"].string, pr = v["precision"].string
        let y = v["year"].int, m = v["month"].int, d = v["day"].int
        if pr == "year", let y { return cal == "lunar" ? "Năm âm \(y)" : "Năm \(y)" }
        if cal == "lunar" {
            let base = "\(d ?? 0)/\(m ?? 0)\(v["leap"].isTrue ? " nhuận" : "") âm lịch"
            return y.map { "\(base) năm \(Lich.canChi(year: $0)) (\($0))" } ?? "\(base) (không rõ năm)"
        }
        return CivilDate(y: y ?? 0, m: m ?? 0, d: d ?? 0).dmy
    }

    static func offsets(_ rule: JV) -> String {
        let o = rule["offsets_days"].array.compactMap(\.int)
        if o.isEmpty { return "Chưa chọn mốc nhắc" }
        return o.map { $0 == 0 ? "Đúng ngày" : "\($0) ngày trước" }.joined(separator: ", ")
    }

    static func ruleSummary(_ rule: JV, tienThuong: Bool) -> String {
        guard rule.isObject else { return "Không nhắc" }
        if !rule["enabled"].isTrue { return "Đã tắt nhắc" }
        var s = "\(offsets(rule)) lúc \(rule["time_local"].string)"
        if tienThuong { s += " · Tiên thường (lễ cúng hôm trước ngày giỗ): \(rule["tien_thuong"].isTrue ? "bật" : "tắt")" }
        return s
    }

    static func displayName(_ p: JV) -> String {
        let dx = Text1.capFirst(p["danh_xung"].string), ten = p["ho_ten"].string
        return dx.isEmpty ? ten : "\(dx) \(ten)"
    }

    /// Dòng quan hệ trên bia: viết hoa chữ đầu, bỏ phần trong ngoặc (08 §3.9).
    static func steleRelation(_ q: String) -> String {
        var s = q
        if let i = s.firstIndex(of: "(") { s = String(s[..<i]) }
        s = s.trimmingCharacters(in: .whitespaces)
        guard let f = s.first else { return "" }
        return f.uppercased() + s.dropFirst()
    }

    static func initials(_ name: String) -> String {
        let w = name.split(separator: " ")
        return w.last.map { String($0.prefix(1)).uppercased() } ?? "?"
    }

    static func rfcToLocalDate(_ s: String) -> String? {
        let f = ISO8601DateFormatter()
        guard let d = f.date(from: s) else { return nil }
        let c = Calendar(identifier: .gregorian).dateComponents(in: .current, from: d)
        return CivilDate(y: c.year ?? 0, m: c.month ?? 0, d: c.day ?? 0).dmy
    }
}

// MARK: - Mô hình của mô-đun

@MainActor
final class PeopleModel: ObservableObject {
    static let shared = PeopleModel()

    @Published private(set) var opened = false
    @Published private(set) var openError: String?
    @Published private(set) var persons: [JV] = []
    @Published private(set) var events: [JV] = []
    @Published private(set) var eventTypes: [JV] = []
    @Published private(set) var settings: JV = .null
    @Published private(set) var upcoming: [OccItem] = []
    /// Lần xuất hiện trong ~13 tháng tới (Lần tới, tóm tắt danh sách hồ sơ, ngày giỗ tới).
    @Published private(set) var ahead: [OccItem] = []
    @Published private(set) var summaries: [String: PersonSummary] = [:]
    @Published private(set) var todayReminders: [InAppReminder] = []
    @Published private(set) var showNames = false
    @Published private(set) var backupDue = false
    @Published var toast: String?
    /// Tăng mỗi lần dữ liệu đổi (để màn hình "Theo kỳ" tính lại).
    @Published private(set) var revision = 0

    private(set) var store: LvhvStore?
    var today: CivilDate { PeopleClock.today() }
    lazy var scheduler = ReminderScheduler(center: SystemNotificationCenter())

    /// Mở kho (tạo nếu chưa có). Gọi khi mở mô-đun, hoặc khi khởi động mà kho đã có (để lập lịch lại).
    func openIfNeeded() {
        guard store == nil else { return }
        let dir = PeoplePaths.storeDir()
        do {
            try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
            #if DEBUG
            DemoSeed.prepareStoreErrorIfRequested(dir)
            #endif
            store = try LvhvStore.open(dir: dir.path)
            PeoplePaths.excludeFromBackup(dir)
            opened = true
            openError = nil
            reload()
        } catch {
            openError = Core.message(error)
        }
    }

    func reload() {
        guard let st = store else { return }
        do {
            persons = JV.parse(try st.personsJson())?.array ?? []
            events = JV.parse(try st.eventsJson())?.array ?? []
            eventTypes = JV.parse(try st.eventTypesJson())?.array ?? []
            settings = JV.parse(try st.settingsJson()) ?? .null
            showNames = (try? st.lockScreenNames()) ?? false
            backupDue = (try? st.backupReminderDue()) ?? false
            let t = today
            let days = UInt32(max(settings["upcoming_days"].int ?? 30, 1))
            upcoming = Core.decode([OccItem].self, try st.upcomingJson(today: t.sd, days: days)) ?? []
            ahead = Core.decode([OccItem].self, try st.upcomingJson(today: t.sd, days: 400)) ?? []
            var sums: [String: PersonSummary] = [:]
            for p in persons {
                if let id = p["id"].str, let j = try? personSummaryJson(personJson: p.text, today: t.sd),
                   let s = Core.decode(PersonSummary.self, j) { sums[id] = s }
            }
            summaries = sums
            todayReminders = loadTodayReminders()
            #if DEBUG
            if UserDefaults.standard.bool(forKey: "lvhvNagDue") { backupDue = true }
            #endif
            revision += 1
        } catch {
            openError = Core.message(error)
        }
    }

    /// Lời nhắc có giờ nhắc rơi vào hôm nay (thẻ trong ứng dụng, 08 §5.5). Văn bản có tên — chỉ hiện trong ứng dụng.
    private func loadTodayReminders() -> [InAppReminder] {
        guard let st = store, let ds = try? st.datasetJson() else { return [] }
        let t = today
        var c = DateComponents(); c.year = t.y; c.month = t.m; c.day = t.d
        let midnight = Calendar(identifier: .gregorian).date(from: c) ?? Date()
        let now = Int64(midnight.timeIntervalSince1970) - 60
        guard let j = try? remindersJson(datasetJson: ds, nowUnixS: now, zone: DeviceZones.current(at: midnight),
                                         horizonDays: 2, showNames: true),
              let list = Core.decode([InAppReminder].self, j) else { return [] }
        return list.filter { $0.fire_local.hasPrefix(t.iso) }
    }

    // MARK: Tra cứu

    func person(_ id: String) -> JV? { persons.first { $0["id"].str == id } }
    func events(of id: String) -> [JV] {
        events.filter { $0["person_ids"].array.contains(.string(id)) }
    }
    func typeName(_ id: String) -> String {
        eventTypes.first { $0["id"].str == id }?["ten"].str ?? "Sự kiện"
    }
    func eventTitle(_ e: JV) -> String {
        let t = e["tieu_de"].string
        return t.isEmpty ? typeName(e["type_id"].string) : t
    }
    /// Lần xuất hiện sắp tới của một người (tối đa `limit`, trong 12 tháng — TK-34).
    func next(for id: String, limit: Int = 5) -> [OccItem] {
        let horizon = today.adding(366)
        let mine = ahead.filter { ($0.entity_id == id || $0.person_ids.contains(id)) && $0.civil <= horizon }
        return Array(mine.prefix(limit))
    }
    func nextGio(for id: String) -> OccItem? {
        ahead.first { $0.class == "gio" && $0.entity_id == id }
    }
    var deceased: [JV] { sorted(persons.filter { $0["kind"].str == "deceased" }) }
    var living: [JV] { sorted(persons.filter { $0["kind"].str == "living" }) }

    /// Theo tên (chữ cuối của họ tên) rồi cả họ tên, so sánh tiếng Việt (TK-30).
    private func sorted(_ list: [JV]) -> [JV] {
        let vi = Locale(identifier: "vi")
        func key(_ p: JV) -> (String, String) {
            let n = p["ho_ten"].string
            return (String(n.split(separator: " ").last ?? ""), n)
        }
        return list.sorted { a, b in
            let (la, fa) = key(a), (lb, fb) = key(b)
            let c = la.compare(lb, options: [.caseInsensitive], range: nil, locale: vi)
            if c != .orderedSame { return c == .orderedAscending }
            return fa.compare(fb, options: [.caseInsensitive], range: nil, locale: vi) == .orderedAscending
        }
    }

    // MARK: Ghi (mỗi lần ghi: tải lại + lập lịch lại thông báo)

    private func after(_ message: String?) {
        reload()
        if let message { toast = message }
        Task { await scheduler.reschedule(store: store, reason: .save) }
    }

    func savePerson(_ p: JV) throws -> JV {
        guard let st = store else { throw LvhvDataError.Storage(message: "Chưa mở được dữ liệu trên máy.") }
        let saved = JV.parse(try st.savePersonJson(json: p.text)) ?? .null
        after("Đã lưu hồ sơ.")
        return saved
    }

    func markDeceased(id: String, death: JV, notice: JV?) throws -> JV {
        guard let st = store else { throw LvhvDataError.Storage(message: "Chưa mở được dữ liệu trên máy.") }
        let saved = JV.parse(try st.markDeceasedJson(personId: id, deathDateValueJson: death.text,
                                                     noticeJson: notice?.text, today: today.sd)) ?? .null
        after("Đã cập nhật hồ sơ.")
        return saved
    }

    /// Xóa hồ sơ; xóa luôn sự kiện chỉ còn gắn với người này (08 §3.10).
    func deletePerson(_ id: String) throws {
        guard let st = store else { return }
        let orphans = events.filter { $0["person_ids"].array == [.string(id)] }.compactMap { $0["id"].str }
        try st.deletePerson(id: id)
        for e in orphans { try? st.deleteEvent(id: e) }
        after("Đã xóa hồ sơ.")
    }

    func saveEvent(_ e: JV) throws {
        guard let st = store else { return }
        _ = try st.saveEventJson(json: e.text)
        after("Đã lưu sự kiện.")
    }

    func deleteEvent(_ id: String) throws {
        guard let st = store else { return }
        try st.deleteEvent(id: id)
        after("Đã xóa sự kiện.")
    }

    /// Loại sự kiện theo tên: dùng lại loại trùng tên (không phân biệt hoa thường), không có thì tạo.
    func eventType(named name: String, recurring: Bool) throws -> String {
        let key = name.lowercased()
        if let t = eventTypes.first(where: { $0["ten"].string.lowercased() == key }), let id = t["id"].str { return id }
        guard let st = store else { throw LvhvDataError.Storage(message: "Chưa mở được dữ liệu trên máy.") }
        let t: JV = ["builtin": nil, "ten": .s(name), "default_recurring": .bool(recurring)]
        let saved = JV.parse(try st.saveEventTypeJson(json: t.text)) ?? .null
        reload()
        return saved["id"].string
    }

    /// Thêm ảnh đã mã hóa lại (S9). Trả id ảnh.
    func addPhoto(_ p: ProcessedPhoto) throws -> String {
        guard let st = store else { throw LvhvDataError.Storage(message: "Chưa mở được dữ liệu trên máy.") }
        let id = try FFI.uuid()
        _ = try st.addPhoto(id: id, mime: "image/jpeg", width: UInt32(p.width), height: UInt32(p.height), bytes: p.jpeg)
        PeoplePaths.excludeFromBackup(PeoplePaths.storeDir())
        return id
    }

    func photoData(_ id: String) -> Data? { try? store?.photoBytes(id: id) }

    /// Lời nhắc kế tiếp đúng như sẽ hiện trên màn hình khóa (văn bản của lõi, theo tùy chọn O-9 của máy).
    func nextLockScreenText() -> (title: String, body: String, fireLocal: String)? {
        guard let st = store,
              let j = try? st.remindersJson(nowUnixS: Int64(PeopleClock.now().timeIntervalSince1970),
                                            zone: DeviceZones.current(at: PeopleClock.now()), horizonDays: 400),
              let first = JV.parse(j)?.array.first else { return nil }
        return (first["title"].string, first["body"].string, first["fire_local"].string)
    }

    func putSettings(_ s: JV) throws {
        guard let st = store else { return }
        try st.putSettingsJson(json: s.text)
        after(nil)
    }

    func setShowNames(_ v: Bool) {
        guard let st = store else { return }
        try? st.setLockScreenNames(show: v)
        after(nil)
    }

    // MARK: .ics và sao lưu

    func icsExport() throws -> IcsExport {
        guard let st = store else { throw LvhvDataError.Storage(message: "Chưa mở được dữ liệu trên máy.") }
        return try FFI.ics(datasetJson: try st.datasetJson(), today: today, now: Date())
    }

    /// Xuất sao lưu mã hóa (Argon2id — vài giây) ở luồng nền.
    func exportBackup(password: String) async throws -> Data {
        guard let st = store else { throw LvhvDataError.Storage(message: "Chưa mở được dữ liệu trên máy.") }
        let v = Core.appVersion, pf = Core.platform
        let data = try await Task.detached(priority: .userInitiated) {
            try st.exportBackup(password: password, appVersion: v, platform: pf)
        }.value
        reload()
        return data
    }

    /// Ghi tệp đã kiểm tra ở luồng nền (MA3N-23-02); tải lại và lập lịch lại trên luồng chính.
    func applyImport(_ b: VerifiedBackup, replace: Bool) async throws {
        guard let st = store else { return }
        let mode: BackupImportMode = replace ? .replace : .merge
        try await Task.detached(priority: .userInitiated) { try st.applyImport(backup: b, mode: mode) }.value
        reload()
        Task { await scheduler.reschedule(store: store, reason: .importBackup) }
    }

    func eventsInPeriod(kind: String, anchor: CivilDate, filter: String) -> PeriodResult? {
        guard let st = store, let j = try? st.eventsInPeriodJson(kind: kind, anchor: anchor.sd, filter: filter) else { return nil }
        return Core.decode(PeriodResult.self, j)
    }

    #if DEBUG
    /// Chỉ cho kiểm thử: dùng kho đã mở sẵn.
    func useStoreForTesting(_ st: LvhvStore) {
        store = st
        opened = true
        reload()
    }
    #endif
}
