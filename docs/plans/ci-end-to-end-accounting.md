# CI end-to-end accounting (one required run, every job, every step)

**Status:** analysis, not a ruling. It is the input to consolidating the required CI (`gunbc.witness.compiler_gate_workflow` → `.github/workflows/witnesses.yml`) into one job over one prepared corpus.

**Instrument.** Every figure below is from ONE run, `gunb-ai/gunbc` actions run **37501346261** (pull_request, 2026-10-06; head `f937ee1b`; all lanes green). The figures come from that run's job API timestamps, the step timings, and the per-phase lines the binary prints (`[floor-phase]`, `[pre-entry]`, `✅ … done in`). They are a dated receipt for that run and are not a standing fact: to re-derive them, re-read any required run's logs the same way, and use the floor's per-claim cost artifact `required-floor-claim-cost` for claim cost. The 787 claim rows sum from that artifact's `observed_cpu_ms` column.

## 0. The run as a whole

| | |
|---|---|
| Run created → aggregate green | 17:09:29 → 18:19:02 = **69.5 min wall** |
| Runner time consumed | rust-unit-tests 5.9 + emit-build 51.6 + generated 19.4 + floor 41.0 + aggregate 0.1 = **~118 runner-min** |
| Time spent evaluating the 787 claims (the thing the floor exists to do) | **13.2 s CPU** in total; the slowest claim is 432 ms |
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
| generated-artifact: docs projections | 2.8 min | Each projection **typechecks its own module set separately** (`✅ typecheck … done in` ×7) |
| generated-artifact: registry projections (59 rostered) | 6.6 min | Same pattern, ~20 separate per-module typechecks, then 59 regenerations |
| generated-artifact: stage0 mirrors | 3.0 min | `regen.corpus_load` 52 s (whole tree again) → frontend → **compile.reconcile 77 s** (reconcile #3) → **compile.emit 5 min CPU** (emit #3) → mirror write |
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
| ↳ **claim evaluation fold** | **4.7 min** | **282 per-claim `compile.reconcile` at ~220 ms ≈ 1.0 min**, plus per-claim compile overhead. The claims themselves total 13.2 s CPU |
| ↳ terminal ledger publish | <0.1 min | The verdict is now known |
| ↳ **fixture-closure-union-emit** (235 fixture compiles, 55 memo hits) | **10.0 min** | Runs **after** the verdict was published |
| D0 measure / publish / adjudicate, cost-receipt upload | <0.1 min | |

## 5. Job `witnesses` (aggregate; the required context)

| Step | Wall | Notes |
|---|---|---|
| Wait for `needs: [emit-build, floor, generated]` | (critical path) | |
| **Queue for a runner** | **15.7 min** | Contends with full jobs for a full self-hosted slot |
| Read the lane results | 0.1 min | |

---

## 6. Redundant work

Each row is work done more than once per run, or work that the run's verdict does not consume. "Once" is the target: done a single time and read by every consumer. The rows here are this run's measured instances; the classes they belong to are the redundancy the consolidation removes.

| # | Work | Where it happens in this run | Times per run | Wall spent | If done once | Removable | Kind |
|---|---|---|---|---|---|---|---|
| R1 | Release build of the seed `gunbc` (+ `claim_executor`) | rust-unit-tests, emit-build, generated, floor | 4 | ~17.2 min | ~4 min | **~13 min of runner time** | duplicated across jobs |
| R2 | Second, dev-profile compile of the whole workspace for clippy | generated | 1 (beside R1) | 1.7 min | could share the check artifacts | small | duplicated profile |
| R3 | Job prelude: checkout, toolchain isolation, toolchain install, pin | every job | 5 | ~1.5 min | ~0.3 min | ~1.2 min | duplicated across jobs |
| R4 | Queueing for a runner | every job | 5 | **34 min** (1.0 + 2.1 + 4.9 + 10.3 + 15.7) | 1 queue | **most of it**, including all 15.7 min of the aggregate's queue | multi-job fan-out |
| R5 | Whole-tree read / tokenize / parse of ~8,000 files | floor `parse`; floor `[pre-entry]` re-tokenize and re-parse heads; floor census parse; emit-build ×2 index; generated `regen.corpus_load` | ≥6 | ~8–9 min | ~1–2 min | **~7 min** | duplicated within and across jobs |
| R6 | Whole-tree **name census / declaration scan** over 7,980 files | floor `[pre-entry]` closure name census; floor unimported-bare-provider frontier; floor discovery authority; emit-build `[census]` ×2 | ≥5 | ~5–6 min | once | **~4–5 min** | duplicated within and across jobs |
| R7 | **compile.reconcile** over large, overlapping closures | floor strict preparation (2,931 modules, 9.9 min); emit-build self-host (3.2 min); emit-build native-cli (3.4 min); generated stage0 mirrors (1.3 min); floor `[pre-entry]` reconcile fills + entry-graph resolve (~2 min) | 5 | **~20 min** | one reconcile of the union closure | **~10+ min**, more once reconcile is fixed | duplicated across jobs + a cost-shape defect (9.9 min for 2,931 modules) |
| R8 | **Per-claim recompile** inside the claim fold | floor claim evaluation | 282 | ~1 min reconcile + ~3.5 min other per-claim compile overhead | the subject is already prepared | **~4 min** | re-preparing a prepared subject |
| R9 | **Seed emit** over overlapping v2 closures | emit-build self-host (12.4 min, 244 files); emit-build native-cli (9.0 min, 209 files); generated stage0 mirrors (5 min) | 3 | ~26 min | one emit of the union | **~10–15 min** | duplicated + cost-shape defect (emit at ~3 s per file) |
| R10 | Discriminating-red rebuild (inject a fault, rebuild, expect a red) | emit-build ×2 | 2 per run, every PR | 8.6 min | once per **change to the instrument**, not per PR | **8.6 min** | re-proves the instrument, not the subject |
| R11 | cargo build of two emitted crates sharing most of their modules | emit-build ×2 | 2 | 7.3 min | one crate or a shared target dir | ~3 min | duplicated |
| R12 | **Fixture-closure-union emit** (235 fixture compiles) | floor, after `terminal-ledger-publish` | 1 | **10.0 min** | — | **10 min on the critical path** (off-path or merge-group only) | runs after the verdict is known |
| R13 | Per-projection separate typechecks in generated-artifact | generated docs + registry projections (~27 `✅ typecheck` lines) | ~27 | ~9.4 min total for the two populations | one typecheck of the union, read by every projection | **~6–7 min** | duplicated within a job |
| R14 | A full self-hosted runner slot to aggregate lane results | `witnesses` aggregate | 1 | 15.7 min queue + 5 s work | in-job | **15.7 min** | fan-out cost |
| R15 | Claim evaluation itself | floor | 1 | 13.2 s CPU | — | **not redundant: this is the work** | — |

### Reading the table

- **Useful work in a ~70-minute, ~118-runner-minute run is small:** 787 claims in 13 s, one seed build, one clippy, one parse, one reconcile of the subject, and, if it stays on the PR path, one emit + build + ~1 s of spawning the built binaries.
- **R4/R14 (queueing) and R1/R3 (per-job setup) exist only because the run is four jobs.** One job removes them by construction.
- **R5–R9 and R13 share one cause: no prepared corpus is shared between consumers.** Every lane, phase and projection re-derives parse → census → resolve → reconcile for itself. DESIGN §2 says to carry the first value to the least common ancestor of its demands. In one job, that ancestor is the job.
- **R7 and R9 are also cost-shape defects** (DESIGN §6, bare minimum cost). A 9.9-minute reconcile and an emitter that spends ~3 s per file are wrong at any n, so they get fixed regardless of the consolidation.
- **R10 and R12 are checks that do not need to run per PR.** R10 re-proves the instrument, and R12 runs after the verdict. They move to the merge queue or to a change-triggered lane, each with a declared `gunbc.rung_drop` row if anything leaves the merge path.
