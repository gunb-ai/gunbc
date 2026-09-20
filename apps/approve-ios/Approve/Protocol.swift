// gunbc.auth.approval_device_wire, realized in Swift. Every constant, framing rule and field order
// here is a transcription of that module; the .dag is the authority and this file invents nothing.
// The bytes the builders produce are checked against vectors the .dag witness emits (ApproveTests).
// The HTTP spelling of these records (JSON bodies, headers, paths) is Wire.swift.
import Foundation

enum Protocol {
    /// gunbc.auth.approval_device_wire approval_enrolment_protocol.
    static let enrolmentProtocol = "gunbc.approval-enrolment.v1"
    /// gunbc.auth.approval_device_wire approval_redemption_protocol.
    static let redemptionProtocol = "gunbc.approval-redemption.v1"
    /// gunbc.auth.approval_device_wire approval_device_read_protocol.
    static let readProtocol = "gunbc.approval-device-read.v1"
    /// gunbc.auth.approval_device_wire approval_device_push_protocol.
    static let pushProtocol = "gunbc.approval-device-push.v1"
    /// gunbc.auth.approval_device_wire approval_device_audience.
    static let audience = "gunbc.roadmap_serve/device-redemption"

    /// framed_field: "<n>:<field>," where n is the field's length in Unicode code points
    /// (string_length in .dag = String.unicodeScalars.count here), the comma part of the frame.
    static func framedField(_ f: String) -> String {
        "\(f.unicodeScalars.count):\(f),"
    }

    /// framed: the frames concatenated with nothing between them. Injective for any field content,
    /// which a separator join is not (capability_text is itself a 0x1F join). Bytes are UTF-8.
    static func framed(_ fields: [String]) -> Data {
        Data(fields.map(framedField).joined().utf8)
    }
}

/// gunbc.auth.approval_device_wire MobilePlatform, spelled by platform_wire.
enum MobilePlatform: String {
    case ios, android
}

/// gunbc.auth.approval_capability ProposedDecision, spelled by proposed_decision_name.
enum ProposedDecision: String {
    case approve, deny
}

/// extdeps.crypto.signature VerifyingKey. The suite and encoding travel explicitly
/// (signature_suite_wire, public_key_encoding_wire) and are the enclave's only values
/// (extdeps.apple.secure_enclave secure_enclave_signing_suite / secure_enclave_public_key_encoding).
struct VerifyingKey: Equatable, Codable {  // Codable only for the local keychain record; the wire spelling is WireEncode.verifyingKey
    static let suite = "ECDSA-P256-SHA256"
    static let encoding = "SEC1-uncompressed"
    var point_b64url: String
}

/// extdeps.crypto.signature SignatureBytes: fixed-width r||s, 64 octets (CryptoKit rawRepresentation),
/// with its suite and encoding explicit (signature_encoding_wire).
struct SignatureBytes: Equatable {
    static let suite = "ECDSA-P256-SHA256"
    static let encoding = "P1363-fixed-width"
    var b64url: String
}

/// enrolment_transcript = framed([protocol, code, platform_wire, point_b64url]). The one-time code
/// the operator types IS the challenge identifier; the login is NOT here — the server derives it.
func enrolmentTranscript(code: String, decisionKey: VerifyingKey, platform: MobilePlatform) -> Data {
    Protocol.framed([
        Protocol.enrolmentProtocol,
        code,
        platform.rawValue,
        decisionKey.point_b64url,
    ])
}

/// gunbc.auth.approval_device_wire enrollment_id_for_code: the enrolment an accepted code produces is
/// named by the code, so a lost enrolment answer costs nothing the app cannot re-derive.
func enrollmentIdForCode(_ code: String) -> String { "enr-" + code }

/// gunbc.auth.approval_device_wire RedemptionChallenge: stateless. nonce_hex is the server's
/// MAC over redemption_challenge_message(escalation_id, request_revision, enrollment_id, expires_at);
/// the app carries both back untouched.
struct RedemptionChallenge: Equatable {
    var expires_at: String
    var nonce_hex: String
}

/// gunbc.auth.approval_device_wire DeviceRedemptionSigningInput, field for field.
struct DeviceRedemptionSigningInput: Equatable {
    var audience: String
    var enrollment_id: String
    var challenge: RedemptionChallenge
    var escalation_id: String
    var request_revision: String
    /// stored_request_json of the record displayed, byte for byte as the server returned it.
    var stored_request_text: String
    var decision: ProposedDecision
    var capability_text: String
    /// The capability's canonical base64url tag exactly as issued.
    var capability_tag_b64url: String
}

/// device_redemption_signing_input = framed([...]). Order is the .dag's: the challenge contributes
/// expires_at then nonce_hex, in that position.
func deviceRedemptionSigningInput(_ i: DeviceRedemptionSigningInput) -> Data {
    Protocol.framed([
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
        i.capability_tag_b64url,
    ])
}

/// device_read_client_data = framed([protocol, path, enrollment_id, requested_at]): what the App
/// Attest assertion on an authenticated GET commits to (no Face ID — reading is not deciding).
func deviceReadClientData(path: String, enrollmentId: String, requestedAt: String) -> Data {
    Protocol.framed([Protocol.readProtocol, path, enrollmentId, requestedAt])
}

/// device_push_update_client_data = framed([protocol, push path, enrollment_id, requested_at,
/// push_update_json]): the token is inside the client data, so an assertion made for one token
/// cannot carry another.
func devicePushUpdateClientData(enrollmentId: String, requestedAt: String, pushBodyJson: String) -> Data {
    Protocol.framed([Protocol.pushProtocol, Route.push, enrollmentId, requestedAt, pushBodyJson])
}

/// PlatformRedemptionProof, iOS arm (wire kind "ios_app_attest_assertion").
struct IosAppAttestAssertion: Equatable {
    var assertion_b64: String
}

/// gunbc.auth.approval_device_wire SignedRedemption: the POST /approve/device/redeem body.
struct SignedRedemption: Equatable {
    var signing_input: DeviceRedemptionSigningInput
    var signature: SignatureBytes
    var platform_proof: IosAppAttestAssertion
}

/// PresentedEnrollmentEvidence, iOS arm: IosAppAttestAttestation (wire kind "ios_app_attest").
struct IosAppAttestBoundDecisionKey: Equatable {
    var attest_key_id: String
    var attestation_b64: String
}

/// PushRegistration ApnsRegistration arm (wire kind "apns"); environment is apns_environment_wire.
struct ApnsRegistration: Equatable {
    var environment: String
    var topic: String
    var token: String
}

/// gunbc.auth.approval_device_wire ApprovalPushHint: one opaque locator, nothing that is decided.
struct ApprovalPushHint: Equatable {
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
