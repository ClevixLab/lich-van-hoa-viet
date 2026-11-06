//! Sinh lời nhắc (02 §7.1), xoay vòng 64 thông báo chờ (02 §7.4) — hàm thuần, không I/O.
//!
//! Thời điểm biểu diễn bằng **phút UTC kể từ JDN 0** (`i64`), để phép so sánh `at > now` đúng cả
//! khi thiết bị ở múi giờ có giờ mùa hè. Múi giờ thiết bị là `Zone` (độ lệch ban đầu + các lần
//! chuyển); Việt Nam: `Zone::fixed(420)`.

use crate::cal::Cal;
use crate::model::*;
use crate::occ::{occurrences_with_tt, rule_of, OccClass, Occurrence, RemType};

/// Múi giờ thiết bị: `initial` (phút, ví dụ +420) áp dụng trước lần chuyển đầu; mỗi phần tử
/// `(t_utc, offset)` của `transitions` (tăng dần theo `t_utc`) đặt độ lệch mới từ thời điểm UTC
/// `t_utc` trở đi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Zone {
    pub initial: i32,
    pub transitions: Vec<(i64, i32)>,
}

impl Zone {
    pub fn fixed(offset_min: i32) -> Zone {
        Zone {
            initial: offset_min,
            transitions: Vec::new(),
        }
    }
    /// Asia/Ho_Chi_Minh hiện hành (+07:00, không đổi giờ mùa hè).
    pub fn vietnam() -> Zone {
        Zone::fixed(7 * 60)
    }
    /// Độ lệch tại thời điểm UTC `t`.
    pub fn offset_at(&self, t: i64) -> i32 {
        let mut off = self.initial;
        for &(tt, o) in &self.transitions {
            if t >= tt {
                off = o;
            } else {
                break;
            }
        }
        off
    }
    /// UTC → giờ đồng hồ (phút từ JDN 0).
    pub fn to_local(&self, t: i64) -> i64 {
        t + i64::from(self.offset_at(t))
    }
    /// `local_instant` (02 §7.1): giờ đồng hồ `local` (phút từ JDN 0) → UTC. Giờ lặp (lùi giờ) →
    /// lần đầu; giờ không tồn tại (bước nhảy) → dời sau đúng khoảng nhảy (tức dùng độ lệch trước
    /// lần chuyển).
    pub fn local_to_utc(&self, local: i64) -> i64 {
        // Các khoảng độ lệch: [start_i, end_i) theo UTC.
        let mut bounds: Vec<(i64, i64, i32)> = Vec::with_capacity(self.transitions.len() + 1);
        let mut start = i64::MIN;
        let mut off = self.initial;
        for &(tt, o) in &self.transitions {
            bounds.push((start, tt, off));
            start = tt;
            off = o;
        }
        bounds.push((start, i64::MAX, off));
        // Lần đầu (UTC sớm nhất) thỏa.
        let mut best: Option<i64> = None;
        for &(a, b, o) in &bounds {
            let u = local - i64::from(o);
            if u >= a && u < b && best.is_none_or(|x| u < x) {
                best = Some(u);
            }
        }
        if let Some(u) = best {
            return u;
        }
        // Khoảng trống: tìm lần chuyển mà `local` rơi vào khe, dùng độ lệch trước lần chuyển.
        for (i, &(a, b, o)) in bounds.iter().enumerate() {
            let _ = a;
            if i + 1 < bounds.len() {
                let o2 = bounds[i + 1].2;
                let gap_lo = b + i64::from(o);
                let gap_hi = b + i64::from(o2);
                if local >= gap_lo && local < gap_hi {
                    return local - i64::from(o);
                }
            }
        }
        local - i64::from(self.initial)
    }
}

/// Phút từ JDN 0 lúc 00:00 giờ đồng hồ của ngày `d` (JDN tính từ trưa; ở đây dùng JDN như số ngày).
pub fn local_minutes(d: Ymd, hhmm: u32) -> i64 {
    d.jdn() * 1440 + i64::from(hhmm)
}

/// Giờ đồng hồ (phút từ JDN 0) → `YYYY-MM-DDTHH:MM`.
pub fn fmt_local(local: i64) -> String {
    let day = local.div_euclid(1440);
    let m = local.rem_euclid(1440) as u32;
    format!("{}T{}", Ymd::from_jdn(day).iso(), fmt_hhmm(m))
}

/// `YYYY-MM-DDTHH:MM` → phút từ JDN 0.
pub fn parse_local(s: &str) -> Option<i64> {
    if s.len() != 16 || s.as_bytes()[10] != b'T' {
        return None;
    }
    Some(local_minutes(Ymd::parse(&s[..10])?, parse_hhmm(&s[11..])?))
}

