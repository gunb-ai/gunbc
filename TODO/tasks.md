# Task Sheet — Active Mega Lane

**Last updated**: 2026-02-25
**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Archive**:
- Completed items: `TODO/TODONE/2026-Q1/tasks-completed.md`
- Archived lane detail snapshot: `TODO/TODONE/2026-Q1/tasks-archived-lanes-2026-02-25.md`
- Backlog: `TODO/backlog.md`

**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

**Scheduling policy (2026-02-25)**: Only **Lane 1** is active.

## Delivery Lane Summary

| Lane | Status | Remaining |
|------|--------|-----------|
| 1: Mega lane — compile+link hardening + interface stubs + fail-closed cleanup | In Progress | NF-4..NF-7 |

---

## Lane 1: Mega Lane (Compiler Contract + Testgen Unblocking + Cleanup)

**Goal**: Deliver one coherent hardening wave that combines:
1. Compile+link no-fallback contract for extern funcs/assets.
2. Interface stub transport + per-profile live test generation.
3. Removal of known fail-open/codegen workaround paths.

**Design references (source of truth)**:
- `docs/design/v4/domain-hard-error-no-fallback-plan.md`
- `docs/design/interface-stub-transport.md`
- `docs/design/v4/externcall-same-module-port-wiring.md` (NF-7)
- `docs/design/v4/extern-bridge-gap-analysis.md` (Phases 5-8: full extern elimination, extends NF-7)

### Track A: Compile+Link No-Fallback Hardening

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **NF-1** | **Extern DSL surface**: Add `extern func` and `extern asset` syntax/typechecking/lowering so runtime-provided behavior is explicit in DSL. | -- | L | **Done** |
| **NF-2** | **Minimal symbol model**: Introduce canonical `SymbolId` + `NodeId` model and lower ops to `Intrinsic`/`Call`/`Extern`. | NF-1 | L | **Done** |
| **NF-3** | **Link step + backend resolver contract**: Add linker stage that resolves extern funcs/assets through backend interfaces and emits hard missing-symbol errors. | NF-2 | L | **Done** |
| **NF-4** | **Collapse link phase into compile-time resolution**: Delete separate linker stage (`Backend` trait, `link()`, `SymbolTable`, `OpRef`, `IntrinsicOp`). Resolution already happens at compile time in `resolve.rs`; `extern` keyword + `ExternCall` + `resolve_extern_call()` remain as the fail-closed contract. | NF-3 | M | In Progress |
| **NF-5** | **Delete fallback surfaces**: Remove passthrough controls/handlers, stub asset fallbacks, and module-name dispatch heuristics. | NF-4 | M | Planned |
| **NF-6** | **Determinism contract hardening**: Add compile receipt digests linked to emit-manifest and CI determinism gates (single-file, CI pipeline, directory compile) with deterministic diagnostic ordering. | NF-5 | M | Planned |
| **NF-7** | **Lowerer design + fix for shadow-fn to `extern func` conversion**: Support converting shadow `fn` items to `extern func` in DSL files by wiring `ExternCall` output ports correctly for same-module calls from function bodies (codegen data flow currently breaks). Keep shadow fn bodies as documented placeholders until this lands. This is a lowerer limitation, not a DSL design choice. Deliver a design note and implementation/tests. | NF-4 | L | Planned |

**What didn't land (carry-forward)**:
- Converting shadow `fn` items to `extern func` in DSL files.
- Current blocker: lowerer does not wire `ExternCall` output ports correctly for same-module calls from function bodies, which breaks codegen data flow.
- Shadow function bodies stay in place with clear documentation for now.
- This is a lowerer limitation, not a design choice.

