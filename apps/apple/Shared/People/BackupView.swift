// Sao lưu và lịch điện thoại (08 §3.8) + Cài đặt nhắc của máy này (O-9, công tắc chung, mặc định hồ sơ mới).
// Mọi đường xuất đều qua hộp xác nhận ghi số người/số mục và loại dữ liệu (S10). Không gửi gì lên mạng.
import SwiftUI
import UniformTypeIdentifiers
#if os(iOS)
import UIKit
#endif

/// Tệp sao lưu (byte đã mã hóa) cho `fileExporter`.
struct BackupDocument: FileDocument {
    static var readableContentTypes: [UTType] { [.data] }
    var data: Data
    init(data: Data) { self.data = data }
    init(configuration: ReadConfiguration) throws { data = configuration.file.regularFileContents ?? Data() }
    func fileWrapper(configuration: WriteConfiguration) throws -> FileWrapper { FileWrapper(regularFileWithContents: data) }
}

/// Xem trước .ics: vài dòng "28/07/2026 — Giỗ đầu Ông Nguyễn Văn A" đọc từ tệp lõi đã sinh.
enum IcsPreview {
    static func lines(_ ics: String, limit: Int = 2) -> [String] {
        // Gỡ dòng gập (RFC 5545 §3.1): CRLF + khoảng trắng.
        let unfolded = ics.replacingOccurrences(of: "\r\n ", with: "").replacingOccurrences(of: "\r\n\t", with: "")
        var out: [String] = []
        var date: String?
        for raw in unfolded.components(separatedBy: "\r\n") {
            if raw.hasPrefix("BEGIN:VEVENT") { date = nil }
            if raw.hasPrefix("DTSTART"), let v = raw.split(separator: ":").last, v.count >= 8 {
                let s = String(v.prefix(8))
                date = "\(s.dropFirst(6).prefix(2))/\(s.dropFirst(4).prefix(2))/\(s.prefix(4))"
            }
            if raw.hasPrefix("SUMMARY"), let i = raw.firstIndex(of: ":"), let d = date {
                let summary = String(raw[raw.index(after: i)...])
                    .replacingOccurrences(of: "\\,", with: ",").replacingOccurrences(of: "\\;", with: ";")
                    .replacingOccurrences(of: "\\\\", with: "\\")
                out.append("\(d) — \(summary)")
                if out.count >= limit { break }
            }
        }
        return out
    }
}

struct BackupSection: View {
    @ObservedObject private var m = PeopleModel.shared
    @State private var icsOpen = false
    @State private var exportOpen = false
    @State private var exportDoc: BackupDocument?
    @State private var exporting = false
    @State private var importing = false
    @State private var picked: URL?
    @State private var importPassword = ""
    @State private var importError: String?
    @State private var verified: VerifiedBackup?
    @State private var preview: BackupPreview?
    @State private var opening = false

