/* Service worker — Lịch và Văn hóa Việt. Sinh bởi build.mjs.
   Chiến lược: precache toàn bộ tệp khi cài, cache-first, xóa cache cũ khi kích hoạt.
   Cập nhật: bản mới chờ (waiting) đến khi người dùng đồng ý tải lại (RS-08).
   Không gọi mạng ra ngoài nguồn gốc của chính ứng dụng. */
const CACHE = "%CACHE_NAME%";
const PRECACHE = %PRECACHE%;
const PREFIX = "lvhv-";

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE).then((c) => c.addAll(PRECACHE.map((u) => new Request(u, { cache: "reload" }))))
  );
  // RS-08: không tự skipWaiting(). Lần cài đầu (chưa có SW cũ) vẫn kích hoạt ngay; bản cập nhật chờ
  // đến khi người dùng bấm "Tải lại" (trang gửi "skipWaiting") hoặc mọi tab cũ đóng — trang đang mở
  // không chạy lẫn JS cũ với cache mới.
});

self.addEventListener("message", (event) => {
  if (event.data === "skipWaiting") self.skipWaiting();
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
  event.respondWith(
    caches.open(CACHE).then(async (cache) => {
      if (req.mode === "navigate") {
        const shell = await cache.match("./");
        if (shell) return shell;
      }
      const hit = await cache.match(req, { ignoreSearch: true });
      if (hit) return hit;
      try {
        return await fetch(req);
      } catch (e) {
        const shell = await cache.match("./");
        if (req.mode === "navigate" && shell) return shell;
        throw e;
      }
    })
  );
});
