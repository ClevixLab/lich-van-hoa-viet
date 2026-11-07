// Hồ sơ một người (08 §3.6, §12f, §12g): Lần tới, Thông tin, Thông tin báo tử (chỉ ở đây), Sự kiện, Nhắc, Xóa.
import QuickLook
import SwiftUI

struct PersonDetailView: View {
    let id: String
    @Binding var path: [PRoute]
    @ObservedObject private var m = PeopleModel.shared
    @State private var confirmDelete = false
    @State private var deleteEvent: JV?
    @State private var error: String?
    @State private var noticeOpen = PersonDetailView.expandDefault
    @State private var viewing: AttOpen?

    private static var expandDefault: Bool {
        #if DEBUG
        return UserDefaults.standard.bool(forKey: "lvhvExpand")
        #else
        return false
        #endif
    }

    var body: some View {
        Group {
            if let p = m.person(id) {
                content(p)
            } else {
                VStack(alignment: .leading, spacing: 12) {
                    PBody("Không tìm thấy hồ sơ này (có thể đã bị xóa).")
                    Button("Về danh sách hồ sơ") { path.removeAll() }.buttonStyle(SecondaryButtonStyle())
                }
                .pageWidth()
            }
        }
        .attachmentViewer($viewing)
        .navigationTitle("Hồ sơ")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
    }

