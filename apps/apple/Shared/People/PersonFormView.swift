// Biểu mẫu thêm/sửa người đã mất (08 §3.4), người đang sống (§3.5) và "Cập nhật: đã mất" (§12g, US-M-07).
import PhotosUI
import QuickLook
import SwiftUI
import UniformTypeIdentifiers

/// Gợi ý cách gọi / quan hệ lấy từ lõi (YC-2026-07): đã viết hoa chữ đầu; người đang sống không có Cụ, Kỵ, Ông Tổ họ,
/// Bà Cô Tổ họ, Bé đỏ; theo giới tính thì từ hợp giới tính lên trước.
enum Suggest {
    static func danhXung(deceased: Bool, gioiTinh: String?) -> [String] {
        danhXungSuggestions(deceased: deceased, gioiTinh: gioiTinh)
    }
    static func quanHe(deceased: Bool) -> [String] { quanHeSuggestions(deceased: deceased) }
}

/// Nút chọn ảnh (PhotosPicker — không cần quyền thư viện ảnh); ảnh được xử lý S9 ở luồng nền.
struct PhotoPickButton: View {
    let title: String
    var maxSide: Int = PhotoProcessor.maxSide
    var maxBytes: Int = PhotoProcessor.maxStoredBytes
    var onPicked: (Result<ProcessedPhoto, PhotoError>) -> Void
    @State private var item: PhotosPickerItem?
    @State private var busy = false

    var body: some View {
        PhotosPicker(selection: $item, matching: .images) {
            Text(busy ? "Đang xử lý ảnh…" : title)
        }
        .buttonStyle(SecondaryButtonStyle())
        .disabled(busy)
        .onChange(of: item) { it in
            guard let it else { return }
            busy = true
            Task {
                let data = try? await it.loadTransferable(type: Data.self)
                let side = maxSide, cap = maxBytes
                let r: Result<ProcessedPhoto, PhotoError> = await Task.detached(priority: .userInitiated) {
                    guard let data else { return .failure(PhotoError.unreadable) }
                    do { return .success(try PhotoProcessor.process(data, maxSide: side, maxBytes: cap)) }
                    catch let e as PhotoError { return .failure(e) }
                    catch { return .failure(PhotoError.unreadable) }
                }.value
                onPicked(r)
                item = nil
                busy = false
            }
        }
    }
}

/// Khối "Thông tin báo tử (không bắt buộc)" — 01 §3.2a, tệp đính kèm 01 §3.8a (YC-2026-07).
struct NoticeEditor: View {
    @Binding var n: NoticeDraft
    var errors: FieldErrors
    let today: CivilDate
    @State private var fileError: String?
    @State private var pickingPdf = false
    @State private var viewing: AttOpen?

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            PFine("Chỉ hiện trên trang hồ sơ này. Không đưa vào lời nhắc, tệp lịch (.ics) hay trang tưởng niệm; có trong tệp sao lưu (đã mã hóa).")
            PTextField(label: "Người báo tin", text: $n.reportedBy, error: errors["notice.reported_by"], id: "notice.reportedBy")
            PTextField(label: "Mất tại", text: $n.place, error: errors["notice.place"], id: "notice.place")
            LabeledField(label: "Ngày nhận tin (dương lịch)") {
                DateInputView(draft: $n.date, purpose: .notice, allowLunar: false, error: errors["notice.date"],
                              today: today, idPrefix: "notice.date")
            }
            PTextField(label: "Lý do", text: $n.reason, error: errors["notice.reason"], multiline: true, id: "notice.reason")
            VStack(alignment: .leading, spacing: 10) {
                Text("Tệp đính kèm (tối đa 6)").ps(15, .semibold).foregroundStyle(C.ink)
                ForEach($n.attachments) { $a in
                    AttachmentRow(att: $a, onView: { open(a) }, onRemove: { n.attachments.removeAll { $0.id == a.id } })
                }
                if n.attachments.count < NoticeDraft.maxAttachments {
                    FlowLayout(spacing: 10) {
                        PhotoPickButton(title: "Thêm ảnh", maxSide: 2048, maxBytes: 2 * 1024 * 1024) { r in
                            switch r {
                            case .success(let p):
                                n.attachments.append(.make(NewAttachment(data: p.jpeg, mime: "image/jpeg", fileName: nil),
                                                           kind: n.attachments.isEmpty ? .tinBaoTu : .giayChungTu))
                                fileError = nil
                            case .failure(let e): fileError = e.message
                            }
                        }
                        Button("Thêm tệp PDF…") { pickingPdf = true }
                            .buttonStyle(SecondaryButtonStyle())
                            .accessibilityIdentifier("notice.addPdf")
                    }
                }
                FieldError(text: fileError ?? errors["notice.attachments"])
                PFine("Ảnh được nén lại và bỏ thông tin vị trí, máy chụp. Tệp PDF được lưu nguyên bản (có thể còn thông tin của máy quét). Tệp chỉ lưu trong ứng dụng trên máy này.")
            }
            PTextField(label: "Ghi chú", text: $n.note, error: errors["notice.note"], multiline: true, id: "notice.note")
        }
        .fileImporter(isPresented: $pickingPdf, allowedContentTypes: [.pdf]) { r in
            if case .success(let u) = r { Task { await addPdf(u) } }
        }
        .attachmentViewer($viewing)
    }

    /// PDF: kiểm cỡ (≤ 10 MB) **trước** khi đọc, rồi chữ ký `%PDF-`; lưu nguyên byte (RA-SOAT §5.3, B6).
    private func addPdf(_ url: URL) async {
        let r: Result<NewAttachment, UIMessage> = await Task.detached(priority: .userInitiated) {
            let scoped = url.startAccessingSecurityScopedResource()
            defer { if scoped { url.stopAccessingSecurityScopedResource() } }
            let size = (try? url.resourceValues(forKeys: [.fileSizeKey]).fileSize) ?? 0
            guard size > 0, size <= 10 * 1024 * 1024 else { return .failure(UIMessage(text: "Tệp PDF lớn hơn 10 MB — hãy chọn tệp nhỏ hơn")) }
            guard let d = try? Data(contentsOf: url) else { return .failure(UIMessage(text: "Không đọc được tệp này")) }
            guard d.starts(with: Array("%PDF-".utf8)) else { return .failure(UIMessage(text: "Tệp này không phải PDF")) }
            let name = String(url.lastPathComponent.prefix(120))
            return .success(NewAttachment(data: d, mime: "application/pdf", fileName: name))
        }.value
        switch r {
        case .success(let a): n.attachments.append(.make(a, kind: .giayChungTu)); fileError = nil
        case .failure(let m): fileError = m.text
        }
    }

    private func open(_ a: AttDraft) {
        let m = PeopleModel.shared
        let data = a.new?.data ?? a.fileId.flatMap { m.attachmentData($0) }
        let pdf = a.new?.isPdf ?? (a.fileId.flatMap { m.attachmentsMeta[$0]?["mime"].str } == "application/pdf")
        guard let data else { return }
        viewing = AttOpen(data: data, pdf: pdf)
    }
}

