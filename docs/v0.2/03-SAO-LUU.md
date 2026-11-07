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

(YC-2026-07, 2026-11-07) Danh sách khối chứa **ảnh rồi tệp đính kèm** (01 §3.8a): trước hết các `Photo` theo thứ
tự `photos`, sau đó các `Attachment` theo thứ tự `attachments`. `photo_count` là tổng số khối. Tệp không có tệp đính
kèm giữ nguyên từng byte như trước (khóa `attachments` của JSON chỉ có khi khác rỗng), nên vector 03 §2.5 không đổi.

- Không nén (ảnh đã nén sẵn; tránh "bom giải nén"). Giới hạn: `json_len` ≤ 64 MiB, mỗi khối ≤ 10 MiB (ảnh ≤ 1 MiB
  theo 01 §3.8, ảnh đính kèm ≤ 2 MiB, PDF ≤ 10 MiB theo 01 §3.8a; giới hạn từng loại kiểm theo bản ghi JSON), tối đa
  20 000 khối, tổng bản rõ ≤ **128 MiB** (giữ nguyên). Vượt giới hạn khi xuất → báo "Quá nhiều ảnh để sao lưu trong một tệp" (v0.2 mã hóa một
  lần trong bộ nhớ; mã hóa theo luồng để dành cho v0.3, rủi ro R-BAK-2).
- Mỗi ảnh phải có bản ghi `Photo` cùng `id` trong JSON và `sha256` khớp; thừa hoặc thiếu → từ chối toàn bộ. Cùng
  quy tắc cho tệp đính kèm (bản ghi `Attachment`, `sha256` và `bytes` khớp, PDF bắt đầu bằng `%PDF-`); khối ảnh
  đứng sau một khối tệp đính kèm → từ chối.
- Ảnh nhập từ tệp sao lưu **được giải mã và mã hóa lại** qua đúng quy trình thêm ảnh (01 §3.8) trước khi lưu, để
  ảnh do người khác tạo không đưa siêu dữ liệu hay dữ liệu dị dạng vào kho.
- (04 Bổ sung B6, 2026-11-07 — thay biện pháp ở dòng trên: **không** mã hóa lại khi nhập, vì sẽ đổi byte và `sha256`,
  trái R6 (a)/(d).) Lõi kiểm **chặt** từng khối trước khi ghi bất cứ thứ gì (`lvhv_store::media` ≡
  `pwa/src/store/media.ts`, cùng mã lỗi; bộ tệp độc hại và mã kỳ vọng ở `schema/test-vectors/media/`): chữ ký khớp
  `mime`; JPEG/WebP đọc kích thước từ phần đầu, không giải mã, phải bằng bản ghi (`Photo`) và ≤ 2048 px; JPEG không có
  APP1/APP13, WebP không có khối hay cờ EXIF/XMP; PDF bắt đầu `%PDF-`, có `%%EOF` trong 1024 byte cuối, ≤ 10 MiB. Sai bất
  kỳ khối nào → từ chối **toàn bộ** tệp, thông báo chung. Cùng bộ kiểm ở `add_photo`, `add_attachment` (lỗi `Input` có
  câu cụ thể) và trong chuyển đổi lược đồ (lỗi → chỉ đọc).
- (Rà soát bảo mật vòng 2, BM3-04/BM3-05, 2026-11-07) Kiểm tra trên **siết thêm** (cả Rust và TS, cùng mã):
  JPEG được duyệt qua mọi đoạn và mọi lượt quét (kể cả ảnh lũy tiến) tới EOI — APP1/APP13 ở bất kỳ đâu bị từ chối, EOI
  phải là 2 byte cuối của tệp; WebP: khối RIFF phải là cả tệp (`riff + 8 == len`, thêm đúng một byte 0 đệm nếu `riff`
  lẻ) và các khối phải phủ đúng tới cuối khối RIFF. Có byte nối sau ảnh → mã mới `trailing_data` (một JPEG có EXIF GPS
  nối sau EOI của ảnh hợp lệ nay bị từ chối). JPEG có SOF cao hoặc rộng = 0 (DNL), WebP có kích thước 0 → `bad_header`.
  PNG không được nhận ở đâu (`bad_mime`), nên không có quy tắc PNG. Ca mới ở `schema/test-vectors/media/` (39 ca; 25 ca
  và 14 tệp cũ giữ nguyên từng byte). Mọi ảnh của vector đã khóa (03 §2.5, R6v2, bộ mẫu đóng băng) kết thúc đúng ở EOI /
  cuối khối RIFF nên không đổi kết quả.
