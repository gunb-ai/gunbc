# gunbc — Design

The one document a reviewer or implementer reads to understand what we are building and the
rules that keep it honest. Read it top to bottom. Everything else is code, `ROADMAP.md` for
where we are, and git history for how we got here.

---

## 1. What we are building

**A program is a dependency graph, and the compiler is a non-executing causal engine over it.**
Meaning lives in the *types and their structure*, not in runtime behavior. The compiler validates
that the graph is structurally coherent — types, arity, units, effects, complexity, ownership,
idempotency, and any dimension a user declares — without ever running the program to find out.
Emission is then mechanical. The only thing that can break a compiled program is external reality
that was never in the graph.

**The bet: model local, derive global.** A normal toolchain hand-writes an adapter per
language/format pair — N×M glue that drifts. We model each target *once* in shared vocabulary and
*derive* the translations by comparing groundings. N models, not N×M adapters. An unfaithful
translation is not a silent best-effort; it is a located diagnostic.

**Emission and ingestion are the same operation, run both ways.** Both are *coercion* — a
structure-preserving search over declared inhabitants, performed by the compiler instead of a
hand-written adapter. Emitting coerces the IR into a target's declared inhabitants; ingesting
coerces a source language's model into the IR. It is a **total decision procedure**, never a proof
search: every attempt either produces a structure-preserving witness or fails closed with a located
mismatch from a *closed* taxonomy (no target candidate / would lose information / opaque atom with
no per-target realization). Faithful refinements (widening a fixed-width `i32` into an arbitrary
`int`) carry a witness; lossy ones (narrowing `int` into `i32`) fail closed.

**Two shapes of target, never blurred.** *Shape A* — programming languages and HDLs — the compiler
emits directly. *Shape B* — YAML, Terraform, SQL, SPICE, English — is a user `.dag` program walking
typed values to produce an artifact. Treating a Shape B artifact as if the compiler emitted it
(or vice versa) is a category error.

This is a compiler, not a competitor to Rust/Python/Go — those are *targets*. It is decidable, not
heuristic.

---

## 2. The substrate (the closed vocabulary)

Everything reduces to a small, closed set. Surface syntax is sugar; it never adds semantic power,
and the vocabulary closes here — new language features are new *models*, not new substrate.

- **Two primitives:** `Node` (identity + structure) and `Edge` (a directed, outgoing-only
  reference). Product, coproduct, cardinality, and truth are *patterns of edge connectivity* — there
  is no separate truth primitive.
- **Six type connectives:** Atom, Conj, Disj, Arrow, Cardinality, Instantiation.
- **Five behaviors:** Value, Transform, Branch, Loop, Bind. This is a typed *total* fragment of the
  lambda calculus + structural coproducts + bounded recursion (the totality choice of Coq/Agda/Idris).
- **Bounded forward execution** is the premise the whole system rests on: time flows forward,
  execution walks a bounded structure and never revisits. Cyclic *relations* are expressible (acyclic
  encodings keyed by stable ids, traversed under a finite measure); cyclic *values* are not.
  Recursion is sugar over a bounded `Loop`. Decidability, termination, and cost analysis all *fall
  out* of this premise rather than being separately proved.
- **Names are opaque.** Never branch on a node's name to make a structural decision. Identity rides a
  dedicated channel (a binding id), never the structure-or-spelling.
- **Files leave the pipeline early.** The semantic IR is file-agnostic, with content-hash identity;
  surface text, AST, and IR stay distinct representations.

---

## 3. The principles (the governing spine)

These are the review criteria. Every finding in review traces to one of these. They are stated as
first principles, not as a numbered rulebook — the old C-/E-/DB- ladders compressed into these.

### P1 — Modeling Faithfulness
Every construct grounds in an identifiable external fact or a structural derivation from one;
ungrounded constructs are not valid authorities. **Grounding is intersubjective:** point at a
framework with shared agreement — a mathematical structure, a standardized machine representation, a
cited spec — not at an internal taxonomy we invented (that just restates the claim at another layer).
**In a closed system, a heuristic is never structurally necessary.** We wrote every layer, so the
richer source the heuristic approximates either exists or can be written; a heuristic is evidence of
a missing upstream fact, not proof the fact is unknowable. This recurs at every scale — a per-fact
heuristic, a hand-rolled derived function, and a stage full of `match` arms are the same symptom.

