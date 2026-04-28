# Design — Emission Model (no separate coercion engine)

**Status:** `PROPOSAL` (2026-04-28). Pending Director sign-off + alignment with [`docs/thesis/target-grounding-proposal.md`](thesis/target-grounding-proposal.md).

**Authority on promotion:** [`THESIS.md`](../THESIS.md) §"Tier 1 — Structural correctness" — "**Coercion = emission: the compiler reads a target spec and translates. No separate coercion engine.**" — this doc operationalizes that thesis claim.

**Supersedes** the "Coercion engine" framing in [`docs/thesis/target-grounding-proposal.md`](thesis/target-grounding-proposal.md) §"Substrate work" and the `T-Ground-Engine` lane framing in [`ROADMAP.md`](../ROADMAP.md) and [`docs/briefs/grounding-manager.md`](briefs/grounding-manager.md). Those documents predate the cascade-promotion thesis-discipline tightening (2026-04-25); they describe the work as "engine + selection + tie-breaking" which contradicts THESIS:171.

**Affects already-merged work:** [PR #989](https://github.com/gunb-ai/gunbc/pull/989) "T-Ground-Engine: Phase 2 pilot-list enumeration (slice 1)" merged on main with the inhabitance-search + selection + tie-breaking framing. **Post-merge realignment is required** per §"Affected lanes" below — slice 1 code on main needs follow-up PR(s) to retract selection logic and use `EmissionDiagnostic` carrier; further slices (Phase 2 slice 2+) hold until the design promotes.

## Goal

**Coercion is structural projection, not a decision process.** Emission reads declared substrate facts and returns either a unique target primitive or a fail-closed diagnostic. There is no parallel authority that "picks up slack" when the substrate is incomplete; if the substrate under-determines, the substrate is incomplete and the right response is to extend the substrate, not bolt on a decider.

This is the same discipline P3 Fail-Closed applies to all consumers: if a fact is missing, the fix is to declare the fact, not to fabricate a plausible default. The "Engine" framing implies that emission is special — that under-determinism at the realization boundary should be resolved by a separate authority. It isn't special; the discipline is the same.

### Why this matters

Three load-bearing reasons no engine should exist:

1. **Thesis faithfulness.** Per THESIS:171, "coercion = emission" is a *concept unification*. Emission *is* coercion; there's no separate phase. An engine that decides between targets reintroduces the phase the thesis says doesn't exist.

2. **Cost-of-change discipline.** An engine becomes a parallel authority. When substrate facts change (new target primitive, new refinement shape, new algebra inhabitance), an engine has to be updated to know about them. Per P2 Boundary Discipline, every fact lives in exactly one place — an engine that holds selection logic *is* a fact (the canonical choice fact, the ordering fact, the tie-breaking policy fact) being held outside the substrate where the rest of the system reads it.

3. **Reviewability.** A reviewer can audit declared substrate facts — the algebra inhabitance is right there in `dsl/extdeps/languages/rust/types.dag` (or its successor). A reviewer cannot easily audit engine selection logic because the logic interacts with all the facts simultaneously. Declared canonical-choice + declared ordering + declared inhabitance is auditable; engine policy is not.

## What "no engine" actually means structurally

The model is:

```
Program declares intent          Substrate declares facts
       │                                   │
       └──────────► STRUCTURAL FOLD ◄─────┘
                          │
                          ▼
                   Unique target primitive
                          OR
                   Fail-closed diagnostic
```

- **Program intent** = `.dag` algebra inhabitance + refinement bounds + (optional) explicit type annotations
- **Substrate facts** = target language specs (per `dsl/extdeps/languages/*/`); each candidate target's structural properties (ownership, lifetime, growability, encoding-invariant, etc. — declared as algebra refinements per Modeling problem 2); declared structural ordering for diagnostic enumeration
- **Structural fold** = mechanical implementation. Reads program intent, walks substrate facts, produces a result. No selection logic; no tie-breaking policy; no "minimum-satisfier" heuristic that lives in the fold itself.
- **Result** = unique target primitive (when fold structurally determines) or typed `EmissionDiagnostic` carrier naming what would resolve the under-determinism (when fold cannot)

The fold is small and mechanical because all the *real work* is in the substrate facts. Anything the fold has to "decide" is a fact the substrate should have declared.

## What the no-engine discipline forces us to model

This is the load-bearing section. The "engine" framing was hiding work — work that is *real* and *hard* and now becomes visible as modeling problems we have to think through carefully. Each is a lane of substrate completion that previously was implicit in "the engine will figure it out."

### Modeling problem 1 — refinement composition with algebra inhabitance

**Question:** how does a refinement bound participate in inhabitance search?

**Worked example.** A `.dag` program declares `count: Int(0..2^32)`. The intent is "non-negative 32-bit integer." Rust offers `u8`, `u16`, `u32`, `u64`, `u128`, `usize`. Each inhabits `Semiring<WordN>` (no negatives → no `OrderedRing`). The fold must determine: which target inhabits `Semiring` *at the refinement bound* `(0..2^32)`?

The honest answer is: `u32` *uniquely* inhabits `Semiring` at exactly that refinement. `u64` inhabits at a wider refinement (also acceptable but not minimum). `u16` does not inhabit at this refinement (overflow). `usize` is platform-dependent.

**What the substrate must declare** for this to be a structural fold:
- Each target primitive declares its `Semiring` inhabitance with a *cardinality bound* attached (`u32` inhabits `Semiring<Word32>` at bound `0..2^32`; not just "inhabits Semiring")
- The fold matches program refinement against target refinement *within the same algebra*
- "Minimum" is determined by *bound subsumption* (`u32`'s bound exactly equals program bound; `u64`'s bound strictly contains program bound; `u32` is "smaller" by a structural ordering on bounds)

**What the substrate cannot do today.** ROADMAP names DB-11 (refinement-carrying qualifiers on primitives) + cardinality-substrate (container cardinality bounds) as substrate prerequisites for `T-Ground-Rust`. That's exactly this: the substrate doesn't yet carry refinement-attached algebra inhabitance.

**The work.** Extending the substrate so each declared inhabitance carries its refinement bound. This is real modeling work. It's NOT "the engine handles refinement"; it's "the substrate declares refinement-bounded inhabitance."

### Modeling problem 2 — surfacing structural differences instead of canonical choice

**Question:** when the substrate appears to declare multiple primitives that inhabit the same algebra at the same refinement, what's *actually* different between them — and is that difference cosmetic or meaningful?

**The honest framing (corrected 2026-04-28 per user direction):** "multiple inhabitants at the same algebra+refinement" is itself a smell. Either the candidates are *structurally equivalent* (cosmetic — same thing under different names) and they collapse into one, or they're *meaningfully different* (different semantic invariants — ownership, lifetime, mutability, encoding) and the difference belongs in the substrate as additional structural facts. There is no third category requiring "canonical choice" engine machinery.

**Worked example.** Rust offers `String`, `Box<str>`, `Vec<u8>`, `Box<[u8]>`, `&str`, `Cow<str>`. Are these cosmetic variants of `FreeMonoid<Char>` or meaningfully different?

| Candidate | Owned? | Growable? | UTF-8 invariant? | Lifetime |
|---|---|---|---|---|
| `String` | yes | yes | yes | self-contained |
| `Box<str>` | yes | no | yes | self-contained |
| `Vec<u8>` | yes | yes | no | self-contained |
| `Box<[u8]>` | yes | no | no | self-contained |
| `&str` | no | n/a | yes | borrowed |
| `Cow<str>` | conditional | conditional | yes | conditional |

These are **meaningfully different** on four structural dimensions: ownership, growability, encoding-invariant, lifetime. Modeling these dimensions as substrate refinements (or as additional algebras) means each combination of program structural intent maps to a unique target. **No canonical needed** — the candidates aren't tied; they inhabit *different* algebra+refinement combinations.

**The work.** Surface every meaningful difference between apparent multi-inhabitants as a structural fact:
- Add ownership as a refinement on `FreeMonoid<Char>` (or a separate algebra `OwnedFreeMonoid<Char>` vs `BorrowedFreeMonoid<Char>`)
- Add growability as a refinement (or as the distinction `FreeMonoid` (growable) vs `Sequence<N>` (fixed-size))
- Add encoding-invariant as a refinement (`FreeMonoid<Char>` UTF-8 vs `FreeMonoid<Byte>` raw)
- Lifetime is a structural property derivable from program use (see Modeling problem 3 below)

After modeling, **the choice falls out**: each combination of (owned, growable, encoded) yields a unique target. The "canonical choice" framing was hiding the fact that we hadn't done this modeling work.

**The principle.** When the substrate looks like it has "multiple inhabitants at same algebra+refinement," ask:
- Is the difference cosmetic? → candidates collapse; substrate has duplicate authority to remove
- Is the difference meaningful? → the meaningful axis belongs in the substrate as a fact; once added, candidates are uniquely distinguished

This is more thesis-faithful than canonical-choice declaration, because it forces honest modeling of what "the same algebra at the same refinement" actually means. **No engine; no canonical-choice metadata; just structural completeness.**

### Modeling problem 3 — structural derivation of program intent (no annotations)

**Question:** when a `.dag` program declares a value, where does the structural intent (ownership, lifetime, growability, encoding) come from?

**Retracted 2026-04-28 per user direction:** the prior framing of this problem proposed `@target(rust) annotate` syntax to let users declare ownership/etc. **No annotations.** Annotations would be a parallel-authority shape — even if structurally well-formed, they introduce a vocabulary outside the program's own structural facts. The thesis position is sharper: **the program's structural intent is derivable from the program itself.**

**Worked example.** Consider a `.dag` value of type `String`:

```
data name: String = "Alice"
fn greet(n: String) -> String { ... }
data result: String = greet(name)
```

What ownership does each `String` need at the Rust target?

- `name`'s lifetime must outlive its uses; if it's used after the binding scope ends, it must be **owned**
- `n`'s lifetime is bounded by the function call; if `greet` doesn't store `n`, `n` can be **borrowed** (`&str`)
- `greet`'s return value must be self-contained (Rust functions can't return references to local data without lifetime annotations); it's **owned**
- `result` receives an owned value; storing it in a binding makes it **owned**

