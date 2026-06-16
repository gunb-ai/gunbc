# gunbc Thesis

This is the parent document. Everything else — ROADMAP, INVARIANTS, MODELING, architecture, and design docs — serves this thesis.

> **v2 is the active development phase.** The live compiler and substrate live in [`src/v2/`](src/v2/). Every thesis claim below applies to v2; earlier v3 references in the self-hosting section describe the v2→v3 transition that v2 supersedes. v3 is frozen as a comparison baseline.

## How this doc is organized

Read this file for the thesis itself and the complete claims list. Extended argumentation now lives under `docs/thesis/`, while `ROADMAP.md`, `INVARIANTS.md`, and `MODELING.md` stay as the live operational companions.

## What gunbc is

gunbc is a causal engine: it validates that a program’s declared causes, dependencies, and drains are structurally coherent before emission becomes a mechanical translation.

If it compiles, the declared intent is sound inside the modeled system; what remains unverifiable is only external reality not carried in the program graph.

## The core abstraction: dependency modeling

A `.dag` program is a dependency graph. Parallelism is the default because independence is visible in the structure; sequential execution is what needs justification.

The compiler reads the dependency graph directly and can choose the target schedule mechanically from the same source.

## The core flip: put the meaning in the types, derive the coercion

Mainstream types are *lean*. `int`, `str`, `char` carry only what the machine needs to move the bits — almost no meaning. That is exactly why coercion between them is unsafe: there is nothing to check a conversion *against*, so mixing a `UserId` and an `AccountId` (both `int`) is invisible. Developers pay for this twice — they hand-write the translations between types, and they re-declare the same concept under thirty names across API, DB, wire, and frontend (the "nicknaming tax").

