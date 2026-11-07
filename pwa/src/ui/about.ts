// Giới thiệu + trang "Nguồn quy tắc" (Ngọc Hạp), cả hai dựng từ nội dung sinh lúc build.
import { IS_MOCK, MAX_YEAR, METHOD, MIN_YEAR } from "../engine";
import { TZ_NOTE } from "./convert";
import {
  APP_VERSION, CADAO, CADAO_ATTRIBUTION, CADAO_IS_MOCK, RULE_SOURCES, RULE_SOURCES_INTRO, RULE_SOURCES_LIMITS,
} from "../gen/content";
import { append, clear, h } from "../dom";
import { applyUpdate, checkForUpdate, type CheckResult } from "../update";

/** Ghi chú nguồn ngắn, hiện cạnh phần xem ngày. */
export const NGOCHAP_NOTE = "Theo Ngọc Hạp Thông Thư; tham khảo";
export const RULES_HREF = "#/nguon-quy-tac";

/** Câu mô tả phương pháp âm lịch đang dùng (hằng sinh lúc build, giống mặc định của lõi Rust). */
const METHOD_TEXT: Record<string, string> = {
  jpl: "Ngày âm lịch tính từ lịch thiên văn JPL DE441 của NASA JPL (thời điểm trăng mới và tiết khí), với ΔT theo Morrison và Stephenson, rồi quy ra ngày theo múi giờ của từng giai đoạn nêu trên. Với những năm rất xa, kết quả kém chắc chắn hơn vì không ai dự báo chính xác được tốc độ quay của Trái Đất (ΔT). Sau năm 2077, vài ngày có trăng mới hoặc tiết khí rơi sát nửa đêm có thể lệch một ngày so với các lịch khác.",
  precise: "Ngày âm lịch tính theo mô hình thiên văn chính xác (Meeus): thời điểm trăng mới và vị trí Mặt Trời được tính đầy đủ, rồi quy ra ngày theo múi giờ của từng giai đoạn nêu trên. Thuật toán Hồ Ngọc Đức, phổ biến trong nhiều lịch Việt, vẫn được giữ trong ứng dụng để đối chiếu; hai cách cho cùng kết quả ở gần như mọi ngày.",
  hnd: "Ngày âm lịch tính theo thuật toán Hồ Ngọc Đức (phổ biến trong các lịch Việt), quy ra ngày theo múi giờ của từng giai đoạn nêu trên.",
};

/** Khối "Phiên bản đang dùng" + nút "Kiểm tra cập nhật". */
function versionPanel(): HTMLElement {
  const out = h("div", { class: "upd-out", role: "status", "aria-live": "polite" });
  const btn = h("button", { type: "button", class: "todaybtn", id: "check-update" }, "Kiểm tra cập nhật");
  const show = (r: CheckResult) => {
    clear(out);
    if (r.status === "latest") out.append(h("p", null, "Bạn đang dùng phiên bản mới nhất."));
    else if (r.status === "offline") out.append(h("p", { class: "warn" }, "Không kiểm tra được (đang ngoại tuyến)."));
    else {
      const go = h("button", { type: "button", class: "todaybtn", id: "apply-update", onclick: () => {
        go.setAttribute("disabled", ""); go.textContent = "Đang cập nhật…"; void applyUpdate();
      } }, "Cập nhật ngay");
      append(out, [
        h("p", { class: "upd-new" }, r.version ? `Có phiên bản mới ${r.version}` : "Có bản cập nhật mới của ứng dụng"),
        r.notes.length ? h("ul", { class: "plain dots upd-notes" }, ...r.notes.map((t) => h("li", null, t))) : null,
        go]);
    }
  };
  btn.addEventListener("click", async () => {
    btn.setAttribute("disabled", "");
    clear(out);
    out.append(h("p", null, "Đang kiểm tra…"));
    try { show(await checkForUpdate()); } finally { btn.removeAttribute("disabled"); }
  });
  return h("section", { class: "panel prose", id: "phien-ban" },
    h("h2", null, "Phiên bản"),
    h("p", { class: "cur-ver" }, `Phiên bản đang dùng: ${APP_VERSION}`),
    h("p", null, btn),
    out);
}

