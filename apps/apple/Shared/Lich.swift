// Lớp truy cập lõi lịch (UniFFI, mô-đun sinh mã Generated/LvhvCore.swift).
// Mọi màn hình chỉ gọi lõi qua tệp này. Dữ liệu JSON khớp pwa/src/types.ts.
import Foundation

// MARK: - Mô hình (Decodable từ JSON của lõi)

struct Gio: Decodable, Hashable {
    let chi: String
    let from: Int
    let to: Int
}

struct NgocHapInfo: Decodable, Hashable {
    let sao_tot: [String]
    let sao_xau: [String]
    let nen: [String]
    let ky: [String]
    let xung_tuoi: [String]
    let score: Int
}

struct DayInfo: Decodable, Hashable {
    struct Solar: Decodable, Hashable { let y: Int; let m: Int; let d: Int; let weekday: Int; let jdn: Int }
    struct Lunar: Decodable, Hashable { let y: Int; let m: Int; let d: Int; let leap: Bool; let month_len: Int }
    struct CanChi: Decodable, Hashable { let year: String; let month: String; let day: String; let hour_ty: String }
    struct Than: Decodable, Hashable { let name: String; let hoang_dao: Bool }
    let solar: Solar
    let lunar: Lunar
    let can_chi: CanChi
    let tiet_khi: String
    let nap_am: String
    let truc: String
    let tu: String
    let than: Than
    let gio_hoang_dao: [Gio]
    let ngoc_hap: NgocHapInfo
}

struct Activity: Decodable, Hashable, Identifiable {
    let id: String
    let name: String
}

struct GoodDay: Decodable, Hashable, Identifiable {
    struct S: Decodable, Hashable { let y: Int; let m: Int; let d: Int }
    let solar: S
    let score: Int
    let reasons: [String]
    var id: Int { solar.y * 10000 + solar.m * 100 + solar.d }
    var date: CivilDate { CivilDate(y: solar.y, m: solar.m, d: solar.d) }
}

struct Cadao: Decodable, Hashable {
    let id: String
    let type: String
    let text: String
    let lines: [String]
}

struct TextBlock: Decodable, Hashable {
    let kind: String
    let text: String
}

/// Ngày dương lịch theo lịch Gregory (không phụ thuộc múi giờ).
struct CivilDate: Hashable, Comparable, Codable {
    var y: Int
    var m: Int
    var d: Int

    static func < (a: CivilDate, b: CivilDate) -> Bool { a.dayNumber < b.dayNumber }

    /// Số ngày (JDN) — Fliegel–Van Flandern, như lõi.
    var dayNumber: Int {
        let a = (14 - m) / 12
        let yy = y + 4800 - a
        let mm = m + 12 * a - 3
        return d + (153 * mm + 2) / 5 + 365 * yy + yy / 4 - yy / 100 + yy / 400 - 32045
    }

    init(y: Int, m: Int, d: Int) { self.y = y; self.m = m; self.d = d }

    init(dayNumber j: Int) {
        let a = j + 32044
        let b = (4 * a + 3) / 146097
        let c = a - (b * 146097) / 4
        let dd = (4 * c + 3) / 1461
        let e = c - (1461 * dd) / 4
        let mm = (5 * e + 2) / 153
        d = e - (153 * mm + 2) / 5 + 1
        m = mm + 3 - 12 * (mm / 10)
        y = b * 100 + dd - 4800 + mm / 10
    }

    func adding(_ n: Int) -> CivilDate { CivilDate(dayNumber: dayNumber + n) }

    /// 0 = Chủ nhật … 6 = Thứ Bảy.
    var weekday: Int { (dayNumber + 1) % 7 }

    var dmy: String { String(format: "%02d/%02d/%d", d, m, y) }

    static func today() -> CivilDate {
        let c = Calendar(identifier: .gregorian).dateComponents(in: .current, from: Date())
        return CivilDate(y: c.year ?? 2026, m: c.month ?? 1, d: c.day ?? 1)
    }

