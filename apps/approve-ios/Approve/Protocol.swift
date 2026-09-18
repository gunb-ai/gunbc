// gunbc.auth.approval_device_redemption, realized in Swift. Every constant and field order here is a
// transcription of that module; the .dag is the authority and this file invents nothing. The bytes
// the two builders produce are checked against vectors the .dag witness emits (ApproveTests).
import Foundation

enum Protocol {
    /// gunbc.auth.approval_capability signing_field_separator = from_code_point(31).
    static let fieldSeparator: UInt8 = 0x1F
    /// gunbc.auth.approval_device_redemption approval_enrolment_protocol.
    static let enrolmentProtocol = "gunbc.approval-enrolment.v1"
    /// gunbc.auth.approval_device_redemption approval_redemption_protocol.
    static let redemptionProtocol = "gunbc.approval-redemption.v1"
    /// gunbc.auth.approval_device_redemption approval_device_audience.
    static let audience = "gunbc.roadmap_serve/device-redemption"

    /// The one join both builders use: UTF-8 of each field, separated by 0x1F, nothing terminal.
    static func join(_ fields: [String]) -> Data {
        var out = Data()
        for (i, f) in fields.enumerated() {
            if i > 0 { out.append(fieldSeparator) }
            out.append(contentsOf: Array(f.utf8))
        }
        return out
    }
}

/// gunbc.auth.approval_device_redemption MobilePlatform, spelled by platform_wire.
enum MobilePlatform: String, Codable {
    case ios, android
}

/// gunbc.auth.approval_capability ProposedDecision, spelled by proposed_decision_name.
enum ProposedDecision: String, Codable {
    case approve, deny
}

/// extdeps.crypto.signature VerifyingKey. suite and encoding are the enclave's fixed values
/// (extdeps.apple.secure_enclave secure_enclave_signing_suite / secure_enclave_public_key_encoding).
struct VerifyingKey: Codable, Equatable {
    var suite: String = "ECDSA-P256-SHA256"
    var encoding: String = "sec1-uncompressed"
    var point_b64url: String
}

/// extdeps.crypto.signature SignatureBytes: fixed-width r||s, 64 octets (CryptoKit rawRepresentation).
struct SignatureBytes: Codable, Equatable {
    var suite: String = "ECDSA-P256-SHA256"
    var encoding: String = "p1363-fixed-width"
    var b64url: String
}

/// gunbc.auth.approval_device_redemption EnrolmentChallenge. The server issues it to a login; the app
/// only ever sees the code the operator types plus the challenge_id the server pairs with it.
struct EnrolmentChallenge: Codable, Equatable {
    var challenge_id: String
    var code: String
}

/// enrolment_transcript: protocol, challenge_id, code, platform_wire, point_b64url. The login is NOT
/// here — the server derives it from the challenge.
func enrolmentTranscript(challenge: EnrolmentChallenge, decisionKey: VerifyingKey, platform: MobilePlatform) -> Data {
    Protocol.join([
        Protocol.enrolmentProtocol,
        challenge.challenge_id,
        challenge.code,
        platform.rawValue,
        decisionKey.point_b64url,
    ])
}

/// gunbc.auth.approval_device_redemption RedemptionChallenge: stateless. nonce_hex is the server's
/// MAC over redemption_challenge_message(escalation_id, request_revision, enrollment_id, expires_at);
/// the app carries both back untouched.
struct RedemptionChallenge: Codable, Equatable {
    var expires_at: String
    var nonce_hex: String
}

/// gunbc.auth.approval_device_redemption DeviceRedemptionSigningInput, field for field.
struct DeviceRedemptionSigningInput: Codable, Equatable {
    var audience: String
    var enrollment_id: String
    var challenge: RedemptionChallenge
    var escalation_id: String
    var request_revision: String
    /// stored_request_json of the record displayed, byte for byte as the server returned it.
    var stored_request_text: String
    var decision: ProposedDecision
    var capability_text: String
    var capability_tag: String
}

/// device_redemption_signing_input. Order is the .dag's: the challenge contributes expires_at then
/// nonce_hex, in that position.
func deviceRedemptionSigningInput(_ i: DeviceRedemptionSigningInput) -> Data {
    Protocol.join([
        Protocol.redemptionProtocol,
        i.audience,
        i.enrollment_id,
        i.challenge.expires_at,
        i.challenge.nonce_hex,
        i.escalation_id,
        i.request_revision,
        i.stored_request_text,
        i.decision.rawValue,
        i.capability_text,
        i.capability_tag,
    ])
}

/// PlatformRedemptionProof, iOS arm only: an App Attest assertion over the same signing input.
struct IosAppAttestAssertion: Codable, Equatable {
    var assertion_b64: String
}

/// gunbc.auth.approval_device_redemption SignedRedemption.
struct SignedRedemption: Codable, Equatable {
    var signing_input: DeviceRedemptionSigningInput
    var signature: SignatureBytes
    var platform_proof: IosAppAttestAssertion
}

/// PlatformEnrollmentEvidence, iOS arm: IosAppAttestBoundDecisionKey.
struct IosAppAttestBoundDecisionKey: Codable, Equatable {
    var attest_key_id: String
    var attestation_b64: String
}

/// PushRegistration ApnsRegistration arm. topic is the bundle id (extdeps.apple.apns ApnsTopic).
struct ApnsRegistration: Codable, Equatable {
    var environment: String
    var topic: String
    var token: String
}

/// ApprovalPushHint: the push carries one opaque locator and nothing that is decided.
struct ApprovalPushHint: Codable, Equatable {
    var notification_id: String
}

enum Base64Url {
    static func encode(_ d: Data) -> String {
        d.base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "=", with: "")
    }
}
