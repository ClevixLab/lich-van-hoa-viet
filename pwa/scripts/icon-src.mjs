// Biểu tượng gốc (tự vẽ): tờ lịch với dải đỏ phía trên, mặt trời và vầng trăng cách điệu.
export function iconSvg({ maskable = false } = {}) {
  // maskable: nền phủ kín, nội dung nằm trong vùng an toàn 80%
  const s = maskable ? 0.72 : 0.86;
  const o = (512 - 512 * s) / 2;
  const bg = maskable
    ? `<rect width="512" height="512" fill="#8f1416"/>`
    : `<rect x="0" y="0" width="512" height="512" rx="112" fill="#8f1416"/>`;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
${bg}
<g transform="translate(${o} ${o}) scale(${s})">
  <rect x="64" y="72" width="384" height="400" rx="34" fill="#fbf9f6"/>
  <path d="M64 106a34 34 0 0 1 34-34h316a34 34 0 0 1 34 34v74H64z" fill="#c8161d"/>
  <g fill="#fbf9f6"><circle cx="156" cy="126" r="13"/><circle cx="256" cy="126" r="13"/><circle cx="356" cy="126" r="13"/></g>
  <circle cx="198" cy="312" r="70" fill="#c8161d"/>
  <circle cx="334" cy="312" r="62" fill="#b07a1e"/>
  <circle cx="360" cy="296" r="54" fill="#fbf9f6"/>
  <rect x="112" y="420" width="288" height="10" rx="5" fill="#e3d9d2"/>
</g>
</svg>`;
}
