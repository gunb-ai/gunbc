> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 1 Stage 1b (substrate keyed-lookup) | Consumes: INVARIANTS.md § E-9 "External realization lives on Arrow.body"

# Design DB-14 — Substrate external primitives (banked against E-9)

**Design blocker:** DB-14 (substrate primitives backed by target-native implementations)
**Consumer:** Lane 1 Stage 1b (substrate keyed-lookup accessors: `port`, `node`, `resolve_producer`)
**Status:** Third revision. Rewritten against [E-9](./invariants/e-9-external-realization-lives-on-arrow-body.md) landed in the same PR.

---

## Correction history

**Revision 1.** Proposed per-`(accessor × target)` `SubstrateAccessorBinding` declarations in `substrate.dag` + bootstrap pass rewriting each accessor's Arrow body to `ExternalRealization(binding)`. Reviewers (codex + chatgpt) flagged: `compile_to_dag` bootstrap has no active target, all specs load into one Dag, walking bindings would rewrite each Arrow body N times (last-binding-wins bug), and per-target realization in substrate violates "one new target = one spec file."

**Revision 2.** Moved target dispatch out of bootstrap to emission. Used `meta_tag = substrate_accessor` + per-target `CallableRealization` in each spec. Reviewers (round 2, unanimous) flagged: authority split three ways (Arrow stub-body + meta_tag + spec lookup); external-vs-user-defined distinction moved OFF `Arrow.body` onto side marker; violates the thesis's "substrates meet at `Transform → Arrow` and `Arrow → body`." Meta-review issued PAUSE_AND_REGROUP: "bank the rule first, then redesign."

**Revision 3.** Banked [E-9](./invariants/e-9-external-realization-lives-on-arrow-body.md) in the same PR. Placed the "external" fact on `Arrow.body` via a `SubstrateAccessor` marker type + `SubstrateAccessorBinding` table; bootstrap walked bindings, rewrote Arrow bodies. Reviewer flagged a smaller version of the same authority-split class: the `accessor → marker` relation lived twice (in the binding table AND in the rewritten Arrow body), the "one binding per accessor" invariant was prose not shape, and duplicate/malformed bindings admitted illegal states the model said were impossible.

**This revision (R4).** Drops the `SubstrateAccessor` marker type and `SubstrateAccessorBinding` binding table entirely. The accessor's own declaration IS the identity the spec realizes — `ExternalRealization` self-references. One enumeration list (`substrate_accessors: List<DeclarationRef>`) tells bootstrap which Arrows to rewrite; duplicates are a fail-closed bootstrap diagnostic. Spec realizations reference the accessor declaration directly.

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

## Design (R4)

### Principle (E-9)

The structural authority for "this callable is externally realized" is `Arrow.body == ArrowBody::ExternalRealization(ref)`. Nothing else. Emission dispatches on `Arrow.body`.

### Accessor is its own identity — no separate marker, no binding table

R3 added a `SubstrateAccessor` marker type + `SubstrateAccessorBinding` table. The reviewer correctly flagged this as a smaller version of the authority-split class — the `accessor → marker` relation lived both in the permanent binding table and in the rewritten Arrow body, the "one binding per accessor" invariant was prose-only, and duplicates could slip past.

R4 removes the marker layer entirely. **The accessor declaration IS the identity the spec realizes.** `ExternalRealization` self-references.

```dag
// src/v3/std/substrate.dag
fn port(d: Dag, id: PortId) -> DagPort? {
  accessor
}
fn node(d: Dag, id: NodeId) -> Behavior? {
  accessor
}
fn resolve_producer(d: Dag, port_id: PortId) -> Behavior? {
  accessor
}
```

Stub bodies follow the pipeline.dag pattern (unparseable-to-M1(2.7) block, later rewritten by bootstrap). The keyword `accessor` inside the block is a visual marker for contributors; it is NOT load-bearing for the machinery.

### One enumeration — the authoritative list of which Arrows are accessors

Bootstrap needs to know which Arrows to rewrite. R4 uses a single enumeration, replacing the R3 per-accessor binding table:

```dag
// src/v3/std/substrate.dag
data substrate_accessors: List<DeclarationRef> = [
  port,
  node,
  resolve_producer,
]
```

One list. No pair type. No per-accessor binding declaration. Adding a new accessor is: declare the Arrow, append the ref to the list. Deleting: remove from list, delete the Arrow.

