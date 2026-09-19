// THE HTTP WIRE, mirrored from gunbc.auth.approval_device_wire "The HTTP wire" exactly: request and
// response bodies, every sum's "kind", the crypto carriers, the header names and the path-segment
// rule. Encoding mirrors extdeps.languages.json.emit serialize_json byte for byte (members in the
// authored order, ", " and ": " separators, controls as \u00XX upper-case, no other escaping) so
// the envelope fixtures can be matched as bytes. Decoding mirrors wire_object / wire_string: an
// unknown member, a missing member, an empty string or an unadmitted kind is refused at its path.
// Nothing outside this file spells a path, a header or a JSON key.
import Foundation

// ── serialize_json ───────────────────────────────────────────────────────────────────────────
indirect enum WireJson {
    case string(String)
    case object([(String, WireJson)])
    case array([WireJson])

    /// escape_json_string: \b \t \n \f \r \" \\, other C0 controls as \u00XX (upper-case hex),
    /// everything else — including "/" and non-ASCII — literal.
    static func escape(_ s: String) -> String {
        var out = ""
        for u in s.unicodeScalars {
            switch u.value {
            case 8: out += "\\b"
            case 9: out += "\\t"
            case 10: out += "\\n"
            case 12: out += "\\f"
            case 13: out += "\\r"
            case 34: out += "\\\""
            case 92: out += "\\\\"
            case 0...31: out += String(format: "\\u%04X", u.value)
            default: out.unicodeScalars.append(u)
            }
        }
        return out
    }

    var serialized: String {
        switch self {
        case .string(let s): return "\"" + WireJson.escape(s) + "\""
        case .array(let xs): return "[" + xs.map(\.serialized).joined(separator: ", ") + "]"
        case .object(let kvs): return "{" + kvs.map { "\"" + WireJson.escape($0.0) + "\": " + $0.1.serialized }.joined(separator: ", ") + "}"
        }
    }
}

// ── Strict readers (wire_object / wire_string) ───────────────────────────────────────────────
enum WireError: Error, LocalizedError {
    case configMissing(String)
    /// The server answered with a refusal status; the body is shown, never interpreted.
    case status(Int, String)
    /// A 2xx whose body did not decode: the request MAY have taken effect. Path of the member.
    case refused(at: String, cause: String)
    /// No answer at all: the request MAY have taken effect.
    case transport(String)

    /// True for the arms after which the server's state is unknown to the app.
    var outcomeUnknown: Bool {
        switch self {
        case .refused, .transport: return true
        case .configMissing, .status: return false
        }
    }

    var errorDescription: String? {
        switch self {
        case .configMissing(let m): return m
        case .status(let s, let body): return "server refused: HTTP \(s) \(body)"
        case .refused(let at, let cause): return "server answer refused at \(at): \(cause) (outcome unknown)"
        case .transport(let m): return "server unreached (outcome unknown): \(m)"
        }
    }
}

/// One decoded JSON object with exactly the declared members. Divergence, stated: Foundation's
/// parser keeps the LAST of a duplicated member where json_object_unique_member refuses; the app
/// reads server answers, not attacker input, and the server side keeps the strict rule.
struct WireObject {
    let at: String
    let members: [String: Any]

    init(_ v: Any?, at: String, allowed: [String]) throws {
        guard let o = v as? [String: Any] else { throw WireError.refused(at: at, cause: "not an object") }
        let unknown = o.keys.filter { !allowed.contains($0) }.sorted()
        guard unknown.isEmpty else { throw WireError.refused(at: at, cause: "unknown member: " + unknown.joined(separator: ", ")) }
        self.at = at
        self.members = o
    }

    private func path(_ key: String) -> String { at == "$" ? key : at + "." + key }

    func string(_ key: String) throws -> String {
        guard let v = members[key] else { throw WireError.refused(at: path(key), cause: "missing") }
        guard let s = v as? String else { throw WireError.refused(at: path(key), cause: "not a string") }
        guard !s.isEmpty else { throw WireError.refused(at: path(key), cause: "empty") }
        return s
    }

