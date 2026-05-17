# Modeling Discipline — Practices Implementing the Invariants

> Purpose: a short checklist of *modeling practices* that implement the
> five invariant principles declared in [INVARIANTS.md](../INVARIANTS.md).
> The invariants are the reviewer-facing rubric; the practices below are
> the concrete patterns each invariant manifests in modeling work.
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
> Full derivations, worked examples, and the background modeling analysis
> live in [v3-modeling-analysis.md](v3-modeling-analysis.md).

## Nine Modeling Practices

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

A reviewer should name specifically whether the diff satisfies each
relevant practice, where it could be violated, and whether the existing
checks are structural (type-system enforced) or merely behavioral
(convention).

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
the justification recorded in `src/v4/DECISIONS.md` (Practice 9), not as
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

Every enum with N ≥ 2 variants must be classified as one of:

- **🟢 GREEN (terminal)** — no richer source exists. The variants trace
  to irreducible distinctions at the user-input boundary (literals,
  keywords, source locations). Requires a **ledger entry**: a written
  record of which dissolution patterns were attempted and why they
  failed. **GREEN is consumer-independent.** "No consumer needs the
  decomposition yet" is *not* a basis for GREEN — only "no richer source
  *exists*" is. The test: if you can *name* the richer structure the
  variants project over — the axes, the source set — then a richer
  source exists, and the coproduct is **not** GREEN, regardless of
  whether anything consumes that structure today. A faithful enumeration
  of a spec's surface labels whose meaning decomposes into namable axes
  (e.g. a net-kind enum that is `{resolution, default, drivers}`) is
  YELLOW, not GREEN — see the next bullet.

- **🟡 YELLOW (scaffold)** — a richer source exists but the
  decomposition is deferred. Requires a **named trigger**: the specific
  condition that un-defers it. The trigger is *either* (a) substrate
  work that isn't ready yet, *or* (b) **the first consumer of the
  *meaning***. Decomposition is correctly bounded by consumers — do not
  model meaning nothing reads — so an enumeration whose decomposition is
  *known* but not yet *needed* is YELLOW-deferred-on-consumer, never
  GREEN (the richer source exists; only the work is deferred). A
  consumer-triggered YELLOW entry must **pre-assign the obligation**: it
  states explicitly that the first consumer of the meaning owes the
  structural decomposition — *not* a local lookup. Firing that trigger
  is the sanctioned, expected path, not a reopening of settled work.

- **🔴 RED (dissolvable-now)** — richer source exists and extraction
  is cheap. Do it immediately, before the next consumer is added.

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
or a `.dag` `type X = A | B | …`) must be classified (🟢/🟡/🔴), with a
ledger entry if GREEN or a named trigger if YELLOW. Per Practice 9 the
classification *ledger* and the *trigger* live in `src/v4/DECISIONS.md`
— *not* an in-file `Practice 4: ...` block. **But the coproduct itself
keeps a required one-line classification tag carrying the 🟢/🟡/🔴
emoji** (operator directive 2026-05-17) — e.g. `// 🟡 coproduct
dissolution — DECISIONS.md OS-1`. The emoji stays *on the coproduct* so
a reader sees the classification at the type; the decision-making (the
ledger, the dissolution patterns tried, the named trigger) lives in
`DECISIONS.md`. A coproduct with no in-file 🟢/🟡/🔴 tag, or no
`DECISIONS.md` classification entry, is unfinished modeling and blocks
review.

**The lookup smell (the consumer-trigger backstop).** A `match` over a
foreign-label coproduct, written *inside a consumer* — a lens, a
transform, any file that is not the type's own — to recover a structural
fact, **is the decomposition written in the wrong place.** The match
arms *are* the axis: `Wor => OR, Wand => AND, …` is literally the
`resolution` axis of the net-kind decomposition. The fix is never to
keep the lookup; it is to push that structure into the type — fire the
YELLOW trigger and decompose. A reviewer who sees such a match flags it:
structural content discovered in a consumer belongs in the type, not the
consumer. This is the same channel K-1 closes for `Symbol` — a `match`
on opaque labels is exactly where heuristics smuggle in. Until a
machine-checked meta-lens detects fired triggers, this review smell *is*
the enforcement.

