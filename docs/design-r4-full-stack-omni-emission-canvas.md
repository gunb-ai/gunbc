---
status: Mgr canvas (R4 substrate-shape questions for Director ratification; surfaced per Director msg_22a1c596 + PM msg_83ce8113 routing 2026-05-13 + operator directive 2026-05-13)
authority parent: R3 Substrate Manager (warm-wolf-698)
authoring date: 2026-05-13
scope: R4 post-R3 (NO implementation pre-R3; canvas-as-deliverable only)
target: `WISHLIST.md §R4 entry "Full-stack-from-one-.dag with React framework substrate"` (PM authors WISHLIST update post-canvas-ratification)
companion: Director-owned path (a) — visceral 4-layer TODO demo using substrate that already exists
---

# R4 Full-stack omni-emission — TS + React substrate canvas

## §0. Status

Director ratified path (b) canvas dispatch via PM msg_83ce8113 relaying msg_22a1c596 on 2026-05-13. Operator directive (verbatim):

> "i wanted to generate a full stack program from one .dag program ... can we integrate with a common frontend framework like react, model the framework/language layers properly, and emit an entire full stack application - including db stuff like sql?"

Operator ratified path (a) + path (b) parallel; Director split:
- **path (a)**: Director-owned 4-layer TODO demo using existing substrate (already-landed extdeps/rust + OpenAPI + SQL DDL + Markdown emission via gate #28 omni_layers_share_one_node_tree)
- **path (b)** (THIS CANVAS): R4 substrate authoring for TS-as-Shape-A + React-as-framework-substrate

**Scope hard-bound**: canvas-only; no implementation pre-R3 close.

## §1. Authority

- Operator directive 2026-05-13 (verbatim §0)
- PM scoping read msg_8917d867 → 3 paths surfaced
- Operator ratified path (a)+(b) in PM chat
- Director ratification msg_22a1c596 — substrate audit + 5-Q canvas framing
- PM relay msg_83ce8113 routing to Substrate Mgr

## §2. Substrate audit at HEAD (Director-verified)

```
$ ls dsl/extdeps/languages/
bash  dag  go  python  rust
```

- **TS is NOT present** — path (b) is genuinely net-new substrate authoring
- Rust is the canonical Shape-A target precedent (`dsl/extdeps/languages/rust/{types,syntax,naming,emit,async}.dag`)
- Gate #28 `omni_layers_share_one_node_tree` is CONSUMER_LANDED + PASSING per `m1_5_omni_shape_b_openapi_test.rs` — one `compile_to_dag` per workflow; multi-target emission (Rust + OpenAPI + Markdown + SQL DDL) shares the same `Dag`
- Gate #18 `numeric_width_refinements_landed` shows the existing language-spec consumer-evidence pattern (Grounding G2 primitive rows in `dsl/extdeps/languages/rust/primitives.dag`)
- No React-shaped carriers exist at HEAD; no `Component` / `Props` / `Hook` / `Lifecycle` / `Effect` types in `dsl/std/` or `dsl/extdeps/`

## §3. Q1 — TS LanguageSpec carrier shape

### Background

Rust's language spec at `dsl/extdeps/languages/rust/` uses a fixed shape:
- `types.dag` declares `InhabitantDecl` rows for which Rust types inhabit which algebraic structures
- `syntax.dag` + `naming.dag` + `emit.dag` carry the emission surface
- `async.dag` carries effect-specific concerns

TS differs structurally:
- **Structural types, not nominal**: TS's type identity is shape-based (`{x: number, y: number}` ≡ `Point` if shape matches); Rust's is nominal (`Point` ≠ `{x: i32, y: i32}` without explicit declaration)
- **Type-erased at runtime**: TS types vanish on emit; Rust types are runtime ABI-load-bearing
- **Class-vs-interface tension**: TS has both nominal (`class`) and structural (`interface`) — DSL ingest direction matters

### Candidate Q1-a — Parallel-to-Rust (5-file mirror)

`dsl/extdeps/languages/typescript/{types,syntax,naming,emit,async}.dag` — strict mirror of Rust shape; structural-vs-nominal distinction is encoded in field metadata on `InhabitantDecl` rows or via TS-specific declaration variant.

Pros:
- Strict-mirror precedent (`feedback_strict_mirror_vs_novel_substrate_fact` applies)
- Reuses existing LanguageSpec consumer (gate #28 + Grounding G2 pattern)
- Cost-of-change-1 for adding a new TS-emitted type

Cons:
- Forces TS's structural-typing semantics into a nominal-typing-shaped slot
- Class-vs-interface duality may need a sum-type extension on `InhabitantDecl`

### Candidate Q1-b — Extended LanguageSpec carrier with structural-vs-nominal axis

Extend `dsl/std/coercion.dag` (or wherever `InhabitantDecl` lives) with a `TypingDiscipline = Nominal | Structural` field. Rust inhabitants set `Nominal`; TS inhabitants set `Structural`. Emit logic branches on discipline.

Pros:
- Encodes the genuine semantic difference at the type level (Practice 2/6)
- Future-extensible (e.g., refinement-typing if a Liquid Haskell target ever lands)
- Single-authority on emission discipline

Cons:
- Touches foundational `std.coercion` substrate (cross-cutting impact)
- May surface migration scope on existing Rust `InhabitantDecl` rows (lazy migration acceptable)

### Candidate Q1-c — Substrate Mgr discretion

(Reserved per Director's pattern in gate #62 / gate #105 canvas chains; not pre-proposed.)

**Mgr recommendation**: Q1-b. The structural-vs-nominal distinction is load-bearing for TS emission (any TS emit must respect it); encoding it in the carrier is cleaner than burying it in lang-specific files. Migration scope is bounded (Rust adds `TypingDiscipline = Nominal` field, structural new TS rows set `Structural`).

## §4. Q2 — React-as-framework-substrate carriers (Shape-A vs Shape-B)

### Background

Director's lean: likely Shape-A given component-graph is **rendering substrate, not data substrate**. Need explicit framing.

R3's Shape-A/Shape-B distinction:
- **Shape-A**: target-language emission (Rust, Python, TS source code)
- **Shape-B**: data-format emission (OpenAPI YAML, SQL DDL, Markdown, JSON Schema)

React components are TS source-code generation that happens to produce a tree-shaped UI runtime. The emitted artifact is `.tsx` source. By that test, Shape-A.

### Candidate carriers

```dag
type Component { name, props: List<PropSpec>, state: List<StateSpec>, body: ComponentBody }
type PropSpec { name, type_ref: TypeRef, required: Bool }
type StateSpec { name, type_ref: TypeRef, initial: Expression }
type Hook { name, kind: HookKind, dependencies: List<Reference> }
type HookKind = UseState | UseEffect | UseMemo | UseCallback | UseContext | UseRef | Custom(Identifier)
type Lifecycle = OnMount | OnUnmount | OnUpdate { triggers: List<Reference> }
type Effect { hook: Hook, body: Expression, cleanup: Expression? }
type ComponentBody = Render { jsx: JSXTree } | Composite { sub_components: List<ComponentRef> }
```

Sketch only — actual ratified shape requires Director disposition. Each carrier needs Practice 4 analysis (most are 🟢 GREEN; `HookKind` is potential 🟡 YELLOW — depends whether Custom needs strict consumer evidence).

### Candidate Q2-a — Shape-A (component as source-code subject)

React components live in `dsl/extdeps/languages/typescript/react/` (or similar `dsl/extdeps/frameworks/react/` adjacent to languages/). The emit target is `.tsx` source. `Component.body` lowers to JSX which lowers to TS source.

Pros:
- Matches the actual emission target (TS source file)
- Aligns with Rust's Shape-A pattern (Rust source emission)
- Composition with #28 omni_layers_share_one_node_tree: one `Dag` → React Shape-A + OpenAPI Shape-B + SQL DDL Shape-B + Markdown Shape-B

Cons:
- React-isms (hooks, lifecycle, effects) are framework-tier abstractions, not language-tier — may want a separate "framework" carrier layer above language

### Candidate Q2-b — Shape-B (component as declarative tree)

React component-tree is data substrate; emission lowers tree → JSX file. Component lives alongside OpenAPI as Shape-B targets.

Pros:
- Component-tree IS declarative data
- Decouples React-shape from TS-source-shape (could emit same tree to a future framework target)

Cons:
- Framework binding (React-specific hooks, lifecycle) doesn't fit Shape-B's data-format positioning
- Emission still passes through TS source — Shape-A intent

### Candidate Q2-c — Hybrid framework-tier (new shape category)

Introduce a third shape between Shape-A (language) and Shape-B (data format): `Shape-F` for **framework-tier**. React fits here naturally — framework-specific but emits through a language (TS).

Pros:
- Captures the genuine architectural layer (framework above language)
- Future-extensible (Vue, Svelte, Express, Django all framework-tier)

Cons:
- Introduces a new shape category — broader R4 substrate impact
- Practice 4 analysis required for the new Shape-F sum/category

**Mgr recommendation**: Q2-a. Pragma: React components ARE TS source code at emit time; the framework-vs-language distinction is internal to the TS lane and doesn't need a new shape category. Practice 4 cleaner.

## §5. Q3 — Ingest direction

### Two ingest directions

- **TS class → Component**: source code is the ingest format; DSL absorbs existing React components
- **`.dag` declaration → emitted JSX**: DSL is the authoring surface; JSX is emission target

### Candidate Q3-a — `.dag` → JSX (DSL authority)

Authors write `.dag` Component values; emit produces `.tsx`. Single source of truth = `.dag`.

Pros:
- Matches gate #28 omni-emission pattern (one `Dag` → multiple emit targets)
- DSL is the authority; emitted code is mechanical
- Composition with Cluster F lens framework natural

Cons:
- Authors familiar with React must learn `.dag` Component DSL
- Existing React code can't be incrementally absorbed

### Candidate Q3-b — TS → Component (ingest existing code)

DSL absorbs existing `.tsx` files into `Component` carrier values.

Pros:
- Incremental adoption path for existing codebases
- DSL gains React-authority without rewrites

Cons:
- Introduces parse-from-source-code direction (TS AST → Component); inverse of normal emit pipeline
- Source-of-truth ambiguity (which authority wins when TS source diverges from `.dag` model?)

### Candidate Q3-c — Bidirectional with `.dag` authority

`.dag` is canonical; TS source is **always** emitted; ingest direction only exists for migration tooling.

Pros:
- Single authority preserved
- Migration path available without ongoing parallel-rep debt

Cons:
- Migration tooling becomes second-class; may bit-rot

**Mgr recommendation**: Q3-a. Single authority discipline; migration tooling is a separate concern not load-bearing for the canvas.

## §6. Q4 — Cross-target consistency story (gate #28 invariant extension)

### Existing invariant

Gate #28 `omni_layers_share_one_node_tree` — CONSUMER_LANDED + PASSING via `m1_5_omni_shape_b_openapi_test.rs`. Asserts: one `compile_to_dag` per workflow; Shape A `emit_rust` + canonical route extraction + Shape B OpenAPI YAML + Markdown + SQL DDL projections all share the same `Dag`.

### Extension for full-stack

The full-stack scenario adds:
- Rust backend (existing Shape-A)
- TS client (new Shape-A)
- React UI (new Shape-A or Shape-F)
- OpenAPI spec (existing Shape-B; wire contract)
- SQL DDL (existing Shape-B; DB schema)

All five derive from the same `.dag` workflow. Gate #28's invariant naturally extends — but the test surface must enumerate the 5 (or 6 with Markdown) projection targets.

### Mgr recommendation

Extend gate #28 test to assert 5-target consistency: same `Dag` → Rust + TS + React + OpenAPI + SQL DDL. Author a new gate (e.g., `omni_full_stack_share_one_node_tree`) parallel to #28 if the existing test surface can't cleanly extend, OR fold into #28's existing test with new projection assertions. Director disposition required.

## §7. Q5 — Composition with Cluster F lens framework

### Background

Cluster F's lens framework (complexity / cost / parallelism / effect_enumeration per row #105 + gate #73 canonical-4) applies to `.dag` programs and produces `DimensionReport<Lens>`. Question: do React renderer specs become lens-application targets?

### Possible compositions

1. **Component cost lens**: count React renders, identify infinite-render loops at lens-time (Practice 4 cost-lens consumer)
2. **Component effect lens**: enumerate side effects in `useEffect` hooks (effect_enumeration lens consumer)
3. **Component complexity lens**: tree-depth × hook-count × prop-graph-complexity (complexity lens consumer)
4. **Component parallelism lens**: identify which renders can run in parallel React 18+ concurrent mode (parallelism lens consumer)

Each fits naturally into the existing lens framework. The substrate question is whether `Component` is a `Behavior::Bind` (lens-readable) or a separate substrate-kind.

### Candidate Q5-a — Component is `Behavior::Bind`

`Component` declarations bind into the same Behavior space lens framework reads. All 4 lenses apply uniformly. Practice 4 GREEN.

### Candidate Q5-b — Component is separate substrate kind

Component is a non-Behavior substrate; lens framework needs a Component-applicable adapter.

**Mgr recommendation**: Q5-a — `Component` is `Behavior::Bind`. Strict-mirror of how other carriers participate in lens framework; no new substrate kind. Lens-application natural.

## §8. Practice 4 (coproduct dissolution) overview

New sum-types proposed in canvas:

| Sum type | Practice 4 |
|---|---|
| `HookKind = UseState \| UseEffect \| UseMemo \| UseCallback \| UseContext \| UseRef \| Custom(Identifier)` | 🟡 YELLOW — Custom arm requires consumer-evidence-required; 7-arm closed enumeration covers React 18 standard hooks |
| `ComponentBody = Render \| Composite` | 🟢 GREEN — distinct semantic axes |
| `Lifecycle = OnMount \| OnUnmount \| OnUpdate { triggers }` | 🟢 GREEN — distinct lifecycle phases |
| `TypingDiscipline = Nominal \| Structural` (Q1-b) | 🟢 GREEN — captures genuine semantic difference |

No 🔴 RED proposed. Worker brief (post-R3) authors Practice 4 receipts per variant.

## §9. Cost-of-change accounting

Per `INVARIANTS.md` "Cost of Change":

| State | Files to edit to add one new full-stack endpoint |
|---|---|
| Pre-canvas (today) | ≥5 (Rust handler + OpenAPI spec + SQL DDL + TS client + React component) |
| Post-canvas (R4 substrate landed) | 1 (`.dag` declaration; all 5 emit targets derive) |

5x reduction. This is the substrate-progress payoff.

## §10. R4 phase plan (canvas-only; for ratification framing)

1. **R4-Phase-1**: TS LanguageSpec carrier landing (Q1 ratified shape)
2. **R4-Phase-2**: React framework substrate carriers (Q2 ratified shape) + JSX emission
3. **R4-Phase-3**: Cross-target consistency invariant extension (Q4 ratified shape) + #28-extension or new gate
4. **R4-Phase-4**: Lens framework composition (Q5 ratified shape) — component-applicable lens reads
5. **R4-Phase-5**: Visceral demo extension — extend path (a) TODO demo with React UI + TS client deriving from same `.dag`

Each phase = separate worker brief; standard canvas → ratification → worker brief → PR pattern.

## §11. Director-pending anti-patterns (for R4 worker review)

1. TS LanguageSpec landed without structural-vs-nominal distinction encoded at carrier level (Q1-b: Practice 2/6)
2. React components landed as Shape-B (Director's Q2 lean: Shape-A)
3. Ingest direction admits TS source as authority (Q3-a single-authority)
4. Gate #28 omni-emission invariant extension forgotten (Q4)
5. Component carrier NOT participating in lens framework (Q5-a)
6. New `Shape-F` introduced without explicit ratification (against Mgr-rec Q2-a)

## §12. Open questions for ratification

Director ratification on:

- **Q1** TS LanguageSpec: a / b (Mgr-rec) / c (discretion)
- **Q2** React carrier shape: a (Mgr-rec) / b / c
- **Q3** Ingest direction: a (Mgr-rec) / b / c
- **Q4** Cross-target invariant: extend #28 OR new gate
- **Q5** Lens composition: a (Mgr-rec) / b
- **Practice 4** disposition on `HookKind` (canvas §8 🟡 YELLOW): Custom arm consumer-evidence required pre-R4?

## §13. Reference

- Operator directive 2026-05-13 (PM msg_83ce8113 verbatim)
- PM relay msg_83ce8113 (relaying Director msg_22a1c596)
- Operator chat ratification of path (a)+(b)
- Substrate audit (Director-verified): `dsl/extdeps/languages/` lacks TS
- Gate #28: `docs/r3-program-plan.md:256` + `m1_5_omni_shape_b_openapi_test.rs`
- Rust LanguageSpec precedent: `dsl/extdeps/languages/rust/`
- WISHLIST.md (pending PM update post-ratification)
- Companion canvas: Director-owned path (a) 4-layer TODO demo

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-13
**Scope hard-bound**: R4 post-R3; no implementation pre-R3 close.
