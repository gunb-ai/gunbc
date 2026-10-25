import Foundation
import SwiftUI
import CryptoKit
import UIKit
import UserNotifications

@MainActor
final class AppState: ObservableObject {
    /// Entering a phase drops the errors of operations that are not relevant in it (OperationErrors.enter).
    @Published private(set) var state: EnrolmentState = .signedOut {
        didSet { errors.enter(state.phase) }
    }
    @Published var pending: [PendingApproval] = []
    /// The escalation a tapped alert named (ApprovalPushHint.notification_id). It only ORDERS the
    /// inbox: the card shown is still one the server listed as pending, read over the authenticated
    /// route, and a locator naming nothing pending selects nothing.
    @Published var focused: String?

    /// The card the inbox shows first: the tapped request when it is pending, else the oldest.
    var front: PendingApproval? {
        if let f = focused, let p = pending.first(where: { $0.escalation_id == f }) { return p }
        return pending.first
    }
    /// One current error per operation (the generated OperationErrors): every attempt records its
    /// outcome, so the next success of an operation removes that operation's error.
    @Published private(set) var errors = OperationErrors()
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
            errors.record(.configuration, .failed(error.localizedDescription))
        }
    }

    func record(_ operation: AppOperation, _ outcome: OperationOutcome) {
        errors.record(operation, outcome)
    }

    /// Called once by the root view: read the persisted state, then register for APNs on every
    /// enrolled launch (a token rotates on reinstall/restore; the server must learn each one).
    func install() async {
        guard !installed else { return }
        installed = true
        do {
            state = try EnrolmentStore.load()
            record(.enrolmentRead, .succeeded)
        } catch { record(.enrolmentRead, .failed(error.localizedDescription)) }
        if state.enrolled != nil {
            UIApplication.shared.registerForRemoteNotifications()
            await convergePushToken()
        }
    }

    private func requireClient() throws -> Client {
        guard let client else { throw WireError.configMissing("no server configuration") }
        return client
    }

    private func transition(_ s: EnrolmentState) throws {
        try EnrolmentStore.save(s)
        state = s
    }

    // ── APNs: one serialized convergence loop over the latest desired token ──────────────────
    /// The token most recently delivered is the DESIRED token; the loop below converges the
    /// server onto it. One loop at a time (`converging`), and a completion records success only
    /// if the token it forwarded is still the desired one — otherwise it continues with the newer.
    private var converging = false

    func tokenDelivered(_ hex: String) {
        apnsToken = hex
        record(.apnsRegistration, .succeeded)
        guard installed else { return }            // buffered until install()
        Task { await convergePushToken() }
    }

    /// PUT /approve/device/push until the server holds the desired token. Idempotent to call: a
    /// second caller while a loop runs returns at once and the running loop sees the new desired
    /// token on its next iteration. Runs after any adoption of Enrolled too.
    ///
    /// Exit discipline: the loop stops only when the server holds the desired token, when there is
    /// nothing to converge (not enrolled, no token), or when the EXACT attempt that failed — this
    /// enrolment identity AND this token — is still what would be attempted next. A failure whose
    /// enrolment or token has since changed continues at once with the current pair, so a failed
    /// update for an old enrolment never suppresses the same token for a newly adopted one. After a
    /// stop, the restart is owned by tokenDelivered and signIn(): a newer token or a new enrolment
    /// starts a fresh loop; nothing retries the same failed (enrolment, token) pair.
    private struct PushAttempt: Equatable {
        var enrollmentId: String
        var attestKeyId: String
        var token: String
    }

    func convergePushToken() async {
        guard !converging else { return }
        converging = true
        var failedAttempt: PushAttempt?
        defer { converging = false }
        while case .enrolled(let e) = state, let desired = apnsToken, e.forwarded_apns_token != desired,
              PushAttempt(enrollmentId: e.enrollment_id, attestKeyId: e.attest_key_id, token: desired) != failedAttempt {
            let attempt = PushAttempt(enrollmentId: e.enrollment_id, attestKeyId: e.attest_key_id, token: desired)
            do {
                let client = try requireClient()
                let push = registration(client.config, desired)
                let body = WireEncode.pushUpdate(push)
                let requestedAt = Self.now()
                let auth = try await assertion(e, requestedAt: requestedAt,
                                               clientData: devicePushUpdateClientData(enrollmentId: e.enrollment_id, requestedAt: requestedAt, pushBodyJson: body))
                try await client.updatePush(push, auth)
                // Record success only if the token is STILL desired and the enrolment the update
                // was made for is STILL the current one; otherwise loop and re-derive.
                guard apnsToken == desired, case .enrolled(var now) = state, now.enrollment_id == e.enrollment_id,
                      now.attest_key_id == e.attest_key_id else { continue }
                now.forwarded_apns_token = desired
                try transition(.enrolled(now))
                record(.pushForward, .succeeded)
            } catch {
                record(.pushForward, .failed("APNs token not forwarded: \(error.localizedDescription)"))
                // Superseded in flight (token or enrolment changed): go on with the current pair.
                guard apnsToken == desired, case .enrolled(let now) = state,
                      now.enrollment_id == attempt.enrollmentId, now.attest_key_id == attempt.attestKeyId else { continue }
                failedAttempt = attempt                // exactly this pair failed: stop here
            }
        }
    }

    /// Every value from the refusing ServerConfig; no arm of this can fabricate a default.
    private func registration(_ c: ServerConfig, _ token: String) -> ApnsRegistration {
        ApnsRegistration(environment: c.apnsEnvironment, topic: c.apnsTopic, token: token)
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
    /// Google sign-in enrolment (gunbc.auth.approval_google_enrolment). Order: both device keys, the
    /// transcript over them, nonce = base64url(sha256(transcript)), Google sign-in with that nonce,
    /// attestation over the transcript, ONE POST. An unanswered POST is re-sent with the identical
    /// body (the server is idempotent); nothing is persisted before the answer. Which phase follows
    /// is the generated EnrolmentMachine's decision.
    func signIn() async {
        do {
            try AppAttest.requireSupported()
            let key = try DecisionKey.create()
            let pub = DecisionKey.verifyingKey(key)
            let attestKeyId = try await AppAttest.generateKey()
            let transcript = googleEnrolmentTranscript(decisionKey: pub, platform: .ios, attestKeyId: attestKeyId)
            let idToken = try await GoogleSignInFlow.idToken(nonce: googleEnrolmentNonce(transcript))
            let attestation = try await AppAttest.attest(keyId: attestKeyId, transcript: transcript).attestation_b64
            let client = try requireClient()
            let token = try await awaitToken()
            let submission = GoogleEnrolmentRequest(
                platform: .ios, decision_key: pub, attest_key_id: attestKeyId,
                attestation_b64: attestation, id_token: idToken)
            let grant: EnrolmentGrant
            do { grant = try await client.enrolGoogle(submission) }
            catch WireError.transport { grant = try await client.enrolGoogle(submission) }   // identical body, once
            // signIn always creates a fresh key, so the sign-in registered a NEW key.
            try apply(.signedIn(.newKey), enrolled: EnrolledDevice(
                enrollment_id: grant.enrollment_id, decision_key_blob: key.dataRepresentation,
                attest_key_id: attestKeyId, forwarded_apns_token: nil), push: registration(client.config, token))
            record(.signIn, .succeeded)
        } catch {
            record(.signIn, .failed(error.localizedDescription))
            try? apply(.refused, enrolled: nil, push: nil)
        }
    }

    /// Move to the phase the generated machine names for this answer; Enrolled is entered only with a device.
    private func apply(_ answer: EnrolmentAnswer, enrolled: EnrolledDevice?, push: ApnsRegistration?) throws {
        let next = EnrolmentMachine.next(state.phase, on: answer)
        switch (next, enrolled) {
        case (.enrolled, let e?):
            try transition(.enrolled(e))
            UIApplication.shared.registerForRemoteNotifications()
            Task { await convergePushToken() }
        case (.revoked, _):
            if case .enrolled(let e) = state { try transition(.revoked(e, reason: "the server reports this enrolment revoked")) }
        default:
            return
        }
    }

    /// Discard the dead enrolment and key: the phone reads as signed out.
    func signOut() {
        switch state {
        case .revoked, .enrolled:
            do {
                try transition(.signedOut)
                record(.signOut, .succeeded)
            } catch { record(.signOut, .failed(error.localizedDescription)) }
        default:
            return
        }
    }

    // ── Authenticated reads ──────────────────────────────────────────────────────────────────
    /// An App Attest assertion over device_read_client_data for this path, now. No Face ID: reading
    /// is not deciding. requested_at is a Timestamp in the .dag's spelling (RFC 3339, UTC, seconds).
    private static func now() -> String {
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime]
        return f.string(from: Date())
    }

    private func assertion(_ e: EnrolledDevice, requestedAt: String, clientData: Data) async throws -> ReadAuth {
        let a = try await AppAttest.assert(keyId: e.attest_key_id, clientData: clientData)
        return ReadAuth(enrollmentId: e.enrollment_id, requestedAt: requestedAt, assertionB64: a.assertion_b64)
    }

    private func readAuth(_ e: EnrolledDevice, path: String) async throws -> ReadAuth {
        let requestedAt = Self.now()
        return try await assertion(e, requestedAt: requestedAt,
                                   clientData: deviceReadClientData(path: path, enrollmentId: e.enrollment_id, requestedAt: requestedAt))
    }

    private func readAuth(path: String) async throws -> ReadAuth {
        guard let e = state.enrolled else { throw WireError.configMissing("this phone is not enrolled") }
        return try await readAuth(e, path: path)
    }

    func fetch(_ escalationId: String) async throws -> FetchedRequest {
        let path = Route.request(escalationId)
        return try await requireClient().fetch(path, try await readAuth(path: path))
    }

    // ── Inbox ────────────────────────────────────────────────────────────────────────────────
    enum RefreshResult { case changed, unchanged, failed }

    /// The list, re-read and REPLACED: the server answers exactly the requests that still need the
    /// operator (gunbc.auth.approval_decision_store pending_escalations), so nothing is kept from an
    /// earlier read. The result is the real one, so the background-push callback can report
    /// .newData / .noData / .failed truthfully.
    @discardableResult
    func refresh() async -> RefreshResult {
        guard state.enrolled != nil else { return .unchanged }
        do {
            let fresh = try await requireClient().pending(try await readAuth(path: Route.pending))
            let changed = fresh != pending
            pending = fresh
            record(.inboxRefresh, .succeeded)
            return changed ? .changed : .unchanged
        } catch {
            record(.inboxRefresh, .failed(error.localizedDescription))
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
            // A key the enclave refuses is dead: discard it; the phone reads as signed out.
            if case .keyInvalidated = k { try transition(.signedOut) }
            throw k
        }
        let signature = try DecisionKey.sign(key, bytes)
        let proof = try await AppAttest.assert(keyId: e.attest_key_id, clientData: bytes)
        let outcome = try await client.redeem(SignedRedemption(signing_input: input, signature: signature, platform_proof: proof))
        // DeviceRedemptionOutcome arms that end this enrolment's authority, by their wire name.
        if outcome.outcome == OutcomeName.enrollmentRevoked || outcome.outcome == OutcomeName.enrollmentUnknown {
            try transition(.revoked(e, reason: outcome.message))
        }
        await refresh()
        return outcome
    }
}
