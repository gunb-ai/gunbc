# Modeling Queue — Semantic Erasure Elimination

**Last updated**: 2026-02-19  
**Source**: external modeling feedback (items 7-16)  
**Scope**: CI/testing process modeling and adjacent semantic-integrity work

Use this as the intake queue for modeling-first hardening work. When an item is
prioritized, move it into a sprint table in `TODO/tasks.md` with the same ID.

## Design-First Policy (Required)

For every modeling task (`M*`), implementation must be preceded by a reviewed
design artifact with concrete DAG structures.

Required design content:

1. concrete typed DAG shape: nodes, typed ports, edge kinds, and dependencies,
2. explicit resource/admission model: what is mutually exclusive and why,
3. invalidation/state model: key payload and miss/failure semantics, and
4. no-fallback boundary behavior and migration/cutover plan.

Promotion rule:

1. when scheduled, create paired tasks `<ID>-D` (design) and `<ID>` (implementation),
2. `<ID>` must depend on `<ID>-D`.

## Highest ROI Next 3

1. `M10` mandatory resource declarations + auto-wiring
2. `M11` strict dry-run in CI/testgen to fail on missing modeling
3. `M8` semantically inert metadata op (stop using `Validate(Custom(...))` as metadata carrier)

## Intake Tasks

| ID | Task | Minimum modeled unit | Coordination contract (mutual exclusion + downstream) | Acceptance | Deps | Size |
|----|------|----------------------|---------------------------------------------------------|------------|------|------|
| **M7** | **Secret redaction by default**: make accidental display/logging always redacted. | Capability-split secret representation (`SecretValue` runtime + redacted render type/capability). | Any logging/debug/display path must consume redacted view only; plaintext extraction is explicit and transport-boundary only. | `Display`/`Debug`/`to_string()` paths are always redacted for secret-bearing values; explicit plaintext extraction is grep-auditable; regression tests prove no plaintext emission in renderers. | — | M |
| **M8** | **Separate metadata from validation semantics**: stop carrying descriptive metadata via `Validate(Custom(...))`. | New inert metadata op/annotation (`TypeOp::Meta` or equivalent typed metadata carrier). | Metadata remains traversable but cannot fail or alter execution semantics; only true validators live in `Validate`. | SystemModel->Dag mapping emits metadata via inert channel; runtime behavior unchanged if metadata is erased; validator rejects metadata-over-`Validate` for new code paths. | — | M |
| **M9** | **Typed dependency markers**: replace `dep:system::<target>` string conventions. | Typed dependency identity (`DependencyNodeId` constructor or typed edge/tag). | Dependency semantics are structural, not string-prefix interpreted; downstream walkers match typed kind/target. | No runtime/validator logic depends on string prefixes for dependency-kind detection; round-trip tests cover system/secret dependency graph mapping. | M8 | S |
| **M10** | **Mandatory resource declarations + auto-wiring** for effectful ops. | Build-time rule: effectful ops must declare resource ports/claims; auto-wiring helper for filesystem/manifest claims. | Scheduler admission denies undeclared I/O; conflicting claims are mutually exclusive by construction; downstream execution waits on committed producers. | DAG build fails closed when effectful node lacks claims; auto-wiring removes manual edge boilerplate for common resources; concurrency tests prove safe parallel read/read and blocked write/write. | WF1, WF2 | L |
| **M11** | **Strict dry-run mode**: prevent permissive mocks from hiding missing wiring. | `DryRunMode::{Lenient,Strict}` with poison/`UNSET` defaults. | In strict mode, any consumption of unset resource/env inputs is a hard failure before downstream execution. | `--dry-run=strict` fails on missing resource/env wiring; CI/testgen path uses strict mode; lenient mode preserved for developer ergonomics. | M10 | M |
| **M12** | **Coercion proof nodes/receipts**: verify shape coercions, not just non-crash execution. | Assertion node set or shape-receipt channel (`value_kind`, cardinality, optional hash). | Tests must assert coercion contracts before downstream nodes are treated as valid. | Generated coercion tests assert shape/cardinality invariants (scalar->list, non-nested list, etc.); failures are explicit and localized. | M11 | S |
| **M13** | **Registry->CLI->Make contract tests** to prevent semantic drift. | Hermetic round-trip harness from registry entrypoint metadata to emitted argv semantics. | Repeatability/cardinality semantics are tested once and enforced across registry, makegen, and CLI parsing. | Fast integration tests catch dropped repeatable flags and cardinality drift; failing contract blocks CI. | WF8 | M |
| **M14** | **Single inventory authority for tools/binaries/resource providers**: remove duplicated hardcoded lists. | Inventory-backed canonical registration model for binaries + provides/consumes metadata. | Downstream consumers (Make/CI/resource maps) derive from one source; additions/renames propagate atomically. | Adding a tool requires one registration point; generated lists replace manual per-file edits; drift tests validate parity outputs. | M13 | M |
| **M15** | **Typed install planning**: remove stringly/lossy installer bridging. | `PackageManagerId` typed parse + explicit `InstallPlan` policy model. | Install selection policy is explicit and testable; adapter does not silently drop required fields. | Unknown package manager IDs fail closed; selection policy is deterministic and documented; adapter preserves/validates script/url requirements instead of default-dropping. | — | M |
| **M16** | **Unify SystemModel invocation contracts with TransportBehavior specs**. | Shared invocation spec model reused by SystemModel and transport behavior definitions. | Request construction, validation, and testgen consume one objective contract; no parallel spec drift. | `Invocation::Rest`-style behavior is represented via shared transport spec types; contract tests prove parity between system model routing and transport behavior routing. | R8, R10, M8 | M |

