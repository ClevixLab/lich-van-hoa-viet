// Mô-đun "Người thân" (08 §1): bốn trang con Sắp tới · Theo kỳ · Hồ sơ · Sao lưu, và các trang mở từ đó.
import SwiftUI
#if os(iOS)
import UIKit
#endif

enum PSection: String, CaseIterable, Identifiable {
    case upcoming = "Sắp tới", period = "Theo kỳ", profiles = "Hồ sơ", backup = "Sao lưu"
    var id: String { rawValue }
}

/// Trang không có trên thanh phân đoạn. Chỉ chứa id ngẫu nhiên — không tên, không ngày (S4').
enum PRoute: Hashable {
    case person(String)
    case add(String)            // "deceased" | "living"
    case edit(String)
    case event(String, String?) // id người, id sự kiện (nil = thêm)
    case markDeceased(String)
    case memorial(String)
    case editTray(String)       // sửa hồ sơ, mở sẵn mục "Ảnh mâm lễ ngày giỗ" (08 §16.1)
    case settings
}

struct PeopleView: View {
    @ObservedObject private var m = PeopleModel.shared
    @State private var section: PSection = .upcoming
    @State private var path: [PRoute] = []
    @State private var started = false

    var body: some View {
        NavigationStack(path: $path) {
            root
                .navigationTitle("Người thân")
                .navigationDestination(for: PRoute.self) { r in destination(r) }
                #if os(iOS)
                .toolbar(.hidden, for: .navigationBar)
                #endif
        }
        .onAppear {
            m.openIfNeeded()
            guard !started else { return }
            started = true
            #if DEBUG
            if let link = DemoSeed.deepLink(m) { section = link.0; path = link.1 }
            #endif
        }
    }

    private var root: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                HStack(alignment: .top) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Người thân").serif(30, .semibold).foregroundStyle(C.ink).accessibilityAddTraits(.isHeader)
                        PBody("Ngày giỗ, sinh nhật và sự kiện của gia đình. Dữ liệu chỉ nằm trên máy này.")
                    }
                    Spacer(minLength: 8)
                    NavigationLink(value: PRoute.settings) {
                        Image(systemName: "bell.badge").font(.title3).frame(width: 44, height: 44)
                            .background(C.paper, in: Circle())
                    }
                    .buttonStyle(.plain)
                    .foregroundStyle(C.ink)
                    .accessibilityLabel("Cài đặt nhắc")
                    .accessibilityIdentifier("people.settings")
                }
                SegmentChips(options: PSection.allCases.map { SegOpt($0, $0.rawValue) }, selection: $section,
                             idPrefix: "people.sections")
                ToastBar(text: $m.toast)
                if m.readOnly { readOnlyBanner }
                if let e = m.openError {
                    PCard {
                        Text("Không mở được dữ liệu trên máy: \(e). Lịch vẫn dùng được bình thường.")
                            .ps(16).foregroundStyle(C.redInk).fixedSize(horizontal: false, vertical: true)
                    }
                } else if !m.opened {
                    PBody("Đang mở sổ người thân…")
                } else if DemoSeed.isDiag {
                    UpgradeDiagView()
                } else {
                    switch section {
                    case .upcoming: UpcomingSection(section: $section)
                    case .period: PeriodSection()
                    case .profiles: ProfilesSection()
                    case .backup: BackupSection()
                    }
                }
            }
            .pageWidth(960)
        }
        .background(C.wall)
    }

    /// Chuyển đổi dữ liệu lỗi (01 §7.1, B5): câu chuẩn của mọi nền tảng (08 §12l TKN3-X-03, "mở lại ứng dụng" thay cho
    /// "tải lại trang"); câu của lõi chỉ ở mục "Chi tiết kỹ thuật" đóng sẵn (DV23-02).
    private var readOnlyBanner: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Image(systemName: "lock.fill").foregroundStyle(C.redInk).accessibilityHidden(true)
                (Text("Chưa chuyển được dữ liệu sang bản mới.").bold()
                 + Text(" Dữ liệu cũ vẫn còn nguyên trên máy và đang mở ở chế độ chỉ đọc: xem được, nhưng chưa thêm hay sửa được."))
                    .ps(16).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
                    .accessibilityIdentifier("people.readOnly.text")
            }
            Text("Hãy tạo ngay một tệp sao lưu để giữ dữ liệu: vào Sao lưu → “Tạo tệp sao lưu…”, rồi cất tệp ở nơi an toàn. Sau đó thử mở lại ứng dụng; nếu vẫn thấy thông báo này, hãy báo cho Clevix Lab kèm tệp sao lưu.")
                .ps(15).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
            Button("Đi tới Sao lưu") { path.removeAll(); section = .backup }
                .buttonStyle(PrimaryButtonStyle())
                .accessibilityIdentifier("people.readOnly.backup")
            if let msg = m.migration?.message, !msg.isEmpty {
                DisclosureGroup("Chi tiết kỹ thuật") {
                    Text(msg).ps(13).foregroundStyle(C.ink2).fixedSize(horizontal: false, vertical: true)
                        .textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .ps(14)
                .tint(C.ink2)
                .accessibilityIdentifier("people.readOnly.details")
            }
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(C.redTint, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("people.readOnly")
    }

    @ViewBuilder
    private func destination(_ r: PRoute) -> some View {
        Group {
            switch r {
            case .person(let id): PersonDetailView(id: id, path: $path)
            // Chế độ chỉ đọc (08 §12l TKN3-P-01): không dựng biểu mẫu ghi.
            case .add where m.readOnly, .edit where m.readOnly, .editTray where m.readOnly, .event where m.readOnly,
                 .markDeceased where m.readOnly:
                ReadOnlyBlockedView()
            case .add(let kind): PersonFormView(kind: kind, editId: nil, path: $path)
            case .edit(let id): PersonFormView(kind: m.person(id)?["kind"].str ?? "living", editId: id, path: $path)
            case .editTray(let id): PersonFormView(kind: "deceased", editId: id, path: $path, focusTray: true)
            case .event(let pid, let eid): EventFormView(personId: pid, eventId: eid, path: $path)
            case .markDeceased(let id): MarkDeceasedView(id: id, path: $path)
            case .memorial(let id): MemorialView(id: id)
            case .settings: PeopleSettingsView()
            }
        }
        .background(C.wall)
    }
}

