# v4 Compiler Migration — problems, modeling gaps, and the target shape

**Status:** working document (2026-06-03). This is a *migration plan and checklist*,
not a fact ledger. The authoritative per-site record is the inline `🟡 dissolve-on-arrival`
marks in the `.dag` files and the rules in `INVARIANTS.md` / `MODELING.md` / `THESIS.md`.
Where this doc and a mark disagree, the mark wins. Do not duplicate mark contents here —
link the file/function and let the mark carry the detail (operator standing principle,
2026-05-19).

---

## Governing invariant: no code without a consumer (operator, 2026-06-04)

**A consumer is anything that breaks when the behavior is wrong.** Typecheck and
`.contains()` greps are *fake* consumers — they pass whether or not the code runs correctly.
Growing a compiler-sized **specification that doesn't execute** is the direct result of
gating on fake consumers.

Rule: **before writing a model/function, name a real consumer; if there is none, it is
experimental and is archived (not kept) until a consumer exists.** Consumer strength, weakest
to strongest: (1) *you reading the executed output* — the floor, but it forces execution;
(2) *an executed assertion* (`run(emit(add)) == expected`) — falsifiable; (3) *other code
that depends on the behavior and breaks if it's wrong* — correctness + regression. Treat (1)
as a consumer to *upgrade*, not a resting state.

**Map vs territory** (resolves the top-down tension): top-down structure-first design is fine
*as a map* — its consumer is you reading it to plan, so it belongs in **docs** (or
explicitly-experimental, archived `.dag`). Code/models-as-artifacts are **territory** and need
a real consumer. The failure was writing the map in executable `.dag` and treating it as
territory. Keep designing structure-first **in the map**; promote a piece to **territory**
only when a consumer pulls it.

**Archive-by-default, not hard-delete.** Reasoning about / porting unconsumed code is the
trap (high effort, and it may be wrong since it never ran). Archiving requires *no* reasoning:
move it out of the active set (git-recoverable), restore when a consumer appears.

---

## Part 0 — Ground truth: what actually runs today

Established empirically (v2 stage0 interpreter `gunbc run --claim-run`, 2026-06-03):

- **The compiler that runs is v2/v3 Rust.** The `src/v4/compiler/*.dag` stage *logic* is
  parsed + typechecked (via the v3 Rust frontend) and asserted by v3 text-grep tests, but
  is **not executed** as a compiler. The thing that *runs* a v4 TestClaim is the v2 Rust
  tree-walking interpreter (`src/v2/stage0/src/v2_interpreter.rs`), not the v4 stages.
- **Does v4 run end-to-end? No.** Two independent gaps:
  - **Ingest is staged/unimplemented.** `compiler/00_compile.dag` has
    `compile_ingest_staging` (tokenize→parse→normalize→resolve→validate_then_compile) but
    its header says *"legacy Source staging until ingest lands"*; claims feed **fixture**
    inferred/emitted trees, not real source ingestion.
  - **Emit is intractable, not just untested.** Running the MVP `fn add(x,y)` emit anchor
    (`anchor_mvp1_emit_accepts` → `emit = serialize ∘ translate`) on the lazy `--claim-run`
    path **did not complete in 600s** (step-zero, 2026-06-04) — and it did *not* `Reject`
    either (the fuel guard fail-closes fast if a measure simply runs out). So it is not
    benign-slow and not a clean unsound-measure reject; it is **catastrophic blowup** —
    exponential walk (branching recursion with no memoization re-walking shared nodes) or an
    astronomically-large computed measure. Release won't save a 600s+ trivial case; it is
    algorithmic. The expected output (`rust_mvp1_source_text = "fn add(x: i32, y: i32) -> i32
    { x + y }"`) exists and the claim is wired, but "v4 produces Rust" is asserted only by v3
    `.contains(...)`, **never demonstrated by execution — and cannot currently be.**
- **Does v4 produce any Rust? No — the serializer cannot emit `fn add` in 10 minutes.** The
  Rust strings in the corpus are hand-authored expected fixtures.

**Implication:** the 4,316-line `06_translate` is not just unexecuted, it is **broken for its
simplest input** — so it must be *rebuilt along an executing spine*, not ported. The first
real milestone is emit's first real consumer (an executed `run(emit(add)) == expected`
witness); getting it green is a genuine rework (diagnose measure-vs-walk-vs-memoization), not
a small helper.

