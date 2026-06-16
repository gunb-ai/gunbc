> Part of: [THESIS.md](THESIS.md) — self-hosting is a verification
> artifact. When the compiler compiles itself to identical output,
> the causal engine is proven internally consistent.

# Bootstrap

Stage0 (`src/v1/stage0/`) is the bootstrap seed for the self-hosted
compiler. It is a committed, reproducible artifact — never hand-edited
in ordinary development. Seed bumps are explicit maintenance operations
with a documented procedure.

## Current v2 mechanism

v1 stage0 regeneration is owned by the `regen_stage0` binary:

```bash
cargo run -p v1-compiler --bin regen_stage0
cargo run -p v1-compiler --bin regen_stage0 -- --verify
```

The first command writes the generated stage0 Rust files. The
`--verify` form performs a fresh self-compile and compares it to the
committed stage0 seed without writing.

Local parity: `make stage0-freshness-check` runs the `--verify` form.
CI runs the same check via `make ci` when validating the workspace.

## How it works

```
.dag source  --(stage0 binary compiles)--> new .rs files
new .rs files  --(cargo builds)----------> new stage0 binary
```

The stage0 binary reads `.dag` source, runs the full pipeline
(parse, resolve, normalize, infer, emit), and produces Rust `.rs`
files. Those files replace `src/v1/stage0/src/` and become the next
stage0 binary.

**Source roots:** `src/v1` (compiler source) + `dsl` (std, extdeps).
Import resolution is transitive — the compiler follows `import`
declarations, no manual file lists.

**Hand-maintained files** (not overwritten by `regen_stage0`):
- `cli_run.rs`
- `rest_transport_facts.rs`
- `v1_interpreter.rs`
- `Cargo.toml` (manifest, outside `src/`)

The generated-output registry lives in
`src/v1/stage0/src/bin/regen_stage0.rs`; both write mode and
`--verify` mode consume that one list.

## Ordinary regeneration

When a `.dag` change does not alter the compiler/stage0 boundary
(types, enums, function signatures shared between `.dag` and `.rs`):

```bash
cargo run -p v1-compiler --bin regen_stage0
```

This builds the current stage0 binary, self-compiles all `.dag`
source, copies the output to `src/v1/stage0/src/`, and verifies
the generated crate can be formatted.

Commit the `.dag` changes and the regenerated stage0 together:

```bash
git add src/v1/*.dag dsl/ src/v1/stage0/src/
git commit -m "Description of .dag change"
```

**CI verifies:** `cargo run -p v1-compiler --bin regen_stage0 -- --verify`.
Any `.dag` change that breaks regeneration blocks the PR.

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
2. Regenerate stage0: `cargo run -p v1-compiler --bin regen_stage0`
3. Verify: `cargo check -p v1-compiler` passes.
4. Commit and land the bridge.

### Step B: Complete

1. Remove the compatibility bridge — switch fully to the new shape.
2. Regenerate stage0 again.
3. Verify: `cargo check -p v1-compiler` passes.
4. Commit and land the completion.

### When a bridge is not possible

