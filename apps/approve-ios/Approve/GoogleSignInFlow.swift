// THE ONE PLACE THE APP TALKS TO GoogleSignIn-iOS (>= 9.0.0, pinned in project.yml by
// gunbc.approve_ios_project). Enrolment signs in with a nonce the app computed from the device keys
// (googleEnrolmentNonce), so the ID token Google returns is bound to those keys and to nothing else.
//
// No serverClientID is configured: the token's aud is the iOS client alone (GoogleClient.clientID,
// emitted from gunbc.auth.tracker_web_oauth_client approve_ios_oauth_client), the audience the server
// admits (gunbc.auth.approval_google_enrolment). The redirect URL scheme is the reversed client id,
// declared in Info.plist by the same project spec.
import Foundation
import GoogleSignIn
import UIKit

enum GoogleSignInFlow {
    enum Failure: LocalizedError {
        case noPresenter
        case noIdToken

        var errorDescription: String? {
            switch self {
            case .noPresenter: return "No window to present Google sign-in from."
            case .noIdToken: return "Google sign-in returned no ID token."
            }
        }
    }

    /// The ID token for a sign-in whose nonce is `nonce`. A cancelled or failed sign-in throws; the
    /// caller reads that as the machine's `.refused` answer.
    @MainActor
    static func idToken(nonce: String) async throws -> String {
        guard let presenter = UIApplication.shared.connectedScenes
            .compactMap({ ($0 as? UIWindowScene)?.keyWindow?.rootViewController }).first else {
            throw Failure.noPresenter
        }
        GIDSignIn.sharedInstance.configuration = GIDConfiguration(clientID: GoogleClient.clientID)
        let result = try await GIDSignIn.sharedInstance.signIn(withPresenting: presenter, hint: nil, additionalScopes: nil, nonce: nonce)
        guard let token = result.user.idToken?.tokenString else { throw Failure.noIdToken }
        return token
    }
}
