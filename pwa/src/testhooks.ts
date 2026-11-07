// Móc chỉ dành cho kiểm thử. __LVHV_TEST_HOOKS__ do build.mjs thay lúc dựng: bản dựng chính (dist/) là false nên
// esbuild bỏ hẳn phần thân dưới đây — không tên móc nào nằm trong gói phát hành (smoke.mjs kiểm điều này).
// Chỉ bản dựng kiểm thử (LVHV_TEST_HOOKS=1, vào thư mục riêng) mới có móc.
// Nơi gọi cũng phải bọc `if (__LVHV_TEST_HOOKS__)` để esbuild bỏ cả lời gọi (khai báo ở src/globals.d.ts).

/** Chờ "cổng" kiểm thử `name` nếu trang kiểm thử đặt `window.__lvhvGates[name]` (một hàm trả Promise). */
export async function testGate(name: string): Promise<void> {
  if (!__LVHV_TEST_HOOKS__) return;
  const gate = (globalThis as { __lvhvGates?: Record<string, () => Promise<void>> }).__lvhvGates?.[name];
  if (gate) await gate();
}