These are **structural facts derivable from program use** — lifetime, escape, storage. Rust's borrow checker derives them at compile time; gunbc's fold can do the same as part of inhabitance search.

**What the substrate must declare:**
- Each candidate target type's structural properties (ownership, lifetime, growability, encoding) — modeled per Modeling problem 2 as algebra refinements
- A *structural lifetime/escape analyzer* that reads the program's bindings, function signatures, and use sites, and concludes the required ownership for each value
- The fold composes: program intent (derived structurally) + target candidate properties (declared structurally) → unique inhabitant

**The work.** Two pieces:
1. **Structural property model on target types** (covered in Modeling problem 2)
2. **Lifetime/escape analyzer** that derives program intent without annotations. This is essentially Rust's borrow checker run forwards: instead of "is this borrow valid?", ask "what ownership does this value need given how the program uses it?" The answer is structural, falls out of program graph + binding scopes + function signatures.

**Why this is more thesis-faithful than annotations:** the program already declares what it does (bindings + uses + signatures). Asking the user to *also* annotate is duplicate authority — the use pattern is itself the declaration of intent. Annotations would force the user to keep the use and the annotation in sync, with no way to dissolve drift.

**Open question:** does this analyzer live in the Evaluator program (post-R3), or is it a substrate-completion lane in R2 alongside LanguageSpec? **Recommendation:** R2 lane. The fold needs lifetime analysis to determine target type for any program with non-trivial scoping. Defer is dishonest — the canonical examples (String/&str/Cow) require it from day one. Lane name suggestion: **T-Ground-Lifetime-Analyzer**.

### Modeling problem 4 — declared structural ordering ("which is smaller")

**Re-framed 2026-04-28:** "declared structural ordering" is no longer about choosing between tied candidates ("minimum-satisfier"). After Modeling problem 2's correction, ties don't exist — meaningful differences are modeled structurally so candidates uniquely match program intent. Ordering remains useful for *diagnostic enumeration* (telling users "your unrefined `Int` could be Int8/Int16/.../Int128 — here are the bounds") but is no longer load-bearing for emission.

**Question:** when the fold needs to pick "minimum" satisfier, what declares the ordering?

**Re-framing:** the fold should *not* "pick minimum." After Modeling problem 2's correction, the right reframe is: when the program is structurally complete (all relevant axes refined), there is exactly one matching candidate; "smaller" doesn't enter. When the program is structurally under-specified, the fold fails-closed and the diagnostic enumerates the candidates in some order so the user knows what refinement to add.

**The honest answer:** the ordering is *structural*, declared in the substrate, used for diagnostic enumeration only. For integer carriers, "smaller" means "narrower bound" — declared by the bound carrier itself. For ownership variants of strings, there is no natural ordering; the substrate declares an enumeration order for diagnostics. **Neither is engine policy** — both are declared facts.

