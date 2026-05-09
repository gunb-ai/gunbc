# R3 Cluster M Phase 3 — R1C-D/E Pilot Worker Brief (pre-Phase-1)

**Status:** PRE-AUTH DISPATCH-READY (worker-facing). 3-test pilot per Phase 3 coordinator skeleton §3 (R1C-D/E sub-class flagged pre-Phase-1-eligible at gunbc#828 c#4413268466).

**Owner**: worker (TBD on dispatch); coordinator: Verification Mgr (wise-bear-525 / gunbc#2075).

**Authority**:
- Phase 3 coordinator skeleton: [`r3-v-cluster-m-84-bulkport-coordinator.md`](r3-v-cluster-m-84-bulkport-coordinator.md) §3 R1C-D/E row.
- Director sanity-check pilot greenlight: gunbc#828 c#4413268466 (\"3-test sub-batch as pre-Phase-1 pilot dispatch\").
- Director re-task (Task A): gunbc#828 c#4413880134 (2026-05-09).
- Locked design: [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §3 (migration audit) + §1.1 (no hand-authored Rust test files post-R3).
- Strict-zero close-condition: Director Ask 4 ratification (no closure-allowed exceptions; bulk-port = full \`EXPECTED_HAND_AUTHORED_TEST\`).

**Substrate-of-truth (cite-and-execute)**: locked design §3 + Phase 3 skeleton.

---

## §0. Scope

3 hand-Rust tests under `src/v3/compiler/tests/integration/`:
1. `r1c_d_pb_census_gates_test.rs` — runner-side receipt for 6 PB census `.dag` TestClaim fixtures.
2. `r1c_e_emit_gates_dag_test.rs` — runner-side receipt for T-Emit `.dag` TestClaim wrappers (bin-substitution at compile time via `env!("CARGO_BIN_EXE_r1c_e_emit_gates")`).
3. `r1c_e_emit_gates_omni_dag_test.rs` — runner-side receipt for multi-target `omni-demo` claim (`#[ignore]`; needs go + python3 + cargo).

These are all **test wrappers** that already consume `.dag` `TestClaim` declarations; the hand-Rust exists to handle (a) bin-path substitution, (b) `#[ignore]` gating for toolchain-required tests, (c) the runner-side receipt assertions.

**Pilot purpose**: validate the Phase 3 bulk-port pattern at small scale before Cluster M Phase 1 substrate locks. Catch discipline-pattern issues early (vs surfacing mid-Phase-3 across 102 tests). SG-0 census decrements by 3 (or fewer with explicit reasons).

## §1. Out of scope (STOP+PING)

- Inventing new `TestPredicate` variants — use existing `ExecuteCommand` (T-Emit case) + suite/dispatch evaluation (R1C-D case). Both are existing 🟡 Scaffold per locked design §1 / `src/v3/std/verification.dag`.
- Touching `r1_gates.template.dag` content — the splice discipline (per #973 / r1c_e parallel) is established; do not regress.
- Cementing-test discipline (#87) work — different brief.

## §2. Migration discipline (per-test analysis)

Each of the 3 tests has a different reason for hand-Rust today:

**R1C-D — `r1c_d_pb_census_gates_test.rs`** (~52 lines):
- Hand-Rust reason: runner-side receipt that 6 specific census predicate dispatches are wired (no `NotYetImplemented` from missing dispatch arms). Plus per-claim Pass/Fail-vs-NotYetImplemented assertion.
- Migration target: a new `.dag` `TestClaim` predicate variant... NO — that would be substrate change. **Better**: testgen-driven Rust test code generated from a declarative DAG describing \"these 6 claim names must dispatch (not NotYetImplemented)\". Per locked design §1.3 Path B (emit Rust test code from `.dag` declaration).
- Acceptance: testgen produces a Rust test that runs the 6-suite and asserts no-NotYetImplemented, replacing the hand-Rust file.

**R1C-E — `r1c_e_emit_gates_dag_test.rs`** (~70 lines):
- Hand-Rust reason: bin-path substitution at compile-time via `env!("CARGO_BIN_EXE_r1c_e_emit_gates")` — the `.dag` template has a `__R1C_E_BIN__` placeholder that's substituted before `compile_to_dag`. Plus suite-runs-all-Pass receipt.
- Migration target: testgen-Rust path (Path B) where the substitution is part of the generated code; placeholder + bin name encoded in the `.dag` declaration; testgen emits the substitution + run.
- Acceptance: testgen produces Rust test wrapper from the `.dag` template + bin name; replaces the hand-Rust file.

**R1C-E omni — `r1c_e_emit_gates_omni_dag_test.rs`** (~75 lines, `#[ignore]`):
- Hand-Rust reason: same bin-substitution pattern; `#[ignore]` because needs go + python3 toolchain.
- Migration target: same testgen-Rust shape with ignore-attribute encoded in `.dag` declaration (e.g., a `requires: [Toolchain(\"go\"), Toolchain(\"python3\")]` field on the TestClaim or a `tags: [\"ignore-default\"]` annotation that testgen emits as `#[ignore]`).
- Acceptance: testgen produces Rust test wrapper with `#[ignore]` attribute derived from declaration; replaces hand-Rust file.

## §3. First migration target

Suggested order (smallest-first, building testgen capability):
1. **R1C-D first** (smallest, no bin-substitution, no `#[ignore]`). Validates testgen-from-declarative-suite pattern.
2. **R1C-E (non-omni)** second. Validates bin-substitution emit pattern.
3. **R1C-E omni** third. Validates `#[ignore]` attribute emit pattern.

If testgen capability for any of these surfaces shape-questions (e.g., `requires:` on TestClaim variant for toolchain gating), **STOP+PING** the Verification Mgr (wise-bear-525); shape questions feed back into Cluster M Phase 1 canvas authoring.

## §4. Acceptance criteria (per migration step)

For each of the 3 tests:
1. New testgen target generates Rust test code (Path B per locked design §1.3) consuming the existing `.dag` `TestClaim` declaration (or a new declaration with placeholder/ignore attributes).
2. Generated Rust test passes via BuildBuddy (`cargo test -p v3-compiler --test integration <test_name>`).
3. The corresponding hand-Rust file under `tests/integration/` is **deleted** (not stub-replaced).
4. SG-0 census `EXPECTED_HAND_AUTHORED_TEST` decrements by 1.
5. PR body: `SG-0 hand-path delta: -1`.

Final pilot-close acceptance (post-3-test land):
- SG-0 census −3 from baseline.
- Pilot results inform Cluster M Phase 3 per-class brief authoring (lessons learned section in PR body).

## §5. Receipt + ledger updates

- Per-test: SG-0 census line removes the hand-Rust file entry.
- Lane-Mgr signoff (per Phase 3 coordinator skeleton §4): Verification Mgr (this lane) reviews behavioral fidelity for R1C-D/E (these are Verification-tier wrappers).
- Pilot receipt: a short \"R1C-D/E pilot lessons\" section appended to Phase 3 coordinator skeleton when all 3 tests land.

## §6. Velocity context

Per Phase 3 skeleton §6: 3 of ~50-65 SG-0 test entries dissolve via this pilot. Reverses the +3.3/day SG-0 growth trajectory immediately (per Director c#4413268466 reasoning §2). Builds Phase 3 confidence; surfaces testgen-capability shape questions early.

## §7. Cross-Mgr dependencies

- Substrate Mgr (warm-wolf-698 #2068): if R1C-E `requires:` toolchain-gating surfaces a TestClaim shape question, that's a substrate canvas; STOP+PING and Verification Mgr surfaces to Substrate.
- No other cross-Mgr dependencies expected at this scale.

## §8. Worker dispatch posture

Pre-Phase-1 pilot dispatch authorized per Director c#4413268466 + re-task c#4413880134. Spawn-authority queued at PM #846 alongside Cluster M Phase 1 + F-α + G1.a + Evaluator second-strategy.

Standing-authority dispatch by Verification Mgr post-PR-merge.

---

**End of brief.**