    func object(_ key: String, allowed: [String]) throws -> WireObject {
        guard let v = members[key] else { throw WireError.refused(at: path(key), cause: "missing") }
        return try WireObject(v, at: path(key), allowed: allowed)
    }

    func array(_ key: String) throws -> [Any] {
        guard let v = members[key] else { throw WireError.refused(at: path(key), cause: "missing") }
        guard let a = v as? [Any] else { throw WireError.refused(at: path(key), cause: "not an array") }
        return a
    }

    /// The member must equal the one admitted spelling.
    func expect(_ key: String, _ admitted: String, _ cause: String) throws {
        guard try string(key) == admitted else { throw WireError.refused(at: path(key), cause: cause) }
    }

    /// A sum: the "kind" member selects the arm, and the object then admits exactly THAT arm's
    /// members — a member of another arm is refused (decode per arm, never per union).
    static func sum(_ v: Any?, at: String, arms: [String: [String]]) throws -> (kind: String, object: WireObject) {
        guard let o = v as? [String: Any] else { throw WireError.refused(at: at, cause: "not an object") }
        let path = at == "$" ? "kind" : at + ".kind"
        guard let k = o["kind"] else { throw WireError.refused(at: path, cause: "missing") }
        guard let kind = k as? String, !kind.isEmpty else { throw WireError.refused(at: path, cause: "not a string") }
        guard let allowed = arms[kind] else { throw WireError.refused(at: path, cause: "not an admitted kind") }
        return (kind, try WireObject(v, at: at, allowed: ["kind"] + allowed))
    }

    static func document(_ data: Data, allowed: [String]) throws -> WireObject {
        let v: Any
        do { v = try JSONSerialization.jsonObject(with: data, options: [.fragmentsAllowed]) }
        catch { throw WireError.refused(at: "$", cause: error.localizedDescription) }
        return try WireObject(v, at: "$", allowed: allowed)
    }
}

// ── Crypto carriers and sums ─────────────────────────────────────────────────────────────────
enum WireEncode {
    /// verifying_key_json
    static func verifyingKey(_ k: VerifyingKey) -> WireJson {
        .object([("suite", .string(VerifyingKey.suite)), ("encoding", .string(VerifyingKey.encoding)), ("point_b64url", .string(k.point_b64url))])
    }
    /// signature_bytes_json
    static func signature(_ b: SignatureBytes) -> WireJson {
        .object([("suite", .string(SignatureBytes.suite)), ("encoding", .string(SignatureBytes.encoding)), ("b64url", .string(b.b64url))])
    }
    /// push_registration_json, ApnsRegistration arm
    static func push(_ p: ApnsRegistration) -> WireJson {
        .object([("kind", .string("apns")), ("environment", .string(p.environment)), ("topic", .string(p.topic)), ("token", .string(p.token))])
    }
    /// presented_evidence_json, IosAppAttestAttestation arm
    static func evidence(_ e: IosAppAttestBoundDecisionKey) -> WireJson {
        .object([("kind", .string("ios_app_attest")), ("attest_key_id", .string(e.attest_key_id)), ("attestation_b64", .string(e.attestation_b64))])
    }
    /// platform_proof_json, IosAppAttestAssertion arm
    static func proof(_ p: IosAppAttestAssertion) -> WireJson {
        .object([("kind", .string("ios_app_attest_assertion")), ("assertion_b64", .string(p.assertion_b64))])
    }
    /// redemption_challenge_json
    static func challenge(_ c: RedemptionChallenge) -> WireJson {
        .object([("expires_at", .string(c.expires_at)), ("nonce_hex", .string(c.nonce_hex))])
    }
    /// signing_input_json
    static func signingInput(_ i: DeviceRedemptionSigningInput) -> WireJson {
        .object([
            ("audience", .string(i.audience)),
            ("enrollment_id", .string(i.enrollment_id)),
            ("challenge", challenge(i.challenge)),
            ("escalation_id", .string(i.escalation_id)),
            ("request_revision", .string(i.request_revision)),
            ("stored_request_text", .string(i.stored_request_text)),
            ("decision", .string(i.decision.rawValue)),
            ("capability_text", .string(i.capability_text)),
            ("capability_tag_b64url", .string(i.capability_tag_b64url)),
        ])
    }
    /// enrolment_request_json
    static func enrolmentRequest(_ r: EnrolmentRequest) -> String {
        WireJson.object([
            ("code", .string(r.code)),
            ("platform", .string(r.platform.rawValue)),
            ("decision_key", verifyingKey(r.decision_key)),
            ("evidence", evidence(r.evidence)),
            ("push", push(r.push)),
        ]).serialized
    }
    /// signed_redemption_json
    static func signedRedemption(_ r: SignedRedemption) -> String {
        WireJson.object([
            ("signing_input", signingInput(r.signing_input)),
            ("signature", signature(r.signature)),
            ("platform_proof", proof(r.platform_proof)),
        ]).serialized
    }
    /// push_update_json
    static func pushUpdate(_ p: ApnsRegistration) -> String { push(p).serialized }
}