### P2 — Boundary Discipline
Every fact lives in exactly one authoritative place, and boundaries carry enough declared information
for a mechanical consumer. The cost of a change is proportional to how many files encode the same
fact; the governing metric is that changing one concept touches **one** file.
**The layer DAG is strict:** `std/` (universal primitives) ← `extdeps/` (external vendor/language
models, import `std/` only) ← `compiler/` (pipeline stages) ← `workflow/` (orchestration). Imports
point only toward `std/`; a reverse edge is a violation even if it "happens to work." A consumer must
not reverse-engineer a lower layer's storage shape, and there is exactly one authority for equality —
a per-consumer include/skip rule is parallel-authority drift that silently fabricates false-equals.

### P3 — Fail-Closed
Every path either succeeds fully or fails with a typed, located diagnostic. No fabricated plausible
output — no null sentinel for a missing argument, no `<error:unknown>` placeholder type, no silent
clone to dodge an ownership gap, no `"Unknown"` string. A scan-all-and-hope fallback that "usually
works" is *worse* than failing: it fabricates evidence. Every defensive guard is a modeling gap that
should be pushed into the structure so the bad state is unrepresentable. Distinguish honest carriers:
a bounded "forever" (2^63−1) is not "unknown" (a hard fail-closed error) — never collapse them.

### P4 — Decidability
Every accepted program stays within a closed, bounded substrate whose correctness questions are
structurally decidable; **lowering is the receipt.** The analyzer is a *checker of proofs*, not a
*discoverer*: a cycle in a lowered graph that should be acyclic is a defect report against lowering
("fix the compiler"), never a budget to widen around. No fixpoint or fuel for cases that are
impossible by construction.

### P5 — Progress Is Dissolution
A change counts as progress only if it reduces ad-hoc state, duplicate authority, or implicit
behavior. This is not a production codebase: there are **no bridges, deprecations, or staged
migrations as a steady state.** A representation change lands atomically — new form exists, old form
deletes, every consumer migrates, in one change. Every scaffold lands with a *named, checkable
dissolution trigger* and a live strictly-decreasing ratchet *now* — a "pending" with no live ratchet
is a bridge, not a scaffold. The default review posture is debt-negative: dissolve at least as much
ad-hoc state as you add.

### Decomposition (first-class)
*Nothing is opaque that isn't genuinely atomic.* A representation is **over-compressed** when it
hides structure its grounded source actually names. "Coproduct dissolution" and the "stringly-atom"
smell are one principle, named in two eras — we coined the first before we modeled atoms, so the
leaf side had no name. Decomposition is three moves that always travel together:

1. **Decompress** — reveal grounded structure.
   - *Sum side:* a sum whose every inhabitant carries every variant is a product in disguise →
     `Cost = Time | Space | Energy` becomes `{ time, space, energy }`.
   - *Leaf side:* a `String`/`Int` standing for named parts becomes those parts →
     `socket: "LGA4926"` becomes `CpuSocket { package: LandGridArray, contact_count: 4926 }`,
     each part a token a cited source names.
2. **Map** — for each revealed part, DFS the existing concept DAG (this *is* M9, §4). The part
   attaches to the concept that **already exists**. A part is atomic *here* only if grounded **and**
   no richer existing concept covers it — so `energy` lands on the existing `Joules`, never a fresh
   `Int`.
3. **Reduce** — consolidate the duplicates the map surfaces. Two names for one concept collapse to
   one when they *coincide* (reduce to structurally-equal canonical `Node`s) — `Joulezzzz` → `Joules`;
   `Arch | CpuArchitecture | TargetArchitecture` → one ISA-family concept.

