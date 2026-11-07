import SwiftUI
import UIKit

// ── Errors: one presentation everywhere ─────────────────────────────────────────────────────
/// An error message as the app records it ("<what failed>: server refused: HTTP 503 {json}"), read
/// for display: a short title, the server's own words, and the status. The raw text is kept for
/// Details, so nothing the server said is hidden.
struct ErrorDigest {
    let title: String
    let summary: String?
    let status: Int?
    let raw: String

    init(_ raw: String) {
        self.raw = raw
        var head = raw
        var json: [String: Any]?
        if let brace = raw.firstIndex(of: "{") {
            head = String(raw[..<brace])
            json = (try? JSONSerialization.jsonObject(with: Data(raw[brace...].utf8))) as? [String: Any]
        }
        if let r = head.range(of: #"HTTP \d{3}"#, options: .regularExpression) {
            status = Int(head[r].dropFirst(5))
        } else {
            status = nil
        }
        let clauses = head.components(separatedBy: ": ").map { $0.trimmingCharacters(in: .whitespaces) }.filter { !$0.isEmpty }
        let lead = clauses.first ?? ""
        if let outcome = json?["outcome"] as? String {
            title = Self.humanize(outcome)
        } else if lead.isEmpty || lead.hasPrefix("server refused") || lead.hasPrefix("HTTP") {
            title = status == nil ? "Something went wrong" : "The server refused"
        } else {
            title = lead.prefix(1).uppercased() + lead.dropFirst()
        }
        if let m = (json?["message"] as? String) ?? (json?["refused"] as? String) {
            summary = m
        } else if json == nil, clauses.count > 1 {
            summary = clauses.dropFirst().joined(separator: ": ")
        } else if clauses.isEmpty || title == "Something went wrong" {
            summary = raw
        } else {
            summary = nil
        }
    }

    /// "DeviceCapabilityRefused" -> "Capability refused".
    static func humanize(_ name: String) -> String {
        let bare = name.hasPrefix("Device") ? String(name.dropFirst(6)) : name
        var words: [String] = []
        for ch in bare {
            if ch.isUppercase || words.isEmpty { words.append(String(ch)) } else { words[words.count - 1].append(ch) }
        }
        guard let first = words.first else { return name }
        return ([first] + words.dropFirst().map { $0.lowercased() }).joined(separator: " ")
    }
}

/// One error, presented the same way on every screen.
struct ErrorNotice: View {
    let message: String
    @State private var showDetails = false

    var body: some View {
        let d = ErrorDigest(message)
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Image(systemName: "exclamationmark.triangle.fill").foregroundStyle(.red)
                Text(d.title).font(.subheadline.weight(.semibold))
                Spacer(minLength: 0)
                if let code = d.status {
                    Text("HTTP \(code)").font(.caption2.monospaced())
                        .padding(.horizontal, 6).padding(.vertical, 2)
                        .background(Capsule().fill(Color.red.opacity(0.15)))
                }
            }
            if let s = d.summary {
                Text(s).font(.footnote).foregroundStyle(.secondary).lineLimit(showDetails ? nil : 3)
            }
            Button(showDetails ? "Hide details" : "Details") { withAnimation(.snappy) { showDetails.toggle() } }
                .font(.caption)
            if showDetails {
                Text(d.raw).font(.caption2.monospaced()).textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: 14, style: .continuous).fill(Color.red.opacity(0.08)))
        .overlay(RoundedRectangle(cornerRadius: 14, style: .continuous).strokeBorder(Color.red.opacity(0.25)))
    }
}

/// The current errors, one per failed operation (OperationErrors.shown); nothing here is a log.
struct ErrorStack: View {
    let messages: [String]
    var body: some View {
        VStack(spacing: 8) {
            ForEach(messages, id: \.self) { ErrorNotice(message: $0).transition(.move(edge: .top).combined(with: .opacity)) }
        }
        .animation(.snappy, value: messages)
    }
}

