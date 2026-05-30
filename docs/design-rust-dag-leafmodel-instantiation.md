# LeafModelClaim instantiation for M = rust.dag

**Authors:** Target Realization Mgr (keen-heron-687) + Modeling DFS Mgr (proud-pike-680).
**Status:** Step 3 co-author deliverable per PR #3959 §11.4 operator dispatch sequence. Modeling DFS approval msg_badac9f3 (2026-05-30); covers shapes, fact_id discipline, scope-by-phase.
**Scope:** The Subject / Expectation / fact_id space when the generic `LeafModelClaim<M, Subject, Expectation>` carrier (PR #3959 `docs/planning/v4-leaf-model-verification-2026-05-30.md` §5) is instantiated with M = `rust.dag`.
**Anchors:** `docs/planning/v4-leaf-model-verification-2026-05-30.md` §5 Layer A (claim corpus C(M) rule — model owns facts; sibling claim files at `test/claim/language_model/<model>.dag` own verification obligations), §5 carrier-shape block (canonical `LeafModelClaim<M, Subject, Expectation>` + `FalsificationCase` + `LeafModelFixture` + `LeafModelVerificationReport`), §7 (rust.dag worked examples R1/R2a/R2b/R3-external/R3-internal) including §7 Phase-1 scope ("ONLY R1 + R2a + R2b + R3-external + R3-internal").
**Out of scope:** Step 4 (Runtime/TestClaim) implementation of fixture runner; Step 5 (TR) SG-1 dispatch.

---

## §1. RustClaimSubject — CLOSED coproduct (modeling-escalation-only extension)

```dag
type RustClaimSubject =
  | RustPrimitiveTypeSubject       { primitive: RustPrimitive }              // 🟢 firm
  | RustAlgebraInhabitanceSubject  { algebra: AlgebraCarrier, on: RustPrimitive }  // 🟢 firm
  | RustAtomRealizationSubject     { atom: Node }                             // 🟡 gated — see §1.1
  | RustCollectionRealizationSubject { collection: Node }                     // 🟡 gated — see §1.1
  | RustGrammarProductionSubject   { production: GrammarProduction }          // 🟡 gated — see §1.1
  | RustLexRuleSubject             { rule: LexRule }                          // 🟡 gated — see §1.1
```

**Closure rationale.** Open-extension coproducts in claim space are the heuristic-enum pattern operator rejected (memory `feedback_heuristics_recoverable_to_substrate`). New subject arms must enter via modeling escalation (Modeling DFS Mgr authority), not worker initiative — keeps the verification surface modelable end-to-end.

### §1.1 Per-arm dispositions (modeling-discipline Practice 4)

| Arm | Disposition | Rationale + dissolve-on-arrival trigger |
| --- | ----------- | --------------------------------------- |
| `RustPrimitiveTypeSubject` | 🟢 firm | Rust primitive type space is fixed by the Rust Reference (i8..i128, u8..u128, isize, usize, bool, char, str, unit, never, f32, f64). The set is enumerable, model-owned (rust.dag `rust_facts_*` records), and not expected to dissolve. No trigger. |
| `RustAlgebraInhabitanceSubject` | 🟢 firm | Mirrors std/algebra carrier (OrderedRing, ApproximateField, BooleanAlgebra, ...) paired with a RustPrimitive. Both sides are firm; no trigger. |
| `RustAtomRealizationSubject` | 🟡 gated | Dissolve-on-arrival: SG-1 (TargetAtomRealization) + SG-5 (TargetCollectionRealization) land per PR #3938 §10.1 / §10.3. Once those rows exist for the full atom/collection set (Symbol/Bool/Char + Set/Map/List/...), this arm transitions 🟢 firm. Currently 🟡 because the underlying realization carrier is partly PLANNED. |
| `RustCollectionRealizationSubject` | 🟡 gated | Same as above — dissolves to 🟢 firm when SG-5 row set lands. |
| `RustGrammarProductionSubject` | 🟡 gated | Dissolve-on-arrival: T-4.17 wave 2 grammar productions complete on main. Currently grammar block is wave-1+wave-2-partial. Transitions 🟢 firm when grammar is closed. |
| `RustLexRuleSubject` | 🟡 gated | Dissolve-on-arrival: lex-rule block on main reaches Rust-Reference parity. Currently partial; transitions 🟢 firm when complete. |

**No 🔴 arms.** A 🔴 (provisional / candidate-for-removal) arm would be one where the modeling decision to include is itself uncertain. All six arms above are structurally justified by existing rust.dag fact families; none are speculative.

**Node-keyed atom/collection identity (TR-authority constraint).** `RustAtomRealizationSubject.atom` and `RustCollectionRealizationSubject.collection` are typed as `Node` (canonical source-node identity), NOT `Symbol` (raw spelling). If R3-internal becomes spelling-keyed, the falsification probe can spuriously succeed by string-matching what is not actually a single-authority change. This constraint is enforced by Modeling DFS in the generic `LeafModelClaim` shape and reflected here as the Subject-arm field type.

## §2. RustClaimExpectation — CLOSED coproduct

```dag
type RustClaimExpectation =
  | RustcAcceptsExpectation              { invocation_form: TargetInvocation }                                                       // 🟢 firm
  | RustcRejectsExpectation              { invocation_form: TargetInvocation, expected_error_code: Rustc_ErrorCode_Phase1 }          // 🟡 gated — Phase-1 subset (see §2.1)
  | RustRuntimeBehaviorExpectation       { invocation_form: TargetInvocation, expected_outcome: RuntimeOutcome }                     // 🟢 firm
  | RustEmitProjectionEqualityExpectation { atom_realization_row: Node, type_emit_must_change: Bool, value_emit_must_change: Bool }  // 🟡 gated — exercisable post-SG-1 only
  | RustGrammarRoundTripExpectation      { production: GrammarProduction }                                                           // 🟡 gated — T-36 round-trip on grammar production complete
```

**Per-arm dispositions:** `RustcAcceptsExpectation` and `RustRuntimeBehaviorExpectation` are 🟢 firm (rustc invocation surface is stable; runtime outcomes use std/runtime.dag carrier). `RustcRejectsExpectation` is 🟡 because its `expected_error_code` field is the Phase-1 subset; dissolves 🟢 when full `RustcErrorCode` coproduct lands per §2.1. `RustEmitProjectionEqualityExpectation` is 🟡 — exercisable only after SG-1 lands the `TargetAtomRealization` row to mutate; dissolves 🟢 when SG-1 lands. `RustGrammarRoundTripExpectation` is 🟡 — dissolves 🟢 when T-36 round-trip on rust.dag grammar productions is complete.

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

## §3. fact_id discipline — C(rust.dag) corpus per §5 Layer A

The acceptance contract from `docs/planning/v4-leaf-model-verification-2026-05-30.md` §5 Layer A (paraphrased — full text at the cited section):

> The leaf model owns FACTS (stable, claimable fact IDs declared in the model itself).
> The claim corpus owns VERIFICATION OBLIGATIONS (LeafModelClaim rows in sibling files referencing those fact IDs).
>
> Operationalized: for every claimable `fact_id` declared by M, the claim corpus C(M) contains one-or-more `LeafModelClaim` rows referencing `fact_id`.

### §3.0 Claimability — model-owned predicate (closes the 232-Symbol gap)

`rust.dag` at main HEAD `edc8cba73` declares ~232 top-level Symbols. NOT all are claimable facts; some are field-name tags (e.g., `rust_facts_field_surface_spelling`), edge-label discriminators (e.g., `target_model_edge_*`), token-class identifiers (e.g., `rust_token_kw_fn`), or structural sentinels with no verification angle. Treating all 232 as Phase-1 obligations would generate ~227 `not_checked` rows for non-claimable scaffolding, drowning the actual signal in noise — AND would leave the "which Symbols are claimable" question silently in the report-author's hands (P2 violation: claimability is a model-level fact, not a report-author decision).

**Model-owned claimability predicate** (declared in `src/v4/extdeps/languages/rust.dag` per Layer A authority):

```dag
// Step 4 deliverable (quick-tern-735): model-owned predicate enumerating
// the claimable fact_id subset for verification corpus generation.
fn rust_dag_claimable_fact_ids() -> List<Node> {
  // Returns the Node identities (per the §3.1 projection-fn discipline) of
  // every fact in rust.dag that BOTH (a) has a verification angle the runner
  // can exercise and (b) is not a scaffolding/tag-only declaration.
  //
  // Each entry maps 1:1 to a LeafModelClaim row in C(rust.dag) per the
  // Layer A contract.
  //
  // Concrete contents — enumerated in §3.2 below.
  [ ... ]
}
```

**Status:** the predicate function is a **Step 4 deliverable** (Runtime/TestClaim Mgr `quick-tern-735`); this spec defines its contract and the Phase-1 contents. Authoring the function moves the boundary into substrate authority where it belongs, instead of leaving it as docs-only narrative.

### §3.1 Three classes of claimable fact_id in rust.dag

Per the Node-typed `fact_id` discipline ratified by Modeling DFS msg_d5181972 and PR #3970 (`src/v4/extdeps/languages/rust.dag` projection fns `rust_leaf_model_fact_id_node_*`), claimable facts split into three structural classes:

1. **Top-level `data <name>: <RecordType>` declarations** — facts carried as full Conj-bundle records that the runner can introspect. Example: `rust_facts_i32: RustIntegerPrimitiveFacts` (rust.dag:574). Projection-fn: `rust_integer_facts_node(facts: rust_facts_i32)`.
2. **Derived facts via model-owned functions** — facts computed by a rust.dag function from other facts. Example: `rust_integer_algebra_inhabitance(rust_facts_i32)` returns an `AlgebraInhabitanceDecl` (rust.dag:1768). Projection-fn: `rust_integer_algebra_inhabitance(facts: rust_facts_i32).algebra`.
3. **Realization-row facts (planned)** — facts to be declared by SG-1 (`TargetAtomRealization`) and SG-5 (`TargetCollectionRealization`) per PR #3938 §10.1/§10.3. Example: `rust_atom_realization_symbol` (currently a placeholder Symbol on rust.dag; SG-1 will land the actual row).

**Explicitly NOT claimable** (structural-tag-only / scaffolding; not in the predicate's return list):

- Field-name tag Symbols (`rust_facts_field_*` at rust.dag:433-441) — used as edge-name discriminators on Conj-bundle records; carry no independent verification angle.
- Edge-label tag Symbols (`target_model_edge_*`) — same as above for TargetModelBundle edges.
- Token-class identifiers (`rust_token_*` at rust.dag:366+) — owned by grammar/lex blocks; verified via `RustGrammarProductionSubject` + `RustLexRuleSubject` at the production/rule level, not per-token.
- Connective discriminators (`rust_int_kind_*`, `rust_int_width_*`) — used as enum-style discriminators in derived facts; verified via the parent derived fact.
- Local binding Symbols inside `RustConcreteSyntaxToken` examples (`rust_binding_*`) — example-data, not model facts.

### §3.2 Phase-1 enumeration — exact fact_id list

The Phase-1 claimable subset is **4 fact_ids, 5 LeafModelClaim rows**. The full Phase-1 contents of `rust_dag_claimable_fact_ids()` for the Step 4 implementation:

```dag
// Phase 1 of rust_dag_claimable_fact_ids() — exhaustively enumerated.
// Phase 2 widens this list per §6 deferral.

[
  rust_leaf_model_fact_id_node_rust_facts_i32(),                                  // R1 — Class 1 fact
  rust_leaf_model_fact_id_node_algebra_inhabitance_rust_facts_i32(),              // R2a + R2b share this fact (two angles)
  rust_leaf_model_fact_id_node_atom_realization_symbol()                          // R3-external + R3-internal share this fact (two angles)
]
```

Three fact_ids in the Phase-1 list; 4th fact_id (R3) is named in §5 sketches via shared identity with the algebra Symbol. Per the Layer A contract this list, when iterated, MUST produce the 5 Phase-1 LeafModelClaim rows in C(rust.dag) — and only those.

### §3.3 Phase-2+ widening — exhaustive enumeration of remaining claimable facts

Every other claimable fact in rust.dag (estimated per the §3 inventory table: integer facts excluding i32 = 11 entries, float facts = 2, non-integer facts excluding planned realization rows = 5, T-4.17 grammar productions, lex rules, planned TargetCollectionRealization rows for SG-5) enters `rust_dag_claimable_fact_ids()` in Phase 2 with corresponding `LeafModelClaim` rows at `not_checked` verdict until Phase-2 fixtures land. The "238 declared facts vs ~5 verified" gap concern raised in PR #3971 review is closed by this enumeration discipline: Phase 2 widens the list to every claimable fact, and every entry has a row in C(rust.dag).

**Non-claimable Symbols (the ~227 scaffolding entries) NEVER enter the predicate's return list,** so they don't generate spurious `not_checked` rows. The §3.0 predicate is the load-bearing single authority that distinguishes "fact that needs a claim" from "tag Symbol that doesn't."

For M = rust.dag, the fact_id space at main HEAD comprises **all** facts the model declares, whether as top-level `data Symbol` lines OR as derived facts computed by model-owned functions (e.g., `rust_integer_algebra_inhabitance(facts)` at rust.dag:1768 derives an `AlgebraInhabitanceDecl` from a `RustIntegerPrimitiveFacts` record). The fact_id discipline accepts both shapes; verification claims reference the canonical form the model exposes (the Symbol for `data`-declared facts; the derived-value key for function-computed facts).

Inventory at main HEAD `edc8cba73` (approximate counts; row counts verified against rust.dag line ranges):

| fact_id family | count | Source location | Phase 1? |
| -------------- | ----- | --------------- | -------- |
| Primitive type facts (`rust_facts_*` records) | 19 total | Integer (`RustIntegerPrimitiveFacts`, 12 entries): rust.dag:562-633 covering `i8`/`i16`/`i32`/`i64`/`i128`/`u8`/`u16`/`u32`/`u64`/`u128`/`isize`/`usize`. Float (`RustFloatPrimitiveFacts`, 2 entries): rust.dag:634-647 covering `f32`/`f64`. Non-integer (`RustNonIntegerPrimitiveFacts`, 5 entries): rust.dag:648-700 covering `bool`/`char`/`str`/`unit`/`never`. | R1 covers `rust_facts_i32` (rust.dag:574) only |
| Algebra inhabitance (function-derived from primitive facts) | per-primitive | rust.dag:1768 `rust_integer_algebra_inhabitance(facts)` derives `AlgebraInhabitanceDecl` per `rust_facts_*` row | R2a/R2b cover the derived inhabitance for `rust_facts_i32` only |
| `rust_std_projection_*` | 19 | rust.dag:265-283 | none in Phase 1 |
| `rust_surface_spelling_*` | 19 | rust.dag:285-303 | none in Phase 1 |
| `rust_repr_*` / `rust_ieee754_*` / `rust_str_*` / `rust_char_*` | 7 | rust.dag:305-312 | none in Phase 1 |
| `rust_inhabitant_*` | 19 | rust.dag:314-332 | none in Phase 1 |
| `rust_inhabitant_field_*` | ~20 | rust.dag:334-353 | none in Phase 1 |
| `rust_coercion_field_*` | ~10 | rust.dag:354-364 | none in Phase 1 |
| Grammar productions (T-4.17 wave 1+2) | varies | rust.dag grammar block | none in Phase 1 |
| Lex rules | varies | rust.dag lex block | none in Phase 1 |
| TargetAtomRealization rows (Symbol/Bool/Char) | 3 (planned) | **Not on main yet — to be declared by SG-1 dispatch (Step 5).** R3-internal claim row authored at Step 4 references this fact_id (cited as `rust_atom_realization_symbol` for the Symbol carrier specifically) and stays `not_checked`/`GAP` until SG-1 lands. | R3 (Symbol only — fact_id `rust_atom_realization_symbol` covered by TWO LeafModelClaim rows: R3-external + R3-internal verification angles) |
| TargetCollectionRealization rows (Set/…) | varies (planned) | **Not on main yet — to be declared by SG-5 dispatch.** | none in Phase 1 |

**Every fact_id above MUST be present in C(rust.dag) as one-or-more LeafModelClaim rows** — Phase 1 covers 4 fact_ids: (a) `rust_facts_i32` (rust.dag:574 — top-level data Symbol declaring `RustIntegerPrimitiveFacts` for i32), (b) `rust_integer_algebra_inhabitance(rust_facts_i32)` × `RustClaimExpectation.RustcAcceptsExpectation` angle (derived-fact key — algebra-operations claim; R2a), (c) `rust_integer_algebra_inhabitance(rust_facts_i32)` × `RustClaimExpectation.RustRuntimeBehaviorExpectation` angle (derived-fact key — overflow-semantics claim; R2b), (d) `rust_atom_realization_symbol` (planned — Symbol-carrier TargetAtomRealization row, to be declared by SG-1 dispatch in Step 5; cited in Step 4 R3-internal scaffold). Note that R2a and R2b reference the SAME derived fact but at distinct verification angles, mirroring the R3 two-rows-per-fact pattern. 5 LeafModelClaim rows total: R1 (1 row on (a)), R2a (1 row on (b)), R2b (1 row on (c)), R3-external + R3-internal (2 rows on (d)). Per the §5 Layer A contract above, a fact_id may carry multiple `LeafModelClaim` rows (one per verification angle); the contract is "every fact_id has at least one claim," not "every fact_id has exactly one claim." The remaining ~85+ fact_ids enter C(rust.dag) with one `not_checked` row each until Phase 2 drains them. They are NOT invisible debt — they appear explicitly in `LeafModelVerificationReport<rust.dag>.totals.not_checked`.

The 94 catalog sentinels (count per `docs/planning/v4-leaf-model-verification-2026-05-30.md` §7 line 198; the canonical-home spec PR #3952 first surfaced this scaffold during pre-dispatch verification, and §7 of the planning doc records the count) all land here. None are dropped; none are "implicit." This is the operator's framing made operational: every model fact has a verification obligation in C(M).

## §4. R3-internal timing — Step 4 authors row, Step 5 lands fact

Per DFS msg_badac9f3 answer (4):

| Step | Work | R3-internal state |
| ---- | ---- | ----------------- |
| Step 4 (quick-tern-735) | Author R3-internal claim row in C(rust.dag) per §1/§2 shape; runner stub references `atom_realization_row: Node` slot but cannot exercise (no row to mutate yet) | `not_checked` OR explicit `GAP` verdict — runner correctly reports unfilled-slot, NOT a false PROVEN |
| Step 5 (TR Mgr → SG-1 worker) | SG-1 lands `TargetAtomRealization` row for Symbol/Bool/Char in `v4.std.target_model` per the canonical-home spec | R3-internal becomes exercisable; runner mutates row, re-emits, verifies both type-emit AND value-emit changed → `PROVEN` |

**Hard rule:** SG-1 is NOT considered closed until R3-internal flips from `not_checked` to `PROVEN` post-Step-5 with both `type_emit_must_change=true` and `value_emit_must_change=true` matching observed behavior. If only one emit changes, the single-authority gap that SG-1 was supposed to fix is still present — the carrier landed but the consumers diverge.

**Do NOT block Step 4 on SG-1 landing.** Step 4 can ship with R3-internal scaffolded-but-not-exercising; the GAP is honest progress, not failure.

## §4.1 FalsificationCase shape — substrate gap surfaced (PR #3970 follow-up)

PR #3970 (silent-cat-599) authored `FalsificationCase<Subject, Expectation> { subject_variant: Subject; expected_failure_mode: Verdict<Subject> }`. That shape captures wrongness **only when the falsification is in the subject** — but for many Phase-1 claims the wrongness lives in the **artifact** (e.g., R1 emits a string where i32 is expected; the Subject `RustPrimitiveTypeSubject { primitive: Rust_I32 }` is unchanged) or in the **expectation** (R2b varies the expected `OverflowAction` field, not the Subject).

The current shape forces every falsification probe to be expressible as a "deliberately wrong subject," which contradicts the §5 sketches that mutate artifact or expectation while keeping the subject identical. Per the codex blocking review on this PR (sha 39ac95c6, root-cause #3 + inline :123), this is a real shape gap.

**Proposed substrate extension** (forwarded to silent-cat-599 / Modeling DFS for PR #3970 follow-up — NOT a unilateral TR-side change to a DFS-owned carrier):

```dag
// Proposed extension (await PR #3970 follow-up + DFS ratification):
type FalsificationCase<Subject, Expectation> {
  variant: FalsificationVariant<Subject, Expectation>
  expected_failure_mode: Verdict<Subject>
}

type FalsificationVariant<Subject, Expectation> =
  | SubjectVariant     { subject_variant: Subject }                    // current shape — wrongness in subject
  | ArtifactVariant    { artifact_variant: TargetArtifact }            // wrongness in emitted artifact only
  | ExpectationVariant { expectation_variant: Expectation }            // wrongness in expectation only
  | CompoundVariant    { subject_variant: Optional<Subject>, artifact_variant: Optional<TargetArtifact>, expectation_variant: Optional<Expectation> }
```

This widens `FalsificationCase` to carry the actual locus of wrongness without forcing it into the subject slot. Every Phase-1 §5 sketch then types cleanly: R1 uses `ArtifactVariant`, R2b uses `ExpectationVariant` (per-field overflow-disposition probe), R3-external uses `SubjectVariant` (the Symbol carrier IS the wrong subject when its realization row is mismatched), R3-internal uses `ExpectationVariant` (vary the `RustEmitProjectionEqualityExpectation.type_emit_must_change` / `value_emit_must_change` flags).

**Pending the substrate extension**, §5 sketches below use a hybrid notation that names the actual locus of wrongness even though the current generic carrier types only `subject_variant`. Step 4 (quick-tern-735) is the right consumer to surface the shape gap concretely when building the runner; expect a substrate change request to silent-cat-599's follow-up PR before Phase-1 runner fully lands.

## §5. Per-claim sketches (Phase 1 — 4 fact_ids, 6 LeafModelClaim rows post-R2b split)

Schematic — full fixture content is Step 4 deliverable. R2b is split into per-OverflowAction-field sub-claims per the codex review (root-cause #4 + inline :142); Phase 1 row count grows from 5 to 6 (R1 + R2a + R2b-debug-default + R2b-release-default + R3-external + R3-internal). The R2b split closes the gap where a wrong release/wrap fact could pass Phase 1 under a debug-only check.

**Note on falsification-locus notation:** each sketch below names the wrongness locus explicitly (`subject_variant:`, `artifact_variant:`, `expectation_variant:`). Until the §4.1 substrate extension lands, the current `FalsificationCase` carrier types only `subject_variant`; the other forms are documented intent that Step 4 will need substrate to express cleanly.

### R1 — Rust i32 primitive declaration
```text
fact_id:    rust_facts_i32                  // rust.dag:574 (top-level data Symbol; RustIntegerPrimitiveFacts record for i32)
subject:    RustPrimitiveTypeSubject { primitive: Rust_I32 }
expectation: RustcAcceptsExpectation { invocation_form: rustc("pub fn r1_test() -> i32 { 0i32 }") }
falsification (locus: artifact — pending §4.1 ArtifactVariant substrate):
  artifact_variant: rustc("pub fn r1_test() -> i32 { \"string\" }")
  expected_failure_mode: RustcRejectsExpectation { ..., expected_error_code: E0308 }
  // Wrongness lives in the emitted artifact (string in i32-typed position), not the subject.
  // The subject "RustPrimitiveTypeSubject { primitive: Rust_I32 }" is correct on both paths;
  // what's varied is what gets emitted into the i32-typed slot.
```

### R2a — i32 supports algebra ops
```text
fact_id:    rust_integer_algebra_inhabitance(rust_facts_i32)    // derived-fact key per rust.dag:1768 function applied to rust.dag:574 facts; algebra-operations verification angle
subject:    RustAlgebraInhabitanceSubject { algebra: OrderedRingCarrier, on: Rust_I32 }
expectation: RustcAcceptsExpectation { invocation_form: rustc("pub fn r2a_test(a: i32, b: i32) -> (i32, bool) { (a + b, a < b) }") }
falsification:
  subject_variant: claim non-existent op (e.g., i32::log2_exact)
  expected_failure_mode: RustcRejectsExpectation { ..., expected_error_code: E0599 }
```

### R2b — i32 overflow semantics declared (SPLIT per OverflowDisposition field)

`rust.dag:235-241` declares `OverflowDisposition<IRCarrier>` with **four** OverflowAction fields:

```dag
type OverflowDisposition<IRCarrier> {
  ir_carrier: IRCarrier
  checked_arithmetic_debug_default: OverflowAction              // i32: PanicOnOverflow
  checked_arithmetic_release_default: OverflowAction            // i32: TwoComplementWrap
  checked_arithmetic_overflow_checks_enabled: OverflowAction    // i32: PanicOnOverflow
  checked_arithmetic_overflow_checks_disabled: OverflowAction   // i32: TwoComplementWrap
}
```

A single R2b claim that only exercises debug-default would let a wrong release-default fact pass Phase 1 silently. R2b therefore splits into **one sub-claim per OverflowAction field** — all four — sharing the `rust_integer_algebra_inhabitance(rust_facts_i32)` fact_id. Phase-1 count: R2b becomes 4 LeafModelClaim rows on one fact_id at distinct verification angles.

For Phase-1 fixture economy: implement the two distinct-value sub-claims as live fixtures (debug-default `PanicOnOverflow` + release-default `TwoComplementWrap` — they cover both `OverflowAction` cases the model declares for i32); the two overflow-checks-enabled/disabled sub-claims author at `not_checked` since they duplicate the action values of debug/release-default. The 4-row enumeration still lands; only 2 carry runner-exercising fixtures.

```text
// R2b sub-claim 1 of 4 — debug-default
fact_id:    rust_integer_algebra_inhabitance(rust_facts_i32)
subject:    RustAlgebraInhabitanceSubject { algebra: OrderedRingCarrier, on: Rust_I32 }
expectation: RustRuntimeBehaviorExpectation {
              invocation_form: rustc-then-run("[debug build] i32::MAX + 1"),
              expected_outcome: Panic { panic_classification: rust_leaf_arithmetic_overflow_panic }
            }
falsification (locus: expectation — pending §4.1 ExpectationVariant substrate):
  expectation_variant: RustRuntimeBehaviorExpectation { ..., expected_outcome: Wrap }
  expected_failure_mode: observed actual outcome is Panic (not Wrap), so the mutated
                         expectation diverges → the original Panic claim PROVEN, the
                         Wrap variant correctly REJECTED.

// R2b sub-claim 2 of 4 — release-default
fact_id:    rust_integer_algebra_inhabitance(rust_facts_i32)
subject:    RustAlgebraInhabitanceSubject { algebra: OrderedRingCarrier, on: Rust_I32 }
expectation: RustRuntimeBehaviorExpectation {
              invocation_form: rustc-then-run("[release build] i32::MAX + 1"),
              expected_outcome: Wrap
            }
falsification (locus: expectation):
  expectation_variant: RustRuntimeBehaviorExpectation { ..., expected_outcome: Panic { panic_classification: rust_leaf_arithmetic_overflow_panic } }
  expected_failure_mode: observed actual outcome is Wrap (not Panic) under release

// R2b sub-claim 3 of 4 — overflow_checks_enabled  (Phase 1: not_checked stub; same expected_outcome as debug-default)
// R2b sub-claim 4 of 4 — overflow_checks_disabled (Phase 1: not_checked stub; same expected_outcome as release-default)
```

The 4-row R2b structure makes the OverflowDisposition coverage gap explicit: a wrong release-default fact in rust.dag now requires its dedicated row to be `PROVEN`, not just R2b-debug. The two `not_checked` stubs surface the overflow-checks-enabled/disabled fields as known verification debt even though their values are model-derived-from debug/release.

### R3-external — Symbol projection to Rust shape accepted by rustc
```text
fact_id:    rust_atom_realization_symbol    // PLANNED — to be declared by SG-1 dispatch (Step 5); not on main yet. Step 4 authors this row at not_checked/GAP per §4. Shared with R3-internal — two verification angles on one fact
subject:    RustAtomRealizationSubject { atom: <Symbol carrier Node> }
expectation: RustcAcceptsExpectation { invocation_form: rustc(emitted-from-TargetAtomRealization-row) }
falsification (locus: subject — the deliberately-wrong Subject IS the real change here):
  subject_variant: RustAtomRealizationSubject { atom: <Symbol carrier Node with mismatched TargetAtomRealization row — type alias + value-constructor call> }
  expected_failure_mode: RustcRejectsExpectation { ..., expected_error_code: E0423 }    // the SG-1 root-cause error
  // Wrongness lives in the Subject's atom Node identity (different realization row); current
  // FalsificationCase.subject_variant slot types this cleanly without §4.1 extension.
```

### R3-internal — TargetAtomRealization row mutation receipt
```text
fact_id:    rust_atom_realization_symbol    // PLANNED (see R3-external note) — shared fact with R3-external; this is the mutation-receipt angle
subject:    RustAtomRealizationSubject { atom: <Symbol carrier Node> }
expectation: RustEmitProjectionEqualityExpectation {
              atom_realization_row: <Symbol carrier Node>,
              type_emit_must_change: true,
              value_emit_must_change: true
            }
falsification (locus: expectation — pending §4.1 ExpectationVariant substrate):
  expectation_variant: RustEmitProjectionEqualityExpectation {
                         atom_realization_row: <same Symbol carrier Node>,
                         type_emit_must_change: false,
                         value_emit_must_change: false
                       }
  expected_failure_mode: structural-equality oracle observes one-or-both emit outputs DID change after row mutation
                         (i.e., the wrong-expectation claim "row mutation has no effect" is REJECTED by observed reality,
                          which proves the original Expectation "both must change" is PROVEN).
  // Wrongness is in the Expectation's must-change flags, not in the Subject (same atom Node both paths).
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