**Scaffold exception:** early-milestone code (marked `// scaffold:
<sunset-milestone>`) can skip the classification annotation until the
sunset milestone. Scaffolds must be revisited before sunset.

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
(see DECISIONS.md): both groundings, reduced to canonical `Node`s, are
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
task, **TASKS.md T-30**, a hard prerequisite of the per-language rework
(T-4). It is deliberately *not* this document: convention is exactly
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
3. it carries **no coincidence evidence** — no `src/v4/DECISIONS.md`
   entry proving `X` and `Y` coincide, cited from the file by at most a
   one-line tag (Practice 9).

A hollow declaration **blocks review**. The fix is one of: invent the
fact-bundle (now `X` carries ≥ 1 fact of its own), or supply the
coincidence evidence (now the reuse is licensed and cited).

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
never a record of the work done. Rationale lives in `src/v4/DECISIONS.md`
(single authority); process notes live in the commit message. A comment
that records that the file was de-prosed is itself the prose to remove.

**The spec.** After de-prose, a `.dag` file's comments are ONLY these
four things — nothing else survives:

1. **Line 1** — the file-path line.
2. **A terse header** — exactly four lines: `Scope:` (one line),
   `Owns:` (carrier *names* only, no rationale), `Consumes:` (one line),
   `Status:` (one line). Nothing else: no `Brief:`, no `Seams:`, no
   `HEADER RECONCILE`, no `Deferred (N)` rationale, no multi-line block.
3. **A per-carrier anchor** — at most one `// Anchor: <spec URL>` line.
4. **A one-line tag.** Two cases:
   - **Required — coproduct classification tag.** Every coproduct (a
     `type` with N ≥ 2 variants) carries a one-line tag with its
     🟢/🟡/🔴 classification emoji (operator directive 2026-05-17), e.g.
     `// 🟡 coproduct dissolution — DECISIONS.md OS-1`. The emoji stays
     *on the coproduct*; the ledger / dissolution patterns / named
     trigger live in `DECISIONS.md` (Practice 4). This is not optional —
     a coproduct with no in-file 🟢/🟡/🔴 tag blocks review.
   - **Optional — concept tag / cite.** For any type, at most one
     further one-liner where genuinely useful: a concept tag where the
     concept is non-obvious from name + structure, *or* a one-line cite
     to a `DECISIONS.md` entry (e.g. `// coincides: <DECISIONS.md ref>`
     — the Practice-8 coincidence cite).
   Never a description of the type; never a `Practice N: ...` rationale
   line; never a `see docs/X` pointer.

Everything else is **removed**: per-type descriptions, all Practice-N
rationale, all multi-line rationale, `Seams`/`Brief`/process-meta
blocks. Architectural decisions move to `src/v4/DECISIONS.md`; process
notes — de-prose receipts, "HEADER RECONCILE", "per directive X" — move
to the **commit message**, never the file.

**What to check:** count `comment-lines / total-lines`. The hard target
is that a modeled `.dag` file is roughly **under 20% comment lines**. A
file substantially above 20% has not been de-prosed — the pass is
nominal, not real. (A file audited at 58% comment lines *after* a
"de-prose" pass is a failed pass; verify the percentage, do not accept a
nominal pass.) Load-bearing files keep the terse four-line header
contract — but that header *is* the whole of item 2, not a license for
more. The <20% figure is a **heuristic** for prose bloat, not a hard
floor: a small carrier file (under ~25 lines) whose mandated four-line
header alone exceeds 20% is compliant if that header is all the comments
are. Never pad a file to lower the percentage — content-compliance
(comments are *only* the four allowed things) is the real bar.

**Why:** prose in the file is a second authority. It drifts from the
structure it narrates and from `DECISIONS.md`; it is the
documentation-side hollow alias (Practice 8) — it looks like modeling
and isn't. The structure *is* the model; the terse header is the single
machine-readable boundary contract; everything else is removed.

**Supersession — Practice 9 governs every in-file artifact.** Several
earlier Practices were written when an in-file comment *was* the
enforcement mechanism: a discard justification (Practice 3), a coproduct
classification + ledger/trigger (Practice 4), a coincidence-evidence
proof (Practice 8). Practice 9 supersedes all of them, under one uniform
rule:

