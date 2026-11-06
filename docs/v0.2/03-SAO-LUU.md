# Tệp sao lưu mã hóa — định dạng, mô hình đe dọa, thư viện — v0.2

Trạng thái: **KHÓA (G1) — 2026-11-06, chờ chủ dự án ký.** Hai mục mở ảnh hưởng tới tài liệu này (O-7 cách chạy
mật mã trong PWA, O-8 mức kiểm toán chấp nhận được cho Argon2) phải được chủ dự án quyết định **trước khi viết mã
đợt 2**; định dạng tệp dưới đây không phụ thuộc hai quyết định đó.

Nguyên tắc: **không tự viết thuật toán mật mã.** Chỉ ghép hai thành phần chuẩn (Argon2id, XChaCha20-Poly1305) từ
thư viện có sẵn theo cách dùng được tài liệu của thư viện khuyến nghị. Không có cửa sau, không có khóa khôi phục:
**quên mật khẩu là mất dữ liệu trong tệp** — giao diện nói rõ điều này trước khi xuất.

## 1. Tổng quan

```
mật khẩu (NFC, UTF-8) ──Argon2id(salt, m, t, p)──► khóa 32 byte
JSON dữ liệu (01 §5) + byte ảnh ──đóng gói (§2.3)──► bản rõ
bản rõ ──XChaCha20-Poly1305(khóa, nonce 24 byte, AAD = toàn bộ phần đầu)──► bản mã ‖ thẻ 16 byte
tệp = phần đầu (72 byte) ‖ bản mã ‖ thẻ
```

- Tên tệp mặc định: `LichVanHoaViet-saoluu-<YYYY-MM-DD>.lvhv` (không chứa tên người). Kiểu MIME khi chia sẻ:
  `application/octet-stream`.
- Mỗi lần xuất sinh **salt mới** và **nonce mới** bằng bộ sinh ngẫu nhiên an toàn của hệ điều hành
  (`getrandom`/`OsRng`; trong trình duyệt `crypto.getRandomValues`). Vì vậy mỗi tệp có khóa riêng; nonce 192 bit
  ngẫu nhiên đủ an toàn cho XChaCha20 (đó là lý do chọn biến thể X).

## 2. Định dạng nhị phân (phiên bản tệp 1)

### 2.1 Phần đầu — 72 byte, số nguyên little-endian

| Vị trí | Độ dài | Trường | Giá trị phiên bản 1 |
|---|---|---|---|
| 0 | 8 | `magic` | ASCII `LVHVBAK` + byte `0x00` |
| 8 | 2 | `file_version` | `1` |
| 10 | 1 | `kdf_id` | `1` = Argon2id, phiên bản thuật toán 0x13 (RFC 9106) |
| 11 | 1 | `aead_id` | `1` = XChaCha20-Poly1305 (khóa 32, nonce 24, thẻ 16 byte) |
| 12 | 4 | `m_cost_kib` | `65536` (64 MiB) |
| 16 | 4 | `t_cost` | `3` |
| 20 | 4 | `p_cost` | `4` |
| 24 | 16 | `salt` | ngẫu nhiên |
| 40 | 24 | `nonce` | ngẫu nhiên |
| 64 | 8 | `plaintext_len` | độ dài bản rõ (byte) |

- Tham số Argon2id mặc định theo **phương án khuyến nghị thứ hai của RFC 9106 §4** (t = 3, p = 4, m = 2¹⁶ KiB =
  64 MiB, salt 128 bit, khóa 256 bit), vì phương án thứ nhất (2 GiB) không chạy được trên điện thoại và trình duyệt.
  Thời gian đo thật trên thiết bị tham chiếu là tiêu chí P8 (`04`); nếu vượt ngưỡng, chỉ được **giảm `t`/`p`**, không
  giảm `m` dưới 64 MiB, và phải ghi lại.
- Toàn bộ 72 byte phần đầu là **dữ liệu liên kết (AAD)** của AEAD: sửa bất kỳ byte nào (kể cả hạ tham số KDF) làm
  giải mã thất bại.

### 2.2 Kiểm tra trước khi tốn tài nguyên (chống từ chối dịch vụ)

Khi nhập, **trước khi** chạy Argon2:

