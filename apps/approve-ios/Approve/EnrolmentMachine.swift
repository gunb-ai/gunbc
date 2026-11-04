// The app's enrolment state machine, printed from gunbc.approve_ios_enrolment_machine: which phase follows which answer
// of the Google sign-in and enrol POST. EnrolmentState (hand-written) keeps the payloads and persistence and calls this.
// GENERATED from gunbc.approve_ios_swift_enrolment_machine from the Swift grammar rows (v2.extdeps.languages.swift.rows); do not edit.
import Foundation

/// gunbc.approve_ios_enrolment_machine EnrolmentPhase, spelled by phase_name.
enum EnrolmentPhase: String, CaseIterable {
    case signedOut, enrolled, revoked
}

/// gunbc.approve_ios_enrolment_machine SignInKey: whether a sign-in registered a key the phone already held.
enum SignInKey: Equatable {
    case newKey, sameKey
}

/// gunbc.approve_ios_enrolment_machine EnrolmentAnswer: what the sign-in and enrol POST taught the app.
enum EnrolmentAnswer: Equatable {
    case refused, unanswered, serverReportsRevoked
    case signedIn(SignInKey)
}

/// gunbc.approve_ios_enrolment_machine enrolment_next. The app's UI glue calls this and persists the result; it decides nothing itself.
enum EnrolmentMachine {
    static func next(_ from: EnrolmentPhase, on answer: EnrolmentAnswer) -> EnrolmentPhase {
        switch (from, answer) {
        case (.signedOut, .signedIn(.newKey)): return .enrolled
        case (.signedOut, .signedIn(.sameKey)): return .enrolled
        case (.enrolled, .serverReportsRevoked): return .revoked
        case (.revoked, .signedIn(.newKey)): return .enrolled
        default: return from
        }
    }
}
