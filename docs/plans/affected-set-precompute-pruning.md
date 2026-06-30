# Affected-set de-fork: `v2.lens.affected_set` as single authority; dissolve Rust parallel implementation

**Status: DESIGN SKETCH — no implementation. Returns to stern-moth-225 → operator for sign-off before any code lands.**

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

### Implementation 2 — Rust parallel (LIVE)

`src/v1/stage0/src/cli_run.rs`:
- `NodeFrontierSeeds` struct (L3275): three fields — `overlapping_data_items: HashSet<(String, String)>`, `edited_test_fns: HashSet<(String, String)>`, `force_run_all: bool`
- `collect_frontier_seeds_from_diff_line_ranges` (L3290): diff line ranges → `NodeFrontierSeeds`
- `entry_frontier_nodes_from_seeds` (L3362): seeds → node list (requires resolved `InterpContext`)
- `entry_touches_frontier_seeds` (L3389): requires full resolved `InterpContext`; per-entry reverse-reachability check returning `bool`
- Floor dispatch in `run_discovery_rows` (L3721–3731): per-row skip predicate consuming the above
- `precompute_whole_tree_published_mock_keys` (L890): transitive mock-closure pre-pass (separate concern, but same skip-decision owner — see Step 4)

**Status: LIVE.** Runs on every CI floor invocation. The per-row skip (L3714) and the unconditional precompute (L3509) are the two hot paths this de-fork dissolves.

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

Given `dependency_lens(root: graph.root) → List<DependencyView>`, compute the reverse-reachability closure from the edit-locus nodes via `affected_set_closure` (fixpoint). The result (`RerunNodeSet`) gates **both** consumers:

1. **Witness selection** (currently Rust per-row skip) — `floor_witness_run_disposition(diff, frontier, touches_frontier)` in `affected_set_floor_runner.dag`
2. **Precompute-skip** — same query, second consumer: `RerunNodeSetProduced { nodes: [] }` (empty frontier) means no witness will run and no mock keys are consumed → precompute not called. `RerunNodeSetFailClosed` → run precompute in full (fail-closed).

Both consumers are ONE query, not two predicates. The realization seam is wiring the `.dag` query result to the Rust floor dispatcher at the floor-entry boundary — until the dispatcher itself migrates to `.dag`.

---

## Step 3 — PROVE NOT INERT: `.dag`-vs-Rust equivalence on real corpus

**Prerequisite before Steps 4–5.** The `.dag` authority is authored and green in unit tests but INERT in production. Before consuming or deleting, prove it is not inert on a real diff.

**Acceptance bar:** run both implementations against the same real gunbc corpus diff; assert `v2.lens.affected_set` result ⊇ Rust `entry_touches_frontier_seeds` result for the same diff. The `.dag` query is a superset-safe approximation by design — every witness the Rust impl marks "run" must also be marked "run" by the `.dag` impl; the `.dag` impl may add more (never fewer — that would be unsound). Superset proven by execution, not by reading the code.

This is loyal-bee acceptance witness **(a)**.

---

## Step 4 — CONSUME: wire the floor to the `.dag` query

Three consumer sites, one query:

**Consumer 1 — floor witness selection.** `run_discovery_rows` (cli_run.rs:3721–3731): replace the per-row `entry_touches_frontier_seeds(ctx, entry, seeds)` call with a membership check against the `.dag` frontier. The query runs once per floor invocation over the resolved graph; the per-row check becomes `node_in_set(frontier, entry_node)`. `NodeFrontierSeeds` disappears from this site.

**Consumer 2 — precompute-skip.** Gate `precompute_whole_tree_published_mock_keys` (cli_run.rs:3509) on `RerunNodeSetProduced { nodes: [] }` (empty `.dag` frontier). When the query returns an empty frontier, no witness will run and no mock keys are observed; the precompute is skipped. `RerunNodeSetFailClosed` → run precompute in full. This is the origin-problem optimization (wall-clock + peak-RSS on scoped diff): the precompute today runs unconditionally before the skip logic; this consumer gates it on the same single query.

**Consumer 3 — wiring_liveness preflight.** `wiring_liveness_preflight.dag` already imports `ReExecFrontier` from `affected_set.dag` as a consumer. Verify it remains correctly positioned (consumer, not reimplementor) after migration. No migration needed.

**Seam shape:** the `.dag` query result (`RerunNodeSet`) crosses the realization boundary once at floor entry — the same point `collect_frontier_seeds_from_diff_line_ranges` is called today. One query evaluation, two consumers, zero parallel impls.

---

## Step 5 — DELETE: dissolve the Rust parallel implementation

Incremental, fail-safe. After Step 4 consumers are wired and CI floor is green:

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

**All three required before implementation-complete (loyal-bee + operator):**

**(a) `.dag`-vs-Rust equivalence on real corpus diff** — run both impls against the same real gunbc diff; assert `.dag` result ⊇ Rust result. Proves the `.dag` authority is not inert and is safe to substitute. Must use a real diff that fires at least one witness (not a clean-tree trivial case).

**(b) N→1 confirmed** — the Rust path (Step 5) is deleted; `NodeFrontierSeeds` no longer exists in the codebase. CI floor stays green. Proves de-fork completed, not merely layered.

**(c) Real gunbc floor wall-clock + peak-RSS drop on scoped diff** — `gunbc test` on a 1-file diff disjoint from the mock closure AND the affected witness frontier: floor runs 0 witnesses AND skips the precompute. Logged wall-clock and peak-RSS drop materially vs a full-corpus baseline. On the REAL floor with live glob-discovery, not a synthetic fixture.

Sign-off on this sketch = commitment to deliver all three receipts green before merge.
