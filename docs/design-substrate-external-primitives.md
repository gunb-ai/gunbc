> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 1 Stage 1b (substrate keyed-lookup) | Consumes: INVARIANTS.md § E-9 "External realization lives on Arrow.body"

# Design DB-14 — Substrate external primitives (banked against E-9)

**Design blocker:** DB-14 (substrate primitives backed by target-native implementations)
**Consumer:** Lane 1 Stage 1b (substrate keyed-lookup accessors: `port`, `node`, `resolve_producer`)
**Status:** Third revision. Rewritten against [E-9](../INVARIANTS.md#e-9-external-realization-lives-on-arrowbody-2026-04-17) landed in the same PR.

---

## Correction history

**Revision 1.** Proposed per-`(accessor × target)` `SubstrateAccessorBinding` declarations in `substrate.dag` + bootstrap pass rewriting each accessor's Arrow body to `ExternalRealization(binding)`. Reviewers (codex + chatgpt) flagged: `compile_to_dag` bootstrap has no active target, all specs load into one Dag, walking bindings would rewrite each Arrow body N times (last-binding-wins bug), and per-target realization in substrate violates "one new target = one spec file."

**Revision 2.** Moved target dispatch out of bootstrap to emission. Used `meta_tag = substrate_accessor` + per-target `CallableRealization` in each spec. Reviewers (round 2, unanimous) flagged: authority split three ways (Arrow stub-body + meta_tag + spec lookup); external-vs-user-defined distinction moved OFF `Arrow.body` onto side marker; violates the thesis's "substrates meet at `Transform → Arrow` and `Arrow → body`." Meta-review issued PAUSE_AND_REGROUP: "bank the rule first, then redesign."

**This revision (R3).** Banks [E-9](../INVARIANTS.md#e-9-external-realization-lives-on-arrowbody-2026-04-17) in the same PR. Places the "external" fact on `Arrow.body` as the single structural authority. Keeps target dispatch at emission via per-target spec files. No meta_tag, no convention-based markers, no side lookup table.

---

## Problem

A compiler-internal function whose authoritative implementation is Rust (performance: HashMap vs linear walk; architecture: substrate primitive) cannot be declared in `src/v3/std/*.dag` with a `.dag` body:

- Bootstrap loads every file in the std/spec/compiler set.
- Every `fn` declaration's body lowers to a sub-DAG of `Transform`/`Bind`/etc. nodes.
- Those nodes end up in `dag.nodes()` before the user's code is lowered.
- Tests using "first Callable Transform" heuristics break; more broadly, any consumer that assumes the program's DAG is the user's breaks.
- Also wrong: the `.dag` body IS a different implementation than what the compiler runs (O(n) linear walk vs O(1) HashMap).

Lane 1 Stage 1b's original design called for declaring `port`/`node`/`resolve_producer` in substrate.dag with `.dag` bodies. PR #495 shipped 1a separately; 1b's first attempt (PR #501) hit this exact pollution and was reverted.

---

## Design (R3)

### Principle (E-9)

The structural authority for "this callable is externally realized" is `Arrow.body == ArrowBody::ExternalRealization(ref)`. Nothing else. Emission dispatches on `Arrow.body`.

### Target-neutral accessor markers in substrate.dag

One declaration per accessor, target-neutral. These markers exist only as identity nodes — shared between substrate (for `Arrow.body` to reference) and spec files (for realization records to reference).

```dag
// src/v3/std/substrate.dag
type SubstrateAccessor {
  name: String   // debugging aid; structural identity is by DeclarationId
}

data port_accessor: SubstrateAccessor = { name: "port" }
data node_accessor: SubstrateAccessor = { name: "node" }
data resolve_producer_accessor: SubstrateAccessor = { name: "resolve_producer" }
```

No per-target info. No carrier template. Just identity.

### Accessor Arrows + one-per-accessor bootstrap binding

Accessor fns declared with trivial stub bodies (mirroring pipeline.dag pattern). A single `SubstrateAccessorBinding` per accessor links it to its marker.

```dag
// src/v3/std/substrate.dag
fn port(d: Dag, id: PortId) -> DagPort? {
  accessor port
}
fn node(d: Dag, id: NodeId) -> Behavior? {
  accessor node
}
fn resolve_producer(d: Dag, port_id: PortId) -> Behavior? {
  accessor resolve_producer
}

type SubstrateAccessorBinding {
  accessor: DeclarationRef       // the Arrow (e.g., `port`)
  marker: DeclarationRef         // the SubstrateAccessor instance (e.g., `port_accessor`)
}

data port_binding: SubstrateAccessorBinding = {
  accessor: port
  marker: port_accessor
}
data node_binding: SubstrateAccessorBinding = {
  accessor: node
  marker: node_accessor
}
data resolve_producer_binding: SubstrateAccessorBinding = {
  accessor: resolve_producer
  marker: resolve_producer_accessor
}
```

**One binding per accessor — NOT per target.** The revision-1 last-binding-wins bug is impossible: there's exactly one binding per accessor, the bootstrap rewrite is deterministic, and there's no per-target info at this layer.

### Bootstrap upgrade (parallels pipeline.dag precisely)

Extend `bootstrap.rs` with `upgrade_substrate_accessor_bodies`, mirroring `upgrade_pipeline_stage_bodies`:

```rust
// src/v3/compiler/src/bootstrap.rs
fn upgrade_substrate_accessor_bodies(dag: &mut Dag) {
    for binding in substrate_accessor_bindings(dag) {
        let accessor_decl = dag.declaration_mut(binding.accessor);
        match &mut accessor_decl.connective {
            TypeConnective::Arrow { body, .. } => {
                *body = ArrowBody::ExternalRealization(binding.marker);
            }
            _ => report_substrate_accessor_error(
                dag,
                format!("substrate accessor `{}` must lower to an Arrow",
                        accessor_decl.name),
            ),
        }
    }
}
```

Fail-closed: if an accessor declaration isn't an Arrow, or the binding's accessor/marker edges don't resolve, bootstrap attaches a diagnostic and fails. No silent bypass.

**Post-bootstrap invariant (E-9):** each accessor's `Arrow.body == ExternalRealization(marker)`. The marker is the single structural fact. Any emitter, lens, or self-hosted analyzer walking the substrate reaches "this is externally realized" through one path: Arrow.body.

### Per-target realizations (decentralized spec files, reusing existing shape)

Each target's spec file declares a `BehaviorRealization` entry per accessor it supports. `BehaviorRealization` already exists in `src/v3/std/emit_model.dag` with the shape `{ language, target, carrier, cost }` — the same shape `rust_let_stmt`, `rust_if_expr`, and every other behavior-variant realization uses today. We reuse it directly; no new realization type.

```dag
// src/v3/spec/rust.dag
data rust_port: BehaviorRealization = {
  language: rust_language
  target: port_accessor                    // <- references the marker
  carrier: "({d}).port_opt({id}).cloned()"
  cost: 1
}

data rust_node: BehaviorRealization = {
  language: rust_language
  target: node_accessor
  carrier: "({d}).node_opt({id}).cloned()"
  cost: 1
}

data rust_resolve_producer: BehaviorRealization = {
  language: rust_language
  target: resolve_producer_accessor
  carrier: "({d}).resolve_producer({port_id})"
  cost: 1
}
```

Analogous `go_*` in `go.dag`, `python_*` in `python.dag`. Each spec is self-contained — adding a new target means adding one spec file with realizations for accessors it supports. No substrate edit.

**Note on `BehaviorRealization.target` semantics.** Today its usages point at Behavior-kind declarations (`Bind`, `Branch`, `Transform`, `Loop`, `Value`). DB-14 extends usage to include `SubstrateAccessor`-typed declarations. The field's type is `DeclarationRef` — structurally unconstrained — so this is not a schema violation. The semantic widening of `target` ("the thing being realized" rather than "the Behavior variant being realized") is legitimate; a future PR may rename `target` → `realizes` for clarity or consider unifying OperatorRealization/BehaviorRealization/etc. into a general `TargetCarrierRealization`. That consolidation is out of scope for DB-14 and tracked separately.

### Emission dispatch

When an emitter renders a `Transform` node whose target is `Callable(decl_id)`:

```
decl = dag.declaration(decl_id)
if decl.connective is TypeConnective::Arrow { body: ExternalRealization(marker_id), .. }:
    # E-9: Arrow.body is the single authority. No meta_tag check, no name match.
    realization = find_realization_in_active_spec(marker_id, current_language)
    if realization is None:
        emit Diagnostic::ResolveError {
            name: "target `{current_language}` does not realize `{decl.name}`",
            span: Transform.span,
        }
        return unresolved
    render_carrier_template(realization.carrier, Transform.inputs, param_names)
else:
    # Fall through to existing user-defined-fn rendering path (ArrowBody::UserDefined).
    ...
```

`find_realization_in_active_spec(marker_id, language)`: walks `BehaviorRealization` data items in the loaded Dag, returns the one whose `target == marker_id` and `language == current_language`. If multiple match or none match for a used accessor, emission fails closed.

`render_carrier_template`: string substitution of `{arg_name}` → rendered input expression. Aligned with existing `rust_let_stmt` / `rust_if_expr` template handling in `emit_rust.rs::render_template`.

Wired in all three current emit files (`emit_rust.rs`, `emit_go.rs`, `emit_python.rs`). When Lane 1e collapses emitters into a single walker + per-target specs, this dispatch becomes one site. No design change at that point — just three-to-one consolidation. **E-9 holds identically through that transition** because the structural fact lives on Arrow.body, not in emit-file-specific code.

---

## Why this design satisfies all three reviewer BLOCKING concerns

**BLOCKING 1 (illegal states unrepresentable):** The three illegal states from revision 2 are now structurally ruled out:
- *Marked accessor with a real body*: impossible — `ArrowBody::ExternalRealization(_)` is the body; there IS no user-defined sub-DAG competing with it.
- *Unmarked accessor with realizations*: an orphan `BehaviorRealization` pointing at a marker that no Arrow.body references is still findable (it's just dead data), but it has no consumer — an accessor used by lens code MUST have Arrow.body reference matching the realization's target; otherwise emission fails closed with a diagnostic.
- *Stub-bodied accessor whose semantic authority exists only in emitter convention*: impossible — post-bootstrap, stub bodies are replaced with `ExternalRealization`. Any stub-bodied accessor discovered at emission time is a bootstrap failure (diagnosed).

**BLOCKING 2 (single-authority metadata):** One authority — `Arrow.body`. The marker (`port_accessor` etc.) is a target-neutral identity fact that both substrate and specs reference, but it's not a parallel authority for "externally realized" — that fact is structural on the body variant. Spec-side realization records are DOWNSTREAM CONSUMERS of the marker identity, not competing authorities.

**BLOCKING 3 (API-level enforcement):** E-9 is the enforcement. Arrow body variant is structurally checked by every consumer (emit, lens, self-hosted analyzer). No discipline-checked stub-body convention; no convention-level meta_tag. A "substrate accessor with a non-trivial body" is impossible because there's no body to be non-trivial — it's `ExternalRealization(marker)`.

---

## Rejected alternatives

- **Write `.dag` bodies for substrate accessors** (revision 0, session's first attempt). Pollutes bootstrap; implementation is a lie. Rejected.

- **Per-(accessor × target) bindings in substrate + bootstrap target dispatch** (revision 1). Bootstrap is target-agnostic; `last-binding-wins` bug; violates "one new target = one spec file." Rejected.

- **`meta_tag = substrate_accessor` + per-target spec search, stub body preserved on Arrow** (revision 2). Splits authority three ways. Moves external-vs-user-defined OFF `Arrow.body` — violates E-9 (and the thesis's substrate boundary). Rejected.

- **Add a new `TransformTarget::Substrate(DeclarationId)` variant.** Forks TransformTarget at the wrong level — externality belongs at Arrow.body, not at the call site. Rejected.

- **Add a new `ArrowBody` variant parallel to `ExternalRealization`.** Duplicates the existing variant for the same concept. Rejected.

- **Substrate `PrimitiveKind` enum parallel to `OperatorKind`.** Closed coproduct growth, would need dissolution trigger. Rejected.

- **Algebra-field pattern (substrate accessors as `Substrate<D>` algebra fields).** `resolve_operator_arrow` is keyed by `OperatorKind` enum with binary-operator arity assumption; doesn't fit arbitrary-signature substrate accessors. Rejected.

- **New `SubstrateAccessorRealization` type parallel to `BehaviorRealization`.** Proliferates realization schemas when the existing `{ language, target, carrier, cost }` shape already fits. Rejected per codex's "collapse to one code-verified realization family" concern.

---

## Open questions

1. **Semantic widening of `BehaviorRealization.target`.** Today used for Behavior kinds (Bind/Branch/Transform/etc.); DB-14 extends to SubstrateAccessor markers. Legitimate because the field type is `DeclarationRef`, but the name `target` suggests Behavior specifically. Options: rename `target` → `realizes` (broader), or leave and document. A future unification PR may also merge Operator/Behavior/substrate realization types into `TargetCarrierRealization`. Neither is in DB-14's scope.

2. **`PythonCallableRealization`'s status.** `python.dag` still uses a Python-specific type (`PythonCallableRealization`) instead of the shared `CallableRealization`. This is an orthogonal debt item already noted in the existing code comment. DB-14 uses `BehaviorRealization` (the correctly-shared schema) throughout, so it doesn't aggravate this. The Python-callable migration remains a separate task.

3. **Accessor marker shape.** Chose `type SubstrateAccessor { name: String }` (identity + debug-friendly name). Could be structurally nameless (identity-only) if future reflection doesn't need the name. Debug clarity wins for now; `name` is not structurally load-bearing.

4. **`emit.dag` migration impact.** When emission moves from `emit_rust.rs`/`emit_go.rs`/`emit_python.rs` into `.dag` (Lane 1e + beyond), the E-9 dispatch rule applies unchanged: `emit.dag` walks Arrow.body and handles the `ExternalRealization` variant structurally. No design refactor required at that boundary.

---

## Acceptance (Lane 1 Stage 1b owns)

- [ ] E-9 invariant present in INVARIANTS.md (this PR lands it alongside DB-14)
- [ ] `SubstrateAccessor` type + three accessor markers (`port_accessor`, `node_accessor`, `resolve_producer_accessor`) declared in `substrate.dag`
- [ ] Three accessor fns (`port`, `node`, `resolve_producer`) declared as Arrows with stub bodies in `substrate.dag`
- [ ] Three `SubstrateAccessorBinding` data items (one per accessor, NOT per target) in `substrate.dag`
- [ ] `bootstrap.rs::upgrade_substrate_accessor_bodies` implemented, mirroring `upgrade_pipeline_stage_bodies`; fail-closed on missing/non-Arrow accessors
- [ ] Three per-target `BehaviorRealization` data items in `src/v3/spec/rust.dag`, `go.dag`, `python.dag` (9 total), `target` fields reference the accessor markers
- [ ] Post-bootstrap invariant test: each accessor's `Arrow.body == ExternalRealization(marker)` — NOT `Unparsed`, NOT a real sub-DAG. Mirrors pipeline.dag's invariant test (from DB-16 §3)
- [ ] `dag.nodes()` delta from substrate.dag load + bootstrap is bounded — zero Callable Transforms contributed by the three accessor declarations
- [ ] Emission dispatch on `ArrowBody::ExternalRealization` wired in `emit_rust.rs`, `emit_go.rs`, `emit_python.rs`
- [ ] Fail-closed emission test: a program that calls a substrate accessor whose marker has no realization in the active target's spec produces a diagnostic naming the (target, accessor) pair
- [ ] Three existing lenses (`complexity.dag`, `provenance.dag`, `unused_parameters.dag`) migrate to call `port(d, id)` / `node(d, id)` / `resolve_producer(d, id)`; oracle tests pass
- [ ] Line count reduction in `src/v3/lenses/*.dag` ≥ 15%
- [ ] INVARIANTS.md L-7 landed (lenses don't reconstruct lookup locally) — separate from E-9

---

## Associations

- **[E-9 (INVARIANTS.md)](../INVARIANTS.md#e-9-external-realization-lives-on-arrowbody-2026-04-17)** — the rule this design implements. Landed in the same PR.
- **Lane 1 Stage 1b** ([lane1-stage-b-substrate-keyed-lookup.md](./lane1-stage-b-substrate-keyed-lookup.md)) — the stage that consumes DB-14.
- **DB-5** ([design-substrate-keyed-lookup-api.md](./design-substrate-keyed-lookup-api.md)) — specifies the three accessor signatures (`port`, `node`, `resolve_producer`) this design realizes.
- **DB-16** ([design-fn-external-body-reconciliation.md](./design-fn-external-body-reconciliation.md)) — companion: clarifies that `FnExternalBody` + bootstrap-rewrite pattern applies to both pipeline stages AND substrate accessors.
- **`src/v3/compiler/pipeline.dag`** — the production template; DB-14 uses the exact same parser/bootstrap pattern.
- **`src/v3/compiler/src/bootstrap.rs::upgrade_pipeline_stage_bodies`** — the production implementation DB-14 mirrors.
- **`src/v3/std/emit_model.dag::BehaviorRealization`** — the existing realization shape DB-14 reuses.
- **`src/v3/compiler/src/dag.rs::ArrowBody::ExternalRealization`** — the existing substrate variant DB-14 places on accessor Arrow bodies.
- **`src/v3/compiler/src/infer.rs::is_realization_shape`** — existing dispatch-time check; DB-14 doesn't extend this (markers aren't inferred, they're looked up at emission).
