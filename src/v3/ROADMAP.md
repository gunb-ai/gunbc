# v3 Roadmap

Single source of truth for v3 status, active work, and deferred items.
Supersedes `src/v3/M1_FOLLOWUPS.md` (now a stub redirect). Historical
M1(2.5) task list lives in `src/v3/M1_TASKS.md`; design oracle in
`src/v3/M1_DESIGN.md`; M0 retrospective in `src/v3/M0_RETROSPECTIVE.md`.

> Design spec: [docs/v3-spec.md](../../docs/v3-spec.md)
> Validation: [docs/v3-validation-experiments.md](../../docs/v3-validation-experiments.md)
> Lineage: [docs/design-lineage.md](../../docs/design-lineage.md)

## Status at a glance

| Milestone | State | Notes |
|-----------|-------|-------|
| **M0** Skeleton | ✅ Complete | 40 acceptance tests green. PR #441 merged. M0_RETROSPECTIVE.md closes it. |
| **M1(2.5)** Substrate rework | ⏳ In review | PR #445 open. 40 M0 + 3 M1 substrate = 43 green. Substrate = six-variant `TypeConnective`, Declaration table, `meta_tag`/`inhabits` split, `ArrowBody` with `Pending` scaffold. |
| **M1(2.6)** FACTS FLOW + SINGLE AUTHORITY | 🚧 Active | Rolled into PR #445 per Option C from the ChatGPT review. Parser extensions, real `dsl/std/*.dag` consumption, SubstStack + §8.9 inhabitance walks, delete `inject_primitive_operators`. |
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

### The one remaining bridge

After Option C, the only name-based bridge in the compiler is:
1. `OPERATOR_FIELD_MAP` in `infer.rs` — maps `"+"` → `"add"`, `"-"` →
   `"sub"`, etc. Localized constant, used only by `resolve_arrow`
   during §8.9 walks. Dissolves in M2+ once the surface grammar
   exposes algebra field access directly (e.g., `Int.add(a, b)`).
2. `declaration_to_type_shape` structural walk — maps `Int`/`Bool`/
   `String`/`Word64`/`Classical`/... named root declarations to
   `TypeShape::Primitive`. Dissolves in M2 when `TypeShape` itself
   becomes `DeclarationId`-carrying.

Both are documented and non-spreading; each has a clear dissolution
trigger tied to a future milestone.

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
- Delete `OPERATOR_FIELD_MAP` and `declaration_to_type_shape`
  walk-based bridge (both dissolve when surface grammar / port types
  do)

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
