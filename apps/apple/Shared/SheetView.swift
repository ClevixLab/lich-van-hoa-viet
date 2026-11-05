// Tờ lịch — màn hình chính. Vuốt ngang để lật ngày (iOS); phím ← → và nút trên macOS.
import SwiftUI

struct SheetView: View {
    let date: CivilDate
    let today: CivilDate
    var openMonth: () -> Void

    var body: some View {
        if let info = Lich.dayInfo(date) {
            ScrollView {
                DaySheet(date: date, info: info, isToday: date == today, openMonth: openMonth)
                    .frame(maxWidth: 560)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                    .frame(maxWidth: .infinity)
            }
        } else {
            Text("Không tính được ngày này.").foregroundStyle(C.ink2)
        }
    }
}

struct DaySheet: View {
    let date: CivilDate
    let info: DayInfo
    let isToday: Bool
    var openMonth: () -> Void
    @State private var detailsOpen = false

    var body: some View {
        let wd = info.solar.weekday
        let sunday = wd == 0
        let big = sunday ? C.red : C.ink
        ZStack(alignment: .top) {
            VStack(spacing: 0) {
                Button(action: openMonth) {
                    HStack(spacing: 10) {
                        Text("Tháng \(date.m)").serif(24, .semibold)
                        Text(verbatim: "\(date.y)").serif(24).opacity(0.82)
                        Image(systemName: "calendar").opacity(0.8)
                    }
                    .foregroundStyle(C.bandInk)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 18)
                    .background(C.redDeep)
                }
                .buttonStyle(.plain)
                .accessibilityLabel("Tháng \(date.m) năm \(String(date.y)). Mở bảng tháng để chọn ngày khác")
                Scallop().fill(C.redDeep).frame(height: 6)

                VStack(alignment: .leading, spacing: 0) {
                    VStack(spacing: 2) {
                        Text(Fmt.weekday[wd]).serif(26, .semibold).foregroundStyle(big)
                        Text("\(date.d)").serif(150, .semibold).foregroundStyle(big).lineLimit(1).minimumScaleFactor(0.5)
                        if isToday {
                            Text("Hôm nay").sans(14, .semibold).foregroundStyle(C.redInk)
                                .padding(.horizontal, 12).padding(.vertical, 3)
                                .background(C.redTint, in: Capsule())
                        }
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.top, 12).padding(.bottom, 8)
                    .accessibilityElement(children: .ignore)
                    .accessibilityLabel("\(Fmt.weekday[wd]), ngày \(date.d) tháng \(date.m) năm \(String(date.y)). Âm lịch \(info.lunar.d) tháng \(info.lunar.m)\(info.lunar.leap ? " nhuận" : "") năm \(info.can_chi.year)")
                    .accessibilityAddTraits(.isHeader)

                    Divider().overlay(C.rule)
                    LunarBlock(info: info)
                    Divider().overlay(C.rule)
                    Labels(info: info)
                    SectionTitle("Giờ hoàng đạo")
                    HoursGrid(gio: info.gio_hoang_dao)
                    HStack(alignment: .top, spacing: 14) {
                        ExpandableList(title: "Nên làm", items: info.ngoc_hap.nen, accent: C.green)
                        ExpandableList(title: "Kỵ", items: info.ngoc_hap.ky, accent: C.redInk)
                    }
                    .padding(.vertical, 8)
                    if let cd = Lich.cadao(date) {
                        VStack(spacing: 4) {
                            ForEach(Array(cd.lines.enumerated()), id: \.offset) { _, line in
                                Text(line).serif(18).foregroundStyle(C.ink).multilineTextAlignment(.center)
                            }
                            Text(cd.type.prefix(1).uppercased() + cd.type.dropFirst()).sans(13, .semibold).foregroundStyle(C.gold).padding(.top, 4)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(16)
                        .background(C.goldTint, in: RoundedRectangle(cornerRadius: 12))
                        .padding(.vertical, 12)
                        .accessibilityElement(children: .combine)
                        .accessibilityLabel("Ca dao trong ngày: \(cd.text)")
                    }
                    Details(info: info, open: $detailsOpen)
                    Text("Theo Ngọc Hạp Thông Thư; tham khảo").sans(12).foregroundStyle(C.ink3).padding(.vertical, 12)
                }
                .padding(.horizontal, 18)
            }
            .background(C.paper)
            .clipShape(RoundedRectangle(cornerRadius: 6))
            .padding(.top, 9)
            .shadow(color: .black.opacity(0.12), radius: 14, y: 8)

            HStack {
                Spacer()
                Ring()
                Spacer()
                Ring()
                Spacer()
            }
        }
    }
}