// MARK: - Sắp tới (08 §3.1)

struct UpcomingSection: View {
    @Binding var section: PSection
    @ObservedObject private var m = PeopleModel.shared
    @ObservedObject private var sched = PeopleModel.shared.scheduler
    @State private var dismissed: Set<String> = []

    var body: some View {
        let today = m.today
        Group {
            if m.persons.isEmpty {
                welcome
            } else {
                ViewThatFits(in: .horizontal) {
                    HStack(alignment: .top, spacing: 16) {
                        VStack(alignment: .leading, spacing: 14) {
                            reminderCards(today)
                            list(today)
                        }
                        VStack(alignment: .leading, spacing: 14) {
                            if m.backupDue { backupNag }
                            if sched.permission == .denied { permissionCard }
                            PCard { addButtons }
                        }
                        .frame(width: 300)
                    }
                    .frame(minWidth: 740)
                    VStack(alignment: .leading, spacing: 14) {
                        reminderCards(today)
                        if m.backupDue { backupNag }
                        if sched.permission == .denied { permissionCard }
                        list(today)
                        addButtons
                    }
                }
            }
        }
        .task { await sched.refreshPermission() }
    }

    @ViewBuilder
    private func list(_ today: CivilDate) -> some View {
        VStack(alignment: .leading, spacing: 14) {
                PHeading("30 ngày tới")
                if m.upcoming.isEmpty {
                    PCard {
                        PBody("Trong 30 ngày tới không có ngày giỗ, sinh nhật hay sự kiện nào.")
                        Button("Xem theo tháng, quý, năm") { section = .period }
                            .buttonStyle(.plain).ps(16, .semibold).foregroundStyle(C.redInk)
                            .frame(minHeight: 44)
                    }
                } else {
                    LazyVStack(alignment: .leading, spacing: 10) {
                        ForEach(m.upcoming) { it in
                            VStack(alignment: .leading, spacing: 6) {
                                if let pid = it.personId {
                                    NavigationLink(value: PRoute.person(pid)) {
                                        OccRow(item: it, today: today, extra: extra(it), chevron: true)
                                    }
                                    .buttonStyle(.plain)
                                } else {
                                    OccRow(item: it, today: today, extra: extra(it))
                                }
                                if it.class == "gio" && it.civil == today {
                                    NavigationLink(value: PRoute.memorial(it.entity_id)) {
                                        Text("Mở trang tưởng niệm")
                                    }
                                    .buttonStyle(PrimaryButtonStyle())
                                }
                            }
                        }
                    }
                }
        }
    }

