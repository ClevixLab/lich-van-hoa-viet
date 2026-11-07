// Giới thiệu và Nguồn quy tắc — cùng nội dung với pwa/src/ui/about.ts.
import SwiftUI

/// Câu mô tả phương pháp âm lịch (giống METHOD_TEXT của pwa/src/ui/about.ts).
private let methodText = [
    "jpl": "Ngày âm lịch tính từ lịch thiên văn JPL DE441 của NASA JPL (thời điểm trăng mới và tiết khí), với ΔT theo Morrison và Stephenson, rồi quy ra ngày theo múi giờ của từng giai đoạn nêu trên. Với những năm rất xa, kết quả kém chắc chắn hơn vì không ai dự báo chính xác được tốc độ quay của Trái Đất (ΔT). Sau năm 2077, vài ngày có trăng mới hoặc tiết khí rơi sát nửa đêm có thể lệch một ngày so với các lịch khác.",
    "precise": "Ngày âm lịch tính theo mô hình thiên văn chính xác (Meeus): thời điểm trăng mới và vị trí Mặt Trời được tính đầy đủ, rồi quy ra ngày theo múi giờ của từng giai đoạn nêu trên. Thuật toán Hồ Ngọc Đức, phổ biến trong nhiều lịch Việt, vẫn được giữ trong ứng dụng để đối chiếu; hai cách cho cùng kết quả ở gần như mọi ngày.",
    "hnd": "Ngày âm lịch tính theo thuật toán Hồ Ngọc Đức (phổ biến trong các lịch Việt), quy ra ngày theo múi giờ của từng giai đoạn nêu trên.",
]

private let tzNote = "Trước năm 1929, âm lịch tính theo giờ địa phương Bắc Kinh; 1929–1967 theo múi giờ UTC+8; từ 1968 theo UTC+7 (chuẩn miền Bắc). Miền Nam dùng UTC+8 đến năm 1975, nên một số tháng 1968–1975 có thể khác lịch miền Nam thời ấy."

/// "Ứng dụng giúp bạn" (bản web 0.2.1, PR #73): tiêu đề in đậm + phần còn lại.
private let helps: [(String, String)] = [
    ("Tờ lịch mỗi ngày", " từ năm 1900 đến năm 3000: ngày âm, Can Chi, tiết khí, giờ hoàng đạo và một câu ca dao, tục ngữ."),
    ("Chọn ngày tốt", " cho việc lớn: sao tốt, sao xấu, việc nên làm và nên tránh, theo tuổi của bạn."),
    ("Đổi ngày", " âm sang dương và ngược lại, kể cả tháng nhuận."),
    ("Người thân", " — ghi lại người đã khuất và người đang sống trong gia đình. Ngày giỗ được tự tính theo âm lịch mỗi năm (giỗ đầu, giỗ hết, giỗ thường), cùng sinh nhật dương lịch, sinh nhật âm lịch và những dịp riêng như tốt nghiệp, ngày cưới."),
    ("Nhắc trước", " 1, 3, 5, 10, 15 ngày hoặc số ngày bạn tự chọn, để kịp sắm lễ, báo anh em họ hàng."),
    ("Xem cả nhà có việc gì", " trong tuần, tháng, quý hay cả năm; lọc riêng người đã mất hoặc người đang sống."),
    ("Trang tưởng niệm ngày giỗ", ": bia ghi tên, năm sinh năm mất theo Can Chi, ảnh chân dung, bát hương ba nén và đôi nến. Gia đình có thể thêm ảnh mâm lễ của chính nhà mình."),
    ("Thông tin báo tử", " khi một người thân qua đời: ai báo tin, mất ở đâu, ghi chú, kèm ảnh tin báo tử hoặc bản chụp giấy chứng tử."),
]