Some changes cannot be bridged (e.g., a type field is removed and
the stage0 binary's generated code references it in 500 sites). In
these cases, the stage0 `.rs` files may be directly patched as a
**seed bump**:

1. Make the `.dag` source change.
2. Patch `src/v1/stage0/src/*.rs` to match the new representation.
   Document what was patched and why in the commit message.
3. Rebuild: `cargo check -p v1-compiler`
4. Regenerate: `cargo run -p v1-compiler --bin regen_stage0`
5. Verify the regenerated output matches the patched state (no diff).
6. Commit everything together.

Seed bumps are maintenance operations, not ordinary development. Every
seed bump commit message must state: what representation changed, why
a bridge was not feasible, and how many sites were patched.

## Complexity-specific bootstrap guardrails

When regeneration is blocked and a complexity fix must touch stage0:

1. Patch `src/v1/complexity.dag` first.
2. Mirror the same algorithm change into `src/v1/stage0/src/v1_compiler_complexity.rs`.
3. Update the complexity parity audit in `src/v1/tests/src/source_audit.rs`.
4. Add or update a focused regression test in `src/v1/tests/src/pipeline.rs`.
5. Run `cargo test -p v1-compiler-tests pipeline::strict_complexity_violation_count -- --ignored --nocapture`.

The recent self-compile OOM root cause was not raw memory budget. It was:
- repeated complexity classification/report traversal on large compiles
- stale stage0 recursion classification relative to the source DAG logic

## PR classification

Every PR that touches `.dag` compiler source should state which kind
of change it is:

| Classification | Meaning | Workflow |
|---------------|---------|----------|
| **regen-safe** | Stage0 can compile the new `.dag` without issues | Ordinary regeneration |
| **bootstrap-breaking** | Stage0 cannot compile the new `.dag` | Two-step bridge, or seed bump with justification |

## Forbidden

- Direct hand edits to `src/v1/stage0/src/*.rs` outside an explicit
  seed bump workflow.
- Adding new bootstrap source lists in ad hoc places. All source
  resolution goes through `regen_stage0`'s source-root and import-driven
  resolution.
- Landing `.dag` changes that require stage0 edits without documenting
  the compatibility plan.
- Committing `.dag` changes without regenerating stage0 (once CI gate
  is active).

## Bootstrap manifest (planned)

The generated-output registry in `regen_stage0` is the authority for
which stage0 files are overwritten. Source discovery remains
import-driven from `src/v1`, `dsl`, and the generated method-template
projection source root.

## Verification

After any stage0 change:

```bash
cargo check -p v1-compiler                    # stage0 compiles
cargo test -p v1-compiler-tests               # fast tests pass
cargo clippy --all-targets -- -D warnings     # lint clean
```

After regeneration (once emitted Rust errors reach 0):

```bash
cargo run -p v1-compiler --bin regen_stage0
git diff --exit-code src/v1/stage0/           # no drift
```

## Current state (2026-04-04)

**BOOTSTRAP D GREEN.** Fixed-point convergence achieved (pass-3 = pass-2).
- 0 diagnostics, 0 cargo check errors
- 253 tests pass (32 fail — same as committed baseline)
- Diagnostic ratchet: 0

### .dag changes (branch: bootstrap-d-regen)
- `05_emit.dag`: `panic!` → `compile_error!` in type position (3 sites)
- `05_emit_rust.dag`:
  - `recursive_types: {}` → `empty_map()` (1 site)
  - Bridge fallbacks for unresolved empty_map (4 sites)
  - Type annotations disabled in 3 emit paths (Pass A)
  - `impl Fn` → `impl Fn + Clone` for TCO callable params

### Convergence iterations

| Pass | .dag changes | Result | Root cause |
|------|-------------|--------|------------|
| 1 | committed .dag | 1 error: `()` for `Rc<HashMap>` | `{}` literal emits as unit |
| 1-fix | `recursive_types: empty_map()` | 0 errors, pass-1 binary works | — |
| 2a | + inference fix | 90 errors | compile_error! strings inherited from .dag |
| 2b | + bridge fallbacks, no infer fix | 58 errors | staleness divergence |
| 2c | + bridge fallbacks + infer fix | 58 errors (SAME) | inference fix is orthogonal |
| 3 | + disable annotations + Clone | 0 errors pass-2 | fixed point achieved |

### Pass-2 error breakdown (58 errors)

| Category | Count | Pattern |
|----------|-------|---------|
| E0308 type mismatch | 47 | `Rc<Vec<()>>` vs concrete types |
| E0599 no method | 8 | `.clone()` on `impl Fn(...)` |
| compile_error! | 3 | CompilerError type in type position |

### Root cause (confirmed)

**The type annotation feature (`fe9fb7f27`) is bootstrap-breaking.**

- Stale `emit_typed_let` (line 3002): `emit_let_binding(name, val_str, Rust)` → `let x = val;`
- New `emit_typed_let` (05_emit_rust.dag:2824): adds `render_rust_type` annotation → `let x: Type = val;`

Bootstrap chain:
1. **Stale binary** compiles new .dag → pass-1 **without annotations** (stale uses old emit code)
2. **Pass-1 binary** (has new emit code from step 1) compiles new .dag → pass-2 **with annotations**
3. Annotations lock in under-resolved types: `Rc<Vec<()>>`, `Rc<HashMap<_, _>>`, `Option<i64>`
4. Without annotations, Rust infers correct types from usage context

**The 47 E0308 errors are all places where:**
- .dag inference returns `unit_type` for empty collections (by design — no expected propagation)
- Without annotation: Rust infers the right type from the expression context
- With annotation: the wrong type is locked in and propagates

**The 8 E0599 errors**: `impl Fn` params in TCO loops need `.clone()` but `impl Fn` doesn't implement Clone. Separate issue from annotations.

**The 3 compile_error!**: CompilerError type unresolved in type position. Separate inference gap.

### Fix strategy: monotone bootstrapping

Each bootstrap-breaking feature gets its own convergence pass:

**Pass A (DONE)**: Revert annotation emission + fix `impl Fn + Clone`.
- Disabled type annotations in 3 emit paths: `emit_typed_let`, `emit_func_body`, `emit_tco_init_stmt`
- Added `+ Clone` bound to `impl Fn` callable params in Rust emission
- Result: pass-2 = pass-1 = pass-3 (fixed point)

**Pass B (next)**: Fix empty collection inference, re-enable annotations.
- Fix inference for `[]` and `empty_map()` to propagate expected types
- Re-enable annotation emission
- Regen → pass-2 has correct annotations → convergence

**Pass C**: Fix CompilerError type resolution (3 sites).