---

## Part 1 — What "good" means (the rules these files answer to)

| rule | source | one-line |
|---|---|---|
| **Don't hand-roll a derived operation** | MODELING M1 / Practice 10 | if behavior is determined by a modeled type's shape, model the missing fact — don't write the operation |
| **Every emitter special case = ungrounded concept upstream** | THESIS (epistemic stacking) | per-variant code is a smell pointing at a missing substrate fact |
| **Structural decompression** | THESIS / INVARIANTS | coproduct tags dissolve into coordinate facts; closed system guarantees termination |
| **Bounded forward execution (P4)** | INVARIANTS Decidability | recursion must carry an explicit finite bound; lowers to `Loop`/bounded fold |
| **Derived homomorphism** | THESIS | write N target-models; compiler *derives* the N×M translations — no per-language adapters |
| **One result pattern (M6)** | MODELING | generic `Outcome` carrier, not bespoke result records |

Key consequence used throughout this doc: there are **two kinds of bulk** in the stage
files, and they are NOT the same problem:

- **(A) hand-rolled dispatch** — per-variant `match` arms re-deriving behavior from a
  coproduct's shape. *Rule violation* (M1/Practice 10). Dissolvable now; the `🟡` marks name targets.
- **(B) fuel triads** — `*_bounded(remaining: Int)` + `*_go` + measure wrappers. *Not a
  violation* — they satisfy P4's explicit-bound requirement. Verbose because the
  structural-termination *checker* (`std/termination.dag` design: "checker not discoverer")
  **is not landed**. Dissolving them is gated on landing that checker.

---

## Part 2 — The axes (where each is modeled, status, gap, target)

### 2.1 Homomorphism / coercion
- **Where:** `std/find_witness.dag` (`find_witness` — coercion as witness search);
  `std/constraints.dag` (`solve_constraints → find_witness`, T-9); coercion sites in
  `compiler/06_translate.dag` (`coercion_candidates_from_target_model`, `coerce_grounded_node`).
- **Design:** translation = homomorphism *derived* by comparing groundings (THESIS derived
  homomorphism). The cited architecture doc `docs/design-v4-compiler-homomorphism.md` is
  **referenced by marks but absent from the repo** (removed in a public-flip; cite the
  concept, not the path).
- **Status:** identity-MVP wiring; `constraints.dag` candidate/predicate bodies are `🟡`
  scaffolds until "T-9 ground + T-25-tail" land.
- **Gap → target:** coercion should be a single witness-search over declared inhabitance,
  consumed by translate; today translate carries coercion logic inline. Target = translate
  calls `find_witness`; no bespoke coercion arms.

### 2.2 Projection / pure functions
- **Where:** `std/projection.dag`; `project_type_expression_*` (many) in `06_translate`;
  `03_resolve`, `01_tokenize`, `02_parse`.
- **Status:** projection is the bulk of `06_translate` (type-expr → target shape), heavy with
  fuel triads (B) and per-variant arms (A).
- **Gap → target:** projection over the `Node`/type-connective coproduct should be one
  derived morphism + one structural-fold combinator, not ~150 functions. Purity already
  holds (`.dag` is pure by construction; CODING "data + free functions").

### 2.3 Omni-emission / ingestion (bidirectionality)
- **Where:** emit — `05_emit.dag` (`emit = serialize_target ∘ translate`), `06_translate`
  (`target_serialize_source_from_model` = grammar-inverse lex walk). ingest — `00_compile`
  (`compile_ingest_staging`), `std/module_batch.dag` (T-35 compile-time ingest carrier).
- **Pivot:** `std/grammar.dag` — *"canonical grammar model shared by compiler stages and
  target-language specs."* Parse (source→Node) and serialize (Node→source) are inverses over
  **one** grammar. That shared grammar IS the bidirectionality mechanism.
- **Status:** emit side modeled but **does not execute** (Part 0); ingest side **staged, not
  landed** ("legacy Source staging until ingest lands"); "emit→ingest fidelity is W1b/T-36".
- **Gap → target:** one grammar drives both directions; `parse` and `serialize` derived as
  inverses. Bidirectional round-trip (`source → Node → source` identity) is the acceptance
  test. Neither direction runs green today.

