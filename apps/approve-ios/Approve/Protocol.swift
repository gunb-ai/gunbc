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

/// extdeps.crypto.signature VerifyingKey on the wire: the point only. suite and encoding are the
/// enclave's fixed values (extdeps.apple.secure_enclave secure_enclave_signing_suite /
/// secure_enclave_public_key_encoding) and the server fills them from its enrolled key.
struct VerifyingKey: Codable, Equatable {
    var point_b64url: String
}

/// extdeps.crypto.signature SignatureBytes: fixed-width r||s, 64 octets (CryptoKit rawRepresentation),
/// travelling as signature_b64url.
struct SignatureBytes: Equatable {
    var b64url: String
}

/// enrolment_transcript: protocol, code, platform_wire, point_b64url. The one-time code the operator
/// types IS the challenge identifier; the login is NOT here — the server derives it from the code.
func enrolmentTranscript(code: String, decisionKey: VerifyingKey, platform: MobilePlatform) -> Data {
    Protocol.join([
        Protocol.enrolmentProtocol,
        code,
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

/// gunbc.auth.approval_device_redemption DeviceRedemptionSigningInput, field for field. On the wire the
/// challenge is flattened to challenge_expires_at + nonce_hex (the fixture's keys).
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

    enum CodingKeys: String, CodingKey {
        case audience, enrollment_id, challenge_expires_at, nonce_hex, escalation_id, request_revision
        case stored_request_text, decision, capability_text, capability_tag
    }

    init(audience: String, enrollment_id: String, challenge: RedemptionChallenge, escalation_id: String,
         request_revision: String, stored_request_text: String, decision: ProposedDecision,
         capability_text: String, capability_tag: String) {
        self.audience = audience; self.enrollment_id = enrollment_id; self.challenge = challenge
        self.escalation_id = escalation_id; self.request_revision = request_revision
        self.stored_request_text = stored_request_text; self.decision = decision
        self.capability_text = capability_text; self.capability_tag = capability_tag
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        audience = try c.decode(String.self, forKey: .audience)
        enrollment_id = try c.decode(String.self, forKey: .enrollment_id)
        challenge = RedemptionChallenge(expires_at: try c.decode(String.self, forKey: .challenge_expires_at),
                                        nonce_hex: try c.decode(String.self, forKey: .nonce_hex))
        escalation_id = try c.decode(String.self, forKey: .escalation_id)
        request_revision = try c.decode(String.self, forKey: .request_revision)
        stored_request_text = try c.decode(String.self, forKey: .stored_request_text)
        decision = try c.decode(ProposedDecision.self, forKey: .decision)
        capability_text = try c.decode(String.self, forKey: .capability_text)
        capability_tag = try c.decode(String.self, forKey: .capability_tag)
    }

    func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        try c.encode(audience, forKey: .audience)
        try c.encode(enrollment_id, forKey: .enrollment_id)
        try c.encode(challenge.expires_at, forKey: .challenge_expires_at)
        try c.encode(challenge.nonce_hex, forKey: .nonce_hex)
        try c.encode(escalation_id, forKey: .escalation_id)
        try c.encode(request_revision, forKey: .request_revision)
        try c.encode(stored_request_text, forKey: .stored_request_text)
        try c.encode(decision, forKey: .decision)
        try c.encode(capability_text, forKey: .capability_text)
        try c.encode(capability_tag, forKey: .capability_tag)
    }
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

/// gunbc.auth.approval_device_redemption SignedRedemption: the POST /approve/device/redeem body.
struct SignedRedemption: Codable, Equatable {
    var signing_input: DeviceRedemptionSigningInput
    var signature_b64url: String
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
