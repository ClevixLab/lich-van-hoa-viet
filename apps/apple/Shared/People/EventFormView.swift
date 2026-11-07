// Thêm / sửa sự kiện của một người (08 §3.7; B4: sự kiện của người đã mất không có khối nhắc).
import SwiftUI

struct EventFormView: View {
    let personId: String
    let eventId: String?
    @Binding var path: [PRoute]
    @ObservedObject private var m = PeopleModel.shared
    @State private var d: EventDraft
    @State private var original: EventDraft
    @State private var submitted = false
    @State private var confirmDiscard = false
    @State private var saveError: String?

    init(personId: String, eventId: String?, path: Binding<[PRoute]>) {
        self.personId = personId
        self.eventId = eventId
        _path = path
        let m = PeopleModel.shared
        let draft: EventDraft
        if let eid = eventId, let e = m.events.first(where: { $0["id"].str == eid }) {
            draft = EventDraft(event: e, settings: m.settings)
        } else {
            draft = EventDraft(personId: personId, types: m.eventTypes, settings: m.settings)
        }
        _d = State(initialValue: draft)
        _original = State(initialValue: draft)
    }

    private var deceased: Bool { m.person(personId)?["kind"].str == "deceased" }

    var body: some View {
        let errs = d.validate(personDeceased: deceased)
        let show = submitted ? errs : FieldErrors()
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                VStack(alignment: .leading, spacing: 4) {
                    Text(eventId == nil ? "Thêm sự kiện" : "Sửa sự kiện").serif(28, .semibold).foregroundStyle(C.ink)
                        .accessibilityAddTraits(.isHeader)
                    Text(m.person(personId).map(PFmt.displayName) ?? "").ps(17).foregroundStyle(C.ink2)
                }
                if submitted { ErrorSummary(errors: errs) }
                PCard {
                    LabeledField(label: "Loại *", error: show["type"]) {
                        Picker("Loại", selection: $d.typeId) {
                            ForEach(m.eventTypes, id: \.self) { t in Text(t["ten"].string).tag(t["id"].string) }
                            Text("Loại khác…").tag(EventDraft.newType)
                        }
                        .pickerStyle(.menu)
                        .labelsHidden()
                        .accessibilityIdentifier("event.type")
                    }
                    if d.typeId == EventDraft.newType {
                        PTextField(label: "Tên loại mới", text: $d.newTypeName, id: "event.newType")
                    }
                    PTextField(label: "Tiêu đề", text: $d.title, hint: "Để trống sẽ dùng tên loại", error: show["tieu_de"],
                               id: "event.title")
                    PTextField(label: "Ghi chú", text: $d.note, error: show["ghi_chu"], multiline: true, id: "event.note")
                }
                PCard {
                    PHeading("Ngày *")
                    DateInputView(draft: $d.date, purpose: .event, allowUnknownYear: d.recurring,
                                  error: submitted ? errs["date"] : errs["date"].flatMap { $0.hasPrefix("Hãy nhập") ? nil : $0 },
                                  today: m.today, idPrefix: "event.date")
                    Toggle("Lặp lại hằng năm", isOn: $d.recurring).ps(16).accessibilityIdentifier("event.recurring")
                }
                PCard {
                    if deceased {
                        PHeading("Nhắc")
                        PBody("Không nhắc sự kiện của người đã mất — với người đã mất, ứng dụng chỉ nhắc ngày giỗ. Sự kiện vẫn được lưu và hiện trên hồ sơ.")
                    } else {
                        RuleEditor(rule: $d.rule, title: "Nhắc sự kiện", errors: show, prefix: "rule")
                    }
                }
                HStack(spacing: 12) {
                    Spacer()
                    Button("Hủy") { cancel() }.buttonStyle(SecondaryButtonStyle())
                    Button("Lưu") { save() }.buttonStyle(PrimaryButtonStyle()).accessibilityIdentifier("event.save")
                }
            }
            .pageWidth(680)
        }
        .scrollDismissesKeyboard(.interactively)
        .navigationTitle(eventId == nil ? "Thêm sự kiện" : "Sửa sự kiện")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        .navigationBarBackButtonHidden(true)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) { Button("Hủy") { cancel() } }
            ToolbarItem(placement: .confirmationAction) { Button("Lưu") { save() } }
        }
        .onChange(of: d.typeId) { t in
            if eventId == nil, let ty = m.eventTypes.first(where: { $0["id"].str == t }) {
                d.recurring = ty["default_recurring"].isTrue
            }
        }
        .alert("Bỏ các thay đổi chưa lưu?", isPresented: $confirmDiscard) {
            Button("Tiếp tục sửa", role: .cancel) {}
            Button("Bỏ thay đổi", role: .destructive) { if !path.isEmpty { path.removeLast() } }
        }
        .alert("Chưa lưu được", isPresented: Binding(get: { saveError != nil }, set: { if !$0 { saveError = nil } })) {
            Button("Đóng", role: .cancel) {}
        } message: { Text(saveError ?? "") }
    }

    private func cancel() {
        if d != original { confirmDiscard = true } else if !path.isEmpty { path.removeLast() }
    }

    private func save() {
        submitted = true
        guard d.validate(personDeceased: deceased).isEmpty else { return }
        do {
            var tid = d.typeId
            if tid == EventDraft.newType {
                tid = try m.eventType(named: Text1.clean(d.newTypeName), recurring: d.recurring)
            }
            try m.saveEvent(d.build(typeId: tid, personDeceased: deceased))
            original = d
            if !path.isEmpty { path.removeLast() }
        } catch {
            saveError = "Chưa lưu được: \(Core.message(error)). Dữ liệu đang nhập vẫn còn trên màn hình."
        }
    }
}
