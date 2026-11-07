// YC-2026-10 — ảnh mâm lễ (08 §16; 01 §3.7 `memorial.tray_photo_id`; 04 R15; 07 §10 ADR-V02-01).
// Thứ tự trên màn hình tưởng niệm: ảnh mâm lễ của gia đình → ảnh Clevix đã duyệt cùng `tray_style` → không có mâm.
import CryptoKit
import ImageIO
import PhotosUI
import SwiftUI
import UniformTypeIdentifiers
#if os(iOS)
import UIKit
#endif

/// Ảnh mâm lễ: cạnh dài ≤ 1600 px, ≤ 1 MB, mã hóa lại và bỏ siêu dữ liệu như ảnh chân dung (01 §3.8, S9).
enum TrayPhoto {
    static let maxSide = 1600
    static let maxBytes = 1_048_576

    /// Câu báo lỗi của §16.2 (như ảnh chân dung).
    static func message(_ e: PhotoError) -> String {
        switch e {
        case .tooLarge, .tooManyPixels: return "Ảnh quá lớn (tối đa 25 MB)."
        case .unreadable, .stillTooBig: return "Không đọc được ảnh này. Hãy chọn một ảnh khác."
        }
    }

    /// Giải mã, thu nhỏ, mã hóa lại ở luồng nền.
    static func process(_ data: Data?) async -> Result<ProcessedPhoto, PhotoError> {
        await Task.detached(priority: .userInitiated) {
            guard let data else { return .failure(PhotoError.unreadable) }
            do { return .success(try PhotoProcessor.process(data, maxSide: maxSide, maxBytes: maxBytes)) }
            catch let e as PhotoError { return .failure(e) }
            catch { return .failure(PhotoError.unreadable) }
        }.value
    }
}

// MARK: - Chọn ảnh

/// Nút chọn ảnh mâm lễ. iPhone/iPad: hộp chọn có "Chụp ảnh" (khi có máy ảnh) và "Chọn từ thư viện" (PhotosPicker, không
/// cần quyền thư viện). Mac: hộp chọn tệp ảnh. Ảnh được xử lý ở luồng nền; `busy` báo "Đang xử lý ảnh…" cho biểu mẫu.
struct TrayPickButton: View {
    let title: String
    var a11y: String? = nil
    @Binding var busy: Bool
    var onPicked: (Result<ProcessedPhoto, PhotoError>) -> Void
    @State private var item: PhotosPickerItem?
    @State private var showLibrary = false
    @State private var choose = false
    @State private var showCamera = false
    @State private var showFiles = false

    var body: some View {
        Button(title) { start() }
            .buttonStyle(SecondaryButtonStyle())
            .disabled(busy)
            .accessibilityLabel(a11y ?? title)
            .accessibilityIdentifier("form.tray.pick")
            .photosPicker(isPresented: $showLibrary, selection: $item, matching: .images)
            .onChange(of: item) { it in
                guard let it else { return }
                busy = true
                Task {
                    let data = try? await it.loadTransferable(type: Data.self)
                    finish(await TrayPhoto.process(data))
                    item = nil
                }
            }
            #if os(iOS)
            .confirmationDialog("Ảnh mâm lễ ngày giỗ", isPresented: $choose, titleVisibility: .visible) {
                Button("Chụp ảnh") { showCamera = true }
                Button("Chọn từ thư viện") { showLibrary = true }
                Button("Hủy", role: .cancel) {}
            }
            .fullScreenCover(isPresented: $showCamera) {
                CameraPicker { image in
                    showCamera = false
                    guard let image else { return }
                    busy = true
                    Task {
                        let data = await Task.detached(priority: .userInitiated) { image.jpegData(compressionQuality: 0.9) }.value
                        finish(await TrayPhoto.process(data))
                    }
                }
                .ignoresSafeArea()
            }
            #endif
            .fileImporter(isPresented: $showFiles, allowedContentTypes: [.image]) { r in
                guard case .success(let url) = r else { return }
                busy = true
                Task {
                    let r: Result<ProcessedPhoto, PhotoError> = await Task.detached(priority: .userInitiated) {
                        let scoped = url.startAccessingSecurityScopedResource()
                        defer { if scoped { url.stopAccessingSecurityScopedResource() } }
                        // Kiểm cỡ trước khi đọc (≤ 25 MB, 01 §3.8).
                        let size = (try? url.resourceValues(forKeys: [.fileSizeKey]).fileSize) ?? 0
                        guard size <= PhotoProcessor.maxInputBytes else { return .failure(PhotoError.tooLarge) }
                        guard size > 0, let data = try? Data(contentsOf: url) else { return .failure(PhotoError.unreadable) }
                        do { return .success(try PhotoProcessor.process(data, maxSide: TrayPhoto.maxSide, maxBytes: TrayPhoto.maxBytes)) }
                        catch let e as PhotoError { return .failure(e) }
                        catch { return .failure(PhotoError.unreadable) }
                    }.value
                    finish(r)
                }
            }
    }

