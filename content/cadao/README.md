# content/cadao — "Lời hay ý đẹp" (v0.1)

Bộ ca dao, tục ngữ, thành ngữ Việt Nam. Mỗi ngày tờ lịch hiển thị một câu.

| Tệp | Nội dung |
|---|---|
| `cadao.json` | Dữ liệu (427 câu) |
| `SOURCES.md` | Nguồn, giấy phép, **nội dung ghi công ứng dụng phải hiển thị**, hạn chế khi thu thập |
| `select.md` | Quy tắc chọn câu theo ngày, xác định, giống nhau trên mọi nền tảng |
| `validate.py` | Kiểm tra (C1, loại, chủ đề, trùng lặp, `order`) và in thống kê |
| `build/` | `data.txt` (nguồn biên soạn), `sources.py` (danh mục nguồn), `build.py` (sinh `cadao.json`) |

Tạo lại dữ liệu: `python3 content/cadao/build/build.py && python3 content/cadao/validate.py`

Giấy phép dữ liệu: `cadao.json` phát hành theo **CC BY-SA 4.0** (xem SOURCES.md §C). Các câu dân gian thuộc phạm vi công cộng.

## Lược đồ một mục

```json
{"id": "cd0001", "type": "ca dao" | "tục ngữ" | "thành ngữ",
 "text": "dòng 1 / dòng 2", "lines": ["dòng 1", "dòng 2"],
 "theme": "hieu_thao", "meaning": "",
 "sources": [{"title": "...", "url": "https://...", "license": "..."}],
 "order": 123}
```
- `text` = các `lines` nối bằng `" / "`. Ca dao có ≥ 2 dòng (thường là lục bát); tục ngữ/thành ngữ có 1 dòng. Khi hiển thị, nên xuống dòng theo `lines`.
- `meaning`: hiện để trống. Nếu viết thêm, phải tự viết, ≤ 25 chữ, không chép từ trang nào.
- Văn bản ở dạng Unicode NFC.

## Chính sách tuyển chọn (quyết định của chủ dự án, v0.1)

**Chỉ nhận** ca dao, tục ngữ, thành ngữ của văn học dân gian (khuyết danh, phạm vi công cộng).
**Không nhận** câu của tác giả cá nhân (danh ngôn, thơ có tác giả, kể cả thơ hiện đại "giống ca dao" lưu truyền trên mạng), không nhận bản dịch câu nói nước ngoài. Câu gốc Hán‑Việt đã thành tục ngữ trong dân gian (ví dụ "Uống nước nhớ nguồn", "Tiên học lễ, hậu học văn", "Ôn cố tri tân") được giữ vì không gắn với một tác giả.

Tờ lịch dành cho mọi lứa tuổi, nên ưu tiên chủ đề tích cực, khôn ngoan, ấm áp. 12 chủ đề: `gia_dinh`, `hieu_thao`, `hoc_tap`, `can_cu`, `ban_be`, `biet_on`, `kien_nhan`, `thien_nhien`, `nong_nghiep`, `nhan_ai`, `que_huong`, `dao_duc`.

