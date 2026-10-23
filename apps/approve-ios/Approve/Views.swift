import SwiftUI

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
            if let e = state.lastError { Section("Refused") { Text(e).foregroundStyle(.red) } }
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

struct InboxView: View {
    @EnvironmentObject var state: AppState
    var body: some View {
        List {
            if state.pending.isEmpty { Text("Nothing pending").foregroundStyle(.secondary) }
            ForEach(state.pending) { p in
                NavigationLink(value: p) {
                    VStack(alignment: .leading) {
                        Text(p.escalation_id).font(.headline)
                        Text("revision \(p.request_revision)").font(.caption).foregroundStyle(.secondary)
                    }
                }
            }
            if let e = state.lastError { Section("Error") { Text(e).foregroundStyle(.red) } }
        }
        .navigationTitle("Approvals")
        .navigationDestination(for: PendingApproval.self) { DetailView(pending: $0) }
        .refreshable { await state.refresh() }
        .task { await state.refresh() }
    }
}

struct DetailView: View {
    @EnvironmentObject var state: AppState
    let pending: PendingApproval
    @State private var fetched: FetchedRequest?
    @State private var summary: StoredRequestSummary?
    @State private var outcome: RedemptionOutcome?
    @State private var error: String?
    @State private var busy = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if let f = fetched {
                    if let s = summary {
                        Text(s.purpose).font(.title3)
                        Text("requested by \(s.requester)")
                        Label(s.destructive ? "Destructive" : "Not destructive", systemImage: s.destructive ? "exclamationmark.triangle" : "checkmark.shield")
                            .foregroundStyle(s.destructive ? .red : .secondary)
                        // The request's own expiry, from the stored record. The access lifetime is
                        // not carried by the store yet (approval_target_frontier).
                        Text("Request expires \(s.expires_at)").font(.caption).foregroundStyle(.secondary)
                        Text("Refresh required after \(f.challenge.expires_at)").font(.caption2).foregroundStyle(.secondary)
                    } else {
                        Text("Stored request unreadable; deciding is refused.").foregroundStyle(.red)
                    }
                    DisclosureGroup("Stored request (exactly what is signed)") {
                        Text(f.stored_request_text)
                            .font(.system(.footnote, design: .monospaced))
                            .textSelection(.enabled)
                    }
                    if let o = outcome {
                        VStack(alignment: .leading) {
                            Text(o.outcome).font(.headline)
                            Text(o.message)
                        }
                    } else {
                        HStack {
                            Button("Deny", role: .destructive) { decide(f, .deny) }
                            Spacer()
                            Button("Approve") { decide(f, .approve) }.buttonStyle(.borderedProminent)
                        }
                        .disabled(busy || summary == nil)
                    }
                } else if error == nil {
                    ProgressView()
                }
                if let e = error { Text(e).foregroundStyle(.red) }
            }
            .padding()
        }
        .navigationTitle(pending.escalation_id)
        .task {
            do {
                let f = try await state.fetch(pending.escalation_id)
                fetched = f
                do { summary = try WireDecode.storedRequest(f.stored_request_text) }
                catch { self.error = error.localizedDescription }
            } catch { self.error = error.localizedDescription }
        }
    }

    private func decide(_ f: FetchedRequest, _ d: ProposedDecision) {
        busy = true
        Task {
            do { outcome = try await state.redeem(f, d) }
            catch { self.error = error.localizedDescription }
            busy = false
        }
    }
}
