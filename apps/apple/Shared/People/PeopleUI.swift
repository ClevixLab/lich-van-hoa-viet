// Thành phần giao diện dùng chung của mô-đun "Người thân" (08 §2), theo HIG của Apple.
import SwiftUI

// MARK: - Chữ: co giãn theo cỡ chữ A/A+/A++ của ứng dụng và theo Dynamic Type của máy

enum PeopleType {
    static func factor(_ d: DynamicTypeSize) -> CGFloat {
        switch d {
        case .xSmall: return 0.82
        case .small: return 0.88
        case .medium: return 0.94
        case .large: return 1
        case .xLarge: return 1.1
        case .xxLarge: return 1.2
        case .xxxLarge: return 1.3
        case .accessibility1: return 1.5
        case .accessibility2: return 1.7
        case .accessibility3: return 1.9
        case .accessibility4: return 2.1
        case .accessibility5: return 2.3
        @unknown default: return 1
        }
    }
}

struct PSansFont: ViewModifier {
    let size: CGFloat
    let weight: Font.Weight
    @Environment(\.textScale) private var scale
    @Environment(\.dynamicTypeSize) private var dts
    func body(content: Content) -> some View {
        content.font(.system(size: size * scale * PeopleType.factor(dts), weight: weight))
    }
}

extension View {
    /// Chữ không chân của mô-đun (co theo Dynamic Type).
    func ps(_ size: CGFloat, _ weight: Font.Weight = .regular) -> some View { modifier(PSansFont(size: size, weight: weight)) }
}

// MARK: - Khung, nút

struct PCard<Content: View>: View {
    var padding: CGFloat = 16
    @ViewBuilder var content: Content
    var body: some View {
        VStack(alignment: .leading, spacing: 10) { content }
            .padding(padding)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(C.paper, in: RoundedRectangle(cornerRadius: 16, style: .continuous))
    }
}

struct PHeading: View {
    let text: String
    var size: CGFloat = 20
    init(_ t: String, size: CGFloat = 20) { text = t; self.size = size }
    var body: some View {
        Text(text).serif(size, .semibold).foregroundStyle(C.ink).accessibilityAddTraits(.isHeader)
            .fixedSize(horizontal: false, vertical: true)
    }
}

struct PFine: View {
    let text: String
    init(_ t: String) { text = t }
    var body: some View {
        Text(text).ps(14).foregroundStyle(C.ink3).fixedSize(horizontal: false, vertical: true)
    }
}

struct PBody: View {
    let text: String
    init(_ t: String) { text = t }
    var body: some View {
        Text(text).ps(16).foregroundStyle(C.ink2).fixedSize(horizontal: false, vertical: true)
    }
}

/// Nút chính (viên thuốc đỏ) — một nút chính mỗi khung (TK-20).
struct PrimaryButtonStyle: ButtonStyle {
    @Environment(\.isEnabled) private var enabled
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .ps(16, .semibold)
            .foregroundStyle(.white)
            .padding(.horizontal, 20)
            .frame(minHeight: 44)
            .background(C.redDeep.opacity(enabled ? (configuration.isPressed ? 0.8 : 1) : 0.4), in: Capsule())
            .contentShape(Capsule())
    }
}

struct SecondaryButtonStyle: ButtonStyle {
    var destructive = false
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .ps(16, .semibold)
            .foregroundStyle(destructive ? C.redInk : C.ink)
            .padding(.horizontal, 18)
            .frame(minHeight: 44)
            .background(C.paper2.opacity(configuration.isPressed ? 0.6 : 1), in: Capsule())
            .overlay(Capsule().stroke(destructive ? C.redInk : C.rule, lineWidth: 1))
            .contentShape(Capsule())
    }
}