**Structural uniqueness.** Duplicates in the list are a fail-closed bootstrap diagnostic — a single pre-mutation pass validates the list contains each DeclarationRef at most once. If the compiler's existing `List` validation supports uniqueness natively at the substrate level, the check becomes structural; if not, it lives as bootstrap-side validation with a clear error. Either way, the "two bindings for one accessor" class is an explicit diagnostic, not a last-write-wins silent drift.

### Bootstrap upgrade — self-referential ExternalRealization

Extend `bootstrap.rs` with `upgrade_substrate_accessor_bodies`:

```rust
// src/v3/compiler/src/bootstrap.rs
fn upgrade_substrate_accessor_bodies(dag: &mut Dag) {
    let accessors = read_substrate_accessors_list(dag);

    // Pre-mutation uniqueness check — fail closed on any duplicate.
    if let Some(duplicate) = first_duplicate(&accessors) {
        report_substrate_accessor_error(
            dag,
            format!("`substrate_accessors` lists `{}` more than once — \
                     each accessor must appear exactly once",
                    dag.declaration(duplicate).name),
        );
        return;
    }

    for accessor_id in accessors {
        let accessor_decl = dag.declaration_mut(accessor_id);
        match &mut accessor_decl.connective {
            TypeConnective::Arrow { body, .. } => {
                // Self-reference: the accessor IS its own identity.
                *body = ArrowBody::ExternalRealization(accessor_id);
            }
            _ => report_substrate_accessor_error(
                dag,
                format!("substrate accessor `{}` must lower to an Arrow, got {:?}",
                        accessor_decl.name, accessor_decl.connective),
            ),
        }
    }
}
```

Fail-closed on two conditions:
- **Duplicate accessor in list:** pre-mutation diagnostic; bootstrap halts before any rewrite. Eliminates last-write-wins.
- **Accessor declaration isn't an Arrow:** per-item diagnostic; bootstrap notes the error and continues (other accessors still upgrade correctly, but the Dag carries the diagnostic).

**Post-bootstrap invariant (E-9):** each accessor's `Arrow.body == ExternalRealization(accessor_id)`. The accessor IS its own identity reference. No marker declaration exists to disagree with the Arrow body — there's nothing else for the identity to live in.

### Per-target realizations — direct reference, no indirection

Each target's spec declares a `BehaviorRealization` entry per accessor, reusing the existing `{ language, target, carrier, cost }` shape. The `target` field points at the accessor's own declaration directly:

```dag
// src/v3/spec/rust.dag
data rust_port: BehaviorRealization = {
  language: rust_language
  target: port                    // <- the accessor declaration, directly
  carrier: "({d}).port_opt({id}).cloned()"
  cost: 1
}

data rust_node: BehaviorRealization = {
  language: rust_language
  target: node
  carrier: "({d}).node_opt({id}).cloned()"
  cost: 1
}

data rust_resolve_producer: BehaviorRealization = {
  language: rust_language
  target: resolve_producer
  carrier: "({d}).resolve_producer({port_id})"
  cost: 1
}
```

Analogous `go_*` in `go.dag`, `python_*` in `python.dag`. Each spec is self-contained.

The accessor declaration plays exactly one role here — identity. It is the declaration the Arrow.body references (via self-reference) AND the declaration the spec realization targets. Emission joins them through that single identity.

**Note on `BehaviorRealization.target` semantics.** Today its usages point at Behavior-kind declarations (`Bind`, `Branch`, `Transform`, `Loop`, `Value`). DB-14 extends usage to include externally-realized Arrow declarations (accessor Arrows). The field's type is `DeclarationRef` — structurally unconstrained — so this is not a schema violation. The semantic widening of `target` ("the thing being realized") is legitimate; a future PR may rename `target` → `realizes` for clarity or consider unifying OperatorRealization/BehaviorRealization/etc. into a general `TargetCarrierRealization`. Out of scope for DB-14.

### Emission dispatch

When an emitter renders a `Transform` node whose target is `Callable(decl_id)`:

