# Bootstrap

Stage0 (`src/v2/stage0/`) is the bootstrap seed for the self-hosted
compiler. It is a committed, reproducible artifact — never hand-edited
in ordinary development. Seed bumps are explicit maintenance operations
with a documented procedure.

## How it works

```
.dag source  --(stage0 binary compiles)--> new .rs files
new .rs files  --(cargo builds)----------> new stage0 binary
```

The stage0 binary reads `.dag` source, runs the full pipeline
(parse, resolve, normalize, infer, emit), and produces Rust `.rs`
files. Those files replace `src/v2/stage0/src/` and become the next
stage0 binary.

**Source roots:** `src/v2` (compiler source) + `dsl` (std, extdeps).
Import resolution is transitive — the compiler follows `import`
declarations, no manual file lists.

**Hand-maintained files** (not regenerated):
- `v2_rt.rs` — runtime primitives
- `compiler_tests.rs` — test suite (reads .dag files from disk, no embedded source)
- `main.rs` — CLI entry point
- `Cargo.toml` — dependencies

## Ordinary regeneration

When a `.dag` change does not alter the compiler/stage0 boundary
(types, enums, function signatures shared between `.dag` and `.rs`):

```bash
./scripts/regenerate-stage0.sh
```

This builds the current stage0 binary, self-compiles all `.dag`
source, copies the output to `src/v2/stage0/src/`, and verifies
`cargo check` passes.

Commit the `.dag` changes and the regenerated stage0 together:

```bash
git add src/v2/*.dag dsl/ src/v2/stage0/src/
git commit -m "Description of .dag change"
```

**CI verifies:** `regenerate-stage0.sh && git diff --exit-code src/v2/stage0/`
(planned — not yet wired). Any `.dag` change that breaks regeneration
blocks the PR.

## Bootstrap-breaking changes

A change is **bootstrap-breaking** when the current committed stage0
cannot compile the new `.dag` source — because the change alters
something the stage0 binary depends on:

- Node struct fields (D-series dissolutions)
- Enum variant layout (e.g., MethodSemantics restructuring)
- Container representation (e.g., FF-8 Rc-wrapping)
- Runtime function signatures
- Import structure (new modules the binary can't resolve)
- Parser syntax (e.g., generic function support)

These changes require a **two-step compatibility window**:

### Step A: Bridge

1. Teach the compiler to handle **both** old and new representations.
   The `.dag` source accepts both shapes temporarily.
2. Regenerate stage0: `./scripts/regenerate-stage0.sh`
3. Verify: `cargo check -p v2-compiler` passes.
4. Commit and land the bridge.

### Step B: Complete

1. Remove the compatibility bridge — switch fully to the new shape.
2. Regenerate stage0 again.
3. Verify: `cargo check -p v2-compiler` passes.
4. Commit and land the completion.

### When a bridge is not possible

Some changes cannot be bridged (e.g., a type field is removed and
the stage0 binary's generated code references it in 500 sites). In
these cases, the stage0 `.rs` files may be directly patched as a
**seed bump**:

1. Make the `.dag` source change.
2. Patch `src/v2/stage0/src/*.rs` to match the new representation.
   Document what was patched and why in the commit message.
3. Rebuild: `cargo check -p v2-compiler`
4. Regenerate: `./scripts/regenerate-stage0.sh`
5. Verify the regenerated output matches the patched state (no diff).
6. Commit everything together.

Seed bumps are maintenance operations, not ordinary development. Every
seed bump commit message must state: what representation changed, why
a bridge was not feasible, and how many sites were patched.

## PR classification

Every PR that touches `.dag` compiler source should state which kind
of change it is:

| Classification | Meaning | Workflow |
|---------------|---------|----------|
| **regen-safe** | Stage0 can compile the new `.dag` without issues | Ordinary regeneration |
| **bootstrap-breaking** | Stage0 cannot compile the new `.dag` | Two-step bridge, or seed bump with justification |

## Forbidden

- Direct hand edits to `src/v2/stage0/src/*.rs` outside an explicit
  seed bump workflow.
- Adding new bootstrap source lists in ad hoc places. All source
  resolution goes through `--source-root` and import-driven resolution.
- Landing `.dag` changes that require stage0 edits without documenting
  the compatibility plan.
- Committing `.dag` changes without regenerating stage0 (once CI gate
  is active).

## Bootstrap manifest (planned)

Today, multiple places maintain independent lists of "what bootstrap
sees" — `prepare_sources()` in bootstrap tests, embedded source
constants in `compiler_tests.rs`, the regen script. These will
converge to a single manifest consumed everywhere. Until then, the
regen script's `--source-root` resolution is the authority.

## Verification

After any stage0 change:

```bash
cargo check -p v2-compiler                    # stage0 compiles
cargo test -p v2-compiler-tests               # fast tests pass
cargo clippy --all-targets -- -D warnings     # lint clean
```

After regeneration (once emitted Rust errors reach 0):

```bash
./scripts/regenerate-stage0.sh
git diff --exit-code src/v2/stage0/           # no drift
```

## Current state (2026-03-29)

- **Regeneration works:** 40 files emitted, 0 diagnostics, ~112MB, ~2 min.
- **Emitted Rust errors:** 1397 (emitter codegen bugs, not pipeline failures).
  These must reach 0 before the CI freshness gate can be activated.
- **Blockers removed this session:**
  - `trace.dag` generic function import (parser limitation)
  - 689 redundant O(n) Vec clones in stage0
  - O(n^2) complexity report (skipped for >100 functions)
  - 22 D1 dissolution field access errors in `.dag` source