    private func content(_ p: JV) -> some View {
        let deceased = p["kind"].str == "deceased"
        let name = PFmt.displayName(p)
        return ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                ToastBar(text: $m.toast)
                header(p, deceased: deceased, name: name)
                ViewThatFits(in: .horizontal) {
                    HStack(alignment: .top, spacing: 14) {
                        VStack(spacing: 14) { next(p, deceased: deceased); info(p, deceased: deceased); notice(p) }
                            .frame(minWidth: 380, idealWidth: 420, maxWidth: .infinity)
                        VStack(spacing: 14) { eventsCard(p, deceased: deceased); remindCard(p, deceased: deceased); deleteCard(name) }
                            .frame(width: 300)
                    }
                    .frame(minWidth: 740)
                    VStack(spacing: 14) {
                        next(p, deceased: deceased)
                        info(p, deceased: deceased)
                        notice(p)
                        eventsCard(p, deceased: deceased)
                        remindCard(p, deceased: deceased)
                        deleteCard(name)
                    }
                }
            }
            .pageWidth(1000)
        }
        .alert("Xóa hồ sơ của \(name)?", isPresented: $confirmDelete) {
            Button("Giữ lại", role: .cancel) {}
            Button("Xóa hồ sơ", role: .destructive) {
                do { try m.deletePerson(id); path.removeAll() } catch { self.error = Core.message(error) }
            }
        } message: {
            Text("Ngày giỗ, lời nhắc và ảnh của người này sẽ bị xóa khỏi máy. Sự kiện gắn với người này vẫn còn nếu có người khác cùng tham gia. Không hoàn tác được.")
        }
        .alert("Xóa sự kiện \"\(deleteEvent.map { m.eventTitle($0) } ?? "")\"?", isPresented: Binding(
            get: { deleteEvent != nil }, set: { if !$0 { deleteEvent = nil } })) {
            Button("Giữ lại", role: .cancel) { deleteEvent = nil }
            Button("Xóa sự kiện", role: .destructive) {
                if let e = deleteEvent, let eid = e["id"].str {
                    do { try m.deleteEvent(eid) } catch { self.error = Core.message(error) }
                }
                deleteEvent = nil
            }
        } message: { Text("Không hoàn tác được.") }
        .alert("Chưa thực hiện được", isPresented: Binding(get: { error != nil }, set: { if !$0 { error = nil } })) {
            Button("Đóng", role: .cancel) {}
        } message: { Text(error ?? "") }
    }

    private func header(_ p: JV, deceased: Bool, name: String) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            ViewThatFits(in: .horizontal) {
                HStack(alignment: .center, spacing: 16) {
                    if deceased { PhotoView(photoId: p["photo_id"].str, initials: PFmt.initials(p["ho_ten"].string)) }
                    titleBlock(p, name: name)
                    Spacer(minLength: 0)
                }
                VStack(alignment: .leading, spacing: 12) {
                    if deceased { PhotoView(photoId: p["photo_id"].str, initials: PFmt.initials(p["ho_ten"].string)) }
                    titleBlock(p, name: name)
                }
            }
            actions(p, deceased: deceased)
        }
    }

    private func titleBlock(_ p: JV, name: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(name).serif(26, .semibold).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
                .accessibilityAddTraits(.isHeader).accessibilityIdentifier("person.name")
            if !p["quan_he"].string.isEmpty { Text(PFmt.cap(p["quan_he"].string)).ps(16).foregroundStyle(C.ink2) }
            if !p["ten_hieu"].string.isEmpty { Text("Tên hiệu: \(p["ten_hieu"].string)").ps(15).foregroundStyle(C.ink2) }
        }
    }

    private func actions(_ p: JV, deceased: Bool) -> some View {
        FlowLayout(spacing: 10) {
            if deceased {
                NavigationLink(value: PRoute.memorial(id)) { Text("Tưởng niệm") }
                    .buttonStyle(PrimaryButtonStyle()).accessibilityIdentifier("person.memorial")
            }
            // Chế độ chỉ đọc: ẩn mọi việc ghi (08 §12l TKN3-P-01).
            if !m.readOnly {
                NavigationLink(value: PRoute.edit(id)) { Text("Sửa") }
                    .buttonStyle(SecondaryButtonStyle()).accessibilityIdentifier("person.edit")
                if !deceased {
                    NavigationLink(value: PRoute.markDeceased(id)) { Text("Cập nhật: đã mất") }
                        .buttonStyle(SecondaryButtonStyle()).accessibilityIdentifier("person.markDeceased")
                }
            }
        }
    }

    private func next(_ p: JV, deceased: Bool) -> some View {
        let list = m.next(for: id)
        return PCard {
            PHeading("Lần tới")
            if list.isEmpty {
                PBody(deceased ? "Chưa tính được ngày giỗ tới." : "Chưa có ngày sinh hay sự kiện nào trong 12 tháng tới.")
            }
            ForEach(list) { it in
                VStack(alignment: .leading, spacing: 2) {
                    Text(it.line1).ps(16, .semibold).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
                    Text("\(it.dateText) · \(PFmt.distance(it.civil.dayNumber - m.today.dayNumber).lowercased())")
                        .ps(14).foregroundStyle(C.ink2).fixedSize(horizontal: false, vertical: true)
                }
                .accessibilityElement(children: .combine)
            }
        }
    }

    private func info(_ p: JV, deceased: Bool) -> some View {
        let s = m.summaries[id]
        let b = p["birth"]
        let d = p["death"]
        return PCard {
            PHeading("Thông tin")
            if deceased {
                let entered = d["lunar_source"].str == "entered"
                InfoRow(label: "Ngày mất", value: PFmt.dateValue(d["input"]))
                InfoRow(label: "Âm lịch", value: (s?.death_lunar_text ?? PFmt.lunarText(d["lunar"]))
                        + (entered ? " (nhập theo âm lịch)" : " (đổi từ ngày dương)"))
                if entered, let sd = DateCheck.solar(d["solar"]) {
                    InfoRow(label: "Dương lịch", value: "\(Fmt.weekday[sd.weekdayCore]), \(sd.dmy)")
                }
                InfoRow(label: "Tuổi", value: s?.tuoi_mat_text ?? "")
            }
            if b.isObject {
                InfoRow(label: "Ngày sinh", value: PFmt.dateValue(b["input"]))
                if let ly = b["lunar_year"].int {
                    InfoRow(label: "Năm âm sinh", value: "\(ly) — \(b["can_chi"].str ?? Lich.canChi(year: ly))"
                            + (b["lunar_year_approx"].isTrue ? " (khoảng)" : ""))
                }
                InfoRow(label: "Nơi sinh", value: b["noi_sinh"].string)
            }
            if !deceased, let t = s?.tuoi_duong {
                InfoRow(label: "Tuổi", value: "\(t)" + (s?.tuoi_am.map { " (tuổi âm \($0))" } ?? ""))
            }
            if deceased { InfoRow(label: "Nơi an táng", value: p["noi_an_tang"].string) }
            if deceased {
                // YC-2026-10 (08 §16.2): chỉ báo có hay chưa — ảnh hiện trên màn hình tưởng niệm.
                InfoRow(label: "Ảnh mâm lễ", value: p["memorial"]["tray_photo_id"].str == nil ? "Chưa có" : "Đã có")
                    .accessibilityIdentifier("person.tray")
            }
            InfoRow(label: "Giới tính", value: p["gioi_tinh"].str == "nam" ? "Nam" : p["gioi_tinh"].str == "nu" ? "Nữ" : "Không ghi")
            InfoRow(label: "Ghi chú", value: p["ghi_chu"].string)
            if !deceased && !b.isObject && p["ghi_chu"].string.isEmpty { PBody("Chưa có ngày sinh.") }
        }
    }

    /// Thông tin báo tử — chỉ trên trang hồ sơ (R-PRIV-2), thu gọn.
    @ViewBuilder
    private func notice(_ p: JV) -> some View {
        let n = p["death"]["notice"]
        if n.isObject {
            PCard {
                DisclosureGroup(isExpanded: $noticeOpen) {
                    VStack(alignment: .leading, spacing: 8) {
                        InfoRow(label: "Người báo tin", value: n["reported_by"].string)
                        InfoRow(label: "Mất tại", value: n["place"].string)
                        if n["notice_date"].isObject { InfoRow(label: "Ngày nhận tin", value: PFmt.dateValue(n["notice_date"])) }
                        InfoRow(label: "Lý do", value: n["reason"].string)
                        InfoRow(label: "Ghi chú", value: n["note"].string)
                        let atts = NoticeDraft(n).attachments
                        if !atts.isEmpty {
                            Text("Tệp đính kèm").ps(15).foregroundStyle(C.ink2)
                            ForEach(Array(atts.enumerated()), id: \.element.id) { i, a in
                                HStack(alignment: .top, spacing: 12) {
                                    AttachmentThumb(att: a, size: CGSize(width: 96, height: 120))
                                    VStack(alignment: .leading, spacing: 4) {
                                        Text(AttKind.title(a.kind)).ps(15, .semibold).foregroundStyle(C.ink)
                                        if let f = a.fileId, let name = m.attachmentsMeta[f]?["file_name"].str {
                                            Text(name).ps(14).foregroundStyle(C.ink2).fixedSize(horizontal: false, vertical: true)
                                        }
                                        if let f = a.fileId, m.attachmentsMeta[f]?["mime"].str == "application/pdf" {
                                            PFine("Tệp PDF được lưu nguyên bản")
                                        }
                                        Button("Xem") { open(a) }.buttonStyle(SecondaryButtonStyle())
                                            .accessibilityIdentifier("person.att.view.\(i)")
                                    }
                                }
                            }
                        }
                    }
                    .padding(.top, 8)
                } label: {
                    Text("Thông tin báo tử").serif(20, .semibold).foregroundStyle(C.ink)
                }
                .accessibilityElement(children: .contain)
                .accessibilityIdentifier("person.notice")
            }
        }
    }

    private func eventsCard(_ p: JV, deceased: Bool) -> some View {
        let evs = m.events(of: id)
        return PCard {
            HStack {
                PHeading("Sự kiện")
                Spacer()
                if !m.readOnly {
                    NavigationLink(value: PRoute.event(id, nil)) { Text("+ Thêm sự kiện") }
                        .buttonStyle(SecondaryButtonStyle()).accessibilityIdentifier("person.addEvent")
                }
            }
            if evs.isEmpty { PBody("Chưa có sự kiện nào.") }
            ForEach(evs, id: \.self) { e in
                VStack(alignment: .leading, spacing: 6) {
                    Text("\(m.eventTitle(e)) · \(PFmt.dateValue(e["date"])) · \(e["recurring"].isTrue ? "hằng năm" : "một lần")")
                        .ps(16).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
                    if !m.readOnly {
                        HStack(spacing: 10) {
                            NavigationLink(value: PRoute.event(id, e["id"].string)) { Text("Sửa") }
                                .buttonStyle(SecondaryButtonStyle())
                                .accessibilityIdentifier("person.event.edit")
                            Button("Xóa") { deleteEvent = e }.buttonStyle(SecondaryButtonStyle(destructive: true))
                                .accessibilityIdentifier("person.event.delete")
                        }
                    }
                }
            }
            if deceased {
                PFine("Không nhắc sinh nhật và sự kiện của người đã mất — chỉ nhắc ngày giỗ. Thông tin vẫn được lưu và sao lưu.")
            }
        }
    }

    private func remindCard(_ p: JV, deceased: Bool) -> some View {
        let r = p["reminders"]
        return PCard {
            PHeading("Nhắc")
            if deceased {
                InfoRow(label: "Ngày giỗ", value: PFmt.ruleSummary(r["gio"], tienThuong: true))
            } else {
                InfoRow(label: "Sinh nhật", value: PFmt.ruleSummary(r["sinh_nhat"], tienThuong: false))
                if p["sinh_nhat_am"].isTrue {
                    InfoRow(label: "Sinh nhật âm lịch", value: PFmt.ruleSummary(r["sinh_nhat_am"], tienThuong: false))
                }
            }
            if m.settings["reminders_enabled"].bool == false {
                PFine("Lời nhắc đang tắt chung trong Cài đặt nhắc.")
            }
        }
    }

    /// Ảnh xem ngay trong ứng dụng; PDF hỏi lại rồi mới mở Quick Look (BM3-03, S10).
    private func open(_ a: AttDraft) {
        guard let f = a.fileId, let data = m.attachmentData(f) else { return }
        viewing = AttOpen(data: data, pdf: m.attachmentsMeta[f]?["mime"].str == "application/pdf")
    }

    @ViewBuilder
    private func deleteCard(_ name: String) -> some View {
        if !m.readOnly {
            HStack {
                Button("Xóa hồ sơ") { confirmDelete = true }
                    .buttonStyle(SecondaryButtonStyle(destructive: true))
                    .accessibilityIdentifier("person.delete")
                Spacer()
            }
        }
    }
}
