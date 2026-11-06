# Biên bản đo — v0.2 đợt 2: lưu trữ và sao lưu (vai B2)

Ngày đo: 2026-11-06. Máy đo: Linux x64, 4 vCPU (môi trường dựng), Rust 1.97 (`--release`), Node v22.23.4,
Chromium 153.0.8010.12 (Playwright 1.63.0, headless). Mã: nhánh `v0.2-w1-store` (commit ghi trong PR). Dữ liệu
thô: `pwa/test/metrics.json`, `pwa/test/store/out/results.json` (artefact CI `store-r6`), đầu ra `r2_full`
(dưới đây). Kết luận theo quy ước 04: ĐẠT / KHÔNG ĐẠT / CHƯA ĐỦ BẰNG CHỨNG, **theo từng nền tảng**.

"Native (Rust/SQLite)" dưới đây là crate `lvhv-store` chạy trên Linux — **chưa** phải ứng dụng Android/iOS/macOS/
Windows (lớp FFI và giao diện là đợt 3). Không kết luận nào dưới đây áp cho thiết bị thật.

## R2 — đổi ngày mất dương → âm khi lưu, khứ hồi, 1900–3000

| Phần | Rust / SQLite (`cargo run --release -p lvhv-store --example r2_full`, 384 s) | TypeScript / IndexedDB (`node pwa/test/store/r2-derive.mjs --store`) |
|---|---|---|
| (a) 402 132 ngày dương qua `save_death` của lớp lưu trữ → ghi (V1–V8 + giao dịch) → đóng/mở → đọc lại | 0 lỗi; băm `b6c8229d…71ada` | 0 lỗi (803 895 bản ghi so sau đóng/mở); băm **bằng** Rust |
| (b) 401 763 ngày âm tồn tại 1900–2999 (gồm tháng nhuận, ngày 30) | 0 lỗi; băm `19e97185…e0780` | 0 lỗi; băm **bằng** Rust |
| (c) 26 622 ngày không tồn tại (dương 29–31 sai, tháng nhuận không có, 30 tháng thiếu) | 26 622/26 622 bị từ chối, thông báo đúng 01 §6.1; kho không đổi | quy tắc giống hệt (cùng mã `derive.ts`); chưa chạy riêng toàn dải (c) |

Kết luận R2: **Rust/SQLite (Linux): ĐẠT**; Rust ≡ TypeScript ở (a), (b). **PWA: CHƯA ĐỦ BẰNG CHỨNG** — (a)(b) chạy
qua lớp IndexedDB nhưng là `fake-indexeddb` trong Node, không phải trình duyệt; (c) toàn dải chưa chạy phía TS.
Thiếu: chạy toàn dải trong Chromium thật (script cho máy trạm, đợt 3). Android/iOS/macOS/Windows: CHƯA ĐỦ BẰNG
CHỨNG (chưa có FFI/ứng dụng).

## R6 — sao lưu

| Phần | Rust / SQLite | PWA (Node + fake-indexeddb) | PWA (Chromium thật, kiểm thử khói) |
|---|---|---|---|
| (a) khứ hồi bộ R6 (60 người, 30 sự kiện, 20 ảnh) | giống hệt (`state_hash` 45f170c2…) | giống hệt, `stateHash` **=** Rust | 1 người + 1 ảnh: giống hệt |
| (a2) 20 mật khẩu sai; NFD đúng | 20/20 từ chối, NFD nhận, kho không đổi | 20/20, NFD nhận, kho không đổi | 1 mật khẩu sai: thông báo chung |
| (b) 21 tệp hỏng 03 §2.5 + lật bit 65 536 biến thể | 0 được chấp nhận (biến thể đổi salt/m/t/p chạy Argon2id thật), kho không đổi | 0/65 536 (152 qua Argon2id thật, 827 s), kho không đổi | — |
| (c) Argon2id RFC 9106 §5.3; XChaCha20-Poly1305 draft A.3 | khớp | khớp | — |
| (d) kiểm chéo | tệp PWA → SQLite: giống hệt; tệp salt/nonce cố định giống từng byte | tệp Rust → IndexedDB: giống hệt | mở được `example-v1.lvhv` của Rust |
| (e) 31 ca gộp | đúng kết quả viết tường minh; SQLite = hàm thuần | sha256 kết quả = Rust; IndexedDB = hàm thuần | — |

