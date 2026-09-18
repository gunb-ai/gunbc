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
                    Text("The server may or may not have accepted this enrolment. Re-read before generating anything new.")
                    Button("Re-read enrolment") { run { await state.resolveUnknownSubmission() } }.disabled(busy)
                    Button("Retry the same submission") { run { await state.retrySubmission() } }.disabled(busy)
                }
            default:
                EmptyView()
            }
            Section { Button("Start over", role: .destructive) { state.startOver() } }
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
            Section { Button("Enrol again") { state.startOver() } }
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

/// What the stored request says today, read from its existing fields. Once the store carries
/// ApprovalTarget and the access lifetime (approval_target_frontier) these become typed facts
/// returned by the server; until then nothing here is invented — an absent field renders as absent.
struct StoredRequestSummary {
    var purpose: String?
    var destructive: Bool?

    init(json: String) {
        let obj = (try? JSONSerialization.jsonObject(with: Data(json.utf8))) as? [String: Any]
        purpose = obj?["purpose"] as? String
        destructive = obj?["destructive"] as? Bool
    }
}

struct DetailView: View {
    @EnvironmentObject var state: AppState
    let pending: PendingApproval
    @State private var fetched: FetchedRequest?
    @State private var outcome: RedemptionOutcome?
    @State private var error: String?
    @State private var busy = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if let f = fetched {
                    let summary = StoredRequestSummary(json: f.stored_request_text)
                    if let purpose = summary.purpose {
                        Text(purpose).font(.title3)
                    }
                    if let d = summary.destructive {
                        Label(d ? "Destructive" : "Not destructive", systemImage: d ? "exclamationmark.triangle" : "checkmark.shield")
                            .foregroundStyle(d ? .red : .secondary)
                    }
                    // The deadline shown is the LINK's expiry (the redemption challenge), not how long
                    // any access lasts; the access lifetime is not carried by the store yet.
                    Text("Link expires \(f.challenge.expires_at)").font(.caption).foregroundStyle(.secondary)
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
                        .disabled(busy)
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
            do { fetched = try await state.fetch(pending.escalation_id) }
            catch { self.error = error.localizedDescription }
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
