# Modeling Queue — Semantic Erasure Elimination

**Last updated**: 2026-02-23
**Source**: external modeling feedback (items 7-16)
**Scope**: CI/testing process modeling and adjacent semantic-integrity work

Use this as the intake queue for modeling-first hardening work. When an item is
prioritized, move it into a sprint table in `TODO/tasks.md` with the same ID.

## Compositional Modeling Frame

Every task in this queue serves one of two goals from the **Foundation Close-Out**
meta-lane (see `docs/handbook.md` § "Compositional Modeling Philosophy"):

- **Lane A ("One Representation")**: Delete every shadow, fallback, or parallel
  description. Each concept has exactly one canonical source. Tasks: M9, M14, M16, M18, M20, M22.
- **Lane B ("Proven Correct")**: Make invariants machine-enforced. Undeclared I/O is
  rejected, missing data poisons execution, coercions are shape-checked, equivalent work
  is deduplicated. Tasks: M7, M8, M10, M11, M12, M13, M17, M19, M20, M22.

The guiding principle: external systems are **compositions of layered concerns**
(TCP → TLS → HTTP → REST → provider → operation). Each layer imposes invariants.
The compiler derives transport code, mocks, and test obligations from the composition.
Where the Rust substrate currently hand-wires what the DSL can derive, the modeling
tasks in this queue eliminate that gap.

Inspirational reference: `the-gunbai` Understanding pattern — external systems modeled
as structured data (behaviors, constraints, assumptions, dependencies) from which blocks,
tests, and prerequisites are automatically derived.

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
| **M17** | **Global flattening + context-free work identity**: guarantee dedup across intra/inter workflow boundaries. | Flattening pass from orchestration refs to one global typed execution DAG + `WorkIdentity` model independent of orchestration node names. | Equivalent work from different workflow entrypoints unifies into one execution vertex; dependents fan out from one commit/result. | Planner resolves/merges equivalent work identities before scheduling; key payload does not depend on workflow node names; tests prove `ci` and `test-all` share hits for same work/data. | WF1-D, WF3-D, WF4-D | L |
| **M18** | **Single semantic authority / projection-only surfaces**: eliminate parallel truths across DAG, Make, CLI, and reports. | Canonical semantic model with generated projections (wrappers/views), plus drift validators. | Projections cannot author new dependencies/effects/claims; only canonical graph/contracts can. | Make/CLI/report definitions are generated or validated against canonical model; drift fails CI; no duplicate authored dependency graphs remain. | M17 | M |
| **M19** | **Formal non-redundancy proof harness**: encode planner invariants as executable property tests. | Invariant suite over resolved global DAG + ledger/key behavior. | Preflight/CI must prove at-most-once execution, minimal dirty closure, and single-writer ordering constraints. | Property/integration tests fail on duplicate execution opportunities, non-minimal execute sets, or concurrent unordered writers; planner emits proof diagnostics. | M17, M18 | M |
| **M20** | **Repository self-understanding model**: model the repo's own structure (workspace graph, generator edges, commit policies, toolchain requirements) as canonical data from which .gitignore, Makefile, CI, and bootstrap derive. Inspired by `the-gunbai` `understanding/repo.rs` + `workspace.rs` + `generator.rs` + `codegen_layering.rs`. | Workspace model (`CrateTier` + `CrateSpec`), generator edge graph (producer→consumer from `#[tool_target]` outputs), commit policy model (replaces handwritten `.gitignore`), toolchain requirements. | Workspace model is single source of truth; layering violations (Foundation depending on Application) fail tests; generator graph is acyclic; `.gitignore` is derived from commit policies; adding a new crate requires one `CrateSpec` entry. | Workspace model matches Cargo.toml (test); tier layering invariants enforced (test); generator cycle detection (test); `.gitignore` derived from policies not handwritten; toolchain requirements canonical. | — | L |
| **M21** | **Structural primitives for consistent cross-backend codegen**: decompose opaque identity primitives (Bool, Int, etc.) into structural type DAGs; add `PlatformRepr` metadata; replace per-backend hardcoded `map_to_*_type()` with shared `TypeShape` derivation + per-backend rendering. | `TypeShape` enum (Platform/Coproduct/Product/Brand/Container/Opaque), `PlatformRepr` metadata payload, `type_shape()` extractor, per-backend `render_*_type(TypeShape)`. | All emit backends derive types from DAG structure, not string-name matching; adding a new type to the registry automatically gets correct representations in Rust/Go/C/MIPS; `TypeShape::Opaque` is a diagnostic (strict mode: error). | Bool is structural Coproduct; Int/Float carry PlatformRepr; all 4 backends use shared derivation; exhaustiveness test fails on Opaque; adding a Product/Coproduct type requires zero per-backend code. | M8 | L |
| **M22** | **Annotation-to-DAG modeling migration**: Eliminate 17 inert/unused annotations by modeling their intent as first-class DAG concerns. Phase 0: delete noise annotations + add unknown-annotation warnings. Phase 1: `@contract` → compile-time proof obligations. Phase 2: `@error_map` → transport error classification nodes. Phase 3: `@retry` → transport retry policy nodes. Phase 4: `@requires` → resource/capability edges. Phase 5: `@testgen_skip` → emit consumption. | Annotation census (43→~26 active + zero inert); `@contract` generates typed test obligations per implementation; `@error_map`/`@retry` compose into protocol stack transport DAG; `@requires` becomes structural resource edges. | `@contract` → proof obligation tests generated for every interface implementation; `@error_map` → error classification in transport DAG + per-status testgen; `@retry` → retry wrapper in transport DAG; `@requires` → DAG resource edges; unknown annotations warn (strict: error). | Zero inert annotations remain; `@contract` has CI-gated test coverage for all implementations; `@error_map`/`@retry` compose with protocol stack; `@requires` feeds M10 resource admission; unknown annotation warnings in default mode, errors in strict. | M8, M10, M12, M16 | L |

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

