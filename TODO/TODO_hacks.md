# Hacks & Fallbacks

**Status**: Active
**Date**: 2026-02-07
**Last reconciled**: 2026-02-13 (external review cross-checked against codebase)
**DSL Alignment**: Debt ledger; prioritize items that block DSL migration tracks
**Track**: F — Debt Ledger

Sweep of explicit fallbacks and "best effort" behaviors in the codebase and
recent changes. These are not necessarily bugs, but they are places where
behavior can silently degrade or hide missing wiring.

### Reconciliation notes (2026-02-13)

External review flagged 14+ items. Cross-check found:
- **4 already fixed**: verify→ensure fallback, scalar witness list fallback,
  registry/makegen/CLI drift, secret redaction (now type-enforced via `SecretString`)
- **12 still outstanding**: see individual items below
- **4 claims in review were factually wrong**:
  - "Secret redaction is by convention" — `Value::Secret(SecretString)` is type-enforced,
    `print_value` handles it, inner value is private
  - "Flat 5-minute shell timeout" — `ShellRequest.timeout_ms` defaults to `None` and
    the executor doesn't implement timeout handling at all (see new item §15)
  - "Executor is single-threaded" — `execute_flat_parallel` is the default; sequential
    only when CI context is present
  - "`check-disallowed-methods.sh` is redundant with clippy" — historically false
    at the time (script was enforcing pragma placement). The script was later
    removed on 2026-02-14; current enforcement is clippy + pragma policy checks.

---

## ~~1. Boundary mock sequences fall back to static values~~ RESOLVED (2026-02-13)

**Where**: `core/exec/src/intercept.rs`, `core/test/src/mock_spec.rs`

**What changed**:
- Removed `SequenceExhaustion` enum and all fallback paths.
- `BoundaryMock::with_sequence()` now takes only a sequence (no default value);
  exhaustion always panics/errors.
- Removed `with_sequence_strict`, `set_sequence_strict`, `boundary_sequence_strict`
  — there is no lenient variant.
- `MockSpec::boundary_sequence()` takes 3 args (node, port, sequence) instead of 4.
- Executor error path uses `has_sequence()` instead of `is_strict()`.
- All existing tests updated; no escape hatches or migration flags.

---

## ~~2. Scalar witnesses fall back to list values for count > 1~~ RESOLVED (2026-02-13)

**Where**: `core/ir/src/contract.rs` (`witnesses`, fallback branch)

**What changed**: Removed the list fallback; scalar-with-count>1 now returns
`Err(WitnessError::InvalidCardinality)` instead of silently producing
`Value::List(...)`. The path is unreachable for well-formed DAGs (cardinality > 1
requires a wrapper kind), but fails loudly if it occurs.

---

## ~~3. Map<K,V> resolves to identity; key/value types not enforced~~ RESOLVED (2026-02-13)

**Where**: `core/ir/src/type_registry.rs`, `core/ir/src/type_lib.rs`,
`core/ir/src/contract.rs`, `core/ir/src/type_op.rs`

**What changed**:
- Added `WrapperKind::Map` to the type system (`type_op.rs`).
- Created `type_lib::map(value_type)` using the SubDag pattern (same as
  `list()`, `optional()`, etc.) — value type DAG is included as a SubDag
  node and validated per-element.
- `type_registry.rs` now resolves `Map<K,V>` through `type_lib::map(value_dag)`
  instead of `type_lib::identity()`. Both `TypeExpr::Map` and
  `TypeExpr::Wrapper(WrapperKind::Map, ...)` resolve correctly.
- `contract.rs` handles `WrapperKind::Map` uniformly: cardinality is `ONE`,
  witness generation covers count=0 (empty map), count=1 (single entry),
  and count>1 (multi-entry map with keyed witnesses).
- `TypeContract::from_type_dag()` recurses into Map SubDags to extract the
  inner base type.