**The work:** declared ordering exists per *category* of inhabitance, attached to the algebra. For algebras with a natural ordering (cardinality bounds), the ordering falls out of the bound carrier. For algebras without (ownership/lifetime variants), the substrate declares a diagnostic-only enumeration order. The fold itself does not consult ordering for emission decisions; it consults ordering only when constructing fail-closed diagnostics.

This is substrate completion. The "minimum-satisfier selection" framing was hiding this modeling decision under engine policy; the corrected framing is "ordering is for telling users what their alternatives are, not for the fold to pick between them."

### Modeling problem 5 — fail-closed diagnostic surface

**Question:** when the fold fails (no inhabitant; multiple inhabitants because the program is under-refined on some structural axis), what does the diagnostic look like?

**The diagnostic is itself a structural fact.** It must name:
- What the program declared (the algebra + refinement that was searched)
- What the substrate declared (the inhabitants found, or the absence)
- What would resolve the under-determinism (refinement to add, structural axis to declare on the program, or substrate fact to extend if no candidate exists)

**Worked example.** User writes `Int` without bounds. Fold runs, finds 5 candidates (Int8/Int16/Int32/Int64/Int128). All 5 are meaningfully different (bound differs); the program hasn't said which bound it needs. Fold returns:

```
EmissionDiagnostic::UnderRefined {
  program_intent: AlgebraInhabitance(OrderedRing, refinement: None),
  candidates: [Int8, Int16, Int32, Int64, Int128],
  unspecified_axis: "bound",
  resolution_hints: [
    "add refinement bound: `Int(min..max)` will narrow the search",
    "for typical 64-bit integer use: `Int(-2^63..2^63)` grounds to Int64",
    "for 32-bit: `Int(-2^31..2^31)` grounds to Int32"
  ]
}
```

**The work:** designing the `EmissionDiagnostic` carrier in the substrate. This is small but load-bearing — it's the structural surface for "the substrate is complete; the program is not." (Distinguish from `EmissionDiagnostic::NoInhabitant` for "the substrate doesn't have a candidate for this case at all.")

### Modeling problem 6 — language spec as substrate

**Question:** what's the *substrate shape* of a language spec?

Today: scattered across `dsl/std/coercion.dag` (schema) + `dsl/extdeps/languages/{rust,python,go}/types.dag` (instantiation tables). Per [`docs/thesis/target-grounding-proposal.md`](thesis/target-grounding-proposal.md), the current shape is "table-driven coercion" and is bootstrap scaffolding to dissolve.

**The structural shape needs:**
- Declared primitive set (with refinement-bound shape per primitive)
- Declared algebra inhabitance per primitive (with refinement parameters)
- Declared structural axes that distinguish candidates when multiple primitives appear to inhabit the same algebra (per Modeling problem 2 corrected: model the meaningful axis as a refinement; cosmetic equivalents collapse)
- Declared structural ordering for diagnostic enumeration
- Declared construction patterns (how a target value of this primitive is *constructed* from other target values — needed for emission of compound types)
- Declared operator dispatch (how `OrderedRing.add` projects onto `i64.add` vs `BigInt.add` etc. — already partially in `MethodContract`)
- Declared external-realization shape (per E-9: external realization lives on `Arrow.body`)

**A "language spec" is therefore a structured `.dag` declaration**, not a table. THESIS already gestures at this (Concept unification: "Target language spec = transport spec = interpreter runtime"). The work is naming the schema.

**The work:** authoring the `LanguageSpec` schema in `src/v3/std/` (or equivalent canonical location), populating it for Rust + Python + Go, dissolving the existing `coercion.dag` schema via `T-Ground-Dissolve` once the new shape is consumed.

### Modeling problem 7 — cross-target uniformity declarations

**Question:** when does `.dag` algebra map to "the Rust thing AND the Python thing AND the Go thing" all at once?

**Worked example.** `.dag` `String` should emit to Rust `String`, Python `str`, Go `string`. The mapping isn't separate per target — it's *the same algebraic intent realized in three target-language vocabularies*.

**What the substrate must declare:**
- Per-target language specs (Modeling problem 6) cover *each* mapping individually
- A *cross-target meta-spec* declares: which inhabitances are *required* to have at least one structural-completeness candidate across all Shape A targets? (e.g., "every Shape A target must have a candidate inhabiting `OrderedRing` covering the common bound `(-2^31..2^31)`, because `.dag` `Int(-2^31..2^31)` is portability-required")
- Without this meta-spec, a target language spec could omit an inhabitance covering a portable case without diagnostic, breaking cross-target portability silently

**The work:** designing the cross-target meta-spec. This is substrate-level — declares which inhabitances are portability requirements vs target-specific niceties. Failing to declare this leaves portability as policy rather than structural fact.

### Modeling problem 8 — first-class language-spec emission (dogfooding)

**Question:** if a language spec is `.dag`, does the compiler emit Shape A on its own spec?

**The thesis answer is yes.** Per Self-hosting facet 1 + Omni-emission, the compiler should be `.dag`-authored end to end including its own external boundaries. A language spec authored as `.dag` *should* itself emit to a target — at minimum, to a structured artifact (e.g., a JSON schema) that consumers can read.

**This is a dogfooding question.** If the language spec is `.dag` but doesn't emit, it's a special case. If it emits, the spec composes with omni-emission.

**The work:** decide whether language specs are first-class compiler subjects. **Recommendation: yes, but post-R3.** R2 + R3 land the language spec substrate; post-R3 work makes the spec a first-class compiler subject (probably as part of ecosystem buildout).

## How this changes R2 / R3 lane structure

### What "T-Ground-Engine" was hiding

The lane was sized as "M (~1-2 weeks first-cut)" per [`docs/thesis/target-grounding-proposal.md`](thesis/target-grounding-proposal.md):344. Decomposing that M into the modeling problems above:

| Modeling problem | Lane home | Size |
|---|---|---|
| 1. Refinement composition | T-Ground-Rust XL (extends substrate per-target) + T-Substrate cardinality-substrate prereq (already in R2) | Folds into existing |
| 2. Structural axes (replaces "canonical choice" framing) | T-Ground-Rust + T-Ground-Python + T-Ground-Go (each declares the structural axes — ownership/growability/encoding/etc. — that distinguish its primitive families) | Folds into existing |
| 3. User annotation as program substrate | **NEW LANE** — substrate work for program-side `@target` annotations | M |
| 4. Declared structural ordering | T-Substrate (declared ordering on cardinality bounds; declared enumeration order on alternates) | Folds into existing |
| 5. Fail-closed diagnostic surface | **NEW LANE** — `EmissionDiagnostic` carrier substrate | S |
| 6. Language spec as substrate | **NEW LANE** — `LanguageSpec` schema authoring + per-target population | M (was hidden as part of "T-Ground-Engine") |
| 7. Cross-target uniformity meta-spec | **NEW LANE** — cross-target portability requirements | S |
| 8. First-class language-spec emission | **POST-R3** — dogfooding | not in R2/R3 |

