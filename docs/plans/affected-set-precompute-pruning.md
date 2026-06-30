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

`line_ranges_by_file: HashMap<String, Vec<LineRange>>` — produced by `floor_git_diff_range() → parse_unified_diff_line_ranges` at cli_run.rs:3533–3536 (the diff-parse block moved before the precompute call in §3). This is the **same map** already consumed by `collect_frontier_seeds_from_diff_line_ranges` at cli_run.rs:3541 for execution-skip. `diff_touches_published_mock_closure` is a second consumer of the same map — no re-derivation. `collect_frontier_seeds_from_diff_line_ranges` produces `NodeFrontierSeeds` for the execution-skip guard; it does not yield `line_ranges_by_file`. (`NodeArtifactProvenance` is a `.dag` type; it is not present in this v1 Rust path.)

---

## 6. Discriminating witness (new — what #5971 does NOT prove)

`#5971` proves execution-count skip (59/59 skipped on a disjoint diff). It does **not** prove the precompute shrinks.

**Required new witness:** `witness_precompute_skipped_on_scoped_diff` — a Rust control (SCAFFOLD, same dissolution trigger as Move A) that:

**Implementation prerequisite — tri-state carrier:**

Today `whole_tree_published_keys: Option<HashSet<String>>` has two `None` meanings: (a) precompute ran and returned empty keys (`keys.is_empty()` at cli_run.rs:3510), and (b) the proposed skip. These are indistinguishable and make a structural `None`-assertion flaky on a corpus with no declarers, or ambiguous by construction. The implementation **must** introduce a tri-state:

```rust
enum PrecomputeOutcome {
    Skipped,                        // diff does not touch mock closure → precompute not called
    EmptyKeys,                      // ran, declarers found, produced zero keys
    Keys(HashSet<String>),          // ran, produced non-empty keys
}
```

`run_discovery_corpus_with_options` and `run_discovery_rows` take `PrecomputeOutcome` instead of `Option<HashSet<String>>`. The existing `None`/`Some` split at line 3510 becomes `EmptyKeys`/`Keys(_)`; the new skip path is `Skipped`.

**Structural assertions (the wall — non-flaky, §5 correct):**

1. Constructs a synthetic diff that modifies one file **outside** the PublishedMockCase transitive closure (e.g. a change to a test-only `.dag` file).
2. Calls `run_discovery_corpus_with_options` with `skip_unaffected_node_frontier: true`.
3. Asserts `outcome == PrecomputeOutcome::Skipped`. Unambiguous: `Skipped` cannot arise from a ran-empty path. Cannot flake.
4. Fail-closed control (second assertion): a synthetic diff that modifies a file **inside** the PublishedMockCase closure → assert `outcome == PrecomputeOutcome::Keys(_)` (precompute ran and produced keys).

**Measured evidence (logged, not the pass/fail gate):**

5. Log wall-clock delta and peak-RSS delta (VmHWM before/after) for both branches. These are reported as `[measurement]` lines satisfying the operator success bar ("wall-clock must move on a scoped diff") without being threshold assertions that flake on noisy runners.

The structural None/Some is the executable green-by-execution proof. The measured delta is the evidence that the skip is economically meaningful. Both are required; only the structural assertion is the pass/fail gate.

---

## 7. Fail-safe

Over-declaration is always safe: if `diff_touches_published_mock_closure` returns `true` (or errors), `precompute_whole_tree_published_mock_keys` runs in full. Skipping is only sound when `false` — the predicate is monotonically safe to be conservative. If the intersection check itself fails, fall back to running the precompute (fail-closed, same pattern as `floor_git_diff_range` failure at line 3524).

Passing `None` for `whole_tree_published_keys` when skipped is sound **only** when no affected witness in the pruned run actually uses the mock key set at eval time. This is the second discriminating assertion in §6 item 3 (the fail-closed control). If unsound, the fallback is trivially available: run the precompute.

---

## 8. Soundness lemma: why the PublishedMockCase-declarer closure is the correct affected-set

Verified against `precompute_whole_tree_published_mock_keys` (cli_run.rs:890–937):

1. `build_module_index(&dsl_roots)` builds a **lookup table** over all modules. This is used by `resolve_transitively` to follow import edges — it is NOT the resolve graph.
2. `declarers` = only modules whose raw content contains `"PublishedMockCase"` (text scan, line 910).
3. `all_sources = resolve_transitively(declarers, &index, seen)` = the **transitive import closure** of those declarers. No other modules enter `all_sources`.
4. `resolved_graph_from_sources(all_sources, ...)` builds the interpreter graph over exactly `all_sources`. Non-members are absent from the graph.
5. `resolve_published_mock_keys(&ctx)` evaluates within this graph — it can only read declarations present in `all_sources`.

**Lemma:** a file F ∉ `all_sources` cannot influence `resolve_published_mock_keys` output. Proof: if F could affect the output, its declarations would need to be in the interpreter graph — but the graph is bounded to `all_sources`. F could enter `all_sources` only if it is (transitively) imported by a declarer — but then F **is** in `all_sources`. Contradiction. ∎

**No whole-tree scan in the hot path:** `build_module_index` touches all files but returns only a map of `(module_path → SourceFile)`. The resolve step (expensive) is bounded to `all_sources`. A non-declarer file changed on disk does not enter the resolved graph unless it is transitively imported by a declarer, in which case it is already in the closure.

**If the body were to widen:** should future work add a whole-tree scan inside the fn (e.g., a second `index.values()` pass that does not filter by declarer membership), the closure `diff_touches_published_mock_closure` must widen to match — this is the implementation-time check, not a sign-off assumption.

---

## 9. Acceptance witness (operator-required)

**End-user story:** after this lands, `gunbc test` on a small change re-precomputes and re-runs only the diff-affected witness slice instead of the whole ~537s / 8.7 GiB corpus. That is the deliverable this sign-off commits to.

**Demonstration must be on the REAL gunbc test floor** (live glob-discovery CI floor on the actual corpus — not a synthetic fixture). Three parts, all required:

- **(a) Structural:** `gunbc test` on a scoped 1-file diff that is disjoint from the PublishedMockCase closure shows `whole_tree_published_keys == None` (precompute skipped) in the run output.
- **(b) Measured:** wall-clock AND peak-RSS on that real floor run drop materially vs a full-corpus baseline run. Logged as `[measurement]` output, not a threshold assertion.
- **(c) Control:** a hub / in-closure 1-file diff (a file inside the PublishedMockCase transitive import closure) pays the precompute (`Some(_)`) and runs the affected witness slice — proves the skip is conditional, not always-on.

Synthetic corpus numbers (e.g., a test-file-only mini-corpus) do NOT satisfy this bar. The acceptance receipt must be a real `gunbc test` invocation against the live floor showing parts (a)–(c).

Sign-off on this sketch = commitment to deliver that green-by-execution receipt on the real floor.

---

## 10. Scope boundary with tidy-hawk-120

tidy-hawk-120 (#5959) owns M1 (within-walk resolve memo) and M2 (`RunnableCompile` content-addressed plan node). Neither addresses the **skip-the-precompute-entirely** case — they optimize the precompute when it runs. This sketch owns the **decision not to run it**. The two lanes can land independently and compose additively.

Coordinate with tidy-hawk-120 before implementation: confirm their M1 memo does not assume `precompute_whole_tree_published_mock_keys` always runs (i.e., the `None` early-exit path in §3 must not break M1's cache contract).
