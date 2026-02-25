# Interface Stub Transport & Per-Profile Live Tests

## Context

Modules using abstract interfaces (e.g., `uses issues: IssueProvider`) fail to compile without `--profile`, blocking testgen for ~10 modules. The interface **does** define enough behavior for DryRun tests — capability port shapes (inputs/outputs), behavioral contracts (`@readonly`, `@idempotent`). Transport details are unknowable without a profile, but DryRun mocks transport anyway.

**Part 1**: Generate stub transport triplets from interface capability shapes in the lowerer. Any compilation without `--profile` produces a structurally valid DAG with stub transport nodes. DryRun works; Real execution errors with a clear "requires --profile" message. Unblocks testgen for all interface-using modules.

**Part 2**: Generate one live test per profile for integration/medium coverage.

---

## Part 1: Interface Stub Transport

### What changes

The lowerer currently has two gates that reject interface-using modules without a profile:
1. `enforce_profile_for_bound_uses()` (line 1056) — early validation, fails fast
2. `resolve_service_call_source()` (line 5117) — late resolution, can't find endpoint

The fix: generate stub transport triplets from `InterfaceDef.capabilities` (same structure as `ResourceDef.capabilities` at line 4590+), register them in the endpoint registry, and let the existing `cap_key` lookup resolve them.

### IS-1: Add `InterfaceStub` to `ServiceTransportClass` (S)

**File**: `core/daglang/daglang-lower/src/lib.rs` (line 229)

Add `InterfaceStub` variant. Audit all `match` arms on this enum for exhaustiveness.

### IS-2: Add `add_interface_stub_transport_triplets()` (M)

**File**: `core/daglang/daglang-lower/src/lib.rs`

Mirror the existing resource capability transport pattern (lines 4590-4730). For each `InterfaceDef` whose name is in `profile_bound_interfaces`:

- Walk `interface.capabilities`
- Create prepare/execute/parse triplet nodes using `capability_prepare_ports()` (line 4387 — already handles `CapabilityDef` with `spec: None` fallback to capability inputs)
- Set `transport: ServiceTransportClass::InterfaceStub`, `spec: None`
- Register in `ServiceEndpointRegistry` under same key patterns as services

Key: `capability_prepare_ports()` already falls through to `capability.inputs` when `metadata.spec` is `None` (line 4394). No changes needed to that function.

### IS-3: Relax `enforce_profile_for_bound_uses()` (S)

**File**: `core/daglang/daglang-lower/src/lib.rs` (line 1056)

Convert from hard error to informational. Return `HashSet<String>` of interface types needing stubs instead of `Err`. Replace call at line 1615 with:

```rust
let stub_interfaces = interfaces_needing_stubs(project, active_profile, &profile_bound_interfaces);
```

### IS-4: Wire stubs into lowering flow (S)

**File**: `core/daglang/daglang-lower/src/lib.rs` (around line 1615)

After `add_service_transport_triplets()`, conditionally call `add_interface_stub_transport_triplets()` when `stub_interfaces` is non-empty. Merge into main `endpoints_by_full` registry (concrete entries take priority via `or_insert`).

### IS-5: Update `resolve_service_call_source()` fallback (S)

**File**: `core/daglang/daglang-lower/src/lib.rs` (line 5117)

When `active_profile_bindings` is `None`, instead of erroring, try the same `cap_key` lookup that the resource capability path uses (lines 5102-5113). Only error if stub lookup also fails.

### IS-6: Handle `InterfaceStub` in DynOp resolver (M)

**File**: `gunbc-dag/src/resolve.rs`

Add three small ops:
- `InterfaceStubPrepareOp` — packages inputs into a stub `TransportRequest`
- `InterfaceStubExecuteOp` — errors with clear message ("requires --profile") in Real mode; auto-mocked in DryRun
- `InterfaceStubParseOp` — safety net (never reached in DryRun)

Branch on `metadata.transport == InterfaceStub` in the existing `resolve_service_transport()`.