    private func start() {
        #if os(iOS)
        if UIImagePickerController.isSourceTypeAvailable(.camera) { choose = true } else { showLibrary = true }
        #else
        showFiles = true
        #endif
    }

    private func finish(_ r: Result<ProcessedPhoto, PhotoError>) {
        onPicked(r)
        busy = false
    }
}

#if os(iOS)
/// Máy ảnh của hệ thống (UIImagePickerController). Ảnh chụp được mã hóa JPEG ở luồng nền rồi đi qua cùng đường xử lý S9.
struct CameraPicker: UIViewControllerRepresentable {
    var onDone: (UIImage?) -> Void

    func makeUIViewController(context: Context) -> UIImagePickerController {
        let c = UIImagePickerController()
        c.sourceType = .camera
        c.cameraCaptureMode = .photo
        c.delegate = context.coordinator
        return c
    }

    func updateUIViewController(_ c: UIImagePickerController, context: Context) {}
    func makeCoordinator() -> Coordinator { Coordinator(onDone) }

    final class Coordinator: NSObject, UIImagePickerControllerDelegate, UINavigationControllerDelegate {
        let onDone: (UIImage?) -> Void
        init(_ f: @escaping (UIImage?) -> Void) { onDone = f }
        func imagePickerController(_ picker: UIImagePickerController,
                                   didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]) {
            onDone(info[.originalImage] as? UIImage)
        }
        func imagePickerControllerDidCancel(_ picker: UIImagePickerController) { onDone(nil) }
    }
}
#endif

// MARK: - Gói ảnh Clevix (08 §16.3)

/// Một ảnh Clevix đã duyệt trong gói đi kèm ứng dụng.
struct TrayPackItem {
    let id: String
    let style: String
    let url: URL
    let author: String
    let license: String
    let credit: String?
}

/// Gói ảnh mâm lễ nhúng lúc dựng (`gen-trays.sh` → `TrayPack/` trong bundle). Chỉ nhận mục `trang_thai = duyet`, có tệp,
/// đúng `sha256`, JPEG/WebP. Không tải gì qua mạng. Gói hiện rỗng → không có mâm.
final class TrayPack {
    static let shared = TrayPack()
    private(set) var items: [TrayPackItem] = []

    init(dir: URL? = TrayPack.defaultDir()) {
        guard let dir else { return }
        items = Self.load(dir)
    }

    static func defaultDir() -> URL? {
        #if DEBUG
        if UserDefaults.standard.string(forKey: "lvhvTrayPack") == "test" { return TrayPackTest.make() }
        #endif
        return Bundle.main.url(forResource: "TrayPack", withExtension: nil)
    }

    static func load(_ dir: URL) -> [TrayPackItem] {
        guard let d = try? Data(contentsOf: dir.appendingPathComponent("manifest.json")),
              let j = JV.parse(String(decoding: d, as: UTF8.self)) else { return [] }
        var out: [TrayPackItem] = []
        for it in j["items"].array {
            guard it["trang_thai"].str == "duyet",
                  let id = it["id"].str, let style = it["tray_style"].str, let file = it["file"].str,
                  !file.contains("/"), ["image/jpeg", "image/webp"].contains(it["mime"].str ?? ""),
                  let sha = it["sha256"].str else { continue }
            let url = dir.appendingPathComponent(file)
            guard let bytes = try? Data(contentsOf: url),
                  SHA256.hash(data: bytes).map({ String(format: "%02x", $0) }).joined() == sha.lowercased() else { continue }
            out.append(TrayPackItem(id: id, style: style, url: url, author: it["tac_gia"].string,
                                    license: it["giay_phep"].string, credit: it["ghi_cong"].str))
        }
        return out
    }

    func item(style: String) -> TrayPackItem? { items.first { $0.style == style } }
}

/// Ảnh mâm sẽ hiện trên màn hình tưởng niệm của một người.
struct TrayChoice {
    let image: CGImage
    let family: Bool
    let creditLine: String?
}