    private var lastBackup: String {
        if let s = m.settings["last_backup_at"].str, let d = PFmt.rfcToLocalDate(s) { return "Lần sao lưu gần nhất: \(d)" }
        return "Chưa sao lưu lần nào"
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            PCard {
                PHeading("Lịch của máy: xuất tệp lịch (.ics)")
                PBody("Ứng dụng đã tự nhắc bằng thông báo trên máy này. Nếu muốn ứng dụng Lịch (hoặc một máy khác) cũng nhắc, hãy xuất tệp lịch (.ics) và mở bằng ứng dụng Lịch.")
                Button("Xuất tệp lịch (.ics)…") { icsOpen = true }
                    .buttonStyle(PrimaryButtonStyle())
                    .accessibilityIdentifier("backup.ics")
            }
            PCard {
                PHeading("Sao lưu dữ liệu")
                PBody(lastBackup)
                PFine("Dữ liệu người thân không tự sao lưu lên iCloud. Hãy tạo tệp sao lưu có mật khẩu và cất ở nơi an toàn.")
                Button("Tạo tệp sao lưu…") { exportOpen = true }
                    .buttonStyle(PrimaryButtonStyle())
                    .disabled(m.persons.isEmpty)
                    .accessibilityIdentifier("backup.export")
            }
            PCard {
                PHeading("Khôi phục từ tệp sao lưu")
                Button("Chọn tệp sao lưu…") { importing = true }
                    .buttonStyle(SecondaryButtonStyle())
                    .accessibilityIdentifier("backup.pick")
                PBody(picked.map { "Đã chọn: \($0.lastPathComponent)" } ?? "Chưa chọn tệp.")
                LabeledField(label: "Mật khẩu của tệp", error: importError) {
                    SecureField("", text: $importPassword)
                        .textFieldStyle(.roundedBorder)
                        .ps(17)
                        .passwordContent()
                        .accessibilityLabel("Mật khẩu của tệp")
                }
                Button(opening ? "Đang mở tệp…" : "Mở tệp") { openFile() }
                    .buttonStyle(SecondaryButtonStyle())
                    .disabled(picked == nil || importPassword.isEmpty || opening)
            }
            PCard {
                PHeading("Nhắc sao lưu")
                Picker("Nhắc sao lưu khi có thay đổi chưa sao lưu sau", selection: Binding(
                    get: { m.settings["backup_nag_days"].int ?? 30 },
                    set: { v in var s = m.settings; s["backup_nag_days"] = .i(v); try? m.putSettings(s) })) {
                    Text("30 ngày").tag(30)
                    Text("60 ngày").tag(60)
                    Text("90 ngày").tag(90)
                    Text("Không nhắc").tag(0)
                }
                .pickerStyle(.menu)
                PFine("Nhắc sao lưu khi có thay đổi chưa sao lưu sau số ngày đã chọn.")
            }
        }
        .sheet(isPresented: $icsOpen) {
            IcsConfirmSheet().frameForMac()
        }
        .sheet(isPresented: $exportOpen) {
            BackupExportSheet { data in
                exportDoc = BackupDocument(data: data)
                exportOpen = false
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.4) { exporting = true }
            }
            .frameForMac()
        }
        .fileExporter(isPresented: $exporting, document: exportDoc, contentType: .data,
                      defaultFilename: "LichVanHoaViet-saoluu-\(m.today.iso).lvhv") { r in
            if case .success = r {
                m.toast = "Đã tạo tệp sao lưu. Hãy cất tệp ở nơi an toàn (ổ đám mây, USB) — tệp không tự gửi đi đâu."
            }
            exportDoc = nil
        }
        .fileImporter(isPresented: $importing, allowedContentTypes: [.data, .item]) { r in
            if case .success(let u) = r { picked = u; importError = nil }
        }
        .sheet(item: Binding(get: { preview.map { PreviewBox(p: $0) } }, set: { if $0 == nil { preview = nil; verified = nil } })) { box in
            RestoreSheet(preview: box.p) { replace in
                guard let v = verified else { return }
                do {
                    try await m.applyImport(v, replace: replace)
                    m.toast = "Đã khôi phục: \(box.p.persons) người, \(box.p.events) sự kiện."
                } catch {
                    importError = "Chưa khôi phục được — dữ liệu trên máy giữ nguyên."
                }
                preview = nil
                verified = nil
                importPassword = ""
            }
            .frameForMac()
        }
    }

    struct PreviewBox: Identifiable { let p: BackupPreview; var id: String { p.exportedAt } }

    /// 72 byte đầu + tối đa 128 MiB dữ liệu + 16 byte thẻ (03 §2, S12): lớn hơn thì không đọc.
    static let maxBackupBytes = 72 + 128 * 1024 * 1024 + 16

    private func openFile() {
        guard let url = picked else { return }
        let pw = importPassword
        opening = true
        importError = nil
        let limit = Self.maxBackupBytes
        Task {
            // Đọc tệp (tới 128 MiB) và giải mã ở luồng nền (MA3N-23-02); chỉ cập nhật giao diện trên luồng chính.
            let data: Data? = await Task.detached(priority: .userInitiated) { () -> Data? in
                let scoped = url.startAccessingSecurityScopedResource()
                defer { if scoped { url.stopAccessingSecurityScopedResource() } }
                let size = (try? url.resourceValues(forKeys: [.fileSizeKey]).fileSize) ?? 0
                guard size <= limit else { return nil }
                return try? Data(contentsOf: url)
            }.value
            guard let data else {
                importError = "Sai mật khẩu hoặc tệp bị hỏng"
                opening = false
                return
            }
            do {
                let v = try await Task.detached(priority: .userInitiated) { try openBackup(file: data, password: pw) }.value
                verified = v
                preview = v.preview()
            } catch {
                importError = Core.message(error)
            }
            opening = false
        }
    }
}

