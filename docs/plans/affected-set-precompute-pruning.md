# Affected-set de-fork: `v2.lens.affected_set` as single authority; dissolve Rust parallel implementation

**Status: PARTIAL IMPLEMENTATION — prep (#6065) + witness (a) partial (#6072) landed; Step 4 consumer-1 landed (#6061); Steps 4–5 continuing.**

**ROADMAP:** `1-affected-set-defork` (`dag/gunbc/roadmap_authority.dag:173`).

**Merge sequence:** PR #6065 (disposition-kernel prep) merges first; #6072 rebases onto fresh `main` so its diff collapses to witness-(a)-delta only. Do **not** merge #6072 before #6065.

---

## Implementation receipts (2026-07-01)

| Step | Status | Receipt |
|------|--------|---------|
| Prep — disposition kernel all axes | **GREEN** (PR #6065/#6072) | `floor_witness_run_disposition` with all three axes; `affected_set_disposition_both_axes_test.dag`; `floor_disposition_kernel_alignment` in `cli_run.rs` (disposition tautology on shared Rust inputs — NOT witness (a)); union-resolve S1 adds shared index |
| Step 3 witness (a) — **edited_test_fns axis** | **GREEN (partial)** | `floor_witness_a_prove` in `cli_run.rs`: fixture unified diff → Rust `edited_test_fns` vs independent `.dag` `floor_test_fn_declaration_edited`; mandatory RED under-selection; `.dag` claim `affected_set_witness_a_prove_test.dag` |
| Step 3 witness (a) — **node-frontier axis** | **BLOCKED** | Whole-tree `InferredTree` + `NodeArtifactProvenance` over live corpus — same resolve-grounding gate as `wiring_liveness_whole_tree` / `whole_tree_resolved_ctx` (`v2.lens.resolved_imports` open thread). `.dag` closure smoke on `provenance_producer` fixture only; Rust `NodeFrontierSeeds` equivalence deferred |
| Step 4 migrate floor — consumer 1 | **GREEN** (PR #6061) | Floor witness selection wired to `floor_witness_run_disposition` with all three axes (`touches_frontier`, `function_edited`, `entry_file_touched`); disposition kernel covers all three; union-resolve S1 (PR #6234) co-resolved floor runner through shared index |
| Step 4 migrate floor — consumer 2 | **IN PROGRESS** | Precompute-skip: gate `precompute_whole_tree_published_mock_keys` on FULL "no witness will run" predicate (empty frontier AND no directly-edited test functions); remaining: skip-before-resolve |
| Step 5 delete Rust parallel | **NOT STARTED** | `NodeFrontierSeeds`, `entry_touches_frontier_seeds`, etc. intact; gates on Step 4 completion |

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

**Full skip decision (cli_run.rs:3721–3731):** a witness is SKIPPED only when ALL three conditions hold:
1. `!touches_frontier` — node frontier: affected set does not touch this witness
2. `!function_edited` — function not directly edited: the witness's function declaration is unchanged
3. `!entry_file_touched` — entry file closure unchanged: no non-data-fn edits in the entry's import closure

A witness RUNS when ANY condition fires. The `.dag` authority's `floor_witness_run_disposition` now models all three axes (landed PR #6061/#6065/#6072). Migration wired consumer-1 (witness selection) with all three axes live; consumer-2 (precompute-skip) gates remain in Step 4.

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

**Consumer 1 — floor witness selection.** `run_discovery_rows` (cli_run.rs:3721–3731): the current Rust skip gate has THREE axes:
- Axis (i) node-frontier: `entry_touches_frontier_seeds` → maps to `affected_set_closure` + `floor_witness_run_disposition`'s `touches_frontier` parameter
- Axis (ii) function-edited: `frontier_seeds.edited_test_fns` check → live in `floor_witness_run_disposition` at `src/v2/workflow/affected_set_floor_runner.dag:114-141` (landed PR #6065/#6072)
- Axis (iii) entry-file-touched: `entry_touches_frontier_seeds` closure check for non-data-fn edits in import closure → live as `entry_file_touched` parameter

A witness runs when `touches_frontier || function_edited || entry_file_touched`; the `.dag` skip disposition correctly models all three axes. Migration can proceed to consumer-2 (precompute-skip) gates once all axes verification passes.

**Consumer 2 — precompute-skip.** Gate `precompute_whole_tree_published_mock_keys` (cli_run.rs:3509) on the FULL "no witness will run" predicate, not just the node frontier. A scoped diff can still run witnesses via the `function_edited` path even when the `.dag` frontier is empty. Correct guard: `RerunNodeSetProduced { nodes: [] }` (empty node frontier) **AND** `edited_test_fns.is_empty()` (no directly-edited test functions). Both conditions together are required.

**Conservative gate acknowledged:** gating on empty-frontier + empty-edited-test-fns is stricter than the mock-declarer-closure disjointness gate (the closure check at cli_run.rs:890–937 would allow skipping precompute even when witnesses run, as long as none touch the mock closure). This plan chooses the simpler, conservative empty-frontier+empty-edited gate. A future tightening to the mock-closure gate is a follow-on optimization, not in scope here.

**Consumer 3 — wiring_liveness preflight.** `wiring_liveness_preflight.dag` already imports `ReExecFrontier` from `affected_set.dag` as a consumer. Verify it remains correctly positioned (consumer, not reimplementor) after migration. No migration needed.

**Seam shape:** the `.dag` query result (`RerunNodeSet`) crosses the realization boundary once at floor entry — the same point `collect_frontier_seeds_from_diff_line_ranges` is called today. One query evaluation, two consumers, zero parallel impls.

**Host-scaffold witness classifier (ROADMAP `2-host-scaffold-classifier-defork`, #6224):** live-tree witnesses must never take node-frontier skip (`floor_host_scaffold_would_skip` → `false`). Interim classification is the `floor:host_scaffold` marker plus the Rust `witness_test_fn_uses_live_host_scan` scaffold in `cli_run.rs` (conservative, fail-closed). **Dissolve-on:** `reads_live_tree` disposition on `TestClaim` rows — delete the text classifier when the substrate carries the fact.

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

---

## FENCED FOLLOW-ON — the "REAL fix" (behind resolver-B #6155)

**Status: FENCED — do not start until resolver-B (#6155) lands a green receipt.** Captured 2026-07-02 by the CI-repair dispatch lane so this is a discoverable dispatchable item, not tribal memory. The prior runtime-provenance PR (#6105, `Provenance ingest at CI floor runtime`) was **CLOSED** as not-landable; its branch **`session/quick-carp-343`** is preserved as harvest material (the runtime overlay, `affected_set_floor_runner.dag`, and the `pa_ingest` fixtures).

### Why #6105 was closed — measured, not asserted

A floor-repair scout instrumented `#6105`'s HEAD (`af155b04`) with resolve/typecheck counters and ran the `pa_ingest` single-witness fixture. **Affected-set pruning as built is work-neutral theater — it INCREASES resolve+typecheck and only saves the witness EVAL:**

| Arm | entry_resolve | typecheck_module | witness_evals | total_resolve_ms |
|-----|--------------:|-----------------:|--------------:|-----------------:|
| **PRESENT** (live overlay, witness skipped) | **6** | **28** | 0 | **3885** |
| **RUN-ALL** (skip disabled) | **1** | **27** | 1 | **2042** |

Root cause: the node-frontier skip in `run_discovery_rows` fires **after** the witness-entry closure is already resolved (resolve at entry change → `should_skip` → `continue` skips only the EVAL), and `resolve_floor_runner_context` + `floor_runner_eval_context` each resolve the full `affected_set_floor_runner.dag` closure (2× duplicate). Wall/skip-count look like a win; total work multiplied — the #6127 width-1 shard trap in miniature.

### Acceptance witness (c) is strengthened by the operator standing rule (2026-07-02)

Wall-clock + peak-RSS (current (c)) are NOT sufficient — they can improve while total work explodes (exactly how #6127's 30.9× CPU regression hid under a fine-looking wall). **Any scheduling/pruning change must carry a before/after RESOLVE-COUNT + TYPECHECK-COUNT receipt** showing the PRESENT arm **strictly fewer** than run-all, and the fail-closed arm **equal** to run-all. The current overlay fails this. Add this count receipt to (c).

### The two-part REAL fix

1. **Move the skip BEFORE witness-entry resolve.** The frontier/provenance skip decision must be made from the changed-paths closure alone, so a skipped witness's own import closure is never resolved. A skip that still resolves saves nothing on the axis that matters.
2. **Share the duplicate `floor_runner` resolves.** `resolve_floor_runner_context` and `floor_runner_eval_context` resolve the same `affected_set_floor_runner.dag` closure twice per run; memoize/share it (the M1 `walk_memo` in `claim_executor` is the precedent — a `heavy_whole_tree_resolve`-keyed in-process memo).

**Fence rationale:** both parts amplify or are amplified by the resolve-duplication that resolver-B (#6155, compositional `build_type_env` — kill the whole-ancestry-per-module quadratic) is removing. Landing pruning on top of the current quadratic resolve would multiply the very work #6155 is deleting. Start this only after #6155 lands green; harvest `session/quick-carp-343`; deliver acceptance (a)(b)(c)+count-receipt.