/// Một lời nhắc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reminder {
    /// `notif_id` (02 §7.1).
    pub id: String,
    /// Thời điểm (phút UTC từ JDN 0).
    pub at: i64,
    /// Giờ đồng hồ thiết bị lúc nhắc.
    pub at_local: i64,
    pub rtype: RemType,
    pub occ: Occurrence,
}

/// `notif_id(rule_id, occ, k)` = `"lvhv:" + rule_id + ":" + occ.key + ":" + k` (k = `"tt"` cho
/// tiên thường).
pub fn notif_id(rule_id: &str, occ: &Occurrence, t: RemType) -> String {
    let k = match t {
        RemType::Offset(k) => k.to_string(),
        RemType::TienThuong => "tt".to_string(),
    };
    format!("lvhv:{rule_id}:{}:{k}", occ.key.text())
}

/// `reminders_for` (02 §7.1). `occ` là lần **giỗ** (không phải lần tiên thường) khi `class = Gio`.
pub fn reminders_for(
    reminders_enabled: bool,
    rule: &ReminderRule,
    occ: &Occurrence,
    now: i64,
    tz: &Zone,
) -> Vec<Reminder> {
    let mut out = Vec::new();
    if !reminders_enabled || !rule.enabled {
        return out;
    }
    let d = occ.date.jdn();
    for &k in &rule.offsets_days {
        let day = Ymd::from_jdn(d - i64::from(k));
        let local = local_minutes(day, rule.time_local);
        let t = tz.local_to_utc(local);
        if t > now {
            out.push(Reminder {
                id: notif_id(&rule.id, occ, RemType::Offset(k)),
                at: t,
                at_local: tz.to_local(t),
                rtype: RemType::Offset(k),
                occ: occ.clone(),
            });
        }
    }
    if rule.tien_thuong && occ.class == OccClass::Gio {
        let local = local_minutes(Ymd::from_jdn(d - 1), rule.time_local);
        let t = tz.local_to_utc(local);
        if t > now {
            out.push(Reminder {
                id: notif_id(&rule.id, occ, RemType::TienThuong),
                at: t,
                at_local: tz.to_local(t),
                rtype: RemType::TienThuong,
                occ: occ.clone(),
            });
        }
    }
    out
}

/// Thứ tự (at, id) của 02 §7.4.
pub fn sort_reminders(v: &mut [Reminder]) {
    v.sort_by(|a, b| {
        a.at.cmp(&b.at)
            .then_with(|| a.id.as_bytes().cmp(b.id.as_bytes()))
    });
}

/// Mọi lời nhắc có `at > now` của các lần xuất hiện có ngày dương trong `[from, to]` (đã sắp).
/// Mốc nhắc trước tối đa 60 ngày, nên một lời nhắc ở thời điểm t thuộc lần xuất hiện có ngày trong
/// `[t, t + 60 ngày]` — người gọi chọn `[from, to]` cho đủ.
pub fn all_reminders(
    cal: &Cal,
    ds: &Dataset,
    from: Ymd,
    to: Ymd,
    now: i64,
    tz: &Zone,
) -> Vec<Reminder> {
    let mut out = Vec::new();
    for o in occurrences_with_tt(cal, ds, from, to) {
        if o.class == OccClass::TienThuong {
            continue; // sinh từ lần giỗ (02 §7.1)
        }
        if let Some(rule) = rule_of(ds, &o) {
            out.extend(reminders_for(ds.reminders_enabled, rule, &o, now, tz));
        }
    }
    sort_reminders(&mut out);
    out
}

/// Dung lượng hàng đợi iOS (02 §7.4).
pub const CAP: usize = 64;
/// Cửa sổ lập lịch (02 §7.4).
pub const WINDOW_DAYS: i64 = 400;
pub const SENTINEL_ID: &str = "lvhv:sentinel";
pub const SENTINEL_TEXT: &str = "Mở Lịch Việt để tiếp tục nhận lời nhắc giỗ và sinh nhật";

/// Một yêu cầu thông báo đang chờ (mô phỏng `UNNotificationRequest`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    pub id: String,
    pub at: i64,
    /// Văn bản (để phát hiện "cùng id nhưng khác nội dung").
    pub content: String,
}

/// Kết quả `reschedule`: tập cần giữ, và các thao tác so với `pending` hiện có.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Plan {
    pub keep: Vec<Pending>,
    pub remove: Vec<String>,
    pub add: Vec<Pending>,
    /// Cùng id nhưng khác `at` hoặc nội dung.
    pub replace: Vec<Pending>,
}

