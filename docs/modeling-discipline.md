# Modeling Discipline — Practices Implementing the Invariants

> **Restored 2026-06-11.** This file was deleted in the #4192 public-visibility
> flip while still cited as the enforcement authority by INVARIANTS.md,
> DIRECTION-CHECKLIST.md, and live `.dag` 🟡 marks — leaving reviewers approving
> against "Practice N" citations they could not read (the gap that let PR #4627's
> dissolution findings through review). It is restored as the public review
> rubric; references to retired internal ledgers (`src/v2/TASKS.md`,
> `src/v2/DECISIONS.md`, audit worksheets) are re-pointed to PR review and
> dashboard work items.

> Purpose: a short checklist of *modeling practices* that implement the
> five invariant principles declared in [INVARIANTS.md](../INVARIANTS.md).
> The invariants are the reviewer-facing rubric; the practices below are
> the concrete patterns each invariant manifests in modeling work.
>
> **No comment-duplicating ledgers (standing rule, 2026-05-19).** Do not
> create or maintain docs that re-list facts whose source-of-truth is an inline
> model mark (`🟢`/`🟡`/`🔴`, `feature:`/`consumer:` gate, dissolution slug).
> Deleted as maintained mark ledgers: `src/v2/STRUCTURE.md` and
> `docs/audit/dissolution-inventory.md`. `src/v2/DECISIONS.md` is permitted only
> as the bounded P1-KEYSTONE D2-reversal / fact-bundle reseed record, not as a
> carrier-mark ledger. The **inline mark on the carrier** is
> the system; verbose dissolution-pattern analysis and architectural debate live
> in **PR review** (process receipts in the commit message). Unchanged:
> dissolution is merge-blocking (INVARIANTS §P5), yellow must dissolve, Practice
> 10 review heuristic. Non-comment receipts only: [INVARIANTS.md](../INVARIANTS.md)
> §P5(b), PR review / dashboard work items (the `src/v2/TASKS.md` ledger is
> retired), and
> [`docs/modeling/grounding-worked-examples.md`](modeling/grounding-worked-examples.md)
> (*coincide*). The bounded census worksheets that once carried audit exceptions
> (`docs/audit/coproduct-anemia-inventory.md`) were retired with the visibility
> flip; do not recreate competing repo-wide mark sweeps.
> Cost-of-change = 1 per [CLAUDE.md](../CLAUDE.md).
>
> This document supplements, rather than parallels, INVARIANTS.md's
> taxonomy. Each practice names the invariant principle it serves.
>
> **Why these Practices exist.** Every modeling rule below serves one
> thing: making each target's model correct, complete, and honest enough
> that the compiler-**derived homomorphism** between targets is sound.
> Read any Practice as: *this protects the homomorphism.* (THESIS.md →
> "The derived homomorphism"; [the-derived-homomorphism.md](thesis/the-derived-homomorphism.md).)
>
> Full derivations and worked examples live in
> [`docs/modeling/grounding-worked-examples.md`](modeling/grounding-worked-examples.md);
> the background v3 modeling analysis is historical (deleted in the visibility
> flip; recover via git history if needed).

## The three facets — a cross-reference convention

**Fact modeling**, **coercion**, and **translation** are not three
separate topics — they are three facets of the one **derived
homomorphism** ([THESIS.md](../THESIS.md) → "The derived homomorphism";
[the-derived-homomorphism.md](thesis/the-derived-homomorphism.md)):

- **fact modeling** — *produces the homomorphism's inputs.* You model
  each target's facts so the compiler can derive the structure-preserving
  map; the modeling discipline exists to make those inputs correct.
- **coercion** — *the verification facet.* The mechanical fold that
  checks a candidate map preserves structure (coercion ⊂ the
  homomorphism).
- **translation** — *the homomorphism applied.* "Translation" between
  two targets **is** the derived homomorphism.

**Convention.** Wherever a doc *discusses* one of these three as a
topic — at its defining mention / discussion-point, **section-level, not
every token** — frame it as a facet of the derived homomorphism and
cross-ref the THESIS section. The connection should always be one click
away from where the concept is taught; a parenthetical on every
occurrence is noise.

## Modeling Practices

Each practice implements one of the five invariant principles from
INVARIANTS.md. A reviewer works from the five principles; the practices
below are the concrete patterns that inform each check — what to look
for, how to tell whether the invariant is structurally enforced vs
merely behaviorally respected.

