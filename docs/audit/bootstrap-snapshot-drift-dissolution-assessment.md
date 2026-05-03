# Bootstrap Snapshot Drift Dissolution Assessment

## Context

PR #1543 edited bootstrap-loaded substrate authority (`src/v3/std/induction.dag`)
without the paired `bootstrap_generated.rs` /
`bootstrap_generated_without_parse_surface.rs` regeneration. The existing `v3`
job caught the stale snapshots through `regen_bootstrap --verify`, but the PR
still merged because the required `ci` job stayed green. PR #1557 repaired the
snapshots after the fact.

The deeper issue is that the repository has committed generated Rust snapshots
under `src/v3/compiler/src/`. A gate catches drift; it does not dissolve that
drift surface. This assessment records why the gate is an interim fallback for
this slice rather than claiming the generated-on-disk model is terminal.

## Path 1: Regenerate On Every Build

Target shape: make the current `bootstrap-regen-fresh` behavior the default
build path, so `build.rs` regenerates the bootstrap snapshots into `OUT_DIR` and
the checked-in Rust snapshots disappear.

This is the cleanest dissolution direction, but it is not a safe wiring-only
change:

- `src/v3/compiler/src/dag.rs` currently `include!`s
  `bootstrap_std_generated.rs`, `bootstrap_generated.rs`, and
  `bootstrap_generated_without_parse_surface.rs` from the source tree.
- The fresh path lives behind the `bootstrap-regen-fresh` feature in
  `src/v3/compiler/src/bootstrap_regen_fresh.rs` and is intentionally excluded
  from default builds; `regen_bootstrap.rs` documents that boundary.
- Moving generation into `build.rs` would make the build script responsible for
  compiling/parsing the same v3 compiler substrate it is preparing for the
  crate. That needs a separate generator binary/library boundary or a staged
  bootstrap plan; it cannot be done by only flipping the feature default.
- The generated modules are also used by tests and normal library consumers as
  Rust source. Replacing them with `OUT_DIR` generation changes include paths,
  rebuild costs, and the SG-0 generated-file census.

Disposition: pursue as a dedicated dissolution lane. It is broader than this
pre-merge enforcement slice.

## Path 2: Treat Snapshots As Opaque Artifacts

Target shape: stop reviewing the bootstrap snapshots as editable source and
treat them as compiled or opaque generated artifacts.

This does not fit the current Cargo/Rust consumption shape:

- The snapshots are included as Rust modules and type-checked with the crate.
  Cargo needs Rust source (from the source tree today, or `OUT_DIR` after Path
  1), not an opaque binary blob.
- Making them opaque while still committing them would preserve the drift
  surface but remove useful textual diagnostics from `regen_bootstrap --verify`.
- A real opaque-artifact path still needs a producer/consumer contract for how
  the artifact is generated, loaded, and verified. That is a larger packaging
  change, not a pre-merge gate.

Disposition: not the next small step. If selected, it should follow an explicit
artifact packaging design rather than replacing one checked-in parallel
representation with another.

## Interim Gate

Until one dissolution path is implemented, the minimal fail-closed improvement
is to run the existing verifier in the required `ci` job:

```sh
cargo run -p v3-compiler --features bootstrap-regen-fresh --bin regen_bootstrap -- --verify
```

This reuses the existing fresh-compile-vs-committed-snapshot authority and moves
it to the merge-blocking path that #1543 bypassed. It covers bootstrap-loaded
substrate authority edits generally because `regen_bootstrap` rebuilds the
snapshots from the full configured `.dag` source set, not from the specific
`induction.dag` case.

This gate is a fallback enforcement measure. The remaining debt is to dissolve
the checked-in generated snapshot surface through Path 1 or a separately
designed artifact path.

TODO: remove this required `ci` job gate once the committed bootstrap generated
snapshot surface is dissolved. Tracking issue: #1559.
