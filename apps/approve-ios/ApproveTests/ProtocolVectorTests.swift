// The Swift byte builders against the vectors the .dag witness EMITS from device_redemption_signing_input
// and enrolment_transcript (dag/test/fixture/approval_device_redemption/vectors.json). No expected byte
// is written here: a hand-copied expectation would be a second authority for the join.
import XCTest
@testable import Approve

struct Vectors: Decodable {
    struct Redemption: Decodable {
        struct Input: Decodable {
            var audience: String
            var enrollment_id: String
            var challenge_expires_at: String
            var nonce_hex: String
            var escalation_id: String
            var request_revision: String
            var stored_request_text: String
            var decision: String
            var capability_text: String
            var capability_tag: String
        }
        var name: String
        var input: Input
        var expected: String
    }
    struct Enrolment: Decodable {
        struct Input: Decodable {
            var code: String
            var platform: String
            var point_b64url: String
        }
        var name: String
        var input: Input
        var expected: String
    }
    var separator_code_point: Int
    var redemption: [Redemption]
    var enrolment: [Enrolment]
}

final class ProtocolVectorTests: XCTestCase {
    /// Absence is a failure, never a skip: a green run with no vectors would establish nothing.
    private func load() throws -> Vectors {
        let url = try XCTUnwrap(Bundle(for: Self.self).url(forResource: "vectors", withExtension: "json"),
                                "vectors.json missing from the test bundle — the .dag witness has not emitted it")
        let v = try JSONDecoder().decode(Vectors.self, from: Data(contentsOf: url))
        XCTAssertFalse(v.redemption.isEmpty, "no redemption vectors")
        XCTAssertFalse(v.enrolment.isEmpty, "no enrolment vectors")
        return v
    }

    func testSeparatorIsTheCapabilitySigningFieldSeparator() throws {
        XCTAssertEqual(Int(Protocol.fieldSeparator), try load().separator_code_point)
    }

    func testRedemptionSigningInputMatchesEveryVector() throws {
        for v in try load().redemption {
            let decision = try XCTUnwrap(ProposedDecision(rawValue: v.input.decision), v.name)
            let input = DeviceRedemptionSigningInput(
                audience: v.input.audience,
                enrollment_id: v.input.enrollment_id,
                challenge: RedemptionChallenge(expires_at: v.input.challenge_expires_at, nonce_hex: v.input.nonce_hex),
                escalation_id: v.input.escalation_id,
                request_revision: v.input.request_revision,
                stored_request_text: v.input.stored_request_text,
                decision: decision,
                capability_text: v.input.capability_text,
                capability_tag: v.input.capability_tag
            )
            XCTAssertEqual(deviceRedemptionSigningInput(input), Data(v.expected.utf8), v.name)
        }
    }

    func testEnrolmentTranscriptMatchesEveryVector() throws {
        for v in try load().enrolment {
            let platform = try XCTUnwrap(MobilePlatform(rawValue: v.input.platform), v.name)
            let got = enrolmentTranscript(
                code: v.input.code,
                decisionKey: VerifyingKey(point_b64url: v.input.point_b64url),
                platform: platform
            )
            XCTAssertEqual(got, Data(v.expected.utf8), v.name)
        }
    }

    /// The wire spelling of the signing input is the fixture's flat key set: a round trip must
    /// preserve every field, and the challenge must flatten to challenge_expires_at + nonce_hex.
    func testSigningInputWireKeysAreTheFixtureKeys() throws {
        let input = DeviceRedemptionSigningInput(
            audience: "a", enrollment_id: "e", challenge: RedemptionChallenge(expires_at: "x", nonce_hex: "n"),
            escalation_id: "s", request_revision: "r", stored_request_text: "t", decision: .deny,
            capability_text: "c", capability_tag: "g")
        let json = try JSONSerialization.jsonObject(with: JSONEncoder().encode(input)) as? [String: Any]
        XCTAssertEqual(Set(json?.keys ?? []), ["audience", "enrollment_id", "challenge_expires_at", "nonce_hex",
            "escalation_id", "request_revision", "stored_request_text", "decision", "capability_text", "capability_tag"])
        XCTAssertEqual(try JSONDecoder().decode(DeviceRedemptionSigningInput.self, from: JSONEncoder().encode(input)), input)
    }

    /// Discriminating control on the builder itself: swapping the verb must move the bytes, and the
    /// join must place exactly one 0x1F between fields.
    func testBuilderDiscriminatesTheVerbAndJoinsWithUnitSeparator() {
        let base = DeviceRedemptionSigningInput(
            audience: "a", enrollment_id: "e", challenge: RedemptionChallenge(expires_at: "x", nonce_hex: "n"),
            escalation_id: "s", request_revision: "r", stored_request_text: "t", decision: .approve,
            capability_text: "c", capability_tag: "g")
        var denied = base; denied.decision = .deny
        XCTAssertNotEqual(deviceRedemptionSigningInput(base), deviceRedemptionSigningInput(denied))
        XCTAssertEqual(deviceRedemptionSigningInput(base).filter { $0 == 0x1F }.count, 10)
    }
}
