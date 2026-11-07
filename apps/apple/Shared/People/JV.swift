// Giá trị JSON tổng quát — dữ liệu của lõi (Person, Event, AppSettings…) đi qua FFI dạng chuỗi JSON.
// Giữ nguyên mọi trường (kể cả trường giao diện không sửa, ví dụ `memorial.bia_tieu_de`) khi đọc rồi ghi lại.
import Foundation

enum JV: Codable, Equatable, Hashable {
    case null
    case bool(Bool)
    case int(Int64)
    case double(Double)
    case string(String)
    case array([JV])
    case object([String: JV])

    init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if c.decodeNil() { self = .null; return }
        if let b = try? c.decode(Bool.self) { self = .bool(b); return }
        if let i = try? c.decode(Int64.self) { self = .int(i); return }
        if let d = try? c.decode(Double.self) { self = .double(d); return }
        if let s = try? c.decode(String.self) { self = .string(s); return }
        if let a = try? c.decode([JV].self) { self = .array(a); return }
        if let o = try? c.decode([String: JV].self) { self = .object(o); return }
        throw DecodingError.dataCorruptedError(in: c, debugDescription: "JSON không hợp lệ")
    }

    func encode(to encoder: Encoder) throws {
        var c = encoder.singleValueContainer()
        switch self {
        case .null: try c.encodeNil()
        case .bool(let b): try c.encode(b)
        case .int(let i): try c.encode(i)
        case .double(let d): try c.encode(d)
        case .string(let s): try c.encode(s)
        case .array(let a): try c.encode(a)
        case .object(let o): try c.encode(o)
        }
    }

    static func parse(_ text: String) -> JV? {
        try? JSONDecoder().decode(JV.self, from: Data(text.utf8))
    }

    /// Chuỗi JSON (khóa sắp xếp để ổn định khi so sánh trong kiểm thử).
    var text: String {
        let e = JSONEncoder()
        e.outputFormatting = [.sortedKeys, .withoutEscapingSlashes]
        guard let d = try? e.encode(self) else { return "null" }
        return String(decoding: d, as: UTF8.self)
    }

    subscript(key: String) -> JV {
        get {
            if case .object(let o) = self { return o[key] ?? .null }
            return .null
        }
        set {
            var o: [String: JV]
            if case .object(let x) = self { o = x } else { o = [:] }
            o[key] = newValue
            self = .object(o)
        }
    }

    subscript(index: Int) -> JV {
        if case .array(let a) = self, a.indices.contains(index) { return a[index] }
        return .null
    }

    /// Bản sao không có khóa `key`.
    func removing(_ key: String) -> JV {
        guard case .object(var o) = self else { return self }
        o.removeValue(forKey: key)
        return .object(o)
    }

    var isNull: Bool { if case .null = self { return true }; return false }
    var str: String? { if case .string(let s) = self { return s }; return nil }
    var string: String { str ?? "" }
    var int: Int? {
        switch self {
        case .int(let i): return Int(i)
        case .double(let d) where d == d.rounded(): return Int(d)
        default: return nil
        }
    }
    var bool: Bool? { if case .bool(let b) = self { return b }; return nil }
    var isTrue: Bool { bool == true }
    var array: [JV] { if case .array(let a) = self { return a }; return [] }
    var object: [String: JV]? { if case .object(let o) = self { return o }; return nil }
    var isObject: Bool { object != nil }

    static func s(_ v: String) -> JV { .string(v) }
    static func i(_ v: Int) -> JV { .int(Int64(v)) }
    static func opt(_ v: String?) -> JV { v.map { .string($0) } ?? .null }
    static func opt(_ v: Int?) -> JV { v.map { .int(Int64($0)) } ?? .null }
}

extension JV: ExpressibleByStringLiteral, ExpressibleByIntegerLiteral, ExpressibleByBooleanLiteral,
    ExpressibleByArrayLiteral, ExpressibleByDictionaryLiteral, ExpressibleByNilLiteral {
    init(stringLiteral value: String) { self = .string(value) }
    init(integerLiteral value: Int) { self = .int(Int64(value)) }
    init(booleanLiteral value: Bool) { self = .bool(value) }
    init(arrayLiteral elements: JV...) { self = .array(elements) }
    init(dictionaryLiteral elements: (String, JV)...) {
        var o: [String: JV] = [:]
        for (k, v) in elements { o[k] = v }
        self = .object(o)
    }
    init(nilLiteral: ()) { self = .null }
}