### 2.4 Lenses / testgen
- **Where:** `lens/*.dag` (22 files: affected_set, application, complexity, cost, coverage,
  edit_locus, effect, fact_density, idempotency, ownership, parallelism,
  structural_resolution, synthesis, subsumption, unused_parameters, …);
  `lens/registry.dag` (registry binding); `lens/testgen.dag` (test generation).
- **Status:** lenses are pure functions over `InferredTree` (`import v4.compiler.infer`) —
  this is the thesis "correctness dimensions" surface, and it's the **most reusable, most
  validated** part (many lens claims run green). 21/22 lens files reachable.
- **Gap → target:** lenses are in good shape relative to the stages; they are a **KEEP**
  anchor. testgen couples to facet-3 ("tests are data"); its claims are part of the spec.

### 2.5 Language concepts vs compiler (separation)
- **Where:** language facts in `extdeps/languages/*.dag` (18 specs: rust, python, go,
  typescript, kotlin, swift, java, cpp, wasm, dag, lean, llvm_ir, …) per M3 (extdeps model
  specs). Compiler stages in `compiler/*.dag`.
- **Status:** the separation largely **holds** — prior analysis found ~0 per-language
  branches in the stages; stages read the language spec (`TargetModel`) as data. Shared
  interface = `std/grammar.dag`, `std/node.dag`, `std/target_model.dag`.
- **Gap → target:** preserve this. The derived homomorphism *depends* on it (N models, not
  N×M code). Any per-language `match` arm that creeps into a stage is a regression.

### 2.6 Termination (the fuel triads) — the big lever
- **Where:** `std/termination.dag` (`DescentEvidence = Strict | NonIncreasing |
  DescentUnknown`, inhabits `BoundedLattice`); consumed implicitly by every `*_bounded`/`*_go`
  triad in `06_translate` (20 `_bounded` + 10 `_go`) and `04_infer` (10 `_bounded`).
- **Status:** the analyzer "derives cost from bounded structure rather than validating
  proofs"; the **checker** that would let bounds be *derived* from descent evidence is
  *"design, not landed behavior."* So bounds are hand-threaded today (correct per P4).
- **Gap → target:** land the structural-termination checker (generalize DB-9 mutual-recursion
  dissolution). Then the explicit `remaining: Int` measures dissolve into derived descent
  proofs, collapsing the dominant bulk of `06_translate`. **Substrate-first; escalate /
  model-before-implement — not a worker-improvises change.**

---

## Part 3 — Per-compiler-file map

`lines` from current tree. "A" = hand-rolled dispatch (dissolvable now). "B" = fuel triads
(gated on 2.6). Type-surface columns mark what claims/lenses import and must be preserved.

| file | lines | role | main problems | must-preserve surface |
|---|---|---|---|---|
| `06_translate.dag` | 4316 | translate + projection + serialize + coercion + use-site ownership | **A + B (dominant)**; coercion inline (2.1); projection sprawl (2.2) | `TargetNodeTree`, `translate`, `target_serialize_source_from_model` |
| `05_eval.dag` | 1988 | interpretation algebra + **claim-runner surface** | A + B; mixes runner *types* with interpretation *logic* | `run_test_claim`, `TestClaimRun`, `test_claim_input` (imported by 49 claims) |
| `02_parse.dag` | 893 | grammar validation | explicit `🟡` predicate-dissolution marks (A) | `ParseTree` |
| `04_infer.dag` | 625 | inference | 10 `_bounded` (B) | `InferredTree`, `InferredFacts` (imported by 43 claims + 8 lenses) |
| `03_resolve` / `03_name_resolve` / `03_normalize` | 590/575/246 | K-1 resolve, cross-file admission, sugar dissolution | A | `ResolvedTree`, `NormalizedTree` |
| `00_compile.dag` | 403 | compile entry + **ingest staging (unlanded)** | ingest not landed (2.3) | `compile`, `CompileOutput`, `CompileMode` |
| `01_tokenize.dag` | 526 | tokenize | B | tokens |
| `emit_host.dag` | 383 | host-process emit bridge | — | (2 claims) |
| `source_authority.dag` | 409 | source authority contract | — | (1 claim) |
| `05_emit.dag` | 42 | `emit = serialize ∘ translate` | thin; fine | `emit` |
| `07_target_carriers.dag` | 18 | `TargetModel` / `TargetSource` carriers | thin; fine | `TargetModel`, `TargetSource` |
| `self_host.dag` | 150 | self-host ratchet scaffold | scaffold | (2 claims) |