/// The desk's notice strip: one line naming the latest problem and how many more, which opens the
/// papers to the full list. The inbox's one place for errors, so nothing else on it moves for them.
struct ErrorStrip: View {
    let messages: [String]
    let open: () -> Void
    var body: some View {
        if let first = messages.first {
            Button(action: open) {
                HStack(spacing: 8) {
                    Image(systemName: "exclamationmark.triangle.fill").foregroundStyle(.red)
                    Text(ErrorDigest(first).title).lineLimit(1)
                    if messages.count > 1 { Text("+\(messages.count - 1)").foregroundStyle(.white.opacity(0.5)) }
                    Spacer(minLength: 4)
                    Text("Details").foregroundStyle(.white.opacity(0.5))
                    Image(systemName: "chevron.up").foregroundStyle(.white.opacity(0.5))
                }
                .font(.footnote.monospaced())
                .foregroundStyle(.white.opacity(0.85))
                .padding(.horizontal, 12).padding(.vertical, 8)
                .background(Capsule().fill(Color.red.opacity(0.16)))
                .overlay(Capsule().strokeBorder(Color.red.opacity(0.4)))
            }
            .buttonStyle(.plain)
            .transition(.move(edge: .top).combined(with: .opacity))
        }
    }
}

/// The app's errors (configuration, push forwarding, inbox refresh, sign-in) in one fixed place:
/// pinned under the navigation bar, so content never moves to make room for them inline.
struct AppErrorsInset: ViewModifier {
    let messages: [String]
    func body(content: Content) -> some View {
        content.safeAreaInset(edge: .top, spacing: 0) {
            if !messages.isEmpty {
                ErrorStack(messages: messages).padding(.horizontal).padding(.vertical, 8)
                    .background(.bar)
            }
        }
    }
}

struct RootView: View {
    @EnvironmentObject var state: AppState
    var body: some View {
        NavigationStack {
            Group {
                switch state.state {
                case .signedOut: SignInView()
                case .enrolled: InboxView()
                case .revoked(_, let reason): RevokedView(reason: reason)
                }
            }
            // Off the inbox, the app's errors are pinned at the top; on it, they live on the desk.
            .modifier(AppErrorsInset(messages: state.state.enrolled == nil ? state.errors.shown : []))
        }
        // Off the inbox, the garden runs along the bottom; on it, the garden is in the booth's yard.
        .safeAreaInset(edge: .bottom, spacing: 0) {
            if state.state.enrolled == nil { GardenStrip(recruits: state.recruits) }
        }
    }
}

/// The only way in: Google sign-in. Nothing to type; a refused or abandoned sign-in leaves this screen.
struct SignInView: View {
    @EnvironmentObject var state: AppState
    @State private var busy = false

    var body: some View {
        Form {
            Section {
                Button(busy ? "Signing in…" : "Sign in with Google") {
                    busy = true
                    Task { await state.signIn(); busy = false }
                }
                .disabled(busy)
            } footer: {
                Text("Creates a Secure Enclave key that only unlocks with Face ID, attests this app instance, and enrols this phone under your Google account.")
            }
        }
        .navigationTitle("Sign in")
    }
}

struct RevokedView: View {
    @EnvironmentObject var state: AppState
    let reason: String
    var body: some View {
        Form {
            Section("Enrolment revoked") { Text(reason) }
            Section { Button("Sign in again with a new key") { state.signOut() } }
        }
        .navigationTitle("Enrolment revoked")
    }
}

/// ONE REQUEST AT A TIME, full screen (operator direction 2026-10-24: one giant button, "Tinder for
/// approvals"). The deck is exactly the server's set of requests that still need the operator
/// (gunbc.auth.approval_decision_store pending_escalations), in the order the server answers; the
/// first is the card. Deciding re-lists, so the decided request leaves and the next card appears.
/// When nothing needs attention the screen says so and shows nothing else but a current error.
struct InboxView: View {
    @EnvironmentObject var state: AppState
    var body: some View {
        GeometryReader { geo in
            let top = geo.safeAreaInsets.top
            let scene = (geo.size.height + top) * 0.5
            ZStack(alignment: .top) {
                Desk.wood.ignoresSafeArea()
                if let p = state.front {
                    RequestCard(pending: p, remaining: state.pending.count - 1,
                                line: state.pending.filter { $0.escalation_id != p.escalation_id }.map(\.escalation_id),
                                sceneHeight: scene, topInset: top, available: geo.size.height)
                        .id(p.escalation_id)
                        // The booth stays put between bots; only the bot and its papers change.
                        .transition(.identity)
                } else {
                    VStack(spacing: 0) {
                        CheckpointScene(verdict: .pending, verdictAt: nil, arrivedAt: .distantPast, queue: 0, occupied: false,
                                        recruits: state.recruits, topInset: top)
                            .frame(height: scene)
                            .padding(.top, -top)
                        ScrollView {
                            if !state.errors.shown.isEmpty {
                                ErrorStack(messages: state.errors.shown).padding()
                            }
                            ContentUnavailableView {
                                Label("Nobody at the window", systemImage: "cup.and.saucer")
                            } description: {
                                Text("It won't last. Pull down to check again.")
                            }
                            .frame(minHeight: geo.size.height - scene + top - 40)
                        }
                        .scrollBounceBehavior(.always)
                        .refreshable { await state.refresh() }
                        .environment(\.colorScheme, .dark)
                    }
                }
            }
            .animation(.snappy, value: state.front?.escalation_id)
        }
        .toolbar(.hidden, for: .navigationBar)
        .task { await state.refresh() }
    }
}

