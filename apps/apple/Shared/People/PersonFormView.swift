// Biểu mẫu thêm/sửa người đã mất (08 §3.4), người đang sống (§3.5) và "Cập nhật: đã mất" (§12g, US-M-07).
import PhotosUI
import SwiftUI

/// Gợi ý từ thường ngày (quyết định của chủ dự án: không dùng từ Hán-Việt cổ, không có ô "dòng đầu bia").
enum Suggest {
    static let danhXung = ["Cụ", "Kỵ", "Ông", "Bà", "Bác", "Chú", "Thím", "Cô", "Dì", "Cậu", "Mợ", "Anh", "Chị", "Em",
                           "Bố", "Mẹ", "Cha", "Má", "Ba"]
    static let quanHeDeceased = ["Bố", "Mẹ", "Ông nội", "Bà nội", "Ông ngoại", "Bà ngoại", "Cụ ông", "Cụ bà", "Kỵ ông",
                                 "Kỵ bà", "Ông cố", "Bà cố", "Ông sơ", "Bà sơ", "Bác", "Chú", "Thím", "Cô", "Dì", "Cậu",
                                 "Mợ", "Anh", "Chị", "Em", "Chồng", "Vợ", "Con"]
    static let quanHeLiving = ["Bố", "Mẹ", "Ông nội", "Bà nội", "Ông ngoại", "Bà ngoại", "Bác", "Chú", "Thím", "Cô", "Dì",
                               "Cậu", "Mợ", "Anh", "Chị", "Em", "Chồng", "Vợ", "Con trai", "Con gái", "Cháu"]
}

/// Nút chọn ảnh (PhotosPicker — không cần quyền thư viện ảnh); ảnh được xử lý S9 ở luồng nền.
struct PhotoPickButton: View {
    let title: String
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
                let r: Result<ProcessedPhoto, PhotoError> = await Task.detached(priority: .userInitiated) {
                    guard let data else { return .failure(PhotoError.unreadable) }
                    do { return .success(try PhotoProcessor.process(data)) }
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

/// Khối "Thông tin báo tử (không bắt buộc)" — 01 §3.2a.
struct NoticeEditor: View {
    @Binding var n: NoticeDraft
    var errors: FieldErrors
    let today: CivilDate
    @State private var photoError: String?

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
            VStack(alignment: .leading, spacing: 8) {
                Text("Ảnh giấy báo tử (tối đa 4)").ps(15, .semibold).foregroundStyle(C.ink)
                FlowLayout(spacing: 10) {
                    ForEach(n.photoIds, id: \.self) { pid in
                        thumb(PhotoView(photoId: pid, initials: "", size: CGSize(width: 96, height: 120), oval: false)) {
                            n.photoIds.removeAll { $0 == pid }
                        }
                    }
                    ForEach(Array(n.newPhotos.enumerated()), id: \.offset) { i, p in
                        thumb(PhotoView(newPhoto: p, initials: "", size: CGSize(width: 96, height: 120), oval: false)) {
                            if n.newPhotos.indices.contains(i) { n.newPhotos.remove(at: i) }
                        }
                    }
                }
                if n.photoCount < 4 {
                    PhotoPickButton(title: "Thêm ảnh giấy báo tử") { r in
                        switch r {
                        case .success(let p): n.newPhotos.append(p); photoError = nil
                        case .failure(let e): photoError = e.message
                        }
                    }
                }
                FieldError(text: photoError ?? errors["notice.photos"])
                PFine("Ảnh chỉ lưu trong ứng dụng trên máy này, đã bỏ thông tin vị trí và máy chụp.")
            }
            PTextField(label: "Ghi chú", text: $n.note, error: errors["notice.note"], multiline: true, id: "notice.note")
        }
    }