| Điều kiện | Xử lý |
|---|---|
| `magic` sai, tệp < 72 + 16 byte | "Đây không phải tệp sao lưu của Lịch và Văn hóa Việt" |
| `file_version` > 1 | "Tệp được tạo bởi phiên bản mới hơn — hãy cập nhật ứng dụng" |
| `kdf_id ≠ 1`, `aead_id ≠ 1` | Từ chối |
| `m_cost_kib` ∉ [65 536, 1 048 576], `t_cost` ∉ [1, 10], `p_cost` ∉ [1, 8] | Từ chối (chặn tệp ép máy cấp 4 GiB hoặc chạy hàng giờ; chặn tệp hạ tham số yếu) |
| `plaintext_len` ≠ kích thước tệp − 72 − 16, hoặc > 128 MiB | Từ chối |

### 2.3 Bản rõ (sau khi giải mã)

```
bản rõ := "LVHVPT01" (8 byte ASCII)
          ‖ u32 json_len ‖ json (UTF-8, đúng lược đồ backup-v{schema_version}, 01 §5)
          ‖ u32 photo_count
          ‖ photo_count × ( 16 byte UUID ảnh (nhị phân) ‖ u32 len ‖ len byte ảnh )
```

- Không nén (ảnh đã nén sẵn; tránh "bom giải nén"). Giới hạn: `json_len` ≤ 64 MiB, mỗi ảnh ≤ 1 MiB (01 §3.8),
  tổng bản rõ ≤ **128 MiB**. Vượt giới hạn khi xuất → báo "Quá nhiều ảnh để sao lưu trong một tệp" (v0.2 mã hóa một
  lần trong bộ nhớ; mã hóa theo luồng để dành cho v0.3, rủi ro R-BAK-2).
- Mỗi ảnh phải có bản ghi `Photo` cùng `id` trong JSON và `sha256` khớp; thừa hoặc thiếu → từ chối toàn bộ.
- Ảnh nhập từ tệp sao lưu **được giải mã và mã hóa lại** qua đúng quy trình thêm ảnh (01 §3.8) trước khi lưu, để
  ảnh do người khác tạo không đưa siêu dữ liệu hay dữ liệu dị dạng vào kho.

### 2.4 Quy trình

```
export(password):
  require len_codepoints(NFC(password)) >= 8; hỏi nhập lại; hiện cảnh báo "quên mật khẩu là mất dữ liệu"
  salt, nonce := random(16), random(24)
  key := Argon2id(NFC(password) as UTF-8, salt, m, t, p, out = 32)
  pt  := pack(json, photos)                        # §2.3
  hdr := header(…, len(pt))
  ct  := XChaCha20Poly1305.encrypt(key, nonce, pt, aad = hdr)
  zeroize(key, pt)
  ghi tệp qua hộp thoại lưu của hệ điều hành / tải về (không ghi tệp tạm bản rõ)
  settings.last_backup_at := now

import(file, password):
  kiểm tra §2.2
  key := Argon2id(…tham số trong phần đầu…)
  pt  := decrypt(key, nonce, ct, aad = hdr)        # thất bại → "Sai mật khẩu hoặc tệp bị hỏng" (một thông báo chung)
  zeroize(key)
  unpack → kiểm tra 01 §6.3 → xem trước → Thay toàn bộ / Gộp → một giao dịch
```

- Không phân biệt "sai mật khẩu" với "tệp bị sửa" (AEAD không cho biết, và không nên đoán).
- Bản rõ chỉ ở trong bộ nhớ; không ghi tệp tạm, không đưa vào bộ nhớ đệm của Service Worker.
- Không giới hạn số lần thử trong ứng dụng (kẻ tấn công ngoại tuyến cũng không bị giới hạn); sức chống dò mật khẩu
  đến từ Argon2id và độ dài mật khẩu.

### 2.5 Vector kiểm thử

- Đợt 2 tạo và đưa vào kho `docs/v0.2/schema/test-vectors/`: (1) tệp mẫu từ `example-backup-v1.json` với mật khẩu
  `"mật khẩu thử nghiệm"`, salt/nonce cố định (chỉ dùng cho kiểm thử, hàm `export_with_fixed_rng`); (2) các tệp hỏng:
  sửa 1 byte ở mỗi trường phần đầu, cắt cụt, `m_cost_kib` = 4 194 304, `file_version` = 2.
- Thành phần mật mã được kiểm bằng vector chuẩn: Argon2id theo RFC 9106 §5.3; XChaCha20-Poly1305 theo vector của
  bản nháp IETF `draft-irtf-cfrg-xchacha` (phụ lục A.3). Hai bản cài đặt (Rust và bản dùng trong PWA nếu khác, xem
  O-7) phải giải mã được tệp của nhau (tiêu chí R6 trong `04`).

## 3. Mô hình đe dọa của tệp sao lưu (STRIDE)