Kết luận R6: **CHƯA ĐỦ BẰNG CHỨNG.** Đã đạt ngưỡng ở mọi phần đã chạy (0 sai lệch, 0 tệp hỏng/mật khẩu sai được
chấp nhận), nhưng thiếu: (i) R6(a) với **bộ dữ liệu R3** (bộ R3 do vai B1 làm ở đợt 1, chưa có) — đã dùng bộ
R6 riêng 60/30/20; (ii) cặp nền tảng thật Android/iOS/macOS/Windows ↔ PWA (chưa có ứng dụng native); (iii) bản giải
mã độc lập bằng Python của **bên không tham gia xây dựng đặc tả** (bắt buộc vì O-7 chọn JavaScript); (iv) PWA
(b) chạy trên fake-indexeddb, không phải trình duyệt.

## P và S liên quan

| Mã | Kết quả | Kết luận |
|---|---|---|
| P3 | tải đầu 351,9 KB gzip (360×780, sau khi rebase lên 0.1.2) so mốc 351,6 KB của 0.1.1: **+0,3 KB** (tách mã + hàm `storage` nhỏ) | ĐẠT (< 500 KB) |
| P6 (phần mật mã) | 2 tệp tải lười (`store-*.js`, `backup-crypto-*.js`) = 21,7 KB gzip; 0 tệp trong lần tải đầu và trong precache; build dừng nếu vi phạm | ĐẠT phần mật mã; phần tưởng niệm là đợt 4 |
| P8 | Argon2id 64 MiB/t3/p4, **chỉ đo tham khảo** trên máy dựng: Chromium headless xuất 2,6 s, nhập 2,5 s (gồm cả ghi kho); Node 1,1 s cho riêng KDF | CHƯA ĐỦ BẰNG CHỨNG — cần điện thoại Android tầm trung, iPhone, máy tính (trung vị 5 lần) |
| S1/S2 | `core/` không đổi; `lvhv-store`, `lvhv-backup` có `#![forbid(unsafe_code)]` (CI kiểm) | ĐẠT |
| S2b | CSP `pwa/dist/index.html` giống từng byte 0.1.1 (kiểm thử khói); 0 vi phạm CSP khi chạy Argon2id/XChaCha trong Chromium; không có WebAssembly | ĐẠT |
| S4' (phần lưu trữ) | dữ liệu chỉ ở IndexedDB `lvhv`; `localStorage` chỉ `lvhv.ui`; lần tải đầu không tạo IndexedDB | ĐẠT phần đã đo; kịch bản R7 đầy đủ là đợt 3 |
| S8 | chỉ API của `argon2`, `chacha20poly1305`, `zeroize`, `getrandom`, `@noble/ciphers`, `@noble/hashes`, `crypto.getRandomValues`, WebCrypto SHA-256 (toàn vẹn ảnh); 0 hàm mật mã tự viết | ĐẠT theo rà mã của người viết — cần người duyệt B9 xác nhận |
| S11 | ghim chính xác (`=x.y.z`, npm không `^`/`~`, CI kiểm); `npm audit --omit=dev` 0 cảnh báo (cục bộ); `cargo deny check advisories bans` trên CI | xem kết quả job `deps` của PR |
| S12 | `m_cost_kib` = 4 GiB bị từ chối trước Argon2; JSON 10 000 cấp bị từ chối trước khi phân tích; 10 000 ảnh rỗng bị từ chối | ĐẠT phần logic; **bộ nhớ đỉnh < 200 MB chưa đo** → CHƯA ĐỦ BẰNG CHỨNG cho ngưỡng bộ nhớ |

## Rủi ro mở ghi thêm

- R-CRY-1 (O-8): cả hai bản Argon2id (RustCrypto `argon2` 0.6.0, `@noble/hashes` 2.4.0) **không** có kiểm toán độc
  lập; XChaCha20-Poly1305 chỉ có kiểm toán cho bản cũ hơn (03 Bổ sung B1.2). Bù: vector chuẩn + kiểm chéo
  từng byte giữa hai bản cài đặt độc lập.
- Tải lười cần mạng ở lần đầu mở Sao lưu sau mỗi lần cập nhật (tệp không nằm trong precache, theo P6). Ngoại
  tuyến hoàn toàn từ lúc cài → chưa mở được Sao lưu cho đến khi có mạng một lần. Đợt 3 cân nhắc tải nền sau
  lần tải đầu (vẫn đạt P6 nếu không nằm trong danh sách yêu cầu của lần tải đầu).
