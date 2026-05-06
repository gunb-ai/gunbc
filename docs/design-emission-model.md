# Design — Emission Model (no separate coercion engine)

**Status:** `PROPOSAL` (2026-04-28). Pending Director sign-off + alignment with [`docs/thesis/target-grounding-proposal.md`](thesis/target-grounding-proposal.md).

**Authority on promotion:** [`THESIS.md`](../THESIS.md) §"Tier 1 — Structural correctness" — "**Coercion = emission: the compiler reads a target spec and translates. No separate coercion engine.**" — this doc operationalizes that thesis claim.

**Supersedes** the "Coercion engine" framing in [`docs/thesis/target-grounding-proposal.md`](thesis/target-grounding-proposal.md) §"Substrate work" and the `T-Ground-Engine` lane framing in [`ROADMAP.md`](../ROADMAP.md) and [`docs/briefs/grounding-manager.md`](briefs/grounding-manager.md). Those documents predate the cascade-promotion thesis-discipline tightening (2026-04-25); they describe the work as "engine + selection + tie-breaking" which contradicts THESIS:171.

**Affects already-merged work:** [PR #989](https://github.com/gunb-ai/gunbc/pull/989) "T-Ground-Engine: Phase 2 pilot-list enumeration (slice 1)" merged on main under the prior framing. **Actual slice-1 footprint** (per R2 Grounding Manager review 2026-04-28; verified at `src/v3/grounding_engine/src/lib.rs`): ~370 lines of structural-equality validation (a one-way mirror-consistency probe between `Dag::rust_pilot_primitives()` and the `RUST_PILOT_PRIMITIVES` Rust mirror). Failure type is `StructureMismatch`. **There is no selection logic, no inhabitance-search, no tie-breaking** to retract. **Post-merge realignment** per §"Affected lanes" below = rename + re-home (slice-1's mirror-consistency probe → T-Ground-LanguageSpec scope) + introduce typed `EmissionDiagnostic` carrier when fold consumers actually start using it; further slices (Phase 2 slice 2+) hold until LanguageSpec schema lands.

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

- **Program intent** = `.dag` algebra inhabitance + refinement bounds + program-derived structural facts (lifetime, escape, ownership inferred from binding scopes and use sites — see Modeling problem 3 corrected). **Not annotations.** Annotations were retracted as parallel authority.
- **Substrate facts** = target language specs (per `dsl/extdeps/languages/*/`); each candidate target's structural properties (ownership, lifetime, growability, encoding-invariant, etc. — declared as algebra refinements per Modeling problem 2); declared structural ordering for diagnostic enumeration
- **Structural fold** = mechanical implementation. Reads program intent, walks substrate facts, produces a result. No selection logic; no tie-breaking policy; no "minimum-satisfier" heuristic that lives in the fold itself.
- **Result** = unique target primitive (when fold structurally determines) or typed `EmissionDiagnostic` carrier naming what would resolve the under-determinism (when fold cannot)

The fold is small and mechanical because all the *real work* is in the substrate facts. Anything the fold has to "decide" is a fact the substrate should have declared.

## What the no-engine discipline forces us to model

This is the load-bearing section. The "engine" framing was hiding work — work that is *real* and *hard* and now becomes visible as modeling problems we have to think through carefully. Each is a lane of substrate completion that previously was implicit in "the engine will figure it out."

### Modeling problem 1 — refinement composition with algebra inhabitance

**Question:** how does a refinement bound participate in inhabitance search?

**Worked example.** A `.dag` program declares `count: Int(0..2^32)`. The intent is "non-negative 32-bit integer." Rust offers `u8`, `u16`, `u32`, `u64`, `u128`, `usize`. Each inhabits `Semiring<WordN>` (no negatives → no `OrderedRing`). The fold must determine: which target inhabits `Semiring` *at the refinement bound* `(0..2^32)`?

The honest answer is: `u32` is the unique inhabitant of `Semiring` at exactly the program's refinement `(0..2^32)`. The other candidates are *different inhabitances* with *different refinement bounds* — not "wider valid candidates" the fold could fall back to:

- `u8` / `u16` inhabit at *narrower* refinements `(0..2^8)` / `(0..2^16)` — overflow at the program's bound; rejected by the structural inhabitance check, not by ordering
- `u64` / `u128` inhabit at *wider* refinements `(0..2^64)` / `(0..2^128)` — these are **different inhabitance facts**, not "acceptable wider" emission candidates. The fold's emission predicate is exact-bound match, so `u64` is **not** a candidate for `Int(0..2^32)`. It IS a *diagnostic alternative*: if the program author wants `u64`, they declare `Int(0..2^64)` instead. Ordering is for surfacing this alternative in the diagnostic, never for picking an emission target.
- `usize` is platform-dependent — its bound is not a fixed `0..2^N`; it's a different inhabitance shape (architecture-dependent), separately diagnostic-surfaced.

**What the substrate must declare** for this to be a structural fold:
- Each target primitive declares its `Semiring` inhabitance with a *cardinality bound* attached (`u32` inhabits `Semiring<Word32>` at bound `0..2^32`; not just "inhabits Semiring")
- The fold matches program refinement against target refinement *within the same algebra*
- The fold uses **exact-bound match**: `u32`'s bound exactly equals the program's `(0..2^32)` refinement; `u64`'s bound `(0..2^64)` is a *different bound* (different inhabitance), not a "wider valid" candidate. No subsumption used for emission.

**What the substrate cannot do today.** ROADMAP names DB-11 (refinement-carrying qualifiers on primitives) + cardinality-substrate (container cardinality bounds) as substrate prerequisites for `T-Ground-Rust`. That's exactly this: the substrate doesn't yet carry refinement-attached algebra inhabitance.

**The work.** Extending the substrate so each declared inhabitance carries its refinement bound. This is real modeling work. It's NOT "the engine handles refinement"; it's "the substrate declares refinement-bounded inhabitance."

### Modeling problem 2 — surfacing structural differences instead of canonical choice

**Question:** when the substrate appears to declare multiple primitives that inhabit the same algebra at the same refinement, what's *actually* different between them — and is that difference cosmetic or meaningful?

**The honest framing (corrected 2026-04-28 per user direction):** "multiple inhabitants at the same algebra+refinement" is itself a smell. Either the candidates are *structurally equivalent* (cosmetic — same thing under different names) and they collapse into one, or they're *meaningfully different* (different semantic invariants — ownership, lifetime, mutability, encoding) and the difference belongs in the substrate as additional structural facts. There is no third category requiring "canonical choice" engine machinery.

**Worked example.** When the program writes `.dag` `String` (semantically: a sequence of unicode chars), what Rust types are candidates? The answer depends on which algebra the program intends.

**Algebra distinction first** (corrected 2026-04-28 per user clarification):

| Algebra | Semantic | Rust candidates |
|---|---|---|
| `FreeMonoid<Char>` | sequence of unicode chars (UTF-8 by Rust's definition of `str`) | `String`, `Box<str>`, `&str`, `Cow<str>` |
| `FreeMonoid<Byte>` | sequence of raw bytes (no UTF-8 invariant) | `Vec<u8>`, `Box<[u8]>` |

So `Vec<u8>` is **not** a candidate for `.dag` `String` — it inhabits a *different algebra*. The algebra choice (FreeMonoid<Char> vs FreeMonoid<Byte>) carries the encoding distinction structurally; encoding is not a separate refinement axis.

**Within `FreeMonoid<Char>`** (the algebra that `.dag` `String` typically inhabits), the candidates differ on three structural axes:

| Candidate | Owned? | Growable? | Lifetime |
|---|---|---|---|
| `String` | yes | yes | self-contained |
| `Box<str>` | yes | no | self-contained |
| `&str` | no | n/a | borrowed |
| `Cow<str>` | conditional | conditional | conditional |

These are **meaningfully different** on three structural dimensions: ownership, growability, lifetime. Modeling these as substrate refinements (or as additional algebras) means each combination of program structural intent maps to a unique target. **No canonical needed** — the candidates inhabit *different* algebra+refinement combinations.

**The work.** Surface every meaningful difference between apparent multi-inhabitants as a structural fact:
- The algebra choice (FreeMonoid<Char> vs FreeMonoid<Byte>) carries the encoding distinction structurally — UTF-8 vs raw bytes is the algebra, not a refinement axis
- Add ownership as a refinement on `FreeMonoid<Char>` (or a separate algebra `OwnedFreeMonoid<Char>` vs `BorrowedFreeMonoid<Char>`)
- Add growability as a refinement (or as the distinction `FreeMonoid` (growable) vs `Sequence<N>` (fixed-size))
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

**Open question:** does this analyzer live in the Evaluator program (post-R3), or is it a substrate-completion lane in R2 alongside LanguageSpec? **Recommendation:** R2 lane. The fold needs lifetime analysis to determine target type for any program with non-trivial scoping. Defer is dishonest — the worked examples (String/&str/Cow) require it from day one. Lane name suggestion: **T-Ground-Lifetime-Analyzer**.

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

### Modeling problem 8 — cost lens over emission must be structural composition (not a separate dimension)

**Load-bearing claim** (per user direction 2026-04-28): the cost lens should be **FREE for coercion**. If it isn't, that's a structural modeling gap — analyze it, don't paper over it with a separate "coercion cost" dimension.

**Why this is a thesis-faithfulness test, not a feature request.** THESIS already commits to two unifications:
1. **"Coercion = emission"** (THESIS:171, 186) — emission *is* coercion; not a separate phase
2. **"Coercion cost = complexity"** (THESIS:185) — coercion's cost *is* the complexity dimension

Composing these: emission's cost = coercion's cost = complexity. So the cost lens applied to a program-with-emission target should *automatically* include realization cost. **No new lens, no new dimension, no per-target cost lookup table** — just the existing complexity lens reading structural facts that include target-language-spec-declared per-primitive costs.

If the cost lens *cannot* analyze coercion for free, exactly one of three gaps exists:

| Gap | Failure mode | What it implies |
|---|---|---|
| **(a) Cost lens doesn't read target-side facts** | Cost lens currently analyzes `.dag` programs, not target-emitted programs. Per-primitive realization cost (e.g., `u32.add` = O(1), `BigInt.add` = O(n)) isn't in the lens's input set | Modeling gap — wire cost lens to read language-spec realization-cost declarations |
| **(b) Cost lens has its own per-target table** | The lens duplicates target language spec by maintaining its own per-target cost facts, parallel to the language spec's primitive declarations | P2 violation — parallel authority. Cost facts live in language spec; lens consumes them. No duplicate table |
| **(c) "Coercion = emission" is reviewer-convention, not structure** | The unification is asserted but not held by construction; emission proceeds via paths the cost lens can't structurally see | Thesis-faithfulness gap — emission must produce a *cost-analyzable* artifact for the unification to hold by construction |

**Required substrate facts for the unification to be structurally true:**

1. **Algebra-level cost** — `dsl/std/algebra.dag` declares per-operation cost shape (e.g., `OrderedRing.add` has cost `O(1)` symbolic; `Vec.append` has cost `O(amortized 1)`)
2. **Target-primitive realization cost** — language spec (`dsl/extdeps/languages/{rust,python,go}/`) declares per-primitive cost shape per operation (e.g., Rust's `u32.add` = O(1) word op; Rust's `BigInt.add` = O(digit_count))
3. **Composition rule** — cost lens reads (1) program's algebra-level cost + (2) emitted target's per-primitive cost via the language spec → produces total realization cost. Composition is structural fold, not engine policy.

**If all three are declared facts, the cost lens is free for coercion by construction.** No additional work needed beyond reading the substrate.

### Worked examples — cost lens applied to emitted target

**Setup:** assume the substrate facts above are declared. Cost lens runs the same way for both examples; only the input refinement differs.

#### Example A — `Int(0..2^32) + Int(0..2^32)` → Rust `u32 + u32`

**Program input:**
```
data x: Int(0..2^32) = 100
data y: Int(0..2^32) = 50
data z: Int(0..2^32) = x + y
```

**Cost-lens fold:**
1. Algebra cost (from `algebra.dag`): `Semiring.add` carries cost shape `O(1)` symbolic
2. Emission resolves `x + y` to Rust `u32 + u32` (per Example 2)
3. Target realization cost (from language spec): Rust `u32.add` = O(1) word operation
4. **Composed**: O(1) algebra × O(1) realization = **O(1) total**

**Cost lens output:** `CostExpr(work=1, span=1, asymptotic_class=O(1))`. Reading: this addition is constant-time per word. No engine policy; just composition of declared facts.

#### Example B — same program emitted as `BigInt + BigInt` (hypothetical, post-R3 substrate)

If the user widened the bound to require `BigInt`:
```
data x: Int(0..2^512) = 100
data y: Int(0..2^512) = 50
data z: Int(0..2^512) = x + y
```

**Cost-lens fold:**
1. Algebra cost: same — `Semiring.add` symbolic shape O(1)
2. Emission resolves to Rust `BigInt + BigInt` (post-R3, when arbitrary-precision substrate lands)
3. Target realization cost (declared on Rust language spec for BigInt): `BigInt.add` = O(digit_count)
4. **Composed**: O(1) algebra × O(digit_count) realization = **O(digit_count) total**

**Cost lens output:** `CostExpr(work=O(digit_count), span=O(digit_count), asymptotic_class=O(n))`. Same program, structurally larger bound, structurally larger cost. **The compiler doesn't choose** — the user's program structure (the bound they declared) determines the target, and the cost lens reports the consequence.

#### Example C — coercion across types in one expression

```
data n: Int(0..2^32) = 100
data total: Int(0..2^64) = n + n
```

**Cost-lens fold:**
1. The expression `n + n` has program-side type `Int(0..2^64)` (program declared); inputs are `Int(0..2^32)`
2. Emission emits *some* coercion path: `n: u32 → u64; n: u32 → u64; u64 + u64`
3. Algebra cost: `Semiring.add` = O(1)
4. Target realization cost:
   - `u32 → u64` widen: O(1) (declared in language spec; no cost for word-widening on 64-bit)
   - `u64.add`: O(1)
5. **Composed**: O(1) widening × 2 + O(1) add = **O(1) total**

**Coercion cost is not extra.** It's just the cost of the widening operations declared in the language spec, composed structurally with the algebra cost. **No "coercion dimension" — the cost lens already analyzes coercion because coercion IS the widening operations declared on target primitives.**

### Where the gaps actually are today

Per the lens-capability register and v3 lens audit (2026-04-21):
- **`src/v3/lenses/complexity.dag`** — currently produces a single integer depth per port. Does NOT yet read target-side per-primitive cost facts. **Gap (a)** is real.
- **`src/v3/lenses/cost.dag`** — PROXY. No named size variables; Dimension wiring deferred. Does NOT yet compose with target realization. **Gap (a) + gap (c)**.
- **Language specs (`dsl/extdeps/languages/*/`)** — declare primitive types but do NOT yet declare per-primitive cost shapes. **Substrate fact (2) above is missing.**
- **§6a `MethodContract`** — DOES declare per-method cost shape (Goal 5 R2 receipt). This is the start of substrate fact (1) for method-call costs, but not yet generalized to all algebra operations.

**Current assessment:** the unification "coercion cost = complexity" is asserted but **not held by construction**. The cost lens is *not* free for coercion today; it would require:
- Generalizing per-method cost (§6a `MethodContract`) to per-operation cost on every algebra in `algebra.dag`
- Adding per-primitive realization-cost declarations to language specs
- Wiring the cost lens to compose `.dag`-algebra cost + target-realization cost via the language spec

**This is real substrate work**, not feature work. It belongs in R2 + R3:

| Substrate completion task | Lane home | Sizing |
|---|---|---|
| Per-operation cost on every algebra in `algebra.dag` | R2-T-Substrate / extends §6a `MethodContract` pattern | M |
| Per-primitive realization-cost declarations on language specs | R2-T-Ground-LanguageSpec (folds into existing scope per Modeling problem 6) | S (per target; ~3 targets) |
| Cost lens composition rule (`.dag` algebra cost × target realization cost) | R3-T-CostLens-Composition (new lane; consumes both above) | M |
| Verify "coercion cost = complexity" holds by construction | R3-T-Verification-L4-L7-Direct extension OR new structural acceptance gate | S |

**The R3 lane (T-CostLens-Composition) is post-R2-Evaluator** because the verification needs the Evaluator to construct cost witnesses at runtime. Adding it to R3 keeps the consequence-layer framing — once R2 lands the substrate facts (algebra cost + target realization cost), R3 lands the lens composition + verification.

### Recommendation: add T-CostLens-Composition to R3

Per the dispositions above, the unification "coercion cost = complexity" needs:
- R2: substrate facts (algebra cost shapes + target realization costs)
- R3: composition lane (`T-CostLens-Composition`)

**Proposed addition to r3-structure.md lane structure:**

| Lane | Size | Manager | Covers | R2-close dependency |
|---|---|---|---|---|
| **T-CostLens-Composition** | M | **Verification Manager** (or new Cost Manager) | Cost lens reads (1) algebra-level cost + (2) target-primitive realization cost via language spec; composes structurally; verifies "coercion cost = complexity" holds by construction. **No "coercion cost" dimension** — falls out of existing complexity lens reading substrate facts | R2-Evaluator (witness construction) + R2-T-Ground-LanguageSpec (target realization-cost declarations) + R2 algebra-cost extension |

This makes the cost lens free for coercion by *substrate construction* — exactly what the user direction said it should be.

**DECISION (locked 2026-04-28 per user direction):** **R3.** T-CostLens-Composition lands in R3 (lane 10 of 10 in the updated `docs/r3-structure.md`). Deferring would leave "coercion cost = complexity" asserted-not-structural, falsifying the thesis claim by construction.

### Modeling problem 9 — first-class language-spec emission (dogfooding)

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
| 3. ~~User annotation as program substrate~~ → **Structural derivation of program intent** (no annotations) | **NEW LANE** — `T-Ground-Lifetime-Analyzer`: derives ownership / lifetime / growability from program structure (bindings, signatures, escape) | M |
| 4. Declared structural ordering | T-Substrate (declared ordering on cardinality bounds; declared enumeration order on alternates) | Folds into existing |
| 5. Fail-closed diagnostic surface | **NEW LANE** — `EmissionDiagnostic` carrier substrate | S |
| 6. Language spec as substrate | **NEW LANE** — `LanguageSpec` schema authoring + per-target population | M (was hidden as part of "T-Ground-Engine") |
| 7. Cross-target uniformity meta-spec | **NEW LANE** — cross-target portability requirements | S |
| 8. Cost lens over emission (structural composition) | R2 substrate facts (algebra cost on `algebra.dag` + per-primitive realization cost on language specs) + **R3 lane T-CostLens-Composition** for the lens fold + structural verification that "coercion cost = complexity" holds by construction | M (R3) + S (R2 substrate extensions) |
| 9. First-class language-spec emission | **POST-R3** — dogfooding | not in R2/R3 |

What remains as a "fold lane":
- **T-Ground-Coercion-Fold** (rename of T-Ground-Engine) — the mechanical implementation that reads declared facts and returns unique answer or `EmissionDiagnostic`. **S size, not M.** Most of the work was in the modeling problems above; the fold itself is small.

### Recommended R2 lane structure update

Replace `T-Ground-Engine` with five lanes:

| Lane | Size | Owns |
|---|---|---|
| **T-Ground-Coercion-Fold** | S | The mechanical structural fold. Reads program intent + substrate facts; returns unique target or `EmissionDiagnostic`. Replaces `T-Ground-Engine` framing |
| **T-Ground-LanguageSpec** | M | `LanguageSpec` schema authoring + per-target population + dissolve of existing `coercion.dag` table-driven shape via T-Ground-Dissolve |
| **T-Ground-Lifetime-Analyzer** | M | Structural derivation of program intent (ownership / lifetime / growability / encoding) from program use — bindings, function signatures, escape analysis. Replaces the retracted `T-Ground-Annotation` lane (annotations rejected per Modeling problem 3 corrected — intent must derive structurally from program, not from a parallel annotation substrate) |
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

R3 keeps the verification lanes that *prove* the no-engine claim — split per Codex Pattern B finding 2026-04-28 into `T-Verification-L4-L7-Direct` (Evaluator-direct: L4 + L7) and `T-Verification-L5-Corpus` (corpus-driven: L5 only):

- L4 (`l4_emit_eval_match`): for every program, emitted target output equals `.dag` evaluation. **If the fold fabricates a target choice that .dag doesn't evaluate to, L4 fails — verifies the no-engine claim by execution.**
- L5 (`l5_cross_target_consistency`): emitted Rust/Python/Go produce equivalent runtime behavior. **Any engine policy that resolves inconsistently across targets would fail L5.**
- L6 (`l6_structural_form_coverage`) — **moved to R2's T-Ground-CrossTarget-Meta lane** as a structural cross-product fold (per gpt-5-5-pro Pattern B finding 2026-04-28). Not part of the R3 verification surface; the R3 surface is {L4, L5, L7}. The R2 structural fold walks `(6 connectives × 5 behaviors × cardinality) × Shape A targets` and verifies each pair has an emission path declared.
- L7 (`l7_algebraic_laws_witnessed`): every algebra has runtime-constructed witnesses. **Verifies that algebra inhabitance is declared and structural, not engine-policy-asserted.**

These four together are the structural test of the no-engine discipline.

## Worked examples

These are concrete walkthroughs of the structural fold for representative cases. Each example includes the substrate facts it depends on, the program input, the fold steps, and the expected output (target code OR `EmissionDiagnostic`). **These serve as test cases** — the structure is intentionally reproducible: each example can be lifted into a `.dag` `TestClaim` once the substrate lanes (T-Ground-LanguageSpec + T-Ground-Lifetime-Analyzer + T-Ground-Diagnostic + T-Ground-CrossTarget-Meta) land.

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

**Demonstrates:** refinement composition with algebra inhabitance (Modeling problem 1); **exact-bound matching** is the structural emission predicate (corrected 2026-04-28 per gpt-5-5-pro Pattern B finding — subsumption + minimum-selection was contradicting Modeling problem 4's "ordering is diagnostic-only" framing).

**Substrate facts (must be declared):**
```
inhabits UInt8   : Semiring  bound = (0..2^8)
inhabits UInt16  : Semiring  bound = (0..2^16)
inhabits UInt32  : Semiring  bound = (0..2^32)
inhabits UInt64  : Semiring  bound = (0..2^64)
inhabits UInt128 : Semiring  bound = (0..2^128)

// Note: per Modeling problem 4 corrected, structural ordering on bounds is
// DIAGNOSTIC-ONLY (used for enumerating alternatives in error messages); the
// fold uses exact-bound match for emission, not subsumption + minimum-pick.
```

**Program input:**
```
data count: Int(0..2^32) = 100
```

**Fold steps:**
1. Read program intent: algebra = `Semiring` (no negatives → not `OrderedRing`); refinement bound = `(0..2^32)`
2. Walk substrate inhabitants of `Semiring`: { UInt8, UInt16, UInt32, UInt64, UInt128 }
3. Apply **exact-bound** match (per Modeling problem 4 corrected: ordering is diagnostic-only; the fold does not consult ordering for emission):
   - UInt8 (0..2^8): NO — bound differs
   - UInt16 (0..2^16): NO — bound differs
   - UInt32 (0..2^32): YES — bound exactly matches program refinement
   - UInt64 (0..2^64): NO — bound differs (a different inhabitance, not a "wider valid" candidate)
   - UInt128 (0..2^128): NO — bound differs
4. Result: unique answer = `UInt32` (exactly one match by structural-bound predicate; no ordering consulted)

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

**Note on bound matching (corrected 2026-04-28 per gpt-5-5-pro Pattern B finding):** the fold uses *exact* refinement matching, not bound-subsumption + minimum-selection. UInt64's bound `(0..2^64)` is not "a valid wider candidate" for `Int(0..2^32)`; it's a *different inhabitance* with a *different bound*. If the program writes a refinement bound that no candidate exactly matches (e.g., `Int(0..1000)`), the fold fails-closed with a diagnostic naming the nearest declared candidates (e.g., "narrow to `Int(0..2^16)` for UInt16; widen to `Int(0..2^32)` for UInt32"). This is consistent with Modeling problem 4 corrected: ordering is diagnostic-only; the fold's emission predicate is exact match. Removing min-selection from the emission predicate dissolves the contradiction the prior framing introduced.

**Note (corrected 2026-04-28):** the original phrasing here used "minimum bound" / "subsumption ordering," which contradicted Modeling problem 4's "ordering is diagnostic-only" framing. The corrected fold uses **exact-bound match** as the structural predicate. UInt64 / UInt128 are different inhabitances with different bounds, not "wider valid candidates." See the §"Note on bound matching" callout above.

---

### Example 3 — `String` (top-level data binding) → Rust `Box<str>` (ownership + non-growability derived from program structure)

**Demonstrates:** Modeling problem 2 (structural distinctions, not canonical choice) + Modeling problem 3 (lifetime/ownership derived from program structure, no annotations). The `.dag` `String` and Rust `String`/`Box<str>`/`&str`/`Cow<str>` are *meaningfully different* on (ownership, growability, encoding, lifetime). Modeling those differences as substrate facts + deriving program intent from program structure yields a unique answer.

**Substrate facts (must be declared):**
```
// String family — meaningfully different on multiple axes
// Algebra carries encoding structurally — FreeMonoid<Char> is UTF-8 (Rust's
// definition of `str`); FreeMonoid<Byte> is raw bytes. No "encoding" refinement
// axis; encoding IS the algebra.

inhabits RustString : FreeMonoid<Char>  ownership = Owned        growable = yes  lifetime = self
inhabits BoxedStr   : FreeMonoid<Char>  ownership = Owned        growable = no   lifetime = self
inhabits StrSlice   : FreeMonoid<Char>  ownership = Borrowed     growable = n/a  lifetime = source
inhabits CowOfStr   : FreeMonoid<Char>  ownership = Conditional  growable = n/a  lifetime = conditional

// Different algebra (FreeMonoid<Byte>) — NOT candidates for `.dag` String;
// candidates only when program intent is FreeMonoid<Byte>:
inhabits VecOfBytes : FreeMonoid<Byte>  ownership = Owned        growable = yes  lifetime = self
inhabits BoxedBytes : FreeMonoid<Byte>  ownership = Owned        growable = no   lifetime = self
```

**Program input:**
```
data name: String = "Alice"
```

**Fold steps:**
1. Read program intent: algebra = `FreeMonoid<Char>` (`.dag` `String` is UTF-8 sequence of chars); refinement = none
2. **Lifetime/escape analysis:** `name` is a top-level data binding. It has no source it could borrow from (it's constructed from a literal); it lives for the program's duration. Structural intent: `ownership = Owned`, `lifetime = self`
3. **Growability analysis (structural derivation from program use):** scan all use sites of `name` in the program. Are there any mutation/growth calls (`.push`, `.append`, etc.)? Assuming the program is fully shown (no growth calls): structurally `growable = no`. **This is structural derivation from program facts, not ordering-as-emission policy** — the fold reads the program's actual use sites; absence of growth-mutating calls is a structural fact, not a default-canonical engine pick.
4. Walk substrate inhabitants matching `ownership = Owned`, `lifetime = self`, `growable = no`: { BoxedStr } (RustString has `growable = yes` — different inhabitance with different structural axis; not a candidate when the program structurally has `growable = no`)
5. Result: unique answer = `BoxedStr` → emit `Box<str>`

**Note on growability derivation (corrected 2026-04-28 per gpt-5-5-pro Pattern B finding):** the prior framing said "apply structural ordering on growability — `growable = no` is structurally smaller." That contradicted Modeling problem 4 corrected (ordering is diagnostic-only). The correction: growability is structurally derived from program use, not selected by ordering. RustString and BoxedStr are *different inhabitances* on the growability axis, just like UInt32 and UInt64 are different inhabitances on the bound axis. A program with no growth calls structurally has `growable = no`; the fold matches BoxedStr exactly.

**Open caveat:** if the program uses `name` but the fold cannot determine from program structure whether growth is required (e.g., complex closure / dynamic dispatch / something the lifetime analyzer can't see through), the fold fails-closed with `EmissionDiagnostic::UnderRefined { axis: "growability" }` and a resolution hint. The discipline is the same as the lifetime axis: derive structurally where possible; fail-closed where not. **No ordering used for emission.**

**Surfaces a real design call.** Is `Box<str>` really the right answer for `data name: String = "Alice"`?

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

**DECISION (locked 2026-04-28 per user direction): strict.** The thesis discipline is "program structure determines emission" — pragmatic adds engine policy ("guess intended use"). If users need ergonomic defaults, they should be substrate-declared via refinement defaults, not engine guesses. So `data name: String = "Alice"` emits `Box<str>` if no use site requires growth.

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

**Lifetime/escape analyzer scope** (Director-locked 2026-04-28; see §"Open design calls surfaced by the examples" item 2 for the canonical record): R2 covers (a) top-level data bindings (Example 3), (b) function parameters with transient use (this Example), (c) function return values (must be Owned). **(d) closures, (e) async lifetimes, (f) Pin/self-referential land in R3** — folded into `T-LensProducer-Retirement` (the lifetime analyzer is structurally what replaces `lens_apply.rs`'s reflection work). Per codex BLOCKING on `b2107ab0`: this callout previously said "defer to post-R3 if needed" which contradicted the locked R3 decision — single-authority resolved by aligning the callout with the locked decision and pointing at the canonical record.

---

### Example 5 — `Int(0..2^32)` for an under-modeled algebra (signedness ambiguity)

**Demonstrates:** fail-closed when program intent under-determines the algebra (not just the refinement). Different from Example 1's under-refined-bound case.

**Substrate facts:** as Example 2.

**Program input:**
```
data x: Int = 0
```

without algebra annotation (in a hypothetical substrate where `Int` is a type-alias spanning both `OrderedRing` and `Semiring`):

**Fold steps:**
1. Read program intent: type `Int` resolves to algebra = ?; the program hasn't structurally determined whether negative values are part of the value space
2. Walk inhabitants: { OrderedRing inhabitants × Semiring inhabitants } — different algebras
3. The fold cannot determine which algebra the program intends
4. Result: fail-closed → `EmissionDiagnostic::UnderRefined { axis: "algebra" }`

**Expected output:**
```
EmissionDiagnostic::UnderRefined {
  program_intent: AlgebraInhabitance(algebra: ambiguous, refinement: (0..2^32)),
  unspecified_axis: "algebra (signedness)",
  resolution_hints: [
    "use Word32 or UInt32 to declare unsigned (Semiring) intent",
    "use Int32 to declare signed (OrderedRing) intent",
  ]
}
```

**Test claim shape:**
```
fold_dag_int_ambiguous_algebra_fails_closed: TestClaim {
  setup: rust_language_spec_with_int_alias_spanning_both_algebras()
  source: "data x: Int = 0"
  expected_emission: None
  expected_diagnostic: matches(EmissionDiagnostic::UnderRefined { unspecified_axis: "algebra", .. })
}
```

This is a different fail-closed shape from Example 1 (under-specified bound) and Example 6 (no inhabitant). Example 1 is "algebra known, bound missing"; this is "algebra ambiguous"; Example 6 is "bound known, no candidate covers it." All three are typed `EmissionDiagnostic` variants.

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
3. Apply **exact-bound match** (per Modeling problem 4 corrected): no candidate has bound exactly `(0..2^65)`. Int128 has bound `(-2^127..2^127)` — different bound, different inhabitance.
4. Result: zero candidates → fail-closed (no candidate exactly matches the declared refinement)

**Restated program input (genuinely exceeds all candidates' bound widths):**
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
    "narrow to a candidate bound: Int(-2^127..2^127) grounds to Int128 (exact match required, not subsumption)"
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
   - Growability analysis (structural derivation from program use): scan all use sites of `nums`; no `.push` / mutation calls in the program → structurally `growable = no` (program-derived structural fact, not ordering-as-emission policy per Modeling problem 4 corrected)
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
// Python ints are arbitrary precision — declared structurally as Unbounded
// (the Interval<Int> Unbounded variant), not as "no bound parameter" (which
// would be a missing structural fact).
inhabits int : OrderedRing  bound = StaticBound(Unbounded)
```

The fold uses a **single structural predicate** for bound matching across all targets — match-by-`BoundDeclaration`, where `BoundDeclaration` is a sum type at the substrate level **wrapping `Interval<Int>` for the static case** and adding `PlatformDependent` as a distinct kind (not an interval — it resolves to an interval at target-platform time):

```
type BoundDeclaration
  = StaticBound(Interval<Int>)               // Compile-time-known interval; Interval<Int> further has variants ExactInterval { lo, hi } and Unbounded
  | PlatformDependent                         // Target-platform-determined; not an interval — resolves at target-platform time
```

`Interval<Int>` (per Q1 consolidation) carries the value-domain interval shape — `ExactInterval { lo, hi }` for `i32`/`u32`/`int32`/etc.; `Unbounded` for Python int / arbitrary-precision integers. `PlatformDependent` is a SEPARATE kind because its actual interval depends on which target platform is being emitted to (Rust `usize` is `[0, 2^N)` where N is platform-determined; Go `int` similarly).

(Per codex BLOCKING finding on `c98981634`: the prior "algebra-uniqueness when no bound parameter exists" framing was a target-specific *second* emission predicate. Per codex BLOCKING finding 2026-04-28T17:06: the prior `BoundDeclaration = Interval<Int>` claim split substrate authority because `PlatformDependent` isn't an interval. The corrected predicate is uniform — every target's inhabitance declares its `BoundDeclaration` structurally; the fold's single match predicate handles `StaticBound(...)` via Interval<Int> variant matching and `PlatformDependent` via kind-only matching.)

**Important:** `StaticBound(Unbounded)` does NOT match an *under-refined* program (one that declares no bound at all, like `data x: Int`). The program must declare its bound explicitly — even if that bound is "any" (`data x: Int(any)` parses to `StaticBound(Unbounded)` on the program side). Implicit-bound programs fail-closed uniformly across all targets, matching cross-target consistency: a program that doesn't declare its structural intent is incomplete, regardless of which target the fold runs on.

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

**Cross-target portability meta-spec:** every Shape A target must declare at least one inhabitant of `OrderedRing` whose `BoundDeclaration` *structurally matches* the program's bound `(-2^31..2^31)` — i.e., either a `StaticBound(ExactInterval(lo, hi))` with matching range, or a `StaticBound(Unbounded)`. Verified at substrate-load time using the same single match predicate the fold uses (no separate "covering" relation).

**Program input:**
```
data x: Int(-2^31..2^31) = 0
```

**Fold runs three times (one per target), with the **same algorithm** — one structural match on `BoundDeclaration`:**

- **Rust:** candidates filtered by `BoundDeclaration` match against program's `StaticBound(ExactInterval(-2^31, 2^31))`. `Int32`'s `StaticBound(ExactInterval(-2^31, 2^31))` matches; `Int8/16/64/128` are `StaticBound(ExactInterval(...))` at different ranges (no match); `usize` is `PlatformDependent` (no match — different kind). Emit `0i32`.
- **Python:** candidates filtered by the same predicate. Python `int`'s `StaticBound(Unbounded)` matches an explicit program-side `StaticBound(...)` (the Interval<Int> Unbounded variant matches any explicit interval — declared property of the inhabitance, not subsumption fallback). Emit `0`.
- **Go:** candidates filtered by the same predicate. `int32`'s `StaticBound(ExactInterval(-2^31, 2^31))` matches; `int8/16/64` are `StaticBound(ExactInterval(...))` at different ranges (no match); `int` is `PlatformDependent` (no match — different kind). Emit `int32(0)`.

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

**Why this works without canonical-choice:** the program is *fully refined* — it declares its bound. Each target's inhabitance declares its `BoundDeclaration` structurally (`StaticBound(ExactInterval(...))` for Rust int32 / Go int32; `StaticBound(Unbounded)` for Python int; `PlatformDependent` for usize / Go int). The fold uses a **single structural predicate** — `match(program.bound, target.bound)` — with these explicit rules (per codex BLOCKING `dfc4bc382` re P2/P3 single-authority for exact-vs-unbounded matching):

- **Target = `StaticBound(Unbounded)`** (e.g., Python int): matches any program-side `StaticBound(*)` — both ExactInterval and Unbounded. The target's Unbounded is the *universal-accept* declaration: an arbitrary-precision target can hold values of any declared range.
- **Target = `StaticBound(ExactInterval(lo, hi))`** (e.g., Rust int32, Go int32): matches program's `StaticBound(ExactInterval(lo', hi'))` iff `lo == lo' AND hi == hi'` (exact range equality). Does NOT match program's `StaticBound(Unbounded)` — a fixed-bound target cannot hold arbitrary-precision values.
- **Target = `PlatformDependent`** (e.g., usize, Go int): matches program's `PlatformDependent` only — kind-only match. The actual interval is determined at target-platform resolution time.

The match is **asymmetric** — target's Unbounded universally accepts; program's Unbounded only matches target's Unbounded. This asymmetry is structurally honest: the target's bound declaration says "what I can hold"; the program's bound declaration says "what I require." Universal-accept on the target side captures "Python int holds anything"; universal-require on the program side (`Int(any)`) demands a target that holds anything (which is only Python int's `StaticBound(Unbounded)`, not Rust int32's `StaticBound(ExactInterval(...))`).

No "exact-bound when parameterized, algebra-uniqueness when not" dual-predicate fallback (per codex BLOCKING on `c98981634`); no substrate split between Interval-shaped bounds and platform-dependent bounds (per codex BLOCKING 2026-04-28T17:06); single match predicate handles both Unbounded-universal-accept and ExactInterval-exact-equality (per codex BLOCKING 2026-04-28T18:04). The cross-target meta-spec uses the same single predicate at substrate-load time.

**Compare to under-refined Example 1:** that program (`data x: Int` — no bound declared at all) fails-closed *uniformly across all targets*. On Rust, no `BoundDeclaration` matches an unspecified-bound program (every Rust int's `StaticBound(ExactInterval(...))` demands an explicit program range). On Python, the same: `StaticBound(Unbounded)` matches *explicit* program bounds (including `StaticBound(Unbounded)` from `Int(any)`), not the absence of a bound declaration. On Go, same as Rust. Cross-target consistency holds: the same program either grounds on all targets (when fully refined, like Example 8's `Int(-2^31..2^31)`) or fails-closed on all (when under-refined, like Example 1's `Int`). To express "any bound is fine" for Python int, the program writes `data x: Int(any)` — explicit declaration replaces implicit defaults, matching the no-engine discipline.

---

### What these examples collectively prove

When the 8 examples above pass as `.dag` `TestClaim` declarations:

1. **No engine** — the fold is mechanical at every step. Each step reads a declared substrate fact (inhabits / refinement / structural property / lifetime analysis result) and applies it; nothing is decided by policy.
2. **Under-refinement fails closed, not silently picked** (Examples 1, 5, 6) — when the program hasn't declared enough structural facts to uniquely determine the target, the diagnostic surfaces what's missing. There is no "canonical default" engine fallback.
3. **Refinement composes structurally with exact-bound match** (Examples 2, 6) — bounds participate in the fold as exact match (program refinement = candidate refinement). Bound subsumption + minimum-selection is *not* used by the fold for emission (it would re-introduce ordering as engine policy, contradicting Modeling problem 4 corrected). Ordering is diagnostic-only.
4. **Apparent multi-inhabitance dissolves through structural modeling** (Examples 3, 4, 7) — what looked like "multiple inhabitants needing canonical choice" was actually meaningful structural differences (ownership, growability, lifetime, encoding). Modeling those differences as substrate facts + deriving program intent from program structure yields a unique answer per use site.
5. **Program intent is derived from program structure, not annotations** (Examples 3, 4, 7) — lifetime/escape/use analysis reads the program graph and produces the structural facts the fold composes against. No annotation surface.
6. **Fail-closed has typed diagnostics with resolution hints** (Examples 1, 5, 6) — when the fold can't determine, `EmissionDiagnostic` names what would resolve (refinement to add, substrate fact to declare, or genuine "no candidate covers this case").
7. **Compound types compose recursively at every level** (Example 7) — outer level + inner level each apply structural fold; both levels independently fail-closed if either is under-refined.
8. **Cross-target works because each language spec is independent + portability meta-spec enforces "can match"** (Example 8) — three folds run independently, three results emerge from declared facts. A program that fully-refines on its own structure grounds on all targets that have an inhabitant; one that under-refines fails-closed where targets can't disambiguate.

**These are the structural test of "no separate coercion engine"** per THESIS:171. If any example required engine policy to produce its expected output, the no-engine claim would be falsified. Each example is reproducible against the substrate facts named in its setup.

### Open design calls surfaced by the examples

The reframe from "canonical choice + annotations" to "structural modeling + program-derived intent" surfaces real design calls that need Director sign-off **before R2-T-Ground dispatch (the 5 engine-reframe lanes: Coercion-Fold, LanguageSpec, Lifetime-Analyzer, Diagnostic, CrossTarget-Meta — plus the existing T-Ground-Dissolve sibling lane in the 11-lane Grounding program)**. R2-Evaluator dispatch is unrelated parallel work and not gated on these calls — see §"Open call 1" Timeline.

1. **Strict vs pragmatic minimally-complete** (Example 3): does `data name: String = "Alice"` emit `Box<str>` (strict — what the program declares) or `String` (pragmatic — what the program might want later)? **DECISION (locked 2026-04-28 per user direction): strict.** Pragmatic adds engine policy disguised as ergonomics. If users need ergonomic defaults, they declare them via substrate-level refinement defaults, not engine guesses.
2. **Lifetime/escape analyzer scope** (Example 4): R2 covers (a) top-level data bindings, (b) function parameters with transient use, (c) function return values via R2-T-Ground-Lifetime-Analyzer. **DECISION (locked 2026-04-28 per user direction): d/e/f LAND IN R3** — closures (d), async lifetimes (e), self-referential / Pin (f) are R3 work, not deferred to post-R3. **Lane home: T-LensProducer-Retirement** (folded into existing R3 lane; the lifetime analyzer is structurally what replaces `lens_apply.rs`'s reflection work, so the advanced lifetime cases land alongside the retirement). The R3 verification surface (`T-Verification-L4-L7-Direct`) needs the analyzer for non-trivial programs; deferring to post-R3 would constrain R3's runnable corpus.
3. **Apparent multi-inhabitance audit** (general): for every case that looked like "multiple inhabitants needing canonical," re-audit per Modeling problem 2 corrected: is the difference cosmetic (collapse) or meaningful (model the structural axis)? **Recommendation: enumerate the cases as part of T-Ground-LanguageSpec lane scope; each case is a sub-task that either retracts a candidate or extends substrate refinement.**
4. **Required structural axes for Rust target** (general): the String family example surfaced (ownership, growability, encoding, lifetime). The integer family used (bound, signedness via algebra). What other axes does Rust need? Float family (precision, NaN-handling), reference family (mutability, lifetime, raw vs reference). **Recommendation: T-Ground-Rust XL lane scope explicitly enumerates the structural axes needed per Rust primitive family before substrate-population begins.**

These are the design questions the engine framing was hiding. Surfacing them as open calls means **R2-T-Ground dispatch waits on real design work**, not on engine implementation. R2-Evaluator dispatch is parallel and proceeds independently — see §"Open call 1" Timeline for the gate scope.

## Affected lanes (post-merge realignment)

[PR #989](https://github.com/gunb-ai/gunbc/pull/989) "T-Ground-Engine: Phase 2 pilot-list enumeration (slice 1)" merged on main (stern-ant-452 + merry-bat) before this design doc was authored. **Actual slice-1 footprint** (per R2 Grounding Manager review 2026-04-28; verified at `src/v3/grounding_engine/src/lib.rs`): ~370 lines of structural-equality validation (`validate_loaded_rust_primitive_type_structure`, `validate_rust_primitive_type_structure`, `validate_mirror_consistency`, `validate_first_rust_pilot_row_matches_mirror`). Failure type is `StructureMismatch { location, expected, actual }`. **There is no selection logic, no inhabitance-search, no tie-breaking** — slice-1 is a one-way mirror-consistency probe between `Dag::rust_pilot_primitives()` and the `RUST_PILOT_PRIMITIVES` Rust mirror. `Coercion-Fold`'s `EmissionDiagnostic::{UnderRefined, NoInhabitant}` framing has no edge in slice-1; the only failure channel today is `StructureMismatch`. **Post-merge realignment options:**

**(a)** Follow-up PR **renames** the `grounding_engine` crate / types per `T-Ground-Coercion-Fold` (or re-homes the mirror probe under `T-Ground-LanguageSpec` once that lands), and **introduces** `EmissionDiagnostic` as a separate typed carrier for under-determinism (currently no consumer exists; the carrier lands when fold work begins). Slice-1's mirror-consistency probe stays in place. **Sizing: S (rename + crate-relocation + introduce unused typed EmissionDiagnostic).** Not M — there's nothing to "retract" because there's no selection logic to remove.

**(b)** Hold further slices (Phase 2 slice 2+) until LanguageSpec schema lands; slice-1 code on main remains as-is until follow-up cleanup wave. Avoids further engine-framed code while LanguageSpec is designed.

**(c)** Combine: ship (b) immediately (hold further slices) and queue (a) as a follow-up cleanup PR once LanguageSpec lands and consumers can route through the new substrate.

**Recommendation: (c).** (b) is the immediate hold; (a) is the rename/re-home/re-type cleanup that follows once LanguageSpec lands. (c) is the realistic sequencing — slice-1's mirror-consistency probe stays on main, gets renamed and re-homed under LanguageSpec, and the typed `EmissionDiagnostic` lands when fold consumers actually start using it.

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

**Per Director directive 2026-04-28 (gpt-5-5-pro reflective analysis):** the following dissolutions are tracked as cascade items alongside the engine retraction, each is a substrate-completion or bridge-retirement instance:

- **MethodContract consolidation** (gpt-5-5-pro Finding #11): `dsl/extdeps/languages/{rust,python,go}/runtime.dag` declares `MethodTranslation { dag_method, rust_template }` AND `dsl/extdeps/languages/{rust,python,go}/emit.dag` declares `SimpleMethodSpec { method_name, template, wraps_result }` — different schemas, already-drifted templates (e.g., Rust `count`: runtime says `"{recv}.len()"`, emit says `"({recv}.len() as i64)"`; placeholder names also differ — `{arg0}` vs `{arg}`). Same pattern across Rust/Python/Go = parallel-rep × 3. **Substrate-completion sub-lane:** consolidate to single `MethodContract { dag_method, runtime_template, emit_template, wraps_result, placeholder_convention }` per-target row in T-Ground-LanguageSpec scope. Method-translation IS substrate (program-side `dag_method` → target-side template); two parallel authorities for one fact violates the engine-retraction discipline directly.
- **`Bool inhabits BooleanAlgebra<Bool>` dissolution** (gpt-5-5-pro Finding #1+#2): `src/v3/compiler/src/bootstrap.rs:91-174` has `patch_kernel_bool_boolean_algebra_inhabits` allocating BooleanAlgebra<Bool> Instantiation directly because the v2 compiler surface doesn't accept `type … inhabits … =` syntax in `dsl/`. Comment names dissolution explicitly. **Cascade target:** when v2 compiler surface lands, `type Bool inhabits BooleanAlgebra<Bool> = True | False` declared in `dsl/std/algebra.dag` (or kernel.dag); patch + operator-resolver fallback both retire mechanically. Lane home: T-Ground-Coercion-Fold (substrate-completion) or future T-Bridge-Retirement R3 sub-lane.
- **`include_str!` retirement** (gpt-5-5-pro Finding #12): `pipeline_authority` reads pipeline stage order **only** from `PipelineStageBinding` rows in the Dag (structural). `fn compile` stays `ArrowBody::Unparsed`, so compile-body stage order is not a lowered substrate fact yet; **PR #1171 (2026-04-29)** suspended the prior `reconcile_with_compile_body` path rather than swapping `include_str!` for runtime file IO (still a source-text side channel). **`bridge_include_str_side_channels_retired` for this site remains open** until derivation or a structural compile-body witness. **R3 T-Bridge-Retirement sub-lane:** unified ledger of `include_str!` side-channels across the codebase; each instance retires when its consumer can read the structured authority directly.

**Ownership:** Director or Grounding Manager #860 (cascade-author per the cascade-promotion pattern from `docs/design-pure-bootstrap-zero.md`).

### 3. Post-R3 dogfooding decision

Modeling problem 9 (first-class language-spec emission) is **post-R3** (locked 2026-04-28; Director may revisit). Whether it ships as part of ecosystem buildout or as an explicit later release is open. Probably ecosystem buildout — it's a dogfooding capability that doesn't gate any thesis claim.

### 4. Lens-as-parametric-monoid framework — separate authority

The cost-lens-over-emission framing in Modeling problem 8 generalizes structurally: any lens that folds over the compositional DAG via a (cost-basis, monoid, side-condition) tuple. Per Director response on #1078, this is a **separate design authority** — `docs/design-lens-framework.md` (PROPOSAL → LIVE post-merge of #1078, Director-authored).

**Substrate sequencing locked 2026-04-28:**
- **R2-T-Substrate-Lens-Primitive** (small substrate addition; ~1.5-2 weeks at gunbc velocity) declares `Lens<C>` parametric type + generic fold + cost-basis discipline. **Lands in R2** to avoid dual-representation risk per user direction "anything pushed out compounds exponentially."
- **R3 lanes consume `Lens<C>` differently** (per codex BLOCKING `f5f63c7d9`): **T-CostLens-Composition** is a `Lens<SymbolicCost>` instance — structural fold over substrate facts. **T-Verification-L4-L7-Direct** is NOT a `Lens<C>` instance — it's a *runtime equivalence check* that compares emit-target output vs .dag eval result; the lens framework's `read: (Dag, Behavior) → Witness<C>` cannot read emitted target artifacts. T-Verification-L4-L7-Direct *consumes* `Lens<C>` instances as inputs (e.g., `Lens<SymbolicCost>` for cost-related claims, `Lens<EmissionPathPresent>` for structural pre-checks) but the lane itself is corpus-driven runtime, not structural fold. T-Bridge-Retirement (where applicable) similarly.
- **L6 is a substrate-load-time cross-product completeness check, NOT a `Lens<C>` instance** (per codex BLOCKING 2026-04-28T21:26 — same input-space mismatch as L4/L7: `Lens<C>.read: (Dag, Behavior) → Witness<C>` reads PER-BEHAVIOR substrate facts, but L6's fold ranges over `(substrate-form × Shape-A-target)` pairs, which are not per-Behavior). The Director's earlier "L6 collapses to `Lens<EmissionPathPresent>`" framing was structurally incorrect — Lens<C>'s input space doesn't match L6's input space. L6 lives in R2-T-Ground-CrossTarget-Meta as its own structural primitive: a substrate-load-time completeness check that walks every `(connective × behavior × target)` cell and verifies an emission-path declaration exists; failure surfaces as a typed `Diagnostic` (kind = `MissingEmissionPath`). No Lens<C> involvement; no substrate generalization needed.

**Director's locked design decisions** (per response on #1078, recorded here for `design-lens-framework.md` authoring):
1. **Pure monoidal**, not stateful. Memory-peak (anamorphism + state) is a separate framework.
2. **Result type:** see [`docs/design-lens-framework.md`](design-lens-framework.md) §"Lens<C> primitive" — the framework reuses the existing `DimensionReport<Carrier>` from `src/v3/std/dimensions.dag:51-61` verbatim. Single carrier authority lives in the lens-framework doc; this row no longer restates the shape (per codex BLOCKING on `a9326224`: restating the shape across multiple authority docs created drift opportunities — single-authority discipline applies).
3. **Higher-order shapes:** function-valued cost basis derived from signature. Meta-lens (lens-on-lens) deferred post-R3.
4. **Cross-domain composition:** explicit declaration only. `Lens<C> × Lens<D> = Lens<(C, D)>` with product monoid; side-conditions compose conjunctively. User-declared, not auto-derived.
5. **User-authored lens substrate:** T-LensAPI rescope to lens-as-monoid in same wave as `Lens<C>` lands. User-lens surface inherits structurally.
6. **Three worked instances sufficient for generality validation:** complexity (additive numeric monoid), tenant-flow (set union + categorical authorization), IFC (lattice join + downgrade rejection). Stretch goals (memory-peak, energy, latency) are post-R3 instances.

**The doc [`docs/design-lens-framework.md`](design-lens-framework.md)** (PROPOSAL skeleton authored 2026-04-28; Director extends post-#1078-merge for full spec) is the separate authority for the lens framework — three worked instances (complexity / tenant-flow / IFC), Director's 6 locked design decisions, migration plan for the 4 existing PROXY/STUB lenses, and the up-front validation checklist (design-phase + implementation-phase + migration-phase self-checks). This is the contract R2-T-Substrate-Lens-Primitive substrate work delivers and R3 lens instances consume.

**Ownership and timeline** (Director-locked 2026-04-28; clarified 2026-04-28 per gpt-5-5-pro BLOCKING re P2 single-authority): the substrate primitive (`Lens<C>` + generic fold + cost-basis discipline) **lands in R2** as `R2-T-Substrate-Lens-Primitive` per the substrate sequencing above (sized ~1.5-2 weeks at gunbc velocity). The framework's design-doc full spec is **Director-authored post-#1078-merge** (PROPOSAL skeleton already in `docs/design-lens-framework.md`; full spec extension is the Director's next authoring deliverable, not a worker task). **Stretch instances** (memory-peak lens, energy lens, latency lens) are **post-R3** — they're additional lens-framework instances beyond the 3 worked examples (complexity / tenant-flow / IFC) needed for R2's "3 instances sufficient for generality validation" gate; they don't gate R2 substrate primitive landing. No ambiguity: substrate primitive R2; design-doc full spec Director-authored post-#1078; stretch instances post-R3.

## Cross-refs

- Parent thesis claim: [`THESIS.md`](../THESIS.md) §"Tier 1 — Structural correctness" — Coercion = emission, no separate coercion engine
- Architectural authority: [`docs/single-emitter-design.md`](single-emitter-design.md) — coercion = emission; algebra-homomorphism-not-lookup
- Predecessor (superseded framing): [`docs/thesis/target-grounding-proposal.md`](thesis/target-grounding-proposal.md) — "Coercion engine" row at `:165` and `:344`
- ROADMAP lane: [`ROADMAP.md`](../ROADMAP.md) §"Post-R1 Grounding lanes" — `T-Ground-Engine` row to supersede
- Manager brief: [`docs/briefs/grounding-manager.md`](briefs/grounding-manager.md) — lane list to amend
- R2/R3 planning: [`docs/r2-structure.md`](r2-structure.md), [`docs/r3-structure.md`](r3-structure.md), [`docs/thesis/r2-r3-thesis-mapping.md`](thesis/r2-r3-thesis-mapping.md)
- Substrate dependencies named: DB-11 (refinement-carrying qualifiers), cardinality-substrate; DB-18 (parametric algebra attachment); E-9 (external realization on `Arrow.body`)
- INVARIANTS: [`INVARIANTS.md`](../INVARIANTS.md) §P3 Fail-Closed; §P2 Boundary Discipline; §P1 Modeling Faithfulness

---

## Design questions to surface and lock before dispatch

**Status:** SURFACED 2026-04-28 per Director directive — "surface all design questions up front; modeling per-language is incredibly hard; ensure verification (testing) discussion + discussion up front; otherwise leaving complex modeling problems open-ended will fail (per `feedback_holistic_over_patches.md`, `project_ownership_holistic.md` — alias/clone class)."

**Why this section exists:** the no-engine reframe surfaced design questions the engine framing was hiding. Each question below is one where a worker hitting it mid-dispatch could escalate into design rework. The surfacing names the question explicitly, enumerates the alternatives, lists cascade implications, and proposes a TestClaim shape regardless of which alternative is chosen — so verification discipline locks the answer once the Director picks an alternative.

**Scope:** these are the modeling-hard questions for **T-Ground-* substrate dispatch** (per-target grounding). The lens-framework spec questions are surfaced separately in [`docs/design-lens-framework.md`](design-lens-framework.md) §"Design questions to lock before substrate dispatch."

### Q1 — `BoundDeclaration` substrate type

**Status:** REFERENCED in Examples 1 and 8 as a sum type `ExactBound { range } | AnyBound | PlatformDependentBound`; NOT yet declared in `dsl/std/`.

**Question:** what file declares `BoundDeclaration`, what's its precise shape, and what's the parse syntax for `Int(any)` and `Int(platform)` programs?

**Alternatives:**
- (a) Declare `BoundDeclaration` as a new top-level type in `dsl/std/inhabitance.dag` (new file). Parse syntax: `Int(any)` → `BoundDeclaration::AnyBound`; `Int(platform)` → `BoundDeclaration::PlatformDependentBound`; `Int(0..2^32)` → `BoundDeclaration::ExactBound { range: NumericRange { lo: 0, hi: 2^32 } }`.
- (b) Declare in existing `dsl/std/algebra.dag` alongside `OrderedRing` / `Semiring` etc. (no new file).
- (c) Decompose: `ExactBound` becomes part of refinement substrate; `AnyBound` is a flag on the inhabitance fact; `PlatformDependentBound` is a separate `PlatformDependentInhabitance` carrier. (Not a sum type — multiple substrate facts.)

**Cascade implications:**
- Parser changes needed for `Int(any)` / `Int(platform)` syntax in all three alternatives. Lane: T-Ground-Coercion-Fold needs the parse + AST shape.
- `NumericRange` may itself need substrate (lo/hi pair with cardinality). May overlap with existing cardinality-substrate (DB-11).
- Each target's inhabitance fact needs a `bound: BoundDeclaration` field (Rust int32, Python int, Go int32 all carry it). T-Ground-LanguageSpec consumer.

**TestClaim shape (regardless of alternative):**
- `bound_declaration_carries_three_variants_structurally` (counts variants if (a)/(b); checks decomposition if (c))
- `exact_bound_matches_program_range_exactly` (Example 8 Rust int32 case)
- `any_bound_matches_explicit_program_bound` (Example 8 Python int case)
- `any_bound_does_not_match_under_refined_program` (Example 1 cross-target consistency case — `Int` with no bound fails on all targets)
- `platform_dependent_bound_matches_only_explicit_platform_decl` (usize / Go `int` case)

**Recommendation:** **(a)** — new file `dsl/std/inhabitance.dag`. Reasoning: BoundDeclaration is one of several inhabitance-fact carriers (ExactBound, AnyBound, PlatformDependentBound, future `BorrowedBound`, etc.); collecting them in one substrate file matches the modeling discipline. Parser change is bounded (one new postfix `(any)` / `(platform)` keyword). Keeps `algebra.dag` focused on algebraic structures.

**DECISION (Director-locked 2026-04-28 via dialogue; refined 2026-04-28T17:06 per codex BLOCKING):** Q1 resolves through **structural consolidation** rather than (a)/(b)/(c) directly. The shared underlying modeling for `CardinalityBound`, `SizeBound`, and the value-domain part of `BoundDeclaration` is **interval over a totally ordered set** — a parametric `Interval<D>` substrate concept. But `PlatformDependent` is NOT an interval (it resolves to an interval at target-platform time; doesn't fit Interval<D>'s variants). So `BoundDeclaration` is a sum that wraps Interval<Int> for the static case and adds PlatformDependent as a distinct kind. The decision:

- **Declare `Interval<D>` as the shared parent** in substrate. Variants: `ExactInterval { lo: D, hi: D } | Unbounded`. `D` is the ordered domain (Cardinal, Int, Ordinal).
- **`BoundDeclaration = StaticBound(Interval<Int>) | PlatformDependent`** — `StaticBound` carries the value-domain interval (with explicit `lo`); `PlatformDependent` is a distinct kind whose interval depends on target platform. The fold's single match predicate dispatches on the outer sum, then matches `Interval<Int>` variants for `StaticBound` cases.
- **`CardinalityBound` and `SizeBound` retrofit** as `Interval<Cardinal>` instances (additive — existing accessor patterns continue to work via aliases).
- **`LoopBound::Cardinality` retrofit** as `Interval<Ordinal>`-like. **`LoopBound::Descent` stays distinct** (termination witness — well-founded recursion, not an interval).
- **`CostBound` stays distinct** (asymptotic equivalence class — different math; not an interval).

**Why consolidation:** the `feedback_epistemic_stacking` modeling discipline says every concept attaches to an ontological DAG. Adding `BoundDeclaration` as a fifth bound type without recognizing the shared parent would be parallel-representation debt (per `feedback_parallel_representation_debt`). Director directive 2026-04-28: "each of these are probably distinct - but also probably share some underlying (modeling)" — the shared modeling is `Interval<D>`.

**Sequencing:** the consolidation is **prepended** as `PR-PreF` (a new pre-cadence design PR before PR-F) so the substrate landing for `BoundDeclaration` consumes a clean `Interval<D>` parent rather than introducing a fifth distinct bound type.

**PR-PreF scope** (sized 2-3 days at gunbc velocity):
- Declare `type Interval<D>` parametric in `src/v3/std/substrate.dag` (siblings to `CardinalityBound` and `LoopBound`)
- Retrofit `CardinalityBound`, `SizeBound` as `Interval<Cardinal>` instances (additive — existing variant accessors stay working via aliases like `AtMostOne ≡ ExactInterval { lo: 0, hi: 1 }`)
- Retrofit `LoopBound::Cardinality` similarly; `LoopBound::Descent` stays
- `CostBound` not touched
- Acceptance: existing CardinalityBound + SizeBound + LoopBound consumers compile unchanged; `Interval<D>` substrate-form ratchet green; structural-form census stays at zero

**Cascade benefit on Q5:** with `Interval<D>` as shared parent, Q5 recommendation (a) "cardinality is connectives axis" is reinforced — `List<T>` carries `Interval<Cardinal>::Unbounded`; `Atom` carries `Interval<Cardinal>::ExactInterval { lo: 1, hi: 1 }`. The L6 cross-product fold becomes `connectives × behaviors × targets` (90 cells) without a separate cardinality-variant axis.

**Q1 refinement (Director-authored 2026-04-29) — asymmetric parent assignment for distinct bound carriers**

Q1's DECISION names `LoopBound::Descent` and `CostBound` as "distinct" from `Interval<D>` but doesn't name their parents. Workers retrofitting bound types would re-litigate; per INVARIANTS.md#p1-modeling-faithfulness substrate-fact-introduction Step 1 (DAG-ancestor check), the parent is determined by the bound's **algebra shape**, not by name similarity. Naming each carrier's parent explicitly:

| Bound carrier | Algebra shape | Parent |
|---|---|---|
| CardinalityBound (`List<T>` / `Atom`) | ordered numeric (Cardinal) | `Interval<Cardinal>` |
| SizeBound | ordered numeric (Cardinal) | `Interval<Cardinal>` |
| LoopBound::Cardinality | ordered numeric (Ordinal) | `Interval<Ordinal>` |
| **LoopBound::Descent** | **lattice (DescentEvidence)** | **`BoundedLattice<DescentEvidence>`** (per `dsl/std/termination.dag:60-67`; declared via lens-extensible inhabitance per Q6.5 of design-lens-framework.md) |
| **CostBound** | **lattice (BigOClass)** | **`BoundedLattice<BigOClass>`** (per Q3 + design-lens-framework.md Instance 1's `inhabits BigOClass : BoundedLattice<BigOClass>` declaration) |

The asymmetry is structurally honest: ordered-numeric bounds have natural interval parent (Interval<D> with `lo`/`hi` bounds and totally ordered comparison); lattice-typed bounds have natural lattice parent (BoundedLattice<C> with `meet`/`join` and partial ordering). Forcing all bounds under Interval<D> would mis-encode lattice-typed semantics; forcing all bounds under BoundedLattice<C> would mis-encode the ordered-numeric range structure.

**Cascade implications for retrofit:**
- PR-PreF scope unchanged: only ordered-numeric bounds retrofit to `Interval<D>` (CardinalityBound, SizeBound, LoopBound::Cardinality).
- LoopBound::Descent retrofits onto `BoundedLattice<DescentEvidence>` parent. Currently `dsl/std/termination.dag:60-67` declares the inhabitance via comments only ("DescentEvidence inhabits BoundedLattice"); the inhabitance should land structurally per Q6.5's lens-extensible inhabitance discipline (declared in the substrate's own `.dag`, not via comments).
- CostBound retrofits onto `BoundedLattice<BigOClass>` parent (per Q3's `Cost<Unit> = Dimension<Unit, SymbolicExpr>` lock; the asymptotic-class projection is a BoundedLattice instance per design-lens-framework.md Instance 1).
- Workers retrofitting bound types apply this rule deterministically: ordered-numeric → `Interval<D>`; lattice-typed → `BoundedLattice<C>`. No re-litigation per workers; rule is mechanical.

**Anti-bridge invariant:** workers MUST NOT collapse lattice-typed bounds into `Interval<D>` by encoding lattice ordering as `ExactInterval` (e.g., DescentEvidence::Strict ↔ ExactInterval(1, 1)). Lattice ordering is partial; interval ordering is total. The bridge would create false equivalence between distinct algebra structures and break the structural-fact discipline.

**TestClaim shape:**
- `bound_carrier_parent_matches_algebra_shape` — verifies each bound carrier's declared parent matches its algebra (ordered-numeric → Interval<D>; lattice → BoundedLattice<C>)
- `no_lattice_to_interval_collapse_bridge` — anti-bridge enforcement; checks no bound carrier has both `Interval<D>` parent AND lattice-typed values

**DECISION (Director-authored 2026-04-29):** asymmetric parent assignment locked. Workers retrofit bound types per their algebra shape: ordered-numeric → `Interval<D>`; lattice-typed → `BoundedLattice<C>`. Cross-references with design-lens-framework.md Q6.5 (lens-extensible inhabitance pattern for declaring lattice-typed bound parents structurally rather than via comments).

### Q2 — Required structural axes per Rust primitive family (and per target)

**Status:** STRING family enumerated in Example 3 (ownership / growability / encoding / lifetime); INTEGER family used in Examples 1/2/8 (bound / signedness via algebra). Other Rust families not enumerated. Python and Go axes not enumerated at all.

**Question:** before T-Ground-Rust / T-Ground-Python / T-Ground-Go can populate substrate, what's the EXHAUSTIVE list of structural axes per primitive family per target?

**Alternatives:**
- (a) Enumerate all axes for all 5 primitive families × 3 targets in this PR before dispatch. Estimated: 30+ axes across 15 cells. Substantial scope.
- (b) Lock the SHAPE (per-family axis declaration discipline) here; cadence PR-F (Rust axes), PR-G (Python axes), PR-H (Go axes) before respective T-Ground-Rust / Python / Go dispatch.
- (c) Lock only the families surfaced by worked examples (Integer + String for Rust; same for Python and Go). Other families (Float, Reference, Composite) discovered during dispatch and modeled inline. RISK: this is the alias/clone shape — open-ended modeling.

**Cascade implications:**
- Each axis is a substrate field that must declare a TYPE (e.g., `mutability: Mutability` where `Mutability = Mutable | Immutable`).
- Axis enumeration affects T-Ground-LanguageSpec scope (each language spec must declare each axis per primitive).
- L6 cross-product fold (R2-T-Ground-CrossTarget-Meta) iterates over `(connectives × behaviors × cardinality_variants × axis_combinations) × targets` — axis count is multiplicative.
- Per-language differences (Rust has `Pin` + lifetime; Python has refcounting; Go has GC + interface dispatch) — what axes are PER-LANGUAGE vs SHARED-CROSS-LANGUAGE?

**TestClaim shape:**
- Per axis: `<target>_<family>_<axis>_carries_structurally` (e.g., `rust_string_ownership_carries_structurally`)
- Per inhabitance: `<target>_<primitive>_inhabits_<algebra>_at_<axis_combination>` (e.g., `rust_box_str_inhabits_freemonoid_char_at_owned_nongrowable_self`)
- Cross-target: `axis_<X>_consistent_across_targets` where applicable (e.g., `bound_consistent_across_rust_python_go`)

**Recommendation:** **(b)** — lock per-family axis discipline here; cadence PR-F (Rust axes), PR-G (Python axes), PR-H (Go axes). Each is a focused 1-2 day design PR with TestClaim acceptance gate. Reason: enumerating in this PR makes #1078 enormous; deferring entirely is the alias/clone risk. Cadenced sub-PRs with TestClaim gates is the structural discipline.

**DECISION (Director-locked 2026-04-28 via dialogue): (b3') — Emission-biased non-violating minimal target modeling.** Refines (b) with three additional constraints:

1. **Goal: faithful target-language modeling.** The end-state is full structural shape — Rust references / lifetimes / pointers / etc. modeled as Rust *actually* defines them. Time-bounded velocity drives the cadence, not a different goal.

2. **Bias: model what emission needs first.** Worked examples in this doc surface axes; new axes added as new `.dag` patterns surface them. Director directive: "we don't need to model 'rust' - we need to model our emission into rust/go/python."

3. **Invariant — non-violating:** what we model must be a CORRECT SUBSET of the target's actual semantics. We can't claim `&T` is mutable. We can't claim Python `int` overflows. The subset is faithful (aligned with target reality), not a custom abstraction. Per-inhabitance modeling validates against the target language's actual specification (per-target-spec audit).

4. **Reference/pointer concepts share a parent** — same DAG-grounding move as Q1's `Interval<D>` and Director synthesis's `Monoid<C>`. Rust `Box<T>`/`&T`/`Rc<T>`, Go `*T`, Python object reference all share underlying modeling. Declare `ReferenceModel<T>` parametric in substrate with axes (`lifetime`, `mutability`, `ownership`, `representation`); each target's pointer/reference inhabitances declare which combination of axes they cover. Same epistemic-stacking discipline applied recursively.

**Verification — four-property framework** (per Director directive: "for any combination of `.dag`, we can demonstrate that its the minimal, performant, correct and faithful representation"):

| Property | By construction or by test? | Where it lives |
|---|---|---|
| **Faithful** | By construction (structural fold) + per-inhabitance non-violation gate (target-language-spec validation per axis) | `Lens<FaithfulnessVerdict>` (structural lens; reads substrate facts) + per-target test harness (emit a stub program; verify it compiles through the target's actual compiler) + spec-audit reviewer trail |
| **Correct** | By runtime test (NOT a structural lens) | L4 emit/eval match (existing) — *runtime equivalence check* comparing emit-target output vs .dag eval result. **Not a `Lens<C>` instance** (per codex BLOCKING `f5f63c7d9`): `Lens.read` cannot read emitted target artifacts. Lives in T-Verification-L4-L7-Direct as corpus-driven runtime harness. |
| **Minimal** | By comparison (could be structural or runtime) | If "minimality" is structurally definable (e.g., emission size = sum of substrate-declared sizes), `Lens<MinimalityVerdict>` works. If minimality requires comparing against alternative emissions (run alt-emit; compare), it's runtime — lives in T-Verification harness alongside L4. The structural version is preferred when expressible. |
| **Performant** | Structural (reads substrate cost facts) | `Lens<PerformanceVerdict>` — reads per-target `RealizationCost` from substrate (per Q3 `(c3) RealizationCost { storage, access }` shape); checks for pathological patterns (e.g., O(n²) cost where O(n) is available). Structural lens; no runtime needed. |

**Distinction (per codex BLOCKING `f5f63c7d9`):** structural-fold properties (Faithful, Performant, structural-Minimal) are `Lens<C>` instances reading substrate facts. Runtime-equivalence properties (Correct = L4 emit/eval match; runtime-Minimal if not expressible structurally) live in T-Verification-L4-L7-Direct as corpus-driven harness — NOT lens instances. The lens framework is for *structural folds over .dag*; it doesn't generalize to "compare emit-target output vs eval result" because `Lens.read: (Dag, Behavior) → Witness<C>` only reads substrate, not emitted artifacts.

**Cadence consequence:** PR-F is bounded to "axes the worked examples surface for Rust" — not "everything Rust offers." PR-G/H similarly. Each PR adds its target's axes per worked-examples-driven priority.

**What "lock the shape" means here, concretely:**
1. Each axis is declared as a *substrate sum type* in `dsl/std/inhabitance.dag` (or per-target file like `dsl/std/rust.dag`). Format: `type Mutability = Mutable | Immutable`.
2. Each per-target inhabitance fact includes a struct of axis values: `inhabits BoxStr : FreeMonoid<Char> { ownership = Owned, growability = NotGrowable, lifetime = SelfContained }`.
3. The fold reads inhabitance facts via `Dag::declarations()` and matches on axis values (no inferred axes; if an axis isn't declared, fail-closed).

### Q3 — Per-primitive realization cost field shape on language specs

**Status:** REFERENCED in Modeling problem 8 and `T-CostLens-Composition` lane scope as "per-primitive realization cost via the language spec." NOT yet specified.

**Question:** what's the substrate shape for declaring per-primitive realization cost on a language spec?

**Alternatives:**
- (a) `cost: CostExpr` field on each inhabitance fact. (Reuses `CostExpr` — assumed to be defined in `dsl/std/cost.dag` or similar.)
- (b) Separate `realization_cost` declaration: `data rust_int32_add_cost: CostExpr = CostExpr(work=1, span=1, class=O(1))`. Loose-coupled to the inhabitance fact.
- (c) Cost is per-OPERATION on each inhabitance, not per-primitive: `inhabits Int32 : OrderedRing { cost_of_add = CostExpr(...), cost_of_mul = CostExpr(...) }`. Fine-grained per algebra operation.

**Cascade implications:**
- The lens-framework instance for `Lens<SymbolicCost>` reads this field via `read(dag, behavior)`. Lane: R2-T-Substrate-Lens-Primitive consumer of substrate; R3-T-CostLens-Composition consumer of the realization-cost facts.
- `CostExpr` itself: does it already exist in substrate? Or does it need to be declared here? (Currently `cost.dag` is a PROXY per `design-emission-model.md:310`.)
- Cost asymmetry: an operation's cost may differ between targets (e.g., Rust `BigInt.add` is O(digits) vs Python `int.add` is O(digits) but with different constants). How is the SHARED algebraic cost (target-agnostic) distinguished from the REALIZATION cost (target-specific)?

**TestClaim shape:**
- `realization_cost_field_carries_costexpr_per_inhabitance`
- `cost_lens_reads_realization_via_dag_explicitly` (no hidden lookup)
- `target_specific_cost_differs_across_targets_for_same_algebra_op` (e.g., Rust `BigInt.add` cost ≠ Python `int.add` cost)

**Recommendation:** **(c)** — per-operation cost on each inhabitance, fine-grained. Reason: matches the algebra-inhabitance discipline (each algebra has known operations; each inhabitance declares the operation's realization cost on that target). Loose coupling (b) creates parallel-representation debt (cost lives in two places: inhabitance + cost declaration). Per-primitive (a) doesn't disambiguate which operation's cost is meant.

**DECISION (Director-locked 2026-04-28 via dialogue): (c5) — RealizationCost as record of `Cost<Unit>` coordinates, with `Cost<Unit> = Dimension<Unit, SymbolicExpr>` and `Unit` substrate-declared primitives (sibling to existing SI base units in dimensions.dag).**

Director directive: cost is *not* a coproduct of "Time | Space | Energy" — those are coordinates, not alternatives. Per `feedback_coproduct_dissolution`: dissolve into coordinates. Director's nervousness about modeling fundamental concepts (Time, Space, Energy) is addressed by treating them as **substrate-declared primitives** (sibling to Meters/Seconds/Kilograms in `dimensions.dag:93-99`), not user-extensible labels.

**Substrate shape:**

```
// Existing substrate (PR #886, dimensions.dag):
type Dimension<Unit, Carrier> { value: Carrier }
type SymbolicExpr   // existing complexity-expression carrier

// New primitive Unit types (sibling to Meters, Seconds, Kilograms, ...):
type Bits             // storage / memory cells (digital)
type CPUCycles        // computation steps (single-core)
// (Future: type Joules, type NetworkOps — added as substrate primitives,
//  not user-extensible coproducts)

// Cost<Unit> dissolves into Dimension<Unit, SymbolicExpr>:
type Cost<Unit> = Dimension<Unit, SymbolicExpr>

// RealizationCost is a record (coordinates, not sum):
type RealizationCost {
  storage: Cost<Bits>                       // "N bits per stored representation"
  access:  Map<AlgebraOp, Cost<CPUCycles>>  // "M cycles per algebra operation"
}

// Existing SymbolicCost (algorithmic complexity) refines as a record with
// time-flavored coordinates:
type SymbolicCost {
  work: Cost<CPUCycles>     // sequential time
  span: Cost<CPUCycles>     // parallel time (critical path)
}

// Each inhabitance declares its realization cost:
inhabits Int32 : OrderedRing {
  realization = RealizationCost {
    storage = Cost<Bits> { value: 32 }                     // 32-bit register
    access  = {
      add: Cost<CPUCycles> { value: 1 }                    // ADD instruction
      mul: Cost<CPUCycles> { value: 1 }                    // MUL instruction
    }
  }
}

inhabits SPICEDigitalIntCircuit : OrderedRing<Int(N)> {
  realization = RealizationCost {
    storage = Cost<Bits> { value: N }                      // N-bit synthesized circuit
    access  = {
      add: Cost<CPUCycles> { value: N }                    // adder circuit
      mul: Cost<CPUCycles> { value: N^2 }                  // multiplier circuit
    }
  }
}
```

**Why (c5) lands cleanly:**

1. **No new framework** — reuses existing `Dimension<Unit, Carrier>` from PR #886. Same substrate that carries `Duration<Seconds>` carries `Cost<Bits>`. One DAG-grounded concept (Q1's epistemic-stacking discipline applied recursively).
2. **No coproduct compression** — Time/Space/Energy aren't alternatives; they're coordinates in a record. Dissolution per `feedback_coproduct_dissolution`.
3. **Treats fundamental concepts as primitives** — Bits and CPUCycles are substrate-declared sibling to Meters/Seconds. Not user-extensible labels; substrate primitives matching the existing SI-unit pattern.
4. **Cross-target comparisons fall out per coordinate** — pSPICE-vs-Verilog: "same .dag concept, different `Cost<Bits>` profile" or "different `Cost<CPUCycles>` profile." Each axis compares independently.
5. **Future axes attach by adding a Unit primitive** — `type Joules` lands when energy analysis matters; no framework change required.

**Sparse fail-closed access map:** `Map<AlgebraOp, Cost<CPUCycles>>` is sparse — only declared ops have cost; missing op = `Witness.Violates` per the lens framework's read-channel discipline. Forces honest modeling; no silent zero-cost.

**Cadence consequence:** PR-I (Q3 + Q4 cadence) lands the substrate primitives (`Bits`, `CPUCycles`) + `Cost<Unit>` alias + `RealizationCost` record + per-inhabitance declarations on language specs. Sized within the original 1-2 day estimate (one substrate type alias + two primitive types + record schema; per-inhabitance declarations come during T-Ground-LanguageSpec dispatch).

### Q4 — L4 emit/eval match acceptance corpus

**Status:** L5 corpus type DECIDED ("algebraic equivalence over curated corpus"). L4 corpus content NOT enumerated.

**Question:** what programs go in the L4 emit/eval-match certification corpus?

**Alternatives:**
- (a) Curated representative programs across all combinations of (substrate connective × behavior × cardinality × target). Comprehensive but large.
- (b) Hand-curated minimal set covering "interesting" cases (recursion, branch, loop bound, composite types). Smaller; depends on author judgment.
- (c) Generated cross-product corpus: every `(connective × behavior × cardinality)` combination × every Shape A target. Mechanical; could be large but bounded.
- (d) User-program corpus: when L4 is being authored, sweep `dsl/std/` and `src/v3/std/` for actual `.dag` programs and use those. Drives self-hosting verification.

**Cascade implications:**
- Corpus authoring is its own work. Lane: T-Verification-L4-L7-Direct (R3); needs Evaluator (R2) for runtime equivalence checks.
- Cross-target consistency: L5 needs the L4 corpus first (per r3-structure.md). L4 corpus shape affects L5's coverage.
- Acceptance: how do we know the corpus is "complete enough"? Is L6 (structural form coverage at R2) the completeness check? L6 verifies emission paths exist; L4 verifies emit/eval match.

**TestClaim shape:**
- `l4_corpus_covers_every_substrate_connective` (completeness — at least one program per connective)
- `l4_corpus_covers_every_l1_behavior` (completeness — at least one program per behavior)
- `l4_emit_eval_match_holds_per_corpus_program_per_target` (the actual L4 claim)

**Recommendation:** **(c) + (d) hybrid** — generated cross-product corpus for completeness coverage + user-program corpus (`dsl/std/` + `src/v3/std/` + `dsl/examples/`) for self-hosting/realism coverage. Reason: (c) gives completeness guarantee per L6's structural-form coverage; (d) drives self-hosting verification (compiler is written in `.dag`; the compiler IS the corpus for itself). (a) and (b) have judgment gaps. Cadence PR-I (L4 corpus authoring spec) before T-Verification-L4-L7-Direct dispatch.

**Cascade from Q2 lock — universal four-property claim** (per Director directive "for any combination of `.dag`, we can demonstrate that its the minimal, performant, correct and faithful representation"): the L4 corpus is the verification surface for this UNIVERSAL property over `.dag` programs. The corpus's acceptance gate must demonstrate all four properties hold for every (program × target) pair, not just emit/eval match. Concrete additions to PR-I (L4 corpus authoring spec):

- **Non-violation gate** (Faithful sub-property): every emitted program in the corpus passes through the target's actual compiler/type-checker without errors. Stronger and cheaper than emit/eval match — catches "we modeled `&T` as mutable" violations even without runtime.
- **Minimality lens applied to corpus output:** for each (program × target), assert the emission has no unused machinery (per `Lens<MinimalityVerdict>`).
- **Performance lens applied to corpus output:** for each (program × target), assert no per-target pathological pattern (per `Lens<PerformanceVerdict>`).
- **Cross-target equivalence (L5 tie-in):** L5 corpus is built on top of L4 corpus per existing decision; the four-property gate runs on L4; L5 layers cross-target consistency on top.

PR-I scope grows: from "L4 corpus shape" to "L4 corpus shape + per-target-compiler harness + four-property lens-instance applications." Sizing increases (~3-4 days vs the original 1-2 day estimate) but the verification claim is now demonstrably universal-over-corpus rather than just emit/eval match.

### Q5 — L6 cross-product fold cardinality variant enumeration

**Status:** L6 reclassified to R2-T-Ground-CrossTarget-Meta as a structural cross-product fold. Cross-product is `(6 connectives × 5 behaviors × cardinality variants) × Shape A targets`. "Cardinality variants" NOT enumerated.

**Question:** what cardinality variants does L6 enumerate over?

**Alternatives:**
- (a) `{ Singleton, Atomic, ListOf<T>, ConjOf<T,U,...>, DisjOf<T,U,...>, ArrowOf<T,U> }` — 6 variants matching the 6 connectives. (Then connectives × cardinality is redundant — they're the same axis.)
- (b) `{ Bounded(N), Unbounded, Empty }` — 3 variants describing collection cardinality (independent of element type).
- (c) `{ Required, Optional, ZeroOrMore, OneOrMore }` — 4 variants describing field/parameter cardinality.
- (d) Decomposed: cardinality is multiple axes — `presence: Required | Optional`, `multiplicity: One | Many`, `bounded: Bounded(N) | Unbounded`. (Parametric.)

**Cascade implications:**
- Each cardinality variant must have an emission path declaration per target (the L6 acceptance gate).
- If (a): the axis collapses with connectives — L6 is just `connectives × behaviors × targets` (no separate axis).
- If (b)/(c)/(d): the axis multiplies the substrate cross-product. Affects per-target axis enumeration (Q2).

**TestClaim shape:**
- `l6_cardinality_axis_carries_structurally`
- `l6_emission_path_declared_per_(connective_behavior_cardinality)_per_target`
- `l6_cross_product_complete_no_missing_emission_paths`

**Recommendation:** **(a)** — cardinality is the connectives axis. Reason: in v3 substrate, cardinality is a property of the type connective (List has unbounded cardinality; Conj has fixed cardinality = number of fields; Atom is singleton). Treating cardinality as a separate axis would double-count. The L6 fold becomes `connectives × behaviors × targets` = 6 × 5 × 3 = 90 cells. Manageable.

**Reinforced by Q1 consolidation (Director-locked 2026-04-28):** with `Interval<D>` declared as the shared parent for bound concepts (per Q1 DECISION above), cardinality lands on the connective via `Interval<Cardinal>` instances — `List<T>` carries `Interval<Cardinal>::Unbounded`, `Atom` carries `Interval<Cardinal>::ExactInterval { lo: 1, hi: 1 }`, etc. The L6 cardinality axis collapses into the connective axis by construction. PR-J (Q5 cadence) becomes likely no-op.

### Pre-dispatch design-PR cadence (per Director directive 2026-04-28)

The cadence below names the focused design PRs that lock per-target modeling before dispatch. Each is bounded (1-2 days, except PR-PreF at 2-3 days), has a TestClaim acceptance gate, and lands BEFORE the corresponding T-Ground / T-Verification dispatch. Modeled on the PR-A through PR-E cadence that locked R2-Evaluator design.

| PR | Locks | Before dispatch of | TestClaim gate |
|---|---|---|---|
| **PR-PreF** | `Interval<D>` substrate consolidation (shared parent for CardinalityBound / SizeBound / LoopBound::Cardinality; sets up Q1 instance) | All subsequent cadence PRs | `Interval<D>` substrate-form ratchet green; existing CardinalityBound + SizeBound + LoopBound consumers compile unchanged via additive retrofit |
| **PR-F** | `BoundDeclaration = Interval<Int>` (Q1 instance, consumes PR-PreF parent) + Rust structural axes (Q2 partial) | T-Ground-Coercion-Fold + T-Ground-Rust | All Q1 + Q2-Rust TestClaims pass |
| **PR-G** | Python structural axes (Q2 partial) | T-Ground-Python | Q2-Python TestClaims pass |
| **PR-H** | Go structural axes (Q2 partial) | T-Ground-Go | Q2-Go TestClaims pass |
| **PR-I** | Per-primitive realization cost field shape (Q3) + L4 corpus authoring spec (Q4) | T-Ground-LanguageSpec + T-Verification-L4-L7-Direct | Q3 + Q4 TestClaims pass |
| **PR-J** | L6 cardinality enumeration (Q5) — likely no-op given PR-PreF consolidation reinforces recommendation (a) | T-Ground-CrossTarget-Meta | Q5 TestClaims pass (only if (a) rejected, which is unlikely after PR-PreF) |

**Director's role:** sign off on Q1-Q5 alternatives + recommendations above, OR override with different choice. Each cadence PR consumes the locked decisions; without sign-off, dispatch waits.