/// "Bắt đầu trong một phút" — chữ đậm viết bằng **…**.
private let steps = [
    "Mở mục **Người thân**, chọn **Hồ sơ**, rồi bấm “+ Thêm người đã mất” hoặc “+ Thêm người đang sống”.",
    "Nhập họ tên, cách gọi (Ông, Bà, Bố, Mẹ…) và quan hệ (ví dụ “Ông nội”). Với người đã mất, nhập ngày mất theo dương lịch hoặc âm lịch — ứng dụng tự đổi và tính ngày giỗ hằng năm.",
    "Chọn nhắc trước mấy ngày, bấm “Lưu”.",
    "Mở **Sắp tới** để xem các ngày giỗ, sinh nhật gần nhất; mở **Theo kỳ** để xem theo tuần, tháng, quý, năm.",
    "Đúng ngày giỗ, bấm “Mở trang tưởng niệm” để thắp nén hương tưởng nhớ.",
    "Vào **Sao lưu** để cất một bản sao có mật khẩu, hoặc xuất tệp lịch (.ics) sang ứng dụng Lịch.",
]

/// "Mẹo để không lỡ ngày nào" — bản iPhone, iPad, Mac: thông báo thật của máy (02 §7.4), không cần .ics hay
/// "Thêm vào Màn hình chính" như bản web; không hứa hẹn bản chưa phát hành.
private let tips = [
    "Hãy cho phép ứng dụng gửi thông báo (Người thân › nút chuông › Cài đặt nhắc). Máy tự báo đúng giờ đã đặt, cả khi ứng dụng đang đóng hay máy vừa khởi động lại.",
    "Hệ điều hành chỉ giữ tối đa 64 thông báo đang chờ cho mỗi ứng dụng. Khi có nhiều lời nhắc hơn, ứng dụng đặt trước những lời nhắc gần nhất, kèm một thông báo nhắc bạn mở ứng dụng để đặt tiếp — vì vậy thỉnh thoảng hãy mở ứng dụng.",
    "Theo mặc định, thông báo trên màn hình khóa không ghi tên người thân; bạn có thể bật hiện tên trong Cài đặt nhắc.",
    "Thỉnh thoảng tạo tệp sao lưu và cất ở nơi an toàn (Tệp, iCloud Drive, máy tính). Tệp này mở được trên máy khác có ứng dụng và trên bản web.",
]

struct AboutView: View {
    @State private var showRules = false

    private var version: String { AppUpdate.version }

