# v3 Roadmap

Single source of truth for v3 status, active work, and deferred items.
Supersedes `src/v3/M1_FOLLOWUPS.md` (now a stub redirect). The shipped
M1(2.5)–M1(3) substrate design rationale is preserved as a historical
record in `src/v3/M1_DESIGN.md` (marked historical; authoritative
docs are the code itself). M0 retrospective in
`src/v3/M0_RETROSPECTIVE.md`. Substrate-consumer gap enumeration in
`src/v3/DOWNSTREAM_REQUIREMENTS.md` — read before proposing new
substrate fields.

> Design spec: [docs/v3-spec.md](../../docs/v3-spec.md)
> Validation: [docs/v3-validation-experiments.md](../../docs/v3-validation-experiments.md)
> Lineage: [docs/design-lineage.md](../../docs/design-lineage.md)

## Status at a glance

| Milestone | State | Notes |
|-----------|-------|-------|
| **M0** Skeleton | ✅ Complete | 40 acceptance tests green. PR #441 merged. M0_RETROSPECTIVE.md closes it. |
| **M1(2.5)** Substrate rework | ✅ Landed on PR #445 | Substrate = six-variant `TypeConnective`, Declaration table, `meta_tag`/`inhabits` split, `ArrowBody` with `Pending` scaffold. Initial handoff was 42 green (40 M0 + 2 substrate); see `M0_RETROSPECTIVE.md` §"M1(2.5) addendum" for the historical snapshot. |
| **M1(2.6)** FACTS FLOW + SINGLE AUTHORITY | ✅ Landed on PR #445 | Rolled into the same PR per Option C. Parser extensions for real `dsl/std/*.dag` files, `include_str!` bootstrap over seven std modules, SubstStack + §8.9 operator dispatch, deleted `inject_primitive_operators`, anonymized TypeParam/variant/realization child declarations, duplicate-name fail-closed, `ExternalRealization` typed-edge check at both construction and dispatch, bootstrap drift routed through `Dag::attach_diagnostic` instead of panic. |
| **M1(2.7)** Enumeration-driven substrate fix | ✅ Landed on PR #445 (R7 + R9) | R7: primitive identity cache, `TransformTarget`/`OperatorKind` coproducts, `ArrowBody::Unparsed`, `SurfaceItem::{Fn, FnExternalBody, Data, Module, Import}` split, `TemplateArgument` stub branch deleted. R9 (ChatGPT follow-up): `std/algebra.dag` extended with direct operator fields (`sub`, `div`, `eq`, `ne`, `lt`, `le`, `gt`, `ge` on `OrderedRing<T>`); `resolve_operator_arrow` rewritten as a structural §8.9 walk that reads algebra field signatures and substitutes the receiver type parameter to the source declaration; `Declaration.value_body: Option<ValueBody>` added so data items are structurally distinguishable from type aliases. Class-5 gaps (Bool operator grounding, collection-algebra receivers, data body parsing) tracked in `DOWNSTREAM_REQUIREMENTS.md`. |
| **M1(2.8)** Match expression parser catch-up | ✅ Landed on PR #445 | Added `SurfaceExpr::Match` + `SurfacePattern::BareVariant` parsing. Extended `Path` with `BranchPattern { UnresolvedVariant, ResolvedVariant }` phase coproduct. Generalized Branch input check from "must be Bool" to "must be Disj" — `if`/`else` still works because Bool IS a Disj in types.dag. `if`/`else` lowering rewired to emit explicit `UnresolvedVariant{"True"}/{"False"}` patterns instead of positional convention. New infer-time pattern resolution pass walks each Branch's scrutinee type and resolves each path's variant name scoped against the Disj children. New class-5 gap #4 (variant RHS expressions blocked on anonymization) tracked for logic.dag's `classical_not`/`and`/`or` which still load as `FnExternalBody`. **Current: 41 M0 + 22 M1 substrate + 7 real-stdlib parse smoke + 1 realization smoke = 71 green.** |
| **M1(3)** First downstream consumer (PR-B) | ✅ Landed on PR #445 | The first v3 emitter pipeline ran end-to-end: `compile_to_dag("let x: Int = 1 + 2") → emit_rust → rustc → execute → "3"`. Substrate additions: `ValueBody::Structural { fields: Vec<(String, LiteralBits)> }` for record-literal data bodies, `SurfaceExpr::Record` parser support, `lower_record_to_structural` inhabitance check (walks the type's Conj, fail-closed on extras / missing / wrong-type fields), `src/v3/spec/rust.dag` as the first extdeps fixture in production bootstrap (declaring `Realization` + 18 `data rust_*` items covering primitives, operators, and structural templates). New lenses + emitter: `lens_cost.rs` (third pure-reader lens, ~80 lines, follows the lens_depth/lens_provenance template), `emit_rust.rs` (~340 lines, builds a `(target_name, op_name) → carrier` index from rust.dag declarations and walks the DAG translating per Behavior). End-to-end roundtrip test gated behind `#[ignore]` so CI doesn't depend on a Rust toolchain. **Current: 41 M0 + 35 M1 substrate + 6 lens_cost + 7 emit_rust + 7 real-stdlib parse smoke + 1 realization smoke = 97 green.** |
| **L1** Reflection framework | ✅ Complete | PR #466 merged 2026-04-16. All prereqs shipped: Prereq 0 (HoF, #460), Prereq 0.5 (implicit generics, #466), Prereq 1 (FieldProject, #458), Prereq 2 (Path.binding, #458), Prereq 3 (contextual lambda, #460), Prereq 4 (list.dag bootstrap, #463), Prereq 5 (pipe sugar, #462). substrate.dag reflects Dag/Behavior/Declaration types. First lens migration (unused_parameters.dag) compiles, matches handwritten oracle, self-analyzes to zero. Optional-handle support (T? with Some/None) landed. Module-mode emission + crate-linked roundtrip proven. |
| **L1.5** Clean bootstrap | 🟡 In progress (2026-04-16) | Test authority types (#474 ✅), ownership Phase 1 / 72→6 clones (#475 ✅), dependency+rendering design doc (#477 ✅). **Remaining:** pipeline composition declaration + fixed-point regen (#476 — PAUSED pending Option B authority migration: pipeline.dag becomes live authority, Rust derives from it). Ownership Phase 2 (→ clone count 1) and multi-target validation (go.dag) queued as parallel tracks. See `SELF_HOSTING.md` §2, §11, §14 and `docs/dependency-and-rendering-design.md`. |
| **Post-A/B** Lane Plan | 🟡 Planned (2026-04-17) | Four major lanes derived backward from THESIS.md claims, sixteen stages total (per-stage t-shirt sizes S/M/L/XL), **no backlog** — every open thesis obligation is placed in a lane. Master: [post-l15-phase-plan.md](../../docs/post-l15-phase-plan.md) (includes sequencing + dependency graph). Lane 1 (emission unification): [lane1-stage-b-substrate-keyed-lookup.md](../../docs/lane1-stage-b-substrate-keyed-lookup.md), [phase1-lane1-l15-tail.md](../../docs/phase1-lane1-l15-tail.md), [phase1-lane2-clean-emission-invariant.md](../../docs/phase1-lane2-clean-emission-invariant.md), [phase1-lane3-consolidation-build-plan.md](../../docs/phase1-lane3-consolidation-build-plan.md). Lane 2 (compile-time proofs): [lane2-compile-time-proofs.md](../../docs/lane2-compile-time-proofs.md). Lane 3 (self-hosting cycle): [lane3-self-hosting-cycle.md](../../docs/lane3-self-hosting-cycle.md). Lane 4 (completion): [lane4-completion.md](../../docs/lane4-completion.md). |
| **M1(4)** Multi-target emission | ⏸ Absorbed into Lane 1 | Originally planned as parallel `emit_go` / `emit_python` walks. Lane 1 consolidates all emitters into a single generic walker + per-target specs, then adds Verilog + SPICE + English as the smoking-gun. "One file per target" framing inverted: each target is one spec file, zero new Rust. |
| **M2** Feature parity | ⏸ Absorbed into Lane 3 Stage 3a | Generics in user code: ✅ (Prereq 0.5). Match in user code: ✅ (M1(2.8) + Prereq 2). List operations: ✅ (Prereq 4, structural List<T>). Recursion → Loop: ✅ (numeric descent). **Remaining:** transport declarations, interpreter (`dag run`), mutual recursion → Loop (§2.4), `data` value semantics, `where` refinement, full surface generics — all blockers for `compiler.dag`, sequenced in [lane3-self-hosting-cycle.md](../../docs/lane3-self-hosting-cycle.md) Stage 3a. |
| **M3** Self-hosting | ⏸ Absorbed into Lane 3 Stage 3c | `.dag` rewrite of the compiler IS the self-hosting cycle: `compiler.dag` → Lane 1e emitter → Rust → `rustc` → identical binary. Design: [lane3-self-hosting-cycle.md](../../docs/lane3-self-hosting-cycle.md), SELF_HOSTING.md. |
| **M4** Thesis completion | ⏸ Absorbed across Lanes 1–3 | "All lenses, verification, omni-emission" decomposes to: omni-emission = Lane 1e + 1f. Lenses = Lane 2 (idempotency, symbolic cost, parallelism, user-dimensions). Verification = Lane 3b (diagnostics-as-corrections). No longer a vague milestone — every component has an owning lane. |

## Principles

- Keep it simple. If a file gets large, something is wrong.
- Behaviors compose from std/. Hardcoded rules = missing modeling.
- Every decision should trace to a validation experiment or a v2 lesson.
- v2 is the reference implementation and test oracle.
- **Facts flow forward from declaration source to consumer.** Bootstrap
  fixtures that parallel `dsl/std/*.dag` are debt — delete them the
  moment the parser can consume the real files.
- **Single authority.** One declaration per concept. If the compiler
  needs a primitive operator, it walks inhabitance, not a bootstrap
  pre-registration table.
- **ROADMAP is the tracker.** All in-flight work, deferrals, and
  follow-ups from merged PRs live in `src/v3/ROADMAP.md` (and the lane
  / design docs it points at) — not in GitHub issues. A PR merging
  in/out of main MUST leave the ROADMAP reflecting the new state:
  new ✅ checkboxes for what shipped, new entries in the Active
  Deferrals section for anything deferred. Reviewers block on this.
  Rationale: GitHub issues fork authority from the code+docs the ROADMAP
  points at, and they rot silently when a session forgets to update
  them. A single file the whole project reads every day doesn't rot.

## Sketch vs Oracle framing (M0–M2)

**The Rust at `src/v3/compiler/` is a sketch, not an oracle.** During
M0–M2, the Rust implementation exists to validate the substrate
design — to discover whether the L1 decomposition, port invariants,
lens architecture, and diagnostic system hold when built against.
Discovery, not specification. The `.dag` version (M3) is the real v3;
it will be written fresh against the same test suite.

Style consequences:
- **Style matches Rust, not .dag.** Imperative patterns (mutable Dag,
  HashMap scope mutation, fixpoint loops) are fine where they fit
  Rust's affordances.
- **Refactor only where the pattern is structurally gapped.** M0.6's
  immutable-scope refactor was an example (`&mut HashMap` had no .dag
  analogue). The M1(2.5) two-pass lowering with
  `resolve_pending_identifiers` is another (the sweep pattern works
  in any language).
- **At M3**, the Rust's role transitions. Re-evaluate patterns during
  the port attempt, not pre-emptively.

## Architecture

```
Source text → tokenize → parse → lower → Dag (declarations + behaviors)
                                          │
                                          ├── infer (writes port state)
                                          ├── lenses read the DAG (cost, ownership, effects, ...)
                                          └── emitter translates DAG + LanguageSpec → text
```

Five L1 behaviors: `Value`, `Transform`, `Branch`, `Loop`, `Bind`.
Six type connectives: `Atom`, `Conj`, `Disj`, `Arrow`, `Cardinality`,
`Instantiation`. Both sets are terminal at M1(2.5); extension requires
the C1-class stop signal (all four dissolution patterns must be re-run
before any new variant lands — see `INVARIANTS.md` §"Scaffold boundaries"
and §"Semantic authority after lowering").

## M0 — Skeleton (complete)

See `M0_RETROSPECTIVE.md` for the full retrospective. Highlights:
- Five L1 behaviors survived 10 milestones + 3 reviewer rounds.
- `PortState::{Uninferred, Resolved, Unresolved}` became structural
  after the M0.6 refactor; the fail-closed biconditional holds by
  construction.
- Spans live on every Behavior structurally, not in a side table.
- Declaration of terminal status: five behaviors, documented. Adding
  a 6th triggers the C1 stop signal.

## M1(2.5) — Substrate rework (shipped in PR #445)

Historical design oracle preserved in `M1_DESIGN.md` (marked
historical). Substrate changes landed:
- `TypeConnective` six-variant enum
- `Declaration` struct with canonical `type_params`, separate `meta_tag`
  and `inhabits` edges
- `ArrowBody { UserDefined, ExternalRealization, Pending }`
- Two-pass lowering with `resolve_pending_identifiers` post-sweep
  (fail-closed for unresolved identifier stubs)
- `build_template_arguments` for fail-closed template arity check
- Dissolution ledger receipts in `dag.rs`
- §6.5 realization smoke test (`inject_realization_stub` +
  `smoke_int_add_external_realization`)
- v3 CI job runs `cargo test -p v3-compiler` + clippy in its own job
  parallel to the v2 `ci` pipeline.

Review cycle items absorbed into the PR:
- **Codex**: all four blockers fixed (variant allocation ordering,
  stub diagnostics via post-sweep, canonical `type_params` slot,
  dissolution ledger receipts).
- **ChatGPT** (mechanical): unwrap_or/template mismatch replaced with
  `build_template_arguments`; FAIL-CLOSED gap for declarations closed
  via phantom-port diagnostics.

## M1(2.6) — FACTS FLOW FORWARD + SINGLE AUTHORITY (active, PR #445)

**Why now:** ChatGPT's review flagged two blocking structural patterns
that harden the wrong shape if landed. Rolling M1(2.6) into PR #445
closes both before any downstream code depends on the interim
bootstrap-as-authority shape.

### Concerns being resolved

1. **FACTS FLOW FORWARD** — bootstrap currently embeds four fixture
   strings instead of parsing `dsl/std/*.dag`. Every primitive change
   means editing bootstrap, not the source of truth.
2. **SINGLE AUTHORITY** — `bootstrap::inject_primitive_operators`
   registers `"+"`/`"-"`/... as named Arrow declarations parallel to
   `dsl/std/algebra.dag`'s OrderedRing.add/sub/mul/... These are
   duplicate representations of the same fact.

### Phases

| # | Phase | Status |
|---|-------|--------|
| 0 | Consolidate tracking into this ROADMAP | ✅ |
| 1 | Parser extensions for real std/ syntax (`module`/`import`/`match`/`data`/`where`/`=>`/`.`, block-body fn skip, data body skip, record-payload sum variants) | ⏳ |
| 2 | Bootstrap consumes real `dsl/std/{logic,bit,algebra,types}.dag` via `include_str!` | ⏳ |
| 3 | `SubstStack` + §8.9 inhabitance walks in `infer.rs`; hardcoded `OPERATOR_FIELD_MAP` as the last localized bridge | ⏳ |
| 4 | Delete `inject_primitive_operators`; refactor `declaration_to_type_shape` to walk-based | ⏳ |
| 5 | Test updates (`assert_target_name` follows identifier payload; substrate tests keep walking real declarations) | ⏳ |
| 6 | Clippy + audits + commit + force-push PR #445 | ⏳ |

### Scope boundaries at M1(2.6)

**In:** enough parser surface to consume the four bootstrap files.
SubstStack + §8.9 operator dispatch. Delete bootstrap's operator
injection. Walk-based primitive bridge.

**Out:** match/pipe/lambda/named-arg expression parsing (function
bodies stay opaque), `data` value semantics, `where` refinement
checking, full surface generics, transport declarations, `List<T>` in
user code, `TypeShape → DeclarationId` migration.

### Bridges at end of M1(2.6)

After M1(2.6) landed, one localized bridge remained:
`OPERATOR_FIELD_MAP` in `infer.rs`, mapping operator symbols to
algebra field names for the §8.9 inhabitance fast path. M1(2.7)
deleted it — operator dispatch became fully structural via
`TransformTarget::Operator(OperatorKind)`. See the M1(2.7) section
below.

## M1(2.7) — Enumeration-driven substrate fix (landed on PR #445)

**Why this pass:** every review round on PR #445 caught the same
bug shape — one substrate field carrying multiple downstream jobs,
with a sibling string as the discriminator. The enumeration pass
(diagnostic-only commit) walked every substrate consumer, cataloged
14 structural gaps in `DOWNSTREAM_REQUIREMENTS.md`, and the fix PR
resolved all 14 in one coherent substrate change rather than
reactive per-reviewer fixes.

Artifact: [`src/v3/DOWNSTREAM_REQUIREMENTS.md`](DOWNSTREAM_REQUIREMENTS.md).
Scope: both the read side (`infer.rs`, `lens_depth.rs`,
`lens_provenance.rs`) and the write side (`parse.rs` →
`lower.rs` boundary). Re-run when cost + ownership lenses land.

### Resolved gaps

**Class 1 — Primitive type identity** (4 gaps: Q1, Q2, Q4, QW5).
Resolved by adding a `PrimitiveCache` on `Dag` populated at
bootstrap. `Dag::int_shape()`, `Dag::bool_shape()`,
`Dag::string_shape()` return cached `TypeShape`s in O(1). The
QW5 `lower_type_for_port` whitelist is gone — port-type resolution
now goes through `type_to_declaration_id` (same authority that
declaration-side lowering uses). Fail-closed port diagnostics are
preserved via a top-level fresh-stub check.

**Class 2 — Operator dispatch** (2 gaps: Q3, Q4). Resolved by
structurally splitting operator dispatch from identifier
resolution. `TransformTarget { Callable(DeclarationId),
Operator(OperatorKind) }` replaces the single `target:
DeclarationId` field. `OperatorKind { Arithmetic(ArithmeticOp),
Comparison(ComparisonOp) }` encodes the output-type rule as
variants. `SurfaceExpr::Operator` is a first-class parser shape
— operators never allocate stub declarations. `OPERATOR_FIELD_MAP`,
`is_operator_name`, `is_comparison_operator`,
`unresolved_operator_name` all deleted.

**Class 3 — Scaffold honesty** (3 gaps: QW1, QW2, QW4). Resolved
by making every surface form a real `SurfaceItem` with tracked
dissolution.

- **QW1** `fn foo(x) -> T { body }` now parses as
  `SurfaceItem::FnExternalBody` (sibling variant to `Fn`, not an
  `Option<body>` discriminator). Lowers to a declaration whose
  connective is an `Arrow` with `ArrowBody::Unparsed(body_span)`.
  The signature flows forward — callers can type-check against it —
  and the body stays scaffolded until the M2+ parser adopts
  match/pipe/lambda.
- **QW2** `data name: Type = { body }` parses as
  `SurfaceItem::Data`. Lowers to a declaration whose connective
  resolves from the type annotation through `type_to_connective`.
  The `kernel_algebra_profile` / `kernel_type_set` / etc. tables
  in `dsl/std/*.dag` now survive into the declaration table.
- **QW3** `module` and `import` items become
  `SurfaceItem::Module { path }` / `SurfaceItem::Import { path,
  names }`. No-op at M1(2.7) but the parsed facts are preserved
  for M2+ module scoping to consume.
- **QW4** `TemplateArgument` stub self-reference branch deleted.
  When `build_template_arguments` encounters a stub template, it
  returns `Vec::new()` — the stub's own diagnostic is the
  authoritative failure, and no `TemplateArgument` is constructed
  in a state its field contract declares invalid.

**Class 4 — Parallel authorities** (2 gaps: Q8, QW5). Resolved
by the same PrimitiveCache introduced in Class 1. Q8's
`is_realization_shape` compares `meta_tag` against
`Dag::realization_meta_id()` (cached `DeclarationId`) instead of
comparing a name to the literal `"Realization"`. QW5 is addressed
alongside Class 1.

### Remaining scaffolds after M1(2.7)

Four tracked scaffolds, each with a named dissolution trigger:

- **`ArrowBody::Pending`** — realization lag. Dissolves via the
  §8.11 monotonic-decrease ratchet when every realization arrow
  binds to a real `ExternalRealization` declaration (M3).
- **`ArrowBody::Unparsed`** — block-body lag. Dissolves when the
  M2+ surface grammar adopts match/pipe/lambda/etc. so block
  bodies lower to full `UserDefined(NodeId)` arrows.
- **`ValueBody::Unparsed`** — data-body lag. Dissolves when the
  M2+ parser adopts record/map/list literal `SurfaceExpr`s so
  data declarations lower to `ValueBody::Structural(NodeId)`
  pointing at a value sub-DAG.
- **`TransformTarget::Operator` + `OperatorKind`** — surface
  operator shim. Dissolves when the M2+ parser desugars `a + b`
  to direct algebra-field `Call`s (or adds explicit field-access
  syntax like `Int.add(a, b)`).

All four are documented in dissolution ledgers with explicit
triggers; none are spreading. The `OPERATOR_FIELD_MAP` name
bridge is gone; operator dispatch walks `std/algebra.dag`
algebra fields as consumed authority.

Three **class-5 gaps** surfaced by M1(2.7) R9 remain open as M2
work: Bool operator grounding (no structural link from
`Classical` to `BooleanAlgebra`), collection-algebra receivers
(`FreeMonoid`/`Set`/`Map` receivers are the parameterized
algebra, not the type parameter), and data body parsing
(`kernel_algebra_profile` et al. still aren't structurally
consumable). See `DOWNSTREAM_REQUIREMENTS.md` class 5.

## M1(3) — What PR-B validated

The deferred items above were structured around three open
questions: (a) does the substrate at M1(2.8) actually support a
downstream consumer, (b) does the lens template generalize beyond
the first two reader lenses, and (c) does "add a new emission
target = one spec-file edit" survive contact with a real spec
file. PR-B answered each:

- **(a) Substrate sufficiency.** PR-B added one substrate variant
  (`ValueBody::Structural`) and one parser surface form (record
  literals in data-item position). Every other piece of
  emit_rust + lens_cost reads through existing connectives. The
  thesis assumption that reader lenses + structural emission cost
  zero substrate work for "the next consumer" is validated for the
  PR-B class of consumers: anything whose facts can ride on
  Realization records with literal-only fields. Class-5 gaps #3
  (nested records / port-carried field values), #4 (variant
  constructors as values), and #6 (declaration references as
  values) are still open and will surface as new consumers push
  past PR-B's scope.