    private func extra(_ it: OccItem) -> String? {
        it.class == "gio" ? m.summaries[it.entity_id]?.tuoi_mat_text : nil
    }

    private var welcome: some View {
        PCard {
            PHeading("Chưa có ai trong sổ")
            PBody("Thêm ông bà, cha mẹ đã khuất để được nhắc ngày giỗ theo âm lịch; thêm người đang sống để nhớ sinh nhật và ngày kỷ niệm.")
            addButtons
        }
    }

    @ViewBuilder
    private var addButtons: some View {
        if !m.readOnly {
            FlowLayout(spacing: 10) {
                NavigationLink(value: PRoute.add("deceased")) { Text("+ Thêm người đã mất") }
                    .buttonStyle(PrimaryButtonStyle())
                    .accessibilityIdentifier("people.addDeceased")
                NavigationLink(value: PRoute.add("living")) { Text("+ Thêm người đang sống") }
                    .buttonStyle(SecondaryButtonStyle())
                    .accessibilityIdentifier("people.addLiving")
            }
        }
    }

    @ViewBuilder
    private func reminderCards(_ today: CivilDate) -> some View {
        let list = m.todayReminders.filter { !dismissed.contains($0.id) }
        let shown = Array(list.prefix(3))
        ForEach(shown) { r in
            VStack(alignment: .leading, spacing: 8) {
                Label { Text(r.title).ps(17, .semibold).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true) }
                    icon: { Image(systemName: "bell.fill").foregroundStyle(C.gold) }
                if !r.body.isEmpty { Text(r.body).ps(15).foregroundStyle(C.ink2) }
                FlowLayout(spacing: 8) {
                    if r.occ.class == "gio" && r.occ.civil == today {
                        NavigationLink(value: PRoute.memorial(r.occ.entity_id)) { Text("Mở trang tưởng niệm") }
                            .buttonStyle(PrimaryButtonStyle())
                    }
                    if let pid = r.occ.personId {
                        NavigationLink(value: PRoute.person(pid)) { Text("Xem") }.buttonStyle(SecondaryButtonStyle())
                    }
                    Button("Đã biết") { dismissed.insert(r.id) }.buttonStyle(SecondaryButtonStyle())
                }
            }
            .padding(14)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(C.goldTint, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
            .accessibilityElement(children: .contain)
        }
        if list.count > 3 {
            PFine("… và \(list.count - 3) lời nhắc khác hôm nay")
        }
    }

    private var backupNag: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Dữ liệu người thân đã thay đổi hơn \(m.settings["backup_nag_days"].int ?? 30) ngày mà chưa sao lưu. Nếu mất máy, sẽ không lấy lại được.")
                .ps(15).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
            Button("Sao lưu ngay") { section = .backup }.buttonStyle(PrimaryButtonStyle())
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(C.goldTint, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
    }

    private var permissionCard: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Thông báo của ứng dụng đang bị tắt trong Cài đặt của máy, nên điện thoại sẽ không tự nhắc. Danh sách dưới đây vẫn đầy đủ.")
                .ps(15).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
            #if os(iOS)
            Button("Mở Cài đặt") {
                if let u = URL(string: UIApplication.openSettingsURLString) { UIApplication.shared.open(u) }
            }
            .buttonStyle(SecondaryButtonStyle())
            #endif
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(C.redTint, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
    }
}

// MARK: - Theo kỳ (08 §3.2, R13)

struct PeriodSection: View {
    @ObservedObject private var m = PeopleModel.shared
    @State private var kind = "thang"
    @State private var anchor = PeopleClock.today()
    @State private var filter = "tat_ca"
    @State private var result: PeriodResult?

    struct Opt: Identifiable { let id: String; let title: String }
    private static let kinds = [Opt(id: "tuan", title: "Tuần"), Opt(id: "thang", title: "Tháng"),
                                Opt(id: "quy", title: "Quý"), Opt(id: "nam", title: "Năm")]
    private static let filters = [Opt(id: "tat_ca", title: "Tất cả"), Opt(id: "da_mat", title: "Người đã mất"),
                                  Opt(id: "dang_song", title: "Người đang sống")]

