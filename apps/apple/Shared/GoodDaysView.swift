// Chọn ngày tốt. Năm sinh chỉ giữ trong @State khi màn hình đang mở; không ghi xuống máy.
import SwiftUI

private enum RangeKey: String, CaseIterable, Identifiable {
    case week = "Tuần này", month = "Tháng này", three = "3 tháng tới", custom = "Tự chọn"
    var id: String { rawValue }
}

struct GoodDaysView: View {
    let today: CivilDate
    var openDay: (CivilDate) -> Void

    @State private var act: String?
    @State private var range: RangeKey = .month
    @State private var fromText = ""
    @State private var toText = ""
    @State private var birth = ""
    @State private var shown = 15
    private let page = 15

    private var birthYear: Int? {
        guard let y = Int(birth), (1900...today.y).contains(y) else { return nil }
        return y
    }

    private func compute() -> (String, [GoodDay])? {
        guard let a = act else { return nil }
        let r: (CivilDate, CivilDate)?
        switch range {
        case .week: r = (today, today.adding((7 - today.weekday) % 7))
        case .month: r = (today, CivilDate(y: today.y, m: today.m, d: CivilDate.daysInMonth(today.y, today.m)))
        case .three: r = (today, today.adding(89))
        case .custom:
            if let f = CivilDate.parse(fromText), let t = CivilDate.parse(toText) { r = (f, t) } else { r = nil }
        }
        guard let (f, t) = r else { return ("Hãy nhập đủ ngày bắt đầu và ngày kết thúc (dạng ngày/tháng/năm).", []) }
        if !Lich.inRange(f) || !Lich.inRange(t) { return ("Lịch hỗ trợ từ năm \(Lich.minYear) đến năm \(Lich.maxYear).", []) }
        let span = t.dayNumber - f.dayNumber
        if span < 0 { return ("Ngày kết thúc cần sau ngày bắt đầu.", []) }
        if span > 366 { return ("Khoảng tìm tối đa là một năm. Hãy thu hẹp lại.", []) }
        switch Lich.findGoodDays(a, from: f, to: t, birthLunarYear: birthYear) {
        case .error(let m): return (m, [])
        case .ok(let list):
            let name = Lich.activities.first { $0.id == a }?.name ?? ""
            let tuoi = birthYear.map { ", tuổi \(Lich.canChi(year: $0))" } ?? ""
            return ("\(name), từ \(f.dmy) đến \(t.dmy)\(tuoi): \(list.isEmpty ? "chưa có ngày phù hợp" : "\(list.count) ngày phù hợp").", list)
        }
    }