- **(b) Lens template scaling.** Cost lens landed at ~80 lines,
  matching depth (~50) and provenance (~40) shapes. Three data
  points on the same curve. The "lens-storage" decision the
  earlier roadmap deferred to M1(3) **dissolved** — no PR-B lens
  needs storage. Pure-reader walks return on demand and
  memoization, if ever needed, is a transparent local concern.
- **(c) Spec-file → emission isomorphism.** `src/v3/spec/rust.dag`
  is the only Rust-syntax source in the codebase.
  emit_rust contains zero hardcoded operator strings, type names,
  or template fragments — every per-target token traces to a
  rust.dag carrier read through the RealizationIndex. Editing
  `"i64"` to `"int64_t"` in rust.dag would propagate to every
  emitted let statement without touching emit_rust.rs.

The follow-up work the previous M1(3) plan named (writer lenses,
multi-target emission, §8.11 ratchet) shifts shape:

- **Writer lenses** — superseded. PR-B proved the dissolution.
  When a future lens has a real reason to persist intermediate
  state (e.g., cross-program optimization fixpoints), the storage
  question fires then on its own merits, not as a pre-emptive
  M1 milestone.
- **Multi-target emission (M1(4))** — go.dag and python.dag, each
  built as a parallel extdeps fixture with ~40 declarations and a
  parallel ~340-line `emit_go` / `emit_python`. The emit walks
  reuse PR-B's RealizationIndex pattern; the "one spec-file edit"
  claim becomes empirical.
