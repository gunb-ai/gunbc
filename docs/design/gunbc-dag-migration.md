# gunbc-dag Migration: Rust → Pure DSL

**Goal**: Reduce gunbc-dag from a 23,107-line Rust crate with domain logic, compiler infrastructure, and repo-specific conventions to a thin execution harness. Domain knowledge lives in `.dag` files. Generic infrastructure lives in `core/` crates.

**Principle**: gunbc-dag should contain ONLY:
1. Bootstrap exceptions (ci.rs, codegen_cli.rs) — can't be generated
2. Resolver bridge (compile DSL → resolve to executable ops) — structural necessity
3. Generated test files (280,603 lines, machine-produced)

Everything else is either generic infrastructure misplaced in a repo-specific crate, or domain knowledge that should be DSL data/functions.

## Current Inventory (23,107 non-generated lines)

### Category A: Generic Infrastructure → Move to core/ (~10,800 lines)

Code that is not repo-specific. It implements generic algorithms that any project using the DSL compiler would need. Currently trapped in gunbc-dag, preventing reuse.

| Module | Lines | What it does | Target crate |
|--------|-------|--------------|--------------|
| `resolve_service.rs` | 2,177 | Generic REST/Shell/File service interpreters parameterized by `ServiceOperationSpec`. Zero repo-specific logic. | `core/ir` or new `core/resolve` |
| `resolve.rs` | 2,294 | Resolution framework: `LoweredOp` → `DynOp`. ~70% generic (adapter ops, primitive mapping, resource lifecycle). 30% is domain dispatch (`resolve_domain()`). | Split: generic → `core/resolve`, domain dispatch stays |
| `workflow/planner.rs` | 572 | Deterministic DAG planning: topo sort, key materialization, miss reasons, critical path | New `core/workflow` |
| `workflow/executor.rs` | 415 | Sequential unit execution, timing, fail-closed semantics | `core/workflow` |
| `workflow/admission.rs` | 521 | Resource claim validation, conflict detection | `core/workflow` |
| `workflow/coordination.rs` | 185 | Readiness analysis: prerequisite + data input checks | `core/workflow` |
| `workflow/schema.rs` | 165 | WorkflowSpec, WorkflowUnit, WorkflowOp types | `core/workflow` |
| `workflow/key.rs` | 149 | Materialization keys, work identity, canonical digests | `core/workflow` |
| `workflow/process_registry.rs` | 316 | ProcessUnitRef, UnitClaim, AccessMode registry types | `core/workflow` |
| `workflow/slo.rs` | 303 | SLO checking, slow-unit reporting, execution reports | `core/workflow` |
| `workflow/projection.rs` | 111 | Execute-set equivalence checking | `core/workflow` |
| `workflow/proof.rs` | 91 | Non-redundancy verification | `core/workflow` |
| `workflow/errors.rs` | 108 | WorkflowAdmissionError | `core/workflow` |
| `mock_defaults.rs` | 583 | ~60% generic: MockSpec builder, response probing, terminal observability. ~40% GCP-specific field mappings. | Split: generic → `core/test`, GCP mappings stay or → DSL |
| `fidelity.rs` | 354 | Generic pattern: compile DSL → extract fn bodies → evaluate. Content is DSL files. | Pattern → `core/codegen` |
| `dsl_builder.rs` | 384 | Compile-then-resolve pattern. ~80% generic. | Pattern → `core/codegen`, layout adapter stays |
| `testgen_dag/dag_test_discovery.rs` | 930 | Module discovery, test-block discovery, auto-testgen pipeline | `core/codegen` or `core/testgen` |
| `testgen_dag/graph.rs` | 315 | Content-upsert DAG builder for testgen targets | `core/codegen` |
| `testgen_dag/mock_interpreter.rs` | 365 | DSL Expr → Value runtime interpreter for test blocks | `core/codegen` |
| `testgen_dag/ops.rs` | 231 | TestgenOp::Generate/AutoGenerate executables | `core/codegen` |
| `testgen_dag/profile_discovery.rs` | 315 | Profile filesystem walk, interface binding extraction | `core/codegen` |