Tài sản: **B1** nội dung tệp (tên, quan hệ, ngày sinh/mất, nơi an táng, ảnh, lời tưởng niệm — dữ liệu cá nhân của
người dùng và của người thân); **B2** tính toàn vẹn dữ liệu khi nhập lại; **B3** tính sẵn sàng của ứng dụng khi gặp
tệp độc hại. Ranh giới tin cậy: tệp rời khỏi ứng dụng và đi qua nơi không tin cậy (thư mục Tải về, ổ đĩa đám mây,
Zalo, thư điện tử, USB). Kẻ tấn công giả định: có được tệp (mất máy, lộ ổ đám mây, gửi nhầm), hoặc gửi cho người dùng
một tệp do mình tạo.

| # | Loại | Mối đe dọa | Biện pháp | Rủi ro còn lại |
|---|---|---|---|---|
| BK-S1 | Giả mạo | Kẻ xấu tạo tệp sao lưu bằng mật khẩu của chính họ, dụ người dùng nhập ("tệp gia phả của họ hàng") | Xem trước trước khi ghi; mặc định **Gộp**, không thay toàn bộ; dữ liệu chỉ là văn bản hiển thị (không thực thi, không HTML); ảnh mã hóa lại (§2.3) | Thấp: người dùng có thể nhận dữ liệu sai nội dung — vấn đề lòng tin, không phải thực thi mã |
| BK-T1 | Can thiệp | Sửa bản mã, phần đầu, cắt hoặc nối tệp | AEAD với AAD = phần đầu; kiểm tra độ dài §2.2 | Không đáng kể |
| BK-T2 | Can thiệp | Hạ tham số KDF để dễ dò mật khẩu ở tệp sau | Tham số nằm trong AAD; ngưỡng dưới khi nhập; ứng dụng luôn xuất bằng tham số mặc định của nó, không chép tham số của tệp cũ | Không đáng kể |
| BK-T3 | Can thiệp | Tệp cũ bị đưa lại (rollback) để ghi đè dữ liệu mới hơn | Gộp theo `rev`/`updated_at` (01 §6.3); xem trước hiện ngày xuất của tệp | Thấp nếu chọn "Thay toàn bộ" — có hộp xác nhận ghi rõ ngày xuất |
| BK-R1 | Chối bỏ | — | Không áp dụng: dữ liệu cục bộ của một người, không có giao dịch giữa các bên | — |
| BK-I1 | Lộ thông tin | Dò mật khẩu ngoại tuyến | Argon2id 64 MiB/t=3/p=4; mật khẩu ≥ 8 điểm mã, gợi ý dùng cụm từ; đồng hồ đo độ mạnh chỉ là gợi ý | **Trung bình** với mật khẩu yếu: mật khẩu 8 chữ số có thể bị dò. Đây là giới hạn cơ bản của mã hóa bằng mật khẩu |
| BK-I2 | Lộ thông tin | Siêu dữ liệu ngoài phần mã hóa: tên tệp, kích thước | Tên tệp không chứa tên người; phần đầu chỉ có tham số và số ngẫu nhiên | Thấp: kích thước tệp tiết lộ áng chừng số ảnh |
| BK-I3 | Lộ thông tin | Bản rõ rơi ra đĩa (tệp tạm, bộ nhớ đệm, nhật ký) | Không ghi tệp tạm; không ghi nhật ký nội dung; `zeroize` khóa và bản rõ ở lõi Rust (với JS chỉ là cố gắng, không bảo đảm — ghi chú) | Thấp |
| BK-I4 | Lộ thông tin | Ảnh mang GPS/EXIF | Ảnh đã được mã hóa lại khi thêm (01 §3.8) — trong tệp không có EXIF | Không đáng kể (kiểm ở S9) |
| BK-I5 | Lộ thông tin | Mật khẩu bị lộ qua bàn phím/clipboard | Ô mật khẩu kiểu `password`, tắt tự sửa và gợi ý; không lưu mật khẩu | Phụ thuộc bàn phím của hệ điều hành |
| BK-D1 | Từ chối dịch vụ | Tham số KDF khổng lồ, tệp khổng lồ, JSON lồng sâu | §2.2 kiểm tra trước; JSON giới hạn độ sâu 32 và 64 MiB | Thấp: tệp 128 MiB hợp lệ vẫn tốn bộ nhớ trên máy yếu |
| BK-D2 | Từ chối dịch vụ | Quên mật khẩu | Cảnh báo trước khi xuất; không có cơ chế khôi phục (có chủ đích) | **Cao về hậu quả, do người dùng** — ghi trong hướng dẫn |
| BK-E1 | Nâng quyền | Lỗi bộ phân tích JSON hoặc bộ giải mã ảnh bị khai thác bằng tệp dị dạng | Phân tích bằng ngôn ngữ an toàn bộ nhớ (Rust `serde_json`, `JSON.parse`); chỉ giải mã ảnh **sau** khi AEAD xác thực, tức chỉ với tệp mà người dùng có mật khẩu; ảnh đi qua bộ giải mã của hệ điều hành rồi mã hóa lại | Thấp; phụ thuộc bộ giải mã ảnh của hệ điều hành |