    var body: some View {
        let res = compute()
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                Text("Chọn ngày tốt").serif(30, .semibold).foregroundStyle(C.ink).accessibilityAddTraits(.isHeader)
                P("Chọn việc định làm và khoảng thời gian. Ngày được xếp từ tốt nhất.")
                Panel {
                    H2("Việc định làm")
                    LazyVGrid(columns: [GridItem(.flexible(), spacing: 8), GridItem(.flexible(), spacing: 8)], spacing: 8) {
                        ForEach(Lich.activities) { a in
                            let sel = act == a.id
                            Button { act = a.id; shown = page } label: {
                                Text(a.name).sans(15, sel ? .semibold : .regular).multilineTextAlignment(.center)
                                    .foregroundStyle(sel ? .white : C.ink)
                                    .frame(maxWidth: .infinity, minHeight: 44).padding(.horizontal, 6)
                                    .background(sel ? C.redDeep : C.paper2, in: RoundedRectangle(cornerRadius: 12))
                            }
                            .buttonStyle(.plain)
                            .accessibilityAddTraits(sel ? .isSelected : [])
                        }
                    }
                    H2("Trong khoảng").padding(.top, 8)
                    Picker("Trong khoảng", selection: $range) {
                        ForEach(RangeKey.allCases) { Text($0.rawValue).tag($0) }
                    }
                    .pickerStyle(.segmented)
                    .labelsHidden()
                    .onChange(of: range) { _ in shown = page }
                    if range == .custom {
                        HStack {
                            TextField("Từ ngày (ngày/tháng/năm)", text: $fromText).textFieldStyle(.roundedBorder)
                            TextField("Đến ngày", text: $toText).textFieldStyle(.roundedBorder)
                        }
                    }
                    Text("Năm sinh âm lịch (không bắt buộc)").sans(15, .semibold).foregroundStyle(C.ink).padding(.top, 8)
                    HStack {
                        TextField("Ví dụ 1976", text: $birth)
                            .textFieldStyle(.roundedBorder)
                            #if os(iOS)
                            .keyboardType(.numberPad)
                            #endif
                            .frame(maxWidth: 160)
                            .onChange(of: birth) { t in
                                let d = String(t.filter(\.isNumber).prefix(4))
                                if d != t { birth = d }
                            }
                        if let y = birthYear { Text("Tuổi \(Lich.canChi(year: y))").serif(18).foregroundStyle(C.gold) }
                    }
                    Text("Nhập năm âm lịch của ngày sinh (năm tuổi). Ai sinh trước Tết Nguyên đán thì lấy năm trước, ví dụ sinh 20/01/1977 dương lịch (trước Tết Đinh Tỵ) là năm Bính Thìn 1976.").sans(12).foregroundStyle(C.ink3).fixedSize(horizontal: false, vertical: true)
                    Text("Năm sinh chỉ dùng để tránh ngày xung tuổi, không được lưu lại.").sans(12).foregroundStyle(C.ink3)
                }
                Text("Kết quả").serif(24, .semibold).foregroundStyle(C.ink).accessibilityAddTraits(.isHeader)
                if act == nil {
                    P("Chọn một việc ở trên để xem những ngày phù hợp.")
                } else if let (msg, list) = res {
                    P(msg)
                    if list.isEmpty && msg.contains("chưa có") {
                        Text("Hãy thử mở rộng khoảng thời gian, chẳng hạn 3 tháng tới.").sans(14).foregroundStyle(C.ink3)
                    }
                    ForEach(list.prefix(shown)) { g in GoodDayRow(g: g) { openDay(g.date) } }
                    if list.count > shown {
                        Button("Xem thêm \(min(page, list.count - shown)) ngày") { shown += page }
                            .buttonStyle(.bordered).tint(C.redInk).frame(maxWidth: .infinity)
                    }
                }
            }
            .padding(16)
            .frame(maxWidth: 720)
            .frame(maxWidth: .infinity)
        }
        .onAppear {
            if fromText.isEmpty { fromText = today.dmy; toText = today.adding(30).dmy }
        }
    }
}

private struct GoodDayRow: View {
    let g: GoodDay
    var action: () -> Void
    var body: some View {
        let s = g.date
        let wd = s.weekday
        let l = Lich.lunar(s)
        let info = Lich.dayInfo(s)
        let ld = Int(l?.d ?? 0), lm = Int(l?.m ?? 0), leap = l?.leap ?? false
        Button(action: action) {
            HStack(alignment: .top, spacing: 10) {
                VStack(spacing: 0) {
                    Text("\(s.d)").serif(30, .bold).foregroundStyle(wd == 0 ? C.red : C.ink)
                    Text("/\(s.m)").sans(12).foregroundStyle(C.ink3)
                }
                .frame(width: 54)
                VStack(alignment: .leading, spacing: 2) {
                    Text("\(Fmt.weekday[wd]), \(s.dmy)").sans(16, .semibold).foregroundStyle(C.ink)
                    Text("Âm lịch \(ld)/\(lm)\(leap ? " nhuận" : ""), ngày \(info?.can_chi.day ?? "")").sans(14).foregroundStyle(C.ink2)
                    if let f = info?.gio_hoang_dao.first {
                        Text("Giờ tốt đầu tiên: \(f.chi) (\(f.from)–\(f.to) giờ)").sans(14).foregroundStyle(C.ink2)
                    }
                    Text(g.reasons.joined(separator: "; ")).sans(12).foregroundStyle(C.green).fixedSize(horizontal: false, vertical: true)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                VStack(spacing: 0) {
                    Text("\(g.score)").serif(24, .bold).foregroundStyle(C.gold)
                    Text("điểm").sans(11).foregroundStyle(C.ink3)
                }
            }
            .padding(12)
            .background(C.paper, in: RoundedRectangle(cornerRadius: 14))
            .overlay(RoundedRectangle(cornerRadius: 14).stroke(C.rule))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("\(Fmt.weekday[wd]) \(s.dmy), âm lịch \(ld)/\(lm)\(leap ? " nhuận" : ""), điểm \(g.score). Mở tờ lịch ngày này")
        .accessibilityAddTraits(.isButton)
    }
}