**Design**: `docs/design/modeling/protocol-stack-layering.md`

- [ ] Split `transport.http_rest` into `transport.http` and `transport.rest` system models with `depends_on`.
- [ ] Define shared invocation contract model consumed by both SystemModel and transport behavior specs.
- [ ] Implement behavioral property inheritance across protocol layers (REST inherits HTTP properties).
- [ ] Migrate `Invocation::Rest`/equivalents to use shared transport behavior representations.
- [ ] Ensure request construction/routing validation/testgen derive from the shared contract layer.
- [ ] Add parity tests proving SystemModel-derived behavior and TransportBehavior-derived behavior are structurally equivalent.
- [ ] Remove duplicate spec surfaces or add strict consistency checks where temporary dual definitions remain.

### M17 Checklist — Global Flattening + Context-Free Work Identity

- [ ] Define `WorkIdentity` so equivalent work is independent of orchestration node naming (`ci.*` vs `test_all.*`).
- [ ] Define flattening contract: all process-invocation references are expanded/resolved before scheduling.
- [ ] Ensure key payload upstream contribution is keyed by consuming input ports, not upstream node labels.
- [ ] Add dedup merge rule for equivalent `(WorkIdentity, key payload)` vertices with fan-out edge rewiring.
- [ ] Add cross-workflow tests proving shared cache hits and single execution for equivalent work.

### M18 Checklist — Single Semantic Authority / Projection-Only

- [ ] Declare one canonical semantic source for workflow dependencies/effects/claims.
- [ ] Ensure Make/CLI/report surfaces are generated projections or strict validated views.
- [ ] Add drift checks that fail when projection semantics diverge from canonical model.
- [ ] Remove or deprecate manually maintained duplicate dependency graphs.
- [ ] Add migration notes and tooling for cutover from authored projections.

### M19 Checklist — Formal Non-Redundancy Proof Harness

- [ ] Add preflight invariant checker for single-writer ordering: unordered concurrent writers are rejected.
- [ ] Add at-most-once execution invariant checks over `(WorkIdentity, MaterializationDigest)`.
- [ ] Add minimal-dirty-closure checks comparing executed set vs computed transitive dirty closure.
- [ ] Add projection-equivalence tests proving generated wrappers cannot alter execute set.
- [ ] Emit actionable diagnostics (which invariant failed, nodes/resources involved) on proof failure.

### M20 Checklist — Repository Self-Understanding Model

**Design**: `docs/design/modeling/repo-self-understanding.md`

