# Design — Emission Model (no separate coercion engine)

**Status:** `PROPOSAL` (2026-04-28). Pending Director sign-off + alignment with [`docs/thesis/target-grounding-proposal.md`](thesis/target-grounding-proposal.md).

**Authority on promotion:** [`THESIS.md`](../THESIS.md) §"Tier 1 — Structural correctness" — "**Coercion = emission: the compiler reads a target spec and translates. No separate coercion engine.**" — this doc operationalizes that thesis claim.

**Supersedes** the "Coercion engine" framing in [`docs/thesis/target-grounding-proposal.md`](thesis/target-grounding-proposal.md) §"Substrate work" and the `T-Ground-Engine` lane framing in [`ROADMAP.md`](../ROADMAP.md) and [`docs/briefs/grounding-manager.md`](briefs/grounding-manager.md). Those documents predate the cascade-promotion thesis-discipline tightening (2026-04-25); they describe the work as "engine + selection + tie-breaking" which contradicts THESIS:171.

**Affects in-flight work:** [PR #989](https://github.com/gunb-ai/gunbc/pull/989) "Engine Phase 2" (stern-ant-452 + merry-bat) is implementing inhabitance-search + selection + tie-breaking. This doc supersedes that framing; PR #989 needs realignment per §"Affected lanes" below.

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
- **Substrate facts** = target language specs (per `dsl/extdeps/languages/*/`), declared canonical choices, declared structural ordering
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

### Modeling problem 2 — canonical choice when multiple inhabitants exist at the same refinement

**Question:** when the substrate genuinely declares multiple primitives that inhabit the same algebra at the same refinement, how is the canonical choice declared?

**Worked example.** Rust offers `String` (owned, heap-allocated, mutable), `&str` (borrowed, immutable), `Cow<str>` (lazy clone-on-write). All inhabit `FreeMonoid<Char>` for `.dag` `String`. All are valid emissions of `.dag` `String`. None is structurally "smaller" — they have different ownership semantics, not different sizes.

**Three resolution options:**

**(a) Substrate declares canonical.** A fact in `dsl/extdeps/languages/rust/types.dag` declares `String` as canonical for `FreeMonoid<Char>` when no further refinement is given. User overrides via explicit annotation. The fold reads the canonical-fact; no engine policy.

**(b) Program-side annotation is required.** `.dag` programs that emit to Rust must annotate `String`-shaped values with `: Owned` / `: Borrowed` / `: CowLazy` (or similar substrate-declared refinement). No annotation = fail-closed. No canonical at the language level; user authority via the program.

**(c) Hybrid.** Substrate declares a canonical *plus* the substrate exposes the option set via a typed query so users can see what they can override to. Fold reads canonical when no annotation; reads annotation when present.

**Open call:** which option? **Recommendation: (a) + diagnostic surface naming the override options.** Canonical gives ergonomic defaults; diagnostic surface gives discoverability. (b) is honest but punishes ergonomics for cases where the canonical is obviously right (e.g., `.dag` `String` → Rust `String`).

**The work.** Extending the substrate so each language spec declares (i) which inhabitance is canonical when refinement is silent, (ii) which inhabitances are valid alternates that user annotation can select. This is substrate completion, not engine work.

### Modeling problem 3 — user annotation as program-side substrate

**Question:** when a user writes a target-specific annotation (e.g., `: Vec<T>` instead of letting the fold pick from `Vec<T>` / `[T; N]` / `Box<[T]>`), where does that annotation live structurally?

**The naive answer is wrong.** It would be tempting to say "the annotation lives at the program-target boundary as an emission-time hint." That's an engine-shaped answer — it implies emission has its own state. The correct answer is: annotations live as **typed program-side substrate facts** that compose with the program's structural facts the same way any other declaration does.

**Worked example.** A program declares:
```
data items: List<Int> = [1, 2, 3]
@target(rust) annotate items: Vec<Int64>
```

The `@target` annotation is a substrate-level declaration that *associates* the program-side identity `items` with a Rust-target-specific realization. The fold reads program-side annotations the same way it reads target-side language specs: as declared facts.

**What the substrate must declare:**
- Schema for target-specific annotations on program-side declarations
- Composition rule: program annotation + language spec → unique realization (when both consistent) or fail-closed (when inconsistent — e.g., user annotates `Vec<i64>` but program type is `String`)

**The work.** Designing the program-side annotation substrate. This is genuinely thesis-faithful: emission still reads facts; user authority enters via program-substrate facts; no engine.

### Modeling problem 4 — declared structural ordering ("which is smaller")

**Question:** when the fold needs to pick "minimum" satisfier, what declares the ordering?

**Re-framing:** the fold should *not* "pick minimum." It should look up canonical (Modeling problem 2). But for diagnostics — telling users "your unannotated `Int` could be `i32`, `i64`, `i128`; canonical is `i64`; here are the alternatives" — the substrate needs to enumerate inhabitants in some order.

**The honest answer:** the ordering is *structural*, declared in the substrate. For integer carriers, "smaller" means "narrower bound" — declared by the bound carrier itself. For ownership variants of strings, there is no natural ordering; the substrate declares an enumeration order for diagnostics, not a "minimum."

**The work:** declared ordering exists per *category* of inhabitance, attached to the algebra. For algebras with a natural ordering (cardinality bounds), the ordering falls out of the bound carrier. For algebras without (ownership/lifetime variants), the substrate either declares a diagnostic-only enumeration order or omits ordering and emits all alternates as equally-canonical-eligible.

This is substrate completion. The "minimum-satisfier selection" framing was hiding this modeling decision under engine policy.

### Modeling problem 5 — fail-closed diagnostic surface

**Question:** when the fold fails (no inhabitant; multiple inhabitants without canonical; inconsistent annotation), what does the diagnostic look like?

**The diagnostic is itself a structural fact.** It must name:
- What the program declared (the algebra + refinement that was searched)
- What the substrate declared (the inhabitants found, or the absence)
- What would resolve the under-determinism (canonical to declare, refinement to add, annotation to write)

**Worked example.** User writes `Int` without bounds. Fold runs, finds 7 candidates (i8/i16/i32/i64/i128 + isize/usize wide bound). No canonical because `Int` without refinement has no canonical declared (per recommendation in Modeling problem 2). Fold returns:

```
EmissionDiagnostic::UnderDetermined {
  program_intent: AlgebraInhabitance(OrderedRing, refinement: None),
  candidates: [Int8, Int16, Int32, Int64, Int128, ISize, USize],
  canonical: None,
  resolution_hints: [
    "add refinement bound: `Int(min..max)` will narrow the search",
    "declare canonical at language level: extend dsl/extdeps/languages/rust/types.dag",
    "annotate program-side: `@target(rust) annotate field: Int64`"
  ]
}
```

**The work:** designing the `EmissionDiagnostic` carrier in the substrate. This is small but load-bearing — it's the structural surface for "the substrate is incomplete in this specific way."

### Modeling problem 6 — language spec as substrate

**Question:** what's the *substrate shape* of a language spec?

Today: scattered across `dsl/std/coercion.dag` (schema) + `dsl/extdeps/languages/{rust,python,go}/types.dag` (instantiation tables). Per [`docs/thesis/target-grounding-proposal.md`](thesis/target-grounding-proposal.md), the current shape is "table-driven coercion" and is bootstrap scaffolding to dissolve.

**The structural shape needs:**
- Declared primitive set (with refinement-bound shape per primitive)
- Declared algebra inhabitance per primitive (with refinement parameters)
- Declared canonical choices when multiple primitives inhabit the same algebra at the same refinement
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
- A *cross-target meta-spec* declares: which inhabitances are *required* to be canonical across all Shape A targets? (e.g., "every Shape A target must declare canonical for `FreeMonoid<Char>` at no-refinement, because `.dag` `String` is portable across all targets")
- Without this meta-spec, a target language spec could omit a canonical declaration without diagnostic, breaking cross-target portability silently

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
| 2. Canonical choice | T-Ground-Rust + T-Ground-Python + T-Ground-Go (each declares canonical for its primitive set) | Folds into existing |
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

## Affected lanes (in flight)

[PR #989](https://github.com/gunb-ai/gunbc/pull/989) "Engine Phase 2" (stern-ant-452 + merry-bat) is implementing the prior framing. **Realignment options:**

**(a)** Continue PR #989 with renamed scope as `T-Ground-Coercion-Fold` — but only the structural-fold subset; remove selection logic and tie-breaking; use `EmissionDiagnostic` carrier for under-determinism. This is the smallest change to in-flight work.

**(b)** Pause PR #989 until the modeling problems (1-7) above are scoped as lanes and at least Modeling problem 6 (LanguageSpec schema) lands. PR #989 then consumes the new substrate.

**(c)** Re-dispatch PR #989's worker on the new lane structure — they continue contributing but to the modeling-problem lanes rather than the engine lane.

**Recommendation: (b).** PR #989 currently has under-determined scope because the substrate it should consume doesn't exist yet. Pausing prevents baking in selection logic that will need rework once LanguageSpec schema lands.

This is a Director call (cross-program coordination); see Open call 1 below.

## Open calls

### 1. Director sign-off on no-engine discipline + R2 lane restructure

**Required before:** R2-T-Ground-Engine work continues (PR #989 + any new dispatches under current framing).

**Decision needed:**
- Adopt this design (no engine; modeling problems are lanes)
- Resolve PR #989 status (recommendation: pause until LanguageSpec lands)
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
