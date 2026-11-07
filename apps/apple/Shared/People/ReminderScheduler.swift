// Thông báo cục bộ (UNUserNotificationCenter) — xoay vòng 64 yêu cầu chờ theo 02 §7.4.
// Kế hoạch (giữ / gỡ / thêm / thay, kể cả thông báo nhắc mở ứng dụng "lvhv:sentinel") do lõi tính
// (`LvhvStore.reschedule_json`); tệp này chỉ đọc yêu cầu đang chờ, gọi lõi và áp kế hoạch.
// Gọi khi: mở ứng dụng / trở lại tiền cảnh, lưu dữ liệu, nhập sao lưu, đổi múi giờ, chạm vào thông báo.
import Foundation
import UserNotifications

/// Một yêu cầu đang chờ của hệ điều hành (chỉ của ứng dụng này, id "lvhv:…").
struct PendingNotif: Equatable {
    let id: String
    let atUnix: Int64
    let title: String
    let body: String
    /// Nội dung để lõi so sánh: `title` hoặc `title + "\n" + body` (như `to_pending` của lõi).
    var content: String { body.isEmpty ? title : "\(title)\n\(body)" }
}

/// Một thông báo cần đặt (từ kế hoạch của lõi).
struct PlannedNotif: Decodable, Equatable {
    let id: String
    let at_unix_s: Int64
    let fire_local: String
    let title: String
    let body: String
    let content: String
}

struct ReschedulePlan: Decodable, Equatable {
    let keep: [PlannedNotif]
    let remove: [String]
    let add: [PlannedNotif]
    let replace: [PlannedNotif]
}

enum NotifAuth: Equatable { case notDetermined, denied, authorized }

/// Lớp mỏng quanh UNUserNotificationCenter để kiểm thử được bằng bản giả.
protocol NotificationCenterLike: AnyObject {
    func authorization() async -> NotifAuth
    func requestAuthorization() async -> Bool
    func pending() async -> [PendingNotif]
    func remove(ids: [String])
    func add(_ p: PlannedNotif) async throws
}

final class SystemNotificationCenter: NotificationCenterLike {
    private var c: UNUserNotificationCenter { UNUserNotificationCenter.current() }

    func authorization() async -> NotifAuth {
        let s = await c.notificationSettings()
        switch s.authorizationStatus {
        case .notDetermined: return .notDetermined
        case .denied: return .denied
        default: return .authorized
        }
    }

    func requestAuthorization() async -> Bool {
        (try? await c.requestAuthorization(options: [.alert, .sound, .badge])) ?? false
    }

    func pending() async -> [PendingNotif] {
        let reqs = await c.pendingNotificationRequests()
        return reqs.filter { $0.identifier.hasPrefix("lvhv:") }.map { r in
            let fromInfo = (r.content.userInfo["at"] as? NSNumber)?.int64Value
            let fromTrigger = (r.trigger as? UNCalendarNotificationTrigger)?.nextTriggerDate()
                .map { Int64($0.timeIntervalSince1970) }
            return PendingNotif(id: r.identifier, atUnix: fromInfo ?? fromTrigger ?? 0,
                                title: r.content.title, body: r.content.body)
        }
    }

    func remove(ids: [String]) {
        guard !ids.isEmpty else { return }
        c.removePendingNotificationRequests(withIdentifiers: ids)
    }

    func add(_ p: PlannedNotif) async throws {
        guard let comps = ReminderScheduler.components(p.fire_local) else { return }
        let content = UNMutableNotificationContent()
        content.title = p.title
        content.body = p.body
        content.sound = .default
        content.threadIdentifier = "lvhv"
        // Chỉ thời điểm (để lần sau báo lại cho lõi) — không có tên, không có thông tin báo tử.
        content.userInfo = ["at": NSNumber(value: p.at_unix_s)]
        let trigger = UNCalendarNotificationTrigger(dateMatching: comps, repeats: false)
        try await c.add(UNNotificationRequest(identifier: p.id, content: content, trigger: trigger))
    }
}

/// Múi giờ thiết bị cho lõi: độ lệch hiện tại + các lần chuyển giờ trong 401 ngày tới.
enum DeviceZones {
    static func current(at now: Date = Date(), tz: TimeZone = .current) -> DeviceZone {
        var trans: [ZoneTransition] = []
        var t = now
        let limit = now.addingTimeInterval(401 * 86_400)
        while trans.count < 32, let n = tz.nextDaylightSavingTimeTransition(after: t), n > t, n <= limit {
            trans.append(ZoneTransition(atUnixS: Int64(n.timeIntervalSince1970),
                                        offsetMin: Int32(tz.secondsFromGMT(for: n) / 60)))
            t = n
        }
        return DeviceZone(offsetMin: Int32(tz.secondsFromGMT(for: now) / 60), transitions: trans)
    }
}

@MainActor
final class ReminderScheduler: ObservableObject {
    enum Reason { case appOpen, save, importBackup, timezone, tap, settings }

    struct Outcome: Equatable {
        var added = 0
        var removed = 0
        var replaced = 0
        var kept = 0
        var permission: NotifAuth = .notDetermined
        var error: String?
    }

