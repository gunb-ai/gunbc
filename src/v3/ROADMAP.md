# v3 Roadmap

Single source of truth for v3 status, active work, and deferred items.
Supersedes `src/v3/M1_FOLLOWUPS.md` (now a stub redirect). Historical
M1(2.5) task list lives in `src/v3/M1_TASKS.md`; design oracle in
`src/v3/M1_DESIGN.md`; M0 retrospective in `src/v3/M0_RETROSPECTIVE.md`.
Substrate-consumer gap enumeration in
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
| **M1(2.7)** Enumeration-driven substrate fix | ✅ Landed on PR #445 | Enumeration pass produced `DOWNSTREAM_REQUIREMENTS.md` (14 gaps across 4 classes); fix PR resolved all gaps structurally. Primitive identity cache (`Dag::int_shape` / `bool_shape` / `string_shape` / `realization_meta_id`); `TransformTarget { Callable, Operator }` + `OperatorKind { Arithmetic, Comparison }` coproducts; `ArrowBody::Unparsed(SourceSpan)` for block-body scaffolds; `SurfaceItem::{Fn, FnExternalBody, Data, Module, Import}` split so parser-absorbed items become real facts that flow forward; `TemplateArgument` stub self-reference branch deleted. **Current: 41 M0 + 11 M1 substrate + 7 real-stdlib parse smoke + 1 realization smoke = 60 green.** |
| **M1(3)** Cost lens | ⏸ Deferred | First writer lens. Forces the lens storage decision. |
| **M1(4)** Rust emitter | ⏸ Deferred | Single target. |
| **M2** Feature parity | ⏸ Deferred | Generics in user code, match in user code, transport declarations, interpreter, recursion → Loop. |
| **M3** Self-hosting | ⏸ Deferred | `.dag` rewrite of the compiler. |
| **M4** Thesis completion | ⏸ Deferred | All lenses, verification, omni-emission. |

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
the C1-class stop signal in `M1_DESIGN.md` §8.10.

## M0 — Skeleton (complete)

See `M0_RETROSPECTIVE.md` for the full retrospective. Highlights:
- Five L1 behaviors survived 10 milestones + 3 reviewer rounds.
- `PortState::{Uninferred, Resolved, Unresolved}` became structural
  after the M0.6 refactor; the fail-closed biconditional holds by
  construction.
- Spans live on every Behavior structurally, not in a side table.
- Declaration of terminal status: five behaviors, documented. Adding
  a 6th triggers the C1 stop signal.

## M1(2.5) — Substrate rework (in PR #445)

See `M1_TASKS.md` for the implementer checklist and `M1_DESIGN.md` for
the design oracle. Substrate changes landed:
- `TypeConnective` six-variant enum
- `Declaration` struct with canonical `type_params`, separate `meta_tag`
  and `inhabits` edges
- `ArrowBody { UserDefined, ExternalRealization, Pending }`
- Two-pass lowering with `resolve_pending_identifiers` post-sweep
  (fail-closed for unresolved identifier stubs)
- `build_template_arguments` for fail-closed template arity check
- Dissolution ledger receipts in `dag.rs` (mirrors `M1_DESIGN.md` §Q7)
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

### The one remaining scaffold

After M1(2.7), the only substrate scaffold is `ArrowBody::Pending`
(realization lag) and `ArrowBody::Unparsed(SourceSpan)` (block-body
lag). Both have named dissolution triggers:

- **`Pending`** dissolves via the §8.11 monotonic-decrease ratchet
  when every realization arrow binds to a real `ExternalRealization`
  declaration (M3).
- **`Unparsed`** dissolves when the M2+ surface grammar adopts
  match/pipe/lambda/etc. so block bodies lower to full
  `UserDefined(NodeId)` arrows.

Both are tracked and non-spreading. The `OPERATOR_FIELD_MAP`
bridge is gone entirely; operator dispatch is fully structural.

## M1(3)+ — deferred work (unchanged since PR #441)

- **(3) Cost lens** — first writer lens. Forces the lens-storage
  decision (Node field vs side table vs computed fresh). 2–4 days for
  the first cut. Until this lands, the v3 vs v2 performance proof of
  concept is not exercised.
- **(4) Success bar validated for writer lenses.** By the end of cost
  lens work, "minimum substrate change for a new lens" should be
  zero. Gating acceptance for emission work.
- **(5) Ownership lens** — second writer lens. Reuses the cost lens'
  storage mechanism; if it doesn't generalize, that's a signal.
- **(6) Emit Rust from DAG** — single target, minimal. Only after cost
  + ownership prove extensibility.
- **§8.11 Pending ratchet** — CI count of `ArrowBody::Pending`
  declarations; monotonic decrease through M3. Doc-only today. CI
  wiring is M1(3) work.
- **`ArrowBody::Pending` removal** — once the ratchet hits zero by M3,
  delete the variant.

## M2 — Feature parity with v2 subset (deferred)

- Generics in user code
- `Optional` / `Cardinality` user-code surface
- Service calls (transport declarations)
- Pattern matching in user-code fn bodies (Branch with destructuring)
- Interpreter (`dag run`)
- Recursive functions → `Loop` lowering
- Match expressions / pipe / lambda / named arguments in user code
- `data` value semantics
- `where` refinement checking
- `TypeShape` → `DeclarationId` migration

## M3 — Self-hosting (deferred)

- v3 compiles itself
- Bootstrap: v2 compiles v3 stage0, v3 compiles v3 → fixed point
- All v2 test programs compile under v3 with same output
- `OPERATOR_FIELD_MAP` and the port-type whitelist already
  dissolved at M1(2.7); no carried-forward bridges remain at M3.

## M4 — Thesis completion (deferred)

- All lenses operational (cost, ownership, effects, termination,
  algebra, space)
- Diagnostics as corrections (Level 1–2)
- L4 verification: emitted code matches DAG evaluation
- User-defined observational lenses
- Omni-emission projection rules

## What NOT to build yet

- Generic dimension mechanism (user-defined optimization lenses)
- Multi-target emission (start with Rust only)
- Advanced diagnostics (Level 3 auto-fix)
- Async/concurrent emission strategies

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