/// Phần chọn của `reschedule` (02 §7.4): từ `desired` (đã sắp theo (at, id), mọi phần tử
/// `at > now` và trong cửa sổ) chọn tối đa 64 yêu cầu.
pub fn select_keep(desired: &[Pending]) -> Vec<Pending> {
    if desired.len() > CAP {
        let mut keep: Vec<Pending> = desired[..CAP - 1].to_vec();
        let last = keep[CAP - 2].at;
        keep.push(Pending {
            id: SENTINEL_ID.to_string(),
            at: last + 1,
            content: SENTINEL_TEXT.to_string(),
        });
        keep
    } else {
        desired.to_vec()
    }
}

/// `reschedule(now)` (02 §7.4) dạng thuần: `desired_all` là mọi lời nhắc tương lai (bất kỳ thứ
/// tự), `pending` là hàng đợi hiện có.
pub fn reschedule(now: i64, desired_all: &[Pending], pending: &[Pending]) -> Plan {
    let hi = now + WINDOW_DAYS * 1440;
    let mut desired: Vec<Pending> = desired_all
        .iter()
        .filter(|p| p.at > now && p.at <= hi)
        .cloned()
        .collect();
    desired.sort_by(|a, b| {
        a.at.cmp(&b.at)
            .then_with(|| a.id.as_bytes().cmp(b.id.as_bytes()))
    });
    let keep = select_keep(&desired);
    let mut plan = Plan::default();
    for p in pending {
        if !keep.iter().any(|k| k.id == p.id) {
            plan.remove.push(p.id.clone());
        }
    }
    for k in &keep {
        match pending.iter().find(|p| p.id == k.id) {
            None => plan.add.push(k.clone()),
            Some(p) if p.at != k.at || p.content != k.content => plan.replace.push(k.clone()),
            Some(_) => {}
        }
    }
    plan.keep = keep;
    plan
}

/// Áp `plan` vào hàng đợi (mô phỏng hệ điều hành): kết quả là `plan.keep` theo thứ tự (at, id).
pub fn apply_plan(pending: &[Pending], plan: &Plan) -> Vec<Pending> {
    let mut q: Vec<Pending> = pending
        .iter()
        .filter(|p| !plan.remove.contains(&p.id))
        .cloned()
        .collect();
    for r in &plan.replace {
        if let Some(x) = q.iter_mut().find(|p| p.id == r.id) {
            *x = r.clone();
        }
    }
    q.extend(plan.add.iter().cloned());
    q.sort_by(|a, b| {
        a.at.cmp(&b.at)
            .then_with(|| a.id.as_bytes().cmp(b.id.as_bytes()))
    });
    q
}

/// Chuyển lời nhắc thành yêu cầu chờ (nội dung = văn bản màn hình khóa theo `show_names`, O-9).
pub fn to_pending(cal: &Cal, r: &Reminder, show_names: bool) -> Pending {
    let t = crate::occ::notification_text(cal, &r.occ, r.rtype, show_names);
    Pending {
        id: r.id.clone(),
        at: r.at,
        content: if t.body.is_empty() {
            t.title
        } else {
            format!("{}\n{}", t.title, t.body)
        },
    }
}

/// Bộ sinh số giả ngẫu nhiên mulberry32 (32 bit; giống hệt bản TypeScript) cho mô phỏng R12.
#[derive(Debug, Clone)]
pub struct Mulberry32(pub u32);

impl Mulberry32 {
    pub fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_add(0x6D2B_79F5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        t ^ (t >> 14)
    }
}

/// Một lần mở ứng dụng trong mô phỏng R12.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimStep {
    /// Ngày (giờ đồng hồ) mở ứng dụng.
    pub day: Ymd,
    /// Số lời nhắc tương lai trong cửa sổ 400 ngày.
    pub future: usize,
    /// Số yêu cầu chờ sau `reschedule`.
    pub pending: usize,
    pub sentinel: bool,
    /// Số lời nhắc trong cửa sổ bị `reschedule` bỏ (không giữ).
    pub dropped: usize,
    /// Vi phạm bất biến (mã I1/I2/I3), rỗng nếu không có.
    pub violations: Vec<&'static str>,
}

/// Kết quả mô phỏng R12.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SimResult {
    pub steps: Vec<SimStep>,
    pub opens: usize,
    /// Thông báo đã giao (gồm thông báo nhắc mở ứng dụng).
    pub delivered: usize,
    pub sentinel_delivered: usize,
    pub max_pending: usize,
    pub violations: usize,
    /// (Thông tin, không phải bất biến) lời nhắc đã qua thời điểm mà không được giao.
    pub missed: usize,
    /// … trong đó không có thông báo nhắc mở ứng dụng nào được giao trước hoặc cùng phút.
    pub missed_without_sentinel: usize,
}