### Track B: Interface Stub Transport

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **IS-1** | **Add `InterfaceStub` to `ServiceTransportClass`**: New enum variant in `daglang-lower`. Audit all match arms for exhaustiveness. | -- | S | **Done** |
| **IS-2** | **Add `add_interface_stub_transport_triplets()`**: Mirror resource capability transport pattern. Walk `InterfaceDef.capabilities`, create prepare/execute/parse triplets with `InterfaceStub` transport class. | IS-1 | M | **Done** |
| **IS-3** | **Relax `enforce_profile_for_bound_uses()`**: Convert hard error to informational. Return `HashSet<String>` of interface types needing stubs. (Stopgap: `requires_profile` filter in `build_testgen_graph_auto()` skips these modules — remove filter when IS-3 lands.) *Runtime graceful degradation in place*: `AutoGenerate` now emits placeholder content on compile error instead of hard-failing the testgen DAG. | IS-1 | S | **Done** |
| **IS-4** | **Wire stubs into lowering flow**: Call `add_interface_stub_transport_triplets()` after service transport, merge into endpoint registry. | IS-2, IS-3 | S | **Done** |
| **IS-5** | **Update `resolve_service_call_source()` fallback**: Try `cap_key` lookup when `active_profile_bindings` is `None`. Only error if stub lookup also fails. | IS-3 | S | **Done** |
| **IS-6** | **Handle `InterfaceStub` in DynOp resolver**: `InterfaceStubPrepareOp`, `InterfaceStubExecuteOp` (errors in Real mode, auto-mocked in DryRun), `InterfaceStubParseOp`. | -- | M | **Done** |
| **IS-7** | **Verify auto-mock compatibility**: Confirm stub execute nodes carry `ServiceTransportExecute` obligation for auto-mock. | IS-4 | S | **Done** |
| **IS-8** | **Tests**: Lowerer test (no profile -> stub triplets), resolver test (Real mode error), integration (`make test-all`). | IS-4, IS-6, IS-7 | M | **Done** |

### Track C: Per-Profile Live Tests

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **PT-1** | **Profile discovery module**: Scan `dsl/profiles/*.dag`, extract profile name, bound interfaces, env/secret requirements, inferred test class. | IS-8 | M | **Done** |
| **PT-2** | **Augment `CompilableModule` with interface imports**: Upgrade `requires_profile: bool` to `interface_imports: HashSet<String>` populated from `import interfaces.*` in AST. | IS-8 | S | **Done** |
| **PT-3** | **Add `LiveProfileTestConfig` to `TestgenTargetDef`**: `profile_name`, `test_class`, `fermi_cost`, `required`, `required_any_of`, `dag_builder_call`. | IS-8 | S | **Done** |
| **PT-4** | **Add `build_dsl_graph_with_profile()`**: New compilation path threading `profile` through `CompileOptions` with `allow_placeholder_env`. | IS-8 | M | **Done** |
| **PT-5** | **Generate per-profile test sections in codegen**: `build_per_profile_live_flow_sections()` — one `test_live_flow_{module}_{profile}()` per config, gated by env requirements. | PT-3, PT-4 | M | **Done** |
| **PT-6** | **Wire profile discovery into auto-testgen pipeline**: `discover_profiles()` in graph build, `profiles_for_module()` per module, populate `live_profile_tests`. | PT-1, PT-2, PT-5 | M | **Done** |