### Danh mục loại trừ
1. **Thô tục, tình dục**, kể cả nói lóng, ẩn ý.
2. **Bạo lực** hoặc khuyến khích trừng phạt thân thể. Ví dụ đã loại: "Thương cho roi cho vọt…", "Yêu cho roi cho vọt…".
3. **Chế giễu, định kiến với nhóm người** (giới tính, vùng miền, dân tộc, khuyết tật, nghề nghiệp, tôn giáo, người nghèo, mẹ chồng/nàng dâu…). Ví dụ đã loại: các câu "Nhất sĩ nhì nông…", "Phi thương bất phú", "Con gái là con người ta…", "Trọng nam khinh nữ", các câu về thầy bói, thầy cúng, hát xướng ("Xướng ca vô loài"), câu chê người lười, câu "Đàn ông…/Đàn bà…".
4. **Mê tín có thể gây hại** hoặc kiêng kỵ không có cơ sở (ngày giờ xấu, kiêng tuổi, "Có kiêng có lành", "Làm nhà năm kim lâu…", quét nhà ngày Tết…). Tục ngữ kinh nghiệm thời tiết, nông vụ được giữ vì là tri thức dân gian quan sát được.
5. **Nhạy cảm chính trị**: quan lại, vua chúa, chiến tranh, chống đế quốc/phong kiến, so sánh chế độ. Ví dụ đã loại: "Phép vua thua lệ làng", "Con vua thì lại làm vua…".
6. **Nghĩa tiêu cực, gây tranh cãi, mỉa mai, hoài nghi, buồn thương thân phận**: ca dao "than thân", châm biếm; câu khuyên khôn lỏi ("Khôn sống mống chết", "Thật thà là cha thằng dại", "Ngu si hưởng thái bình"); câu về rượu, cờ bạc; tục ngữ dễ hiểu thành ủng hộ thờ ơ ("Cháy nhà hàng xóm bình chân như vại"); câu có nghĩa chê bai dù dùng phổ biến (ăn cháo đá bát, qua cầu rút ván…).
7. **Tình yêu đôi lứa, chuyện vợ chồng có ẩn ý** hoặc phụ thuộc ngữ cảnh. Chỉ giữ vài câu vợ chồng hòa thuận phù hợp mọi lứa tuổi ("Thuận vợ thuận chồng…", "Râu tôm nấu với ruột bầu…").
8. **Địa danh dễ gây chê bai vùng miền**, ví dụ "Sông Thao nước đục người đen…" và các câu so sánh vùng. Câu ca ngợi cảnh đẹp địa phương được giữ.
9. **Chữ chưa chắc chắn**: câu mà người biên soạn không chắc dạng phổ biến hoặc có nhiều dị bản tranh cãi thì để lại cho phiên bản sau.

### Chính tả
- Theo chính tả hiện hành, dạng lưu hành phổ biến nhất (sách giáo khoa, tuyển tập). Chuẩn hóa chính tả cũ của sách 1896–1897 (ví dụ "nhứt" → "nhất", "cữa" → "cửa"). Giữ "Thuốc đắng dã tật" (dạng phổ biến, dị bản "giã tật"). "Biển Đông" viết hoa, tên tháng âm lịch viết hoa ("tháng Giêng", "tháng Chạp").
- Chống trùng: so sánh sau khi bỏ dấu, chữ thường, bỏ dấu câu và gộp khoảng trắng (`validate.py::norm`).

## Điểm chưa chắc, cần người đọc soát (trước bản phát hành)
- Nguồn trực tuyến chỉ xác nhận **phần mở đầu** của mỗi câu (xem SOURCES.md §D). Các dòng sau của ca dao dài (ví dụ "Người ta đi cấy lấy công…", "Tháng Chạp là tháng trồng khoai…", "Trâu ơi ta bảo trâu này…") được viết theo dạng phổ biến; nên đối chiếu với tuyển tập in.
- Một vài câu có dị bản phổ biến ngang nhau: "Ăn ngay nói thật, mọi tật mọi lành"; "Học thầy học bạn, vô vạn phong lưu"; "Thứ nhất thì tu tại gia…"; "Tháng Bảy kiến đàn, đại hàn hồng thủy"; "Ơn thầy…" (đã bỏ); "Đứng bên ni đồng…" (giữ hai dòng chính).
- Một số câu chủ đề `dao_duc`/`can_cu` có giọng răn dạy nhưng không gay gắt ("Ngồi ăn không, núi cũng mòn", "Ăn không lo, của kho cũng hết"); chủ dự án có thể bỏ nếu muốn giọng hoàn toàn tích cực.
- "Tậu trâu, lấy vợ, làm nhà…" có thể bị hiểu là nói về hôn nhân như một gánh nặng; giữ vì đây là câu kinh nghiệm nông thôn phổ biến, nhưng có thể bỏ.
- Chủ đề `ban_be` (16) và `biet_on` (19) ít câu hơn các chủ đề khác; nên bổ sung ở v0.2.
