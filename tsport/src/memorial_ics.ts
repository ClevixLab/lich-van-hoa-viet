// Port of memorial/src/ics.rs — .ics export (02 §8, RFC 5545).

import { isValidSolar } from "./jd.ts";
import type { Cal } from "./memorial_cal.ts";
import { type Dataset, type ReminderRule, type Ymd, iso, ymd, ymdFromJdn, ymdJdn } from "./memorial_model.ts";
import { type OccClass, type Occurrence, type RemType, description, keyText, occurrencesWithTt, reminderTextNamed, ruleOf, uidKind } from "./memorial_occ.ts";

export const PRODID = "-//Clevix Lab//Lich va Van hoa Viet 0.2.0//VI";
export const CALNAME = "Ngày giỗ và sinh nhật";
export const UID_DOMAIN = "lich.clevix.vn";

export interface IcsOptions { today: Ymd; dtstamp: string; only_ids: string[] | null }

export function tenYearsEnd(today: Ymd): Ymd {
  const y = today.y + 10;
  return isValidSolar(y, today.m, today.d) ? ymd(y, today.m, today.d) : ymd(y, 3, 1);
}

export const fileName = (today: Ymd): string => `LichVanHoaViet-ngay-gio-${iso(today)}.ics`;

export function escapeText(s: string): string {
  let o = "";
  for (const c of s) {
    if (c === "\\") o += "\\\\";
    else if (c === ";") o += "\\;";
    else if (c === ",") o += "\\,";
    else if (c === "\n") o += "\\n";
    else if (c !== "\r") o += c;
  }
  return o;
}

function utf8Len(cp: number): number {
  return cp < 0x80 ? 1 : cp < 0x800 ? 2 : cp < 0x10000 ? 3 : 4;
}

export function foldInto(out: string[], line: string): void {
  let used = 0;
  let cur = "";
  for (const c of line) {
    const n = utf8Len(c.codePointAt(0) as number);
    if (used + n > 75) {
      cur += "\r\n ";
      used = 1;
    }
    cur += c;
    used += n;
  }
  out.push(cur + "\r\n");
}

export function trigger(k: number, timeLocal: number): string {
  const dur = timeLocal - k * 1440;
  if (dur === 0) return "PT0S";
  const a = Math.abs(dur);
  const d = Math.floor(a / 1440), h = Math.floor((a % 1440) / 60), m = a % 60;
  let s = dur < 0 ? "-P" : "P";
  if (d > 0) s += `${d}D`;
  if (h > 0 || m > 0) {
    s += "T";
    if (h > 0) s += `${h}H`;
    if (m > 0) s += `${m}M`;
  }
  return s;
}

const p2 = (n: number) => String(n).padStart(2, "0");
const icsDate = (d: Ymd) => `${String(d.y).padStart(4, "0")}${p2(d.m)}${p2(d.d)}`;

export const uid = (occ: Occurrence): string => `${occ.entity_id}-${uidKind(occ.class)}-${keyText(occ.key)}@${UID_DOMAIN}`;

function categories(c: OccClass): string {
  if (c === "gio" || c === "tien_thuong") return "Ngày giỗ";
  if (c === "sinh_nhat" || c === "sinh_nhat_am") return "Sinh nhật";
  return "Sự kiện";
}

export function icsOccurrences(cal: Cal, ds: Dataset, opt: IcsOptions): [Occurrence, ReminderRule | null][] {
  const to = ymdFromJdn(ymdJdn(tenYearsEnd(opt.today)) - 1);
  const out: [Occurrence, ReminderRule | null][] = [];
  for (const o of occurrencesWithTt(cal, ds, opt.today, to)) {
    const rule = ruleOf(ds, o);
    const selected = opt.only_ids !== null ? opt.only_ids.includes(o.entity_id) : rule !== null && rule.enabled;
    if (selected) out.push([o, rule]);
  }
  return out;
}

export function generateIcs(cal: Cal, ds: Dataset, opt: IcsOptions): string {
  const out: string[] = [];
  const l = (s: string) => foldInto(out, s);
  for (const s of [
    "BEGIN:VCALENDAR", "VERSION:2.0", `PRODID:${PRODID}`, "CALSCALE:GREGORIAN", "METHOD:PUBLISH",
    `X-WR-CALNAME:${escapeText(CALNAME)}`, "X-WR-TIMEZONE:Asia/Ho_Chi_Minh",
    "BEGIN:VTIMEZONE", "TZID:Asia/Ho_Chi_Minh", "BEGIN:STANDARD", "DTSTART:19700101T000000",
    "TZOFFSETFROM:+0700", "TZOFFSETTO:+0700", "TZNAME:+07", "END:STANDARD", "END:VTIMEZONE",
  ]) l(s);
  for (const [o, rule] of icsOccurrences(cal, ds, opt)) {
    l("BEGIN:VEVENT");
    l(`UID:${uid(o)}`);
    l(`DTSTAMP:${opt.dtstamp}`);
    l(`SEQUENCE:${o.rev}`);
    l(`DTSTART;VALUE=DATE:${icsDate(o.date)}`);
    l(`DTEND;VALUE=DATE:${icsDate(ymdFromJdn(ymdJdn(o.date) + 1))}`);
    l(`SUMMARY:${escapeText(o.headline)}`);
    const desc = description(cal, ds, o);
    if (desc !== "") l(`DESCRIPTION:${escapeText(desc)}`);
    l("TRANSP:TRANSPARENT");
    l(`CATEGORIES:${escapeText(categories(o.class))}`);
    if (rule !== null && rule.enabled) {
      const alarms: [number, RemType][] = o.class === "tien_thuong"
        ? [[0, "tt"]]
        : rule.offsets_days.slice().reverse().map((k) => [k, k] as [number, RemType]);
      for (const [k, t] of alarms) {
        const txt = reminderTextNamed(cal, o, t);
        l("BEGIN:VALARM");
        l("ACTION:DISPLAY");
        l(`DESCRIPTION:${escapeText(txt.title)}`);
        l(`TRIGGER;RELATED=START:${trigger(k, rule.time_local)}`);
        l("END:VALARM");
      }
    }
    l("END:VEVENT");
  }
  l("END:VCALENDAR");
  return out.join("");
}
