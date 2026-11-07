import GoogleSignIn
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
                .onOpenURL { _ = GIDSignIn.sharedInstance.handle($0) }
                .environmentObject(state)
                .task {
                    delegate.attach(state)
                    await state.install()
                }
                .onChange(of: scenePhase) { _, phase in
                    // Returning to the foreground re-lists: a push may have been missed or coalesced.
                    if phase == .active {
                        Task { await state.refresh() }
                        Task { await state.resyncPush() }
                    }
                }
        }
    }
}

/// APNs token delivery and push receipt. The push carries only ApprovalPushHint.notification_id
/// with generic alert text; nothing in it is displayed as the request — the app re-lists, and a
/// tapped alert's locator only chooses which pending request is shown first.
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
        Task { @MainActor in state?.record(.apnsRegistration, .failed("APNs registration failed: \(error.localizedDescription)")) }
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

    // The UN* arguments are not Sendable: these callbacks are nonisolated so they never cross the
    // main actor, and hop with at most a plain String (the tapped alert's locator).
    //
    // UIKit requires the completion on the main thread, and promptly: the async forms of these
    // callbacks complete wherever the nonisolated function returns -- off the main thread -- and only
    // after the awaited re-list, and UIKit aborted the app on a tapped alert (SIGABRT in
    // -[UIApplication _updateSnapshotAndStateRestorationWithAction:windowScene:], 2026-11-04). So each
    // completes on the main actor before the re-list starts, and the re-list runs after.
    nonisolated func userNotificationCenter(_ center: UNUserNotificationCenter,
                                            willPresent notification: UNNotification,
                                            withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void) {
        let done = MainActorCompletion(completionHandler)
        Task { @MainActor in
            done.call([.banner, .sound])
            _ = await self.state?.refresh()
        }
    }

    /// The operator tapped the alert: the inbox opens the request the alert named, and re-lists.
    /// The hint is read here as plain text (Sendable) and only orders the list (AppState.front).
    nonisolated func userNotificationCenter(_ center: UNUserNotificationCenter,
                                            didReceive response: UNNotificationResponse,
                                            withCompletionHandler completionHandler: @escaping () -> Void) {
        let locator = try? WireDecode.pushHint(response.notification.request.content.userInfo).notification_id
        let done = MainActorCompletion<Void>({ _ in completionHandler() })
        Task { @MainActor in
            self.state?.focused = locator
            done.call(())
            _ = await self.state?.refresh()
        }
    }
}

/// A UserNotifications completion handler, carried to the main actor and called there exactly as
/// UIKit requires. The framework's block is not annotated Sendable; it is only ever invoked on the
/// main actor, once, which is what makes moving it there sound.
private struct MainActorCompletion<Value>: @unchecked Sendable {
    private let body: (Value) -> Void
    init(_ body: @escaping (Value) -> Void) { self.body = body }
    @MainActor func call(_ value: Value) { body(value) }
}
