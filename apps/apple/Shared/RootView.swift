// Khung ứng dụng: thanh trên (cỡ chữ, sáng/tối), điều hướng ngày, 4 thẻ.
import SwiftUI

enum Tab: String, CaseIterable, Identifiable {
    case sheet = "Tờ lịch", good = "Chọn ngày tốt", convert = "Đổi ngày", people = "Người thân", about = "Giới thiệu"
    var id: String { rawValue }
    var symbol: String {
        switch self {
        case .sheet: return "calendar"
        case .good: return "star"
        case .convert: return "arrow.left.arrow.right"
        case .people: return "person.2"
        case .about: return "info.circle"
        }
    }
}

/// Chỉ lưu cỡ chữ và chế độ sáng/tối (UserDefaults, lý do CA92.1 trong PrivacyInfo.xcprivacy).
enum ThemeMode: Int { case system = 0, light = 1, dark = 2 }

struct RootView: View {
    @AppStorage("size") private var size = 1
    @AppStorage("theme") private var themeRaw = 0
    /// Cờ giao diện duy nhất ngoài cỡ chữ/sáng tối: hết hạn "Để sau" của thẻ nhắc cập nhật (giây, 1970).
    @AppStorage("updateSnoozedUntil") private var snoozedUntil: Double = 0
    @State private var tab: Tab = RootView.initialTab
    @State private var date = Lich.clamp(CivilDate.today())
    @State private var today = CivilDate.today()
    @State private var monthOpen = false
    @Environment(\.scenePhase) private var phase

    private let scales: [CGFloat] = [1, 1.15, 1.32]

    private static var initialTab: Tab {
        #if DEBUG
        if UserDefaults.standard.string(forKey: "lvhvScreen") != nil { return .people }
        #endif
        return .sheet
    }

    /// Mở ứng dụng / trở lại tiền cảnh / đổi múi giờ: lập lịch lại thông báo nếu đã có sổ người thân (02 §7.4).
    private func refreshPeople(_ reason: ReminderScheduler.Reason) {
        let m = PeopleModel.shared
        guard PeoplePaths.storeExists || m.opened else { return }
        m.openIfNeeded()
        m.reload()
        Task { await m.scheduler.reschedule(store: m.store, reason: reason) }
    }

    private var scheme: ColorScheme? {
        switch ThemeMode(rawValue: themeRaw) ?? .system {
        case .system: return nil
        case .light: return .light
        case .dark: return .dark
        }
    }

    private var showReminder: Bool {
        _ = today // tính lại khi quay lại ứng dụng (scenePhase cập nhật today)
        return AppUpdate.shouldRemind(
            buildDate: AppUpdate.buildDate, now: Date(),
            snoozedUntil: snoozedUntil > 0 ? Date(timeIntervalSince1970: snoozedUntil) : nil)
    }

    private func openDay(_ d: CivilDate) {
        date = Lich.clamp(d)
        tab = .sheet
    }

    private func step(_ n: Int) {
        let d = date.adding(n)
        if Lich.inRange(d) { withAnimation(.easeInOut(duration: 0.2)) { date = d } }
    }

    var body: some View {
        VStack(spacing: 0) {
            topBar
            if tab == .sheet { dayNav }
            if tab == .sheet && showReminder {
                UpdateReminderCard(
                    onCheck: { AppUpdate.openStore() },
                    onLater: { snoozedUntil = AppUpdate.snoozeUntil(from: Date()).timeIntervalSince1970 }
                )
            }
            content.frame(maxWidth: .infinity, maxHeight: .infinity)
            #if os(iOS)
            bottomBar
            #endif
        }
        .background(C.wall.ignoresSafeArea())
        .environment(\.textScale, scales[min(max(size, 1), 3) - 1])
        .preferredColorScheme(scheme)
        .tint(C.redDeep)
        .sheet(isPresented: $monthOpen) {
            MonthSheet(selected: date, today: today, onPick: { d in monthOpen = false; openDay(d) }, onClose: { monthOpen = false })
                .environment(\.textScale, scales[min(max(size, 1), 3) - 1])
                .preferredColorScheme(scheme)
                #if os(macOS)
                .frame(minWidth: 460, minHeight: 600)
                #endif
        }
        .onChange(of: phase) { p in
            if p == .active {
                today = CivilDate.today()
                refreshPeople(.appOpen)
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: .NSSystemTimeZoneDidChange)) { _ in refreshPeople(.timezone) }
        .onReceive(NotificationCenter.default.publisher(for: NotificationDelegate.openPeople)) { _ in tab = .people }
        #if os(macOS)
        .frame(minWidth: 420, minHeight: 640)
        #endif
    }