enum WireDecode {
    static func verifyingKey(_ o: WireObject) throws -> VerifyingKey {
        try o.expect("suite", VerifyingKey.suite, "not the admitted suite")
        try o.expect("encoding", VerifyingKey.encoding, "not the admitted encoding")
        return VerifyingKey(point_b64url: try o.string("point_b64url"))
    }
    static func signature(_ o: WireObject) throws -> SignatureBytes {
        try o.expect("suite", SignatureBytes.suite, "not the admitted suite")
        try o.expect("encoding", SignatureBytes.encoding, "not the admitted encoding")
        return SignatureBytes(b64url: try o.string("b64url"))
    }
    /// Android's arm is modeled and encodable upstream; the app refuses it as the server does.
    static func push(_ v: Any?, at: String) throws -> ApnsRegistration {
        let (kind, o) = try WireObject.sum(v, at: at, arms: ["apns": ["environment", "topic", "token"]])
        guard kind == "apns" else { throw WireError.refused(at: at + ".kind", cause: "not an admitted push provider") }
        let env = try o.string("environment")
        guard env == "production" || env == "development" else { throw WireError.refused(at: "push.environment", cause: "not an APNs environment") }
        return ApnsRegistration(environment: env, topic: try o.string("topic"), token: try o.string("token"))
    }
    static func evidence(_ v: Any?, at: String) throws -> IosAppAttestBoundDecisionKey {
        let (_, o) = try WireObject.sum(v, at: at, arms: ["ios_app_attest": ["attest_key_id", "attestation_b64"]])
        return IosAppAttestBoundDecisionKey(attest_key_id: try o.string("attest_key_id"), attestation_b64: try o.string("attestation_b64"))
    }
    static func proof(_ v: Any?, at: String) throws -> IosAppAttestAssertion {
        let (_, o) = try WireObject.sum(v, at: at, arms: ["ios_app_attest_assertion": ["assertion_b64"]])
        return IosAppAttestAssertion(assertion_b64: try o.string("assertion_b64"))
    }
    static func challenge(_ o: WireObject) throws -> RedemptionChallenge {
        RedemptionChallenge(expires_at: try o.string("expires_at"), nonce_hex: try o.string("nonce_hex"))
    }
    static func signingInput(_ o: WireObject) throws -> DeviceRedemptionSigningInput {
        let d = try o.string("decision")
        guard let decision = ProposedDecision(rawValue: d) else { throw WireError.refused(at: "signing_input.decision", cause: "not approve or deny") }
        return DeviceRedemptionSigningInput(
            audience: try o.string("audience"),
            enrollment_id: try o.string("enrollment_id"),
            challenge: try challenge(o.object("challenge", allowed: ["expires_at", "nonce_hex"])),
            escalation_id: try o.string("escalation_id"),
            request_revision: try o.string("request_revision"),
            stored_request_text: try o.string("stored_request_text"),
            decision: decision,
            capability_text: try o.string("capability_text"),
            capability_tag_b64url: try o.string("capability_tag_b64url"))
    }
    static func verbCapability(_ o: WireObject) throws -> VerbCapability {
        VerbCapability(capability_text: try o.string("capability_text"), capability_tag_b64url: try o.string("capability_tag_b64url"))
    }