What remains as a "fold lane":
- **T-Ground-Coercion-Fold** (rename of T-Ground-Engine) — the mechanical implementation that reads declared facts and returns unique answer or `EmissionDiagnostic`. **S size, not M.** Most of the work was in the modeling problems above; the fold itself is small.

### Recommended R2 lane structure update

Replace `T-Ground-Engine` with five lanes:

| Lane | Size | Owns |
|---|---|---|
| **T-Ground-Coercion-Fold** | S | The mechanical structural fold. Reads program intent + substrate facts; returns unique target or `EmissionDiagnostic`. Replaces `T-Ground-Engine` framing |
| **T-Ground-LanguageSpec** | M | `LanguageSpec` schema authoring + per-target population + dissolve of existing `coercion.dag` table-driven shape via T-Ground-Dissolve |
| **T-Ground-Annotation** | M | Program-side `@target` annotation substrate; composition rule with language spec |
| **T-Ground-Diagnostic** | S | `EmissionDiagnostic` carrier substrate; resolution-hint structure |
| **T-Ground-CrossTarget-Meta** | S | Cross-target portability requirements meta-spec |

Plus existing:
- T-Ground-Pilot (S, landed)
- T-Ground-Rust (XL, in R2 amendment 2026-04-28; absorbs Modeling problems 1, 2, 4 for Rust target)
- T-Ground-Python (L, in R2 amendment 2026-04-28; absorbs Modeling problems 1, 2, 4 for Python target)
- T-Ground-Go (L, in R2 amendment 2026-04-28; absorbs Modeling problems 1, 2, 4 for Go target — partially landed via #910)
- T-Ground-Tests (S)
- T-Ground-Dissolve (S)

**Net effect on R2:**
- Lane count grows from 7 to 11 (Engine → 5 lanes)
- Total scope is the same or slightly larger; *visibility* of scope is much higher
- Honest about what work the substrate completion actually involves

### Recommended R3 lane structure update

R3 keeps T-Verification-L4L7 as the verification lane that *proves* the no-engine claim:

- L4 (`l4_emit_eval_match`): for every program, emitted target output equals `.dag` evaluation. **If the fold fabricates a target choice that .dag doesn't evaluate to, L4 fails — verifies the no-engine claim by execution.**
- L5 (`l5_cross_target_consistency`): emitted Rust/Python/Go produce equivalent runtime behavior. **Any engine policy that resolves inconsistently across targets would fail L5.**
- L6 (`l6_structural_form_coverage`): every structural form emits to every target. **Verifies that fail-closed gaps are explicitly diagnosed, not silently engine-resolved.**
- L7 (`l7_algebraic_laws_witnessed`): every algebra has runtime-constructed witnesses. **Verifies that algebra inhabitance is declared and structural, not engine-policy-asserted.**

These four together are the structural test of the no-engine discipline.

## Worked examples

These are concrete walkthroughs of the structural fold for representative cases. Each example includes the substrate facts it depends on, the program input, the fold steps, and the expected output (target code OR `EmissionDiagnostic`). **These serve as test cases** — the structure is intentionally reproducible: each example can be lifted into a `.dag` `TestClaim` once the substrate lanes (T-Ground-LanguageSpec + T-Ground-Annotation + T-Ground-Diagnostic + T-Ground-CrossTarget-Meta) land.

### Example 1 — `Int` (no refinement) → fail-closed `EmissionDiagnostic::UnderRefined`

**Demonstrates:** the honest answer for an under-specified program. **There is no canonical Int → i64 default**; the program hasn't said which integer width it needs, and Int8/Int16/Int32/Int64/Int128 are *meaningfully different* (different bounds, different memory). Defaulting to Int64 would be engine choice — which the thesis rules out.

**Substrate facts (must be declared):**
```
// In dsl/extdeps/languages/rust/types.dag (or successor LanguageSpec home)
inhabits Int8   : OrderedRing  bound = (-2^7..2^7)
inhabits Int16  : OrderedRing  bound = (-2^15..2^15)
inhabits Int32  : OrderedRing  bound = (-2^31..2^31)
inhabits Int64  : OrderedRing  bound = (-2^63..2^63)
inhabits Int128 : OrderedRing  bound = (-2^127..2^127)
```

(Note: no `canonical` fact. Per Modeling problem 2 corrected, "canonical declaration" was the wrong framing — the bound differences ARE meaningful, so they belong as refinements, not as candidates needing canonical disambiguation.)

**Program input:**
```
data x: Int = 0
```

**Fold steps:**
1. Read program intent: algebra = `OrderedRing`; refinement = none (`Int` without bounds)
2. Walk substrate inhabitants of `OrderedRing`: { Int8, Int16, Int32, Int64, Int128 }
3. Apply refinement filter: refinement = none → 5 candidates remain (all are *strictly wider* than zero candidates; no candidate is uniquely matched)
4. The candidates have different bounds. The program hasn't declared which bound it needs. **The program is structurally under-specified.**
5. Result: fail-closed → `EmissionDiagnostic::UnderRefined`

**Expected output:**
```
EmissionDiagnostic::UnderRefined {
  program_intent: AlgebraInhabitance(OrderedRing, refinement: None),
  candidates: [Int8, Int16, Int32, Int64, Int128],
  resolution_hints: [
    "add a refinement bound: `Int(min..max)` will narrow the search",
    "for typical 64-bit integer use: `Int(-2^63..2^63)` grounds to Int64",
    "for 32-bit: `Int(-2^31..2^31)` grounds to Int32"
  ]
}
```

**Test claim shape:**
```
fold_dag_int_unrefined_fails_closed: TestClaim {
  setup: standard_rust_language_spec()
  source: "data x: Int = 0"
  expected_emission: None
  expected_diagnostic: matches(EmissionDiagnostic::UnderRefined { candidates: [_; 5], .. })
}
```

**Why this is the right answer:** `Int8` is not a cosmetic variant of `Int64`; the bound is structurally meaningful. A program that wrote `Int` and got `i64` by engine policy would have its semantic intent silently chosen by the compiler. The thesis-honest answer is: tell the user the program is under-specified; show what they could write to be specific.

---

### Example 2 — `Int(0..2^32)` → Rust `u32` (refinement-driven)

**Demonstrates:** refinement composition with algebra inhabitance (Modeling problem 1); minimum bound matching is structural via subsumption, not engine policy.

**Substrate facts (must be declared):**
```
inhabits UInt8   : Semiring  bound = (0..2^8)
inhabits UInt16  : Semiring  bound = (0..2^16)
inhabits UInt32  : Semiring  bound = (0..2^32)
inhabits UInt64  : Semiring  bound = (0..2^64)
inhabits UInt128 : Semiring  bound = (0..2^128)

// Declared structural ordering on Semiring bounds: bound subsumption
// (a is "smaller than or equal to" b iff a's range is contained in b's)
```

**Program input:**
```
data count: Int(0..2^32) = 100
```

**Fold steps:**
1. Read program intent: algebra = `Semiring` (no negatives → not `OrderedRing`); refinement = `(0..2^32)`
2. Walk substrate inhabitants of `Semiring`: { UInt8, UInt16, UInt32, UInt64, UInt128 }
3. Apply refinement filter: program bound `(0..2^32)` must be ⊆ candidate bound
   - UInt8 (0..2^8): NO — program bound exceeds
   - UInt16 (0..2^16): NO — program bound exceeds
   - UInt32 (0..2^32): YES — exact match (program bound ⊆ candidate bound)
   - UInt64 (0..2^64): YES — strict subset
   - UInt128 (0..2^128): YES — strict subset
4. Apply minimum-bound match (declared structural ordering): `UInt32` is the minimum (its bound exactly equals program bound)
5. Result: unique answer = `UInt32`

**Expected output:** emit `100u32`.

**Test claim shape:**
```
fold_dag_int_refined_to_rust_u32: TestClaim {
  setup: standard_rust_language_spec()
  source: "data count: Int(0..2^32) = 100"
  expected_emission: "100u32"
  expected_diagnostic: None
}
```

**Note:** "minimum bound" is structurally declared via bound subsumption ordering, not engine "minimum-satisfier" policy. The candidate that exactly matches the program's refinement is structurally distinguished from candidates that strictly contain it.

---

### Example 3 — `String` (top-level data binding) → Rust `String` (ownership derived from program structure)

**Demonstrates:** Modeling problem 2 (structural distinctions, not canonical choice) + Modeling problem 3 (lifetime/ownership derived from program structure, no annotations). The `.dag` `String` and Rust `String`/`Box<str>`/`&str`/`Cow<str>` are *meaningfully different* on (ownership, growability, encoding, lifetime). Modeling those differences as substrate facts + deriving program intent from program structure yields a unique answer.

**Substrate facts (must be declared):**
```
// String family — meaningfully different on multiple axes
inhabits RustString : FreeMonoid<Char>  ownership = Owned        growable = yes  encoding = UTF-8  lifetime = self
inhabits BoxedStr   : FreeMonoid<Char>  ownership = Owned        growable = no   encoding = UTF-8  lifetime = self
inhabits VecOfBytes : FreeMonoid<Byte>  ownership = Owned        growable = yes  encoding = raw    lifetime = self
inhabits BoxedBytes : FreeMonoid<Byte>  ownership = Owned        growable = no   encoding = raw    lifetime = self
inhabits StrSlice   : FreeMonoid<Char>  ownership = Borrowed     growable = n/a  encoding = UTF-8  lifetime = source
inhabits CowOfStr   : FreeMonoid<Char>  ownership = Conditional  growable = n/a  encoding = UTF-8  lifetime = conditional
```

**Program input:**
```
data name: String = "Alice"
```

**Fold steps:**
1. Read program intent: algebra = `FreeMonoid<Char>` (`.dag` `String` is UTF-8 sequence of chars); refinement = none
2. **Lifetime/escape analysis:** `name` is a top-level data binding. It has no source it could borrow from (it's constructed from a literal); it lives for the program's duration. Structural intent: `ownership = Owned`, `lifetime = self`
3. **Growability analysis:** `name` is bound to a literal once; nothing in the program graph mutates or grows it. Without growability evidence, default refinement is the *minimal* growability needed. But `growable = no` (BoxedStr) is *strictly more constrained* than `growable = yes` (RustString) — the latter subsumes the former. **Question for the substrate:** which is "minimally complete"?
4. Walk substrate inhabitants of `FreeMonoid<Char>` matching `ownership = Owned`, `lifetime = self`: { RustString (growable=yes), BoxedStr (growable=no) }
5. **Apply structural ordering on growability:** `growable = no` is structurally smaller (fewer capabilities required). The program doesn't *require* growability; `BoxedStr` is the minimally-complete answer.
6. Result: unique answer = `BoxedStr` → emit `Box<str>` ?

**Wait — this surfaces a real design call.** Is `Box<str>` really the right answer for `data name: String = "Alice"`?

Two readings of "minimally complete":
- **Strict:** the smallest target type that satisfies *exactly* what the program declares. Program declares no growth → emit non-growable. Result: `Box<str>`.
- **Pragmatic:** the smallest target type that satisfies *what the program declares* + *common future use cases that don't require additional substrate*. Result: `String` (allows growth without re-deriving).

The thesis answer is **strict**. Pragmatic is engine policy ("guess what the user might want later"). If the program doesn't need growth, emit `Box<str>`. If the user wants growth, they declare it: `data name: String + Growable = "Alice"` (or whatever the substrate refinement looks like).

**Expected output (strict reading):** `Box<str>` (a `Box::from("Alice")` or equivalent). If `Box<str>` is unfamiliar in the audience, the diagnostic explains why: "your declaration doesn't request growth; `Box<str>` is the minimal Owned UTF-8 sequence."

**Test claim shape:**
```
fold_dag_string_top_level_data_to_rust_boxed_str: TestClaim {
  setup: standard_rust_language_spec_with_string_family()
  source: "data name: String = \"Alice\""
  expected_emission_contains: "Box<str>"
  expected_diagnostic: None
}
```

**Open call surfaced by this example:** strict vs pragmatic minimally-complete. **Recommendation:** strict. The thesis discipline is "program structure determines emission" — pragmatic adds engine policy ("guess intended use"). If users need ergonomic defaults, they should be substrate-declared via refinement defaults, not engine guesses. **Director sign-off needed.**

---

### Example 4 — `String` (passed transiently to function) → Rust `&str` (ownership derived from program structure)

**Demonstrates:** the same value-shape program-structure-derived intent → different Rust target depending on lifetime/escape pattern. **No annotations.**

**Substrate facts:** as Example 3.

**Program input:**
```
data name: String = "Alice"
fn greet(n: String) -> Unit { ... }
greet(name)
```

**Fold steps for `n` parameter inside `greet`:**
1. Read program intent: algebra = `FreeMonoid<Char>`; refinement = none
2. **Lifetime/escape analysis:** `n` is a function parameter. Does `greet` store `n` past its call? Two cases:
   - **Case A:** `greet`'s body uses `n` only transiently (passed to other functions, used in expressions, not stored in any binding outliving the call). Structural intent for `n`: `ownership = Borrowed`, `lifetime = caller`
   - **Case B:** `greet`'s body stores `n` in a binding with `'static` or escapes it via return. Structural intent for `n`: `ownership = Owned`
3. (Suppose Case A applies based on `greet`'s body structure.)
4. Walk substrate inhabitants matching `ownership = Borrowed`, `lifetime = caller`: { StrSlice }
5. Result: unique answer = `StrSlice` → emit `n: &str` in `greet`'s signature

**For the call-site `greet(name)`:**
1. `name`'s lifetime extends through the `greet(name)` call. It's bound to a `&` borrow for the duration.
2. Emit: `greet(&name)` (with `name` itself emitted per Example 3)

**Expected output:**
```rust
let name: Box<str> = Box::from("Alice");  // per Example 3
greet(&name);                              // borrow for the call
fn greet(n: &str) { ... }                  // signature derived from Case A
```

**Test claim shape:**
```
fold_dag_string_function_param_transient_to_rust_strslice: TestClaim {
  setup: standard_rust_language_spec_with_string_family()
  source: """
    data name: String = "Alice"
    fn greet(n: String) -> Unit { /* transient use */ }
    greet(name)
  """
  expected_signature_emission_contains: "n: &str"
  expected_call_site_emission_contains: "greet(&name)"
  expected_diagnostic: None
}
```

**Why this works without annotations:** the fold derives ownership from the program's own structural facts (lifetime of bindings, function-body use patterns, escape analysis). Rust's borrow checker does this work in reverse (validating user's annotations); gunbc's fold does it forward (deriving the right Rust target from program structure). **The program already declares its intent through use; annotations would be parallel authority.**

**Open call:** how rich does the lifetime/escape analyzer need to be in R2? Recommendation: cover (a) top-level data bindings (Example 3), (b) function parameters with transient use (this Example), (c) function return values (must be Owned). Defer (d) closures, (e) async lifetimes, (f) Pin/self-referential to post-R3 if needed. **Director sign-off on R2 scope.**

---

### Example 5 — duplicate of Example 1; consolidated

> **Consolidated 2026-04-28:** this example previously demonstrated "no canonical declared" using a substrate variant intentionally missing a canonical fact. Per Modeling problem 2 corrected, "canonical declaration" was the wrong framing — the bound differences ARE meaningful and belong as refinements, not as candidates needing canonical disambiguation. Example 1 already demonstrates the correct fail-closed-on-under-refined case (`UnderRefined` diagnostic carrier). This example is retained as a placeholder slot to preserve example numbering through the doc; the test-case shape has migrated to Example 1.

---

### Example 6 — `Int(0..2^65)` → fail-closed `EmissionDiagnostic::NoInhabitant`

**Demonstrates:** Modeling problem 5 again, but for the "no inhabitant" case rather than "multiple without canonical."

**Substrate facts:** as in Example 1 (Rust integer family up to Int128).

**Program input:**
```
data huge: Int(0..2^65) = 36893488147419103232
```

**Fold steps:**
1. Read program intent: algebra = `OrderedRing`; refinement = `(0..2^65)`
2. Walk substrate inhabitants: { Int8, Int16, Int32, Int64, Int128 }
3. Apply refinement filter: program bound `(0..2^65)` must be ⊆ candidate bound
   - Int8 through Int64: NO — program bound exceeds (Int64 caps at 2^63)
   - Int128: YES — strict subset
4. Result: 1 candidate (Int128)
5. Result: unique answer = `Int128`

**Wait** — actually this case succeeds with `Int128`. Let me restate to demonstrate fail-closed-on-no-inhabitant:

**Restated program input (genuinely exceeds all candidates):**
```
data astronomical: Int(0..2^200) = 1
```

**Fold steps (restated):**
1. Read program intent: algebra = `OrderedRing`; refinement = `(0..2^200)`
2. Walk substrate inhabitants: { Int8, Int16, Int32, Int64, Int128 }
3. Apply refinement filter: program bound exceeds Int128's `2^127` — no candidates pass
4. Result: zero candidates → fail-closed

**Expected output:**
```
EmissionDiagnostic::NoInhabitant {
  program_intent: AlgebraInhabitance(OrderedRing, refinement: (0..2^200)),
  candidates_considered: [Int8, Int16, Int32, Int64, Int128],
  resolution_hints: [
    "Rust's largest signed integer is i128 (range -2^127..2^127); Int(0..2^200) cannot ground to a Rust primitive",
    "consider arbitrary-precision integer carrier (post-R3 substrate work)",
    "narrow the bound: Int(0..2^127) grounds to Int128"
  ]
}
```

**Test claim shape:**
```
fold_dag_int_exceeds_target_fails_closed: TestClaim {
  setup: standard_rust_language_spec()
  source: "data astronomical: Int(0..2^200) = 1"
  expected_emission: None
  expected_diagnostic: matches(EmissionDiagnostic::NoInhabitant { .. })
}
```

---

### Example 7 — `List<Int(0..2^32)>` (top-level data binding) → Rust `Box<[u32]>` (compound, recursive fold, all structurally complete)

**Demonstrates:** how the fold composes through container types when each level is structurally complete; what "minimally complete" means recursively.

**Substrate facts (must be declared):**
```
// Container family — meaningfully different on (ownership, growability, lifetime)
inhabits Vec<T>      : Container<T>  ownership = Owned     growable = yes  lifetime = self
inhabits BoxOfT      : Container<T>  ownership = Owned     growable = no   lifetime = self
inhabits SliceOfT    : Container<T>  ownership = Borrowed  growable = n/a  lifetime = source

// Plus Rust integer family from Example 2 (UInt32 inhabits Semiring at bound 0..2^32)
```

**Program input:**
```
data nums: List<Int(0..2^32)> = [1, 2, 3]
```

**Fold steps:**
1. **Outer fold (List):**
   - Read program intent: algebra = `Container<T>` where T = `Int(0..2^32)`; refinement = none on the container itself
   - Lifetime/escape analysis: top-level data binding → `ownership = Owned`, `lifetime = self`
   - Growability analysis: bound to literal once, no growth in program → minimally complete is `growable = no`
   - Walk substrate inhabitants matching: { BoxOfT (ownership = Owned, growable = no, lifetime = self) }
   - Outer-level result: `BoxOfT<T>`
2. **Recursive fold on T (= `Int(0..2^32)`):**
   - Per Example 2: refinement-driven match → `UInt32`
3. **Compose:** `BoxOfT<UInt32>` → emit `Box<[u32]>`

**Expected output:**
```rust
let nums: Box<[u32]> = Box::from([1u32, 2u32, 3u32]);
```

**Test claim shape:**
```
fold_dag_list_int_refined_top_level_to_rust_box_slice_u32: TestClaim {
  setup: standard_rust_language_spec_with_container_family()
  source: "data nums: List<Int(0..2^32)> = [1, 2, 3]"
  expected_emission_contains: ["Box<[u32]>"]
  expected_diagnostic: None
}
```

**Note 1:** the recursive fold composes both levels structurally. Each level reads its own facts, returns its result; outer composes. No engine state crosses levels.

**Note 2:** `Vec<u32>` would be a *valid but not minimally complete* answer. The program doesn't request growth; `Box<[u32]>` is structurally smaller (no growth machinery). Per Example 3's open call (strict vs pragmatic), strict reading produces `Box<[u32]>`. If the program later needs growth, it declares it: `List<Int(0..2^32)> + Growable`.

**Note 3:** if the program uses `nums` in a way that *requires* growth (e.g., `nums.push(4)` is a substrate operation that requires `growable = yes`), the lifetime/escape analyzer surfaces the required refinement upward, and the fold matches `Vec<u32>` instead. The required refinement is itself a structural fact derived from program use.

---

### Example 8 — Cross-target consistency: `Int(-2^31..2^31)` → Rust `i32` AND Python `int` AND Go `int32`

**Demonstrates:** Modeling problem 7 (cross-target uniformity); same `.dag` algebra+refinement reaches three target-language vocabularies via three independent language specs. Note: the program is *fully refined* (specific bound), so each target deterministically grounds. No canonical needed.

**Substrate facts:**

**Rust spec** (signed integer family as Example 2; Int32 inhabits OrderedRing at `(-2^31..2^31)`).

**Python spec:**
```
// Python ints are arbitrary precision — there's exactly one inhabitant
inhabits int : OrderedRing  bound = unbounded
```

(Note: Python has no integer family with bound differences. Every refinement-bounded `.dag` Int grounds to Python `int` — Python's arbitrary-precision int *strictly subsumes* every bound. This is structurally complete; not a "canonical choice." The substrate just declares one inhabitant and bound subsumption matches.)

**Go spec:**
```
inhabits int8   : OrderedRing  bound = (-2^7..2^7)
inhabits int16  : OrderedRing  bound = (-2^15..2^15)
inhabits int32  : OrderedRing  bound = (-2^31..2^31)
inhabits int64  : OrderedRing  bound = (-2^63..2^63)

// Note: Go also has architecture-dependent `int` — but architecture-dependent is
// itself a structural fact (refinement = platform-dependent). It's not "canonical";
// it's a *different algebra inhabitance* with refinement = platform. Defer use of
// `int` until lifetime/platform analysis can structurally derive its applicability.
```

**Cross-target portability meta-spec:** every Shape A target must declare at least one inhabitant of `OrderedRing` covering the bound `(-2^31..2^31)` (the most common 32-bit signed range). Verified at substrate-load time.

**Program input:**
```
data x: Int(-2^31..2^31) = 0
```

**Fold runs three times (one per target), with the same algorithm:**

- **Rust:** match candidates, refinement filter → only `Int32` exact-matches. Emit `0i32`.
- **Python:** match candidates, refinement filter → `int` matches by subsumption (program bound ⊆ unbounded). Emit `0`.
- **Go:** match candidates, refinement filter → only `int32` exact-matches. Emit `int32(0)`.

**Test claim shape:**
```
fold_dag_int_refined_cross_target_consistent: TestClaim {
  setup: rust_python_go_language_specs_with_portability_meta()
  source: "data x: Int(-2^31..2^31) = 0"
  expected_emissions: {
    rust:   contains("i32"),
    python: contains("int"),
    go:     contains("int32")
  }
  expected_diagnostic: None
}
```

**Why this works without canonical-choice:** the program is *fully refined* — it declares its bound. Each language spec independently models its inhabitance with bound parameters. Bound subsumption matches the candidate; no policy. The cross-target meta-spec only enforces that all targets *can* match (have an inhabitant covering the program's bound); it doesn't pick which one — that falls out of structure.

**Compare to under-refined Example 1:** that program (`data x: Int`) failed-closed on Rust because the bound was unspecified and Int8/Int64/Int128 are meaningfully different. On Python, the same under-refined program would *succeed* — Python has only one inhabitant, so under-refinement doesn't produce ambiguity. Cross-target consistency means the same program either grounds on all targets or fails-closed where it can't; it doesn't silently produce different semantics.

---

### What these examples collectively prove

When the 8 examples above pass as `.dag` `TestClaim` declarations:

1. **No engine** — the fold is mechanical at every step. Each step reads a declared substrate fact (inhabits / refinement / structural property / lifetime analysis result) and applies it; nothing is decided by policy.
2. **Under-refinement fails closed, not silently picked** (Examples 1, 5, 6) — when the program hasn't declared enough structural facts to uniquely determine the target, the diagnostic surfaces what's missing. There is no "canonical default" engine fallback.
3. **Refinement composes structurally** (Examples 2, 6) — bounds participate in the fold via subsumption ordering. "Minimum-satisfier" is a structural property, not an engine choice.
4. **Apparent multi-inhabitance dissolves through structural modeling** (Examples 3, 4, 7) — what looked like "multiple inhabitants needing canonical choice" was actually meaningful structural differences (ownership, growability, lifetime, encoding). Modeling those differences as substrate facts + deriving program intent from program structure yields a unique answer per use site.
5. **Program intent is derived from program structure, not annotations** (Examples 3, 4, 7) — lifetime/escape/use analysis reads the program graph and produces the structural facts the fold composes against. No annotation surface.
6. **Fail-closed has typed diagnostics with resolution hints** (Examples 1, 5, 6) — when the fold can't determine, `EmissionDiagnostic` names what would resolve (refinement to add, substrate fact to declare, or genuine "no candidate covers this case").
7. **Compound types compose recursively at every level** (Example 7) — outer level + inner level each apply structural fold; both levels independently fail-closed if either is under-refined.
8. **Cross-target works because each language spec is independent + portability meta-spec enforces "can match"** (Example 8) — three folds run independently, three results emerge from declared facts. A program that fully-refines on its own structure grounds on all targets that have an inhabitant; one that under-refines fails-closed where targets can't disambiguate.

**These are the structural test of "no separate coercion engine"** per THESIS:171. If any example required engine policy to produce its expected output, the no-engine claim would be falsified. Each example is reproducible against the substrate facts named in its setup.

### Open design calls surfaced by the examples

The reframe from "canonical choice + annotations" to "structural modeling + program-derived intent" surfaces real design calls that need Director sign-off before R2-Evaluator dispatch:

1. **Strict vs pragmatic minimally-complete** (Example 3): does `data name: String = "Alice"` emit `Box<str>` (strict — what the program declares) or `String` (pragmatic — what the program might want later)? **Recommendation: strict.** Pragmatic adds engine policy disguised as ergonomics.
2. **Lifetime/escape analyzer scope in R2** (Example 4): cover (a) top-level data bindings, (b) function parameters with transient use, (c) function return values; defer (d) closures, (e) async, (f) self-referential to post-R3 if needed. **Recommendation: a/b/c in R2; d/e/f deferred unless a Goal-1 demo requires.**
3. **Apparent multi-inhabitance audit** (general): for every case that looked like "multiple inhabitants needing canonical," re-audit per Modeling problem 2 corrected: is the difference cosmetic (collapse) or meaningful (model the structural axis)? **Recommendation: enumerate the cases as part of T-Ground-LanguageSpec lane scope; each case is a sub-task that either retracts a candidate or extends substrate refinement.**
4. **Required structural axes for Rust target** (general): the String family example surfaced (ownership, growability, encoding, lifetime). The integer family used (bound, signedness via algebra). What other axes does Rust need? Float family (precision, NaN-handling), reference family (mutability, lifetime, raw vs reference). **Recommendation: T-Ground-Rust XL lane scope explicitly enumerates the structural axes needed per Rust primitive family before substrate-population begins.**

These are the design questions the engine framing was hiding. Surfacing them as open calls means R2-Evaluator dispatch waits on real design work, not on engine implementation.

## Affected lanes (post-merge realignment)

[PR #989](https://github.com/gunb-ai/gunbc/pull/989) "T-Ground-Engine: Phase 2 pilot-list enumeration (slice 1)" merged on main (stern-ant-452 + merry-bat) before this design doc was authored. The slice-1 code on main implements the prior framing (inhabitance-search + selection + tie-breaking). **Post-merge realignment options:**

**(a)** Follow-up PR retracts selection logic + tie-breaking from slice-1 code; renames the lane (or its primary types) per `T-Ground-Coercion-Fold`; introduces `EmissionDiagnostic` carrier for under-determinism. Slice-1 code stays on main; semantics are corrected in-place.

**(b)** Hold further slices (Phase 2 slice 2+) until LanguageSpec schema lands; slice-1 code on main remains as-is until follow-up cleanup wave. Avoids further engine-framed code while LanguageSpec is designed.

**(c)** Combine: ship (b) immediately (hold further slices) and queue (a) as a follow-up cleanup PR once LanguageSpec lands and consumers can route through the new substrate.

**Recommendation: (c).** (b) is the immediate hold; (a) is the substantive cleanup that follows once LanguageSpec lands. (c) is the realistic sequencing — slice-1 stays on main but doesn't get more engine-framed siblings.

This is a Director call (cross-program coordination); see Open call 1 below.

## Open calls

### 1. Director sign-off on no-engine discipline + R2 lane restructure

**Required before:** any further engine-framed slice ships (Phase 2 slice 2+ or sibling dispatches).

**Decision needed:**
- Adopt this design (no engine; modeling problems are lanes)
- Resolve PR #989 post-merge status (recommendation: option (c) — hold further slices + queue follow-up cleanup once LanguageSpec lands)
- Approve replacement R2 lane structure (T-Ground-Engine → 5 lanes per §"Recommended R2 lane structure update")

**Ownership:** Director #828.

**Timeline:** R2-Evaluator dispatch is unrelated; can proceed. R2-T-Ground work pauses on the engine lane until decision lands.

### 2. Cascade across upstream docs

If §1 lands, the following docs need amendment:

- [`ROADMAP.md`](../ROADMAP.md) §"Post-R1 Grounding lanes" — replace T-Ground-Engine row with 5 new lanes
- [`docs/thesis/target-grounding-proposal.md`](thesis/target-grounding-proposal.md) §"Substrate work" — supersede "Coercion engine" framing; cite this doc
- [`docs/briefs/grounding-manager.md`](briefs/grounding-manager.md) — update lane list
- [`docs/r2-structure.md`](r2-structure.md) — already references this doc on land (current PR #1078)
- [`docs/r3-structure.md`](r3-structure.md) — verification lanes already align (L4-L7); inline reference to be added
- [`docs/thesis/r2-r3-thesis-mapping.md`](thesis/r2-r3-thesis-mapping.md) — coercion-related rows updated to cite this doc

**Ownership:** Director or Grounding Manager #860 (cascade-author per the cascade-promotion pattern from `docs/design-pure-bootstrap-zero.md`).

### 3. Post-R3 dogfooding decision

Modeling problem 8 (first-class language-spec emission) is post-R3. Whether it ships as part of ecosystem buildout or as an explicit later release is open.

**Ownership:** post-R3, not pre-promotion.

## Cross-refs

- Parent thesis claim: [`THESIS.md`](../THESIS.md) §"Tier 1 — Structural correctness" — Coercion = emission, no separate coercion engine
- Architectural authority: [`docs/single-emitter-design.md`](single-emitter-design.md) — coercion = emission; algebra-homomorphism-not-lookup
- Predecessor (superseded framing): [`docs/thesis/target-grounding-proposal.md`](thesis/target-grounding-proposal.md) — "Coercion engine" row at `:165` and `:344`
- ROADMAP lane: [`ROADMAP.md`](../ROADMAP.md) §"Post-R1 Grounding lanes" — `T-Ground-Engine` row to supersede
- Manager brief: [`docs/briefs/grounding-manager.md`](briefs/grounding-manager.md) — lane list to amend
- R2/R3 planning: [`docs/r2-structure.md`](r2-structure.md), [`docs/r3-structure.md`](r3-structure.md), [`docs/thesis/r2-r3-thesis-mapping.md`](thesis/r2-r3-thesis-mapping.md)
- Substrate dependencies named: DB-11 (refinement-carrying qualifiers), cardinality-substrate; DB-18 (parametric algebra attachment); E-9 (external realization on `Arrow.body`)
- INVARIANTS: [`INVARIANTS.md`](../INVARIANTS.md) §P3 Fail-Closed; §P2 Boundary Discipline; §P1 Modeling Faithfulness
