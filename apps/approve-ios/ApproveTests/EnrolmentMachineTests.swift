// The enrolment machine against the ruling, stated independently of the table (expected_moves in the generator).
// GENERATED from gunbc.approve_ios_swift_enrolment_machine from the Swift grammar rows (v2.extdeps.languages.swift.rows); do not edit.
import XCTest
@testable import Approve

final class EnrolmentMachineTests: XCTestCase {
    /// The ruling, one assertion per cell of the phase x answer product: sign-in enrols, any refusal or no answer leaves the phase, the server's revoked report revokes an enrolment, and only a NEW key leaves Revoked.
    func testEveryTransitionMatchesTheRuling() {
        XCTAssertEqual(EnrolmentMachine.next(.signedOut, on: .signedIn(.newKey)), .enrolled, "signedOut")
        XCTAssertEqual(EnrolmentMachine.next(.signedOut, on: .signedIn(.sameKey)), .enrolled, "signedOut")
        XCTAssertEqual(EnrolmentMachine.next(.signedOut, on: .refused), .signedOut, "signedOut")
        XCTAssertEqual(EnrolmentMachine.next(.signedOut, on: .unanswered), .signedOut, "signedOut")
        XCTAssertEqual(EnrolmentMachine.next(.signedOut, on: .serverReportsRevoked), .signedOut, "signedOut")
        XCTAssertEqual(EnrolmentMachine.next(.enrolled, on: .signedIn(.newKey)), .enrolled, "enrolled")
        XCTAssertEqual(EnrolmentMachine.next(.enrolled, on: .signedIn(.sameKey)), .enrolled, "enrolled")
        XCTAssertEqual(EnrolmentMachine.next(.enrolled, on: .refused), .enrolled, "enrolled")
        XCTAssertEqual(EnrolmentMachine.next(.enrolled, on: .unanswered), .enrolled, "enrolled")
        XCTAssertEqual(EnrolmentMachine.next(.enrolled, on: .serverReportsRevoked), .revoked, "enrolled")
        XCTAssertEqual(EnrolmentMachine.next(.revoked, on: .signedIn(.newKey)), .enrolled, "revoked")
        XCTAssertEqual(EnrolmentMachine.next(.revoked, on: .signedIn(.sameKey)), .revoked, "revoked")
        XCTAssertEqual(EnrolmentMachine.next(.revoked, on: .refused), .revoked, "revoked")
        XCTAssertEqual(EnrolmentMachine.next(.revoked, on: .unanswered), .revoked, "revoked")
        XCTAssertEqual(EnrolmentMachine.next(.revoked, on: .serverReportsRevoked), .revoked, "revoked")
    }
}