- **§8.11 Pending ratchet** — still doc-only. CI wiring is a
  separate housekeeping task tracked in DOWNSTREAM_REQUIREMENTS.
- **`ArrowBody::Pending` removal** — once the ratchet hits zero by
  M3, delete the variant.

## M2 — Feature parity with v2 subset (deferred)

- Generics in user code
- `Optional` / `Cardinality` user-code surface
- Service calls (transport declarations)
- Pattern matching in user-code fn bodies (Branch with destructuring)
- Interpreter (`dag run`)
- Recursive functions → `Loop` lowering (includes n-way mutual
  recursion → bounded `descend` over SCC-ordered nodes). v3's
  `lower.rs` already detects mutual recursion via
  `compute_mutually_recursive` but currently REJECTS it. The
  lowering step that transforms cycles into bounded Loop nodes
  with descend semantics is the missing piece. The thesis's
  lowering table (INVARIANTS.md §"Recursive syntax is sugar")
  commits to handling every call pattern including mutual
  recursion; this is the implementation of that commitment.
  **Prereq for:** the complexity port (complexity reads SCC
  structure from the substrate; if lowering doesn't produce it,
  complexity has to reconstruct it). Also a general language
  completeness feature — any `.dag` program with mutual recursion
  should compile, not fail at lowering.