    private struct Key: Hashable { let kind: String; let anchor: CivilDate; let filter: String; let rev: Int }

    var body: some View {
        let today = m.today
        VStack(alignment: .leading, spacing: 12) {
            FlowLayout(spacing: 8) {
                ForEach(Self.kinds) { k in
                    ChoiceChip(title: k.title, on: kind == k.id) { kind = k.id }
                        .accessibilityIdentifier("period.kind.\(k.id)")
                }
            }
            FlowLayout(spacing: 8) {
                ForEach(Self.filters) { f in
                    ChoiceChip(title: f.title, on: filter == f.id) { filter = f.id }
                        .accessibilityIdentifier("period.filter.\(f.id)")
                }
            }
            if filter == "da_mat" {
                PFine("Chỉ có ngày giỗ (và tiên thường nếu bật nhắc). Không nhắc sinh nhật và sự kiện của người đã mất.")
            } else if filter == "dang_song" {
                PFine("Sinh nhật và sự kiện của người đang sống.")
            }
            if let r = result, let from = CivilDate(iso: r.from), let to = CivilDate(iso: r.to) {
                HStack(spacing: 8) {
                    NavCircle(system: "chevron.left", label: "Kỳ trước") { move(-1) }
                    Text(PFmt.periodTitle(kind: kind, from: from, to: to))
                        .serif(19, .semibold).foregroundStyle(C.ink)
                        .multilineTextAlignment(.center)
                        .frame(maxWidth: .infinity)
                        .fixedSize(horizontal: false, vertical: true)
                        .accessibilityIdentifier("period.title")
                    NavCircle(system: "chevron.right", label: "Kỳ sau") { move(1) }
                }
                HStack {
                    if from <= today && today <= to {
                        PFine("Đang xem kỳ có hôm nay")
                    } else {
                        Button("Hôm nay") { anchor = today }.buttonStyle(SecondaryButtonStyle())
                    }
                    Spacer()
                    Text("\(r.items.count) mục").ps(15, .semibold).foregroundStyle(C.ink2)
                        .accessibilityIdentifier("period.count")
                }
                if m.persons.isEmpty {
                    PBody("Chưa có ai trong sổ. Thêm người ở trang \"Hồ sơ\".")
                } else if r.items.isEmpty {
                    let f = Self.filters.first { $0.id == filter }?.title ?? ""
                    PCard {
                        PBody("Không có ngày giỗ, sinh nhật hay sự kiện nào trong \(PFmt.periodNoun(kind))."
                              + (filter == "tat_ca" ? "" : " (đang lọc: \(f))"))
                    }
                } else if kind == "quy" || kind == "nam" {
                    let groups = Dictionary(grouping: r.items) { $0.civil.y * 100 + $0.civil.m }
                    ForEach(groups.keys.sorted(), id: \.self) { k in
                        Text("Tháng \(k % 100)/\(String(k / 100))").serif(18, .semibold).foregroundStyle(C.ink)
                            .accessibilityAddTraits(.isHeader).padding(.top, 4)
                        rows(groups[k] ?? [], today)
                    }
                } else {
                    rows(r.items, today)
                }
            }
        }
        .task(id: Key(kind: kind, anchor: anchor, filter: filter, rev: m.revision)) {
            result = m.eventsInPeriod(kind: kind, anchor: anchor, filter: filter)
        }
        .onAppear {
            #if DEBUG
            if let k = UserDefaults.standard.string(forKey: "lvhvPeriodKind") { kind = k }
            #endif
        }
    }

    private func rows(_ items: [OccItem], _ today: CivilDate) -> some View {
        LazyVStack(alignment: .leading, spacing: 10) {
            ForEach(items) { it in
                if let pid = it.personId {
                    NavigationLink(value: PRoute.person(pid)) {
                        OccRow(item: it, today: today, extra: it.class == "gio" ? m.summaries[it.entity_id]?.tuoi_mat_text : nil,
                               chevron: true)
                    }
                    .buttonStyle(.plain)
                } else {
                    OccRow(item: it, today: today)
                }
            }
        }
    }

    private func move(_ dir: Int) {
        let r: PeriodRange?
        if dir < 0 {
            r = try? periodPrev(kind: kind, anchor: anchor.sd)
        } else {
            r = try? periodNext(kind: kind, anchor: anchor.sd)
        }
        if let r { anchor = CivilDate(r.from) }
    }
}

