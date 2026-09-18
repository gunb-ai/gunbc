import Foundation
import SwiftUI
import CryptoKit
import UIKit
import UserNotifications

/// What this phone has enrolled. Persisted; the enclave key itself is loaded from the keychain blob.
struct Enrolment: Codable, Equatable {
    var enrollment_id: String
    var attest_key_id: String
}

@MainActor
final class AppState: ObservableObject {
    @Published var enrolment: Enrolment?
    @Published var pending: [PendingApproval] = []
    @Published var lastError: String?
    /// The APNs device token, delivered asynchronously after registration; enrolment waits for it.
    @Published var apnsToken: String?

    private static let enrolmentDefault = "ai.gunb.approve.enrolment"
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
        if let d = UserDefaults.standard.data(forKey: Self.enrolmentDefault) {
            enrolment = try? JSONDecoder().decode(Enrolment.self, from: d)
        }
    }

    private func requireClient() throws -> Client {
        guard let client else { throw WireError.configMissing(lastError ?? "no server configuration") }
        return client
    }

    // ── Enrolment ────────────────────────────────────────────────────────────────────────────
    /// The order is the protocol's: refuse without App Attest; enclave key; App Attest key; attest
    /// over SHA256(enrolment_transcript); APNs registration; POST.
    func enrol(code: String) async {
        lastError = nil
        do {
            let client = try requireClient()
            try AppAttest.requireSupported()
            let key = try DecisionKey.create()
            let decisionKey = DecisionKey.verifyingKey(key)
            let attestKeyId = try await AppAttest.generateKey()
            let transcript = enrolmentTranscript(code: code, decisionKey: decisionKey, platform: .ios)
            let evidence = try await AppAttest.attest(keyId: attestKeyId, transcript: transcript)
            let token = try await registerForPush()
            let push = ApnsRegistration(
                environment: client.config.apnsEnvironment,
                topic: Bundle.main.bundleIdentifier ?? "ai.gunb.approve",
                token: token
            )
            let grant = try await client.enrol(EnrolmentSubmission(
                code: code, platform: .ios, decision_key: decisionKey, evidence: evidence, push: push
            ))
            let e = Enrolment(enrollment_id: grant.enrollment_id, attest_key_id: attestKeyId)
            UserDefaults.standard.set(try JSONEncoder().encode(e), forKey: Self.enrolmentDefault)
            enrolment = e
        } catch {
            lastError = error.localizedDescription
        }
    }

    private func registerForPush() async throws -> String {
        let granted = try await UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound])
        guard granted else { throw WireError.configMissing("notifications were not permitted") }
        UIApplication.shared.registerForRemoteNotifications()
        // The token arrives on the app delegate; wait for it rather than enrolling without push.
        for _ in 0..<50 {
            if let t = apnsToken { return t }
            try await Task.sleep(nanoseconds: 200_000_000)
        }
        throw WireError.configMissing("APNs did not deliver a device token")
    }

    // ── Authenticated reads ──────────────────────────────────────────────────────────────────
    /// An App Attest assertion over device_read_client_data for this path, now. No Face ID: reading
    /// is not deciding. requested_at is a Timestamp in the .dag's spelling (RFC 3339, UTC, seconds).
    private func readAuth(path: String) async throws -> ReadAuth {
        guard let enrolment else { throw WireError.configMissing("this phone is not enrolled") }
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime]
        let requestedAt = f.string(from: Date())
        let clientData = deviceReadClientData(path: path, enrollmentId: enrolment.enrollment_id, requestedAt: requestedAt)
        let assertion = try await AppAttest.assert(keyId: enrolment.attest_key_id, clientData: clientData)
        return ReadAuth(enrollmentId: enrolment.enrollment_id, requestedAt: requestedAt, assertionB64: assertion.assertion_b64)
    }

    func fetch(_ escalationId: String) async throws -> FetchedRequest {
        let path = Route.request(escalationId)
        return try await requireClient().fetch(escalationId, try await readAuth(path: path))
    }

    // ── Inbox ────────────────────────────────────────────────────────────────────────────────
    func refresh() async {
        do { pending = try await requireClient().pending(try await readAuth(path: Route.pending)) }
        catch { lastError = error.localizedDescription }
    }

    // ── Redemption ───────────────────────────────────────────────────────────────────────────
    /// Signs over exactly what the server returned. signature(for:) is where Face ID happens.
    func redeem(_ r: FetchedRequest, _ decision: ProposedDecision) async throws -> RedemptionOutcome {
        let client = try requireClient()
        guard let enrolment else { throw WireError.configMissing("this phone is not enrolled") }
        guard let key = try DecisionKey.load() else { throw WireError.configMissing("decision key missing; re-enrol") }
        let cap = r.capability(for: decision)
        let input = DeviceRedemptionSigningInput(
            audience: Protocol.audience,
            enrollment_id: enrolment.enrollment_id,
            challenge: r.challenge,
            escalation_id: r.escalation_id,
            request_revision: r.request_revision,
            stored_request_text: r.stored_request_text,
            decision: decision,
            capability_text: cap.capability_text,
            capability_tag_hex: cap.capability_tag_hex
        )
        let bytes = deviceRedemptionSigningInput(input)
        let signature = try DecisionKey.sign(key, bytes)
        let proof = try await AppAttest.assert(keyId: enrolment.attest_key_id, clientData: bytes)
        let outcome = try await client.redeem(SignedRedemption(signing_input: input, signature_b64url: signature.b64url, platform_proof: proof))
        await refresh()
        return outcome
    }
}
