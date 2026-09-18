import Foundation
import SwiftUI
import CryptoKit
import UIKit
import UserNotifications

@MainActor
final class AppState: ObservableObject {
    @Published private(set) var state: EnrolmentState = .unenrolled
    @Published var pending: [PendingApproval] = []
    @Published var lastError: String?
    /// The APNs token most recently delivered by the system. Arrives asynchronously; buffered here
    /// so a token that lands before the state is installed is not lost.
    @Published private(set) var apnsToken: String?
    private var installed = false

    let config: ServerConfig?
    let client: Client?

    init() {
        do {
            let c = try ServerConfig.fromBundle()
            config = c
            client = Client(config: c)
        } catch {
            config = nil
            client = nil
            lastError = error.localizedDescription
        }
    }

    /// Called once by the root view: read the persisted state, then register for APNs on every
    /// enrolled launch (a token rotates on reinstall/restore; the server must learn each one).
    func install() async {
        guard !installed else { return }
        installed = true
        do { state = try EnrolmentStore.load() } catch { lastError = error.localizedDescription }
        if state.enrolled != nil {
            UIApplication.shared.registerForRemoteNotifications()
            if let t = apnsToken { await forwardToken(t) }
        }
    }

    private func requireClient() throws -> Client {
        guard let client else { throw WireError.configMissing(lastError ?? "no server configuration") }
        return client
    }

    private func transition(_ s: EnrolmentState) throws {
        try EnrolmentStore.save(s)
        state = s
    }

    // ── APNs ─────────────────────────────────────────────────────────────────────────────────
    func tokenDelivered(_ hex: String) {
        apnsToken = hex
        guard installed, state.enrolled != nil else { return }   // buffered until install()
        Task { await forwardToken(hex) }
    }

    /// Forward a token the server has not been told about. The update route is not modeled yet, so
    /// this refuses visibly (WireError.unmodeled) rather than pretending the server knows.
    private func forwardToken(_ hex: String) async {
        guard case .enrolled(var e) = state, e.forwarded_apns_token != hex else { return }
        do {
            let client = try requireClient()
            try await client.updatePushRegistration(registration(hex), try await readAuth(path: Route.pending))
            e.forwarded_apns_token = hex
            try transition(.enrolled(e))
        } catch {
            lastError = "APNs token not forwarded: \(error.localizedDescription)"
        }
    }

    private func registration(_ token: String) -> ApnsRegistration {
        ApnsRegistration(
            environment: config?.apnsEnvironment ?? "",
            topic: Bundle.main.bundleIdentifier ?? "ai.gunb.approve",
            token: token
        )
    }