extension View {
    @ViewBuilder
    func frameForMac() -> some View {
        #if os(macOS)
        self.frame(minWidth: 480, idealWidth: 520, minHeight: 420)
        #else
        self
        #endif
    }
}

// MARK: - Hộp xác nhận .ics (S10, 02 §8.2)

struct IcsConfirmSheet: View {
    @Environment(\.dismiss) private var dismiss
    @ObservedObject private var m = PeopleModel.shared
    @State private var ics: IcsExport?
    @State private var fileURL: URL?
    @State private var error: String?

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 12) {
                    if let e = error {
                        FieldError(text: e)
                    } else if let ics {
                        if ics.events == 0 {
                            PBody("Chưa có hồ sơ nào bật nhắc.")
                        } else {
                            let to = m.today.adding(3652 - 1)
                            PBody("Tệp này chứa tên, ngày giỗ, sinh nhật và sự kiện của \(ics.persons) người. Bất kỳ ai có tệp đều đọc được.")
                            PBody("Gồm \(ics.events) ngày trong 10 năm tới (từ \(m.today.dmy) đến \(to.dmy)), mỗi ngày có lời nhắc như đã đặt.")
                            Text("Xem trước:").ps(15, .semibold).foregroundStyle(C.ink)
                            ForEach(IcsPreview.lines(ics.content), id: \.self) { l in
                                Text("• \(l)").ps(15).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
                            }
                            if ics.events > 2 { PFine("… và \(ics.events - 2) ngày khác") }
                            PFine("Mở tệp bằng ứng dụng Lịch của điện thoại hoặc máy tính để nhập.")
                        }
                    } else {
                        PBody("Đang chuẩn bị tệp…")
                    }
                }
                .padding(20)
            }
            .background(C.wall)
            .navigationTitle("Xuất tệp lịch")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button("Hủy") { dismiss() } }
                ToolbarItem(placement: .confirmationAction) {
                    if let u = fileURL, (ics?.events ?? 0) > 0 {
                        ShareLink(item: u) { Text("Lưu / chia sẻ tệp .ics") }
                            .accessibilityIdentifier("ics.share")
                    }
                }
            }
        }
        .task {
            do {
                let x = try m.icsExport()
                ics = x
                let dir = FileManager.default.temporaryDirectory.appendingPathComponent("ics", isDirectory: true)
                try? FileManager.default.removeItem(at: dir)
                try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
                let u = dir.appendingPathComponent(x.fileName)
                try Data(x.content.utf8).write(to: u, options: [.atomic])
                fileURL = u
            } catch {
                self.error = Core.message(error)
            }
        }
        .onDisappear {
            if let u = fileURL { try? FileManager.default.removeItem(at: u) }
        }
    }
}

// MARK: - Tạo tệp sao lưu (03 §2)

struct BackupExportSheet: View {
    var onDone: (Data) -> Void
    @Environment(\.dismiss) private var dismiss
    @ObservedObject private var m = PeopleModel.shared
    @State private var pw = ""
    @State private var pw2 = ""
    @State private var submitted = false
    @State private var busy = false
    @State private var error: String?