/// The desk the papers are laid on, below the window.
enum Desk {
    static let wood = Color(hex: 0x2E2620)
}

/// The card, as a customs desk (operator direction 2026-11-02: "Arstotzka"): a bot waits at the
/// barrier, its request is the entry permit on the desk, and the officer stamps it. Swipe the permit
/// right to approve, left to deny, or press a stamp. Face ID is the enclave's own prompt on signing
/// (AppState.redeem). The stamp lands, and the bot reacts, only on the server's DeviceRedeemed: a
/// refused or failed redemption leaves the permit unstamped and shows why.
struct RequestCard: View {
    @EnvironmentObject var state: AppState
    let pending: PendingApproval
    let remaining: Int
    /// The requests behind this one, in order: the bots in line.
    var line: [String] = []
    let sceneHeight: CGFloat
    let topInset: CGFloat
    /// The page's height below the status bar: what the papers grow to fill.
    let available: CGFloat
    @State private var fetched: FetchedRequest?
    @State private var summary: StoredRequestSummary?
    @State private var outcome: RedemptionOutcome?
    /// This card's own operations (requestFetch, redemption), under the same lifetime rule.
    @State private var errors = OperationErrors()
    @State private var busy = false
    @State private var drag: CGFloat = 0
    /// How far the permit is being pushed up toward its full view.
    @State private var lift: CGFloat = 0
    /// The papers pulled up to fill the page: swipe the permit up (or tap it); drag the handle down
    /// (or tap it) to put them back on the desk.
    @State private var showFull = false
    @State private var pull: CGFloat = 0
    @State private var verdict: Verdict = .pending
    @State private var verdictAt: Date?
    @State private var arrivedAt = Date()
    /// The papers are on the desk once the bot has stepped up and slid them through the slot.
    @State private var handed = false
    private let threshold: CGFloat = 120

    /// gunbc.auth.approval_device_redemption's admitted arm (OutcomeName.all), the only outcome that stamps.
    private static let admitted = "DeviceRedeemed"

    private var decidable: Bool { handed && fetched != nil && summary != nil && !busy && verdict == .pending }

    var body: some View {
        let deskHeight = available - (sceneHeight - topInset)
        ZStack(alignment: .bottom) {
            VStack(spacing: 0) {
                CheckpointScene(verdict: verdict, verdictAt: verdictAt, arrivedAt: arrivedAt, queue: remaining,
                                recruits: state.recruits, topInset: topInset, visitor: pending.escalation_id, line: line)
                    .frame(height: sceneHeight)
                    .padding(.top, -topInset)
                Spacer(minLength: 0)
            }
            // One panel: the desk, which grows over the scene to show the whole request.
            VStack(spacing: 0) {
                if showFull {
                    handle
                    ScrollView {
                        VStack(spacing: 14) {
                            ErrorStack(messages: allErrors).environment(\.colorScheme, .dark)
                            paper(full: true)
                        }
                        .padding(.horizontal).padding(.vertical, 14)
                    }
                } else {
                    // Nothing on the desk scrolls, so a swipe up on the papers only ever raises them.
                    VStack(spacing: 12) {
                        ErrorStrip(messages: allErrors) { setFull(true) }
                        desk
                    }
                    .padding(.horizontal).padding(.top, 12)
                    .animation(.snappy, value: allErrors)
                }
                stamps { d in
                    // Deciding from the full view puts the papers back down to be stamped.
                    if showFull { setFull(false) }
                    if let f = fetched { decide(f, d) }
                }
                .padding(.horizontal).padding(.top, 10).padding(.bottom, 4)
            }
            .frame(height: max(showFull ? available - pull : deskHeight, deskHeight), alignment: .top)
            .clipped()
            .background(Desk.wood)
            .clipShape(UnevenRoundedRectangle(topLeadingRadius: showFull ? 16 : 0, topTrailingRadius: showFull ? 16 : 0))
            .shadow(color: .black.opacity(showFull ? 0.35 : 0), radius: 12, y: -2)
        }
        .frame(height: available, alignment: .bottom)
        .task {
            // The bot steps up (Checkpoint.arriveTime), then slides its papers through.
            try? await Task.sleep(nanoseconds: 1_500_000_000)
            withAnimation(.spring(response: 0.45, dampingFraction: 0.8)) { handed = true }
            #if DEBUG
            if Demo.full { try? await Task.sleep(nanoseconds: 600_000_000); setFull(true) }
            #endif
        }
        .task { await load() }
    }

