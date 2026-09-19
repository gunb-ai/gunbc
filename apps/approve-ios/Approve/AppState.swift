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
            await convergePushToken()
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

    // ── APNs: one serialized convergence loop over the latest desired token ──────────────────
    /// The token most recently delivered is the DESIRED token; the loop below converges the
    /// server onto it. One loop at a time (`converging`), and a completion records success only
    /// if the token it forwarded is still the desired one — otherwise it continues with the newer.
    private var converging = false

    func tokenDelivered(_ hex: String) {
        apnsToken = hex
        guard installed else { return }            // buffered until install()
        Task { await convergePushToken() }
    }

    /// PUT /approve/device/push until the server holds the desired token. Idempotent to call: a
    /// second caller while a loop runs returns at once and the running loop sees the new desired
    /// token on its next iteration. Runs after any adoption of Enrolled too, so a token buffered
    /// during SubmissionUnknown is forwarded.
    ///
    /// Exit discipline: the loop stops only when the server holds the desired token, when there is
    /// nothing to converge (not enrolled, no token), or when the request for the STILL-desired token
    /// failed — a failure for a token that was superseded while in flight continues at once with the
    /// newer one. On exit with desired != forwarded, the deferred check restarts the loop once a
    /// NEWER token has arrived (never a tight retry on the same failed token).
    func convergePushToken() async {
        guard !converging else { return }
        converging = true
        var failedToken: String?
        defer {
            converging = false
            if case .enrolled(let e) = state, let desired = apnsToken, e.forwarded_apns_token != desired, desired != failedToken {
                Task { await convergePushToken() }
            }
        }
        while case .enrolled(let e) = state, let desired = apnsToken, e.forwarded_apns_token != desired, desired != failedToken {
            do {
                let client = try requireClient()
                let body = WireEncode.pushUpdate(registration(desired))
                let requestedAt = Self.now()
                let auth = try await assertion(e, requestedAt: requestedAt,
                                               clientData: devicePushUpdateClientData(enrollmentId: e.enrollment_id, requestedAt: requestedAt, pushBodyJson: body))
                try await client.updatePush(bodyJson: body, auth)
                // Record success only if the token is STILL desired and the enrolment the update
                // was made for is STILL the current one; otherwise loop and re-derive.
                guard apnsToken == desired, case .enrolled(var now) = state, now.enrollment_id == e.enrollment_id,
                      now.attest_key_id == e.attest_key_id else { continue }
                now.forwarded_apns_token = desired
                try transition(.enrolled(now))
            } catch {
                lastError = "APNs token not forwarded: \(error.localizedDescription)"
                if apnsToken != desired { continue }   // superseded in flight: go on with the newer
                failedToken = desired                  // still desired: stop; a newer token restarts
            }
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
    /// generated), register for push, then PERSIST SubmissionUnknown and only then POST. From the
    /// moment the POST may have left the device the durable state says so: a crash, a transport or
    /// decode failure, a failed persist of Enrolled, or a refusal after a retry ("code already
    /// used") all leave SubmissionUnknown standing, and only an authenticated readback resolves it.
    func submit() async {
        lastError = nil
        let p: PreparedEnrolment
        switch state {
        case .prepared(var q):
            do {
                if q.attestation_b64 == nil {
                    q.attestation_b64 = try await AppAttest.attest(keyId: q.attest_key_id, transcript: q.transcript).attestation_b64
                    try transition(.prepared(q))
                }
            } catch { lastError = error.localizedDescription; return }
            p = q
        case .submissionUnknown(let q):
            p = q                                   // a retry re-sends the same material
        default:
            return
        }
        do {
            let client = try requireClient()
            let token = try await awaitToken()
            let submission = EnrolmentRequest(
                code: p.code, platform: .ios, decision_key: p.decision_key_pub,
                evidence: IosAppAttestBoundDecisionKey(attest_key_id: p.attest_key_id, attestation_b64: p.attestation_b64!),
                push: registration(token))
            try transition(.submissionUnknown(p))   // BEFORE the bytes leave
            let grant = try await client.enrol(submission)
            try adopt(EnrolledDevice(
                enrollment_id: grant.enrollment_id, decision_key_blob: p.decision_key_blob,
                attest_key_id: p.attest_key_id, forwarded_apns_token: token))
        } catch {
            // Whatever failed — transport, decode, a refusal, persisting Enrolled — the durable
            // state stays SubmissionUnknown; readback is the only way forward.
            lastError = error.localizedDescription
        }
    }

    /// The one place Enrolled is entered. The attest key id is persisted as ENROLLED only here,
    /// after the server verified it; then APNs registration is (re)requested and push-token
    /// convergence runs for any buffered or rotated token.
    private func adopt(_ e: EnrolledDevice) throws {
        try transition(.enrolled(e))
        UIApplication.shared.registerForRemoteNotifications()
        Task { await convergePushToken() }
    }

    /// The only resolution of SubmissionUnknown: GET /approve/device/enrollments/<enrollment_id>
    /// under the App Attest key just attested, keyed by enrollment_id_for_code over the persisted
    /// code. Active (and naming the expected id) adopts; revoked is terminal. EVERYTHING ELSE leaves
    /// SubmissionUnknown standing: the wire defines no absence response, and the server answers a
    /// bare refusal for unknown, unspent and unreadable alike, so a 404 is not proof of absence —
    /// "Retry the same submission" is the safe move for a true absence.
    func resolveUnknownSubmission() async {
        lastError = nil
        guard case .submissionUnknown(let p) = state else { return }
        let enrollmentId = enrollmentIdForCode(p.code)
        let probe = EnrolledDevice(enrollment_id: enrollmentId, decision_key_blob: p.decision_key_blob,
                                   attest_key_id: p.attest_key_id, forwarded_apns_token: nil)
        do {
            let client = try requireClient()
            let path = Route.enrollment(enrollmentId)
            let back = try await client.readback(path, try await readAuth(probe, path: path))
            guard back.enrollment_id == enrollmentId else {
                throw WireError.refused(at: "enrollment_id", cause: "readback names \(back.enrollment_id), expected \(enrollmentId)")
            }
            switch back.standing {
            case .active:
                try adopt(EnrolledDevice(
                    enrollment_id: back.enrollment_id, decision_key_blob: p.decision_key_blob,
                    attest_key_id: p.attest_key_id, forwarded_apns_token: nil))
            case .revoked:
                try transition(.revoked(probe, reason: "enrolment \(back.enrollment_id) is revoked"))
            }
        } catch {
            lastError = error.localizedDescription
        }
    }

    /// Re-send the identical POST from SubmissionUnknown (same key, same attestation, same
    /// transcript). The state does not leave SubmissionUnknown to do it.
    func retrySubmission() async { await submit() }

    /// Only from Prepared: nothing has left the device, so nothing can be stranded on the server.
    func startOver() {
        guard case .prepared = state else { return }
        do { try transition(.unenrolled) } catch { lastError = error.localizedDescription }
    }

    /// From a terminal state (key invalidated, revoked): the old enrolment decides nothing any more.
    func enrolAgain() {
        switch state {
        case .keyInvalidated, .revoked:
            do { try transition(.unenrolled) } catch { lastError = error.localizedDescription }
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
        let outcome = try await client.redeem(SignedRedemption(signing_input: input, signature: signature, platform_proof: proof))
        // DeviceRedemptionOutcome arms that end this enrolment's authority, by their wire name.
        if outcome.outcome == "DeviceEnrollmentRevoked" || outcome.outcome == "DeviceEnrollmentUnknown" {
            try transition(.revoked(e, reason: outcome.message))
        }
        await refresh()
        return outcome
    }
}
