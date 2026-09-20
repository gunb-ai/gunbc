import SwiftUI
import UserNotifications

@main
struct ApproveApp: App {
    @UIApplicationDelegateAdaptor(AppDelegate.self) private var delegate
    @StateObject private var state = AppState()
    @Environment(\.scenePhase) private var scenePhase

    var body: some Scene {
        WindowGroup {
            RootView()
                .environmentObject(state)
                .task {
                    delegate.attach(state)
                    await state.install()
                }
                .onChange(of: scenePhase) { _, phase in
                    // Returning to the foreground re-lists: a push may have been missed or coalesced.
                    if phase == .active { Task { await state.refresh() } }
                }
        }
    }
}

/// APNs token delivery and push receipt. The push carries only ApprovalPushHint.notification_id
/// with generic alert text; nothing in it is displayed as the request — the app re-lists.
final class AppDelegate: NSObject, UIApplicationDelegate, UNUserNotificationCenterDelegate {
    weak var state: AppState?
    /// A token that arrives before the state object is attached is kept, not dropped.
    private var bufferedToken: String?

    func application(_ application: UIApplication,
                     didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]? = nil) -> Bool {
        UNUserNotificationCenter.current().delegate = self
        return true
    }

    func application(_ application: UIApplication, didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
        let hex = deviceToken.map { String(format: "%02x", $0) }.joined()
        Task { @MainActor in
            if let state { state.tokenDelivered(hex) } else { bufferedToken = hex }
        }
    }

    @MainActor func attach(_ s: AppState) {
        state = s
        if let t = bufferedToken { s.tokenDelivered(t); bufferedToken = nil }
    }

    func application(_ application: UIApplication, didFailToRegisterForRemoteNotificationsWithError error: Error) {
        Task { @MainActor in state?.lastError = "APNs registration failed: \(error.localizedDescription)" }
    }

    /// Background delivery: the result is the real one from re-listing, never a constant.
    func application(_ application: UIApplication,
                     didReceiveRemoteNotification userInfo: [AnyHashable: Any]) async -> UIBackgroundFetchResult {
        // The hint is decoded (a push without one is not this protocol's) and selects nothing:
        // a push wakes the list.
        guard (try? WireDecode.pushHint(userInfo)) != nil, let state else { return .noData }
        switch await state.refresh() {
        case .changed: return .newData
        case .unchanged: return .noData
        case .failed: return .failed
        }
    }

    func userNotificationCenter(_ center: UNUserNotificationCenter,
                                willPresent notification: UNNotification) async -> UNNotificationPresentationOptions {
        await state?.refresh()
        return [.banner, .sound]
    }

    /// The operator tapped the alert: re-list, then the inbox shows what is pending.
    func userNotificationCenter(_ center: UNUserNotificationCenter, didReceive response: UNNotificationResponse) async {
        await state?.refresh()
    }
}