**Subtotal: ~10,809 lines of generic code in a repo-specific crate.**

### Category B: Repo-Specific Domain Logic → DSL Data/Functions (~6,100 lines)

Hard-coded Rust tables, mappings, and conventions that encode domain knowledge about *this repo*. Should be `.dag` data declarations and functions evaluated at compile/runtime.

| Module | Lines | What it encodes | DSL replacement |
|--------|-------|-----------------|-----------------|
| `makegen/registry.rs` | 2,207 | Build commands, meta-targets, tool registry, BuildConfig | DSL data: `data build_config`, `data meta_targets` |
| `extern_impls.rs` | 637 | 6 Rust implementations for `extern func` declarations | Blocked on: recursive types (render_tree), inventory access (discover_tools) |
| `policy/pragma.rs` | 546 | Clippy allowlist rules, dead_code rules, lint policy | Already mirrored in DSL (`arch_rules.dag`, `clippy_policy.dag`). Delete after Phase 2. |
| `workflow/catalog.rs` | 576 | Hardcoded workflow variant table, stage-to-claims mapping | DSL data: `data workflow_catalog`, `data default_claims` |
| `dsl_registry.rs` | 487 | Tool discovery conventions, func→CLI mapping | DSL conventions via annotations or data declarations |
| `workflow/unit_commands.rs` | 425 | Per-workflow command tables mapping NodeId→cargo commands | DSL data: `data ci_commands`, `data test_all_commands`, etc. |
| `resources.rs` | 342 | Resource definitions, input globs, output paths | DSL data: `data testgen_resources`, `data makegen_resources` |
| `makegen/gitignore.rs` | 372 | Gitignore category patterns, build-system rules | DSL data: `data gitignore_categories` |
| `workflow/capabilities.rs` | 231 | Miss-reason enum variants, capability semantics | DSL sum type + fn |
| `workflow/global_plan.rs` | 186 | Cross-workflow dedup, capability namespaces | DSL pipeline annotations |
| `ci/mod.rs` | 99 | CI config, integrations, secret extraction | DSL data: `data ci_config`. Secret extraction needs inventory bridge. |
| `docgen/mod.rs` | 94 | Read target table (9 entries) | DSL data: `data docgen_read_targets` |
| `workflow/spec_builders.rs` | 93 | Workflow spec constructors | DSL pipeline → WorkflowSpec derivation in compiler |

**Subtotal: ~6,295 lines of domain knowledge in Rust that should be DSL.**

### Category C: Thin Wrappers → Delete (~145 lines)

One-line delegates to `build_dsl_graph("path")`. Structural entrypoint inference already makes these unnecessary.

| Module | Lines | Replacement |
|--------|-------|-------------|
| `bootstrap/mod.rs` | 25 | Structural inference from `tools/bootstrap.dag` |
| `build/mod.rs` | 24 | Structural inference from `tools/build.dag` |
| `codegen/mod.rs` | 31 | Structural inference from `tools/codegen.dag` |
| `infra/mod.rs` | 21 | Structural inference from `tools/infra.dag` |
| `deps_tool.rs` | 12 | Structural inference from `tools/deps.dag` |
| `gist.rs` | 5 | Delete (comment only) |
| `embedded_assets.rs` | 27 | Inline into makegen |

**Subtotal: ~145 lines deletable immediately or with minor refactoring.**

### Category D: Runtime Glue — Keep, Shrink (~1,950 lines)

Execution harness, DAG construction primitives, DSL evaluation bridges. Must remain Rust but is already minimal.

