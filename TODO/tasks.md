# Task Sheet — Active Lanes Only

**Last updated**: 2026-02-25
**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Archive**:
- Completed items: `TODO/TODONE/2026-Q1/tasks-completed.md`
- Archived lane detail snapshot: `TODO/TODONE/2026-Q1/tasks-archived-lanes-2026-02-25.md`
- Backlog: `TODO/backlog.md`

**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

**Scheduling policy (2026-02-25)**: Only **Lane 7** and **Lane 8** are active.

## Delivery Lane Summary

| Lane | Status | Remaining |
|------|--------|-----------|
| 7: Compile+link no-fallback hardening | Planned | NF-1..NF-6 |
| 8: Interface stub transport + fail-closed cleanup | Planned | IS-1..IS-8, PT-1..PT-6, FC-1..FC-9 |

---

## Lane 7: Compile+Link No-Fallback Hardening

**Goal**: Eliminate string-coupled/runtime fallback behavior by adopting compile+link semantics: extern symbol resolution, hard missing-symbol errors, and deterministic receipts.

**Design reference (source of truth)**: `docs/design/v4/domain-hard-error-no-fallback-plan.md`

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **NF-1** | **Extern DSL surface**: Add `extern func` and `extern asset` syntax/typechecking/lowering so runtime-provided behavior is explicit in DSL. | -- | L | Planned |
| **NF-2** | **Minimal symbol model**: Introduce canonical `SymbolId` + `NodeId` model and lower ops to `Intrinsic`/`Call`/`Extern`. | NF-1 | L | Planned |
| **NF-3** | **Link step + backend resolver contract**: Add linker stage that resolves extern funcs/assets through backend interfaces and emits hard missing-symbol errors. | NF-2 | L | Planned |
| **NF-4** | **Runtime/asset migration to extern symbols**: Convert existing runtime handler + embedded asset flows to extern symbol resolution. Remove hidden authority from CLI/emitter registries. | NF-3 | L | Planned |
| **NF-5** | **Delete fallback surfaces**: Remove passthrough controls/handlers, stub asset fallbacks, and module-name dispatch heuristics. | NF-4 | M | Planned |
| **NF-6** | **Determinism contract hardening**: Add compile receipt digests linked to emit-manifest and CI determinism gates (single-file, CI pipeline, directory compile) with deterministic diagnostic ordering. | NF-5 | M | Planned |

### Lane 7 exit criteria

1. No CLI/emitter/runtime fallback path remains for unresolved extern funcs/assets.
2. Runtime and embedded assets resolve through link-time extern symbol contracts.
3. Missing symbol failures are deterministic in both set and order.
4. Determinism receipts and emit manifests are stable across repeated runs.

---

## Lane 8: Interface Stub Transport + Per-Profile Live Tests + Fail-Closed Cleanup

**Goal**: Unblock testgen for interface-using modules (Part 1/2) and retire remaining fail-open/codegen workaround paths called out in 2026-02-24 cleanup feedback (Part 3).

**Design reference (source of truth)**: `docs/design/interface-stub-transport.md`

### Part 1: Interface Stub Transport

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **IS-1** | **Add `InterfaceStub` to `ServiceTransportClass`**: New enum variant in `daglang-lower`. Audit all match arms for exhaustiveness. | -- | S | Planned |
| **IS-2** | **Add `add_interface_stub_transport_triplets()`**: Mirror resource capability transport pattern. Walk `InterfaceDef.capabilities`, create prepare/execute/parse triplets with `InterfaceStub` transport class. | IS-1 | M | Planned |
| **IS-3** | **Relax `enforce_profile_for_bound_uses()`**: Convert hard error to informational. Return `HashSet<String>` of interface types needing stubs. | IS-1 | S | Planned |
| **IS-4** | **Wire stubs into lowering flow**: Call `add_interface_stub_transport_triplets()` after service transport, merge into endpoint registry. | IS-2, IS-3 | S | Planned |
| **IS-5** | **Update `resolve_service_call_source()` fallback**: Try `cap_key` lookup when `active_profile_bindings` is `None`. Only error if stub lookup also fails. | IS-3 | S | Planned |
| **IS-6** | **Handle `InterfaceStub` in DynOp resolver**: `InterfaceStubPrepareOp`, `InterfaceStubExecuteOp` (errors in Real mode, auto-mocked in DryRun), `InterfaceStubParseOp`. | -- | M | Planned |
| **IS-7** | **Verify auto-mock compatibility**: Confirm stub execute nodes carry `ServiceTransportExecute` obligation for auto-mock. | IS-4 | S | Planned |
| **IS-8** | **Tests**: Lowerer test (no profile -> stub triplets), resolver test (Real mode error), integration (`make test-all`). | IS-4, IS-6, IS-7 | M | Planned |

### Part 1 dependency graph

```
IS-1 ──┬──> IS-2 ──> IS-4 ──> IS-7 ──> IS-8
       │              ^
IS-3 ──┘──> IS-5 ────/
IS-6 (parallel with IS-2..IS-5)
```

