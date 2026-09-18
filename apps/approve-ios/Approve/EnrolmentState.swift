// THE DURABLE ENROLMENT STATE MACHINE. Every transition is persisted BEFORE the remote call that
// depends on it, so a crash, a lost answer or a relaunch resumes from a state that says exactly what
// the app has already done — and never generates a second key or a second attestation where the
// server may already hold the first.
//
//   Unenrolled
//     -> Prepared          keys generated, transcript fixed, attestation not yet accepted by the server
//     -> SubmissionUnknown the POST went out and no decodable answer came back
//     -> Enrolled          the server verified the attestation and issued enrollment_id
//     -> KeyInvalidated    the enclave refuses the key (biometric set changed, blob dead)
//     -> Revoked           the server says this enrolment no longer decides
import Foundation

/// Everything a retry needs, fixed at preparation. The App Attest key id and the transcript are
/// reused on retry so the attestation is over the SAME clientDataHash (Apple: retry attestKey with
/// the same key after a failed submission). decision_key_blob is the enclave's wrapped key.
struct PreparedEnrolment: Codable, Equatable {
    var code: String
    var decision_key_blob: Data
    var decision_key_pub: VerifyingKey
    var attest_key_id: String
    var transcript: Data
    /// The attestation, once generated, so a retry re-sends the same bytes.
    var attestation_b64: String?
}

struct EnrolledDevice: Codable, Equatable {
    var enrollment_id: String
    var decision_key_blob: Data
    /// Persisted here only once the server verified the attestation under it.
    var attest_key_id: String
    /// The last APNs token the server was told about, so a rotated token is forwarded once.
    var forwarded_apns_token: String?
}

enum EnrolmentState: Codable, Equatable {
    case unenrolled
    case prepared(PreparedEnrolment)
    case submissionUnknown(PreparedEnrolment)
    case enrolled(EnrolledDevice)
    case keyInvalidated(EnrolledDevice, reason: String)
    case revoked(EnrolledDevice, reason: String)

    var enrolled: EnrolledDevice? {
        if case .enrolled(let e) = self { return e }
        return nil
    }
}

enum EnrolmentStore {
    private static let account = "enrolment-state"

    static func load() throws -> EnrolmentState {
        guard let d = try KeychainItem.read(account) else { return .unenrolled }
        return try JSONDecoder().decode(EnrolmentState.self, from: d)
    }

    static func save(_ s: EnrolmentState) throws {
        try KeychainItem.write(account, try JSONEncoder().encode(s))
    }
}
