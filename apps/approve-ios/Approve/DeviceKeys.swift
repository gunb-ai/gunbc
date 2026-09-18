// The two device keys, in the roles extdeps.apple.secure_enclave and extdeps.apple.app_attest give
// them: the CryptoKit enclave key DECIDES (biometric-gated at creation), the App Attest key proves
// THE APP. Neither private key leaves the device.
import Foundation
import CryptoKit
import DeviceCheck
import Security

enum DeviceKeyError: Error, LocalizedError {
    case appAttestUnsupported
    case accessControl(String)
    case keychain(OSStatus)

    var errorDescription: String? {
        switch self {
        case .appAttestUnsupported:
            // extdeps.apple.app_attest app_attest_unavailable_in_simulator_note: refuse, never enrol without an attestation.
            return "App Attest is not supported on this device (Simulator, or no Secure Enclave). Enrolment refused."
        case .accessControl(let m): return "access control: \(m)"
        case .keychain(let s): return "keychain: OSStatus \(s)"
        }
    }
}

/// The decision key. Access policy is bound to the key at CREATION: [.privateKeyUsage,
/// .biometryCurrentSet] (extdeps.apple.secure_enclave BiometryCurrentSet). The enclave itself
/// prompts Face ID when signature(for:) is called; there is deliberately no LAContext pre-check —
/// approval_key_access_policy_note: "An LAContext prompt the app shows before calling an unguarded
/// key is a UI, not a gate."
enum DecisionKey {
    private static let account = "ai.gunb.approve.decision-key"

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
        let key = try SecureEnclave.P256.Signing.PrivateKey(accessControl: ac)
        try store(key.dataRepresentation)
        return key
    }

    static func load() throws -> SecureEnclave.P256.Signing.PrivateKey? {
        guard let blob = try read() else { return nil }
        return try SecureEnclave.P256.Signing.PrivateKey(dataRepresentation: blob)
    }

    /// extdeps.crypto.signature Sec1Uncompressed = CryptoKit x963Representation, 65 octets.
    static func verifyingKey(_ key: SecureEnclave.P256.Signing.PrivateKey) -> VerifyingKey {
        VerifyingKey(point_b64url: Base64Url.encode(key.publicKey.x963Representation))
    }

    /// enclave_signs_message_not_digest_note: the exact input bytes go to signature(for:), which
    /// SHA-256-hashes them. rawRepresentation is the fixed-width r||s (P1363FixedWidth).
    static func sign(_ key: SecureEnclave.P256.Signing.PrivateKey, _ input: Data) throws -> SignatureBytes {
        SignatureBytes(b64url: Base64Url.encode(try key.signature(for: input).rawRepresentation))
    }

    // The wrapped blob is opaque and device-bound (private_key_never_leaves_note); the keychain
    // item only remembers which enclave key is enrolled.
    private static func store(_ blob: Data) throws {
        let q: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: account,
            kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
            kSecValueData as String: blob,
        ]
        SecItemDelete(q as CFDictionary)
        let s = SecItemAdd(q as CFDictionary, nil)
        guard s == errSecSuccess else { throw DeviceKeyError.keychain(s) }
    }

    private static func read() throws -> Data? {
        let q: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: account,
            kSecReturnData as String: true,
        ]
        var out: CFTypeRef?
        let s = SecItemCopyMatching(q as CFDictionary, &out)
        if s == errSecItemNotFound { return nil }
        guard s == errSecSuccess, let d = out as? Data else { throw DeviceKeyError.keychain(s) }
        return d
    }
}

/// The App Attest key. It certifies the app instance, not the operator, and nothing about the
/// decision key; the link is the clientDataHash committing to enrolment_transcript
/// (attestation_proves_the_app_not_the_person_note).
enum AppAttest {
    private static let keyIdDefault = "ai.gunb.approve.attest-key-id"

    static func requireSupported() throws {
        guard DCAppAttestService.shared.isSupported else { throw DeviceKeyError.appAttestUnsupported }
    }

    static func generateKey() async throws -> String {
        try requireSupported()
        let id = try await DCAppAttestService.shared.generateKey()
        UserDefaults.standard.set(id, forKey: keyIdDefault)
        return id
    }

    static var keyId: String? { UserDefaults.standard.string(forKey: keyIdDefault) }

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