// MARK: - Xem tệp đính kèm (BM3-03, S10)

/// Yêu cầu xem một tệp đính kèm.
struct AttOpen: Identifiable, Equatable {
    let id = UUID()
    let data: Data
    let pdf: Bool
    static func == (a: AttOpen, b: AttOpen) -> Bool { a.id == b.id }
}

private struct AttImage: Identifiable {
    let id = UUID()
    let img: CGImage
}

/// Ảnh: xem ngay trong ứng dụng (không có nút Chia sẻ, không ghi tệp tạm). PDF: hộp hỏi lại, "Hủy" là nút mặc định, nói rõ
/// tệp sẽ mở trong trình xem có nút Chia sẻ; chỉ khi bấm "Mở tệp" mới ghi bản chép tạm tên chung và mở Quick Look (§5.3).
struct AttachmentViewer: ViewModifier {
    @Binding var request: AttOpen?
    @State private var image: AttImage?
    @State private var pdfData: Data?
    @State private var confirmPdf = false
    @State private var preview: URL?
    @Environment(\.scenePhase) private var phase

    func body(content: Content) -> some View {
        content
            // DV23-04 (BM2-04): trình xem PDF của hệ thống nằm ngoài lớp che của ứng dụng — rời tiền cảnh thì đóng nó
            // (tệp tạm bị xóa theo), để ảnh của bộ chuyển ứng dụng chỉ thấy lớp che.
            .onChange(of: phase) { ph in if ph != .active, preview != nil { preview = nil } }
            .onChange(of: request) { r in
                guard let r else { return }
                request = nil
                if r.pdf {
                    pdfData = r.data
                    confirmPdf = true
                } else if let img = PhotoProcessor.cgImage(r.data) {
                    image = AttImage(img: img)
                }
            }
            .alert("Mở tệp PDF?", isPresented: $confirmPdf) {
                Button("Hủy", role: .cancel) { pdfData = nil }
                    .keyboardShortcut(.defaultAction)
                    .accessibilityIdentifier("att.pdf.cancel")
                Button("Mở tệp") {
                    if let d = pdfData { preview = try? AttachmentTemp.write(d, pdf: true) }
                    pdfData = nil
                }
                .accessibilityIdentifier("att.pdf.open")
            } message: {
                Text("Tệp sẽ mở trong trình xem của hệ thống. Trình xem đó có nút Chia sẻ, có thể gửi tệp ra ngoài ứng dụng (tin nhắn, thư, lưu vào Tệp). Chỉ mở khi bạn đang ở nơi riêng tư.")
            }
            .quickLookPreview($preview)
            .onChange(of: preview) { u in if u == nil { AttachmentTemp.clear() } }
            .sheet(item: $image) { AttachmentImageView(image: $0.img) }
    }
}

