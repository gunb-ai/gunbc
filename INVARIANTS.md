> Part of: [THESIS.md](THESIS.md) — these invariants are the structural rules that enforce causal consistency. The thesis says "every causal link is validated"; this document says how.

# Compiler and Runtime Invariants

This is the reviewer-facing invariant index. Five principles anchor everything else.

## The five principles

Every rule in this repo descends from one of five first principles. Growing sub-rules at the top level (C-1..C-10, E-5..E-9, L-7/L-8, DB-N) was how receipts got captured over time, but the underlying commitments compress to:

1. **Modeling Faithfulness** — every construct grounds in a declared source; ungrounded = invalid.
2. **Boundary Discipline** — boundaries carry enough declared information for mechanical consumers; every fact lives in exactly one authoritative place.
3. **Fail-Closed** — every path either succeeds fully or fails with a typed diagnostic; no fabricated plausible output.
4. **Decidability** — the accepted language stays within a bounded substrate whose correctness is structurally provable; lowering is the receipt.
5. **Progress Is Dissolution** — a change counts as progress only if it reduces ad-hoc state; scaffolds need dissolution paths.

Two load-bearing headings from the previous organization map directly into these principles: **Verifiability** rolls into Decidability (verification is a structural consequence of a closed system), and **Sustainability** rolls into Progress Is Dissolution (sustainability is the long-run framing of cost-of-change). Their sub-rules distribute into the principles where their motivating teeth live — detailed in the appendix.

Each principle below carries: the rule, why it stands alone, problem/solution shapes for pattern-matching, a historical dissolution receipt, and a cross-reference to the related rule IDs that elaborate it. Each numbered rule has a stable `#id` anchor for in-file citation (e.g., `INVARIANTS.md#c-8`, `INVARIANTS.md#e-9`); the appendix below indexes every ID to its home principle.

---

## Read this first — the specification-without-execution trap (especially LLM agents)

This codebase once grew a compiler-sized body of `.dag` that **typechecks, passes its
grep-style claims, and does not run** — `emit` could not produce `fn add` (it hung >600s)
despite a full corpus of "passing" claims. The lesson is cheap to state and expensive to
relearn; the rest of this document is the structural rules that keep you out of it, but this
is the orientation:

