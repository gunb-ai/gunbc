# CI end-to-end accounting (one required run, every job, every step)

**Status:** analysis, not a ruling. It is the input to consolidating the required CI (`gunbc.witness.compiler_gate_workflow` → `.github/workflows/witnesses.yml`) into one job over one prepared corpus.

**Instrument.** Every figure below is from ONE run, `gunb-ai/gunbc` actions run **37501346261** (pull_request, 2026-10-06; head `f937ee1b`; all lanes green). The figures come from that run's job API timestamps, the step timings, and the per-phase lines the binary prints (`[floor-phase]`, `[pre-entry]`, `✅ … done in`). They are a dated receipt for that run and are not a standing fact: to re-derive them, re-read any required run's logs the same way, and use the floor's per-claim cost artifact `required-floor-claim-cost` for claim cost. The 787 claim rows sum from that artifact's `observed_cpu_ms` column.

## 0. The run as a whole

| | |
|---|---|
| Run created → aggregate green | 17:09:29 → 18:19:02 = **69.6 min wall** |
| Accumulated queue time | 33.9 min, which overlaps across jobs and cannot be subtracted from wall time |
| Runner time consumed | rust-unit-tests 5.9 + emit-build 51.6 + generated 19.4 + floor 41.0 + aggregate 0.1 = **~118 runner-min** |
| Marginal CPU of the 787 claims | **13.2 s** in total. `run_claim_measured` subtracts shared-artifact fill CPU (fixture compilation and other shared fills), which is reported separately, so this is **not** the full cost of the testing: a compiler test's compilation is test work |
| Jobs that each build the seed `gunbc` from scratch | 4 (rust-unit-tests via `cargo test`, emit-build, generated, floor) |

Critical path: emit-build (51.6 min) → the aggregate then waits **15.7 min for a free runner** to run a 5-second job.

## 1. Job `rust-unit-tests` (non-blocking under rung drop `rust_unit_tests_over_their_cap`)

| Step | Wall | Notes |
|---|---|---|
| Queue for runner | 1.0 min | |
| Checkout, toolchain isolation, toolchain install, rustup pin | ~0.3 min | Same prelude in every job |
| `cargo test --release -p v1-compiler --lib` | ~5.5 min | Release build of v1-compiler + test harness, then the tests |

## 2. Job `emit-build` (required)

| Step / phase | Wall | Notes |
|---|---|---|
| Queue for runner | 2.1 min | |
| Prelude (checkout `fetch-depth: 0`, toolchain) | 0.3 min | |
| **Build the compiler** `cargo build --release -p v1-compiler --bin gunbc` | **4.1 min** | Seed build #2 |
| `gunbc test //gunbc/instruments:self-host` | **25.9 min** | Breakdown below |
| ↳ load + index, `[census]` 7,742 modules name-census only | 1.0 min | Whole-tree read/parse #n |
| ↳ compile.frontend / normalize | 0.2 min | |
| ↳ **compile.reconcile** (closure of `v2.compiler.compile`) | **3.2 min** | Reconcile #1 |
| ↳ analyses | <0.1 min | |
| ↳ **compile.emit** (seed emitter, 244 files) | **12.4 min** | Emit #1 |
| ↳ cargo build of the emitted crate | 4.1 min | |
| ↳ discriminating red: inject a type error, rebuild, expect failure | **4.8 min** | Re-proves the instrument, not the PR |
| ↳ spawn the built binary (census pair over `fixtures/native_cli_door`) | <0.1 min | **The actual execution evidence is ~1 s** |
| `gunbc test //gunbc/instruments:v2-native-cli` | **20.9 min** | Breakdown below |
| ↳ load + index, `[census]` 7,777 modules | 1.0 min | Whole-tree read/parse again |
| ↳ **compile.reconcile** (closure of `v2.cli.compile_cli`) | **3.4 min** | Reconcile #2, heavily overlapping closure |
| ↳ **compile.emit** (209 files) | **9.0 min** | Emit #2, heavily overlapping file set |
| ↳ cargo build of the emitted crate | 3.2 min | |
| ↳ discriminating red rebuild | **3.8 min** | |
| ↳ door / refusal / filesystem controls | <0.1 min | ~1 s |

