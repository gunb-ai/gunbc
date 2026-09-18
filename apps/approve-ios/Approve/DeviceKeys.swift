// The two device keys, in the roles extdeps.apple.secure_enclave and extdeps.apple.app_attest give
// them: the CryptoKit enclave key DECIDES (biometric-gated at creation), the App Attest key proves
// THE APP. Neither private key leaves the device. Persistence of the app's enrolment record is in
// EnrolmentStore; this file holds only the key operations.
import Foundation
import CryptoKit
import DeviceCheck
import Security

enum DeviceKeyError: Error, LocalizedError {
    case appAttestUnsupported
    case accessControl(String)
    /// The enclave refused to sign because the biometric set changed (biometryCurrentSet), or the
    /// wrapped key no longer unwraps: the enrolment's key is dead and re-enrolment is the remedy.
    case keyInvalidated(String)

    var errorDescription: String? {
        switch self {
        case .appAttestUnsupported:
            // extdeps.apple.app_attest app_attest_unavailable_in_simulator_note: refuse, never enrol without an attestation.
            return "App Attest is not supported on this device (Simulator, or no Secure Enclave). Enrolment refused."
        case .accessControl(let m): return "access control: \(m)"
        case .keyInvalidated(let m): return "decision key invalidated: \(m)"
        }
    }
}

/// The decision key. Access policy is bound to the key at CREATION: [.privateKeyUsage,
/// .biometryCurrentSet] (extdeps.apple.secure_enclave BiometryCurrentSet). The enclave itself
/// prompts Face ID when signature(for:) is called; there is deliberately no LAContext pre-check —
/// approval_key_access_policy_note: "An LAContext prompt the app shows before calling an unguarded
/// key is a UI, not a gate."
enum DecisionKey {
    static func create() throws -> SecureEnclave.P256.Signing.PrivateKey {
        var err: Unmanaged<CFError>?
        guard let ac = SecAccessControlCreateWithFlags(
            nil,
            kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
            [.privateKeyUsage, .biometryCurrentSet],
            &err
        ) else {
            throw DeviceKeyError.accessControl(err?.takeRetainedValue().localizedDescription ?? "unknown")
        }
        return try SecureEnclave.P256.Signing.PrivateKey(accessControl: ac)
    }

    /// Rebuild the enclave key from its wrapped blob (private_key_never_leaves_note: opaque and
    /// device-bound). Failure here means the key is gone for good.
    static func load(_ blob: Data) throws -> SecureEnclave.P256.Signing.PrivateKey {
        do { return try SecureEnclave.P256.Signing.PrivateKey(dataRepresentation: blob) }
        catch { throw DeviceKeyError.keyInvalidated("wrapped key does not unwrap: \(error.localizedDescription)") }
    }

    /// extdeps.crypto.signature Sec1Uncompressed = CryptoKit x963Representation, 65 octets.
    static func verifyingKey(_ key: SecureEnclave.P256.Signing.PrivateKey) -> VerifyingKey {
        VerifyingKey(point_b64url: Base64Url.encode(key.publicKey.x963Representation))
    }

    /// enclave_signs_message_not_digest_note: the exact input bytes go to signature(for:), which
    /// SHA-256-hashes them. rawRepresentation is the fixed-width r||s (P1363FixedWidth). A sign
    /// refusal is surfaced as is: CryptoKit does not distinguish the operator cancelling Face ID from
    /// a key the changed biometric set invalidated, so no state transition is inferred from it —
    /// only a wrapped key that no longer unwraps (load) is a known-dead key.
    static func sign(_ key: SecureEnclave.P256.Signing.PrivateKey, _ input: Data) throws -> SignatureBytes {
        SignatureBytes(b64url: Base64Url.encode(try key.signature(for: input).rawRepresentation))
    }
}

/// The App Attest key. It certifies the app instance, not the operator, and nothing about the
/// decision key; the link is the clientDataHash committing to enrolment_transcript
/// (attestation_proves_the_app_not_the_person_note). The key id is persisted by EnrolmentStore as
/// part of the Prepared record so a retried enrolment attests with the SAME key over the SAME hash,
/// which is Apple's instruction for a failed attestKey submission.
enum AppAttest {
    static func requireSupported() throws {
        guard DCAppAttestService.shared.isSupported else { throw DeviceKeyError.appAttestUnsupported }
    }

    static func generateKey() async throws -> String {
        try requireSupported()
        return try await DCAppAttestService.shared.generateKey()
    }

    /// clientDataHash = SHA256(enrolment_transcript bytes).
    static func attest(keyId: String, transcript: Data) async throws -> IosAppAttestBoundDecisionKey {
        let hash = Data(SHA256.hash(data: transcript))
        let attestation = try await DCAppAttestService.shared.attestKey(keyId, clientDataHash: hash)
        return IosAppAttestBoundDecisionKey(attest_key_id: keyId, attestation_b64: attestation.base64EncodedString())
    }

    /// An assertion whose clientData is EXACTLY the given bytes: the redemption signing input (the same
    /// bytes the decision key signs) or device_read_client_data for an authenticated GET. The API takes
    /// the hash; the server hashes the same bytes.
    static func assert(keyId: String, clientData: Data) async throws -> IosAppAttestAssertion {
        let hash = Data(SHA256.hash(data: clientData))
        let assertion = try await DCAppAttestService.shared.generateAssertion(keyId, clientDataHash: hash)
        return IosAppAttestAssertion(assertion_b64: assertion.base64EncodedString())
    }
}

/// One keychain item, one query shape: class + service + account. kSecValueData appears only in the
/// add and update dictionaries, never in a query. Every status is checked; nothing is inferred from a
/// nil result.
enum KeychainItem {
    enum Failure: Error, LocalizedError {
        case status(String, OSStatus)
        var errorDescription: String? {
            if case .status(let op, let s) = self { return "keychain \(op): OSStatus \(s)" }
            return nil
        }
    }

    private static let service = "ai.gunb.approve"

    private static func query(_ account: String) -> [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
    }

    static func read(_ account: String) throws -> Data? {
        var q = query(account)
        q[kSecReturnData as String] = true
        q[kSecMatchLimit as String] = kSecMatchLimitOne
        var out: CFTypeRef?
        let s = SecItemCopyMatching(q as CFDictionary, &out)
        switch s {
        case errSecSuccess:
            guard let d = out as? Data else { throw Failure.status("read returned non-data", s) }
            return d
        case errSecItemNotFound:
            return nil
        default:
            throw Failure.status("read", s)
        }
    }

    static func write(_ account: String, _ value: Data) throws {
        let update: [String: Any] = [kSecValueData as String: value]
        let s = SecItemUpdate(query(account) as CFDictionary, update as CFDictionary)
        switch s {
        case errSecSuccess:
            return
        case errSecItemNotFound:
            var add = query(account)
            add[kSecAttrAccessible as String] = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
            add[kSecValueData as String] = value
            let a = SecItemAdd(add as CFDictionary, nil)
            guard a == errSecSuccess else { throw Failure.status("add", a) }
        default:
            throw Failure.status("update", s)
        }
    }

    static func delete(_ account: String) throws {
        let s = SecItemDelete(query(account) as CFDictionary)
        guard s == errSecSuccess || s == errSecItemNotFound else { throw Failure.status("delete", s) }
    }
}