**`workflow/ci.dag` (6044 lines)** — *not* a compiler stage. Facet-4 CI workflow model,
imported by 6 workflow claims. Bloated; separate reset/trim track gated on those claims.

**Reset shape (all Tier-2 files):** port/logic split — keep the must-preserve type surface so
the 253-claim corpus + 8 lenses keep compiling, replace the logic behind it. Naive deletion
breaks ~150 claims + 8 lenses at compile time.

---

## Part 4 — Test claims: what works, and why they're valuable

- **253 claim files**, **124 with directly-runnable zero-arg `-> Bool` witnesses**, 37
  `run_test_claim`-data based, 218 declare a `TestClaim`.
- **What runs green:** self-contained categories (e.g. `lens_coverage` 5/5) run in ms via the
  pre-existing `gunbc run --claim-run`. Categories whose closures pull in the heavy stages
  stall on the v2 frontend O(n²) wall (not a v4 logic failure). (A batch `claim-corpus` runner
  was used as the Part-0 *diagnostic instrument*, then reverted per E-10 — it had no standing
  consumer and couldn't run emit anyway; it lives in git history, restorable when a real
  corpus-gate consumer exists.)
- **What does NOT validate by execution:** the `mvp1_*_translate` / emit claims — they are
  `CompilesClaim`/data-based (no Bool witness) and the emit path doesn't complete when run
  (Part 0). `algebra_laws` witnesses return `false` (investigate: expected-falsification vs
  real regression).
- **Why valuable:** the claims are the **executable spec** for each stage — they define the
  behavior a reset/rebuilt stage must satisfy. The 124 Bool-witness files are the **gate**.
  Curate, don't delete: a claim that only text-greps or never runs is a spec gap to upgrade
  to an executable witness, not dead weight.

---

## Part 5 — Execution-first plan (consumer-driven, archive-by-default)

Supersedes the prior "port the logic, don't `rm`" framing: that protected fake-consumed code,
and step-zero shows the protected code (`06_translate` emit) is *broken*, not merely
unexecuted. The gate is now **"does this run green when executed,"** not typecheck+grep.

- [x] **S0. Diagnose emit's blowup — DONE (2026-06-04).** Root cause: **`fold_node`
      (`std/node.dag:94`) is a non-memoized catamorphism** — it recurses into every edge
      target with no visited-set/dedup. The `Node` graph is a **shared DAG** (the i32
      inhabitant the `fn add` signature references 3× is itself a deep shared sub-DAG), so the
      fold walks **paths, not nodes** → exponential. Empirically: counting *one tiny
      function's* nodes hangs >75s while unrelated witnesses run in 60ms. Not emit-specific,
      not the target-model size, not cyclic (the value is finite + acyclic — so this is a
      **performance pathology, not a P4/boundedness violation**). It is the **walk primitive
      itself**, which Appendix B mislabeled "DONE" (the no-consumer disease at the foundation).
- [ ] **S1. First real consumer: an executed emit witness.** A `run(emit(add)) == "fn add…"`
      Bool witness, run via the **pre-existing** `run --claim-run` (no batch runner needed).
      This is the milestone the whole project has been missing — the crossing from
      *specification* to *running system*.