    private var photoCount: Int {
        var ids = Set<String>()
        for p in m.persons {
            if let id = p["photo_id"].str { ids.insert(id) }
            for c in p["death"]["notice"]["certificate_photo_ids"].array { if let id = c.str { ids.insert(id) } }
        }
        return ids.count
    }
    private var hasNotice: Bool { m.persons.contains { $0["death"]["notice"].isObject } }
    private var pwError: String? { pw.count < 8 ? "Mật khẩu cần ít nhất 8 ký tự" : nil }
    private var pw2Error: String? { pw != pw2 ? "Hai lần nhập mật khẩu chưa giống nhau" : nil }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 14) {
                    PBody("Tệp sẽ chứa \(m.persons.count) người, \(m.events.count) sự kiện, \(photoCount) ảnh — đã mã hóa bằng mật khẩu.")
                    if hasNotice {
                        PBody("Tệp gồm cả thông tin báo tử (người báo tin, nơi mất, lý do, ảnh giấy báo tử) — chỉ ai biết mật khẩu mới đọc được.")
                    }
                    LabeledField(label: "Mật khẩu (ít nhất 8 ký tự)", error: submitted ? pwError : nil) {
                        SecureField("", text: $pw).textFieldStyle(.roundedBorder).ps(17).passwordContent()
                            .accessibilityLabel("Mật khẩu").accessibilityIdentifier("backup.pw")
                    }
                    LabeledField(label: "Nhập lại mật khẩu", error: submitted ? pw2Error : nil) {
                        SecureField("", text: $pw2).textFieldStyle(.roundedBorder).ps(17).passwordContent()
                            .accessibilityLabel("Nhập lại mật khẩu").accessibilityIdentifier("backup.pw2")
                    }
                    Text("Lưu ý: quên mật khẩu là mất dữ liệu trong tệp — không ai lấy lại được, kể cả Clevix Lab.")
                        .ps(15).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
                        .padding(12).frame(maxWidth: .infinity, alignment: .leading)
                        .background(C.goldTint, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
                    PFine("Gợi ý: dùng một câu dễ nhớ, ví dụ “nhà ở Hà Đông có cây bàng”.")
                    if busy {
                        HStack(spacing: 10) {
                            ProgressView()
                            Text("Đang mã hóa… (có thể mất vài giây)").ps(15).foregroundStyle(C.ink2)
                        }
                    }
                    FieldError(text: error)
                }
                .padding(20)
            }
            .background(C.wall)
            .navigationTitle("Tạo tệp sao lưu")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button("Hủy") { dismiss() }.disabled(busy) }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Tạo và lưu tệp") { run() }.disabled(busy).accessibilityIdentifier("backup.create")
                }
            }
        }
        .interactiveDismissDisabled(busy)
    }

    private func run() {
        submitted = true
        guard pwError == nil, pw2Error == nil else { return }
        busy = true
        error = nil
        let password = pw
        Task {
            do {
                let data = try await m.exportBackup(password: password)
                pw = ""; pw2 = ""
                busy = false
                onDone(data)
            } catch {
                self.error = Core.message(error)
                busy = false
            }
        }
    }
}

// MARK: - Xem trước khôi phục (01 §6.3)

struct RestoreSheet: View {
    let preview: BackupPreview
    var onApply: (Bool) async -> Void
    @Environment(\.dismiss) private var dismiss
    @ObservedObject private var m = PeopleModel.shared
    @State private var replace = false
    @State private var confirmReplace = false
    @State private var busy = false

    private func run(_ r: Bool) {
        busy = true
        Task {
            await onApply(r)
            busy = false
        }
    }

    private var platformName: String {
        switch preview.platform {
        case "pwa": return "ứng dụng web"
        case "android": return "Android"
        case "ios": return "iPhone/iPad"
        case "macos": return "máy Mac"
        case "windows": return "Windows"
        default: return "ứng dụng cài trên máy"
        }
    }
    private var exported: String { PFmt.rfcToLocalDate(preview.exportedAt) ?? preview.exportedAt }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 14) {
                    PBody("Tệp xuất ngày \(exported) từ \(platformName): \(preview.persons) người, \(preview.events) sự kiện, \(preview.photos) ảnh.")
                    Text("Cách nhập:").ps(15, .semibold).foregroundStyle(C.ink)
                    Picker("Cách nhập", selection: $replace) {
                        Text("Gộp vào dữ liệu đang có (khuyên dùng) — giữ bản mới hơn của từng hồ sơ").tag(false)
                        Text("Thay toàn bộ — xóa dữ liệu đang có trên máy rồi dùng dữ liệu trong tệp").tag(true)
                    }
                    .pickerStyle(.inline)
                    .labelsHidden()
                    .disabled(busy)
                    if busy {
                        HStack(spacing: 10) {
                            ProgressView()
                            Text("Đang khôi phục… (có thể mất vài giây)").ps(15).foregroundStyle(C.ink2)
                        }
                    }
                }
                .padding(20)
            }
            .background(C.wall)
            .navigationTitle("Khôi phục dữ liệu")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button("Hủy") { dismiss() } }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Khôi phục") { if replace { confirmReplace = true } else { run(false) } }
                        .disabled(busy)
                }
            }
            .alert("Thay toàn bộ dữ liệu?", isPresented: $confirmReplace) {
                Button("Hủy", role: .cancel) {}
                Button("Thay toàn bộ", role: .destructive) { run(true) }
            } message: {
                Text("Dữ liệu hiện có trên máy (\(m.persons.count) người) sẽ bị thay bằng dữ liệu trong tệp xuất ngày \(exported). Không hoàn tác được.")
            }
        }
        .interactiveDismissDisabled(busy)
    }
}

// MARK: - Cài đặt nhắc

