// THE ONE FILE THAT KNOWS THE HTTP SURFACE. Route paths are the data rows of
// gunbc.auth.approval_device_wire (approval_device_*_path); field names are the .dag field names.
// Nothing outside this file spells a path, a header or a JSON key that is not a protocol field.
import Foundation

struct ServerConfig {
    /// APPROVE_SERVER_HOST from Config/Team.xcconfig via Info.plist. Plain URLSession over the
    /// tailnet: no new network path.
    let host: String
    let apnsEnvironment: String

    static func fromBundle() throws -> ServerConfig {
        let info = Bundle.main.infoDictionary ?? [:]
        guard let host = info["ApproveServerHost"] as? String, !host.isEmpty else {
            throw WireError.configMissing("APPROVE_SERVER_HOST is empty in Config/Team.xcconfig")
        }
        guard let env = info["ApproveApnsEnvironment"] as? String, !env.isEmpty else {
            throw WireError.configMissing("APNS_ENVIRONMENT is empty in Config/Team.xcconfig")
        }
        // A configured host is a host, never a URL fragment: refuse anything URLComponents will not
        // carry as one, rather than force-building a URL that lies about where it points.
        var probe = URLComponents()
        probe.scheme = "https"
        probe.host = host
        guard probe.url != nil, probe.host == host, !host.contains("/"), !host.contains("?"), !host.contains("#") else {
            throw WireError.configMissing("APPROVE_SERVER_HOST is not a bare host: \(host)")
        }
        return ServerConfig(host: host, apnsEnvironment: env)
    }

    /// path is an already-encoded route path (Route.*); URLComponents refuses a malformed one.
    func url(_ path: String) throws -> URL {
        var c = URLComponents()
        c.scheme = "https"
        c.host = host
        c.percentEncodedPath = path
        guard let u = c.url else { throw WireError.configMissing("route path malformed: \(path)") }
        return u
    }
}

enum Route {
    /// approval_device_enrol_path
    static let enrol = "/approve/device/enrol"
    /// approval_device_pending_path
    static let pending = "/approve/device/pending"
    /// approval_device_request_path_prefix + one path segment carrying escalation_id. The segment
    /// encoding is the wire module's (pending: gunbc.auth.approval_device_wire path-segment encoding);
    /// until it lands the app admits only RFC 3986 unreserved characters and REFUSES anything else,
    /// so no id reaches the wire under a spelling the server might read differently.
    static func request(_ escalationId: String) throws -> String {
        let unreserved = CharacterSet(charactersIn: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~")
        guard !escalationId.isEmpty, escalationId.unicodeScalars.allSatisfy({ unreserved.contains($0) }) else {
            throw WireError.unmodeled("escalation_id contains characters outside the admitted path-segment alphabet: \(escalationId)")
        }
        return "/approve/device/requests/\(escalationId)"
    }
    /// approval_device_redeem_path
    static let redeem = "/approve/device/redeem"
}

/// Authenticated reads: the assertion over device_read_client_data and the two values the server
/// needs to recompute it. Admitted only inside approval_device_read_skew (60 s).
enum ReadHeader {
    static let assertion = "X-Approval-Assertion"
    static let enrollment = "X-Approval-Enrollment"
    static let requestedAt = "X-Approval-Requested-At"
}

/// The credentials an authenticated GET carries; produced by the caller so this file signs nothing.
struct ReadAuth {
    var enrollmentId: String
    var requestedAt: String
    var assertionB64: String
}

/// POST /approve/device/enrol body. The login is not sent: the server derives it from the code.
struct EnrolmentSubmission: Codable {
    var code: String
    var platform: MobilePlatform
    var decision_key: VerifyingKey
    var evidence: IosAppAttestBoundDecisionKey
    var push: ApnsRegistration
}

/// The server's answer to enrolment: the enrollment_id the redemption signs over.
struct EnrolmentGrant: Codable {
    var enrollment_id: String
}

/// One row of GET /approve/device/pending.
struct PendingApproval: Codable, Identifiable, Hashable {
    var escalation_id: String
    var request_revision: String
    var id: String { escalation_id }
}

/// The capability one verb redeems: approval_capability_signing_input text and its canonical
/// base64url tag exactly as issued — sent back as the server returned it, never re-encoded.
struct VerbCapability: Codable, Equatable {
    var capability_text: String
    var capability_tag_b64url: String
}

/// GET /approve/device/requests/<escalation_id>: the stored request as the server returns it, byte
/// for byte, a fresh stateless challenge, and BOTH verbs' capabilities — the operator chooses after
/// reading, and the app signs with the chosen verb's pair.
struct FetchedRequest: Codable, Equatable {
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

/// The server's typed outcome (DeviceRedemptionOutcome): the arm name travels as "outcome", the
/// device_redemption_reason as "message". The app renders both and interprets neither.
struct RedemptionOutcome: Codable, Equatable {
    var outcome: String
    var message: String
}

enum WireError: Error, LocalizedError {
    case configMissing(String)
    /// The server answered with a refusal status; the body is shown, never interpreted.
    case status(Int, String)
    /// A 2xx whose body did not decode: the request MAY have taken effect.
    case undecodable(String)
    /// No answer at all: the request MAY have taken effect.
    case transport(String)
    /// A route or encoding the wire module does not model yet. Refused, never improvised.
    case unmodeled(String)