- All 6 `WrapperKind` variants are now handled exhaustively in every match
  across the codebase (cardinality, witness generation, registry resolution,
  rendering).

**Result**: Map typing is now uniform with all other container types. Value
types are enforced through the SubDag validation chain.

---

## ~~4. Verify target falls back to ensure target~~ RESOLVED (2026-02-14)

**Where**: `gunbc-dag/src/makegen/registry.rs`

**What changed**:
- `ResourceTargetMap` now requires an explicit `verify_target` for every entry
  (no implicit `Option` fallback).
- Makefile meta-target rendering now fails fast if any `ResourceNeed` cannot
  be resolved to a concrete target (no silent dependency skips).

**Result**: verify dependency selection is explicit and deterministic; fallback
behavior is removed from the resource mapping path.

---

## ~~5. CI provider detection and unsupported commands degrade silently~~ RESOLVED (2026-02-14)

**Where**: `core/ir/src/transport/ci/provider.rs`,
`core/ir/src/transport/ci/providers/gitlab.rs`,
`core/ir/src/transport/ci/command.rs`

**What changed**:
- Added strict detection path (`detect_provider_strict`) that errors when a CI
  environment is detected but no supported provider marker is present.
- `CiContext::detect()` now uses strict detection.
- `CiContext::emit()` now enforces `provider.supports(cmd)` and panics on
  unsupported commands instead of emitting degraded fallback output.

**Result**: CI command emission is strict-by-default in executor paths, with
explicit failure on unknown providers or unsupported commands.

---

## 6. Mtime freshness fast path falls back to full hashing on any IO miss

**Where**: `core/ir/src/resource/managed.rs`

**What happens**: If any glob or mtime lookup fails, freshness checks fall back
to full hash comparison without surfacing the IO error reason.

**Why it's a hack**: Silent fallback hides IO issues and can produce unexpected
slowdowns without any diagnostic signal.

**Suggested fix**: Add a warning/diagnostic or a distinct freshness state that
captures the reason for the fallback. Consider making fallback behavior
configurable.

**Update (2026-02-18)**: Fallback diagnostics are now structured through
`ManifestFreshness` rather than raw `eprintln!`:
- `ManifestFreshness::FreshWithDiagnostic(note)` when full-hash verification is
  still fresh after mtime fallback.
- `ManifestFreshness::Stale(reason)` now includes the fallback reason context
  when hash verifies stale.
Callers (e.g., codegen freshness checks) can render the diagnostic in their
normal status/error output path.

---

## 7. ~~Dead code: `CredentialOp` + env-var providers (~500 lines)~~ DONE

Removed `lib/transport/src/credential.rs` (487 lines), `CredentialProvider` trait,
and all stale references. `Credential`, `AuthScheme`, `Secret`, `SecretSource`,
`CredentialError` remain (actively used).

---

## 8. ~~Duplicated `map_dag_ops` / `map_node_ops` (5 copies)~~ DONE

Replaced all 5 local copies with `Dag::map_ops()` (already in `core/ir`).
Consolidated 4 copies of `build_cloud_credential_graph_for_runtime` into
`lib/cloud-ops/src/graph.rs` (public, re-exported from `lib.rs`).

---

## 9. ~~`Clippy::upsert_and_run` has zero callers~~ DONE

Removed `upsert_and_run` and unused imports (`CliToolError`, `Value`,
`execute_cli_tool_op`, `HashMap`) from `lib/tools/clippy/src/ops.rs`.

---

## 10. DAG typing is structural at edge-level, but node I/O is still dynamic

**Where**:
- `core/ir/src/builder.rs` (strong edge/type/cardinality checks)
- `core/ir/src/types.rs` (`TypeId(pub String)`)
- `core/exec/src/lib.rs` (`Executable::execute(HashMap<String, Value>)`)
- `core/exec/src/execute.rs` (`execute_single_node` and input mock injection)
- `core/codegen/src/testgen/codegen.rs` (mock type checks skip `input_mocks`)

