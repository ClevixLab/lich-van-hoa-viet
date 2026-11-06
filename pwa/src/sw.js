/* Service worker — Lịch và Văn hóa Việt. Sinh bởi build.mjs.
   Chiến lược: precache toàn bộ tệp của lần tải đầu khi cài, cache-first, xóa cache cũ khi kích hoạt; tệp tải
   lười (LAZY) được cache khi trang nạp lần đầu.
   Riêng version.json: luôn lấy qua mạng (cache: "no-store"), không bao giờ trả từ cache — trang dùng tệp
   này để biết có phiên bản mới hay không; mất mạng thì báo lỗi để trang hiện "đang ngoại tuyến".
   Cập nhật: bản mới chờ (waiting) đến khi người dùng đồng ý (RS-08).
   Không gọi mạng ra ngoài nguồn gốc của chính ứng dụng. */
const CACHE = "%CACHE_NAME%";
const PRECACHE = %PRECACHE%;
// Tệp tải lười (mô-đun lưu trữ và mật mã, P6): không tải khi cài; cache lần đầu trang nạp chúng để sau đó
// dùng được khi không có mạng.
const LAZY = new Set(%LAZY%.map((u) => new URL(u, self.registration.scope).pathname));
const VERSION_URL = new URL(%VERSION_JSON%, self.registration.scope).pathname;
const PREFIX = "lvhv-";

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE).then((c) => c.addAll(PRECACHE.map((u) => new Request(u, { cache: "reload" }))))
  );
  // RS-08: không tự skipWaiting(). Lần cài đầu (chưa có SW cũ) vẫn kích hoạt ngay; bản cập nhật chờ
  // đến khi người dùng bấm "Tải lại" (trang gửi "skipWaiting") hoặc mọi tab cũ đóng — trang đang mở
  // không chạy lẫn JS cũ với cache mới.
});

// Trang đồng ý cập nhật → kích hoạt bản đang chờ. Nhận cả thông điệp "skipWaiting" của bản 0.1 (trang cũ
// đang mở gửi chuỗi này khi người dùng bấm "Tải lại") và "SKIP_WAITING" / { type: "SKIP_WAITING" } từ 0.1.1.
self.addEventListener("message", (event) => {
  const d = event.data;
  if (d === "skipWaiting" || d === "SKIP_WAITING" || (d && d.type === "SKIP_WAITING")) self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys()
      .then((keys) => Promise.all(keys.filter((k) => k.startsWith(PREFIX) && k !== CACHE).map((k) => caches.delete(k))))
      .then(() => self.clients.claim())
  );
});

self.addEventListener("fetch", (event) => {
  const req = event.request;
  if (req.method !== "GET") return;
  const url = new URL(req.url);
  if (url.origin !== self.location.origin) return;
  if (url.pathname === VERSION_URL) {
    // Chỉ qua mạng, không đọc/ghi cache. Lỗi mạng → Response.error() (fetch của trang bị từ chối).
    event.respondWith(fetch(req, { cache: "no-store" }).catch(() => Response.error()));
    return;
  }
  event.respondWith(
    caches.open(CACHE).then(async (cache) => {
      if (req.mode === "navigate") {
        const shell = await cache.match("./");
        if (shell) return shell;
      }
      const hit = await cache.match(req, { ignoreSearch: true });
      if (hit) return hit;
      try {
        const res = await fetch(req);
        if (res.ok && LAZY.has(url.pathname)) await cache.put(req, res.clone());
        return res;
      } catch (e) {
        const shell = await cache.match("./");
        if (req.mode === "navigate" && shell) return shell;
        throw e;
      }
    })
  );
});