/// Bố cục dòng tự xuống hàng (nhóm nút chọn, chip mốc nhắc).
struct FlowLayout: Layout {
    var spacing: CGFloat = 8
    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let maxW = proposal.width ?? .infinity
        var x: CGFloat = 0, y: CGFloat = 0, rowH: CGFloat = 0, width: CGFloat = 0
        for s in subviews {
            let sz = s.sizeThatFits(.unspecified)
            if x > 0 && x + sz.width > maxW { x = 0; y += rowH + spacing; rowH = 0 }
            x += sz.width + spacing
            rowH = max(rowH, sz.height)
            width = max(width, x - spacing)
        }
        return CGSize(width: min(width, maxW), height: y + rowH)
    }
    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        var x = bounds.minX, y = bounds.minY, rowH: CGFloat = 0
        for s in subviews {
            let sz = s.sizeThatFits(.unspecified)
            if x > bounds.minX && x + sz.width > bounds.maxX { x = bounds.minX; y += rowH + spacing; rowH = 0 }
            s.place(at: CGPoint(x: x, y: y), proposal: ProposedViewSize(sz))
            x += sz.width + spacing
            rowH = max(rowH, sz.height)
        }
    }
}

/// Nút chọn bật/tắt (mốc nhắc, loại kỳ, bộ lọc) — trạng thái đọc được qua `isSelected`.
struct ChoiceChip: View {
    let title: String
    let on: Bool
    var action: () -> Void
    var body: some View {
        Button(action: action) {
            Text(title).ps(15, .semibold)
                .foregroundStyle(on ? Color.white : C.ink)
                .padding(.horizontal, 14)
                .frame(minHeight: 44)
                .background(on ? C.redDeep : C.paper2, in: Capsule())
                .overlay(Capsule().stroke(on ? C.redDeep : C.rule))
                .contentShape(Capsule())
        }
        .buttonStyle(.plain)
        .accessibilityAddTraits(on ? .isSelected : [])
    }
}

// MARK: - Trường nhập

struct FieldError: View {
    let text: String?
    var body: some View {
        if let text {
            Label { Text(text).ps(14) } icon: { Image(systemName: "exclamationmark.circle") }
                .foregroundStyle(C.redInk)
                .fixedSize(horizontal: false, vertical: true)
        }
    }
}

/// Trường có nhãn thấy được phía trên (dễ đọc khi chữ lớn), lời gợi ý và lỗi phía dưới.
struct LabeledField<Content: View>: View {
    let label: String
    var hint: String? = nil
    var error: String? = nil
    @ViewBuilder var content: Content
    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(label).ps(15, .semibold).foregroundStyle(C.ink)
            content
            if let hint { PFine(hint) }
            FieldError(text: error)
        }
    }
}

struct PTextField: View {
    let label: String
    @Binding var text: String
    var hint: String? = nil
    var error: String? = nil
    var multiline = false
    var suggestions: [String] = []
    var id: String? = nil

    var body: some View {
        LabeledField(label: label, hint: hint, error: error) {
            Group {
                if multiline {
                    TextField("", text: $text, axis: .vertical).lineLimit(3...8)
                } else {
                    TextField("", text: $text)
                }
            }
            .textFieldStyle(.roundedBorder)
            .ps(17)
            .autocorrectionDisabled(true)
            .accessibilityLabel(label)
            .accessibilityIdentifier(id ?? label)
            if !suggestions.isEmpty {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 6) {
                        ForEach(suggestions, id: \.self) { s in
                            Button(s) { text = s }
                                .buttonStyle(.plain)
                                .ps(14)
                                .foregroundStyle(C.ink2)
                                .padding(.horizontal, 10)
                                .frame(minHeight: 36)
                                .background(C.paper2, in: Capsule())
                                .accessibilityLabel("Gợi ý: \(s)")
                        }
                    }
                }
            }
        }
    }
}

/// Ô số nhỏ (ngày, tháng, năm, giờ, phút) với nhãn phía trên.
struct NumField: View {
    let label: String
    @Binding var text: String
    var width: CGFloat = 64
    var id: String? = nil
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label).ps(13).foregroundStyle(C.ink2)
            TextField("", text: $text)
                .textFieldStyle(.roundedBorder)
                .ps(18)
                .frame(minWidth: width, maxWidth: width * 1.6)
                #if os(iOS)
                .keyboardType(.numberPad)
                #endif
                .accessibilityLabel(label)
                .accessibilityIdentifier(id ?? label)
        }
    }
}

// MARK: - Ô nhập ngày (08 §2): Dương/Âm, nhuận, không rõ năm / chỉ biết năm, kết quả đổi lịch trực tiếp

struct DateInputView: View {
    @Binding var draft: DateDraft
    let purpose: DatePurpose
    var allowLunar = true
    var allowYearOnly = false
    var allowUnknownYear = false
    var living = false
    var error: String?
    let today: CivilDate
    var idPrefix = "date"