**What happens**:
- We get strong DAG build-time guarantees for edge wiring:
  - type compatibility
  - cardinality compatibility
  - fan-in rejection for scalar ports
  - cycle prevention
- But node execution boundaries are still `HashMap<String, Value>`, so wrong
  value types can still be injected at runtime (especially entrypoint ports).
- `MockSpec` type mismatch checks currently validate `transport_mocks` and
  `boundary_mocks`, but not `input_mocks`.

**Why it's a hack**:
- The model appears strongly typed end-to-end, but there is still a dynamic
  escape hatch at node call boundaries.
- This is where regressions like optional bool/string coercion drift or
  semantic placeholder values can bypass structural guarantees.

**Supporting examples (current repo)**:
1. `parse_impersonate` expects a REST payload with `accessToken`, but the type
   system only knows `TransportResponse`; shape-valid placeholders can still be
   behavior-invalid (`lib/gcp-ops/src/ops.rs`).
2. `compare_*_content.check_mode` wrong-type tests (`Value::Str("<WRONG>")`)
   are meaningful because node entrypoint inputs are runtime maps, not typed
   structs (`gunbc-dag/src/*/generated_tests.rs`).
3. `BlobOps::CompareContent` relies on strict input extraction
   (`optional_bool_strict`) to reject wrong-typed inputs at runtime
   (`lib/blob/src/lib.rs`).
4. CLI parsing is now fail-closed (unknown flags and invalid ints are hard
   errors), but node execution boundaries still accept dynamic `Value` maps
   (`core/exec/src/lib.rs`, `core/exec/src/execute.rs`).

**Suggested fix (incremental, DAG-first)**:
1. Add `input_mocks` type validation in testgen coverage checks (same level as
   existing boundary/transport mock compatibility checks).
2. Generate typed node input/output wrappers from DAG signatures (e.g.
   `ParseImpersonateIn`, `ParseImpersonateOut`) and use them in generated tests
   + helper APIs.
3. Add typed entrypoint injection APIs (`set_input_typed`) to avoid ad-hoc
   `Value` maps for common call paths.
4. Introduce refined semantic types for carrier payloads where needed:
   - `ImpersonationResponse` (validated schema)
   - `AccessToken`
   - `ScopeSet`
5. Keep transport/world effects as runtime checks; push everything else to
   DAG build-time or generated type wrappers.

**Update (2026-02-18)**:
- Testgen validation now checks `MockSpec::input_mocks` for:
  - unknown node/input-port references (`Unknown mock slots` panic path)
  - value/port type mismatches (`Mock value type mismatch` panic path)
- Added regression tests in `core/codegen/src/testgen/codegen.rs`:
  - `test_input_mock_type_mismatch_detected`
  - `test_input_mock_unknown_port_detected`

**Boundary of guarantee (explicit)**:
- Compile/build time can guarantee: DAG structure, port compatibility,
  cardinality, typed wrappers, and mock shape/type correctness.
- Runtime must still validate: external provider payloads, auth scopes,
  permissions, and freshness/availability of real resources.

---

## ~~11. Swapped TCP timeout fields — PROBABLE BUG~~ FIXED (2026-02-14)

**Where**: `lib/transport/src/executor.rs` (`execute_tcp`)

**What changed**:
- `write_timeout_ms` now maps to `set_write_timeout(...)`.
- `read_timeout_ms` maps to `set_read_timeout(...)`.
- Legacy `connect_timeout(...)` builder calls are kept as compatibility aliases
  to `write_timeout(...)` during migration.

**Result**: read/write timeout routing is explicit and no longer ambiguous.

---

## ~~12. `panic!()` in Result-returning `Executable` impl~~ RESOLVED (2026-02-14 audit)

`lib/transport/src/cli.rs` no longer panics for unexpected `CliToolOp`
variants in `execute_cli_tool_op`; these paths now return invariant
errors (`CliToolError::invariant(...)`) instead of crashing.

---