### Part 2: Per-Profile Live Tests

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **PT-1** | **Profile discovery module**: Scan `dsl/profiles/*.dag`, extract profile name, bound interfaces, env/secret requirements, inferred test class. | IS-8 | M | Planned |
| **PT-2** | **Augment `CompilableModule` with interface imports**: Add `interface_imports: HashSet<String>` populated from `import interfaces.*` in AST. | IS-8 | S | Planned |
| **PT-3** | **Add `LiveProfileTestConfig` to `TestgenTargetDef`**: `profile_name`, `test_class`, `fermi_cost`, `required`, `required_any_of`, `dag_builder_call`. | IS-8 | S | Planned |
| **PT-4** | **Add `build_dsl_graph_with_profile()`**: New compilation path threading `profile` through `CompileOptions` with `allow_placeholder_env`. | IS-8 | M | Planned |
| **PT-5** | **Generate per-profile test sections in codegen**: `build_per_profile_live_flow_sections()` — one `test_live_flow_{module}_{profile}()` per config, gated by env requirements. | PT-3, PT-4 | M | Planned |
| **PT-6** | **Wire profile discovery into auto-testgen pipeline**: `discover_profiles()` in graph build, `profiles_for_module()` per module, populate `live_profile_tests`. | PT-1, PT-2, PT-5 | M | Planned |

### Part 2 dependency graph

```
PT-1 ──> PT-6
PT-2 ──> PT-6
PT-3 ──> PT-5 ──> PT-6
PT-4 ──> PT-5
```

### Part 3: Fail-Closed Cleanup + Workaround Removal

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **FC-1** | **Fail closed for unsupported DSL expression codegen**: Replace comment/raw-code fallbacks (`/* unsupported ... */`) with typed errors in expression compilation/rendering paths. | -- | M | Planned |
| **FC-2** | **String interpolation correctness**: Implement `StringInterp` generation correctly or hard-error until fully supported. Remove `"{}"` placeholder-only emission. | FC-1 | M | Planned |
| **FC-3** | **Record/variant context typing fix**: Remove field-name-as-type-context behavior in record rendering; plumb real type context or make variants fully-qualified. | FC-1 | M | Planned |
| **FC-4** | **No panic in lowering for unsupported patterns**: Replace panic-based unsupported `PatternOp` paths with structured lowering errors. | -- | S | Planned |
| **FC-5** | **SubDag runtime op cfg hygiene**: Verify SubDag dispatch runtime op is not `#[cfg(test)]`-gated in production and add release-build guardrail coverage. | -- | S | Planned |
| **FC-6** | **Pipeline dispatch contract hardening**: Make stage progression contract explicit for resume/current-stage behavior; validate expected fields and improve diagnostics. | FC-5 | S | Planned |
| **FC-7** | **Remove node-name substring extraction hacks**: Replace `content_upsert_path_` ID substring checks with explicit metadata/annotation-based output path extraction. | NF-2 | M | Planned |
| **FC-8** | **Fail-closed content extraction + test restoration**: Make `extract_file_contents` schema handling fail-closed (or explicit warning mode) and restore non-empty gist dry-run tests with real assertions. | FC-7 | M | Planned |
| **FC-9** | **Define and enforce `\\xHH` string semantics**: Align lexer + emitter behavior for hex-escape handling and add deterministic contract tests. | FC-1 | S | Planned |

### Part 3 dependency graph

```
FC-1 ──> FC-2
   └──> FC-3
   └──> FC-9
FC-5 ──> FC-6
NF-2 ──> FC-7 ──> FC-8
FC-4 (parallel)
```

### Lane 8 exit criteria

1. All interface-using modules compile without `--profile` and produce valid DryRun-testable DAGs.
2. Testgen coverage increases from 21/30 to ~30/30 compilable modules.
3. Per-profile live tests appear in generated test files, gated by env requirements.
4. No comment/raw-code fallback paths remain in expression codegen for unsupported constructs.
5. Known fail-open hacks from 2026-02-24 cleanup feedback are resolved or replaced by explicit errors.
6. `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings` clean.

### Lane 8 files touched

| File | Changes |
|------|---------|
| `core/daglang/daglang-lower/src/lib.rs` | `InterfaceStub` variant, stub transport triplets, relaxed validation (IS-1..IS-5) |
| `gunbc-dag/src/resolve.rs` | Stub ops in DynOp resolver (IS-6) |
| `gunbc-dag/src/mock_defaults.rs` | Auto-mock verification (IS-7) |
| `gunbc-dag/src/testgen_dag/profile_discovery.rs` | New — profile scanning (PT-1) |
| `gunbc-dag/src/testgen_dag/dag_test_discovery.rs` | Interface imports on `CompilableModule` (PT-2) |
| `core/codegen/src/registry.rs` | `LiveProfileTestConfig` (PT-3) |
| `gunbc-dag/src/dsl_builder.rs` | Profile-aware compilation (PT-4) |
| `core/codegen/src/testgen/codegen.rs` | Per-profile test generation (PT-5) |
| `gunbc-dag/src/testgen_dag/{graph.rs, ops.rs}` | Pipeline wiring (PT-6) |
| `core/codegen/src/*`, `core/daglang/daglang-*/src/*`, `gunbc-dag/src/*` | Fail-closed cleanup, fallback removal, and diagnostics hardening (FC-1..FC-9) |
