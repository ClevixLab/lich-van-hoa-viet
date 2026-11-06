# Chọn "Lời hay ý đẹp" hằng ngày — quy tắc xác định

Mục tiêu: mỗi ngày dương lịch hiển thị đúng một câu; mọi nền tảng (Rust core, PWA, Android, desktop) cho **cùng một kết quả**; không lặp lại câu nào trong N ngày liên tiếp (N = số câu trong `cadao.json`, hiện 427).

## Quy tắc (đã chọn)

```
pos   = (JDN − 2451545) mod N        // mod luôn ≥ 0 (Euclid)
entry = phần tử có  entry.order == pos
```

- `JDN` là số ngày Julius (Julian Day Number) **nguyên** của ngày dương lịch đang xem, tính theo lịch Gregory, *theo ngày địa phương* hiển thị trên tờ lịch (không phụ thuộc múi giờ máy chủ). Lõi lịch đã có hàm JDN; dùng lại hàm đó.
- 2451545 = JDN của 2000-01-01 (mốc cố định).
- `N = len(cadao.json)`.
- Trong ngôn ngữ có `%` trả số âm (JS, C, Rust `%`), dùng `((a % N) + N) % N` hoặc `rem_euclid`.

Ví dụ: 2026-11-06 → JDN 2461351 → pos = 9806 mod 427 = 412 → câu có `order = 412`
(validate.py in ra câu này để đối chiếu chéo giữa các nền tảng).

Vì `order` là một hoán vị của 0..N−1, trong N ngày liên tiếp mỗi câu xuất hiện đúng một lần. Thứ tự đã được xáo trộn nên các câu cùng chủ đề không đứng liền nhau.

## Trường `order` được sinh thế nào

`order` được **lưu sẵn trong JSON**; ứng dụng không cần chạy lại thuật toán này. Thuật toán chỉ để kiểm chứng và tái tạo (`build/build.py`, `validate.py`):

1. Khởi tạo `perm = [0, 1, …, N−1]` (chỉ số theo thứ tự `id` cd0001…).
2. `x = 0x4C564E31` (hạt giống, "LVN1").
3. Với `i` từ `N−1` xuống `1`: `x = xorshift32(x)`; `j = x mod (i+1)`; đổi chỗ `perm[i]` ↔ `perm[j]` (Fisher–Yates).
4. Gán `entries[perm[pos]].order = pos`.

`xorshift32(x)`: `x ^= x << 13; x ^= x >> 17; x ^= x << 5;` (mọi phép tính cắt về 32 bit không dấu).

## Khi bổ sung/bớt câu (phiên bản sau)

Thay đổi N sẽ làm đổi câu của mọi ngày. Điều này chấp nhận được giữa các phiên bản dữ liệu, nhưng:
- Ghi số phiên bản dữ liệu trong bản phát hành; mọi nền tảng phải dùng cùng một tệp `cadao.json`.
- Luôn chạy lại `build/build.py` rồi `validate.py` (validate kiểm tra `order` đúng với thuật toán trên).

## Các phương án đã cân nhắc và không chọn

- `index = (JDN · 2654435761 mod 2^32) mod N` (băm Knuth): đơn giản nhưng **có lặp** trong N ngày và có câu không bao giờ xuất hiện trong một chu kỳ; bị loại.
- `index = (JDN − 2451545) mod N` trên thứ tự `id`: không lặp nhưng các câu cùng chủ đề hiện liền nhau nhiều ngày (vì dữ liệu nhóm theo chủ đề); nên dùng thêm hoán vị `order`.