## ~~13. ~70 `.expect()` calls in production graph builders~~ RESOLVED (2026-02-14)

**Where**: `gunbc-dag/src/ci/graph.rs`, `gunbc-dag/src/workspace/subdags/bootstrap.rs`,
`gunbc-dag/src/workspace/subdags/deps.rs`, `core/exec/src/topo.rs`

**What changed**:
- `build_bootstrap_subdag` and deps subdag builders now return `Result<_, BuilderError>`
  and propagate builder failures with `?`.
- `build_workspace_dag` now returns `Result` and propagates subdag build errors.
- `ci::graph` codegen inlining no longer uses `.expect()` in production code paths;
  internal violations now return `BuilderError::InternalInvariant(...)`.
- `topo_sort` no longer unwraps malformed edge references.

**Result**: the cited production builder/execution paths now fail with
recoverable errors instead of panicking.

---

## 14. Fermi guard skips live tests instead of running them in CI

**Where**: `core/test/src/fermi.rs` — `guard()`, `guard_test_with_env()`

**What happened**: The fermi guard previously panicked in CI when secrets
were missing and `GUNBC_TEST_MAX_COST` wasn't explicitly set. This was
designed to catch CI misconfigurations, but it conflated two concerns:
cost limits (CI config) and secret availability (environment provisioning).
The panic was removed (2026-02-13) because it caused real CI failures for
tests like `test_live_flow_github_credential_lifecycle` that require GCP
WIF credentials not provisioned on the runner.

**What remains**: Live flow tests (`live_flow_tests` in testgen) are
intended to run in CI but currently can't because the runner lacks the
required secrets. These tests are silently skipped.

**Intended fix**: The CI workflow should derive secret requirements from
the repo's testgen metadata (the `live_required` / `live_required_any_of`
annotations on mock specs). This would allow the workflow to:
1. Scan all `testgen_target` annotations for `live_required` secrets
2. Provision exactly the secrets that live tests need (via GitHub Actions
   secrets + GCP Workload Identity Federation)
3. Set `GUNBC_TEST_MAX_COST` appropriately for the live test tier

This turns secret provisioning from a manual CI configuration step into
something derivable from the state of the repo — when a new live test is
added with `live_required("NEW_SECRET")`, CI should automatically know
it needs to provision `NEW_SECRET`.

**Blocked on**: GCP WIF setup for the GitHub Actions runner + a codegen
pass that extracts secret requirements from testgen metadata into the
CI workflow YAML.

---

## ~~15. Shell command timeout is defined but not implemented~~ RESOLVED (2026-02-13)

**Where**: `lib/transport/src/executor.rs` (`execute_shell`)

**What changed**:
- `execute_shell` now polls the child process with `try_wait()` when
  `timeout_ms` is set, kills the child and returns a `TransportError` if
  the deadline is exceeded.
- When no timeout is set, behavior is unchanged (`wait_with_output`).
- Added tests: `test_shell_timeout_kills_slow_command`,
  `test_shell_timeout_allows_fast_command`.

---

## ~~16. CI/local execution path split duplicates display/reporting logic~~ RESOLVED (2026-02-13)

**Where**: `core/exec/src/display.rs`, `core/exec/src/progress.rs`,
`core/exec/src/ci_context.rs`

**What changed**:
- Extended `ProgressObserver` trait with CI-aware hooks: `on_secret_output`,
  `on_failure_diagnostics`, `on_boundary_output`, `requires_sequential`.
- Implemented `ProgressObserver` on `CiContext` — maps observer hooks to
  CI workflow commands (groups, error annotations, secret masking).
- Added `ComposedObserver` adapter to fan out to two observers.
- `display.rs` now uses a single `run_plain` path that composes
  `NonTtyProgressObserver` with `CiContext` via `ComposedObserver` when
  in CI, instead of branching into separate CI/local display functions.

**Result**: One execution path for both CI and local. CI grouping and
progress rendering are observer callbacks, not separate codepaths.

