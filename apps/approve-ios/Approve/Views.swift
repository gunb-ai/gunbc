import SwiftUI

struct RootView: View {
    @EnvironmentObject var state: AppState
    var body: some View {
        NavigationStack {
            switch state.state {
            case .unenrolled: EnrolView()
            case .prepared, .submissionUnknown: EnrolmentPendingView()
            case .enrolled: InboxView()
            case .keyInvalidated(_, let reason): DeadEnrolmentView(title: "Decision key invalidated", reason: reason)
            case .revoked(_, let reason): DeadEnrolmentView(title: "Enrolment revoked", reason: reason)
            }
        }
    }
}

struct EnrolView: View {
    @EnvironmentObject var state: AppState
    @State private var code = ""
    @State private var busy = false

    var body: some View {
        Form {
            Section("One-time code printed by the enrol command on srv1") {
                TextField("code", text: $code).keyboardType(.numberPad)
            }
            Section {
                Button(busy ? "Enrolling…" : "Enrol this phone") {
                    busy = true
                    Task { await state.prepare(code: code); busy = false }
                }
                .disabled(busy || code.isEmpty)
            } footer: {
                Text("Creates a Secure Enclave key that only unlocks with Face ID, attests this app instance, and registers for push.")
            }
            if let e = state.lastError { Section("Refused") { Text(e).foregroundStyle(.red) } }
        }
        .navigationTitle("Enrol")
    }
}

/// Prepared (keys exist, server has not accepted) or SubmissionUnknown (POST sent, answer lost).
struct EnrolmentPendingView: View {
    @EnvironmentObject var state: AppState
    @State private var busy = false

    var body: some View {
        Form {
            switch state.state {
            case .prepared:
                Section("Enrolment prepared, not yet accepted") {
                    Text("Keys are generated on this phone. Submitting sends the same attestation again; nothing new is generated.")
                    Button(busy ? "Submitting…" : "Submit enrolment") { run { await state.submit() } }.disabled(busy)
                }
            case .submissionUnknown:
                Section("Submission outcome unknown") {
                    Text("The server may or may not have accepted this enrolment. The keys, attestation and code are kept until an authenticated re-read says active, revoked or absent; nothing new is generated.")
                    Button("Re-read enrolment") { run { await state.resolveUnknownSubmission() } }.disabled(busy)
                    Button("Retry the same submission") { run { await state.retrySubmission() } }.disabled(busy)
                }
            default:
                EmptyView()
            }
            if case .prepared = state.state {
                Section { Button("Start over", role: .destructive) { state.startOver() } }
            }
            if let e = state.lastError { Section("Refused") { Text(e).foregroundStyle(.red) } }
        }
        .navigationTitle("Enrolment")
    }

    private func run(_ op: @escaping () async -> Void) {
        busy = true
        Task { await op(); busy = false }
    }
}

struct DeadEnrolmentView: View {
    @EnvironmentObject var state: AppState
    let title: String
    let reason: String
    var body: some View {
        Form {
            Section(title) { Text(reason) }
            Section { Button("Enrol again") { state.enrolAgain() } }
        }
        .navigationTitle(title)
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