    /// Every current error, this card's first: shown as one strip on the desk, in full when raised.
    private var allErrors: [String] { errors.shown + state.errors.shown }

    private func setFull(_ on: Bool) {
        withAnimation(.spring(response: 0.42, dampingFraction: 0.86)) { showFull = on; pull = 0 }
    }

    /// The top of the raised papers: drag it down, or tap it, to put them back on the desk.
    private var handle: some View {
        VStack(spacing: 6) {
            Capsule().fill(.white.opacity(0.35)).frame(width: 40, height: 5)
            Label("Back to the desk", systemImage: "chevron.down").font(.caption.monospaced()).foregroundStyle(.white.opacity(0.55))
        }
        .frame(maxWidth: .infinity)
        .padding(.top, 10).padding(.bottom, 4)
        .contentShape(Rectangle())
        .onTapGesture { setFull(false) }
        .gesture(
            DragGesture(minimumDistance: 8)
                .onChanged { v in pull = max(v.translation.height, 0) }
                .onEnded { v in if v.translation.height > 80 || v.predictedEndTranslation.height > 200 { setFull(false) } else { setFull(true) } }
        )
        .accessibilityAddTraits(.isButton)
    }

    private var desk: some View {
        VStack(spacing: 16) {
            if handed {
                permit.transition(.move(edge: .top).combined(with: .opacity))
            } else {
                Text("Waiting for papers…").font(.footnote.monospaced()).foregroundStyle(.white.opacity(0.5))
                    .frame(maxWidth: .infinity, minHeight: 120)
            }
            // Status sits directly above the stamps so a decision's result is always on screen.
            if busy && verdict == .pending {
                HStack(spacing: 8) { ProgressView(); Text("Sending decision…") }.frame(maxWidth: .infinity)
                    .foregroundStyle(.white.opacity(0.8))
            }
            if let o = outcome {
                VStack(alignment: .leading, spacing: 4) {
                    Text(ErrorDigest.humanize(o.outcome)).font(.headline)
                    Text(o.message)
                }
                .foregroundStyle(.white.opacity(0.85))
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }

    private func stamps(_ act: @escaping (ProposedDecision) -> Void) -> some View {
        VStack(spacing: 6) {
            HStack(spacing: 20) {
                Button { act(.deny) } label: { StampBlock(title: "DENY", ink: Garden.stop) }
                    .accessibilityLabel("Deny")
                Button { act(.approve) } label: { StampBlock(title: "APPROVE", ink: Garden.success) }
                    .accessibilityLabel("Approve")
            }
            .buttonStyle(StampPressStyle())
            .disabled(!decidable)
            if remaining > 0 {
                Text(remaining == 1 ? "1 more in line" : "\(remaining) more in line").font(.footnote.monospaced()).foregroundStyle(.white.opacity(0.5))
            }
        }
    }

    private func load() async {
        do {
            let f = try await state.fetch(pending.escalation_id)
            fetched = f
            summary = try WireDecode.storedRequest(f.stored_request_text)
            errors.record(.requestFetch, .succeeded)
            #if DEBUG
            if Demo.autoplay {
                try? await Task.sleep(nanoseconds: 2_500_000_000)
                decide(f, Demo.autoplayDecision(pending.escalation_id))
            }
            #endif
        } catch { errors.record(.requestFetch, .failed(error.localizedDescription)) }
    }

    /// The request as an entry permit: what is asked, by whom, until when. On the desk it is a card
    /// (swipe it left or right to decide, up to read all of it); the full view adds the exact text.
    private func paper(full: Bool) -> some View {
        let id = pending.escalation_id
        return VStack(alignment: .leading, spacing: 0) {
            // The plinth: the request's arrangement, what it is called and where it comes from. On the
            // desk it takes whatever room is spare.
            HStack(alignment: .bottom, spacing: 12) {
                BotPortrait(id: id)
                    .frame(maxWidth: 130, maxHeight: .infinity)
                VStack(alignment: .leading, spacing: 4) {
                    if let s = summary {
                        Text(s.destructive ? "DESTRUCTIVE" : "NON-DESTRUCTIVE")
                            .font(.caption2.monospaced().weight(.bold))
                            .padding(.horizontal, 7).padding(.vertical, 3)
                            .foregroundStyle(s.destructive ? .white : Permit.ink)
                            .background(Capsule().fill(s.destructive ? Garden.stop : Permit.ink.opacity(0.1)))
                        Spacer(minLength: 6)
                    }
                    Text(Wardrobe.describe(id)).font(.system(.subheadline, design: .serif).italic()).fixedSize(horizontal: false, vertical: true)
                    Text("from \(Wardrobe.origin(of: id))").font(.caption2.monospaced()).foregroundStyle(Permit.ink.opacity(0.6))
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottomLeading)
            }
            .padding(.horizontal, 14).padding(.top, full ? 16 : 22).padding(.bottom, 12)
            .frame(minHeight: full ? 190 : 84, maxHeight: full ? 230 : .infinity)
            .background(Color(hex: 0xC9C2B2).opacity(0.45))
            .overlay(alignment: .top) {
                if !full { Capsule().fill(Permit.ink.opacity(0.25)).frame(width: 36, height: 4).padding(.top, 8) }
            }
            VStack(alignment: .leading, spacing: full ? 10 : 6) {
                if let s = summary {
                    Text(s.purpose).font(.system(.title3, design: .serif).weight(.semibold)).lineLimit(full ? nil : 2)
                    PermitField(label: "BY", value: Text(s.requester))
                    PermitField(label: "EXPIRES", value: Permit.expiry(s.expires_at))
                } else if fetched != nil {
                    Text("Stored request unreadable; deciding is refused.").foregroundStyle(.red)
                } else if errors.shown.isEmpty {
                    ProgressView().frame(maxWidth: .infinity)
                }
                PermitField(label: "ID", value: Text(pending.escalation_id).font(.caption2.monospaced()))
                if full, let f = fetched {
                    Rectangle().fill(Permit.ink.opacity(0.2)).frame(height: 1)
                    Text("EXACTLY WHAT IS SIGNED").font(.caption2.monospaced().weight(.bold)).foregroundStyle(Permit.ink.opacity(0.6))
                    Text(f.stored_request_text)
                        .font(.system(.footnote, design: .monospaced))
                        .textSelection(.enabled)
                }
            }
            .padding(16)
            .fixedSize(horizontal: false, vertical: true)          // the words are never squeezed; the plinth gives
        }
        .foregroundStyle(Permit.ink)
        .tint(Color(hex: 0x2F5E8C))
        .environment(\.colorScheme, .light)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(Permit.paper)
        .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        .shadow(color: .black.opacity(0.18), radius: 8, y: 3)
    }

    /// The permit on the desk: swipe right to approve, left to deny, up (or tap) for the full view.
    private var permit: some View {
        paper(full: false)
            .overlay {
                if verdict != .pending {
                    VerdictStamp(verdict: verdict).transition(.scale(scale: 2.6).combined(with: .opacity))
                }
            }
            .overlay(alignment: .top) {
                if drag > 0 { Text("APPROVE").font(.title.bold().monospaced()).foregroundStyle(Garden.success).opacity(Double(min(drag / threshold, 1))).padding() }
                else if drag < 0 { Text("DENY").font(.title.bold().monospaced()).foregroundStyle(Garden.stop).opacity(Double(min(-drag / threshold, 1))).padding() }
            }
            .offset(x: drag, y: lift)
            .rotationEffect(.degrees(Double(drag / 20)))
            .contentShape(Rectangle())
            .onTapGesture { if fetched != nil { setFull(true) } }
            .simultaneousGesture(
                DragGesture(minimumDistance: 16)
                    .onChanged { v in
                        let horizontal = abs(v.translation.width) > abs(v.translation.height)
                        if horizontal { if decidable { drag = v.translation.width } }
                        else if v.translation.height < 0 { lift = max(v.translation.height / 3, -40) }
                    }
                    .onEnded { v in
                        let w = v.translation.width, h = v.translation.height
                        let horizontal = abs(w) > abs(h)
                        withAnimation(.spring()) { drag = 0; lift = 0 }
                        if !horizontal {
                            if h < -50 && fetched != nil { setFull(true) }
                            return
                        }
                        guard decidable, let f = fetched else { return }
                        if w > threshold { decide(f, .approve) } else if w < -threshold { decide(f, .deny) }
                    }
            )
    }

    /// Redeem; on the server's admission stamp the permit and let the bot react, then re-list so the
    /// decided request leaves and the next bot steps up. Any other outcome is the server's answer and
    /// is shown as is, unstamped.
    private func decide(_ f: FetchedRequest, _ d: ProposedDecision) {
        busy = true
        Task {
            do {
                let o = try await state.redeem(f, d)
                errors.record(.redemption, .succeeded)
                if o.outcome == Self.admitted {
                    verdictAt = Date()
                    withAnimation(.spring(response: 0.28, dampingFraction: 0.55)) { verdict = d == .approve ? .approved : .denied }
                    UINotificationFeedbackGenerator().notificationOccurred(d == .approve ? .success : .warning)
                    if d == .approve { state.recruit() }
                    try? await Task.sleep(nanoseconds: 2_400_000_000)
                } else {
                    outcome = o
                }
            } catch { errors.record(.redemption, .failed(error.localizedDescription)) }
            busy = false
            await state.refresh()
        }
    }
}

/// The permit's paper and ink, light and dark.
enum Permit {
    // Paper is paper in either mode: the desk around it is what is dark.
    static let paper = Color(hex: 0xF4EAD5)
    static let ink = Color(hex: 0x3B3226)

    /// "in 59 minutes", from the request's own RFC 3339 expiry; the raw text if it does not parse.
    static func expiry(_ raw: String) -> Text {
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime]
        guard let d = f.date(from: raw) else { return Text(raw) }
        return d < Date() ? Text("expired ") + Text(d, format: .relative(presentation: .named)) : Text(d, format: .relative(presentation: .named))
    }
}

struct PermitField: View {
    let label: String
    let value: Text
    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 10) {
            Text(label).font(.caption2.monospaced().weight(.bold)).foregroundStyle(Permit.ink.opacity(0.6)).frame(width: 58, alignment: .leading)
            value.font(.subheadline).lineLimit(2).truncationMode(.middle).frame(maxWidth: .infinity, alignment: .leading)
        }
    }
}