## Execution Checklists (Review Gate)

Use these checklists as implementation gates. A task is not implementation-ready
until its checklist is reviewed, and not complete until all checklist items are done.

### M7 Checklist — Secret Redaction By Default

- [ ] Replace generic plaintext extraction naming with explicit transport-only capability naming (for example `expose_plaintext_for_transport`).
- [ ] Ensure `SecretString` `Debug`, `Display`, and `ToString`-derived paths always redact.
- [ ] Add clippy disallow rules (or equivalent guardrails) for plaintext-extraction method usage outside approved transport boundary modules.
- [ ] Audit and migrate existing plaintext extraction callsites to approved boundary paths.
- [ ] Add regression tests covering accidental formatting (`{:?}`, `{}`, `.to_string()`) and logging paths.

### M8 Checklist — Semantically Inert Metadata (`TypeOp::Meta`)

- [ ] Add `TypeOp::Meta(MetadataPayload)` with typed metadata payload variants (no string-prefix encoding for semantics).
- [ ] Migrate SystemModel type-DAG generation from `Validate(Custom("meta:..."))` and `Validate(Custom("property:..."))` to `Meta(...)`.
- [ ] Keep `Validate(...)` exclusively for semantic checks that can fail.
- [ ] Add compatibility parsing for legacy metadata markers only where required for migration.
- [ ] Add an erasure-invariance test: removing `Meta` nodes must not change runtime behavior.
- [ ] Add validation that rejects newly authored metadata-over-`Validate(Custom(...))` in strict mode.

### M9 Checklist — Typed Dependency Markers

- [ ] Replace string-encoded dependency node identifiers (for example `dep:system::<id>`) with typed dependency identity/edge metadata.
- [ ] Remove runtime/validator logic that infers dependency kind from string prefixes.
- [ ] Introduce typed constructors/helpers so dependency markers cannot drift by convention.
- [ ] Add round-trip tests for system and secret dependency mapping across build/register/derive paths.
- [ ] Add migration tests ensuring legacy graphs either translate deterministically or fail with actionable diagnostics.

### M10 Checklist — Mandatory Resource Declarations + Auto-Wiring

- [ ] Add a build-time validator: effectful workflow/DAG units must declare required resource ports/claims.
- [ ] Add auto-wiring helpers for common resources (filesystem, manifest, toolchain/network handles) to reduce manual edge wiring.
- [ ] Ensure scheduler/executor admission derives claims structurally from declared ports/ops (no side tables).
- [ ] Add conflict tests covering read/read allowed and write/write denied for shared resources.
- [ ] Add failure tests for undeclared effectful I/O and missing required resource edges.

### M11 Checklist — Strict DryRun With Poisoned Missing Inputs

- [ ] Introduce global dry-run mode enum (lenient/strict) used consistently across planner/executor paths.
- [ ] Add poison/unset value model for missing resource/env acquisitions in strict dry-run.
- [ ] Ensure environment/resource boundary nodes emit poison in strict mode when not explicitly wired or mocked.
- [ ] Add executor fail-fast on poison consumption with data-flow trace to missing acquisition.
- [ ] Wire strict dry-run mode into CI/testgen/integration test paths.
- [ ] Add tests proving lenient mode remains ergonomic while strict mode fails on missing modeling.

### M15 Checklist — Typed Package Manager Modeling

- [ ] Introduce strict `PackageManagerId` model (no `Unknown` fallback in strict parse path).
- [ ] Migrate installer/tool-upsert bridging from raw `&str pm_id` to typed IDs.
- [ ] Make install-option selection policy explicit and documented (not implicit first-match list order).
- [ ] Add compatibility adapter only where necessary for legacy manifests; unknown IDs must fail closed in strict mode.
- [ ] Preserve required install fields during bridging (no silent `script/url` drops).
- [ ] Add exhaustive tests for supported package managers and selection-policy determinism.

### M16 Checklist — SystemModel/TransportBehavior Unification

- [ ] Define shared invocation contract model consumed by both SystemModel and transport behavior specs.
- [ ] Migrate `Invocation::Rest`/equivalents to use shared transport behavior representations.
- [ ] Ensure request construction/routing validation/testgen derive from the shared contract layer.
- [ ] Add parity tests proving SystemModel-derived behavior and TransportBehavior-derived behavior are structurally equivalent.
- [ ] Remove duplicate spec surfaces or add strict consistency checks where temporary dual definitions remain.

## Suggested Dependency Lanes

```
Lane A (graph semantics):            M8 -> M9 -> M16
Lane B (workflow execution safety):  M10 -> M11 -> M12
Lane C (process contract drift):     M13 -> M14
Lane D (security/install typing):    M7, M15
```