**The invariant that keeps it from being self-defeating: net concepts must not grow by
re-invention.** A decomposition that mints a new authority for a concept that already exists is a
*failed* decomposition — it traded one compression for one duplication. Correct decomposition is
concept-count neutral-or-negative. This single principle subsumes four older criteria: DFS-before-
defining (the map step), the no-nickname rule (the reduce step), and single-authority (the net-
concept constraint).

**Why we can be stricter than a normal compiler.** A normal compiler's leaves are `i32`/`String` —
opaque by design; it cannot tell a genuine atom from a compressed one, and "are these two the same
concept" is not even expressible. Our leaves are *grounded fact-bundles with citations* and the
concept DAG is declared, so both questions have structural answers. The same closure that makes
decompression terminate makes consolidation decidable in principle. *(Open, deliberately parked:
whether a lens can mechanically diagnose the leaf side. The principle is the review criterion either
way.)*

---

## 4. Why grounding is load-bearing (epistemic stacking)

Every concept is a node in an ontological DAG rooted at minimal primitives, and **operations fall
out of inhabitance** — they are never declared per type. `Int` inhabits `OrderedRing`, so `add` and
the total order come for free; sign semantics come from the witness, not a flag. Downstream, **the
epistemic chain *is* the emission algorithm**: to emit a value you walk its grounding. Every special
case an emitter needs is evidence of an ungrounded concept upstream — the fix is to model the missing
fact, not to grow the emitter.

This is why there is **no third option for a concept**: it is either a genuine primitive (a logical
or algebraic root, or a real user-input boundary) or an unfinished composition. "Just treat it as
opaque" is never an answer inside the system — opacity is for layer boundaries (§6), not for
ducking the modeling.

---

## 5. How to model (the day-to-day discipline)

- **DFS the concept DAG before inventing vocabulary (M9).** Before declaring a new type, search from
  `std/` for the concept it should attach to. This applies to operations too. Use the canonical CS
  name from the field (with a cited anchor in the module header); a type named for something it isn't
  is a *nickname* and a modeling violation (`ModulePath` was `FreeMonoid<Symbol>`, a qualified name,
  not a path through a graph).
- **Decomposition how-to (§3) is the daily move:** decompress → map → reduce. The decompression
  patterns: fact-placement, variant-is-data, algebraic-form, dimensional, parameterized-family. The
  test for whether something dissolves: *can you name the coordinate space it tags?* If yes, it
  dissolves; if no, it is a terminal user-input boundary and the coproduct stays.
- **Fact-bundle modeling — invent or reuse, never bare-alias.** A type modeling an external spec
  either *invents* a fact-bundle (a `Conj` of the facts the spec states) or *reuses* a `std/` carrier
  **on proven coincidence** (the two reduce to structurally-equal canonical `Node`s). Default
  *separate* for `extdeps/` (we don't fully know those systems), *reuse* for internal layers (we
  wrote both sides) — always naming the evidence. `type RustI32 = Int32` is *hollow*: it asserts an
  identity it never proved and silently drops the spec's facts (width, signedness, representation). A
  hollow alias passes every shape-checker precisely because minimality is invisible to one.
- **Project, don't enumerate. Don't hand-roll a derived operation.** If a function's behavior is
  fixed by a modeled type's shape, it is re-deriving something the compiler already derives — model
  the missing fact and consume the derived operation (a catamorphism, a traversal, a structural fact).
  Watch the converse too: over-modeling (nominalizing a thing that should be a plain value).
- **A finished stage is one fold:** `stage(x) = fold_carrier(x, algebra(model))`. Non-fold residue
  is exactly one of two things — a named irreducible kernel (a fixpoint solver, char-class matching:
  *fold the traversal, name the kernel*) or un-migrated modeling (a decision the model hasn't
  absorbed yet, tracked debt). There is no third. The volume of non-fold residue *measures* how much
  decision-making still lives in code instead of the model. Migrate by deleting the arm in the same
  change that adds the row — grafting the fold beside the surviving arms is the anti-pattern; the file
  must shrink.