extension View {
    func attachmentViewer(_ request: Binding<AttOpen?>) -> some View { modifier(AttachmentViewer(request: request)) }
}

/// Trang xem ảnh đính kèm trong ứng dụng: chụm hai ngón để phóng to, chạm hai lần để về cỡ vừa.
struct AttachmentImageView: View {
    let image: CGImage
    @Environment(\.dismiss) private var dismiss
    @State private var scale: CGFloat = 1
    @State private var base: CGFloat = 1

    var body: some View {
        NavigationStack {
            GeometryReader { g in
                ScrollView([.horizontal, .vertical], showsIndicators: false) {
                    Image(decorative: image, scale: 1)
                        .resizable()
                        .interpolation(.high)
                        .scaledToFit()
                        .frame(width: g.size.width * scale, height: g.size.height * scale)
                        .accessibilityLabel("Ảnh tệp đính kèm")
                }
                .gesture(MagnificationGesture()
                    .onChanged { v in scale = min(max(base * v, 1), 5) }
                    .onEnded { _ in base = scale })
                .onTapGesture(count: 2) { scale = 1; base = 1 }
            }
            .background(C.wall)
            .accessibilityIdentifier("att.image")
            .navigationTitle("Tệp đính kèm")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Đóng") { dismiss() }.accessibilityIdentifier("att.image.close")
                }
            }
        }
        .tint(C.redInk)
        .privacyCover()
        #if os(macOS)
        .frame(minWidth: 520, minHeight: 600)
        #endif
    }
}

/// Câu báo lỗi dùng làm `Error`.
struct UIMessage: Error { let text: String }

/// Một tệp đính kèm trong biểu mẫu: ảnh nhỏ / biểu tượng PDF, chọn loại, Xem (Quick Look), Bỏ.
struct AttachmentRow: View {
    @Binding var att: AttDraft
    var onView: () -> Void
    var onRemove: () -> Void
    @ObservedObject private var m = PeopleModel.shared

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .top, spacing: 12) {
                AttachmentThumb(att: att)
                VStack(alignment: .leading, spacing: 4) {
                    Text(title).ps(15, .semibold).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
                    if isPdf { PFine("Tệp PDF được lưu nguyên bản") }
                }
                Spacer(minLength: 0)
            }
            SegmentChips(options: AttKind.allCases.map { SegOpt($0.rawValue, $0.title) }, selection: $att.kind,
                         idPrefix: "notice.att.kind")
            HStack(spacing: 10) {
                Button("Xem", action: onView).buttonStyle(SecondaryButtonStyle())
                    .accessibilityIdentifier("notice.att.view")
                Button("Bỏ", action: onRemove).buttonStyle(SecondaryButtonStyle(destructive: true))
            }
        }
        .padding(12)
        .background(C.paper2, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private var isPdf: Bool {
        att.new?.isPdf ?? (att.fileId.flatMap { m.attachmentsMeta[$0]?["mime"].str } == "application/pdf")
    }
    private var title: String {
        let name = att.new?.fileName ?? att.fileId.flatMap { m.attachmentsMeta[$0]?["file_name"].str }
        return name ?? (isPdf ? "Tệp PDF" : "Ảnh")
    }
}

/// Ảnh nhỏ của tệp đính kèm (giải mã có trần điểm ảnh); PDF không bao giờ được hiển thị trong ứng dụng.
struct AttachmentThumb: View {
    let att: AttDraft
    var size = CGSize(width: 72, height: 90)
    @ObservedObject private var m = PeopleModel.shared

    var body: some View {
        Group {
            if let img = image {
                Image(decorative: img, scale: 1).resizable().scaledToFill()
            } else {
                Image(systemName: "doc.text").font(.system(size: 28)).foregroundStyle(C.ink2)
                    .frame(maxWidth: .infinity, maxHeight: .infinity).background(C.paper)
            }
        }
        .frame(width: size.width, height: size.height)
        .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
        .overlay(RoundedRectangle(cornerRadius: 8, style: .continuous).stroke(C.rule))
        .accessibilityHidden(true)
    }

    private var image: CGImage? {
        if let n = att.new { return n.isPdf ? nil : PhotoProcessor.cgImage(n.data, maxPixel: 400) }
        guard let f = att.fileId, m.attachmentsMeta[f]?["mime"].str != "application/pdf" else { return nil }
        return PhotoCache.shared.image("att:" + f) { m.attachmentData(f) }
    }
}

// MARK: - Thêm / sửa hồ sơ

struct PersonFormView: View {
    let kind: String
    let editId: String?
    @Binding var path: [PRoute]
    @ObservedObject private var m = PeopleModel.shared
    @State private var d: PersonDraft
    @State private var original: PersonDraft
    @State private var last: PersonDraft
    @State private var touched: Set<String> = []
    @State private var submitted = false
    @State private var saveError: String?
    @State private var confirmDiscard = false
    @State private var photoError: String?
    @State private var noticeOpen: Bool
    @State private var memorialOpen: Bool
    @State private var trayBusy = false
    @State private var trayError: String?
    private let focusTray: Bool