- [ ] **S2. Make `fold_node` sharing-aware (memoize by content-hash), then make S1 green.**
      The fix: dedup the fold so each distinct node folds once (hash-consing; `fold_node_content_hash`
      is half the machinery) → O(#nodes), and a fail-closed cycle guard. **Expect two rounds**
      (per the consumer invariant): the memo fix makes emit *terminate*; the now-running witness
      will then likely expose emit-*logic* bugs hidden behind the hang — that is the consumer
      doing its job, not a regression. Re-derive emit along a spine that runs; do **not** port
      the 4,316 lines. *Open design fork (decide before implementing): interpreter-level memo
      (v2 executor) vs a `.dag` memoized-fold combinator vs hash-consed `Node` construction.*
- [ ] **S3. Archive everything the executing spine doesn't touch.** Re-root reachability on
      **executing** consumers (green-running claims + executing v3 tests), *not* all claims
      (which over-count via type-imports). Everything outside that set is archived
      (git-recoverable), not audited or ported. Validates the "massive unused code" intuition.
- [ ] **S4. Grow the spine one consumer at a time.** Each new piece of structure is pulled in
      by the next real consumer and validated green. Top-down vision stays the map (this doc);
      code crystallizes along the always-executing spine.

**Substrate work (M-items) is downstream of S1–S2 and only justified once a consumer needs it:**
coercion→`find_witness`, the coproduct-morphism / `fold_node` dissolutions (Appendices A/B),
and the structural-termination checker (the big lever for the fuel triads). These are *map*
until an executing consumer pulls them into *territory*. Spike the termination checker early
only as a feasibility probe (it's the existential bet), not as elaboration.

**Scope note (don't re-make the over-claim one level up):** a memoized/visit-once `fold_node`
gives a structural termination bound **for the node-fold family** (most of translate / serialize
/ projection) — so it subsumes much of M6's scope as a side effect. It does **not** cover
recursion that isn't a structural node-fold (inference fixpoints, worklist passes in
`04_infer`). Memoization **shrinks** M6; it does not retire it.

**Non-goals / guardrails:** don't add per-language branches to stages (breaks the derived
homomorphism); don't elaborate the substrate (Appendices A/B) before emit runs green —
un-executed elegance is unfalsifiable; don't cement Rust to satisfy a census ratchet; archive
unconsumed code, don't reason about it.

---

## Appendix A — What "better-modeled" looks like (worked examples)

**The architecture we're aiming at:** every concept is defined *individually, once*, and the
**compiler generates the glue between concepts** — instead of a human hand-writing the glue
per concept-pair or per concept-variant. That is the derived homomorphism (THESIS) and
epistemic stacking applied to the compiler itself.

**The measuring stick** (CLAUDE.md "Cost of Change"): *when the language grows by one type,
one variant, one transport, how many files need editing? The answer should be 1.* Every
example below is graded by that number. "Hand-rolled glue" makes it N; "derived glue" makes
it 1.

### Example 1 — per-variant dispatch: hand-rolled glue → derived coproduct morphism

**Before** (`compiler/02_parse.dag:302`, real):

```dag
fn expr_is_nullable_given_set(expr: GrammarExpr, nullable_set: Set<Symbol>) -> Bool {
  match expr {
    Terminal { token_class: _ }        => false
    Nonterminal { production: name }   => nullable_set.member(name)
    Sequence { left, right }           => is_nullable(left) && is_nullable(right)
    Choice   { left, right }           => is_nullable(left) || is_nullable(right)
    Optional { element: _ }            => true
    Repeat   { element: _ }            => true
  }
}
```

**The problem.** This is one of **4 functions** (`expr_is_nullable_given_set`,
`expr_direct_left_corner_list`, `expr_has_undefined_nonterminal`, the parse walk) that each
`match` the **same 6 `GrammarExpr` variants** — **24 arms total**. Add a 7th variant and you
edit all four; nothing structurally forces them to stay in sync; the fact "an `Optional` is
nullable" is re-stated implicitly in every function. **Cost-of-change = N (one edit per
query function).** This is exactly what M1/Practice 10 forbids — the behavior is determined
by the variant's shape, so it's re-deriving something the model should carry. The file's own
`🟡` mark says so: *"substrate-derived nullable query morphism over GrammarExpr coproduct,
per-variant fold eliminating hand-written match arms."*

**After — Level 1 (writable today): one catamorphism + per-query algebras.**
The four functions all share the *same recursion over `GrammarExpr`* and differ only in the
per-variant combinator. So write the walk **once**, parameterized by an algebra; each query is
then just data (no recursion, no `match`):

