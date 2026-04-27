# R1 Closure Manager Brief

**Status:** ACTIVE on creation. Dissolves on R1 all-gates-green declared per [`ROADMAP.md §"Release R1 Program"`](../../ROADMAP.md) acceptance criterion.

## Orient before reading

- **R1 acceptance authority:** [`ROADMAP.md §"Release R1 Program"`](../../ROADMAP.md) is single authority on R1 gate semantics. R1 closes when every lane's acceptance `TestClaim` (a) compiles as a `.dag` declaration AND (b) evaluates true at release.
- **Strict-vs-pragmatic interpretation:** strict per THESIS commitment ("the release gate IS a `.dag` program. This is the thesis eating its own dogfood"). Ratchet-only enforcement (Rust integration tests checking census drift) does NOT count as a green gate; every gate must be a `.dag` `TestClaim` evaluating true.
- **Why this manager exists:** an audit against current HEAD found 12+ unwired gates under strict reading (PB census predicates not wired as `.dag` TestClaims; T-P0 / T-Sub / T-Demo fixtures not authored; `MockBackedInvariant` runner returns `NotYetImplemented`; T-Emit gates rely on host harness rather than `.dag` wrappers). Ad-hoc dispatch across existing R1 lane owners has not closed these because the R2 rework concentrated R2 program-manager attention forward; R1 closure work needs dedicated dispatch authority.
- **Interaction with R2 managers:** the 6 R2 managers (per [`docs/r2-structure.md`](../r2-structure.md)) spawn on R1 close. This manager is short-lived: it dissolves the moment R1's all-gates-green criterion fires, before R2 managers spawn. No overlap.
- **Lane structure source:** the gate audit + lane decomposition lives below in §"Owned deliverables". Each lane is mutually exclusive at the fixture-file / runner-dispatch-arm level; no two lanes touch the same source surface.

## Program scope (R1 Closure)

Close every R1 gate listed in `ROADMAP.md §"Lane acceptance — .dag gates"` under strict reading. The 9 R1 lanes (T-P0, T-Sub, T-Emit, T-LaneE, T-TestGen, T-LensAPI, T-PB-A, T-PB-B, T-Demo) own ~30 gate predicates; this manager dispatches the closure of the unwired subset (~12 gates) without re-owning gate authority that already evaluates green.

**Explicitly NOT in scope:**
- Reopening lane scope decisions (T-LaneE evidence requirements, T-LensAPI capability list, etc. — those are owned by the original lane authority).
- Authoring R2 work — anything that requires R2 substrate or R2 manager dispatch is out of scope; if a closure attempt surfaces an R2 substrate gap, escalate to Director per the escalation discipline.
- Pure Bootstrap to Zero program work beyond what's required for the 6 PB census gates to fire as `.dag` TestClaims — `docs/design-pure-bootstrap-zero.md` continues to govern the broader program; this manager only owns the gate-as-`.dag`-predicate slice.

## Owned deliverables (6 mutually-exclusive lanes)

