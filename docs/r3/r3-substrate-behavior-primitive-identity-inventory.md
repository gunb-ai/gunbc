# R3 — Behavior → primitive-identity wiring inventory (HEAD-state)

**Status:** **DESCRIPTIVE READINESS** — worker-authored input for Substrate Mgr (warm-wolf-698 #2068) canvas-pair authoring (Q-Lens-Target-Context + Q-Cost-Composition-Layering) tracked at #2175. **Non-normative.** No prescription on canvas-shape, target-context routing, or composition-layering disposition.

**Authority:** Substrate Mgr disposition (b) on #2068 (deep-crane-22 inbox #2247 reply), paralleling PR #2218 / zesty-moth-793 `r3-v-valuebody-variant-inventory.md` precedent. Canvas authoring remains Mgr scope; this doc reports current-state wiring at HEAD only.

**Sources at HEAD (worktree `session/deep-crane-22`):**
- Behavior coproduct: `src/v3/std/substrate.dag:466-471`
- TransformTarget: `src/v3/std/substrate.dag:312-318`
- Per-variant nodes: `src/v3/std/substrate.dag:409-462`
- Substrate markers (.dag): `src/v3/spec/v3_l1.dag:57-62`
- Realization meta-types (.dag): `src/v3/std/emit_model.dag:17-121`
- Behavior realization rows (.dag, Rust): `src/v3/spec/rust.dag:1184-1203`
- Cost-lens consumer: `src/v3/lenses/cost.dag:69-185`
- Rust-side emit indexes: `src/v3/compiler/src/emit/rust_target.rs:535-580`
- Rust marker→DeclarationId resolution: `src/v3/compiler/src/dag.rs:4215-4222`
- Python-side emit indexes: `src/v3/compiler/src/emit/python_target.rs:144-148`

## 1. Behavior variant census (substrate `.dag` side)

`Behavior` is a 5-variant terminal coproduct at `src/v3/std/substrate.dag:466-471`:

| # | Variant | Payload (substrate type) | Payload location | Primitive-identity anchor (HEAD) |
| --- | --- | --- | --- | --- |
| 1 | `Value(ValueNode)` | `ValueNode { id, payload: LiteralBits, result_port, span, lane2_workflow }` | `substrate.dag:409-415` | none direct — payload is a `LiteralBits` literal; identity (if any) flows through declared-type of the producing port |
| 2 | `Transform(TransformNode)` | `TransformNode { id, target: TransformTarget, inputs, result_port, span }` | `substrate.dag:417-423` | **`target.Callable(DeclarationId)`** / `target.FieldProject{field_label, field_child: DeclarationId?}` / `target.Operator(OperatorKind)` (substrate.dag:312-318) |
| 3 | `Branch(BranchNode)` | `BranchNode { id, input, paths: List<BranchPath>, result_port, span, emit_participation: BranchEmitParticipation? }` | `substrate.dag:430-437` | per-path `pattern: BranchPattern = UnresolvedVariant{name,span} \| ResolvedVariant(DeclarationId)` (substrate.dag:322-324); plus L1 marker `Branch` for emit dispatch |
| 4 | `Loop(LoopNode)` | `LoopNode { id, source, init, body: NodeId, bound: LoopBound, result_port, span }` | `substrate.dag:439-447` | `body: NodeId` (graph-relative, not a declaration); `bound: LoopBound = Cardinality{count: PortId} \| Descent{cluster: ClusterId, measure: PortId}`; L1 marker `Loop` for emit dispatch |
| 5 | `Bind(BindNode)` | `BindNode { id, name: String, result_port, params: List<PortId>, span, lane2_workflow, emit_participation: BindEmitParticipation? }` | `substrate.dag:455-462` | `name: String` (no DeclarationId on the node itself; resolved at use); L1 marker `Bind` for emit dispatch |

## 2. L1 substrate markers (`src/v3/spec/v3_l1.dag:57-62`)

Empty Conj sentinels — typed handles only, no payload:

```
type ValueBehavior {}
type Transform {}
type Branch {}
type Loop {}
type Bind {}
type Main {}
```

These are the per-variant **DeclarationRef anchors** that target spec rows (`BehaviorRealization`, etc.) reference via `target: DeclarationRef`. They mediate the substrate↔emit join: per-target rows attach to the marker's DeclarationId, not to the runtime variant tag directly.

Naming-debt note (file comment at v3_l1.dag:64-67): only `Value` collided with PB-Runtime's union carrier and was renamed `ValueBehavior`; the other four markers stay short.

## 3. Realization meta-types (`src/v3/std/emit_model.dag`)

Five realization carriers consume the Behavior→primitive-identity join Rust-side:

| Meta-type | File:line | Key fields | Used for |
| --- | --- | --- | --- |
| `TypeRealization` | `emit_model.dag:17-24` | `language, target: DeclarationRef, carrier: String, is_copy, fields, cost` | per-target carrier for declared types |
| `OperatorRealization` | `emit_model.dag:100-106` | `language, target, op: DeclarationRef, carrier, cost` | rendering `Transform { target: Operator(_), .. }` |
| `BehaviorRealization` | `emit_model.dag:108-113` | `language, target: DeclarationRef, carrier: String, cost` | rendering substrate behaviors via marker (let / if-else / main) |
| `CallableRealization` | `emit_model.dag:115-121` | `language, target, strategy: CallableStrategy, parameters, cost` | rendering `Transform { target: Callable(decl), .. }` |
| `TypeInstantiationRealization` | `emit_model.dag:197-202` | `language, target, carrier, cost` | rendering generic-template instantiations (e.g., `List<T> -> Vec<T>`) |

All five carry `language: DeclarationRef` (typed pointer to `rust_language` / `python_language` / `go_language`) — emit-side filters by matching against the active target's language-spec declaration id.

## 4. Per-target row population (Rust example, `src/v3/spec/rust.dag:1184-1203`)

Three `BehaviorRealization` rows at HEAD:

```
data rust_let_stmt: BehaviorRealization = {
  language: rust_language
  target: Bind
  carrier: "let {name}: {type} = {value};"
  cost: 0
}

data rust_if_expr: BehaviorRealization = {
  language: rust_language
  target: Branch
  carrier: "if {cond} { {then} } else { {else} }"
  cost: 0
}

data rust_main_wrap: BehaviorRealization = {
  language: rust_language
  target: Main
  carrier: "fn main() { {body} println!({quote}{{}}{quote}, {final}); }"
  cost: 0
}
```

**Coverage gap (descriptive):** of the 5 Behavior variants + Main wrapper, only `Bind` / `Branch` / `Main` have `BehaviorRealization` rows. `Value` / `Transform` / `Loop` are realized through different routes (Value: `LiteralBits` rendering via `LiteralSyntax`; Transform: `OperatorRealization` or `CallableRealization`; Loop: `for_each_syntax` inside `ControlFlowSyntax` per `emit_model.dag:287-296`). The marker→DeclarationId join is therefore not uniformly the per-Behavior-variant authority surface.

## 5. Rust emit-side wiring (`src/v3/compiler/src/emit/rust_target.rs:535-580`)

`RealizationIndexes` carries six `HashMap<DeclarationId, …Binding>` indexes, all built **emit-time** from the per-target `data` rows:

| Index | Key | Value | Source rows |
| --- | --- | --- | --- |
| `types` | `DeclarationId` | `TypeRealizationBinding` | `data rust_*: TypeRealization` |
| `instantiations` | `DeclarationId` | `TypeInstantiationBinding` | `data rust_*: TypeInstantiationRealization` |
| `operators` | `(DeclarationId, DeclarationId)` (operand_type, algebra_field) | `String` | `data rust_*: OperatorRealization` |
| `behaviors` | `DeclarationId` (marker decl) | `String` (carrier) | `data rust_*: BehaviorRealization` |
| `callables` | `DeclarationId` | `RustCallableStrategyBinding` | `data rust_*: CallableRealization` |
| `patterns` | `DeclarationId` | `PatternRealizationBinding` | `data rust_*: PatternRealization` |

The marker→DeclarationId resolution happens once at bootstrap end at `dag.rs:4215-4222` via `Dag::declaration_by_name`:

```
self.substrate_markers.value     = self.declaration_by_name("ValueBehavior").map(|d| d.id);
self.substrate_markers.transform = self.declaration_by_name("Transform").map(|d| d.id);
self.substrate_markers.branch    = self.declaration_by_name("Branch").map(|d| d.id);
self.substrate_markers.r#loop    = self.declaration_by_name("Loop").map(|d| d.id);
self.substrate_markers.bind      = self.declaration_by_name("Bind").map(|d| d.id);
self.substrate_markers.main      = self.declaration_by_name("Main").map(|d| d.id);
```

Downstream emit dispatch reads these typed handles (no name strings cross the boundary post-bootstrap). Python emitter mirrors this with a similar four-index struct at `python_target.rs:144-148` (no `behaviors` / `operators` indexes today — narrower coverage).

## 6. Cost-lens consumer pattern (`src/v3/lenses/cost.dag:69-185`)

The cost lens at HEAD is a **bespoke `compute_symbolic_costs(d: Dag)` Behavior fold**, NOT a `Lens<SymbolicCost>` instance. Per-variant entry derivation at `cost.dag:118-134`:

```
fn entry_for(d: Dag, acc: List<SymbolicCostEntry>, behavior: Behavior) -> SymbolicCostEntry =
  match behavior {
    Value(v)     => { port: v.result_port, cost: hit_symbolic_cost_lookup(ConstantCost(0)) }
    Transform(t) => { port: t.result_port, cost: transform_cost(acc, t.inputs) }
    Branch(b)    => { port: b.result_port, cost: branch_cost(acc, b.input, b.paths) }
    Loop(l)      => { port: l.result_port, cost: loop_cost(d, acc, l) }
    Bind(bind)   => { port: bind.result_port, cost: lookup_cost(acc, bind.result_port) }
  }
```

**Observable wiring shape today:**
- The fold dispatches structurally on the substrate variant (5-arm `match` over `Behavior`).
- It does NOT consume per-target realization rows. Cost values are produced from the substrate-shape alone — `ConstantCost(0)` for `Value` / `Bind` params, `transform_cost` over `t.inputs` (graph topology), `combine_iterate(linear_at(l.source), body_cost(...))` for `Loop`.
- No reference to `TypeRealization.cost` / `CallableRealization.cost` / `BehaviorRealization.cost` fields (the `cost: Int` columns on `emit_model.dag:23, 105, 112, 120, 202`) appears in `cost.dag` at HEAD.
- Author-comment block at `cost.dag:280-322` explicitly cites this gap as #2175 canvas-tier territory: "**.dag-side iteration over `List<CallableRealization>` / `List<TypeRealization>` has zero precedent at HEAD; realizations consumed Rust-side at emit time only.**"

## 7. Substrate-precedent observations (descriptive only)

- **One join works structurally today**: `Transform { target: Callable(DeclarationId) }` at `substrate.dag:313` is a typed-DeclarationId edge directly on the runtime carrier. A `.dag`-side cost lookup `t.target → Callable(decl) → Lookup<CallableRealization>` would not require a new substrate primitive — only `.dag`-side iteration precedent over `List<CallableRealization>`, which (per cost.dag:319) does not exist at HEAD.
- **Other variants lack equivalent typed anchors**: `Value(v)` carries `LiteralBits` (data, not declaration); `Bind(b)` carries `String` (resolved by name at use, not pre-bound); `Loop(l)` carries `NodeId` body (graph-internal, not declaration); `Branch(b).paths[i].pattern` carries `ResolvedVariant(DeclarationId)` per path (per-path, not whole-Branch identity).
- **Marker-mediation is the join surface for emit-side today**: `BehaviorRealization.target: DeclarationRef → {Bind, Branch, Loop, Main, …}` lets `behaviors: HashMap<DeclarationId, String>` key carrier strings off the marker id — a uniform shape across variants, populated only for the variants that need a per-target carrier template (Bind/Branch/Main at HEAD). The marker mediation gives a structural anchor for variants that lack a typed-payload identity (Value, Bind, Loop), but only at emit-template granularity (carrier string + `cost: Int`), not at the Behavior-fold granularity the cost lens operates at.
- **Per-target language filtering**: every realization meta-type carries `language: DeclarationRef` (`emit_model.dag:18, 101, 109, 116, 198`); Rust-side emit indexes filter at construction by matching against the active target's `LanguageSpec` declaration id. A `.dag`-side consumer of realization rows would need analogous active-target context — **this is the open Q-Lens-Target-Context axis** per cost.dag:313-314 (`Lens<C>.read: fn(Dag, Behavior) -> Witness<C>` carries no target context).

## 8. Source-citation summary table (file:line)

| Concern | Authority |
| --- | --- |
| Behavior coproduct (5 variants) | `src/v3/std/substrate.dag:466-471` |
| Per-variant node payloads | `src/v3/std/substrate.dag:409-415, 417-423, 430-437, 439-447, 455-462` |
| TransformTarget (3 variants) | `src/v3/std/substrate.dag:312-318` |
| BranchPattern (2 variants) | `src/v3/std/substrate.dag:322-324` |
| LoopBound (2 variants) | `src/v3/std/substrate.dag:399-401` |
| L1 markers (.dag) | `src/v3/spec/v3_l1.dag:57-62` |
| Realization meta-types (.dag) | `src/v3/std/emit_model.dag:17-24, 100-121, 197-202` |
| Rust BehaviorRealization rows | `src/v3/spec/rust.dag:1184-1203` |
| Rust emit indexes | `src/v3/compiler/src/emit/rust_target.rs:535-580` |
| Rust marker→DeclarationId resolution | `src/v3/compiler/src/dag.rs:4215-4222` |
| Python emit indexes (narrower) | `src/v3/compiler/src/emit/python_target.rs:144-148` |
| Cost-lens Behavior fold | `src/v3/lenses/cost.dag:69-185` |
| Cost-lens canvas-deferral note | `src/v3/lenses/cost.dag:280-322` |

## 9. Out of scope (Substrate-Mgr-authority disclaimer)

This doc does NOT:
- propose canvas options for #2175 (Q-Lens-Target-Context / Q-Cost-Composition-Layering — Mgr authoring)
- propose any wiring substrate-shape (Mgr disposition)
- hand-edit `src/v3/std/substrate.dag` or `src/v3/lenses/cost.dag` (Mgr-tier substrate decisions)
- claim a `Lens<C>.read` refactor shape (Director-deferred to its own canvas cycle per α-narrow ratification at gunbc#828 c#4400772335)
- claim a `cost_lens: Lens<SymbolicCost>` data declaration is authorable today (cost.dag:367-417 explicitly forbids until class-5 data-body lowering)

This doc DOES:
- inventory the 5 Behavior variants + their typed payload-identity anchors at HEAD
- inventory the 5 realization meta-types and their per-target row shapes
- inventory the Rust emit-side index population pattern (HashMap<DeclarationId, …Binding>)
- inventory the cost-lens consumer's current Behavior-fold shape (no realization-row consumption at HEAD)
- cite source file:line precision for each claim

## Footer

Descriptive readiness input only. Canvas-shape ratification is Substrate-Mgr-then-Director authority per #2175 + the α-narrow / β / γ / ε ratification cascade cited in the issue body. Findings 1–4 from #2175's body are independently cross-referenced here at file:line precision (cost.dag:308-322); this doc adds variant-anchor depth + realization-row inventory not previously enumerated in one artifact.

— Authored by deep-crane-22 (worker session) per Substrate Mgr warm-wolf-698 disposition (b) on #2068.