    private var topBar: some View {
        HStack(spacing: 8) {
            #if os(macOS)
            Picker("", selection: $tab) {
                ForEach(Tab.allCases) { t in Label(t.rawValue, systemImage: t.symbol).tag(t) }
            }
            .pickerStyle(.segmented)
            .labelsHidden()
            .frame(maxWidth: 680)
            #else
            Text("Lịch Việt").serif(20, .semibold).foregroundStyle(C.ink).lineLimit(1).minimumScaleFactor(0.6)
            #endif
            Spacer()
            Picker("Cỡ chữ", selection: $size) {
                Text("A").tag(1)
                Text("A+").tag(2)
                Text("A++").tag(3)
            }
            .pickerStyle(.segmented)
            .frame(width: 150)
            .accessibilityLabel("Cỡ chữ")
            Button {
                themeRaw = themeRaw == 0 ? 2 : themeRaw == 2 ? 1 : 0
            } label: {
                Image(systemName: "circle.lefthalf.filled").font(.title3).frame(width: 40, height: 40)
                    .background(C.paper, in: Circle())
            }
            .buttonStyle(.plain)
            .foregroundStyle(C.ink)
            .accessibilityLabel(themeRaw == 0 ? "Giao diện theo máy. Chạm để chuyển sang tối" : themeRaw == 2 ? "Giao diện tối. Chạm để chuyển sang sáng" : "Giao diện sáng. Chạm để theo máy")
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
    }

    private var dayNav: some View {
        HStack {
            NavCircle(system: "chevron.left", label: "Ngày trước", enabled: date > Lich.minDate) { step(-1) }
                .keyboardShortcut(.leftArrow, modifiers: [])
            Spacer()
            Button { withAnimation { date = Lich.clamp(today) } } label: {
                Text("Hôm nay").sans(17, .semibold).foregroundStyle(.white)
                    .padding(.horizontal, 28).padding(.vertical, 11)
                    .background(C.redDeep.opacity(date == today ? 0.45 : 1), in: Capsule())
            }
            .buttonStyle(.plain)
            .disabled(date == today)
            .keyboardShortcut("t", modifiers: [.command])
            Spacer()
            NavCircle(system: "chevron.right", label: "Ngày sau", enabled: date < Lich.maxDate) { step(1) }
                .keyboardShortcut(.rightArrow, modifiers: [])
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 4)
    }

    @ViewBuilder
    private var content: some View {
        switch tab {
        case .sheet:
            SheetView(date: date, today: today) { monthOpen = true }
                .id(date)
                .transition(.opacity)
                .gesture(
                    DragGesture(minimumDistance: 30)
                        .onEnded { v in
                            guard abs(v.translation.width) > abs(v.translation.height) * 1.5 else { return }
                            step(v.translation.width < 0 ? 1 : -1)
                        }
                )
        case .good:
            GoodDaysView(today: today, openDay: openDay)
        case .convert:
            ConvertView(current: date, today: today, openDay: openDay)
        case .people:
            PeopleView()
        case .about:
            AboutView()
        }
    }

    #if os(iOS)
    private var bottomBar: some View {
        HStack(spacing: 0) {
            ForEach(Tab.allCases) { t in
                Button { tab = t } label: {
                    VStack(spacing: 4) {
                        Image(systemName: t.symbol).font(.title3)
                        Text(t.rawValue).sans(12, .semibold).multilineTextAlignment(.center).lineLimit(2)
                    }
                    .foregroundStyle(tab == t ? C.redInk : C.ink2)
                    .frame(maxWidth: .infinity, minHeight: 54)
                    .background(tab == t ? C.redTint : .clear, in: RoundedRectangle(cornerRadius: 14))
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .accessibilityAddTraits(tab == t ? .isSelected : [])
            }
        }
        .padding(.horizontal, 6)
        .padding(.top, 6)
        .background(C.paper.ignoresSafeArea(edges: .bottom))
    }
    #endif
}
