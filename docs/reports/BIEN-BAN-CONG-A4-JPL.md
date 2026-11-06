# Biên bản cổng A4 — đổi phương pháp mặc định sang `Jpl` (JPL DE441)

Ngày: 2026-11-07. Căn cứ: docs/02 "Bổ sung A4" (khóa 2026-11-07, trước khi viết mã). Đặc tả: docs/03 §17.

**Ghi rõ:** Bổ sung A4 được chủ dự án quyết định **sau khi đã thấy kết quả** các kiểm định trước (những ngày lệch còn
lại giữa `Precise` và bên kiểm định đều do Sóc hoặc Trung khí sát nửa đêm). Tiêu chí L14–L17 được khóa trước khi viết
mã của `Jpl`; ngưỡng không thay đổi sau đó.

## Kết luận các tiêu chí (A4 mục 4: `Jpl` chỉ thành mặc định khi L14–L17 ĐẠT và L12, P1, P3 vẫn ĐẠT)

| Mã | Kết luận | Ai làm | PR | Bằng chứng |
|---|---|---|---|---|
| L12 | **ĐẠT** | nhóm dự án | #10, #11 | `scripts/check-parity.sh`: `day_info` TypeScript ≡ Rust từng byte ở 402 132 ngày 1900–3000 cho cả `hnd`, `precise`, `jpl`; khứ hồi dương → âm → dương 1900–3000 0 lỗi (`core/tests/range1900.rs::l12_roundtrip_1900_3000`) |
| L14 | **ĐẠT** | bên kiểm định độc lập | #9 | `docs/reports/BAO-CAO-KIEM-DINH-JPL.md`: 40 090/40 090 sự kiện, chênh lớn nhất 0,0172 s (ngưỡng 2 s) |
| L15 | **ĐẠT** | bên kiểm định độc lập | #9 | cùng báo cáo: 0 sai lệch ở 402 132 ngày, 8 trường; 0 ngày "không phân định được" |
| L16 | **ĐẠT** | nhóm dự án | #10 | `docs/reports/BAO-CAO-JPL-L16.md`: 1 238 ngày khác `Precise` trong 107 đoạn (2077–2999), 0 đoạn "nguyên nhân khác"; 1900-01-01 → 2077-11-14 không ngày nào khác |
| L17 | **ĐẠT** | cả hai bên | #9, #10 | Tết 1900–1999: 0 sai lệch theo cách chấm A3; bảng 1900–1999 của Hồ Ngọc Đức 1237/1237 tháng (hai báo cáo trên) |
| P1 | **ĐẠT** | nhóm dự án | #10, #11 | `lvhv-dump --method jpl bench` (bản release, Linux x86_64, 2 vCPU): `day_info` trung bình 20–35 µs, p99 39–82 µs, lần gọi đầu (giải mã bảng) 0,32–0,50 ms (< 1 ms) |
| P3 | **ĐẠT** | nhóm dự án | #10, #11 | `pwa/test/smoke.mjs` (`metrics.json`): lần tải đầu 357,0 KB gzip (< 500 KB), đã gồm bảng ngày `Jpl` |

L13 không áp dụng cho `Jpl` (A4 mục 4); với `precise` và `hnd`, kết quả 2000–3000 vẫn giống hệt tệp chuẩn L13.

## Lựa chọn của dự án mà chủ dự án cần biết

1. **Phạm vi bảng mở rộng đến 3002-02-01** (A4 ghi đến 3001-03-31): cần để tính ngày âm cuối năm 3000, tức dải ngày
   đã khóa của L15. Không tiêu chí nào thay đổi; các sự kiện thêm vào cũng được L14 kiểm.
2. **ΔT sau 2019 là đoạn nối Hermite bậc ba 2019–2800** sang parabol dài hạn của Stephenson, Morrison và Hohenkerk
   (2016). Các tác giả không công bố quy tắc nối; toàn văn bài bổ sung 2021 không truy cập được, nên chưa loại trừ
   được việc bài đó có parabol mới.
3. **Không hiệu chỉnh gia tốc thủy triều của Mặt Trăng** (bài công bố DE441 không cho giá trị ṅ).
4. **Đính chính nguồn DE441** (docs/03 §17.2, 2026-11-07): bảng sinh từ hai tệp phần tải ở kho NAIF của JPL (đường dẫn
   ssd.jpl.nasa.gov ghi trong đặc tả bản đầu trả 404); bên kiểm định độc lập dùng tệp một khối `de441.bsp`. Cùng lời
   giải DE441; chênh giữa hai bên tối đa 0,0172 s (L14).

## Thay đổi khi đổi mặc định (bản 0.1.4)

- `DEFAULT_METHOD = Method::Jpl` (lõi Rust) và `"jpl"` (tsport, PWA, FFI, ứng dụng trên máy). `Precise` và `Hnd` vẫn
  trong mã để đối chiếu.
- Lược đồ sao lưu nhận thêm `"jpl"`; bản ghi cũ ghi `"precise"` hoặc `"hnd"` vẫn đọc được (giữ ngày âm đã lưu, quy tắc V5).
- Người dùng: ngày âm, Can Chi và tiết khí 1900–2076 không đổi; một số ngày sau 2077 khác bản trước (danh sách đầy đủ
  trong `BAO-CAO-JPL-L16.md`).