gunbc flips the model. **Put the meaning into the types, and a safe coercion between any two becomes *derivable*** — because a structure-preserving map between two meaning-rich types is determined by their meanings. The compiler finds the witness that relates them. The thirty nicknames for one concept become thirty names for overlapping structures the compiler can see through, so the N² hand-written translations collapse: you declare the genuine semantic decisions once, and the compiler derives everything downstream. This is the one idea under the rest of the thesis — **emit is coercion *to* a target, ingest is coercion *from* a source, the glue developers write is coercion the compiler should derive. One engine** ([the derived homomorphism](#the-derived-homomorphism--model-local-derive-global)).

Three things keep this from being the old unsafe implicit coercion with better branding:

- **The safety is in what it *refuses*, not in automating everything.** The compiler derives every coercion the meaning *determines*. But some coercions lose information (rich → coarse) and some must add it (the target lacks a carrier, so it must be *realized*). Those can never be silent. The flip's safety is the **fail-closed boundary**: derive what's determined; fail closed — or realize with an explicit receipt — on loss, ambiguity, or a missing carrier. *That boundary is the product.* "All automatic" minus the boundary is the problem we're escaping.
- **It only works if the types are actually distinct.** "The compiler sees what each type needs" requires `UserId ≠ AccountId` to be *enforced*, not just named. That distinctness is the **keystone**: no enforcement, no derivable coercion — meaning-in-the-types is cosmetic without it.
- **Developers still make the genuine decisions.** The derivable part — most of it, the pure nicknaming — derives fully. Where two systems truly *disagree* on semantics (not just naming), the compiler detects the mismatch but cannot invent the resolution; that stays a human decision it **surfaces, not guesses**. The precise claim: declare the genuine semantic decisions once; the compiler derives everything downstream.

**Positioning.** Rich types that *derive* coercion have relatives — dependent and refinement types, prover-backed coercions, type-driven deriving. gunbc's distinctive bet is not "rich types"; it is that the language is **closed and total**, so coercion is a **decision procedure that always terminates with a verdict** — not a heuristic, not an open-ended proof search. The one-liner: **decidable, not heuristic; derive behavior, not just shape.**

**The litmus.** Does a design move meaning *into* the types and *derive* the coercion — or does it re-introduce manual translation and coarse types? That single question catches most drift.

## Why this works

`.dag` is designed as a closed system: bounded data, bounded iteration, and composition that preserves those bounds.

The extended derivations under this section moved into `docs/thesis/` so this top-level doc stays small enough to read as an entry point.

### Concept unification

Distinct-looking mechanisms often collapse into the same structural mechanism once the system is closed and modeled honestly.

See [docs/thesis/concept-unification.md](docs/thesis/concept-unification.md).

### Structural decompression

Coproducts and other categorical tags should be treated as compressed structure whenever the underlying coordinate facts are available.

See [docs/thesis/structural-decompression.md](docs/thesis/structural-decompression.md).

### Why decompression always works in a closed system

The decompression argument depends on the compiler owning both the producer and consumer sides of the modeled exchange.

See [docs/thesis/structural-decompression.md](docs/thesis/structural-decompression.md#why-decompression-always-works-in-a-closed-system).

### Epistemic stacking: every concept grounds in primitives

Concepts should attach into an explicit ontology rooted in minimal primitives rather than floating as opaque names.

See [docs/thesis/epistemic-stacking.md](docs/thesis/epistemic-stacking.md).

### The substrate: two coordinated shapes

The thesis depends on coordinated type and behavior substrates rather than a flattened single bag of nodes.

See [docs/thesis/the-substrate-two-coordinated-shapes.md](docs/thesis/the-substrate-two-coordinated-shapes.md).

### Compositional layering: below-boundary opacity by construction

Lower layers should provide declared facts without leaking storage choices or forcing downstream reinterpretation.

See [docs/thesis/compositional-layering.md](docs/thesis/compositional-layering.md). Current-state audit — how close v2 is to this principle, concept by concept (the "touch-once contract"): [docs/audit/v2-encapsulation-touch-once-contract-2026-06-05.md](docs/audit/v2-encapsulation-touch-once-contract-2026-06-05.md).

### Self-inspection: the substrate is its own subject

The same substrate that models user programs should be able to model and analyze the compiler’s own structures.

This is the read axis of **programmatic access to the code** (paired with the write axis, "show the correct code", below). The roof over both axes — and the measured current-state — is assessed in ctrl planning doc `gunbc-planning/programmatic-access-single-roof-2026-06-07.md` (ctrl#1481): the substrate's *types-as-data* half is real, but the runtime *reflection-by-execution* half is **measured unbuilt** (ctrl#1480 Q2) — a build, not a settled fact.

See [docs/thesis/self-inspection.md](docs/thesis/self-inspection.md).

### Compiler–`std/` consolidation: no dual representations at the compiler/user boundary

The mature compiler uses only `std/` concepts plus a minimal set of pure `.dag` compiler APIs. A "token" in the compiler is the same `Token` a user's formatter uses; a "file" is the same `File`. Compiler-specific taxonomies that duplicate user-facing concepts are dual representations at the architectural layer, dissolved by consolidation.

See [docs/thesis/compiler-std-consolidation.md](docs/thesis/compiler-std-consolidation.md).

### Two groundings: static validation vs efficient realization

The thesis separates semantic grounding from efficient target realization without duplicating authority.

See [docs/thesis/two-groundings-static-validation-vs-efficient-realization.md](docs/thesis/two-groundings-static-validation-vs-efficient-realization.md).

### Target realization efficiency

Target growth should cost one spec file, not a new compiler path per target.

See [docs/thesis/target-realization-efficiency.md](docs/thesis/target-realization-efficiency.md).

### The derived homomorphism — model local, derive global

Every target — language, format, service, persistence layer — is modeled once in the shared substrate. Translation between any two targets is then a homomorphism the compiler *derives* by comparing groundings, never an adapter anyone authors: you write N target-models, the compiler derives the N×M translations. Integration becomes local — you model only your own target — and every translation that cannot be made faithful surfaces as an explicit, located diagnostic rather than a silent bug. The thesis rests on one honest bet: that target-modeling, done in shared vocabulary, is correct, bounded, and checkable — which is what the modeling discipline exists to secure. This is the purpose the rest of the design serves; a reviewer should read every modeling rule as protecting this homomorphism.

See [docs/thesis/the-derived-homomorphism.md](docs/thesis/the-derived-homomorphism.md).

## Correctness dimensions

Correctness dimensions are the thesis mechanism for adding new proof obligations without inventing parallel infrastructures. A dimension — complexity, cost, idempotency, ownership, parallelism, or any user-declared invariant — is a structural fact carried by the program's data model, not a behavioral check run at test time. Validation is reading the structure; it is not running the code.

Consequence: correctness scales with the structural surface, not with human attention. Mainstream languages catch invariant violations via tests, profilers, schema validators, and production postmortems. gunbc catches them by structural derivation — compile-time proofs for Tier 1/2 dimensions, and a structurally-derived test surface for Tier 3 (where emitted code runs but the test surface is TestClaim data, not hand-authored behavior assertions).

See [docs/thesis/correctness-dimensions.md](docs/thesis/correctness-dimensions.md).

### User-defined dimensions

User-declared dimensions extend the same structural proof surface rather than opening a second rule system. A user writes a lens in `.dag` — e.g., "max external HTTP calls per workflow," "bounded memory footprint per request," "no cross-tenant data flow" — and the compiler validates every program against it using the same mechanism it uses for built-in dimensions.

Consequence: the ceiling of what gunbc can prove is user-extensible. Domain-specific correctness concerns that mainstream languages cannot model become structural facts in gunbc.

See [docs/thesis/correctness-dimensions.md](docs/thesis/correctness-dimensions.md#user-defined-dimensions).

## Error handling: show the correct code

Diagnostics should point to the structurally correct program, not just report that the current one is wrong.

## What falls out

Tier-1 structural correctness, Tier-2 runtime safety, and Tier-3 verification are consequences of the modeled structure once the thesis closes.

See [docs/thesis/what-falls-out.md](docs/thesis/what-falls-out.md).

## What else falls out

Automatic parallelism, memoization, omni-emission, algebraic simplification, and related benefits are consequences of the same structural commitments.

See [docs/thesis/what-else-falls-out.md](docs/thesis/what-else-falls-out.md).

## What .dag catches that normal compilers don't

The thesis claims a compiler can reject structural, effect, and complexity bugs that ordinary compilers never model.

See [docs/thesis/what-dag-catches-that-normal-compilers-dont.md](docs/thesis/what-dag-catches-that-normal-compilers-dont.md).

## How the docs connect

The doc map and the single-authority rule (one fact, one home; every other mention links to it) live in one place: see [docs/thesis/doc-authority.md](docs/thesis/doc-authority.md).

## Thesis claims — complete list

Every claim the thesis makes, in one place. The ROADMAP tracks progress toward each. If a claim isn't here, the project doesn't claim it. If a claim IS here but the ROADMAP has no track for it, that's a gap.

**Core abstraction:**
- .dag is dependency modeling software. The program IS a dependency graph. Parallelism is the default; sequential execution requires a data dependency to justify it.

**Correctness is structural, not behavioral (meta-claim):**
- Every correctness dimension — type, arity, unit, effect, complexity, ownership, idempotency, and any user-declared invariant — is a structural fact carried by the program's data model.
- The proof and test surface is structurally derived, not hand-maintained. Tier 1 and Tier 2 proofs close at compile time by reading the structure. Tier 3 runs emitted code, but the test surface is generated from structural `TestClaim` declarations in `.dag` — not hand-authored behavior assertions. Under the 0-floor target (per [`docs/design-pure-bootstrap-zero.md`](docs/design-pure-bootstrap-zero.md), LIVE 2026-04-25 — supersedes the prior ≤5-floor framing in `docs/design-pure-bootstrap.md`), the prior `TESTING.md §"Post-R2 shape"` residual carve-out (compiler-internal unit tests + external-toolchain boundary tests) is **retracted**: helper unit tests vanish naturally as their hand-Rust subjects dissolve, and boundary tests migrate to `ExecuteCommand`-based `.dag` `TestClaim` declarations per the cascade promotion. TESTING.md is the single authority on the migration path.
- The dimension system is first-class user-extensible: a user writes a lens in `.dag`, and the compiler validates every program against it using the same mechanism it uses for built-in dimensions.
- What mainstream languages catch via testing, profiling, schema validators, integration test suites, and production postmortems, gunbc catches by structurally deriving the proof or test — compile-time proofs for Tier 1/2, structurally-derived test surface for Tier 3.

**Tier 1 — Structural correctness (impossible to write the bug):**
- Type mismatches, field typos, non-exhaustive matches, bare container types, circular dependencies, stale imports, cross-target drift — all caught at compile time.
- CX gate: every recursive function terminates with a proven bound.
- Coercion = emission: the compiler reads a target spec and translates. No separate coercion engine.
- Ownership: the compiler proves no aliased mutation in emitted code.
- **Grounding completeness**: target-side primitive types are structurally modeled from the target language reference (Rust Reference §Types, Python data model, Go specification), with algebra inhabitance declared structurally — not string-typed shortcuts in a lookup table. Mapping from a `.dag` type to a target primitive is a structural algebra-homomorphism search over declared inhabitance, not a name-keyed table lookup. If a `.dag` type cannot be structurally grounded to a target primitive, the compiler refuses to emit (fail-closed). See `docs/single-emitter-design.md` for architecture; the target-grounding proposal ([PR #695](https://github.com/gunb-ai/gunbc/pull/695), landing at `docs/thesis/target-grounding-proposal.md` on merge) for the concrete work breakdown; ROADMAP for the lane structure.

**Tier 2 — Runtime safety (proven safe or total):**
- Division by zero, integer overflow, out-of-bounds, force-unwrap, partial functions — either proven safe at compile time or made total. No partial functions in the runtime.

**Tier 3 — Verification from structure:**
- L4: emitted code executes and matches .dag evaluation.
- L5: same .dag produces same behavior in Rust/Python/Go.
- L6: every structural form compiles to every target.
- L7: operations obey declared algebraic laws.

**Concept unifications:**
- Coercion cost = complexity.
- Coercion = emission.
- Target language spec = transport spec = interpreter runtime.
- Idempotency + cancellation + redundancy = algebraic simplification.

**Epistemic stacking (load-bearing for codegen — must not be dropped):**
- Every concept is a node in an ontological DAG rooted at a minimal set of primitives. No concept is opaque.
- Root primitives: Magma (closure), Monoid, BooleanAlgebra (classical logic), FreeMonoid<T> (free constructions). Declared in `dsl/std/algebra.dag`.
- Concrete types attach by inhabitance (Int inhabits OrderedRing, String inhabits FreeMonoid<Char>, etc.). Operations fall out; they are never declared separately.
- The epistemic chain IS the emission algorithm. Every emitter special case is evidence of an ungrounded concept upstream.
- Math primitives and domain primitives share one substrate. A user declares `type CIWorkflow<Step>` the same way `Int` is declared, and it projects onto code the same way.
- Substrate test for any candidate Declaration shape: can it host `dsl/std/algebra.dag` as-is? If not, the shape is too narrow.

**Substrate shape (two coordinated substrates — must not be flattened):**
- Types are Node trees with six connectives: `Atom | Conj | Disj | Arrow | Cardinality | Instantiation`. `service`, `fn`, `type`, `operation` are surface sugar over this layer (`MODELING.md` §"Composition layer"). `Instantiation` matches C++ template-instantiation vocabulary and is ONLY used for type parameterization; value construction (`transport shell { argv: [...] }`) uses plain Conj with an optional inhabits tag.
- Computation is five L1 behaviors: `Value | Transform | Branch | Loop | Bind`. Validated by M0 under three reviewer rounds; the stop signal never fired.
- Composition: Transform holds a FunctionRef to an Arrow declaration in the type substrate; the Arrow's body is a sub-DAG of L1 behaviors (for user functions) or a realization declaration in `dsl/extdeps/` (for primitives).
- Substrate extension is a C1-class stop signal (seventh connective or sixth behavior) — all four dissolution patterns from §"Structural decompression" must fail with structural arguments before extension is allowed.
- Future candidate (NOT committed): unified substrate dissolving the five behaviors into patterns over Node. Recorded for future consideration only; revisiting requires new failure pressure, not aesthetic preference.

**Free consequences (fall out when Tiers 1-2 close):**
- Automatic parallelism from dependency graph.
- Automatic memoization from purity + cost.
- Incremental cross-run execution from purity + bounded execution + dependency graph.
- Space bound proofs from CX.
- Cross-language optimization from shared cost algebra.

**Omni-emission (1:1 effort applied to full-stack systems):**
- One workflow declaration projects onto every layer of a real application: DB schema, backend service, API client, frontend form, documentation — from one source.
- Coherence between layers is structural, not checked — drift is impossible because every layer derives from the same Node tree (directly via compiler emission for Shape A targets, or indirectly via Shape B user programs walking typed values).
- **Shape A — compiler language targets**: programming languages (Rust, Python, Go, TypeScript, Swift, HDLs). Compiler emits directly via a language spec in `dsl/extdeps/languages/`. Adding a new Shape A target costs one language spec. `O(1)` per target; zero compiler/emitter changes.
- **Shape B — user-program artifacts**: YAML configs, Terraform HCL, Kubernetes manifests, SPICE netlists, natural-language docs, SQL schemas, JSON Schema, OpenAPI specs. Emitted by `.dag` programs walking typed values via `concat`/`fold`/`match`. Per ROADMAP Track 16: these are not compiler render targets; they are user code generating target strings. Adding a Shape B target is writing one reusable `.dag` emitter program.
- Target-level cost complexity composes with `.dag`-level CX via language-spec realization costs — the compiler can compute per-target complexity bounds statically (see §"Target realization efficiency").
- The two-shape distinction follows ROADMAP Track 16's explicit decision: the compiler emits programming languages; everything else is user code.
- Cost scaling: Shape A is `O(1)` per language target; Shape B is `O(1)` per artifact class. Neither is `O(N × M)`. Effort scales with conceptual content, not with layer or target count.
- Emission is independent of intent. You declare what the system does; separately, you declare what artifacts it becomes.
- See [`docs/thesis/what-else-falls-out.md`](docs/thesis/what-else-falls-out.md) §"Two shapes of omni-emission" for the full Shape A vs Shape B treatment, including the per-target cost structure and the load-bearing reason the distinction must not be blurred.

**Meta-process modeling:**
- Bootstrap and build orchestration modeled as .dag workflows (`src/v2/workflow/bootstrap.dag`). CI is hand-authored directly in `.github/workflows/ci.yml` (the prior `src/v2/workflow/ci.dag` mirror was a descriptive-only model with no runtime consumer and was deleted; ci.yml is the direct authority). The project does not model its own work-direction process as `.dag` data (see facet 4 below).
- `dag run` is the primary execution path.
- Adding a Node field or a target language requires editing one .dag file. (CI gates are the exception: they are hand-authored in `.github/workflows/ci.yml`, the direct CI authority — not modeled as `.dag` data.)

**Self-hosting — four facets:**

Self-hosting is not one capability; it's four. All four are targets.

1. **Compiler written in the language it compiles.** `.dag` source authors
   the compiler. Partially true today — `.dag` authors key compiler passes
   (visible in `dsl/gunbc/` and emitted Rust), while stage0 Rust (see SG-0
   census for the live count) remains as sketch scaffold pending
   dissolution. The direction predates the Pure Bootstrap program; PB is
   the trajectory, not the origin.

2. **Compiler self-emits (fixed-point).** Compiling `compiler.dag` produces
   bit-identical output to what currently ships. The `.dag` graph is the
   source of truth; the emitted Rust tree is one realization of it — not a
   parallel authority requiring manual sync. **Pure Bootstrap's primary
   deliverable.** Strictly stronger than "the compiler can compile itself":
   the compiler's own source of truth is the `.dag` graph.

3. **Tests are data too.** The test suite equivalent of v1's hand-authored
   `pipeline.rs` (`src/v1/tests/src/pipeline.rs` — the large pipeline/
   contract test file; live LOC reads from the file) exists only as
   `.dag` `TestClaim` declarations and generated target-language test code.
   Under the 0-floor target (per `docs/design-pure-bootstrap-zero.md`, LIVE
   2026-04-25), the prior `TESTING.md §"Post-R2 shape"` two-residual carve-out
   is **retracted**: helper unit tests vanish naturally as their hand-Rust
   subjects dissolve; boundary tests that invoke external toolchains (rustc,
   go, python) migrate to `ExecuteCommand`-based `.dag` `TestClaim`
   declarations. Everything ports to `.dag`. **Pure Bootstrap's secondary
   deliverable, couples to testgen.**

4. **Recursive-flex / self-application.** gunbc applies its own correctness/
   cost/parallelism lenses to its own **build pipeline**, which
   is modeled as `.dag` data (`src/v2/workflow/bootstrap.dag`; CI itself is
   hand-authored in `.github/workflows/ci.yml`, not modeled). The same lens framework users get for their own
   programs applies recursively to gunbc's own build/CI behavior —
   typed lenses for cost / complexity / parallelism over the
   pipeline that produces gunbc itself. (Timing is a projection of the
   **cost** lens — cost is the time/complexity dimension, U2 — not a
   separate lens; Theme-A planning audit, 2026-05-17.) This
   distinguishes gunbc from compilers that don't validate their own
   production pipeline.

   **Scope (narrowed):** gunbc does not model its own work-direction
   (briefs, cycles, retirement) as `.dag` data. This facet claims only
   lens self-application to the build/CI pipeline. The six live
   `src/v2/workflow/` files are: `bootstrap.dag` (build orchestration),
   `lens_ci_gate.dag` (CI pass/fail gate; replaced the deleted `ci.dag`
   descriptive-only mirror), `affected_set_selection.dag` (CI
   affected-set authority), `scheduler.dag`, `cli.dag`, and `release.dag`
   (`v2.workflow.release_dist` — GH Releases binary matrix + hand-synced
   `release.yml` until YamlStatic emission). The **facet-4 lens
   self-application scope** is the CI-pipeline files: `{ bootstrap,
   lens_ci_gate, affected_set_selection }`. `scheduler`, `cli`, and
   `release` are in-tree workflow files but do not expand facet-4 scope.

Cost-of-change: editing any compiler concept — a new pass, substrate fact,
target-language detail, or pipeline/contract test assertion — stays at
one `.dag` file. No
matching hand edits to a Rust stage0 file. Stage0 Rust compiler internals
(tokenize, parse, lower, infer, emit, lenses, std library) are emitted
from the `.dag` graph and committed — not hand authored. Tests follow
the cascade-promoted shape in facet 3 above: all pipeline/contract tests
are `.dag` `TestClaim` data; the prior `TESTING.md §"Post-R2 shape"`
two-residual carve-out is **retracted** under 0-floor (compiler-internal
unit tests + external-toolchain boundary tests both dissolve — helpers
vanish with their subjects, boundary tests migrate to `ExecuteCommand`-based
`.dag` `TestClaim` declarations). Hand-maintained surface target: **0**
per [`docs/design-pure-bootstrap-zero.md`](docs/design-pure-bootstrap-zero.md)
(LIVE 2026-04-25; supersedes the prior ≤5-floor framing in
`docs/design-pure-bootstrap.md`).
The live *count* of currently hand-authored files is tracked per-generation:
v1 authority (proven): `src/v1/` stage0 census — the production self-host model
at ~97% (2 hand-maintained of 62 stage0 files). (The former v3 generation and
its SG-0 census have been removed.)
v2 authority (active): `src/v2/compiler/self_host.dag` — hand-authored-file
ratchet pending; v2's substrate is already at 0 hand-maintained `.rs` in the
compiler tree (`.dag` source only). Generated escape hatch is acceptable
for additional files; hand-authored files are not.

Fixed-point acceptance: the binary compiles `compiler.dag` and produces
bit-identical stage0 Rust plus bit-identical emitted artifacts.
`compiler.dag`'s `hand_maintained_src` list monotonically shrinks to the
empty set per [`docs/design-pure-bootstrap-zero.md`](docs/design-pure-bootstrap-zero.md).
Active implementation: `src/v2/compiler/self_host.dag`; runner work
continues as substrate stages complete.

**Audience duality — opt-in depth (meta-feature):**
- Core language stays approachable — types, functions, match, effects,
  workflows. Any engineer can write a gunbc program and get multi-target
  emission without learning the lens/proof surface.
- Advanced surface is opt-in — lenses, cementing tests, user-authored static
  reflection, complexity/cost/idempotency proofs. Opening these adds depth
  without changing the base language.
- gunbc does not pick a tribe. Normal programmers get glue generation;
  principal engineers get structural proofs. The same compiler serves both
  because depth is a surface the user opts into.
- Two canonical demo fixtures exercise both audiences (structural-proof
  vs glue-generation), and user-authored lenses extend the proof surface
  without changing the base language.

**Adoption model — economics, not enforcement:**
- The thesis claims every program gets complexity, effects, termination,
  idempotency, and ownership for free — by construction, not by opt-in.
  This is structurally true (see §"Substrate shape" + `docs/thesis/epistemic-stacking.md`):
  the language vocabulary is the six type connectives and five behaviors;
  every program decomposes through them; lenses are folds over that
  decomposition. There is no in-language way to author a program the lenses
  can't read.
- "Leaving the stack" inside the language means composing primitives into
  named patterns (namespacing). The compiler sees through; lenses still
  apply. **Still inside the stack.**
- "Leaving the stack" outside the language means writing a different
  compiler on different primitives. The thesis does not prevent this and
  does not need to — gunbc's lenses are folds over *our* primitives, so
  they don't apply to a different compiler's outputs by construction
  (until those outputs are grounded back into `.dag`). See
  `docs/thesis/epistemic-stacking.md` §"Positive corollary."
- Adoption is therefore gated by **economics, not enforcement**: low cost
  of entry (one composition layer, no annotation surface, surface syntax
  is sugar over six connectives) × high free value (every lens applies to
  every program). The recruiting mechanism is "you get the guarantees by
  using the language at all," not a license check or a static analyzer
  flagging non-compliant code. The user surface stays approachable; the
  guarantees are unavoidable.

**Tests are structural data:**
- All tests are `TestClaim` declarations in `.dag` under the 0-floor cascade
  promotion (the prior `TESTING.md §"Post-R2 shape"` residual carve-out is
  retracted; helper unit tests vanish with their hand-Rust subjects, boundary
  tests migrate to `ExecuteCommand`-based `.dag` `TestClaim` declarations).
  Within the `.dag` surface, hand-authored tests and generated tests share
  one predicate vocabulary — the predicates ARE the test-writing language.
  TESTING.md is the single authority on the migration path.
- Manual tests are upstream of code: behavioral contracts the code must
  satisfy. Testgen is downstream of code: structural coverage derived from
  the program the user wrote.
- Rust-authored tests are a language smell. Every hand-authored `.rs` test
  flags a predicate, effect-model, or mock surface the language doesn't yet
  express.   The operational release gate is zero Rust-authored tests outside the
  pure-bootstrap residual (empty under cascade promotion).
- Consequence of the pure-function posture: effects are explicit parameters,
  mocking is dependency-injection-by-construction, no hidden state means no
  flaky tests.

**Enumerable impossible-bug classes:**
- The thesis obligates naming the bug classes that become impossible by
  construction. Not "bugs in general" — enumerable, teachable classes.
- Initial committed list (release demo readiness):
  - **[release]** Suboptimal-complexity contract violation: a function annotated
    `complexity ≤ O(n log n)` whose actual complexity exceeds it errors at
    compile time, not review time.
  - **[release]** Idempotency-contract violation: a function marked `@idempotent`
    whose structure admits non-idempotent composition errors.
  - **[release]** Transport/type drift: client and server cannot hold different
    types for the same field — both derive from the same declaration.
  - **[post-release]** Nested-optional flatten: `Option<Option<T>>` accessor patterns
    that normal languages require hand-unwrapping. Gated on cardinality
    refinement substrate work.
  - **[post-release]** Unenumerated effects: operations are intrinsically read-shaped
    or write-shaped via their type-signature shape (returned-modified-resource
    indicates write; returns-derived-value-only indicates read). Consumers
    walk the signatures directly; there is no parallel taxonomy or annotation
    layer to declare or maintain. Tracking effects as a separate enumerated
    concept IS the bug pattern, dissolved by construction. Every external
    mutable resource (file handles, sockets, db connections) is modeled as a
    typed parameter that's returned modified — same pattern as IO-monad
    World-threading without the monad. Redundant operations (reads of the
    same key with no intervening write-effect on that resource) are
    structurally provable as identical via referential transparency, and
    rejected at compile time; legitimate re-read uses an explicit `reread()`
    primitive that structurally tags the intent. Transactional grouping is a
    derived structural fact from Bind composition + typed transaction
    primitives (`Transaction → Transaction'`), not a separate concept. Tier 1
    (impossible by construction), not Tier 2 (lens-detected). See
    [`docs/briefs/t-impossiblebugs-unenumerated-effects-design.md`](docs/briefs/t-impossiblebugs-unenumerated-effects-design.md)
    §Q5.5 for the OperationEffect-taxonomy retirement rationale + audit-as-
    existence-check; §Q1-Q3 for the 5-behavior compositional-fold mechanism +
    worked examples.
  - **[post-release]** Unhandled diagnostic paths: Tier 2 runtime-safety proofs make
    division-by-zero, OOB, and force-unwrap either proven safe or made
    total — never partial. Gated on Tier 2 substrate (post-R1).
- Adding a bug class to this list is a thesis commitment; removing one
  requires a named dissolution (the structural proof became trivial).
- Release-scoped classes must demo at first public release; post-release
  classes are thesis-committed but not in the initial demo scope.

**Modeling discipline:**
- Every declared type has at least one structural consumer.
- Every service boundary uses typed enums, not String/Bool proxies.
- No fabrication sentinels (`__BUG_*`, `__EMIT_BUG_*`). Missing facts are compile-time errors, not runtime strings.
- No duplicate record shapes. One type per concept.
- A finished compiler stage is one fold over its model: `stage(x) = fold_carrier(x, algebra(model))` — or a thin zero-residue composition of such folds, glued only by monadic sequencing (`bind_outcome` / `∘`). A pure fold expresses only intent — the decisions live as data in the model (`std/`/`extdeps/`), the fold owns traversal. Non-fold residue is a litmus, not a ban: it is either a named irreducible kernel (a solver, char-matching — *fold the traversal, name the kernel*) or un-migrated modeling — code making a decision the model hasn't absorbed yet. Monadic composition glue is plumbing, not residue. The volume of non-fold control-flow in a stage measures how much decision-making still lives in code instead of the model. `05_emit` (`emit = serialize_target ∘ translate`, 43 lines) is the existence proof of the composition form; MODELING.md M11 is the rule, docs/modeling-discipline.md Practice 12 the review rubric.

## The test: when is it real?

The causal engine is real when a user can declare their intent:

```dag
type Order { customer: String  amount: Float  status: OrderStatus }
type OrderStatus = Pending | Approved | Declined | Refunded

service OrderService {
  fn create_order(req: CreateOrderRequest) -> Order via rest::post("/orders")
  fn get_order(id: String) -> Order via rest::get("/orders/{id}")
}
```

...and the compiler:
1. **Validates** every causal link — types, fields, transports, termination, ownership (Tier 1)
2. **Proves** that no internal operation can fail at runtime (Tier 2)
3. **Generates** tests that verify the declared behavior matches actual behavior (Tier 3)
4. **Emits** to any target language as mechanical translation

The only possible failure is external: the REST endpoint doesn't exist, the network is down, the upstream service violates its contract. Everything inside the causal graph is proven sound.
