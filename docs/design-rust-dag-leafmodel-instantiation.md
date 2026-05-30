# LeafModelClaim instantiation for M = rust.dag

**Authors:** Target Realization Mgr (keen-heron-687) + Modeling DFS Mgr (proud-pike-680).
**Status:** Step 3 co-author deliverable per PR #3959 §11.4 operator dispatch sequence. Modeling DFS approval msg_badac9f3 (2026-05-30); covers shapes, fact_id discipline, scope-by-phase.
**Scope:** The Subject / Expectation / fact_id space when the generic `LeafModelClaim<M, Subject, Expectation>` carrier (PR #3959 `docs/planning/v4-leaf-model-verification-2026-05-30.md` §5) is instantiated with M = `rust.dag`.
**Anchors:** v4-leaf-model-verification-2026-05-30.md §5 (canonical claim carriers), §7 (rust.dag worked examples R1/R2a/R2b/R3-external/R3-internal), §6 (0d7a413c9 claim-corpus C(M) rule).
**Out of scope:** Step 4 (Runtime/TestClaim) implementation of fixture runner; Step 5 (TR) SG-1 dispatch.

---

## §1. RustClaimSubject — CLOSED coproduct (modeling-escalation-only extension)

```dag
type RustClaimSubject =
  | RustPrimitiveTypeSubject       { primitive: RustPrimitive }
  | RustAlgebraInhabitanceSubject  { algebra: AlgebraCarrier, on: RustPrimitive }
  | RustAtomRealizationSubject     { atom: Node }              // Node-keyed, NOT spelling-keyed
  | RustCollectionRealizationSubject { collection: Node }       // post-SG-5
  | RustGrammarProductionSubject   { production: GrammarProduction }
  | RustLexRuleSubject             { rule: LexRule }
```

**Closure rationale.** Open-extension coproducts in claim space are the heuristic-enum pattern operator rejected (memory `feedback_heuristics_recoverable_to_substrate`). New subject arms must enter via modeling escalation (Modeling DFS Mgr authority), not worker initiative — keeps the verification surface modelable end-to-end.

**Node-keyed atom/collection identity (TR-authority constraint).** `RustAtomRealizationSubject.atom` and `RustCollectionRealizationSubject.collection` are typed as `Node` (canonical source-node identity), NOT `Symbol` (raw spelling). If R3-internal becomes spelling-keyed, the falsification probe can spuriously succeed by string-matching what is not actually a single-authority change. This constraint is enforced by Modeling DFS in the generic `LeafModelClaim` shape and reflected here as the Subject-arm field type.

## §2. RustClaimExpectation — CLOSED coproduct

```dag
type RustClaimExpectation =
  | RustcAcceptsExpectation              { invocation_form: TargetInvocation }
  | RustcRejectsExpectation              { invocation_form: TargetInvocation, expected_error_code: Rustc_ErrorCode_Phase1 }
  | RustRuntimeBehaviorExpectation       { invocation_form: TargetInvocation, expected_outcome: RuntimeOutcome }
  | RustEmitProjectionEqualityExpectation { atom_realization_row: Node, type_emit_must_change: Bool, value_emit_must_change: Bool }
  | RustGrammarRoundTripExpectation      { production: GrammarProduction }
```

### §2.1 Phase-1 closed rustc error-code subset

```dag
// 🟡 dissolve-on-arrival: full RustcErrorCode coproduct — Phase 2.
// Rationale: full rustc error-code catalog is itself a leaf-model verification
// problem (rustc's diagnostic surface is a leaf model). Phase 1 stays minimal.
type Rustc_ErrorCode_Phase1 =
  | E0107   // wrong number of generic arguments (SG-2 evidence)
  | E0308   // mismatched types (R1 falsification probe)
  | E0423   // expected value, found type (SG-1 evidence)
  | E0599   // method not found (R2a falsification probe)
```

Expanding past these four requires a modeling-escalation justification. The dissolve-on-arrival note becomes actionable when a separate work item authors `rustc.dag` (or equivalent) as a leaf model in its own right.

### §2.2 RuntimeOutcome — std/ home (not rust.dag-specific)

Per DFS msg_badac9f3 answer (3): the runtime-outcome taxonomy is generic; Python/Go/etc. have analogous outcomes (panic, wrap, checked-return, exception-raise, …). Carrier lives in `src/v4/std/runtime.dag` (existing file). `rust.dag` references the std carrier; the per-target invocation_form remains rust.dag-specific.

Sketch (proposed addition to `std/runtime.dag` — Modeling DFS to ratify exact naming separately):

```dag
type RuntimeOutcome =
  | Panic            { panic_classification: Symbol }
  | Wrap             // wrapping arithmetic
  | CheckedReturn    { wrapped_value: Node }
  | ExceptionRaise   { exception_node: Node }
  // …extension via modeling escalation only
```

## §3. fact_id discipline — C(rust.dag) corpus per 0d7a413c9

The 0d7a413c9 acceptance contract:

> for every `fact_id` declared by M, the claim corpus C(M) contains a `LeafModelClaim` referencing `fact_id`.

For M = rust.dag, the fact_id space at main HEAD comprises **all** Symbol-named declarations the model already carries. Inventory (approximate counts, current main):

| fact_id family | count | Source location | Phase 1? |
| -------------- | ----- | --------------- | -------- |
| Primitive types (rust_primitive_*) | 13 | derived from RustPrimitive coproduct | R1 covers `i32` only |
| Algebra inhabitance assertions | 4 | rust.dag inhabitance declarations | R2a/R2b cover `OrderedRing<Int32>` only |
| `rust_std_projection_*` | 19 | rust.dag:265-283 | none in Phase 1 |
| `rust_surface_spelling_*` | 19 | rust.dag:285-303 | none in Phase 1 |
| `rust_repr_*` / `rust_ieee754_*` / `rust_str_*` / `rust_char_*` | 7 | rust.dag:305-312 | none in Phase 1 |
| `rust_inhabitant_*` | 19 | rust.dag:314-332 | none in Phase 1 |
| `rust_inhabitant_field_*` | ~20 | rust.dag:334-353 | none in Phase 1 |
| `rust_coercion_field_*` | ~10 | rust.dag:354-364 | none in Phase 1 |
| Grammar productions (T-4.17 wave 1+2) | ~varies | rust.dag grammar block | none in Phase 1 |
| Lex rules | ~varies | rust.dag lex block | none in Phase 1 |
| TargetAtomRealization rows (Symbol/Bool/Char) | 3 | rust.dag — **post SG-1** | R3-external + R3-internal (Symbol only) |
| TargetCollectionRealization rows (Set/…) | varies | rust.dag — **post SG-5** | none in Phase 1 |

**Every fact_id above MUST be present in C(rust.dag) as a LeafModelClaim** — Phase 1 dispatches 5 fixtures across 4 claim IDs (R1, R2a, R2b, R3-external + R3-internal); the remaining ~85+ fact_ids enter C(rust.dag) as `LeafModelClaim` rows with `not_checked` verdict state until Phase 2 drains them. They are NOT invisible debt — they appear explicitly in `LeafModelVerificationReport<rust.dag>.totals.not_checked`.

The 94 catalog sentinels (per the canonical-home spec §3) all land here. None are dropped; none are "implicit." This is the operator's framing made operational: every model fact has a verification obligation in C(M).

## §4. R3-internal timing — Step 4 authors row, Step 5 lands fact

Per DFS msg_badac9f3 answer (4):

| Step | Work | R3-internal state |
| ---- | ---- | ----------------- |
| Step 4 (quick-tern-735) | Author R3-internal claim row in C(rust.dag) per §1/§2 shape; runner stub references `atom_realization_row: Node` slot but cannot exercise (no row to mutate yet) | `not_checked` OR explicit `GAP` verdict — runner correctly reports unfilled-slot, NOT a false PROVEN |
| Step 5 (TR Mgr → SG-1 worker) | SG-1 lands `TargetAtomRealization` row for Symbol/Bool/Char in `v4.std.target_model` per the canonical-home spec | R3-internal becomes exercisable; runner mutates row, re-emits, verifies both type-emit AND value-emit changed → `PROVEN` |

**Hard rule:** SG-1 is NOT considered closed until R3-internal flips from `not_checked` to `PROVEN` post-Step-5 with both `type_emit_must_change=true` and `value_emit_must_change=true` matching observed behavior. If only one emit changes, the single-authority gap that SG-1 was supposed to fix is still present — the carrier landed but the consumers diverge.

**Do NOT block Step 4 on SG-1 landing.** Step 4 can ship with R3-internal scaffolded-but-not-exercising; the GAP is honest progress, not failure.

## §5. Per-claim sketches (Phase 1 only — 5 fixtures, 4 claim IDs)

Schematic — full fixture content is Step 4 deliverable.

### R1 — Rust i32 primitive declaration
```text
fact_id:    rust_primitive_i32
subject:    RustPrimitiveTypeSubject { primitive: Rust_I32 }
expectation: RustcAcceptsExpectation { invocation_form: rustc("pub fn r1_test() -> i32 { 0i32 }") }
falsification:
  subject_variant: RustPrimitiveTypeSubject { primitive: Rust_I32 }    // same subject
  expected_failure_mode: RustcRejectsExpectation { ..., expected_error_code: E0308 }
  // emit: pub fn r1_test() -> i32 { "string" }
```

### R2a — i32 supports algebra ops
```text
fact_id:    rust_algebra_ops_int32
subject:    RustAlgebraInhabitanceSubject { algebra: OrderedRingCarrier, on: Rust_I32 }
expectation: RustcAcceptsExpectation { invocation_form: rustc("pub fn r2a_test(a: i32, b: i32) -> (i32, bool) { (a + b, a < b) }") }
falsification:
  subject_variant: claim non-existent op (e.g., i32::log2_exact)
  expected_failure_mode: RustcRejectsExpectation { ..., expected_error_code: E0599 }
```

### R2b — i32 overflow semantics declared
```text
fact_id:    rust_algebra_overflow_int32
subject:    RustAlgebraInhabitanceSubject { algebra: OrderedRingCarrier, on: Rust_I32 }
expectation: RustRuntimeBehaviorExpectation { invocation_form: rustc-then-run("debug: i32::MAX + 1"), expected_outcome: Panic { panic_classification: arithmetic_overflow } }
falsification:
  subject_variant: claim i32 is unbounded OrderedRing<Int> (no width)
  expected_failure_mode: RustRuntimeBehaviorExpectation predicting unbounded behavior diverges from actual i32::MAX result
```

### R3-external — Symbol projection to Rust shape accepted by rustc
```text
fact_id:    rust_atom_realization_symbol_external
subject:    RustAtomRealizationSubject { atom: <Symbol carrier Node> }
expectation: RustcAcceptsExpectation { invocation_form: rustc(emitted-from-TargetAtomRealization-row) }
falsification:
  subject_variant: deliberately mismatched projection (Symbol declared as String alias + value-constructor call)
  expected_failure_mode: RustcRejectsExpectation { ..., expected_error_code: E0423 }    // the SG-1 root-cause error
```

### R3-internal — TargetAtomRealization row mutation receipt
```text
fact_id:    rust_atom_realization_symbol_internal
subject:    RustAtomRealizationSubject { atom: <Symbol carrier Node> }
expectation: RustEmitProjectionEqualityExpectation {
              atom_realization_row: <Symbol carrier Node>,
              type_emit_must_change: true,
              value_emit_must_change: true
            }
falsification:
  subject_variant: same row, but flip BOTH type_emit_must_change AND value_emit_must_change to false
  expected_failure_mode: structural-equality oracle observes one-or-both emit outputs DID change after row mutation
                         (i.e., the model's claim "row mutation has no effect" is rejected by observed reality)
```

## §6. Out-of-scope (deferred to other Steps / Phase 2)

- Fixture content authoring (Step 4 owns).
- `rustc` invocation harness implementation (Step 4 owns; quick-tern-735).
- SG-1 substrate row authoring (Step 5; zesty-carp-242 worker brief already drafted; held).
- Map<K,V> realization claims (post-SG-5; Witness-fn lookup semantics need separate ratification per vivid-heron-767 scope decision).
- Full RustcErrorCode coproduct (Phase 2; tied to potential `rustc.dag` leaf-model authoring).
- Grammar/lex per-rule claims beyond R1-R3-internal (Phase 2).
- Other language files (python.dag, go.dag, …) — same instantiation pattern, separate co-author cycles.

## §7. Coordination

| Sibling | Touchpoint | Status |
| ------- | ---------- | ------ |
| Modeling DFS (proud-pike-680) | Generic LeafModelClaim shape; this M=rust.dag instantiation; Node-keyed atom_realization_row enforcement | **Approved msg_badac9f3 2026-05-30** |
| Runtime/TestClaim (quick-tern-735) | Step 4 consumer — fixture runner reads C(rust.dag) authored per this spec | **Notify on PR open** — they consume this shape to scaffold the runner + R3-internal stub |
| Compiler Spine (smart-stag-871) | 06_translate.dag is the emit path R3-internal probes via row mutation; no consumer-side authoring needed here | **FYI on PR open** |
| SG-1 worker (zesty-carp-242, held) | R3-internal becomes exercisable post-Step-5 when worker lands the TargetAtomRealization row | **Already held; no new action; will re-dispatch per Step 5 timing** |
| Self-host/Release (nimble-crane-490) | R3-internal PROVEN is a prerequisite for declaring SG-1 closed | **No pre-PR coordination needed** |

---

**End of spec.** Step 3 deliverable. Awaiting any sibling-manager review before opening a PR; otherwise will draft PR with this file + a comment cross-referencing PR #3959 and the §11.4 operator dispatch sequence.