- Match expressions / pipe / lambda / named arguments in user code
- `data` value semantics
- `where` refinement checking
- `TypeShape` → `DeclarationId` migration

## M3 — Self-hosting (deferred)

Full design: [`src/v3/SELF_HOSTING.md`](SELF_HOSTING.md). Key points:

- **The pipeline is a `.dag` composition** (§2.1). Stages are typed
  functions with declared input/output types, composed as `let`
  bindings with explicit dependencies. The compiler reads its own
  pipeline structure the same way it reads any user program's
  dependency graph. Stage contracts, per-stage fixed-point, and
  self-analysis via lenses all fall out of this shape.
- **Dependency order:** L1 (reflection) → **L1.5 (clean bootstrap
  — immediate, the process is the first feature)** → L2 (lens
  migrations) + L2.5 (per-stage domain modeling, parallel with L2)
  → L3 (pipeline stages in `.dag`: emit → lower → infer → parse)
  → L4 (full self-hosting). L1.5 lands the pipeline composition
  declaration and per-stage fixed-point verification BEFORE any
  other post-reflection work. Every subsequent change goes through
  a bootstrap process that's already structurally sound. L2.5
  models each stage's inputs/outputs to spec BEFORE implementation;
  L2.5 runs ahead of L3 by one stage. See §2 for the full diagram.