    static func daysInMonth(_ y: Int, _ m: Int) -> Int {
        switch m {
        case 1, 3, 5, 7, 8, 10, 12: return 31
        case 4, 6, 9, 11: return 30
        default: return (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 ? 29 : 28
        }
    }

    static func parse(_ s: String) -> CivilDate? {
        let p = s.split(whereSeparator: { $0 == "/" || $0 == "-" || $0 == "." }).map { Int($0.trimmingCharacters(in: .whitespaces)) }
        guard p.count == 3, let d = p[0], let m = p[1], let y = p[2], (1...12).contains(m), d >= 1, d <= daysInMonth(y, m) else { return nil }
        return CivilDate(y: y, m: m, d: d)
    }
}

enum Fmt {
    static let weekday = ["Chủ nhật", "Thứ Hai", "Thứ Ba", "Thứ Tư", "Thứ Năm", "Thứ Sáu", "Thứ Bảy"]
    static let weekdayShort = ["CN", "T2", "T3", "T4", "T5", "T6", "T7"]
    static let lunarMonth = ["", "Giêng", "Hai", "Ba", "Tư", "Năm", "Sáu", "Bảy", "Tám", "Chín", "Mười", "Mười Một", "Chạp"]
    static func hourRange(_ g: Gio) -> String { "\(g.from)–\(g.to) giờ" }
    static func lunarDayLabel(_ d: Int) -> String { d <= 10 ? "Mùng \(d)" : "Ngày \(d)" }
}

// MARK: - Truy cập lõi

enum Lich {
    static let minYear = 2000
    static let maxYear = 3000
    static let minDate = CivilDate(y: 2000, m: 1, d: 1)
    static let maxDate = CivilDate(y: 3000, m: 12, d: 31)

    static func inRange(_ d: CivilDate) -> Bool { d >= minDate && d <= maxDate }
    static func clamp(_ d: CivilDate) -> CivilDate { min(max(d, minDate), maxDate) }

    private static let decoder = JSONDecoder()
    private static let cache = NSCache<NSNumber, Box>()
    private final class Box { let v: DayInfo; init(_ v: DayInfo) { self.v = v } }

    static func dayInfo(_ d: CivilDate) -> DayInfo? {
        let key = NSNumber(value: d.dayNumber)
        if let b = cache.object(forKey: key) { return b.v }
        guard let json = try? dayInfoJson(y: Int32(d.y), m: Int32(d.m), d: Int32(d.d)),
              let v = try? decoder.decode(DayInfo.self, from: Data(json.utf8)) else { return nil }
        cache.setObject(Box(v), forKey: key)
        return v
    }

    static func monthLunar(_ y: Int, _ m: Int) -> [LunarDay] {
        (try? monthLunarDays(y: Int32(y), m: Int32(m))) ?? []
    }

    static func lunar(_ d: CivilDate) -> LunarDay? {
        try? solarToLunar(y: Int32(d.y), m: Int32(d.m), d: Int32(d.d))
    }

    static func toSolar(ly: Int, lm: Int, ld: Int, leap: Bool) -> CivilDate? {
        guard let s = lunarToSolar(ly: Int32(ly), lm: Int32(lm), ld: Int32(ld), leap: leap) else { return nil }
        return CivilDate(y: Int(s.y), m: Int(s.m), d: Int(s.d))
    }

    static func rating(_ score: Int) -> (label: String, level: Int) {
        let r = dayRating(score: Int32(score))
        return (r.label, Int(r.level))
    }

    static func canChi(year: Int) -> String { yearCanChi(y: Int32(year)) }

    static let activities: [Activity] = {
        (try? decoder.decode([Activity].self, from: Data(activitiesJson().utf8))) ?? []
    }()

    enum GoodDaysResult { case ok([GoodDay]); case error(String) }

    /// `birthLunarYear`: năm ÂM lịch của ngày sinh (năm tuổi), không phải năm dương.
    static func findGoodDays(_ act: String, from: CivilDate, to: CivilDate, birthLunarYear: Int?) -> GoodDaysResult {
        do {
            let j = try findGoodDaysJson(
                activity: act,
                from: SolarDay(y: Int32(from.y), m: Int32(from.m), d: Int32(from.d)),
                to: SolarDay(y: Int32(to.y), m: Int32(to.m), d: Int32(to.d)),
                birthLunarYear: birthLunarYear.map { Int32($0) }
            )
            return .ok((try? decoder.decode([GoodDay].self, from: Data(j.utf8))) ?? [])
        } catch let e as LvhvError {
            return .error(message(e))
        } catch {
            return .error("Không tìm được ngày.")
        }
    }

    static func message(_ e: LvhvError) -> String {
        switch e {
        case .InvalidDate(let m), .OutOfRange(let m), .UnknownActivity(let m), .BadRange(let m), .Unavailable(let m):
            return m
        }
    }

    static func cadao(_ d: CivilDate) -> Cadao? {
        guard let j = try? cadaoFor(y: Int32(d.y), m: Int32(d.m), d: Int32(d.d)), j != "null" else { return nil }
        return try? decoder.decode(Cadao.self, from: Data(j.utf8))
    }

    static let attribution: String = cadaoAttribution()
    static let cadaoTotal: Int = Int(cadaoCount())
    static let method: String = lunarMethod()

    private static func blocks(_ j: String) -> [TextBlock] {
        (try? decoder.decode([TextBlock].self, from: Data(j.utf8))) ?? []
    }

    static let cadaoSources: [TextBlock] = blocks(cadaoSourcesBlocksJson())
    static let ruleSources: [TextBlock] = blocks(ngochapSourcesBlocksJson())
}