    // Request decoders exist so the envelope fixtures round-trip through the app's own encoders.
    static func enrolmentRequest(_ data: Data) throws -> EnrolmentRequest {
        let o = try WireObject.document(data, allowed: ["code", "platform", "decision_key", "evidence", "push"])
        try o.expect("platform", MobilePlatform.ios.rawValue, "not an admitted platform")
        return EnrolmentRequest(
            code: try o.string("code"), platform: .ios,
            decision_key: try verifyingKey(o.object("decision_key", allowed: ["suite", "encoding", "point_b64url"])),
            evidence: try evidence(o.members["evidence"], at: "evidence"),
            push: try push(o.members["push"], at: "push"))
    }
    static func signedRedemption(_ data: Data) throws -> SignedRedemption {
        let o = try WireObject.document(data, allowed: ["signing_input", "signature", "platform_proof"])
        return SignedRedemption(
            signing_input: try signingInput(o.object("signing_input", allowed: ["audience", "enrollment_id", "challenge", "escalation_id", "request_revision", "stored_request_text", "decision", "capability_text", "capability_tag_b64url"])),
            signature: try signature(o.object("signature", allowed: ["suite", "encoding", "b64url"])),
            platform_proof: try proof(o.members["platform_proof"], at: "platform_proof"))
    }
    static func pushUpdate(_ data: Data) throws -> ApnsRegistration {
        let v: Any
        do { v = try JSONSerialization.jsonObject(with: data, options: [.fragmentsAllowed]) }
        catch { throw WireError.refused(at: "$", cause: error.localizedDescription) }
        return try push(v, at: "$")
    }

    // Responses.
    static func enrolmentGrant(_ data: Data) throws -> EnrolmentGrant {
        EnrolmentGrant(enrollment_id: try WireObject.document(data, allowed: ["enrollment_id"]).string("enrollment_id"))
    }
    static func pendingList(_ data: Data) throws -> [PendingApproval] {
        let o = try WireObject.document(data, allowed: ["pending"])
        return try o.array("pending").enumerated().map { i, row in
            let r = try WireObject(row, at: "pending[\(i)]", allowed: ["escalation_id", "request_revision"])
            return PendingApproval(escalation_id: try r.string("escalation_id"), request_revision: try r.string("request_revision"))
        }
    }
    static func fetchedRequest(_ data: Data) throws -> FetchedRequest {
        let o = try WireObject.document(data, allowed: ["escalation_id", "request_revision", "stored_request_text", "challenge", "approve", "deny"])
        let cap = ["capability_text", "capability_tag_b64url"]
        return FetchedRequest(
            escalation_id: try o.string("escalation_id"),
            request_revision: try o.string("request_revision"),
            stored_request_text: try o.string("stored_request_text"),
            challenge: try challenge(o.object("challenge", allowed: ["expires_at", "nonce_hex"])),
            approve: try verbCapability(o.object("approve", allowed: cap)),
            deny: try verbCapability(o.object("deny", allowed: cap)))
    }
    static func enrolmentReadback(_ data: Data) throws -> EnrolmentReadback {
        let o = try WireObject.document(data, allowed: ["enrollment_id", "standing"])
        let s = try o.string("standing")
        guard let standing = EnrolmentStandingWire(rawValue: s) else { throw WireError.refused(at: "standing", cause: "not an enrolment standing") }
        return EnrolmentReadback(enrollment_id: try o.string("enrollment_id"), standing: standing)
    }
    static func redemptionResponse(_ data: Data) throws -> RedemptionOutcome {
        let o = try WireObject.document(data, allowed: ["outcome", "message"])
        return RedemptionOutcome(outcome: try o.string("outcome"), message: try o.string("message"))
    }
}

