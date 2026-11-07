// Ảnh chân dung và ảnh giấy báo tử (01 §3.8, S9): giải mã bằng ImageIO, xoay theo EXIF, thu cạnh dài ≤ 1024 px
// (không phóng to), vẽ lại vào một CGImage mới rồi mã hóa lại JPEG; sau đó bỏ mọi đoạn APP1…APP15 và COM khỏi
// luồng JPEG — byte lưu không còn EXIF, GPS, XMP, IPTC. Không cần quyền thư viện ảnh (dùng PhotosPicker).
import CoreGraphics
import Foundation
import ImageIO
import SwiftUI
import UniformTypeIdentifiers

struct ProcessedPhoto: Equatable {
    let jpeg: Data
    let width: Int
    let height: Int
}

enum PhotoError: Error, Equatable {
    case tooLarge
    case unreadable
    case tooManyPixels(Int, Int)
    case stillTooBig

    var message: String {
        switch self {
        case .tooLarge: return "Ảnh lớn hơn 25 MB — hãy chọn ảnh nhỏ hơn"
        case .unreadable: return "Không đọc được ảnh này (định dạng không hỗ trợ)"
        case .tooManyPixels(let w, let h): return "Ảnh quá lớn (\(w) × \(h) điểm ảnh) — hãy chọn ảnh nhỏ hơn"
        case .stillTooBig: return "Ảnh quá chi tiết, chưa nén được dưới 1 MB"
        }
    }
}

enum PhotoProcessor {
    static let maxInputBytes = 25 * 1024 * 1024
    static let maxStoredBytes = 1_048_576
    static let maxSide = 1024

    static func process(_ data: Data, maxSide: Int = PhotoProcessor.maxSide) throws -> ProcessedPhoto {
        if data.count > maxInputBytes { throw PhotoError.tooLarge }
        let srcOpts = [kCGImageSourceShouldCache: false] as CFDictionary
        guard let src = CGImageSourceCreateWithData(data as CFData, srcOpts), CGImageSourceGetCount(src) > 0 else {
            throw PhotoError.unreadable
        }
        if let props = CGImageSourceCopyPropertiesAtIndex(src, 0, nil) as? [CFString: Any],
           let w = (props[kCGImagePropertyPixelWidth] as? NSNumber)?.intValue,
           let h = (props[kCGImagePropertyPixelHeight] as? NSNumber)?.intValue {
            if w > 12_000 || h > 12_000 || w * h > 50_000_000 { throw PhotoError.tooManyPixels(w, h) }
        }
        let thumbOpts: [CFString: Any] = [
            kCGImageSourceCreateThumbnailFromImageAlways: true,
            kCGImageSourceCreateThumbnailWithTransform: true, // xoay theo EXIF trước khi bỏ siêu dữ liệu
            kCGImageSourceThumbnailMaxPixelSize: maxSide,
            kCGImageSourceShouldCacheImmediately: true,
        ]
        guard let img = CGImageSourceCreateThumbnailAtIndex(src, 0, thumbOpts as CFDictionary) else {
            throw PhotoError.unreadable
        }
        let w = img.width, h = img.height
        guard w > 0, h > 0, let space = CGColorSpace(name: CGColorSpace.sRGB),
              let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: 0, space: space,
                                  bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue) else {
            throw PhotoError.unreadable
        }
        let rect = CGRect(x: 0, y: 0, width: w, height: h)
        ctx.setFillColor(CGColor(red: 1, green: 1, blue: 1, alpha: 1))
        ctx.fill(rect)
        ctx.draw(img, in: rect)
        guard let flat = ctx.makeImage() else { throw PhotoError.unreadable }
        for q in [0.85, 0.75, 0.6, 0.45, 0.3] {
            if let d = encode(flat, quality: q) {
                let clean = JPEGSanitizer.strip(d)
                if clean.count <= maxStoredBytes { return ProcessedPhoto(jpeg: clean, width: w, height: h) }
            }
        }
        throw PhotoError.stillTooBig
    }

    static func encode(_ img: CGImage, quality: Double) -> Data? {
        let out = NSMutableData()
        guard let dest = CGImageDestinationCreateWithData(out, UTType.jpeg.identifier as CFString, 1, nil) else { return nil }
        CGImageDestinationAddImage(dest, img, [kCGImageDestinationLossyCompressionQuality: quality] as CFDictionary)
        guard CGImageDestinationFinalize(dest) else { return nil }
        return out as Data
    }

    static func cgImage(_ data: Data) -> CGImage? {
        guard let src = CGImageSourceCreateWithData(data as CFData, nil) else { return nil }
        return CGImageSourceCreateImageAtIndex(src, 0, nil)
    }
}

