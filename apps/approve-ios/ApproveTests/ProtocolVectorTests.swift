// The Swift byte builders against the vectors the .dag witness EMITS from device_redemption_signing_input
// and google_enrolment_transcript and google_enrolment_nonce (dag/test/fixture/approval_device_redemption/vectors.json). No expected byte
// is written here: a hand-copied expectation would be a second authority for the join.
// GENERATED from gunbc.approve_ios_swift_protocol_vector_tests from the Swift grammar rows (v2.extdeps.languages.swift.rows); do not edit.
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
            var attest_key_id: String
            var platform: String
            var point_b64url: String
        }

        var name: String
        var input: Input
        var expected: String
        /// google_enrolment_nonce over `expected`, emitted by the .dag.
        var nonce: String
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

    struct Refusal: Decodable {
        var name: String
        var body: String
        var at: String
        var cause: String
    }

    struct PathSegment: Decodable {
        var input: String
        var encoded: String
    }

    struct Escape: Decodable {
        var input: String
        var string_json: String
        var object_json: String
    }

    struct Surface: Decodable {
        var name: String
        var value: String
    }

    struct StoredRequest: Decodable {
        struct Input: Decodable {
            var requester: String
            var purpose: String
            var destructive: Bool
            var expires_at: String
        }

        var name: String
        var input: Input
        /// stored_request_json over a StoredApprovalRequest carrying `input`, emitted by the witness.
        var json: String
    }

    var framing: String
    var redemption: [Redemption]
    var enrolment: [Enrolment]
    var read: [Read]
    var envelope: [Envelope]
    /// refusal: malformed push-update bodies with the location and cause the server's decoder gives them.
    var refusal: [Refusal]
    /// path_segment: input -> encoded, emitted by the .dag path_segment fold.
    var path_segment: [PathSegment]
    /// surface: the one method, and the route paths, by name.
    var surface: [Surface]
    /// escape: characters the emitter has to escape, with the bytes serialize_json gives them.
    /// Optional in the Codable so that a fixture predating the section (it is emitted by
    /// gunbc.auth.approval_device_redemption_fixtures escape_specimens) fails ONLY the test that
    /// reads it, never the whole suite; that test refuses nil loudly.
    var escape: [Escape]?
    /// stored_request: the store's own rendering, so the detail screen's reader is joined to it.
    /// Optional in the Codable so that a fixture predating the section (it is emitted by
    /// gunbc.auth.approval_device_redemption_fixtures stored_request_specimens) fails ONLY the test
    /// that reads it, never the whole suite; that test refuses nil loudly.
    var stored_request: [StoredRequest]?
}

final class ProtocolVectorTests: XCTestCase {
    /// Absence is a failure, never a skip: a green run with no vectors would establish nothing.
    private func load() throws -> Vectors {
        let url = try XCTUnwrap(Bundle(for: Self.self).url(forResource: "vectors", withExtension: "json"), "vectors.json missing from the test bundle \u{2014} the .dag witness has not emitted it")
        let v = try JSONDecoder().decode(Vectors.self, from: Data(contentsOf: url))
        XCTAssertFalse(v.redemption.isEmpty, "no redemption vectors")
        XCTAssertFalse(v.enrolment.isEmpty, "no enrolment vectors")
        XCTAssertFalse(v.read.isEmpty, "no read vectors")
        XCTAssertFalse(v.envelope.isEmpty, "no envelope vectors")
        XCTAssertFalse(v.refusal.isEmpty, "no refusal vectors")
        XCTAssertFalse(v.path_segment.isEmpty, "no path_segment vectors")
        XCTAssertFalse(v.surface.isEmpty, "no surface vectors")
        return v
    }

    /// Framing counts CODE POINTS, not UTF-8 bytes and not grapheme clusters: "é" is 1, a flag emoji
    /// (two scalars) is 2, and a field containing "," or ":" frames unchanged.
    func testFramedFieldCountsUnicodeScalars() {
        XCTAssertEqual(Protocol.framedField("\u{E9}"), "1:\u{E9},")
        XCTAssertEqual(Protocol.framedField("\u{1F1EC}\u{1F1E7}"), "2:\u{1F1EC}\u{1F1E7},")
        XCTAssertEqual(Protocol.framedField("a,b:c"), "5:a,b:c,")
        XCTAssertEqual(Protocol.framedField(""), "0:,")
    }