    init(kind: String, editId: String?, path: Binding<[PRoute]>, focusTray: Bool = false) {
        self.kind = kind
        self.editId = editId
        self.focusTray = focusTray
        _memorialOpen = State(initialValue: focusTray)
        _path = path
        let m = PeopleModel.shared
        var draft = editId.flatMap { m.person($0) }.map { PersonDraft(person: $0, settings: m.settings) }
            ?? PersonDraft(kind: kind, settings: m.settings)
        var startSubmitted = false
        #if DEBUG
        if editId == nil, let pf = UserDefaults.standard.string(forKey: "lvhvPrefill") {
            DemoSeed.prefill(&draft, pf)
            startSubmitted = pf == "loi"
        }
        #endif
        _submitted = State(initialValue: startSubmitted)
        _d = State(initialValue: draft)
        _original = State(initialValue: draft)
        _last = State(initialValue: draft)
        _noticeOpen = State(initialValue: !draft.notice.isEmpty)
    }

    private var deceased: Bool { d.deceased }
    private var live: FieldErrors { d.validate(today: m.today) }

    private func err(_ key: String) -> String? {
        let group = String(key.split(separator: ".").first ?? "")
        guard submitted || touched.contains(key) || touched.contains(group), let msg = live[key] else { return nil }
        // Đang gõ dở ngày: chưa báo "Hãy nhập đủ…" cho tới khi bấm Lưu (08 §3.4 TK-03, bản native).
        if !submitted && msg.hasPrefix("Hãy nhập") { return nil }
        return msg
    }