    var body: some View {
        if showRules {
            RulesView { showRules = false }
        } else {
            ScrollView {
                VStack(alignment: .leading, spacing: 12) {
                    HStack(spacing: 14) {
                        AppIconView().frame(width: 72, height: 72)
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Lịch và Văn hóa Việt").serif(26, .semibold).foregroundStyle(C.ink).accessibilityAddTraits(.isHeader)
                            Text("Clevix Lab").sans(16, .semibold).foregroundStyle(C.redInk)
                            Text("Phiên bản \(version) (thử nghiệm)").sans(12).foregroundStyle(C.ink3)
                        }
                    }
                    Panel {
                        H2("Phiên bản và cập nhật")
                        P(AppUpdate.runningVersion)
                        P(AppUpdate.explain)
                        Button(action: AppUpdate.openStore) {
                            Text(AppUpdate.button).sans(15, .semibold).foregroundStyle(.white)
                                .padding(.horizontal, 16).padding(.vertical, 10)
                                .background(C.redDeep, in: Capsule())
                        }
                        .buttonStyle(.plain)
                    }
                    Panel {
                        H2("Về ứng dụng")
                        Text("Lịch và Văn hóa Việt là cuốn lịch vạn niên miễn phí và cũng là sổ tay gia đình: xem ngày âm dương, chọn ngày tốt, và không bao giờ quên ngày giỗ ông bà, cha mẹ hay sinh nhật người thân. Không quảng cáo, không cần tài khoản, dữ liệu chỉ nằm trên máy của bạn.")
                            .sans(17).foregroundStyle(C.ink).fixedSize(horizontal: false, vertical: true)
                            .accessibilityIdentifier("about.lead")
                        H3("Ứng dụng giúp bạn")
                        ForEach(Array(helps.enumerated()), id: \.offset) { _, it in
                            Bullet { Text(it.0).bold().foregroundColor(C.ink) + Text(it.1) }
                        }
                        H3("Bắt đầu trong một phút")
                        ForEach(Array(steps.enumerated()), id: \.offset) { i, st in
                            HStack(alignment: .firstTextBaseline, spacing: 10) {
                                Text("\(i + 1).").sans(16, .semibold).foregroundStyle(C.redInk).frame(minWidth: 22, alignment: .trailing)
                                Text(LocalizedStringKey(st)).sans(16).foregroundStyle(C.ink2).fixedSize(horizontal: false, vertical: true)
                            }
                            .accessibilityElement(children: .combine)
                        }
                        H3("Mẹo để không lỡ ngày nào")
                        ForEach(Array(tips.enumerated()), id: \.offset) { _, t in Bullet { Text(t) } }
                    }
                    Panel {
                        H2("Cách tính lịch")
                        P("Từ năm 1968, âm lịch tính theo múi giờ UTC+7 (giờ Hà Nội).")
                        P(tzNote)
                        P(methodText[Lich.method] ?? "Ngày âm lịch tính theo phương pháp \(Lich.method).")
                        P("Phần xem ngày (sao tốt, sao xấu, việc nên làm và nên tránh, trực, hoàng đạo, hắc đạo) được biên soạn theo quan niệm truyền thống trong sách Ngọc Hạp Thông Thư. Đây là nét văn hóa dân gian, chỉ có giá trị tham khảo; những quyết định quan trọng xin cân nhắc thêm hoàn cảnh thực tế. Điểm số là quy ước của ứng dụng, không có trong sách.")
                        Button("Nguồn quy tắc xem ngày") { showRules = true }.buttonStyle(.plain).foregroundStyle(C.redInk)
                            .frame(minHeight: 44)
                    }
                    Panel {
                        H2("Quyền riêng tư")
                        P("Ứng dụng không thu thập dữ liệu, không có tài khoản, không quảng cáo và không gửi thông tin đi đâu. Mọi tính toán diễn ra ngay trên máy của bạn. Ứng dụng không dùng mạng và hoạt động hoàn toàn khi không có mạng.")
                        P("Hồ sơ người thân, ảnh, thông tin báo tử và tệp đính kèm chỉ được lưu trong vùng dữ liệu riêng của ứng dụng trên máy này, không đưa vào bản sao lưu iCloud của máy. Trên iPhone và iPad, ảnh và tệp đính kèm chỉ đọc được khi máy đã mở khóa. Ảnh được tự bỏ thông tin vị trí và máy chụp. Tệp sao lưu được mã hóa bằng mật khẩu do bạn đặt; ứng dụng không giữ mật khẩu đó.")
                        P("Dữ liệu vẫn còn khi bạn đóng ứng dụng, tắt máy hay cập nhật ứng dụng. Dữ liệu sẽ mất nếu bạn xóa ứng dụng khỏi máy. Vì vậy hãy sao lưu định kỳ.")
                        P("Máy chỉ ghi nhớ thêm cỡ chữ, giao diện sáng tối mà bạn chọn và ngày bạn bấm \"Để sau\" ở thẻ nhắc cập nhật. Năm sinh nhập ở mục Chọn ngày tốt chỉ dùng trong lúc tính, không được lưu lại.")
                    }
                    Panel {
                        H2("Nguồn ca dao, tục ngữ")
                        P(Lich.attribution)
                        P("Bộ dữ liệu gồm \(Lich.cadaoTotal) câu, phát hành theo giấy phép CC BY-SA 4.0 (Creative Commons Ghi công – Chia sẻ tương tự). Wikiquote và Wikimedia không liên quan và không xác nhận ứng dụng này.")
                    }
                    Panel {
                        H2("Phông chữ")
                        P("Phông chữ tiêu đề là bản rút gọn cho chữ Việt của phông Lora (The Lora Project Authors), cấp phép SIL Open Font License 1.1 (tệp OFL.txt trong gói ứng dụng); bản rút gọn mang tên \"LVHV Serif\" theo điều kiện giấy phép.")
                    }
                    // Dòng bản quyền hai dòng (giống bản web): "© 2026 Clevix Lab" chữ serif, © nhỏ 0,82× và
                    // nhẹ hơn; dưới là "Phiên bản X.Y.Z" nhỏ, nhạt.
                    VStack(spacing: 2) {
                        HStack(alignment: .firstTextBaseline, spacing: 0) {
                            Text(AppUpdate.copyrightSign).serif(14 * 0.82).foregroundStyle(C.ink2)
                            Text(AppUpdate.copyrightHolder).serif(14, .semibold).foregroundStyle(C.ink)
                        }
                        .accessibilityElement(children: .ignore)
                        .accessibilityLabel(AppUpdate.copyright)
                        Text(AppUpdate.versionLine).kerning(0.4).sans(11).foregroundStyle(C.ink3)
                    }
                    .multilineTextAlignment(.center)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding(.vertical, 8)
                }
                .padding(16)
                .frame(maxWidth: 720)
                .frame(maxWidth: .infinity)
            }
        }
    }
}

