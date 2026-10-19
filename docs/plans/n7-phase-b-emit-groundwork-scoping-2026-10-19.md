# N7 Phase B (Emit-one → N7-green): emit-side groundwork scoping

Scoping note, read-only. Authority: `dag/gunbc/roadmap/roadmap_authority.dag` node `native-seven`
(`roadmap_node` id `native-seven`, carriers `src/v2/compiler/infer.dag`) — its `first_slice` names
Phase B ("execution: each member reaches its expected verdict natively and the label becomes an
executing required check") and says the phase "owes iteration direction for map/all (#13069), a v2
evaluation meaning for roster-bound concat (#13284) and NativeTestPassed rows (#13210)."
Conventions per DESIGN.md §3 (one authority; cite the symbol, not the position), §6b (defect worked
at its root), §7 (the emitted compiler decides; the seed only emits).

Reads taken: DESIGN.md §3/§6b/§7; the `native-seven` roadmap node; `src/v2/compiler/fold_lowering.dag`;
`src/v2/compiler/eval.dag`; `src/v2/compiler/native_test_vocabulary.dag`;
`src/v2/compiler/source_root_eval.dag`; `src/v1/05_emit_rust.dag`;
`dag/gunbc/plans/for_while_loop_sugar.dag`; `v2.extdeps.runtimes.v2_evaluator`, `v2.program`.

## What Phase B actually executes (shapes every item below)

The native lane is seed-emit + emitted-binary execution: `src/v1/05_emit_rust.dag`
`emit_source_root_eval_driver_main_rs` renders the emitted crate's `main` as one call into the v2
driver fold, and `src/v2/compiler/source_root_eval.dag` states "THE RENDERED MAIN CALLS THIS AND
NOTHING ELSE DECIDES THE VERDICT" and "the seed's remaining job is emit + cargo, and every semantic
decision the lane makes" lives in the fold. So Phase B's execution semantics are authored in the v2
driver closure (`v2.compiler.source_root_eval` and what its closure reaches), not in the v1 emitter.
Two consequences found by inspection:

- The emitted crate "carries a module only when this closure reaches it"
  (`source_root_eval.dag`, frontier-ratchet comment). `v2.compiler.eval` imports
  `v2.extdeps.runtimes.v2_evaluator` and `source_root_eval.dag` imports it directly, so the
  interpreter is in the closure. `v2.compiler.fold_lowering` is NOT imported by
  `source_root_eval.dag`, `eval.dag`, or `lowering.dag` (only by `infer.dag` and test claims) — a
  fold that executes `Loop` must add it to the driver closure.
- Both named loop-step rejections are OUTSIDE that closure's use: `v2_eval_step_loop`
  (`src/v2/extdeps/runtimes/v2_evaluator.dag`) and `program_step_loop` (`src/v2/program.dag`) are
  cited in `dag/gunbc/plans/for_while_loop_sugar.dag` §5 as rejecting every `Loop`, but the lane's
  emitted main does not drive `v2.program`. Item 1 is therefore scoped to the loop machinery the
  closure actually carries, and a scoping question on the other two is left open (§3b conformance:
  one concept, one executor).

## Candidates (max 3), each with dependency evidence by symbol

### 1. Fold iteration direction for map/all — #13069. Classification: INDEPENDENT of Phase A.

- Anchors: `FoldIterationDirection` (type, `src/v2/compiler/fold_lowering.dag` line ~1200) with its
  readers `callback_realization_iteration_direction`, `fold_roster_form_iteration_direction`,
  `fold_realized_path_iteration_direction`, `fold_realized_iteration_direction`, consumed by
  `fold_recurrence_encoding`; failure home
  `dag/gunbc/recurring_failure_mode/fold_realized_loop_carries_no_iteration_direction.dag`
  (PR #13069).
- Independence evidence: `fold_lowering.dag` is imported only by `v2.compiler.infer` and test
  claims; nothing in the Phase A blocker chain (the node's `first_slice` list: #13307, #13325,
  #13210, #13320, #13408/#13453/#13378, #13284 typing, #13379, Optional de-fork, EmptyPattern,
  XL-2) names it. The form is derivable from the already-decided loop-domain representation: S0's
  edge half landed (#12548: `Loop` admits one `^loop_domain_edge`, read through `v2.std.node`
  `loop_edge_role` — `for_while_loop_sugar.dag` status paragraph), so the encoding does not wait on
  any Phase A member typing.
- Rework risk: one shared consumer, `infer.dag`. The encoding must stay a pure function of the
  `CollectionRosterForm` / `CallbackRealization` / realized-declaration shape already on the tree;
  if Phase A's map/all work changes those shapes, adjust — the direction is keyed on the realized
  declaration path, the stable part.

### 2. v2 evaluation meaning for roster-bound concat — #13284. Classification: BLOCKED on a named Phase A output.

- Anchor: the roadmap `first_slice` lists "kernel-text concat (#13284)" INSIDE the Phase A chain,
  and the 2026-10-15 node note records `v2.compiler.service_realization` first refusing "on its own
  unbound concat and to_string". So the authority for what concat MEANS (its typing/binding) lands
  in Phase A.
- Evidence the eval half is genuinely absent: `src/v2/compiler/eval.dag` has zero `concat` hits; the
  interpreter dispatches through `v2.extdeps.runtimes.v2_evaluator` (`v2_evaluator_interpretation`,
  imported at `eval.dag` line 23).
- Why not now: writing a v2 evaluation meaning before Phase A lands the concat binding at its
  declaration home (the 2026-10-15 rulings: no built-ins, no resolver alias tier, NAMESPACE-XL
  waves) invents a second authority for concat semantics — DESIGN §3. The realization shape to
  mirror is `v2.test.claim.compiler.collection_concat_realization` (exists); start after the Phase
  A concat resolution is on main.

### 3. NativeTestPassed rows — #13210, the verdict join in the driver fold. Classification: INDEPENDENT of Phase A; DECLINE its emit-side half.

- Anchors: `NativeTestPassed` (sum-member, `src/v2/compiler/native_test_vocabulary.dag` line ~53;
  rendering arm line ~148). Consumer side already exists: `src/v1/05_emit_rust.dag` line ~21626
  renders `NativeTestVerdict::NativeTestPassed`. Producer side is the gap, and its designated home is
  the fold, not the emitter ("every semantic decision the lane makes" — `source_root_eval.dag`;
  "A decision taken in emitted Rust is a decision no [gate sees]" there, line ~1471).
- Independence evidence: the vocabulary module and the driver fold are execution-side only; no Phase
  A blocker names `native_test_vocabulary`. Safe scope now: the verdict-vocabulary join — mapping an
  executed member's outcome onto the `NativeTestVerdict` rows the driver main reports — WITHOUT
  asserting any member's expected verdict (that is Phase A green work).
- DECLINED: anything touching `v2.compiler.emit` / `emit_module` / `emit_semantic_decl` keying.
  #13936 (emitter re-key by declaration identity, owner stern-raven-480) is in flight; per brief, no
  duplication. Also left out: `v2_eval_step_loop` / `program_step_loop` — executing them is not on
  the lane's path (see "What Phase B actually executes"), and choosing the lane's single loop
  executor is a design call to raise with the node owner, not to pre-empt here.

## Open questions for the node owner

1. Which loop executor is the lane's one authority: a fold-carried loop interpreter in the
   `source_root_eval` closure, or making `v2_eval_step_loop`/`program_step_loop` the shared one and
   pulling their module into the closure? (DESIGN §2 horizontal — one concept, every scale.)
2. Confirm #13210's "rows" means driver-fold-produced verdict rows, given the consumer arm already
   exists in `src/v1/05_emit_rust.dag`.