    let center: NotificationCenterLike
    @Published private(set) var permission: NotifAuth = .notDetermined
    @Published private(set) var last = Outcome()
    private var running = false
    private var rerun = false

    init(center: NotificationCenterLike) { self.center = center }

    /// "2026-07-25T07:00" → (năm, tháng, ngày, giờ, phút) không gắn múi giờ (02 §7.4).
    nonisolated static func components(_ fireLocal: String) -> DateComponents? {
        let parts = fireLocal.split(separator: "T")
        guard parts.count == 2 else { return nil }
        let d = parts[0].split(separator: "-").compactMap { Int($0) }
        let h = parts[1].split(separator: ":").compactMap { Int($0) }
        guard d.count == 3, h.count == 2 else { return nil }
        var c = DateComponents()
        c.year = d[0]; c.month = d[1]; c.day = d[2]; c.hour = h[0]; c.minute = h[1]
        return c
    }

    /// `[{"id","at_unix_s","content"}]` cho `reschedule_json`.
    nonisolated static func pendingJSON(_ list: [PendingNotif]) -> String {
        let arr: [JV] = list.map { ["id": .s($0.id), "at_unix_s": .int($0.atUnix), "content": .s($0.content)] }
        return JV.array(arr).text
    }

    /// Người dùng bấm "Cho phép thông báo" trong Cài đặt nhắc.
    func requestPermission(store: LvhvStore?) async {
        if await center.authorization() == .notDetermined { _ = await center.requestAuthorization() }
        permission = await center.authorization()
        await reschedule(store: store, reason: .settings)
    }

    func refreshPermission() async {
        permission = await center.authorization()
    }

    /// Lập lịch lại. Xin quyền thông báo lần đầu khi người dùng vừa lưu dữ liệu có nhắc (không xin khi mở
    /// ứng dụng). Bị từ chối → không đặt gì; giao diện nhắc mở Cài đặt.
    @discardableResult
    func reschedule(store: LvhvStore?, reason: Reason, now: Date = Date(), zone: DeviceZone? = nil) async -> Outcome {
        guard let store else { return Outcome() }
        #if DEBUG
        if UserDefaults.standard.bool(forKey: "lvhvNoNotif") { return Outcome() }
        #endif
        if running { rerun = true; return last }
        running = true
        defer { running = false }
        var out = Outcome()
        repeat {
            rerun = false
            out = await once(store: store, reason: reason, now: now, zone: zone)
        } while rerun
        last = out
        return out
    }

    private func once(store: LvhvStore, reason: Reason, now: Date, zone: DeviceZone?) async -> Outcome {
        var out = Outcome()
        let pending = await center.pending()
        let z = zone ?? DeviceZones.current(at: now)
        let plan: ReschedulePlan
        do {
            let j = try store.rescheduleJson(nowUnixS: Int64(now.timeIntervalSince1970), zone: z,
                                             pendingJson: Self.pendingJSON(pending))
            guard let p = Core.decode(ReschedulePlan.self, j) else {
                out.error = "Không đọc được kế hoạch nhắc."
                return out
            }
            plan = p
        } catch {
            out.error = Core.message(error)
            return out
        }
        var auth = await center.authorization()
        if auth == .notDetermined && !plan.keep.isEmpty && reason != .appOpen && reason != .timezone {
            auth = await center.requestAuthorization() ? .authorized : .denied
        }
        permission = auth
        out.permission = auth
        guard auth == .authorized else {
            // Không có quyền: gỡ mọi yêu cầu cũ để lần cấp quyền sau đặt lại từ đầu.
            center.remove(ids: pending.map(\.id))
            out.removed = pending.count
            return out
        }
        center.remove(ids: plan.remove + plan.replace.map(\.id))
        out.removed = plan.remove.count
        for p in plan.add {
            do { try await center.add(p); out.added += 1 } catch { out.error = error.localizedDescription }
        }
        for p in plan.replace {
            do { try await center.add(p); out.replaced += 1 } catch { out.error = error.localizedDescription }
        }
        out.kept = plan.keep.count
        return out
    }
}

/// Đại biểu thông báo: hiện cả khi đang mở ứng dụng; chạm → mở "Người thân" và lập lịch lại.
final class NotificationDelegate: NSObject, UNUserNotificationCenterDelegate {
    static let shared = NotificationDelegate()
    static let openPeople = Notification.Name("lvhv.openPeople")

    func userNotificationCenter(_ center: UNUserNotificationCenter, willPresent notification: UNNotification,
                                withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void) {
        completionHandler([.banner, .list, .sound])
    }

    func userNotificationCenter(_ center: UNUserNotificationCenter, didReceive response: UNNotificationResponse,
                                withCompletionHandler completionHandler: @escaping () -> Void) {
        let id = response.notification.request.identifier
        DispatchQueue.main.async {
            NotificationCenter.default.post(name: Self.openPeople, object: nil)
            if id.hasPrefix("lvhv:") {
                Task { @MainActor in
                    let m = PeopleModel.shared
                    m.openIfNeeded()
                    await m.scheduler.reschedule(store: m.store, reason: .tap)
                }
            }
        }
        completionHandler()
    }
}