- **Infer is the research gate.** Every other stage migration is
  engineering with known patterns. Inference-as-data has no
  production precedent. The I0-I8 experiment sequence
  (`docs/inference-as-data-experiments.md`) is the empirical
  gate. I0 passed (no decidability blocker); the write-surface
  decision (§5 of SELF_HOSTING.md) is the next open question.
- **Schema migration as structural operation** (§10). Schema
  changes become a typed pipeline: structural diff → patch
  generation → bridge build → bridge compile → fixed-point
  verification. Zero manual stage0 edits. Option C (native
  `.dag` patch language) upfront.
- v3 compiles itself
- Bootstrap: v2 compiles v3 stage0, v3 compiles v3 → fixed point
- All v2 test programs compile under v3 with same output
- `OPERATOR_FIELD_MAP` and the port-type whitelist already
  dissolved at M1(2.7); no carried-forward bridges remain at M3.

## M4 — Thesis completion (deferred)

- All lenses operational (cost, ownership, effects, termination,
  algebra, space)
- Diagnostics as corrections — correction field on Diagnostic
  at L1.5 (§14.6 of SELF_HOSTING.md), roundtrip-tested per
  diagnostic variant and per lens. Every lens ships with
  correction computation and fix-roundtrip tests as acceptance
  criteria.