### IS-7: Verify auto-mock compatibility (S)

**Files**: `gunbc-dag/src/mock_defaults.rs`, `core/exec/src/execute.rs`

Existing auto-mock keys on `ObligationCategory::ServiceTransportExecute` — interface stub execute nodes carry this same obligation. Likely no changes needed.

### IS-8: Tests (M)

- Lowerer test: compile `InterfaceDef` + `func` using it, no profile -> stub triplets with correct port shapes
- Resolver test: stub ops resolve correctly, execute op errors in Real mode
- Integration: `make test-all` passes

### Dependency graph

```
IS-1 ──┬──> IS-2 ──> IS-4 ──> IS-7 ──> IS-8
       │              ^
IS-3 ──┘──> IS-5 ────/
IS-6 (parallel with IS-2..IS-5)
```

---

## Part 2: Per-Profile Live Tests (Follow-up)

### PT-1: Profile discovery module (M)

**New file**: `gunbc-dag/src/testgen_dag/profile_discovery.rs`

Scan `dsl/profiles/*.dag`, parse ASTs to extract:
- Profile name and bound interface names
- Env requirements (from `env("VAR")` config entries)
- Secret requirements (from `secret("NAME")` config entries)
- Inferred test class/fermi cost (hermetic stubs -> `Hermetic/XS`, env-backed -> `Integration/M`, secret-backed -> `Integration/L`)

### PT-2: Augment `CompilableModule` with interface imports (S)

**File**: `gunbc-dag/src/testgen_dag/dag_test_discovery.rs`

Add `interface_imports: HashSet<String>`. Populate during `collect_dag_files()` by checking `import interfaces.*` in the AST.

### PT-3: Add `LiveProfileTestConfig` to `TestgenTargetDef` (S)

**File**: `core/codegen/src/registry.rs`

New struct with `profile_name`, `test_class`, `fermi_cost`, `required`, `required_any_of`, `dag_builder_call`. Add `live_profile_tests: Vec<LiveProfileTestConfig>` to `TestgenTargetDef`.

### PT-4: Add `build_dsl_graph_with_profile()` (M)

**File**: `gunbc-dag/src/dsl_builder.rs`

New compilation path threading `profile: &str` through `CompileOptions`. Needs `allow_placeholder_env: bool` in `CompileOptions` so testgen can compile with profiles referencing missing env vars.

### PT-5: Generate per-profile test sections in codegen (M)

**File**: `core/codegen/src/testgen/codegen.rs`

New `build_per_profile_live_flow_sections()`. For each `LiveProfileTestConfig`:
- Generate `test_live_flow_{module}_{profile}()`
- Gate with `guard_test_with_env()` using profile's env requirements
- Build DAG with `build_dsl_graph_with_profile(path, profile)`
- Execute with `ExecutionMode::Real`

### PT-6: Wire profile discovery into auto-testgen pipeline (M)

**Files**: `gunbc-dag/src/testgen_dag/{graph.rs, ops.rs, dag_test_discovery.rs}`

- `build_testgen_graph_auto()`: call `discover_profiles()` once, then `profiles_for_module()` per module
- `TestgenOp::AutoGenerate`: add `base_profile: Option<String>` and `live_profile_tests`
- `auto_testgen_for_module()`: use first available profile for base compilation; populate `live_profile_tests` for all applicable profiles

### Dependency graph

```
PT-1 ──> PT-6
PT-2 ──> PT-6
PT-3 ──> PT-5 ──> PT-6
PT-4 ──> PT-5
```

---

## Verification

### Part 1
1. `make test-all` passes — testgen succeeds for `test_control_flow`, `sdlc_stages`, etc.
2. `cargo test --workspace` — all existing + new tests pass
3. `cargo clippy --all-targets -- -D warnings` — clean

### Part 2
1. Per-profile live test functions appear in generated test files
2. `cargo test test_live_flow_sdlc_worker_unit_test` runs (hermetic, no env vars)
3. Integration tests skip gracefully when env vars missing
