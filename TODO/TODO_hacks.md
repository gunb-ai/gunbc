# Hacks & Fallbacks (2026-02-08 scan)

**Status**: Active
**Date**: 2026-02-07

Sweep of explicit fallbacks and "best effort" behaviors in the codebase and
recent changes. These are not necessarily bugs, but they are places where
behavior can silently degrade or hide missing wiring.

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

## 4. Verify target falls back to ensure target

**Where**: `gunbc-dag/src/makegen/registry.rs`

**What happens**: `ExecMode::Verify` uses `verify_target` when present, else
falls back to `ensure_target`.

**Why it's a hack**: Verify mode can mutate state, contradicting the expected
"check-only" semantics. Callers may not realize the fallback is happening.

**Suggested fix**: Require explicit verify targets for resources referenced in
verify workflows or make the fallback opt-in via a flag.

---

## 5. CI provider detection and unsupported commands degrade silently

**Where**: `core/ir/src/transport/ci/provider.rs`,
`core/ir/src/transport/ci/providers/gitlab.rs`,
`core/ir/src/transport/ci/command.rs`

**What happens**: Provider detection falls back to plain text when no CI
environment is detected. GitLab renders unsupported commands as colored text
instead of native annotations/outputs.

**Why it's a hack**: CI features (annotations, outputs, summaries) can silently
degrade with no signal, hiding configuration mistakes or unsupported providers.

**Suggested fix**: Add a "strict CI" mode or explicit provider selection that
errors on unknown providers or unsupported commands. Expose `supports()` checks
in render paths.

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
