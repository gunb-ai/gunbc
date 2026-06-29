# DRAFT for operator review — Lever B(i): minimal per-kind Node sum type

> **Sub-move (i) ONLY.** This sketch covers the per-kind sum-type split. The arena/`u32`-index **container** is sub-move (ii), a separate later lane — not modeled here.
>
> Grounding: [`representation-minimization.md`](representation-minimization.md) item 3 · [`dsl/gunbc/plans/representation_minimization.dag`](../dsl/gunbc/plans/representation_minimization.dag) · parent node anatomy (`Node` @ `src/v1/stage0/src/v1_std_core.rs:742`).
>
> Executable companion (same branch, PR #5936): `src/v2/std/node_minimal.dag` + `src/v2/test/claim/manual/node_minimal_representation_test.dag`.

## Problem (the waste)

The v1 bootstrap `Node` is an **18-field superset struct** (`size_of::<Node>() = 144 B`; 12 pointer fields + inline `name` + `ident`; Rc-wrapped ≈ 160 B/node before heap payloads). A leaf expression node (e.g. `ExprVar`) uses roughly **3 live fields** (`span`, `expr_data`, empty `children`) but pays for all 18 — ~76% dead per leaf × ~627k nodes in whole-tree resolve.

Lever B(i) attacks **bytes-per-node field waste** by replacing the superset with a **closed per-kind sum type** where each variant carries only the fields its kind reads.

## Authority confirmed first (constraint c)

**The v2 `.dag` Node model is the substrate authority** — not the v1 `.rs` seed sizes.

| Layer | Location | Shape |
|-------|----------|-------|
| **Authority (v2)** | `src/v2/std/node.dag` | `Node { kind: NodeKind, children: List<Edge>, occurrence_id: NodeOccurrenceId }` where `NodeKind = TypeNode \| ComputationNode` |
| **Bootstrap artifact (v1, doomed)** | `src/v1/stage0/src/v1_std_core.rs:742` | 18-field superset + Rc graph |
| **Resolved graph (v2 pipeline)** | `src/v2/compiler/03_resolve.dag` | `ResolvedTree = Node` (env-free) |
| **Inference facts (v2 pipeline)** | `src/v2/compiler/04_infer.dag` | `InferredTree { root: Node, facts: Map<Node, InferredFacts> }` — v2 analog of v1's `inferred`/`type_env`/`func_env` side data |

The v2 substrate Node is **already minimal for the semantic graph** (3 fields). The v1 superset conflates (a) semantic graph, (b) surface syntax (`expr_data`, spans, names), and (c) inference metadata (`inferred`, recursion flags) into one struct. B(i) **decomposes that conflation** without forking the v2 substrate.

## Proposed model: `MinimalResolvedNode` (sub-move i)

New module **`v2.std.node_minimal`** (scaffold — dissolves into `v2.std.node` when #5879 emitter determinism lands; see Disposition bind).

### Closed sum type (six kinds)

| Variant | Carries | Maps from v1 superset fields |
|---------|---------|------------------------------|
| `MrnSubstrateGraph` | `NodeKind`, `List<Edge>`, `NodeOccurrenceId` | **None** — this *is* v2 authority Node; zero superset-field mapping |
| `MrnSurfaceExprLeaf` | `SurfaceExprTag`, `SurfaceProvenance`, `InferenceSlot` | `span`, `ident`, `expr_data`, `inferred` |
| `MrnSurfaceExprParent` | above + `children` | + `children` |
| `MrnSurfaceConnective` | `Connective`, provenance, `children`, `params`, `uses`, inference | `connective`, `span`, `ident`, `children`, `params`, `uses`, `inferred` |
| `MrnSurfaceNamedDecl` | `SurfaceDeclRole`, `NamedProvenance`, `SurfaceDeclPayload`, inference | `name`, `ident`, `span`, `ident_span`, `children`/`params`/`uses`/`body`, `inferred`, `return_cardinality` |
| `MrnSurfaceMatchArm` | `MatchPatternSlot`, provenance, `children` | `span`, `ident`, `match_pattern`, `children` |

Side-table refs modeled (not inline heap strings): `SpanRef { file_id, start, end }`, `IdentRef { value }` (0 = absent; **ident is retained**, not deleted — refuted-by-execution in parent plan).

### Superset fields — disposition (honest accounting)

| v1 field | B(i) home | Notes |
|----------|-----------|-------|
| `name`, `ident`, `span`, `ident_span` | Named/surface provenance + side tables | `ident` kept for module/import fast-path |
| `children`, `params`, `uses`, `body` | Per-kind payload lists / `SurfaceDeclPayload` | Only on kinds that read them |
| `connective`, `expr_data` | Kind discriminator payloads | Eliminates cross-kind dead storage |
| `inferred` | `InferenceSlot` **or** v2 `InferredTree.facts` side-map | Substrate graph nodes: facts live in side-map (v2 already splits) |
| `return_cardinality` | Named decl variant only | |
| `match_pattern` | Match-arm variant only | |
| `transport`, `properties`, `type_annotation` | **Owed: extend `SurfaceNamedDecl` payload** before emit | Not yet in executable sketch variants — listed here so the split cannot silently drop them |
| `is_self_recursive`, `has_non_tail_self_call` | **Owed: fn-item metadata arm or infer facts** | Descent/termination metadata; candidate for `InferredFacts`, not every node |
| Rc/`Vec` child storage | **Sub-move (ii) — NOT in this sketch** | Arena/`u32` slices replace pointer fields at realization |

## NOT-edited boundary (constraint b — load-bearing)

These files were **not modified** on branch `session/deep-seal-583` / PR #5936:

- `src/v2/std/node.dag` — load-bearing substrate `Node` / `NodeKind` / `fold_node`
- `src/v1/stage0/src/v1_std_core.rs` — v1 bootstrap Node struct
- `src/v1/stage0/src/v1_compiler_infer.rs` — **`build_type_env`** and infer seed
- Any v1 Rust seed emit/infer/resolve implementation

**Added only:** `src/v2/std/node_minimal.dag` (sibling scaffold), witnesses, this plan sketch.

## Sub-move (ii) explicitly out of scope

- No `u32` arena index, no `ChildSlice { start, len }`, no Rc replacement
- No struct-of-arrays / columnar layout
- `List<Edge>` remains in the B(i) sketch; container swap is a **separate lane** gated on the same #5879 emit gate but modeled independently

## Checkpoint criteria (parent review)

### 1. Discriminating witness — field coverage (green-by-execution)

**Landed (PR #5936, runs green):**

```
cargo test -p v1-compiler-tests v2_node_minimal_representation_compiles_and_witnesses_hold
```

| Witness | What it proves |
|---------|----------------|
| `node_minimal_expr_leaf_field_count_below_superset_holds` | Expr-leaf kind: **4 live fields < 18** superset |
| `node_minimal_expr_leaf_drops_dead_fields_holds` | Expr-leaf does **not** carry `connective`/`params`/`uses`/`body` |
| `node_minimal_substrate_round_trips_v2_node_holds` | `minimal_substrate_node_from_v2` preserves v2 `Node` kind/children/occurrence |
| `node_minimal_substrate_has_no_superset_fields_holds` | Substrate kind maps **zero** v1 superset fields (authority is already minimal) |
| `node_minimal_kind_of_expr_leaf_holds` | Kind classifier is discriminating |

**Owed before emit integration (not yet green):**

- Per-accessor witness: for each v1 field **read at runtime** on the resolve/infer path, assert the owning `MinimalNodeKind` lists that field in `minimal_node_kind_superset_fields` (generated from `node_field_roles` / accessor census — quiet-gull-17 receipt)
- Extension witnesses for `transport`/`properties`/`type_annotation`/`is_self_recursive`/`has_non_tail_self_call` once variant arms are authored (table above)
- End-to-end: whole-tree resolve diagnostic fingerprint **unchanged** after realization swap (gated on #5879)

### 2. Measured byte / RSS delta

| Metric | Status |
|--------|--------|
| **Structural (now)** | Expr-leaf: 18 fields → 4 mapped (~78% field-slot reduction at type level). v1 seed struct: 144 B/node fixed regardless of kind. |
| **Runtime RSS (owed)** | No RSS movement yet — sketch is `.dag` authority only; v1 seed still allocates 144 B superset. **Measure after #5879**: `measure_whole_tree_resolve` width=1 VmHWM before/after emit regenerates `MinimalResolvedNode` realization. Parent plan estimate for B(i)+B(ii) combined: ~40–55% node-graph cut; **B(i) alone** = eliminate ~76% dead field slots per leaf kind (exact RSS TBD at emit). |

### 3. Named NOT-edited boundary

See section above. Operator gate: escalate before any edit to `v2.std.node` `Node` type or `build_type_env`.

## Sequencing / gates

1. **#5879 emitter determinism** (stern-fox-585) — THE GATE. Until bit-identical emit fixed point, representation changes cannot regenerate from `.dag`; hand-edits would cement v1 seed.
2. **This sketch (B(i))** — model + witnesses; Disposition scaffold binds dissolution to `v2.std.node.Node`.
3. **B(ii) arena container** — separate work item; composes multiplicatively with B(i), does not block modeling B(i).

## Dissolution trigger

When #5879 lands and emit regenerates the Node realization from `v2.std.node_minimal` into the v2 self-host fixed point, this sketch + scaffold dissolve into `v2.std.node` as the single authority; v1 18-field struct dies with the seed (representation-minimization.md §Dissolution).
