#!/usr/bin/env bash
# Ghi ảnh chụp giao diện (từ kiểm thử XCUITest) vào chính nhánh PR, thư mục apps/apple/screenshots/[SUBDIR].
# Dùng trong .github/workflows/apple.yml (tải tạo phẩm của Actions bị chặn ở môi trường rà soát).
# Cần: HEAD_REF, RUN_NO, HEAD_SHA, RUNNER_TEMP; tùy chọn SHOTS (thư mục ảnh), SUBDIR (thư mục con), RESULT (xcresult).
set -u
SHOTS="${SHOTS:-$RUNNER_TEMP/shots}"
SUBDIR="${SUBDIR:-}"
RESULT="${RESULT:-apps/apple/build/ios-tests.xcresult}"
N=$(ls "$SHOTS"/*.png 2>/dev/null | wc -l | tr -d ' ')
echo "Số ảnh: $N"
if [ "$N" = "0" ] && [ -d "$RESULT" ]; then
  xcrun xcresulttool export attachments --path "$RESULT" --output-path "$SHOTS" || true
  N=$(ls "$SHOTS"/*.png 2>/dev/null | wc -l | tr -d ' ')
fi
[ "$N" = "0" ] && exit 0
# Thu nhỏ (ảnh gốc 3x) để kho không phình: cạnh ngang còn khoảng một nửa.
for f in "$SHOTS"/*.png; do
  W=$(sips -g pixelWidth "$f" | awk '/pixelWidth/{print $2}')
  if [ "$W" -gt 1100 ]; then sips --resampleWidth $(( W / 2 > 750 ? W / 2 : 750 )) "$f" >/dev/null; fi
done
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
WT="$RUNNER_TEMP/wt-shots-${SUBDIR:-main}"
git fetch -q origin "$HEAD_REF"
git worktree add -q "$WT" "origin/$HEAD_REF"
cd "$WT"
DEST="apps/apple/screenshots${SUBDIR:+/$SUBDIR}"
mkdir -p "$DEST"
if [ -n "$SUBDIR" ]; then rm -f "$DEST"/*.png; else rm -f apps/apple/screenshots/*.png; fi
cp "$SHOTS"/*.png "$DEST/"
git add -A apps/apple/screenshots
if git diff --cached --quiet; then exit 0; fi
git commit -q -m "Ảnh chụp giao diện Apple${SUBDIR:+ ($SUBDIR)} — CI lần $RUN_NO (${HEAD_SHA:0:7})" \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01SKZXHbLz8ZHfr7rpCwfPLv"
for i in 1 2 3; do
  if git push -q origin "HEAD:$HEAD_REF"; then exit 0; fi
  # Nhánh có commit mới: chỉ ghép khi commit mới là commit ảnh của job khác (cùng đầu mã); nếu có mã mới thì bỏ.
  git fetch -q origin "$HEAD_REF"
  if git log --format=%s "HEAD..origin/$HEAD_REF" | grep -qv "^Ảnh chụp giao diện Apple"; then
    echo "::warning::Nhánh đã có commit mã mới — bỏ qua ghi ảnh lần này"; exit 0
  fi
  git rebase -q "origin/$HEAD_REF" || { git rebase --abort; exit 0; }
done
echo "::warning::Không đẩy được ảnh sau 3 lần"
exit 0
