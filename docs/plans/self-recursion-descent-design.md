# Self-recursive call without a descending argument: design

RFM: `a_self_recursive_call_without_a_descending_argument_is_accepted` (from stern-bat-401). Read-only design; no runs were made. Every claim below is by cited grep/source inspection at origin/main 9b4c4d214f94 and names what was not inspected.

## 1. Where the evidence is computed, who reads it, why nothing refuses

- Lattice: `std.termination` `DescentEvidence = Strict | NonIncreasing | DescentUnknown` (BoundedLattice, top Strict, bottom DescentUnknown = fail-closed, DESIGN §4). Relations: `std.induction` `SubValueRelation` (`StrictSubValue`, `IteratedSubValue`, `ArithmeticDescent`, `PreservedValue`, `NonIncreasingValue`, `StrictAxisErased`, `MixedTop`).
- Producer (v1): `v1.04_infer` `annotate_descent` (with `DescentContext`) writes `ExprCall.descent_evidence: List<SubValueRelation>?` on every call node, after `infer_item` sets `is_self_recursive` / `has_non_tail_self_call`.
- Readers found: `v1.complexity` (`collect_self_call_evidence`, `collect_call_evidence`, `derive_edge_evidence`, `merge_param_evidence`, `has_arithmetic_descent`, `extract_shrink_factor`), which folds it into the complexity/census result; `v1.compile` `serialize_descent_evidence` (JSON export); `05_emit` and `02_parse` only pattern-match `descent_evidence: _` (pass-through). `is_tail_recursive` in `v1.compile` uses only the two shape flags.
- Why nothing refuses: every reader is a cost/census consumer or a pass-through. No acceptance-path reader folds a self-call's evidence into a diagnostic, so `DescentUnknown` (or `NonIncreasing`) on a self-call is computed, serialized, and discarded for acceptance. The lattice's fail-closed bottom has no consumer that fails closed. This is §5's "typed outcome nobody reads". Not inspected: other binaries reading the serialized JSON.

## 2. Advisory-first plan (as Q1b)

No new vocabulary: consume `DescentEvidence` and `SubValueRelation` as they are.
1. Step 1, advisory. In v1 `infer_item`, after `annotate_descent`, meet the evidence of each self-call of an `is_self_recursive` item with `descent_evidence_lattice_meet`. If the meet over a self-call's arguments is not `Strict` (no argument strictly descends, lexicographic: some Strict dimension with the rest NonIncreasing), emit a diagnostic of its OWN class `SelfCallWithoutDescent { fn, call_span, evidence }` carrying `NonIncreasing` or `DescentUnknown` so the two are distinct. It is advisory (counted, typed, located), not an existing reason's, and does not fail the compile. The count is emitted every compile by the compiler itself, so no census instrument is needed. Hand Rust in `compile_clean.rs` mirrors the Q1b `DeclaredTypeInhabitanceUnmodeled` path.
2. Step 2, refusal by class: flip the class to refusing once the advisory's count over the compiled corpus is dispositioned (each site gets a Strict descent, a `where`-checked measure, or a declared `rung_drop` row). Refusal is the same diagnostic with severity changed; no second path.
3. Dissolution: the advisory's severity flag is deleted at step 2; RED and controls stay enrolled (§4b meta-4).

`NonIncreasing`-only (the argument is preserved or shrinks without strictness) is refused at step 2 as well: a self-call with no strict dimension does not terminate by this evidence. A genuinely intended unbounded loop (server loop) must be declared, not inferred.

## 3. RED and controls

- RED specimen shape: `gunbc.durable_append_log_store` `attempt_append_log_census_row` (#15471, fixed on jolly-swift-832's branch; not on main, so not re-read here): a self-call passing its parameter unchanged / recomputed. Minimal fixture: `fn f(n: Nat) -> Nat { if n == 0 { 0 } else { f(n: n) } }` expects the advisory (step 1) and refusal (step 2).
- Controls (accepted both before and after): `f(n: n - 1)` under the `n == 0` guard (Strict via `ArithmeticDescent`); structural descent on a match-bound sub-value (`StrictSubValue`); a non-recursive fn calling another fn with the same argument (not `is_self_recursive`, untouched); a tail self-call with a lexicographic pair where the first is Strict.
- Mutual recursion: OUT OF SCOPE for this step. `is_self_recursive` is per-item; mutual recursion needs an SCC over the call graph (first named trigger: an SCC-level `is_self_recursive` carrier). Declared stall, not silently covered.
- Evidence: run the RED before/after on the same fixture via the real acceptance path, as witness claims beside the Q1b ones.

## 4. Does v2 have the same hole?

Partly unknown, explicitly. In v2 `v2.std.cardinality` `loop_multiplicity_from_rows` is fail-closed: an unregistered measure is `DescentUnknown` -> `RequiresTerminationProof`, and `lexicographic_multiplicity` requires a Strict dimension (a NonIncreasing-only measure returns `RequiresTerminationProof`, i.e. v2 does NOT accept NonIncreasing alone). `v2.compiler.infer` `infer_descent_witness_for_node` turns that into `^infer_descent_not_derived`. So a v2 `Loop` without a strict descent is refused where its fact is consumed. The open question: DESIGN §4 says recursion is sugar over `Loop`, but I did not find a v2 ingest path that lowers a self-recursive `fn` to a `Loop` with a measure, nor an acceptance consumer of `inferred_facts_descent` beyond `v2.lens.parallelism`. If a self-recursive fn is NOT lowered to Loop, v2 has the same hole by absence of any descent obligation. Smallest check before claiming either way: ingest a self-recursive fixture through v2 and observe whether a `Loop` node (and a descent obligation) appears. Not run (read-only).
