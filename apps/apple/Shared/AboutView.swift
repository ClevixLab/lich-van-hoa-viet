// Giới thiệu và Nguồn quy tắc — cùng nội dung với pwa/src/ui/about.ts.
import SwiftUI

private let methodLabel = [
    "hnd": "thuật toán Hồ Ngọc Đức (phổ biến trong các lịch Việt)",
    "precise": "mô hình thiên văn chính xác hơn (Meeus đầy đủ)",
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
                        P("Lịch âm dương Việt Nam theo múi giờ UTC+7, từ năm 2000 đến năm 3000: ngày âm, Can Chi, tiết khí, giờ hoàng đạo, chọn ngày tốt, và một câu ca dao, tục ngữ cho mỗi ngày.")
                        P("Ngày âm lịch tính theo \(methodLabel[Lich.method] ?? Lich.method).")
                        P("Phần xem ngày (sao tốt, sao xấu, việc nên làm và nên tránh, trực, hoàng đạo, hắc đạo) được biên soạn theo quan niệm truyền thống trong sách Ngọc Hạp Thông Thư. Đây là nét văn hóa dân gian, chỉ có giá trị tham khảo; những quyết định quan trọng xin cân nhắc thêm hoàn cảnh thực tế. Điểm số là quy ước của ứng dụng, không có trong sách.")
                        Button("Nguồn quy tắc xem ngày") { showRules = true }.buttonStyle(.plain).foregroundStyle(C.redInk)
                    }
                    Panel {
                        H2("Quyền riêng tư")
                        P("Ứng dụng không thu thập dữ liệu, không có tài khoản, không quảng cáo và không gửi thông tin đi đâu. Mọi tính toán diễn ra ngay trên máy của bạn.")
                        P("Ứng dụng không dùng mạng và hoạt động hoàn toàn khi không có mạng. Máy chỉ ghi nhớ cỡ chữ, giao diện sáng tối mà bạn chọn và ngày bạn bấm \"Để sau\" ở thẻ nhắc cập nhật.")
                        P("Năm sinh nhập ở mục Chọn ngày tốt chỉ dùng trong lúc tính, không được lưu lại.")
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
