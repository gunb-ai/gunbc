# PB-1-e parse-surface-free bootstrap regen — P5 receipt (hand-Rust)

**Status:** PR-scoped planning artifact — satisfies `INVARIANTS.md` §P5
*Dispatch-Discipline Mechanisms* **(b)** for intentional hand-Rust under
`src/v3/compiler/src/bootstrap_regen_fresh.rs` (parse-surface-free bootstrap
path: `WITHOUT_PARSE_SURFACE_EXCLUDED_FIXTURE_PATHS`, `spec_iter` exclusion in
`load_runtime_bootstrap_authorities`).

## Exactly one disposition (mechanism (b), option 3)

**Explicit deferral** — lane **T-PB-A** (non-test hand-Rust → 0-floor per pure
bootstrap program).

**Cited ROADMAP row (one hop):** `ROADMAP.md` in this repo, **`### Goals (the six non-negotiables)`** — numbered goal **6. Self-hosting (Pure Bootstrap).** (references
`docs/design-pure-bootstrap-zero.md`), **and** the **`### Nine lanes`** table row
**T-PB-A** (`pb_hand_rust_at_shim_floor`, SG-0 census, `docs/design-pure-bootstrap-zero.md`).

## Why this Rust exists (interim, not steady state)

PB-1-e ships two committed bootstrap snapshots: full (`bootstrap_generated.rs`)
and parse-surface-free (`bootstrap_generated_without_parse_surface.rs`). The
latter must not load `parse_surface.dag` or the Rust/Go/Python spec files that
depend on it, or resolve diagnostics pollute the embedded graph (see comments
on `WITHOUT_PARSE_SURFACE_EXCLUDED_FIXTURE_PATHS` in
`bootstrap_regen_fresh.rs`).

## Named dissolution trigger

**Primary:** `bootstrap.rs` and this module’s module-level note — **PB-Bootstrap-Process**
replaces the regen host with a declared `bootstrap.dag` / generated producer
path; this hand-Rust regen module **deletes** in favor of that single authority.

**Secondary (substrate convergence):** if `parse_surface` types and spec
imports converge such that the excluded specs load cleanly without
`parse_surface.dag`, the exclusion list and `spec_iter` filter **shrink** or
disappear in the same PR that proves diagnostic-clean `compile_full_bootstrap_without_parse_surface_dag_from_std_seed` without those exclusions (checkable:
`regen_bootstrap --verify` + empty bootstrap diagnostics on the no-parse-surface
snapshot).

## Checkable ratchet

Until dissolution: any change to `BOOTSTRAP_FIXTURE_PATH_KEYS` or the
parse-surface-free fixture set must keep
`compile_full_bootstrap_without_parse_surface_dag_from_std_seed` diagnostic-clean;
`regen_bootstrap --verify` remains the acid test (`PB-1-e` mechanism (ii)).