@MainActor
enum TrayResolve {
    /// (1) ảnh gia đình → (2) ảnh Clevix cùng `tray_style` → (3) không có.
    static func choose(person p: JV, model m: PeopleModel, pack: TrayPack = .shared) -> TrayChoice? {
        if let id = p["memorial"]["tray_photo_id"].str,
           let img = PhotoCache.shared.image(id, load: { m.photoData(id) }) {
            return TrayChoice(image: img, family: true, creditLine: nil)
        }
        let style = p["memorial"]["tray_style"].str ?? "truyen_thong"
        guard let it = pack.item(style: style),
              let img = PhotoCache.shared.image("traypack:" + it.id, load: { try? Data(contentsOf: it.url) }) else { return nil }
        let credit = it.credit.map { _ in "Ảnh: \(it.author) · \(it.license)" }
        return TrayChoice(image: img, family: false, creditLine: credit)
    }
}

/// Ảnh mâm dưới hàng bàn thờ: rộng tối đa 480, cắt vừa khung 4:3 ở giữa, bo góc 12, viền mảnh vàng đồng.
struct TrayImageView: View {
    let choice: TrayChoice
    var border: Color

    var body: some View {
        VStack(spacing: 6) {
            Color.clear
                .aspectRatio(4.0 / 3.0, contentMode: .fit)
                .overlay(Image(decorative: choice.image, scale: 1).resizable().scaledToFill())
                .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                .overlay(RoundedRectangle(cornerRadius: 12, style: .continuous).stroke(border, lineWidth: 1))
                .frame(maxWidth: 480)
                .accessibilityElement()
                .accessibilityAddTraits(.isImage)
                .accessibilityLabel(choice.family ? "Ảnh mâm lễ của gia đình" : "Ảnh mâm lễ")
                .accessibilityIdentifier("memorial.tray")
            if let c = choice.creditLine {
                Text(c).ps(12).foregroundStyle(border).multilineTextAlignment(.center)
                    .fixedSize(horizontal: false, vertical: true)
                    .accessibilityIdentifier("memorial.trayCredit")
            }
        }
    }
}

#if DEBUG
/// Gói thử (08 §16.4 ca 5), chỉ bản Debug: `-lvhvTrayPack test` → tạo trong thư mục tạm một gói có một ảnh đã duyệt,
/// một ảnh đang chờ và một ảnh sai `sha256` (hai ảnh sau phải bị bỏ qua).
enum TrayPackTest {
    static func make() -> URL? {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent("traypack-test", isDirectory: true)
        try? FileManager.default.removeItem(at: dir)
        guard (try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)) != nil,
              let jpg = image() else { return nil }
        let sha = SHA256.hash(data: jpg).map { String(format: "%02x", $0) }.joined()
        try? jpg.write(to: dir.appendingPathComponent("thu-01.jpg"))
        try? jpg.write(to: dir.appendingPathComponent("thu-02.jpg"))
        try? jpg.write(to: dir.appendingPathComponent("thu-03.jpg"))
        func item(_ id: String, _ file: String, _ sha: String, _ state: String) -> String {
            """
            {"id":"\(id)","tray_style":"truyen_thong","file":"\(file)","mime":"image/jpeg","width":640,"height":480,\
            "sha256":"\(sha)","nguon":"ảnh thử vẽ trong mã","giay_phep":"CC BY 4.0","tac_gia":"Clevix Lab",\
            "ghi_cong":"Clevix Lab, ảnh thử, CC BY 4.0","duyet_boi":"ca kiểm","ngay_duyet":"2026-11-07","trang_thai":"\(state)"}
            """
        }
        let manifest = """
        {"pack_version":1,"items":[\(item("cho", "thu-02.jpg", sha, "cho")),\
        \(item("sai", "thu-03.jpg", String(repeating: "0", count: 64), "duyet")),\
        \(item("thu-01", "thu-01.jpg", sha, "duyet"))]}
        """
        try? Data(manifest.utf8).write(to: dir.appendingPathComponent("manifest.json"))
        return dir
    }

    /// Ảnh thử một mâm vẽ đơn giản (không phải ảnh thật — chỉ để kiểm thứ tự chọn ảnh).
    private static func image() -> Data? {
        let w = 640, h = 480
        guard let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: 0,
                                  space: CGColorSpace(name: CGColorSpace.sRGB)!,
                                  bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue) else { return nil }
        ctx.setFillColor(CGColor(red: 0.36, green: 0.22, blue: 0.14, alpha: 1))
        ctx.fill(CGRect(x: 0, y: 0, width: w, height: h))
        ctx.setFillColor(CGColor(red: 0.78, green: 0.62, blue: 0.3, alpha: 1))
        ctx.fillEllipse(in: CGRect(x: 120, y: 120, width: 400, height: 160))
        ctx.setFillColor(CGColor(red: 0.9, green: 0.45, blue: 0.15, alpha: 1))
        ctx.fillEllipse(in: CGRect(x: 250, y: 200, width: 140, height: 110))
        guard let img = ctx.makeImage() else { return nil }
        return PhotoProcessor.encode(img, quality: 0.8)
    }
}
#endif
