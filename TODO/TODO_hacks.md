# Hacks & Fallbacks

**Status**: Active
**Date**: 2026-02-07
**Last reconciled**: 2026-02-13 (external review cross-checked against codebase)

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
  - "`check-disallowed-methods.sh` is redundant with clippy" — script enforces *where*
    `#[allow(clippy::disallowed_methods)]` can appear, complementary to clippy

---

## 1. Boundary mock sequences fall back to static values

**Where**: `core/exec/src/intercept.rs`, `core/test/src/mock_spec.rs`

**What happens**: `BoundaryMock::next_value()` consumes a sequence and then
falls back to the static `value`. `MockSpec::boundary_sequence()` mirrors this
behavior by providing a default fallback value.

**Why it's a hack**: Extra calls beyond the planned sequence are silently
accepted, so tests can pass even if call counts change or loops are added.

**Suggested fix**: Add a strict mode (or per-mock flag) that errors once the
sequence is exhausted unless a "repeat last" or explicit fallback mode is set.

**Update (2026-02-07)**: Added strict sequence support; strict mocks now error
on exhaustion during execution. Default behavior still falls back.

---

## 2. Scalar witnesses fall back to list values for count > 1

**Where**: `core/ir/src/contract.rs` (`witnesses`, fallback branch)

**What happens**: When a scalar type has a boundary count > 1 and no list/set
wrapper is present, witnesses are emitted as `Value::List(...)`.

**Why it's a hack**: This mixes scalar and list semantics and hides inconsistent
cardinality/type modeling. It can mask a missing wrapper or incorrect type.

**Suggested fix**: Treat scalar-with-count>1 as an error unless a list/set
wrapper is present. Alternatively, inject a wrapper type explicitly before
generating witnesses.

**Update (2026-02-07)**: Removed the list fallback and now treat this case as
an error (panic) during witness generation. This should be unreachable for
wrapper-derived cardinalities but fails loudly if it occurs.

---

## 3. Map<K,V> resolves to identity; key/value types not enforced

**Where**: `core/ir/src/type_registry.rs`, `core/ir/src/value.rs`

**What happens**: `Map<K,V>` parses and resolves to an identity DAG when not
explicitly registered. Key/value types are only syntax-checked, and runtime
`Value::Map` uses `BTreeMap<String, Value>` with no enforcement of `K`/`V`.

**Why it's a hack**: The type expression conveys more structure than the
runtime enforces. Invalid or non-string keys can slip through silently.

**Suggested fix**: Enforce `K == String`, add a map type DAG that validates
value types, and reject/diagnose unknown `Map<...>` types. (Related to the
Map under-specification note in `TODO_hacks`.)

**Update (2026-02-07)**: Type expression validation now rejects non-`String`
map keys. Value typing is still not enforced.

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

**Update (2026-02-13)**: IO error reasons are now captured and emitted via
`eprintln!` at the fallback site (`glob failed for ...`, `mtime failed for ...`).
The `MtimeResult::MaybeStale` reason is also surfaced before falling through to
full hash. Fallback is no longer silent.

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
4. CLI parsing still has permissive coercion (`Int` uses parse-or-0) which is
   runtime behavior, not structural typing (`core/cli/src/lib.rs`).

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

**Boundary of guarantee (explicit)**:
- Compile/build time can guarantee: DAG structure, port compatibility,
  cardinality, typed wrappers, and mock shape/type correctness.
- Runtime must still validate: external provider payloads, auth scopes,
  permissions, and freshness/availability of real resources.

---

## ~~11. Swapped TCP timeout fields — PROBABLE BUG~~ FIXED (2026-02-14)

**Where**: `lib/transport/src/executor.rs` (`execute_tcp`)

**What changed**:
- `connect_timeout_ms` now controls `TcpStream::connect_timeout(...)`.
- `read_timeout_ms` now maps to both `set_read_timeout(...)` and
  `set_write_timeout(...)` for socket I/O timing.

**Result**: connect timeout and I/O timeout semantics are no longer swapped.

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

## 15. Shell command timeout is defined but not implemented

**Where**: `core/ir/src/transport/mod.rs` (`ShellRequest.timeout_ms`),
`lib/transport/src/executor.rs` (`execute_shell`)

**What happens**: `ShellRequest` has a `timeout_ms: Option<u64>` field and a
`.timeout(ms)` builder method, but `execute_shell` calls
`child.wait_with_output()` with no timeout handling. The field is dead wiring.

