// THE ONE FILE THAT KNOWS THE HTTP SURFACE. Route paths and envelope names are data rows of
// gunbc.auth.approval_device_redemption (the server adopted these paths as rows, so the server matches
// the app and not the other way round). Field names are the .dag field names. Nothing outside this
// file spells a path or a JSON key that is not a protocol field.
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
        return ServerConfig(host: host, apnsEnvironment: env)
    }

    func url(_ path: String) -> URL { URL(string: "https://\(host)\(path)")! }
}

enum Route {
    static let enrol = "/approve/device/enrol"
    static let pending = "/approve/device/pending"
    static func request(_ escalationId: String) -> String { "/approve/device/requests/\(escalationId)" }
    static let redeem = "/approve/device/redeem"
}

/// POST /approve/device/enrol body. The login is not sent: the server derives it from the challenge.
struct EnrolmentSubmission: Codable {
    var challenge: EnrolmentChallenge
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

/// GET /approve/device/requests/<escalation_id>: the stored request as the server returns it, byte
/// for byte, with a fresh stateless challenge and the capability the redemption names.
struct FetchedRequest: Codable, Equatable {
    var escalation_id: String
    var request_revision: String
    var stored_request_text: String
    var challenge: RedemptionChallenge
    var capability_text: String
    var capability_tag: String
}

/// The server's typed outcome (DeviceRedemptionOutcome): the arm name travels as "outcome", the
/// device_redemption_reason as "message". The app renders both and interprets neither.
struct RedemptionOutcome: Codable, Equatable {
    var outcome: String
    var message: String
}

enum WireError: Error, LocalizedError {
    case configMissing(String)
    case status(Int, String)
    case undecodable(String)

    var errorDescription: String? {
        switch self {
        case .configMissing(let m): return m
        case .status(let s, let body): return "server refused: HTTP \(s) \(body)"
        case .undecodable(let m): return "server answer undecodable: \(m)"
        }
    }
}

struct Client {
    let config: ServerConfig
    let session = URLSession(configuration: .ephemeral)

    private func send<T: Decodable>(_ method: String, _ path: String, body: (any Encodable)? = nil) async throws -> T {
        var req = URLRequest(url: config.url(path))
        req.httpMethod = method
        if let body {
            req.httpBody = try JSONEncoder().encode(AnyEncodable(body))
            req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        }
        let (data, resp) = try await session.data(for: req)
        let status = (resp as? HTTPURLResponse)?.statusCode ?? -1
        // A refusal is a typed, located error; the body is shown, never guessed at.
        guard (200..<300).contains(status) else {
            throw WireError.status(status, String(decoding: data, as: UTF8.self))
        }
        do { return try JSONDecoder().decode(T.self, from: data) }
        catch { throw WireError.undecodable("\(error)") }
    }

    func enrol(_ s: EnrolmentSubmission) async throws -> EnrolmentGrant { try await send("POST", Route.enrol, body: s) }
    func pending() async throws -> [PendingApproval] { try await send("GET", Route.pending) }
    func fetch(_ escalationId: String) async throws -> FetchedRequest { try await send("GET", Route.request(escalationId)) }
    func redeem(_ r: SignedRedemption) async throws -> RedemptionOutcome { try await send("POST", Route.redeem, body: r) }
}

private struct AnyEncodable: Encodable {
    let value: any Encodable
    init(_ value: any Encodable) { self.value = value }
    func encode(to encoder: Encoder) throws { try value.encode(to: encoder) }
}