private struct Ring: View {
    var body: some View {
        RoundedRectangle(cornerRadius: 8)
            .fill(LinearGradient(colors: [Color(white: 0.84), Color(white: 0.6)], startPoint: .top, endPoint: .bottom))
            .frame(width: 14, height: 22)
            .accessibilityHidden(true)
    }
}

/// Mép răng cưa dưới dải đỏ.
struct Scallop: Shape {
    func path(in r: CGRect) -> Path {
        var p = Path()
        let step: CGFloat = 10, rad: CGFloat = 3.4
        var x: CGFloat = step / 2
        while x < r.width + step {
            p.addEllipse(in: CGRect(x: x - rad, y: -rad, width: rad * 2, height: rad * 2))
            x += step
        }
        return p
    }
}

private struct LunarBlock: View {
    let info: DayInfo
    var body: some View {
        let key = info.lunar.d == 1 || info.lunar.d == 15
        HStack(alignment: .center, spacing: 0) {
            VStack(spacing: 2) {
                Text("Âm lịch").sans(14).foregroundStyle(C.ink3)
                Text("\(info.lunar.d)").serif(56, .bold).foregroundStyle(key ? C.red : C.gold)
                Text("tháng \(Fmt.lunarMonth[info.lunar.m])\(info.lunar.leap ? " (nhuận)" : "")").serif(19).foregroundStyle(C.ink).multilineTextAlignment(.center)
                Text(info.lunar.month_len == 30 ? "tháng đủ" : "tháng thiếu").sans(12).foregroundStyle(C.ink3)
            }
            .frame(maxWidth: .infinity)
            Rectangle().fill(C.rule).frame(width: 1).padding(.vertical, 4)
            VStack(alignment: .leading, spacing: 6) {
                KV(k: "Năm", v: info.can_chi.year)
                KV(k: "Tháng", v: info.can_chi.month)
                KV(k: "Ngày", v: info.can_chi.day)
                KV(k: "Tiết", v: info.tiet_khi)
            }
            .padding(.leading, 16)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(.vertical, 14)
        .fixedSize(horizontal: false, vertical: true)
    }
}

private struct KV: View {
    let k: String
    let v: String
    var body: some View {
        HStack(alignment: .firstTextBaseline) {
            Text(k).sans(14).foregroundStyle(C.ink3).frame(width: 58, alignment: .leading)
            Text(v).serif(19).foregroundStyle(C.ink)
        }
        .accessibilityElement(children: .combine)
    }
}

private struct Labels: View {
    let info: DayInfo
    var body: some View {
        let r = Lich.rating(info.ngoc_hap.score)
        let (rb, rf): (Color, Color) = r.level >= 2 ? (C.greenTint, C.green) : r.level == 1 ? (C.paper2, C.ink) : (C.redTint, C.redInk)
        VStack(spacing: 8) {
            LabelBox(k: info.than.hoang_dao ? "Hoàng đạo" : "Hắc đạo", v: info.than.name,
                     bg: info.than.hoang_dao ? C.goldTint : C.paper2, fg: info.than.hoang_dao ? C.gold : C.ink)
            HStack(spacing: 8) {
                LabelBox(k: "Trực", v: info.truc, bg: C.paper2, fg: C.ink)
                LabelBox(k: "Đánh giá", v: r.label, bg: rb, fg: rf)
            }
        }
        .padding(.vertical, 14)
    }
}

private struct LabelBox: View {
    let k: String
    let v: String
    let bg: Color
    let fg: Color
    var body: some View {
        VStack(spacing: 2) {
            Text(k).sans(14).foregroundStyle(C.ink3)
            Text(v).sans(17, .semibold).foregroundStyle(fg)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 10)
        .background(bg, in: RoundedRectangle(cornerRadius: 12))
        .accessibilityElement(children: .combine)
    }
}

struct SectionTitle: View {
    let text: String
    init(_ t: String) { text = t }
    var body: some View {
        Text(text).serif(20, .semibold).foregroundStyle(C.ink).padding(.top, 10).padding(.bottom, 6).accessibilityAddTraits(.isHeader)
    }
}

private struct HoursGrid: View {
    let gio: [Gio]
    var body: some View {
        LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 8), count: 3), spacing: 8) {
            ForEach(gio, id: \.self) { g in
                VStack(spacing: 2) {
                    Text(g.chi).serif(18, .semibold).foregroundStyle(C.ink)
                    Text(Fmt.hourRange(g)).sans(12).foregroundStyle(C.ink3)
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 8)
                .overlay(RoundedRectangle(cornerRadius: 10).stroke(C.rule))
                .accessibilityElement(children: .ignore)
                .accessibilityLabel("Giờ \(g.chi), \(Fmt.hourRange(g))")
            }
        }
    }
}

