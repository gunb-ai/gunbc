# Parse-Population Baseline — Measurement Report

Scope: measured evidence for the `emission-real-consumer-profiles` row. Realization-vs-algorithm attribution is not establishable on srv1 under current host policy (§7) and is raised as a criteria question for the reviewer.

Row: `emission-real-consumer-profiles`.
Host: srv1 (`srv1.tailecbe08.ts.net`, aarch64, kernel 7.0.0-43-generic), heavy-slot evidence runs, `HEAVY_OWNER=bright-hawk-891`, one stage per acquisition.
Pinned sha: `65230868362de3efc0ddfeeaeef5d83eeb2fa385` (origin/main).
Companion carrier (single home for the populations): `dag/gunbc/parse_population_baseline_roster.dag`, executed by `dag/test/claim/parse_population_baseline_roster_witness.dag`.

## 1. Premise finding — the baseline did not exist; it had to be taken

`first_slice` claims a parse-population profile "already taken" over the five largest modules and a 165-module sample with "perf self time grouped by collection, allocation, provenance index, lexer and parser". Independent verification on origin/main found no such artifact:

| command | result | real rc |
|---|---|---|
| `git grep -inE "population.profile" origin/main` | 2 hits, both *references* to a profile (roadmap row text; `emit_state_threading_specimen.dag:7`), not one | 0 |
| `git grep -in "perf self time" origin/main` | 1 hit — this row's own `first_slice` text | 0 |
| `git grep -in "five largest" origin/main` | 0 hits | 1 |
| `git grep -in "5 largest" origin/main` | 1 hit — this row's own text | 0 |
| `git grep -inE "parse-population-baseline\|parse_population_baseline" origin/main` | 0 hits | 1 |
| `git ls-tree -r --name-only origin/main \| grep -i parse.population` | `parse_population_refusals.dag`, one claim witness, one grammar-gap test — none is a profile | 0 |

Conclusion: the row's premise is stale. The baseline was never taken; this report takes it.

## 2. Host instrumentation — what the row asked for vs what is permitted

`perf` exists at `/usr/bin/perf` on srv1, but `perf_event_paranoid=4` blocks every event class. Verified (read-only):

```
perf record -g -o /tmp/probe_perf.data -- sleep 1
  → Error: Failure to open event 'armv8_pmuv3_0/cycles/Pu' ... Access to performance monitoring and observability operations is limited.
  RECORD_EXIT=255
perf record -e cpu-clock -g -o /tmp/probe_cpu.data -- sleep 1
  → Error: Failure to open event 'cpu-clock:u' on PMU 'software' ... limited.
  RECORD_EXIT=255
```

No hardware *or* software event samples at paranoid 4. valgrind, strace, ltrace, gdb are not installed (installing any is a host change requiring the operator). Therefore **the row's `first_slice` "perf self time" is not obtainable on srv1 under current policy.** This measurement substitutes whole-process wall clock and peak RSS via GNU `/usr/bin/time`, and §7 states plainly why that substitute cannot carry the attribution the row wants.

## 3. Emitted artifact — proof it is emitted, not interpreted

Measured artifact (read from disk, not a build log):

```
path:    ~/tmp-bright-hawk-891/v2-native-cli-gen-one
sha256:  530e9847e475df1b5f304ee2d943c61e4fb9e615383a5dbbb4794162c487a150
size:    26927792 bytes
closure_identity: 103fa764af8fbcb69c9de0d531d892c436eff2105ba3bbb9ffbcd829afcab198
seed_sha256:      92b4459778ad4a3b4352d7aebf753042e77f6db923a6771871f2c7d10e078509
build time: 2026-11-24T16:50:56Z (slot 4 acquisition)
```

It is emitted, not an interpreted stand-in: `target/release/gunbc test //gunbc/instruments:v2-native-cli` translates the v2 `.dag` closure to Rust source and `cargo`/`rustc` builds that into a native ELF binary. Confirmed native: `readelf -S` shows `.symtab` with mangled Rust symbols; `objdump -d` shows real aarch64 prologues (`stp x29, x30, [sp, #N]`; `add x29, sp, #imm`). `perf -g` call graphs would work via frame pointers (AAPCS64 default) *if* sampling were permitted.

## 4. The two fixed populations (committed, byte-identity proven)