---

## ~~17. Preflight bypasses CI grouping and structured error reporting~~ RESOLVED (2026-02-13)

**Where**: `lib/transport/src/preflight.rs`, `gunbc-dag/src/bin/ci.rs`

**What changed**:
- Added `ensure_lint_upsert_with_ci(ci: Option<&mut CiContext>)` which
  wraps the entire preflight in a CI group and emits `::error::` annotations
  on failure.
- `run_lint_upsert` now accepts an optional `CiContext` and wraps each step
  (codegen-dag, testgen, pragma, clippy, test) in collapsed CI sub-groups.
- `ci.rs` passes a `CiContext::detect()` to preflight for structured output.
- `ensure_lint_upsert()` delegates to `ensure_lint_upsert_with_ci(None)` for
  backward compatibility (non-CI binaries).

**Result**: Preflight failures in CI produce GitHub annotations and are
wrapped in collapsible CI groups. Each step is a separate sub-group.

---

## 18. Report node receives raw unstructured strings

**Where**: `gunbc-dag/src/ci/ops.rs` (lines 920-1099),
`core/ir/src/render_ir.rs` (`StructuredBlock::Raw`)

**What happens**: `execute_report` formats CI report sections as
`StructuredBlock::Raw(format!(...))`. Stderr from build/test/clippy is stuffed
into raw strings and truncated at 60 lines / 500 chars as a band-aid.

**Why it's a hack**: Truncation improves readability but the underlying data
is unstructured text. Tooling can't programmatically find the "real" error.

**Suggested fix**: Add stage-specific extractors (build errors, clippy warnings,
test failures) and a unified error field convention so tooling can locate the
actual error automatically.

**Added**: 2026-02-13 (reconciliation)

---

## ~~19. Unknown CLI flags are silently ignored~~ RESOLVED (2026-02-13)

**Where**: `core/cli/src/lib.rs`, all `gunbc-dag/src/bin/*.rs`

**What changed**:
- Added `ParseError::UnknownFlag` variant to the schema-driven parser.
  Any arg starting with `-` that doesn't match a known flag now returns
  `ParseError::UnknownFlag { flag }`. Generated CLIs inherit this via
  `gunbc_cli::parse()`.
- All 8 manual binary parsers (codegen, makegen, testgen, bootstrap,
  pragma, build, docgen, ci) now error on unknown flags instead of
  silently ignoring them.
- `ci.rs` uses a declarative style (`args.iter().any()`), so unknown-flag
  checking was added as an explicit post-parse validation loop.
- Entrypoint port-name matching loops (`match port_name.0.as_str()`)
  in makegen/testgen/bootstrap/pragma are NOT flag parsing and remain
  unchanged — they correctly skip unknown entrypoint ports.

**Result**: Unknown flags are now a hard error everywhere. No escape hatches.

---

## ~~20. Unknown --mode values warn and proceed with default~~ RESOLVED (2026-02-13)

**Where**: all `gunbc-dag/src/bin/*.rs` (codegen, makegen, testgen, bootstrap,
pragma, ci)

**What changed**:
- Added `ExecMode::parse_strict()` which returns `Result<Self, String>` with
  a descriptive error on unknown mode values.
- All 6 binaries now use `parse_strict` and `process::exit(1)` on unknown mode,
  instead of warning and continuing.

**Result**: Unknown `--mode` values are now a hard error everywhere.

---

## 21. Transport executor test coverage — partial

**Where**: `lib/transport/src/executor.rs`

**What changed (2026-02-13)**: Added unit tests for pure helper functions
(`url_encode`, `append_query`, `is_unreserved_url_byte`) — 9 tests covering
RFC 3986 unreserved preservation, percent-encoding, query string `?`/`&` logic,
empty inputs, and Unicode multi-byte encoding.

**What remains**: `execute_rest`, `execute_http`, `execute_tcp` still have no
direct tests. These require network I/O and are better addressed with the
transport DAG migration (`TODO/TODO_transport_dag_migration.md`).