- the **record relocates** — an architectural decision, a classification
  *ledger*, a discard justification, a coincidence proof all move to
  `src/v4/DECISIONS.md`; a process receipt (`HEADER RECONCILE`, "per
  directive X", a de-prose note) moves to the **commit message**;
- the `.dag` file keeps the **item-4 one-line tag** — for a coproduct, a
  *required* 🟢/🟡/🔴 classification tag (e.g.
  `// 🟡 coproduct dissolution — DECISIONS.md OS-1`); optionally one
  further concept tag or a one-line `// coincides: <DECISIONS.md ref>`
  cite. The classification *emoji* stays on the coproduct; only the
  *ledger / patterns-tried / named trigger* relocate.

Wherever an earlier Practice says "record X in a comment," read it as
"record X in `DECISIONS.md`; the file keeps the one-line tag." The same
applies to `DECISIONS.md` rules that mandated an in-file block — D5's
`HEADER RECONCILE` receipt moves to the commit message. There is no
in-file artifact mandate anywhere that Practice 9 does not override.

## Calibration: Blocking vs Non-blocking

A finding is **BLOCKING** if fixing it in a later PR would be meaningfully
harder than fixing it now — i.e., if merging this PR commits the project
to a shape that is expensive to change.

A finding is **NON-BLOCKING** if it's a cleanup that can land later at
roughly the same cost.

**Substrate-level issues are almost always BLOCKING** because the
substrate sets patterns that get copied. Once a bad shape propagates
through three consumers, changing it means changing all three plus the
substrate.

**Performance issues are almost always NON-BLOCKING** because they can
be optimized later without changing interfaces.

**Test coverage gaps depend:** gap in a high-risk invariant → BLOCKING;
gap in a low-risk area → NON-BLOCKING.

**When in doubt, prefer BLOCKING.** It is better to ask for a small
rework now than to accept a substrate bug that propagates through three
milestones before anyone notices.

## For Reviewers

A review applies the **five invariant principles from
[INVARIANTS.md](../INVARIANTS.md)**. The nine modeling practices in this
document are the concrete patterns that inform each check — consult them when a
principle's abstract statement needs a recognizable failure shape.

For each relevant principle and its implementing practices:

1. Name specifically whether the diff satisfies the invariant.
2. If violated, cite the exact file and line.
3. State whether the existing check is structural (type system enforced)
   or merely behavioral (convention).
4. For new coproducts: verify the coproduct carries its required
   one-line 🟢/🟡/🔴 classification tag *in the file* (Practice 4 /
   Practice 9 item 4) **and** has a `DECISIONS.md` entry for the ledger /
   patterns-tried / named trigger — the emoji on the type, the
   decision-making in `DECISIONS.md`. Also verify
   that Practice 4 pattern 5 (parameterized family) was applied — an
   enum that is `F<X>` per variant is an enumerated copy, not a
   coproduct. Verify GREEN is consumer-**independent**: a namable richer
   source ⇒ YELLOW (with a consumer trigger + pre-assigned obligation),
   never GREEN. Flag any `match` over a foreign-label coproduct inside a
   *consumer* as a misplaced decomposition (the lookup smell).
5. For any new family of declarations (enum or not): verify it is a
   projection over its source set (Practice 7), not a hand-enumeration —
   the cost-of-change test is adding one element to the source set.
6. For any type modeling an external spec primitive (Practice 8): verify
   it is a fact-bundle (invents the facts the spec states) or a *cited*
   coincidence reuse of a `std/` carrier — not a bare alias. Apply the
   structural fact-density gate: a bare alias of a spec primitive with
   no coincidence-evidence `DECISIONS.md` entry is hollow and blocks. For any emit
   artifact: verify it is grounded grammar-as-data, not a string
   template.
7. For any `.dag` file in the diff (Practice 9): count `comment-lines /
   total-lines`. A modeled file substantially above ~20% comment lines
   has not been de-prosed — the comments must reduce to the four allowed
   things (file-path line, terse four-line header, per-carrier anchor,
   one-line tag: the required 🟢/🟡/🔴 tag on each coproduct plus an
   optional concept tag / `DECISIONS.md` cite). Process-meta prose in the file
   (`HEADER RECONCILE`, de-prose receipts) is itself a finding.
8. For cross-stage boundaries: verify facts flow forward.
9. Classify every finding as BLOCKING or NON-BLOCKING per the calibration
   above.

This document is the distilled version of modeling principles. For the
full analysis and additional worked examples, see
[v3-modeling-analysis.md](v3-modeling-analysis.md).
