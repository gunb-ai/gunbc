// DESIGN §6b re-derivation for the approved #13069 build (fold iteration direction), prepared 2026-19
// before building, per the manager brief. N: a fold-family call's lowered Loop must carry the
// iteration direction its own synthesis obeys, and a contradicted direction must be refused on the
// real acceptance path. Authority for the fact: v2.compiler.fold_lowering FoldIterationDirection
// (fact side already on main: f314f5c986d, N7-1b, controls ccr_map_loop_iterates_right_to_left /
// ccr_fold_list_and_fold_list_right_are_told_apart — both measured red when the row is flipped).
// Failure home: dag/gunbc/recurring_failure_mode/fold_realized_loop_carries_no_iteration_direction.

// THE CONSUMERS (measured at HEAD 574991e6918, not assumed): call sites of
// fold_realized_iteration_direction / fold_realized_path_iteration_direction /
// fold_roster_form_iteration_direction / callback_realization_iteration_direction exist ONLY in
// src/v2/test/claim/compiler/collection_callback_realization.dag (8 sites). The compiler-side
// consumers are the assembly points: fold_recurrence_encoding (fold_lowering.dag:1389, called from
// body_lower_fold_encoding_from, body_lowering_fold.dag:3621, and the callback synthesis,
// body_lowering_fold.dag:3747, and from fold_call_to_encoding, fold_lowering.dag:1683). There is NO
// execution-side consumer: v2_eval_step_loop (v2.extdeps.runtimes.v2_evaluator) and
// program_step_loop (v2.program) refuse every Loop, and BOTH ARE OUT OF BOUND for this build
// (operator owns the loop-executor question; this build does not touch either and builds on neither
// reading). Note measured against the brief: v2.compiler/infer imports v2.compiler.fold_lowering
// (fold_step_formal_at, fold_realized_member_role, ...) but has ZERO references to any
// iteration_direction symbol — the build therefore does not touch infer.dag at all, so the
// Phase-A-coupling guardrail holds trivially.

// THE MECHANISM each consumer uses to reach the fact, and the boundary making it well-defined:
// 1. fold_recurrence_encoding — passes the authored head atom as the Loop's
//    ^loop_realized_declaration_edge (its `realized` operand). Well-defined when the head resolves:
//    v2.compiler.resolve resolve_realized_declaration (resolve.dag:4778) checks it on the acceptance
//    path — the boundary is resolve, and a
//    Loop carrying an absent or unreadable realized edge yields Absent from
//    fold_realized_iteration_direction (total; fails toward Absent, never a default).
// 2. The ccr claims — read the direction at supplied realized paths via
//    fold_realized_path_iteration_direction, which is well-defined by (1): the path resolves or the
//    claim reads Absent, and ccr_a_non_fold_path_has_no_direction pins that refusal-of-default.
// 3. fold_roster_form_iteration_direction / callback_realization_iteration_direction — the DERIVATION,
//    keyed on the identity-typed collection form (CollectionRosterForm.fold_iteration_direction) and
//    on the CallbackRealization. Well-defined by the roster rows being the key (the 13210/13317
//    re-keying precedent), not string shape.

// THE EARLIEST UNJUSTIFIED BOUNDARY (the gap this build closes): for the CALLBACK operations the
// declared direction and the synthesized step shape are two hand-written sibling matches over the
// SAME CallbackRealization sum — callback_realization_iteration_direction (fold_lowering.dag:1204:
// CallbackMapsToCons/CallbackFiltersByCondition => Right, CallbackAllByAnd => Left) and
// body_lower_callback_step_body (body_lowering_fold.dag:3691: CallbackMapsToCons => cons(head,
// tail=acc) — order-preserving only right-to-left; CallbackFiltersByCondition => the kept-member
// cons branch, right; CallbackAllByAnd => `acc && body` join, left). Nothing cross-checks them: flip
// the declared arm (or a roster row's fold_iteration_direction) and lowering still accepts, so the
// encoding carries a direction its own step body contradicts — the failure-mode row's exact
// "direction carried as inert text". This is UPSTREAM of resolve and UPSTREAM of the execution wall:
// the boundary between the declared fact and its first consumer-shaped use, the earliest point where
// a wrong direction is decidable without an executor. The prose claim at fold_lowering.dag:1197-1198
// ("preserves the collection's order only RIGHT TO LEFT") is asserted, not enforced — this build
// enforces it.

// WHY NOT FARTHER UPSTREAM: §6b asks for the EARLIEST boundary, not the root. The root (nobody
// consumes the direction at all) is the executor — the step_loop question, with the operator.
// Authoring the direction check where the direction is DERIVED (fold_lowering) and REFUSING where the
// shape is SYNTHESIZED is the earliest decidable consumer: an authored-fold step body is user code
// whose shape cannot refute a declared order (a fold is order-bound by contract to its executor,
// fold vs fold_list_right share one encoding and are told apart by the realized declaration — the
// shipped precedent), so shape-enforcement is honest ONLY for the mechanically-synthesized callback
// steps. The executor-side enforcement is NOT faked or pre-empted here.

// THE FIX, one authority (§3): the direction is read once by callback_realization_iteration_direction
// (unchanged); a new shape-evidence reader in the same authority module,
// fold_callback_step_iteration_evidence(step), derives the direction from the SYNTHESIZED step's
// canonical shape (cons-with-acc-tail => Right, kept-member cons branch => Right, &&-join on the
// acc => Left, unreadable => Absent); the callback encoding assembly refuses
// (^fold_lowering_direction_contradicts_step_shape) when evidence is Present and disagrees with the
// declared direction, and fails closed on Absent for the callback route (the shape there is
// canonical — unreadable means a synthesis bug, never a pass). Keyed on the realization identity via
// CallbackRealization, never string matching on step text. The assembly is parameterized on the
// declared direction so a control can inject the contradiction (precedent: fold_recurrence_encoding
// is directly exercised by claims over its operands; the real route's value still flows from
// callback_realization_iteration_direction — one authority, one input).

// CONTROLS (joined on one leaf per §3b; each cites the mechanism it exercises):
// - RED / discriminating: a fold-family map call lowered on the REAL route with the declared
//   direction flipped against the cons step shape is REFUSED at lowering, with the refusal symbol
//   above, on the acceptance path (v2.compiler.body_lowering_fold callback assembly -> emitted-binary
//   closure). Measured red BEFORE the wall exists: with the agreement check deleted it goes red while
//   both step_loop handlers still refuse every Loop — enforcement does not wait on, or read, an
//   executor. Measurement recorded at implementation in claim text and PR body.
// - Accepted positive control: unmutated map/filter/any/all lower accepted, each with
//   fold_callback_step_iteration_evidence agreeing with callback_realization_iteration_direction, and
//   fold_list vs fold_list_right keep their shipped ccr_told_apart control untouched.
// - Refusal-of-default: an unreadable synthesized step shape on the callback route refuses rather
//   than passing (no silent Absent).
// CEILING claim (what green will NOT say): this closes the callback-synthesis contradiction boundary
// only. It does NOT execute fold-realized Loops, does not enforce an authored fold's declared order
// against user step bodies, and does not touch the executor boundary. The failure-mode row's RUNG text
// will be updated to record the boundary closed and to keep the executor NEXT-RUNG TRIGGER intact.

// ACCEPTANCE: `gunbc test //gunbc/instruments:self-host` (and the v2-native-cli row) green on the
// remote runner pinned to this build's head sha under a real cgroup memory bind (DESIGN: the memory
// envelope is part of the verdict); PR opened against main, never merged.
