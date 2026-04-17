> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 1 Stage 1b (substrate keyed-lookup)

# Design DB-14 — Substrate external primitives

**Design blocker:** DB-14 (substrate primitives backed by target-native implementations)
**Consumer:** Lane 1 Stage 1b (substrate keyed-lookup accessors: `port`, `node`, `resolve_producer`)
**Status:** Design ready for implementer review.
**Origin:** Lane 1 Stage 1b escalation (2026-04-17) — the session's initial attempt declared `.dag` linear-walk bodies for the accessors and hit bootstrap pollution (recursive bodies lowered into every user program's `dag.nodes()`). The compiler already has the right mechanism for this case; this doc points at it.

---

## TL;DR

The substrate already has `ArrowBody::ExternalRealization(DeclarationId)`. The compiler already type-checks, dispatches, and bootstrap-upgrades Arrows through this variant end-to-end for its own pipeline stages. The gap for user-code-callable primitives (substrate accessors, future substrate ops) is **emission dispatch only**. DB-14 codifies the pattern and specifies the emission wiring.

**No new substrate concept. No new TransformTarget variant. No coproduct extension.**

---

## Problem

A compiler-internal function whose authoritative implementation is Rust (because of performance, because the implementation is a HashMap, because it's a substrate primitive) cannot be declared in `src/v3/std/*.dag` with a `.dag` body:

- Bootstrap loads every `.dag` file in the std/spec/compiler set.
- Every fn declaration's body lowers to a sub-DAG of `Transform` / `Bind` / etc. nodes.
- Those nodes end up in `dag.nodes()` before the user's code is lowered.
- Tests that use "first Callable Transform" heuristics (and, more broadly, any consumer that assumes the program's DAG is just the user's) break.
- Worse: the `.dag` body IS a wrong implementation. Writing a linear walk for `port(d, id)` in `.dag` is O(n) while the Rust side has an O(1) HashMap. The `.dag` body is a lie about what the compiler actually runs.

Lane 1 Stage 1b's original design called for declaring `port(d, id) -> DagPort?`, `node(d, id) -> Behavior?`, `resolve_producer(d, id) -> Behavior?` in `substrate.dag` with full `.dag` bodies. The session's implementation correctly compiled but polluted bootstrap and was reverted. The escalation flagged this as "a missing substrate primitive" — that framing turned out to be wrong. The primitive is present; the pattern for using it just wasn't in the design doc.

---

## Key finding: the mechanism exists

### `ArrowBody::ExternalRealization` (substrate, `dag.rs:519`)

> "Primitive whose realization is declared in an extdeps language spec. DeclarationId points at the realization declaration via a typed edge; inference verifies signature compatibility."

This is the variant we want. An Arrow with `body: ExternalRealization(realization_id)` says: the Arrow's signature type-checks against callers, but the body is not a `.dag` sub-DAG — it's provided by the target, referenced through `realization_id`.

### Inference wiring (`infer.rs:896-916`)

Dispatch-time check on an `ExternalRealization` target: the linked declaration must be a `Conj` with a non-`None` `meta_tag` edge. The meta_tag IS the realization marker (round-10 correction: no separate `Realization` declaration cached in the `Dag`). Structural; no string comparison; asserted at both construction (`assert_realization_shape`) and dispatch (`is_realization_shape`).

Call sites type-check normally against the Arrow's signature. The `ExternalRealization` body is not walked.

### Production template: `src/v3/compiler/pipeline.dag`

The compiler's own pipeline stages are the living example:

```dag
type CompilerHostRealization { symbol: String }

data parse_realization: CompilerHostRealization = {
  symbol: "v3_compiler::parse"
}

data lower_realization: CompilerHostRealization = {
  symbol: "v3_compiler::lower"
}

// ... and fn declarations with trivial type-correct stub bodies:
fn parse(source: String, file: String) -> Dag = ...  // stub body
```

Bootstrap upgrade (`bootstrap.rs:238-252`):

```rust
for stage in stages {
    let stage_decl = dag.declaration_mut(stage.stage);
    match &mut stage_decl.connective {
        TypeConnective::Arrow { body, .. } => {
            *body = ArrowBody::ExternalRealization(stage.realization);
        }
        _ => report_pipeline_authority_error(...),
    }
}
```

The file comment (pipeline.dag:10-12) is explicit:

> "The source bodies stay trivially type-correct so the file parses and lowers cleanly today."

Trivially-stub bodies lower to minimal or zero `dag.nodes()` content. After bootstrap upgrade, the body is `ExternalRealization` — not walked at inference, not contributing further Transforms. The runtime dispatch target is the realization declaration's `symbol` field, resolved by the compiler host.

---

## The one real gap: emission

Grep confirms no emitter — `emit_rust.rs`, `emit_go.rs`, `emit_python.rs` — references `ExternalRealization`. Pipeline stages don't hit emission because they ARE the compiler (runtime-only, never emitted into target code). Substrate accessors called from **user-code lenses** (`complexity.dag` calling `port(d, id)`) WILL hit emission, and today the emitter has no branch for `ArrowBody::ExternalRealization` on a Callable Transform's target.

This is the sole new-code item in DB-14.

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

### 4. Substrate accessor marker

Accessor declarations carry a `meta_tag` marker so emission can distinguish them from ordinary user-defined fns without name-matching or module-location hacks.

```dag
// src/v3/std/substrate.dag
data substrate_accessor: MetaTag

// And each accessor's declaration carries meta_tag: Some(substrate_accessor_id).
// (Achieved via the existing meta_tag edge on Declaration.)
```

### 5. Per-target realizations (decentralized, no central roster)

**Correction from an earlier revision.** An earlier draft of this doc proposed per-`(accessor × target)` `SubstrateAccessorBinding` declarations in `substrate.dag` plus a bootstrap pass that rewrote each accessor's Arrow body to `ExternalRealization(binding)`. Reviewers on PR #497 correctly flagged that as wrong on two axes:

- **Bootstrap is target-agnostic.** `compile_to_dag(source, file)` has no target input; `bootstrap.rs` loads ALL v3 specs into one Dag. Walking `SubstrateAccessorBinding`s and rewriting Arrow bodies would visit every (accessor × target) pair and "last binding wins" — at least two emitters would read the wrong carrier.
- **Thesis violation.** The "active target" assumption bakes one target's realization into substrate-visible state. Violates "adding a new target = one spec file": substrate gains an entry per target, not just the target's own spec.

**The correct shape:** realizations are declared by each target's spec file (decentralized), emission dispatches on the current target at render time (not at bootstrap), and the accessor declaration itself stays target-agnostic.

```dag
// src/v3/spec/rust.dag
data rust_port_realization: CallableRealization = {
  target: port                        // DeclarationRef to substrate.dag's `port`
  carrier: "({d}).port_opt({id}).cloned()"
}

data rust_node_realization: CallableRealization = {
  target: node
  carrier: "({d}).node_opt({id}).cloned()"
}

data rust_resolve_producer_realization: CallableRealization = {
  target: resolve_producer
  carrier: "({d}).resolve_producer({port_id})"
}
```

Analogous `go_*` in `go.dag` and `python_*` in `python.dag`. Each spec is self-contained: adding a new target means adding one spec file with N realizations (one per accessor it supports). No entry anywhere else grows.

**No bootstrap Arrow-body rewrite.** The accessor declarations stay as ordinary Arrows in `substrate.dag` (trivial stub bodies mirroring `pipeline.dag`'s pattern so parsing + type-checking succeed). The body is never walked at emission because dispatch goes through the realization lookup below.

### 6. Emission dispatch — the new code (target dispatch at render time)

When an emitter renders a `Transform` node:

```
if Transform.target is TransformTarget::Callable(decl_id):
    decl = dag.declaration(decl_id)
    if decl.meta_tag == Some(substrate_accessor_meta_id):
        // Look up the realization in the active target's spec.
        realization = find_realization_in_spec(active_target_spec, decl_id)
        if realization is None:
            emit Diagnostic::ResolveError {
                name: "target `{active_target}` does not realize substrate accessor `{decl.name}`",
                span: Transform.span,
            }
            return unresolved
        carrier = read_field(realization, "carrier")  # String template
        return render_template(carrier, Transform.inputs, param_names)
    else:
        # existing user-defined-fn rendering path
        ...
```

Where `find_realization_in_spec(spec, accessor_decl_id)` walks the spec's declared `CallableRealization` data items and finds the one whose `target` field matches `accessor_decl_id`. This is the same mechanism operator realizations already use (e.g., `rust_int_add` in `rust.dag` points at the `add` arrow on the `IntSemiring` algebra field).

**Fail-closed at emission.** If a user lens calls a substrate accessor that the active target doesn't realize, emission produces a diagnostic naming the (target, accessor) pair — not a silent skip.

**`render_template`** is string substitution with `{arg_name}` → rendered input expression, aligned with existing `rust_collection_ops` template handling. Implementer should confirm the existing template mechanism applies here OR implement a minimal version aligned with it.

Wired in all three emit files today. When Lane 1e collapses the emitters into a single walker + per-target specs, this dispatch becomes one site instead of three — the per-target lookup moves from per-emit-file dispatch to a spec-declared template lookup, but the structural dispatch (meta_tag match → realization lookup) is unchanged.

---

## Rejected alternatives

- **Write `.dag` bodies for substrate accessors** — the session's first attempt. Pollutes bootstrap; implementation is a lie (the real impl is Rust HashMap, not linear walk). Rejected; this is the trigger for DB-14.

- **Add a new `TransformTarget::Substrate(DeclarationId)` variant** — forks a substrate coproduct whose existing variants (`Callable`, `FieldProject`, `Operator`) have their own dissolution paths. Adds scaffolding on scaffolding. The fork belongs at the `ArrowBody` level (where `ExternalRealization` already lives), not at the TransformTarget level. Rejected.

- **Add a new `ArrowBody::TargetProvided(RealizationRef)` variant** — duplicates the existing `ArrowBody::ExternalRealization`. Same concept, new name. Rejected.

- **Substrate `PrimitiveKind` enum parallel to `OperatorKind`** — a closed coproduct that grows every time a new substrate primitive appears. Violates "structural not taxonomic" and would need a dissolution trigger on day one. Rejected.

- **Algebra-field pattern (substrate accessors as `Substrate<D>` algebra fields)** — elegant on paper: Dag inhabits Substrate, accessors emerge from algebra inhabitance like `add` emerges from `Ring`. But `resolve_operator_arrow` (the existing algebra-field dispatch) is keyed by `OperatorKind` (a closed enum) via `op_kind.algebra_field_name()`, and its arity assumption is binary-operators-with-receiver substitution. Adapting it to arbitrary-signature substrate accessors requires either expanding `OperatorKind` (another closed-coproduct growth) or forking the resolution logic. The ExternalRealization path is already wired end-to-end without these stretches. Rejected. (If future substrate concepts genuinely ARE algebra-shaped — e.g., a `Lens<D>` algebra with `read`/`write` operators — that path stays open for them; DB-14 isn't the right case.)

- **Tree-shaking / dead-code elimination at the end of lowering** — pruning unused bootstrap declarations from user DAGs is a legitimate separate optimization but doesn't address the fundamental issue (the `.dag` body is a wrong implementation). Out of scope for DB-14.

- **Lazy body lowering** — fn bodies lower on first call-site resolution rather than at bootstrap. Architecturally clean but very broad; touches the lowering pipeline's fundamental invariants. Out of scope for DB-14; orthogonal to this case.

- **Bootstrap-time Arrow-body rewrite to `ExternalRealization(binding)`** (earlier revision of this doc) — reviewers on PR #497 flagged this as wrong: bootstrap has no active target, all-specs-loaded-into-one-Dag means walking bindings would rewrite the same accessor body multiple times (last-binding-wins bug), and baking per-target realization into substrate state violates the thesis's "one new target = one spec file" rule. Replaced by: target dispatch at emission time via per-target spec files declaring `CallableRealization` records, keyed by the accessor declaration. Rejected.

- **Central roster of `SubstrateAccessorBinding` declarations in `substrate.dag`** (earlier revision) — violates decentralization. Every new target's spec file should contribute realizations on its own; substrate should not enumerate targets. Replaced by: each target's own spec declares its realizations. Rejected.

---

## Open questions

1. **Template syntax alignment.** `rust_collection_ops` et al. use `{arg_name}` placeholder substitution today. Does an existing template-render function handle this, or is the implementation per-consumer? If per-consumer, DB-14's emission code can use the same shape rather than invent a new one. Verify by reading the emitter's existing collection-ops template handling.

2. **Reuse `CallableRealization` or a new type?** `CallableRealization` already exists for operator realizations (e.g., `rust_int_add`). Reusing it for substrate accessors is natural; they're structurally the same ("declared fn, target-provided body"). Adding a new `SubstrateAccessorRealization` would fork authority. Prefer reuse; name the specific instances `rust_port_realization` etc. and rely on the `target` field to distinguish.

3. **Does Lane 1e collapse affect the wiring?** The emission dispatch has to be wired in all three emit files today. When Lane 1e lands a single generic walker + per-target specs, the dispatch becomes one code path in the walker + three spec entries. No design change — just three → one consolidation.

4. **Is the "trivial stub body" constraint checkable?** Pipeline.dag relies on discipline ("trivially type-correct stub bodies") rather than a mechanical check. An accessor declaration that sneaks in a non-trivial body would silently contribute Transforms to every user DAG. Consider adding an acceptance-gate test that asserts `dag.nodes()` growth from loading substrate.dag is bounded (e.g., zero Callable Transforms contributed by the three accessor declarations). Not a blocker for the initial landing but a useful ratchet.

5. **Meta-tag vs. implicit marker.** The corrected design uses `meta_tag` on each accessor to let emission distinguish substrate accessors from user fns. Alternative: the emission dispatch tries to find a realization in the active target's spec for EVERY Callable target; if found, dispatch via realization; else fall through to user-fn rendering. That would remove the meta_tag requirement. Tradeoff: implicit "try spec first" is cheap but less self-describing; explicit meta_tag is clearer but costs one substrate-level edge per accessor. Leaning meta_tag because emission staying fail-closed ("declared marker but no realization for this target" → diagnostic) is cleaner with the marker.

---

## Acceptance (Lane 1 Stage 1b owns)

- [ ] `substrate_accessor` meta-tag declared in `substrate.dag`; each accessor declaration carries it via `meta_tag`
- [ ] `port`, `node`, `resolve_producer` declared as fn arrows with trivial stub bodies in `substrate.dag`
- [ ] Per-target `CallableRealization` data items declared in `src/v3/spec/rust.dag`, `go.dag`, `python.dag` — one per (spec × accessor) pair, decentralized, no central roster
- [ ] No bootstrap-time Arrow-body rewrite; target dispatch happens at emission
- [ ] `dag.nodes()` delta from substrate.dag load is bounded — no Callable Transforms from the three accessors in user DAGs
- [ ] Emission dispatch in `emit_rust.rs`, `emit_go.rs`, `emit_python.rs` checks `meta_tag == substrate_accessor`, looks up the current-target's `CallableRealization` by accessor, renders per carrier template
- [ ] Fail-closed at emission: a target whose spec doesn't declare a realization for an accessor used by the program emits a diagnostic naming the (target, accessor) pair
- [ ] Existing `m1_substrate_test` tests (the ones the session's first attempt broke) continue to pass
- [ ] Three existing lenses (`complexity.dag`, `provenance.dag`, `unused_parameters.dag`) migrate to call `port(d, id)` / `node(d, id)` / `resolve_producer(d, id)`; oracle tests pass
- [ ] Line count reduction in `src/v3/lenses/*.dag` ≥ 15% (per the relaxed threshold from DB-5's revised acceptance)
- [ ] INVARIANTS.md L-7 landed (lenses don't reconstruct lookup locally)

---

## Associations

- **Lane 1 Stage 1b** ([lane1-stage-b-substrate-keyed-lookup.md](./lane1-stage-b-substrate-keyed-lookup.md)) — this doc unblocks that stage. 1b's scope folds DB-14's acceptance in.
- **DB-5 Substrate keyed-lookup API** ([design-substrate-keyed-lookup-api.md](./design-substrate-keyed-lookup-api.md)) — DB-5 specified the three query functions; DB-14 specifies HOW they're declared and emitted.
- **`src/v3/compiler/pipeline.dag`** — the production template. Implementer reads this first.
- **`src/v3/compiler/src/bootstrap.rs`** — contains `upgrade_pipeline_stage_bodies`; DB-14 adds `upgrade_substrate_accessor_bodies` in the same style.
- **`src/v3/compiler/src/dag.rs:511-533`** — `ArrowBody::ExternalRealization` substrate variant.
- **`src/v3/compiler/src/infer.rs:896-916`** — inference-time dispatch check; no changes needed.
- **`src/v3/compiler/src/infer.rs:2920`** — `is_realization_shape`; no changes needed.
- **`src/v3/compiler/src/emit_rust.rs` / `emit_go.rs` / `emit_python.rs`** — emission dispatch gets the new `ExternalRealization` branch.
- **Lane 1 Stage 1e** ([phase1-lane3-consolidation-build-plan.md](./phase1-lane3-consolidation-build-plan.md)) — when 1e collapses emitters, the three dispatch sites become one.

---

## Not banked-dissolutions entries

DB-14 rejects shapes (α/β-duplicate/γ/δ) but they aren't added to the banked-dissolutions ratchet. Those names (`TransformTarget::Substrate`, `PrimitiveKind`, `ArrowBody::TargetProvided`) are too narrow to become recurring lane-doc drift. If a future doc proposes one of them, the reviewer points at this doc's rejected-alternatives section. The ratchet stays focused on shapes that have shown repeat-violation patterns.