- L4 verification: emitted code matches DAG evaluation
- User-defined observational lenses
- Omni-emission projection rules
- **Test generation at all three layers (§14 of SELF_HOSTING.md):**
  structural testgen (from types, L1.5), behavioral testgen (from
  transforms, L2.6), composition testgen (from pipelines, L3).
  KF-3 becomes empirical progressively across these layers.
  Mock generation + dry-run mode (L2.6) closes the environmental
  boundary bug class.
- **Ownership + clone elision (§14.7 of SELF_HOSTING.md):**
  dedicated parallel track. Phase 1 (lens_fanout + basic clone
  elision) at L1.5 — every generated artifact benefits from day
  one. Full v2 ownership.dag migration (719 lines) at L2.
  Self-analysis clone-count ratchet at zero by L3. This is the
  v2 20-minute self-compile prevention — non-negotiable before
  generated artifacts accumulate.

## Post-A/B Lane Plan

Four major lanes derived backward from the thesis, sixteen stages
total. Per-stage sizes use S/M/L/XL t-shirts; lane totals are
aggregate sizes, not calendar weeks. Full plan with sequencing and
dependency graph:
[../../docs/post-l15-phase-plan.md](../../docs/post-l15-phase-plan.md).

| Lane | Size | Closes | Design doc |
|---|---|---|---|
| **Lane 1 — Emission unification** | XL (six stages) | "Adding a new target = one spec file, zero new Rust" | 4 stage docs, master embedded in phase plan |
| **Lane 2 — Compile-time proofs** | XL (six stages) | "Structural properties are inescapable" (idempotency, symbolic cost, parallelism, user dims) | [lane2-compile-time-proofs.md](../../docs/lane2-compile-time-proofs.md) |
| **Lane 3 — Self-hosting cycle** | XL (three stages, one with five sub-stages) | "Causal engine: compiler is its own first consumer" | [lane3-self-hosting-cycle.md](../../docs/lane3-self-hosting-cycle.md) |
| **Lane 4 — Completion layer** | L (four stages) | Transport declarations, `dag run`, side effects, space bounds, async emission | [lane4-completion.md](../../docs/lane4-completion.md) |