    /// True for the two arms after which the server's state is unknown to the app.
    var outcomeUnknown: Bool {
        switch self {
        case .undecodable, .transport: return true
        case .configMissing, .status, .unmodeled: return false
        }
    }

    var errorDescription: String? {
        switch self {
        case .configMissing(let m): return m
        case .status(let s, let body): return "server refused: HTTP \(s) \(body)"
        case .undecodable(let m): return "server answer undecodable (outcome unknown): \(m)"
        case .transport(let m): return "server unreached (outcome unknown): \(m)"
        case .unmodeled(let m): return "not modeled by the wire yet: \(m)"
        }
    }
}

struct Client {
    let config: ServerConfig
    let session = URLSession(configuration: .ephemeral)

    private func send<T: Decodable>(_ method: String, _ path: String, body: (any Encodable)? = nil,
                                    read: ReadAuth? = nil) async throws -> T {
        var req = URLRequest(url: try config.url(path))
        req.httpMethod = method
        if let read {
            req.setValue(read.assertionB64, forHTTPHeaderField: ReadHeader.assertion)
            req.setValue(read.enrollmentId, forHTTPHeaderField: ReadHeader.enrollment)
            req.setValue(read.requestedAt, forHTTPHeaderField: ReadHeader.requestedAt)
        }
        if let body {
            req.httpBody = try JSONEncoder().encode(AnyEncodable(body))
            req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        }
        let data: Data
        let resp: URLResponse
        do { (data, resp) = try await session.data(for: req) }
        catch { throw WireError.transport(error.localizedDescription) }
        guard let status = (resp as? HTTPURLResponse)?.statusCode else {
            throw WireError.undecodable("no HTTP status")
        }
        // A refusal is a typed, located error; the body is shown, never guessed at.
        guard (200..<300).contains(status) else {
            throw WireError.status(status, String(decoding: data, as: UTF8.self))
        }
        do { return try JSONDecoder().decode(T.self, from: data) }
        catch { throw WireError.undecodable("\(error)") }
    }

    func enrol(_ s: EnrolmentSubmission) async throws -> EnrolmentGrant { try await send("POST", Route.enrol, body: s) }
    func pending(_ read: ReadAuth) async throws -> [PendingApproval] { try await send("GET", Route.pending, read: read) }
    func fetch(_ path: String, _ read: ReadAuth) async throws -> FetchedRequest {
        try await send("GET", path, read: read)
    }
    func redeem(_ r: SignedRedemption) async throws -> RedemptionOutcome { try await send("POST", Route.redeem, body: r) }

    // Two routes the wire module does not carry yet. Each REFUSES, typed, so the states that need
    // them (SubmissionUnknown, a rotated APNs token) are visible as blocked rather than papered over
    // with an envelope this file would have had to invent.
    /// Re-read this device's enrolment after an ambiguous POST, identified by the decision key it
    /// prepared (pending: enrolment readback route, including how the device authenticates it).
    func readbackEnrolment(decisionKey: VerifyingKey) async throws -> EnrolmentGrant {
        throw WireError.unmodeled("enrolment readback route")
    }
    /// Forward a rotated APNs token (pending: push-registration update route).
    func updatePushRegistration(_ push: ApnsRegistration, _ read: ReadAuth) async throws {
        throw WireError.unmodeled("push-registration update route")
    }
}

private struct AnyEncodable: Encodable {
    let value: any Encodable
    init(_ value: any Encodable) { self.value = value }
    func encode(to encoder: Encoder) throws { try value.encode(to: encoder) }
}