Điều tệp sao lưu **không** bảo vệ: thiết bị đã bị chiếm quyền khi đang xuất/nhập; người biết mật khẩu; dữ liệu đang
nằm không mã hóa trong kho của ứng dụng trên máy (v0.2 dựa vào mã hóa toàn thiết bị của hệ điều hành; mã hóa kho cục
bộ là mục cho v0.3).

## 4. Thư viện

### 4.1 Lựa chọn và ghim phiên bản (native — Rust)

Mật mã đặt trong một crate mới `lvhv-backup` (không đặt trong `core/`, để `core` vẫn **0 phụ thuộc** — tiêu chí S2
của v0.1 giữ nguyên). Ghim chính xác trong `Cargo.toml`, khóa bằng `Cargo.lock`:

| Crate (RustCrypto) | Ghim | Phát hành (crates.io, xem 2026-11-06) | MSRV | Dùng cho |
|---|---|---|---|---|
| `argon2` | `=0.6.0` | 2026-08-27 | 1.85 | Argon2id, `Params::new(65536, 3, 4, Some(32))`, `Algorithm::Argon2id`, `Version::V0x13` |
| `chacha20poly1305` | `=0.11.0` | 2026-06-28 | 1.85 | `XChaCha20Poly1305` (`KeyInit`, `Aead` với `Payload { msg, aad }`) |
| `zeroize` | `=1.9.1` | 2026-10-06 | 1.85 | Xóa khóa, bản rõ |

Các phụ thuộc bắc cầu (`blake2`, `chacha20`, `poly1305`, `cpufeatures`, `getrandom`…) do `Cargo.lock` khóa; CI thêm
bước `cargo deny check advisories` (cơ sở dữ liệu RustSec) — bước này phải chạy trên CI, vì môi trường dựng tài liệu
G1 không truy cập được rustsec.org. Mọi nâng phiên bản là một PR riêng, kèm chạy lại vector kiểm thử §2.5.

### 4.2 Tình trạng kiểm toán (nguồn tự truy cập 2026-11-06)

| Thư viện | Bằng chứng kiểm toán | Đánh giá |
|---|---|---|
| `chacha20poly1305` (RustCrypto/AEADs) | NCC Group, "Public Report – RustCrypto AES-GCM and ChaCha20+Poly1305 Implementation Review", do MobileCoin tài trợ, báo cáo v1.0 ngày 2020-02-12, công bố 2020-02-26; phạm vi là mã ở commit `a15698fd…` của RustCrypto/AEADs (và commit tương ứng của `chacha20`, `poly1305`); **không có lỗ hổng**, chỉ ghi chú về hiệu năng. Tài liệu crate (docs.rs, 0.11.0) ghi: "has received one security audit by NCC Group, with no significant findings", và cảnh báo không dùng trên CPU nhân không hằng thời gian (một số PowerPC 32 bit, vi điều khiển không phải ARM) | **Có kiểm toán, nhưng của mã năm 2019–2020**, không phải 0.11.0; báo cáo không nhắc tới biến thể XChaCha (có vẻ ngoài phạm vi). Tài liệu crate cũng ghi XChaCha20-Poly1305 không có đặc tả chính thức (bản nháp IETF đã hết hạn). Nhân XChaCha = HChaCha20 + ChaCha20 dùng chung mã đã kiểm toán |
| `argon2` (RustCrypto/password-hashes) | **Không tìm thấy** báo cáo kiểm toán độc lập công bố: trang crate trên docs.rs và lib.rs không có tuyên bố kiểm toán | **CHƯA ĐỦ BẰNG CHỨNG** cho yêu cầu "thư viện đã kiểm toán" — mục mở **O-8** |
| `@noble/ciphers` (JavaScript, nếu O-7 chọn JS) | Cure53, "Audit-Report noble crypto libs", 2024-09-01, phạm vi **noble-ciphers 0.6.0** toàn bộ; 8 phát hiện (1 cao, 1 trung bình về AES — không phải ChaCha). README: kiểm toán độc lập ở bản 1.0.0 (9/2024), tự kiểm toán của tác giả ở 2.2.0 (4/2026). Bản mới nhất trên npm: 2.4.0 (2026-08-27) | Có kiểm toán độc lập cho bản cũ hơn |
| `@noble/hashes` → `argon2` (JavaScript) | README ghi kiểm toán Cure53 bản 1.0.0 (1/2022) **trừ** `blake3`, `sha3-addons`, `sha1`, `argon2` ("have not been audited") | **Không có kiểm toán** cho Argon2 |
| libsodium (C) — phương án thay thế | Kiểm toán của Cryptography Engineering (Matthew Green) cho libsodium 1.0.12/1.0.13, công bố 2017-08-16, "không có lỗi nghiêm trọng"; trang công bố không nói rõ `crypto_pwhash` (Argon2) có trong phạm vi hay không | Kiểm toán cũ; cần mã C và lớp gọi `unsafe` qua FFI, trái tinh thần lõi Rust thuần |