// ── Records the wire carries ─────────────────────────────────────────────────────────────────
/// EnrolmentRequest: the POST /approve/device/enrol body. The login is not sent: the server derives it.
struct EnrolmentRequest: Equatable {
    var code: String
    var platform: MobilePlatform
    var decision_key: VerifyingKey
    var evidence: IosAppAttestBoundDecisionKey
    var push: ApnsRegistration
}

struct EnrolmentGrant: Equatable { var enrollment_id: String }

struct PendingApproval: Identifiable, Hashable {
    var escalation_id: String
    var request_revision: String
    var id: String { escalation_id }
}

/// VerbCapability: the text and the canonical base64url tag exactly as issued, sent back untouched.
struct VerbCapability: Equatable {
    var capability_text: String
    var capability_tag_b64url: String
}

/// FetchedRequest: the stored request byte for byte, a fresh challenge, and BOTH verbs' capabilities.
struct FetchedRequest: Equatable {
    var escalation_id: String
    var request_revision: String
    var stored_request_text: String
    var challenge: RedemptionChallenge
    var approve: VerbCapability
    var deny: VerbCapability

    func capability(for d: ProposedDecision) -> VerbCapability {
        switch d {
        case .approve: return approve
        case .deny: return deny
        }
    }
}

enum EnrolmentStandingWire: String { case active, revoked }

struct EnrolmentReadback: Equatable {
    var enrollment_id: String
    var standing: EnrolmentStandingWire
}

/// RedemptionResponse: the DeviceRedemptionOutcome arm name and the operator-facing reason. The app
/// renders the reason and branches only on the name.
struct RedemptionOutcome: Equatable {
    var outcome: String
    var message: String
}

// ── Headers, paths ───────────────────────────────────────────────────────────────────────────
enum ReadHeader {
    static let enrollment = "X-Approval-Enrollment"
    static let requestedAt = "X-Approval-Requested-At"
    static let assertion = "X-Approval-Assertion"
}

/// The credentials an authenticated GET or PUT carries; produced by the caller so this file signs nothing.
struct ReadAuth {
    var enrollmentId: String
    var requestedAt: String
    var assertionB64: String
}

enum Route {
    static let enrol = "/approve/device/enrol"
    static let pending = "/approve/device/pending"
    static let redeem = "/approve/device/redeem"
    static let push = "/approve/device/push"
    static let requestPrefix = "/approve/device/requests/"
    static let enrollmentPrefix = "/approve/device/enrollments/"

    /// path_segment = "id-" + base64url(UTF-8(identity)), padded URL-safe alphabet (A-Z a-z 0-9 - _
    /// with "=" padding): total and injective, never refused, no percent escape for any layer to
    /// decode, never a dot segment, no case-equivalent spelling. Transported verbatim.
    static func segment(_ s: String) -> String {
        "id-" + Data(s.utf8).base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
    }
    /// device_request_path
    static func request(_ escalationId: String) -> String { requestPrefix + segment(escalationId) }
    /// device_enrollment_path
    static func enrollment(_ enrollmentId: String) -> String { enrollmentPrefix + segment(enrollmentId) }
}

// ── Transport ────────────────────────────────────────────────────────────────────────────────
struct ServerConfig {
    /// APPROVE_SERVER_HOST from Config/Team.xcconfig via Info.plist. Plain URLSession over the
    /// tailnet: no new network path.
    let host: String
    /// apns_environment_wire: "development" | "production".
    let apnsEnvironment: String
    /// extdeps.apple.apns ApnsTopic: the bundle identifier, read from the bundle and never defaulted.
    let apnsTopic: String