    private var calChoices: (String, String) {
        purpose == .death ? ("Nhập theo dương lịch", "Nhập theo âm lịch") : ("Dương lịch", "Âm lịch")
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            if allowLunar {
                Picker("Lịch", selection: $draft.cal) {
                    Text(calChoices.0).tag(CalKind.solar)
                    Text(calChoices.1).tag(CalKind.lunar)
                }
                .pickerStyle(.segmented)
                .labelsHidden()
                .accessibilityIdentifier("\(idPrefix).cal")
            }
            if allowYearOnly {
                Picker("Độ chính xác", selection: $draft.yearOnly) {
                    Text("Biết đủ ngày").tag(false)
                    Text("Chỉ biết năm").tag(true)
                }
                .pickerStyle(.segmented)
                .labelsHidden()
            }
            HStack(alignment: .bottom, spacing: 10) {
                if !draft.yearOnly {
                    NumField(label: "Ngày", text: $draft.day, width: 56, id: "\(idPrefix).day")
                    NumField(label: "Tháng", text: $draft.month, width: 56, id: "\(idPrefix).month")
                }
                if !(draft.cal == .lunar && draft.unknownYear && !draft.yearOnly) {
                    NumField(label: "Năm", text: $draft.year, width: 80, id: "\(idPrefix).year")
                }
                Spacer(minLength: 0)
            }
            if draft.cal == .lunar && !draft.yearOnly { lunarOptions }
            if let error {
                FieldError(text: error)
            } else if let line = resultLine {
                Text(line).ps(15, .semibold).foregroundStyle(C.green)
                    .fixedSize(horizontal: false, vertical: true)
                    .accessibilityIdentifier("\(idPrefix).result")
            }
        }
        .onChange(of: draft.year) { _ in fixLeap() }
        .onChange(of: draft.cal) { _ in fixLeap() }
    }

    @ViewBuilder
    private var lunarOptions: some View {
        if allowUnknownYear {
            Toggle(purpose == .death ? "Không rõ năm mất" : "Không rõ năm", isOn: $draft.unknownYear)
                .ps(16)
                .accessibilityIdentifier("\(idPrefix).unknownYear")
        }
        if let y = draft.yearNumber, (1899...3000).contains(y) {
            if let lm = DateCheck.leapMonth(ofLunarYear: y) {
                Toggle("Tháng nhuận (năm \(String(y)) nhuận tháng \(lm))", isOn: $draft.leap).ps(16)
                    .accessibilityIdentifier("\(idPrefix).leap")
            } else {
                PFine("Năm \(String(y)) không có tháng nhuận")
            }
        } else {
            Toggle("Tháng nhuận", isOn: $draft.leap).ps(16).accessibilityIdentifier("\(idPrefix).leap")
        }
    }

    private func fixLeap() {
        if draft.cal == .solar { draft.leap = false; draft.unknownYear = false; return }
        if let y = draft.yearNumber, (1899...3000).contains(y), DateCheck.leapMonth(ofLunarYear: y) == nil {
            draft.leap = false
        }
    }

    private var resultLine: String? {
        if draft.isEmpty { return nil }
        let c: CheckedDate
        switch purpose {
        case .death: c = DateCheck.death(draft, today: today)
        case .birth: c = DateCheck.birth(draft, today: today)
        case .event: c = DateCheck.event(draft)
        case .notice: return nil
        }
        return DateCheck.resultLine(c, purpose: purpose, living: living, today: today)
    }
}

// MARK: - Nhóm nhắc (08 §3.4 "Nhắc ngày giỗ")

struct RuleEditor: View {
    @Binding var rule: RuleDraft
    let title: String
    var tienThuong = false
    var errors: FieldErrors
    let prefix: String
    var showToggle = true