    var body: some View {
        ScrollViewReader { proxy in
            form
                .onAppear {
                    guard focusTray else { return }
                    // Mở từ màn hình tưởng niệm (08 §16.1): cuộn tới mục "Ảnh mâm lễ ngày giỗ".
                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.4) {
                        withAnimation { proxy.scrollTo("form.traySection", anchor: .top) }
                    }
                }
        }
    }

    private var form: some View {
        let errs = live
        return ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                VStack(alignment: .leading, spacing: 4) {
                    Text(editId == nil ? (deceased ? "Thêm người đã mất" : "Thêm người đang sống") : "Sửa hồ sơ")
                        .serif(28, .semibold).foregroundStyle(C.ink).accessibilityAddTraits(.isHeader)
                    if editId != nil { Text(PFmt.displayName(d.base)).ps(17).foregroundStyle(C.ink2) }
                }
                if submitted { ErrorSummary(errors: errs) }
                identityCard
                if deceased { deathCard }
                birthCard
                if deceased {
                    burialCard
                    PCard {
                        DisclosureGroup(isExpanded: $noticeOpen) {
                            NoticeEditor(n: $d.notice, errors: submitted || touched.contains("notice") ? errs : FieldErrors(),
                                         today: m.today)
                                .padding(.top, 10)
                        } label: {
                            Text("Thông tin báo tử (không bắt buộc)").serif(19, .semibold).foregroundStyle(C.ink)
                        }
                        .accessibilityIdentifier("form.notice")
                    }
                    PCard {
                        RuleEditor(rule: $d.gio, title: "Nhắc ngày giỗ", tienThuong: true,
                                   errors: submitted || touched.contains("gio") ? errs : FieldErrors(), prefix: "gio")
                    }
                    memorialCard
                } else {
                    livingReminders(errs)
                }
                buttons
            }
            .pageWidth(680)
        }
        .scrollDismissesKeyboard(.interactively)
        .navigationTitle(editId == nil ? "Thêm hồ sơ" : "Sửa hồ sơ")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        .navigationBarBackButtonHidden(true)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) { Button("Hủy") { cancel() } }
            ToolbarItem(placement: .confirmationAction) {
                Button("Lưu") { save() }.disabled(trayBusy).accessibilityIdentifier("form.saveTop")
            }
        }
        .onChange(of: d) { new in
            touched.formUnion(Self.changed(last, new))
            last = new
        }
        .alert("Bỏ các thay đổi chưa lưu?", isPresented: $confirmDiscard) {
            Button("Tiếp tục sửa", role: .cancel) {}
            Button("Bỏ thay đổi", role: .destructive) { if !path.isEmpty { path.removeLast() } }
        }
        .alert("Chưa lưu được", isPresented: Binding(get: { saveError != nil }, set: { if !$0 { saveError = nil } })) {
            Button("Đóng", role: .cancel) {}
        } message: { Text(saveError ?? "") }
    }

    static func changed(_ a: PersonDraft, _ b: PersonDraft) -> Set<String> {
        var s = Set<String>()
        if a.danhXung != b.danhXung { s.insert("danh_xung") }
        if a.hoTen != b.hoTen { s.insert("ho_ten") }
        if a.tenHieu != b.tenHieu { s.insert("ten_hieu") }
        if a.quanHe != b.quanHe { s.insert("quan_he") }
        if a.ghiChu != b.ghiChu { s.insert("ghi_chu") }
        if a.death != b.death { s.insert("death") }
        if a.birth != b.birth { s.insert("birth") }
        if a.noiSinh != b.noiSinh { s.insert("noi_sinh") }
        if a.noiAnTang != b.noiAnTang { s.insert("noi_an_tang") }
        if a.tuoiText != b.tuoiText || a.tuoiManual != b.tuoiManual { s.insert("tuoi") }
        if a.notice != b.notice { s.insert("notice") }
        if a.gio != b.gio { s.insert("gio") }
        if a.sinhNhat != b.sinhNhat { s.insert("sinh_nhat") }
        if a.sinhNhatAm != b.sinhNhatAm { s.insert("sinh_nhat_am") }
        if a.loiTuongNiem != b.loiTuongNiem { s.insert("loi_tuong_niem") }
        if a.nguong != b.nguong { s.insert("nguong") }
        if a.trayPhotoId != b.trayPhotoId || a.newTray != b.newTray { s.insert("tray") }
        return s
    }

    private var identityCard: some View {
        PCard {
            PHeading(deceased ? "Người đã mất" : "Người đang sống")
            LabeledField(label: "Giới tính") {
                SegmentChips(options: [SegOpt("nam", "Nam"), SegOpt("nu", "Nữ"), SegOpt("", "Không ghi")],
                             selection: Binding(get: { d.gioiTinh ?? "" }, set: { d.gioiTinh = $0.isEmpty ? nil : $0 }),
                             idPrefix: "form.gioiTinh")
            }
            PTextField(label: "Cách gọi (đứng trước tên)", text: $d.danhXung, error: err("danh_xung"),
                       suggestions: Suggest.danhXung(deceased: deceased, gioiTinh: d.gioiTinh), id: "form.danhXung")
            PTextField(label: "Họ và tên *", text: $d.hoTen, error: err("ho_ten"), id: "form.hoTen")
            if deceased {
                PTextField(label: "Tên hiệu / pháp danh", text: $d.tenHieu, error: err("ten_hieu"), id: "form.tenHieu")
            }
            PTextField(label: "Quan hệ với bạn", text: $d.quanHe,
                       hint: deceased ? "Trang tưởng niệm ghi quan hệ này phía trên tên, ví dụ “Ông nội”, “Mẹ”. Để trống thì chỉ ghi tên." : nil,
                       error: err("quan_he"),
                       suggestions: Suggest.quanHe(deceased: deceased), id: "form.quanHe")
            PTextField(label: "Ghi chú", text: $d.ghiChu, error: err("ghi_chu"), multiline: true, id: "form.ghiChu")
        }
    }

    private var deathCard: some View {
        PCard {
            PHeading("Ngày mất *")
            DateInputView(draft: $d.death, purpose: .death, allowUnknownYear: true, error: err("death"),
                          today: m.today, idPrefix: "death")
        }
    }

    private var birthCard: some View {
        PCard {
            PHeading(deceased ? "Ngày sinh và nơi sinh" : "Ngày sinh")
            PFine("Không bắt buộc.")
            DateInputView(draft: $d.birth, purpose: .birth, allowYearOnly: true, living: !deceased, error: err("birth"),
                          today: m.today, idPrefix: "birth")
            if !deceased && d.isFeb29 {
                Picker("Năm không nhuận", selection: $d.feb29) {
                    Text("Năm không nhuận: mừng 28/02").tag("feb28")
                    Text("… mừng 01/03").tag("mar1")
                }
                .pickerStyle(.inline)
            }
            PTextField(label: "Nơi sinh", text: $d.noiSinh, error: err("noi_sinh"), id: "form.noiSinh")
            if deceased && DateCheck.death(d.death, today: m.today).ok { ageBlock }
        }
    }

    @ViewBuilder
    private var ageBlock: some View {
        VStack(alignment: .leading, spacing: 8) {
            if d.tuoiManual {
                HStack(alignment: .bottom, spacing: 10) {
                    NumField(label: "Tuổi (1–150)", text: $d.tuoiText, width: 64, id: "form.tuoi")
                    Button("Tính lại") { d.tuoiManual = false }.buttonStyle(SecondaryButtonStyle())
                }
                FieldError(text: err("tuoi"))
            } else {
                Text(d.ageLine(today: m.today) ?? "Chưa đủ năm sinh để tính tuổi")
                    .ps(16, .semibold).foregroundStyle(C.ink).accessibilityIdentifier("form.age")
                Button("Sửa tuổi") { d.tuoiManual = true }.buttonStyle(SecondaryButtonStyle())
            }
        }
    }

    private var burialCard: some View {
        PCard {
            PHeading("Nơi an táng và ảnh")
            PTextField(label: "Nơi an táng", text: $d.noiAnTang, error: err("noi_an_tang"), id: "form.noiAnTang")
            Text("Ảnh chân dung").ps(15, .semibold).foregroundStyle(C.ink)
            HStack(alignment: .center, spacing: 14) {
                PhotoView(photoId: d.photoId, newPhoto: d.newPhoto, initials: PFmt.initials(d.hoTen),
                          size: CGSize(width: 96, height: 120))
                VStack(alignment: .leading, spacing: 8) {
                    PhotoPickButton(title: d.photoId == nil && d.newPhoto == nil ? "Chọn ảnh" : "Đổi ảnh") { r in
                        switch r {
                        case .success(let p): d.newPhoto = p; photoError = nil
                        case .failure(let e): photoError = e.message
                        }
                    }
                    .accessibilityLabel(d.photoId == nil && d.newPhoto == nil ? "Chọn ảnh chân dung" : "Đổi ảnh chân dung")
                    if d.photoId != nil || d.newPhoto != nil {
                        Button("Bỏ ảnh") { d.photoId = nil; d.newPhoto = nil }
                            .buttonStyle(SecondaryButtonStyle(destructive: true))
                            .accessibilityLabel("Bỏ ảnh chân dung")
                    }
                }
            }
            FieldError(text: photoError)
            PFine("Ảnh chỉ lưu trong ứng dụng trên máy này, đã bỏ thông tin vị trí và máy chụp.")
        }
    }

    private var memorialCard: some View {
        PCard {
            DisclosureGroup(isExpanded: $memorialOpen) {
                VStack(alignment: .leading, spacing: 14) {
                    Text("Giỗ hết (giỗ lần cuối của thời gian để tang) vào").ps(15, .semibold).foregroundStyle(C.ink)
                    Picker("Giỗ hết", selection: $d.gioHetNam) {
                        Text("Năm thứ 2 sau năm mất").tag(2)
                        Text("Năm thứ 3 sau năm mất").tag(3)
                    }
                    .pickerStyle(.inline)
                    .labelsHidden()
                    PTextField(label: "Lời tưởng niệm", text: $d.loiTuongNiem, error: err("loi_tuong_niem"), multiline: true,
                               id: "form.loiTuongNiem")
                    traySection.id("form.traySection")
                    LabeledField(label: "Mất trước tuổi này thì bia ghi “hưởng dương” (mất sớm) thay cho “hưởng thọ”",
                                 error: err("nguong")) {
                        NumField(label: "Tuổi", text: $d.nguong, width: 64, id: "form.nguong")
                    }
                }
                .padding(.top, 10)
            } label: {
                Text("Tưởng niệm").serif(19, .semibold).foregroundStyle(C.ink)
            }
        }
    }

    @ViewBuilder
    private func livingReminders(_ errs: FieldErrors) -> some View {
        PCard {
            RuleEditor(rule: $d.sinhNhat, title: "Nhắc sinh nhật",
                       errors: submitted || touched.contains("sinh_nhat") ? errs : FieldErrors(), prefix: "sinh_nhat")
            Divider()
            Toggle("Nhắc cả sinh nhật âm lịch hằng năm", isOn: $d.sinhNhatAmOn)
                .ps(16)
                .disabled(!d.birthFullDay)
            if !d.birthFullDay { PFine("Cần biết đủ ngày sinh") }
            if d.sinhNhatAmOn && d.birthFullDay {
                Toggle("Nhắc sinh nhật âm lịch giống nhắc sinh nhật dương lịch", isOn: $d.amSame).ps(16)
            }
        }
        if d.sinhNhatAmOn && d.birthFullDay && !d.amSame {
            PCard {
                RuleEditor(rule: $d.sinhNhatAm, title: "Nhắc sinh nhật âm lịch",
                           errors: submitted || touched.contains("sinh_nhat_am") ? errs : FieldErrors(), prefix: "sinh_nhat_am")
            }
        }
    }

    private var buttons: some View {
        HStack(spacing: 12) {
            Spacer()
            Button("Hủy") { cancel() }.buttonStyle(SecondaryButtonStyle())
            Button("Lưu") { save() }.buttonStyle(PrimaryButtonStyle()).disabled(trayBusy).accessibilityIdentifier("form.save")
        }
        .padding(.top, 4)
    }

    private func cancel() {
        if d != original { confirmDiscard = true } else if !path.isEmpty { path.removeLast() }
    }

    /// Mục "Ảnh mâm lễ ngày giỗ" (08 §16.2), ngay sau "Lời tưởng niệm".
    private var traySection: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Ảnh mâm lễ ngày giỗ").ps(15, .semibold).foregroundStyle(C.ink).accessibilityAddTraits(.isHeader)
            if trayBusy {
                HStack(spacing: 10) {
                    ProgressView()
                    Text("Đang xử lý ảnh…").ps(15).foregroundStyle(C.ink2)
                }
                .accessibilityElement(children: .combine)
                .accessibilityIdentifier("form.tray.busy")
            }
            if d.hasTray {
                HStack(alignment: .center, spacing: 14) {
                    // N-2: PhotoView ẩn khỏi trợ năng; bọc bằng một phần tử ảnh có nhãn và định danh.
                    ZStack {
                        PhotoView(photoId: d.trayPhotoId, newPhoto: d.newTray, initials: "", size: CGSize(width: 120, height: 90),
                                  oval: false)
                    }
                        .accessibilityElement(children: .ignore)
                        .accessibilityAddTraits(.isImage)
                        .accessibilityLabel("Ảnh mâm lễ đã chọn")
                        .accessibilityIdentifier("form.tray.thumb")
                    VStack(alignment: .leading, spacing: 8) {
                        TrayPickButton(title: "Đổi ảnh", a11y: "Đổi ảnh mâm lễ", busy: $trayBusy, onPicked: trayPicked)
                        Button("Bỏ ảnh") { d.trayPhotoId = nil; d.newTray = nil; trayError = nil }
                            .buttonStyle(SecondaryButtonStyle(destructive: true))
                            .disabled(trayBusy)
                            .accessibilityLabel("Bỏ ảnh mâm lễ")
                            .accessibilityIdentifier("form.tray.remove")
                    }
                }
            } else {
                if !trayBusy {
                    Text("Chưa có ảnh. Ngày giỗ chỉ hiện bát hương và nến.").ps(15).foregroundStyle(C.ink2)
                        .fixedSize(horizontal: false, vertical: true)
                        .accessibilityIdentifier("form.tray.none")
                }
                // G-3: nút (chủ của hộp chọn ảnh và tác vụ xử lý) luôn ở trong cây; khi đang xử lý chỉ bị khóa.
                TrayPickButton(title: "Chọn ảnh mâm lễ…", busy: $trayBusy, onPicked: trayPicked)
            }
            FieldError(text: trayError)
            PFine("Ảnh chỉ lưu trên máy này và trong tệp sao lưu mã hóa. Ứng dụng tự bỏ thông tin vị trí và máy chụp trong ảnh.")
        }
    }

    private func trayPicked(_ r: Result<ProcessedPhoto, PhotoError>) {
        switch r {
        case .success(let p): d.newTray = p; trayError = nil
        case .failure(let e): trayError = TrayPhoto.message(e)
        }
    }

    private func save() {
        submitted = true
        guard !trayBusy, d.validate(today: m.today).isEmpty else { return }
        var draft = d
        var addedPhoto: String?
        var addedTray: String?
        var addedFiles: [String] = []
        do {
            if let np = draft.newPhoto { let id = try m.addPhoto(np); addedPhoto = id; draft.photoId = id; draft.newPhoto = nil }
            // Ảnh mâm lễ chỉ ghi vào kho lúc Lưu; Hủy thì không có gì để xóa. Ảnh cũ bị thay/bỏ do lõi xóa khi lưu.
            if draft.deceased, let nt = draft.newTray {
                let id = try m.addPhoto(nt); addedTray = id; draft.trayPhotoId = id; draft.newTray = nil
            }
            if !draft.deceased { draft.trayPhotoId = nil; draft.newTray = nil }
            addedFiles = try PeopleModel.uploadAttachments(&draft.notice)
            let json = draft.build(photoId: draft.photoId, today: m.today)
            let saved = try m.savePerson(json)
            d = draft
            let id = saved["id"].string
            original = d
            if !path.isEmpty { path.removeLast() }
            if editId == nil && !id.isEmpty { path.append(.person(id)) }
        } catch {
            // Lưu hỏng: xóa ảnh và tệp vừa ghi mà chưa gắn hồ sơ (A-6); dữ liệu đang nhập vẫn trên màn hình.
            if let p = addedPhoto { m.discardPhoto(p) }
            if let t = addedTray { m.discardPhoto(t) }
            for f in addedFiles { m.discardAttachment(f) }
            saveError = "Chưa lưu được: \(Core.message(error)). Dữ liệu đang nhập vẫn còn trên màn hình."
        }
    }
}