/// Mô phỏng R12 (04): thời gian trôi từ `start` đến `end` (giờ đồng hồ). Ngày đầu luôn "mở ứng
/// dụng" lúc `open_hhmm`; mỗi ngày sau mở với xác suất `p_num / p_den` (bộ sinh `Mulberry32(seed)`,
/// một số mỗi ngày khi `0 < p < 1`). Thông báo có `at ≤ t` được giao và rời hàng đợi.
///
/// Bất biến:
/// - I1 (mọi ngày): số yêu cầu chờ ≤ 64;
/// - I2 (sau mỗi `reschedule`): nếu số lời nhắc tương lai trong cửa sổ ≥ 64 thì 63 cái sớm nhất
///   (theo (at, id)) đang chờ và thông báo nhắc mở ứng dụng có `at` = `at` của cái thứ 63 + 1 phút;
/// - I3 (sau mỗi `reschedule`): mọi lời nhắc trong cửa sổ không được giữ có `at` ≥ `at` của thông
///   báo nhắc mở ứng dụng (thông báo này đứng trước hoặc cùng phút với nó).
#[allow(clippy::too_many_arguments)]
pub fn simulate(
    cal: &Cal,
    ds: &Dataset,
    tz: &Zone,
    start: Ymd,
    end: Ymd,
    open_hhmm: u32,
    p_num: u32,
    p_den: u32,
    seed: u32,
    show_names: bool,
) -> SimResult {
    let horizon_to = Ymd::from_jdn(end.jdn() + WINDOW_DAYS + 61);
    let all = all_reminders(cal, ds, start, horizon_to, i64::MIN, tz);
    let all_p: Vec<Pending> = all.iter().map(|r| to_pending(cal, r, show_names)).collect();
    let mut rng = Mulberry32(seed);
    let mut pending: Vec<Pending> = Vec::new();
    let mut res = SimResult::default();
    let mut delivered_ids: std::collections::HashSet<String> = Default::default();
    let mut sentinel_times: Vec<i64> = Vec::new();
    let t_start = tz.local_to_utc(local_minutes(start, open_hhmm));
    for j in start.jdn()..=end.jdn() {
        let day = Ymd::from_jdn(j);
        let t_open = tz.local_to_utc(local_minutes(day, open_hhmm));
        pending.retain(|p| {
            if p.at <= t_open {
                res.delivered += 1;
                if p.id == SENTINEL_ID {
                    res.sentinel_delivered += 1;
                    sentinel_times.push(p.at);
                } else {
                    delivered_ids.insert(p.id.clone());
                }
                false
            } else {
                true
            }
        });
        let opened = if j == start.jdn() {
            true
        } else if p_num == 0 {
            false
        } else if p_num >= p_den {
            true
        } else {
            let r = rng.next_u32();
            u64::from(r) * u64::from(p_den) < u64::from(p_num) << 32
        };
        if opened {
            res.opens += 1;
            let plan = reschedule(t_open, &all_p, &pending);
            pending = apply_plan(&pending, &plan);
            let hi = t_open + WINDOW_DAYS * 1440;
            let fut: Vec<&Pending> = all_p
                .iter()
                .filter(|p| p.at > t_open && p.at <= hi)
                .collect();
            let sentinel = pending.iter().find(|p| p.id == SENTINEL_ID).map(|p| p.at);
            let mut v = Vec::new();
            if fut.len() >= CAP {
                let ok63 = fut[..CAP - 1]
                    .iter()
                    .all(|f| pending.iter().any(|p| p.id == f.id && p.at == f.at));
                let want_s = if fut.len() > CAP {
                    Some(fut[CAP - 2].at + 1)
                } else {
                    None
                };
                if !ok63 || sentinel != want_s {
                    v.push("I2");
                }
            }
            let kept = |id: &str| pending.iter().any(|p| p.id == id);
            let dropped: Vec<&&Pending> = fut.iter().filter(|f| !kept(&f.id)).collect();
            if dropped.iter().any(|f| sentinel.is_none_or(|s| f.at < s)) {
                v.push("I3");
            }
            if pending.len() > CAP {
                v.push("I1");
            }
            res.violations += v.len();
            res.max_pending = res.max_pending.max(pending.len());
            res.steps.push(SimStep {
                day,
                future: fut.len(),
                pending: pending.len(),
                sentinel: sentinel.is_some(),
                dropped: dropped.len(),
                violations: v,
            });
        } else if pending.len() > CAP {
            res.violations += 1;
        }
    }
    let t_end = tz.local_to_utc(local_minutes(end, open_hhmm));
    for p in all_p.iter().filter(|p| p.at > t_start && p.at <= t_end) {
        if !delivered_ids.contains(&p.id) {
            res.missed += 1;
            if !sentinel_times.iter().any(|&s| s <= p.at) {
                res.missed_without_sentinel += 1;
            }
        }
    }
    res
}