**Added**: 2026-02-13 (reconciliation)

---

## 22. Coercion coverage tests don't verify actual coercion

**Where**: `core/codegen/src/testgen/codegen.rs` (`build_coercion_coverage_tests`),
`core/exec/src/execute.rs` (fan-in logic)

**What happens**: Generated coercion tests build a DAG, execute in DryRun, and
assert it didn't crash. They don't verify that the coercion actually happened
(e.g., scalar wrapped into list).

**Why it's a hack**: Smoke tests won't catch shape bugs like nested-list vs
flat-list coercion errors.

**Suggested fix**: Either add `inputs` to `LogEntry` so tests can assert on
what target nodes received, or inject shape-assert nodes into test graphs.

**Blocked on**: Design decision — adding inputs to LogEntry doubles log memory;
shape-assert nodes require more complex testgen logic.

**Added**: 2026-02-13 (reconciliation)

---

## 23. DryRun defaults mask missing resource wiring

**Where**: `lib/tools/deps/src/graph_mock.rs`, `lib/tools/deps/src/env.rs`

**What happens**: DryRun mocks use deterministic defaults (`Platform::Linux`,
`Timestamp(0)`, empty `EnvVars`) so a DAG can appear to work even if it never
properly acquired/wired those resources.

**Why it's a hack**: Missing resource wiring is silently papered over by
defaults instead of failing loudly.

**Suggested fix**: Consider a "strict DryRun" mode where env-node outputs
default to poison/UNSET unless explicitly mocked.

**Added**: 2026-02-13 (reconciliation)

---

## ~~24. Multiple `cargo run` invocations in Makefile + duplicated binary CLI parsing~~ RESOLVED (2026-02-14)

**Where**: `Makefile` (lines 22, 30, 42, 46, 50, 54-64),
`gunbc-dag/src/bin/*.rs` (all 8 binaries)

**CLI parsing — RESOLVED (2026-02-14)**: Extracted `BinaryArgs` builder in
`core/cli/src/binary_args.rs` and routed binary parsing through shared
`gunbc_cli::parse()`. Deprecated `--check`/`-c` handling was removed; binaries
now accept only canonical flags (`--mode` and canonicalized `--<kebab(name)>`
for string params). Deleted `parse_resource_mode()` from `ci.rs`.

**Makefile overhead — RESOLVED (2026-02-14)**:
- All cargo Makefile tool targets (including manual binaries `pragma`, `ci`,
  and `build-all`) now execute through one shared infra:
  `build-release-bins` + direct `target/release/<binary>`.
- `lint-upsert` no longer depends on the `pragma` Make target; it runs the
  pragma command directly, so maintenance flows remain intact without
  reintroducing cargo-run paths.
- Added a dedicated `deps-config` / `deps-config-check` path with
  `gunbc-deps-config`, eliminating the old `deps` fallback mapping for
  `build:deps_config`.

**Residual analysis topic (non-blocking)**:
- Buck2 mode still renders cargo-based invocations for repo generator binaries;
  a future cross-build-system unification pass may further reduce divergence.

**Added**: 2026-02-13 (reconciliation)
**Updated**: 2026-02-14 (all Makefile cargo tool targets unified to
build-release-bins + direct release binaries; deps-config verify path added;
step-mode generated CLIs now dispatch via shared `gunbc_cli::parse_step_mode`)

---

## ~~25. Probe-observer lowering/analysis is computed in multiple places~~ RESOLVED (2026-02-14)

**Where**: `core/codegen/src/testgen/codegen.rs`

**What changed**:
- Added a single `ProbeObserverBundle` model (`analysis`, `report`,
  `lowering_error`) computed once per test module generation.
- Header coverage reporting now reads the precomputed bundle report.
- Probe-observer section generation now reads the same bundle (including
  lowering-failure diagnostic path) instead of recomputing lowering/analysis.