    private func awaitToken() async throws -> String {
        let granted = try await UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound])
        guard granted else { throw WireError.configMissing("notifications were not permitted") }
        UIApplication.shared.registerForRemoteNotifications()
        for _ in 0..<50 {
            if let t = apnsToken { return t }
            try await Task.sleep(nanoseconds: 200_000_000)
        }
        throw WireError.configMissing("APNs did not deliver a device token")
    }

    // ── Enrolment ────────────────────────────────────────────────────────────────────────────
    /// Step 1, local only: refuse without App Attest; generate both keys; fix the transcript; PERSIST
    /// as Prepared before anything leaves the device.
    func prepare(code: String) async {
        lastError = nil
        do {
            try AppAttest.requireSupported()
            let key = try DecisionKey.create()
            let pub = DecisionKey.verifyingKey(key)
            let attestKeyId = try await AppAttest.generateKey()
            let transcript = enrolmentTranscript(code: code, decisionKey: pub, platform: .ios)
            try transition(.prepared(PreparedEnrolment(
                code: code, decision_key_blob: key.dataRepresentation, decision_key_pub: pub,
                attest_key_id: attestKeyId, transcript: transcript, attestation_b64: nil)))
            await submit()
        } catch {
            lastError = error.localizedDescription
        }
    }

    /// Step 2, remote: attest over the fixed transcript (reusing a previous attestation if one was
    /// generated), register for push, POST. A lost answer moves to SubmissionUnknown; a refusal
    /// stays Prepared so the operator can retry with the SAME material.
    func submit() async {
        lastError = nil
        guard case .prepared(var p) = state else { return }
        do {
            let client = try requireClient()
            if p.attestation_b64 == nil {
                p.attestation_b64 = try await AppAttest.attest(keyId: p.attest_key_id, transcript: p.transcript).attestation_b64
                try transition(.prepared(p))
            }
            let token = try await awaitToken()
            let submission = EnrolmentSubmission(
                code: p.code, platform: .ios, decision_key: p.decision_key_pub,
                evidence: IosAppAttestBoundDecisionKey(attest_key_id: p.attest_key_id, attestation_b64: p.attestation_b64!),
                push: registration(token))
            let grant: EnrolmentGrant
            do { grant = try await client.enrol(submission) }
            catch let e as WireError where e.outcomeUnknown {
                try transition(.submissionUnknown(p))
                throw e
            }
            // The attest key id is persisted as ENROLLED only now, after the server verified it.
            try transition(.enrolled(EnrolledDevice(
                enrollment_id: grant.enrollment_id, decision_key_blob: p.decision_key_blob,
                attest_key_id: p.attest_key_id, forwarded_apns_token: token)))
        } catch {
            lastError = error.localizedDescription
        }
    }

    /// After an ambiguous POST: re-read before generating anything new. The readback route is not
    /// modeled yet, so this refuses typed; the operator sees the state, not a fake answer.
    func resolveUnknownSubmission() async {
        lastError = nil
        guard case .submissionUnknown(let p) = state else { return }
        do {
            let client = try requireClient()
            let grant = try await client.readbackEnrolment(decisionKey: p.decision_key_pub)
            try transition(.enrolled(EnrolledDevice(
                enrollment_id: grant.enrollment_id, decision_key_blob: p.decision_key_blob,
                attest_key_id: p.attest_key_id, forwarded_apns_token: nil)))
        } catch {
            lastError = error.localizedDescription
        }
    }

    /// Retry the identical POST (same key, same attestation, same transcript): generates nothing new.
    func retrySubmission() async {
        guard case .submissionUnknown(let p) = state else { return }
        do { try transition(.prepared(p)) } catch { lastError = error.localizedDescription; return }
        await submit()
    }

    func startOver() {
        do { try transition(.unenrolled) } catch { lastError = error.localizedDescription }
    }

    // ── Authenticated reads ──────────────────────────────────────────────────────────────────
    /// An App Attest assertion over device_read_client_data for this path, now. No Face ID: reading
    /// is not deciding. requested_at is a Timestamp in the .dag's spelling (RFC 3339, UTC, seconds).
    private func readAuth(path: String) async throws -> ReadAuth {
        guard let e = state.enrolled else { throw WireError.configMissing("this phone is not enrolled") }
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime]
        let requestedAt = f.string(from: Date())
        let clientData = deviceReadClientData(path: path, enrollmentId: e.enrollment_id, requestedAt: requestedAt)
        let assertion = try await AppAttest.assert(keyId: e.attest_key_id, clientData: clientData)
        return ReadAuth(enrollmentId: e.enrollment_id, requestedAt: requestedAt, assertionB64: assertion.assertion_b64)
    }

    func fetch(_ escalationId: String) async throws -> FetchedRequest {
        let path = try Route.request(escalationId)
        return try await requireClient().fetch(path, try await readAuth(path: path))
    }

    // ── Inbox ────────────────────────────────────────────────────────────────────────────────
    enum RefreshResult { case changed, unchanged, failed }

    /// The list, re-read. The result is the real one, so the background-push callback can report
    /// .newData / .noData / .failed truthfully.
    @discardableResult
    func refresh() async -> RefreshResult {
        guard state.enrolled != nil else { return .unchanged }
        do {
            let fresh = try await requireClient().pending(try await readAuth(path: Route.pending))
            let changed = fresh != pending
            pending = fresh
            return changed ? .changed : .unchanged
        } catch {
            lastError = error.localizedDescription
            return .failed
        }
    }

    // ── Redemption ───────────────────────────────────────────────────────────────────────────
    /// Signs over exactly what the server returned. signature(for:) is where Face ID happens.
    func redeem(_ r: FetchedRequest, _ decision: ProposedDecision) async throws -> RedemptionOutcome {
        let client = try requireClient()
        guard case .enrolled(let e) = state else { throw WireError.configMissing("this phone is not enrolled") }
        let cap = r.capability(for: decision)
        let input = DeviceRedemptionSigningInput(
            audience: Protocol.audience,
            enrollment_id: e.enrollment_id,
            challenge: r.challenge,
            escalation_id: r.escalation_id,
            request_revision: r.request_revision,
            stored_request_text: r.stored_request_text,
            decision: decision,
            capability_text: cap.capability_text,
            capability_tag_b64url: cap.capability_tag_b64url
        )
        let bytes = deviceRedemptionSigningInput(input)
        let key: SecureEnclave.P256.Signing.PrivateKey
        do { key = try DecisionKey.load(e.decision_key_blob) }
        catch let k as DeviceKeyError {
            if case .keyInvalidated(let why) = k { try transition(.keyInvalidated(e, reason: why)) }
            throw k
        }
        let signature = try DecisionKey.sign(key, bytes)
        let proof = try await AppAttest.assert(keyId: e.attest_key_id, clientData: bytes)
        let outcome = try await client.redeem(SignedRedemption(signing_input: input, signature_b64url: signature.b64url, platform_proof: proof))
        // DeviceRedemptionOutcome arms that end this enrolment's authority, by their wire name.
        if outcome.outcome == "DeviceEnrollmentRevoked" || outcome.outcome == "DeviceEnrollmentUnknown" {
            try transition(.revoked(e, reason: outcome.message))
        }
        await refresh()
        return outcome
    }
}
