// The app's enrolment state machine, printed from gunbc.approve_ios_enrolment_machine: which phase follows which answer
// of the enrol POST. EnrolmentState (hand-written) keeps the payloads and persistence and calls this.
// GENERATED from gunbc.approve_ios_swift_enrolment_machine from the Swift grammar rows (v2.extdeps.languages.swift.rows); do not edit.
import Foundation

/// gunbc.approve_ios_enrolment_machine EnrolmentPhase, spelled by phase_name.
enum EnrolmentPhase: String, CaseIterable {
    case unenrolled, prepared, submissionUnknown, enrolled, keyInvalidated, revoked
}

/// gunbc.approve_ios_enrolment_machine EnrolmentAnswer: what the enrol POST taught the app.
enum EnrolmentAnswer: Equatable {
    case granted, unanswered
    case refused(EnrolmentRefusalKind)
}

/// gunbc.approve_ios_enrolment_machine enrolment_next. The app's UI glue calls this and persists the result; it decides nothing itself.
enum EnrolmentMachine {
    static func next(_ from: EnrolmentPhase, on answer: EnrolmentAnswer) -> EnrolmentPhase {
        switch (from, answer) {
        case (.submissionUnknown, .granted): return .enrolled
        case (.submissionUnknown, .refused(.code_not_issued)): return .unenrolled
        case (.submissionUnknown, .refused(.code_expired)): return .unenrolled
        default: return from
        }
    }
}
