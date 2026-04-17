> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 1 Stage 1b (substrate keyed-lookup)

# Design DB-14 — Substrate external primitives

**Design blocker:** DB-14 (substrate primitives backed by target-native implementations)
**Consumer:** Lane 1 Stage 1b (substrate keyed-lookup accessors: `port`, `node`, `resolve_producer`)
**Status:** Landed on PR #501 (commits 0c07e9a5 and follow-up). This doc reflects the shipped design after review rounds 1b.3 (target selector) and 1b.4 (fail-closed coverage check).
**Origin:** Lane 1 Stage 1b escalation (2026-04-17) — the session's initial attempt declared `.dag` linear-walk bodies for the accessors and hit bootstrap pollution (recursive bodies lowered into every user program's `dag.nodes()`). An interim design used bootstrap's `ArrowBody::ExternalRealization` upgrade pattern from `pipeline.dag`; review 1b.3 flagged that pattern as unsafe under target variation, and the landed design moved dispatch to emission time.

---

## TL;DR

**Two separate dispatch patterns for Rust-backed primitives:**

1. **`pipeline.dag` pattern (target-invariant primitives):** bootstrap upgrades each stage's Arrow body to `ArrowBody::ExternalRealization(realization_id)`. Correct because pipeline stages run IN the compiler and have exactly one realization per stage.

