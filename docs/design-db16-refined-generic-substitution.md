> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Extends: [design-m2-feature-parity.md §DB-11](./design-m2-feature-parity.md) | Unblocks: Lane 3 Stage 3a.3 closure (🟡 Partial → ✅ Shipped)

# Design DB-16 — refined generic substitution (3a.3 closure)

**Design blocker:** DB-16 (refined-generic substitution through the `where`-refinement carrier)
**Consumer:** Lane 3 Stage 3a.3 — the sole remaining blocker for promoting 3a.3 from 🟡 Partial → ✅ Shipped (see `src/v3/ROADMAP.md:477` and `:496`).
**Status:** Design ready for implementer review. Implementation and tests land in a separate Part 2 PR.
**Companion:** [DB-11](./design-m2-feature-parity.md#db-11--where-refinement-predicates-3a3-size-m) — the design whose consumer wiring this doc extends. DB-16 is a *direct extension* of DB-11's substrate shape, composite-canonical refinement form, and flatten-and-subset discharge machinery; it does not introduce parallel authorities.

---

## Scope

DB-16 closes exactly one hole left open by DB-11 consumer wiring (PR #515): **refined generic parameter substitution at call sites**. For a signature of the shape

```
fn f<T>(x: T where pred(x)) -> T
```

called with `f(n)` where `n: Int where pred(n)`, DB-11's existing `predicate_discharges` pipeline would discharge the refinement if it were handed the callee's expected shape as a refined *concrete* `Int`. It is not. `signature_type_shape` (`infer.rs:3381`) currently terminates on the callee's *template T-param decl* before the substitution that would materialize that refined `Int` has a chance to run. DB-16 reorders the walk so substitution precedes the refinement-carrier identity-terminator, and re-attaches the template's refinement to the substituted base. Everything downstream — flatten-and-subset discharge, structural callable identity, `refinement_ports_equal` — stays unchanged.

**Explicitly in scope:**

- Single-parameter refinements (the DB-11 fragment: `where pred(x)` with one scope-bound free variable and a predicate body of `Value` / `Transform` nodes only).
- Single-conjunct and composite (`&&`-joined) refinement roots — i.e., the full fragment `clone_predicate_body` and `predicate_discharges` support in DB-11.
- Substitution through a single `Instantiation` layer and direct `TypeParam` references (the two arms of `signature_type_shape` at `infer.rs:3404–3415`).
- Symmetric discharge at multiple call sites of the same generic template.

**Explicitly NOT in scope (each a separately tracked follow-up or a DB already scheduled):**

- **Multi-parameter refinements** (e.g., `where pred(x, y)` ranging over two scope-bound parameters). DB-11's proof theory is single-scope-bound-var by construction (`narrowable_var_name`); DB-16 inherits that restriction. Extending past it is its own design increment.
- **Narrowing over already-refined generics.** If an arm introduces a further predicate on a T-param that already carries a refinement, the composite-canonical construction from DB-11 should handle it structurally once substitution runs. Document-only: if an edge case surfaces during Part 2 implementation, file a follow-up; do not widen DB-16's scope to pre-address it.
- **Mutual-recursion-across-refined-generics.** Owned by DB-9 (mutual recursion lowering). Out of scope by construction.
- **Emission for narrowed ports with refined generics.** Inherits the Lane 1e dependency already documented in `ROADMAP.md:502`. When Lane 1e's single-emitter consolidation lands, the narrowed-port producer shim covers refined generics as a special case of the same fix.
- **Cross-module refined generics.** Matches DB-11's §Open-question 4 stance: `compiler.dag` work is single-module today; cross-module is its own stage.

---

## Problem statement

The DB-11 consumer-wiring PR (#515) landed a complete discharge pipeline for concrete refined parameters: lowering produces a single composite-canonical refinement `Declaration` per refined port; `signature_type_shape` terminates on that declaration so the callee side keeps the predicate edge visible; `check_refinement_discharge` (`infer.rs:1021`) consults both sides' refinement edges; `predicate_discharges` (`infer.rs:1069`) runs a flatten-and-subset comparison over conjunct leaves; `refinement_targets_equal` (`infer.rs:1218`) compares `TransformTarget::Callable`s structurally; `refinement_predicate_out_of_fragment` (`lower.rs:489`) gates admitted predicate shapes down to what the walker can actually see.

Fourteen `test_3a3_*` tests lock that pipeline in place.

One test case that is *not* in that suite — and whose absence is the blocker — is the refined generic:

```
fn f<T>(x: T where pred(x)) -> T = x
fn caller(n: Int where pred(n)) -> Int = f(n)
```

### The concrete failure

At `caller`'s call to `f`, `decide_transform` (`infer.rs:795`) pulls the callee's input-port expected shape by walking `f`'s parameter declaration through `signature_type_shape`. That declaration carries:
- `connective: Atom(ResolvedIdentifier(T_param_decl))` — aliases the T-param;
- `refinement: Some(pred_decl)` — the lowered `where pred(x)` predicate.

`signature_type_shape` at lines 3390–3402 first checks `decl.name` (None on the anonymous refined-declaration); then hits the DB-11 identity-terminator at lines 3400–3402:

```rust
if decl.refinement.is_some() {
    return Some(TypeShape::new(current));
}
```

and returns the refined-T-param declaration's id as the expected shape. The `match &decl.connective` at lines 3403–3428 — which contains the `TypeParam` and `Instantiation` arms that *would* consult the substitution stack and substitute T → Int — is never reached.

When `check_refinement_discharge` runs at line 980, it compares:
- `expected.declaration`: the callee's refined-T-param declaration.
- `actual.declaration`: the caller's refined-Int declaration.

Both carry refinements. `predicate_discharges` then walks both predicate bodies through `refinement_ports_equal`. The predicates are structurally equivalent (same operator, same call shape, same constant operands), but the parameter slots point into two different declarations (`T_param` on callee, `Int` on caller), and the comparison bottoms out at type-mismatched `FieldProject` / `Callable` targets that reference differently-rooted declaration chains. Discharge fails.

### The expected behavior

`f(n)` above should discharge cleanly. The user's intent is unambiguous: the callee's `T where pred(x)` becomes `Int where pred(x)` after the `T := Int` substitution induced by passing `n: Int where pred(n)`. A hand-authored signature `fn f(x: Int where pred(x)) -> Int` invoked identically discharges today (DB-11 test `test_3a3_call_with_matching_refined_arg_compiles`). The generic form and the authored-concrete form should reach the same discharge outcome — that is what "generic" means.

### Worked example

With the current pipeline:

```
          caller's arg port                  callee's param decl
          ──────────────────                 ────────────────────
  refined_n_decl (connective=              refined_T_decl (connective=
    Atom(ResolvedIdentifier(Int_decl)),      Atom(ResolvedIdentifier(T_param_decl)),
    refinement=Some(n_pred_decl))            refinement=Some(T_pred_decl))
                  │                                       │
                  ▼                                       ▼
   signature_type_shape returns          signature_type_shape returns
   refined_n_decl (identity-term)        refined_T_decl (identity-term)
                  │                                       │
                  └─────────────┬─────────────────────────┘
                                ▼
                 check_refinement_discharge
                                │
                                ▼
                 predicate_discharges over bodies
                  referencing two different param slots
                  rooted in two different base decls
                                │
                                ▼
                         FAILS (structural mismatch
                         on target-decl walks inside
                         refinement_ports_equal)
```

With DB-16's substituted-carrier reorder:

```
          caller's arg port                  callee's param decl
          ──────────────────                 ────────────────────
  refined_n_decl                           refined_T_decl
  (refinement=Some(n_pred_decl))           (refinement=Some(T_pred_decl),
                                            connective=Atom(ResolvedIdentifier(T_param)))
                  │                                       │
                  ▼                                       ▼
   signature_type_shape returns          signature_type_shape sees refinement+TypeParam,
   refined_n_decl                         substitutes T := Int, re-attaches T_pred_decl
                                          (with param slot re-pointed via
                                           clone_predicate_body) to yield a FRESH
                                           substituted_refined_Int_decl
                  │                                       │
                  └─────────────┬─────────────────────────┘
                                ▼
                 check_refinement_discharge
                                │
                                ▼
                 predicate_discharges over bodies both
                 rooted in Int-based param slots
                                │
                                ▼
                         DISCHARGES (flatten-and-subset,
                         unchanged from DB-11)
```

The second diagram's outcome is identical in shape to what the hand-authored `fn f(x: Int where pred(x))` already produces under DB-11. DB-16 is the code path that makes the generic form structurally equivalent to the authored-concrete form.

---

## Design: substituted refinement-carrier model

**Single authority, producer-consumer split.** The substituted refined carrier is constructed at one site — the existing `materialize_callable_signature_instantiations` phase (D2). `signature_type_shape` never writes; it reads the phase's output (D1). The split is explicit in every section below: D2 is the sole producer, D1 is a pure consumer.

### D1 — Read-only lookup gate in `signature_type_shape`

The DB-11 identity-terminator (`infer.rs:3400–3402`) exists to preserve the predicate edge on the callee side when the declaration's connective points at a concrete base. The original rationale is intact: `Int where d != 0` (`connective: Atom(ResolvedIdentifier(Int))`, `refinement: Some(_)`) must not be walked to `Int` before `check_refinement_discharge` runs, or the predicate edge is lost.

But that rationale is about a *concrete base already in view*. When the substitution stack has a binding for the base's ultimate target, the base is not yet in view — it lives behind substitution. Terminating without substituting drops the substitution, not the refinement.

**Substrate shape reminder.** `lower_parameter_refinement` (`lower.rs:330`) produces refined carriers whose connective is `Atom(ResolvedIdentifier(base_decl_id))` — regardless of whether the underlying base is a concrete type or a generic `TypeParam`. For `fn f<T>(x: T where pred(x))`, `base_decl_id` is the `TypeParam` atom declaration itself (generated by `lower.rs:1050`). For `fn f(x: Int where pred(x))`, `base_decl_id` is the named `Int` declaration. The distinction DB-16 needs is not visible on the refined carrier's connective directly; it is visible by walking one `ResolvedIdentifier` hop to the base declaration and inspecting its connective.

`signature_type_shape` stays `&Dag` (read-only). DB-16 adds a pre-terminator branch that, when substitution applies, asks: *has the materialize phase already produced a substituted-refined carrier for this `(template_refined, subst)` combination?* If yes, return that carrier. If no, fall through to the DB-11 terminator and let the retry machinery handle it.

```rust
// src/v3/compiler/src/infer.rs — signature_type_shape (DB-16 sketch)
fn signature_type_shape(
    dag: &Dag,
    current: DeclarationId,
    subst: &SubstStack,
    depth: usize,
) -> Option<TypeShape> {
    if depth >= WALK_DEPTH_LIMIT {
        return None;
    }
    let decl = dag.declaration(current);
    if decl.name.is_some() {
        return Some(TypeShape::new(current));
    }

    // DB-16 (3a.3 closure): read-only lookup. When the refinement's
    // base requires substitution, check whether the materialize phase
    // has already produced a substituted-refined carrier and return it
    // if so. Construction lives in D2 (the phase), not here.
    if decl.refinement.is_some()
        && refinement_base_requires_substitution(dag, current, subst)
    {
        if let Some(materialized) =
            find_equivalent_substituted_refined_decl(dag, current, subst)
        {
            return Some(TypeShape::new(materialized));
        }
        // Lookup miss: the materialize phase didn't produce a carrier
        // for this combination (TypeParam unbound at phase time, or
        // the call-site shape wasn't in the phase's walk). Fall
        // through to the DB-11 identity-terminator; downstream
        // `is_retryable_generic_decl` (`infer.rs:986`) classifies this
        // as a retry, matching pre-DB-16 behavior for unresolved shapes.
    }

    if decl.refinement.is_some() {
        return Some(TypeShape::new(current));
    }

    match &decl.connective {
        TypeConnective::Instantiation { .. } => {
            resolve_decl_with_subst(dag, current, subst, depth + 1)
                .map(TypeShape::new)
                .or_else(|| Some(TypeShape::new(current)))
        }
        TypeConnective::Atom(AtomPayload::TypeParam(_)) => {
            if let Some(bound) = subst.lookup(current) {
                signature_type_shape(dag, bound, subst, depth + 1)
            } else {
                Some(TypeShape::new(current))
            }
        }
        // ... remaining arms unchanged
    }
}
```

Two read-only helpers, both `&Dag`:

- `refinement_base_requires_substitution(dag, current, subst)` — walks from the refined carrier through `Atom(ResolvedIdentifier(_))` hops (depth-bounded) and returns `true` iff the walk lands on a `TypeParam` bound in `subst` OR an `Instantiation` whose arguments reference substitution-stack-bound TypeParams. For a concrete refined carrier (`Int where pred(x)`) the walk short-circuits to `false`. For a refined generic with `T` bound, `true`. For a refined generic with `T` unbound, `false` — fall through to identity-terminator + retry.
- `find_equivalent_substituted_refined_decl(dag, template_refined, subst)` — scans `dag.declarations()` for an anonymous refined declaration whose base matches `resolve_decl_with_subst(dag, template_base, subst, 0)` and whose predicate body walks structurally equal to what cloning the template's body with the given `subst` would produce. Mirrors `find_equivalent_anonymous_instantiation` (`infer.rs:3484`). Pure read; no allocation.

Three properties this sketch preserves:

1. **Unrefined declarations take the same code path they always did.** The `decl.refinement.is_some()` guard brackets only the refined case; the rest of the function is unchanged.
2. **Concrete refined declarations** see the base-walk short-circuit to `false` and fall through to the DB-11 identity-terminator unchanged. All 16 `test_3a3_*` tests exercise concrete refined types; none see a behavior change.
3. **The new lookup fires only when (a) substitution actually applies AND (b) the phase has pre-materialized a carrier.** Unbound TypeParams, missing materializations, and out-of-bounds walks all fall through to the retry machinery — `signature_type_shape` never synthesizes a carrier itself.

### D2 — Construction: `concretize_decl_with_subst`'s new refinement branch

The substituted-refined carrier is constructed during `materialize_callable_signature_instantiations` (`infer.rs:2236-2264`) — the existing inference phase that already walks every call-site Transform with a `Callable` target, builds the substitution stack from the call's arguments, and invokes `concretize_decl_with_subst` (`infer.rs:2706`) on each of the callee's inputs and output declarations. DB-16 extends `concretize_decl_with_subst` with a new branch that fires when the declaration being concretized carries a `refinement` edge. The branch produces a fresh anonymous `Declaration` structurally identical to what `lower_parameter_refinement` (`lower.rs:269`) would produce if the user had written `fn f(x: Int where pred(x))` directly.

This is the **sole construction site** for substituted refined carriers. The phase holds `&mut Dag`. `signature_type_shape` (D1) never writes. `check_refinement_discharge` (D5) never writes. Any other consumer that needs a substituted-refined shape reads it from the Dag via D1's lookup.

**Construction walk (in the new `concretize_decl_with_subst` refinement branch):**

1. **Resolve the substituted base.** Reuse `resolve_decl_with_subst` (`infer.rs:3432`) — it already handles the `TypeParam` + `Instantiation` + `ResolvedIdentifier` chain. Called with the refined carrier's own `DeclarationId`, it traverses the `Atom(ResolvedIdentifier(base))` hop, then the base's own connective, and returns the concrete substituted base (e.g., `Int_decl` when `T := Int` is in `subst`). D1's gate has already established that substitution is required and that the relevant TypeParam is bound; resolution failure at this step is an internal inconsistency, not a legitimate absence. **Fail closed per C-8:** attach `Diagnostic::ResolveError { name: "refined-generic substitution: substituted base did not resolve" }` on the refined decl's span and return `template_refined`. Returning the template carrier preserves downstream type shape — `signature_type_shape`'s retry machinery takes over — while the diagnostic surfaces the substrate-integrity violation instead of masking it as "inference couldn't resolve."
2. **Extract the refinement's predicate slots.** Read the refinement pointer `decl.refinement` (guaranteed `Some` here by D1's gate) and decompose the predicate declaration into `(original_param_port, original_body_port)` using the same read pattern as `predicate_info` (`infer.rs:1168`) and `outer_predicate_slots` (`lower.rs:736`). If the shape is not a well-formed `Arrow { body: UserDefined(bind) }` with exactly one param slot, this is a substrate-integrity violation: `lower.rs` builds this shape exclusively, so reaching step 2 with any other shape means either lowering is broken or substrate invariants have drifted. **Fail closed per C-8:** attach `Diagnostic::ResolveError { name: "refined-generic substitution: malformed predicate shape" }` on the predicate decl's span and return `template_refined`. Silent degradation is explicitly rejected — the whole point of DB-16 is to surface these facts, not to hide them behind inference retries.
3. **Allocate a fresh composite parameter port typed as the substituted base.** Same allocation pattern as `build_narrowed_refinement` (`lower.rs:635–639`). The new port's type is the substituted base declaration — which may be a named concrete type (`Int`) or another nested `Instantiation` that downstream walks can resolve.
4. **Clone the refinement body into the fresh slot, substituting through Transform targets.** Invoke `clone_predicate_body` (`lower.rs:762`) — **extended with a new `subst: &SubstStack` parameter** — with `source_port = original_body_port`, `substitute_from = original_param_port`, `substitute_to = fresh_param_port`, and the active substitution stack. Depth-bounded by `DEPTH_LIMIT = 64`. The walk produces a fresh sub-DAG that references the fresh param port everywhere the original referenced the template param slot, **and, crucially, concretizes any declaration-level references embedded in `Transform` targets by routing them through `concretize_decl_with_subst`**:

   - **`Operator(_)`:** no substitution. Operator kinds are intrinsic; their typing is resolved via `resolve_operator_arrow`, which already handles TypeParam operands through its own substitution pipeline.
   - **`FieldProject { field_label, field_child }`:** substitute `field_child` via `concretize_decl_with_subst`. `field_child` is the record declaration the projection is defined on. When the record is generic (e.g., a projection on `Cons<T>` from a predicate scoped under `fn f<T>(x: List<T> where ...)`), `field_child` carries the template's `T`; substitution re-roots it to the concrete instantiation (e.g., `Cons<Int>`).
   - **`Callable(id)`:** substitute `id` via `concretize_decl_with_subst`. When `id` is an `Instantiation` whose arguments reference TypeParams bound in `subst`, the result is a fresh Instantiation with concretized arguments (e.g., `always_false<T -> S_f_param>` → `always_false<T -> Int>`, deduped via `find_equivalent_anonymous_instantiation`). When `id` is a bare callable or carries no substitution-relevant TypeParam references, the substitution is a no-op.

   Walks `Value` and `Transform` nodes only. `Branch` / `Loop` / `Bind` encounters at this stage are a substrate-integrity violation: `refinement_predicate_out_of_fragment` (`lower.rs:489`) rejects these shapes at lowering time, so any predicate body reaching the materialize phase is in-fragment by construction. **Fail closed per C-8:** attach `Diagnostic::ResolveError { name: "refined-generic substitution: out-of-fragment predicate body reached materialization" }` on the refined decl's span and return `template_refined`. A silent fallthrough here would mask disagreement between the lowering gate and the materialization walk — a design-level coherence bug worth surfacing.

   **Why the Transform-target substitution is load-bearing (not an optional hardening).** Without it, a predicate body containing a generic helper call — e.g., `fn f<S>(d: S where always_false(d, d)) -> S` where `always_false` is generic — would, after cloning, still carry a `Callable(Instantiation{template: always_false, arguments: [T -> S_f_param]})` reference. At a call site `f(n: Int where always_false(n, n))`, the caller's refined-Int carrier would reference `Instantiation{template: always_false, arguments: [T -> Int]}`. Structural comparison via `declaration_shapes_equivalent` (`infer.rs:3579-3618`) bottoms out at atom-to-atom for the Instantiation-argument comparison and returns `false` on `S_f_param` (TypeParam) vs `Int_decl` (concrete) — discharge silently fails. The Transform-target substitution closes this hole by ensuring the cloned body is *literally* the body the user would have written if they had declared the refinement with the concrete type directly. This preserves the "Facts Flow Forward" axis the rest of the design is grounded on: substitution carries the concrete fact into every decl-level reference inside the predicate body, not just the parameter slot.
5. **Wrap the cloned body in a fresh Bind + predicate-Arrow `Declaration`.** Same Bind shape `lower_parameter_refinement` produces (`lower.rs:300–307`): `params: [fresh_param_port]`, `value: cloned_body_port`, `name: "<refinement:substituted>"`. Wrapped in an Arrow `Declaration` whose `inputs: [substituted_base_decl]`, `output: bool_decl`, `body: UserDefined(fresh_bind_id)` — matching `lower.rs:310–324`.
6. **Allocate the substituted-refined-carrier `Declaration`.** `connective: Atom(ResolvedIdentifier(substituted_base_decl))`, `refinement: Some(fresh_pred_decl)`, all other fields defaulted — structurally identical to what `lower_parameter_refinement` returns (`lower.rs:326–337`).
7. **Return the fresh carrier's `DeclarationId`** wrapped in `TypeShape::new(...)`.

The resulting declaration cannot be distinguished from a user-authored concrete refined declaration by any downstream consumer: same Arrow-predicate Bind shape, same Bool output type, same connective, same refinement edge, same conjunct-leaf decomposition. `check_refinement_discharge` → `predicate_discharges` → `body_discharges` → `collect_conjunct_leaves` → `refinement_ports_equal` sees the same substrate on both sides.

### D3 — Write-access constraint on `signature_type_shape`

DB-11's `signature_type_shape` takes `dag: &Dag`. DB-16's materialization needs `&mut Dag` to allocate the fresh substituted-refined declarations. Decision locked (replaces the option-(i)/option-(ii) branch originally sketched here):

**Materialization runs in the existing `materialize_callable_signature_instantiations` phase (`infer.rs:2236-2264`), which holds `&mut Dag`.** The phase already walks every call-site Transform with a `Callable` target, pushes the call's arguments onto a `SubstStack`, and invokes `concretize_decl_with_subst` (`infer.rs:2706`) on each of the callee's input and output declarations to pre-materialize the specialized signature. DB-16 extends `concretize_decl_with_subst` with a branch for refinement-bearing declarations: the branch walks the base through substitution, clones the predicate body with Transform-target substitution per D2 step 4, and allocates the fresh substituted-refined `Declaration`. Downstream, `signature_type_shape` stays `&Dag` (no widening) and reads the already-materialized carrier through a content-keyed lookup.

**Single authority, not parallel side table.** Every substituted-refined carrier is a canonical `Declaration` in the Dag — same visibility, same consumer surface as any user-authored refinement. There is no infer-local "pseudo-shape" that only some consumers see.

The lookup is `find_equivalent_substituted_refined_decl(dag, template_refined, subst)`: a linear scan over `dag.declarations()` matching on (a) `connective == Atom(ResolvedIdentifier(substituted_base))` where `substituted_base = resolve_decl_with_subst(template_base, subst, 0)`, and (b) `refinement: Some(pred)` whose body walks **strictly structurally equal** to the cloned-and-substituted predicate body under the substitution. The equivalence relation is `predicate_bodies_equal_under_subst` — a strict lockstep walker modeled on DB-11's `refinement_ports_equal` (not on `predicate_discharges`, which is composite-subset and would over-match). Transform-target comparison within the walk goes through `callable_decls_equal_under_subst`, which handles template-side Instantiations carrying reattachment artifacts (see `normalized_instantiation_args` below). Dedup mirrors `find_equivalent_anonymous_instantiation` (`infer.rs:3484`).

Two distinct call sites that require the same (template, substitution) combination end up sharing one `DeclarationId`, not producing two structurally-equivalent copies. The dedup also matches user-authored concrete refined carriers: when the caller's own `where` clause produces a structurally-equivalent carrier (e.g., `caller(n: Int where pred(n))` when the callee is `f<T>(x: T where pred(x))`), the scan returns the caller's carrier and no new carrier is allocated. This is a stronger guarantee than "one carrier per (template, subst)": it is "one carrier per structural equivalence class, inclusive of user-authored forms."

**`normalized_instantiation_args`: self-binding-only filter.** Template-side Instantiations produced by `resolve_callable_target` under outer generic scopes can carry self-bindings `[X → X]` as reattachment artifacts when an outer TypeParam unifies with itself pending further inference. These are no-op under `SubstStack::lookup` (which short-circuits to `None` on self-bindings) but inflate argument-length comparisons in strict structural checks. `normalized_instantiation_args` strips **only** those self-bindings before comparison; non-self retained callable arguments carry semantic identity per `retained_template_arguments_for_target` and are preserved. Two Instantiations that differ only by a non-self retained binding correctly compare unequal.

This closes the chatgpt-review concern on D3/D7 being a "parallel semantic side table": the cache is the Dag itself, keyed by strict structural equivalence, consumed through the same boundary every Declaration is consumed through. Any consumer of DB-16's output — `check_refinement_discharge`, emission, analysis lenses — sees a canonical `DeclarationId`.

### D4 — Why this is structurally identical to a user-authored concrete refinement

The load-bearing claim — and the reviewer-facing one — is that the fresh substituted-refined declaration produced by D2 is **substrate-equivalent** to what `lower_parameter_refinement` produces for `fn f(x: Int where pred(x))`. Argument:

1. **Same connective shape.** `Atom(ResolvedIdentifier(Int_decl))` in both cases (D2.4 allocates exactly this; `lower_parameter_refinement` line 330 allocates exactly this).
2. **Same refinement-declaration shape.** Arrow with `inputs: [Int_decl]`, `output: Bool_decl`, `body: UserDefined(bind)` in both cases (D2.3 vs `lower_parameter_refinement` lines 313–317).
3. **Same Bind shape.** `params: [param_port]`, `value: body_port` in both cases (D2.3 vs `lower_parameter_refinement` lines 301–307).
4. **Same body sub-DAG (up to concretized decl-level references).** `clone_predicate_body` reproduces the original predicate's `Value` and `Transform` node topology, re-points the parameter slot, AND concretizes every declaration-level reference embedded in Transform targets through the active substitution stack (D2 step 4). For a predicate whose body contains only `Operator` targets over primitives (the common case — `==`, `!=`, `>`, etc.), the walk reduces to DB-11's identity-cloning because no Transform target carries a substitution-relevant reference. For a predicate containing `Callable` or generic-record `FieldProject` targets whose decl-level references were tied to the outer template's TypeParams, the walk concretizes those references — the resulting body is structurally identical, under `declaration_shapes_equivalent`, to the body the user would have authored if they had declared the refinement on the concrete type directly. The DB-11 narrowing technique (`build_narrowed_refinement`) continues to use the degenerate empty-substitution form, so the DB-11 acceptance suite (`test_3a3_narrowed_already_refined_param_preserves_outer_refinement`, `test_3a3_conjunction_discharge_ignores_grouping`) sees no behavior change.

Any downstream consumer that walks the new carrier (discharge, narrowing, shape-equality, emission) sees a substrate shape already covered by DB-11's invariants. No new consumer contract is introduced.

### D5 — Discharge consumes the substituted carrier unchanged

`check_refinement_discharge` receives two `TypeShape`s; after D1+D2, both sides now resolve to concrete-refined declarations. The flow through `predicate_discharges` → `body_discharges` → `collect_conjunct_leaves` → `refinement_ports_equal` is identical to the concrete-concrete case DB-11 already handles. Specifically:

- **Flatten-and-subset** (DB-11 `body_discharges`) works unchanged. Conjunct decomposition ignores parameter-slot identity; it operates on the `Transform(Logical(And), ...)` tree shape alone.
- **`refinement_ports_equal`** compares leaves pairwise, using the refinement's own `Bind.params[0]` as the parameter sentinel on each side. Both sides' Binds carry their own fresh param port — which is exactly what DB-11 already guarantees for concrete-refined pairs (see the acceptance test `test_3a3_callable_predicate_structural_identity_across_sites` — distinct ports, structural equivalence).
- **`refinement_targets_equal`**, via `declaration_shapes_equivalent`, handles `Callable(Instantiation{...})` targets structurally. Confirmed by reading `infer.rs:3579–3631`: the `Instantiation` / `Instantiation` arm compares template + argument structure recursively. A call-site-materialized fresh `Instantiation` on one side vs. an authored `Instantiation` on the other pass this check today. No DB-16 extension needed; confirmation test added in §Acceptance.

### D6 — Preserves the no-entailment commitment

DB-11's core proof-theory claim (`design-m2-feature-parity.md` §DB-11 Q2) is that discharge is pure structural equality over resolved predicate expression DAGs — no SMT, no implication, no ordering, no algebra entailment. DB-16 preserves this commitment verbatim:

- D2 constructs the substituted carrier by **cloning the original predicate body with forward substitution**, not by transforming, simplifying, or reasoning about it. The sub-DAG after cloning is the same sub-DAG before cloning, with (a) the parameter slot re-pointed and (b) decl-level references inside Transform targets concretized through the substitution stack. Substitution is the categorical operation that makes `T := Int` write `Int` everywhere `T` appeared — it is not an inference step, not a logical implication, not an ordering claim. It is the operation that makes a generic template become the concrete instance it stood for.
- D5 reuses DB-11's flatten-and-subset discharge unchanged. No new comparison modes. No new predicate fragment. No entailment relation across predicates.
- The expected-side and actual-side carriers are compared by the same pairwise structural walk.

In particular, DB-16 does not let a refined-generic `f<T>(x: T where pred_a(x))` accept an `Int where pred_b(x)` argument where `pred_b` structurally differs from `pred_a` — that case fails discharge today for concrete refinements, and it continues to fail under DB-16 because the substitution at D2 preserves `pred_a`'s body verbatim; only the parameter-slot reference is re-pointed.

**Invariants preserved:**

- Strict Forward Progress (`INVARIANTS.md:425`) — all walks depth-bounded by `WALK_DEPTH_LIMIT` and `DEPTH_LIMIT`, as in DB-11.
- Decidability (`INVARIANTS.md:482`) — discharge remains pure structural equality over finite predicate bodies. No unbounded inference.
- Bounded kernel — no new substrate variant, no new node shape, no new declaration shape. DB-16 is pure consumer wiring + reorder inside `signature_type_shape`.
- DB-11's five-Behavior substrate-integrity lock-in (`test_3a3_substrate_integrity_behavior_still_five_variants`) — unchanged by construction.

### D7 — Dedup via structural match over Dag declarations (detail)

Because the materialize phase may run across fixpoint iterations and `signature_type_shape` may look up the substituted carrier many times during inference, allocating a fresh carrier on every materialization would produce a DAG with many structurally-equivalent-but-not-id-equal refined declarations. The dedup is handled by `find_equivalent_substituted_refined_decl` (D3): before allocating, scan `dag.declarations()` for an existing anonymous refined declaration whose base matches the substituted base and whose predicate body walks structurally equal to the cloned body via `refinement_ports_equal`. If found, reuse its `DeclarationId`; otherwise allocate. Mirrors `find_equivalent_anonymous_instantiation` (`infer.rs:3484`).

This is dedup, not a cache. The "memoized state" is the Dag's `declarations` vector itself; there is no parallel side table. The dedup is also not load-bearing for correctness — DB-11's identity-via-structural-comparison makes duplicate declarations discharge each other fine (`test_3a3_callable_predicate_structural_identity_across_sites` demonstrates this) — but dedup is load-bearing for **substrate hygiene** (avoid unbounded growth of the declarations vector across fixpoint iterations) and for **invariant clarity** (canonical `DeclarationId` per substitution, so any consumer reasoning about identity sees a single authority per structural shape).

---

## Implementation pointer

This section is a forward reference only; full implementation + tests ship in the follow-up Part 2 PR.

- **Touches (construction side, D2).** Extend `concretize_decl_with_subst` (`infer.rs:2706`) with a refinement branch: when the declaration being concretized has `refinement: Some(_)` and the base requires substitution, build a fresh substituted-refined `Declaration` via the 7-step walk in D2. Reuses `resolve_decl_with_subst` (`infer.rs:3432`) for the base and extends `clone_predicate_body` (`lower.rs:762`) with a new `subst: &SubstStack` parameter whose Transform-target walk routes `Callable(id)` / `FieldProject.field_child` through `concretize_decl_with_subst`. `clone_predicate_body` and `outer_predicate_slots` (`lower.rs:736`) become `pub(crate)`. DB-11's callers in `lower.rs` pass an empty `SubstStack` and see no behavior change. All diagnostics (D2 fail-closed paths) register via the phase's `&mut Dag`.
- **Touches (consumer side, D1).** `signature_type_shape` (`infer.rs:3381-3430`) gains a read-only pre-terminator branch: when `refinement_base_requires_substitution` fires, call `find_equivalent_substituted_refined_decl` and return the pre-materialized carrier if found; otherwise fall through to the DB-11 identity-terminator. Both new helpers take `&Dag` and allocate nothing. `signature_type_shape` keeps its `&Dag` contract; no widening, no ripple into `resolve_arrow_walk` or its callers.
- **Does NOT touch.** `check_refinement_discharge`, `predicate_discharges`, `body_discharges`, `collect_conjunct_leaves`, `refinement_ports_equal`, `refinement_targets_equal`, `declaration_shapes_equivalent`, `lower_parameter_refinements_phase`, `lower_parameter_refinement`, `narrow_scope_for_predicate`, `build_narrowed_refinement`, `refinement_predicate_out_of_fragment`, `strip_refinement_to_base`. These are DB-11's consumer-wiring authority; DB-16 extends their input surface, not their bodies.
- **Substrate.** No `dag.rs` changes. No new variant, no new edge, no new field. Five-Behavior lock-in preserved.
- **Admitted surface.** Unchanged. `refinement_predicate_out_of_fragment` already gates the admitted fragment to what the walker supports; DB-16 extends what the walker supports for generics, not what's admitted.

---

## Acceptance

Tests land in `src/v3/compiler/tests/m2_feature_parity_test.rs` under the `test_3a4_*` prefix (adjacent to DB-11's `test_3a3_*` suite; the `3a4` prefix signals "3a.3 closure, post-DB-11"). Baseline suite:

1. **`test_3a4_refined_generic_discharges_across_substitution`** — the core case. `fn f<T>(x: T where pred(x)) -> T = x` with `fn caller(n: Int where pred(n)) -> Int = f(n)` compiles cleanly (no diagnostic). Locks the D1+D2 reorder.

2. **`test_3a4_refined_generic_distinct_refinement_rejects`** — negative case. `fn f<T>(x: T where pred_a(x)) -> T = x` called with `n: Int where pred_b(n)` (structurally distinct predicates) produces a refinement-discharge diagnostic at the call site. Locks D6 no-entailment under substitution.

3. **`test_3a4_refined_generic_identity_across_instantiation_sites`** — structural identity. The same template `fn f<T>(x: T where pred(x)) -> T` called from two distinct call sites with matching concrete refined args discharges symmetrically at both. Locks D5's confirmation that the cache / re-materialization produces structurally-equal carriers across sites.

4. **`test_3a4_refined_generic_literal_arg_rejects`** — argument shape. `f(0)` where `0: Int` (no refinement on the literal) against the callee's refined `T` rejects with the "no narrowing branch in scope" diagnostic (the DB-11-unchanged branch of `check_refinement_discharge`). Verifies substitution does not accidentally weaken the callee's refinement when the argument is unrefined.

5. **`test_3a4_refined_generic_retry_on_unbound_type_param`** — fixpoint interaction. A call site where `T` is not yet bound (an earlier-iteration retry case) must still retry rather than fail hard; D1's fallback `_ => return Some(TypeShape::new(current))` arm + `is_retryable_generic_decl` wiring keeps the existing retry semantics. **Deferred to ROADMAP follow-up** (`Landing: DB-16` → `Follow-up — fixpoint-retry explicit test`): the retry-then-succeed outcome is currently implicit-covered by the multi-site and callable-in-predicate bonus tests (both depend on fixpoint convergence through retry iterations); explicit construction of the TypeParam-unbound-then-bound scenario requires synthesized fixpoint-iteration timing and is tracked with a 1-month yellow-flag threshold.

6. **`test_3a4_refined_generic_composite_discharges`** — conjunction interaction. `fn f<T>(x: T where pred_a(x) && pred_b(x)) -> T = x` with `n: Int where pred_a(n) && pred_b(n)` discharges. Locks D5's claim that flatten-and-subset is unchanged under substitution.

7. **`test_3a4_refined_generic_narrowing_composite_discharges`** — narrowing interaction. `fn f<T>(x: T where pred_a(x)) -> T = x` called from inside an arm that narrows `n: Int where pred_a(n)` via `if pred_b(n) then f(n) else ...` discharges (composite `pred_a && pred_b` subsumes callee's `pred_a`). Locks the narrowing+substitution composition.

8. **`test_3a4_refined_generic_substrate_integrity_behavior_still_five_variants`** — substrate lock-in. Mirrors DB-11's `test_3a3_substrate_integrity_behavior_still_five_variants`. `type Behavior` remains at exactly five variants (`Value`, `Transform`, `Branch`, `Loop`, `Bind`) — DB-16 adds no new behavior, no new declaration-shape variant.

9. **`test_3a4_refined_generic_callable_in_predicate_discharges`** — Transform-target substitution (D2 step 4). `fn h<T>(x: T, y: T) -> Bool = 0 == 1` with `fn f<S>(d: S where h(d, d)) -> S = d` called as `f(n)` where `n: Int where h(n, n)` discharges. Locks the Transform-target substitution path: without the substitution, the cloned predicate body's `Callable(Instantiation{template: h, args: [T -> S_f_param]})` would mismatch the caller's `Callable(Instantiation{template: h, args: [T -> Int]})` at `declaration_shapes_equivalent`'s Instantiation-argument bottom and discharge would silently fail. The test is load-bearing against the class of regression named in the codex review on Part 1.

10. **`test_3a4_refined_generic_callable_in_predicate_distinct_template_rejects`** — negative counterpart to #9. Same shape as #9 but the caller's predicate calls a *different* generic helper (`h2` instead of `h`). Discharge must reject — confirms that Transform-target substitution preserves no-entailment (D6): substitution concretizes arguments, but does not weaken identity of the callable itself.

11. **`test_3a4_refined_generic_field_project_in_predicate_discharges`** — `FieldProject.field_child` substitution (D2 step 4, FieldProject arm). Shape:

    ```
    type Box<T> { inner: T, tag: Int }
    fn f<T>(x: Box<T> where x.tag != 0) -> Box<T> = x
    fn caller(b: Box<Int> where b.tag != 0) -> Box<Int> = f(b)
    ```

    `x.tag` lowers to a `Transform` with `target: FieldProject { field_label: "tag", field_child: Box<T>_decl }`. At the call site `f(b)` with `T := Int`, the cloned predicate body's FieldProject target must have `field_child` substituted to `Box<Int>_decl`, matching the caller's refined-Box<Int> carrier's predicate body. Without the substitution, the cloned body references `Box<T>_decl` while the caller's body references `Box<Int>_decl` — discharge fails at the first Transform node comparison. Tag-field-over-Int is used (not `x.inner == x.inner` over generic T) so the operator arm is unambiguously concrete and the test isolates the FieldProject substitution path; pairs symmetrically with #9 (which isolates the Callable arm). Locks the claim in D2 step 4 that FieldProject is in the admitted Transform-target substitution class.

Additional tests MAY be added if Part 2 implementation uncovers an edge case (e.g., nested `Instantiation` layers under the refined-carrier connective, or refined-generic-passed-to-refined-generic) — size remains S+ (bounded).

---

## Rejected alternatives

### RA-1. Option (b): reject refinements on type parameters at lowering

Verdict: **rejected.**

This is the alternative named in `ROADMAP.md:496` alongside option (a). It would amend `lower_parameter_refinements_phase` (`lower.rs:385`) — or `refinement_predicate_out_of_fragment` (`lower.rs:489`) — to emit an "unsupported: refinement on type parameter" diagnostic whenever a `where` clause targets a generic parameter.

**Primary cost: ratchet flip on the admitted-surface gate DB-11 just closed.** Commit `fdc796844` ("DB-11 (3a.3): reject out-of-fragment `where` predicate shapes at lowering") explicitly narrowed the admitted surface to match the supported fragment — its point was to convert silent discharge failures into honest boundary diagnostics while holding the supported fragment constant. Option (b) would do the opposite at the type-parameter boundary: narrow admitted further *to match the non-extension of supported*. Before DB-16, the pipeline admits refined generics but discharge silently fails; option (b) "fixes" this by retroactively un-admitting a surface any user who wrote `fn f<T>(x: T where pred(x))` reasonably expected to work. This flips the admitted-vs-supported ratchet from "narrowing admitted to match supported" (DB-11's stance) to "narrowing admitted to dodge extending supported" (option (b)'s stance) — a regression on PR #515's closure, not a closure of its own.

**Secondary cost: reviewer-named wrong path.** The ChatGPT review of commit `df5fc7b3f` ("flatten-and-subset conjunction discharge") on PR #515 framed the choice explicitly: *"If 3a.3 is intentionally concrete-only, reject refined type-parameter `where` clauses at lowering; if it wants true generic refinements, the next step should be a substituted refinement-carrier model, not another local walker exception."* The reviewer-named correct path is the substituted-carrier model — DB-16 option (a). Option (b) is the reviewer-named "intentional concrete-only" surrender, applicable only if refined generics are being retired as a goal. They are not; they are the 3a.3 closure blocker.

**Tertiary cost: project norm violation.** `feedback_construction_over_ratchets` rules that rejection-as-fix is a gate-pattern (move the error earlier without resolving the modeling gap) and substitution-as-fix is a model-pattern (the correct modeling makes the violation dissolve). Quoting: *"the correct path to reducing complexity violations is to implement the correct modeling first (structural authorities in std/, first-class concept facts), and then violations dissolve naturally. Adding analyzer-local heuristics ... just moves violations slightly upstream without solving them."* Option (b) is exactly the analyzer-local upstream-move; option (a) is the modeling-first dissolution.

### RA-2. Walker-local identity-terminator extension without substitution

Verdict: **rejected.**

This alternative would keep the identity-terminator at line 3400–3402 in place and bolt on special-case logic for refined-TypeParam refined-Instantiation declarations — e.g., a secondary walk that compares template+argument structure alongside predicate bodies, or a conditional that relaxes the id-comparison when one side's base resolves through a subst stack.

Named explicitly because the PR #515 reviewer called this out as the wrong path. The problem: every special case in `refinement_ports_equal` / `refinement_targets_equal` that tries to bridge template vs. concrete through comparison instead of substitution re-introduces exactly the parallel-authority pattern DB-11's composite-canonical form was built to avoid. Substitution is the structural operation that collapses the distinction between template and concrete; the walker then reads one substrate shape, not two. A walker-local extension is the heuristic-patch version of this fix; the substituted carrier is the modeling version. Rejected per `feedback_construction_over_ratchets` and per reviewer consensus.

### RA-3. Defer closure: push DB-11 to "✅ Shipped except refined generics"

Verdict: **rejected.**

Option: leave 3a.3 at 🟡 Partial permanently with an orphan "refined generics deferred" bullet; ship the rest of the arc under "shipped enough." Cost: the orphan bullet has no downstream consumer relying on its closure (DB-9 mutual recursion, Lane 3 Stage 3c self-hosting cycle, Lane 2, Lane 1 are all independent), so it rots — becomes documentation-only. Meanwhile any user who writes `fn f<T>(x: T where pred(x))` gets a silent-failure (without a lowering-time reject) or a ratchet-regression (with one). DB-16 is small — `signature_type_shape` reorder + one helper using existing machinery — and the cost of landing it is less than the cost of the orphan-bullet debt. Rejected.

### RA-4. Substitute-then-compare but without re-attachment

Verdict: **rejected.**

Variant: substitute the callee's refined-T-param declaration's base connective in place *without allocating a fresh substituted-refined carrier*, and have `check_refinement_discharge` do the re-attachment at comparison time.

Construction has to happen somewhere. Doing it at comparison time scatters construction logic across the discharge walker and couples `check_refinement_discharge` to `SubstStack`. Doing it at the existing `materialize_callable_signature_instantiations` phase (D2) keeps construction at the substrate-production site — the phase is already the authoritative place where specialized signatures become canonical Dag declarations (see the existing `concretize_decl_with_subst` flow for `Instantiation` inputs/outputs). D1's `signature_type_shape` lookup then reads the phase's output. One construction authority per substrate shape, matching DB-11's "composite-canonical refinement form" principle. Rejected.

### RA-5. Monomorphize generics pre-discharge

Verdict: **rejected.**

Pre-flatten every generic function into per-argument-type clones at lowering time, then discharge works concretely and DB-16 becomes trivially unnecessary. Cost: violates the template + substituted-arguments identity model `retained_template_arguments_for_target` (`lower.rs:3450`) was built around. DB-12 explicitly kept generics as template-not-monomorphized. Rejected.

---

## Associations

- **DB-11** ([design-m2-feature-parity.md §DB-11](./design-m2-feature-parity.md#db-11--where-refinement-predicates-3a3-size-m)) — the design DB-16 extends. DB-11's substrate shape, composite-canonical refinement form, flatten-and-subset discharge, and out-of-fragment gate are all consumed unchanged. DB-16's new code is one construction branch in `concretize_decl_with_subst` (D2) plus one read-only lookup gate in `signature_type_shape` (D1); no DB-11 function body is modified.
- **DB-12 / 3a.4** — generic surface parsing / lowering, on which refined generics depend syntactically. DB-16 is conceptually downstream of DB-12 (refined generics are the intersection of DB-11 and DB-12) but does not edit DB-12's code paths.
- **DB-1 Correction shape** ([design-correction-shape.md](./design-correction-shape.md)) — RA-1's rejected lowering-time diagnostic would emit a Correction. DB-16 retains DB-11's existing call-site discharge diagnostic shape unchanged.
- **`src/v3/compiler/src/infer.rs`** — `concretize_decl_with_subst` (construction site, new refinement branch), `materialize_callable_signature_instantiations` (phase driver, unchanged but now exercised for refined generics), `signature_type_shape` (consumer site, gains read-only lookup gate), `resolve_decl_with_subst` + `SubstStack` (reused), `check_refinement_discharge` / `predicate_discharges` / `refinement_ports_equal` / `refinement_targets_equal` / `declaration_shapes_equivalent` (downstream consumers, unchanged).
- **`src/v3/compiler/src/lower.rs`** — `clone_predicate_body` (gains a `subst: &SubstStack` parameter; called from the new `concretize_decl_with_subst` refinement branch) and `outer_predicate_slots` (reused for slot extraction); both become `pub(crate)`. `lower_parameter_refinement`, `narrow_scope_for_predicate`, `build_narrowed_refinement`, `refinement_predicate_out_of_fragment` (DB-11 authorities, unchanged — `build_narrowed_refinement`'s existing call to `clone_predicate_body` passes an empty `SubstStack`).
- **`src/v3/compiler/src/dag.rs`** — `Declaration` shape (unchanged; `refinement: Option<DeclarationId>` edge consumed, not extended).
- **ROADMAP anchor** (`src/v3/ROADMAP.md:477`, `:496`) — DB-16 closing is the sole gate on 3a.3's 🟡 → ✅ promotion. Part 2 PR flips the status row.
- **`INVARIANTS.md`** — strict forward progress (§425) and decidability (§482) preserved verbatim; DB-16 adds no unbounded walk and no new proof-theoretic reasoning.

---

## Open questions

1. **~~`signature_type_shape` `&Dag` vs `&mut Dag`~~** (D3) — **resolved.** Phase-based materialization wins: DB-16 extends `concretize_decl_with_subst` (already called from `materialize_callable_signature_instantiations`, which holds `&mut Dag`). `signature_type_shape` stays `&Dag`; no ripple. See revised D3.

2. **`clone_predicate_body` API extension.** Currently private to `lower.rs` and takes no substitution context. DB-16 both (a) widens its visibility to `pub(crate)` and (b) adds a `subst: &SubstStack` parameter whose Transform-target walk routes `Callable(id)` and `FieldProject.field_child` through `concretize_decl_with_subst`. DB-11's existing callers in `lower.rs` (`build_narrowed_refinement`) pass `&SubstStack::new()` and see no behavior change — the substitution lookup on an empty stack is unconditionally `None`, so concretization is a no-op on every decl walk. Mechanical caller update; covered by DB-11's existing 16 `test_3a3_*` tests as a regression guard. Final signature locked in Part 2.

3. **Interaction with `is_retryable_generic_decl`** (`infer.rs:986`). The retry path handles unbound TypeParams in the `actual` or `expected` shape. D1's `refinement_base_requires_substitution` gate returns `false` when the underlying TypeParam is unbound, so `signature_type_shape` falls through to the identity-terminator and returns the template carrier id — exactly the pre-DB-16 behavior that the retry path currently accepts. Part 2 confirms the shape reaches `is_retryable_generic_decl` and is classified correctly; no design change expected.

4. **Emission narrowed-port shim interaction** (noted in ROADMAP follow-up at `:502`). Once Lane 1e lands the single-emitter consolidation and the narrowed-port producer shim, check that the shim handles substituted-refined carriers as a special case of concrete-refined ports — which, by D4's structural-identity argument, it should for free. Verification only.

5. **Nested `Instantiation` under a refined carrier.** `List<Int> where pred(x)` — the connective is `Instantiation { template: List, arguments: [{ T -> Int }] }` and the refinement is a predicate over lists. D2 walks through `resolve_decl_with_subst`, which handles nested `Instantiation` via recursion. Part 2 adds a targeted test to confirm; likely no code change beyond what's already sketched.

---