```dag
// The structural recursion over the coproduct, written ONCE. Every query reuses it.
type GrammarExprAlgebra<R> {
  on_terminal:    R                 // Terminal contributes this
  on_nonterminal: (Symbol) -> R     // Nonterminal contributes from its name
  combine_seq:    (R, R) -> R       // Sequence folds its two children
  combine_choice: (R, R) -> R       // Choice folds its two children
  on_optional:    (R) -> R          // Optional, from the element's result
  on_repeat:      (R) -> R          // Repeat,   from the element's result
}

fn fold_grammar_expr<R>(expr: GrammarExpr, alg: GrammarExprAlgebra<R>) -> R {
  match expr {
    Terminal    { token_class: _ }   => alg.on_terminal
    Nonterminal { production: name } => alg.on_nonterminal(name)
    Sequence    { left, right }      => alg.combine_seq(fold_grammar_expr(left, alg),
                                                        fold_grammar_expr(right, alg))
    Choice      { left, right }      => alg.combine_choice(fold_grammar_expr(left, alg),
                                                           fold_grammar_expr(right, alg))
    Optional    { element }          => alg.on_optional(fold_grammar_expr(element, alg))
    Repeat      { element }          => alg.on_repeat(fold_grammar_expr(element, alg))
  }
}

// "Is this expr nullable?" is now an ALGEBRA — its per-variant facts, nothing else.
fn nullable_algebra(nullable_set: Set<Symbol>) -> GrammarExprAlgebra<Bool> {
  GrammarExprAlgebra {
    on_terminal:    false,                              // a token never matches empty
    on_nonterminal: fn(name) { nullable_set.member(name) },
    combine_seq:    fn(l, r) { l && r },                // both halves nullable
    combine_choice: fn(l, r) { l || r },                // either side nullable
    on_optional:    fn(_) { true },                     // X? matches empty
    on_repeat:      fn(_) { true },                     // X* matches empty
  }
}

fn expr_is_nullable_given_set(expr: GrammarExpr, nullable_set: Set<Symbol>) -> Bool {
  fold_grammar_expr(expr: expr, alg: nullable_algebra(nullable_set))
}
```

`expr_direct_left_corner_list` and `expr_has_undefined_nonterminal` become their own
algebras over the *same* `fold_grammar_expr`. The 24 hand-written arms collapse to **6** (in
one fold) plus a few lines of data per query. Adding a 7th `GrammarExpr` variant = **one** arm
in `fold_grammar_expr` + one field per algebra, and the compiler's exhaustiveness check forces
every algebra to account for it (no silent desync). Recursion and termination live in one place.

**After — Level 2 (the end-state: the compiler generates the walk).**
Level 1 still hand-writes the 6-arm walk. The thesis target removes even that: `GrammarExpr`
*is* a `Node` (a `Disj` of variants, each a `Conj` of edges), and the substrate already has a
generic `fold_node` over *any* `Node` (see `std/algebra.dag` + the existing `fold_node` spine).
A query supplies only its per-connective contribution; the walk is the generic fold:

```dag
fn expr_is_nullable(expr: Node, nullable_set: Set<Symbol>) -> Bool {
  fold_node(node: expr, interpret: nullable_contribution(nullable_set))
}
```

Now a new `GrammarExpr` variant needs **zero** edits to any walk — `fold_node` already folds a
new `Disj` arm structurally; you only declare the variant's contribution fact (and if you
don't, it fails closed via `DescentUnknown`). **That** is "concepts defined individually, the
compiler generates the glue": the variant + its one fact is the concept; the nullable /
left-corner / undefined-ref / parse-walk queries are all generated over it. Cost-of-change = 1.

**What Level 2 actually requires** (grounded in current code; this is a *named, bound* path,
not speculation):

1. **A generic `Node` catamorphism — DONE.** `fold_node<R>(n, algebra: NodeFold<R>)` already
   exists (`std/node.dag:94`), plus `fold_node_content_hash`. The walk primitive is in place.
2. **`GrammarExpr` projected onto canonical `Node` edges — task T-7.1.** Today `GrammarExpr`
   is a bespoke `type` (`grammar.dag:139`) with a hand-written recursive walker. The tracked
   dissolution mark (`grammar.dag:1048`) says the first PR projecting it to canonical `Node`
   edges **deletes the recursive coproduct walker** and routes everything through that
   projection. Until this lands, `fold_node` can't see `GrammarExpr` as a foldable `Node`.