- **Model just-in-time; the mark is the authority.** A change that touches an unmodeled concept
  models it *in that change* — never a `P-*`/TODO placeholder. Debt is marked on the carrier, not in a
  parallel ledger doc; the mark is authoritative and debate belongs in PR review. A piece of
  structural content found living in a consumer belongs in the type (the lookup smell).
- **Dissolution has exactly three dispositions:** *dissolve-now* (a directive — do it, it jumps the
  queue), *terminal* (genuinely consumer-independent, done), or *gated* (bound to a specific named
  arrival). There is no fourth, and an unmarked scaffold defaults to the failure-to-dissolve it
  actually is.

---

## 6. Enforcement: lenses and opacity

- **Lenses are the invariant-enforcement primitive — not grep.** A lens walks the DAG and flags
  violations; it is a *pure reader that stores nothing* (no annotation tables alongside the
  substrate). A new analysis is a new lens over the same `Node` tree users write — zero substrate
  edits. Verification predicates are substrate *consumers*, reading the same declared facts every
  other consumer reads; a verifier with its own parallel copy of the facts will drift from them.
- **Correctness is one mechanism across all dimensions:** declare the lattice in `std/` → compute at
  binding sites → carry through the IR → enforce at consumption. Users declare their own dimensions
  the same way the compiler declares its own; the framework is uniform.
- **Guarantee tiers, and the trap in the middle.** Coverage runs from "unrepresentable by
  construction" to "fundamentally limited." The dangerous tier is the one where the *machinery exists
  but nothing gates on it* — it gives the illusion of coverage. Reflection riding host enumeration or
  a hand-fed node proves nothing; prove a read axis by execution with a no-host-enumeration control.
- **Opacity is single-authority's missing half.** Single authority means a fact has one home;
  *opacity* means a below-boundary change is unnameable to consumers — the *rename test*: if renaming
  an internal detail breaks a consumer, the boundary leaks. A type can have one authority and still
  leak its representation to dozens of callers (the v2 `SourceSpan` had one authority yet leaked
  offsets to ~27 callers and was "fixed" seven times). Make the carrier opaque; the only real proof is
  a metamorphic test that swaps the representation and expects no consumer to notice. Key "known
  primitive" decisions on structural identity (a declaration id via a typed edge), never on a name —
  a string-keyed primitive roster silently dissolves opacity.

---

## 7. The Rust seed (shrinks to zero)

The hand-written Rust is a bootstrap *seed*, live until self-hosting reaches its fixed point, and
monotonically shrinking toward zero. While it exists it follows the substrate's own discipline: data
plus free functions, pure by default, impurity confined to named edges. Carriers are structured
types, never primitives-with-sentinels. Handles are typed. Domain polymorphism is a match on
variants, not dynamic dispatch. `&mut` is "pure by borrow," not a license for spooky action. A
hand-written `.rs` *test* is a smell — an unexpressed language feature (see §8).

---

## 8. Verification discipline

- **Tests are structural data (`TestClaim` declarations in `.dag`), evaluated by substrate or
  generated runners.** A hand-authored `.rs` test marks a feature the language can't yet express; the
  release gate is zero Rust tests outside the pure-bootstrap residual. A white-box test that mirrors a
  declaration gets *deleted*, not migrated.
- **Tests are hermetic, behavior-driven, and unit-first.** Mock a minimal `Dag` and assert on
  behavior; do not compile a whole program end-to-end unless the pipeline genuinely *is* the unit
  under test. The directory carries the tier classification.

### The specification-without-execution trap (the most expensive lesson)
This codebase once grew a compiler-sized body of `.dag` that **typechecks, passes its grep-style
claims, and does not run** — `emit` could not produce `fn add` despite a full corpus of "passing"
claims. The discipline that keeps us out of it:

- **A consumer is anything that breaks when the behavior is wrong.** A typecheck and a `.contains()`
  grep are **not** consumers — they pass whether or not the code runs correctly. (This is **E-10**.)
- **"Done" means a real consumer running green *by execution*, plus a discriminating input that goes
  *red* when the behavior is wrong.** Merged ≠ proven. Landed ≠ proven. Compiles-clean ≠ works. A
  green that only ever exercised the first match arm proves nothing.
