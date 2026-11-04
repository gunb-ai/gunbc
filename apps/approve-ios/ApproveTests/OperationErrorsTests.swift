// The error lifetime against the ruling, stated independently of the model (error_scenarios and expected_relevance in the generator).
// GENERATED from gunbc.approve_ios_swift_operation_errors from the Swift grammar rows (v2.extdeps.languages.swift.rows); do not edit.
import XCTest
@testable import Approve

final class OperationErrorsTests: XCTestCase {
    /// The operator's report: an inbox read refused 503 is gone once the next inbox read succeeds.
    func testAnErrorClearsOnTheSameOperationsNextSuccess() {
        var errors = OperationErrors()
        errors.record(.inboxRefresh, .failed("server refused 503"))
        errors.record(.inboxRefresh, .succeeded)
        XCTAssertEqual(errors.shown, [], "testAnErrorClearsOnTheSameOperationsNextSuccess")
    }

    /// The positive control: a failure with no later attempt of its operation is shown.
    func testAFailureIsShownUntilItsOperationAnswers() {
        var errors = OperationErrors()
        errors.record(.inboxRefresh, .failed("server refused 503"))
        XCTAssertEqual(errors.shown, ["server refused 503"], "testAFailureIsShownUntilItsOperationAnswers")
    }

    /// Errors are keyed by operation: an inbox success says nothing about a push token that was not forwarded.
    func testAnotherOperationsSuccessLeavesTheError() {
        var errors = OperationErrors()
        errors.record(.pushForward, .failed("APNs token not forwarded"))
        errors.record(.inboxRefresh, .succeeded)
        XCTAssertEqual(errors.shown, ["APNs token not forwarded"], "testAnotherOperationsSuccessLeavesTheError")
    }

    /// One slot per operation: the latest failure is the current state, not a log of every attempt.
    func testARepeatedFailureReplacesAndNeverAccumulates() {
        var errors = OperationErrors()
        errors.record(.inboxRefresh, .failed("server refused 503"))
        errors.record(.inboxRefresh, .failed("transport failed"))
        XCTAssertEqual(errors.shown, ["transport failed"], "testARepeatedFailureReplacesAndNeverAccumulates")
    }

    /// An operation no longer relevant takes its error with it; the launch-time configuration error stays.
    func testLeavingEnrolledDropsTheEnrolmentsErrors() {
        var errors = OperationErrors()
        errors.record(.inboxRefresh, .failed("server refused 503"))
        errors.record(.configuration, .failed("no server configuration"))
        errors.enter(.signedOut)
        XCTAssertEqual(errors.shown, ["no server configuration"], "testLeavingEnrolledDropsTheEnrolmentsErrors")
    }

    /// The relevance ruling, one assertion per cell of the operation x phase product, stated independently of the table.
    func testEveryRelevanceCellMatchesTheRuling() {
        XCTAssertEqual(OperationErrors.relevant(.configuration, in: .signedOut), true, "configuration in signedOut")
        XCTAssertEqual(OperationErrors.relevant(.configuration, in: .enrolled), true, "configuration in enrolled")
        XCTAssertEqual(OperationErrors.relevant(.configuration, in: .revoked), true, "configuration in revoked")
        XCTAssertEqual(OperationErrors.relevant(.enrolmentRead, in: .signedOut), true, "enrolmentRead in signedOut")
        XCTAssertEqual(OperationErrors.relevant(.enrolmentRead, in: .enrolled), true, "enrolmentRead in enrolled")
        XCTAssertEqual(OperationErrors.relevant(.enrolmentRead, in: .revoked), true, "enrolmentRead in revoked")
        XCTAssertEqual(OperationErrors.relevant(.signIn, in: .signedOut), true, "signIn in signedOut")
        XCTAssertEqual(OperationErrors.relevant(.signIn, in: .enrolled), false, "signIn in enrolled")
        XCTAssertEqual(OperationErrors.relevant(.signIn, in: .revoked), false, "signIn in revoked")
        XCTAssertEqual(OperationErrors.relevant(.signOut, in: .signedOut), false, "signOut in signedOut")
        XCTAssertEqual(OperationErrors.relevant(.signOut, in: .enrolled), true, "signOut in enrolled")
        XCTAssertEqual(OperationErrors.relevant(.signOut, in: .revoked), true, "signOut in revoked")
        XCTAssertEqual(OperationErrors.relevant(.apnsRegistration, in: .signedOut), false, "apnsRegistration in signedOut")
        XCTAssertEqual(OperationErrors.relevant(.apnsRegistration, in: .enrolled), true, "apnsRegistration in enrolled")
        XCTAssertEqual(OperationErrors.relevant(.apnsRegistration, in: .revoked), false, "apnsRegistration in revoked")
        XCTAssertEqual(OperationErrors.relevant(.pushForward, in: .signedOut), false, "pushForward in signedOut")
        XCTAssertEqual(OperationErrors.relevant(.pushForward, in: .enrolled), true, "pushForward in enrolled")
        XCTAssertEqual(OperationErrors.relevant(.pushForward, in: .revoked), false, "pushForward in revoked")
        XCTAssertEqual(OperationErrors.relevant(.inboxRefresh, in: .signedOut), false, "inboxRefresh in signedOut")
        XCTAssertEqual(OperationErrors.relevant(.inboxRefresh, in: .enrolled), true, "inboxRefresh in enrolled")
        XCTAssertEqual(OperationErrors.relevant(.inboxRefresh, in: .revoked), false, "inboxRefresh in revoked")
        XCTAssertEqual(OperationErrors.relevant(.requestFetch, in: .signedOut), false, "requestFetch in signedOut")
        XCTAssertEqual(OperationErrors.relevant(.requestFetch, in: .enrolled), true, "requestFetch in enrolled")
        XCTAssertEqual(OperationErrors.relevant(.requestFetch, in: .revoked), false, "requestFetch in revoked")
        XCTAssertEqual(OperationErrors.relevant(.redemption, in: .signedOut), false, "redemption in signedOut")
        XCTAssertEqual(OperationErrors.relevant(.redemption, in: .enrolled), true, "redemption in enrolled")
        XCTAssertEqual(OperationErrors.relevant(.redemption, in: .revoked), false, "redemption in revoked")
    }
}