```
decl = dag.declaration(decl_id)
if decl.connective is TypeConnective::Arrow { body: ExternalRealization(self_id), .. }:
    # E-9: Arrow.body is the single authority.
    # self_id == decl_id by R4 invariant (self-reference); assert and use decl_id.
    realization = find_realization_in_active_spec(decl_id, current_language)
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

`find_realization_in_active_spec(accessor_id, language)`: walks `BehaviorRealization` data items in the loaded Dag, returns the one whose `target == accessor_id` and `language == current_language`. If multiple match or none match for a used accessor, emission fails closed.

`render_carrier_template`: string substitution of `{arg_name}` → rendered input expression. Aligned with existing `rust_let_stmt` / `rust_if_expr` template handling in `emit_rust.rs::render_template`.

Wired in all three current emit files. When Lane 1e collapses emitters into a single walker + per-target specs, this becomes one site. **E-9 holds identically** — the structural fact is on Arrow.body; emission-layer consolidation doesn't touch it.

### Self-reference sanity check

`Arrow.body == ExternalRealization(self_decl_id)` is not a structural cycle. The Arrow's connective carries a body variant whose payload is a DeclarationId that happens to name the same declaration. Consumers walk body → DeclarationId → declaration; the walk terminates because the declaration's body is a terminal variant (`ExternalRealization`), not a recursive structure. No graph traversal revisits the Arrow.

Conceptually: "this Arrow is externally realized; my identity for finding my per-target realization IS me." The self-reference is the explicit form of "the accessor is its own identity," which is exactly what R4 makes structural.

---

## Why this design satisfies all five reviewer concerns (R3 review)

**BLOCKING: illegal states unrepresentable (R3 binding table admitted duplicate/malformed).**
R4 drops the binding table. There's no `{accessor, marker}` pair whose pairing could be wrong. The accessor IS its identity; `substrate_accessors` is a flat list; pre-mutation uniqueness check makes "two rows for the same accessor" a fail-closed diagnostic, not a silent last-write-wins.

**BLOCKING: single-authority metadata (R3 represented accessor→marker twice).**
R4 has no marker. The accessor's DeclarationId serves both roles — as Arrow body's self-reference AND as the spec realization's `target`. One identity, one authoritative source. There's no second representation for the accessor→marker relation to drift against.

**BLOCKING: API-level enforcement (R3 "one binding per accessor" was prose).**
R4's uniqueness is enforced pre-mutation in bootstrap with an explicit diagnostic. Any attempt to list an accessor twice in `substrate_accessors` halts bootstrap before a single Arrow body is rewritten. The "one per accessor" rule is checked by machinery, not by reviewer vigilance.

**Fail-closed: duplicate binding / wrong-kind marker (R3 partially behavioral).**
R4: duplicate → pre-mutation diagnostic, bootstrap halts. Wrong-kind marker → impossible; there are no markers.

**Facts flow forward (R3 was already satisfied, preserved in R4).**
Arrow.body still carries the "externally realized" fact structurally. The walk `Transform → Arrow → body → ExternalRealization(self_id) → decl` reaches emission through the exact thesis-boundary path (`Transform → Arrow` and `Arrow → body`).

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

- **`SubstrateAccessor` marker type + per-accessor `SubstrateAccessorBinding` table** (revision 3). The `accessor → marker` relation ended up represented twice (in the binding table AND in the rewritten Arrow body); "one binding per accessor" was prose, not shape; duplicates admitted last-write-wins at the bootstrap iteration. Replaced by: the accessor declaration IS its own identity; ExternalRealization self-references; one enumeration list of accessors drives bootstrap with structural uniqueness. Rejected per round-3 reviewer feedback.

---

## Open questions

1. **Semantic widening of `BehaviorRealization.target`.** Today used for Behavior kinds (Bind/Branch/Transform/etc.); DB-14 extends to externally-realized Arrow declarations. Legitimate because the field type is `DeclarationRef`, but the name `target` suggests Behavior specifically. Options: rename `target` → `realizes` (broader), or leave and document. A future unification PR may also merge Operator/Behavior realization types into `TargetCarrierRealization`. Neither is in DB-14's scope.

2. **`PythonCallableRealization`'s status.** `python.dag` still uses a Python-specific type (`PythonCallableRealization`) instead of the shared `CallableRealization`. Orthogonal debt item; DB-14 uses `BehaviorRealization` throughout, so it doesn't aggravate this.

3. **List uniqueness at the substrate level.** R4 relies on a pre-mutation check in bootstrap to diagnose duplicates in `substrate_accessors`. If v3's `List<DeclarationRef>` substrate supports set-like uniqueness natively (e.g., `Set<DeclarationRef>`), the check becomes structural rather than imperative. Worth verifying which shape v3's list-or-set substrate currently exposes; if only `List`, the bootstrap-side check stays but is tiny and local.

4. **`emit.dag` migration impact.** When emission moves from `emit_rust.rs`/`emit_go.rs`/`emit_python.rs` into `.dag` (Lane 1e + beyond), the E-9 dispatch rule applies unchanged: `emit.dag` walks Arrow.body and handles the `ExternalRealization` variant structurally. No design refactor required at that boundary.

---

## Acceptance (Lane 1 Stage 1b owns)

- [ ] E-9 invariant present in INVARIANTS.md (this PR lands it alongside DB-14)
- [ ] Three accessor fns (`port`, `node`, `resolve_producer`) declared as Arrows with trivial stub bodies in `substrate.dag`. **No `SubstrateAccessor` type; no `SubstrateAccessorBinding` table.**
- [ ] Single `data substrate_accessors: List<DeclarationRef> = [port, node, resolve_producer]` in `substrate.dag`
- [ ] `bootstrap.rs::upgrade_substrate_accessor_bodies` implemented: (a) pre-mutation uniqueness check on `substrate_accessors` — duplicates are a fail-closed diagnostic; (b) for each accessor, rewrite Arrow body to `ExternalRealization(self_id)` (self-reference); non-Arrow connectives fail closed.
- [ ] Three per-target `BehaviorRealization` data items in `src/v3/spec/rust.dag`, `go.dag`, `python.dag` (9 total); each `target` field references the accessor declaration **directly** (no marker indirection)
- [ ] Post-bootstrap invariant test: each accessor's `Arrow.body == ExternalRealization(self_id)` where `self_id` matches the accessor's own DeclarationId. Mirrors pipeline.dag's invariant test (DB-16 §3).
- [ ] Duplicate-list regression test: a test fixture adds `port` twice to `substrate_accessors`, bootstrap emits the expected fail-closed diagnostic pointing at the duplicate
- [ ] `dag.nodes()` delta from substrate.dag load + bootstrap is bounded — zero Callable Transforms contributed by the three accessor declarations
- [ ] Emission dispatch on `ArrowBody::ExternalRealization` wired in `emit_rust.rs`, `emit_go.rs`, `emit_python.rs`
- [ ] Fail-closed emission test: a program that calls a substrate accessor whose declaration has no realization in the active target's spec produces a diagnostic naming the (target, accessor) pair
- [ ] Three existing lenses (`complexity.dag`, `provenance.dag`, `unused_parameters.dag`) migrate to call `port(d, id)` / `node(d, id)` / `resolve_producer(d, id)`; oracle tests pass
- [ ] Line count reduction in `src/v3/lenses/*.dag` ≥ 15%
- [ ] INVARIANTS.md L-7 landed (lenses don't reconstruct lookup locally) — separate from E-9

---

## Associations

- **[E-9 (INVARIANTS.md)](./invariants/e-9-external-realization-lives-on-arrow-body.md)** — the rule this design implements. Landed in the same PR.
- **Lane 1 Stage 1b** ([lane1-stage-b-substrate-keyed-lookup.md](./lane1-stage-b-substrate-keyed-lookup.md)) — the stage that consumes DB-14.
- **DB-5** ([design-substrate-keyed-lookup-api.md](./design-substrate-keyed-lookup-api.md)) — specifies the three accessor signatures (`port`, `node`, `resolve_producer`) this design realizes.
- **DB-16** ([design-fn-external-body-reconciliation.md](./design-fn-external-body-reconciliation.md)) — companion: clarifies that `FnExternalBody` + bootstrap-rewrite pattern applies to both pipeline stages AND substrate accessors.
- **`src/v3/compiler/pipeline.dag`** — the production template; DB-14 uses the exact same parser/bootstrap pattern.
- **`src/v3/compiler/src/bootstrap.rs::upgrade_pipeline_stage_bodies`** — the production implementation DB-14 mirrors.
- **`src/v3/std/emit_model.dag::BehaviorRealization`** — the existing realization shape DB-14 reuses.
- **`src/v3/compiler/src/dag.rs::ArrowBody::ExternalRealization`** — the existing substrate variant DB-14 places on accessor Arrow bodies.
- **`src/v3/compiler/src/infer.rs::is_realization_shape`** — existing dispatch-time check; DB-14 doesn't extend this (markers aren't inferred, they're looked up at emission).
