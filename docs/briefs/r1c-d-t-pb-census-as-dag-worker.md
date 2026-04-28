# R1C-D — T-PB census-as-`.dag` worker brief `(M-L, R1 close)`

> **R1 Closure Manager dispatch.** Per [`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md) §"Owned deliverables" lane R1C-D. Closes 6 unwired T-PB census gates (T-PB-A ×4 + T-PB-B ×2) under strict reading of `THESIS.md §"Tests are structural data"`. **Depends on R1C-A** (T-TestGen schema extensions) — predicate shapes must land before this brief dispatches. Reports to R1 Closure Manager.

## Read first

- **[`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md)** — manager scope; R1C-A → R1C-D dependency edge.
- **[`docs/briefs/r1c-a-t-testgen-schema-extensions-worker.md`](r1c-a-t-testgen-schema-extensions-worker.md)** — sibling brief; Sub-deliverable B authors the predicate shapes this brief consumes. **R1C-A Sub-deliverable B is landed on main (PR #939).** Dispatch is worker-availability gated, not schema-blocked.
- **[`docs/briefs/r2-manager-brief-authority-matrix.md`](r2-manager-brief-authority-matrix.md)** — local review checklist (status consistency + pre-author verification invariants apply).
- **[`docs/escalation-paths.md`](../escalation-paths.md)** — escalation discipline.
- **[`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md)** — LIVE 2026-04-25; 0-floor target authority for the cascade-promoted gate semantics.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`TESTING.md`](../../TESTING.md)**.

## Pre-author audit receipt (per matrix verification invariant)

Pre-author grep against main HEAD `407a8bcb1` established:

- **Census authority** lives at `src/v3/compiler/tests/integration/sg0_census_test.rs`:
  - `EXPECTED_HAND_AUTHORED_NON_TEST` (lines 166-206) — 41 file paths (T-PB-A subset).
  - `EXPECTED_HAND_AUTHORED_TEST` (lines 213-288) — 77 test file paths (T-PB-B subset).
  - `EXPECTED_HAND_AUTHORED_FRAGMENTS` (line 297) — 1 entry (`src/v3/compiler/parse_parser_body.txt`).
  - Drift-detection test `sg0_v3_hand_authored_census` at `:387-451` panics with narrative on mismatch (no generated-partition branching beyond set subtraction).
  - Sub-ratchet checks: non-test sub-ratchet at `:511-539`; test sub-ratchet at `:539-565`. T-PB-A / T-PB-B split is structurally enforced.
- **Existing `.dag` test fixtures with external state**: `r1_gates.dag` and `t_demo_fixtures.dag` have **no template** for fixtures that read file lists or census state. R1C-D authors the first such fixtures.
- **0-floor target authority**: [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) (LIVE 2026-04-25) — non-test hand-Rust + Rust-authored tests both → 0; cascade promotion retracts the prior `TESTING.md §"Post-R2 shape"` residual carve-out.
- **6 PB census gate names + tags** (per [`ROADMAP.md` lines 67-68](../../ROADMAP.md)):
  - T-PB-A: `pb_hand_rust_at_shim_floor` `[ext]`, `lens_producer_files_remaining` `[ext]`, `pb_self_compile_fixed_point` `[ext]`, `pb_compiler_std_ratchet_zero` `[ext]`.
  - T-PB-B: `pb_test_file_generated_from_dag` `[ext]`, `pb_rust_tests_outside_residual_zero` `[ext]`.
  - All `[ext]` = require T-TestGen schema extensions (R1C-A Sub-deliverable B).
- **Lens-producer enumeration**: per ROADMAP line 65, `lens_producer_files_remaining` enumeration is "declared in `sg0_census_test.rs` at scoping time" — meaning the lens-producer subset of `EXPECTED_HAND_AUTHORED_NON_TEST` is a list R1C-A scopes alongside the predicate shape; R1C-D's fixture cites it.

## Frame — wire 6 census gates as `.dag` TestClaims

Under strict reading of `THESIS.md §"Tests are structural data"` ("the release gate IS a `.dag` program"), the 6 PB census gates currently enforced as Rust ratchets in `sg0_census_test.rs` must also be wired as `.dag` TestClaim fixtures. The Rust ratchets stay as authoritative drift-detection (the panic + narrative mechanism); the `.dag` fixtures are the **release gates** that R1 close evaluates.

R1C-D doesn't dissolve the Rust ratchets — they remain as the authority `.dag` fixtures read against. R1C-D adds the `.dag` layer on top.

Each gate's `.dag` fixture reads the same census-authority data the Rust ratchet reads, expressed via R1C-A's predicate shape. The fixture evaluates `Pass` when the census state matches the gate's threshold.

## Six fixture-authoring deliverables

Each gate is a separate fixture. All 6 land in `r1_gates.dag` (or sibling `r1_pb_census_gates.dag` if the file gets too large — at the worker's discretion).

### D.1 — `pb_hand_rust_at_shim_floor` `[ext: CensusBoundCheck]`

**Predicate shape (per R1C-A Sub-deliverable B):** `CensusBoundCheck { authority: DeclarationRef, list_constant: Symbol, bound: Int }`.

**Fixture body:** `TestClaim` with `predicate = CensusBoundCheck { authority: <ref to sg0_census_test>, list_constant: "EXPECTED_HAND_AUTHORED_NON_TEST", bound: 0 }`.

**Acceptance:** gate evaluates `Pass` when `EXPECTED_HAND_AUTHORED_NON_TEST.len() ≤ 0` (i.e., when the list is empty under 0-floor cascade-promotion). Pre-cascade-promotion baseline (~41 entries today) means the gate is RED until cascade-promotion work closes the list to 0; R1C-D authors the fixture, gate-close pacing depends on the dissolution work owned by Pure Bootstrap to Zero program.

**Note on R1 close:** the gate compiling is sufficient for R1's *meta-acceptance (a)*; gate evaluating `Pass` is *meta-acceptance (b)*. If the cascade-promotion 0-floor isn't reached by R1 close declaration time, this gate stays RED and R1 doesn't close; that's a Director-arbitrated scope decision (extend R1 timeline vs concede the gate per cascade authority). **STOP-AND-ESCALATE if this scenario surfaces.**

### D.2 — `lens_producer_files_remaining` `[ext: CensusSubsetCount]`

**Predicate shape:** `CensusSubsetCount { authority: DeclarationRef, list_constant: Symbol, subset_predicate: <pattern matcher> }`.

**Fixture body:** `TestClaim` with `predicate = CensusSubsetCount { authority: <ref>, list_constant: "EXPECTED_HAND_AUTHORED_NON_TEST", subset_predicate: <lens-producer pattern> }`. Subset predicate identifies lens-producer files; R1C-A scopes the enumeration alongside the predicate shape (per ROADMAP line 65).

**Acceptance:** gate evaluates `Pass` when count of lens-producer files in census = 0. Same dissolution-pacing dependency as D.1.

### D.3 — `pb_self_compile_fixed_point` `[ext: FixedPointConverges]`

**Predicate shape:** `FixedPointConverges { compile_target: Path, expected: SnapshotRef }`.

**Fixture body:** `TestClaim` asserting `compiler.dag` self-compile produces bit-identical stage0 Rust + emitted artifacts.

**Acceptance:** gate evaluates `Pass` when fixed-point holds. Receipt currently lives in `tests/integration/pb1_bootstrap_full_snapshot_test.rs` per the audit; R1C-D wraps that as a `.dag` predicate.

**Note:** if R1C-A's predicate shape proposal for `FixedPointConverges` doesn't capture all of what the integration test asserts, escalate per STOP discipline rather than over-shaping the predicate.

### D.4 — `pb_compiler_std_ratchet_zero` `[ext: RatchetZero]`

**Predicate shape:** `RatchetZero { authority: DeclarationRef, ratchet_kind: ConsolidationRatchet }`.

**Fixture body:** `TestClaim` reading the consolidation-ratchet authority (compiler-local types not in positive-def set count).

**Acceptance:** gate evaluates `Pass` when ratchet count = 0.

### D.5 — `pb_test_file_generated_from_dag` `[ext: GeneratedFromDag]`

**Predicate shape:** `GeneratedFromDag { authority: DeclarationRef, generated_paths: List<Path> }`.

**Fixture body:** `TestClaim` asserting test files are generated-from-`.dag` rather than hand-authored.

**Acceptance:** gate evaluates `Pass` when the generated-paths list covers the partition.

### D.6 — `pb_rust_tests_outside_residual_zero` `[ext: CensusBoundCheck]`

**Predicate shape:** `CensusBoundCheck` (same shape as D.1, applied to the test subset).

**Fixture body:** `TestClaim` with `list_constant: "EXPECTED_HAND_AUTHORED_TEST", bound: 0`.

**Acceptance:** gate evaluates `Pass` when `EXPECTED_HAND_AUTHORED_TEST.len() = 0` per cascade promotion.

## Slice — author 6 fixtures, single PR

Single PR (or split per gate if fixture-authoring discovers per-gate edge cases that warrant separate review). Steps:

1. **Verify R1C-A Sub-deliverable B has landed** (predicate shapes + runner dispatch arms in `test_runner.rs`). If not landed, STOP.
2. Author 6 fixtures in `r1_gates.dag` (or sibling) using the predicate shapes.
3. For each fixture, verify runner dispatch returns the correct evaluation against current census state.
4. Update R1 gate status table in this brief (or in `r1-closure-manager.md`'s working state) with per-gate `Pass` / `Fail` status.
5. Surface gates that are RED to R1 Closure Manager as dissolution-work-pending (cross-manager queue).

## Acceptance (R1C-D as a whole)

- [ ] R1C-A Sub-deliverable B's predicate shapes are landed on main BEFORE this brief dispatches (verify; STOP otherwise).
- [ ] 6 `.dag` TestClaim fixtures authored, one per gate.
- [ ] Each fixture references the correct census authority + predicate shape.
- [ ] Runner dispatches each fixture cleanly (no `NotYetImplemented` returns).
- [ ] Gates evaluate against current census state — `Pass` or `Fail` per current data, not error.
- [ ] RED gates surfaced to R1 Closure Manager with dissolution-work-pending status (cross-manager queue → Pure Bootstrap to Zero program coordination).
- [ ] All other R1 gates remain green.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] R1 Closure Manager lane status updated to "R1C-D: 6/6 fixtures authored; gates evaluating against current census; N/6 green at landing time."

## STOP-AND-ESCALATE

Per [`docs/escalation-paths.md`](../escalation-paths.md):

- **R1C-A Sub-deliverable B has not landed** → STOP. R1C-D requires the predicate shapes; do not author fixtures against undefined predicates. Escalate to R1 Closure Manager.
- **A predicate shape from R1C-A turns out not to capture the gate's actual semantics** → STOP. Surface back to R1C-A for shape revision rather than working around with creative fixture content. Co-authoring back-pressure is the right cycle; over-shaping the fixture is a `feedback_construction_over_ratchets` violation.
- **Cascade-promotion 0-floor work hasn't reduced census to 0 by R1 close declaration time** → gates D.1, D.2, D.6 stay RED. STOP and escalate to Director for R1 timeline / concession decision per `feedback_foundation_over_speed`. Do not relax gate threshold to make RED gates green.
- **Census drift discovered during fixture authoring** (Rust ratchet test fails) → STOP. The Rust ratchet is authoritative on census state; if it fails, the underlying state is in flux and `.dag` fixture authoring is premature.
- **DB-8 fixed-point drifts** → STOP immediately.

## Cross-refs

- Parent: [`docs/briefs/r1-closure-manager.md`](r1-closure-manager.md) lane R1C-D.
- Upstream dependency: [`docs/briefs/r1c-a-t-testgen-schema-extensions-worker.md`](r1c-a-t-testgen-schema-extensions-worker.md) — Sub-deliverable B (predicate shapes) is the prerequisite.
- Authority matrix categorization: Category 1 (worker brief) per [`docs/briefs/r2-manager-brief-authority-matrix.md`](r2-manager-brief-authority-matrix.md).
- Census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs:166-297` (constants); `:387-451` (drift test); `:511-565` (sub-ratchets).
- 0-floor authority: [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) (LIVE 2026-04-25).
- Gate authority: [`ROADMAP.md` lines 67-68](../../ROADMAP.md) (T-PB-A + T-PB-B `[ext]` predicate names).
- Existing fixture template: [`src/v3/compiler/tests/fixtures/r1_gates.dag`](../../src/v3/compiler/tests/fixtures/r1_gates.dag) (no census-reading fixtures yet — R1C-D authors the first).
- Cross-program coordination: Pure Bootstrap to Zero program (post-R1 PB Manager) for cascade-promotion dissolution-work-pending gates.
- Discipline anchors: `feedback_construction_over_ratchets`, `feedback_audit_adjacent_authority_first`, `feedback_thesis_gate_state_drift`, `feedback_verify_thesis_claims`, `feedback_foundation_over_speed`.
- Escalation discipline: [`docs/escalation-paths.md`](../escalation-paths.md).