    private var chips: [Int] {
        Array(Set(RuleDraft.quick).union(rule.offsets)).sorted()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            if showToggle {
                Toggle(title, isOn: $rule.enabled).ps(16, .semibold).accessibilityIdentifier("\(prefix).enabled")
            } else {
                PHeading(title)
            }
            VStack(alignment: .leading, spacing: 10) {
                Text("Nhắc trước").ps(15, .semibold).foregroundStyle(C.ink)
                FlowLayout {
                    ForEach(chips, id: \.self) { k in
                        ChoiceChip(title: k == 0 ? "Đúng ngày" : "\(k) ngày", on: rule.offsets.contains(k)) {
                            if rule.offsets.contains(k) { rule.offsets.remove(k) } else { rule.offsets.insert(k) }
                        }
                    }
                }
                HStack(alignment: .bottom, spacing: 8) {
                    NumField(label: "Tự đặt (1–60 ngày)", text: $rule.custom, width: 64, id: "\(prefix).custom")
                    Text("ngày trước").ps(15).foregroundStyle(C.ink2).padding(.bottom, 8)
                }
                FieldError(text: errors[prefix + ".custom"] ?? errors[prefix + ".offsets"])
                if let w = rule.warning {
                    Text(w).ps(14).foregroundStyle(C.gold).fixedSize(horizontal: false, vertical: true)
                }
                Text("Giờ nhắc (24 giờ)").ps(15, .semibold).foregroundStyle(C.ink)
                HStack(alignment: .bottom, spacing: 6) {
                    NumField(label: "Giờ", text: $rule.hour, width: 52, id: "\(prefix).hour")
                    Text(":").ps(20, .semibold).padding(.bottom, 6)
                    NumField(label: "Phút", text: $rule.minute, width: 52, id: "\(prefix).minute")
                }
                FieldError(text: errors[prefix + ".time"])
                if tienThuong {
                    Toggle("Nhắc thêm ngày tiên thường (lễ cúng hôm trước ngày giỗ)", isOn: $rule.tienThuong).ps(16)
                }
            }
            .disabled(!rule.enabled)
            .opacity(rule.enabled ? 1 : 0.5)
        }
    }
}

// MARK: - Mục sự kiện (08 §2 `.pitem`)

enum OccIcon {
    static func symbol(_ cls: String) -> String {
        switch cls {
        case "gio": return "flame"
        case "tien_thuong": return "fork.knife"
        case "sinh_nhat", "sinh_nhat_am": return "gift"
        default: return "star"
        }
    }
}

struct OccRow: View {
    let item: OccItem
    let today: CivilDate
    var extra: String? = nil
    var chevron = false

    private var days: Int { item.civil.dayNumber - today.dayNumber }

    var body: some View {
        let d = item.civil
        HStack(alignment: .top, spacing: 12) {
            VStack(spacing: 0) {
                Text(Fmt.weekdayShort[d.weekdayCore]).ps(13, .semibold).foregroundStyle(C.redInk)
                Text("\(d.d)").serif(26, .semibold).foregroundStyle(C.ink)
                Text("thg \(d.m)").ps(12).foregroundStyle(C.ink2)
            }
            .frame(minWidth: 56)
            .padding(.vertical, 6)
            .background(C.paper2, in: RoundedRectangle(cornerRadius: 10, style: .continuous))
            .accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 3) {
                Text(item.headline).ps(17, .semibold).foregroundStyle(C.ink)
                    .fixedSize(horizontal: false, vertical: true)
                Text(item.dateText).ps(14).foregroundStyle(C.ink2)
                    .fixedSize(horizontal: false, vertical: true)
                if let extra { Text(extra).ps(14).foregroundStyle(C.ink3).fixedSize(horizontal: false, vertical: true) }
                HStack(spacing: 6) {
                    Image(systemName: OccIcon.symbol(item.class)).accessibilityHidden(true)
                    Text("\(item.class_text) · \(PFmt.distance(days))").ps(14, .semibold)
                }
                .foregroundStyle(days == 0 ? C.redInk : C.ink2)
            }
            Spacer(minLength: 0)
            if chevron {
                Image(systemName: "chevron.right").foregroundStyle(C.ink3).accessibilityHidden(true)
                    .padding(.top, 6)
            }
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(C.paper, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(RoundedRectangle(cornerRadius: 14, style: .continuous)
            .stroke(days == 0 ? C.red : Color.clear, lineWidth: 2))
        .opacity(days < 0 ? 0.7 : 1)
        .accessibilityElement(children: .combine)
    }
}

// MARK: - Ảnh

/// Ảnh từ kho (theo id) hoặc ảnh vừa chọn; không có ảnh → chữ cái đầu.
struct PhotoView: View {
    var photoId: String?
    var newPhoto: ProcessedPhoto?
    var initials: String
    var size: CGSize = CGSize(width: 96, height: 120)
    var oval = true
    var dark = false
    @ObservedObject private var m = PeopleModel.shared