**Why it's a hack**: Callers can set a timeout and believe it's enforced, but
it silently does nothing.

**Suggested fix**: Implement timeout via `wait_timeout` or a background
watchdog thread that kills the child process, or remove the field until
implemented.

**Added**: 2026-02-13 (reconciliation)

---

## 16. CI/local execution path split duplicates display/reporting logic

**Where**: `core/exec/src/display.rs` (lines 121-173), `gunbc-dag/src/bin/ci.rs`

**What happens**: `execute_and_display` branches into completely different
functions for CI (`run_with_ci`) vs local (`run_with_progress` /
`run_non_tty_summary` / `run_classic`). Improvements to one path don't
propagate to the other.

**Why it's a hack**: Display/reporting logic drift between CI and local makes
failures harder to reproduce and debug across environments.

**Suggested fix**: One execution path, parameterized by a `DisplayConfig` /
`CiContext`. CI grouping and progress rendering should be observer callbacks,
not separate codepaths.

**Added**: 2026-02-13 (reconciliation)

---

## 17. Preflight bypasses CI grouping and structured error reporting

**Where**: `lib/transport/src/preflight.rs` (lines 52, 306-365)

**What happens**: Preflight uses raw `println!`/`eprintln!` throughout. It runs
before the DAG and has no access to `CiContext`, so its output appears as
unstructured noise in CI logs with no `::error::` annotations.

**Why it's a hack**: Preflight failures in CI produce no GitHub annotations and
aren't wrapped in CI groups, making them hard to find in logs.

**Suggested fix**: Either make preflight a DAG node, or at minimum accept a
`CiContext` and emit CI groups + structured error commands.

**Added**: 2026-02-13 (reconciliation)

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

## 19. Unknown CLI flags are silently ignored

**Where**: `core/cli/src/lib.rs` (line 164, 216), all `gunbc-dag/src/bin/*.rs`

**What happens**: The schema-driven parser and all manual CLI parsers use a
`_ => {}` catch-all for unrecognized flags. Typos like `--extensinos` become
silent no-ops.

**Why it's a hack**: Silent flag ignoring is the #1 cause of "why isn't my
flag working?" confusion, especially in CI where output is hard to inspect.

**Suggested fix**: Add a strict mode (especially for CI) that errors on unknown
flags, keeping permissive behavior as a compatibility option.

**Added**: 2026-02-13 (reconciliation)

---

## 20. Unknown --mode values warn and proceed with default

**Where**: all `gunbc-dag/src/bin/*.rs` (codegen, makegen, testgen, bootstrap,
pragma, ci)

**What happens**: If `ExecMode::parse()` returns `None` for an unrecognized
mode string, every binary prints a warning and continues with the current or
hardcoded default mode.

**Why it's a hack**: In CI, warnings are easily missed. A typo in `--mode`
can cause a verify workflow to silently run in ensure mode.

**Suggested fix**: In CI context, treat unknown mode as a hard error.

**Added**: 2026-02-13 (reconciliation)

---

## 21. Transport executor has zero test coverage for tcp/rest/http

**Where**: `lib/transport/src/executor.rs` (`execute_rest`, `execute_http`,
`execute_tcp`)

**What happens**: Testgen generates tests for Prepare/Parse nodes and
DryRun-intercepts Execute nodes, so the actual transport I/O boundary is never
tested. TCP has a confirmed blind spot (a swapped-timeout bug survived because
of this).

**Why it's a hack**: The most error-prone code (network I/O, timeout handling,
error mapping) has zero direct test coverage.

**Suggested fix**: Add unit tests per transport (at minimum: request building,
response parsing, error mapping, timeout behavior). See
`TODO/TODO_transport_dag_migration.md` for the scoped plan.

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

## 24. Multiple `cargo run --release` invocations in Makefile

**Where**: `Makefile` (lines 22, 30, 42, 46, 50, 54-64)

**What happens**: The generated Makefile invokes `cargo run -p gunbc-dag --bin X
--release` 6+ times for codegen, testgen, pragma, makegen, bootstrap. Each
invocation incurs full cargo overhead (dependency resolution, compilation
checks, binary loading).

**Why it's a hack**: Repeated cargo invocations are a CI-time multiplier. The
binaries are already compiled; running them via `cargo run` re-checks
dependencies every time.

**Suggested fix**: Build once (`cargo build --release`), then invoke binaries
directly from `target/release/`. Or use a single orchestrator binary with
subcommands.

**Added**: 2026-02-13 (reconciliation)
