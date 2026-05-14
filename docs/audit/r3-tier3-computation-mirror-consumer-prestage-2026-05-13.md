# Tier-3 computation mirror — consumer-wiring pre-stage audit

**Authority:** `docs/r3-program-plan.md` §1.8 row **#2** `tier3_computation_mirror_dissolved` — **CONSUMER_LANDED** (PR #2789 trivial constructors + `kernel_algebra_profile` / `type_iteration_dimension` substrate-authority ratchet); **PASSING** still requires Evaluator-backed dissolution of remaining **`SizeBound` / `CallPattern`** host mirrors and wider `std.computation` mirror surface (`lower_call_pattern`, `size_bound_param`, `is_constant_bound`, `constant_bound_value`, `algebra_profile_to_dimension`).

**Mgr lane:** R3 Debt-Paydown (`zesty-boar-261`). **Director dispatch:** `msg_2e9a11ee-3a11-4d1c-a03b-999a6cf209fc` — **pre-stage only** (Evaluator Mgr execution lane still escalated blocked per operator `msg_acf78d37`; no host `evaluate_body` claims here).

**Snapshot:** 2026-05-13 — inventory from ripgrep over `src/v3/compiler/` at authoring time (hand + generated + tests + benches).

---

## §1. Symbol inventory (host mirror API surface)

| Symbol | Canonical definition | Notes |
|--------|---------------------|--------|
| `SizeBound` | `src/v3/compiler/src/dag.rs` (`mod computation`) | Host coproduct scaffold; §1.8 #2 residual mirror class. |
| `CallPattern` | `dag.rs` (same module) | Host coproduct; pairs with `SizeBound` via `lower_call_pattern`. |
| `lower_call_pattern` | `dag.rs` | Maps `CallPattern` → `LoweringTarget` / `SizeBound`. |
| `size_bound_param` | `dag.rs` | Extracts structural param witness where applicable. |
| `is_constant_bound` | `dag.rs` | Predicate on `SizeBound`. |
| `constant_bound_value` | `dag.rs` | Numeric witness for constant bounds. |
| `algebra_profile_to_dimension` | `dag.rs` | Maps `AlgebraProfile` → `IterationDimension`. |
| `type_iteration_dimension` | `dag.rs` | **Slice B ratcheted:** must delegate through `BOOTSTRAPPED_DAG.kernel_algebra_profile` (see `m2_substrate_inhabitance_test.rs`). |

`bootstrap_generated.rs` / `bootstrap_generated_without_parse_surface.rs` contain **string table entries** for the symbol names (bootstrap metadata) — **not** behavioral consumers.

---

## §2. Consumer-side call-site / type-use inventory

### 2.A Hand-written integration — **primary ratchet + behavioral consumer**

| Artifact | Role |
|----------|------|
| `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs` | Imports and exercises **`lower_call_pattern`**, **`size_bound_param`**, **`is_constant_bound`**, **`constant_bound_value`**, **`algebra_profile_to_dimension`**, **`type_iteration_dimension`**, `CallPattern`, `SizeBound`, `AlgebraProfile`, …; hosts **`tier3_computation_mirror_*`** ratchets; documents that **full mirror dissolution** requires **evaluated** `std.computation` block (comments ~L528–L551). |

### 2.B Performance harness

| Artifact | Role |
|----------|------|
| `src/v3/compiler/benches/tier3_mirror_perf.rs` | **`lower_call_pattern`**, **`type_iteration_dimension`**, `CallPattern`, `positive_descent_count` — Phase-1 timing toward tier-3 perf budget brief. |

### 2.C Generated lenses (Dag-emitted Rust; still **typed** against host `CallPattern`)

| Artifact | Role |
|----------|------|
| `src/v3/compiler/src/complexity_lens_generated.rs` | From `src/v3/lenses/complexity.dag` — **`CallPattern`** in `complexity_when_descent_unknown`, `pattern_to_iter_bound`, … |
| `src/v3/compiler/src/lens_cost_symbolic_generated.rs` | From `src/v3/lenses/cost.dag` — **`CallPattern`** in `pattern_to_iter_bound` / recursive transform cost paths. |

