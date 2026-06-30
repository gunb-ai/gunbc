# Affected-set precompute pruning: `precompute_whole_tree_published_mock_keys` skip on scoped diff

**Status: DESIGN SKETCH — no implementation. Returns to stern-moth-225 → loyal-bee/operator for sign-off before any code lands.**

---

## 1. Problem

Execution-skip (#5971 Move A, 59/59 witnesses skipped on a disjoint diff) proves the EXECUTION loop prunes correctly. But it is cosmetic: `precompute_whole_tree_published_mock_keys` + `build_multi_entry_index` at `cli_run.rs:3509/3518` run **unconditionally**, before the skip logic at `cli_run.rs:3520`. Paying the full precompute on every scoped diff eliminates the wall-clock and peak-memory win even when all witnesses are skipped.

---

## 2. What the two halves own

| Half | Owner | What it covers |
|------|-------|----------------|
| **Execution skip** | this lane (#5971, done) | guards `entry_touches_frontier_seeds` (cli_run.rs:3713); 59/59 witnesses skipped on disjoint diff | 
| **Precompute skip** | this lane (this sketch) | guards `precompute_whole_tree_published_mock_keys` call at cli_run.rs:3509 |
| **Precompute cache** | tidy-hawk-120 (#5959 M1) | within-walk resolve memo; avoids re-paying the precompute cost across corpus shards / compile subprocess invocations **when the precompute does run** |

They compose cleanly: a scoped diff that doesn't touch the mock closure → precompute skipped (this sketch) AND affected witnesses skipped (#5971) → both wall-clock and peak-memory drop. tidy-hawk-120's cache applies only when the precompute IS needed; this sketch decides whether it is needed at all.

---

## 3. Consumer fn + plug point

**Consumer:** `run_discovery_corpus_with_options` (cli_run.rs:3468).

**Plug point:** the call to `precompute_whole_tree_published_mock_keys(source_roots)` at line 3509.

**Guard shape:**

```
// Current (unconditional):
let whole_tree_published_keys = match precompute_whole_tree_published_mock_keys(source_roots) { … };

// Sketch (guarded):
let whole_tree_published_keys =
    if skip_enabled && !diff_touches_published_mock_closure(source_roots, &line_ranges_by_file) {
        None   // safe: no affected witness in the pruned corpus uses mock keys
    } else {
        match precompute_whole_tree_published_mock_keys(source_roots) { … }
    };
```

This requires moving the diff-parsing block (currently lines 3520–3552) **before** line 3509 so `line_ranges_by_file` is available at the plug point.

---

## 4. New function: `diff_touches_published_mock_closure`

**Signature:** `fn diff_touches_published_mock_closure(source_roots: &[String], diff_ranges: &HashMap<String, Vec<LineRange>>) -> bool`

**Cost:** module-level only — text scan + dependency-graph traversal. No type-check, no eval (cheaper than the `resolve_transitively` step inside the full precompute).

**Algorithm:**
1. Build module index (already done for `build_multi_entry_index`; share the call).
2. Find PublishedMockCase declarers via text scan (same `.contains("PublishedMockCase")` filter as precompute line 910).
3. Walk the transitive import closure of those declarers at module-graph level (parse-only, no resolve).
4. Return `true` if any file in `diff_ranges` is in that closure; `false` otherwise.

**Invariant:** over-inclusive is safe (false-positive → precompute runs → correct). False-negative is unsound (miss a declarer that changed → mock keys stale). The `.contains` prefilter is already proven over-inclusive (precompute comment, line 907).

---

## 5. Input carrier

`NodeArtifactProvenance` (the diff base → affected-closure carrier). The `line_ranges_by_file` it yields (via `collect_frontier_seeds_from_diff_line_ranges`) is the same set used by execution-skip and is reused here without re-derivation.

---

## 6. Discriminating witness (new — what #5971 does NOT prove)

`#5971` proves execution-count skip (59/59 skipped on a disjoint diff). It does **not** prove the precompute shrinks.

**Required new witness:** `witness_precompute_skipped_on_scoped_diff` — a Rust control (SCAFFOLD, same dissolution trigger as Move A) that:

1. Constructs a synthetic diff that modifies one file **outside** the PublishedMockCase transitive closure (e.g. a change to a test-only `.dag` file).
2. Calls `run_discovery_corpus_with_options` with `skip_unaffected_node_frontier: true`.
3. Asserts:
   - Wall-clock for the run is **materially shorter** than a full precompute run (measured delta, not just a skip flag).
   - Peak RSS (measured via `/proc/self/status` VmHWM before/after the call) does **not** include the ~1.46 GiB transient `ResolvedGraph` the precompute produces.
4. A second assertion (fail-closed control): a synthetic diff that modifies a file **inside** the PublishedMockCase closure → precompute IS invoked → wall-clock and peak ARE paid.

**Success bar (operator-specified):** "if wall-clock does not move on a scoped diff, it is not done."

---

## 7. Fail-safe

Over-declaration is always safe: if `diff_touches_published_mock_closure` returns `true` (or errors), `precompute_whole_tree_published_mock_keys` runs in full. Skipping is only sound when `false` — the predicate is monotonically safe to be conservative. If the intersection check itself fails, fall back to running the precompute (fail-closed, same pattern as `floor_git_diff_range` failure at line 3524).

Passing `None` for `whole_tree_published_keys` when skipped is sound **only** when no affected witness in the pruned run actually uses the mock key set at eval time. This is the second discriminating assertion in §6 item 3 (the fail-closed control). If unsound, the fallback is trivially available: run the precompute.

---

## 8. Scope boundary with tidy-hawk-120

tidy-hawk-120 (#5959) owns M1 (within-walk resolve memo) and M2 (`RunnableCompile` content-addressed plan node). Neither addresses the **skip-the-precompute-entirely** case — they optimize the precompute when it runs. This sketch owns the **decision not to run it**. The two lanes can land independently and compose additively.

Coordinate with tidy-hawk-120 before implementation: confirm their M1 memo does not assume `precompute_whole_tree_published_mock_keys` always runs (i.e., the `None` early-exit path in §3 must not break M1's cache contract).