3. **Substrate-derived coproduct morphism — the shared capability the `🟡` marks name.** The
   fold must read a variant's discriminant + payload *structurally* and dispatch by a declared
   fact, not a `match` arm (`algebra.dag:638/647` carry the interim discriminant-projection
   scaffolds). This is the same "substrate-derived morphism over the coproduct" every
   per-variant `🟡` mark in the compiler is gated on.
4. **Per-variant contribution as a declared algebra.** Each variant's combine op (`⊥/⊤/∧/∨`)
   declared as a homomorphism into a `Monoid`/`BooleanAlgebra` (`std/algebra.dag`); the query
   picks the algebra, the fold applies it. (Nullable = BooleanAlgebra; left-corner = a
   Set-union monoid; etc.)
5. **Derived termination — §2.6 / checklist M6.** A *generic* fold over an arbitrary `Node`
   needs its bound derived from the data's finiteness (the `std/termination.dag`
   "checker not discoverer" design), or it falls back to a hand-threaded fuel param — which
   defeats the point. So Level 2 for the *whole* compiler is gated on the termination checker.
6. **Fail-closed default.** A variant with no declared contribution must error
   (`DescentUnknown`), never silently default (M5/P3).

Net: **(1) is done; (2) is one bound task (T-7.1); (3)–(4) are the coproduct-morphism +
algebra-inhabitance substrate work; (5) is the termination lever.** Level 1 is the safe
intermediate you can land per-query *today* without any of (2)–(5); Level 2 is what those
substrate capabilities buy you.

**Why it's better.** Adding a `GrammarExpr` variant = declare its facts once; *all* queries
update for free. **Cost-of-change = 1**, desync becomes impossible, and "what nullability
means for this variant" lives in one place next to the variant. The *concept* (the variant +
its facts) is defined individually; the *glue* (the queries) is generated.

### Example 2 — coercion: per-pair adapters → `find_witness` derivation

**Before** (the smell the thesis names). Mapping a `.dag` type to a target-language primitive
by writing the mapping for each (source, target) pair: `i32` for Rust here, `int` for Python
there, a lookup table keyed by name. With N source types and M targets that trends toward
**N×M hand-written entries**, and every new target re-touches the mapping.

**After** (`std/find_witness.dag`, the "unified candidate-enumeration-and-check primitive").
Define each type's grounding/inhabitance **once** — N source models + M target models = **N+M**
— and let the compiler derive the coercion by `find_witness`: enumerate candidate target
primitives, check declared inhabitance, return the **unique** witness, **fail-closed** on
none (`find_witness_no_candidate_diagnostic`) or ambiguous
(`find_witness_ambiguous_candidate_diagnostic`).

**Why it's better.** **N+M models, not N×M adapters.** Adding a target language is *one spec
file in `extdeps/languages/`* with **zero edits** to any coercion/translate code — the literal
statement of "concepts defined individually, compiler generates the glue." It also fails
closed: an ungroundable type is a located diagnostic, never a silent wrong mapping (M5, P3).
(Status today: `find_witness` exists; translate still carries some coercion inline — that's
migration item **M5**.)

### Example 3 — termination: hand-threaded fuel → derived descent proof

**Before** (`compiler/06_translate.dag`, the dominant bulk — real shape):

```dag
fn token_item_to_source(item: Node, target: TargetModel) -> Outcome<TargetSource> {
  bind_outcome(o: token_item_serialize_measure(item, target),     // compute the bound
               f: fn(measure) { token_item_to_source_bounded(measure, item, target) })
}
fn token_item_to_source_bounded(remaining: Int, item: Node, target: TargetModel) -> ... {
  // body threads `remaining - 1` into every recursive call
}
```