**Hard sequencing:** Lane 1 Stage 1b gates Lane 2 start. Lane 1 Stage
1e gates Lane 3 Stage 3c and Lane 4 Stage 4d. Lane 2 Stage 2f gates
Lane 4 Stages 4b/4c. Lane 3 Stage 3a gates Lane 4 Stage 4a. Critical
path is six stages: `1a → 1b → 1c → 1d → 1e → 3c` (five M, one L).

**Nothing is backlog.** Every item previously marked "deferred M3/M4"
or "what NOT to build yet" is now a stage in a lane with acceptance
gates. Including async emission.

Lane 1 stages and their design docs:
- 1a: [phase1-lane1-l15-tail.md](../../docs/phase1-lane1-l15-tail.md)
- 1b: [lane1-stage-b-substrate-keyed-lookup.md](../../docs/lane1-stage-b-substrate-keyed-lookup.md)
- 1c: [phase1-lane2-clean-emission-invariant.md](../../docs/phase1-lane2-clean-emission-invariant.md)
- 1d: [phase1-lane3-consolidation-build-plan.md](../../docs/phase1-lane3-consolidation-build-plan.md)
- 1e, 1f: written just before each stage starts, informed by what 1a–1d learn

Each stage carries scope, direction, escalation criteria, and
acceptance gates. See the master plan for sequencing details and the
full acceptance checklist for "plan complete."

## Active deferrals — follow-up work from merged PRs

**Discipline:** every PR that defers scope appends an entry below. A
deferral clears when the follow-up PR lands and the entry moves to the
top-table "landed" record. No deferral lives outside this list — if
someone says "we'll do that later," it's either in this list or it's
fiction.

Format: `- [PR #N] title — scope remaining, size, triggering follow-up context.`

### Lane 3 Stage 3a

Sub-stage status (as of 2026-04-17):

| Sub-stage | State | Landed in | Notes |
|---|---|---|---|
| 3a.1 mutual recursion (DB-9, L) | ⏸ Deferred | — | Design approved; unblocks Lane 3 Stage 3c (self-hosting cycle). See **Deferral: 3a.1** below. |
| 3a.2 `data` value semantics (DB-10, S) | ✅ Shipped | PR #496 | Inlining-at-lowering chosen over emit-time inlining — trade-off recorded in DB-10. |
| 3a.3 `where` refinement (DB-11, M→L overrun) | 🟡 Partial | PR #496 | Parser foundation + substrate field `Declaration.refinement` shipped. Semantics deferred — see **Deferral: 3a.3-full** below. |
| 3a.4 surface generics (DB-12, S) | ✅ Shipped | PR #496 | Tests-only landing; infrastructure already wired. |
| 3a.5 Disj dotted-path (DB-13, S) | ✅ Shipped | PR #496 | Tests-only landing; infrastructure already wired. |

**Deferral: 3a.1 mutual recursion (L).** SCC detection + cluster descent verification + flip the `test_mutual_recursion_is_rejected` lock-in test. Unblocks Lane 3 Stage 3c eventually (self-hosting cycle needs mutual recursion in `compiler.dag`). Parallel-startable; no dependencies from tonight's merged PRs. Design: [design-mutual-recursion-lowering.md](../../docs/design-mutual-recursion-lowering.md) (DB-9). Acceptance in DB-9 §Acceptance.

