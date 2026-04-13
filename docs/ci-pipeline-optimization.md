# CI Pipeline Optimization: eliminate redundant self-compiles

## Problem

CI takes ~18 minutes. Of that, ~15 minutes is redundant computation.
The CI pipeline runs 8 gates sequentially. Five of them independently
run `cargo build -p v2-compiler --release` then
`v2-compiler compile --source-root src/v2 --source-root dsl`.

| Gate | Builds compiler? | Self-compiles .dag? | Notes |
|------|-----------------|-------------------|-------|
| clippy | Full workspace | No | Lint only |
| tests | Test crate | No | Unit tests, ~40s |
| full_dsl_compiles | Release binary | YES (1x) | |
| diagnostic_ratchet | Release binary | YES (1x) | Counts diagnostics |
| l1_ratchet | No | No | grep only, <1s |
| freshness | Release binary | YES (1x) + diff | |
| performance_ratchet | Release binary | YES (1x, timed) | |
| bootstrap_fixed_point | Release binary | YES (2x) | pass1 == pass2 |

That's **6 self-compiles of the same .dag source with the same binary**
(1+1+1+1+2). Each takes ~37s on CI. Total: ~220s just on redundant
self-compile, plus ~50s × 5 for redundant `cargo build --release`.

## What CI actually needs to verify

Strip away the implementation, these are the 8 claims:

1. **Lint clean** — no clippy warnings (workspace scope)
2. **Unit tests pass** — 395 tests (test crate scope)
3. **All .dag compiles** — compiler handles the full dsl/ surface
4. **Diagnostic ratchet holds** — CX violation count <= threshold
5. **L1 type knowledge = 0** — no type constructors in compiler .dag
6. **Stage0 is fresh** — committed .rs matches what compiler produces
7. **Performance holds** — self-compile within time budget
8. **Fixed point** — regen(regen(source)) == regen(source)

Claims 3, 4, 6, 7, 8 ALL start from the same input: "build the
release binary, compile all .dag source." They differ only in what
they CHECK on the output.

## The fix: compile once, check many

**Phase 1 (the build):**
```
cargo build -p v2-compiler --release
```
One build. Shared by all gates that need the binary.

**Phase 2 (the single self-compile):**
```
v2-compiler compile --source-root src/v2 --source-root dsl \
  --output-dir .ci-output
```
One compile. This produces:
- The generated .rs files (for freshness check)
- Diagnostics on stderr (for diagnostic ratchet)
- Timing data (for performance ratchet)

**Phase 3 (the checks — all read from Phase 2 output):**
- **full_dsl_compiles:** Phase 2 succeeded → pass
- **diagnostic_ratchet:** Count diagnostics from Phase 2 stderr → compare to threshold
- **freshness:** diff .ci-output vs committed stage0 → pass/fail
- **performance_ratchet:** Phase 2 wall time → compare to budget

**Phase 4 (fixed-point — one additional compile):**
```
# Copy phase 2 output to stage0, rebuild, compile again
cp .ci-output/src/*.rs src/v2/stage0/src/
cargo build -p v2-compiler --release  # rebuild with new stage0
v2-compiler compile ... --output-dir .ci-pass2
diff -r .ci-output .ci-pass2  # fixed point
```

This is 1 build + 1 compile (Phase 1+2), then 1 rebuild + 1
recompile (Phase 4). Total: 2 builds + 2 compiles, vs the
current 5 builds + 6 compiles.

**L1, lint, tests are independent and unchanged.**

## Expected impact

| Metric | Current | After |
|--------|---------|-------|
| Self-compiles | 6 | 2 |
| cargo build --release | 5 | 2 |
| Estimated CI time | ~18 min | ~6-8 min |

The ~3x speedup comes from eliminating ~4 redundant self-compiles
(~37s each = ~150s) and ~3 redundant builds (~50s each = ~150s).

## Implementation in compile.dag

The current `compile_sources` function returns `PipelineResult`
which already carries `diagnostics`, `complexity`, `ownership`,
and `files`. The CI runner just needs to READ these instead of
re-computing them.

The change: add a `compile_with_ci_claims` function (or extend
`compile_sources`) that returns the pipeline result PLUS the CI
claim data:

