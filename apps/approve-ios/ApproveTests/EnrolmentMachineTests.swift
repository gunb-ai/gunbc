// The enrolment machine against the ruling, stated independently of the table (leaves_submission_unknown in the generator).
// GENERATED from gunbc.approve_ios_swift_enrolment_machine from the Swift grammar rows (v2.extdeps.languages.swift.rows); do not edit.
import XCTest
@testable import Approve

final class EnrolmentMachineTests: XCTestCase {
    /// code_expired and code_not_issued prove the code minted no enrolment: they, and only they, leave SubmissionUnknown. Every other arm stays.
    func testOnlyTheTwoNoEnrolmentRefusalsLeaveSubmissionUnknown() {
        let leaves = ["code_expired", "code_not_issued"]
        for kind in EnrolmentRefusalKind.allCases {
            let expected: EnrolmentPhase = leaves.contains(kind.rawValue) ? .unenrolled : .submissionUnknown
            XCTAssertEqual(EnrolmentMachine.next(.submissionUnknown, on: .refused(kind)), expected, kind.rawValue)
        }
    }

    /// A transport failure, an undecodable body or any answer the app may not act on leaves SubmissionUnknown standing.
    func testAnUnansweredSubmissionStaysUnknown() {
        XCTAssertEqual(EnrolmentMachine.next(.submissionUnknown, on: .unanswered), .submissionUnknown, "unanswered")
    }

    /// A grant is the one way SubmissionUnknown becomes Enrolled.
    func testAGrantEnrols() {
        XCTAssertEqual(EnrolmentMachine.next(.submissionUnknown, on: .granted), .enrolled, "granted")
    }

    /// No answer to the enrol POST moves any other phase: the answer is only about a submission in flight.
    func testNoOtherPhaseMoves() {
        for phase in EnrolmentPhase.allCases {
            if phase != .submissionUnknown {
                XCTAssertEqual(EnrolmentMachine.next(phase, on: .granted), phase, phase.rawValue)
                XCTAssertEqual(EnrolmentMachine.next(phase, on: .unanswered), phase, phase.rawValue)
                for kind in EnrolmentRefusalKind.allCases {
                    XCTAssertEqual(EnrolmentMachine.next(phase, on: .refused(kind)), phase, phase.rawValue)
                }
            }
        }
    }
}