    func testRedemptionSigningInputMatchesEveryVector() throws {
        for v in try load().redemption {
            let decision = try XCTUnwrap(ProposedDecision(rawValue: v.input.decision), v.name)
            let input = DeviceRedemptionSigningInput(audience: v.input.audience, enrollment_id: v.input.enrollment_id, challenge: RedemptionChallenge(expires_at: v.input.challenge_expires_at, nonce_hex: v.input.nonce_hex), escalation_id: v.input.escalation_id, request_revision: v.input.request_revision, stored_request_text: v.input.stored_request_text, decision: decision, capability_text: v.input.capability_text, capability_tag_b64url: v.input.capability_tag_b64url)
            XCTAssertEqual(deviceRedemptionSigningInput(input), Data(v.expected.utf8), v.name)
        }
    }

    /// google_enrolment_transcript and google_enrolment_nonce, byte for byte: the nonce the app hands Google is base64url(sha256(transcript)) as the .dag computes it.
    func testEnrolmentTranscriptAndNonceMatchEveryVector() throws {
        for v in try load().enrolment {
            let platform = try XCTUnwrap(MobilePlatform(rawValue: v.input.platform), v.name)
            let got = googleEnrolmentTranscript(decisionKey: VerifyingKey(point_b64url: v.input.point_b64url), platform: platform, attestKeyId: v.input.attest_key_id)
            XCTAssertEqual(got, Data(v.expected.utf8), v.name)
            XCTAssertEqual(googleEnrolmentNonce(got), v.nonce, v.name)
        }
    }

    /// Discriminating control on the builder itself: swapping the verb must move the bytes, and a
    /// field that contains the frame's own punctuation must not collide with a split field.
    func testBuilderDiscriminatesTheVerbAndFramingIsInjective() {
        let base = DeviceRedemptionSigningInput(audience: "a", enrollment_id: "e", challenge: RedemptionChallenge(expires_at: "x", nonce_hex: "n"), escalation_id: "s", request_revision: "r", stored_request_text: "t", decision: .approve, capability_text: "c", capability_tag_b64url: "g")
        var denied = base
        denied.decision = .deny
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
        XCTAssertEqual(WireEncode.googleEnrolmentRequest(try WireDecode.googleEnrolmentRequest(Data(body.utf8))), body)
    }

    func testSignedRedemptionRoundTripsToTheFixtureBytes() throws {
        let body = try envelope("signed_redemption")
        XCTAssertEqual(WireEncode.signedRedemption(try WireDecode.signedRedemption(Data(body.utf8))), body)
    }

    func testPushUpdateRoundTripsToTheFixtureBytes() throws {
        let body = try envelope("push_update")
        XCTAssertEqual(WireEncode.pushUpdate(try WireDecode.pushUpdate(Data(body.utf8))), body)
    }

    /// The read-authentication body and the push-update envelope { auth, push } are ENCODED only by
    /// the app (the server decodes them), so they are checked against the fixture bytes directly
    /// from the fixture's own field values.
    func testReadAuthenticationEncodesToTheFixtureBytes() throws {
        let body = try envelope("read_authentication")
        let auth = ReadAuth(enrollmentId: "genr-ns6JWpS7rWx_HTwBdUWdBNszEYI7obJE1rjh0AdEfo4=", requestedAt: "2026-09-18T12:00:30Z", assertionB64: "omlzaWduYXR1cmU")
        XCTAssertEqual(WireEncode.readRequest(auth), body)
        let pushBody = try envelope("push_update")
        let push = try WireDecode.pushUpdate(Data(pushBody.utf8))
        XCTAssertEqual(WireEncode.pushUpdateRequest(auth, push), try envelope("push_update_request"))
    }

