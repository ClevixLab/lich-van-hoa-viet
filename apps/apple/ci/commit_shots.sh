#!/usr/bin/env bash
# Ghi ảnh chụp giao diện (từ kiểm thử XCUITest) vào chính nhánh PR, thư mục apps/apple/screenshots/.
# Dùng trong .github/workflows/apple.yml (tải tạo phẩm của Actions bị chặn ở môi trường rà soát).
# Cần: HEAD_REF, RUN_NO, HEAD_SHA, RUNNER_TEMP.
set -u
SHOTS="$RUNNER_TEMP/shots"
N=$(ls "$SHOTS"/*.png 2>/dev/null | wc -l | tr -d ' ')
echo "Số ảnh: $N"
if [ "$N" = "0" ] && [ -d apps/apple/build/ios-tests.xcresult ]; then
  xcrun xcresulttool export attachments --path apps/apple/build/ios-tests.xcresult --output-path "$SHOTS" || true
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
git fetch -q origin "$HEAD_REF"
git worktree add -q "$RUNNER_TEMP/wt" "origin/$HEAD_REF"
cd "$RUNNER_TEMP/wt"
rm -rf apps/apple/screenshots
mkdir -p apps/apple/screenshots
cp "$SHOTS"/*.png apps/apple/screenshots/
git add -A apps/apple/screenshots
if git diff --cached --quiet; then exit 0; fi
git commit -q -m "Ảnh chụp giao diện Apple — CI lần $RUN_NO (${HEAD_SHA:0:7})" \
  -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01SKZXHbLz8ZHfr7rpCwfPLv"
git push -q origin "HEAD:$HEAD_REF" || echo "::warning::Nhánh đã có commit mới — bỏ qua ghi ảnh lần này"
exit 0