| Lane | Size | Scope | Depends on | Status |
|---|---|---|---|---|
| **R1C-A** | M-L | T-TestGen schema extensions (three coupled sub-deliverables per the worker brief): (A) M1(2.8) list-body lowering for `data` declarations (compiler work in `lower.rs`); (B) scope predicate shapes for the 6 PB-census gates (T-PB-A `pb_hand_rust_at_shim_floor`, `lens_producer_files_remaining`, `pb_self_compile_fixed_point`, `pb_compiler_std_ratchet_zero` + T-PB-B `pb_test_file_generated_from_dag`, `pb_rust_tests_outside_residual_zero`); (C) `MockBackedInvariant` minimal-demo fixture closing `testgen_mock_backed_integration_safe`. Sized M-L (was M before audit revealed Sub-deliverable A is actual compiler work). | none | WORKER BRIEF AUTHORED — [`r1c-a-t-testgen-schema-extensions-worker.md`](r1c-a-t-testgen-schema-extensions-worker.md); ready to dispatch |
| **R1C-B** | S | T-P0 fixture authoring: 3 TestClaim fixtures (`p0_repeat_string_correct` Day-1; `p0_no_fabrication_sentinel` ext; `p0_rest_ops_aligned` ext). Features already work (oracle test for `repeat_string` is green; sentinel + ops require schema extensions from R1C-A IF they require new predicates beyond DB-15's existing surface — verify at brief authoring). | possibly R1C-A (if `[ext]` predicates need new shapes) | WORKER BRIEF AUTHORED — [`r1c-b-t-p0-fixtures-worker.md`](r1c-b-t-p0-fixtures-worker.md); dispatchable (with audit) |
| **R1C-C** | XS | T-Sub fixture authoring: 1 TestClaim fixture for `sub_type_alias_where_lowers` (PR #703 already landed the feature). | none | **R1C-C: 1/1 gates green** — PR #879 / merge `adda0eac` (`DeclarationHasRefinement("PositiveInt")` strict `.dag` receipt; `sub_type_alias_where_lowers_gate` runner green). |
| **R1C-D** | M-L | T-PB census-as-`.dag` wiring: 6 PB census gates as `.dag` TestClaims. Merges what was previously T-PB-A and T-PB-B into one lane because both share the predicate-shape work R1C-A scopes; splitting would duplicate the schema-consumer pattern. Authoritative ratchets remain `EXPECTED_HAND_AUTHORED_NON_TEST` + `EXPECTED_HAND_AUTHORED_FRAGMENTS` (T-PB-A subset) and `EXPECTED_HAND_AUTHORED_TEST` (T-PB-B subset) in `src/v3/compiler/tests/integration/sg0_census_test.rs`; the `.dag` TestClaims read those census values via the predicate shape R1C-A scopes. | R1C-A | WORKER BRIEF AUTHORED — [`r1c-d-t-pb-census-as-dag-worker.md`](r1c-d-t-pb-census-as-dag-worker.md); dispatch gated on R1C-A Sub-deliverable B landing |
| **R1C-E** | S | T-Emit `.dag` TestClaim wrappers: 3 ExecuteCommand-based wrappers around the existing host harness (`emit_rust_fixtures_rustc_green` `[ext: ExecuteCommand]`; `emit_generic_bounds_survive` `[ext]`; `emit_omni_demo_fixtures_green` `[ext: ForAllTargets + ExecuteCommand]`). PB-Runtime `ExecuteCommand` runner landed PR #792, so this lane is dispatchable Day-1. Existing host harness in `tests/boundary/m1_3_emit_rust_test.rs` and `tests/boundary/m1_5_emit_omni_demo_test.rs` becomes the input to the `.dag` wrapper, not the gate authority. | none | WORKER BRIEF AUTHORED — [`r1c-e-t-emit-dag-wrappers-worker.md`](r1c-e-t-emit-dag-wrappers-worker.md); dispatchable Day-1 |
| **R1C-F** | S | T-Demo user-authored-lens fixture: 1 TestClaim fixture for `demo_user_authored_lens_rejects_violating_program` (consumes `user_authored_lens_compiles` from T-LensAPI which is GREEN). | none | **R1C-F: 1/1 gates green** — PR #880 (`demo_user_authored_lens_rejects_violating_program_suite` Passes via `LensOutputEquals(named_function_count, …)`; demo blurb at [`docs/demos/r1c-f-user-authored-lens-rejection.md`](../demos/r1c-f-user-authored-lens-rejection.md)). |

**Lane mutual-exclusivity property:** each lane owns a disjoint set of fixture files and runner-dispatch arms. R1C-B writes new TestClaim fixtures under `tests/fixtures/` keyed `p0_*`; R1C-C writes one keyed `sub_type_alias_where_*`; R1C-D writes 6 keyed `pb_*`; R1C-E writes 3 keyed `emit_*`; R1C-F writes 1 keyed `demo_user_authored_lens_*`. R1C-A is the schema-extension lane — it touches the runner's predicate-dispatch table and the DB-15 schema, no fixture files. PR conflicts between lanes are structurally impossible at the source-surface level.

**Critical path:** `R1C-A → R1C-D` (schema must land before census-as-`.dag` can compile). All other lanes parallel-dispatchable Day-1.

## Cross-lane / cross-program dependencies

**Produces (signals to user):**
- Lane-close declarations as each of R1C-A through R1C-F's gate predicates evaluate true. Aggregated R1 all-gates-green signal is the dissolution trigger for this manager.

**Consumes:**
- T-LensAPI lane close (already GREEN) — required for R1C-F.
- PB-Runtime ExecuteCommand runner (PR #792, landed) — required for R1C-E.

**Adjacent territory:**
- **Pure Bootstrap to Zero program** (`docs/design-pure-bootstrap-zero.md`) — R1C-D shares predicate-shape work with the PB program's 0-floor target. Coordinate with the eventual R2 Pure Bootstrap Manager (which spawns post-R1) to ensure R1C-D's predicate shape is forward-compatible with post-R1 PB program work.
- **R2 Release Manager** (per `docs/r2-structure.md` §"R2 Release Manager") — receives this manager's all-gates-green signal as the R1→R2 transition trigger.

## Authority

- **Autonomous dispatch authority through R1 close.** Authors all R1C lane sub-briefs without Director. Dispatches workers against each lane. Resolves R1 closure-internal scope refinements; escalates blockers and substrate-class scope changes to Director.
- **Per `INVARIANTS.md §P5` dispatch-discipline:** every R1C worker brief that introduces a scaffold names its dissolution trigger + adjacent ROADMAP debt row + contributes-or-defers stance; every PR introducing hand-Rust under `src/v3/` fills the per-PR gate.
- **What this manager does NOT have authority over:** reopening lane-acceptance gate semantics (those are owned by the original R1 lane authority + ROADMAP single-authority rule); deferring strict-interpretation gates to pragmatic ratchet-only enforcement (locked decision per user direction 2026-04-26).

## Reporting cadence

- **Lane-close → user (release coordination):** each R1C lane closure surfaces an "it runs" artifact per the demo discipline pattern from `docs/r2-structure.md` §"Demo discipline" (running fixture + 1-paragraph "what this demonstrates"; before/after for new gate predicates).
- **Aggregated R1 all-gates-green declaration → Director:** triggers R1→R2 transition mechanics per `docs/r2-structure.md` §"Transition mechanics" step 1.
- **Blockers + scope changes → Director.**

## Sub-briefs (authored / pending)

Authored — all 6 lane worker briefs landed on PR #847:
- R1C-A: [`r1c-a-t-testgen-schema-extensions-worker.md`](r1c-a-t-testgen-schema-extensions-worker.md) (M-L; 3 sub-deliverables; ready to dispatch)
- R1C-B: [`r1c-b-t-p0-fixtures-worker.md`](r1c-b-t-p0-fixtures-worker.md) (S; 3 fixtures; dispatchable with audit step for `[ext]` gates)
- R1C-C: [`r1c-c-t-sub-fixture-worker.md`](r1c-c-t-sub-fixture-worker.md) (XS; 1 fixture; **closed by PR #879**)
- R1C-D: [`r1c-d-t-pb-census-as-dag-worker.md`](r1c-d-t-pb-census-as-dag-worker.md) (M-L; 6 fixtures; dispatch gated on R1C-A Sub-deliverable B landing)
- R1C-E: [`r1c-e-t-emit-dag-wrappers-worker.md`](r1c-e-t-emit-dag-wrappers-worker.md) (S; 3 wrappers; dispatchable Day-1)
- R1C-F: [`r1c-f-t-demo-user-lens-fixture-worker.md`](r1c-f-t-demo-user-lens-fixture-worker.md) (S; 1 fixture; dispatchable Day-1)

Pending: none. The R1 Closure Manager lane queue is fully authored — manager dispatches against the worker briefs at spawn (or pre-spawn if R1 closure work begins before R2 spawn declaration).

## Working state (fill on dispatch)

Lane status table refreshes here as work lands. Initial state: all 6 lane worker briefs authored; dispatch sequence: R1C-A first (critical-path enabler), then R1C-D after R1C-A Sub-deliverable B lands; R1C-B/C/E/F parallel-dispatchable Day-1.

| Lane | Gates (strict `.dag` receipts) | Status |
| --- | --- | --- |
| R1C-C | `sub_type_alias_where_lowers` (`DeclarationHasRefinement("PositiveInt")` on the DB-11 witness; `sub_type_alias_where_lowers_gate` in `r1_gates.template.dag` + `test_runner_runs_sub_type_alias_where_lowers_gate`) | **Closed — PR #879 merged** (`adda0eac`; all CI green before squash merge) |

## Cross-refs

- Parent: [`ROADMAP.md §"Release R1 Program"`](../../ROADMAP.md) — R1 lane structure + acceptance authority.
- Strict-interpretation source: [`THESIS.md §"Tests are structural data"`](../../THESIS.md) — "the release gate IS a `.dag` program."
- Schema authority: [`ROADMAP.md §"Lane acceptance — .dag gates"`](../../ROADMAP.md) — Day-1 vs ext predicate tagging.
- T-TestGen scoping authority: ROADMAP line 65 — "T-TestGen also owns scoping the predicate shape for `[ext]` gates that other lanes consume."
- Census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs` — `EXPECTED_HAND_AUTHORED_NON_TEST` + `EXPECTED_HAND_AUTHORED_FRAGMENTS` + `EXPECTED_HAND_AUTHORED_TEST` ratchets.
- ExecuteCommand authority: PR #792 (PB-Runtime worker brief; `t_pb_b_1_execute_command_boundary` receipt).
- R2 transition: [`docs/r2-structure.md` §"Transition mechanics"](../r2-structure.md) — R1 close → R2 spawn sequence.
