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
| **Resolve memo** | tidy-hawk-120 (#5959 M1) | within-walk `resolve_entry_graph` memo across `claim_executor` batches (~105s Axis B); cuts resolve cost when the corpus runs — orthogonal to precompute skip (does NOT cache `precompute_whole_tree_published_mock_keys`) |

They compose additively and independently: a scoped diff that doesn't touch the mock closure → precompute skipped (this sketch) AND affected witnesses skipped (#5971) → both wall-clock and peak-memory drop. tidy-hawk-120 M1 reduces resolve cost when the corpus runs; this sketch decides whether the precompute runs at all. The two lanes do not share a cache contract.

---

## 3. Consumer fn + plug point

**Consumer:** `run_discovery_corpus_with_options` (cli_run.rs:3468).

**Plug point:** the call to `precompute_whole_tree_published_mock_keys(source_roots)` at line 3509.

**Guard shape:**

```
// Current (unconditional):
let whole_tree_published_keys = match precompute_whole_tree_published_mock_keys(source_roots) { … };

// Sketch (guarded) — uses PrecomputeOutcome tri-state (see §6):
// SOUNDNESS CONSTRAINT (see §7): skip is only safe when zero witnesses will execute
// (all_witnesses_will_skip) AND the diff does not touch the mock closure.
// The mock-closure check alone is insufficient — see §7 for why.
let outcome: PrecomputeOutcome =
    if skip_enabled
        && !diff_touches_published_mock_closure(source_roots, &line_ranges_by_file)
        && all_witnesses_will_skip(&rows, &frontier_seeds)   // REQUIRED: see §7
    {
        PrecomputeOutcome::Skipped
    } else {
        match precompute_whole_tree_published_mock_keys(source_roots) {
            Ok(keys) if keys.is_empty() => PrecomputeOutcome::EmptyKeys,
            Ok(keys) => PrecomputeOutcome::Keys(keys),
            Err(e) => return Err(…),
        }
    };
```

This requires moving the diff-parsing block (currently lines 3520–3552) **before** line 3509 so `line_ranges_by_file` and `frontier_seeds` are available at the plug point.

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

## 4b. Existing predicate: `all_witnesses_will_skip`

This is NOT a new function — it is a bulk pre-flight check using the existing per-row skip predicate already applied inside `run_discovery_rows` (cli_run.rs:3721–3731).

**Signature:** `fn all_witnesses_will_skip(frontier_seeds: &NodeFrontierSeeds) -> bool`

**`NodeFrontierSeeds` fields** (cli_run.rs:3275–3278): `overlapping_data_items: HashSet<(String, String)>`, `edited_test_fns: HashSet<(String, String)>`, `force_run_all: bool`. There is no `frontier_files` field.

**Algorithm:** The per-row inner loop (cli_run.rs:3721–3731) skips a witness when `!current_entry_touches && !function_edited`. `current_entry_touches` is computed by `entry_touches_frontier_seeds` — which requires a full resolved `InterpContext` and cannot be called at the precompute plug point. The safe pre-flight check avoids resolve by checking the frontier seed cardinality directly:

```rust
fn all_witnesses_will_skip(frontier_seeds: &NodeFrontierSeeds) -> bool {
    // Conservative: if any frontier seeds exist, we cannot guarantee all witnesses
    // skip without resolving each entry. The inner loop's entry_touches_frontier_seeds
    // call (cli_run.rs:3714) builds a per-entry frontier from overlapping_data_items;
    // if that set is empty, entry_frontier_nodes_from_seeds returns empty for every
    // entry and entry_touches_frontier_seeds returns false — all witnesses skip.
    !frontier_seeds.force_run_all
        && frontier_seeds.overlapping_data_items.is_empty()
        && frontier_seeds.edited_test_fns.is_empty()
}
```

This is O(1). It is conservative (over-inclusive towards running precompute): a diff that populates `overlapping_data_items` but whose changed items are NOT imported by any witness would still return `false` here, causing the precompute to run unnecessarily. That is safe — the precompute is always correct; skipping it is the optimization.

**Tie to cli_run.rs:3721–3731:** the inner loop's `!current_entry_touches` reduces to `false` (do not skip) whenever `overlapping_data_items` is non-empty, because `entry_frontier_nodes_from_seeds` builds a frontier from those items for each entry. The `!function_edited` check reduces to `false` whenever `edited_test_fns` is non-empty and matches the row. This pre-flight returns `false` conservatively when either set is non-empty. If the inner loop's skip predicate gains a new field in the future, `all_witnesses_will_skip` must check it or fall back to returning `false` (fail-closed).

**Invariant:** over-inclusive is safe (returns `false` when unsure → precompute runs → correct). False-positive (returns `true` when a witness would have run) is unsound — it would skip the precompute while a witness observes missing keys via the per-entry fallback (v1_interpreter.rs:1114–1121).

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

1. Constructs a synthetic diff that modifies one file that is outside **both** the PublishedMockCase transitive closure **and** every witness's node frontier (i.e. not transitively imported by any test witness in the corpus). A test-only `.dag` edit outside the mock closure is insufficient if it is imported by a witness — such a diff would fire the node frontier and leave witnesses to run, preventing `Skipped` from being asserted (the conjunction guard requires `all_witnesses_will_skip`).
2. Calls `run_discovery_corpus_with_options` with `skip_unaffected_node_frontier: true`.
3. Asserts `outcome == PrecomputeOutcome::Skipped`. Unambiguous: `Skipped` cannot arise from a ran-empty path. Cannot flake.
4. Fail-closed control (second assertion): a synthetic diff that modifies a file **inside** the PublishedMockCase closure → assert `outcome != PrecomputeOutcome::Skipped` (i.e. `EmptyKeys | Keys(_)` — precompute ran). `Keys(_)` specifically is not required here: declarers could legitimately produce zero keys (`EmptyKeys`) while still proving the precompute was not skipped.

**Measured evidence (logged, not the pass/fail gate):**

5. Log wall-clock delta and peak-RSS delta (VmHWM before/after) for both branches. These are reported as `[measurement]` lines satisfying the operator success bar ("wall-clock must move on a scoped diff") without being threshold assertions that flake on noisy runners.

The structural `PrecomputeOutcome::Skipped` / `!= Skipped` check is the executable green-by-execution proof. The measured delta is the evidence that the skip is economically meaningful. Both are required; only the structural assertion is the pass/fail gate.

---

## 7. Fail-safe and soundness constraint

Over-declaration is always safe: if `diff_touches_published_mock_closure` returns `true` (or errors), `precompute_whole_tree_published_mock_keys` runs in full. If the intersection check itself fails, fall back to running the precompute (fail-closed, same pattern as `floor_git_diff_range` failure at line 3524).

**Why the mock-closure check alone is insufficient:** a diff can miss the PublishedMockCase closure yet still leave witnesses to run (node-frontier / test-fn edits outside the closure). If any of those witnesses call into the mock key registry at eval time (v1_interpreter.rs:1114–1121), they fall back to per-entry `resolve_published_mock_keys` rather than the whole-tree seed, potentially under-populating `governed_services` for corpus-outside-closure mocks (M4.1). This is a fail-open path relative to the unconditional precompute.

**Required tighter guard (see §3 sketch):** skip precompute only when `!diff_touches_published_mock_closure(...)` **AND** `all_witnesses_will_skip(...)` — i.e. the frontier check confirms zero witnesses will execute. When witnesses do run, run the precompute in full (fail-closed). The conjunction makes skipping safe: `Skipped` means both "keys unused" and "no running witness can observe the omission." (Note: M1 from tidy-hawk-120 #5959 is a resolve memo, not a precompute cache — it cannot substitute for the precompute result; see §10.)

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

- **(a) Structural:** `gunbc test` on a scoped 1-file diff that is disjoint from **both** the PublishedMockCase closure **and** every witness's node frontier (i.e. produces empty `NodeFrontierSeeds`: `overlapping_data_items` empty, `edited_test_fns` empty, `force_run_all` false) logs `outcome == PrecomputeOutcome::Skipped` (precompute not called). A diff outside the closure but inside a witness frontier would correctly run precompute (`outcome != Skipped`) and fail this criterion — the diff must satisfy both conjuncts of the §3 guard. This is the `PrecomputeOutcome` tri-state from §6, not `whole_tree_published_keys == None` (which is ambiguous — see §6 implementation prerequisite).
- **(b) Measured:** wall-clock AND peak-RSS on that real floor run drop materially vs a full-corpus baseline run. Logged as `[measurement]` output, not a threshold assertion.
- **(c) Control:** a hub / in-closure 1-file diff (a file inside the PublishedMockCase transitive import closure) logs `outcome != PrecomputeOutcome::Skipped` (i.e. `EmptyKeys | Keys(_)` — precompute ran) and runs the affected witness slice — proves the skip is conditional, not always-on. `Keys(_)` is not required here: the structural guarantee is that the precompute was not skipped, not that it yielded non-empty results (consistent with §6 item 4).

Synthetic corpus numbers (e.g., a test-file-only mini-corpus) do NOT satisfy this bar. The acceptance receipt must be a real `gunbc test` invocation against the live floor showing parts (a)–(c).

Sign-off on this sketch = commitment to deliver that green-by-execution receipt on the real floor.

---

## 10. Scope boundary with tidy-hawk-120

tidy-hawk-120 (#5959) owns M1 (within-walk `resolve_entry_graph` memo across `claim_executor` batches, ~105s Axis B) and M2 (`RunnableCompile` content-addressed plan node). M1 memos resolve, not `precompute_whole_tree_published_mock_keys` — it does **not** cache or serve the precompute result. This sketch owns the **decision not to run the precompute at all**. The two lanes are orthogonal and can land independently.

Coordinate with tidy-hawk-120 before implementation on one structural point: confirm M1's contract does not assume `precompute_whole_tree_published_mock_keys` always runs (the `PrecomputeOutcome::Skipped` early-exit in §3 must not violate M1's assumptions about resolve inputs, if any).