// MARK: - Cập nhật: đã mất (US-M-07)

struct MarkDeceasedView: View {
    let id: String
    @Binding var path: [PRoute]
    @ObservedObject private var m = PeopleModel.shared
    @State private var death = MarkDeceasedView.demoDeath
    @State private var notice = NoticeDraft()
    @State private var noticeOpen = true

    private static var demoDeath: DateDraft {
        var d = DateDraft()
        #if DEBUG
        if UserDefaults.standard.string(forKey: "lvhvPrefill") == "mark" { d.day = "1"; d.month = "11"; d.year = "2026" }
        #endif
        return d
    }
    @State private var submitted = false
    @State private var confirm = false
    @State private var saveError: String?
    @State private var confirmDiscard = false

    private var name: String { m.person(id).map(PFmt.displayName) ?? "" }
    private var dirty: Bool { !death.isEmpty || !notice.isEmpty }

    private func cancel() {
        if dirty { confirmDiscard = true } else if !path.isEmpty { path.removeLast() }
    }

    private func trySave() {
        submitted = true
        if errors.isEmpty { confirm = true }
    }

    private var errors: FieldErrors {
        var e = FieldErrors()
        if let msg = DateCheck.death(death, today: m.today).error { e.set("death", msg) }
        e.merge(notice.validate(today: m.today))
        return e
    }