The selection rule, membership, per-member byte size, population rank and pinned tree sha have their **single home on the typed `.dag` carrier** `gunbc.parse_population_baseline_roster` (`dag/gunbc/parse_population_baseline_roster.dag`), executed by its rostered claim `test.claim.parse_population_baseline_roster_witness` — per `gunbc.design_argument::planning_has_one_home` this report holds no copy of the rule. The populations measured below were staged from the tree at the pinned sha (the instrument run's own module lists); they agree with the carrier's declared benchmark by construction — same pinned sha, same selection rule — but the **provenance of every measured cell is the instrument run, not a read of these rows**: the harness does not consume `roster.path`, and making it do so (or a carrier↔tree reconciliation instrument) is a declared follow-up with a bounded trigger, out of this slice. Per `gunbc.design_argument::planning_has_one_home` this report holds no copy of the rule; per `gunbc.design_argument::consumption` **no §5/§6 number is roster-derived** — the roster is a declared benchmark, not a measurement input. Both roots (`dag/` and `src/v2/`) are required because Population 1 spans both (four `dag/` data files + one `src/v2/` roster).

The verb accepts only **governed tree directories** as `--source-root` (a copied dump at an arbitrary path is refused: `REFUSED: source root is not a governed tree directory`). The governed-tree predicate matches by directory name (`ends_with "/dag"`, `"/src/v2"`), so each population was staged under scratch directories named `…/dag` and `…/src/v2`, preserving relative paths. Each staged file is byte-identical to the same path at the pinned sha:

```
NAMED_POP=pop5    files=5   bytes=4575463   sha_mismatches=0
NAMED_POP=pop165  files=165 bytes=29960631  sha_mismatches=0
```

## 5. Cells — raw output, real rc, load alongside every cell

`rc` is taken from the command directly (`RC=$?` immediately after `/usr/bin/time … "$BIN" "$@"`), never through a pipe. The verb counts one extra module per population (166 for the staged 165, 10567 for 10562) — a stable +~5 offset from the enumeration, reported as the verb prints it.

Machine: srv1, 48–50 concurrent sessions, load average 15–21 during the sweep (other lanes running `claim_batch` over the floor and a `--regen-round-cost` converge; slots 2,3,6,7,11 held by four other owners).

| cell | population | runs | wall s (min / median / max) | peak RSS KB (median) | modules | declared refusals | load avg (at run) | real rc |
|---|---|---|---|---|---|---|---|---|
| REF | full `dag`+`src/v2` roots | 1 | 563.94 | 657660 | 10567 | 0 | 20.56, 21.24, 23.08 (48 users) | 0 |
| A | Five Largest (5) | 3 | 20.16 / 22.79 / 23.30 | 653412 | 5 | 0 | 18.3–21.0 (48–50 users) | 0 |
| B | 165-Sample | 3 | 132.19 / 132.51 / 143.84 | 491264 | 166 | 0 | 15.30–21.57 (48–49 users) | 0 |
| C | 165-Sample verdict-only (`--reuse-from`) | 1 | 136.66 | 488408 | 166 | reused=0 | 16.64, 19.96, 21.13 (50 users) | 0 |

Refusal comparability: the reference run over the full roots reports **0 declared refusals**, and cells A and B each report **0 refusals** for their modules — equal to the reference baseline, so both cells measure the same program and are publishable.

Verdict-only control (cell C) printed:
```
parse-verdict reuse: ancestor=none reused=0 miss_identity_absent=166 miss_identity_mismatch=0 miss_entry_absent=0 miss_digest_unseen=0 miss_declared_refused=0
```

## 6. What the whole-invocation figures do show

- A large **fixed process startup** dominates small inputs: 5 modules cost 20–23 s wall, ~the same order as a large fraction of the 165-module run. `PopulationReport{text,held,detail}` carries no per-phase counters, so this fixed cost cannot be split into grammar acquisition / closure load / enumerate / lex / parse from inside the verb. The wall figure is **process-level**, not parse cost.
- **Per-byte** wall is similar across the two fixed populations (pop5 ≈ 22.79s/4.58MB = 4.98 s/MB; pop165 ≈ 132.51s/29.96MB = 4.42 s/MB). Per the row's own rule this is a **scaling warning, not a confirmed scaling finding** — the two populations have different file-size mixes and both include the same fixed startup; a controlled size series of one source shape would be needed to confirm any per-byte claim, and whole-invocation timing cannot deliver that series. **Additionally**, the 165 population is a deliberate **size-tail** sample (next-165-largest by blob size), not a representative corpus sample — declared in the `gunbc.parse_population_baseline_roster` header comment (review 98282/98328; annotation only, the substantive size-descending selection is enforced by `parse_population_check_rows_ordered_descending`, the 170-size by the cardinality probes) (`gunbc.design_argument::epistemic_humility`). Both populations sit in the heavy file-size tail, so this comparison is tail-vs-tail and asserts nothing about the whole-corpus size mix.
- peak RSS ~0.49 GB (pop165) to ~0.65 GB (pop5, full roots) — allocator-resident peak, also whole-process.

## 7. Attribution findings — the honest negative

The row's `boundary` asks each hot spot be attributed to a building-block/composite contract it violates, **or to the algorithm itself** (realization vs algorithm). On srv1 under current policy this attribution **cannot be produced**, for two independent reasons, stated plainly so the numbers are not read as the ones the ticket asked for:

1. **The named instrument is unavailable.** `first_slice` names "perf self time"; perf cannot sample at `perf_event_paranoid=4` (§2), and valgrind/strace/gdb are absent. A self-time histogram by collection/allocation/provenance/lexer/parser — the split the row wants — requires one of those and was not produced.

2. **Whole-invocation wall/RSS cannot carry per-operation attribution.** The ground for this limit is observable, not borrowed: the emitted verb's report type `PopulationReport { text: String, held: Bool, detail: String }` in module `v2.cli.population_verbs` (`src/v2/cli/population_verbs.dag`) carries no per-phase counters — there is nothing in the verb's output to decompose a single wall/RSS figure against, so wall/RSS stays a whole-process figure. If the reviewer relies on a standing ruling here, it is not in `gunbc.design_ruling` (grep for it returns no file) and would have to be recorded there to be citable; this report does not rest on any ruling it cannot name a carrier for. Cell C confirms the row's intended realization control is **not establishable locally**: `--reuse-from` (and `--receipts`/`--write-identity`) require `--compiler`, and reuse consumes a CI **published-ancestor** restore (ancestor/identity/receipts keyed by ancestor sha + compiler) that has no local write equivalent. With no such cache, `reused=0`, `miss_identity_absent=166`, and verdict-only wall (136.66 s) equals full-artifact median (132.51 s) within run-to-run noise — i.e. the "verdict-only speedup cannot hide unchanged map construction" control cannot separate map construction from verdict computation on this host.

   **Control-as-designed result — read this before spending a slot on it.** The verdict-only control **could not answer the question it was designed to answer on this workload, and this is a result, not a failure of execution.** Its purpose (from the row's `boundary`: a verdict-only speedup must not hide unchanged map construction) requires the reuse path to actually hit (`reused > 0`), so that a verdict-only run reuses the map and any residual cost is verdict computation. On srv1 reuse cannot hit without a CI **published-ancestor** cache, so cell C re-measured full construction (`reused=0`, `miss_identity_absent=166`, 136.66 s) against the full-artifact median (132.51 s) — the gap is run-to-run noise. The control is therefore **inconclusive by construction on this host**, not under-powered: repeating C with more samples will NOT separate the two, because the reuse path that carries the signal is absent. Do **not** spend an acquisition repeating C until either the per-phase-counters row (`v2-verb-reports-per-phase-counters`, pull 17071) lets the verb decompose its own cost, or a perf window (operator decision) makes sampling available — those change what the verb can report; more repeats of the whole-process figure do not.

**Seed-vs-emitted is not a usable realization cell either.** The v1 seed binary has no `parse-population` subcommand (`unrecognized subcommand`, exit 2); a seed cell is a missing capability, not a cost. The realization split the verb *can* express is `--reuse-from` vs reuse-OFF, which §7(2) shows is not establishable here.

**Therefore:** the baseline is taken (fixed populations, byte-identity proven, refusal-comparable, wall/RSS min+median with load), but the realization-vs-algorithm attribution the `boundary` requires is a criteria decision for the reviewer. It needs one of: (a) an operator-approved `perf_event_paranoid ≤ 2` window on srv1 for true self-time sampling with frame-pointer call graphs, or (b) per-phase counters emitted by the parse-population verb (a carrier change), or (c) acceptance of a CI published-ancestor receipts cache to make the verdict-only control live. None of (a)/(b)/(c) is in this row's cleared scope.

## 8. Reproducibility observation (relay)

While building the measured artifact I observed that two emit builds of the same closure produced different files with an identical `.text` section. That section-by-section observation is relay data for the reproducibility row and is carried there, not duplicated in this report. The artifact this report measured is identified in §3.

## 9. Acquisition count actually used

Productive heavy-slot acquisitions on srv1 (each its own slot, `HEAVY_OWNER=bright-hawk-891`):
1 build + 1 REF + 3 cell-A + 3 cell-B + 1 cell-C = **9**.
Additionally 4 heavy-slot touches refused instantly at argument validation (wall 0.00 s, no work: reference missing the verb token; reference missing `cd` to the checkout; `--receipts`/`--write-identity` and `--reuse-from` missing the required `--compiler`). Total slots touched = 13. This exceeds the lane-manager's planning estimate of 5; the excess is the 3-repeats-per-cell requirement and discovering that the reuse flags require `--compiler`. Every acquisition logged `heavy-slot: slot N/12 acquired by bright-hawk-891 at <ts>` as its first line and printed `uptime`/`nproc`/live slot holders alongside the cell.
