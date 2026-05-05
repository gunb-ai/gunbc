# P0-C — Remove `__BUG_NO_PROFILE_…` fabrication sentinel `(S-M)`

> **RETIRED** (2026-05-05). Dissolution landed prior to this brief's framing being audited. `container_param_name_required` no longer exists in `dsl/std/types.dag`; only `container_param_name` returning `String?` remains at `dsl/std/types.dag:103-109`. All callers `match` the Option. The `__BUG_NO_PROFILE_` sentinel literal is absent from the entire `src/` and `dsl/` tree. Ratchet test exists at `src/v2/tests/src/bug_sentinel_ratchet.rs` (passes). Brief content below preserved for historical context.

## Context

Exploratory analysis found a fail-open fabrication pattern in `dsl/std/types.dag:115-119`:

```
fn container_param_name_required(kind_name: String, index: Int) -> String {
  match container_param_name(kind_name: kind_name, index: index) {
    Some { value: n } => n
    None => concat("__BUG_NO_PROFILE_", kind_name)
  }
}
```

When the lookup misses, the function **fabricates a valid-looking `String`** by concatenating a magic prefix. Downstream consumers see a string that looks like a name but encodes a failed lookup. This violates M5 ("silence is fabrication") and C-8 (fail-closed discipline — every detectable problem is a typed Diagnostic).

The sentinel is **duplicated** in test scaffolding at `src/v2/tests/src/compiler-tests.rs:2350-2354`:

```
let key_name =
    container_param_name("Map".to_string(), 0).unwrap_or("__BUG_NO_PROFILE_Map".to_string());
let val_name =
    container_param_name("Map".to_string(), 1).unwrap_or("__BUG_NO_PROFILE_Map".to_string());
```

Same false authority in two places: production `.dag` and test scaffolding.

Also per `INVARIANTS.md#c-8`: no fabrication sentinels (`__BUG_*`, `__EMIT_BUG_*`). Missing facts are compile-time errors, not runtime strings.

## Read first

- `dsl/std/types.dag:65-130` — full container-kind / container_param_name region
- `dsl/std/types.dag:115-119` — the sentinel site
- `src/v2/tests/src/compiler-tests.rs:2320-2400` — the test-side duplicate
- Any other call site of `container_param_name_required` (`grep -rn "container_param_name_required"`)
- `INVARIANTS.md#c-8` (fail-closed) and `INVARIANTS.md#p3-fail-closed` (no fabricated plausible output)

## Work

1. **Remove the sentinel from `dsl/std/types.dag`**:
   - `container_param_name_required` returns `Option<String>` (or `Result<String, Diagnostic>` if the typing prefers). No more concat-fabrication.
   - The caller of `container_param_name_required` must handle the miss case — either emit a typed diagnostic or propagate the `None` forward.
   - If every caller was relying on the fabricated string "working anyway," that's the bug — fix each caller to handle absence explicitly.
2. **Remove the test-side duplicate**:
   - `compiler-tests.rs:2350-2354` uses `unwrap_or("__BUG_NO_PROFILE_Map")`. If the test is testing that `container_param_name("Map", 0)` returns a non-None value, then `unwrap_or` is the wrong primitive — use `expect("...")` with a real error message, or `unwrap()` if the test is asserting presence.
   - Grep the whole repo for `__BUG_NO_PROFILE_` and `__BUG_` — list every occurrence, confirm none survives post-cleanup.
3. **Add a ratchet** that greps source for the sentinel prefix and fails if any survives. Prevents reintroduction. Small, one-line test or a CI check.
4. **Grep for related sentinel classes** — per `INVARIANTS.md#p3-fail-closed`: `__BUG_*`, `__EMIT_BUG_*`, anything string-namespaced with an underscore-underscore prefix. List + triage. If any other sentinels are found, flag as a followup lane; this PR focuses on `__BUG_NO_PROFILE_`.

## Acceptance

- Zero occurrences of `__BUG_NO_PROFILE_` in `dsl/std/types.dag` and `src/v2/tests/src/compiler-tests.rs`.
- `container_param_name_required` returns typed absence (`Option` / `Result`), not a fabricated string.
- Callers handle absence with a real diagnostic or propagation — not another fallback string.
- Ratchet test prevents reintroduction.
- No new fabrication sentinels added anywhere in the diff.

## STOP-AND-ESCALATE

- If removing the sentinel reveals that multiple callers were relying on the fabricated string format (e.g., downstream pattern-matching on `"__BUG_..."` as a signal) — STOP, list them, propose whether those are separate real bugs or the caller needs the same absence-handling treatment.
- If `container_param_name_required` is called from hot paths that can't easily return `Option` (e.g., inside a non-Option-returning emitter walk), surface — may need a broader refactor of the caller's signature before removal is safe.
- If additional `__BUG_*` / `__EMIT_BUG_*` sentinels are found in the broader grep, list them separately — this PR doesn't own their removal, but they become named followups.

## Non-goals

- No full audit of every fallback string in std (just `__BUG_NO_PROFILE_`).
- No changes to the `container_param_name` lookup logic itself — only its caller's miss-handling.
- No refactor of `container_kind` or `container_arity` tables.

## Size: S-M (function signature change + caller updates + ratchet).
