# gunbc Thesis

This is the parent document. Everything else — ROADMAP, INVARIANTS, MODELING, architecture, and design docs — serves this thesis.

## How this doc is organized

Read this file for the thesis itself and the complete claims list. Extended argumentation now lives under `docs/thesis/`, while `ROADMAP.md`, `INVARIANTS.md`, and `MODELING.md` stay as the live operational companions.

## What gunbc is

gunbc is a causal engine: it validates that a program’s declared causes, dependencies, and drains are structurally coherent before emission becomes a mechanical translation.

If it compiles, the declared intent is sound inside the modeled system; what remains unverifiable is only external reality not carried in the program graph.

## The core abstraction: dependency modeling

A `.dag` program is a dependency graph. Parallelism is the default because independence is visible in the structure; sequential execution is what needs justification.

The compiler reads the dependency graph directly and can choose the target schedule mechanically from the same source.

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

See [docs/thesis/compositional-layering.md](docs/thesis/compositional-layering.md).

### Self-inspection: the substrate is its own subject

The same substrate that models user programs should be able to model and analyze the compiler’s own structures.

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

```
/ (direction — start here)
  THESIS.md .............. this file — the goal
  ROADMAP.md ............. current state and work plan
  INVARIANTS.md .......... rules that protect the thesis
  MODELING.md ............ how to extend the language safely

docs/ (project-wide design — read for understanding)
  architecture.md ........ substrate design (Node + Edge)
  algebraic-type-spec.md . type system semantics
  coercion-design.md ..... type coercion algebra (Tier 1, DONE)
  error-examples.md ...... concrete .dag code + expected errors (TDD targets)

src/v2/ (compiler implementation — read when working)
  DESIGN.md .............. compiler design principles
  dimensions-design.md ... general correctness dimension mechanism
  cx-design.md ........... complexity (first dimension instance)
  cx-computation-model.md  CX core model and evidence system
  cx-violation-triage.md . CX violation snapshot
  ownership-design.md .... ownership (second dimension instance)
  compiler-laws.md ....... compiler structural laws
  CM.md .................. concept model gaps
  CM-inventory.md ........ heuristic inventory

src/v2/tests/ (testing — read for verification)
  testing-strategy.md .... generated tests (Tier 3)
```

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
- **Grounding completeness**: target-side primitive types are structurally modeled from the target language reference (Rust Reference §Types, Python data model, Go specification), with algebra inhabitance declared structurally — not string-typed shortcuts in a lookup table. Mapping from a `.dag` type to a target primitive is a structural algebra-homomorphism search over declared inhabitance, not a name-keyed table lookup. If a `.dag` type cannot be structurally grounded to a target primitive, the compiler refuses to emit (fail-closed). See `docs/single-emitter-design.md` for architecture; the target-grounding proposal ([PR #695](https://github.com/gunb-ai/gunbc/pull/695), landing at `docs/thesis/target-grounding-proposal.md` on merge) for the concrete work breakdown; ROADMAP §"Post-R1 Program — Grounding Completeness" for the lane structure.

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
- Composition: Transform holds a FunctionRef to an Arrow declaration in the type substrate; the Arrow's body is a sub-DAG of L1 behaviors (for user functions) or a realization declaration in `extdeps/` (for primitives).
- Substrate extension is a C1-class stop signal (seventh connective or sixth behavior) — all four dissolution patterns from §"Structural decompression" must fail with structural arguments before extension is allowed.
- Future candidate (NOT committed): unified substrate dissolving the five behaviors into patterns over Node. Recorded for future consideration only; revisiting requires new failure pressure, not aesthetic preference.

**Free consequences (fall out when Tiers 1-2 close):**
- Automatic parallelism from dependency graph.
- Automatic memoization from purity + cost.
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

**Meta-process modeling:**
- Bootstrap, CI, dev process modeled as .dag workflows.
- `dag run` is the primary execution path.
- Adding a CI gate, a Node field, or a target language requires editing one .dag file.

**Self-hosting — three facets:**

Self-hosting is not one capability; it's three. All three are targets.

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

3. **Tests are data too.** The test suite equivalent of v2's hand-authored
   `pipeline.rs` (`src/v2/tests/src/pipeline.rs` — the large pipeline/
   contract test file; live LOC reads from the file) exists only as
   `.dag` `TestClaim` declarations and generated target-language test code.
   Under the 0-floor target (per `docs/design-pure-bootstrap-zero.md`, LIVE
   2026-04-25), the prior `TESTING.md §"Post-R2 shape"` two-residual carve-out
   is **retracted**: helper unit tests vanish naturally as their hand-Rust
   subjects dissolve; boundary tests that invoke external toolchains (rustc,
   go, python) migrate to `ExecuteCommand`-based `.dag` `TestClaim`
   declarations. Everything ports to `.dag`. **Pure Bootstrap's secondary
   deliverable, couples to testgen.**

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
The live *count* of currently hand-authored files reads from the full
SG-0 census in `src/v3/compiler/tests/integration/sg0_census_test.rs`
(authority on the census): file-level **`EXPECTED_HAND_AUTHORED_NON_TEST`**
plus **`EXPECTED_HAND_AUTHORED_FRAGMENTS`** (crate-root scaffolds) for the
T-PB-A non-test surface, **`EXPECTED_HAND_AUTHORED_TEST`** for the T-PB-B test
surface, and mechanical checks that paths land in the correct sub-ratchet.
Both subsets shrink toward 0 under the 0-floor cascade promotion.
Generated escape hatch is acceptable for additional files; hand-authored
files are not.
v2 achieves this pattern at ~97% (2 hand-maintained of 62 stage0 files);
v3's trajectory is the Pure Bootstrap to Zero program (0 hand-maintained).

Fixed-point acceptance: v3 binary compiles `compiler.dag` → produces
bit-identical stage0 Rust + bit-identical emitted artifacts.
`compiler.dag`'s `hand_maintained_src` list monotonically shrinks to the
empty set per [`docs/design-pure-bootstrap-zero.md`](docs/design-pure-bootstrap-zero.md)
(LIVE 2026-04-25; supersedes the ≤5 irreducible-shim framing in
`docs/design-pure-bootstrap.md`).

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
- **Tracks via ROADMAP:** T-Demo's two fixtures exercise both audiences
  (`fixture_compiler_nerd_canonical` for structural-proof audience;
  `fixture_integration_canonical` for glue-generation audience), and
  T-LensAPI provides the opt-in-depth mechanism (user-authored lenses
  extend the proof surface without changing the base language).

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
  express. The operational release gate is ROADMAP T-PB-B's
  `pb_rust_tests_outside_residual_zero`: zero Rust-authored tests exist
  (predicate name retains "outside residual" for predicate-rename housekeeping;
  semantically the residual is empty under cascade promotion, so "outside
  residual" means "all"). The acceptance claim lives in ROADMAP.
- Consequence of the pure-function posture: effects are explicit parameters,
  mocking is dependency-injection-by-construction, no hidden state means no
  flaky tests.

**Enumerable impossible-bug classes:**
- The thesis obligates naming the bug classes that become impossible by
  construction. Not "bugs in general" — enumerable, teachable classes.
- Initial committed list (R1 demo readiness tagged — see ROADMAP §"Release R1 Program"):
  - **[R1]** Suboptimal-complexity contract violation: a function annotated
    `complexity ≤ O(n log n)` whose actual complexity exceeds it errors at
    compile time, not review time. Demo via T-LaneE output on the
    compiler-nerd fixture.
  - **[R1]** Idempotency-contract violation: a function marked `@idempotent`
    whose structure admits non-idempotent composition errors. Lens is
    already COMPLETE per the lens capability register.
  - **[R1]** Transport/type drift: client and server cannot hold different
    types for the same field — both derive from the same declaration. Demo
    via T-Emit multi-target output on the integration fixture.
  - **[R2+]** Nested-optional flatten: `Option<Option<T>>` accessor patterns
    that normal languages require hand-unwrapping. Gated on cardinality
    refinement substrate work.
  - **[R2+]** Unenumerated effects: every effect performed in this system is
    structurally visible at compile time; impossible to hide an effect because
    the substrate IS the registry (effects derive from the composition of
    typed primitive operations, parallel to how complexity derives from
    bounded-kernel composition — same closed-system mechanism). There is no
    declared-effects-vs-inferred-effects mismatch because there is no separate
    "declared" surface — the structural fact is the only authority. Redundant
    effects (e.g., reads of the same key with no intervening write-effect) are
    structurally provable as identical via referential transparency, and
    rejected at compile time; legitimate re-read uses an explicit `reread()`
    primitive that structurally tags the intent. Tier 1 (impossible by
    construction), not Tier 2 (lens-detected). See
    [`docs/briefs/t-impossiblebugs-unenumerated-effects-design.md`](docs/briefs/t-impossiblebugs-unenumerated-effects-design.md)
    for the 5-behavior compositional-fold mechanism + worked examples.
  - **[R2+]** Unhandled diagnostic paths: Tier 2 runtime-safety proofs make
    division-by-zero, OOB, and force-unwrap either proven safe or made
    total — never partial. Gated on Tier 2 substrate (post-R1).
- Adding a bug class to this list is a thesis commitment; removing one
  requires a named dissolution (the structural proof became trivial).
- [R1] classes must demo at release; [R2+] classes are thesis-committed but
  not demo-scope for R1 (see ROADMAP T-Demo scoping note).

**Modeling discipline:**
- Every declared type has at least one structural consumer.
- Every service boundary uses typed enums, not String/Bool proxies.
- No fabrication sentinels (`__BUG_*`, `__EMIT_BUG_*`). Missing facts are compile-time errors, not runtime strings.
- No duplicate record shapes. One type per concept.

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