Mapping:
- Practice 1 (Fail-closed) — implements **P3: Fail-Closed** (detection behavior)
- Practice 2 (Illegal states unrepresentable) — implements **P2: Boundary Discipline** (structural type shape — make illegal states impossible to construct at the stage boundary, not merely detect them)
- Practice 3 (Facts flow forward) — implements **P2: Boundary Discipline**
- Practice 4 (Coproduct dissolution) — implements **P1: Modeling Faithfulness**
- Practice 5 (Single-authority metadata) — implements **P2: Boundary Discipline**
- Practice 6 (API-level enforcement over convention) — implements **P2: Boundary Discipline**
- Practice 7 (Projection over enumeration) — implements **P1: Modeling Faithfulness**
- Practice 8 (Fact-bundle modeling) — implements **P1: Modeling Faithfulness**
- Practice 9 (No-prose discipline) — implements **P2: Boundary Discipline**
- Practice 10 (Don't hand-roll a derived operation) — implements **P1: Modeling Faithfulness** and the *Do not hand-roll a derived operation* invariant.
- Practice 11 (Parameterize, don't duplicate; respect concept-home boundaries) — implements **P2: Boundary Discipline** + M2 (no duplicate type authorities). The design-time meta-practice that catches duplication and boundary-crossing *before* a downstream worker authors them; upstream of Practice 4 / 5 / 10.
- Practice 12 (A finished stage is a fold; non-fold residue measures unmodeled decision) — implements **P1: Modeling Faithfulness** at **stage scale**; the whole-stage lift of Practice 10. Home of record MODELING.md M11.
- Practice 13 (Encountered concepts are modeled, not deferred — JIT) — implements **P1: Modeling Faithfulness** + **P5: Progress Is Dissolution** (a change that ships an un-dissolved scaffold is not progress). Merge-blocking trigger: a diff that *touches or introduces* a concept it leaves unmodeled (raw `Int`/`String` for a domain quantity, stringly closed-set, bare alias, homeless reference). Home of record MODELING.md M12.

A reviewer should name specifically whether the diff satisfies each
relevant practice, where it could be violated, and whether the existing
checks are structural (type-system enforced) or merely behavioral
(convention).

> **Ledger-doc retirement (operator 2026-05-19):** `src/v2/STRUCTURE.md` and
> `docs/audit/dissolution-inventory.md` are **deleted**, and
> `src/v2/DECISIONS.md` is not a maintained mark ledger — no maintained
> manifests or parallel prose ledgers for facts that belong in inline `//`
> marks. The only live `DECISIONS.md` scope is the bounded P1-KEYSTONE
> D2-reversal / fact-bundle reseed record. **Standing principle:** do not create
> or maintain docs that duplicate comment or model marks; marks plus PR review
> are authoritative. Classification and dissolution **discipline** (Practices 4
> / 9 / 10, INVARIANTS P5) is unchanged. Where older text below names
> `DECISIONS.md` for carrier-mark ledgers, read **PR review, issues, and commit
> messages** unless that sentence was rewritten in this PR.

### 1. Fail-closed

Every failure path goes through the diagnostic mechanism. No silent
`None` returns. No panics on user-reachable paths. No success results
with unresolved state.

**What to check:** If a code path fails, does it fail through a
diagnostic, or does it fail silently? If a function returns `Option<T>`
on error, can the caller distinguish "not yet computed" from "failed"?

**Example violation:** A function returns `None` on error without
writing a diagnostic. The caller sees `None` and may treat it as "not
yet computed."

**Fix:** Replace `None`-on-error with diagnostic writes. Return
`Option<T>` only when the absent case is a legitimate non-error state.

### 2. Illegal states unrepresentable

Data models must not admit combinations that the invariants forbid.
If the reviewer can imagine a combination of field values that
"shouldn't happen," the type is wrong.

**What to check:** Does any combination of field values represent an
illegal state? Is `Option<T>` being used to mean two different `None`s?
Are there product types with mutually exclusive field values?

**Example violation:** `Port { value_type: Option<TypeShape> }` where
`None` means both "not inferred yet" and "inference failed." The
invariant "`None` iff diagnostic exists" is runtime-checked rather
than type-enforced.

**Fix:** `PortState::Uninferred | Resolved(TypeShape) | Unresolved(PortId)`.

### 3. Facts flow forward

Every piece of structured information produced at one stage of the
compiler must be either consumed by the next stage, carried forward as
a field on downstream data structures, or explicitly discarded — with
the justification recorded in **PR review**, in an [INVARIANTS.md](../INVARIANTS.md) §P5 row,
or a dashboard work item (same PR as the change when receipt-shaped);
process receipts go in the commit message (Practice 9), not as
in-file prose. Silent drops are violations.

**What to check:** For each cross-stage boundary touched in the diff
(parse→lower, lower→infer, infer→lens, lens→emit), enumerate the fields
on upstream types and verify each is handled downstream.

**Example violation:** Parser computes `SourceSpan`, lowering discards
it, inference cannot produce located diagnostics. The fact existed
upstream but didn't survive.

**Fix:** Add `span` field to every downstream behavior node. Facts
that die at a boundary either get carried or get justified.

### 4. Coproduct dissolution

Flat coproducts (Rust enums, tagged unions, sum types with N named
variants) are compressed references to richer structure. In a closed
system where we own all the definitions, most coproducts are unfinished
modeling — the richer structure exists, we just haven't written it down.

**Dissolution dispositions — the universal rule.** Every coproduct,
every type modeling a spec primitive (Practice 8), and every function
that could hand-roll a derived operation (Practice 10) is held to one
question — **can this dissolve now? if not, why not?** The answer is
exactly one of three dispositions, the shared 🔴/🟡/🟢 vocabulary used by
Practices 4, 8, and 10. There is no fourth: an unclassified item is
unfinished modeling and blocks merge.

- **🔴 dissolve-now** — it *can* dissolve and nothing blocks it, so it
  **must**. 🔴 is a directive, never a standing state. Dissolution is
  debt, and debt is paid before new work — a 🔴 **jumps the queue**,
  scheduled ahead of feature work in its lane. Fix it in this PR; a 🔴
  the audit finds on merged code is the next thing done.
- **🟢 terminal** — the question is answered by *there is nothing to
  dissolve into*: genuinely irreducible, no richer source exists.
  **Consumer-independent** — a *namable* richer source means it is not
  🟢, regardless of whether anything consumes it today.
- **🟡 gated** — it cannot dissolve now, and the *only* legitimate
  reason is a **named arrival** it waits on. A 🟡 is **transient** — a
  committed surface→dissolve loop, never an indefinite tracked comment.
  Every 🟡 MUST bind a **dissolution plan**, as a **merge requirement**:
  1. **gate kind** — `feature` (a substrate primitive or capability that
     does not exist yet) or `consumer` (the first consumer of the
     decomposed meaning is not here yet);
  2. **the bound dissolution plan** — not merely a name, a *committed
     path to 🟢*: the named missing primitive (or consumer), **the
     substrate PR or task that will land it**, and **the dissolution
     follow-up that converts this 🟡 to 🟢** once it lands. A 🟡 with no
     bound dissolution PR is **not a valid 🟡** — that is the
     comment-graveyard failure mode, and it blocks merge;
  3. **dissolve-on-arrival** — the follow-up dispatches *immediately*
     when the substrate PR lands; the 🟡→🟢 conversion is mandatory, not
     optional. A 🟡 is a pre-committed obligation with a committed exit,
     never a parking spot. Dissolution debt is *burned down*, not banked
     — INVARIANTS P5, "Progress Is Dissolution".

  `feature`- and `consumer`-gating are one dimension, not two — both are
  *waiting on an arrival*; a consumer is itself a kind of feature, the
  awaited thing being the consumer's existence. The label records only
  *what* arrives. A bare 🟡 ("deferred") is the overloaded form this
  rule retires: every 🟡 carries `feature:<name>` or `consumer:<name>` so
  a reader sees at a glance *that* it is gated and *on what*, and the
  audit can check whether the gate has already opened (⇒ the 🟡 is stale
  and flips to 🔴).

Applied to a **coproduct** (any `type X = A | B | …` with N ≥ 2
variants), the three dispositions are:

- **🟢 terminal** — the variants are irreducible distinctions at the
  user-input boundary (literals, keywords, source locations); no richer
  structure can be named. Requires **PR review** (same PR) recording which
  dissolution patterns were tried and why each failed.
- **🟡 gated** — a richer source exists but decomposition waits on a
  named arrival (`feature:` substrate not ready, or `consumer:` no
  consumer reads the meaning yet). A `consumer`-gated entry **pre-assigns
  the obligation**: the first consumer of the meaning owes the
  structural decomposition, not a local lookup. Firing the gate is the
  sanctioned, expected path, not a reopening of settled work.
- **🔴 dissolve-now** — a richer source exists and extraction is cheap;
  do it now, before the next consumer is added.

**Five dissolution patterns to try in order** before classifying as
terminal:

1. **Fact placement.** Variants trace to different consumers or DAG
   locations. Move each variant's payload to the consumer that uses it.
   The coproduct dissolves into scattered fields.
2. **Variant-is-data.** Variants have the same structural shape with
   different labels. Promote the label to a field. Example:
   `LogLevel::Debug | Info | Warn | Error` → `LogLevel { name, priority }`.
   *Guardrail: only valid when the label space is closed and enumerable,
   not when it's free-form string.*
3. **Algebraic form.** Variants trace to different algebraic structures
   (intro/elim, ops over `std/` types). Express each as a reference to
   its `std/` source. Example: `ArithOp::Add | Sub | Mul | Div` →
   `Apply { function: FunctionRef }` pointing to `std::int::add` etc.
4. **Dimensional.** Variants are points in an M-dimensional space.
   Promote the dimensions to fields. Example:
   `EdgeKind::Consumed | Read | Threaded | Projected` →
   `Edge { source_effect, control_role }`.
5. **Parameterized family.** The N variants are mechanically the same
   shape `F<X>` for X ranging over a known, separately-declared set —
   the coproduct is an *enumerated copy of that set*. It is not N
   variants; it is one generic / one projection over X. Replace the
   enumeration with that projection — see **Practice 7**. Example: a
   `Homomorphism` enum with one variant per algebra structure
   (`HomMagma`, `HomMonoid`, … — each `{ source: X<S>, target: X<T>,
   map }`) is the algebra-structure set copied into a second type; the
   faithful form is one generic `Homomorphism<C>` projected over the
   structures. Patterns 1–4 will *not* catch this — they test coproduct
   *shape*, not parameterized mechanical repetition; that is exactly why
   this pattern exists.

**What to check:** Any new coproduct with N ≥ 2 variants (a Rust enum,
or a `.dag` `type X = A | B | …`) must be classified (🟢/🟡/🔴). **The
coproduct itself keeps a required one-line classification tag carrying
the 🟢/🟡/🔴 emoji** (operator directive 2026-05-17) — e.g. `//
🟡 coproduct dissolution`. The emoji stays *on the coproduct* so a
reader sees the classification at the type. 🟡 gate binding is on the in-file tag
(`feature:` / `consumer:`); verbose dissolution-pattern analysis is argued in
**PR review** (or an [INVARIANTS.md](../INVARIANTS.md) §P5 row when
receipt-shaped). A coproduct
with no in-file 🟢/🟡/🔴 tag is unfinished modeling and blocks review.

**The lookup smell (the consumer-trigger backstop).** A `match` over a
foreign-label coproduct, written *inside a consumer* — a lens, a
transform, any file that is not the type's own — to recover a structural
fact, **is the decomposition written in the wrong place.** The match
arms *are* the axis: `Wor => OR, Wand => AND, …` is literally the
`resolution` axis of the net-kind decomposition. The fix is never to
keep the lookup; it is to push that structure into the type — the
`match` in the consumer *is* the first consumer of the meaning, so it
opens a `consumer:` gate: fire it and decompose. A reviewer who sees such a match flags it:
structural content discovered in a consumer belongs in the type, not the
consumer. This is the same channel K-1 closes for `Symbol` — a `match`
on opaque labels is exactly where heuristics smuggle in. Until a
machine-checked meta-lens detects fired triggers, this review smell *is*
the enforcement.

**Scaffold exception:** early-milestone code (marked `// scaffold:
<sunset-milestone>`) can defer only the *verbose* dissolution-patterns-tried
analysis until the sunset milestone. The exception covers that analysis
**only**. It does **not** waive: (a) the required one-line 🟢/🟡/🔴 tag
on the coproduct; nor (b) — for a 🟡 — the **bound dissolution plan**
(the named missing primitive/consumer, its owning substrate PR or task,
and the dissolve-on-arrival follow-up), on the in-file 🟡 tag and in **PR review**.
A 🟡's
plan-binding is the minimum that makes it a *valid* 🟡 (see the 🟡
disposition above). The gate names the **concrete missing primitive**,
e.g. `// 🟡 feature:<missing-primitive>` — a bare `<sunset-milestone>` is
**not** a valid gate (a milestone-only gate leaves the dissolution path
non-checkable); the sunset milestone records *when* the scaffold is
revisited, not *what* it waits on. Scaffolds must be revisited before
sunset.

**Worked example (v2 retrospective):** `v2::ExprData` had 22 variants.
Failed pattern 1 (every consumer dispatches on all 22), pattern 2
(shapes genuinely differ), pattern 3 partial, pattern 4 partial. The
correct dissolution is pattern 3 for computation-carrying variants
(`Apply` over `std/` functions) and pattern 4 for control-carrying
variants (`Branch`/`Loop` as dimensional decompositions). Result: 22→5
reduction. Not "shorter code" — "same information, properly factored."

### 5. Single-authority metadata

Every piece of metadata about the program (types, diagnostics, spans,
provenance) must have exactly one canonical location. Duplicate
representations are violations. Mutator APIs that live on detachable
child objects violate single-authority because the child can be
separated from its parent.

**What to check:** Is there a second representation of any fact? Do
mutator methods live on child objects that hold references to parents?

**Example violation:** `DiagnosticTable::mark_unresolved(&mut dag, ...)`
where the method lives on a child object holding a reference to its
parent. Another `DiagnosticTable` instance could null the parent's
ports without going through the parent.

**Fix:** `Dag::mark_port_unresolved(&mut self, ...)` — method on the
parent. The child provides data; the parent owns mutation.

### 6. API-level enforcement over convention

When an invariant has to hold, the API should make violations
impossible, not merely undesirable. Convention-level enforcement
("please don't do X") fails under cognitive load. The type system
should stop violations, not documentation.

**What to check:** If a new contributor tried to violate the invariant,
would the type system stop them, or would they need to know the rule?

**Example violation:** `Dag::clear_port_type` is `pub(crate)` with a
doc comment saying "only call from `mark_unresolved`." If another
crate-internal caller forgets, the invariant breaks.

**Fix:** Make `clear_port_type` private to the diagnostic module, or
eliminate it entirely by making the state transition atomic at the
data-model level.

### 7. Projection over enumeration

The substrate is a concept DAG. When a concept is a *mechanical
function* of a parent concept — when its content is fully determined by
walking the parent — it is an **edge** (a projection) in that DAG, not a
sibling **node** authored and maintained on its own. Materialising the
derived family as hand-written declarations is a parallel
representation: it copies the parent's structure, and cost-of-change
becomes ≥ 2 — every addition to the parent forces a matching addition to
the copy, and the two drift.

This is Practice 4's constructive sibling. Practice 4 (pattern 5)
*detects* the enumerated-copy smell in a coproduct; this practice is the
shape to reach for, and it is broader than coproducts — it applies to
any family of declarations, enum or not.

**What to check:** For any family of declarations, ask "is each member
`F(X)` for X over a set that already exists somewhere?" If yes — is the
family written as a projection (one generic, one fold, one lens) or
hand-enumerated? Enumeration is the violation. **The test:** if adding
one element to a source set forces a matching hand-edit elsewhere, you
have an enumeration where a projection belongs. Cost-of-change must be 1
(CLAUDE.md "Cost of Change").

**Worked example — `Homomorphism` (the enumerated form, and the fix).**
A first cut declared a 15-variant `Homomorphism` coproduct: one variant
per algebra structure, each mechanically `Hom{X} { source: X<Source>,
target: X<Target>, map: fn(Source) -> Target }`. Every variant is
identical but for the structure name — the enum is the algebra-structure
*set copied into a second type*. It passed all four original Practice-4
dissolution patterns (it genuinely is not a record, not dimensional, …)
and two independent reviews, because those patterns test coproduct
*shape*, not parameterized mechanical repetition. The faithful form is
one generic `Homomorphism<C>`, projected over the algebra structures C;
adding an algebra then costs 1, not 2. (If the substrate cannot carry
the higher-kinded `C`, the fallback is still one type — `Homomorphism`
over an algebra-as-data carrier with same-class as a checked predicate —
never the enumeration.)

**Worked example — `Int` (projection done right).** `Int`'s arithmetic
is never re-listed on `Int`. `Int` inhabits `OrderedRing`; its `add` /
`mul` / `compare` *project from* that algebra-instance — reached by
walking `Int → its OrderedRing<Int> → add`. Likewise `Int64` is not a
hand-authored fixed-width type sitting beside `Int8`…`Int128`; it is
`Int` projected onto a machine-width axis (`Int` composed with
`MachineWidth<Word64>` — see INVARIANTS P1's integer worked example).
The fixed-width integers are a projection over the width set, not eleven
enumerated siblings. "Operations fall out of inhabitance" (THESIS,
epistemic stacking) is precisely this practice: the operations are
projected from the algebra, not enumerated on the type.

### 8. Fact-bundle modeling

To model a thing is to assert its *facts*. A type either **invents** a
fact-bundle for its subject — a `Conj` / record whose fields are the
specific facts the source actually states — or **reuses** an existing
`std/` carrier. There is no third option. A **bare alias**
(`type RustI32 = Int32`) does neither: it asserts an identity it has not
proven while modeling nothing of its own. It is *hollow* — the shape
that looks like modeling and isn't.

Modeling is mandatory; deduplication is conditional. You MUST model the
facts. You may collapse your model onto a `std/` carrier ONLY when you
have **proven** the two coincide — and *coincide* has a precise meaning
(see `docs/modeling/grounding-worked-examples.md`): both groundings, reduced to canonical `Node`s, are
structurally equal, expressed in shared `std/` vocabulary. Identity is
an evidenced claim, never an assumed default. "These are obviously the
same" is not evidence.

**Same rule, opposite default — the default tracks what we *know*:**

- **extdeps** (`extdeps/languages/*`, `extdeps/formats/*`,
  `extdeps/frameworks/*`) — we did not write these systems and do not
  fully know them. **Default SEPARATE:** model each primitive's facts
  honestly from its own spec and let the model accumulate. Reuse a
  `std/` carrier *only* with cited coincidence evidence. Keeping
  `RustSignedness` a separate carrier until Rust's reference is checked
  is *honest modeling*, not a dual-authority (Practice 5) violation —
  the two are not yet proven to coincide.
- **internal** (the compiler layers, our own substrate) — we wrote both
  sides; identity is usually *known*. **Default REUSE `std/`** — but
  still name the evidence. A known coincidence left un-cited is still an
  un-modeled claim.

**What to check:** For every external primitive a `.dag` file models,
does the file *state the facts the spec gives* — width, signedness,
representation, range, encoding, lifetime, … — as a structured carrier?
Or does it write `type Foo = Bar` and stop? A bare alias of a spec
primitive whose spec carries facts is **under-modeled** — flag it.

**Example violation:** `type CppInt = Int`. C++'s `int` is *not* the
abstract integer. The standard states facts the alias discards: an
implementation-defined width of *at least* 16 bits, a signed
representation, an `int`-specific range. `type CppInt = Int` asserts
`CppInt` *is* `std/` `Int` — an identity that is false (C++ `int` is
finite-width and platform-varying; `Int` is not).

**Fix:** invent the facts C++ adds, reuse `std/` for the part that
genuinely coincides:
`CppInt = Conj { base: Int, width: Nat, width_proof: Witness<width >= 16>, representation: Representation }`.
The `base: Int` reuse is licensed because C++ integers *are* integers —
a coincidence on the algebra, cited. The `width` / `representation`
fields are invented because they are facts C++ states that `std/` `Int`
does not carry.

**Why hollow aliases survive review (and why this practice exists).** A
bare alias is *structurally minimal* — it passes every structural gate
precisely because there is nothing there to be wrong. The hollowness is
invisible to a shape-checker: the gate sees a valid `type` declaration.
MODELING.md M1 ("types decompose into facts") already said modeling
means asserting facts; the gap was *enforcement*. This practice, the
worked examples it points at, and the structural fact-density gate below
are that enforcement. A reviewer fails a hollow alias *against this
documented bad example*.

**No string-templating (the emit-side hollow alias).** Emission goes
through grounded format / language models — **never** string templates
or fill-in-the-holes artifacts. A string-templated artifact (e.g. an
`InhabitantDecl.template: "Vec<{0}>"` field) produces output the
compiler cannot ground and cannot coercion-check; it drifts silently,
exactly as the bare alias does. It is the *emit-side* equivalent of the
hollow alias — the same failure, on the way out. The canonical
non-templated form is **grammar-as-declarative-bidirectional-data**: the
production data *is* the relation concrete-syntax ⟷ `Node`, checked in
both directions, never a procedural recognizer or a print template. Any
emit artifact that cannot be grounded is a STOP.

**Worked examples.** The good-vs-bad fact-bundle forms for each external
target — Rust, C++, LLVM, Verilog, SPICE, PTX, Go, JSON, TOML,
OpenAPI, … — are worked in full in
[modeling/grounding-worked-examples.md](modeling/grounding-worked-examples.md).
That document is the companion rubric to this practice; consult it for
the concrete shape of a faithful fact-bundle per target.

#### Interim floor: the hollow-alias discriminator

Enforcement of this practice is **two-tier**. The *structural* tier — a
generated checker (`Node -> Outcome`) that makes a hollow alias
*impossible to construct*, not merely review-discouraged — is its own
named dissolution target, **the structural fact-density gate (T-30)**, a
hard prerequisite of the per-language rework (T-4); both are tracked as
dashboard work items (the TASKS.md ledger is retired). It is deliberately *not* this document: convention is exactly
what let D2 through, so the reseed runs under the structural gate, not a
doc.

What follows is the **interim floor** — the discriminator a *reviewer*
applies by hand until T-30's checker lands, and the spec T-30 then
implements. It is not the structural tier; it is the convention-tier
bad-example, written so a review can fail a hollow alias against it.

A declaration is **hollow** when *all three* hold:

1. It is a **bare alias** (`type X = Y`) or a single-field wrapper that
   adds no field of its own; **and**
2. its subject is an **external spec primitive** — something a
   language / format / framework specification names and states facts
   about; **and**
3. it carries **no coincidence evidence** — no cited row in
   [`docs/modeling/grounding-worked-examples.md`](modeling/grounding-worked-examples.md)
   proving `X` and `Y` coincide, cited from the file by at most a one-line
   tag (Practice 9).

A hollow declaration **blocks review**. The fix is one of: invent the
fact-bundle (now `X` carries ≥ 1 fact of its own), or supply the
coincidence evidence (now the reuse is licensed and cited).

A type that is *under-modeled but legitimately deferred* — the spec
facts are known but a substrate carrier or a consumer is not yet here —
is not hollow; it is **🟡 gated** under the shared Dissolution
dispositions (Practice 4). It carries the gate kind (`feature:` /
`consumer:`), the concrete named arrival, and the dissolve-on-arrival
obligation, exactly as a 🟡 coproduct does. "Hollow" is the *unjustified*
under-model; "🟡 gated" is the *justified, tracked* one — the
discriminator between them is whether a concrete, valid gate is named.

**Exempt:** kernel-ambient atoms — `Bool`, `Char`, and the other
irreducible substrate atoms — are *legitimately* atomic. They state no
further facts because there are none to state; an alias onto them, or
their use as a terminal, is not hollow. The gate's discriminator is
"does the *spec* carry facts this declaration drops?" — for a true atom
the answer is no.

### 9. No-prose discipline

A `.dag` file's comments are **not a parallel prose authority**. After
modeling, a file's comments carry only what a mechanical consumer or a
reviewer needs *to use the file* — never rationale, never narration,
never a record of the work done. Rationale is argued in **PR review**;
process notes live in the commit message. A comment
that records that the file was de-prosed is itself the prose to remove.

**The spec.** After de-prose, a `.dag` file's comments are ONLY these
things — nothing else survives. `// Owns:` is a parallel ledger and is
removed from all `.dag` files; ownership is the module body. `// Consumes:`
is likewise removed; consumption is the import graph. The
strict-deprose allowlist files regenerated by `scripts/strict_deprose_dag.py`
carry `// Consumes:` as script-authored input for the header-check gate,
`// Ledger:` as script-generated coproduct inventory, and the emoji-only
RULING-1 slice line (`// 🟢` or `// 🟡`) as the operator-ratified
groundedness marker. Those header fields are generated artifacts, not
hand-maintained prose ledgers.

1. **Line 1** — the file-path line.
2. **A terse header** — ordinary files may keep `Scope:` and `Status:`
   one-liners when they are useful boundary signals. The strict-deprose
   allowlist files additionally carry `Consumes:`, `Ledger:`, and
   emoji-only RULING-1 slice lines. Nothing else: no `Brief:`, no
   `Seams:`, no `HEADER RECONCILE`, no `Deferred (N)` rationale, no
   multi-line block.
3. **A per-carrier or header anchor** — at most one `// Anchor: <spec URL>` line
   for the relevant carrier or file authority.
4. **A one-line tag.** Two cases:
   - **Required — coproduct classification tag.** Every coproduct (a
     `type` with N ≥ 2 variants) carries a one-line tag with its
     🟢/🟡/🔴 classification emoji (operator directive 2026-05-17), e.g.
     `// 🟡 coproduct dissolution`. The emoji stays *on the coproduct*
     (Practice 4). 🟡 gate binding is on the in-file tag; verbose analysis in
     **PR review**. This is not optional — a coproduct with no in-file 🟢/🟡/🔴
     tag blocks
     review.
   - **Optional — concept tag / cite.** For any type, at most one
     further one-liner where genuinely useful: a concept tag where the
     concept is non-obvious from name + structure, *or* a one-line
     coincidence cite (Practice 8) when reuse is non-obvious.
   Never a description of the type; never a `Practice N: ...` rationale
   line; never a `see docs/X` pointer.

Everything else is **removed**: per-type descriptions, all Practice-N
rationale, all multi-line rationale, `Seams`/`Brief`/process-meta
blocks. Architectural decisions land in PR review or
[INVARIANTS.md](../INVARIANTS.md) §P5; process notes — de-prose receipts,
"HEADER RECONCILE", "per directive X" — move to the
**commit message**, never the file.

**What to check:** count `comment-lines / total-lines`. The hard target
is that a modeled `.dag` file is roughly **under 20% comment lines**. A
file substantially above 20% has not been de-prosed — the pass is
nominal, not real. (A file audited at 58% comment lines *after* a
"de-prose" pass is a failed pass; verify the percentage, do not accept a
nominal pass.) Load-bearing files keep the terse header contract — but
that header *is* the whole of item 2, not a license for more. The <20%
figure is a **heuristic** for prose bloat, not a hard
floor: a small carrier file (under ~25 lines) whose mandated four-line
header alone exceeds 20% is compliant if that header is all the comments
are. Never pad a file to lower the percentage — content-compliance
(comments are *only* the four allowed things) is the real bar.

**Why:** prose in the file is a second authority. It drifts from the
structure it narrates; it is the
documentation-side hollow alias (Practice 8) — it looks like modeling
and isn't. The structure *is* the model; the terse header is only a
boundary signal. Machine-readable ownership/consumption ledgers survive
only where a generator owns them; everything else is removed.

**Supersession — Practice 9 governs every in-file artifact.** Several
earlier Practices were written when an in-file comment *was* the
enforcement mechanism: a discard justification (Practice 3), a coproduct
classification + ledger/trigger (Practice 4), a coincidence-evidence
proof (Practice 8). Practice 9 supersedes all of them, under one uniform
rule:

- the **record relocates** — the inline 🟢/🟡/🔴 mark (and `feature:`/`consumer:`
  gate on 🟡) stays on the carrier; verbose dissolution-pattern analysis and
  architectural debate are argued in **PR review**; coincidence proofs land in
  [`docs/modeling/grounding-worked-examples.md`](modeling/grounding-worked-examples.md);
  hand-Rust / test receipts in [INVARIANTS.md](../INVARIANTS.md) §P5(b);
  task-scoped substrate in dashboard work items. Process
  receipts (`HEADER RECONCILE`, "per directive X", a de-prose note) move to the
  **commit message**;
- the `.dag` file keeps the **item-4 one-line tag** — for a coproduct, a
  *required* 🟢/🟡/🔴 classification tag (e.g.
  `// 🟡 coproduct dissolution`); optionally one further concept tag or
  coincidence cite. The classification *emoji* stays on the coproduct.

Wherever an earlier Practice says "record X in a comment," read it as
"keep the one-line tag on the carrier; land non-comment receipts per the
header block; argue verbose analysis in PR review." D5's
`HEADER RECONCILE` receipt moves to the commit message. No earlier
in-file *artifact mandate* survives un-superseded by Practice 9. This
does not mean the file carries no comments at all: Practice 9 itself
*authorizes* the four allowed classes — including the **required**
one-line 🟢/🟡/🔴 coproduct tag (item 4).

### 10. Don't hand-roll a derived operation

The compiler's job is to **derive** operations from a model — that is the
derived homomorphism (top of this doc; THESIS.md). There is a *finite*
set of such derived operations. Hand-rolling one re-derives what the
compiler already derives once: the deficiency is in the **model**, not
the code. The fix is never to polish the hand-rolled construct — it is to
model the missing fact, or name the missing substrate primitive, and let
the operation be derived.

**The invariant this implements** (see [INVARIANTS.md](../INVARIANTS.md)
P1 and [MODELING.md](../MODELING.md) M1):

> **Do not hand-roll a derived operation.** If a function's behavior is
> determined entirely by the shape of a modeled type, it is re-deriving
> something the compiler already derives. The deficiency is in the model,
> not the code — model the missing fact; do not hand-roll the operation.

**The derived-operations registry.** The registry is what makes "you
should be modeling, not coding this" an *objective* call rather than a
reviewer's taste — a hand-rolled instance of any row is a finding:

| # | derived operation | derived from | hand-rolled form → finding | substrate primitive |
|---|---|---|---|---|
| 1 | structural recursion (catamorphism) | a type's structure | walker dissolution | `fold_node` |
| 2 | effect traversal | carrier + collection | traverse dissolution | `traverse` / `sequence` |
| 3 | translation (target → target) | two target models — *this is the derived homomorphism* | hand-written emitter / lowerer | the compiler itself |
| 4 | coercion | the structure-preservation fold — the homomorphism's verification facet | hand-written coercion check | the compiler itself |
| 5 | identity / hashing | a Merkle catamorphism | hand-written hash / equality | `content_hash` |
| 6 | emission + parsing | grammar-as-bidirectional-data | string-templated emitter → emit/template dissolution | the grammar model |
| 7 | property projection | reading a fact off the model | `match`-to-derive → predicate dissolution | a model fact |

**Rows 3 and 4 carry no numbered dissolution finding — on purpose.**
Translation **is** the derived homomorphism; coercion **is** its
verification facet (the three facets, top of this doc). Hand-writing a
cross-target translator or a coercion checker is not a function-scale
review smell a reviewer flags in one diff — it is "you re-wrote the
compiler," a whole-architecture failure caught at architecture review.
The numbered findings below are all *function-scale*: visible in a single
diff. Rows 3/4 are named in the registry for completeness, not as a
reviewer finding.

**The rows-3/4 escalation has a concrete trigger: a new file under
`src/v2/compiler/`.** "Caught at architecture review" is not a standing
gate anyone runs — so the trigger is structural: **a PR that adds a new
file under `src/v2/compiler/` is itself a review finding**, default-block.
Compiler code orchestrates folds over modeled facts; it carries no
per-target, per-operation, or per-fixture knowledge — token layouts,
operator catalogs, arm shapes, and binding names are model data in
`std/` / `extdeps/`. The author must state in the PR, and the reviewer
must accept by name, why the file cannot be rows plus an existing fold.
(INVARIANTS P2 §"Target knowledge in compiler code".)

**Worked example (PR #4627, the rubric-gap receipt).** A 728-line
`compiler/06_value_expression.dag` landed through an approving review
while this document was deleted. Against the registry it carries, in one
diff: *predicate dissolution* — `arrow_has_transform_body`, a `Bool`
helper over Arrow used to gate dispatch at four sites (the property is a
fact the translation rules should carry); *walker dissolution* — bespoke
indexed list-walkers duplicating `list_at_optional`; *carrier/parametric
duplication* (Practice 11) — two hand-rolled find-unique-row folds whose
`Missing | Unique | Ambiguous` accumulator **is** `find_witness`;
*emit/template dissolution* — a `[BoundToken, FixedToken, BoundToken]`
token layout coded in the compiler instead of carried on the realization
row; and boundary violations — target-layer atoms (`^ts_inhabitant_number`)
and fixture binding names (`^dag_binding_param_x`) compared literally in
compiler code. Every one is a named row above; none was citable during
review because the registry was unreadable. (Live corroboration,
2026-06-11: a later #4627 revision reintroduced literal
`^ts_inhabitant_number` / `^ts_inhabitant_field_atom` atoms at new
compiler sites, self-marked 🟡 — this time caught and blocked under the
restored INVARIANTS P2 "Target knowledge in compiler code" rule, with the
fix routed through `declared_inhabitants`.) The four dissolves are
blocking preconditions of the emit-breadth milestone.

**Dissolution findings.** A *dissolution finding* names a hand-rolled
construct that re-derives what the model should provide; its fix always
has the same shape — the construct **dissolves** into *(a substrate
primitive) + (model data)*. Most members are the detection-rubric for a
smell another Practice already names — this is **not** a parallel
vocabulary:

- **coproduct dissolution** — *is* Practice 4, the established finding.
- **carrier dissolution** — a sharpened Practice 4 sub-case: a local
  coproduct that clones an existing `std/` carrier (a `Foo { value } |
  FooRejected { diagnostic }` that *is* `Outcome<T>`). Before classifying
  a coproduct for dissolution, check it against the `std/` carrier set —
  a match makes the finding the sharper, mechanical "delete it, use the
  std carrier," not generic "model this as facts." **See Practice 11
  for the *parametric* generalization** — two operations or carriers
  that differ only by a typed parameter (predicate, domain, policy) are
  parametric duplication, not structural-identity duplication; carrier
  dissolution catches the latter, Practice 11 catches the former.
- **predicate dissolution** — Practice 8 / Practice 7 at the level of a
  *code predicate*: a `match`/`if` on kind or symbol whose purpose is to
  *derive a property* ("is this a binder?", "which sugar is this?")
  rather than to do structurally distinct work. The property is a fact
  the model should carry and the code should *read*. Canonical shape: a
  coproduct discriminant — `free_monoid_non_empty` hand-rolling `match xs
  { Empty => false ; Cons => true }`, or `nat_is_zero` hand-rolling
  `Zero => true ; Succ => false`, derives "which variant" by hand where
  the coproduct already carries it. Mechanical trigger: any new `is_*`,
  `has_*`, `*_is_*`, `*_has_*`, `non_empty`, `is_empty`, or similar
  `Bool` helper over a coproduct that `match`es the value and returns
  `true` for one variant and `false` for another is predicate
  dissolution until proven otherwise. On a substrate / `std/` / reusable
  algebraic helper this is unconditionally blocking.
- **walker dissolution** *(new)* — Practice 7 lifted from
  declaration-families to *traversal*: a function that hand-rolls
  recursion over a structural type (`Node`, AST) — per-node-kind `match`
  arms each re-walking children. The same homomorphism re-implemented per
  stage instead of derived once; the faithful form is a std catamorphism
  (`fold_node`) + a supplied algebra.
- **traverse dissolution** *(new)* — a `fold` whose body is a `match acc
  { Rejected => propagate ; Ok => continue }` ladder: effect-threading
  (failure, short-circuit, accumulation) hand-inlined where `traverse` /
  `sequence` over the effect carrier belongs. Almost always co-occurs
  with walker dissolution — the fold body both recurses *and* threads the
  effect.
- **emit/template dissolution** — the *finding* form of Practice 8's "no
  string-templating" rule: a string-templated emitter
  (`template: "Vec<{0}>"`) where grammar-as-declarative-bidirectional-data
  belongs. The emit-side mirror of walker dissolution.

**The inverse direction — nominalization.** Every finding above detects
*under-modeling*: a hand-rolled construct that should be modeled or
derived. **Nominalization** is the opposite error — *over-modeling*: an
operation declared as a *type*. It is camouflaged precisely because it
*looks* like good modeling — it is *more* type declaration, not less —
so a reviewer scanning for "is this modeled enough?" reads it as
compliant, even exemplary. The rubric checks **both** directions; the
over-modeling direction must be looked for on purpose.

- **nominalization** *(new)* — an operation (a function), or a derived
  operation, declared as a *type*. The tell is a struct whose only
  field(s) are functions and which has no `data` instances —
  `type ListMap<T, U> { apply: fn(List<T>, fn(T) -> U) -> List<U> }`.
  **Discriminant: does the type have more than one meaningful
  inhabitant?** A genuine algebraic *structure* does — `Monoid<T>` has
  the additive monoid, the multiplicative monoid, … — so it is a
  legitimate type. A combinator does not: there is exactly one
  list-`map`. *A type with exactly one meaningful inhabitant is an
  operation in disguise — it must be a `fn`* with a real body, or a
  derived operation. The degenerate single-field `{ field: T }` wrapper
  repeated across N near-identical subjects (e.g. `{ spelling: String }`
  across the seven URI components) is the same finding — N hollow
  wrappers standing where modeled facts belong. Disposition: normally
  🔴 — rewrite as a `fn`; 🟡 only if it should be a *derived* operation
  and the derivation primitive is absent.

**Disposition — per the shared Dissolution dispositions (Practice 4).**
Every dissolution finding — and every audited function that turns out
*not* to be one — carries one of the three dispositions 🔴/🟡/🟢. Naming
it is what stops a reviewer from wrongly demanding the impossible:

- **🔴 dissolve-now** — the substrate primitive the construct should use
  *already exists* (e.g. `Outcome<T>` in `std/diagnostic.dag`). The fix
  is mechanical, belongs in this PR, and jumps the queue.
- **🟡 gated** — the substrate primitive does *not* exist yet (e.g.
  `fold_node` / a fail-closed `traverse`). A derived-operation 🟡 is
  almost always `feature`-gated: the gate names the missing primitive
  *and its owning task*, and the 🟡 carries the dissolve-on-arrival
  obligation (shared rule, Practice 4). It is **BLOCKING unless** that
  gate is recorded as a tracked, named upstream obligation on a declared
  honest scaffold — an untracked 🟡 is silent dissolution debt and blocks
  (Calibration section; INVARIANTS P5 "Progress Is Dissolution").
  Re-blocking a *tracked* 🟡 is pointless churn — blocking cannot land an
  absent primitive.
- **🟢 clean / terminal** — audited and *not* a dissolution finding: the
  recursion or `match` is genuinely irregular (the call graph is not the
  data graph), or the construct already uses the derived operation.

The retroactive v2 dissolution audit applies this legend per-file,
per-finding — a symbol-marked inventory, not prose. The symbol records a
finding's *disposition*; the matching in-file `.dag` tag lands with the
fix, per migration PR — it is not retro-applied across all v2 files at
once.

**Decidability — checker-flaggable vs reviewer-judgment.** Every
dissolution finding is **blocking** — there is no advisory tier and no
nit channel. A finding is resolved only by 🔴 dissolve-now, a tracked
🟡, or a substantiated 🟢; it is never resolved by a free-text
"intentional" / "short-circuiting" dismissal. The column below records
only *who* flags a finding — a checker can mechanically hard-error the
structural ones, the judgment ones a reviewer must decide — but a
reviewer who identifies a finding blocks the PR exactly as a checker
would:

| finding | decidable? | enforcement |
|---|---|---|
| carrier dissolution | structural — type-shape match vs the `std/` carrier set | **hard error** |
| walker dissolution | structural on the clean shape (recursion mirrors a modeled type) | **blocking** — hard error on the clean shape; genuinely-irregular recursion (call graph ≠ data graph) is a clean 🟢, not an advisory |
| traverse dissolution | structural on the clean shape (a `fold` body that is a carrier short-circuit ladder) | **hard error** on the clean shape |
| emit/template dissolution | structural — a literal template-string field | **hard error** on the literal-template shape |
| nominalization | structural — a struct whose only fields are functions with no `data` instances, or N near-identical single-field wrappers | **hard error** on the wrapper shape |
| predicate dissolution | judgment — a `match` *may* be genuinely distinct work, not a derived property | **blocking** — a reviewer who identifies it blocks the PR; a `match` that is genuinely distinct work is a clean 🟢. No advisory / candidate tier. |
| coproduct dissolution | already enforced — per-coproduct 🟢/🟡/🔴 tag (Practices 4 / 9); rationale in PR review | already enforced |

The *enforcement mechanism* — the checker-script build path and the
eventual dissolution lens — is design work (the prior design doc was
deleted in the visibility flip; recover via git history). This Practice
carries only the
classification, which is discipline a reviewer applies by hand. Worked
examples live in
[modeling/grounding-worked-examples.md](modeling/grounding-worked-examples.md).

**What to check.** For any function in the diff: is its behavior fixed by
the *shape* of a modeled type rather than by logic unique to this call
site? If yes, it is a candidate dissolution finding — identify the
registry row, then mark the disposition (🔴 dissolve-now / 🟡 gated /
🟢 clean).
For predicates, verification helpers, and structural walkers, look for
direct matches on lower-layer representation (`Empty`/`Cons`, enum
variants, field conventions, ad hoc list traversal) when a canonical
fold, accessor, query, or substrate fact already exists or should exist.
Do several functions repeat the same recursion? Does the PR call the
helper a "refinement predicate", "short-circuiting primitive", or
"matches sibling style" without explaining why that requires a separate
walker? Those are not sufficient answers: preserve semantic requirements
such as short-circuiting in the canonical surface, and treat existing
sibling helpers with the same shape as accumulated debt, not precedent.
A coproduct's 🟢/🟡/🔴 tag does not disposition a predicate over that
coproduct; predicate dissolution lives on the consumer function and
needs its own disposition.
**Not when** the recursion or `match` is genuinely irregular — the call
graph is not the data graph, the branches do genuinely distinct work.
Irregularity is the honest escape hatch: a derived operation is one whose
shape *is* the data's shape.

### 11. Parameterize, don't duplicate; respect concept-home boundaries

The compiler's primitive set is **small by construction.** Operations,
carriers, and witnesses that *look* like new concepts are usually
parameterizations of existing primitives with a different typed argument.
Before authoring a new operation, carrier, or witness, exhaust the
question:

> Is this an instance of an existing one, with the per-instance
> variation expressed as a typed parameter?

If yes, it is not a new thing — it is a *call site* of the existing
primitive with a parameter.

**The invariant this implements** (see [INVARIANTS.md](../INVARIANTS.md)
P2 and M2 in [MODELING.md](../MODELING.md)):

> **No duplicate type authorities — including parametric duplicates.**
> If two operations or carriers have the same structural shape differing
> only by domain (which language / which target / which file) or by
> predicate / policy (which check, which preservation rule, which
> resolution strategy), that is **one** parameterized operation/carrier
> with the difference as a typed parameter. Apply recursively — to the
> architecture's own carrier set, not just to user-program modeling.

**This is upstream of the dissolution findings in Practice 10.** Carrier
dissolution catches "you re-authored `Outcome<T>`" — *structural identity*
match. Predicate dissolution catches "you hand-rolled `match` on a
variant" — *shape match against the data*. Practice 11 catches the
**meta-pattern those don't see**: *you authored two
operations/carriers/witnesses with similar-but-distinct shapes where one
parameterized declaration would do.* The discriminant is not structural
identity — it is **parametric similarity**: two declarations whose only
difference is a typed parameter.

**Mechanical trigger.** When authoring or reviewing a new substrate
declaration, ask:

1. **Is there an existing declaration with the same shape modulo one
   typed parameter?** If yes, the new declaration is a parameterization;
   the substrate primitive is the parameterized form, and the new
   declaration is a *call site*, not a sibling.
2. **Are you authoring two declarations in this PR (or the same wave)
   that differ only by predicate / domain / policy?** If yes, collapse
   them into one parameterized declaration with the variation as a typed
   parameter; the original two names become call sites in glossary, not
   rows in the primitive table.
3. **Would a reader of your design doc see your two declarations as "the
   same thing in different domains"?** If yes, the doc itself is
   mis-naming — the primitive is the parameterized shape, and the
   domain-named instances belong as glossary call sites, not as primitive
   declarations.

**Worked examples (load-bearing — drawn from real v2 corrections).** Each
row was a real design-review correction; the point of Practice 11 is to
catch them at brief-authoring, not after a worker has implemented them
under a mis-shaped brief.

| Domain-named pair/triple (the wrong shape) | Parameterized form (the right shape) | What the parameter is |
|---|---|---|
| `solve_constraints`, coercion fold, `LawfulRewriteWitness` | `find_witness(facts, candidates, predicate, multiplicity_policy)` | preservation predicate + multiplicity policy |
| `CanonicalGroundingWitness`, `AcyclicityWitness`, `ClosedWorldDependencyWitness` | `StructuralPropertyWitness<P>` | the structural property `P` |
| `Witness<homomorphism>`, `LawfulRewriteWitness` | `HomomorphismWitness<R>` | the preservation rule `R` |
| `BootstrapWitness` + `FixedPointWitness` (two carriers gating one transition) | `PromotionWitness` (composed gate) | which two sub-checks compose |
| `traverse_node`, `sequence_node`, `bind_outcome` (as primitives) | `fold_node` + an Outcome-threading algebra (derived combinators) | the algebra-as-data |
| Three distinct MVP terminuses for the same compile path | One MVP | (collapsed) |
| `ProgramSchedulingEdge`, `TargetPlanSchedulingEdge`, `ArtifactSchedulingEdge` | one `DependencyEdge` with a `DependencyKind` label | the kind |
| `DependencyGraph` as a separate authored carrier | Edge-on-Node parameterized by `DependencyKind` | (the carrier dissolves entirely — first-order facts live as Edge labels on the Node DAG) |
| `TopologicalPlan` / `ReadinessLayer` as authored ledger data | lens output — `fold_node` over Node reading dependency edges | (the carrier dissolves to a derived view; no parallel ledger) |

**Connection to existing practices.**

- **P2 / M2 (single authority)**: Practice 11 sharpens "no duplicate type
  authorities" to mean *no parametric-pair authorities either*. Two
  operations with the same shape modulo a parameter are still two
  authorities — collapse them.
- **Practice 4 (Coproduct dissolution)**: Practice 11 generalizes
  coproduct dissolution from "flat enum with parametric structure" to
  "set of operations / carriers / witnesses with parametric structure."
  A coproduct *dissolves* into richer structure; a parametric pair
  *dissolves* into one parameterized declaration.
- **Practice 10 (Don't hand-roll a derived operation)**: Practice 10
  catches *hand-rolled instances* of a registered derived operation —
  the implementation-time error. Practice 11 catches the *registration
  of a derived operation as a primitive in the first place* — the
  design-time error upstream of it. Without Practice 11, Practice 10
  catches the symptom (hand-rolled emit-templates) while leaving the
  cause (a domain-named "primitive" that's actually a parameterization)
  in the doc, ready to seed the next round.

**Disposition.** A Practice 11 finding is **always BLOCKING** at the
design-doc / brief-authoring stage; merging carries it forward into
every downstream worker's brief, where it propagates as N implementation
findings. Standard 🔴/🟡/🟢:

- **🔴 dissolve-now** — the parameterized form already exists, or can be
  authored in this PR; replace the duplicated declarations with the
  parameterized one + N call sites.
- **🟡 gated** — the parameterized substrate primitive doesn't yet
  exist (e.g. `find_witness` before Pass B); the gate names the missing
  primitive + its owning task; the 🟡 carries the dissolve-on-arrival
  obligation (shared rule, Practice 4). An untracked Practice 11 🟡 is
  the most expensive class of silent dissolution debt because it
  propagates into every downstream worker brief.
- **🟢 clean** — audited and *not* parametric duplication: the two
  declarations have shapes that genuinely differ in more than one
  dimension, or the structural shape itself differs.

#### Parallel-payload vs typed-reference (Practice 11 sub-rule)

A second-order Practice 11 failure that doesn't show up as "two
domain-named primitives" but as **one row carrying both a typed
reference to substrate AND a parallel copy of the referenced fields**.
Operator-direct standing 2026-05-19, ratified for the T-13 family
ratchet 2026-05-21 (crisp-boar-896 Phase 1, `ClassifiedDependencyView<C>`
in `v2.std.dependency`).

Worked instance (T-13 lens family pre-ratchet):

```
type ParallelismDependencyFact {
  source: Node            // ← parallel payload, also at dependency.source
  dependent: Node         // ← parallel payload, also at dependency.dependent
  dependency: DependencyView   // ← typed reference (the authority)
  source_facts: Witness<InferredFacts>     // ← parallel payload, also at tree.facts.lookup(dependency.source)
  dependent_facts: Witness<InferredFacts>  // ← parallel payload, also at tree.facts.lookup(dependency.dependent)
  relation: ParallelismRelation
}
```

The row carries a `DependencyView` reference *and* re-copies its
endpoints under semantic names (`source`/`dependent` / `at`/`owner` /
`use_site`/`declaration`). That is **Practice 2 illegal-state-
representable** (P2): the product type admits `source != dependency.source`
while still type-checking. The defect is not redundancy — it is a
parallel authority for endpoint identity. (Sharpening: even
topologically-neutral copied names like `source`/`dependent` are still
parallel-payload — the carrier above the typed reference IS the
authority; any field that duplicates one of its fields is a second
authority. The unused-parameters variant is worse: semantic names like
`use_site`/`declaration` import `BindsTo` semantics onto every
`DependencyKind`, so a `Contains` row is an illegal state *by
construction*.)

Practice 3 cousin: per-node `InferredFacts` "carried on the row"
duplicate a lookup against `tree.facts` — the row should hold the
typed reference (`dependency`) and consumers read facts at use-sites
through `tree.facts.lookup(dependency.source)`, not from a frozen copy
on the row (Facts Flow Forward — facts flow by reference, never by
copy).

**Mechanical test (when authoring/reviewing).** Does the row combine
a typed reference to a substrate carrier with one or more fields
whose values are derivable from that reference? If yes, the row is
parallel-payload — remove the derived fields; provide free-function
accessors (`classified_source`, `classified_dependent`, …) that
project through the typed reference. The carrier should be
parametric over the lens-specific projection:

```
type ClassifiedDependencyView<C> {
  dependency: DependencyView
  classification: C   // lens-specific (UseRelation | ParallelismRelation | OwnershipMode | …)
}
```

— one substrate primitive, N call sites. This is the same Practice 11
"one parameterized declaration vs N domain-named siblings" shape
applied to a family of *lens projection rows*; the parameter is the
classification.

**Disposition.** Parallel-payload findings are BLOCKING under the
same Practice 11 disposition table (🔴 / 🟡 / 🟢). Recurring instances
across a family escalate to a **family-wide ratchet PR** (substrate
primitive + N call-site migrations + claim updates in one coherent
unit) — the T-13 ratchet is the worked precedent.

**Recurring instances** (memory-grounded; sweep on each new sighting):

| # | Instance | Resolution |
|---|---|---|
| 1 | PR #3448 `DependencyView` | merged |
| 2 | `FileReadReceipt` rename (PR #3447) | merged |
| 3 | `WholeDagFailClosed` rename (PR #3451) | merged |
| 4 | `Extent::WholeFile` drop (PR #3452 reshape) | merged |
| 5 | T-13 `*DependencyFact` family | **this ratchet — `ClassifiedDependencyView<C>` substrate** |

#### Concept-home boundary discipline (Practice 11 companion)

A second meta-rule, structurally distinct enough to call out separately
but in the same "look before you author" family:

> Before adding a field to a substrate file, confirm the field doesn't
> cross a boundary that file's identity is supposed to keep separate.

A file in `extdeps/file_system.dag` models the external file-system
resource. It should not know about compiler `Node`. Adding a
`NodeFileBinding { node: Node, path: FilePath }` to that file crosses the
boundary from "external resource model" into "compiler provenance" —
the relation is real (some `Node` came from some file), but it doesn't
belong in the resource model; it belongs in artifact / projection /
ingest. The same pattern recurs in disguise wherever a substrate file's
identity is implicit: when "file_system" silently becomes
"file_system_plus_provenance," the boundary the file's name encodes has
been crossed.

**Mechanical trigger.** Before adding a field that references a type from
outside the file's concept-home, ask:

1. **Does this field describe the file's concept?** (`file_system.dag`
   should describe files, not file↔Node relationships.)
2. **If the relation is real, where does it belong?** Most often: a
   *different* concept-home (provenance / artifact / projection /
   ingest), not the resource-model file.
3. **Is an existing violation precedent for adding a new one?**
   Strict-mode answer: no. Surface the existing violation as its own
   finding; don't deepen it.

**Disposition.** Standard 🔴/🟡/🟢:

- **🔴** — the right home exists; move the declaration there.
- **🟡** — the right home doesn't exist yet; gate on landing it, with a
  named owning task.
- **🟢** — the field genuinely belongs in this file (audited).

**Worked examples.**

| Wrong home | Right home | What was misplaced |
|---|---|---|
| `NodeFileBinding` in `extdeps/file_system.dag` | `std/artifact.dag` / `std/projection.dag` / ingest-side | Node-to-file *provenance* in the *resource* model |
| `DependencyKind` taxonomy collapsed inside one of the domain-specific edge files (`ProgramSchedulingEdge` etc.) | `std/dependency.dag` (single authority for the kind taxonomy) | classification authority placed inside one consumer |
| Per-stage `StageDiagnosticPolicy` values authored inside the consuming stage files | `std/pipeline.dag` (stage rows carry the values; type lives in `std/diagnostic.dag`) | values + type co-authored at the call site instead of in the substrate home |

The boundary check is mechanical *given the substrate concept-home
list*: each `.dag` file has an implicit identity (what concept it
models), and fields/imports outside that identity are the smell.
Practice 11's companion is the design-time discipline that keeps each
concept-home single-authority.

#### Why Practice 11 is the design-time meta-practice

Practices 1–10 are *file-scale* — a reviewer reads one diff and checks
whether the diff conforms. Practice 11 is *design-scale* — a reviewer
reads the **brief** that produced the diff and asks whether the brief
itself names duplicates as primitives or asks a worker to cross a
concept-home boundary.

A Practice 11 violation in a design doc propagates as N file-scale
violations across every worker the brief dispatches. The cost is
asymmetric: catching it in the design doc deletes the violation once;
catching it after dispatch costs N rounds of corrections + N worker
contexts that need to be re-briefed. **Block Practice 11 findings at the
design-PR layer, even when no implementation hunks are touched.**

### 12. A finished stage is a fold; non-fold residue measures unmodeled decision

Implements **P1: Modeling Faithfulness** and is the **stage-scale**
reading of Practice 10 (*don't hand-roll a derived operation*). Practice
10 is function-scale — it flags one hand-rolled catamorphism in a diff.
Practice 12 is the same finding lifted to the **whole stage**: a compiler
stage's finished shape is one fold over its model,
`stage(x) = fold_carrier(x, algebra(model))` — or a thin zero-residue
composition of such folds (e.g. `serialize_target ∘ translate`) — and the
*volume of non-fold control-flow in the stage* is a measurable proxy for
how much of its decision-making still lives in code instead of the model. (See
[MODELING.md](../MODELING.md) M11 — home of record; THESIS.md "Modeling
discipline"; the frontloaded collapse program
`gunbc-planning/stage-fold-collapse-plan-2026-06-11.md`.)

**The litmus, not a ban.** Non-fold code is permitted but sorts into
**exactly two** categories — there is no third, and "stage logic that's
fine to keep as control-flow" is not a disposition:

- **🟢 named irreducible kernel** — a fixpoint solver, char-class
  matching, real arithmetic: not a catamorphism, *meant* to stay one
  named separated function. *Fold the traversal, name the kernel.*
  Terminal, not debt. The discriminant is the same one Practice 10 uses
  for "genuinely irregular recursion": the **call graph is not the data
  graph**. A solver iterates to a fixpoint; its control flow is not the
  shape of its input.
- **🟡 / 🔴 un-migrated modeling** — any other non-fold arm: a `match`
  that derives a property, an `if` that special-cases, a `_go`
  accumulator, a `_bounded` fuel parameter. Each is the code making a
  decision the model has not absorbed; it dissolves to *(an algebra row)
  + (the fold carrier)*. 🔴 if the carrier exists (`fold_node` /
  `fold_grammar_expr` / a frontend fold); 🟡 if the carrier is a named
  missing primitive (gate + owning task + dissolve-on-arrival, per
  Practice 4).

**What to check.** For a stage file (`src/v2/compiler/*`): does the stage
body hand-walk its input, or is every decision pushed into the model? The
finished shape is **zero decision-residue**, realized in one of two forms
— both pass:

- a single `fold_carrier(x, algebra(model))` call (the archetype: one
  traversal, every former arm is an algebra row); or
- a **thin composition of fold-backed stages** whose only glue is monadic
  sequencing (`bind_outcome` / `∘`) and which adds no `match`, `if`, `_go`,
  or `_bounded` of its own. The composition is plumbing, not a decision —
  each composed stage carries its own fold.

Count the residue signals — `_go` accumulators, `_bounded` fuel sites,
per-connective `match` arms *in the stage* (vs rows in the algebra; a
`bind_outcome` chain is not an arm). Each non-zero count is either a named
kernel (🟢, justified by call-graph ≠ data-graph) or modeling debt
(🔴/🟡). A stage that grows its arm count, or grafts a fold *beside*
surviving arms without deleting them, is the **cementation anti-pattern** —
the deletion ratchet (the target file must shrink, `_go`/`_bounded`
strictly down) is its mechanical enforcement. The exit certificate is
numeric: `_go`=0, `_bounded`=0, no per-case `match` in the stage, every
former arm is one algebra row. `05_emit` (`emit = serialize_target ∘
translate`, i.e. `bind_outcome(translate, serialize_target)`, 43 lines) is
the landed existence proof of the **composition** form — zero residue, no
hand-walk, both halves fold-backed.

**Disposition.** Standard 🔴/🟡/🟢 per the shared dispositions
(Practice 4). A 🟢 stage-kernel claim must substantiate call-graph ≠
data-graph; absent that, the residue is debt, not a kernel.

## Calibration: Blocking vs Omit

A finding is **BLOCKING** if fixing it in a later PR would be meaningfully
harder than fixing it now — i.e., if merging this PR commits the project
to a shape that is expensive to change.

Do not use a nit/advisory finding as a third category. If a concern is
valid and PR-relevant, request changes. If it is not serious enough to
require action, omit it from the review.

**Substrate-level issues are almost always BLOCKING** because the
substrate sets patterns that get copied. Once a bad shape propagates
through three consumers, changing it means changing all three plus the
substrate.

**Performance issues are usually omitted** unless they change interfaces,
make a bound false, or create a concrete invariant violation.

**Test coverage gaps depend:** gap in a high-risk invariant → BLOCKING;
gap in a low-risk area → omit.

**When in doubt, prefer BLOCKING.** It is better to ask for a small
rework now than to accept a substrate bug that propagates through three
milestones before anyone notices.

**A 🟡 mark records debt; it never authorizes it.** Marking a finding 🟡
converts nothing into a pass — it is the *reviewable form* of the debt,
not an exemption from review. Concretely: (a) a PR that introduces new 🟡
marks must **enumerate them in the PR description** (mark, gate, bound
dissolution plan), and the reviewer accepts each one **by name** — an
approval that does not mention a new 🟡 has not accepted it; (b) a 🟡's
dissolve-on condition must bind to a **named milestone gate, PR, or
dashboard work item** that blocks on it — a free-floating "later" gate is
the comment-graveyard failure mode and blocks merge (Practice 4); (c)
**aggregate debt is itself a reviewable dimension**: the default posture
under INVARIANTS P5 is debt-negative — a PR should dissolve at least as
many 🟡 marks as it adds, and one that piles on more requires explicit
justification in review, not just per-mark validity. A PR whose primary
deliverable could be model rows but lands as 🟡-marked compiler code is
the wrong shape, not tracked debt.

**Dissolution findings (Practice 10) are an always-BLOCKING class.** A
dissolution finding — walker / traverse / predicate / carrier /
emit-template — is resolved only by a 🔴 / tracked-🟡 / substantiated-🟢
disposition (Practice 10), never graded advisory and never waved off as
a cleanup or by free-text "intentional, no code change."

**Practice 11 findings are always BLOCKING at the design-PR layer**
even when no implementation hunks are present. A parametric-duplication
or concept-home-boundary violation in a design doc propagates as N
file-scale violations across every worker the brief dispatches; the
cheapest place to fix it is the design PR. A Practice 11 finding is
resolved only by 🔴 / tracked-🟡 / substantiated-🟢, like Practice 10.

## For Reviewers

A review applies the **five invariant principles from
[INVARIANTS.md](../INVARIANTS.md)**. The modeling practices in this
document are the concrete patterns that inform each check — consult them when a
principle's abstract statement needs a recognizable failure shape.

For each relevant principle and its implementing practices:

1. Name specifically whether the diff satisfies the invariant.
2. If violated, cite the exact file and line.
3. State whether the existing check is structural (type system enforced)
   or merely behavioral (convention).
4. For new coproducts: verify the coproduct carries its required
   one-line 🟢/🟡/🔴 classification tag *in the file* (Practice 4 /
   Practice 9 item 4) **and** states dissolution patterns / gate binding in
   **PR review** when non-obvious. **Merge requirement — every 🟡 binds a
   dissolution plan:** the gate kind (`feature:`/`consumer:`), the named
   missing primitive/consumer, the substrate PR or task that will land
   it, and the dissolution follow-up that converts the 🟡 to 🟢. A 🟡
   with a vague gate ("deferred", "later substrate") **or no bound
   dissolution PR** blocks merge — an indefinite 🟡 is the comment
   graveyard. A 🟡 whose gate has *already* opened (the feature landed /
   the consumer exists) is stale — it is 🔴, dissolve now. Also
   verify Practice 4 pattern 5 (parameterized family) was applied — an
   enum that is `F<X>` per variant is an enumerated copy, not a
   coproduct. Verify 🟢 is consumer-**independent**: a namable richer
   source ⇒ 🟡, never 🟢. Flag any `match` over a foreign-label coproduct
   inside a *consumer* as a misplaced decomposition (the lookup smell).
5. For any new family of declarations (enum or not): verify it is a
   projection over its source set (Practice 7), not a hand-enumeration —
   the cost-of-change test is adding one element to the source set.
6. For any type modeling an external spec primitive (Practice 8): verify
   it is a fact-bundle (invents the facts the spec states) or a *cited*
   coincidence reuse of a `std/` carrier — not a bare alias. Apply the
   structural fact-density gate: a bare alias of a spec primitive with
   no PR-reviewed coincidence evidence is hollow and blocks. For any emit
   artifact: verify it is grounded grammar-as-data, not a string
   template.
7. For any `.dag` file in the diff (Practice 9): count `comment-lines /
   total-lines`. A modeled file substantially above ~20% comment lines
   has not been de-prosed — the comments must reduce to the allowed
   comment classes from Practice 9 (file-path line, terse header,
   per-carrier or header anchor, one-line tag; plus generated allowlist
   `Consumes:`, `Ledger:`, and emoji-only RULING-1 slice lines
   where `scripts/strict_deprose_dag.py` owns them). Process-meta prose
   in the file (`HEADER RECONCILE`, de-prose receipts) is itself a
   finding.
8. For cross-stage boundaries: verify facts flow forward.
9. For any function whose behavior is fixed by a modeled type's shape
   (Practice 10): identify the derived-operations registry row it
   hand-rolls. For a row that carries a numbered dissolution finding
   (rows 1 / 2 / 5 / 6 / 7), name the finding and mark its disposition
   (🔴 dissolve-now / 🟡 gated / 🟢 clean); a 🟡 finding names its
   `feature:` gate — the missing `std/` primitive + owning task — and is
   **BLOCKING unless** that gate is recorded as a tracked, named upstream
   obligation on a declared honest scaffold; an untracked 🟡 is silent
   substrate debt and blocks (see Practice 10's disposition legend and
   the Calibration section). A hand-rolled registry row 3 (translation) or 4 (coercion)
   carries **no** numbered finding — it is a whole-architecture
   escalation, not a function-scale review finding (per Practice 10's
   rows-3/4 carve-out). For any `is_*`, `has_*`, `*_is_*`, `*_has_*`,
   `non_empty`, `is_empty`, or similar `Bool` helper over a coproduct,
   verify the function itself has a 🔴/🟡/🟢 disposition; the coproduct's
   tag alone is not enough.
10. Classify every finding as BLOCKING or omit it per the calibration
    above.
11. For any **design PR** or **brief PR** (a PR whose hunks include
    `docs/design-*.md`, planning docs, or brief templates): apply
    Practice 11. Enumerate the new operations / carriers / witnesses /
    edge labels the diff introduces, and for each ask "is this a
    parameterization of an existing one?" A 🔴 / 🟡 / 🟢 disposition is
    required per declaration. Also verify the **concept-home boundary
    discipline**: every new field's type-imports stay inside the file's
    concept-home, or carry an explicit cross-home justification.
    Practice 11 findings are BLOCKING at the design-PR layer even when
    no implementation hunks exist (Calibration section above).
12. For any **compiler stage file** (`src/v2/compiler/*`) in the diff
    (Practice 12): is the stage body one `fold_carrier(x, algebra(...))`
    call — or a thin zero-residue composition of fold-backed stages
    (`∘` / `bind_outcome` glue, no added control-flow) — or does it
    hand-walk its input? Count the residue — `_go`
    accumulators, `_bounded` fuel, per-connective `match` arms in the
    stage. Each is a 🟢 named kernel (substantiate call-graph ≠
    data-graph) or 🔴/🟡 un-migrated modeling. A stage whose arm count or
    line count grows, or that grafts a fold beside surviving arms, is the
    cementation anti-pattern — BLOCKING; the deletion ratchet (file
    shrinks, `_go`/`_bounded` down) is the enforcement.
13. For **any** PR (Practice 13 — [MODELING.md](../MODELING.md) M12, *JIT
    modeling*): does the diff **touch or introduce** a concept it leaves
    unmodeled? Trigger shapes — a raw `Int`/`String` standing in for a
    domain quantity (bytes, hertz, price, duration), a stringly value for
    a closed set (M4), a bare alias asserting zero facts (M1), or a
    reference to a concept with no proper home (M10). For each, the
    author must **model it in this PR** (identity + cited facts / catalog,
    per M3/M9/M10). A `P-*` / TODO / "dissolve later" mark that ships the
    raw placeholder is **NOT an acceptable disposition** — it is the
    deferral M12 forbids, and it is **BLOCKING**. The only non-modeling
    dispositions that pass: (a) the value is a genuine modeled *fact*
    (e.g. an `Option` that is really `None` — absence grounded, not
    fabricated, per M5), or (b) modeling is gated on upstream
    compiler/interpreter capability that does not yet exist — in which
    case the correct action is to **escalate the blocker**, not merge the
    placeholder. "Tracked for a follow-up PR" does not satisfy (b).
    Distinguish from *scope* deferral (minimal-slice, consumer-triggered):
    deferring unbuilt **breadth** is fine; deferring the **modeling of a
    concept the diff already touches** is the violation. Cite M12 /
    Practice 13 in the failing review so the obligation is unambiguous.

This document is the distilled version of modeling principles. For the
full analysis and additional worked examples, see
[`docs/modeling/grounding-worked-examples.md`](modeling/grounding-worked-examples.md).