## 3. Job `generated` (required; lane `build`)

| Step / phase | Wall | Notes |
|---|---|---|
| Queue for runner | 4.9 min | |
| Prelude | 0.3 min | |
| **Build the compiler and the witness executor** (release) | **3.9 min** | Seed build #3 |
| **Lint every target** `cargo clippy --all-targets -- -D warnings` | 1.7 min | A second, dev-profile compile of the whole workspace |
| `claim_executor --required-ci --required-lane build` startup | 1.0 min | |
| generated-artifact: docs projections | 2.8 min | The `✅ typecheck <module>` lines are **slow cache-miss module typechecks** (≥2 s), not separate projection invocations |
| generated-artifact: registry projections (59 rostered) | 6.6 min | One shared resolved context for all 59 artifacts; the typecheck lines are slow module misses |
| generated-artifact: stage0 mirrors | (overlaps) | Runs as a **child process concurrently** with the docs and registry checks, so its durations are not additive. Its own v1 mirror population: `regen.corpus_load` 52 s → **compile.reconcile 77 s** → `compile.emit` (the 5-minute line is buffered output, not established CPU) → mirror write |
| Repair-candidate steps (only on drift) | 0 | Skipped on a green run |

## 4. Job `floor` (required; lane `witnesses`)

| Step / phase | Wall | Notes |
|---|---|---|
| Queue for runner | 10.3 min | |
| Prelude | 0.4 min | |
| **Build the compiler and the witness executor** (release) | **3.7 min** | Seed build #4 |
| `Nominal witnesses` step | **36.6 min** | Breakdown below |
| ↳ lane roster + startup | 0.1 min | |
| ↳ phase `parse`: 8,042 files | 3.3 min | Whole-tree parse (68 s CPU to the parse seam, 2.2 min more to `parse OK`) |
| ↳ phase `declarations` + rostered-row join | <0.1 min | |
| ↳ phase `primitive-runtime-body` → `[pre-entry]` | (inside the above) | **Re-tokenizes the whole tree (22 s), re-parses heads (12 s), census-parses (7.5 s), builds the closure name census (15 s), reconcile fills (19 s), `resolve_entry_graph_inclusive` 92 s** |
| ↳ phase `bare-reference-admission` (7,980 files) | 0.2 min | |
| ↳ changed-witness planning: diff, base-declaration census, unimported-bare-provider frontier + gate | 2.3 min | `standing_ms=83188`, another pass over all 7,980 files |
| ↳ diff edits, body-reach / interface-consumer planning | 1.3 min | |
| ↳ gate closure #1 (103 modules) + its own compile | 0.3 min | Compile #a |
| ↳ gate closure #2 (2,931 modules) | 0.4 min | |
| ↳ **strict preparation: compile of the 2,931-module subject** | **11.1 min** | frontend 58 s, normalize 5 s, **compile.reconcile 9.9 min** (reconcile #4, the largest), analyses 6 s |
| ↳ warms (indexes, census, bare-reference edges) | 0.2 min | |
| ↳ declarer discovery + closure-strict-resolve (23 sources) | <0.1 min | |
| ↳ **discovery authority** (re-scans 7,980 sources → 28,986 rows) | **1.6 min** | Another whole-tree pass to find the claims |
| ↳ site projection → 787 claims | 0.2 min | |
| ↳ cross-claim share derivation + install | 0.4 min | |
| ↳ **claim evaluation fold** | **4.7 min** | Includes 282 fixture `compile.reconcile` calls at ~220 ms. These are compiler-test fixtures, so they are test work, not a re-preparation of the subject |
| ↳ terminal ledger publish | <0.1 min | The verdict is now known |
| ↳ **fixture-closure-union-emit** | **10.0 min** | Collects the dependency sources of the 235 fixture compiles that ran earlier, deduplicates them, compiles the union once and emits it. **It is a blocking emitter-coverage check**: its errors fail the floor even though the ledger was published first |
| D0 measure / publish / adjudicate, cost-receipt upload | <0.1 min | |

## 5. Job `witnesses` (aggregate; the required context)

| Step | Wall | Notes |
|---|---|---|
| Wait for `needs: [emit-build, floor, generated]` | (critical path) | |
| **Queue for a runner** | **15.7 min** | Contends with full jobs for a full self-hosted slot |
| Read the lane results | 0.1 min | |

---

## 6. Redundant work (corrected after review)

**Corrections from review of the first draft.** The first version of this table misread several entries:
- **R8:** the per-claim compiles are fixture compiles, and they are test work.
- **R12:** the fixture tail is a blocking check over one union, not post-verdict waste.
- **R13:** the typecheck log lines are slow module cache misses, not separate projection typechecks.
- **R9:** the two native closures cannot be emitted as one union. They declare different `compiler_pipeline_entry` drivers, the emitter admits exactly one entry per closure, and stage0 regenerates its own v1 mirror population.
- **R4:** queue time overlaps across jobs.

The table below keeps only what the code supports.

| # | Work | Where | Runner-min (this run) | Disposition |
|---|---|---|---|---|
| R1 | Seed `gunbc` / `claim_executor` built per job, cold (checkout `git clean -ffdx` removes `target/`; toolchain `cache: false`; fresh isolated `CARGO_HOME`) | emit-build, generated, floor (+ the separate test-harness build in rust-unit-tests) | ~11.8 across the three binary builds | Build or restore once per run and hand the product to every consumer; restore into private writable dirs. **≈7–8 min** |
| R2 | Native fault-and-restoration experiment (inject an error, require the failure, restore, rebuild, compare identity) | emit-build ×2 | ~8.6 | Run when the instrument, producer, build wiring, cache or toolchain changes; keep the small behavioral controls per PR. **≈8.6 min** |
| R3 | Native emit + build with no reuse across runs (fresh private source and target dirs per instrument) | emit-build ×2 | ~38 after R2 | Key each entry's product by its effective inputs (producer, closure and declaration universe, target, runtime deps, driver, toolchain); reuse on a hit and run the controls against it. **The largest single item** |
| R4 | Rust unit tests and clippy on changes that touch no Rust/build input | rust-unit-tests, generated | ~7.6 | Run when Rust, generated Rust, embedded `.dag`, Cargo config or build scripts change |
| R5 | Full heavy run for hand-authored prose (e.g. this doc's own PR) | all | ~118 | Narrow prose route that runs the applicable document checks only |
| R6 | Aggregate waits for a full runner slot | witnesses | 0.08 (15.7 min of latency) | Fold into the required job, or give it a cheap route. Latency win, not a runner-minute win |
| R7 | Fixture emit-check memo key includes the `includes`/`excludes` assertions, which are evaluated after compilation | floor | unmeasured | Key on source and render; evaluate each assertion separately |
| R8 | Cold-path cost: floor strict-preparation `compile.reconcile` 9.9 min; native emits 12.4 / 9.0 min | floor, emit-build | ~30 | Needed whenever inputs change; root-cause the reconcile and emit cost |
| R9 | Hosted `heal-publish` follow-on with nothing to publish | heal-publish | ~0.5 | Outside the headline; small |

### Budget (implementation target, not a forecast)

| Work | Target runner-min |
|---|---:|
| Setup, restore, one seed/executor preparation | ≤4 |
| Corpus acquisition, admission, shared preparation | ≤5 |
| Witness execution, fixture compilation, fixture-union emission | ≤6 |
| Both native products and their execution controls | ≤9 |
| Relevant generated artifacts and Rust checks | ≤4 |
| Reporting and overhead | ≤1 |
| **Total** | **≤29** |

The easy cuts (R1, R2, R4) remove about 23–24 min on an eligible run. The budget also needs R3 (native product reuse) and R8 (cold-path cost). Moving work to the merge queue cuts PR latency but still spends runner-minutes, so PR and merge-group cost are reported together.

### Work programme

1. **Scope and build sharing:** R1, R2, R4, R5, R6.
2. **Native product reuse:** R3.
3. **Cold-path reductions:** R7, R8.