- **A consumer is anything that breaks when the behavior is wrong.** Typecheck and
  `.contains()` greps are **not** consumers — they pass whether or not the code runs
  correctly. (Formalized as the consumer invariant **[E-10](#e-10)**.)
- **"Done" means a consumer running green *by execution* — never typecheck-plus-grep.** If
  nothing has *run* the behavior, it is unverified no matter how finished it looks.
- **The acceleration warning (this is for you, LLM agent):** a capable model emits fluent,
  type-checking, grep-passing code at scale — *exactly the artifact that looks finished
  without running*. Speed buys **specification, not execution**. Treat your own fluent output
  as unverified until a real consumer runs it green. Volume is not progress; a running
  consumer is. *(Receipt for this very warning: an early draft of these notes asserted a
  `fold_node` "non-memoized fold" root-cause for the emit hang and nearly shipped it as a
  numbered invariant — deeper probing on clean `main` refuted it (the cost was a v2-interpreter
  runtime cost, upstream of any fold). An **invariant** was about to be written on an
  unverified diagnosis. That is the disease, in its purest form, and it is why the bar is "by
  execution.")*
- **The foundation is the most dangerous place to mark "done."** Consumers live at the top of
  the stack (emit, the user); base primitives are maximally isolated from them, so a base
  primitive can sit "DONE-by-typecheck" indefinitely and collapse the first time a consumer
  exercises it. Weight foundational claims by whether a consumer has *run* them, not by how
  settled they look.
- **Planning and tidying are the most seductive form of the vacuum.** Reworking invariants,
  drawing roadmaps, reorganizing files all *feel* like progress and none of them run. A clean
  repo with no running keystone is still a project that does not run. Do the cheap, sharp,
  timeboxed version of any such rework, pointed at the nearest running consumer; never let it
  displace the keystone.
- **Seesaw discipline** (how to keep top-down, structure-first design *without* the vacuum):
  commit to the right *shape* on paper (interface + a roadmap entry); build only the
  **minimal slice a real consumer validates**; defer the rest to consumer-triggered roadmap
  items. The minimal slice must exercise the committed shape's **risk**, not dodge it.
- **Map vs territory:** top-down design is fine *as a map* — its consumer is you reading it to
  plan, so it lives in **docs** (or explicitly-experimental, archived `.dag`). Code is
  **territory** and needs a real consumer. Writing the map in executable `.dag` and mistaking
  it for territory is the trap.

---

## P1: Modeling Faithfulness

**Rule:** Every construct grounds in an identifiable external fact or a structural derivation from one; ungrounded constructs are not valid authorities.

**Why it stands alone:** Faithfulness is upstream of the rest. Performance, decidability, verifiability, and sustainability only matter if the model itself is faithful to the reality it claims to represent. Ungrounded authorities are structural fiction — no amount of downstream rigor recovers from them.

**Grounding is intersubjective, not internal.** Every grounding step should point at a framework with *shared agreement* — mathematical structures, widely-adopted computer science abstractions, standardized machine representations. Internal taxonomies we invented ourselves are not grounding; they're restating the same claim at a different layer. Appealing to intersubjective frameworks lets the model's claims be verified against external consensus, not just against other parts of itself. (See [docs/thesis/epistemic-stacking.md](docs/thesis/epistemic-stacking.md) for the long form.)

**In a closed system, heuristics are never structurally necessary.** A heuristic is a compression — a useful shortcut when the modeler doesn't own the reality being modeled. In *open* systems (natural science, user-input domains), the compression is often irreducible: we are consumers of a reality we didn't construct. But the system this codebase builds is *closed* — we wrote every level, from the bounded substrate primitives up through the algebras that ground user-facing types. Every heuristic in our code is therefore *recoverable*: the richer source either exists somewhere in the substrate we wrote, or can be written. The presence of a heuristic is evidence of a missing upstream fact, not proof that the fact is unknowable. Shortcuts are symptoms, not solutions. (See [docs/thesis/structural-decompression.md](docs/thesis/structural-decompression.md) for the long form — the same closure argument generalizes to why coproduct dissolution always terminates in a closed system.)

### Worked example: integer modeling from the ground up

The integer chain in `dsl/std/{bit,integer,algebra}.dag` illustrates what a fully-grounded composition looks like. Every level points at an external consensus framework; no level invents an internal category:

- **`Bit` = `Classical`** (`Bit` declared in `dsl/std/bit.dag`; `Classical` from `dsl/std/logic.dag`) — a single classical truth value. Grounds in two-valued propositional logic (pre-computing mathematical consensus).
- **`Byte` = `{ bits: List<Bit> }`**, **`Word64` = `{ bytes: List<Byte> }`** — records with declared cardinality intent. Grounds in byte-addressable machine-word standards (IEEE / ISO).
- **`OrderedRing<T>`** (from `std.algebra`) — algebra structure carrying `add`, `negate`, `mul`, and a total order. Grounds in ring theory (centuries of mathematical consensus).
- **`Semiring<T>`** — OrderedRing minus `negate`. Grounds in semiring theory (same lineage, explicit weakening).
- **`Int`** (`dsl/std/integer.dag`) — abstract signed integers as `AbelianGroup<GroupCompletion<Nat>>` (construction-chain layer; not a fixed bit-width alias).
- **`Int64`** — `Compose<Int, MachineWidth<Word64>>`: width refinement of abstract `Int` over the machine-width axis (same pattern for `Int8`…`Int128`; unsigned rows use `Compose<UInt, MachineWidth<…>>` with `UInt = Nat`). Grounding to a Rust primitive (`i64`, …) **must be a proven coincidence, not a bare alias** — this is the rule the invariant sets, not a claim about current substrate state. The faithful form models Rust's `i64` as its *own* fact-bundle (width, signedness, two's-complement representation, from the Rust reference) and grounds the composition to it only by proving the two reduce to structurally equal canonical `Node`s; `type Int64 = i64` would be *hollow* — it asserts that identity while modeling neither side. **The per-language Rust-primitive fact-bundles are not yet landed:** modeling them is the scheduled T-4 per-language rework, gated behind P1-KEYSTONE / T-30 (tracked as dashboard work items; the `src/v4/TASKS.md` ledger is retired). This bullet states the form that rework must take. (See `docs/modeling-discipline.md` Practice 8 — fact-bundle modeling — and `docs/modeling/grounding-worked-examples.md` definition of *coincide*.) Operator dispatch consumes abstract algebra facts, not a parallel `OrderedRing<Word*>` substrate per width class (R3 §1.8 gate #19 `numeric_aliases_align_to_refinements`). **Node constructor form:** each algebra carrier (`OrderedRing<T>`, `ApproximateField<T>`, `BooleanAlgebra<T>`, …) exports a Node constructor returning a `Conj` + named edges fact-bundle — NOT a bare `Instantiation` connective and NOT a positional-only `Conj`; named edges are required for fact-density (T-30), content-hash stability (B1), and T-9 coercion-fold walkability. This is the T-2 deliverable form that unblocks T-4 Wave 2b bridge-symbol dissolution (T-2 is likewise a dashboard work item, not a ledger row).

Downstream consumers (lens queries, algebra-driven operator dispatch, cost algebra) read these declared structural facts. Operator symbols come from the algebra's field declarations, not a per-target string table. Sign semantics come from the witness, not a flag. The chain is *verifiable against consensus at every step*; that is what "grounded in a declared source" means concretely.

### Worked example: effect algebra (idempotency as partitioned structure)

`src/v3/std/effects.dag` models operation effects as inhabitants of algebraic structures declared in `dsl/std/algebra.dag`:

- **`ReadEffect`** — identity function on state. Trivially idempotent.
- **`UpsertEffect`** — lattice **meet** on `Map<K, V>`. Setting a key to a value, applied twice, equals once. Grounds in lattice theory (Birkhoff 1940).
- **`DeleteEffect`** — lattice meet with bottom on a key. Applied twice = once.
- **`AppendEffect`** — free monoid concatenation. Grounds in monoid theory.

The v3 copy partitions `EffectShape` into `IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)` — idempotency is now encoded in the *outer variant*, not a separate boolean classifier. `is_idempotent_effect(shape) -> Bool` becomes a trivial match (`IsIdempotent → true`, `IsBreaking → false`) whose work is already done by the variant discriminator. The pattern: when a behavioral property is definable from algebra inhabitance, it migrates onto the variant rather than persisting as a side flag. (The legacy `dsl/std/effects.dag` still carries `is_idempotent_effect` as a standalone classifier function; the partitioned v3 form is the dissolution target. Consumer migration from accessor-call to direct pattern match is in progress.)

External authority: lattice theory (Birkhoff 1940), effect algebras (Foulis & Bennett 1994), idempotent semirings (dataflow analysis, routing algebras, tropical geometry).

### Worked example: termination via descent evidence

`dsl/std/termination.dag` models the *proof* of termination as a structural artifact:

- **`DescentEvidence` = `Strict` | `NonIncreasing` | `DescentUnknown`** — at each recursive call edge, the ranking dimension either provably decreases, possibly stays the same, or is unknown.
- **`DescentEvidence` inhabits `BoundedLattice<DescentEvidence>`** with top = `Strict`, bottom = `DescentUnknown` (fail-closed), meet = conservative branch merge, join = optimistic branch merge.

The substrate's stated design principle (from `termination.dag`'s header): the complexity analyzer *should be* a **checker** of termination proofs constructed from this model rather than a **discoverer**. Currently the substrate carries the proof structure as typed facts; the fail-closed analyzer that validates proofs against the model is design, not landed behavior — today's analyzer derives cost from bounded structure rather than validating proofs (the `docs/invariants/decidability-invariant.md` subdoc was deleted in the visibility flip; P4 below is the surviving statement of the invariant). What is landed: the substrate-level evidence carriers, so P4's bounded forward execution premise has a typed representation to reason about. The same `BoundedLattice` machinery types `DescentEvidence`, cost-algebra merges, and effect composition — one algebra, many consumers.

External authority: well-founded relations (Zermelo 1904, von Neumann 1929), ranking functions (Floyd 1967, Turing 1949), size-change termination (Lee, Jones, Ben-Amram 2001), lexicographic + multiset orderings (Dershowitz, Manna 1979).

### Worked example: kernel calculus grounding (substrate ≡ typed total λ-calculus + extensions)

The 5-Behavior substrate (`Behavior::Value | Transform | Branch | Loop | Bind` in `src/v3/std/substrate.dag`) is a *typed total* fragment of lambda calculus + structural coproducts + bounded recursion. The mapping is direct: `Transform` is application `(M N)`; `Bind` is let-binding (sugar for `(λx.N) M`); `Arrow.body` is the lambda abstraction `λ(params).body`; `Branch` extends with primitive coproducts (System F + sums); `Loop` replaces the Y-combinator with bounded recursion (totality choice — same as Coq/Agda/Idris-with-totality).

α-conversion is satisfied by construction (NodeId-based binding has no name shadowing); β-reduction is being reified as a substrate citizen via the PB-Runtime interpreter-as-data work; capture-avoiding substitution is structurally satisfied by DAG-reference identity. The lens framework + `Witness<C>` + `DimensionReport` are **analyses on top of the calculus** (algebraic-effect-handler-shaped per Plotkin & Pretnar 2009), not calculus citizens.

External authority: Church (1932/1936), Curry-Howard correspondence, System F (Girard 1972 / Reynolds 1974), Calculus of Inductive Constructions (Coquand & Huet 1986), Agda's totality discipline. The grounding carries 3 intentional divergences (Loop / Branch / n-ary) and 3 research-level open questions (η-equivalence, confluence, strong normalization).

### Problem shape: Ungrounded heuristic

A downstream stage computes a fact that was never authored — complexity score, provenance category, likely intent — by applying rules to nearby signals. The heuristic has no declared source; outputs are plausible but trace to nothing. Every corner case becomes another rule; the rule set grows unboundedly.

**Solution shape:** Find where the missing fact should have been authored. Add a typed carrier at that layer so the fact propagates forward. The heuristic evaporates; downstream consumers read a declared fact.

**Receipt:** A complexity-scoring pass once encoded several thousand lines of heuristics to rebuild variant provenance during inference. Variant-constructor information had been silently discarded during parse. Dissolution: carry variant-provenance as a typed fact from parse forward; the heuristic pass evaporated.

### Problem shape: Unnamed substrate target

A design claim asserts "this needs no new substrate element — it composes from existing ones" without naming which element carries the new semantic. The claim is ungrounded; reviewers can't verify it, and implementation quietly adds the missing element without a ratchet receipt.

**Solution shape:** Every design commitment names the existing substrate element whose edges or fields will carry the new semantic. If no element qualifies, the design must declare a substrate extension with a dissolution ratchet. "No substrate change" is not a permissible claim without a named target.

**Receipt:** A design once claimed multi-target lowering composed from existing Arrow + Body primitives. On implementation it turned out `Arrow.body` didn't carry target-binding information. Fix: explicit substrate extension for external realization on `Arrow.body`; every future design commitment names its substrate target.

### Problem shape: Hollow alias

A type that models an external spec primitive is declared as a **bare alias** — `type RustI32 = Int32`, `type CppInt = Int` — and stops. It asserts an identity between its subject and a `std/` carrier without modeling its subject's facts and without evidencing the identity. The spec carries facts (width, signedness, representation, range, encoding) the alias silently discards. The alias *looks* grounded — it points at a declared `std/` type — but the grounding is fiction: the two are not the same thing, and nothing proved they were. Worse, a hollow alias is structurally minimal, so it passes every shape-checker — the hollowness is invisible unless a reviewer asks "does the spec carry facts this drops?"

**Solution shape:** Model the facts (*invent or reuse*, never bare-alias). Either invent a fact-bundle — a `Conj` whose fields are the facts the spec states — or reuse a `std/` carrier *with cited evidence that the two coincide* (groundings reduce to structurally equal canonical `Node`s; see `docs/modeling/grounding-worked-examples.md` *coincide* bar and **PR review**, Practice 8 — no maintained `DECISIONS.md` ledger). Default SEPARATE for extdeps (we don't fully know those systems), REUSE for internal layers (we wrote both sides) — always naming the evidence. Enforced by `docs/modeling-discipline.md` Practice 8 and its structural fact-density gate (T-30, a dashboard work item). Kernel-ambient atoms (`Bool`, `Char`) are genuinely atomic and exempt — the spec carries no further facts to drop. The hollow alias is one *trigger shape* of the broader **JIT-modeling** obligation (MODELING.md M12): a change that touches or introduces *any* unmodeled concept — hollow alias, raw `Int`/`String` for a domain quantity, stringly closed-set, homeless reference — models it in that PR, never deferring with a `P-*`/TODO placeholder; the merge-blocking review trigger is Practice 13.

**Receipt:** v4 decision D2 modeled external language primitives as alias-identity declarations (`type RustI32 = Int32`). Each passed structural review precisely because it was minimal. The operator reversed D2 (2026-05-17): the aliases asserted unproven identities and modeled nothing. Dissolution: fact-bundle modeling — every external primitive carries the facts its own spec states; deduplication onto `std/` happens only on proven coincidence.

### Problem shape: Hand-rolled derived operation

**Do not hand-roll a derived operation.** If a function's behavior is determined entirely by the shape of a modeled type, it is re-deriving something the compiler already derives. The deficiency is in the model, not the code — model the missing fact; do not hand-roll the operation.

A function walks a modeled structure, projects a property from a variant shape, threads a standard carrier by hand, or templates an emitter from local strings. The behavior is fixed by the model's shape, so the code is a second authority for a fact the model should carry or derive.

**Solution shape:** Add or consume the substrate primitive that derives the operation — a catamorphism, traversal, structural fact, grammar model, or other declared carrier — and delete the hand-rolled local function. When the primitive is missing, the *exception path is narrow*: a valid 🟡 per `docs/modeling-discipline.md` Practice 4 — the mark binds a dissolution plan (gate kind `feature:`/`consumer:`, the named arrival, the owning substrate PR or dashboard work item, and the dissolve-on-arrival follow-up). A 🟡 with no bound plan is not a valid 🟡 and blocks merge; so does an untracked hand-rolled derived operation.

**Marking is not authorization.** A 🟡 records debt; it never converts a finding into a pass. The PR description enumerates every new 🟡 it introduces, and the reviewer accepts each by name — the default review posture is **debt-negative**: a PR should dissolve at least as many 🟡 marks as it adds, and piling on more requires explicit justification in review (Practice 4 / Calibration in `docs/modeling-discipline.md`). A PR whose primary deliverable could be model rows but lands as 🟡-marked compiler code is the wrong shape, not tracked debt.

**Receipt:** Practice 10 in `docs/modeling-discipline.md` names the derived-operations registry and the dissolution findings (`walker`, `traverse`, `predicate`, `carrier`, `emit/template`, `nominalization`, and coproduct dissolution). It is the review rubric for deciding whether a local helper is real logic or a duplicated derivation; PR #4627 (four registry findings landed through an approving review while the rubric was deleted) is the worked example recorded there.

### Problem shape: Stage carries decisions the model should

**A finished compiler stage is one fold over its model — `stage(x) = fold_carrier(x, algebra(model))`.** This is the stage-level reading of *Do not hand-roll a derived operation*: a stage that walks its input with per-case `match` arms, `_go` accumulators, and `_bounded` fuel is hand-rolling the catamorphism the compiler already derives (`fold_node` / `fold_grammar_expr` / the frontend folds), one arm at a time. The decisions those arms make are facts the model should carry as data rows.

The rule is a **litmus, not a ban.** Non-fold code is permitted but is a *signal*, sorting into exactly two categories — there is no third:

- **A named, separated irreducible kernel.** A fixpoint solver (`solve_constraints`), char-class matching, real arithmetic — these are not catamorphisms and are *meant* to stay as one named function. *Fold the traversal, name the kernel.* Terminal, not debt. (Worked instance: the `04_infer` constraint-*gather* folds onto `fold_node` while the fixpoint solver `solve_constraints` stays a named kernel — the gather/solver split in `gunbc-planning/stage-fold-collapse-plan-2026-06-11.md` §1.6 / §4. The kernel is irreducible because its call graph is not its data graph — Practice 12's discriminant.)
- **Un-migrated modeling.** Any other non-fold control-flow is the code making a decision the model has not absorbed. It is **measured modeling debt**: the decision belongs as a model row, and its residence in the stage is tracked, not permanent.

The volume of non-fold residue in a stage is therefore a *measure* of how much of its decision-making still lives in code rather than the model. This connects to **Heuristics Indicate Lost Structure** (above): a heuristic is a per-fact symptom; a non-fold stage is the same symptom at stage scale. **Home of record:** MODELING.md M11; review rubric: `docs/modeling-discipline.md` Practice 12; the frontloaded collapse program: `gunbc-planning/stage-fold-collapse-plan-2026-06-11.md`. `05_emit` (`emit = serialize_target ∘ translate`, i.e. `bind_outcome(translate, serialize_target)`, 43 lines) is the landed existence proof of the **composition** form: a stage may also be a thin zero-residue composition of fold-backed stages, glued only by monadic sequencing (`bind_outcome` / `∘`) — that glue is plumbing, not residue.

**Solution shape:** Move each per-case arm to one row in `algebra(model)`; delete the `_go` accumulator (the fold owns recursion) and the `_bounded` fuel (a structural fold terminates by construction) in the *same* change that repoints the consumer onto the fold. Add the algebra row and delete the arm in one PR — grafting the fold beside the surviving arms is the cementation anti-pattern (the file must shrink).

### Problem shape: Nickname (type name mismatches concept)

**A type's name must be what the type is — not a convenient label for something else.** A type named `FooBar` is the composition of `Foo` and `Bar`; a type named `NodePath` is `Node` composed with `Path`. If the name does not reflect the actual structure, the type is a nickname — and nicknames are modeling violations.

In the compiler especially, nicknames compound: consumers read the name and build mental models from it; if the name lies, every consumer inherits the lie. The deficiency is in the name, not the code.

**Solution shape:** Rename to reflect what the type actually is. If the concept has no precise name yet, find one — DFS `std/` for the right carrier (MODELING.md M9) before coining a new one. If the type is a composition of two existing concepts, the name should say so.

**Canonical example (ratified 2026-05-27):** `ModulePath` was `FreeMonoid<ModulePathSegment>` where `ModulePathSegment = { name: Symbol }` — a qualified identifier sequence (`[v4, std, algebra]`), not a path through any graph. The `Path` concept in `std/node.dag` is `{ steps: List<Symbol> }` — a route through Node edges. `ModulePath` was a nickname for `QualifiedName`; renamed accordingly. `ModulePathSegment` was a nickname for `Symbol`; deleted.

### Problem shape: Internal vocabulary for a canonical concept

**Rule:** When a concept has a recognized definition in formal CS literature, use the canonical name and structure. Do not coin an internal name for it. Cite the authoritative source (Wikipedia article, RFC, ISO standard, or textbook) in the module header comment of the file that declares the type.

**Why it belongs in P1:** Faithfulness is intersubjective — grounding must point at shared external consensus. Renaming a canonical concept internally asserts the internal name is as authoritative as the external one, which it is not. It also creates a hidden mapping burden: workers must learn the internal name, learn the mapping, and trust the fidelity of that mapping. The external name carries that fidelity for free.

**Problem shape:** A module named after a pipeline stage (e.g. `compiler/tokenize.dag`, `compiler/parse.dag`) defines types whose concepts predate the pipeline — lexical analysis, context-free grammar, production rules. The module that *operates on* a concept is not the authority for its *definition*. Names like `GrammarProduction`, `ModeledLexRules`, `GrammarSchema` are internal nicknames for concepts the field already named (`Production`, `LexRuleSet`, `Grammar`).

**Solution shape:** Before naming a new type, search for its canonical name in the relevant field. If one exists, use it. Place the type definition in a `std/` module named after the concept domain (e.g. `std/lexing.dag`, `std/grammar.dag`), not after the pipeline stage that happens to use it first. The pipeline stage imports from `std/`; it does not define vocabulary.

**Enforcement:** New substrate type names corresponding to recognized CS concepts must cite the grounding source in the file's module header comment using Practice 9's per-carrier anchor form (e.g. `// Anchor: https://en.wikipedia.org/wiki/Lexical_analysis`). Reviewers should ask "does this concept have a canonical name in the field?" before approving new type declarations in `compiler/` or `extdeps/`.

**Receipt:** `src/v4/std/lexing.dag` and `src/v4/std/grammar.dag` are landed — canonical lexical and grammar carriers grounded in Wikipedia *Lexical Analysis* and *Formal Grammar*. `LexRule`/`LexRuleSet`/`LexPattern` and `GrammarProduction`/`GrammarExpr`/`GrammarSchema` are defined in `std/` only; `v4.compiler.tokenize` and `v4.compiler.parse` import from `std/` (zero redefinitions in compiler/). All `extdeps/` consumers import from `std/` (0/45 verified on `main`).

### Problem shape: Existing implementations are not automatically correct

**Rule:** An approved or landed implementation is not evidence that its model is correct — it is only evidence that it passed review at the time. Every implementation is subject to re-examination when a better or more holistic model becomes visible. Prior approval does not foreclose redesign.

**Why it belongs in P1:** Faithfulness is not a one-time gate at merge. Approval meant "no blocking concern at review time," not "this is the best possible model." If workers treat landed code as settled fact, violations compound silently. The discipline of interrogating existing models — asking "given what we now know, is this still the right shape?" — is how the codebase improves.

**The calibration:** This principle does not license endless refactoring. The bar for revisiting a landed model is: (a) a clearer external grounding exists (as with the lexing/grammar vocabulary move), or (b) the existing model creates cascading wrong-direction dependencies, or (c) a more holistic solution would reduce total lines/concepts, not increase them. Cosmetic renames, subjective preference changes, and "could be marginally cleaner" are not sufficient — those produce churn without structural improvement.

**Concrete check:** When a worker encounters a type, function, or module and thinks "this seems slightly off but it's landed," they should ask: does a canonical external model exist that this should conform to? Does the current shape create cross-layer dependencies? Would conforming reduce concepts or add them? If the answer to any of these is yes with clear evidence, surface it — don't silently implement on top of a wrong foundation.

**Problem shape:** Worker finds that `extdeps/` files import from `compiler/` and treats this as load-bearing because multiple PRs approved it. Assumes the existing pattern must be right. Builds T-4.18 on top of the violation. Violation compounds.

**Solution shape:** Treat every landed model as a hypothesis, not a proof. When external grounding reveals a better shape, surface it — even if it requires re-examining previously approved work.

**Receipt (substantial dissolution):** The extdep→compiler coupling is resolved — all `src/v4/extdeps/*.dag` files (including `python.dag`, `rust.dag`, and 43 others) now import zero symbols from `v4.compiler.*` (verified: 0 extdeps→compiler import edges on `main`). `TargetModel` authority has been promoted to `src/v4/std/target_model.dag`; `v4.compiler.target_carriers` is now a hub-route shim that re-exports from `std/`. `std/lexing.dag` and `std/grammar.dag` are landed. **Dissolved:** `LanguageModel` authority is in `src/v4/std/language_model.dag` (typed fact-bundle); `v4.compiler.target_carriers` hub-routes it; `workflow/bootstrap.dag` imports from `std/`. Multiple PRs approved the prior hollow alias before the cross-layer import rule made the violation explicit; prior approval is not proof the model is correct (see above).

### Problem shape: Pattern presence as grounding

**Existing code is not evidence of correctness.** A worker sees that a pattern already appears in the codebase — an import structure, a naming convention, a coupling choice — and replicates it in new code on the basis that "it already works this way." The existing usage is treated as an implicit grounding: if it were wrong, it would have been caught. That inference is false. Code reaches reviewers in every state from correct to inadvertently approved to drifted from a dissolved rule; presence alone proves nothing about invariant compliance.

This failure mode is especially damaging in cross-file pattern replication: a single wrong decision, once in the corpus, propagates as a template for every subsequent worker who sees it. The wrong pattern appears authoritative precisely because it is already there.

**Solution shape:** Before replicating any existing pattern, verify it against the invariants directly. The question is not "does this match existing code?" but "does this satisfy the invariants?" If the existing code itself violates an invariant, it is queued for dissolution — not used as a template for new code. When a reviewer approves, the approval is against the invariants, not the prior pattern. *Prior approval of a wrong pattern is not forward authorization.* If you find an existing violation while implementing something new, open a follow-on item for dissolution; do not silently inherit the violation.

**Receipt:** `extdeps/languages/*.dag` files imported directly from `v4.compiler.parse` and `v4.compiler.tokenize`, coupling external language descriptions to internal pipeline stages. New workers replicated this pattern across 14 language files because it was already present in the corpus. The operator had approved the original file; new workers treated that as license to repeat the shape. The pattern was wrong — extdeps should describe languages through declared substrate types (`std/`), not internal compiler stages. Dissolution: extdeps meet the compiler through declared abstraction boundaries; the coupling is the invariant violation, not the feature.

### Procedure: substrate-fact introduction (decision procedure for new modeling)

This procedure operationalizes P1 — workers stuck on "should I add a new type / variant / field?" can self-serve by walking these three checks in order.

**Step 1: DAG-ancestor check** — search for the shared underlying modeling.

> *"Before declaring a new substrate type, ask: what's the shared underlying concept? Does an ancestor type exist in the substrate already?"*

If a parent concept exists, the new fact attaches via inhabitance (algebra-inhabitance, parametric instance, structural sub-type), NOT as a sibling type. Sibling types proliferate without grounding; inhabitance preserves the DAG. Worked examples:
- `BoundDeclaration` is not a sibling of `CardinalityBound` / `SizeBound` / `LoopBound` — those all share the parent `Interval<D>`. The dissolution: declare `Interval<D>` as the parent; existing types retrofit as instances.
- `Cost<Unit>` is not a sibling of `Duration<Seconds>` / `Money<USD>` — they all share the parent `Dimension<Unit, Carrier>` (one substrate type, multiple `Unit` instantiations).
- `Lens<C>.sequential` is not a parallel `compose + unit` field pair — both project from `Monoid<C>`, the structurally-correct parent.

**Step 2: Coproduct-vs-coordinate check** — distinguish alternatives from coordinates.

> *"If proposing `Foo = A | B | C`, ask: are these alternatives (one-at-a-time) or coordinates (all-at-once)?"*

If any single inhabitant carries values for ALL variants simultaneously, the variants are coordinates of a record (`{ a: A, b: B, c: C }`), not a sum type. Sum types compress what should be coordinates and pay the cost downstream. Per [`docs/thesis/structural-decompression.md`](docs/thesis/structural-decompression.md): dissolve into coordinates; the closed system guarantees the dissolution terminates. Worked examples:
- "Cost = Time | Space | Energy" is wrong shape — every cost has time AND space AND (potentially) energy components. Dissolve to `{ time, space, energy }` (record).
- "BoundDeclaration = ExactBound | AnyBound | PlatformDependent" — these ARE alternatives (a bound is one kind, not all kinds). Sum type is correct here.
- "Resource = CPU | Memory | Disk" — coordinates if a single allocation has all three; sum type if any one allocation is exactly one kind. Apply the test.

The "stop at user-input boundary" rule: dissolve coproducts that compress structural facts; preserve coproducts where the USER genuinely picks among alternatives (e.g., an enum the user types in source).

**Step 3: Primitive-vs-lens-extensible check** — classify substrate-declared vs lens-extensible.

> *"If introducing a leaf type (Time, Bits, Joules, Meters), ask: is this a fundamental primitive or a lens-extensible label?"*

Fundamental primitives (physics, computation, mathematics) declare as substrate primitives sibling to existing primitives — not lens-extensible coproducts. **Lens-extensible** (NOT "user-extensible" — the compiler IS a user of its own system; the lens framework is uniform whether authored by compiler team or user) vocabulary belongs in lens-framework instances or domain dimensions where the lens-framework's structural inhabitance lets any author add new variants the same way. Worked examples:
- `Bits`, `CPUCycles`, `Joules` declare as substrate primitive types (sibling to `Meters`, `Seconds`, `Kilograms` in `src/v3/std/dimensions.dag:93-99` — SI base units). Not `Resource = ... | LensExtensible<Name>`.
- A `Lens<TenantFlow>` instance declares `CapSet` and `Capability` as lens-extensible domain types — those ARE lens-extensible regardless of who authored the lens (compiler team CapabilityViolation kind variants extend the same way as a user-authored lens). Per `feedback_groundedness_gates_lenses`: language vocabulary is primitives + namespacing; the lens framework's inhabitance is the namespacing.
- `BoundDeclaration` variants (`StaticBound`, `PlatformDependent`) are substrate-declared because they're computational primitives (every target's bound has one of these shapes); not lens-extensible.

**Worked-example tracking:** every time a major design decision applies this procedure, the design doc cites which steps were run and what they revealed. Examples lived in `docs/design-emission-model.md` Q1, Q2, Q3 (lock paragraphs include the DAG-ancestor + coproduct + primitive checks) and `docs/design-lens-framework.md` Lens<C> primitive section (sequential: Monoid<C> structural inhabitance per Step 1) — both deleted in the visibility flip; recover via git history. New design docs cite the steps inline.

### Related rules (home-of-record here)

- **Modeling Faithfulness Invariant** — canonical statement
- **Bounded Substrate Seed** — Rust-native seed items must remain narrow; undeclared seed growth is ungrounded
- **Design Commitments Must Name The Substrate Target** — unnamed substrate = ungrounded claim
- **Heuristics Indicate Lost Structure** — heuristic output traces to a dropped upstream fact, not to a declared source
- **Do not hand-roll a derived operation** — if behavior is fixed by modeled shape, model the missing fact or consume the derived operation
- **Documentation Describes Live State** — aspirational docs describe a reality the codebase doesn't embody
- **Substrate-Fact Introduction Procedure** (above) — operational decision procedure for adding new substrate types/variants/fields
- **Model names must reflect what they are.** A type named `FooBar` must be a structural composition or projection of `Foo` and `Bar` — not a convenient label for something else. Nicknaming is a modeling violation because it claims structural identity the type does not have. In the compiler especially, a type name that implies a concept it does not embody creates false mental models for workers and reviewers. Canonical example: `ModulePath = FreeMonoid<ModulePathSegment>` was a nickname for `FreeMonoid<Symbol>` (the declared identifier of a code unit); the name implied graph traversal (`Path { steps: List<Symbol> }` in `std/node.dag`) when no such relationship holds. Correct form: `QualifiedName = FreeMonoid<Symbol>` — the name states exactly what it is. When a type name is wrong, fix the name before building on it. Enforcement via structural lenses is in progress.
- **Internal vocabulary for a canonical concept** — canonical CS names belong in std/, not in compiler/-internal modules
- **Existing implementations are not automatically correct** — landed code is a hypothesis subject to re-examination when clearer grounding or structural evidence appears; prior approval does not foreclose redesign

---

## P2: Boundary Discipline

**Rule:** Boundaries carry enough declared information for mechanical consumers, and every fact lives in exactly one authoritative place.

**Why it stands alone:** The cost of change is proportional to how many files encode the same fact. When a fact lives in two places — even "temporarily" — new consumers must choose between them, and the choice drifts. Consolidation later is the hardest refactor in the codebase. This principle prevents duplicate authority from entering the system in the first place.

**When a boundary counts as landed:** the declaration exists, the realization (Rust binding / data table) exists, *and a generated consumer proof exists* — i.e., generated code somewhere in the compiler consumes the declared surface. Declaration alone is staging, and a hand-written consumer is not generation.

### Problem shape: Parallel authority

A canonical accessor exists (e.g., a reflected substrate function), but a second path gets scaffolded next to it — a local walker, a parallel lookup table, an inline match. Each new consumer picks whichever is nearer. Drift is inevitable because divergence costs nothing.

**Solution shape:** Expose the canonical answer through the declared substrate surface; delete the parallel path in the same change. If the canonical answer doesn't exist yet, extend the substrate reflection *first*, then migrate consumers — not the other way around.

**Receipt:** Id → declaration lookup once had both a canonical Rust accessor and a parallel `.dag` list-walker that each consumer picked between. Dissolution: add the missing reflected accessor, delete the walker, migrate consumers to the typed accessor — the parallel authority disappeared.

### Receipt: language grounding in `extdeps/`

External language slices ground primitives through declared algebra inhabitants
and fact-bundles in `std/`, not spelling-only resolver twins. A bare alias plus
a label asserts grounding without demonstrating it; the faithful form references
a single authority (for example `BooleanAlgebra<Bool>` from `std/logic.dag`) and
proves coincidence structurally. Whole-tree `gunbc compile --source-root src/v4`
is the authoritative parse gate for v4 modules.

### Problem shape: Consumer reverse-engineers storage shape

A downstream stage reads a lower layer not through its declared accessors but by traversing its internal storage — walking a list, checking a field convention, matching on a tag. The consumer must evolve in lockstep with the lower layer because storage shape has become part of the contract by accident.

**Solution shape:** Declare a typed query surface on the lower layer exposing the exact facts the consumer needs. The consumer reads through the typed query; storage shape underneath is free to change without touching consumers.

**Receipt:** Lens implementations once reached into hand-written storage details of the substrate, so every storage refactor cascaded into lens edits. Dissolution: declared substrate query functions (L-7); lenses became consumers of a typed boundary instead of the storage itself.

### Problem shape: Cross-layer import (dependency direction violation)

**Rule:** Module-layer dependencies form a strict DAG. The declared layer order is:

```
std/        — universal primitives; may be imported by any layer
extdeps/    — external dependency modeling; imports std/ only
compiler/   — pipeline stages; imports std/ and extdeps/
workflow/   — orchestration; imports std/ and compiler/
```

An import edge pointing opposite to this order (`extdeps/ → compiler/`, `std/ → compiler/`) is a violation regardless of whether the imported symbol "happens to work." The compiler reads external models; external models do not read the compiler.

**Why it belongs in P2:** Every wrong-direction import creates a hidden parallel authority: the type definition lives in the importing layer's dependency, not in the layer that conceptually owns it. Every consumer of the wrong-direction import inherits that hidden coupling. It also violates the homomorphism premise: the compiler processes Node graphs uniformly; if extdeps/ types are defined in compiler/, the compiler owns the schema of something it should be reading, not defining.

**Problem shape:** `extdeps/python.dag` imports `LexRules`, `GrammarProduction` from `v4.compiler.tokenize` / `v4.compiler.parse`. The compiler now owns the schema of the language model it is supposed to read. Adding or renaming a compiler type forces updates to all language definitions. Moving the compiler requires moving the language definitions.

**Solution shape:** Any type shared between `extdeps/` and `compiler/` belongs in `std/`. Both sides import from `std/`; neither imports the other.

**Enforcement:** PR review must check: does any `extdeps/` file import from `v4.compiler.*`? If yes, the import is a violation unless the type being imported does not belong in std/ (file a finding and require the move). Same check for `std/` importing from `compiler/` or `extdeps/`.

**Receipt (substantial dissolution — one residual):** All `src/v4/extdeps/*.dag` language/format files now import zero symbols from `v4.compiler.*` (0/45 verified on `main`). `TargetModel` authority is in `src/v4/std/target_model.dag`; `v4.compiler.target_carriers` re-exports it (hub-route only, no local redeclare). `src/v4/std/lexing.dag` and `src/v4/std/grammar.dag` are landed. **Dissolved:** `LanguageModel` authority is in `src/v4/std/language_model.dag`; `v4.compiler.target_carriers` hub-routes it; `workflow/bootstrap.dag` imports from `std/`.

### Problem shape: Target knowledge in compiler code (references, not just imports)

The cross-layer rule above polices `import` lines, but the boundary is about *knowledge*, not syntax. Compiler code that **references** target-layer or fixture-layer facts literally — an atom like `^ts_inhabitant_number` compared in a `compiler/` match arm, a fixture binding name like `^dag_binding_param_x` tested by equality, a token layout for one target spelled inline — has imported target knowledge without an import statement. No import-direction check fires, but the compiler now owns per-target facts that belong in `extdeps/` / model rows, and every new target or fixture forces a compiler edit (cost-of-change > 1).

**Solution shape:** Per-target and per-fixture facts are model data — rows in `std/` / `extdeps/` declarations that compiler folds consume generically. A literal target atom in `compiler/` code is a finding with the same weight as a wrong-direction import. Corollary: **a PR adding a new file under `src/v4/compiler/` is itself a review finding, default-block** — the author states, and the reviewer accepts by name, why the contents cannot be model rows plus an existing fold (`docs/modeling-discipline.md` Practice 10, rows 3/4 escalation).

**Receipt:** PR #4627's `06_value_expression.dag` compared `^ts_inhabitant_number` and `^dag_binding_param_x` literally in compiler match arms and inlined a `[BoundToken, FixedToken, BoundToken]` TypeScript call layout — none flagged by the import check, all boundary violations. Dissolving them into realization rows is a blocking precondition of the emit-breadth milestone.

### <a id="p2-host-process-boundary"></a>Host-process boundary discipline

**Rule:** Predicates that spawn **host processes** to evaluate a claim (e.g. `std::process` / `ExecuteCommand` in the test runner) meet the same boundary bar as the rest of the compiler: the contract is **typed and single-authority**, not a growing pile of string-heuristic special cases. Concretely:

- **(a) Typed results, not string authority.** Outcomes for the host predicate are **variants or carriers**; consumers do not use free-form **diagnostic strings** (prefix probes, `contains` checks on mixed channels) as the long-run structural contract. DB-1’s typed-carrier discipline applies; C-5 is the anti-pattern. Where the implementation is still string-shaped, treat that as **tracked debt** with a named dissolution, not a steady-state.
- **(b) Isolated logical-child I/O.** The **logical child** (the command under test) is not allowed to **share a diagnostic channel** with a wrapper, launcher, or sandbox setup so that **wrapper or harness artifacts** can be mistaken for claim evidence. The typed channel to the claim must be separable from setup noise. (C-5-adjacent when a shared stream becomes the only signal.)
- **(c) Setup failure ≠ logical exit.** **Harness or sandbox setup failure** (e.g. cannot enter namespace, `unshare(1)` diagnostics, spawn layout failure) is a **separate typed outcome** from “the user command ran and exited *k*” — the two may not be collapsed or reinterpreted in one hop.
- **(d) No implicit re-execution.** The runner does **not** re-run the user’s command as an **implicit recovery** path. A **second** execution of the same logical command is only allowed when it is **explicit policy** (documented, bounded, and reviewable) — not a hidden fallback to “try again and hope the failure mode changed.” (When policy explicitly allows a bounded confirmation run, the double-exec and idempotence assumptions are part of the stated contract, not a silent escape hatch.)
- **(e) Drains and authority.** Any `read` on a child stream for **presource management** (e.g. **nonblocking drain** so a pipe cannot fill and stall) must not **erase** byte ranges that a **different** step later treats as **evidence** for a distinct outcome (e.g. setup-failure vs exit mismatch on the *same* fd in quick succession). Either those bytes are **captured** into the same **bounded** budget the downstream scan uses, or the later step is documented as not depending on that stream; **no split-brain** where the drain is discard-only and a later string scan is still claimed as the authority. (Same *authority* failure mode: managing back-pressure vs. using the same stream as a typed error channel.)

**Receipt / audit hook:** T-PB-B and PB-Runtime consume this through `src/v3/compiler/src/test_runner.rs` (`evaluate_execute_command_host_outcome` and shared command parsing). The structural move is: **one audit** of the execute path against (a)–(e) when touching host spawns, then **refactor into a typed decision/result model** if the audit is not obvious; ad hoc string heuristics at the same seam without an audit are a smell. (Historical work-stream provenance: PR and review threads, not this file — the `docs/review-findings/` archive was deleted in the visibility flip.)

### Problem shape: Task-scope drift — out-of-brief CI modification

A PR's brief assigns a bounded scope (language fill, extdeps/ substrate, refactor sweep). While working, the worker finds an unrelated CI issue — a `/tmp` reference that should be `${RUNNER_TEMP}`, a stale runner spec, a cache-key rename — and includes the fix in the same commit. The CI change may be correct in isolation, but it bypasses the single-authority contract for CI configuration. **Authority:** `.github/workflows/ci.yml` is the direct hand-edited source of truth. (The prior `src/v4/workflow/ci.dag` model — once slated to become a T-24 CI data-model that would emit ci.yml — was a descriptive-only mirror with no runtime consumer and was deleted (operator GO 2026-06-08); there is no `.dag` CI carrier and ci.yml is not generated.) Bundling incidental CI edits into unrelated PRs bypasses that single-authority contract and makes the history unreadable.

**Solution shape:** Workers do not modify `.github/workflows/ci.yml` unless that file is explicitly listed in their task brief. Authorized exceptions: (a) the task brief names CI files explicitly; (b) a binary/toolchain rename that must update cache-key paths throughout (the rename scope includes CI); (c) T-38 dissolution PRs with named CI step targets. Any real CI issue found outside a worker's brief becomes a separate work item — not a bundled fix.

**Receipt:** A GrammarExpr→FormalProduction refactor PR included a `/tmp` → `${RUNNER_TEMP}` fix (cleaned up before merge). A Wasm Wave 2a language fill PR bundled the same class of fix (stripped before merge, 2026-05-28). The binary-rename PR (#3826) is the ratified exception: renaming `v2-compiler` → `gunbc` spans CI cache paths by definition.

### Related rules (home-of-record here)

- **Host-process boundary discipline** — P2 subsection [above](#p2-host-process-boundary) (T-PB-B / PB-Runtime; PB-Runtime landing in `test_runner` ExecuteCommand)
- **Lenses Are Substrate Declarations** + **Reflected-Facts-When-Landed**
- **Every Dependency Is A Substrate Fact**
- **Minimal Information Per Interface**
- **Layer Opacity** + **Semantic Authority After Lowering**
- **Boundary Sufficiency** + **Explicit Boundary Contracts**
- **Emission Is Translation, Not Decision-Making**
- **No Duplicate Representations** + **No Parallel Implementations** + **Single-Authority Metadata**
- **Root-Cause Depth Invariant** — fix at the deepest unsound boundary, not the first downstream symptom
- **Cross-layer import (dependency direction violation)** — workflow/ → compiler/ → extdeps/ → std/; imports only toward `std/`; no reverse edges
- **Performance Invariant** + **Facts Flow Forward** — redundant work is dependency modeling at the wrong boundary
- **Verification Predicates Are Substrate Consumers** — verification is not its own authority
- **The One Boundary** (from Verifiability) — verification crosses target-specific realization only at declared boundaries
- **L-7: Lenses Consume Declared Substrate Query Functions**
- **L-8: Lens Rust Surfaces Preserve Typed Failure Carriers**
- **DB-5: Substrate Keyed Lookup Is Single-Authority**
- **E-6: No Target-Spec Field Without A Same-PR Consumer** — a target-spec field is real only when a consumer lands with it (same-PR-consumer is a boundary-validity rule, not a progress rule)
- **E-9: External Realization Lives On Arrow.body** — declares the single authority for external semantics
- **DB-14: Substrate External Primitives Materialize Through Declared Arrow.body Plus Target Bindings** — same single-authority principle for external primitives

### Reflection evidence is not structural proof

`reflect_program_dag_nodes_in_file` (via `lens_apply::substrate_reflection::reflect_behavior_list`) projects **complete** substrate-shaped `Behavior` nodes into `FieldValue` (reflection-completeness contract, locked 2026-04-29): every declared field on each `Behavior` variant and nested carriers such as `WorkflowEffect` / `LoopBound` / `BranchPath` is reflected structurally, with no execution semantics and no per-consumer narrowing.

Reflection is **authored against** `src/v3/std/substrate.dag` and uses the same **positional variant-payload** contract as bounded lens projection (`sum_variant_payload` ↔ `variant_payload_for_binding` in `lens_apply.rs`). **That is not yet a closed mechanical theorem** over the whole nested `FieldValue` tree: conformance is enforced where sums are built, and integration tests ratchet **record field order vs. named substrate `Conj`** for selected carriers (`ValueNode`, `TransformNode`, `BindNode`). Extend those tests (or add a walker) as new reflected shapes land—do not treat “no lossy mirror” as automatic proof against every `TypeConnective` row.

**Confidence tag for these gates:** `LensOutputEquals` / `AlgebraicLaw` runners that consume `reflect_program_dag_nodes_in_file` should treat the spine as **intended substrate alignment** (bounded by what the lens algebra proves), not as a substitute for a future full DAG-vs-`FieldValue` conformance gate.

**Witness note:** `Witness<Carrier>` (see `src/v3/std/dimensions.dag`) is not yet exercised by this reflection entry point because it does not appear on the five `Behavior` variants; when lens bodies need witness-shaped facts through the same `FieldValue` carrier, extend the `substrate_reflection` submodule in `lens_apply.rs` with the same substrate-shape rule rather than ad hoc mirrors.

---

## P3: Fail-Closed

**Rule:** Every path either succeeds fully or fails with a typed diagnostic; no fabricated plausible output.

**Why it stands alone:** Fail-closed is about *detection* behavior — when a consumer encounters missing or malformed input, what happens. If the answer is ever "substitute a plausible default," the original failure becomes invisible and diagnostics cascade from the wrong stage. This principle covers C-8 and all nine of its canonical sentinel instances (C-1..C-7, C-9, C-10), which differ only in *which* fabrication was prohibited.

### Problem shape: Fabricated fallback

A consumer encounters missing data and invents a replacement — a null sentinel for a missing argument, an `<error:unknown>` type for an unresolved reference, a silent clone to sidestep an ownership gap, a `"Unknown"` string for a missing name. Downstream code then sees plausible output and proceeds. The original missing fact is invisible; diagnostics cascade from the wrong stage.

**Solution shape:** Treat the detection point as a diagnostic boundary. Produce a typed failure carrier naming what was missing; propagate it forward so every consumer sees "this wasn't provided" instead of a substitute. If a consumer *needs* a value, the upstream contract is too weak — strengthen it, don't paper over.

**Receipt:** Parser error recovery once fabricated dummy literal nodes (C-3's LitNull pattern) to keep parsing moving; inference then treated those as real literals and produced bogus inferred types. Dissolution: typed parse-error carrier surfaced to consumers; inference refuses to proceed past it. The original parse failure became the only diagnostic instead of the tenth.

### Problem shape: Case enumeration for open sets

Behavior is driven by a string-keyed case list — operator symbols, target-language names, encoding formats — with a default branch. Adding a new case means editing the list; missing-case behavior is usually silent fall-through into a generic default. The set grows unboundedly and no reviewer can prove the list is exhaustive.

**Solution shape:** Drive the behavior from a typed data table whose structural properties make exhaustiveness provable. Each row is authored once; consumers read the row. Missing data is a typed diagnostic, not silent fall-through.

**Receipt:** Binary operator dispatch once matched against string symbols with a default branch that silently dropped unknown operators. Dissolution: data table in `parse_tables.dag` with a regen ratchet proving exhaustiveness; fall-through became a typed diagnostic.

### Related rules (home-of-record here)

- **No Fallbacks That Fabricate** — canonical statement
- **C-8: Fail-Closed Compilation** — missing support rejects rather than fabricates (canonical rule)
- **Nine C-series sentinel patterns**, each C-8 applied at a specific fabrication point: **C-1** missing args; **C-2** missing defaults; **C-3** parser recovery dummies; **C-4** placeholder `<error:*>` types; **C-5** string-sentinel error probing; **C-6** `<error:unknown_*>` in emit; **C-7** `Dynamic` as universal fallback; **C-9** empty-node / empty-string fabrication; **C-10** ownership clone as progress fallback
- **Early Detection Invariant** — structural errors fail at the earliest stage that can prove them
- **No Case Enumeration For Open Sets** — case-list fall-through is a fabrication pattern
- **E-8: Unsupported Core Behaviors Fail Closed, Never Collapse Semantically** — missing target support rejects or surfaces unsupported behavior (the "fail closed" choice is fail-closed teeth, not boundary discipline)
- **DB-1: Corrections Are Typed Diagnostic Carriers, Not Ad Hoc Warning Text** — primary home here because the rule's teeth are the anti-warning stance. Cross-reference: the typed-carrier *shape* is Boundary Discipline; the *motivating force* is refusing the fallback-to-warning-text escape hatch

---

## P4: Decidability

**Rule:** Every accepted program stays within a closed, fail-closed system whose correctness questions are structurally decidable.

**Why it stands alone:** Decidability is the semantic commitment that makes the language work. Its foundational premise is **bounded forward execution**: time flows forward, execution walks a bounded structure, and cycles are expressible only as relations over acyclic values — never as direct cyclic values. Bounded iteration, explicit lowering, and closed composition all follow from this premise. Recursion is sugar over a bounded substrate primitive, not a new capability. The previous `Verifiability Invariant` section lived here in substance — verification is what falls out when a closed, faithful system's structural properties become provable by construction; it's not a parallel authority.

> **Note on naming.** The former subdoc `docs/invariants/strict-forward-progress.md` (deleted in the visibility flip; git-recoverable) described bounded forward execution, which is where the content belongs (here, P4). The top-level rule name **"Strict Forward Progress"** as used in reviewer discourse has historically been tied to the dissolution-progress concern — see P5 for that home. The rule-name-vs-subdoc-content drift is pre-existing and not resolved by this rewrite; the subdoc likely wants a rename or split in a future cleanup. Reviewers continue to use "Strict Forward Progress" to mean the dissolution-progress rule until that cleanup lands.

### Problem shape: Unbounded semantic

A surface form is accepted without a proof that it terminates or that its cost is bounded — arbitrary recursion, open-ended self-reference, uncapped iteration. The analyzer either defers termination questions or adds a heuristic timeout. Correctness properties can't be proven structurally.

**Solution shape:** Either lower the surface form to a substrate primitive with an explicit bound (recursion → `Loop` with max depth, iteration → bounded fold), or reject it at the boundary. The substrate admits only forms whose upper bound is explicit.

**Receipt:** Mutual recursion once required an ad hoc termination check because the substrate had no structural representation for it. Dissolution (DB-9): mutual recursion lowers through declared cluster and descent facts; termination becomes a structural property of the lowering, not a separately-proved thing.

### Problem shape: Verification as separate pass

A verification predicate reads its own parallel copy of the facts — a second table, a traversal that re-derives structure, a rule set maintained alongside but not inside the substrate. Verification becomes its own authority; the substrate and the verifier drift.

**Solution shape:** Verification predicates are substrate consumers. They read the same declared facts every other consumer reads. Adding a verification rule means reading an existing substrate edge, not authoring a parallel structure.

**Receipt:** An emission check once walked its own private rendering of the substrate to verify clean-emission properties. Dissolution: verification became a consumer of the declared substrate queries; the private rendering deleted.

### Related rules (home-of-record here)

- **Bounded forward execution** (the foundational premise; the former subdoc is deleted — this bullet is the authority): time flows strictly forward; every execution step moves the computation forward through a bounded structure, never revisiting, never cycling. Decidability, complexity analysis, and termination all follow from this. Cyclic *relations* are expressible (via acyclic encodings — adjacency maps keyed by stable IDs), but direct cyclic *values* are not, and every traversal over a cyclic relation must be bounded by an explicit finite measure.
- **Decidability Invariant** — the canonical statement at the language level (follows from bounded forward execution)
- **Structural Proof From Primitives** — decidability grounds in the primitive algebra
- **Recursive Syntax Is Sugar** — recursive surface forms lower to the bounded substrate without adding semantic power
- **Tight Upper Bounds — No Exceptions**
- **Cost Algebra Is Upstream Of Language Primitives** — cost semantics are part of the modeling substrate, not a post hoc layer
- **Practical Ergonomics** — sugar over the same closed semantic core
- **Closure Property** — composition preserves proof obligations
- **Lowering Table** — the concrete receipt that every surface form maps to the decidable substrate
- **Verifiability Invariant** (entire heading, folded in) + **Structural Proof From Type System** + **What This Replaces** + **Relationship To Decidability**
- **DB-9: Mutual Recursion Lowers Structurally Through Declared Cluster And Descent Facts**
- **DB-8: Deterministic Emission** — same semantic input → same target output (a decidability property of the emission relation)
- **Testing Invariants** — tests prove structural claims; protect single-authority behavior
- **T11: Tiered Test Execution** + **Tier 1 (DryRun)** + **Tier 2 (Selective Real)** + **Tier 3 (Full Real)** — test execution is tiered by structural confidence

---

## P5: Progress Is Dissolution

**Rule:** A change counts as progress only if it reduces ad-hoc state, duplicate authority, or implicit behavior. Scaffolds and intermediate representations need explicit dissolution paths.

**Why it stands alone:** Progress is about *steady-state shape* over time — what kinds of intermediate forms are legitimate. This is the long-run framing of cost-of-change: the governing metric is that when one concept changes, the number of files that must change approaches 1. The previous `Sustainability Invariants` heading lived here in substance — sustainability and dissolution are the same principle viewed at different timescales. This principle refuses bridges, deprecations, and migrations-as-steady-state as legitimate permanent shapes.

### Problem shape: Bridge as steady state

Two representations exist during a migration, connected by a bridge/adapter/shim. The bridge was meant to be temporary, but the migration stalled because each consumer has to move individually, and the bridge became the normal case. Any change now has to maintain both representations.

**Solution shape:** Land representation changes atomically — the new representation exists, the old one deletes, every consumer migrates in the same change. If that's too big for one change, the problem is usually that the new representation isn't actually ready; don't ship the bridge in the meantime.

**Receipt:** A parallel representation for operator metadata once connected through an adapter while consumers migrated one by one. Dissolution: the authority was declared on the new shape, all consumers cut over, adapter deleted — all in a single change.

### Problem shape: Scaffold without dissolution trigger

A scaffold (hand-maintained generated file, interim API surface, staged declaration) is introduced to unblock a lane. No explicit condition exists for when the scaffold should dissolve, so it persists indefinitely and becomes accidentally load-bearing. Future consumers wire into it because it's what exists.

**Solution shape:** Every scaffold lands with a named dissolution trigger — the specific, checkable condition that closes it (e.g., "when the substrate accessor lands, this inline walker dissolves"). A scaffold without a trigger is tracked debt in the wrong category; treat it as the failure-to-dissolve it actually is.

**Receipt:** Staged lens files once existed as hand-authored `.dag` with generated Rust that no consumer read. Dissolution trigger named in the PR: "when parse/parse_surface types converge, `expr_span` consumers wire in." When the trigger was reached, the staged files became real receipts; otherwise they would have been rollback candidates.

### Problem shape: Code without a consumer (E-10)

A model, function, type, or field is written ahead of anything that uses it — justified by top-down design ("define the structure first"). Nothing depends on its behavior, so nothing breaks when the behavior is wrong. Typecheck and grep/`.contains()` assertions pass without the code ever executing, so an entire subsystem accumulates as a **specification that does not run** — plausible, type-checking, and untrue.

**A consumer is anything that breaks when the behavior is wrong.** Strength, weakest to strongest: (1) a human reading the *executed* output (forces execution + that output exists) — the floor, to be *upgraded*, not rested on; (2) an executed assertion (`run(emit(add)) == expected`) — falsifiable; (3) other code that depends on the behavior and breaks if it's wrong — correctness + regression. Typecheck and grep are **not** consumers.

**Solution shape:** Name a real consumer *before* writing a model/function. If there is none, the work is experimental — **archived** (quarantined under `src/v4/experimental/`, git-recoverable), not kept in the active tree, until a consumer exists. Top-down design stays in the *map* (docs, or explicitly-experimental archived `.dag`); code is promoted to *territory* only when a consumer pulls it. Reasoning about / porting consumer-less code is the trap — archive it (no reasoning required) rather than auditing it.

**Review gate:** every review asks "who is the consumer?" of new models / functions / fields. **No real consumer → the code does not enter the active tree: it is routed to `src/v4/experimental/` (it may still land there as experimental); forcing it *into the active tree* anyway requires explicit operator escalation.** The hard-block is on active-tree entry, not on the work existing — i.e. *route the code, block the claim* (operationalized as the reviewer's three questions below). This generalizes [E-6](#e-6) ("no target-spec field without a same-PR consumer") and [DB-4](#db-4) ("clean-emission is a contract with real consumers") from the emission boundary to the whole tree, and enforces the THESIS modeling-discipline rule "every declared type has at least one structural consumer."

**Adoption stance:** applies to new code and refactors (same live-state stance as `CODING.md`). Existing consumer-less code is archive-debt — sweep to `src/v4/experimental/` on touch — not an instant repo-wide failure.

**Receipt (resolved — the dissolution worked):** the v4 compiler grew to a ~4,900-line `06_translate` whose only "consumers" were typecheck + text-grep — and it could not emit `fn add` (step-zero: >600s, no output). The dissolution path was applied and is now **green by execution**: emit was rebuilt as `serialize ∘ translate` behind a real executed witness (`run(emit(add)) == "fn add(x: i32, y: i32) -> i32 { x + y }"`, ~20s), and `06_translate` collapsed toward folds (~4,900 → ~3,973 lines). The lesson stands: typecheck + grep masked a non-running subsystem until an executed consumer exposed — and then fixed — it.

#### Reviewer's three questions (operationalizes [E-10](#e-10))

[E-10](#e-10) states the principle and the consumer test; this is the cheap, every-time check that enforces it. The proof burden is on the **author** — the evidence lives *in the PR*. The reviewer does not investigate: a missing answer is a "no," and a "no" defaults to deny or route-to-experimental, never the benefit of the doubt. (A *stated* gate that needs judgment or effort gets skipped under throughput pressure — so each question is checkable from the PR alone.)

1. **Consumer?** — is there something that breaks if this behavior is wrong — *not* a typecheck, *not* a grep/`.contains()`? (This is E-10's consumer test.) **No → route the code to `src/v4/experimental/`.** It can land as experimental; it does not enter the active tree and is not "done." This is E-10's block-*from-the-active-tree*, not a rejection of the work.
2. **Green by execution, shown?** — does the PR contain the consumer *run* — command + output — rather than "typechecks," "merged," or "looks right"? **No → the done / merge-to-main claim is denied.** Merged ≠ runs.
3. **Green for the right reason?** — does the PR show the case that goes **red when the behavior is wrong** (the discriminating input)? A green that only ever exercised the first arm proves nothing — see the match-on-symbol silent fail-open (#4434). **No → the done / merge-to-main claim is denied.**

**Split:** Q1 routes the *code* (archive-by-default — ungameable, and not a rejection of effort); Q2–Q3 hard-block the *claim* (no executed green + no red-when-wrong case = not done). So "block on no consumer" sharpens to **route the code, block the claim.** This is the operational layer under E-10, not a new numbered authority; materiality (what to rework) is a separate axis and is deliberately out of scope here.

### Related rules (home-of-record here)

- **E-10: No Code Without A Consumer** — no model/function/type/field enters the active tree without something that breaks when its behavior is wrong. *Route the code, block the claim:* no-consumer code is routed to `src/v4/experimental/` (the hard-block is on active-tree entry, not on the work; escalation to override), and a "done" / merge-to-main claim hard-blocks on executed-green-shown + a red-when-wrong case (the reviewer's three questions). Generalizes E-6 / DB-4; enforces the THESIS "every declared type has a structural consumer."
- **Strict Forward Progress** — canonical statement as used in reviewer discourse: a change counts as progress only if it reduces ad-hoc state, duplicate authority, or implicit behavior. Transitional scaffolds need explicit dissolution paths and cannot become the new steady state.
  - *Note:* the former subdoc `docs/invariants/strict-forward-progress.md` (deleted in the visibility flip) described bounded forward execution (the P4 concept), not the dissolution-progress rule the reviewer-facing name has historically carried. Reviewers continue to use "Strict Forward Progress" for the dissolution-progress rule (this bullet); the naming drift dissolved with the subdoc's deletion.
- **Sustainability Invariants** (entire heading, folded in) — cost of change should approach 1
- **No Short-Term Solutions** — this is not a production codebase; no bridges, staged migrations, or compatibility shims justified
- **No Bridges** — bridges normalize half-migrations and hide cleanup cost
- **No Deprecations** — deprecation markers are a production-code tool, not a legitimate steady state
- **Scaffold Boundaries** — scaffolds require explicit dissolution triggers
- **E-I numeric input boundary (cost lane)** — raw `Int` coefficients may appear at recurrence / cost **input** surfaces, but before recursive multiplication, Peano materialization, `int_pow_bounded`, logarithmic iteration, or similar bounded-forward helpers consume them, they must pass the shared M9 ceiling gate in `std.induction` (`bounded_int_pow_exponent`, aligned with `std.termination`’s 256 literal-bridge caps). Skipping that gate duplicates authority and can force depth proportional to a raw exponent before fail-closed rejection (PR #726 meta-review).
- **Escape Hatches** — recurring violations come from API surfaces that make the wrong thing easier than the right thing
- **Scaffold receipts** — every new interim scaffold lands with a named dissolution trigger and a checkable completion condition in the same change.

- **Dispatch-Discipline Mechanisms (a–c)** and **SG-0 receipt tables** — operational P5 enforcement for hand-Rust under `src/v3/` (mechanism **(b)** is what `.github/PULL_REQUEST_TEMPLATE.md` cites for the per-PR dissolution gate).
- **E-5: Clean-Emission Contract Is Satisfied By Construction** — clean-emission obligations belong in declared contracts, not hand-maintained target-side conventions (replacing-convention-by-construction is a progress move, not a boundary clarification)
- **E-7: No Target-Private Realization Schema Without A Dissolution Ratchet** — explicitly names the dissolution discipline in the rule title
- **DB-4: Clean-Emission Behavior Is A Declared Contract With Real Consumers** — same by-construction framing as E-5
- **Engineering Standards** — long-form receipts

## Appendix: ID index

Every numbered ID (C-N, E-N, L-N, DB-N) descends from one principle. The prose-name invariants are indexed under their principle above; this appendix exists so PR-history references like "violates C-8" or "see E-9" resolve quickly.

| ID | Home principle | Short form |
|---|---|---|
| <a id="c-1"></a>C-1 | P3: Fail-Closed | missing args fail closed; no `LitNull` sentinels |
| <a id="c-2"></a>C-2 | P3: Fail-Closed | missing defaults / config fail closed; no `LitNull` sentinels |
| <a id="c-3"></a>C-3 | P3: Fail-Closed | parser recovery may not fabricate dummy `LitNull` nodes |
| <a id="c-4"></a>C-4 | P3: Fail-Closed | placeholder `<error:*>` types forbidden as live carriers |
| <a id="c-5"></a>C-5 | P3: Fail-Closed | error detection may not rely on string-sentinel probing |
| <a id="c-6"></a>C-6 | P3: Fail-Closed | emit may not use `<error:unknown_*>` sentinels |
| <a id="c-7"></a>C-7 | P3: Fail-Closed | `Dynamic` is not a universal compatibility fallback |
| <a id="c-8"></a>C-8 | P3: Fail-Closed | fail-closed compilation (canonical) |
| <a id="c-9"></a>C-9 | P3: Fail-Closed | missing fields / values may not fabricate empty nodes or strings |
| <a id="c-10"></a>C-10 | P3: Fail-Closed | ownership gaps may not silently clone for progress |
| <a id="db-1"></a>DB-1 | P3: Fail-Closed (cross-ref P2) | typed diagnostic carriers, not ad hoc warning text |
| <a id="db-4"></a>DB-4 | P5: Progress Is Dissolution | clean-emission as declared contract with real consumers |
| <a id="db-5"></a>DB-5 | P2: Boundary Discipline | substrate keyed lookup single-authority |
| <a id="db-8"></a>DB-8 | P4: Decidability | deterministic emission |
| <a id="db-9"></a>DB-9 | P4: Decidability | mutual recursion lowers structurally |
| <a id="db-14"></a>DB-14 | P2: Boundary Discipline | external primitives materialize through `Arrow.body` |
| <a id="e-5"></a>E-5 | P5: Progress Is Dissolution | clean-emission contract by construction |
| <a id="e-6"></a>E-6 | P2: Boundary Discipline | no target-spec field without a same-PR consumer |
| <a id="e-7"></a>E-7 | P5: Progress Is Dissolution | no target-private realization schema without a dissolution ratchet |
| <a id="e-8"></a>E-8 | P3: Fail-Closed | unsupported core behaviors fail closed, never collapse semantically |
| <a id="e-9"></a>E-9 | P2: Boundary Discipline | external realization lives on `Arrow.body` |
| <a id="e-10"></a>E-10 | P5: Progress Is Dissolution (generalizes E-6, DB-4) | no model/function/field without a real consumer; route the code (no-consumer → archive-by-default `src/v4/experimental/`), block the claim (done → executed-green + red-when-wrong) |
| <a id="l-7"></a>L-7 | P2: Boundary Discipline | lenses consume declared substrate query functions |
| <a id="l-8"></a>L-8 | P2: Boundary Discipline | lens Rust surfaces preserve typed failure carriers |
| <a id="t-11"></a>T11 | P4: Decidability | tiered test execution (Tier 1/2/3 sub-rules) |

---

## Pointers

- [`docs/thesis/`](docs/thesis/) — thesis essays, including `epistemic-stacking.md` and `structural-decompression.md` referenced from P1