- [ ] Add `workspace_model.rs` to `core/infra` with `CrateTier`, `CrateSpec`, `workspace_crates()`.
- [ ] Add layering validation: Foundation cannot depend on Core/Application, Core cannot depend on Application.
- [ ] Add test proving workspace model matches Cargo.toml workspace members (bidirectional).
- [ ] Add generator edge derivation from `iter_tool_targets()` outputs — producer→consumer graph.
- [ ] Add cycle detection for generator graph; acyclicity test.
- [ ] Add commit policy model (`CommitPolicy`, `CommitReason`) replacing handwritten `.gitignore` logic.
- [ ] Derive `.gitignore` from commit policies; replace `all_tool_outputs_gitignored` test with policy validation.
- [ ] Add toolchain requirements model with canonical version pins.

### M21 Checklist — Structural Primitives for Consistent Codegen

**Design**: `docs/design/modeling/structural-primitives-codegen.md`

- [ ] Add `TypeShape` enum to `core/ir` (Platform, Coproduct, Product, Brand, Container, Opaque).
- [ ] Add `PlatformRepr` metadata payload to `MetadataPayload` (bits, signed, float, discrete).
- [ ] Add `type_shape()` extractor: `Dag<TypeOp>` → `TypeShape` via root node classification.
- [ ] Decompose `type_lib::bool()` from `identity("Bool")` to `coproduct("Bool", [("True", "Unit"), ("False", "Unit")])`.
- [ ] Add `PlatformRepr` metadata to `type_lib::int()` and `type_lib::float()`.
- [ ] Add per-backend `render_*_type(TypeShape)` functions (Rust, Go, C).
- [ ] Replace hardcoded `map_to_rust_type()`, `map_to_go_type()`, `map_to_c_type()` with shared derivation.
- [ ] Add exhaustiveness test: every registered type produces non-Opaque TypeShape (or documented exception).
- [ ] Add cross-backend consistency test: same TypeShape yields semantically equivalent types in all backends.

### M22 Checklist — Annotation-to-DAG Modeling Migration

**Design**: `docs/design/modeling/annotation-to-dag-modeling.md`

Phase 0 (cleanup):
- [ ] Delete noise annotations: `@network`, `@credential`, `@external`, `@derived_from`, `@ledger`.
- [ ] Migrate duplicates: `@test_hermetic` → `@hermetic`, `@test_integration` → `@tier(Integration)`, `@invariant` → `@contract`.
- [ ] Migrate `@format(uuid)` → `@pattern(UUID_REGEX)`.
- [ ] Migrate `@tool(READ/WRITE)` and `@mode(...)` to `TypeOp::Meta` payloads (requires M8).
- [ ] Add unknown-annotation compiler warning (strict mode: error). Remove silent forward-compat acceptance.

Phase 1 (`@contract`):
- [ ] Generate typed test obligations from `@contract` annotations for every interface implementation.
- [ ] Add CI gate that fails if any implementation lacks contract test coverage.
- [ ] Connect to M12 proof obligation framework.

Phase 2 (`@error_map`):
- [ ] Wire `@error_map` into transport DAG as error classification node.
- [ ] Compose with protocol stack default status classification (M16).
- [ ] Testgen derives per-status-code test from error map.

Phase 3 (`@retry`):
- [ ] Wire `@retry` into transport DAG as retry wrapper node.
- [ ] Compose with error classification (retryable status codes).
- [ ] Testgen derives retry + exhaustion tests.

Phase 4 (`@requires`):
- [ ] Wire `@requires` as structural resource/capability edges in DAG.
- [ ] Feed into M10 resource admission checks.

Phase 5 (`@testgen_skip`):
- [ ] Wire `@testgen_skip` into `daglang-emit` test generation to exclude marked items.

## Suggested Dependency Lanes

```
Lane A (graph semantics):            M8 -> M9 -> M16
Lane B (workflow execution safety):  M10 -> M11 -> M12
Lane C (process contract drift):     M13 -> M14
Lane D (security/install typing):    M7, M15
Lane E (global minimality proof):    M17 -> M18 -> M19
Lane F (repo self-model):            M20 -> M14, M18
Lane G (codegen consistency):        M8 -> M21
Lane H (annotation modeling):        M8 + M10 + M12 + M16 -> M22
```
