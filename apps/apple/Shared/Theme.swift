// Bảng màu và phông — giống pwa/src/style.css.
import SwiftUI

#if os(iOS)
import UIKit
typealias PlatformColor = UIColor
#else
import AppKit
typealias PlatformColor = NSColor
#endif

private func hex(_ v: UInt32) -> (CGFloat, CGFloat, CGFloat) {
    (CGFloat((v >> 16) & 0xFF) / 255, CGFloat((v >> 8) & 0xFF) / 255, CGFloat(v & 0xFF) / 255)
}

/// Màu đổi theo chế độ sáng/tối của hệ thống (hoặc của cửa sổ khi người dùng ép).
private func dyn(_ light: UInt32, _ dark: UInt32) -> Color {
    let l = hex(light), d = hex(dark)
    #if os(iOS)
    return Color(UIColor { t in
        let c = t.userInterfaceStyle == .dark ? d : l
        return UIColor(red: c.0, green: c.1, blue: c.2, alpha: 1)
    })
    #else
    return Color(NSColor(name: nil) { a in
        let isDark = a.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
        let c = isDark ? d : l
        return NSColor(srgbRed: c.0, green: c.1, blue: c.2, alpha: 1)
    })
    #endif
}

enum C {
    static let wall = dyn(0xE8E3E1, 0x141011)
    static let paper = dyn(0xFDFCFA, 0x211A1B)
    static let paper2 = dyn(0xF6F2EE, 0x2A2223)
    static let rule = dyn(0xE4DBD5, 0x3A3031)
    static let ink = dyn(0x231A1A, 0xF1E9E6)
    static let ink2 = dyn(0x5E5150, 0xCDBFBB)
    static let ink3 = dyn(0x7A6B69, 0xA99A96)
    static let red = dyn(0xC4161C, 0xFF6B63)
    static let redDeep = dyn(0x8F1416, 0x7A1214)
    static let redInk = dyn(0xB3151B, 0xFF8A83)
    static let redTint = dyn(0xFBECEB, 0x3A1D1E)
    static let gold = dyn(0x8A5E0C, 0xE3B45C)
    static let goldTint = dyn(0xF8EFDC, 0x352A18)
    static let green = dyn(0x1F6B45, 0x7FD1A2)
    static let greenTint = dyn(0xE7F2EC, 0x1B2E24)
    static let bandInk = dyn(0xFFF8F4, 0xFFF1EC)
    /// Ô nhập của mô-đun Người thân: nền và viền (viền ≥ 3:1 với nền thẻ — TKN-I-06).
    static let field = dyn(0xFFFFFF, 0x161112)
    static let fieldBorder = dyn(0x8F807D, 0x8A7B78)
}

/// Phông "LVHV Serif" (Lora rút gọn, OFL 1.1), co giãn theo Dynamic Type và theo cỡ chữ người dùng chọn.
struct SerifFont: ViewModifier {
    let size: CGFloat
    let weight: Font.Weight
    @Environment(\.textScale) private var scale

    func body(content: Content) -> some View {
        let name: String
        switch weight {
        case .bold, .heavy, .black: name = "LVHVSerif-Bold"
        case .semibold, .medium: name = "LVHVSerif-SemiBold"
        default: name = "LVHVSerif-Regular"
        }
        return content.font(.custom(name, size: size * scale, relativeTo: .body))
    }
}

struct SansFont: ViewModifier {
    let size: CGFloat
    let weight: Font.Weight
    @Environment(\.textScale) private var scale
    func body(content: Content) -> some View {
        content.font(.system(size: size * scale, weight: weight))
    }
}

extension View {
    func serif(_ size: CGFloat, _ weight: Font.Weight = .regular) -> some View { modifier(SerifFont(size: size, weight: weight)) }
    func sans(_ size: CGFloat, _ weight: Font.Weight = .regular) -> some View { modifier(SansFont(size: size, weight: weight)) }
}

private struct TextScaleKey: EnvironmentKey { static let defaultValue: CGFloat = 1 }
extension EnvironmentValues {
    var textScale: CGFloat {
        get { self[TextScaleKey.self] }
        set { self[TextScaleKey.self] = newValue }
    }
}

/// Khung bảng nền giấy, bo góc.
struct Panel<Content: View>: View {
    @ViewBuilder var content: Content
    var body: some View {
        VStack(alignment: .leading, spacing: 8) { content }
            .padding(16)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(C.paper, in: RoundedRectangle(cornerRadius: 16, style: .continuous))
    }
}

struct H2: View {
    let text: String
    init(_ t: String) { text = t }
    var body: some View {
        Text(text).serif(20, .semibold).foregroundStyle(C.ink).accessibilityAddTraits(.isHeader)
    }
}

struct P: View {
    let text: String
    init(_ t: String) { text = t }
    var body: some View {
        Text(text).sans(16).foregroundStyle(C.ink2).fixedSize(horizontal: false, vertical: true)
    }
}
