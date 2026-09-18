import SwiftUI

struct RootView: View {
    @EnvironmentObject var state: AppState
    var body: some View {
        NavigationStack {
            if state.enrolment == nil { EnrolView() } else { InboxView() }
        }
    }
}

struct EnrolView: View {
    @EnvironmentObject var state: AppState
    @State private var challengeId = ""
    @State private var code = ""
    @State private var busy = false

    var body: some View {
        Form {
            Section("One-time code from the dashboard") {
                TextField("challenge id", text: $challengeId).textInputAutocapitalization(.never).autocorrectionDisabled()
                TextField("code", text: $code).keyboardType(.numberPad)
            }
            Section {
                Button(busy ? "Enrolling…" : "Enrol this phone") {
                    busy = true
                    Task { await state.enrol(challengeId: challengeId, code: code); busy = false }
                }
                .disabled(busy || challengeId.isEmpty || code.isEmpty)
            } footer: {
                Text("Creates a Secure Enclave key that only unlocks with Face ID, attests this app instance, and registers for push.")
            }
            if let e = state.lastError { Section("Refused") { Text(e).foregroundStyle(.red) } }
        }
        .navigationTitle("Enrol")
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
    @State private var outcome: RedemptionOutcome?
    @State private var error: String?
    @State private var busy = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                if let f = fetched {
                    // The stored request exactly as the server returned it: this is what gets signed.
                    Text(f.stored_request_text)
                        .font(.system(.body, design: .monospaced))
                        .textSelection(.enabled)
                    Text("challenge expires \(f.challenge.expires_at)").font(.caption).foregroundStyle(.secondary)
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
            do { fetched = try await state.client?.fetch(pending.escalation_id) }
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
