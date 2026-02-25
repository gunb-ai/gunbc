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

### Key design decisions

#### D1: Parse node runs in DryRun — it is NOT unreachable

DryRun interception replaces **only the transport executor** (the node whose input port is typed `TransportRequest`). The rest of the DAG — including parse — still runs normally. Therefore parse must be able to execute without real transport details.

**Solution**: Stub execute nodes output typed capability outputs directly (the same port shapes as the interface capability's declared outputs). Parse is a trivial passthrough/identity op. This avoids inventing a stub `TransportResponse` encoding.

The port contract for stub triplets is:

| Stage | Inputs | Outputs |
|-------|--------|---------|
| **prepare** | Capability input fields | `request: TransportRequest` |
| **execute** | `request: TransportRequest` | Typed capability output fields (not `response: TransportResponse`) |
| **parse** | Typed capability output fields | Typed capability output fields (identity passthrough) |

In DryRun, the execute node is intercepted and boundary mocks provide the typed capability outputs as witnesses. Parse forwards them unchanged.

In Real mode, execute errors immediately with "requires --profile".

#### D2: Stub endpoints use a real spec variant (not `spec: None`)

The resolver routing logic (`resolve.rs:731-744`) only routes non-service-module `service_transport::*` nodes when:
```rust
let has_spec = service_metadata.as_ref().is_some_and(|m| m.spec.is_some());
let is_execute = name.starts_with("service_transport::execute::");
if has_spec || is_execute { resolve_service_transport(...) }
```

With `spec: None`, prepare and parse nodes **fall through** to `DeclaredOutputCallableOp` (identity passthrough based on output port names). This would produce wrong behavior silently.

**Solution**: Add `ServiceOperationSpec::InterfaceStub { interface, capability }` so `spec.is_some()` is true. This uses the existing "has spec → resolvable behavior" invariant and avoids sprinkling special cases in the routing logic.

#### D3: Single-path registry resolution (no string munging)

`resolve_service_call_source()` resolves via the endpoint registry using the canonical call path. Stub endpoints are registered under the same key scheme as real endpoints. Resolution is a single lookup — no string rebuild/split fallback.

Policy:
1. Always attempt endpoint resolution via the registry using the canonical call path.
2. If resolved endpoint is a stub and there's no profile binding, return it with `binding_config: None`.
3. Only error if endpoint can't be found at all.

#### D4: Real-mode preflight (optional UX improvement)

Instead of failing at the first stub execute node encountered in topological order, add a preflight check: if `ExecutionMode::Real` and the DAG contains any `InterfaceStub` endpoints, fail fast and list all required interfaces in a single error message.

---

### IS-1: Add `InterfaceStub` to `ServiceTransportClass` and `ServiceOperationSpec` (S)

**File**: `core/daglang/daglang-lower/src/lib.rs`

Add `InterfaceStub` variant to `ServiceTransportClass` (line 229). Add `InterfaceStub { interface: String, capability: String }` variant to `ServiceOperationSpec` (line 259). Audit all `match` arms on both enums for exhaustiveness.

**Invariant preserved**: `spec.is_some()` is true for stub endpoints, so resolver routing at `resolve.rs:739` treats them as concrete.

### IS-2: Add `add_interface_stub_transport_triplets()` (M)

**File**: `core/daglang/daglang-lower/src/lib.rs`

Mirror the existing resource capability transport pattern (lines 4590-4730). For each `InterfaceDef` whose name is in `profile_bound_interfaces` but has no active profile binding:

- Walk `interface.capabilities`
- Create prepare/execute/parse triplet nodes using `capability_prepare_ports()` (line 4387 — already handles `CapabilityDef` with `spec: None` fallback to capability inputs)
- Set `transport: ServiceTransportClass::InterfaceStub`
- Set `spec: Some(ServiceOperationSpec::InterfaceStub { interface, capability })`
- **Execute node outputs**: typed capability output fields (NOT `response: TransportResponse`). Port shapes derived from `CapabilityDef.outputs`.
- **Parse node**: inputs = typed capability output fields, outputs = same (identity). Edges wire execute outputs → parse inputs directly.
- Register in `ServiceEndpointRegistry` under same key patterns as services

Key: `capability_prepare_ports()` already falls through to `capability.inputs` when `metadata.spec` is `None` (line 4394). No changes needed to that function.

### IS-3: Relax `enforce_profile_for_bound_uses()` (S)

**File**: `core/daglang/daglang-lower/src/lib.rs` (line 1056)

Convert from hard error to informational warning. Return `HashSet<String>` of interface types needing stubs instead of `Err`. Replace call at line 1615 with:

```rust
let stub_interfaces = interfaces_needing_stubs(project, active_profile, &profile_bound_interfaces);
```

### IS-4: Wire stubs into lowering flow (S)

**File**: `core/daglang/daglang-lower/src/lib.rs` (around line 1615)

Build endpoint registry in two layers:
1. Real endpoints from active profile bindings (if present) — via existing `add_service_transport_triplets()`
2. Stub endpoints for unresolved interface capabilities — via `add_interface_stub_transport_triplets()`

Real overrides stub deterministically: insert real first, then `or_insert` for stubs (stubs only fill gaps).

### IS-5: Simplify `resolve_service_call_source()` resolution (S)

**File**: `core/daglang/daglang-lower/src/lib.rs` (line 5117)

Remove the special-case error path when `active_profile_bindings` is `None`. Resolution is now a single policy:
1. Always attempt endpoint resolution via the registry using the canonical call path.
2. Stub endpoints resolve normally (they were registered in IS-4).
3. Only error if endpoint can't be found in the registry at all.

No string munging, no cap_key rebuild/split. Same lookup path for real and stub endpoints.

### IS-6: Handle `InterfaceStub` in DynOp resolver (M)

**File**: `gunbc-dag/src/resolve.rs`

In `resolve_service_transport()` (line 846), add a branch for `ServiceOperationSpec::InterfaceStub`:

- **Prepare**: `InterfaceStubPrepareOp` — packages inputs into a `TransportRequest` (same as other prepare ops). Enables structural transport detection.
- **Execute**: `InterfaceStubExecuteOp` — in Real mode, errors with `"Interface call requires --profile: {interface}.{capability} (no active profile bindings)"`. In DryRun, auto-mocked (never actually runs; boundary mocks supply typed outputs).
- **Parse**: `InterfaceStubParseOp` — identity/passthrough. Forwards typed capability outputs from execute (or from DryRun mocks) unchanged.

The existing `(spec, is_prepare, is_parse)` triple match (line 864) gains three new arms — one per InterfaceStub phase. No changes to routing conditions needed (D2 ensures `spec.is_some()`).

**Optional preflight** (D4): In `resolve_service_transport()` or a wrapper, if `ExecutionMode::Real` and spec is `InterfaceStub`, collect all such nodes and emit a single diagnostic listing all required interfaces.

### IS-7: Verify auto-mock compatibility (S)

**Files**: `gunbc-dag/src/mock_defaults.rs`, `core/exec/src/execute.rs`

Two checks:
1. Stub execute nodes carry `ObligationCategory::ServiceTransportExecute` — existing auto-mock keys on this obligation should work. Likely no changes needed.
2. **Structural transport detection**: stub execute node has an input port typed `TransportRequest`. This is what DryRun/testgen uses to identify transport boundaries (type-driven, not name-driven). Verify the port type is set correctly in IS-2.

### IS-8: Tests (M)

- **Lowerer test**: compile `InterfaceDef` + `func` using it, no profile → stub triplets with correct port shapes
- **Structural transport test**: compile without profile, assert the stub execute node has an input port of type `TransportRequest` (guarantees DryRun interception and testgen detection)
- **Parse reachability test**: execute the stub DAG in DryRun, verify parse node runs and forwards mocked outputs correctly
- **Resolver test**: stub ops resolve correctly via `InterfaceStub` spec branch; execute op errors in Real mode
- **Integration**: `make test-all` passes

### Dependency graph

```
IS-1 ──┬──> IS-2 ──> IS-4 ──> IS-7 ──> IS-8
       │              ^
IS-3 ──┘──> IS-5 ────/
IS-6 (parallel with IS-2..IS-5)
```

---

## Part 2: Per-Profile Live Tests (Follow-up)

### PT-1: Profile-aware compilation for testgen (M)

**Files**: `gunbc-dag/src/dsl_builder.rs`, `core/daglang/daglang-lower/src/lib.rs`

Reuse the compiler's existing profile parsing and binding logic — do NOT re-parse `dsl/profiles/*.dag` separately (drift risk). Instead:

- Add `profile: Option<&str>` to `CompileOptions` (or use the existing `--profile` path)
- Add `allow_placeholder_env: bool` to `CompileOptions` so testgen can compile with profiles referencing missing env vars
- Expose profile metadata (bound interfaces, env/secret requirements) from the compiler's own profile registry as a query API

Profile metadata extraction (env/secret requirements, bound interface names) comes from the compiler's parsed `ProfileDef`, not from separate AST walks.

### PT-2: Augment `CompilableModule` with interface imports (S)

**File**: `gunbc-dag/src/testgen_dag/dag_test_discovery.rs`

Add `interface_imports: HashSet<String>`. Populate during `collect_dag_files()` by checking `import interfaces.*` in the AST.

**Stopgap (2026-02-24)**: `requires_profile: bool` added to `CompilableModule` and filtered in `build_testgen_graph_auto()`. Prevents testgen hard-failure for interface-using modules. PT-2 replaces this with the richer `interface_imports: HashSet<String>` and IS-3 removes the need for filtering entirely.

### PT-3: Add `LiveProfileTestConfig` to `TestgenTargetDef` (S)

**File**: `core/codegen/src/registry.rs`

New struct with `profile_name`, `test_class`, `fermi_cost`, `required_env`, `required_any_of`, `dag_builder_call`. Add `live_profile_tests: Vec<LiveProfileTestConfig>` to `TestgenTargetDef`.

### PT-4: Scope live tests to applicable profiles (S)

Generate per-profile live tests only for profiles that **actually bind an interface imported by the module**. Cross-reference `CompilableModule.interface_imports` (PT-2) against profile bindings (PT-1).

Default canonical subset: `unit_test` and `local` profiles. Other profiles opt-in via configuration or annotation.

This bounds test explosion: ~10 interface-using modules × ~2 canonical profiles = ~20 live tests, not N×M.

### PT-5: Generate per-profile test sections in codegen (M)

**File**: `core/codegen/src/testgen/codegen.rs`

New `build_per_profile_live_flow_sections()`. For each `LiveProfileTestConfig`:
- Generate `test_live_flow_{module}_{profile}()`
- Gate with `guard_test_with_env()` using profile's env requirements
- Build DAG via compiler with `--profile` (reusing PT-1, not re-parsing)
- Execute with `ExecutionMode::Real`

### PT-6: Wire into auto-testgen pipeline (M)

**Files**: `gunbc-dag/src/testgen_dag/{graph.rs, ops.rs, dag_test_discovery.rs}`

- `build_testgen_graph_auto()`: query compiler profile registry once, then `profiles_for_module()` per module (using PT-2 interface imports + PT-4 scoping)
- `TestgenOp::AutoGenerate`: add `base_profile: Option<String>` and `live_profile_tests`
- `auto_testgen_for_module()`: use first available profile for base compilation; populate `live_profile_tests` for all applicable profiles

### Dependency graph

```
PT-1 ──> PT-5 ──> PT-6
PT-2 ──> PT-4 ──> PT-6
PT-3 ──> PT-5
```

---

## Verification

### Part 1
1. `make test-all` passes — testgen succeeds for `test_control_flow`, `sdlc_stages`, etc.
2. `cargo test --workspace` — all existing + new tests pass
3. `cargo clippy --all-targets -- -D warnings` — clean
4. Structural assertion: stub execute nodes have `TransportRequest` input port

### Part 2
1. Per-profile live test functions appear in generated test files
2. `cargo test test_live_flow_sdlc_worker_unit_test` runs (hermetic, no env vars)
3. Integration tests skip gracefully when env vars missing
4. Test count bounded: only profiles binding module-imported interfaces
