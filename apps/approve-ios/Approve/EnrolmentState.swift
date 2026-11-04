// THE DURABLE ENROLMENT STATE. Enrolment is one Google sign-in and one idempotent POST (ruling
// gunbc.design_ruling approval_enrolment_is_google_sign_in), so the phone persists nothing before the
// answer: an unanswered POST is re-sent byte-identical within the same sign-in, and a refused or
// abandoned sign-in is "sign in again". Which phase follows which answer is the generated
// EnrolmentMachine's (gunbc.approve_ios_enrolment_machine); this file keeps the payloads.
//
//   signedOut  no enrolment held (also what a key the enclave refuses reads as, after the key is discarded)
//   enrolled   the server verified the attestation and issued enrollment_id
//   revoked    the server says this enrolment no longer decides; only a sign-in with a NEW key leaves it
import Foundation

struct EnrolledDevice: Codable, Equatable {
    var enrollment_id: String
    var decision_key_blob: Data
    /// Persisted here only once the server verified the attestation under it.
    var attest_key_id: String
    /// The last APNs token the server was told about, so a rotated token is forwarded once.
    var forwarded_apns_token: String?
}

enum EnrolmentState: Codable, Equatable {
    case signedOut
    case enrolled(EnrolledDevice)
    case revoked(EnrolledDevice, reason: String)

    var enrolled: EnrolledDevice? {
        if case .enrolled(let e) = self { return e }
        return nil
    }
}

extension EnrolmentState {
    /// The generated machine's phase for this state; the payloads stay here.
    var phase: EnrolmentPhase {
        switch self {
        case .signedOut: return .signedOut
        case .enrolled: return .enrolled
        case .revoked: return .revoked
        }
    }
}

enum EnrolmentStore {
    private static let account = "enrolment-state"

    static func load() throws -> EnrolmentState {
        guard let d = try KeychainItem.read(account) else { return .signedOut }
        return try JSONDecoder().decode(EnrolmentState.self, from: d)
    }

    /// Signed out is the ABSENCE of the item, never a written record: there is nothing to keep.
    static func save(_ s: EnrolmentState) throws {
        if case .signedOut = s { try KeychainItem.delete(account); return }
        try KeychainItem.write(account, try JSONEncoder().encode(s))
    }
}
