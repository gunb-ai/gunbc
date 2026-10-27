import SwiftUI
import UIKit

/// The current errors, one per failed operation (OperationErrors.shown). A section exists only while
/// some operation's latest attempt failed; nothing here is a log.
struct ErrorSection: View {
    let title: String
    let errors: OperationErrors
    var body: some View {
        if !errors.shown.isEmpty {
            Section(title) {
                ForEach(Array(errors.shown.enumerated()), id: \.offset) { Text($0.element).foregroundStyle(.red) }
            }
        }
    }
}

struct RootView: View {
    @EnvironmentObject var state: AppState
    var body: some View {
        NavigationStack {
            switch state.state {
            case .signedOut: SignInView()
            case .enrolled: InboxView()
            case .revoked(_, let reason): RevokedView(reason: reason)
            }
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
            ErrorSection(title: "Refused", errors: state.errors)
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
            ErrorSection(title: "Error", errors: state.errors)
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
            ScrollView {
                VStack(spacing: 16) {
                    if let p = state.pending.first {
                        RequestCard(pending: p, remaining: state.pending.count - 1).id(p.escalation_id)
                    } else {
                        Spacer(minLength: 0)
                        Image(systemName: "checkmark.circle").font(.system(size: 72)).foregroundStyle(.secondary)
                        Text("Nothing needs your attention").font(.title2).foregroundStyle(.secondary)
                        Spacer(minLength: 0)
                        // With a card on screen, these errors render in the card's status block instead.
                        ForEach(Array(state.errors.shown.enumerated()), id: \.offset) { Text($0.element).foregroundStyle(.red) }
                    }
                }
                .padding()
                .frame(minHeight: geo.size.height)
            }
            .refreshable { await state.refresh() }
        }
        .task { await state.refresh() }
    }
}

/// The card: what is asked, by which run, for what; swipe right to approve, left to deny, or the
/// two large buttons. Face ID is the enclave's own prompt on signing (AppState.redeem).
struct RequestCard: View {
    @EnvironmentObject var state: AppState
    let pending: PendingApproval
    let remaining: Int
    @State private var fetched: FetchedRequest?
    @State private var summary: StoredRequestSummary?
    @State private var outcome: RedemptionOutcome?
    /// This card's own operations (requestFetch, redemption), under the same lifetime rule.
    @State private var errors = OperationErrors()
    @State private var busy = false
    @State private var drag: CGFloat = 0
    @State private var expanded = false
    private let threshold: CGFloat = 120

    private var decidable: Bool { fetched != nil && summary != nil && !busy }

    var body: some View {
        VStack(spacing: 20) {
            VStack(alignment: .leading, spacing: 12) {
                if let s = summary {
                    Text(s.purpose).font(.title3.bold()).lineLimit(expanded ? nil : 4)
                    Button(expanded ? "Show less" : "Show full request") { expanded.toggle() }.font(.footnote)
                    Label("requested by \(s.requester)", systemImage: "person.crop.circle")
                    Label(s.destructive ? "Destructive" : "Not destructive", systemImage: s.destructive ? "exclamationmark.triangle.fill" : "checkmark.shield")
                        .foregroundStyle(s.destructive ? .red : .secondary)
                    // The request's own expiry, from the stored record.
                    Text("Expires \(s.expires_at)").font(.caption).foregroundStyle(.secondary)
                } else if fetched != nil {
                    Text("Stored request unreadable; deciding is refused.").foregroundStyle(.red)
                } else if errors.shown.isEmpty {
                    ProgressView().frame(maxWidth: .infinity)
                }
                Text(pending.escalation_id).font(.caption2.monospaced()).foregroundStyle(.secondary)
                if let f = fetched {
                    DisclosureGroup("Exactly what is signed") {
                        Text(f.stored_request_text)
                            .font(.system(.footnote, design: .monospaced))
                            .textSelection(.enabled)
                    }
                }
            }
            .padding(24)
            .frame(maxWidth: .infinity, alignment: .topLeading)
            .background(RoundedRectangle(cornerRadius: 24).fill(Color(uiColor: .secondarySystemBackground)))
            .overlay(alignment: .top) {
                if drag > 0 { Text("APPROVE").font(.title.bold()).foregroundStyle(.green).opacity(Double(min(drag / threshold, 1))).padding() }
                else if drag < 0 { Text("DENY").font(.title.bold()).foregroundStyle(.red).opacity(Double(min(-drag / threshold, 1))).padding() }
            }
            .offset(x: drag)
            .rotationEffect(.degrees(Double(drag / 20)))
            .simultaneousGesture(
                DragGesture(minimumDistance: 24)
                    .onChanged { v in
                        if decidable, abs(v.translation.width) > abs(v.translation.height) { drag = v.translation.width }
                    }
                    .onEnded { v in
                        let w = v.translation.width
                        let horizontal = abs(w) > abs(v.translation.height)
                        withAnimation(.spring()) { drag = 0 }
                        guard decidable, horizontal, let f = fetched else { return }
                        if w > threshold { decide(f, .approve) } else if w < -threshold { decide(f, .deny) }
                    }
            )
            // Status sits directly above the buttons so a decision's result is always on screen.
            if busy {
                HStack(spacing: 8) { ProgressView(); Text("Sending decision…") }.frame(maxWidth: .infinity)
            }
            if let o = outcome {
                VStack(alignment: .leading, spacing: 4) {
                    Text(o.outcome).font(.headline)
                    Text(o.message)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            // One status block: this card's errors, then the app's (push forwarding, inbox refresh).
            ForEach(Array((errors.shown + state.errors.shown).enumerated()), id: \.offset) {
                Text($0.element).foregroundStyle(.red).frame(maxWidth: .infinity, alignment: .leading)
            }
            HStack(spacing: 16) {
                Button(role: .destructive) { if let f = fetched { decide(f, .deny) } } label: {
                    Label("Deny", systemImage: "xmark").frame(maxWidth: .infinity, minHeight: 64)
                }
                .buttonStyle(.bordered)
                Button { if let f = fetched { decide(f, .approve) } } label: {
                    Label("Approve", systemImage: "checkmark").frame(maxWidth: .infinity, minHeight: 64)
                }
                .buttonStyle(.borderedProminent)
            }
            .font(.title3.bold())
            .disabled(!decidable)
            if remaining > 0 { Text("\(remaining) more waiting").font(.footnote).foregroundStyle(.secondary) }
        }
        .task {
            do {
                let f = try await state.fetch(pending.escalation_id)
                fetched = f
                summary = try WireDecode.storedRequest(f.stored_request_text)
                errors.record(.requestFetch, .succeeded)
            } catch { errors.record(.requestFetch, .failed(error.localizedDescription)) }
        }
    }

    /// Redeem, then AppState re-lists: an admitted decision removes this request from the server's
    /// set, so this card is replaced by the next one. A refused outcome is the server's answer and is
    /// shown on the card as is.
    private func decide(_ f: FetchedRequest, _ d: ProposedDecision) {
        busy = true
        Task {
            do {
                outcome = try await state.redeem(f, d)
                errors.record(.redemption, .succeeded)
            } catch { errors.record(.redemption, .failed(error.localizedDescription)) }
            busy = false
        }
    }
}