private struct ExpandableList: View {
    let title: String
    let items: [String]
    let accent: Color
    @State private var open = false
    private let maxList = 4
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title).serif(20, .semibold).foregroundStyle(accent).accessibilityAddTraits(.isHeader)
            if items.isEmpty {
                Text("Không có ghi nhận").sans(15).foregroundStyle(C.ink3)
            } else {
                ForEach(open ? items : Array(items.prefix(maxList)), id: \.self) { t in
                    Text("• \(t)").sans(15).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
                }
                if items.count > maxList {
                    Button(open ? "Thu gọn" : "Xem thêm \(items.count - maxList) việc") { withAnimation { open.toggle() } }
                        .buttonStyle(.plain).foregroundStyle(C.redInk).sans(15, .semibold)
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

private struct Chips: View {
    let items: [String]
    let bg: Color
    let fg: Color
    var body: some View {
        if items.isEmpty {
            Text("Không có").sans(15).foregroundStyle(C.ink3)
        } else {
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 110), spacing: 6, alignment: .leading)], alignment: .leading, spacing: 6) {
                ForEach(items, id: \.self) { t in
                    Text(t).sans(14).foregroundStyle(fg).padding(.horizontal, 10).padding(.vertical, 3).background(bg, in: Capsule())
                }
            }
        }
    }
}

private struct Details: View {
    let info: DayInfo
    @Binding var open: Bool
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Button {
                withAnimation { open.toggle() }
            } label: {
                HStack {
                    Text("Chi tiết").serif(20, .semibold).foregroundStyle(C.ink)
                    Spacer()
                    Image(systemName: "chevron.down").rotationEffect(.degrees(open ? 180 : 0)).foregroundStyle(C.ink2)
                }
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityLabel(open ? "Thu gọn chi tiết" : "Mở chi tiết")
            if open {
                Text("Sao tốt").sans(16, .semibold).foregroundStyle(C.ink)
                Chips(items: info.ngoc_hap.sao_tot, bg: C.greenTint, fg: C.green)
                Text("Sao xấu").sans(16, .semibold).foregroundStyle(C.ink)
                Chips(items: info.ngoc_hap.sao_xau, bg: C.redTint, fg: C.redInk)
                Fact(k: "Nhị thập bát tú", v: "Sao \(info.tu)")
                Fact(k: "Nạp âm", v: info.nap_am)
                Fact(k: "Giờ Tý đầu ngày", v: info.can_chi.hour_ty)
                Fact(k: "Tuổi xung", v: info.ngoc_hap.xung_tuoi.isEmpty ? "Không có" : info.ngoc_hap.xung_tuoi.joined(separator: ", "))
            }
        }
        .padding(14)
        .overlay(RoundedRectangle(cornerRadius: 12).stroke(C.rule))
        .padding(.vertical, 8)
    }
}

private struct Fact: View {
    let k: String
    let v: String
    var body: some View {
        HStack(alignment: .firstTextBaseline) {
            Text(k).sans(15).foregroundStyle(C.ink3).frame(maxWidth: .infinity, alignment: .leading)
            Text(v).sans(16).foregroundStyle(C.ink).frame(maxWidth: .infinity, alignment: .leading)
        }
        .accessibilityElement(children: .combine)
    }
}