**Deferral: 3a.3-full (L).** Lower `SurfaceParam.refinement` to a predicate `Declaration`; call-site structural-DAG comparison (no interning, no SMT entailment — structural equality on resolved predicate expression DAGs); extend M1(2.8) pattern resolution to narrow arm-scoped ports for predicate-checked values (e.g., `if d != 0 then ...` narrows `d`). **Scaffold cost in main:** `Declaration.refinement: Option<DeclarationId>` is now authored-but-unread at ~15 construction sites. Dissolution trigger: this deferral lands. **Yellow-flag threshold:** if scaffold sits >1 week, actively schedule. Design: [design-m2-feature-parity.md §DB-11](../../docs/design-m2-feature-parity.md). Acceptance in DB-11 §Acceptance.

### Lane 2 Stage 2c — test infrastructure

**Deferral: DB-15 tests-as-declarations extensions (M, blocks Lane 2 Stage 2c).** Design doc (R2 draft): [design-test-infra.md](../../docs/design-test-infra.md). R2 consumes the compiler-as-dependency-analyzer thesis: tests are declarations (extending the existing `src/v3/std/verification.dag` `TestClaim`/`TestSuite` authority), resources are references to `dsl/std/resources.dag`, sharing/caching/incremental execution fall out of the compiler's existing dependency walk. No new caches or runner mechanisms — DB-15 names HOW things depend, then the walk does the rest.

Implementation scope (M, once design locks): extend `TestClaim` with `requires: List<ResourceReference>` and two new `TestPredicate` variants (`BehavioralObservation`, `MockBackedInvariant`); apply tautology-avoidance rule structurally; one-file migration proof. Yellow-flag threshold: design must lock before Lane 2 Stage 2c kickoff.

**Prerequisite deferral: `dsl/std/resources.dag` → v3 reconciliation (S).** Zero references to `Resource`/`acquire`/`release` under `src/v3/` today. DB-15's `requires: List<ResourceReference>` is authored but unconsumed until this lands. Options: port declaration into `src/v3/std/resources.dag`, OR make `dsl/std/resources.dag` bootstrap-consumable by v3. Preferring the latter for single-authority. Separable from DB-15 implementation — can land independently. This is also a dissolution-of-dual-representation item; consider parking it in §Scheduled deletions if dsl/v3 duplication is the framing, or keep as a prerequisite deferral here. Preferring here for now since it's narrowly scoped.

### Lane 1 Stage 1b

**Deferral: 1b full implementation (M).** 1b's first attempt escalated (PR #495 shipped 1a; 1b code was reverted). Root cause: `.dag` linear-walk bodies for substrate accessors polluted every user DAG. DB-14 codifies the correct pattern (ExternalRealization mirroring pipeline.dag). Unblocked once DB-14 (PR #497) lands. Design: [design-substrate-external-primitives.md](../../docs/design-substrate-external-primitives.md) (DB-14). Acceptance in DB-14 §Acceptance.

### How the active-deferrals discipline works

1. A PR that defers scope opens or appends an entry in this section with:
   - Name and triggering PR reference.
   - Concrete remaining scope (fields / functions / tests that must land).
   - Size classification (S/M/L/XL).
   - Design-doc link(s) with the acceptance gates.
   - Yellow-flag threshold — how long the deferral can sit before it needs active scheduling.
2. When the follow-up PR merges, it:
   - Removes the deferral entry from this section.
   - Updates the Stage 3a (or relevant) sub-stage table from 🟡 to ✅ or adds new rows.
   - Notes in the commit message which deferral is cleared.
3. A PR reviewer blocks merge if the deferrals section is stale vs the PR's actual changes.

GitHub issues for this kind of tracking are **closed with a pointer here.** Issues exist for external coordination (user-facing bug reports, security advisories); internal deferrals do not live in issues.

## What NOT to build yet

- **Any fourth per-language emit file** (e.g., `emit_verilog.rs`,
  `emit_spice.rs`). Defer all new emit targets until P2 consolidation
  lands — each additional `emit_X.rs` makes the consolidation
  proportionally harder. Covered in [post-l15-phase-plan.md](../../docs/post-l15-phase-plan.md) §"What NOT to do".
- Advanced diagnostics (Level 3 auto-fix) — P4 territory.
- Async/concurrent emission strategies.

These are thesis goals that fall out when the foundation is right.
Let them emerge.

## Open design questions

1. **Bound source tracking** — `Bound` is currently `count: Port`
   (just an Int). The compiler may need to know WHERE the bound came
   from (collection size vs explicit number) to verify structural
   descent. TBD during cost/termination lens work.
2. **Closure context rule** — when a `Bind` (function definition) has
   an edge into a `Loop`, captures inherit the Loop's fan-out and
   termination context. Documented in the spec; needs to be wired
   into ownership and termination lenses.
3. **Carrier refinement (Tier 2 safety)** — `NonZero` divisors,
   `InBounds` indices, no force-unwrap. Likely: refinement predicates
   on Port types, checked at Branch boundaries. Needs design.
4. **Effect composition** — how effects compose across sequential
   nodes, Branches, and Loops. The spec says "pick the strongest" but
   details (commutativity of service calls, ordering constraints) need
   working out.
5. **Lens storage mechanism** — answered by M1(3) cost lens
   implementation. Until then: open.