    var body: some View {
        let errs = errors
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Cập nhật: đã mất").serif(28, .semibold).foregroundStyle(C.ink).accessibilityAddTraits(.isHeader)
                    Text(name).ps(18, .semibold).foregroundStyle(C.ink2)
                }
                PBody("Ghi lại ngày mất của \(name). Hồ sơ, ảnh và ngày sinh được giữ nguyên; ứng dụng sẽ nhắc ngày giỗ hằng năm.")
                if submitted { ErrorSummary(errors: errs) }
                PCard {
                    PHeading("Ngày mất *")
                    DateInputView(draft: $death, purpose: .death, allowUnknownYear: true,
                                  error: submitted ? errs["death"] : errs["death"].flatMap { $0.hasPrefix("Hãy nhập") ? nil : $0 },
                                  today: m.today, idPrefix: "death")
                }
                PCard {
                    DisclosureGroup(isExpanded: $noticeOpen) {
                        NoticeEditor(n: $notice, errors: submitted ? errs : FieldErrors(), today: m.today).padding(.top, 10)
                    } label: {
                        Text("Thông tin báo tử (không bắt buộc)").serif(19, .semibold).foregroundStyle(C.ink)
                    }
                }
                HStack(spacing: 12) {
                    Spacer()
                    Button("Hủy") { cancel() }.buttonStyle(SecondaryButtonStyle())
                    Button("Lưu") { trySave() }
                    .buttonStyle(PrimaryButtonStyle())
                    .accessibilityIdentifier("mark.save")
                }
            }
            .pageWidth(680)
        }
        .scrollDismissesKeyboard(.interactively)
        .navigationTitle("Cập nhật: đã mất")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        // Như các biểu mẫu khác: ẩn nút quay lại (và cử chỉ vuốt) để không mất dữ liệu đang nhập (TKN-I-09).
        .navigationBarBackButtonHidden(true)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) { Button("Hủy") { cancel() } }
            ToolbarItem(placement: .confirmationAction) { Button("Lưu") { trySave() } }
        }
        .alert("Bỏ các thay đổi chưa lưu?", isPresented: $confirmDiscard) {
            Button("Tiếp tục sửa", role: .cancel) {}
            Button("Bỏ thay đổi", role: .destructive) { if !path.isEmpty { path.removeLast() } }
        }
        .alert("Cập nhật \(name) là người đã mất?", isPresented: $confirm) {
            Button("Quay lại", role: .cancel) {}
            Button("Lưu thay đổi") { apply() }
        } message: {
            Text(confirmLines.joined(separator: "\n"))
        }
        .alert("Chưa lưu được", isPresented: Binding(get: { saveError != nil }, set: { if !$0 { saveError = nil } })) {
            Button("Đóng", role: .cancel) {}
        } message: { Text(saveError ?? "") }
    }

    /// Các thay đổi sẽ xảy ra (08 §12g).
    var confirmLines: [String] {
        var lines: [String] = []
        let hasEvents = !m.events(of: id).isEmpty
        lines.append("Sẽ không còn nhắc sinh nhật\(hasEvents ? " và sự kiện" : "") của \(name).")
        let c = DateCheck.death(death, today: m.today)
        let rule = m.person(id)?["reminders"]["gio"].isObject == true ? m.person(id)!["reminders"]["gio"]
            : ["offsets_days": m.settings["default_offsets_days"], "time_local": m.settings["default_time_local"]]
        let offs = PFmt.offsets(rule).lowercased()
        lines.append("Bắt đầu nhắc ngày giỗ hằng năm vào \(PFmt.gioText(c.derived["lunar"])) (\(offs), lúc \(rule["time_local"].string)).")
        lines.append("Họ tên, ảnh, ngày sinh và sự kiện vẫn được giữ trong hồ sơ.")
        if let s = notice.summaryLine { lines.append(s) }
        return lines
    }

    private func apply() {
        let c = DateCheck.death(death, today: m.today)
        guard c.ok else { return }
        var n = notice
        var added: [String] = []
        do {
            added = try PeopleModel.uploadAttachments(&n)
            _ = try m.markDeceased(id: id, death: c.value, notice: n.json(today: m.today))
            notice = n
            if !path.isEmpty { path.removeLast() }
        } catch {
            for f in added { m.discardAttachment(f) }
            saveError = "Chưa lưu được: \(Core.message(error)). Dữ liệu đang nhập vẫn còn trên màn hình."
        }
    }
}
