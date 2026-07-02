# Affected-set de-fork: `v2.lens.affected_set` as single authority; dissolve Rust parallel implementation

**Status: PARTIAL IMPLEMENTATION — prep (#6065) + witness (a) partial (#6072) landed; Steps 4–5 blocked.**

**ROADMAP:** `1-affected-set-defork` (`dag/gunbc/roadmap_authority.dag:173`).

**Merge sequence:** PR #6065 (disposition-kernel prep) merges first; #6072 rebases onto fresh `main` so its diff collapses to witness-(a)-delta only. Do **not** merge #6072 before #6065.

---

## Implementation receipts (2026-07-01)

| Step | Status | Receipt |
|------|--------|---------|
| Prep — disposition kernel both axes | **GREEN** (PR #6065) | `floor_witness_run_disposition` + `function_edited`; `affected_set_disposition_both_axes_test.dag`; `floor_disposition_kernel_alignment` in `cli_run.rs` (disposition tautology on shared Rust inputs — NOT witness (a)) |
| Step 3 witness (a) — **edited_test_fns axis** | **GREEN (partial)** | `floor_witness_a_prove` in `cli_run.rs`: fixture unified diff → Rust `edited_test_fns` vs independent `.dag` `floor_test_fn_declaration_edited`; mandatory RED under-selection; `.dag` claim `affected_set_witness_a_prove_test.dag` |
| Step 3 witness (a) — **node-frontier axis** | **BLOCKED** | Whole-tree `InferredTree` + `NodeArtifactProvenance` over live corpus — same resolve-grounding gate as `wiring_liveness_whole_tree` / `whole_tree_resolved_ctx` (`v2.lens.resolved_imports` open thread). `.dag` closure smoke on `provenance_producer` fixture only; Rust `NodeFrontierSeeds` equivalence deferred |
| Step 4 migrate floor | **NOT STARTED** | Impl-2 live |
| Step 5 delete Rust parallel | **NOT STARTED** | `NodeFrontierSeeds`, `entry_touches_frontier_seeds`, etc. intact |

**Hand-Rust harness discipline:** `floor_witness_a_prove` uses **deterministic fixture unified diffs** (structured shape identical to CI git diff parsing) so every checkout — including `main` after merge — executes impl-vs-impl proof by execution. Branch-only `origin/main...HEAD` non-empty asserts were removed (§5: a gate that only passes on feature branches is not a stable floor witness).

---

## Step 1 — AUDIT: N count + list

**N = 2** genuine parallel reverse-reachability implementations.

### Implementation 1 — `.dag` authority (INERT)

`v2.lens.affected_set` subsystem:
- `src/v2/lens/affected_set.dag`: `affected_set_closure_step` (L356), `affected_set_closure_pass` (L378), `affected_set_closure` (L414, fixpoint), `affected_set_reading` (L431), `affected_set` (L1267)
- `src/v2/workflow/affected_set_floor_runner.dag`: `floor_witness_run_disposition`, `floor_any_node_touches_frontier`, `WitnessRunDisposition = RunWitness | SkipAssumedGreen`
- `src/v2/workflow/affected_set_selection.dag`: `ci_select_from_rerun_nodes`, `ci_select_from_affected_set` — adapter over `RerunNodeSet`/`AffectedSet`
- Primitive input: `v2.std.dependency.dependency_lens` (`src/v2/std/dependency.dag:185`) — folds over `fold_node` to produce `List<DependencyView>` fed into the closure fixpoint

**Status: INERT.** Authored, 20-file test suite under `src/v2/lens/affected_set/`, and `construction_justification: WallAfterGrounding { dissolves_to: SingleAuthority }` at `affected_set.dag:1345`. No live consumer in the CI floor execution path — confirmed via `fail_closed_lockdown.dag` and `realization_measurement_loop.dag`.

**Consumer surface already uses Implementation 1:** `v2.workflow.affected_set_selection`, `v2.std.probe_selector` (via `probe_select_affected_test_claims`), `affected_testgen_ci_runner.dag`, and `affected_set_selection.dag` all import from `v2.lens.affected_set`. The Rust floor (`cli_run.rs`) is the ONLY consumer that does not — strengthening the framing: the rest of the system already routes through the `.dag` authority; only the floor forks it.

### Implementation 2 — Rust parallel (LIVE)

`src/v1/stage0/src/cli_run.rs`:
- `NodeFrontierSeeds` struct (L3275): three fields — `overlapping_data_items: HashSet<(String, String)>`, `edited_test_fns: HashSet<(String, String)>`, `force_run_all: bool`
- `collect_frontier_seeds_from_diff_line_ranges` (L3290): diff line ranges → `NodeFrontierSeeds`
- `entry_frontier_nodes_from_seeds` (L3362): seeds → node list (requires resolved `InterpContext`)
- `entry_touches_frontier_seeds` (L3389): requires full resolved `InterpContext`; per-entry node-frontier reverse-reachability check returning `bool`
- Floor dispatch in `run_discovery_rows` (L3721–3731): per-row skip predicate — see **skip decision** note below
- `precompute_whole_tree_published_mock_keys` (L890): transitive mock-closure pre-pass (separate concern, but same skip-decision owner — see Step 4)

**Status: LIVE.** Runs on every CI floor invocation. The per-row skip (L3725) and the unconditional precompute (L3509) are the two hot paths this de-fork dissolves.

**Full skip decision (cli_run.rs:3721–3731):** a witness is SKIPPED only when BOTH conditions hold:
1. `!current_entry_touches` — node frontier: `entry_touches_frontier_seeds` returns false
2. `!function_edited` — function directly edited: `frontier_seeds.edited_test_fns` has no match for this row's `(entry, function)` pair

A witness RUNS when EITHER fires. The `.dag` authority's `floor_witness_run_disposition` currently models only condition (1) — the `function_edited` bypass (condition 2) is a gap that must be closed before migration (see Step 4 and acceptance witness (a)).

### False positives (NOT affected-set / reverse-reachability for change propagation)

- `argument_reachable_from_axioms` — axiom inference reachability, different query, different domain
- `doc_reachability` (`src/v1/stage0/src/doc_reachability_project.rs`) — markdown link walking, not `.dag` node reachability
- `block_construct_closure_expr` — expression-scope closure, not change-set propagation
- `wiring_liveness.dag` (`v2.lens.wiring_liveness`): forward wiring reachability for dead-wire detection in function parameters. **CONFIRMED FALSE POSITIVE** — different query direction (forward, not reverse), different semantic purpose, different consumer. Does not implement reverse-reachability for change propagation.
- `v2.std.change.AffectedSet` — a type declaration, not an implementation. Shared output carrier consumed by both impls.
- `v2.workflow.affected_set_selection` — a consumer/adapter of Implementation 1 (part of the same subsystem), not an independent impl.

---

## Step 2 — PICK: single authority

**Keeper: Implementation 1** (`v2.lens.affected_set` + `affected_set_floor_runner` + `dependency_lens`).

Rationale: grounded in the substrate as a typed fixpoint over `DependencyView`; has test suite proving superset-safety (`closure_superset_safe.dag`); substrate already marks it as the awaited authority (`WallAfterGrounding { dissolves_to: SingleAuthority }`); Implementation 2 is a seed-layer Rust artifact that must shrink toward zero (DESIGN §7).

**What the `.dag` query expresses:**

Given `dependency_lens(root: graph.root) → List<DependencyView>`, compute the reverse-reachability closure from the edit-locus nodes via `affected_set_closure` (fixpoint). The result (`RerunNodeSet`) drives witness selection and precompute-skip — but the `.dag` model must cover BOTH skip axes before migration (see Step 4).

---

## Step 3 — PROVE NOT INERT: `.dag`-vs-Rust equivalence on real corpus

**This step is a HARD GATE on Steps 4–5. The strict sequence is:**

1. Prove Impl 1 correct on the REAL corpus (this step) — acceptance witness **(a)**
2. Migrate floor to CONSUME Impl 1 (Step 4) — with `.dag` floor green
3. Verify floor stays GREEN consuming it
4. ONLY THEN delete Impl 2 incrementally (Step 5), floor-green at each commit

**Never delete the live Rust implementation before the replacement is proven live-and-correct on the real corpus.**

**Acceptance bar for (a):** run both implementations against the same real gunbc corpus diff; assert that for every witness, the FULL RUN/SKIP decision from the `.dag` authority (node-frontier + function-edited — see Step 4) agrees with the Rust predicate `current_entry_touches || function_edited` (run) vs both-false (skip). The `.dag` query is superset-safe on the node-frontier axis — `affected_set_closure` result ⊇ `entry_touches_frontier_seeds` result — but must also cover the function-edited axis for the equivalence to hold end-to-end. Must use a real diff that fires at least one witness on each axis (not a clean-tree trivial case).

---

## Step 4 — CONSUME: wire the floor to the `.dag` query

Three consumer sites; the full skip decision must be modeled before wiring any of them.

**Consumer 1 — floor witness selection.** `run_discovery_rows` (cli_run.rs:3721–3731): the current Rust skip gate has TWO axes:
- Axis (i) node-frontier: `entry_touches_frontier_seeds` → maps to `affected_set_closure` + `floor_witness_run_disposition`'s `touches_frontier` parameter
- Axis (ii) function-edited: `frontier_seeds.edited_test_fns` check → currently ABSENT from `floor_witness_run_disposition`

Before migration, extend `affected_set_floor_runner.dag`'s `floor_witness_run_disposition` (or its calling context) to also take the `function_edited` result as an input. A witness runs when `touches_frontier || function_edited`; the `.dag` skip disposition must reflect that conjunction. Only after both axes are modeled can the per-row `entry_touches_frontier_seeds + function_edited` calls be replaced by the `.dag` disposition result.

**Consumer 2 — precompute-skip.** Gate `precompute_whole_tree_published_mock_keys` (cli_run.rs:3509) on the FULL "no witness will run" predicate, not just the node frontier. A scoped diff can still run witnesses via the `function_edited` path even when the `.dag` frontier is empty. Correct guard: `RerunNodeSetProduced { nodes: [] }` (empty node frontier) **AND** `edited_test_fns.is_empty()` (no directly-edited test functions). Both conditions together are required.

**Conservative gate acknowledged:** gating on empty-frontier + empty-edited-test-fns is stricter than the mock-declarer-closure disjointness gate (the closure check at cli_run.rs:890–937 would allow skipping precompute even when witnesses run, as long as none touch the mock closure). This plan chooses the simpler, conservative empty-frontier+empty-edited gate. A future tightening to the mock-closure gate is a follow-on optimization, not in scope here.

**Consumer 3 — wiring_liveness preflight.** `wiring_liveness_preflight.dag` already imports `ReExecFrontier` from `affected_set.dag` as a consumer. Verify it remains correctly positioned (consumer, not reimplementor) after migration. No migration needed.

**Seam shape:** the `.dag` query result (`RerunNodeSet`) crosses the realization boundary once at floor entry — the same point `collect_frontier_seeds_from_diff_line_ranges` is called today. One query evaluation, two consumers, zero parallel impls.

---

## Step 5 — DELETE: dissolve the Rust parallel implementation

Incremental, fail-safe. **Only begins after Step 3 (equivalence proven) + Step 4 (floor wired + green).**

1. Delete `NodeFrontierSeeds` struct (cli_run.rs:3275–3278) and its `impl` block
2. Delete `collect_frontier_seeds_from_diff_line_ranges` (cli_run.rs:3290)
3. Delete `entry_frontier_nodes_from_seeds` (cli_run.rs:3362)
4. Delete `entry_touches_frontier_seeds` (cli_run.rs:3389)
5. Remove the per-row call site (cli_run.rs:3714)

Each deletion is a separate commit; CI floor must stay green at every step.

When the Rust path is gone, `construction_justification: WallAfterGrounding { dissolves_to: SingleAuthority }` on `affected_set.dag` dissolves (the single authority has been realized).

---

## Step 6 — WALL: recurrence prevention

After N→1, a new parallel reverse-reachability implementation is a §3 violation. The wall:

The `fail_closed_lockdown.dag` INERT marker on `affected_set.dag` is removed when the live consumer wiring lands (Step 4). Add a corpus-scan lens or CI gate: any new `.rs` function that walks `InterpContext` edges for reachability outside `v2.lens.affected_set`'s call chain is flagged. A text-scan over the Rust seed is sufficient until the seed itself shrinks to zero (DESIGN §7); it does not need to be perfect, only fail-closed on the obvious re-invention pattern.

---

## Acceptance witnesses

**All three required before implementation-complete (loyal-bee + operator). Strict sequence: (a) must be green before Step 5 begins.**

**(a) `.dag`-vs-Rust FULL-PREDICATE equivalence on real corpus diff** — run both impls against the same real gunbc diff; for every witness, assert the `.dag` authority (node-frontier + function-edited, both axes) produces the same run/skip decision as the Rust predicate `current_entry_touches || function_edited`. Must fire at least one witness on each axis. Proves the `.dag` authority is not inert and is safe to substitute end-to-end (not just the node-frontier component).

**(b) N→1 confirmed** — the Rust path (Step 5) is deleted; `NodeFrontierSeeds` no longer exists in the codebase. CI floor stays green. Proves de-fork completed, not merely layered.

**(c) Real gunbc floor wall-clock + peak-RSS drop on scoped diff** — `gunbc test` on a 1-file diff with empty node frontier AND empty `edited_test_fns`: floor runs 0 witnesses AND skips the precompute. Logged wall-clock and peak-RSS drop materially vs a full-corpus baseline. On the REAL floor with live glob-discovery, not a synthetic fixture.

Sign-off on this sketch = commitment to deliver all three receipts green before merge.