// MARK: - Hồ sơ (danh sách, 08 §3.3)

struct ProfilesSection: View {
    @ObservedObject private var m = PeopleModel.shared

    var body: some View {
        ViewThatFits(in: .horizontal) {
            HStack(alignment: .top, spacing: 14) {
                group(title: "Người đã mất", kind: "deceased", list: m.deceased)
                    .frame(minWidth: 300, idealWidth: 320, maxWidth: .infinity)
                group(title: "Người đang sống", kind: "living", list: m.living)
                    .frame(minWidth: 300, idealWidth: 320, maxWidth: .infinity)
            }
            .frame(minWidth: 640)
            VStack(alignment: .leading, spacing: 14) {
                group(title: "Người đã mất", kind: "deceased", list: m.deceased)
                group(title: "Người đang sống", kind: "living", list: m.living)
            }
        }
    }

    private func group(title: String, kind: String, list: [JV]) -> some View {
        PCard {
            HStack {
                PHeading("\(title) (\(list.count))")
                Spacer()
                if !m.readOnly {
                    NavigationLink(value: PRoute.add(kind)) { Text("+ Thêm") }
                        .buttonStyle(SecondaryButtonStyle())
                        .accessibilityLabel("Thêm \(title.lowercased())")
                        .accessibilityIdentifier("profiles.add.\(kind)")
                }
            }
            if list.isEmpty {
                PBody("Chưa có ai.")
            }
            ForEach(list, id: \.self) { p in
                let id = p["id"].string
                NavigationLink(value: PRoute.person(id)) { row(p) }
                    .buttonStyle(.plain)
                if p != list.last { Divider() }
            }
        }
    }

    private func row(_ p: JV) -> some View {
        let id = p["id"].string
        let name = PFmt.displayName(p)
        return HStack(spacing: 12) {
            Text(PFmt.initials(p["ho_ten"].string)).serif(18, .semibold).foregroundStyle(C.ink2)
                .frame(width: 44, height: 44).background(C.paper2, in: Circle()).accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 2) {
                Text(name).ps(17, .semibold).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
                if !p["quan_he"].string.isEmpty { Text(PFmt.cap(p["quan_he"].string)).ps(14).foregroundStyle(C.ink2) }
                Text(summary(p, id)).ps(14).foregroundStyle(C.ink3).fixedSize(horizontal: false, vertical: true)
            }
            Spacer(minLength: 0)
            Image(systemName: "chevron.right").foregroundStyle(C.ink3).accessibilityHidden(true)
        }
        .frame(minHeight: 52)
        .contentShape(Rectangle())
        .accessibilityElement(children: .combine)
    }

    private func summary(_ p: JV, _ id: String) -> String {
        if p["kind"].str == "deceased" {
            var s = "Giỗ \(PFmt.gioText(p["death"]["lunar"]))"
            if let g = m.nextGio(for: id) {
                s += " · lần tới \(g.civil.dmy) (\(PFmt.distance(g.civil.dayNumber - m.today.dayNumber).lowercased()))"
            }
            if p["death"]["lunar"]["year"].isNull { s += " (không rõ năm mất)" }
            return s
        }
        let b = p["birth"]
        guard b.isObject else { return "Chưa có ngày sinh" }
        if let s = DateCheck.solar(b["solar"]) {
            let t = m.summaries[id]?.tuoi_duong.map { " · \($0) tuổi" } ?? ""
            return "Sinh \(s.dmy)\(t)"
        }
        return "Sinh năm \(b["input"]["year"].int.map(String.init) ?? "?")"
    }
}

#if !DEBUG
extension DemoSeed { static let isDiag = false }
enum DemoSeed {}
struct UpgradeDiagView: View { var body: some View { EmptyView() } }
#endif

/// Mở thẳng một biểu mẫu ghi khi đang ở chế độ chỉ đọc (08 §12l TKN3-P-01).
struct ReadOnlyBlockedView: View {
    var body: some View {
        PCard {
            PBody("Đang ở chế độ chỉ đọc nên chưa thêm hay sửa được. Dữ liệu cũ vẫn còn nguyên; hãy tạo một tệp sao lưu ở mục Sao lưu.")
        }
        .pageWidth()
        .accessibilityIdentifier("people.readOnly.blocked")
    }
}