| Module | Lines | Why it stays |
|--------|-------|-------------|
| `lib.rs` | 170 | Crate public API, module organizer |
| `binaries.rs` | 171 | Workspace binary enum, Cargo.toml verification |
| `tool_runner.rs` | 94 | Shared execution ceremony |
| `dry_run.rs` | 90 | fs_env auto-mocking |
| `fs_env.rs` | 79 | DAG construction primitives |
| `pragma/dsl_render.rs` | 124 | DSL evaluation bridge |
| `pragma/mod.rs` | 27 | Module organizer |
| `policy/mod.rs` | 3 | Module organizer |
| `makegen/shared.rs` | 298 | DSL-backed Makefile rendering (already DSL) |
| `makegen/ci_render.rs` | 192 | WorkflowSpec → CI YAML (generic algorithm) |
| `makegen/justfile.rs` | 395 | Justfile rendering |
| `makegen/mod.rs` | 44 | Module organizer |
| `workflow/mod.rs` | 66 | Module organizer |
| `testgen_dag/mod.rs` | 21 | Module organizer |

**Subtotal: ~1,774 lines of necessary runtime glue.**

### Category E: Binaries — Already Covered

See `docs/design/binary-elimination.md` (RT58-RT66). 4,009 lines across 7 binaries, 5 to be eliminated.

## Migration Streams

### Stream A: Extract Generic Infrastructure to core/ (no DSL features needed)

Pure crate reorganization. No DSL changes required. Unblocks other projects from reusing gunbc's generic components.

**A1: Create `core/workflow` crate** (M)
Extract the 7 framework modules (planner, executor, admission, coordination, schema, key, process_registry, slo, projection, proof, errors) into a new `core/workflow` crate. gunbc-dag depends on it, keeps only catalog + unit_commands + spec_builders.

**A2: Move `resolve_service.rs` to `core/`** (S)
100% generic. Zero repo-specific logic. Move to `core/ir/src/transport/service_ops.rs` or a new `core/resolve` crate.

**A3: Split `resolve.rs`** (M)
Extract generic resolution framework (adapter ops, primitive mapping, resource lifecycle, ~1,600 lines) to `core/resolve`. Leave `resolve_domain()` dispatch (~700 lines) in gunbc-dag.

**A4: Move testgen engine to `core/codegen`** (M)
Move `dag_test_discovery.rs`, `graph.rs`, `mock_interpreter.rs`, `ops.rs`, `profile_discovery.rs` to `core/codegen/src/testgen/`. gunbc-dag calls the engine, doesn't own it.

**A5: Split `mock_defaults.rs`** (S)
Generic probing/builder framework → `core/test/src/auto_mock.rs`. GCP-specific field mappings stay in gunbc-dag (or move to DSL data).

**A6: Move `fidelity.rs` pattern to `core/codegen`** (S)
The compile-evaluate pattern is generic. Move to `core/codegen/src/dsl_evaluator.rs`. gunbc-dag calls it with repo-specific DSL file paths.

**A7: Move `dsl_builder.rs` pattern to `core/codegen`** (S)
Extract compile-then-resolve to `core/codegen/src/graph_builder.rs`. gunbc-dag keeps a thin adapter for workspace layout.

### Stream B: Replace Rust Domain Data with DSL (needs DSL evaluation)

Each item converts a hard-coded Rust table into a DSL `data` declaration evaluated at runtime via `evaluate_fn_body()`.

**B1: Workflow catalog → DSL data** (M)
Move `WORKFLOW_VARIANTS` table and `default_claims_for_stage()` to `dsl/config/workflow_catalog.dag`. Evaluate at runtime.

**B2: Unit commands → DSL data** (M)
Move per-workflow command tables to `dsl/config/workflow_commands.dag`. Each workflow unit gets a data record with `program` + `args`. Rust iterates declaratively.

**B3: Resource definitions → DSL data** (S)
Move `REPO_SOURCE_INPUT_GLOBS`, `TESTGEN_INPUT_GLOBS`, etc. to `dsl/config/resources.dag`. ResourceDef construction from DSL data.

**B4: Gitignore patterns → DSL data** (S)
Move gitignore category definitions to `dsl/config/gitignore.dag`. Already data-driven; just moving from Rust to DSL.

**B5: Docgen read targets → DSL data** (S)
Move `DOCGEN_READ_TARGETS` array to `dsl/tools/docgen.dag` as a `data` declaration.

