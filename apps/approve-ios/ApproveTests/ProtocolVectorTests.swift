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
            var capability_tag_b64url: String
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
    struct Read: Decodable {
        struct Input: Decodable {
            var path: String
            var enrollment_id: String
            var requested_at: String
        }
        var name: String
        var input: Input
        var expected: String
    }
    struct Envelope: Decodable {
        var name: String
        var body: String
    }
    var framing: String
    var redemption: [Redemption]
    var enrolment: [Enrolment]
    var read: [Read]
    var envelope: [Envelope]
}

final class ProtocolVectorTests: XCTestCase {
    /// Absence is a failure, never a skip: a green run with no vectors would establish nothing.
    private func load() throws -> Vectors {
        let url = try XCTUnwrap(Bundle(for: Self.self).url(forResource: "vectors", withExtension: "json"),
                                "vectors.json missing from the test bundle — the .dag witness has not emitted it")
        let v = try JSONDecoder().decode(Vectors.self, from: Data(contentsOf: url))
        XCTAssertFalse(v.redemption.isEmpty, "no redemption vectors")
        XCTAssertFalse(v.enrolment.isEmpty, "no enrolment vectors")
        XCTAssertFalse(v.read.isEmpty, "no read vectors")
        XCTAssertFalse(v.envelope.isEmpty, "no envelope vectors")
        return v
    }

    /// Framing counts CODE POINTS, not UTF-8 bytes and not grapheme clusters: "é" is 1, a flag emoji
    /// (two scalars) is 2, and a field containing "," or ":" frames unchanged.
    func testFramedFieldCountsUnicodeScalars() {
        XCTAssertEqual(Protocol.framedField("é"), "1:é,")
        XCTAssertEqual(Protocol.framedField("\u{1F1EC}\u{1F1E7}"), "2:\u{1F1EC}\u{1F1E7},")
        XCTAssertEqual(Protocol.framedField("a,b:c"), "5:a,b:c,")
        XCTAssertEqual(Protocol.framedField(""), "0:,")
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
                capability_tag_b64url: v.input.capability_tag_b64url
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

    /// Discriminating control on the builder itself: swapping the verb must move the bytes, and a
    /// field that contains the frame's own punctuation must not collide with a split field.
    func testBuilderDiscriminatesTheVerbAndFramingIsInjective() {
        let base = DeviceRedemptionSigningInput(
            audience: "a", enrollment_id: "e", challenge: RedemptionChallenge(expires_at: "x", nonce_hex: "n"),
            escalation_id: "s", request_revision: "r", stored_request_text: "t", decision: .approve,
            capability_text: "c", capability_tag_b64url: "g")
        var denied = base; denied.decision = .deny
        XCTAssertNotEqual(deviceRedemptionSigningInput(base), deviceRedemptionSigningInput(denied))
        XCTAssertNotEqual(Protocol.framed(["a,b"]), Protocol.framed(["a", "b"]))
        XCTAssertNotEqual(Protocol.framed(["1:a,"]), Protocol.framed(["a"]))
    }

    func testReadClientDataMatchesEveryVector() throws {
        for v in try load().read {
            let got = deviceReadClientData(path: v.input.path, enrollmentId: v.input.enrollment_id, requestedAt: v.input.requested_at)
            XCTAssertEqual(got, Data(v.expected.utf8), v.name)
        }
    }

    // ── Envelopes: the HTTP wire, matched as bytes ───────────────────────────────────────────
    private func envelope(_ name: String) throws -> String {
        try XCTUnwrap(try load().envelope.first { $0.name == name }?.body, "envelope vector \(name) missing")
    }

    /// Each request body the fixture carries is decoded by the app's strict reader and re-encoded
    /// by the app's encoder; the bytes must be the authority's. No input is typed here.
    func testEnrolmentRequestRoundTripsToTheFixtureBytes() throws {
        let body = try envelope("enrolment_request")
        XCTAssertEqual(WireEncode.enrolmentRequest(try WireDecode.enrolmentRequest(Data(body.utf8))), body)
    }

    func testSignedRedemptionRoundTripsToTheFixtureBytes() throws {
        let body = try envelope("signed_redemption")
        XCTAssertEqual(WireEncode.signedRedemption(try WireDecode.signedRedemption(Data(body.utf8))), body)
    }

    func testPushUpdateRoundTripsToTheFixtureBytes() throws {
        let body = try envelope("push_update")
        XCTAssertEqual(WireEncode.pushUpdate(try WireDecode.pushUpdate(Data(body.utf8))), body)
    }

    /// Every response body decodes strictly, with its declared members present and non-empty.
    func testResponsesDecode() throws {
        XCTAssertFalse(try WireDecode.enrolmentGrant(Data(try envelope("enrolment_grant").utf8)).enrollment_id.isEmpty)
        XCTAssertFalse(try WireDecode.pendingList(Data(try envelope("pending_list").utf8)).isEmpty)
        let f = try WireDecode.fetchedRequest(Data(try envelope("fetched_request").utf8))
        XCTAssertNotEqual(f.approve, f.deny)
        XCTAssertEqual(try WireDecode.enrolmentReadback(Data(try envelope("enrolment_readback").utf8)).standing, .active)
        XCTAssertFalse(try WireDecode.redemptionResponse(Data(try envelope("redemption_response").utf8)).outcome.isEmpty)
    }

    /// The strict reader refuses an unknown member, an empty string and an unadmitted kind.
    func testStrictReaderRefusesUnknownAndEmptyMembers() {
        XCTAssertThrowsError(try WireDecode.enrolmentGrant(Data(#"{"enrollment_id": "e", "extra": 1}"#.utf8)))
        XCTAssertThrowsError(try WireDecode.enrolmentGrant(Data(#"{"enrollment_id": ""}"#.utf8)))
        XCTAssertThrowsError(try WireDecode.pushUpdate(Data(#"{"kind": "fcm", "project": "p", "token": "t"}"#.utf8)))
    }

    /// device_push_update_client_data frames the exact push body; the enrolment id and time are the
    /// read vectors' inputs, so the only bytes compared are the authority's.
    func testPushUpdateClientDataMatchesTheFixture() throws {
        let read = try XCTUnwrap(try load().read.first).input
        let body = try envelope("push_update")
        let got = devicePushUpdateClientData(enrollmentId: read.enrollment_id, requestedAt: read.requested_at, pushBodyJson: body)
        XCTAssertEqual(got, Data(try envelope("push_update_client_data").utf8))
    }

    func testPathsMatchTheFixtureAndRefuseOutsideTheAlphabet() throws {
        let f = try WireDecode.fetchedRequest(Data(try envelope("fetched_request").utf8))
        XCTAssertEqual(try Route.request(f.escalation_id), try envelope("request_path"))
        let g = try WireDecode.enrolmentGrant(Data(try envelope("enrolment_grant").utf8))
        XCTAssertEqual(try Route.enrollment(g.enrollment_id), try envelope("enrollment_path"))
        XCTAssertThrowsError(try Route.request("a/b"))
        XCTAssertThrowsError(try Route.request("a%2Fb"))
        XCTAssertThrowsError(try Route.request(".."))
    }

    /// The emitter's escaping is serialize_json's: controls as \u00XX upper-case, "/" literal.
    func testEmitterEscapesLikeSerializeJson() {
        XCTAssertEqual(WireJson.string("a\u{1F}b/\"c\\").serialized, #""a\u001Fb/\"c\\""#)
        XCTAssertEqual(WireJson.object([("k", .array([.string("x")]))]).serialized, #"{"k": ["x"]}"#)
    }
}