### Track D: Fail-Closed Cleanup + Workaround Removal

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **FC-1** | **Fail closed for unsupported DSL expression codegen**: Replace comment/raw-code fallbacks (`/* unsupported ... */`) with typed errors in expression compilation/rendering paths. | -- | M | **Done** |
| **FC-2** | **String interpolation correctness**: Implement `StringInterp` generation correctly or hard-error until fully supported. Remove `"{}"` placeholder-only emission. | FC-1 | M | **Done** |
| **FC-3** | **Record/variant context typing fix**: Remove field-name-as-type-context behavior in record rendering; plumb real type context or make variants fully-qualified. | FC-1 | M | **Done** |
| **FC-4** | **No panic in lowering for unsupported patterns**: Replace panic-based unsupported `PatternOp` paths with structured lowering errors. | -- | S | **Done** |
| **FC-5** | **SubDag runtime op cfg hygiene**: Verify SubDag dispatch runtime op is not `#[cfg(test)]`-gated in production and add release-build guardrail coverage. | -- | S | **Done** |
| **FC-6** | **Pipeline dispatch contract hardening**: Make stage progression contract explicit for resume/current-stage behavior; validate expected fields and improve diagnostics. | FC-5 | S | **Done** |
| **FC-7** | **Remove node-name substring extraction hacks**: Replace `content_upsert_path_` ID substring checks with explicit metadata/annotation-based output path extraction. | NF-2 | M | **Done** |
| **FC-8** | **Fail-closed content extraction + test restoration**: Make `extract_file_contents` schema handling fail-closed (or explicit warning mode) and restore non-empty gist dry-run tests with real assertions. | FC-7 | M | **Done** |
| **FC-9** | **Define and enforce `\\xHH` string semantics**: Align lexer + emitter behavior for hex-escape handling and add deterministic contract tests. | FC-1 | S | **Done** |
| **FC-10** | **Proper `@local` transport type**: `GenericLocalPrepareOp` wraps inputs in `ShellRequest::new("echo")` as carrier; `GenericLocalParseOp` expects `TransportResponse::Shell`. Add `TransportRequest::Local` / `TransportResponse::Local` variants in `core/ir/src/transport/mod.rs`, add `TransportKind::LocalDirect` in `daglang-emit`, update prepare/parse ops. | -- | M | **Done** |
| **FC-11** | **Collapse `service_prepare_ports()` match arms**: 4 near-identical arms (`Rest`, `Shell`, `File`, `Local`) each doing `spec.input_fields.iter().map(...)`. Add `ServiceOperationSpec::input_fields(&self) -> &[FieldSpec]` method. | -- | S | **Done** |
| **FC-12** | **Fix `WorkspaceBinary` enum alignment**: Hooks added `review-design` binary without updating enum. Add `ReviewDesign` variant to `gunbc-dag/src/binaries.rs`. | -- | S | **Done** |
| **FC-13** | **Fix `workspace_crates()` count**: Removed 3 phantom crates (`lib/git-ops`, `lib/azure-ops`, `lib/markdown`) not in root Cargo.toml workspace members. | -- | S | **Done** |
| **FC-14** | **Tonight note: compiler-wide no-dead-path emit variant (hardening pass)**: Add a compile/codegen variant that prunes unused code paths and imports across generated outputs (Rust/Go/C) so strict `-D warnings` builds do not fail on dead emit branches. Include parity tests proving behavior is unchanged for live paths. | FC-1 | M | **Done** |
| **FC-15** | **Next-night follow-up: make unused-path emission structurally impossible (design-first, hard requirement)**: Write and approve a design doc that makes “emit only reachable code” a compiler invariant. End state must be by-construction reachability in IR/backend contracts, not cleanup/pruning passes. Include: invariant spec, failure modes, proof obligations, migration plan, and contract tests. Then ship one flag-gated vertical slice that enforces the invariant for one target end-to-end. | FC-14 | L | **Done** |

### Mega-lane dependency guide

1. `NF-1 -> NF-2 -> NF-3 -> NF-4 -> NF-5 -> NF-6`, with `NF-7` in parallel after `NF-4`.
2. `IS-1 -> (IS-2, IS-3) -> IS-4 -> IS-7 -> IS-8`, with `IS-6` in parallel.
3. `IS-8 -> (PT-1, PT-2, PT-3, PT-4) -> PT-5 -> PT-6`
4. `FC-1 -> (FC-2, FC-3, FC-9)`
5. `FC-5 -> FC-6`
6. `NF-2 -> FC-7 -> FC-8`
7. `FC-4`, `FC-10`, `FC-11`, `FC-12`, `FC-13`, `FC-14` parallel.
8. `FC-14 -> FC-15`

### Lane 1 files touched (aggregate)

| File | Changes |
|------|---------|
| `core/daglang/daglang-lower/src/lib.rs` | Extern/lowering hardening + `InterfaceStub` + stub triplets |
| `core/daglang/daglang-cli/src/*` | Link-stage/compile-mode contract wiring and diagnostics surfaces |
| `core/daglang/daglang-driver/src/*` | Compile+link orchestration + deterministic receipt plumbing |
| `core/daglang/daglang-emit/src/*` | Remove fallback paths, link-time extern handling, determinism integration |
| `gunbc-dag/src/resolve.rs` | `InterfaceStub` ops + no-fallback resolution cleanup |
| `gunbc-dag/src/mock_defaults.rs` | Auto-mock compatibility checks for stub transports |
| `gunbc-dag/src/testgen_dag/profile_discovery.rs` | Profile scanning |
| `gunbc-dag/src/testgen_dag/dag_test_discovery.rs` | Interface imports on `CompilableModule` |
| `core/codegen/src/registry.rs` | `LiveProfileTestConfig` |
| `gunbc-dag/src/dsl_builder.rs` | Profile-aware compilation |
| `core/codegen/src/testgen/codegen.rs` | Per-profile test generation + fail-closed behavior changes |
| `gunbc-dag/src/testgen_dag/{graph.rs, ops.rs}` | Pipeline wiring for profile-aware testgen |
| `core/codegen/src/*`, `core/daglang/daglang-*/src/*`, `gunbc-dag/src/*` | Fail-closed cleanup, fallback removal, and diagnostics hardening (FC-1..FC-9) |
