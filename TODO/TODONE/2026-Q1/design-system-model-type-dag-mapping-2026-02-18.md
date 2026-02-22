# R1 Design: Map `SystemModel` to `Dag<TypeOp>`

Date: 2026-02-18
Task: `R1` (Wave 2A)

## Scope

Design a structural mapping from the current `SystemModel` shape in
`core/ir/src/system_model.rs` into `Dag<TypeOp>` so behavior contracts become
typed sub-DAGs and downstream derivation can operate on graph structure.

This is a design-only step; no runtime behavior changes are included here.

## Current Data Model (Source)

- `SystemModel`: `id`, `name`, `kind`, `version`, `docs`, `behaviors`, `dependencies`
- `Behavior`: `id`, `description`, `invocation`, `inputs`, `outputs`, `properties`
- `BehaviorInput`: `name`, `input_type`, `required`
- `BehaviorOutput`: `name`, `output_type`
- `InputType`/`OutputType`: `TypeId | TypeDag(TypeId)` references into `TypeRegistry`

## Target Graph Shape

Use one top-level `Dag<TypeOp>` per `SystemModel`, with one typed sub-DAG per behavior.

- Root node:
  - `system::<system_id>`
  - `TypeOp::Identity`
- Behavior anchor nodes:
  - `behavior::<behavior_id>`
  - `TypeOp::Identity`
- Input/output attachment nodes:
  - `behavior::<behavior_id>::input::<input_name>`
  - `behavior::<behavior_id>::output::<output_name>`
  - `TypeOp::Identity`
- Type contract nodes:
  - Materialized by cloning resolved `Dag<TypeOp>` from `TypeRegistry` and attaching under each input/output attachment node.

## Field-to-Node Mapping

| `SystemModel` field | Graph representation |
|---|---|
| `id` | Root node ID suffix (`system::<id>`) |
| `name` | `Validate(Custom("meta:name=<name>"))` on root |
| `kind` | `Validate(Custom("meta:kind=<kind>"))` on root |
| `version` | `Validate(Custom("meta:version=<version>"))` on root |
| `docs` | `Validate(Custom("meta:docs=<docs_ref_or_hash>"))` on root |
| `dependencies` | Edges from root to dependency marker nodes (see below) |
| `behaviors` | One sub-DAG anchor per behavior, attached to root |

| `Behavior` field | Graph representation |
|---|---|
| `id` | Behavior anchor node ID suffix |
| `description` | `Validate(Custom("meta:description=<...>"))` on behavior anchor |
| `invocation` | `Validate(Custom("invocation:<serialized>"))` on behavior anchor |
| `properties` | One predicate per property on behavior anchor: `Validate(Custom("property:<Property>"))` |
| `inputs` | Input attachment nodes + type sub-DAG edges |
| `outputs` | Output attachment nodes + type sub-DAG edges |

| I/O field | Graph representation |
|---|---|
| `BehaviorInput.name` | Input attachment node suffix |
| `BehaviorInput.required=true` | Direct edge to input type sub-DAG root |
| `BehaviorInput.required=false` | Insert `TypeOp::Wrap(WrapperKind::Optional)` before type sub-DAG |
| `BehaviorInput.input_type` | Resolve `TypeRegistry` DAG and attach |
| `BehaviorOutput.name` | Output attachment node suffix |
| `BehaviorOutput.output_type` | Resolve `TypeRegistry` DAG and attach |

## Dependencies Mapping

- `DependencyKind::System(target)`:
  - Add marker node `dep:system::<target>` with `TypeOp::Identity`
  - Edge: `system::<id>` -> `dep:system::<target>`
- `DependencyKind::Secret(secret_id)`:
  - Add marker node `dep:secret::<secret_id>` with `TypeOp::Identity`
  - Edge: `system::<id>` -> `dep:secret::<secret_id>`

These marker nodes are metadata carriers only; they enable structural checks
without introducing a new op kind.

## Behavior Sub-DAG Convention

For each behavior:

1. Create `behavior::<id>` anchor.
2. Attach invocation/property validators as `Validate(Custom(...))` chain.
3. For each input:
   - Create input attachment node.
   - Optionally wrap with `Optional` for non-required inputs.
   - Attach cloned type contract DAG.
4. For each output:
   - Create output attachment node.
   - Attach cloned type contract DAG.

This preserves existing `TypeRegistry` contracts and makes behavior shape
navigable via standard DAG traversal.

## Why This Fits `Dag<TypeOp>`

- Reuses existing `TypeOp` variants; no new enum variant required for R1.
- Keeps type contract semantics in the existing type DAGs from `TypeRegistry`.
- Encodes model metadata through `Validate(Custom(...))` so downstream passes
  can query structure + metadata from one graph representation.

## Planned Follow-on (R2+)

- `R2`: register these system behavior DAGs in `TypeRegistry`
  (naming convention: `System::<system_id>::Behavior::<behavior_id>`).
- `R3`/`R4`: replace property and mapping checks with graph predicates/walks.
- `R5`: remove `rust_type_for_type_id` string mapping in favor of `PortType`
  derived from attached output DAGs.

## Validation Criteria for R1 Completion

- Deterministic mapping rules defined for every current `SystemModel` field.
- Required/optional input semantics represented structurally.
- Dependency edges represented structurally.
- Mapping is implementable without new `TypeOp` variants.
