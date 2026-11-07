// Màn hình tưởng niệm (08 §3.9, R8, R9): bia, ảnh, bàn thờ, ba nén hương. Không âm thanh, không chia sẻ.
// Khói chạy tối đa 2 phút rồi đứng yên; dừng khi rời màn hình hoặc ứng dụng vào nền; Giảm chuyển động → tĩnh ngay.
// Không hiện thông tin báo tử (R-PRIV-2); tiêu đề không có tên người.
import SwiftUI

private enum MC {
    static let bg = Color(red: 0.11, green: 0.08, blue: 0.08)
    static let stone = Color(red: 0.169, green: 0.141, blue: 0.133) // #2b2422
    static let gold = Color(red: 0.91, green: 0.776, blue: 0.478)   // #e8c67a
    static let brass = Color(red: 0.79, green: 0.62, blue: 0.32)
    static let wood = Color(red: 0.35, green: 0.22, blue: 0.14)
    static let woodLight = Color(red: 0.47, green: 0.31, blue: 0.19)
    static let ink = Color(red: 0.95, green: 0.91, blue: 0.86)
}

/// Bia: đỉnh vòm.
struct SteleShape: Shape {
    func path(in r: CGRect) -> Path {
        var p = Path()
        let rad = min(r.width / 2, 70)
        p.move(to: CGPoint(x: r.minX, y: r.maxY))
        p.addLine(to: CGPoint(x: r.minX, y: r.minY + rad))
        p.addArc(center: CGPoint(x: r.minX + rad, y: r.minY + rad), radius: rad, startAngle: .degrees(180),
                 endAngle: .degrees(270), clockwise: false)
        p.addLine(to: CGPoint(x: r.maxX - rad, y: r.minY))
        p.addArc(center: CGPoint(x: r.maxX - rad, y: r.minY + rad), radius: rad, startAngle: .degrees(270),
                 endAngle: .degrees(0), clockwise: false)
        p.addLine(to: CGPoint(x: r.maxX, y: r.maxY))
        p.closeSubpath()
        return p
    }
}