export function renderAbout(root: HTMLElement): void {
  clear(root);
  root.append(
    h("header", { class: "pagehead about-head" },
      h("img", { src: "icons/icon.svg", alt: "", width: "72", height: "72", class: "about-ico" }),
      h("div", null,
        h("h1", null, "Lịch và Văn hóa Việt"),
        h("p", { class: "by" },
          h("img", { src: "icons/clevix-logo-72.png", srcset: "icons/clevix-logo-72.png 1x, icons/clevix-logo-144.png 2x", alt: "Logo Clevix", width: "37", height: "24", class: "by-logo" }),
          "Clevix Lab"),
        h("p", { class: "ver" }, `Phiên bản ${APP_VERSION}`))),
    versionPanel(),
    h("section", { class: "panel prose", id: "ve-ung-dung" },
      h("h2", null, "Về ứng dụng"),
      h("p", { class: "lead" }, "Lịch và Văn hóa Việt là cuốn lịch vạn niên miễn phí và cũng là sổ tay gia đình: xem ngày âm dương, chọn ngày tốt, và không bao giờ quên ngày giỗ ông bà, cha mẹ hay sinh nhật người thân. Không quảng cáo, không cần tài khoản, dữ liệu chỉ nằm trên máy của bạn."),
      h("h3", null, "Ứng dụng giúp bạn"),
      h("ul", { class: "plain dots" },
        h("li", null, h("strong", null, "Tờ lịch mỗi ngày"), ` từ năm ${MIN_YEAR} đến năm ${MAX_YEAR}: ngày âm, Can Chi, tiết khí, giờ hoàng đạo và một câu ca dao, tục ngữ.`),
        h("li", null, h("strong", null, "Chọn ngày tốt"), " cho việc lớn: sao tốt, sao xấu, việc nên làm và nên tránh, theo tuổi của bạn."),
        h("li", null, h("strong", null, "Đổi ngày"), " âm sang dương và ngược lại, kể cả tháng nhuận."),
        h("li", null, h("strong", null, "Người thân"), " — ghi lại người đã khuất và người đang sống trong gia đình. Ngày giỗ được tự tính theo âm lịch mỗi năm (giỗ đầu, giỗ hết, giỗ thường), cùng sinh nhật dương lịch, sinh nhật âm lịch và những dịp riêng như tốt nghiệp, ngày cưới."),
        h("li", null, h("strong", null, "Nhắc trước"), " 1, 3, 5, 10, 15 ngày hoặc số ngày bạn tự chọn, để kịp sắm lễ, báo anh em họ hàng."),
        h("li", null, h("strong", null, "Xem cả nhà có việc gì"), " trong tuần, tháng, quý hay cả năm; lọc riêng người đã mất hoặc người đang sống."),
        h("li", null, h("strong", null, "Trang tưởng niệm ngày giỗ"), ": bia ghi tên, năm sinh năm mất theo Can Chi, ảnh chân dung, bát hương ba nén và đôi nến. Gia đình có thể thêm ảnh mâm lễ của chính nhà mình."),
        h("li", null, h("strong", null, "Thông tin báo tử"), " khi một người thân qua đời: ai báo tin, mất ở đâu, ghi chú, kèm ảnh tin báo tử hoặc bản chụp giấy chứng tử.")),
      h("h3", null, "Bắt đầu trong một phút"),
      h("ol", { class: "steps" },
        h("li", null, "Mở mục ", h("strong", null, "Người thân"), ", chọn ", h("strong", null, "Hồ sơ"), ", rồi bấm “+ Thêm người đã mất” hoặc “+ Thêm người đang sống”."),
        h("li", null, "Nhập họ tên, cách gọi (Ông, Bà, Bố, Mẹ…) và quan hệ (ví dụ “Ông nội”). Với người đã mất, nhập ngày mất theo dương lịch hoặc âm lịch — ứng dụng tự đổi và tính ngày giỗ hằng năm."),
        h("li", null, "Chọn nhắc trước mấy ngày, bấm “Lưu”."),
        h("li", null, "Mở ", h("strong", null, "Sắp tới"), " để xem các ngày giỗ, sinh nhật gần nhất; mở ", h("strong", null, "Theo kỳ"), " để xem theo tuần, tháng, quý, năm."),
        h("li", null, "Đúng ngày giỗ, bấm “Mở trang tưởng niệm” để thắp nén hương tưởng nhớ."),
        h("li", null, "Vào ", h("strong", null, "Sao lưu"), " để cất một bản sao có mật khẩu, hoặc xuất tệp lịch (.ics) sang ứng dụng Lịch của điện thoại.")),
      h("h3", null, "Mẹo để không lỡ ngày nào"),
      h("ul", { class: "plain dots" },
        h("li", null, "Trên bản web, lời nhắc hiện ở đầu trang khi bạn mở ứng dụng; ứng dụng web không tự báo được khi đang đóng. Muốn điện thoại tự báo, hãy xuất tệp lịch (.ics) ở mục Sao lưu và mở bằng ứng dụng Lịch của máy."),
        h("li", null, "Hãy “Thêm vào Màn hình chính” (iPhone: nút Chia sẻ › Thêm vào MH chính; Android: menu trình duyệt › Cài đặt ứng dụng) để dùng như một ứng dụng và để trình duyệt giữ dữ liệu lâu dài."),
        h("li", null, "Thỉnh thoảng tạo tệp sao lưu và cất ở nơi an toàn. Tệp này mở được trên máy khác, trình duyệt khác, và trên bản Android, iPhone, Windows sắp ra mắt."))),
    h("section", { class: "panel prose", id: "cach-tinh-lich" },
      h("h2", null, "Cách tính lịch"),
      h("p", null, "Từ năm 1968, âm lịch tính theo múi giờ UTC+7 (giờ Hà Nội)."),
      h("p", { class: "tz-note" }, TZ_NOTE),
      h("p", { class: "method-note" }, METHOD_TEXT[METHOD] ?? `Ngày âm lịch tính theo phương pháp ${METHOD}.`),
      h("p", null, "Phần xem ngày (sao tốt, sao xấu, việc nên làm và nên tránh, trực, hoàng đạo, hắc đạo) được biên soạn theo quan niệm truyền thống trong sách Ngọc Hạp Thông Thư. Đây là nét văn hóa dân gian, chỉ có giá trị tham khảo; những quyết định quan trọng xin cân nhắc thêm hoàn cảnh thực tế. Điểm số là quy ước của ứng dụng, không có trong sách."),
      h("p", null, h("a", { href: RULES_HREF, class: "textlink" }, "Nguồn quy tắc xem ngày")),
      IS_MOCK ? h("p", { class: "warn" }, "Bản này đang dùng dữ liệu lịch giả lập (MOCK) để thử giao diện. Phần xem ngày chưa phải kết quả chính thức.") : null),
    h("section", { class: "panel prose" },
      h("h2", null, "Quyền riêng tư"),
      h("p", null, "Ứng dụng không thu thập dữ liệu, không có tài khoản, không quảng cáo và không gửi thông tin đi đâu. Mọi tính toán diễn ra ngay trên máy của bạn. Máy chủ lưu trữ trang (GitHub Pages) có thể ghi nhật ký truy cập thông thường (địa chỉ IP, loại trình duyệt) như mọi trang web."),
      h("p", null, "Hồ sơ người thân, ảnh, thông tin báo tử và tệp đính kèm chỉ được lưu trong trình duyệt trên máy này. Ảnh được tự bỏ thông tin vị trí và máy chụp. Tệp sao lưu được mã hóa bằng mật khẩu do bạn đặt; ứng dụng không giữ mật khẩu đó."),
      h("p", null, "Dữ liệu vẫn còn khi bạn đóng ứng dụng hay tắt máy. Dữ liệu sẽ mất nếu bạn xóa dữ liệu duyệt web của trang này hoặc dùng chế độ ẩn danh; trên iPhone, nếu chưa thêm vào Màn hình chính, Safari có thể tự xóa sau một thời gian không mở. Vì vậy hãy sao lưu định kỳ."),
      h("p", null, "Sau lần mở đầu tiên, ứng dụng hoạt động cả khi không có mạng. Năm sinh nhập ở mục Chọn ngày tốt chỉ dùng trong lúc tính, không được lưu lại.")),
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
