// Biểu tượng nét, viết tay (không dùng bộ biểu tượng bên ngoài).
const S = 'viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"';
export const I = {
  prev: `<svg ${S}><path d="M15 5l-7 7 7 7"/></svg>`,
  next: `<svg ${S}><path d="M9 5l7 7-7 7"/></svg>`,
  sheet: `<svg ${S}><rect x="4" y="4" width="16" height="17" rx="2.5"/><path d="M4 9h16M9 2.5v3M15 2.5v3"/><path d="M10.5 13.5h3v4"/></svg>`,
  star: `<svg ${S}><path d="M12 3.5l2.5 5.2 5.6.7-4.1 3.9 1 5.6-5-2.7-5 2.7 1-5.6-4.1-3.9 5.6-.7z"/></svg>`,
  swap: `<svg ${S}><path d="M4 8h13l-3-3M20 16H7l3 3"/></svg>`,
  info: `<svg ${S}><circle cx="12" cy="12" r="8.5"/><path d="M12 11v5.5M12 7.6v.2"/></svg>`,
  sun: `<svg ${S}><circle cx="12" cy="12" r="4"/><path d="M12 2.5v2M12 19.5v2M4.6 4.6 6 6M18 18l1.4 1.4M2.5 12h2M19.5 12h2M4.6 19.4 6 18M18 6l1.4-1.4"/></svg>`,
  moon: `<svg ${S}><path d="M19.5 14.5A8 8 0 0 1 9.5 4.5a8 8 0 1 0 10 10z"/></svg>`,
  auto: `<svg ${S}><circle cx="12" cy="12" r="8.5"/><path d="M12 3.5v17a8.5 8.5 0 0 0 0-17z" fill="currentColor"/></svg>`,
  grid: `<svg ${S}><rect x="3.5" y="5" width="17" height="15" rx="2"/><path d="M3.5 10h17M8 5v-2M16 5v-2M8.5 14h1M11.5 14h1M14.5 14h1M8.5 17h1M11.5 17h1"/></svg>`,
  close: `<svg ${S}><path d="M6 6l12 12M18 6 6 18"/></svg>`,
  down: `<svg ${S}><path d="M6 9l6 6 6-6"/></svg>`,
};