    /// Every response body decodes strictly, with its declared members present and non-empty.
    func testResponsesDecode() throws {
        XCTAssertFalse(try WireDecode.enrolmentGrant(Data(try envelope("enrolment_grant").utf8)).enrollment_id.isEmpty)
        XCTAssertFalse(try WireDecode.pendingList(Data(try envelope("pending_list").utf8)).isEmpty)
        let f = try WireDecode.fetchedRequest(Data(try envelope("fetched_request").utf8))
        XCTAssertNotEqual(f.approve, f.deny)
        XCTAssertEqual(try WireDecode.enrolmentReadback(Data(try envelope("enrolment_readback").utf8)).standing, .active)
        // Every redemption_response_* envelope decodes to an outcome name the app spells (an arm
        // renamed in the .dag reds here), and the two names the app BRANCHES on are joined to their
        // own envelopes, so a renamed revoked/unknown arm cannot leave the phone a live decider.
        let responses = try load().envelope.filter { $0.name.hasPrefix("redemption_response_") }
        XCTAssertFalse(responses.isEmpty, "no redemption_response_* envelopes")
        for r in responses {
            let o = try WireDecode.redemptionResponse(Data(r.body.utf8))
            XCTAssertTrue(OutcomeName.all.contains(o.outcome), r.name + ": unspelled outcome " + o.outcome)
            XCTAssertFalse(o.message.isEmpty, r.name)
        }
        XCTAssertEqual(try WireDecode.redemptionResponse(Data(try envelope("redemption_response_enrollment_revoked").utf8)).outcome, OutcomeName.enrollmentRevoked)
        XCTAssertEqual(try WireDecode.redemptionResponse(Data(try envelope("redemption_response_enrollment_unknown").utf8)).outcome, OutcomeName.enrollmentUnknown)
    }

    /// A malformed push-update body is refused at the location and with the cause the SERVER's decoder
    /// gives it (refusal vectors are decode_push_update's own answers), so a cause or a path root the
    /// transcription restates differently reds here.
    func testPushUpdateRefusalsMatchTheServer() throws {
        for v in try load().refusal {
            do {
                _ = try WireDecode.pushUpdate(Data(v.body.utf8))
                XCTFail("refusal vector \(v.name) was accepted")
            } catch {
                XCTAssertEqual(error.localizedDescription, WireError.refused(at: v.at, cause: v.cause).localizedDescription, v.name)
            }
        }
    }

    /// One envelope per arm of the enrol route's typed refusal (enrolment_refusal_<label>): the arms the
    /// vectors cover are exactly the arms the app spells, and each decodes to the arm its name carries, so an
    /// arm added to the wire without a vector (or renamed) reds here.
    func testEveryEnrolmentRefusalArmDecodes() throws {
        let prefix = "enrolment_refusal_"
        let arms = try load().envelope.filter { $0.name.hasPrefix(prefix) }
        XCTAssertEqual(Set(arms.map { String($0.name.dropFirst(prefix.count)) }), Set(GoogleEnrolmentRefusal.labels))
        for a in arms {
            let kind = try WireDecode.googleEnrolmentRefusal(Data(a.body.utf8))
            XCTAssertEqual(prefix + kind, a.name)
        }
    }

