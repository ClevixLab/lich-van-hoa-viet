// Giới thiệu + trang "Nguồn quy tắc" (Ngọc Hạp), cả hai dựng từ nội dung sinh lúc build.
import { IS_MOCK, METHOD } from "../engine";
import {
  APP_VERSION, CADAO, CADAO_ATTRIBUTION, CADAO_IS_MOCK, RULE_SOURCES, RULE_SOURCES_INTRO, RULE_SOURCES_LIMITS,
} from "../gen/content";
import { clear, h } from "../dom";

/** Ghi chú nguồn ngắn, hiện cạnh phần xem ngày. */
export const NGOCHAP_NOTE = "Theo Ngọc Hạp Thông Thư; tham khảo";
export const RULES_HREF = "#/nguon-quy-tac";

const METHOD_LABEL: Record<string, string> = {
  hnd: "thuật toán Hồ Ngọc Đức (phổ biến trong các lịch Việt)",
  precise: "mô hình thiên văn chính xác hơn (Meeus đầy đủ)",
};

export function renderAbout(root: HTMLElement): void {
  clear(root);
  root.append(
    h("header", { class: "pagehead about-head" },
      h("img", { src: "icons/icon.svg", alt: "", width: "72", height: "72", class: "about-ico" }),
      h("div", null,
        h("h1", null, "Lịch và Văn hóa Việt"),
        h("p", { class: "by" }, "Clevix Lab"),
        h("p", { class: "ver" }, `Phiên bản ${APP_VERSION} (thử nghiệm nội bộ)`))),
    h("section", { class: "panel prose" },
      h("h2", null, "Về ứng dụng"),
      h("p", null, "Lịch âm dương Việt Nam theo múi giờ UTC+7, từ năm 2000 đến năm 3000: ngày âm, Can Chi, tiết khí, giờ hoàng đạo, chọn ngày tốt, và một câu ca dao, tục ngữ cho mỗi ngày."),
      h("p", null, `Ngày âm lịch tính theo ${METHOD_LABEL[METHOD] ?? METHOD}.`),
      h("p", null, "Phần xem ngày (sao tốt, sao xấu, việc nên làm và nên tránh, trực, hoàng đạo, hắc đạo) được biên soạn theo quan niệm truyền thống trong sách Ngọc Hạp Thông Thư. Đây là nét văn hóa dân gian, chỉ có giá trị tham khảo; những quyết định quan trọng xin cân nhắc thêm hoàn cảnh thực tế. Điểm số là quy ước của ứng dụng, không có trong sách."),
      h("p", null, h("a", { href: RULES_HREF, class: "textlink" }, "Nguồn quy tắc xem ngày")),
      IS_MOCK ? h("p", { class: "warn" }, "Bản này đang dùng dữ liệu lịch giả lập (MOCK) để thử giao diện. Phần xem ngày chưa phải kết quả chính thức.") : null),
    h("section", { class: "panel prose" },
      h("h2", null, "Quyền riêng tư"),
      h("p", null, "Ứng dụng không thu thập dữ liệu, không có tài khoản, không quảng cáo và không gửi thông tin đi đâu. Mọi tính toán diễn ra ngay trên máy của bạn. Máy chủ lưu trữ trang (GitHub Pages) có thể ghi nhật ký truy cập thông thường (địa chỉ IP, loại trình duyệt) như mọi trang web."),
      h("p", null, "Sau lần mở đầu tiên, ứng dụng hoạt động cả khi không có mạng. Máy chỉ ghi nhớ cỡ chữ và giao diện sáng tối mà bạn chọn."),
      h("p", null, "Năm sinh nhập ở mục Chọn ngày tốt chỉ dùng trong lúc tính, không được lưu lại.")),
    h("section", { class: "panel prose", id: "nguon-ca-dao" },
      h("h2", null, "Nguồn ca dao, tục ngữ"),
      CADAO_IS_MOCK ? h("p", { class: "warn" }, "Các câu đang hiển thị là câu mẫu (MOCK), chờ bộ nội dung có dẫn nguồn.") : null,
      h("p", null, CADAO_ATTRIBUTION),
      CADAO_IS_MOCK ? null : h("p", null, `Bộ dữ liệu gồm ${CADAO.length} câu, phát hành theo giấy phép CC BY-SA 4.0 (Creative Commons Ghi công – Chia sẻ tương tự). Wikiquote và Wikimedia không liên quan và không xác nhận ứng dụng này.`)),
    h("section", { class: "panel prose" },
      h("h2", null, "Phông chữ"),
      h("p", null, "Phông chữ LichViet Serif là bản sửa đổi (rút gọn cho chữ Việt, đổi tên) của phông Lora — bản quyền 2011 The Lora Project Authors; cấp phép SIL Open Font License 1.1 (tệp fonts/OFL.txt).")),
  );
}

export function renderRuleSources(root: HTMLElement): void {
  const main = RULE_SOURCES.filter((r) => r.code.startsWith("N"));
  const check = RULE_SOURCES.filter((r) => r.code.startsWith("F"));
  const item = (r: (typeof RULE_SOURCES)[number]) =>
    h("li", { class: "srcitem" },
      h("p", { class: "src-t" }, h("span", { class: "src-code" }, r.code), r.title),
      r.lineage ? h("p", { class: "src-m" }, r.lineage) : null,
      r.usedFor ? h("p", { class: "src-m" }, `Dùng cho: ${r.usedFor}`) : null,
      h("p", { class: "src-u" }, r.url));
  clear(root);
  const parts: Array<HTMLElement | null> = [
    h("header", { class: "pagehead" },
      h("h1", null, "Nguồn quy tắc"),
      h("p", null, `${NGOCHAP_NOTE}. Danh sách nguồn dùng để mã hóa quy tắc sao tốt, sao xấu, 12 thần và tuổi xung.`)),
    h("section", { class: "panel prose" },
      h("h2", null, "Cách dùng nguồn"),
      h("p", null, RULE_SOURCES_INTRO),
      h("p", null, "Địa chỉ trang được ghi để đối chiếu; ứng dụng không mở các trang này.")),
    h("section", { class: "panel" },
      h("h2", null, `Nguồn quy tắc (${main.length})`),
      h("ol", { class: "srclist" }, ...main.map(item))),
    check.length ? h("section", { class: "panel" },
      h("h2", null, "Trang dùng để kiểm thử (không dùng để suy quy tắc)"),
      h("ol", { class: "srclist" }, ...check.map(item))) : null,
    RULE_SOURCES_LIMITS.length ? h("section", { class: "panel prose" },
      h("h2", null, "Hạn chế"),
      h("ul", { class: "plain dots" }, ...RULE_SOURCES_LIMITS.map((t) => h("li", null, t)))) : null,
    h("p", { class: "backlink" }, h("a", { href: "#/gioi-thieu", class: "textlink" }, "Về trang Giới thiệu")),
  ];
  root.append(...parts.filter((p): p is HTMLElement => p !== null));
}
