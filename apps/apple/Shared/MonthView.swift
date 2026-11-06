// Bảng tháng "Chọn ngày": Thứ Hai đầu tuần, ngày dương lớn, ngày âm nhỏ; mùng 1 và rằm nổi bật.
import SwiftUI

struct YM: Hashable {
    var y: Int
    var m: Int
    func shifted(_ n: Int) -> YM {
        let t = y * 12 + (m - 1) + n
        let yy = t / 12
        if yy < Lich.minYear { return YM(y: Lich.minYear, m: 1) }
        if yy > Lich.maxYear { return YM(y: Lich.maxYear, m: 12) }
        return YM(y: yy, m: t % 12 + 1)
    }
}

struct MonthSheet: View {
    let selected: CivilDate
    let today: CivilDate
    var onPick: (CivilDate) -> Void
    var onClose: () -> Void
    @State private var view: YM

    init(selected: CivilDate, today: CivilDate, onPick: @escaping (CivilDate) -> Void, onClose: @escaping () -> Void) {
        self.selected = selected
        self.today = today
        self.onPick = onPick
        self.onClose = onClose
        _view = State(initialValue: YM(y: selected.y, m: selected.m))
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                HStack {
                    Text("Chọn ngày").serif(26, .semibold).foregroundStyle(C.ink)
                    Spacer()
                    Button(action: onClose) { Image(systemName: "xmark").font(.title3) }
                        .buttonStyle(.plain).foregroundStyle(C.ink).accessibilityLabel("Đóng")
                }
                MonthGrid(view: $view, selected: selected, today: today, onPick: onPick)
                Button { onPick(today) } label: {
                    Text("Về hôm nay").sans(17, .semibold).padding(.horizontal, 28).padding(.vertical, 12)
                        .foregroundStyle(.white).background(C.redDeep, in: Capsule())
                }
                .buttonStyle(.plain)
                .frame(maxWidth: .infinity)
            }
            .padding(20)
        }
        .background(C.paper)
    }
}

struct MonthGrid: View {
    @Binding var view: YM
    let selected: CivilDate
    let today: CivilDate
    var onPick: (CivilDate) -> Void
    @State private var yearText = ""

    var body: some View {
        let lunar = Lich.monthLunar(view.y, view.m)
        let first = CivilDate(y: view.y, m: view.m, d: 1)
        let lead = (first.weekday + 6) % 7
        let n = CivilDate.daysInMonth(view.y, view.m)
        VStack(spacing: 8) {
            HStack(spacing: 8) {
                NavCircle(system: "chevron.left", label: "Tháng trước", enabled: !(view.y == Lich.minYear && view.m == 1)) { view = view.shifted(-1) }
                Picker("Tháng", selection: $view.m) {
                    ForEach(1...12, id: \.self) { Text("Tháng \($0)").tag($0) }
                }
                .labelsHidden()
                .frame(maxWidth: .infinity)
                TextField("Năm", text: $yearText)
                    .multilineTextAlignment(.center)
                    .textFieldStyle(.roundedBorder)
                    #if os(iOS)
                    .keyboardType(.numberPad)
                    #endif
                    .frame(maxWidth: 110)
                    .accessibilityLabel("Năm")
                    .onChange(of: yearText) { t in
                        let digits = String(t.filter(\.isNumber).prefix(4))
                        if digits != t { yearText = digits }
                        if let y = Int(digits), (Lich.minYear...Lich.maxYear).contains(y), y != view.y { view.y = y }
                    }
                NavCircle(system: "chevron.right", label: "Tháng sau", enabled: !(view.y == Lich.maxYear && view.m == 12)) { view = view.shifted(1) }
            }
            .onAppear { yearText = String(view.y) }
            .onChange(of: view) { v in if yearText != String(v.y) { yearText = String(v.y) } }

            let cols = Array(repeating: GridItem(.flexible(), spacing: 4), count: 7)
            LazyVGrid(columns: cols, spacing: 4) {
                ForEach([1, 2, 3, 4, 5, 6, 0], id: \.self) { w in
                    Text(Fmt.weekdayShort[w]).sans(14, .semibold).foregroundStyle(w == 0 ? C.red : C.ink3)
                        .accessibilityLabel(Fmt.weekday[w])
                }
                ForEach(0..<(lead + n), id: \.self) { i in
                    if i < lead {
                        Color.clear.frame(height: 52)
                    } else {
                        let d = i - lead + 1
                        let date = CivilDate(y: view.y, m: view.m, d: d)
                        dayCell(date: date, col: i % 7, lunar: d - 1 < lunar.count ? lunar[d - 1] : nil)
                    }
                }
            }
            HStack(spacing: 8) {
                RoundedRectangle(cornerRadius: 3).fill(C.goldTint).overlay(RoundedRectangle(cornerRadius: 3).stroke(C.gold)).frame(width: 14, height: 14)
                Text("Mùng 1 và ngày rằm âm lịch").sans(13).foregroundStyle(C.ink3)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    @ViewBuilder
    private func dayCell(date: CivilDate, col: Int, lunar l: LunarDay?) -> some View {
        let ld = Int(l?.d ?? 0), lm = Int(l?.m ?? 0), leap = l?.leap ?? false
        let key = ld == 1 || ld == 15
        let sel = date == selected
        let isToday = date == today
        let sunday = col == 6
        let lunarTxt = ld == 1 ? "\(ld)/\(lm)\(leap ? "n" : "")" : "\(ld)"
        Button { onPick(date) } label: {
            VStack(spacing: 0) {
                Text("\(date.d)").serif(20, .semibold).foregroundStyle(sel ? .white : sunday ? C.red : C.ink)
                Text(lunarTxt).sans(12, key ? .bold : .regular).foregroundStyle(sel ? .white : key ? C.redInk : C.ink3)
            }
            .frame(maxWidth: .infinity, minHeight: 52)
            .background(sel ? C.redDeep : key ? C.goldTint : .clear, in: RoundedRectangle(cornerRadius: 10))
            .overlay(RoundedRectangle(cornerRadius: 10).stroke(isToday && !sel ? C.red : .clear, lineWidth: 1.5))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel("\(Fmt.weekday[(col + 1) % 7]) \(date.d) tháng \(date.m), âm lịch \(ld) tháng \(lm)\(leap ? " nhuận" : "")\(isToday ? ", hôm nay" : "")")
        .accessibilityAddTraits(sel ? .isSelected : [])
    }
}

struct NavCircle: View {
    let system: String
    let label: String
    var enabled = true
    var action: () -> Void
    var body: some View {
        Button(action: action) {
            Image(systemName: system).font(.body.weight(.semibold)).frame(width: 44, height: 44)
                .background(C.paper, in: Circle())
                .overlay(Circle().stroke(C.rule))
        }
        .buttonStyle(.plain)
        .foregroundStyle(C.ink)
        .disabled(!enabled)
        .opacity(enabled ? 1 : 0.4)
        .accessibilityLabel(label)
    }
}
