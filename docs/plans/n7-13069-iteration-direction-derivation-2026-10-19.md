// DESIGN §6b re-derivation for the approved #13069 build (fold iteration direction), prepared 2026-19
// before building, per the manager brief. N: a fold-family call's lowered Loop must carry the
// iteration direction its own synthesis obeys, and a contradicted direction must be refused on the
// real acceptance path. Authority for the fact: v2.compiler.fold_lowering FoldIterationDirection
// (fact side already on main: f314f5c986d, N7-1b, controls ccr_map_loop_iterates_right_to_left /
// ccr_fold_list_and_fold_list_right_are_told_apart — both measured red when their row was flipped at
// that rung; the map-fixture ones are pre-existing red on today's binding frontier, see the flip-run
// note below).
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

// THE FIX, by construction (§5: construction over proof over validation; manager no-go 2026-10-19
// rejected the two-reader cross-check version as validation-with-two-authorities; the manager's row
// ruling of 2026-10-19 (variant B, then the total-match correction the same day)): ONE ROW PER
// CALLBACK REALIZATION, read by a TOTAL match -- callback_step_row(r: CallbackRealization) ->
// CallbackStepRow {init: CallbackInitShape, step_shape: CallbackStepShape} -- keyed on the realization
// identity (a sum member, never a string). The direction is NOT an independent field: it is read by
// exactly one total match over CallbackStepShape (callback_step_shape_iteration_direction:
// cons-shapes => Right, joins => Left), so the contradiction "cons step paired with Left" has NO
// CONSTRUCTOR, and a realization with no row does not COMPILE (v2.compiler.infer
// infer_match_non_exhaustive refuses the non-exhaustive arm set) -- so neither a coverage claim over a
// row population nor a runtime refusal for a rowless realization is needed; both were written, both
// were deleted on the manager's correction. callback_realization_iteration_direction reads the row
// through that one match; body_lower_callback_step_body matches the SAME row's init and step_shape
// (each shape arm BUILDS its body, so there is no free choice left), and the free sibling arms over
// CallbackRealization are deleted; there is no seam: no caller-supplied direction parameter exists
// anywhere. The row lives in v2.compiler.fold_lowering (the direction authority; body_lowering_fold
// already imports from it -- direction, no cycle). If the direction were kept as a second field
// cross-checked by construction, the wrong pair would be writable as a data literal -- that variant is
// rung 2 with a lens, not rung 4; the derived-direction total match is what makes rung 4 honest for
// the contradiction class (the manager's ruling).

// CONTROLS (manager evidence ruling of 2026-10-19, variant B; joined on one leaf per §3b):
// - THE CONTRADICTION CLASS IS RUNG 4 BY CONSTRUCTION: {cons-shape, Left} has no constructor (the
//   direction is read from the shape by callback_step_shape_iteration_direction, never carried beside
//   it), so NO RED of that class is authorable -- stated plainly in the failure-mode row.
// - THE RESIDUAL CLASS (the one shape -> direction match being wrong) is discriminated by EXECUTION on
//   the claim that DISCRIMINATES it: flipping the cons arm (StepConsOfCallbackResult =>
//   IteratesLeftToRight) makes ccr_map_direction_is_read_from_its_step_shape measure RED and green at
//   the unmutated sha (the claim reads map's direction back through the realized path). Both runs
//   pinned to shas, reported to the manager; the mutation is never shipped (the runner keeps no
//   target/ across dispatches, so each run is its own pinned build). Red BEFORE the wall exists: both
//   step_loop handlers still refuse every Loop during the flip, so the RED does not read or wait on an
//   executor. HONESTY NOTE (review 82832 on the PR): the map-fixture claims
//   (ccr_map_loop_iterates_right_to_left and siblings) CANNOT be the flip evidence -- they are
//   pre-existing red at the base sha (identical verdict lists at base and head) and their helper
//   answers false on fixture refusal, not on a wrong direction; an earlier wording of this note and of
//   the failure-mode row named them, and was corrected.
// - ACCEPTED CONTROLS: the direction claims that are green at this head — ccr_map_direction_is_read_from_its_step_shape,
//   ccr_filter_and_any_directions, ccr_fold_and_all_iterate_left_to_right — read each callback's
//   direction off its own row, and the shipped ccr_fold_list_and_fold_list_right_are_told_apart is
//   untouched. The map-fixture claims are PRE-EXISTING RED on the binding frontier and are reported as
//   such, not as accepted controls. The row population itself needs no claim: the total match makes
//   disagreement unwriteable.
// - REFUSAL OF DEFAULT: ccr_a_non_fold_path_has_no_direction keeps the no-default direction reading
//   (a path naming no fold derives Absent, never a default direction).
// CEILING claim (what green will NOT say): this closes the callback-synthesis contradiction boundary
// only. It does NOT execute fold-realized Loops, does not enforce an authored fold's declared order
// against user step bodies, and does not touch the executor boundary. The failure-mode row's RUNG text
// will be updated to record the boundary closed and to keep the executor NEXT-RUNG TRIGGER intact.

// ACCEPTANCE: `gunbc test //gunbc/instruments:self-host` (and the v2-native-cli row) green on the
// remote runner pinned to this build's head sha under a real cgroup memory bind (DESIGN: the memory
// envelope is part of the verdict); PR opened against main, never merged.