- **The acceleration warning, for the LLM agent specifically:** a capable model emits fluent,
  type-checking, grep-passing code at scale — exactly the artifact that *looks* finished without
  running. Treat your own fluent output as unverified until a real consumer runs it green. Volume is
  not progress.
- **Consumer-less code is archived, not audited.** Reasoning about or porting code nothing depends on
  *is* the trap. Name a real consumer before writing a model; if there is none, the work is
  experimental and lives outside the active tree until a consumer pulls it. Commit the *shape* on
  paper, build only the minimal slice that exercises the real **risk**.
- **The foundation is the most dangerous place to mark "done"** — base primitives are maximally
  isolated from consumers, so they can sit "done-by-typecheck" indefinitely and collapse the first
  time a consumer runs them. And **planning/tidying is the most seductive form of the vacuum**:
  reorganizing docs feels like progress and runs nothing. Do the cheap, sharp version pointed at the
  nearest running consumer.

---

## 9. Self-hosting (the end-state shape)

The compiler is a pure transform and an ordinary substrate fact — analyzable by its own lenses. Its
end state has four facets: it is **written in itself**; it **self-emits to a bit-identical fixed
point** (the `.dag` graph is the truth, Rust is one realization of it); its **tests are data**; and
it is **recursively flexible**. The hand-written Rust target is **zero**, reached monotonically. The
compiler's own ontology dissolves into `std/` — there is no dual representation at the
compiler/user boundary. (Whether v4 has *reached* the fixed point is a `ROADMAP.md` question; the
*contract* — bit-identical, over a declared artifact set, behind an operator-gated pin — lives here.)

The backend is a graph→graph coercion engine (§1). Its coercion kinds carry witnesses — widen,
proven-refine, validate, project, transform — and only widening and proven refinement are implicit;
there are no silent coercions, and rendering to target text is trivial and last.

---

## 10. Hard-won lessons (do not relearn the expensive way)

The traps below were each paid for once. They are instances of the principles above; they are
collected here because pattern-matching them is cheaper than re-deriving them.

- **Spec-without-execution / E-10** (§8) — the headline. Everything else is downstream.
- **Hollow alias** — a bare alias passes every shape-checker because minimality is not grounding.
- **Parameterized-family blindness** — a 15-variant mechanically-identical enum passed all four
  shape patterns *and* two reviews. Shape-checking cannot see enumerated copies of one set; test
  *cost-of-change*, not shape.
- **Construct-discard-reconstruct** — the cardinal compiler anti-pattern *and* the real perf cliff
  (re-derivation, not clones; one 6-line fix was 68×). Every "thread X through" must include "delete
  the downstream reconstruction of X." If you can grep-count it, it's probably not the bottleneck.
- **State-space conflation** — an `Option`/`None` standing for more than two meanings is the
  most-repeated defect; split into named variants so illegal states are unrepresentable. Fix it as a
  class, not per field. (Keep `Absence`, `Nullability`, `Unknownness`, `Defaultability` distinct.)
- **Effect taxonomy is the bug** — never reintroduce a separate effect-annotation layer; effects
  derive from signature shape (a returned-modified resource *is* a write).
- **Cache purity** — a cache that flips a verdict on entry order *exposed* a latent impurity, it
  didn't cause it. Key only on declared-input content; byte-identical cached-vs-cold is the standing
  purity oracle.
- **Coercion is proven by a normalized round-trip, not a golden string** — generalizing emit while
  ingest stays deferred proves only a code generator; ingest must not grow its own coercion arms.
- **Reflection evidence is not structural proof** — prove the read axis by execution, with a control
  that removes the host enumeration.
- **Parallel-representation debt** — an honestly-marked scaffold that duplicates a fact with a
  canonical home is still a single-authority violation; "delete the copy and consume the source," not
  "collapse the variants."
- **Internal review finds missing tests; external review finds missing checks** — you need both.

---

*Where we are, the active wave, and how to contribute: `ROADMAP.md`. Agent/harness instructions:
`CLAUDE.md`.*