- (04 Bổ sung B7, 2026-11-07) Kiểm tra trên **bật cho cả ảnh chân dung** (`Photo`) khi nhập. R6 chạy trên bộ R6v2 ảnh thật
  (§B1.6); bộ R6 cũ có ảnh giả nay bị từ chối. Tệp đính kèm (01 §3.8a) được kiểm như trước (gồm X21: byte PDF khai
  `image/jpeg`).
- Định dạng khối của tệp đính kèm (khối ảnh rồi khối tệp đính kèm, mục trên) là định dạng **chuẩn** cho mọi bản
  cài đặt (rà soát BM2-03); vector `supplement-yc05` (một ảnh và một PDF đính kèm) và `scripts/check-store.sh` kiểm Rust ↔
  PWA đọc được tệp của nhau.

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
  trả byte tệp                                    # lõi: export_backup — KHÔNG đổi last_backup_at
  ghi tệp qua hộp thoại lưu của hệ điều hành / tải về (không ghi tệp tạm bản rõ)
  chỉ khi ghi xong: mark_backup_saved(now)        # settings.last_backup_at := now (xem dưới)

import(file, password):
  kiểm tra §2.2
  key := Argon2id(…tham số trong phần đầu…)
  pt  := decrypt(key, nonce, ct, aad = hdr)        # thất bại → "Sai mật khẩu hoặc tệp bị hỏng" (một thông báo chung)
  zeroize(key)
  unpack → kiểm tra 01 §6.3 → xem trước → Thay toàn bộ / Gộp → một giao dịch