**Result**: header + section stay aligned by construction and no longer
maintain separate overlapping probe-observer analysis paths.

---

## 26. Seed policy classification is still testgen-local string matching

**Where**: `core/codegen/src/testgen/codegen.rs`

**Status (2026-02-14)**: RESOLVED. Seed placeholder policy now lives in IR
(`core/ir/src/types.rs`: `SeedPlaceholderPolicy`,
`seed_placeholder_policy_for_type_id`) and testgen queries that API.

**Added**: 2026-02-14 (reconciliation)
**Resolved**: 2026-02-14

---

## 27. CI secret requirements are not modeled from one source of truth

**Where**: workflow env wiring + testgen metadata (`live_required*`)

**Status (2026-02-14)**: RESOLVED. CI secret env wiring now derives from
`DagSpec` metadata (`live_required` + `live_required_any_of`) via
`ci_live_test_secrets()` in `gunbc-dag/src/ci/graph.rs`, with GitHub
auto-provided env vars filtered out.

**Added**: 2026-02-14 (reconciliation)
**Resolved**: 2026-02-14

---

## ~~28. Execution logs still omit node inputs~~ RESOLVED (2026-02-14)

**Where**: `core/exec/src/execute.rs` (`LogEntry`)

**What changed**:
- `LogEntry` captures optional inputs (`inputs: Option<HashMap<String, Value>>`).
- Execution defaults now use `LogDetailLevel::IncludeInputs`.
- Composition-level overrides are modeled in IR (`root < subdag < node < input-port`).

**Added**: 2026-02-14 (reconciliation)
**Resolved**: 2026-02-14

---

## ~~29. Log detail policy does not yet support composition-level inheritance/override~~ RESOLVED (2026-02-14)

**Where**:
- IR metadata (`core/ir/src/node.rs`, `core/ir/src/dag.rs`, `core/ir/src/log_detail.rs`)
- Lowering inheritance (`core/exec/src/lower.rs`)
- Runtime capture resolution (`core/exec/src/execute.rs`)

**What changed**:
- `LogDetailLevel` is now defined centrally in IR.
- `Node` and `Port` support optional `log_detail` overrides.
- Lowering now propagates subdag composition defaults to lowered descendants.
- Runtime capture resolves effective policy through the hierarchy:
  `root execution setting < subdag composition < node < input-port`.

**Result**: log capture policy is now a modeled compositional property instead
of a single global runtime toggle.

**Added**: 2026-02-14
**Resolved**: 2026-02-14

---

## 30. "List" used as type_id AND cardinality shape (dual encoding) — PARTIAL FIXES

**Where**: CLI arg parsing, mock generation, loop patterns, makegen repeatable detection

**What happens**: The design doc says cardinality is the canonical shape layer, but "List" is
deeply embedded as a type_id across 28 files.

**Progress so far**:
- CLI generation now derives list-ness from cardinality (no type_id == "List").
- makegen registry repeatable flags derive from cardinality.
- loop pattern defaults use element types + cardinality.
- deps graphs now use `"String"` element types for dep/tool name lists.
- bootstrap graphs now use list("crate_names", "String") instead of type_id "List".
- language subdags/patterns now use list(..., "String") for list ports.
- registry entrypoints now use type_id "String" + cardinality for extensions.
- CliEntrypoint::new no longer infers cardinality from "List"/"Set".

**Remaining work**: finish removing type_id-encoded cardinality
(e.g., StringList/OptionalString) once the type registry refactor lands.
Mock generation now hard-fails on unknown type_ids and on `List`/`Set`.

**Files**:
- core/ir/src/types.rs
- core/codegen/src/testgen/codegen.rs

**Added**: 2026-02-14 (consolidated from root TODO_hacks)

---

## ~~31. Cardinality test-case cap is a bandaid~~ RESOLVED (2026-02-18)

**Where**: `core/ir/src/types.rs`, `core/ir/src/contract.rs`

