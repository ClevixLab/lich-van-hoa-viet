// Đổi ngày dương ↔ âm.
import SwiftUI

struct ConvertView: View {
    let current: CivilDate
    let today: CivilDate
    var openDay: (CivilDate) -> Void

    @State private var sText = ""
    @State private var ld = 1
    @State private var lm = 1
    @State private var lyText = ""
    @State private var leap = false
    @State private var initialised = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                Text("Đổi ngày").serif(30, .semibold).foregroundStyle(C.ink).accessibilityAddTraits(.isHeader)
                P("Đổi qua lại giữa dương lịch và âm lịch, từ năm \(Lich.minYear) đến năm \(Lich.maxYear).")
                Panel {
                    H2("Dương lịch sang âm lịch")
                    TextField("Ngày dương lịch (ngày/tháng/năm)", text: $sText).textFieldStyle(.roundedBorder)
                    if let s = CivilDate.parse(sText), Lich.inRange(s), let info = Lich.dayInfo(s) {
                        Text("\(info.lunar.d) tháng \(Fmt.lunarMonth[info.lunar.m])\(info.lunar.leap ? " (nhuận)" : "")").serif(26).foregroundStyle(C.ink)
                        Text("Năm \(info.can_chi.year) · ngày \(info.can_chi.day) · tháng \(info.can_chi.month)").sans(16).foregroundStyle(C.ink2)
                        Button("Mở tờ lịch \(Fmt.weekday[s.weekday]), \(s.dmy)") { openDay(s) }.buttonStyle(.plain).foregroundStyle(C.redInk)
                    } else {
                        Text("Hãy nhập một ngày dương lịch từ năm \(Lich.minYear) đến năm \(Lich.maxYear).").sans(15).foregroundStyle(C.redInk)
                    }
                }
                Panel {
                    H2("Âm lịch sang dương lịch")
                    HStack {
                        Picker("Ngày", selection: $ld) { ForEach(1...30, id: \.self) { Text(Fmt.lunarDayLabel($0)).tag($0) } }
                        Picker("Tháng", selection: $lm) { ForEach(1...12, id: \.self) { Text("Tháng \($0) (\(Fmt.lunarMonth[$0]))").tag($0) } }
                        TextField("Năm", text: $lyText)
                            .textFieldStyle(.roundedBorder)
                            #if os(iOS)
                            .keyboardType(.numberPad)
                            #endif
                            .frame(maxWidth: 100)
                            .accessibilityLabel("Năm âm lịch")
                    }
                    Toggle("Tháng nhuận", isOn: $leap)
                    result
                }
                Text("Hôm nay theo máy của bạn: \(today.dmy).").sans(12).foregroundStyle(C.ink3)
            }
            .padding(16)
            .frame(maxWidth: 720)
            .frame(maxWidth: .infinity)
        }
        .onAppear {
            guard !initialised else { return }
            initialised = true
            sText = current.dmy
            if let l = Lich.lunar(Lich.clamp(current)) {
                ld = Int(l.d); lm = Int(l.m); lyText = String(l.y); leap = l.leap
            }
        }
    }

    @ViewBuilder
    private var result: some View {
        if let ly = Int(lyText), (Lich.minYear...Lich.maxYear).contains(ly) {
            if let s = Lich.toSolar(ly: ly, lm: lm, ld: ld, leap: leap) {
                Text("\(Fmt.weekday[s.weekday]), \(s.dmy)").serif(26).foregroundStyle(C.ink)
                Button("Mở tờ lịch ngày này") { openDay(s) }.buttonStyle(.plain).foregroundStyle(C.redInk)
            } else {
                let plain = leap ? Lich.toSolar(ly: ly, lm: lm, ld: ld, leap: false) : nil
                Text(leap
                     ? "Năm \(String(ly)) không có tháng \(lm) nhuận\(plain != nil ? "" : ", hoặc tháng này không có ngày \(ld)"). Hãy bỏ chọn \"Tháng nhuận\" hoặc chọn tháng khác."
                     : "Tháng \(lm) âm lịch năm \(String(ly)) không có ngày \(ld) (tháng thiếu chỉ có 29 ngày).")
                    .sans(15).foregroundStyle(C.redInk)
            }
        } else {
            Text("Hãy nhập năm âm lịch từ \(Lich.minYear) đến \(Lich.maxYear).").sans(15).foregroundStyle(C.redInk)
        }
    }
}