    /// The detail screen's strict read, joined to the store's own rendering: every stored_request
    /// vector (stored_request_json over the witness's input) decodes to exactly that input. No
    /// rendering is typed here.
    func testStoredRequestReadMatchesEveryVector() throws {
        let vectors = try XCTUnwrap(try load().stored_request, "vectors.json has no stored_request section \u{2014} the witness (stored_request_specimens) has not emitted it")
        XCTAssertFalse(vectors.isEmpty, "no stored_request vectors")
        for v in vectors {
            let s = try WireDecode.storedRequest(v.json)
            XCTAssertEqual(s, StoredRequestSummary(requester: v.input.requester, purpose: v.input.purpose, destructive: v.input.destructive, expires_at: v.input.expires_at), v.name)
        }
        // Refusals: a record missing a required member, an unknown member, or no JSON at all.
        XCTAssertThrowsError(try WireDecode.storedRequest(#"{"kind":"request"}"#))
        XCTAssertThrowsError(try WireDecode.storedRequest(#"{"requester":"ops","purpose":"Boot","destructive":true,"expires_at":"x","extra":1}"#))
        XCTAssertThrowsError(try WireDecode.storedRequest("not json"))
    }

    /// The push hint decodes from the APNs custom key and refuses its absence.
    func testPushHintDecodesTheCustomKey() {
        XCTAssertEqual(try? WireDecode.pushHint(["notification_id": "n-1", "aps": ["alert": "Approval requested"]]), ApprovalPushHint(notification_id: "n-1"))
        XCTAssertThrowsError(try WireDecode.pushHint(["aps": ["alert": "x"]]))
    }

    /// The strict reader refuses an unknown member, an empty string, an unadmitted kind, and a
    /// member that belongs to ANOTHER arm of the same sum (decoded per arm, never per union).
    func testStrictReaderRefusesUnknownAndEmptyMembers() {
        XCTAssertThrowsError(try WireDecode.enrolmentGrant(Data(#"{"enrollment_id": "e", "extra": 1}"#.utf8)))
        XCTAssertThrowsError(try WireDecode.enrolmentGrant(Data(#"{"enrollment_id": ""}"#.utf8)))
        XCTAssertThrowsError(try WireDecode.pushUpdate(Data(#"{"kind": "fcm", "project": "p", "token": "t"}"#.utf8)))
        XCTAssertThrowsError(try WireDecode.pushUpdate(Data(#"{"kind": "apns", "environment": "production", "topic": "t", "token": "k", "project": "p"}"#.utf8)))
    }

    /// device_push_update_client_data frames the exact push body; the enrolment id and time are the
    /// read vectors' inputs, so the only bytes compared are the authority's.
    func testPushUpdateClientDataMatchesTheFixture() throws {
        let read = try XCTUnwrap(try load().read.first).input
        let body = try envelope("push_update")
        let got = devicePushUpdateClientData(enrollmentId: read.enrollment_id, requestedAt: read.requested_at, pushBodyJson: body)
        XCTAssertEqual(got, Data(try envelope("push_update_client_data").utf8))
    }

    /// The method and every route the app spells is the fixture's; a missing surface row FAILS, so
    /// a route the wire adds and the app does not spell is caught, not silently absent.
    func testSurfaceMatchesTheFixture() throws {
        let rows = Dictionary(uniqueKeysWithValues: try load().surface.map { ($0.name, $0.value) })
        func surface(_ name: String) throws -> String {
            try XCTUnwrap(rows[name], "surface row \(name) missing")
        }
        XCTAssertEqual("POST", try surface("method_every_device_operation"))
        XCTAssertEqual(Route.enrol, try surface("route_enrol"))
        XCTAssertEqual(Route.pending, try surface("route_pending"))
        XCTAssertEqual(Route.requestPrefix, try surface("route_request_prefix"))
        XCTAssertEqual(Route.redeem, try surface("route_redeem"))
        XCTAssertEqual(Route.enrollmentPrefix, try surface("route_enrollment_prefix"))
        XCTAssertEqual(Route.push, try surface("route_push"))
        // Completeness is an identity join, not a count: the surface names must be exactly the ones
        // the app spells, so a renamed row plus an added one cannot cancel out. The spelled set is
        // generated from gunbc.auth.approval_device_redemption_fixtures surface_vectors, the rows
        // vectors.json is emitted from.
        let spelled: Set<String> = ["method_every_device_operation", "route_enrol", "route_pending", "route_request_prefix", "route_redeem", "route_enrollment_prefix", "route_push"]
        XCTAssertEqual(Set(rows.keys), spelled, "surface rows and the app's spelled set differ: \(Set(rows.keys).symmetricDifference(spelled).sorted())")
    }

    func testPathsMatchTheFixture() throws {
        let f = try WireDecode.fetchedRequest(Data(try envelope("fetched_request").utf8))
        XCTAssertEqual(Route.request(f.escalation_id), try envelope("request_path"))
        let g = try WireDecode.enrolmentGrant(Data(try envelope("enrolment_grant").utf8))
        XCTAssertEqual(Route.enrollment(g.enrollment_id), try envelope("enrollment_path"))
    }

    /// path_segment over every input -> encoded vector the .dag fold emitted: "id-" + padded
    /// base64url of the UTF-8 bytes.
    func testPathSegmentMatchesEveryVector() throws {
        for v in try load().path_segment {
            XCTAssertEqual(Route.segment(v.input), v.encoded, v.input)
        }
        // Alphabet control: the URL-safe variant, padded, never "+" or "/".
        let seg = Route.segment("\u{FFFF}?>")
        XCTAssertTrue(seg.hasPrefix("id-") && !seg.contains("+") && !seg.contains("/"))
    }

    /// The emitter's escaping is serialize_json's, over every specimen the fixture carries.
    func testEmitterEscapesLikeSerializeJson() throws {
        let vectors = try XCTUnwrap(try load().escape, "vectors.json has no escape section \u{2014} it is emitted by gunbc.auth.approval_device_redemption_fixtures escape_specimens")
        XCTAssertFalse(vectors.isEmpty, "no escape specimens")
        for v in vectors {
            XCTAssertEqual(WireJson.string(v.input).serialized, v.string_json, v.input)
            XCTAssertEqual(WireJson.object([("k", .array([.string(v.input)]))]).serialized, v.object_json, v.input)
        }
    }
}