```dag
type CIClaimData {
  diagnostic_count: Int
  complexity_violation_count: Int
  wall_time_ms: Int
  output_files: List<TextFile>
  output_dir: String
}

type CIPipelineResult {
  pipeline: PipelineResult
  claims: CIClaimData
}
```

Then `ci_runner.dag` becomes:

```dag
func run_ci_pipeline() -> ProcessExit {
  // Phase 1+2: build + compile once
  let result = compile_with_ci_claims(sources: all_sources(), target: Rust)

  // Phase 3: check claims from the single compile
  let lint_ok = run_lint()
  let tests_ok = run_tests()
  let l1_ok = run_l1_ratchet()
  let dsl_ok = result.pipeline.diagnostics |> filter(is_error) |> count == 0
  let diag_ok = result.claims.diagnostic_count <= DIAG_RATCHET
  let fresh_ok = diff_against_stage0(output: result.claims.output_dir)
  let perf_ok = result.claims.wall_time_ms <= PERF_RATCHET_MS

  // Phase 4: fixed point (one more build + compile)
  let fp_ok = check_fixed_point(first_output: result.claims.output_dir)

  // Report
  ...
}
```

The key: `compile_with_ci_claims` runs ONCE. Every downstream check
reads from its output. No second compile.

## What changes in ci.dag

The current model — 8 independent `CIGate` items, each with a
`command: String` — doesn't capture the dependency between gates.
The model says "these are 8 independent commands." The reality is
"5 of them share the same prerequisite (build + compile)."

Two options:

**Option A — Keep gate list, make runner smarter.** ci.dag stays
as-is. `ci_runner.dag` internally recognizes that gates 3-8 share
a compilation and runs it once. Simpler to implement, but the
ci.dag model is still lying about independence.

**Option B — Model the dependency.** ci.dag declares compilation
as a shared prerequisite:

```dag
type CIPhase
  = BuildPhase { command: String }
  | CompilePhase { command: String, depends_on: CIPhase }
  | CheckPhase { gate: CIGate, reads_from: CIPhase }
  | IndependentPhase { gate: CIGate }
```

Then the pipeline is:
```dag
data build_phase = BuildPhase { command: build_command(...) }
data compile_phase = CompilePhase { command: compile_command(...), depends_on: build_phase }
data lint = IndependentPhase { gate: lint_gate }
data tests = IndependentPhase { gate: test_gate }
data l1 = IndependentPhase { gate: l1_gate }
data dsl_check = CheckPhase { gate: full_dsl_gate, reads_from: compile_phase }
data diag_check = CheckPhase { gate: diagnostic_ratchet_gate, reads_from: compile_phase }
data fresh_check = CheckPhase { gate: stage0_freshness_gate, reads_from: compile_phase }
data perf_check = CheckPhase { gate: performance_ratchet_gate, reads_from: compile_phase }
data fp_check = CheckPhase { gate: bootstrap_fixed_point_gate, reads_from: compile_phase }
```

This is the honest model. The runner reads the dependency graph
and only runs each phase once.

**Recommendation: Option A for v1, Option B as a follow-up.** The
immediate win is in the runner logic, not the model restructuring.
Option B is the right eventual shape (and it connects to Track 16's
YAML generation — the phase dependencies would drive the YAML
structure too).

## Scope for v1 PR

1. Add `compile_with_ci_claims` or equivalent to `compile.dag`
   that captures wall time, diagnostic count, and output dir.
2. Restructure `ci_runner.dag` to call compile ONCE, then check
   claims from the result.
3. Move freshness check from shell script to a diff on the compile
   output (already have the output dir).
4. Move diagnostic ratchet from a separate test to a count on
   the compile output's diagnostics.
5. Keep lint, unit tests, L1 ratchet as independent shell commands.
6. Keep fixed-point as Phase 4 (needs a rebuild + recompile).
7. Update `.github/workflows/ci.yml` if the entry point changes.

**Not in scope:** Option B modeling (CIPhase dependency graph).
That's Track 16 territory.

## Done when

- CI runs ≤ 2 self-compiles (1 in Phase 2, 1 in Phase 4)
- CI wall time drops from ~18 min to ~6-8 min
- All 8 claims still verified (same coverage, less computation)
- `ci_runner.dag` reads compile output instead of re-running