2. **`substrate.dag` pattern (target-variant primitives):** Arrow bodies stay `Unparsed`. Each emitter builds a per-target `SubstrateAccessorBinding` index at emission time, filtered by `language == <its own LanguageSpec>`, and dispatches through the index. Correct because substrate accessors are CALLED FROM emitted code and need different realizations per target (Rust's `Dag::port_opt` ≠ Python's / Go's).

The substrate-accessor pattern uses `ArrowBody::ExternalRealization` as a SHAPE FOR INFERENCE but never physically upgrades to it — inference treats `Unparsed` as "signature-only, body not walked" anyway, so the net behavior at the type layer is equivalent. The structural difference is where target selection lives: bootstrap (one truth) vs. emission (per-target).

**No new substrate concept. No new TransformTarget variant. No coproduct extension.** `SubstrateAccessorBinding { accessor, realization, language }` is just a data record tagged with a meta-type — the same shape every `TypeRealization` / `OperatorRealization` / etc. already uses.

---

## Problem

A compiler-internal function whose authoritative implementation is Rust (because of performance, because the implementation is a HashMap, because it's a substrate primitive) cannot be declared in `src/v3/std/*.dag` with a `.dag` body:

- Bootstrap loads every `.dag` file in the std/spec/compiler set.
- Every fn declaration's body lowers to a sub-DAG of `Transform` / `Bind` / etc. nodes.
- Those nodes end up in `dag.nodes()` before the user's code is lowered.
- Tests that use "first Callable Transform" heuristics (and, more broadly, any consumer that assumes the program's DAG is just the user's) break.
- Worse: the `.dag` body IS a wrong implementation. Writing a linear walk for `port(d, id)` in `.dag` is O(n) while the Rust side has an O(1) HashMap. The `.dag` body is a lie about what the compiler actually runs.

Lane 1 Stage 1b's original design called for declaring `port(d, id) -> DagPort?`, `node(d, id) -> Behavior?`, `resolve_producer(d, id) -> Behavior?` in `substrate.dag` with full `.dag` bodies. The session's implementation correctly compiled but polluted bootstrap and was reverted.

---

## Key finding: the mechanism exists (for pipeline, with a divergence for substrate accessors)

### `ArrowBody::ExternalRealization` (substrate, `dag.rs`)

> "Primitive whose realization is declared in an extdeps language spec. DeclarationId points at the realization declaration via a typed edge; inference verifies signature compatibility."

This variant is used by `pipeline.dag` — each stage's Arrow body gets upgraded at bootstrap to `ExternalRealization(realization_id)`. Correct for pipeline because there's exactly one realization per stage (pipeline stages are target-invariant compiler runtime).

### Inference wiring (`infer.rs`)

Inference treats both `Unparsed` and `ExternalRealization` as "signature-only, body not walked." Both variants type-check call sites against the Arrow's declared inputs and output. That's why the substrate accessors can leave bodies as `Unparsed` without losing inference coverage.

### Where the substrate-accessor pattern diverges

Review round 1b.3 caught that applying pipeline.dag's bootstrap-upgrade pattern to **target-variant** primitives silently drops target selection: bootstrap walks every `SubstrateAccessorBinding`, upgrades the accessor's Arrow body once, and iteration order decides which target's realization wins. As soon as a second backend lands its binding, the first is overwritten.

The fix is structural, not conventional: make target selection part of the binding record (`language: DeclarationRef`) and move dispatch to emission time, where each emitter already knows which language spec is active.

### Production template: `src/v3/compiler/pipeline.dag` (unchanged)

Pipeline.dag still uses the upgrade pattern exactly as before — its stages are target-invariant, so one-realization-per-stage is the right authority.

```dag
type CompilerHostRealization { symbol: String }
data parse_realization: CompilerHostRealization = { symbol: "v3_compiler::parse" }
fn parse(source: String, file: String) -> Dag { host parse }  // stub
```

Bootstrap (`materialize_pipeline_realizations`) upgrades each stage:

```rust
TypeConnective::Arrow { body, .. } => *body = ArrowBody::ExternalRealization(stage.realization),
```

---

## The real gap (now closed): per-target emission dispatch

For substrate accessors called from **user-code lenses** (`complexity.dag` calling `port(d, id)`), the emitter needs to:

1. Recognize this callable target is a substrate accessor.
2. Select the realization matching the active target (Rust emits Rust's realization, not Go's).
3. Render its `carrier` template.
4. Fail closed if the accessor is declared but no binding exists for the active target.

The landed design wires (1)-(4) in emit_rust's `RealizationIndexes` + `render_substrate_accessor`.

---

## Design

### 1. New realization type for substrate accessors

Analog of `CompilerHostRealization`, but instead of carrying a runtime symbol it carries a per-target rendering template:

```dag
// src/v3/std/substrate.dag  (or a companion file)
type SubstrateAccessorRealization {
  carrier: String
}
```

`carrier` is a template string with `{arg_name}` placeholders matching the Arrow's parameter names. The emitter substitutes per-target.

### 2. Declare the accessors in substrate.dag

```dag
// src/v3/std/substrate.dag
fn port(d: Dag, id: PortId) -> DagPort? = ...            // trivial stub body
fn node(d: Dag, id: NodeId) -> Behavior? = ...           // trivial stub body
fn resolve_producer(d: Dag, port_id: PortId) -> Behavior? = ...  // trivial stub body
```

**Stub body shape:** mirror pipeline.dag's pattern. Literal return of the output type's default (e.g., `None` for `DagPort?`). The stub's only purpose is to make the file parse; the body is replaced at bootstrap.

### 3. Declare realizations per target

```dag
// src/v3/spec/rust.dag
data rust_port_accessor: SubstrateAccessorRealization = {
  carrier: "{d}.port_opt({id}).cloned()"
}

data rust_node_accessor: SubstrateAccessorRealization = {
  carrier: "{d}.node_opt({id}).cloned()"
}

data rust_resolve_producer_accessor: SubstrateAccessorRealization = {
  carrier: "{d}.resolve_producer({port_id})"
}
```

Similar `go_*_accessor` in `go.dag` and `python_*_accessor` in `python.dag` with target-native syntax.

### 4. Binding declarations — with target selector (`language: DeclarationRef`)

```dag
// src/v3/std/substrate.dag
type SubstrateAccessorBinding {
  accessor: DeclarationRef
  realization: DeclarationRef
  language: DeclarationRef   // review round 1b.3 — required target selector
}

// src/v3/spec/rust.dag
data port_binding_rust: SubstrateAccessorBinding = {
  accessor: port
  realization: rust_port_accessor
  language: rust_language
}
// ... etc. for each (accessor × target) pair
```

One binding per (accessor × target language). 3 accessors × 3 current targets = 9 binding declarations when go/python land, mechanical. `language` mirrors the existing `language: DeclarationRef` edge on every shared realization record (`TypeRealization`, `OperatorRealization`, …), so "add a new target" stays one spec file change.

### 5. Bootstrap: NO upgrade for substrate accessors

**This is where substrate accessors diverge from pipeline.dag.** The earlier design (round 1b.1) proposed an `upgrade_substrate_accessor_bodies` pass mirroring `materialize_pipeline_realizations`. Review round 1b.3 flagged that: it walks all bindings and overwrites the accessor's Arrow body in iteration order, silently dropping all but the last target. The landed design keeps the bodies as `Unparsed(body_span)` (the `{ host <name> }` stub) and does target selection at emission time.

Rationale: pipeline stages run IN the compiler and have exactly one realization per stage; bootstrap-upgrade fits. Substrate accessors are called FROM emitted code and have a realization per target; the dispatch authority lives with the emitter, not with the single bootstrap pass.

### 6. Emission dispatch — per-target binding index + fail-closed coverage check

Each emitter builds two structures at start:

- `substrate_accessors: HashMap<accessor_decl, realization_decl>` — per-target map, filtered by `language == <this emitter's LanguageSpec>`.
- `substrate_accessor_universe: HashSet<accessor_decl>` — every accessor referenced by any `SubstrateAccessorBinding` across all target languages.

On each `Transform { target: Callable(template), .. }`:

```
if template in substrate_accessors:
    realization = lookup(template)
    carrier = read_field(realization, "carrier")  # String
    rendered_inputs = [render(input) for input in Transform.inputs]
    return positional_template(carrier, rendered_inputs)

elif template in substrate_accessor_universe:
    # Declared substrate accessor but no binding for the active target.
    # Fail closed — silent fallthrough would emit `func(args)` for a
    # function this target doesn't provide. Post round 1b.4.
    return Err(EmitError with fix instructions)

else:
    # Not a substrate accessor at all — fall through to existing
    # callable dispatch (user functions, list builtins, etc.).
```

Placeholder convention: positional `{p0}`, `{p1}`, … matching the Arrow's parameter order. Chosen over named `{arg_name}` placeholders because declarations don't carry param-name metadata past lowering; positional stays structural without round-tripping the source file.

Carrier templates live in per-target spec files (`src/v3/spec/rust.dag`, future `go.dag` / `python.dag`). Example:

```dag
data rust_port_accessor: SubstrateAccessorRealization = {
  carrier: "({p0}).port_opt({p1}).cloned()"
}
```

Wired in `emit_rust.rs` today (see `build_substrate_accessor_index` + `render_substrate_accessor`). When Lane 1e collapses the three emitters into a single walker + per-target specs, the index becomes one code path in the walker + three spec entries — the dispatch mechanism (filter bindings by active language, lookup-or-fail) is unchanged.

---

## Rejected alternatives

- **Write `.dag` bodies for substrate accessors** — the session's first attempt. Pollutes bootstrap; implementation is a lie (the real impl is Rust HashMap, not linear walk). Rejected; this is the trigger for DB-14.

- **Add a new `TransformTarget::Substrate(DeclarationId)` variant** — forks a substrate coproduct whose existing variants (`Callable`, `FieldProject`, `Operator`) have their own dissolution paths. Adds scaffolding on scaffolding. The fork belongs at the `ArrowBody` level (where `ExternalRealization` already lives), not at the TransformTarget level. Rejected.

- **Add a new `ArrowBody::TargetProvided(RealizationRef)` variant** — duplicates the existing `ArrowBody::ExternalRealization`. Same concept, new name. Rejected.

- **Substrate `PrimitiveKind` enum parallel to `OperatorKind`** — a closed coproduct that grows every time a new substrate primitive appears. Violates "structural not taxonomic" and would need a dissolution trigger on day one. Rejected.

- **Algebra-field pattern (substrate accessors as `Substrate<D>` algebra fields)** — elegant on paper: Dag inhabits Substrate, accessors emerge from algebra inhabitance like `add` emerges from `Ring`. But `resolve_operator_arrow` (the existing algebra-field dispatch) is keyed by `OperatorKind` (a closed enum) via `op_kind.algebra_field_name()`, and its arity assumption is binary-operators-with-receiver substitution. Adapting it to arbitrary-signature substrate accessors requires either expanding `OperatorKind` (another closed-coproduct growth) or forking the resolution logic. The ExternalRealization path is already wired end-to-end without these stretches. Rejected. (If future substrate concepts genuinely ARE algebra-shaped — e.g., a `Lens<D>` algebra with `read`/`write` operators — that path stays open for them; DB-14 isn't the right case.)

- **Tree-shaking / dead-code elimination at the end of lowering** — pruning unused bootstrap declarations from user DAGs is a legitimate separate optimization but doesn't address the fundamental issue (the `.dag` body is a wrong implementation). Out of scope for DB-14.

- **Lazy body lowering** — fn bodies lower on first call-site resolution rather than at bootstrap. Architecturally clean but very broad; touches the lowering pipeline's fundamental invariants. Out of scope for DB-14; orthogonal to this case.

---

## Open questions

1. **Template syntax alignment.** `rust_collection_ops` et al. use `{arg_name}` placeholder substitution today. Does an existing template-render function handle this, or is the implementation per-consumer? If per-consumer, DB-14's emission code can use the same shape rather than invent a new one. Verify by reading the emitter's existing collection-ops template handling.

2. **Single `SubstrateAccessorRealization` type, or per-accessor types?** Pipeline.dag uses one `CompilerHostRealization` type shared across all five stages. Lean; recommended. Per-accessor types would let each accessor declare its own field shape (e.g., multiple carriers for different dispatch conditions), which is unneeded today. Stick with one shared type.

3. **Does Lane 1e collapse affect the wiring?** The emission dispatch has to be wired in all three emit files today. When Lane 1e lands a single generic walker + per-target specs, the dispatch becomes one code path in the walker + three spec entries. No design change — just three → one consolidation.

4. **Is the "trivial stub body" constraint checkable?** Pipeline.dag relies on discipline ("trivially type-correct stub bodies") rather than a mechanical check. An accessor declaration that sneaks in a non-trivial body would silently contribute Transforms to every user DAG. Consider adding an acceptance-gate test that asserts `dag.nodes()` growth from loading substrate.dag is bounded (e.g., zero Callable Transforms contributed by the three accessor declarations after bootstrap upgrade). Not a blocker for the initial landing but a useful ratchet.

---

## Acceptance (Lane 1 Stage 1b owns) — all met on #501

- [x] `SubstrateAccessorRealization` type declared in `src/v3/std/substrate.dag`
- [x] `SubstrateAccessorBinding { accessor, realization, language }` type declared in `src/v3/std/substrate.dag` — `language` is the structural target selector (review 1b.3)
- [x] `port`, `node`, `resolve_producer` declared as fn arrows with trivial `{ host <name> }` stub bodies in `substrate.dag` — bodies stay `Unparsed` through bootstrap
- [x] `SubstrateAccessorBinding` declarations for all (accessor × Rust) pairs in `src/v3/spec/rust.dag` — 3 bindings. Go/Python add 3 each when those backends land; 9 total projected.
- [x] Per-target carrier declarations in `src/v3/spec/rust.dag`. Carrier uses positional `{p0}`, `{p1}` placeholders matching the Arrow's input order.
- [x] Bootstrap does **NOT** upgrade substrate-accessor Arrow bodies. The earlier upgrade design was reverted; see § 5 for rationale. (Pipeline stages still use the upgrade pattern — they're target-invariant.)
- [x] `dag.nodes()` delta from substrate.dag load is bounded — the stub bodies are `Unparsed(body_span)` so nothing lowers into `dag.nodes()`. `substrate_accessors_exist_in_bootstrap_dag` asserts this.
- [x] Emission dispatch wired in `emit_rust.rs` (`RealizationIndexes::substrate_accessors` + `render_substrate_accessor`). Go / Python land when those backends get their bindings.
- [x] Fail-closed coverage check (review 1b.4): `RealizationIndexes::substrate_accessor_universe` holds every accessor across all languages; emission errors if a user-visible accessor is declared but has no binding for the active target. `substrate_accessor_universe_fully_covered_for_rust` pins the invariant.
- [x] `m1_substrate_test` tests from the first attempt continue to pass (the ones broken by `.dag`-body bootstrap pollution).
- [x] Three existing lenses (`complexity.dag`, `provenance.dag`, `unused_parameters.dag`) migrate to call `port(d, id)` / `node(d, id)` / `resolve_producer(d, id)` (provenance + unused_parameters; complexity walks its own accumulator so didn't need the substrate accessors); oracle + snapshot + clone-count tests pass.
- [x] Line count reduction in `src/v3/lenses/*.dag` ≥ 15%: 483 → 400 (−17.2%).
- [x] INVARIANTS.md L-7 landed (lenses don't reconstruct lookup locally). CI grep gate blocks regressions.
- [x] Single-authority enforcement at emission-index build: duplicate `(accessor × language)` bindings fail closed with `EmitError::DuplicateRealization`.

---

## Associations

- **Lane 1 Stage 1b** ([lane1-stage-b-substrate-keyed-lookup.md](./lane1-stage-b-substrate-keyed-lookup.md)) — this doc's consumer. Shipped on PR #501.
- **DB-5 Substrate keyed-lookup API** ([design-substrate-keyed-lookup-api.md](./design-substrate-keyed-lookup-api.md)) — DB-5 specified the three query functions; DB-14 specifies HOW they're declared and emitted.
- **`src/v3/compiler/pipeline.dag`** — the target-invariant template. Uses bootstrap-upgrade pattern (unchanged).
- **`src/v3/compiler/src/bootstrap.rs`** — contains `materialize_pipeline_realizations` (unchanged). Does NOT upgrade substrate accessors; see § 5.
- **`src/v3/compiler/src/dag.rs`** — `ArrowBody::ExternalRealization` variant; `Dag::port_opt` / `node_opt` / `resolve_producer_opt` methods (the Rust-side primitives the carriers call).
- **`src/v3/compiler/src/infer.rs`** — no changes needed; both `Unparsed` and `ExternalRealization` are signature-only variants for inference.
- **`src/v3/compiler/src/emit_rust.rs`** — `RealizationIndexes::substrate_accessors` + `substrate_accessor_universe`, `build_substrate_accessor_index`, `render_substrate_accessor`. Emission-time dispatch authority.
- **Lane 1 Stage 1e** ([phase1-lane3-consolidation-build-plan.md](./phase1-lane3-consolidation-build-plan.md)) — when 1e consolidates emitters, the index-build + render path becomes one code site; the spec files drive per-target carriers.

---

## Not banked-dissolutions entries

DB-14 rejects shapes (α/β-duplicate/γ/δ) but they aren't added to the banked-dissolutions ratchet. Those names (`TransformTarget::Substrate`, `PrimitiveKind`, `ArrowBody::TargetProvided`) are too narrow to become recurring lane-doc drift. If a future doc proposes one of them, the reviewer points at this doc's rejected-alternatives section. The ratchet stays focused on shapes that have shown repeat-violation patterns.