struct RulesView: View {
    var back: () -> Void
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                Button("‹ Về trang Giới thiệu", action: back).buttonStyle(.plain).foregroundStyle(C.redInk)
                Text("Nguồn quy tắc").serif(30, .semibold).foregroundStyle(C.ink).accessibilityAddTraits(.isHeader)
                P("Theo Ngọc Hạp Thông Thư; tham khảo. Địa chỉ trang được ghi để đối chiếu; ứng dụng không mở các trang này.")
                Panel { Blocks(blocks: Lich.ruleSources) }
                Panel {
                    H2("Nguồn ca dao, tục ngữ (chi tiết)")
                    Blocks(blocks: Lich.cadaoSources)
                }
            }
            .padding(16)
            .frame(maxWidth: 720)
            .frame(maxWidth: .infinity)
        }
    }
}

private struct H3: View {
    let text: String
    init(_ t: String) { text = t }
    var body: some View {
        Text(text).sans(17, .semibold).foregroundStyle(C.ink).padding(.top, 6).accessibilityAddTraits(.isHeader)
    }
}

/// Một dòng có dấu chấm đầu dòng; chữ to theo cỡ chữ và xuống dòng.
private struct Bullet: View {
    let text: Text
    init(@ViewBuilder _ t: () -> Text) { text = t() }
    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            Text("•").sans(16).foregroundStyle(C.redInk).accessibilityHidden(true)
            text.sans(16).foregroundStyle(C.ink2).fixedSize(horizontal: false, vertical: true)
        }
    }
}

private struct Blocks: View {
    let blocks: [TextBlock]
    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            ForEach(Array(blocks.enumerated()), id: \.offset) { _, b in
                switch b.kind {
                case "h":
                    Text(b.text).sans(16, .semibold).foregroundStyle(C.ink).padding(.top, 8).accessibilityAddTraits(.isHeader)
                case "li":
                    Text("• \(b.text)").sans(14).foregroundStyle(C.ink2).fixedSize(horizontal: false, vertical: true).textSelection(.enabled)
                default:
                    Text(b.text).sans(14).foregroundStyle(C.ink2).fixedSize(horizontal: false, vertical: true)
                }
            }
        }
    }
}

/// Biểu tượng tờ lịch vẽ bằng SwiftUI (giống pwa/public/icons/icon.svg), để không phụ thuộc tên ảnh AppIcon.
struct AppIconView: View {
    var body: some View {
        GeometryReader { g in
            let s = g.size.width / 512
            ZStack(alignment: .topLeading) {
                RoundedRectangle(cornerRadius: 112 * s).fill(Color(red: 0x8F / 255, green: 0x14 / 255, blue: 0x16 / 255))
                Group {
                    RoundedRectangle(cornerRadius: 34 * s * 0.86).fill(Color(white: 0.98))
                        .frame(width: 384 * s * 0.86, height: 400 * s * 0.86)
                        .offset(x: (35.84 + 64 * 0.86) * s, y: (35.84 + 72 * 0.86) * s)
                    Circle().fill(Color(red: 0xC8 / 255, green: 0x16 / 255, blue: 0x1D / 255))
                        .frame(width: 140 * s * 0.86, height: 140 * s * 0.86)
                        .offset(x: (35.84 + 128 * 0.86) * s, y: (35.84 + 242 * 0.86) * s)
                    Circle().fill(Color(red: 0xB0 / 255, green: 0x7A / 255, blue: 0x1E / 255))
                        .frame(width: 124 * s * 0.86, height: 124 * s * 0.86)
                        .offset(x: (35.84 + 272 * 0.86) * s, y: (35.84 + 250 * 0.86) * s)
                }
            }
        }
        .aspectRatio(1, contentMode: .fit)
        .accessibilityHidden(true)
    }
}