**B6: Makegen registry → DSL data** (L)
The largest file (2,207 lines). Move BuildConfig, MetaTarget definitions, and ToolRegistry derivation to DSL. Keep Cargo command invocation in Rust. This is the most complex item.

**B7: Delete `policy/pragma.rs`** (S, depends: Phase 2 completion)
Once all 3 pragma renders work via `dsl_render.rs`, delete the Rust mirror. Already 2/3 done (allowlist + lint_policy). Blocked on clippy_toml (FC-CF5: recursive types).

### Stream C: Delete Thin Wrappers (trivial)

**C1: Delete tool module wrappers** (S)
Delete `bootstrap/mod.rs`, `build/mod.rs`, `codegen/mod.rs`, `deps_tool.rs`, `infra/mod.rs`, `gist.rs`, `embedded_assets.rs`. Replace with a single generic lookup function that uses structural entrypoint inference to find the right `.dag` module.

### Stream D: Migrate Extern Impls to DSL (needs compiler features)

Already tracked. 6 remaining extern symbols, each blocked on specific DSL features.

| Extern | Blocker | Status |
|--------|---------|--------|
| `render_clippy_toml` | Recursive types (FC-CF5) | Blocked |
| `render_tree` | Recursive types (FC-CF5) | Blocked |
| `build_snapshot_content` | Recursive types (FC-CF5) | Blocked |
| `render_bootstrap_gitignore` | DSL string rendering | Near |
| `render_bootstrap_makefile` | DSL string rendering | Near |
| `discover_tools` | Rust inventory access | Keep as extern |

## Impact

| Stream | Lines Moved/Deleted | gunbc-dag After |
|--------|--------------------|----|
| A (→ core/) | ~10,800 moved | 12,300 |
| B (→ DSL) | ~6,100 deleted (replaced by .dag) | 6,200 |
| C (delete wrappers) | ~145 deleted | 6,055 |
| D (extern → DSL) | ~500 deleted (4 of 6 externs) | 5,555 |
| E (binaries, RT58-66) | ~2,600 deleted | 2,955 |

**Final gunbc-dag**: ~2,955 lines of Rust = lib.rs + domain dispatch resolve + DSL eval bridges + runtime glue + 2 bootstrap binaries. Down from 23,107. An 87% reduction.

## Dependency Order

```
Stream C (delete wrappers) ───────────────────── immediate, no deps

Stream A (→ core/) ──────────────────────────── no DSL features needed
  A2 (resolve_service) ─┐
  A5 (mock_defaults)    ├── independent, parallel
  A6 (fidelity)         │
  A7 (dsl_builder)      ┘
  A1 (workflow crate) ──── after A2 (workflow may use resolve types)
  A3 (resolve split) ───── after A2 (resolve_service moved first)
  A4 (testgen engine) ──── after A5 (testgen uses mock_defaults)

Stream B (→ DSL) ────────────────────────────── needs DSL data evaluation
  B3 (resources)     ─┐
  B4 (gitignore)     ├── simple data, parallel
  B5 (docgen)        ┘
  B7 (pragma delete) ──── blocked on FC-CF5
  B1 (workflow catalog) ── after A1 (workflow types in core/)
  B2 (unit commands) ───── after B1 (uses catalog types)
  B6 (makegen registry) ── last (largest, most complex)

Stream D (extern → DSL) ─────────────────────── blocked on FC-CF5
  render_bootstrap_* ──── near, pending string rendering
  render_clippy_toml ──── blocked on recursive types
  render_tree ──────────── blocked on recursive types
  build_snapshot_content ── blocked on recursive types
  discover_tools ────────── keep as extern (inventory access)
```

## Verification

After each stream, verify:
```bash
cargo test --workspace
cargo clippy --all-targets -- -D warnings
# Stream A: verify moved crates compile independently
# Stream B: verify DSL data evaluation matches Rust output (parity tests)
# Stream C: verify structural inference finds all tools
```
