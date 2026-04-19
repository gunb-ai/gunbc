# gunbc Roadmap

Single source of truth for project status, active work, and deferred
items. Promoted from `src/v3/ROADMAP.md`; the previous root
`ROADMAP.md` listed legacy v2-era Track 1–17 milestones and was
deleted to remove the duplicate-authority footgun that produced
misleading review context.

Supersedes `src/v3/M1_FOLLOWUPS.md` (now a stub redirect). The shipped
M1(2.5)–M1(3) substrate design rationale is preserved as a historical
record in `src/v3/M1_DESIGN.md` (marked historical; authoritative
docs are the code itself). M0 retrospective in
`src/v3/M0_RETROSPECTIVE.md`. Substrate-consumer gap enumeration in
`src/v3/DOWNSTREAM_REQUIREMENTS.md` — read before proposing new
substrate fields.

> Design spec: [docs/v3-spec.md](docs/v3-spec.md)
> Validation: [docs/v3-validation-experiments.md](docs/v3-validation-experiments.md)
> Lineage: [docs/design-lineage.md](docs/design-lineage.md)

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
| **Post-A/B** Lane Plan | 🟡 Planned (2026-04-17) | Four major lanes derived backward from THESIS.md claims, sixteen stages total (per-stage t-shirt sizes S/M/L/XL), **no backlog** — every open thesis obligation is placed in a lane. Master: [post-l15-phase-plan.md](docs/post-l15-phase-plan.md) (includes sequencing + dependency graph). Lane 1 (emission unification): [lane1-stage-b-substrate-keyed-lookup.md](docs/lane1-stage-b-substrate-keyed-lookup.md), [phase1-lane1-l15-tail.md](docs/phase1-lane1-l15-tail.md), [phase1-lane2-clean-emission-invariant.md](docs/phase1-lane2-clean-emission-invariant.md), [phase1-lane3-consolidation-build-plan.md](docs/phase1-lane3-consolidation-build-plan.md). Lane 2 (compile-time proofs): [lane2-compile-time-proofs.md](docs/lane2-compile-time-proofs.md). Lane 3 (self-hosting cycle): [lane3-self-hosting-cycle.md](docs/lane3-self-hosting-cycle.md). Lane 4 (completion): [lane4-completion.md](docs/lane4-completion.md). |
| **M1(4)** Multi-target emission | ⏸ Absorbed into Lane 1 | Originally planned as parallel `emit_go` / `emit_python` walks. Lane 1 consolidates all emitters into a single generic walker + per-target specs, then adds Verilog + SPICE + English as the smoking-gun. "One file per target" framing inverted: each target is one spec file, zero new Rust. |
| **M2** Feature parity | ⏸ Absorbed into Lane 3 Stage 3a | Generics, match, list ops, numeric recursion → Loop, `data` values, `where` refinement, surface generics, Disj dotted-path, mutual recursion → Loop (DB-9 R2 / #519): ✅. **Remaining toward a self-describing `compiler.dag`:** transport declarations, interpreter (`dag run`), and any Stage 3a tail called out in [lane3-self-hosting-cycle.md](docs/lane3-self-hosting-cycle.md). |
| **M3** Self-hosting | ⏸ Absorbed into Lane 3 Stage 3c | `.dag` rewrite of the compiler IS the self-hosting cycle: `compiler.dag` → Lane 1e emitter → Rust → `rustc` → identical binary. Design: [lane3-self-hosting-cycle.md](docs/lane3-self-hosting-cycle.md), SELF_HOSTING.md. |
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
  follow-ups from merged PRs live in this file (and the lane
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
- **`ArrowBody::Unparsed` (case 1 only in this bullet)** — `FnExternalBody`
  block-body **parse lag** in std/. Dissolves when the M2+ surface grammar
  adopts match/pipe/lambda/etc. so those bodies lower to full `UserDefined`
  arrows. **`pipeline.dag` `compile` (case 2c)** and **DB-14 accessors**
  also use `Unparsed` with **different** dissolution triggers — see §Scheduled
  deletions and **DB-16** / **Deferral: E-9 substrate accessor bootstrap rewrite**.
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

## M2 — Feature parity (absorbed into Lane 3 Stage 3a)

**Authority split (avoids two competing “what shipped” lists):**

- **Shipped 3a work** — **only** [**§ Lane 3 Stage 3a**](#lane-3-stage-3a) (sub-stage table + Landing notes). Do not infer shipped scope from this section’s prose; read the table rows 3a.1–3a.5.
- **M2 row** in §Status at a glance — executive summary only; if it disagrees with the 3a table, **the 3a table wins**.

**Remaining gaps** (M2-class tail *not* tracked as a 3a sub-stage row):

- Service calls — **transport declarations**
- **Interpreter** (`dag run`)
- `Optional` / `Cardinality` user-code surface (where not already covered by shipped 3a work)
- `TypeShape` → `DeclarationId` migration
- Parser/body **class-5** gaps (e.g. variant RHS in match arms, `FnExternalBody` islands) — [`DOWNSTREAM_REQUIREMENTS.md`](DOWNSTREAM_REQUIREMENTS.md)

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
[docs/post-l15-phase-plan.md](docs/post-l15-phase-plan.md).

| Lane | Size | Closes | Design doc |
|---|---|---|---|
| **Lane 1 — Emission unification** | XL (six stages) | "Adding a new target = one spec file, zero new Rust" | 4 stage docs, master embedded in phase plan |
| **Lane 2 — Compile-time proofs** | XL (six stages) | "Structural properties are inescapable" (idempotency, symbolic cost, parallelism, user dims) | [lane2-compile-time-proofs.md](docs/lane2-compile-time-proofs.md) |
| **Lane 3 — Self-hosting cycle** | XL (three stages, one with five sub-stages) | "Causal engine: compiler is its own first consumer" | [lane3-self-hosting-cycle.md](docs/lane3-self-hosting-cycle.md) |
| **Lane 4 — Completion layer** | L (four stages) | Transport declarations, `dag run`, side effects, space bounds, async emission | [lane4-completion.md](docs/lane4-completion.md) |

**Hard sequencing:** Lane 1 Stage 1b gates Lane 2 start. Lane 1 Stage
1e gates Lane 3 Stage 3c and Lane 4 Stage 4d. Lane 2 Stage 2f gates
Lane 4 Stages 4b/4c. Lane 3 Stage 3a gates Lane 4 Stage 4a. Critical
path is six stages: `1a → 1b → 1c → 1d → 1e → 3c` (five M, one L).

**Nothing is backlog.** Every item previously marked "deferred M3/M4"
or "what NOT to build yet" is now a stage in a lane with acceptance
gates. Including async emission.

Lane 1 stages and their design docs:
- 1a: [phase1-lane1-l15-tail.md](docs/phase1-lane1-l15-tail.md)
- 1b: [lane1-stage-b-substrate-keyed-lookup.md](docs/lane1-stage-b-substrate-keyed-lookup.md)
- 1c: [phase1-lane2-clean-emission-invariant.md](docs/phase1-lane2-clean-emission-invariant.md)
- 1d: [phase1-lane3-consolidation-build-plan.md](docs/phase1-lane3-consolidation-build-plan.md)
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

Sub-stage status (as of 2026-04-18):

| Sub-stage | State | Landed in | Notes |
|---|---|---|---|
| 3a.1 mutual recursion (DB-9, L) | ✅ Shipped | PR #519 (+ substrate primitives #516) | DB-9 R2 lowering: `LoopBound::Descent`, `Dag.clusters`, `Cluster` / `MemberDescent` / `IntraClusterCall`, Track 9 primitives consumed. See **Landing: 3a.1** below. |
| 3a.2 `data` value semantics (DB-10, S) | ✅ Shipped | PR #496 | Inlining-at-lowering chosen over emit-time inlining — trade-off recorded in DB-10. |
| 3a.3 `where` refinement (DB-11, M→L overrun; DB-16 closure) | ✅ Shipped | PR #496 (foundation) + #515 (3a.3-full) + #522 (DB-16 refined-generic substitution) + #524 (pipeline FnExternalBody docs + test) | Predicate lowering, call-site flatten-and-subset discharge, arm-local narrowing, operator-operand refinement stripping, structural callable-predicate identity, and refined-generic substitution all wired. DB-11's out-of-fragment lowering-time rejection + DB-16's phase-materialized substituted-refined carriers close the 3a.3 admitted-vs-supported gate symmetrically. PR #524 adds the pipeline-stage invariant + `FnExternalBody` documentation (cases 1, 2a, 2c) and tracks substrate accessor `Arrow.body` / E-9 alignment under **Deferral: E-9 substrate accessor bootstrap rewrite** below. See **Landing: 3a.3-full**, **Landing: DB-16 refined-generic substitution**, **DB-16 (`FnExternalBody` reconciliation)**, and that deferral. |
| 3a.4 surface generics (DB-12, S) | ✅ Shipped | PR #496 | Tests-only landing; infrastructure already wired. |
| 3a.5 Disj dotted-path (DB-13, S) | ✅ Shipped | PR #496 | Tests-only landing; infrastructure already wired. |

**Landing: 3a.1 mutual recursion (DB-9 R2, PR #519).** Substrate extension (`LoopBound::Descent`, `Dag.clusters` sidecar, `Cluster` / `MemberDescent` / `IntraClusterCall`) + `compute_mutually_recursive` as cluster-shape producer + lock-in tests flipped from rejection to lowering. Substrate integrity primitives (`NonEmptyList`, `NonSingletonList`, `ParamRef`, `TransformRef`) landed with consumers in PR #516. Unblocks continued work toward Lane 3 Stage 3c (self-hosting cycle). Design: [design-mutual-recursion-lowering.md](docs/design-mutual-recursion-lowering.md) (**DB-9 R2** — supersedes R1 lens-level approach). **`compute_mutually_recursive` uses the same `is_first` filter as lowering** (duplicate top-level `fn` bodies cannot overwrite the mutual-recursion call graph); regression: `mutual_recursion_planner_respects_is_first_on_duplicate_fn` in `m1_substrate_test.rs`. **`substrate.dag`:** `ParamRef` / `TransformRef` carry explicit pointers to this file's Track 9 "Tracked debt — substrate constructor-validation asymmetry" section ([phase-plan §3](docs/phase-plan-2026-04-18.md) combined XS brief — closed).

**Landing: 3a.3-full (L).** Consumer wiring for `Declaration.refinement`:
- **Single construction authority, phase-ordered.** A dedicated `lower_parameter_refinements_phase` is the sole caller of `lower_parameter_refinement` for parameter `where` clauses. It runs between the data pre-pass and the main fn-body pass so predicates referencing top-level `data` constants (e.g., `where d > THRESHOLD` with `data THRESHOLD: Int = 10`) resolve against lowered declarations, not placeholders. `seed_function_signature` seeds the Arrow with base declaration ids only; the refinement phase updates the Arrow inputs with refined decls. `lower_fn_item_expr_body` reads the refined Arrow directly; the previous `lower_fn_item_unparsed` helper is removed (seeding already produced the final `Arrow { body: Unparsed(body_span) }` for `SurfaceItem::FnExternalBody`). Before this refactor, body lowering re-ran `lower_parameter_refinement` and overwrote the Arrow, leaving the seeded predicate Bind + refined Declaration orphaned in the DAG.
- **Composite-canonical refinement form.** A port's refinement is always a single predicate `Declaration` — no alias chain. `lower_parameter_refinement` builds the seed form; `narrow_scope_for_predicate` handles narrowing over an already-refined port by cloning the outer predicate's body (re-pointing the refined-parameter slot at a fresh composite slot via `clone_predicate_body`) and joining it with the new cond via `Transform(Logical(And), [cloned_outer, new])`. The resulting refined Declaration aliases the TRUE BASE directly. A user-written `where outer && new` and a narrowing-produced composite share the same substrate shape.
- **Logical operators as first-class primitives.** `OperatorKind::Logical(LogicalOp::{And, Or})` for `&&` / `||`. Parser inserts `parse_logical_or` / `parse_logical_and` between `parse_expr` and `parse_comparison` (standard precedence); `resolve_operator_arrow` types Logical as Bool → Bool → Bool independent of `lhs_type`; emit_rust/go render `&&` / `||`, emit_python renders `and` / `or`. Unlocks both composite `where` clauses and composite narrowing.
- **Call-site discharge (flatten-and-subset over conjuncts).** `check_refinement_discharge` compares the actual argument's TOP-LEVEL refinement against the callee's expected refinement via `predicate_discharges`. Both predicate bodies are flattened into conjunct leaf multisets by recursively unfolding every `Transform(Logical(And), [lhs, rhs])` root; discharge succeeds iff every expected leaf has a structurally-equal (param-paired) actual leaf. Conjunction associativity and grouping are thereby irrelevant: `a && (b && c)`, `(a && b) && c`, and `a && b && c` share one leaf multiset `{a, b, c}` and discharge each other symmetrically. No chain walk — the composite IS the conjunction, expressed on one Declaration. Pure structural; no SMT, no ordering reasoning, no entailment beyond leaf-membership.
- **`signature_type_shape` stops at refinement carriers** so Arrow walks preserve the refined declaration id on the callee side; type equivalence still follows the `ResolvedIdentifier` alias to compare base types.
- **Operator-operand refinement stripping.** `resolve_operator_arrow` normalizes primitive-operator inputs to the underlying base declaration via `strip_refinement_to_base`. Without this, a refined lhs like `Int where d != 0` was mirrored onto every operand position, causing literal operands (e.g., `10` in `d > 10`) to fail discharge against the mirrored refinement.
- **Structural callable identity.** `refinement_targets_equal` compares `TransformTarget::Callable` targets via `declaration_shapes_equivalent` rather than nominal decl id. Call lowering materializes a fresh `Instantiation` per call-site when the callee has retained template arguments; structural comparison on template + arguments is the authoritative identity.
- **Arm-local narrowing.** `lower_expr`'s `If` arm runs `narrow_scope_for_predicate` on the cond. When the cond is a two-argument `Operator`/`Call` with **exactly one** scope-bound free variable (the candidate parameter), lowering rebuilds the then-arm's scope with that name pointing at a freshly-allocated narrowed port typed as the composite refinement described above. Multi-var predicates skip narrowing — single-parameter refinement is the 3a.3 scope.

Acceptance: `src/v3/compiler/tests/m2_feature_parity_test.rs::test_3a3_*` — 16 tests lock refined-parameter compile, literal-arg rejection, matching-refinement forwarding, distinct-refinement non-discharge, if-predicate narrowing (both unrefined and already-refined caller — the latter exercises composite-canonical conjunct matching), non-narrowing rejection, signatureless regression, `&&` / `||` parse + lower + Bool typing, non-Bool operand rejection, structural-callable-identity-across-sites, out-of-fragment predicate rejection at lowering, substrate integrity (Behavior still 5 variants), and top-level `data` references inside predicates. Design: [design-m2-feature-parity.md §DB-11](docs/design-m2-feature-parity.md).

**DB-16 (`FnExternalBody` reconciliation, PR #524).** [design-fn-external-body-reconciliation.md](docs/design-fn-external-body-reconciliation.md) — documents the **`FnExternalBody` / `ArrowBody::Unparsed` split for pipeline work only:** parse lag (case 1), pipeline host stages (case 2a → bootstrap `ExternalRealization` via `PipelineStageBinding`), **`pipeline.dag` `compile` (case 2c → `Unparsed` persists; `pipeline_compile_order_stage_names` reads `compile`'s body span for ordering authority)**. Invariant `pipeline_stages_lower_to_external_realization_not_unparsed` derives stage names from `pipeline_compile_order_stage_names()` (same authority as bootstrap; excludes `compile` itself). Intentionally **does not** canonically document substrate accessor `Unparsed` semantics (see deferral below). **Scheduled deletions:** `ArrowBody::Unparsed` is **split into three rows** (case 1 vs `compile` 2c vs DB-14 interim) in §Scheduled deletions — the M2 grammar milestone removes **case 1** only; **2c** and accessor interim have separate dissolution triggers.

**Deferral: E-9 substrate accessor bootstrap rewrite (substrate, DB-14 follow-on).** DB-14 substrate accessor callables currently keep `ArrowBody::Unparsed` through bootstrap; emitters pair accessor declarations with per-target realizations via `SubstrateAccessorBinding` (see `bootstrap.rs` DB-14 comment — target-specific realization choice cannot collapse to one id at `Dag::new()` without a redesign). **`INVARIANTS.md` §E-9** requires that external realization appear only as `ArrowBody::ExternalRealization(ref)` on the Arrow, to a target-neutral marker, with per-target specs resolving from that marker — no second “externality” channel. **Dissolution trigger:** a follow-up PR extends bootstrap (or one materialization pass) to rewrite accessor Arrow bodies from `Unparsed` to `ExternalRealization(accessor_marker_id)` for each declared substrate accessor, preserving multi-target resolution through the marker + spec tables (structurally parallel to `materialize_pipeline_realizations`). Until that lands, DB-16 and `parse.rs`/`dag.rs` DB-16-scoped comments avoid framing accessor `Unparsed` as a second legitimate steady-state meaning alongside parse lag. Design: [design-substrate-external-primitives.md](docs/design-substrate-external-primitives.md) (DB-14), E-9.

**Landing: DB-16 refined-generic substitution (S, PR #522).** Design: [design-db16-refined-generic-substitution.md](docs/design-db16-refined-generic-substitution.md) (R3 — unified construction authority, post-codex + post-chatgpt reviews). Closes the final DB-11 blocker by materializing substituted-refined carriers at the phase boundary:

- **Single-authority producer-consumer split.** Construction lives in `concretize_decl_with_subst`'s new refinement branch (called from `materialize_callable_signature_instantiations`, `&mut Dag`). `signature_type_shape` stays `&Dag` and gains a read-only pre-terminator lookup via `find_equivalent_substituted_refined_decl`. One construction site, one consumer site, canonical `DeclarationId` per (template, subst) combination.
- **D1 gate.** `refinement_base_requires_substitution` walks the refined carrier's base through `ResolvedIdentifier` hops; returns `true` iff the walk lands on a `TypeParam` bound in `subst` or an `Instantiation` with substitution-bound arguments. For concrete refined carriers it short-circuits to `false` and the DB-11 identity-terminator fires unchanged — all 16 `test_3a3_*` tests regression-guard this.
- **D2 producer walk.** Seven steps: resolve substituted base → extract predicate slots → allocate fresh composite param port → clone predicate body with Transform-target substitution → wrap in fresh Bind → build fresh predicate-Arrow Declaration → allocate substituted-refined carrier. Each failure mode (substituted base doesn't resolve, malformed predicate shape, out-of-fragment body reaching materialization) registers an explicit `Diagnostic::ResolveError` per C-8 rather than silently returning `None`.
- **Transform-target substitution.** `clone_predicate_body` gains a `subst: &SubstStack` parameter whose Transform-target walk routes `Callable(id)` and `FieldProject.field_child` through `concretize_decl_with_subst`. `Operator(_)` stays untouched. Without this, generic helper calls and generic-record projections inside refinement bodies would retain template-rooted TypeParam references post-clone and mismatch at `declaration_shapes_equivalent`'s atom-to-atom bottom — the codex-caught regression class.
- **Dedup.** `find_equivalent_substituted_refined_decl` is the sole lookup helper; used by both concretize (pre-allocation dedup) and `signature_type_shape` (consumer read). Linear scan over `dag.declarations()` via `predicate_bodies_equal_under_subst` — mirrors `find_equivalent_anonymous_instantiation`'s pattern. The cache is the Dag itself; no parallel side table.

Acceptance: `src/v3/compiler/tests/m2_feature_parity_test.rs::test_3a4_*` — 10 tests lock core discharge, distinct-refinement rejection, cross-site identity (via structural DeclarationId count invariant, not just compile success), literal-arg rejection, composite conjunction, narrowing × substitution composition, Callable-target substitution positive + negative, FieldProject-target substitution, and substrate integrity.

**Follow-up — fixpoint-retry explicit test (not blocking).** `test_3a4_refined_generic_retry_on_unbound_type_param` — exercise the `is_retryable_generic_decl` retry path when a TypeParam is unbound at inference iteration N and bound at N+1, locking the retry-then-succeed outcome (not just the retry classification). Currently implicit-covered by the multi-site and callable-in-predicate bonus tests (both depend on fixpoint convergence through retry iterations); explicit construction of the TypeParam-unbound-then-bound scenario requires synthesized fixpoint-iteration timing. **Yellow-flag threshold: 1 month** after DB-16 Part 2 merge. Audit anchor: Q5 construction-authority invariant preserved under retry.

**Follow-up — DB-16 equality-authority consolidation (not blocking, substrate hygiene).** DB-16's `find_equivalent_substituted_refined_decl` dedup uses a subst-aware comparator stack (`predicate_bodies_equal_under_subst`, `transform_targets_equal_under_subst`, `callable_decls_equal_under_subst`, `normalized_instantiation_args`) that shadows DB-11's `refinement_ports_equal` / `refinement_targets_equal` / `declaration_shapes_equivalent`. The two authorities share structure but decide equivalence separately — the codex-caught retained-argument regression on PR #522 (`12fbaff0f` → `3a897f451`) was evidence that they can drift. An attempted collapse (extend `refinement_ports_equal` with a `subst` parameter; fold self-binding normalization into `declaration_shapes_equivalent`) did not preserve subst-threading through nested Instantiation-argument value comparisons and broke fixpoint convergence — reverted. Correct consolidation path requires threading `&SubstStack` through `declaration_shapes_equivalent` itself (wide call surface; ~20 call sites). **Yellow-flag threshold: 1 month** after DB-16 Part 2 merge. Design anchor: `feedback_substrate_principle_audit` (single-authority invariant). The current parallel stack is correctness-preserving — the dedup walker emits strictly stronger matches than DB-11's discharge relation would — but represents maintenance surface that future drift would re-expose.

Closed (DB-11, PR #515):

- **Admitted surface vs supported fragment.** `lower_parameter_refinements_phase` now calls `refinement_predicate_out_of_fragment` on every `where` predicate before lowering; `Branch` / `Loop` / `Bind`-shaped predicate surfaces (`SurfaceExpr::If` / `Match` / `Lambda`) are rejected at the lowering boundary with an explicit "unsupported shape" diagnostic instead of failing silently at discharge as generic "not equal" mismatches. Admitted surface now matches the fragment `refinement_ports_equal` and `clone_predicate_body` actually support.

Closed (DB-16, PR #522):

- **Refined generic parameter substitution.** Design: [design-db16-refined-generic-substitution.md](docs/design-db16-refined-generic-substitution.md). Single-authority producer-consumer split: `concretize_decl_with_subst`'s new refinement branch (phase-materialized, `&mut Dag`) constructs the substituted-refined carrier; `signature_type_shape`'s new pre-terminator gate (read-only, `&Dag`) looks it up via `find_equivalent_substituted_refined_decl`. `clone_predicate_body` extended with a `subst: &SubstStack` parameter so Transform-target `Callable(id)` and `FieldProject.field_child` references get concretized through the substitution stack during cloning — not just the parameter slot, addressing the `declaration_shapes_equivalent` hole codex caught on R1. DB-11's existing narrowing callers pass `&SubstStack::new()` and see no behavior change; all 16 `test_3a3_*` tests remain green as regression guards. Acceptance: `test_3a4_*` suite locks refined-generic substitution, no-entailment preservation under substitution, cross-site structural identity, literal-argument rejection, composite conjunction, Callable- and FieldProject-target substitution, and five-Behavior substrate integrity.

Follow-up (not blocking): emission for narrowed ports currently errors if `emit_rust` is invoked on a DAG whose narrow ports lack a producer. Acceptable today because Lane 1e's single-emitter consolidation hasn't landed and the 3a.3 acceptance is compile-only; wire a Bind-alias or emission-local name shim alongside Lane 1e when it lands. DB-16's substituted-refined carriers inherit the same narrowed-port shim requirement.

### Lane 2 Stage 2b — workflow idempotency lens

**DB-18 landed — effects algebra + native Rust analysis.** This delivers the **`.dag` + Rust carrier story** and `v3_compiler::analyze_workflow`; it does **not** complete thesis-grade **declared-substrate self-inspection** for workflow facts (that requires reflecting `lane2_workflow` — see **Reflection boundary** below). Do not treat Stage 2b as “fully self-hosted through `.dag` lenses” until that reflection ships. `src/v3/std/effects.dag`: `WorkflowEffect` (linear / branch / loop / parallel), `BranchPredicateRef` + `BranchArm { condition: BranchPredicateRef, ... }`, `WorkflowIdempotencyReport` (`WorkflowCompositionVerdict(CompositionVerdict) | IdempotencyUnsupported(...)`), helpers `nel_to_operation_effect_list`, `report_unsupported_workflow_variant`. `src/v3/compiler/src/dag.rs`: Rust mirrors + `Dag::branch_arm_of` (packages a Bool-resolved port as `BranchPredicateRef` — same Track 9 handle pattern as `param_of` / `as_transform_ref`, not a raw `PortId` on the arm). Consumer: `v3_compiler::analyze_workflow(d, workflow_root: NodeId)` reads workflow facts only from native `Value`/`Bind` node fields (`lane2_workflow` — no parallel side table), set by lowering or staging `Dag::try_register_lane2_workflow_effect` until service lowering attaches the same fields — [`workflow_idempotency.rs`](src/v3/compiler/src/workflow_idempotency.rs) + thin [`lens_idempotency.rs`](src/v3/compiler/src/lens_idempotency.rs); [`src/v3/lenses/idempotency.dag`](src/v3/lenses/idempotency.dag) holds the API + staging note (user-module `match` on `WorkflowEffect` not yet emit-table — full dispatcher is Rust until the class-5 gap closes). Tests: [`lane2_stage_2b_db18_test.rs`](src/v3/compiler/tests/lane2_stage_2b_db18_test.rs). Framing: [lane2-compile-time-proofs.md](docs/lane2-compile-time-proofs.md) Stage 2b.

**Reflection boundary (named staging).** `lane2_workflow` exists only on **compiler-native** `ValueNode` / `BindNode`; it is **not** part of the reflected `Behavior` vocabulary in `substrate.dag` that `.dag` lenses introspect today — so Stage 2b does **not** yet claim full “self-inspection through declared substrate” for that pocket. **Dissolution (tracked):** reflect a workflow-fact carrier through substrate (+ realization wiring) so Rust and `.dag` lenses consume the same inspectable fact; until then `try_register_lane2_workflow_effect` is the explicit test/native hook (documented, not a silent parallel authority).

- **[PR #534] Lane 2 Stage 2b — reflect workflow facts in substrate.** Staging debt: native `lane2_workflow` on `Value`/`Bind` plus `Dag::try_register_lane2_workflow_effect` remain authoritative **only until** the same fact exists on reflected `Behavior` and `src/v3/lenses/idempotency.dag` can stop delegating to Rust. **Clears when:** follow-up PR lands substrate (+ realization) support and removes or demotes the native-only pocket as primary authority. Cross-refs: bullets above; [lane2-compile-time-proofs.md](docs/lane2-compile-time-proofs.md) Stage 2b.
- **[PR #534 → DB-18 Part 2] Lane 2 Stage 2b — R1→R2 carrier-shape reconciliation.** PR #534 shipped the R1-carrier shape (`BranchPredicateRef { port: PortId }` as a raw record, `Dag::branch_arm_of(port, body) -> BranchArm?` validating at the arm constructor, `LinearEffect.ops: NonEmptyList<OperationEffect>`); [design-db18-workflow-effect-carrier.md](docs/design-db18-workflow-effect-carrier.md) locks the R2 shape (`BoolPortRef` typed opaque handle with `bool_port_of(dag, port) -> BoolPortRef?` validating at the handle constructor; `LinearEffect.ops: List<OperationEffect>` admitting the empty monoidal identity). **Clears when:** follow-up PR migrates substrate + Rust mirror to the R2 shape, rewires `BranchArm.condition: BoolPortRef`, relaxes `LinearEffect.ops` to `List`, and updates fixtures (incl. a new empty-`LinearEffect` monoidal-identity fixture). The reflection work above and this shape migration are the two halves of DB-18 Part 2; either or both can land in the follow-up PR.

### Lane 2 Stage 2c — test infrastructure

**DB-15 (R2) — ✅ implementation landed (test-runner wiring remains follow-up).** Design doc: [design-test-infra.md](docs/design-test-infra.md). `src/v3/std/verification.dag`: `TestClaim.requires: List<ResourceReference>` (sole obligation surface for mock transports — `MockBackedInvariant` does not duplicate `ResourceReference`); `TestPredicate` extends with `BehavioralObservation` / `MockBackedInvariant` (tautology-avoiding predicate shapes per R2); `TestObligation` + `materialize_test_obligations` (dependency-walk projection over declared claims — not workflow-structure). **`v3.std.resources`** (`src/v3/std/resources.dag`): minimal `ResourceHandle` / `ResourceReference` carriers — `ResourceHandle` uses the same field labels as `dsl/std/resources.dag` (`type` / `resource_id` / `key` / `cap: Secret`); module name avoids clashing with `dsl/std/resources.dag`'s `std.resources` during bootstrap; full `resource { }` port remains a dissolution item toward single authority with dsl.

**Remaining Stage 2c consumer:** generated test execution / runner integration (out of scope for the DB-15 schema PR).

### Lane 2 Stage 2a / Track 17a boundary

Cleared (this PR, R3-final): the `ComposedEffect` record is gone; `EffectShape` is partitioned by idempotency class; `compose_effects` returns `CompositionVerdict` directly. The final shapes are:
- `type IdempotentShape = ReadEffect | UpsertEffect { key_source } | DeleteEffect { key_source }`
- `type BreakingShape = CreateEffect { cause } | AppendEffect`
- `type EffectShape = IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)`
- `type BreakingOperation { operation_name, shape: BreakingShape }`
- `type CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker: BreakingOperation }`
- (No `ComposedEffect`.)
- `fn compose_effects(effects: List<OperationEffect>) -> CompositionVerdict`

Three illegal-state boundaries dissolved, in order: the old `Bool + String?` admitted `(true, Some(_))` and `(false, None)`; R1's `BrokenBy { first_breaker: OperationEffect }` admitted `OperationEffect { shape: ReadEffect }` in the breaker slot (codex flag); R2's `ComposedEffect { operations, verdict }` admitted `verdict: IdempotentComposition` alongside `operations: [breaker]`, and `BrokenBy { first_breaker }` whose breaker is not in `operations` (ChatGPT flag — records are directly constructible in `.dag`, so the constructor's coherence is behavioral). R3 drops the outer record; `CompositionVerdict` is sound all the way down and `List<OperationEffect>` stays at the caller site. Downstream: `is_idempotent_effect` reduces to a two-arm outer match; `classify_idempotent_disagreement` takes `BreakingShape` directly (dead-arm cleanup); Stage 2c's obligation generator already consumed `List<OperationEffect>` directly, so no consumer regresses. Design: [design-composed-effect-reshape.md](docs/design-composed-effect-reshape.md). v3-only — v2's `dsl/std/effects.dag` stays on the flat `EffectShape` + `Bool + String?` shape per the same scope discipline PR #521 used. The Stage 2b pre-start gate no longer names `ComposedEffect` as an open shape.

**Review arc.** R1 at `b42edc15` landed the `Bool + String?` → `CompositionVerdict` lift with `BrokenBy { first_breaker: OperationEffect }` wrapped in `ComposedEffect { operations, verdict }`. Codex review flagged the inner-variant hole (breaker payload admitted idempotent shapes); R2 partitioned `EffectShape` and narrowed `BrokenBy` to `BreakingOperation`. ChatGPT review then flagged the outer-record hole (operations/verdict correlation was behavioral, not structural) and recommended returning `CompositionVerdict` directly; R3 is that fix. Each round pushed state-space soundness one layer deeper: outer sum → inner payload → record wrapper.

**Known trade-off, not a blocker for Stage 2a.** `BreakingOperation` is a structural copy of its originating `OperationEffect`, not a carrier-relative witness into the input list. A caller can construct a standalone `BreakingOperation` and wrap it in `BrokenBy` without any type-level tie to any `List<OperationEffect>`; the discipline "go through `compose_effects`" is convention, not shape. The right shape for this is an `ElementRef<OperationEffect>`-style handle, analogous to `ParamRef` / `TransformRef` in Track 9 (see `ROADMAP.md:763-772`). Per Track 9's stated policy — "the handle lands when a concrete consumer needs it, not speculatively" — `ElementRef<T>` is deliberately not yet declared, and no current consumer of `CompositionVerdict` exists to pin the graduation against. **Tracked as a consumer-driven follow-up for Stage 2b / Track 17a:** when the Stage 2b lens or the first Track 17a consumer needs to render "position of breaker in workflow" or structurally tie the verdict to its evidence chain, that is the moment to graduate `BreakingOperation` into `ElementRef<OperationEffect>` and ship the `ElementRef<T>` declaration alongside. Until then, the verdict's state-space is sound within itself (sum variants are internally coherent, `BreakingShape` can't admit idempotent shapes); only the verdict-to-evidence-chain tie remains copy-based.

Cleared (prior PR #521): `DerivedOpEffect { method, path_template, shape }` collapsed into `OperationEffect { operation_name, shape }`. The `method` / `path_template` fields were never consumed downstream — both the modifier check and obligation generator project through `shape` alone, and the `ReadEffect` variant already encodes "method was GET/HEAD/OPTIONS." `derive_op_effect` now returns `OperationEffect?` directly, so Stage 2b's `compose_effects` consumes the same shape derivation produces. Future diagnostic rendering that wants the originating method/path should attach a separate evidence carrier to the diagnostic, not smuggle transport facts onto the effect record.

### Lane 2 Stage 2d — symbolic cost (✅ Shipped)

**DB-7 symbolic-cost algebra and per-Behavior lens landed.** Authority: [`design-symbolic-cost-algebra.md`](docs/design-symbolic-cost-algebra.md). Three .dag files + a Rust mirror + acceptance tests:

- `src/v3/std/algebra.dag` — `SymbolicCost` coproduct (7 variants: Constant / Linear / Polynomial / Product / Sum / Log / Unknown) with a stamped 4-pattern dissolution receipt, `SizeVariable { source_port }`, `sequential` / `iterate` / `max_path` composition, `normalize` (drop zero, single-term reduction, `Linear(v) * Linear(v) → Polynomial(v, 2)` collapse), `dominates` partial order.
- `src/v3/std/dimensions.dag` — minimal `Dimension<Carrier>` + `Witness<Carrier>` types for Stage 2f's future generic walker. Full DB-3 abstraction (`analyze`, `DimensionReport`, bootstrap discovery) lands in 2f per its design doc's own sequencing.
- `src/v3/lenses/cost.dag` — per-Behavior lowering: Value → Constant(0); Transform → sequential(1, Σ inputs); Branch → sequential(1, condition, max_path(arms)); Loop → iterate(LinearCost(source), body); Bind → passthrough. Forward-fold accumulator pattern mirrors `lenses/complexity.dag`; `MissingCost` short-circuits every composition wrapper so malformed references never silently substitute a zero leaf.
- Rust mirror in `src/v3/compiler/src/dag.rs`: `SymbolicCost` / `SizeVariable` carriers and `sequential` / `iterate` / `max_path` / `normalize` / `dominates` functions — needed because `emit_rust_module`'s `is_bootstrap_file` filter excludes `src/v3/std/` declarations from Rust emission; same pattern `Behavior` / `LoopBound` use.
- `src/v3/compiler/src/lens_cost_symbolic_generated.rs` via `regen_lens_cost_symbolic` binary. Exposed as `v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostEntry, SymbolicCostLookup}`.
- Acceptance fixture: `src/v3/compiler/tests/lane2_stage_2d_symbolic_cost_test.rs` (20 tests). Covers Value/Transform/Branch/Loop lowering; recursive-fn body-cost fact-flow (PR #537 briansrls/codex BLOCKING); zero-drop normalization; `Linear(v) * Linear(v) → Polynomial(v, 2)` nested-fold fingerprint; cross-variable product stays `Product`; composite-dominance child-walk (PR #537 codex P2); `max_path` three-way step preserving incomparable branches (PR #537 briansrls BLOCKING) with an order-independence pin; dominance partial order (Unknown / Linear / Log / Polynomial degree ordering); `max_path` dominant-selection; checked-in generated-module snapshot guard.

**Loop cost for `LoopBound::Descent` clusters uses `LoopNode.source` as the size-variable carrier.** The cluster's `members: NonSingletonList<MemberDescent>` and `intra_cluster_calls` carry the descent witnesses needed for the termination proof (#519), but the loop's own `source` port is the runtime value being descended upon — which is the honest recursion-depth bound for both `Cardinality` and `Descent` bounds. Richer per-member analysis (distinguishing list-descent from bounded-integer descent so `Descent` clusters report `ConstantCost` for bounded-int descent instead of `LinearCost`) is a Stage 2d follow-up, flagged in DB-7 §"Recursion depth bounds".

**Substrate gap flagged for 2f pre-work: `data symbolic_cost_dimension: Dimension<SymbolicCost> = { ... }` is DEFERRED to Stage 2f.** v3's surface grammar rejects record literals inside `data X: T = { ... }` bodies (DOWNSTREAM_REQUIREMENTS.md class-5 gap #3) and record / match / lambda inside `fn X { body }` block-bodied definitions simultaneously, which both shapes the `Dimension` record's field-carrying receipt pattern requires. The lens ships its behavior-variant lowering as the authority in the interim; Stage 2f materializes the `Dimension<SymbolicCost>` instance alongside the grammar extensions that unlock it. Rationale + authority: DB-7 §"Dimension<SymbolicCost> wiring" documents the target shape; cost.dag's closing comment block records the grammar-gap waypoint.

**Build-script change (documented load-order fix).** `src/v3/compiler/build.rs` now prioritizes `list.dag` and `substrate.dag` ahead of alphabetical order within `STAGED_FILES`. Structural-recursion termination analysis walks a recursing argument back to its declared `Disj` connective via `structural_binding_info_for_variant`; the walk only succeeds after the declaring file has been phase-2 lowered, so std files that recursively descend over `List<T>` or `Behavior` variants (post-2d: `src/v3/std/algebra.dag` and `src/v3/lenses/cost.dag`) need their dependencies loaded first. Without the priority list, alphabetical order put `algebra.dag` and `dimensions.dag` ahead of `list.dag`/`substrate.dag`, and their recursive helpers failed termination against placeholder connectives. Same pattern the spec loader already uses to pin `v3_l1.dag` first.

**Acceptance-test infra migration (documented side effect).** Pre-existing tests that queried `dag.nodes().iter().find_map(Behavior::as_transform)` etc. on the `compile_to_dag` output assumed the bootstrap Dag contributed no `Transform` / `Branch` / `Loop` / `Bind` nodes before the user's source lowered. Lane 2 Stage 2d invalidates that assumption — `src/v3/std/algebra.dag`'s lowered bodies contribute many nodes. Eleven tests across four files migrated to filter by `span.file` (or subtract a bootstrap baseline) so they pin the user-code count/shape rather than the global node list: m0_acceptance's `test_let_binding_produces_dag_shape` (1); m1_substrate_test's `m17_operator_lowers_to_structural_transform_target`, `m17_comparison_operator_lowers_to_structural_transform_target`, `m17_user_function_call_lowers_to_callable_target`, `m18_r15_match_on_aliased_sum_type_compiles`, `m18_r13_mutual_recursion_poisons_callers`, `mutual_recursion_planner_ignores_callable_parameter_shadowing` (6); m1_3_emit_rust_test's three reflected-harness fixtures `rustc_roundtrip_emitted_module_matches_reflected_behavior_payloads`, `rustc_roundtrip_emitted_module_compares_reflected_port_ids_in_list_contains`, `rustc_roundtrip_emitted_module_returns_user_record_list_from_reflected_binds` (3); emit_rust.rs lib test `render_field_project_constructs_owned_list_from_borrowed_nodes` (1). This is a general test-authoring hygiene rule for every future std-module consumer to consider — a dedicated walker in Lane 1e would emit typed span-filtered iterators and dissolve the hand-written `.filter(|t| t.span.file == ...)` idiom.

**Follow-up — `ProductCost` / `SumCost` NSL lift (not blocking, post-normalize-≥2-invariant gates graduation).** `SymbolicCost::ProductCost(List<SymbolicCost>)` and `SymbolicCost::SumCost(List<SymbolicCost>)` admit `[]` and singleton inputs at construction time; `normalize` already reduces both back to scalar variants, so the post-normalize canonical shape has `len ≥ 2`. Lift the field type to `NonSingletonList<SymbolicCost>` (Track 9 vocabulary) so the `≥ 2 elements` fact becomes structural instead of a convention enforced only by `reduce_sum` / `reduce_product`. **Dissolution trigger:** the first call site that needs to pattern-match two guaranteed children without a `len() >= 2` check (likely a richer `normalize` variant or a multi-variable dominance rule). **Yellow-flag threshold: when a second normalization helper is added** — two such helpers is the moment the invariant becomes worth encoding at the type level.

**Follow-up — `PolynomialCost.degree` typed carrier (not blocking, degree-arithmetic surface gates graduation).** `PolynomialCost { var, degree: Int }` — post-normalize, `degree >= 2` (degree = 1 collapses to `Linear`, degree = 0 to `Constant`). The raw `Int` admits 0, 1, and negatives; the domain constraint is behavioral, not structural. Replace with a typed carrier (`NonNegativeInt` at minimum; `DegreeAtLeastTwo` ideally). **Dissolution trigger:** DB-7's degree-arithmetic surface lands (multi-variable polynomial composition like `Polynomial(n, 2) * Polynomial(m, 3) = Polynomial<mixed>`), which needs numeric operations on `degree` and is the natural moment to introduce the typed carrier. **Yellow-flag threshold: 1 month** after the first degree-arithmetic fixture lands on the lens side.

**Follow-up — `LinearCost(v)` vs `PolynomialCost(v, 1)` canonicalization (not blocking, normalization-step extension gates graduation).** The two variants encode the same asymptotic fact; `dominates` and `normalize` explicitly pattern-match on both (`payload.degree <= 1` in the Polynomial-vs-Linear arm). Representation duality (audit Q6). Pick one canonical form — either dissolve `LinearCost` into `PolynomialCost(_, 1)`, OR constrain `PolynomialCost.degree ≥ 2` so `Linear` is the sole degree-1 surface. **Dissolution trigger:** the same normalization extension that introduces the `DegreeAtLeastTwo` typed carrier above — both items collapse cleanly when normalization collapses Linear↔Polynomial(_, 1) as a structural invariant. **Yellow-flag threshold: graduates alongside the typed-degree follow-up** (coupled dissolution).

**Follow-up — `dominates` composite branches .dag↔Rust divergence (not blocking, cross-type mutual-recursion gates graduation).** Rust `src/v3/compiler/src/dag.rs::dominates` correctly walks children for `ProductCost` / `SumCost` (codex-flagged patch on this PR). The `.dag` authority `src/v3/std/algebra.dag::dominates` returns `False` conservatively for composites because #519's mutual-recursion termination analyzer only accepts cluster members sharing a descent parameter type — the list-iterating helper would descend on `List<SymbolicCost>` while `dominates` descends on `SymbolicCost`, so the cluster can't form. Rust composite-dominance is the authority; .dag is an over-approximation. **Dissolution trigger:** substrate extension letting a mutual-recursion cluster admit cross-type members with per-member structural-descent witnesses (list-head / list-tail descent alongside variant-payload descent). Likely a Lane 1e or later addition to the termination-proof surface. **Yellow-flag threshold: 2 months** or whenever a second .dag↔Rust divergence shows up — the divergence between authorities is only tolerable as long as it's isolated to this single call.

**Follow-up — `build.rs` load-order priority list bootstrap scaffold (not blocking, type-readiness signal gates graduation).** `src/v3/compiler/build.rs` prioritizes `list.dag` and `substrate.dag` ahead of alphabetical order so `structural_binding_info_for_variant` sees populated `Disj` connectives when downstream std files (Lane 2 Stage 2d's `algebra.dag` and the lens) are lowered. This is a bootstrap scaffold: the phase-order prerequisite lives in a filename priority list instead of flowing structurally from the declarations/import graph. **Dissolution trigger:** termination analysis grows a type-readiness signal (phase-1 declaration with fully-populated variant-list, rather than requiring phase-2-lowered body) so `structural_binding_info_for_variant` can succeed regardless of file order. When that lands, the priority list collapses to the empty default. **Yellow-flag threshold: whenever a third std file joins the priority list** — two entries is scaffold, three signals the rule is load-bearing enough to need structural lift.

**Follow-up — symbolic-cost lens fixed-point cycle coverage (not blocking, `compile_stage_snapshots` extension gates graduation).** `src/v3/compiler/src/lens_cost_symbolic_generated.rs` is currently out of the `l1_5_fixed_point_test.rs` / `compile_stage_snapshots` stage-replay loop — that loop iterates pipeline stages (`parse` → `lower` → `infer` → `compute_ownership` → `emit` → `lens_complexity`) but does not include `lens_cost_symbolic` as a stage. Staleness today is guarded only by `cost_generated_module_matches_checked_in_snapshot` in `tests/lane2_stage_2d_symbolic_cost_test.rs`, which re-emits the module and compares against the checked-in file. That's drift protection, but narrower than the fixed-point cycle Complexity / Provenance / Structural-Resolution / Unused-Parameters get. **Dissolution trigger:** first-class stage registration for symbolic cost in `pipeline.dag` (adds a `lens_cost_symbolic` stage + realization), mirroring the existing `lens_complexity` wiring. When that lands, the lens's generated file automatically participates in the fixed-point comparison. **Yellow-flag threshold: 1 month** or whenever a second Stage 2d lens wants regen coverage.

**Follow-up — hand-maintained `SymbolicCost` Rust mirror drift ratchet (not blocking, authority unification gates graduation).** PR #537 ChatGPT review (`sha:0f8c215c0`) call-out: the `SymbolicCost` / `SizeVariable` carriers + the composition functions (`sequential`, `iterate`, `max_path`, `normalize`, `dominates`, `reduce_sum`, `reduce_product`, `combine_binary_product`, `drop_dominated_in_sum`) are hand-maintained in `src/v3/compiler/src/dag.rs` because `emit_rust_module`'s `is_bootstrap_file` filter excludes `src/v3/std/` declarations from Rust emission. The scope is bounded (9 fns + 2 carrier types) and the current surface matches `src/v3/std/algebra.dag` modulo the documented composite-dominance gap (Follow-up above). The risk is future consumers attaching to the Rust mirror and not noticing when it drifts from the .dag authority — "declaration is the implementation" gets quietly replaced by "declaration + stronger hand-maintained Rust version" as the operating pattern. **Dissolution trigger:** the same substrate extension that closes the composite-dominance gap — once termination analysis admits cross-type mutual-recursion clusters, the .dag side matches Rust's richness and the hand-maintained mirror becomes `emit_rust_module`-generated like every other substrate type. Until then, a lightweight ratchet (e.g., a test that greps `src/v3/compiler/src/dag.rs` for fn signatures starting with `SymbolicCost::*` / `pub fn sequential|iterate|...` and cross-checks the count against `src/v3/std/algebra.dag` top-level `fn` decls) would catch silent expansion of the Rust surface. **Yellow-flag threshold: when a tenth hand-maintained fn is added to the mirror** — nine current fns is the baseline; a tenth without matching .dag growth is the signal that the scaffold is growing faster than the authority and needs explicit ratchet wiring.


### Lane 1 Stage 1b

**Deferral: 1b full implementation (M).** 1b's first attempt escalated (PR #495 shipped 1a; 1b code was reverted). Root cause: `.dag` linear-walk bodies for substrate accessors polluted every user DAG. DB-14 codifies the correct pattern (ExternalRealization mirroring pipeline.dag). Unblocked once DB-14 (PR #497) lands. Design: [design-substrate-external-primitives.md](docs/design-substrate-external-primitives.md) (DB-14). Acceptance in DB-14 §Acceptance.

### Lane 1 Stage 1d

**🟢 Design complete, pending P2-L1 sign-off.** [phase1-lane3-consolidation-build-plan.md](docs/phase1-lane3-consolidation-build-plan.md) is the authoritative design for how `emit_rust.rs` / `emit_go.rs` / `emit_python.rs` dissolve into one `emit.rs` walker + per-target specs. The plan locks: **walker contract** (§7 — five structural authorities per target + typed `SubstrateAccessorBinding`), **spec reading protocol** (§8 — Q5 compliant, zero name-keyed lookups; typed `DeclarationId` caches mirror the PR #528 `PatternBindingRuleVariants` pattern), **determinism plan** (§9 — invariant D-1, `BTreeMap` everywhere, structural grep gate), **migration plan** (§10 — prerequisite bridge **1e.0** Python schema migration + DB-2's six sub-stages 1e.1–1e.6 with definitions of done), **bootstrap-once + snapshot-ratchet** (§11 — mirrors PR #477 / #518 / #530 `lens_*_generated.rs` pattern), and **determinism test suite** (§12 — `tests/determinism_test.rs` per-fixture 5× re-run per DB-8). Companion inventory docs (PR #540): [`emit-functions-inventory.md`](docs/emit-functions-inventory.md) (92 functions classified), [`spec-field-gaps.md`](docs/spec-field-gaps.md) (P0/P1/P2-tagged gaps), [`emit-bridges.md`](docs/emit-bridges.md) (bridges B11–B19 with dissolution targets). Authority note: `src/v3/spec/<target>.dag` is the bootstrap-loaded staging fixture — the canonical home for Shape A language specs is `dsl/extdeps/languages/<target>/` per THESIS.md §"Bootstrap staging note." **Four of five §Acceptance gates met**; **P2-L1 owner sign-off** is the fifth and only remaining gate. **No new DB number allocated** — the plan extends DB-2 (walker API), DB-4 (clean-emission contract), and DB-8 (fixed-point ratchet) rather than introducing a novel structural decision. **Unblocks Stage 1e dispatch on P2-L1 sign-off.**

### Lane 1 Stage 1c

**Cleared this PR (PR 3 Python pilot):** `python_clean_emission: CleanEmissionContract` landed in `spec/python.dag` with `pattern_bindings = NotApplicablePatternBinding`. `emit_python::CleanEmissionContractBinding::build` reads the contract via the typed `PatternBindingRuleVariants` cache on `Dag` (Lane 1 Stage 1c PR 2.5) and rejects every variant except `NotApplicablePatternBinding`. `render_branch_body_expr` dispatches on the parsed binding and selects the substitute-at-render-time path — the emitter maps each payload-binding port to an extraction expression (`__match._0` / `__match`) inside `arm_locals`, so the source-level identifier never appears at a pattern site. Contract-shape generalized without modification: Python's rule is a legitimate variant of the existing `PatternBindingRule` disjunction, not a shape change. Targeted tests in `m1_4_emit_python_test` prove (a) unused bindings leak no identifier, (b) used bindings render via `__match._0` substitution, and (c) emitted Python passes `python3 -m py_compile` (ignored roundtrip matching the Rust/Go pilots).

**Cleared this PR (PR 4 post_emit_verifier CI gate):** Shared harness landed at `src/v3/compiler/src/post_emit_verifier.rs`. `parse_post_emit_verifier(dag, clean_emission_spec)` consumes all five `PostEmitVerifier` fields (`command`, `args`, `syntax_only`, `expected_exit_code`, `output_policy`) structurally — no hardcoded command strings; a new target only needs a `CleanEmissionContract` data item in its spec file. `run_post_emit_verifier(binding, source_path)` invokes `Command::new(binding.command).args(&binding.args).arg(source_path)` with cwd pinned to the source's parent so rustc / py_compile artifacts stay inside the caller's tmp dir, collects stdout/stderr, and applies `expected_exit_code` + `VerifierOutputPolicyBinding` as the verdict. Pilot roundtrips in `m1_3_emit_rust_test` / `m1_3_emit_go_test` / `m1_4_emit_python_test` now call the harness instead of hardcoding `rustc` / `gofmt` / `python3 -m py_compile` — each target's contract drives its own invocation. Umbrella narrowed: `m2_lens_unused_parameters_migration_test.rs` emits the wrapped module under `#[allow(warnings, clippy::all)] #[deny(unused_variables)]` — the paired deny overrides the warnings group for this specific lint and turns any regression in the three pilots from a silent warning into a rustc error. Remaining follow-up (tracked separately when needed): un-`#[ignore]`'ing the harness roundtrips is a CI-infrastructure concern (verifier binaries available on runners), not rule-dispatch correctness.

### Lane 1 Stage 1d

**Cleared on the current branch (pending merge):** `docs/phase1-lane3-consolidation-build-plan.md` is now the post-pilot Stage 1d authority rather than a pre-pilot sketch. The revision explicitly stamps what survived the Rust/Go/Python pilots (typed `language` filtering, template substitution, clean-emission dispatch, fail-closed unsupported behaviors) and what had to be reshaped before Stage 1e started (Python proves `variant_payload_field_access` is load-bearing alongside `PatternBindingRule`; the first shared entrypoint can land as a one-target scaffold; Python's target-private realization family remains real debt). Stage 1d artifacts (`emit-functions-inventory.md`, `spec-field-gaps.md`, `emit-bridges.md`) were updated to describe the live shared-entrypoint state rather than the pre-scaffold file layout.

### Lane 1 Stage 1e

**In progress on the current branch (pending merge):** `src/v3/compiler/src/emit.rs` now exists as the shared emit entrypoint with `EmitTarget`, `EmitMode`, `EmittedSource`, `emit(...)`, and `emit_module(...)`. The first migrated target is **Go**: its hand-written renderer moved under `emit.rs`, and `src/v3/compiler/src/emit_go.rs` is reduced to a compatibility adapter that forwards into the shared path. Coverage is locked by `m1_3_emit_go_test` plus `emit::tests::*`, including a wrapper-parity test proving `emit_go` / `emit_go_module` are thin adapters over the shared entrypoint. Rust and Python remain on legacy drivers; full recursive walker unification is still the remaining Stage 1e work.

### Cross-cutting — performance

**Deferral: self-compile perf ratchet investigation (M, not on any critical path but compounding).** Self-compile time drifted from ~60s to ~70s in recent cycles (~16% growth). The ratchet keeps getting bumped without a root-cause investigation; each bump normalizes the regression. Scope: (1) profile a single `cargo test -p v3-compiler-tests` run, identify the top hot paths; (2) measure where the 10s came from across recent PRs (bisect across #479, #489, #490 if signal is unclear); (3) either fix the regression or document it as an accepted cost with a new ratchet ceiling. **Yellow-flag threshold: 90s.** If self-compile exceeds that before this deferral is scheduled, it preempts other work. No design doc needed; profiling is a data-gathering exercise.

### Cross-cutting — workflow scripts modeled in .dag

**Deferral: model the commit pipeline in .dag (XL, thesis-coherence, ACTIVE).** This is active now, not "awaiting a second instance." Four hand-written workflow scripts already exist — `.githooks/pre-push` (PRs #503 + #509), `scripts/install-hooks.sh`, `scripts/check-stage0-freshness.sh`, `scripts/regenerate-stage0.sh` — and THESIS.md's meta-process claim already says bootstrap, CI, and dev-workflow should be modeled as `.dag` programs. The tolerated-until-second-instance framing in an earlier draft understated this.

**Pre-push hook as a working example, not a load-bearing contract.** The hand-written hook at PR #509 HEAD fmt-checks, optionally fmt-fixes + auto-commits, and signals the push outcome. A `.dag`-modeled version would inherit the same behavior and add the stdin/delete/HEAD-in-push handling as declared structure — but that full contract belongs in the `.dag` design work, not in the ROADMAP entry as if it were the existing hook's shape.

**Shape A vs Shape B — Shape B.** Per ROADMAP Track 16 (`ROADMAP.md:920-935`), the compiler emits real programming languages (Rust, Go, Python) as Shape A; non-program artifacts (YAML, shell scripts) are Shape B — produced by `.dag` programs via `concat`/`fold`/`match` over structured values. The pre-push hook is Shape B. A `.dag` program walks a `ShellScript` or `HookDefinition` value, constructs the script text, and writes it via `shell.Exec.Run` (or analogous). No compiler-target surface growth; same pattern as `tools/ratchet.dag`'s grep-command generation and Track 16's CI YAML.

**Existing substrate consumed:**
- `dsl/extdeps/git.dag` — `service git.Core` declares `CurrentBranch`, `RemoteBranches`, `LsFiles`, `Diff`, `RevList`, `Show`. Needs extensions for `Commit`/`Push`/`Add`/`StatusClean` (if absent) plus stdin-as-input for pre-push's ref list.
- `dsl/extdeps/cargo.dag` — `service cargo.Build` has `Build`/`Test`/`Clippy`/`Doc`/`Run`. **Missing `Fmt` operation (check + apply).** Small S extension.
- `dsl/extdeps/shell.dag` — `Find`, `Env`, `Which`, `Exec` — adequate for shell primitives the hook needs.
- `dsl/extdeps/github/` — GitHub-specific (not on critical path for pre-push).

**Separable prerequisite deferrals:**
- **`cargo.dag` Fmt operations (S).** Add `operation FmtCheck` / `operation FmtApply` to `service cargo.Build`. Mechanical.
- **Track 15 tool resolution (prerequisite, already tracked as M5 Phase 2 / Track 15).** Every shell-out to `cargo`, `git`, etc. today uses bare command names that depend on PATH. Track 15 exists specifically to replace PATH-based resolution with explicit `Tool { path, version, ... }` lookups; `shell.Which.Check` already exists with zero consumers. The pre-push-hook-in-.dag work must use Track 15's resolution model — not add new bare-command-name call sites. Without this, the emitted hook preserves the hidden-PATH-dependency debt the roadmap already flagged.
- **Shape B emission pattern, no new compiler target.** Per Track 16: `.dag` programs build shell scripts via data manipulation; interpreter runs the program; program writes the file. No substrate amendment, no DB for "shell emission target" — that was a misread of my earlier draft.
- **Hook invocation contract as structural declaration.** The pre-push contract (stdin format, exit codes, the four decision cases) needs a structural shape — probably a small type like `type PrePushHook { read_stdin: ..., decide: ..., emit_result: ... }` authored in a shared location that future hook generators consume. Design work, but lives in a `.dag` program, not a compiler feature. Sizeable because the shape has to generalize across hook kinds (pre-push, pre-commit, post-receive, etc.) or explicitly say it's pre-push-only and future hooks get their own types.
- **Test coverage via DB-15 R2.** `MockBackedInvariant` predicates test the compiled hook against: delete push, HEAD push with drift, cross-branch push, clean push. Validates the Shape B emission pipeline end-to-end.

**Dissolution sequence:**
1. Track 15 tool resolution wired (if not already; see Track 15 entry for current state).
2. `cargo.dag` Fmt operations land (S, mechanical).
3. `scripts/pre-push-hook.dag` declares the workflow as a Shape B program — walks a `PrePushHook` value, builds the script via `concat`/`fold`/`match`, writes via `shell.Exec.Run`.
4. Build invokes the `.dag` program to produce `.githooks/pre-push` (or install-hooks.sh does).
5. Hand-written `.githooks/pre-push` (PRs #503 + #509) goes in §Scheduled deletions with trigger "emitted pre-push hook replaces it."
6. DB-15 R2 test suite verifies behavior across the four scenarios.

**Scope for the other three hand-written scripts.** `install-hooks.sh`, `check-stage0-freshness.sh`, `regenerate-stage0.sh` follow the same pattern — Shape B `.dag` programs emitting shell text. Each gets its own dissolution PR once the pre-push case proves the pattern.

**No design doc committed yet.** This deferral tracks the structural ordering (Track 15 → cargo.Fmt → pre-push-hook.dag → dissolve hand-written). Formal DB lands when someone starts the `pre-push-hook.dag` work and needs to pin down the `PrePushHook` type shape.

### Phase-plan migration candidates (pointer)

Items awaiting director pre-clearance (before they graduate into **scoped deferrals** above) are listed **only** in [`docs/phase-plan-2026-04-18.md`](docs/phase-plan-2026-04-18.md) §5b — do not duplicate or hand-sync bullets here.

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

## Scheduled deletions — scaffolds with named dissolution triggers

**Discipline:** every scaffold in the live substrate lives here with an explicit dissolution trigger, the upstream work it's blocked on, and its enforcement path. When the trigger fires, a PR deletes the scaffold AND removes the row. Unscheduled scaffolds are violations of the scaffold-boundary invariant.

**Relationship to "Active deferrals":** deferrals name work that is in flight; scheduled deletions name artifacts that will disappear. A deferral may REFERENCE a scheduled deletion (e.g., "this sub-stage dissolves `ArrowBody::Pending` per the scheduled-deletions row"), and a scheduled deletion may reference a design blocker (DB-NN) that unlocks structural enforcement.

### Enforcement paths — three kinds

Each scheduled deletion names one enforcement path. **Grep over source code is not an enforcement path** — see §"Grep is not an enforcement path" below.

1. **Structural lens (preferred).** The substrate already carries the fact; a lens walks user DAGs and reports instances. Example: `ArrowBody::Pending` is a substrate variant; a lens walks `d.nodes` and fires on any `Pending` body reachable from user-range roots. Fits the existing `lens_unused_parameters.dag` / `lens_provenance.dag` shape. Writable today.

2. **Needs substrate amendment (DB-NN).** The substrate does not yet expose the fact the lens would need. A design blocker proposes the amendment; the lens becomes writable after the DB lands. The scheduled-deletion row carries a `Needs DB-NN` enforcement marker until the DB lands, then flips to a live lens path.

3. **Compiler-source ratchet (temporary).** For scaffolds that live in the hand-written Rust compiler and can't be lensed until `compiler.dag` self-hosts, a narrow source-level ratchet scoped to `src/v3/compiler/` is acceptable as temporary enforcement. Dissolves automatically when `compiler.dag` self-hosts and the same lens can walk the compiler's own DAG. The row explicitly marks this as temporary.

### Table

| Scaffold | Dissolution trigger | Upstream blocker | Enforcement |
|---|---|---|---|
| `ArrowBody::Pending` | M3 ratchet | Every realization arrow bound to `ExternalRealization` | **Lens** — writable now; walk `d.nodes` for Arrow declarations with `body = Pending` reachable from user-range roots |
| `ArrowBody::Unparsed` (**case 1** — `FnExternalBody` parse lag in std/) | M2+ parser surface | Match / pipe / lambda / block-body parsing so `FnExternalBody` lowers away | **Lens** — user-range + applicable std/ per R14; ratchet fires when block bodies become `SurfaceExpr` |
| `ArrowBody::Unparsed` (**case 2c** — `pipeline.dag` `fn compile` ordering text) | Structural pipeline-order carrier | First-class ordered stage list (or successor substrate) supersedes `compile` body-span parsing | **Not** the M2 grammar milestone — dissolution per [design-fn-external-body-reconciliation.md](docs/design-fn-external-body-reconciliation.md) case 2c; `pipeline_compile_order_stage_names` is the reader today |
| `ArrowBody::Unparsed` (**DB-14 accessor interim**, pre–E-9) | E-9 bootstrap materialization | **Deferral: E-9 substrate accessor bootstrap rewrite** below — `ExternalRealization(marker)` on `Arrow.body` | Clears with that deferral (not the case-1 lens) |
| `ValueBody::Unparsed` | M2+ parser surface | Record / map / list literal parsing | **Lens** — writable now; walk `data` declarations |
| `TransformTarget::Operator` | M2+ parser surface | Operator desugar into algebra-field calls | **Lens** — writable now; walk Transform targets |
| User-range `ResolvedByName` AtomPayload (DB-17 new variant — post-landing, any user-range reference produced via name fallback rather than structural walk) | M2 module scoping | Cross-module structural resolution | **Needs DB-17** (reference-resolution provenance) — once DB-17 lands, lens walks `d.nodes` AtomPayloads for `ResolvedByName` reachable from user-range roots |
| Compiler-internal `declaration_by_name` call sites (bootstrap `substrate_markers` initialization in `dag.rs:1616+`, pipeline-authority wiring in `bootstrap.rs`/`pipeline_authority.rs`, emitter algebra lookups in `emit_go.rs`/`emit_python.rs`) | Self-hosting (most cases) OR specific per-site substrate amendments (e.g., substrate_markers becoming typed edges) | Depends on class — self-hosting for emitter/pipeline sites, specific substrate amendment for marker caches | **Compiler-source ratchet** (temporary, dissolves at self-hosting for most sites) — these are compiler-internal caches/wiring, NOT user-range resolution fallbacks; **DB-17 does not cover them** |
| `Node.name` field (v3 substrate) | `authored_name_at` cross-module span fix + 15 direct reads migrated | Cross-module span resolution via DeclarationId | **Compiler-source ratchet** (temporary, dissolves at self-hosting) |
| `encoding_meet` / `encoding_join` (Rust fns) | Track 8 Phase 2 (user-defined generic emission) | User-defined generic emission for `Lattice<Encoding>` instance | **Compiler-source ratchet** (temporary; becomes lens-able when compiler.dag self-hosts and emission-generated code replaces these hand-written fns) |

### Notes on specific rows

- **`ArrowBody::Unparsed` is three dissolution stories, not one.** DB-16 / PR #524: **case 1** (parse lag, M2 grammar, lens ratchet) is separate from **`pipeline.dag` `compile` (case 2c)** — ordering text read by `pipeline_authority` until a structural pipeline-order fact supersedes span extraction — and from **DB-14 accessors** (interim `Unparsed` until the **E-9** deferral lands). The M2 milestone deletes case-1 uses; it does **not** by itself delete `compile`’s span authority or accessor interim encoding.
- **`declaration_by_name` is a helper name, not a single debt class.** The function at `dag.rs:1459` has 83 call sites that split into distinct classes with separate dissolution paths. [DB-17 (reference-resolution provenance)](docs/design-reference-resolution-provenance.md) narrows its scope to **only the user-range AtomPayload fallback class** (lowering produces `ResolvedByName(id)` when a structural walk falls back to name lookup). DB-17's lens walks user-range AtomPayloads; compiler-internal call sites (bootstrap substrate_markers in `dag.rs`, pipeline authority wiring, emitter algebra lookups in `emit_go.rs`/`emit_python.rs`) are a separate compiler-source class that dissolves at self-hosting (or via per-site substrate amendments — e.g., substrate_markers becoming typed edges rather than name-keyed caches). Keying the scheduled deletion to the helper name conflates these.
- **`Node.name` cluster**: 15 direct reads audited in the Node-to-std migration project; each has a replacement via structural edge (declaration lookup with structural path). Compiler-source ratchet suffices until self-hosting because the enforcement surface is one directory (`src/v3/compiler/`) and audit cadence catches drift.
- **`keyword_to_name` (recon outcome 2026-04-17, no row added):** the bare `keyword_to_name` was renamed to `tok_keyword_to_name` during v2 Phase 0 parser restructure — see `src/v2/parser-design.md:403-408`. The new name still carries the scaffold (parser-side keyword-name logic that duplicates facts from the tokenizer's `SyntaxSpec`), but it lives in `src/v2/02_parse.dag:455` and `src/v2/stage0/src/v2_compiler_parse.rs:1321` — **v2 code**, not v3. Grep confirms zero equivalents in `src/v3/`. V2 is the reference-implementation / test oracle per `ROADMAP.md` §"Sketch vs Oracle framing"; v2 scaffolds dissolve when v3 supersedes v2 entirely, not individually. The v3 Scheduled Deletions list tracks v3-scope scaffolds only.

### Grep is not an enforcement path

Per the compiler-as-dependency-analyzer framing: grep over source text cannot distinguish a real user-range violation from a comment, test fixture, bootstrap path, alias, helper indirection, or trait-dispatched call. It matches strings; the compiler analyzes a graph. Using grep to enforce "the system should be structural" uses a heuristic to enforce the ban on heuristics — the discipline defeats itself on the first move.

Every time a grep gate is proposed over source code, the correct question is: **what substrate fact would make this a lens?** That fact might need a DB; if so, the grep is a signal for the DB, not a substitute for it.

**Narrow exception:** the banked-dissolutions ratchet in `docs/post-l15-phase-plan.md` operates on *documentation text* (lane docs can't restate DB-rejected shapes), not on system behavior. Docs don't have a resolved DAG; they're text. Grep over docs for design-consistency is legitimate. System-level scaffolds get structural enforcement.

### How the scheduled-deletions discipline works

1. **Adding a scaffold.** The PR that introduces a scaffold opens a row here with: scaffold name (file:line or type path), dissolution trigger, upstream blocker, enforcement path (one of the three kinds above).
2. **Enforcement-path classification happens in the same PR.** Structural lens → write the lens or file `lens_TBD` naming the fact to query. Needs DB → file the DB design doc (or reference an existing one). Compiler-source ratchet → explicit; dissolves at self-hosting.
3. **Dissolution.** When the trigger fires, a PR deletes the scaffold AND removes the row. No lingering row after deletion; audit-traceable via git history.
4. **Reviewer gate.** A PR that introduces a scaffold without a row here — or with an enforcement path classified as "grep source code" — is blocked.

## What NOT to build yet

- **Any fourth per-language emit file** (e.g., `emit_verilog.rs`,
  `emit_spice.rs`). Defer all new emit targets until P2 consolidation
  lands — each additional `emit_X.rs` makes the consolidation
  proportionally harder. Covered in [post-l15-phase-plan.md](docs/post-l15-phase-plan.md) §"What NOT to do".
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
