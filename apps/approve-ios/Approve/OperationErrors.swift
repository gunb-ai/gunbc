// The app's error lifetime, printed from gunbc.approve_ios_operation_errors: one current error per operation, removed by
// that operation's next success or by leaving the phases it is relevant in. AppState and the views record every outcome.
// GENERATED from gunbc.approve_ios_swift_operation_errors from the Swift grammar rows (v2.extdeps.languages.swift.rows); do not edit.
import Foundation

/// gunbc.approve_ios_operation_errors AppOperation, spelled by operation_name, in display order.
enum AppOperation: String, CaseIterable {
    case configuration, enrolmentRead, signIn, signOut, apnsRegistration, pushForward, inboxRefresh, requestFetch, redemption
}

/// gunbc.approve_ios_operation_errors OperationOutcome: what one attempt of an operation came to.
enum OperationOutcome: Equatable {
    case succeeded
    case failed(String)
}

/// gunbc.approve_ios_operation_errors: the CURRENT error of each operation, never a log. The glue records every attempt's outcome, success included.
struct OperationErrors: Equatable {
    private var messages: [AppOperation: String] = [:]

    /// slot_after: the operation's slot is decided by this attempt's outcome alone.
    mutating func record(_ operation: AppOperation, _ outcome: OperationOutcome) {
        switch outcome {
        case .succeeded: messages[operation] = nil
        case .failed(let message): messages[operation] = message
        }
    }

    /// enter_phase: an operation not relevant in the phase takes its error with it.
    mutating func enter(_ phase: EnrolmentPhase) {
        for operation in AppOperation.allCases {
            if !OperationErrors.relevant(operation, in: phase) {
                messages[operation] = nil
            }
        }
    }

    /// shown_errors: the current messages, in AppOperation order.
    var shown: [String] {
        return AppOperation.allCases.compactMap { messages[$0] }
    }

    /// operation_relevant_in, printed by evaluating it over the operation x phase product.
    static func relevant(_ operation: AppOperation, in phase: EnrolmentPhase) -> Bool {
        switch (operation, phase) {
        case (.configuration, .signedOut): return true
        case (.configuration, .enrolled): return true
        case (.configuration, .revoked): return true
        case (.enrolmentRead, .signedOut): return true
        case (.enrolmentRead, .enrolled): return true
        case (.enrolmentRead, .revoked): return true
        case (.signIn, .signedOut): return true
        case (.signOut, .enrolled): return true
        case (.signOut, .revoked): return true
        case (.apnsRegistration, .enrolled): return true
        case (.pushForward, .enrolled): return true
        case (.inboxRefresh, .enrolled): return true
        case (.requestFetch, .enrolled): return true
        case (.redemption, .enrolled): return true
        default: return false
        }
    }
}
