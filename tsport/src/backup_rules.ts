// Quy tắc mật khẩu khi tạo sao lưu (03 §2.4) — bản đôi của Rust `lvhv_backup::password_error`, để mọi giao diện dùng
// một quy tắc (duyệt mã MA3N-24-08): ít nhất 8 điểm mã sau NFC; có ô nhập lại thì hai ô bằng nhau sau NFC.

export const MIN_PASSWORD_CODEPOINTS = 8;

/** Lỗi của mật khẩu (thông báo tiếng Việt) hoặc `null` khi hợp lệ. `confirm` = ô nhập lại (bỏ qua khi `undefined`/`null`). */
export function backupPasswordError(password: string, confirm?: string | null): string | null {
  const n = password.normalize("NFC");
  if ([...n].length < MIN_PASSWORD_CODEPOINTS) return "Mật khẩu cần ít nhất 8 ký tự";
  if (confirm !== undefined && confirm !== null && confirm.normalize("NFC") !== n) return "Hai lần nhập mật khẩu chưa giống nhau";
  return null;
}