/// The rubber stamp: APPROVED in green or DENIED in red, set at a slant across the permit.
struct VerdictStamp: View {
    let verdict: Verdict
    var body: some View {
        let approved = verdict == .approved
        let colour = approved ? Garden.success : Garden.stop
        Text(approved ? "APPROVED" : "DENIED")
            .font(.system(size: 38, weight: .black, design: .monospaced))
            .foregroundStyle(colour)
            .padding(.horizontal, 14).padding(.vertical, 4)
            .overlay(RoundedRectangle(cornerRadius: 8).strokeBorder(colour, lineWidth: 5))
            .rotationEffect(.degrees(approved ? -12 : 10))
            .opacity(0.88)
            .accessibilityLabel(approved ? "Approved" : "Denied")
    }
}

/// A decision button: a quiet, inked outline with a seal, the ink a shade muted.
struct StampBlock: View {
    let title: String
    let ink: Color
    @Environment(\.isEnabled) private var isEnabled
    var body: some View {
        Label(title, systemImage: title == "APPROVE" ? "checkmark.seal" : "xmark.seal")
            .font(.system(.subheadline, design: .monospaced).weight(.semibold))
            .tracking(1)
            .foregroundStyle(ink)
            .frame(maxWidth: .infinity, minHeight: 52)
            .background(RoundedRectangle(cornerRadius: 12, style: .continuous).fill(ink.opacity(0.14)))
            .overlay(RoundedRectangle(cornerRadius: 12, style: .continuous).strokeBorder(ink.opacity(0.8), lineWidth: 1.5))
            .opacity(isEnabled ? 1 : 0.4)
    }
}

/// Pressing a decision button settles it a little.
struct StampPressStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .scaleEffect(configuration.isPressed ? 0.97 : 1)
            .brightness(configuration.isPressed ? -0.05 : 0)
            .animation(.snappy(duration: 0.12), value: configuration.isPressed)
    }
}

extension Color {
    init(light: UInt32, dark: UInt32) {
        self.init(uiColor: UIColor { $0.userInterfaceStyle == .dark ? UIColor(Color(hex: dark)) : UIColor(Color(hex: light)) })
    }
}