**The problem.** *Every* recursive function is this triad: a `_bounded` worker carrying an
explicit `remaining: Int`, plus a wrapper that computes the measure. The termination bound is
**re-stated by hand in each function**. (Note: this *satisfies* P4's explicit-bound rule — it
is not a violation — it's just verbose because the bound is hand-threaded rather than derived.)

**After** (the target, gated on **M6**). The recursion is structurally over a *finite* `Node`
tree, so the bound is a **property of the data**. Declare/derive the `DescentEvidence`
(`std/termination.dag`) once and let the analyzer **check** termination from structure (its
stated "checker not discoverer" design). The `remaining`/measure plumbing disappears; the
function is just the structural recursion.

**Why it's better.** The bound is derived once from the data's finiteness instead of restated
per function — adding a recursive helper costs "write the recursion," not "write the recursion
+ a measure + a bounded variant." This is the biggest single bulk reduction in `06_translate`,
which is why it's the high-lever, substrate-first item.

### The through-line

All three are the **same move**: replace glue a human writes *between* concepts with glue the
compiler *derives* from concepts declared individually.

| | concept defined once | glue today | glue target | cost-of-change |
|---|---|---|---|---|
| dispatch | `GrammarExpr` variant + its query facts | 24 hand match-arms | derived coproduct fold | N → **1** |
| coercion | type grounding/inhabitance | per-pair mapping | `find_witness` search | N×M → **N+M** |
| termination | finite `Node` tree | per-fn fuel triad | derived descent proof | N triads → **1 checker** |

Sustainability *is* this number going to 1. A stage file is "good" when adding a language
variant, a target, or a recursive helper touches one declared fact — and the compiler
regenerates everything downstream.

---

## Appendix B — Level 2 file-edit map (the substrate primitives, in v4)

**All Level-2 work is in `src/v4/`.** v2 is only the *executor* — the v2 Rust interpreter runs
the v4 `.dag` so fold-based queries can be validated by running their claims green. v2 is not
edited (it shrinks toward zero per the thesis). Level-2 items 2–6 *are* substrate rework
(model-before-implement); the hard core is #3 and #5.

| # | capability | file(s) | state today → edit | difficulty |
|---|---|---|---|---|
| 1 | generic `fold_node` | `std/node.dag:21,94` | **NOT done (S0 correction)** — it compiles, but is a *non-memoized* catamorphism that path-explodes on shared-DAG `Node`s (can't fold one `fn add` in 600s). Must be made sharing-aware (memoize by content-hash) **first** — this is the real #1 fix, and the precondition for everything below. The "DONE" label was the no-consumer disease at the foundation. | **the #1 fix** |
| 2 | `GrammarExpr` → canonical `Node` (T-7.1) | `std/grammar.dag` (~:1049) | `grammar_expr_to_node` is a hand walker; the `🟡` mark says the projecting PR deletes it and routes through one `Node` projection so `fold_node` folds `GrammarExpr` directly | bounded |
| 3 | substrate-derived coproduct morphism | `std/algebra.dag` (+`std/node.dag`) | variant-discriminant/payload projection is interim scaffold (`algebra.dag:638/647`); make it real so a fold dispatches by declared fact, not `match` | **hard (C1)** |
| 4 | per-variant contribution as declared algebra | `std/algebra.dag` + `compiler/02_parse.dag` | `Monoid`/`BooleanAlgebra` carriers exist; declare each variant's op as data, rewrite the 4 queries as `(algebra + data)` over `fold_node`; builds on #3 | medium |
| 5 | termination **checker** | `std/cardinality.dag` (model) + `compiler/04_infer.dag` (enforcer) | model + proof witnesses exist (`structural_node_size_termination_proof`, `termination_proof_witness_for_node`, `cardinality_descent_not_proven_diagnostic`); the analyzer that **checks** them and gates recursion is "design, not landed" — without it a generic fold still needs hand fuel | **hard (C1), big lever** |
| 6 | fail-closed default | the fold primitive + queries | variant with no declared contribution → diagnostic (pattern: `cardinality_descent_not_proven_diagnostic`); rides on #3/#4 | small |

**Dependency order:** 1 (done) → 2 (first concrete domino) → 3 → 4 → 6; 5 is independent and
heavy, and is what makes a *generic* fold viable without per-function fuel.

**Lower-risk path to *reach* Level 2** without front-loading #3/#5: land #2 (T-7.1), rewrite the
`nullable` query as `fold_node`-over-`grammar_expr_to_node`, and **validate it by running the
parse/grammar claims green** through the interpreter — proving the fold pattern works *executed*
on real substrate before committing to the coproduct-morphism (#3) and termination-checker (#5)
designs.

**Guardrail:** #3 and #5 are C1-class substrate-design changes on load-bearing files
(`std/node`, `std/algebra`, `std/cardinality`); per the intro docs these carry a higher bar and
want explicit sign-off before being touched.