Kết luận cho chủ dự án (không phải quyết định thay): **chưa có lựa chọn nào mà cả Argon2id lẫn XChaCha20-Poly1305 đều
có kiểm toán độc lập hiện hành.** Đề xuất O-8 (A): dùng hai crate RustCrypto đã ghim, bù bằng (i) vector RFC 9106 và
bản nháp XChaCha, (ii) kiểm tra chéo với một bản cài đặt độc lập khác (tiêu chí R6), (iii) ghi rõ trong tài liệu phát
hành rằng Argon2 chưa được kiểm toán độc lập. (B): dùng libsodium (đổi lấy mã C/FFI). (C): đổi KDF sang thuật toán có
kiểm toán — chưa tìm được ứng viên vừa chống GPU vừa có kiểm toán hiện hành, nên không đề xuất.

### 4.3 PWA — mục mở O-7

PWA không chạy được crate Rust nếu không biên dịch sang WebAssembly; WebCrypto của trình duyệt không có Argon2 và
không có XChaCha20-Poly1305. Hai phương án:

| | (A) WebAssembly từ `lvhv-backup` | (B) Thư viện JavaScript `@noble/ciphers` + `@noble/hashes` |
|---|---|---|
| Một bản cài đặt cho mọi nền tảng | Có | Không (hai bản; R6 kiểm chéo) |
| CSP | Phải thêm `'wasm-unsafe-eval'` vào `script-src` (yêu cầu của trình duyệt để biên dịch WebAssembly) — **trái tiêu chí "CSP không đổi"** trừ khi chủ dự án duyệt ngoại lệ ghi trong `04` S2 | Không đổi |
| Kiểm toán | Như §4.2 (Rust) | ChaCha: có (bản cũ); Argon2: không |
| Hiệu năng Argon2id 64 MiB | Tốt hơn | Chậm hơn đáng kể trên điện thoại (đo ở P8) |
| Dung lượng | Tải lười (lazy) khi mở màn hình Sao lưu | Tải lười |

Đề xuất: (A), với ngoại lệ CSP duy nhất `'wasm-unsafe-eval'` và mô-đun WebAssembly chỉ được tải ở màn hình Sao lưu.
Quyết định thuộc chủ dự án; tiêu chí S2 đã đăng ký trước cả hai nhánh.

## 5. Nguồn (truy cập 2026-11-06)

- RFC 9106 *Argon2 Memory-Hard Function for Password Hashing and Proof-of-Work Applications*, §4 (hai phương án
  khuyến nghị, trích lại trong github.com/ranisalt/node-argon2/pull/360).
- crates.io API, các phiên bản `argon2`, `chacha20poly1305`, `zeroize`.
- docs.rs/chacha20poly1305 (0.11.0) — phần "Security Notes"; docs.rs/crate/argon2/latest; lib.rs/crates/argon2.
- nccgroup.com/research/public-report-rustcrypto-aesgcm-and-chacha20pluspoly1305-implementation-review/ và báo cáo
  PDF `NCC_Group_MobileCoin_RustCrypto_AESGCM_ChaCha20Poly1305_Implementation_Review_2020-02-12_v1.0`.
- cure53.de/audit-report_noble-crypto-libs.pdf; README của paulmillr/noble-hashes và noble-ciphers.
- privateinternetaccess.com/blog/2017/08/libsodium-audit-results/.
- Về `'wasm-unsafe-eval'`: tài liệu CSP của nutrient.io trích MDN ("permits WebAssembly compilation and
  instantiation").