struct MemorialView: View {
    let id: String
    @ObservedObject private var m = PeopleModel.shared
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.scenePhase) private var phase
    @Environment(\.dismiss) private var dismiss
    /// Thời điểm khói còn chạy tới (nil = đang đứng yên).
    @State private var smokeUntil: Date?

    static let smokeSeconds: TimeInterval = 120

    var body: some View {
        Group {
            if let p = m.person(id), p["kind"].str == "deceased" {
                content(p)
            } else {
                Text("Không tìm thấy hồ sơ này (có thể đã bị xóa).").ps(16).foregroundStyle(MC.ink).pageWidth()
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(MC.bg.ignoresSafeArea())
        .navigationTitle("Tưởng niệm")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        .toolbarBackground(MC.bg, for: .navigationBar)
        .toolbarBackground(.visible, for: .navigationBar)
        .toolbarColorScheme(.dark, for: .navigationBar)
        #endif
        // Nút quay lại và tiêu đề màu vàng đồng trên nền gỗ, ở cả giao diện sáng và tối (TKN-I-03).
        .navigationBarBackButtonHidden(true)
        .toolbar {
            ToolbarItem(placement: .navigation) {
                Button { dismiss() } label: {
                    HStack(spacing: 4) {
                        Image(systemName: "chevron.left").font(.body.weight(.semibold))
                        Text("Về hồ sơ").ps(17)
                    }
                    .foregroundStyle(MC.gold)
                }
                .accessibilityIdentifier("memorial.back")
            }
            ToolbarItem(placement: .principal) {
                Text("Tưởng niệm").ps(17, .semibold).foregroundStyle(MC.gold)
            }
        }
        .onAppear { light() }
        .onDisappear { smokeUntil = nil }
        .onChange(of: phase) { ph in if ph != .active { smokeUntil = nil } }
    }

    private var incense: Bool {
        guard let p = m.person(id) else { return false }
        return p["memorial"]["tray_style"].str != "cong_giao" && p["memorial"]["incense"].bool != false
    }

    private func light() {
        smokeUntil = (reduceMotion || !incense) ? nil : Date().addingTimeInterval(Self.smokeSeconds)
    }

    private func content(_ p: JV) -> some View {
        ScrollView {
            VStack(spacing: 18) {
                Text(dateLine).ps(16, .semibold).foregroundStyle(MC.gold).multilineTextAlignment(.center)
                    .fixedSize(horizontal: false, vertical: true)
                    .accessibilityIdentifier("memorial.date")
                if incense && !reduceMotion {
                    Button(smokeUntil == nil ? "Thắp lại" : "Dừng khói") {
                        if smokeUntil == nil { light() } else { smokeUntil = nil }
                    }
                    .buttonStyle(.plain)
                    .ps(15, .semibold)
                    .foregroundStyle(MC.gold)
                    .padding(.horizontal, 18)
                    .frame(minHeight: 44)
                    .overlay(Capsule().stroke(MC.brass))
                    .accessibilityIdentifier("memorial.smoke")
                }
                PhotoView(photoId: p["photo_id"].str, initials: PFmt.initials(p["ho_ten"].string),
                          size: CGSize(width: 128, height: 160), dark: true)
                stele(p)
                altar
                let loi = p["memorial"]["loi_tuong_niem"].string
                if !loi.isEmpty {
                    Text("“\(loi)”").serif(18).foregroundStyle(MC.ink).multilineTextAlignment(.center)
                        .fixedSize(horizontal: false, vertical: true)
                        .padding(.horizontal, 12)
                }
            }
            .padding(.vertical, 20)
            .padding(.horizontal, 16)
            .frame(maxWidth: 520)
            .frame(maxWidth: .infinity)
        }
        .environment(\.colorScheme, .dark)
    }

    private var dateLine: String {
        guard let g = m.nextGio(for: id) else { return "" }
        let lunar = g.lunar.map { "\($0.d)/\($0.m) âm lịch" } ?? ""
        if g.civil == m.today {
            let label: String
            switch g.label {
            case "gio_dau": label = "Giỗ đầu"
            case "gio_het": label = "Giỗ hết"
            default: label = "Ngày giỗ"
            }
            return "\(label) · hôm nay \(lunar)"
        }
        return "Ngày giỗ sắp tới: \(lunar), \(g.civil.dmy)"
    }

    private func stele(_ p: JV) -> some View {
        let rel = PFmt.steleRelation(p["quan_he"].string)
        let name = rel.isEmpty ? PFmt.displayName(p) : p["ho_ten"].string
        let long = name.count > 28
        return VStack(spacing: 8) {
            if !rel.isEmpty {
                Text(rel).serif(18, .semibold).foregroundStyle(MC.gold).accessibilityIdentifier("memorial.relation")
            }
            Text(name).serif(long ? 20 : 26, .bold).foregroundStyle(MC.gold).multilineTextAlignment(.center)
                .fixedSize(horizontal: false, vertical: true)
                .accessibilityIdentifier("memorial.name")
            if !p["ten_hieu"].string.isEmpty {
                Text(p["ten_hieu"].string).serif(16).foregroundStyle(MC.gold.opacity(0.9))
            }
            Text(yearsLine(p)).serif(16).foregroundStyle(MC.gold).multilineTextAlignment(.center)
                .fixedSize(horizontal: false, vertical: true)
            if let t = m.summaries[id]?.tuoi_mat_text {
                Text(t).serif(16, .semibold).foregroundStyle(MC.gold)
            }
        }
        .padding(.top, 40)
        .padding(.bottom, 24)
        .padding(.horizontal, 18)
        .frame(maxWidth: 360)
        .frame(maxWidth: .infinity)
        .background(SteleShape().fill(MC.stone))
        .overlay(SteleShape().stroke(MC.brass, lineWidth: 2))
        .padding(.horizontal, 24)
        .accessibilityElement(children: .combine)
    }

    private func yearsLine(_ p: JV) -> String {
        let d = p["death"]["lunar"]
        guard let dy = d["year"].int else { return "Giỗ ngày \(PFmt.gioText(d))" }
        let death = "\(Lich.canChi(year: dy)) \(dy)"
        if let by = p["birth"]["lunar_year"].int {
            return "\(p["birth"]["can_chi"].str ?? Lich.canChi(year: by)) \(by) — \(death)"
        }
        return "Mất năm \(Lich.canChi(year: dy)) (\(dy))"
    }

    private var altar: some View {
        VStack(spacing: 0) {
            HStack(alignment: .bottom, spacing: 22) {
                candle
                if incense {
                    VStack(spacing: 0) {
                        sticks
                        bowl
                    }
                } else {
                    flowers
                }
                fruit
                candle
            }
            Rectangle().fill(MC.woodLight).frame(height: 10)
            Rectangle().fill(MC.wood).frame(height: 26)
        }
        .frame(maxWidth: 380)
        .accessibilityElement()
        .accessibilityLabel(incense ? "Bàn thờ: bát hương với ba nén hương, mâm ngũ quả, hai cây nến" : "Bàn thờ: hoa, mâm quả, hai cây nến")
    }

    private var sticks: some View {
        TimelineView(.animation(minimumInterval: 1.0 / 20, paused: smokeUntil == nil)) { tl in
            let t = tl.date.timeIntervalSinceReferenceDate
            let running = smokeUntil.map { tl.date < $0 } ?? false
            HStack(alignment: .bottom, spacing: 12) {
                ForEach(0..<3, id: \.self) { i in
                    ZStack(alignment: .bottom) {
                        smoke(i: i, t: t, running: running)
                        VStack(spacing: 0) {
                            Circle().fill(Color.orange).frame(width: 5, height: 5)
                            Rectangle().fill(Color(red: 0.75, green: 0.35, blue: 0.25)).frame(width: 3, height: 70)
                        }
                    }
                }
            }
            .frame(height: 150, alignment: .bottom)
        }
    }

    /// Một vệt khói: ba cụm mờ trôi lên rồi tan (đứng yên khi khói dừng).
    private func smoke(i: Int, t: Double, running: Bool) -> some View {
        ZStack {
            ForEach(0..<3, id: \.self) { k in
                let phase = running ? (t / 3.2 + Double(i) * 0.31 + Double(k) / 3).truncatingRemainder(dividingBy: 1) : Double(k) / 3 + 0.15
                Ellipse()
                    .fill(Color.white.opacity(0.16 * (1 - phase)))
                    .frame(width: 8 + 14 * phase, height: 18 + 12 * phase)
                    .blur(radius: 3 + 4 * phase)
                    .offset(x: sin((phase * 2 + Double(i)) * .pi) * 6, y: -78 - 64 * phase)
            }
        }
        .frame(width: 30, height: 150, alignment: .bottom)
        .allowsHitTesting(false)
    }

    private var bowl: some View {
        ZStack(alignment: .top) {
            UnevenBowl().fill(MC.brass).frame(width: 78, height: 36)
            Ellipse().fill(Color(red: 0.55, green: 0.42, blue: 0.22)).frame(width: 78, height: 10)
        }
    }

    private var candle: some View {
        VStack(spacing: 0) {
            Ellipse().fill(Color.yellow.opacity(0.9)).frame(width: 8, height: 14)
            Rectangle().fill(Color(red: 0.86, green: 0.2, blue: 0.16)).frame(width: 12, height: 46)
            Rectangle().fill(MC.brass).frame(width: 22, height: 6)
        }
    }

    private var fruit: some View {
        VStack(spacing: -6) {
            HStack(spacing: -4) {
                Circle().fill(Color(red: 0.95, green: 0.6, blue: 0.15)).frame(width: 20, height: 20)
                Circle().fill(Color(red: 0.45, green: 0.65, blue: 0.25)).frame(width: 20, height: 20)
            }
            HStack(spacing: -4) {
                Circle().fill(Color(red: 0.85, green: 0.25, blue: 0.2)).frame(width: 20, height: 20)
                Circle().fill(Color(red: 0.98, green: 0.82, blue: 0.3)).frame(width: 20, height: 20)
                Circle().fill(Color(red: 0.95, green: 0.6, blue: 0.15)).frame(width: 20, height: 20)
            }
            Ellipse().fill(MC.brass).frame(width: 78, height: 14)
        }
    }

    private var flowers: some View {
        VStack(spacing: 0) {
            HStack(spacing: 2) {
                ForEach(0..<3, id: \.self) { _ in Circle().fill(Color.white.opacity(0.9)).frame(width: 14, height: 14) }
            }
            Rectangle().fill(Color.green.opacity(0.7)).frame(width: 3, height: 40)
            RoundedRectangle(cornerRadius: 4).fill(MC.brass).frame(width: 30, height: 30)
        }
    }
}

/// Bát hương: miệng rộng, đáy thu lại.
struct UnevenBowl: Shape {
    func path(in r: CGRect) -> Path {
        var p = Path()
        p.move(to: CGPoint(x: r.minX, y: r.minY + 5))
        p.addLine(to: CGPoint(x: r.maxX, y: r.minY + 5))
        p.addQuadCurve(to: CGPoint(x: r.maxX - 12, y: r.maxY), control: CGPoint(x: r.maxX, y: r.maxY - 4))
        p.addLine(to: CGPoint(x: r.minX + 12, y: r.maxY))
        p.addQuadCurve(to: CGPoint(x: r.minX, y: r.minY + 5), control: CGPoint(x: r.minX, y: r.maxY - 4))
        p.closeSubpath()
        return p
    }
}