**What changed**:
- Added explicit sampling policy model:
  `CardinalitySamplingStrategy` (`BoundaryOnly`, `BoundaryWithUpperBound`).
- `Cardinality::test_cases_for_tests()` now uses **boundary-only** sampling
  by default (no hardcoded 64-cap mutation).
- Retained optional bounded mode via
  `Cardinality::test_cases_with_strategy(CardinalitySamplingStrategy::BoundaryWithUpperBound(_))`
  for callers that explicitly want clamped stress sampling.

**Files**:
- core/ir/src/types.rs
- core/ir/src/contract.rs

**Added**: 2026-02-14 (consolidated from root TODO_hacks)

---

## 32. `Map` type_id is under-specified for proofs/testgen

**Where**: `core/ir/src/value.rs`, `core/ir/src/types.rs`, `core/codegen/src/testgen/codegen.rs`

**What happens**: Value::Map(BTreeMap<String, Value>) lost type parameter info when
MapStrStr was replaced with generic Map. Codegen can't serialize
Value::Map in general, and there's no way to express "map of string
to string" vs "map of string to json" at the port type level.

**Note**: Item 3 resolved Map *resolution* in the type registry. This item is about
Map *proof generation* — testgen still can't produce typed map witnesses.

**Suggested fix**: Either parametric type IDs (Map<String,String>) or a type DAG /
type expression structure instead of flat String type_id.

**Added**: 2026-02-14 (consolidated from root TODO_hacks)

---

## 33. Cardinality constants are flat — no compositional modeling

**Where**: `core/ir/src/types.rs`

**What happens**: Named cardinality constants (ZERO, ONE, ZERO_OR_ONE, etc.) are syntactic
sugar for interval structs. If "everything is a DAG" then cardinality
constraints could be modeled as composable DAG nodes.

**Why it's a hack**: Cardinality is compile-time only (used for test generation and port
validation), not a first-class runtime concept. Making it compositional would
enable runtime-evaluable multiplicity constraints.

**Added**: 2026-02-14 (consolidated from root TODO_hacks)

---

## 34. Resource capabilities potentially forgeable via TryFrom<Value>

**Where**: Resource trait proposal in `TODO/TODONE/design-resource-acquisition.md`

**What happens**: The resource acquisition design proposes:
```rust
pub trait Resource: Into<Value> + TryFrom<Value>
```

If `TryFrom<Value>` accepts any token-like value, capabilities become forgeable.
A malicious/buggy node could construct a fake handle from data.

**Suggested fix**: Runtime guard ensuring only env nodes can mint capability handles
(e.g., internal IDs stored in executor-side handle table, not in Value itself).

Also: `EnvVars` as observation resource risks spilling secrets unless it's a
filtered projection (the `CredentialOp` approach is safer).

**Update (2026-02-18)**: Capability parse paths enforce secret marker validation
via `ensure_capability_marker(...)`, and regression tests now assert that forged
values without the marker are rejected:
- `FilesystemHandle::try_from(Value)` rejects missing `cap` marker
- `NetworkHandle::try_from(Value)` rejects missing `cap` marker

**Added**: 2026-02-14 (consolidated from root TODO_hacks)

---

## ~~35. Compound shell commands should be replaced with native Rust~~ RESOLVED (2026-02-18)

**Where**: `lib/tools/gist/src/graph.rs`

**What changed**:
- Removed legacy batch-read helpers (`execute_prepare_read_files`,
  `execute_parse_read_files`) and their enum variants (`PrepareReadFiles`,
  `ParseReadFiles`) from `GistGraphOp`.
- Snapshot-mode file acquisition now has a single implementation path:
  per-file loop execution (`PrepareReadFile -> Transport(file read) -> ParseReadFile`)
  plus `CollectFileContents`.
- Updated graph docs/comments to reflect the loop-native transport pattern.

**Added**: 2026-02-14 (consolidated from root TODO_hacks)