```

- **Tạo byte và ghi nhận "đã sao lưu" là hai bước** (duyệt mã native MA3N-23-01). `export_backup` chỉ tạo byte
  tệp và nhớ thời điểm tạo (`meta.last_export_at`). Ứng dụng gọi `mark_backup_saved(now)` **một lần**, chỉ khi hệ
  điều hành báo ghi tệp xong. Khi đó `settings.last_backup_at := now`, và `backup_pending_since` chỉ bị xóa nếu nó
  không muộn hơn lần tạo tệp (thay đổi xảy ra giữa lúc tạo tệp và lúc lưu chưa có trong tệp, nên vẫn chờ sao lưu).
  Hủy hộp "Lưu" hoặc ghi lỗi thì **không** gọi, nên nhắc sao lưu vẫn còn. Gọi khi chưa tạo tệp nào (hoặc gọi lần thứ
  hai cho cùng một tệp) → lỗi đầu vào. Byte tệp không đổi: vectơ R6 giữ nguyên.
- Mật khẩu: ứng dụng dùng `backup_password_error(password, confirm)` của lõi (tsport `backupPasswordError`).
  Hàm trả `null` nếu hợp lệ, "Mật khẩu cần ít nhất 8 ký tự" nếu dưới 8 điểm mã sau NFC, và "Hai lần nhập mật khẩu
  chưa giống nhau" nếu ô nhập lại có giá trị và khác sau NFC. Ô mật khẩu khi khôi phục gọi với `confirm = null`.
- Trước khi đọc cả tệp, ứng dụng native gọi `backup_header_check(72 byte đầu, kích thước tệp)`. Hàm kiểm §2.2,
  kể cả `plaintext_len` khớp kích thước, nên tệp quá lớn hay không phải tệp sao lưu bị từ chối mà không phải đọc
  hết vào bộ nhớ.
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

## Bổ sung B1 — triển khai đợt 2 (2026-11-06, ghi khi viết mã; không đổi định dạng tệp, tham số hay tiêu chí)

Bổ sung này **ghi lại** cách đợt 2 cài đặt §2–§4 theo quyết định G1 (`06-BIEN-BAN-G1.md`: O-7 = thư viện
JavaScript, CSP giữ nguyên 0.1.1, không WebAssembly; O-8 = chấp nhận có điều kiện Argon2 chưa kiểm toán). Định dạng
tệp phiên bản 1, tham số KDF/AEAD và mọi ngưỡng của `04` **không đổi**.

### B1.1 Phiên bản đã ghim

| Nơi | Thư viện | Ghim | Phát hành | Dùng cho |
|---|---|---|---|---|
| Rust `backup/` | `argon2` | `=0.6.0` | 2026-08-27 | Argon2id v0x13 |
| Rust `backup/` | `chacha20poly1305` | `=0.11.0` | 2026-06-28 | XChaCha20-Poly1305 |
| Rust `backup/` | `zeroize` | `=1.9.1` | 2026-10-06 | xóa khóa, mật khẩu, bản rõ |
| Rust `backup/` | `getrandom` | `=0.4.3` | 2026-06-17 | salt, nonce (OS RNG) |
| Rust `backup/`, `store/` | `unicode-normalization` | `=0.1.26` | 2026-10-16 | NFC mật khẩu, khóa tên loại sự kiện |
| Rust `store/` | `rusqlite` (`bundled`, SQLite đóng gói) | `=0.41.1` | 2026-10-28 | kho SQLite |
| Rust `store/` | `sha2` | `=0.11.1` | 2026-10-12 | `sha256` ảnh, băm trạng thái kho |
| PWA (`pwa/package.json`, `dependencies`) | `@noble/ciphers` | `2.4.0` (chính xác) | 2026-08-27 | `xchacha20poly1305` |
| PWA | `@noble/hashes` | `2.4.0` (chính xác) | 2026-08-27 | `argon2idAsync` |
| PWA (chỉ kiểm thử, `devDependencies`) | `fake-indexeddb` | `6.2.6` | 2026-10-16 | IndexedDB trong Node |

Phụ thuộc bắc cầu khóa bằng `Cargo.lock` và `pwa/package-lock.json`. Phiên bản npm và ngày phát hành đọc từ
registry.npmjs.org ngày 2026-11-06; crates.io cùng ngày. SHA-256 và trạng thái kiểm toán bên dưới chỉ đúng với các
phiên bản này — mọi nâng phiên bản là PR riêng, chạy lại `scripts/check-store.sh`.

### B1.2 Tình trạng kiểm toán — phần nào đã, phần nào chưa (ghi cho tài liệu phát hành, theo O-8)

| Thành phần dùng trong ứng dụng | Kiểm toán độc lập | Ghi chú |
|---|---|---|
| XChaCha20-Poly1305, Rust (`chacha20poly1305` 0.11.0) | **Có, nhưng cho mã cũ**: NCC Group 2020 (mã năm 2019–2020; XChaCha không được nêu trong phạm vi) | §4.2 |
| XChaCha20-Poly1305, PWA (`@noble/ciphers` 2.4.0) | **Có, nhưng cho bản cũ**: Cure53, 9/2024, bản 1.0.0, phạm vi "everything" (README của gói 2.4.0). Từ 1.0.0 đến 2.4.0 chỉ có tự kiểm của tác giả ("AI-assisted self-audits" từ 4/2026, theo README) | Không có kiểm toán độc lập cho 2.4.0 |
| Argon2id, Rust (`argon2` 0.6.0) | **Không** — không tìm thấy báo cáo công bố | Rủi ro mở R-CRY-1 |
| Argon2id, PWA (`@noble/hashes` 2.4.0, mô-đun `argon2`) | **Không** — README ghi rõ: kiểm toán Cure53 1/2022 (bản 1.0.0) "everything, besides `blake3`, `sha3-addons`, `sha1` and `argon2`, which have not been audited" | Rủi ro mở R-CRY-1 |
| SHA-256 (WebCrypto `crypto.subtle` ở PWA; `sha2` ở Rust) | Không đánh giá — chỉ dùng cho toàn vẹn ảnh và băm kiểm thử, không cho bí mật | — |

Biện pháp bù đã làm (O-8): (i) vector chuẩn chạy qua đúng bản đã ghim ở **cả hai** bản cài đặt — Argon2id RFC 9106
§5.3 (có secret và AD) và XChaCha20-Poly1305 `draft-irtf-cfrg-xchacha` A.3; (ii) kiểm chéo hai bản cài đặt độc
lập của Argon2id (RustCrypto ↔ noble): cùng khóa cho tham số của ứng dụng (64 MiB, t = 3, p = 4), tệp do Rust
xuất mở trong PWA và ngược lại, và tệp xuất với salt/nonce cố định **giống từng byte** giữa hai bản; (iii) ghi rõ
trong tài liệu phát hành (bảng trên). Bên giải mã độc lập bằng Python (`argon2-cffi` + `PyNaCl`, R6(d) phần thêm
cho O-7 (B)) do **bên không tham gia xây dựng đặc tả** viết — chưa có trong đợt này.

Nguồn: README trong gói npm đã cài (`node_modules/@noble/ciphers/README.md` mục Security, `@noble/hashes/README.md`
mục Security), đọc 2026-11-06; §4.2 cho phần Rust.

### B1.3 PWA: tải lười, CSP, bộ nhớ đệm

- Mật mã chạy bằng JavaScript thuần (`@noble/*`), **không WebAssembly**; CSP sinh ra giống từng byte bản 0.1.1
  (kiểm thử khói so chuỗi, mục S2b).
- `pwa/src/storage.ts` là cửa vào duy nhất; nó nạp `src/store/` bằng `import()`, và `src/store/db.ts` nạp
  `backup-crypto.ts` (+ `@noble/*`) bằng `import()` khi xuất/nhập. `build.mjs` bật tách mã (esbuild
  `splitting`), tính bao đóng import tĩnh của `app-*.js` và **dừng dựng** nếu bao đóng đó chứa `@noble/*` hoặc
  `src/store/` (P6). Danh sách tệp tải lười ghi ở `dist/lazy-manifest.json`; các tệp này **không** nằm trong
  precache, service worker cache chúng khi trang nạp lần đầu (để dùng được ngoại tuyến sau đó).
- Argon2id trong trình duyệt: `argon2idAsync` (nhường luồng chính mỗi 10 ms, có `onProgress`), `maxmem` = m KiB +
  1 MiB. Xóa khóa/bản rõ trong JS bằng `fill(0)` chỉ là cố gắng (BK-I3), không bảo đảm.
- `requestPersistentStorage()` gọi `navigator.storage.persist()` (nếu có) khi lưu hồ sơ đầu tiên và ghi kết quả
  vào kho `meta` (`persist`); giao diện đợt 3 đọc `persistResult()` để hiện cảnh báo 01 §4.1.
- Hook nhắc sao lưu: `backupReminderDue(now)` = có thay đổi chưa sao lưu (`meta.backup_pending_since`, ghi ở thay
  đổi đầu tiên sau lần xuất gần nhất) và `now − backup_pending_since ≥ backup_nag_days` ngày; `backup_nag_days = 0`
  tắt. `exportBackup` chỉ tạo byte và ghi `meta.last_export_at`. Sau khi tải tệp, giao diện gọi
  `markBackupSaved(now)`: đặt `settings.last_backup_at`, xóa `backup_pending_since` nếu không có thay đổi nào sau
  lúc tạo tệp, và xóa `last_export_at` (§2.4). Cùng quy tắc ở Rust (`Store::backup_reminder_due`,
  `Store::mark_backup_saved`).

### B1.4 Những điểm đặc tả chưa nói rõ — cách đã cài (cả hai bản như nhau)

1. **Thứ tự kiểm phần đầu** (§2.2): độ dài/`magic` → `file_version` (> 1: "mới hơn"; = 0: không hỗ trợ) →
   `kdf_id`/`aead_id` → tham số KDF → `plaintext_len`. Mỗi nhánh có mã lỗi ổn định (`not_a_backup`,
   `newer_version`, `unsupported`, `bad_kdf_params`, `bad_length`, `wrong_password_or_corrupt`,
   `malformed_plaintext`); thông báo cho người dùng theo §2.2/§2.4.
2. **Lỗi nội dung sau khi giải mã** (JSON, lược đồ, V2–V8, ảnh): thông báo chung "Sai mật khẩu hoặc tệp bị hỏng";
   riêng `schema_version` lớn hơn → "Tệp được tạo bởi phiên bản mới hơn…" (01 §7).
3. **Số nguyên trong JSON**: `2.0` được coi là `2` (như JSON Schema và `JSON.parse`); mọi số nguyên phải ≤ 2^53 − 1
   (siết thêm duy nhất, để hai bản đọc giống nhau). Chuỗi có nửa cặp thay thế (lone surrogate) bị từ chối ở cả hai.
4. **`\S` của `ho_ten`/`ten`**: theo định nghĩa `\s` của JavaScript.
5. **Kiểu loại có sẵn**: `builtin ≠ null` phải dùng đúng id cố định của 01 §3.5; kho mới tạo bốn loại với
   `created_at = updated_at = 1970-01-01T00:00:00Z`, `rev = 1` (cố định để hai máy trống giống nhau).
6. **Ngày sinh dương đủ ngày nhưng năm 1000–1899** (ngoài miền đổi lịch): `lunar_year = y`,
   `lunar_year_approx = true`, `lunar = null`, `solar = input`.
7. **V5 với `method` khác phương pháp hiện hành**: bỏ qua so `lunar`, `solar`, `method` (giữ giá trị đã lưu); vẫn so
   `lunar_source`. **V5 với `tuoi.manual = true`**: không tính lại; chỉ kiểm năm âm mất ≥ năm âm sinh.
8. **Gộp ảnh**: ảnh trên máy giữ nguyên; ảnh nhập chỉ được thêm nếu có hồ sơ (sau gộp) trỏ tới.
9. **JSON chuẩn của tệp xuất**: khóa sắp theo thứ tự điểm mã, mảng sắp theo `id`, không khoảng trắng — hai bản
   xuất cùng dữ liệu ra cùng byte (với salt/nonce cố định).
10. **Trạng thái kho để so "giống hệt"** (R6): SHA-256 của JSON chuẩn toàn kho (bỏ `app`, `exported_at`) nối với
    `sha256` hex của byte từng ảnh theo thứ tự `id` — `Store::state_hash` (Rust) = `Store.stateHash` (PWA).

### B1.5 Vector kiểm thử (§2.5) và cách chạy

`docs/v0.2/schema/test-vectors/` (sinh tất định bằng `cargo run --release -p lvhv-store --example
gen_test_vectors`; CI sinh lại và so từng byte): `example-v1.lvhv` (+ `example-v1.json`, ảnh mẫu JPEG không EXIF),
21 tệp hỏng trong `corrupt/` (sửa từng trường phần đầu, cắt cụt, nối thêm, `m_cost_kib` = 4 194 304,
`file_version` = 2 …) với mã lỗi mong đợi trong `manifest.json`, `dataset-r6.json` (60 người, 30 sự kiện, 20 ảnh),
`validation-cases.json` (63 ca V1–V8 + độ sâu JSON), `merge-cases.json` (31 ca gộp, kết quả mong đợi viết tường minh
và băm kết quả). `scripts/check-store.sh` chạy toàn bộ phía Rust và PWA và kiểm chéo qua tệp thật (job CI `store`).

### B1.6 Bộ R6v2 — ảnh thật (04 Bổ sung B7; KHÓA 2026-11-07)

Bộ R6 cũ (`dataset-r6.json`, 20 "ảnh" là byte tổng hợp `FF D8 FF E0 … FF D9`, 10 ảnh khai `image/webp`) giữ nguyên trong
kho để đối chiếu lịch sử; mọi tệp và giá trị đã khóa của nó (`manifest.json` → `dataset_r6`) **không đổi**. Từ khi bật
kiểm tra chặt B6 cho ảnh chân dung ở đường nhập sao lưu, bộ R6 cũ **bị từ chối** khi nhập ("Sai mật khẩu hoặc tệp bị
hỏng") — kiểm thử `old_r6_rejected_on_import` (Rust) và mục tương ứng trong `pwa/test/store/run.mjs`.

R6 (a)–(d) và R6 (b) lật bit chạy trên **R6v2** ở `docs/v0.2/schema/test-vectors/r6v2/`: cùng cấu trúc, cùng id, cùng 60
người, 30 sự kiện, 20 ảnh, cùng kiểu và kích thước khai báo (ảnh i: (300 + i) × (400 + i), JPEG khi i chẵn, WebP khi i
lẻ) như R6; chỉ byte ảnh (và `bytes`/`sha256` của bản ghi) khác. Ảnh là dải màu tất định, mã hóa một lần bằng
`make_r6v2_photos.py` (Pillow, tham số cố định), không siêu dữ liệu (JPEG chỉ JFIF APP0; WebP dạng `VP8 ` đơn giản),
qua kiểm tra B6. `dataset-r6v2.json` và `dataset-r6v2-fixed.lvhv` (salt/nonce cố định như §2.5, mật khẩu
"mật khẩu thử nghiệm") sinh tất định bằng `gen_test_vectors` từ các ảnh đã khóa; CI sinh lại và so từng byte.

- `store_state_sha256` (băm kho sau khi nhập vào SQLite và IndexedDB): `9f4f7bb9f47d1a497decde0d1a53ec40f0aebddf9a00c20796a60599e1fdf36e`
- `fixed_rng_file_sha256`: `c8f511be1967cde710fe01e193fee339a6852f862dc5d71007fb5f2ea26222b3` (327973 byte)

| Tệp (`r6v2/`) | sha256 |
|---|---|
| `dataset-r6v2-fixed.lvhv` | `c8f511be1967cde710fe01e193fee339a6852f862dc5d71007fb5f2ea26222b3` |
| `dataset-r6v2.json` | `ff2bb5fb36d9006b3771538d4564078e7f6d36b22ebb8c406d1124383515d735` |
| `make_r6v2_photos.py` | `bdbd644749e03faed54eb03e3808d9300ea782bb20b47be954816f480773d694` |
| `photo-00.jpg` | `ac6b71b54ed164f3dfcba4b554cba2605c46822b51151ae928871478ef33808a` |
| `photo-01.webp` | `3dab4ed58d25b443033860425f8a293a831c03f9bb2518fcf88afde94584e493` |
| `photo-02.jpg` | `58c5416386fde10cd522bbbf1deef2b03f20e5b35462bba2cd16288d583a327a` |
| `photo-03.webp` | `151c65fa88b1163dd7b2ff8ae4813291ae2f2fd4584f5742a706bb6c54fb67c8` |
| `photo-04.jpg` | `bd77168d7095335d412845529dc1ecb6477e89647033fe4f2c84e39cddc08796` |
| `photo-05.webp` | `112fa95eb8b7d6b47d2ce39cb514049877285bf151569f53b022830af56cc2df` |
| `photo-06.jpg` | `dd82cc004402c4aa3660b7e8612f786ee6dd0570b87acb25825cd4a97b5b65f8` |
| `photo-07.webp` | `3609e02c8e3d11b07392ed93528fa0fc19d25b6e0a906ef0aad58a03c015ad5b` |
| `photo-08.jpg` | `e58f56eb2e86c53344b09e13b56358bf961f46d556d02c83fe6cdbaf33d1fdd1` |
| `photo-09.webp` | `71563132c54b9180ed2e7498ab1fa137b207a99acc21868a75e31d97fa703b24` |
| `photo-10.jpg` | `c82916a2d5b33e4b89bcc96c54e1f721d3b0bd3359753f87b413622d7bb4e6f8` |
| `photo-11.webp` | `7852a36cc2f52fe0e0b550f515021e906fe3a28d766f386d71d86fe89abcccc0` |
| `photo-12.jpg` | `4b3e39cd6a58b7e797bbeb79233987741dd9f6c585a9c2eb7aabdc34ea34404a` |
| `photo-13.webp` | `01f328e0a7260f5e5a9e4ba45a10f23f701e469594597594665494de88ef8d51` |
| `photo-14.jpg` | `36878d5ac99420499875ad7b657fca98ae614d4491a09427594be9fae735c798` |
| `photo-15.webp` | `d875245a87198a20d1557b18ebf773fa5c4698b032bb7f6eb234a36cadf6f1af` |
| `photo-16.jpg` | `cbbc05476e12effbe6c71ba4ee5761226dec75662c779af3989d2bbaf381967d` |
| `photo-17.webp` | `88dd823b78d10f3cc941aebcd4d342297e660eff0eb94155f59711800b6f29b7` |
| `photo-18.jpg` | `7f1ef9dbb363c03111c345e1987b7ef1c507b1d42d63e5c84bbc2d70fc35546d` |
| `photo-19.webp` | `bf7f5c80c09042806c2247420d72659e9e37d02303e349d59bc41f4bbd388dd3` |