struct PeopleSettingsView: View {
    @ObservedObject private var m = PeopleModel.shared
    @ObservedObject private var sched = PeopleModel.shared.scheduler
    @State private var defaults = RuleDraft()
    @State private var loaded = false
    @State private var savedDefaults = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                Text("Cài đặt nhắc").serif(28, .semibold).foregroundStyle(C.ink).accessibilityAddTraits(.isHeader)
                PCard {
                    Toggle("Bật lời nhắc", isOn: Binding(
                        get: { m.settings["reminders_enabled"].bool ?? true },
                        set: { v in var s = m.settings; s["reminders_enabled"] = .bool(v); try? m.putSettings(s) }))
                        .ps(17, .semibold)
                        .accessibilityIdentifier("settings.enabled")
                    PFine("Tắt thì không có thông báo nào; danh sách Sắp tới và Theo kỳ vẫn đầy đủ.")
                    Divider()
                    permissionRow
                }
                PCard {
                    Toggle("Hiện tên người trên màn hình khóa", isOn: Binding(
                        get: { m.showNames }, set: { m.setShowNames($0) }))
                        .ps(17, .semibold)
                        .accessibilityIdentifier("settings.showNames")
                    PFine("Khi tắt (mặc định), thông báo chỉ ghi “Sắp đến ngày giỗ”, “Sắp đến sinh nhật” hoặc “Sắp đến sự kiện”, không có tên. Tùy chọn này chỉ lưu trên máy này, không đi vào tệp sao lưu.")
                }
                PCard {
                    RuleEditor(rule: $defaults, title: "Mặc định cho hồ sơ mới", errors: defaults.validate(prefix: "d"),
                               prefix: "d", showToggle: false)
                    HStack {
                        Button("Lưu mặc định") { saveDefaults() }
                            .buttonStyle(SecondaryButtonStyle())
                            .disabled(!defaults.validate(prefix: "d").isEmpty)
                        if savedDefaults { Text("Đã lưu.").ps(15).foregroundStyle(C.green) }
                    }
                }
                PFine("Ứng dụng nhắc bằng thông báo của máy, kể cả khi đang đóng. iPhone và iPad chỉ giữ tối đa 64 thông báo chờ; khi có nhiều hơn, ứng dụng đặt một thông báo nhắc bạn mở ứng dụng để đặt tiếp.")
            }
            .pageWidth(680)
        }
        .navigationTitle("Cài đặt nhắc")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        .task { await sched.refreshPermission() }
        .onAppear {
            guard !loaded else { return }
            loaded = true
            defaults = RuleDraft(defaults: m.settings)
        }
    }

    @ViewBuilder
    private var permissionRow: some View {
        switch sched.permission {
        case .authorized:
            Label("Đã cho phép thông báo.", systemImage: "checkmark.circle").ps(15).foregroundStyle(C.green)
        case .denied:
            Text("Thông báo của ứng dụng đang bị tắt trong Cài đặt của máy.").ps(15).foregroundStyle(C.redInk)
                .fixedSize(horizontal: false, vertical: true)
            #if os(iOS)
            Button("Mở Cài đặt") {
                if let u = URL(string: UIApplication.openSettingsURLString) { UIApplication.shared.open(u) }
            }
            .buttonStyle(SecondaryButtonStyle())
            #endif
        case .notDetermined:
            PFine("Ứng dụng sẽ xin phép gửi thông báo khi bạn lưu hồ sơ có nhắc lần đầu.")
            Button("Cho phép thông báo") { Task { await sched.requestPermission(store: m.store) } }
                .buttonStyle(SecondaryButtonStyle())
                .accessibilityIdentifier("settings.allow")
        }
    }

    private func saveDefaults() {
        var s = m.settings
        s["default_offsets_days"] = .array(defaults.finalOffsets.map { JV.i($0) })
        s["default_time_local"] = .s(defaults.hhmm ?? "07:00")
        try? m.putSettings(s)
        savedDefaults = true
    }
}

extension View {
    /// Ô mật khẩu: không tự sửa chữ, không viết hoa; gợi ý mật khẩu của hệ thống chỉ trên iOS.
    @ViewBuilder
    func passwordContent() -> some View {
        #if os(iOS)
        self.textContentType(.password).textInputAutocapitalization(.never).autocorrectionDisabled(true)
        #else
        self.autocorrectionDisabled(true)
        #endif
    }
}