    var body: some View {
        let shape = RoundedRectangle(cornerRadius: oval ? min(size.width, size.height) / 2 : 10, style: .continuous)
        Group {
            if let img = image {
                Image(decorative: img, scale: 1).resizable().scaledToFill()
            } else {
                Text(initials).serif(size.width * 0.38, .semibold)
                    .foregroundStyle(dark ? Color(red: 0.91, green: 0.78, blue: 0.48) : C.ink2)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .background(dark ? Color(red: 0.17, green: 0.14, blue: 0.13) : C.paper2)
            }
        }
        .frame(width: size.width, height: size.height)
        .clipShape(oval ? AnyShape(Ellipse()) : AnyShape(shape))
        .overlay((oval ? AnyShape(Ellipse()) : AnyShape(shape))
            .stroke(dark ? Color(red: 0.79, green: 0.62, blue: 0.32) : C.rule, lineWidth: dark ? 3 : 1))
        .accessibilityHidden(true)
    }

    private var image: CGImage? {
        if let p = newPhoto { return PhotoProcessor.cgImage(p.jpeg) }
        guard let id = photoId else { return nil }
        return PhotoCache.shared.image(id) { m.photoData(id) }
    }
}

/// Thông báo ngắn ở đầu trang, tự ẩn sau 6 giây (08 §5.2).
struct ToastBar: View {
    @Binding var text: String?
    var body: some View {
        if let t = text {
            HStack(spacing: 10) {
                Image(systemName: "checkmark.circle.fill").foregroundStyle(C.green).accessibilityHidden(true)
                Text(t).ps(15, .semibold).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
                Spacer(minLength: 0)
                Button { text = nil } label: { Image(systemName: "xmark").frame(width: 44, height: 44) }
                    .buttonStyle(.plain).foregroundStyle(C.ink2).accessibilityLabel("Đóng thông báo")
            }
            .padding(.leading, 14)
            .background(C.greenTint, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .accessibilityElement(children: .contain)
            .accessibilityAddTraits(.updatesFrequently)
            .task(id: t) {
                try? await Task.sleep(nanoseconds: 6_000_000_000)
                if text == t { text = nil }
            }
        }
    }
}

/// Khung tóm tắt lỗi đầu biểu mẫu: "Còn n chỗ cần sửa".
struct ErrorSummary: View {
    let errors: FieldErrors
    var body: some View {
        if !errors.isEmpty {
            VStack(alignment: .leading, spacing: 6) {
                Text("Còn \(errors.count) chỗ cần sửa").ps(16, .semibold).foregroundStyle(C.redInk)
                ForEach(errors.items, id: \.key) { e in
                    Text("• \(e.message)").ps(14).foregroundStyle(C.redInk).fixedSize(horizontal: false, vertical: true)
                }
            }
            .padding(14)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(C.redTint, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .accessibilityElement(children: .combine)
            .accessibilityIdentifier("form.errors")
        }
    }
}

/// Dòng "nhãn — giá trị" của bảng thông tin.
struct InfoRow: View {
    let label: String
    let value: String
    var body: some View {
        if !value.isEmpty {
            ViewThatFits(in: .horizontal) {
                HStack(alignment: .firstTextBaseline, spacing: 12) {
                    Text(label).ps(15).foregroundStyle(C.ink2).frame(width: 130, alignment: .leading)
                    Text(value).ps(16).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
                    Spacer(minLength: 0)
                }
                VStack(alignment: .leading, spacing: 2) {
                    Text(label).ps(14).foregroundStyle(C.ink2)
                    Text(value).ps(16).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
                }
            }
            .accessibilityElement(children: .combine)
        }
    }
}

/// Bề rộng nội dung: một cột trên điện thoại, căn giữa có giới hạn trên iPad/Mac (08 §7).
extension View {
    func pageWidth(_ max: CGFloat = 720) -> some View {
        self.padding(16).frame(maxWidth: max).frame(maxWidth: .infinity)
    }
}