/// Bỏ các đoạn APP1…APP15 (EXIF, XMP, ICC, IPTC/APP13…) và COM khỏi một tệp JPEG; giữ APP0 (JFIF), bảng lượng tử,
/// khung ảnh và dữ liệu nén. Tệp không phải JPEG → trả nguyên.
enum JPEGSanitizer {
    static func strip(_ d: Data) -> Data {
        let b = [UInt8](d)
        guard b.count > 4, b[0] == 0xFF, b[1] == 0xD8 else { return d }
        var out: [UInt8] = [0xFF, 0xD8]
        var i = 2
        while i + 1 < b.count {
            guard b[i] == 0xFF else { return d } // hỏng: trả nguyên để không làm mất ảnh
            var m = b[i + 1]
            var j = i + 1
            while m == 0xFF, j + 1 < b.count { j += 1; m = b[j] } // byte đệm
            let start = j - 1
            if m == 0xDA { out.append(contentsOf: b[start...]); return Data(out) } // SOS: phần còn lại là dữ liệu nén
            if m == 0xD9 { out.append(contentsOf: [0xFF, 0xD9]); return Data(out) }
            if (0xD0...0xD7).contains(m) || m == 0x01 { out.append(contentsOf: [0xFF, m]); i = j + 1; continue }
            guard j + 2 < b.count else { return d }
            let len = Int(b[j + 1]) << 8 | Int(b[j + 2])
            let end = j + 1 + len
            guard len >= 2, end <= b.count else { return d }
            let drop = (0xE1...0xEF).contains(m) || m == 0xFE
            if !drop { out.append(contentsOf: b[start..<end]) }
            i = end
        }
        return Data(out)
    }

    /// Các mã đoạn (marker) trước SOS — dùng trong kiểm thử S9.
    static func markers(_ d: Data) -> [UInt8] {
        let b = [UInt8](d)
        var res: [UInt8] = []
        guard b.count > 4, b[0] == 0xFF, b[1] == 0xD8 else { return res }
        var i = 2
        while i + 3 < b.count, b[i] == 0xFF {
            let m = b[i + 1]
            res.append(m)
            if m == 0xDA || m == 0xD9 { break }
            if (0xD0...0xD7).contains(m) || m == 0x01 { i += 2; continue }
            let len = Int(b[i + 2]) << 8 | Int(b[i + 3])
            i += 2 + len
        }
        return res
    }
}

/// Bộ đệm ảnh đã giải mã (theo id ảnh) để danh sách và trang hồ sơ không giải mã lại.
final class PhotoCache {
    static let shared = PhotoCache()
    private let cache = NSCache<NSString, CGImageBox>()
    final class CGImageBox { let img: CGImage; init(_ i: CGImage) { img = i } }

    func image(_ id: String, load: () -> Data?) -> CGImage? {
        if let b = cache.object(forKey: id as NSString) { return b.img }
        guard let d = load(), let img = PhotoProcessor.cgImage(d) else { return nil }
        cache.setObject(CGImageBox(img), forKey: id as NSString)
        return img
    }
}
