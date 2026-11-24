# Serve-entry whole-closure compile — phase breakdown (measurement only)

Node: `node://demand-serve-entry-compile-time` (roadmap_authority.dag row
`demand-serve-entry-compile-time`, owner `self-host`, carrier
`dag/gunbc/roadmap/roadmap_ingress.dag`, parent `demand-engine`).

SLICE TYPE: **MEASUREMENT ONLY** — the first_slice leg ("A phase breakdown of the
compile on one revision, with the closure's module count"). This document writes
**no** reduction, changes **no** step bound, and files **no** acceptance request.
`gunbc.design_ruling::serve_entry_verdict_step_raised_to_45_minutes` stays as it is.

## Route honesty, stated first

- **This slice EXECUTES NO RED CONTROL and claims none.** The row's `red_control`
  names its witness as the *program* `record_plan_serve_entry_verdict_for` via
  `observe_served_command_reaches_bind` (carrier `dag/gunbc/roadmap/roadmap_ingress.dag`),
  to complete under 30 minutes on srv1 with its receipt. `red_at` and `green_at` for
  that leg exist only once a reduction exists (current main over the bound, the
  change's head under it). Nothing here is that witness; every number below is
  "executed as measurement", never "green by execution" of a claim.
- The row's own words separate the two programs: *"A whole-closure `gunbc compile`
  of the entry is a different program, measured separately … and not governed by
  this budget."* Everything below is that second program.
- **Block gate: UNAVAILABLE, never green.** For this node the block gate against
  `REPO=/opt/gunbc/gunbc` returned `REV-UNREACHABLE c48e06d826157ffd9df2c4830ac2985524f8f211`,
  `VERDICT=NO-GATE`, rc 3.
- **Recording is closed** (recorder PRs 16964 / 16965 unmerged, lab not advanced);
  the recorder was not run.
- Gate vs measurement, per the lane's standing rules: the numbers below are
  **MEASUREMENT** (wall and RSS of a real invocation), never a gate green. They use
  the two-root form `dag` + `src/v2` — exactly what the in-repo caller passes — and
  needed no `--source-root src/v1` (both runs exit 0 without it). No claim timing is
  mixed into them.

## Environment (printed by every stage before anything else)

| field | value |
|---|---|
| host | `srv1` (`ssh -n -o BatchMode=yes -o HostName=srv1.tailecbe08.ts.net`) |
| BOOT | `c095c0cd` (`cut -c1-8 /proc/sys/kernel/random/boot_id`) |
| pinned tree | `2bc71eb167915908b0c1774fad71e8c93ce84092` (`TREE_SHA`, `TREE_DIRTY_LINES=0`) |
| binary | `/home/briansrls/scratch/valiant-gull-211/gunbc/target/release/gunbc` |
| binary sha256 (read from disk) | `c3022c764badc6966f8774ac833a30c7f504ac62e8ff408881e0c0a6bc4a637b` |
| cargo in-slot | `/home/briansrls/.cargo/bin/cargo` (`PATH` exported inside the slot) |
| HEAVY_OWNER | `valiant-gull-211` (every stage through `/home/briansrls/bin/heavy-slot`, one stage per acquisition) |
| node-digest | `ed5564b8560b5ba4` (unchanged from the digest given by the manager; `DIGEST_RC=0`, `VERIFICATION controls_ok=yes control=f1c76111c83a0a60 third=b3747b95baf099cc missing_rc=1`, `LAB_HEAD=c48e06d826157ffd9df2c4830ac2985524f8f211`, `GUNBC_SHA256=82e9a74a134dbd67`) |

Acquisitions used: **4 of 4** — build, compile run 1, node-digest pass, compile run 2.

```
build     heavy-slot: slot 1/12 acquired by valiant-gull-211 at 2026-11-24T17:56:15Z
run1      heavy-slot: slot 2/12 acquired by valiant-gull-211 at 2026-11-24T18:58:33Z
digest    heavy-slot: slot 1/12 acquired by valiant-gull-211 at 2026-11-24T19:33:43Z
run2      heavy-slot: slot 4/12 acquired by valiant-gull-211 at 2026-11-24T19:48:54Z
```

Logs: `~/scratch/valiant-gull-211/logs/build1.log`,
`~/tmp-valiant-gull-211/{run1,run2,digest1}.log` and the per-run
`~/tmp-valiant-gull-211/run{1,2}/{stdout,stderr,time_v}.txt`. Nothing was written
outside `~/scratch/valiant-gull-211` and `~/tmp-valiant-gull-211`; no `/opt` path,
no service, no host apply. **Nothing run or read here resolves
`/tmp/fleet-converge-plan/…`** — that fixed path is only in the row's own 2026-11-23
step-command note (its exit 1 was a `Permission denied` writing
`/tmp/fleet-converge-plan/serve_entry_verdict.json`), which is the receipt move the
fleet flagged.

Raw stage lines for the build and the digest pass (each its own acquisition):

```
# build1.log
heavy-slot: slot 1/12 acquired by valiant-gull-211 at 2026-11-24T17:56:15Z
pid=3472336 HEAVY_OWNER=valiant-gull-211  HOST=srv1  BOOT=c095c0cd
TREE_SHA=2bc71eb167915908b0c1774fad71e8c93ce84092  TREE_DIRTY_LINES=0
PREFLIGHT_command_v_cargo=/home/briansrls/.cargo/bin/cargo
--- end cargo build ---  BUILD_RC=0
Finished `release` profile [optimized] target(s) in 5m 47s
GUNBC_BINARY_SHA256=c3022c764badc6966f8774ac833a30c7f504ac62e8ff408881e0c0a6bc4a637b
STAGE_RC=0

# digest1.log
heavy-slot: slot 1/12 acquired by valiant-gull-211 at 2026-11-24T19:33:43Z
pid=359998 HEAVY_OWNER=valiant-gull-211  HOST=srv1  BOOT=c095c0cd
LAB_HEAD=c48e06d826157ffd9df2c4830ac2985524f8f211 GUNBC_SHA256=82e9a74a134dbd67
VERIFICATION controls_ok=yes control=f1c76111c83a0a60 third=b3747b95baf099cc missing_rc=1 missing_out=''
DIGEST demand-serve-entry-compile-time ed5564b8560b5ba4
DIGEST_RC=0
STAGE_RC=0
```

### The program measured, and why this argv

`gunbc compile` requires `--output-dir` (`main.rs:24-28`, `required = true`), so the
bare argv in the row's `boundary` text cannot be executed as written. The whole-closure
compile of the serve entry is therefore run exactly as its only in-repo caller spells it —
`dag/gunbc/live_deploy/pre_swap_admission.dag :: observe_serve_entry_compile_clean`:

```
gunbc compile --source-root dag --source-root src/v2 \
  --entry dag/gunbc/roadmap/roadmap_ingress.dag --output-dir <scratch>/out --target dag
```

cwd = the checkout root, `GUNBC_FLOOR_PHASE_PROFILE=1` armed (see the phase-profile row
below for what that produced).

## The two runs (raw lines, real rc)

Both from `~/tmp-valiant-gull-211/run{1,2}.log`; the rc is the stage's own, read from the
command, never through a pipe.

| | run 1 | run 2 |
|---|---|---|
| wrapper pid / stage pid | `4083561` / `4083575` | `549725` / `549741` |
| slot .who line | `slot2.who -> valiant-gull-211 2026-11-24T18:58:33Z …` | `slot4.who -> valiant-gull-211 2026-11-24T19:48:54Z …` |
| `COMPILE_RC` | `0` | `0` |
| Elapsed (wall clock) | `00:26:37.87` (1597.87 s) | `00:30:06.59` (1806.59 s) |
| User time (seconds) | `1585.39` | `1793.85` |
| System time (seconds) | `10.87` | `11.40` |
| Percent of CPU | `99%` | `99%` |
| Maximum resident set size | `14920016 kbytes` | `14908996 kbytes` |
| stderr / stdout lines | `125046` / `2` | `125046` / `0` |
| stderr sha256 | `8b414f65d0c5e6e6511f458e247245a95d24166c220d5d723f7b2a752c03f917` | `75928fc0d8bb29311563c61eb5e390f86b00ec59a6d2f25678def3338c089f45` |

**The two wall times differ by 13.1 %, so per the slice rule both are reported and the
machine load is printed rather than picking the better one.** `nproc` = `128`.
`uptime` readings (not all inside the run window — the first predates the acquisitions):
`load average: 36.63, 32.15, 34.15` at 16:24Z (before any acquisition), `36.44, 32.25,
29.79` at 20:24Z (minutes after run 2 ended), `33.50, 32.18, 31.29` at 20:35Z, 54–57 users
throughout. Slots held by other lanes during the window (from
`~/.heavy-slots/slot*.who`): `bright-newt-348`, `still-crab-552` (via `eager-koi-371`),
`calm-boar-153`, `bold-ibex-678`, `bold-lark-538`, `zesty-koi-131`. The difference sits in
one phase, not spread across all of them — `compile.reconcile` printed `19 minutes` in run 1
and `22 minutes` in run 2, ≈180 s of the ≈209 s gap.

Closing lines, identical in both runs (stderr):

```
resolved 1679 sources (reference-derived closure entry=dag/gunbc/roadmap/roadmap_ingress.dag)
0 blocking error(s), 31356 advisory diagnostic(s) (policy: gunbc.compile_clean_diagnostic_policy)
compiled: 1 files emitted, 31356 diagnostics
```

## Phase breakdown of the compile transaction

Two instruments print on this route, and only these. Every number below is a line the run
printed; nothing is allocated proportionally.

**Instrument A — the pre-entry receipt.** Producer:
`src/v1/stage0/src/cli_run/pre_entry_phase.rs` (`record` :294, `timed` :304,
`print_compile_transaction_receipt` :316), called once on the compile route at
`src/v1/stage0/src/cli_run.rs:6907` inside `compile_emission_over`. It knows
closure-collection phases only.

**Instrument B — the compile stage marks.** Producer:
`v1_rt::trace_mark` (`src/v1/stage0/src/v1_rt.rs:1736`), rendered by
`v1_rt.rs:1672` (`✓ {phase} done in {human}`) and `v1_rt.rs:1678`
(`◐ started {phase}`); the marks are emitted in
`src/v1/stage0/src/v1_compiler_compile.rs`. `trace_mark` computes a precise `elapsed_ms`
but renders it through `obs_human_duration` (`v1_rt.rs:1630-1642`), which floors to
seconds below 90 s (`OBS_MINUTE_SWITCH_SECONDS`, `v1_rt.rs:1631`) and to **whole minutes**
above it. The precise rows go to the trace ledger, which is armed only in
`src/v1/stage0/src/required_regen_host.rs:4143` and `:10059` (drained at `:4149`,
`:10686`) — **never on the compile route**, so the only numbers this program prints for a
stage are the rounded ones.

| phase (as printed) | run 1 printed | run 2 printed | producer of the printed name |
|---|---|---|---|
| closure collection — pool acquire | `phase=demand_layout_acquire ms=65361` | `ms=68163` | `cli_run/demand_acquire.rs:302` (`pre_entry_phase::timed`) |
| ↳ newline index | `phase=pool_source_newline_index ms=1909` | `ms=1938` | `cli_run/pool_acquire.rs:159` (`attribute`) |
| ↳ tokenize | `phase=pool_source_tokenize ms=38461` | `ms=39208` | `cli_run/pool_acquire.rs:178` |
| heads parse | `phase=pool_heads_parse ms=5083` | `ms=5780` | `cli_run/pool_acquire.rs:292` |
| census parse | `phase=pool_census_parse ms=7268` | `ms=8256` | `cli_run.rs:14269` (`pre_entry_phase::record`) |
| name census build | `phase=closure_name_census_build ms=3502` | `ms=3778` | `cli_run.rs:14534` |
| names from census | `phase=reference_pool_names_from_census ms=68` | `ms=69` | `cli_run/entry_resolve.rs:2984` |
| resolve + parse (front end) | `✓ compile.frontend done in 81 seconds` | `81 seconds` | `v1_compiler_compile.rs:3540` / `:3542` around `front_end_sources` |
| normalize | `✓ compile.normalize done in 3 seconds` | `4 seconds` | `v1_compiler_compile.rs:3565` around `v1_compiler_normalize::normalize_graph` (`:3560`) |
| **typing / inference (reconcile)** | `✓ compile.reconcile done in 19 minutes` | `22 minutes` | `v1_compiler_compile.rs:3574` around `v1_compiler_infer::reconcile_scoped` (called `:3567`-`:3572`, defined `v1_compiler_infer.rs:43521`) |
| analyses | `✓ compile.analyses done in 11 seconds` | `11 seconds` | `v1_compiler_compile.rs:3602` (complexity, ownership, test-reference, ambiguous-leaf) |
| emit | `✓ compile.emit done in 3 minutes` | `3 minutes` | `v1_compiler_compile.rs:3671` / `:3677` |
| ↳ dag collect | `✓ compile.emit.dag.collect done in 70 seconds` | `72 seconds` | `v1_compiler_compile.rs:2819` |
| ↳ dag serialize | `✓ compile.emit.dag.serialize done in 2 minutes` | `2 minutes` | `v1_compiler_compile.rs:2896` |
| build of the emitted crate | **UNATTRIBUTED — phase absent from this program** | **same** | no printed line exists: `--target dag` writes a serialized graph artifact, `out/dag-artifact.json` (631,140,172 bytes, one file, 0 `.rs`). `rustc` is invoked only on the rust target path (`cli_run.rs:9683`, `cli_run/emit_host.rs:3067`), which this argv never takes. |
| anything else inside the transaction | **UNATTRIBUTED** | **UNATTRIBUTED** | see "what I tried" below |

Transaction floors, printed as-is (instrument A):

```
run1: [pre-entry] phase=compile_transaction ms=1592173 scale=closure attribution=inclusive subject=dag/gunbc/roadmap/roadmap_ingress.dag
run1: [pre-entry] phase=compile_transaction_unattributed ms=1526812 scale=closure attribution=floor raw_ms=56293
run1: [pre-entry] phase=process_invocation_total ms=1596497 … boundary=main_entry_to_pre_exit_sampling
run1: [pre-entry] phase=process_unattributed ms=4324 …
run2: [pre-entry] phase=compile_transaction ms=1800614 …
run2: [pre-entry] phase=compile_transaction_unattributed ms=1732451 … raw_ms=59032
run2: [pre-entry] phase=process_invocation_total ms=1805034 …
run2: [pre-entry] phase=process_unattributed ms=4420 …
```

Reconciliation of the row's 2026-11-23 gap (1,798,764 of 1,871,790 ms "UNATTRIBUTED
inside `compile_transaction`"), stated as arithmetic on printed numbers only:

- run 1: attributed-by-A = `1592173 − 1526812 = 65361 ms` — exactly the single
  `attribution=inclusive parent=none` root row, `demand_layout_acquire`. run 2:
  `1800614 − 1732451 = 68163 ms` — again exactly `demand_layout_acquire`. The four other
  root rows printed in the same block are not counted into the span.
- Instrument A records **no row at all** for the compile pipeline, which is why ~96 % of
  the transaction reads "unattributed" there. That is an instrument gap, not an unknown
  cost: instrument B names that interval stage by stage (table above).
- Bounded residual: in run 1 the disjoint A-roots total 81,282 ms and the B-marks are in
  [1,415,000 ; 1,537,996] ms given the minute-flooring, against a 1,592,173 ms transaction;
  in run 2 the A-roots total 86,046 ms and the B-marks are in [1,596,000 ; 1,719,000] ms
  against 1,800,614 ms. So **≤ 119 s (≤ 6.6 %) of each transaction is named by no printed
  line** — the interval between the pre-entry work and the first `trace_mark`, plus the
  rounding floors. It is reported as UNATTRIBUTED, not assigned to a phase.

**What I tried for the UNATTRIBUTED cells, and what came back:**

- `GUNBC_FLOOR_PHASE_PROFILE=1` was set for both runs (its documented knob,
  `src/v1/stage0/src/phase_profile.rs:66`). Printed result:
  `[phase-profile] line count on this route = 0` in both logs. Reason, read from the
  pinned tree: `PhaseProfile::install_from_env` (`phase_profile.rs:65`) is called only at
  `src/v1/stage0/src/bin/claim_batch.rs:1134` and
  `src/v1/stage0/src/claim_executor.rs:408` — never in `main.rs` — so `set_phase`
  (`phase_profile.rs:265`) finds no installed profile and the two call sites that would
  name this interval (`cli_run/entry_resolve.rs:2057` `FloorPhase::Resolve`,
  `:2274` `FloorPhase::Typecheck`) print nothing. The same is true of
  `report_pre_entry_phases` (`main.rs:1289`), whose only call is `main.rs:1179` — the
  **run** route — so the `closure_load / closure_parse / closure_resolve /
  closure_normalize / closure_typecheck_compute / closure_parent_envs /
  closure_reconcile_assembly / closure_ownership` rows and
  `resolve_entry_graph_inclusive` never print for `gunbc compile`.
- Precise stage ms: `trace_mark` keeps them (`v1_rt.rs:1736-1742`), but
  `trace_ledger_arm` / `trace_ledger_drain` are never called on this route, so no
  `wall_ms` table is printed. Naming this interval in milliseconds would need a print
  call, which is a change, and this slice writes none.
- Per-module typing cost: the only per-module timing line is
  `✓ typecheck {module_path} done in …` (`cli_run.rs:12353`); printed hits in both runs'
  stderr: `0`. So "typing cost per module" cannot be attributed with a printed number on
  this route — it is UNATTRIBUTED, and the whole of it falls inside the single
  `compile.reconcile` line.

## Closure module count

Printed by the run, both runs, identically:

```
resolved 1679 sources (reference-derived closure entry=dag/gunbc/roadmap/roadmap_ingress.dag)
```

That is the **reference-derived closure**: 1679 sources. It is not what the compile's
resident set tracks — `gunbc.recurring_failure_mode::entry_scoped_typecheck_rss_tracks_the_name_census`
(`dag/gunbc/recurring_failure_mode/entry_scoped_typecheck_rss_tracks_the_name_census.dag`)
records that an `--entry` typecheck's RSS follows the **name census of every
`--source-root`**, built by `tree_bare_census_for_root` (`cli_run.rs:14554`), not the
reported closure; its own receipt quotes the CLI line in the form
`resolved N sources (reference-derived closure …), M indexed modules in the name census only`.
**In these two runs that `, M …` clause was not printed** — the line stops after the
parenthetical — so the census side has no printed counter here. The only census-side
number this document offers is an executed count, not a printed one: `10564` `.dag` files
under the two roots at the pinned revision (`find dag src/v2 -name '*.dag' -type f | wc -l`;
10,675 files of all kinds), against a measured peak of ~14.2 GiB — the shape that failure
mode describes. The row's 2026-11-23 note reported "closure about 3666 files" for its
compile form; this program at this revision prints 1679, so the two numbers are not the
same quantity and neither should be quoted as the other.

Same row, same two roots, same `--target dag`: its 2026-09-11 specimen was priced as
"Wall 184 s, of which compile.reconcile 60 s", which is why the stage marks above are the
accepted grain for this question.

## Pull 17006 — relay data, read NOT executed

`gh pr view 17006 --repo gunb-ai/gunbc` (read 2026-11-24): title *"WIP: serve-entry verdict
path is a per-run input threaded from its producer (STEP 2)"*, `state=OPEN`,
`isDraft=false` (the brief said draft), author `app/gunbai-bot`, `base=main`,
`mergeable=MERGEABLE`, `headRefOid=2b55b98988623a6e364540b34dee215186e88c22`, 11 files.
**Drift from the brief:** the head given (`da51bc69f0de0e5090a4d91d203b92e5cbd893e1`) is
the PR's *first* commit — still in the branch — and the branch has since advanced by
`2b55b98988 fix(fleet): plan serve-entry verdict path is a per-run input threaded from its producer`.
The branch was **not** checked out and **not** run.

Diff read from `gh pr diff 17006`: 11 files, **zero `.rs` files** — `dag/ops/ci/fleet_converge_workflow.dag`
(+78/−19), `dag/gunbc/live_deploy/pre_swap_admission.dag` (+49/−15),
`dag/gunbc/fleet/fleet_plan_artifact_location.dag` (+28/−1),
`dag/gunbc/live_deploy/swap_serve_entry_compile.dag`, `dag/gunbc/fleet/fleet_plan_receipt.dag`,
four `dag/test/claim/…` witnesses, and `.github/workflows/fleet-converge-pr-plan.yml`
(+12: three `GUNBC_FLEET_CONVERGE_RUN_ARTIFACT_*` env vars on four steps).

**Does it touch anything inside the interval attributed above? No.**

- Zero `.rs` changes: no `trace_mark` name, no `pre_entry_phase` record, no
  `reconcile_scoped`, no emit path can move.
- Token counts across the whole diff: `compile --entry` `0`, `observe_serve_entry_compile_clean`
  `0`, `source-root` `0`, `output-dir` `0`, `target dag` `0`, `closure` `0`,
  `minute|timeout|45` → only `timeout-minutes: 15`×3 and `timeout-minutes: 30` as unchanged
  context (the 45-minute step bound is untouched).
- What it *does* change is the **other** program: `record_plan_serve_entry_verdict_for` /
  `observe_plan_serve_entry_verdict` gain a `run_artifact_dir: NonEmptyStr` parameter,
  `swap_serve_entry_compile_for` reads the destination from the plan pack, and
  `gunbc.fleet.fleet_plan_artifact_location` gains `fleet_plan_run_artifact_dir(root,
  plan_run_id, attempt)` plus three env-name data rows and a new `NonEmptyStr` import.
  That new module body is one more `.dag` file's declarations under the `dag/` census
  root (+28 lines, a `join` reference and a `std.types` import), so it perturbs the census
  the reconcile leg pays for by one module — the closure both my runs walked
  (1679 sources; census over `dag` + `src/v2`) will not be byte-identical to the closure a
  post-merge run walks. This document is therefore the **before** for a post-17006
  re-measure, and the comparison must be made at a single named revision per side.

## Where the cost sits — conclusion

On the pinned revision, the whole-closure compile of the serve entry spends its time in
**one phase**: `compile.reconcile` — the `v1_compiler_infer::reconcile_scoped` typing and
inference pass over the closure — printed `19 minutes` of a 26 min 38 s run and
`22 minutes` of a 30 min 07 s run, i.e. **roughly 71–73 % of wall**, and the phase whose
variance explains the run-to-run gap. Everything the other named phases own is small
against it: closure collection ~81–86 s (of which tokenize ~38–39 s is the single biggest
internal line), front end 81 s, emit 3 min (serialize ~2 min for a 631 MB
`dag-artifact.json`), normalize 3–4 s, analyses 11 s. Peak RSS is flat across runs
(14,920,016 / 14,908,996 kB ≈ 14.2 GiB) while wall moves 13 %, consistent with the
resident-set story in
`gunbc.recurring_failure_mode::entry_scoped_typecheck_rss_tracks_the_name_census`: the
typecheck's cost tracks the whole-root name census, not the 1679-source reference-derived
closure the CLI reports as the subject.

The earliest unjustified boundary the reduction should be argued against is therefore the
cost the reconcile leg pays for modules that are not in the serve entry's reference-derived
closure — `gunbc.design_argument::earliest_unjustified_boundary`, read against
`gunbc.design_argument::single_authority` for who owns the scope decision. That argument
and the step-bound lowering belong to the reduction leg, not this one.

## Appendix — hit counts for every search a conclusion rests on

`git grep -c -F <pattern>` over `*.rs` and `*.dag` at `2bc71eb1679`:

| pattern | hits |
|---|---|
| `compile.frontend.begin` | 2 |
| `compile.normalize.done` | 3 |
| `compile.reconcile.done` | 3 |
| `compile.analyses.done` | 2 |
| `compile.emit.begin` | 2 |
| `compile.emit.dag.collect.done` | 3 |
| `compile.emit.dag.serialize.done` | 2 |
| `reconcile_scoped(` | 10 (definition: `v1_compiler_infer.rs:43521`) |
| `trace_ledger_arm` | 7 (call sites: `required_regen_host.rs:4143`, `:10059`) |
| `install_from_env` | 4 (call sites: `claim_batch.rs:1134`, `claim_executor.rs:408`) |
| `report_pre_entry_phases` | 2 (one def `main.rs:1289`, one call `main.rs:1179`) |
| `print_compile_transaction_receipt` | 3 (one call: `cli_run.rs:6907`) |
| `GUNBC_FLOOR_PHASE_PROFILE` | 5 |
| `tree_bare_census_for_root` | 13 |
| `pre_entry_phase::record(` / `pre_entry_phase::timed(` | 17 (call sites enumerated above) |

Printed-counter checks executed against each run's stderr: `[pre-entry]` rows
`grep -c "[pre-entry]"` → `11` lines each; `[phase-profile]` → `0` both runs;
`grep -c "typecheck .* done in"` → `0` both runs; `find <out> -type f` → `1`
(`dag-artifact.json`), `*.rs` → `0`, `*.dag` → `0`.

## Not claimed by this document

No reduction. No change to `serve_entry_verdict`'s step bound (45 minutes stays). No red
control, no `red_at` / `green_at`, no receipt for the
`record_plan_serve_entry_verdict_for` program. No acceptance request; recording is closed.
No host apply, no `/opt` write, no service touched.