    static func fromBundle() throws -> ServerConfig {
        let info = Bundle.main.infoDictionary ?? [:]
        guard let host = info["ApproveServerHost"] as? String, !host.isEmpty else {
            throw WireError.configMissing("APPROVE_SERVER_HOST is empty in Config/Team.xcconfig")
        }
        guard let env = info["ApproveApnsEnvironment"] as? String, env == "development" || env == "production" else {
            throw WireError.configMissing("APNS_ENVIRONMENT must be development or production in Config/Team.xcconfig")
        }
        guard let topic = Bundle.main.bundleIdentifier, !topic.isEmpty else {
            throw WireError.configMissing("CFBundleIdentifier is missing; the APNs topic cannot be derived")
        }
        // A configured host is a host: refuse anything URLComponents will not carry as one.
        var probe = URLComponents()
        probe.scheme = "https"
        probe.host = host
        guard probe.url != nil, probe.host == host, !host.contains("/"), !host.contains("?"), !host.contains("#") else {
            throw WireError.configMissing("APPROVE_SERVER_HOST is not a bare host: \(host)")
        }
        return ServerConfig(host: host, apnsEnvironment: env, apnsTopic: topic)
    }

    /// path is a Route path whose segments are already base64url; it is set as percentEncodedPath so
    /// URLComponents transports it verbatim and never re-encodes it (the alphabet has nothing to escape).
    func url(_ path: String) throws -> URL {
        var c = URLComponents()
        c.scheme = "https"
        c.host = host
        c.percentEncodedPath = path
        guard let u = c.url else { throw WireError.configMissing("route path malformed: \(path)") }
        return u
    }
}

struct Client {
    let config: ServerConfig
    let session = URLSession(configuration: .ephemeral)

    private func send(_ method: String, _ path: String, body: String? = nil, read: ReadAuth? = nil) async throws -> Data {
        var req = URLRequest(url: try config.url(path))
        req.httpMethod = method
        if let read {
            req.setValue(read.enrollmentId, forHTTPHeaderField: ReadHeader.enrollment)
            req.setValue(read.requestedAt, forHTTPHeaderField: ReadHeader.requestedAt)
            req.setValue(read.assertionB64, forHTTPHeaderField: ReadHeader.assertion)
        }
        if let body {
            req.httpBody = Data(body.utf8)
            req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        }
        let data: Data
        let resp: URLResponse
        do { (data, resp) = try await session.data(for: req) }
        catch { throw WireError.transport(error.localizedDescription) }
        guard let status = (resp as? HTTPURLResponse)?.statusCode else { throw WireError.refused(at: "$", cause: "no HTTP status") }
        // A refusal is a typed, located error; the body is shown, never guessed at.
        guard (200..<300).contains(status) else { throw WireError.status(status, String(decoding: data, as: UTF8.self)) }
        return data
    }

    func enrol(_ r: EnrolmentRequest) async throws -> EnrolmentGrant {
        try WireDecode.enrolmentGrant(await send("POST", Route.enrol, body: WireEncode.enrolmentRequest(r)))
    }
    func pending(_ read: ReadAuth) async throws -> [PendingApproval] {
        try WireDecode.pendingList(await send("GET", Route.pending, read: read))
    }
    func fetch(_ path: String, _ read: ReadAuth) async throws -> FetchedRequest {
        try WireDecode.fetchedRequest(await send("GET", path, read: read))
    }
    func redeem(_ r: SignedRedemption) async throws -> RedemptionOutcome {
        try WireDecode.redemptionResponse(await send("POST", Route.redeem, body: WireEncode.signedRedemption(r)))
    }
    /// GET /approve/device/enrollments/<enrollment_id>, read-assertion authenticated.
    func readback(_ path: String, _ read: ReadAuth) async throws -> EnrolmentReadback {
        try WireDecode.enrolmentReadback(await send("GET", path, read: read))
    }
    /// PUT /approve/device/push; the body is the exact JSON the assertion's client data framed.
    func updatePush(bodyJson: String, _ read: ReadAuth) async throws {
        _ = try await send("PUT", Route.push, body: bodyJson, read: read)
    }
}