### 2.D Integration tests — **indirect** (no direct `lower_call_pattern` import)

| Artifact | Role |
|----------|------|
| `src/v3/compiler/tests/integration/lens_behavioral_parity_demonstration_test.rs` | Imports other `dag` types (`SymbolicCost`, `WorkflowEffect`, …) for gate **#73**; not a direct consumer of the §1 table’s five free functions. |
| `src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs` | `complexity_of` / `dag` behavior types — Band-C receipt; same indirect relationship. |

### 2.E Census / comments

| Artifact | Role |
|----------|------|
| `src/v3/compiler/tests/integration/sg0_census_test.rs` | Comment near census const references mirror names (documentation-only tie to gate #2 narrative). |

---

## §3. Sequencing analysis (retirement readiness — **pre-execution**)

**Hard dependency:** Evaluator Mgr lane must supply **evaluated `std.computation`** authority before the host scaffold in `dag.rs` can dissolve without losing §Acceptance-shaped evidence (see `m2_substrate_inhabitance_test.rs` comments).

**Recommended conceptual order** (consumer-wiring pre-work → execution wave when unblocked):

1. **Substrate / `.dag` authority expansion** — extend `std.computation` (and related induction/computation lowering) so **mirror semantics** are **data-first** where §P2 consumers can attach (Evaluator executes the block).
2. **Ratchet migration** — move `m2_substrate_inhabitance_test.rs` rows from “host literal” expectations toward **generated or fixture-backed** claims as the Dag surface becomes authoritative (preserves fail-closed discipline per INVARIANTS).
3. **Generated lens modules** — `complexity_lens_generated.rs` / `lens_cost_symbolic_generated.rs` **regenerate** when `complexity.dag` / `cost.dag` projections no longer need raw `CallPattern` host typing (ties to lens emitter + cost lens programs, not Debt-Paydown alone).
4. **Bench alignment** — refresh `tier3_mirror_perf.rs` only after public mirror API stabilizes on the new authority path (avoid churning perf baselines prematurely).

**Parallel to PB-0:** Retire **`NON_TEST`** census paths that are **pure wiring** to deprecated mirror call-sites **only** when a **same-PR** dissolution receipt exists (Dispatch-Discipline **(b)**). Use PB-0 cycle briefs + this audit as cross-links when opening those PRs.

**Wave-1 PB1 receipt (2026-05-14):** terminal carrier exposure for
`ShrinkFactor` / `IterationPrimitive` is now guarded by
`tier3_computation_terminal_carriers_have_no_parallel_executable_helpers`.
That test asserts the carrier shapes from the lowered `std.computation` DAG
and forbids new public helper functions that would turn those terminal
carriers into a parallel executable Rust authority. This is a narrow
CONSUMER_LANDED slice only; it does not retire the `SizeBound` / `CallPattern`
lowering scaffold or promote row #2 to PASSING.

---

## §4. Worker brief stub (fires when Evaluator Mgr lane unblocks)

**Title (placeholder):** *Tier-3 computation mirror — consumer-wiring execution worker*

**Dispatch shape (outline):**

1. **Goal:** §1.8 #2 → **PASSING** — no remaining host-mirror obligations for the named `std.computation` surface; `computation_lowering_rust_mirror_matches_dag_authority` and related rows green under Evaluator-backed `std.computation`.
2. **Inputs:** This audit §§1–3; Substrate Mgr / Evaluator Mgr canvases; `docs/r3-structure.md` §Acceptance for Tier-3 dissolution.
3. **Deliverables:** PR sequence removing `dag.rs` mirror variants **only** behind substrate + consumer receipts; integration tests updated or replaced with `.dag` TestClaims per §P2 where applicable.
4. **STOP:** Any new substrate sum-type without Director canvas → escalate (Mgr cannot mint parallel authority).

---

## §5. Changelog

| Date | Change |
|------|--------|
| 2026-05-13 | Initial pre-stage audit (Director `msg_2e9a11ee`). |