    private func thumb(_ v: PhotoView, remove: @escaping () -> Void) -> some View {
        VStack(spacing: 6) {
            v
            Button("Bỏ ảnh", action: remove).buttonStyle(SecondaryButtonStyle(destructive: true))
        }
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
    @State private var memorialOpen = false

    init(kind: String, editId: String?, path: Binding<[PRoute]>) {
        self.kind = kind
        self.editId = editId
        _path = path
        let m = PeopleModel.shared
        let draft = editId.flatMap { m.person($0) }.map { PersonDraft(person: $0, settings: m.settings) }
            ?? PersonDraft(kind: kind, settings: m.settings)
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
        let errs = live
        ScrollView {
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
            ToolbarItem(placement: .confirmationAction) { Button("Lưu") { save() }.accessibilityIdentifier("form.saveTop") }
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
        return s
    }

    private var identityCard: some View {
        PCard {
            PHeading(deceased ? "Người đã mất" : "Người đang sống")
            PTextField(label: "Cách gọi (đứng trước tên)", text: $d.danhXung, error: err("danh_xung"),
                       suggestions: Suggest.danhXung, id: "form.danhXung")
            PTextField(label: "Họ và tên *", text: $d.hoTen, error: err("ho_ten"), id: "form.hoTen")
            if deceased {
                PTextField(label: "Tên hiệu / pháp danh", text: $d.tenHieu, error: err("ten_hieu"), id: "form.tenHieu")
            }
            PTextField(label: "Quan hệ với bạn", text: $d.quanHe,
                       hint: deceased ? "Trang tưởng niệm ghi quan hệ này phía trên tên, ví dụ “Ông nội”, “Mẹ”. Để trống thì chỉ ghi tên." : nil,
                       error: err("quan_he"),
                       suggestions: deceased ? Suggest.quanHeDeceased : Suggest.quanHeLiving, id: "form.quanHe")
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
                    if d.photoId != nil || d.newPhoto != nil {
                        Button("Bỏ ảnh") { d.photoId = nil; d.newPhoto = nil }
                            .buttonStyle(SecondaryButtonStyle(destructive: true))
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
            Button("Lưu") { save() }.buttonStyle(PrimaryButtonStyle()).accessibilityIdentifier("form.save")
        }
        .padding(.top, 4)
    }

    private func cancel() {
        if d != original { confirmDiscard = true } else if !path.isEmpty { path.removeLast() }
    }

    private func save() {
        submitted = true
        guard d.validate(today: m.today).isEmpty else { return }
        do {
            if let np = d.newPhoto { d.photoId = try m.addPhoto(np); d.newPhoto = nil }
            while let np = d.notice.newPhotos.first {
                d.notice.photoIds.append(try m.addPhoto(np))
                d.notice.newPhotos.removeFirst()
            }
            let json = d.build(photoId: d.photoId, noticePhotoIds: d.notice.photoIds, today: m.today)
            let saved = try m.savePerson(json)
            let id = saved["id"].string
            original = d
            if !path.isEmpty { path.removeLast() }
            if editId == nil && !id.isEmpty { path.append(.person(id)) }
        } catch {
            saveError = "Chưa lưu được: \(Core.message(error)). Dữ liệu đang nhập vẫn còn trên màn hình."
        }
    }
}

// MARK: - Cập nhật: đã mất (US-M-07)

struct MarkDeceasedView: View {
    let id: String
    @Binding var path: [PRoute]
    @ObservedObject private var m = PeopleModel.shared
    @State private var death = DateDraft()
    @State private var notice = NoticeDraft()
    @State private var noticeOpen = true
    @State private var submitted = false
    @State private var confirm = false
    @State private var saveError: String?

    private var name: String { m.person(id).map(PFmt.displayName) ?? "" }

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
                    Button("Hủy") { if !path.isEmpty { path.removeLast() } }.buttonStyle(SecondaryButtonStyle())
                    Button("Lưu") {
                        submitted = true
                        if errors.isEmpty { confirm = true }
                    }
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
        lines.append("Bắt đầu nhắc ngày giỗ hằng năm vào \(PFmt.lunarText(c.derived["lunar"])) (\(offs), lúc \(rule["time_local"].string)).")
        lines.append("Họ tên, ảnh, ngày sinh và sự kiện vẫn được giữ trong hồ sơ.")
        if let s = notice.summaryLine { lines.append(s) }
        return lines
    }

    private func apply() {
        let c = DateCheck.death(death, today: m.today)
        guard c.ok else { return }
        do {
            while let np = notice.newPhotos.first {
                notice.photoIds.append(try m.addPhoto(np))
                notice.newPhotos.removeFirst()
            }
            _ = try m.markDeceased(id: id, death: c.value, notice: notice.json(photoIds: notice.photoIds, today: m.today))
            if !path.isEmpty { path.removeLast() }
        } catch {
            saveError = "Chưa lưu được: \(Core.message(error)). Dữ liệu đang nhập vẫn còn trên màn hình."
        }
    }
}
