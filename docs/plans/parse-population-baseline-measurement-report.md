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

The selection rule, membership, per-member byte size, population rank and pinned tree sha have their **single home on the typed `.dag` carrier** `gunbc.parse_population_baseline_roster` (`dag/gunbc/parse_population_baseline_roster.dag`); its rostered claim `test.claim.parse_population_baseline_roster_witness` exercises the rule's self-consistency (count, ordering, uniqueness, partition) and the comparison predicate's discriminating arms. **The selection ALGORITHM is itself modelled and executed, not only commented** (review 98809 F1): `parse_population_selection_precedes` (byte size descending, then path ascending for the tiebreak) with its top-N window `parse_population_rows_outside_top_n` is checked by a discriminating pair (top-2 of `[30,20,10]` leaves exactly one outside; top-3 leaves none) and an equal-size path-tiebreak discriminator, so the shell `sort -k1,1nr -k2,2 | head` is transport applying a rule the carrier owns and tests, not the authority for it. Per `gunbc.design_argument::planning_has_one_home` this report holds no copy of the rule. The populations measured below were staged from the tree at the pinned sha (the instrument run's own module lists); they agree with the carrier's declared benchmark by construction — same pinned sha, same selection rule — and the **wall/RSS of every measured cell is timed by running the staged roots** (a row list cannot itself be timed). **The tie between the declared rows and the tree is NOT established in this merge.** A `.dag` function cannot read the tree, and CI executes no claims, so no `.dag` claim can do it; the shell runner that would (`run_evidence.sh`, which reads the pinned sha and rows out of the carrier and diffs them against `git ls-tree`) was **withdrawn from this PR** in the split ruled by warm-fox-517 (review 98836) — the `.dag` model, predicates and witness land here, the executed tree-reading consumer does not. Per `gunbc.design_argument::consumption` this is a named **gap**, not a claim of execution: the roster's 170-to-tree reconciliation is the open item that route A (bold-cat-668's measurement-first planning row, owner compiler-self-host) closes by reading the rows **through** `gunbc compile` rather than beside it. §4a records the author-side observation of the reconciliation and why it is not a committed check. Both roots (`dag/` and `src/v2/`) are required because Population 1 spans both (four `dag/` data files + one `src/v2/` roster).

The verb accepts only **governed tree directories** as `--source-root` (a copied dump at an arbitrary path is refused: `REFUSED: source root is not a governed tree directory`). The governed-tree predicate matches by directory name (`ends_with "/dag"`, `"/src/v2"`), so roots are staged under scratch directories named `…/dag` and `…/src/v2`, preserving relative paths. The baseline cells in §5 were produced by staging the pinned-sha tree under those governed dirs (the roster-driven staging runner that would build the roots from `roster.path` was withdrawn in the split; §9b); each staged file is byte-identical to the same path at the pinned sha:

```
NAMED_POP=pop5    files=5   bytes=4575463   sha_mismatches=0
NAMED_POP=pop165  files=165 bytes=29960631  sha_mismatches=0
```

### 4a. Carrier↔tree reconciliation — the gap this split does NOT close (route A pending; review 98443 / 98730 / 98809 / 98836)

The 170 roster rows are a hand-ordered list, so the load-bearing check is whether each declared `path` AND
byte size equals the tree at the pinned sha — a comparison against `git ls-tree`, not the roster read back
against itself. That comparison CANNOT live in `.dag` (no filesystem; CI executes no claims). The shell runner
that would have done it (`run_evidence.sh`) was **withdrawn from this PR** in the split: this merge ships the
`.dag` model, its predicates and its witness, and NOT an executed tree-reading consumer. The reconciliation is
therefore a **named gap**, closed by route A (below), not a claim of execution here.

What was observed author-side, on the pre-split runner, is recorded for continuity — it is an **author-side
verification, not a committed executed check**, and no run link is claimed for it in this merge:

```
pinned_sha 65230868362de3efc0ddfeeaeef5d83eeb2fa385
carrier-side rows = 170 ; tree-side (git ls-tree -r -l <sha> -- dag src/v2, top-170 by selection sort) = 170 ; diff rc 0
red control: mistyping the first declared row's byte size turns the leg nonzero and names roadmap_authority.dag
```

The `.dag` `parse_population_reconcile_*` arms are **not** that tree comparison — over hand-built reference row
sets they prove only that the comparison predicate DISCRIMINATES (`parse_population_reconcile_predicate_refuses_bad_row_holds`,
plus the by-name `parse_population_roster_red_control_mistyped_row_fails` in the same claim_batch, both **parses +
required lane** in CI). Reading the real tree and failing by name on the real 170 rows is the open item, not done here.

**Route A is the merge path.** `gunbc compile --target rust` over this carrier already emits all 170 rows as
typed Rust literals to `src/v1/stage0/src/gunbc_parse_population_baseline_roster.rs` (reproduced in §4b), so the
tree reconciliation can be driven from **compiler output** rather than a regex reader beside the grammar — no new
Rust, no seed. Whether the consumer should instead read the binding through a proper emitted route is exactly
the question bold-cat-668's measurement-first planning row (owner compiler-self-host) answers before anything
becomes a new emitted verb; anything reaching the seed goes to the operator first. **Route B (operator scaffold
approval outside the diff) is DECLINED** (neat-badger-614, 2026-11-26) and is not a path forward.

**The drop is rostered, and its guard can fail (review 98901 F1/F2).** Per `gunbc.design_argument::guarantee_ladder`
("Declared drops are rostered in `gunbc.rung_drop`") the gap is a row of the drop roster —
`gunbc.rung_drop.parse_population_carrier_reader_beside_grammar`
(`dag/gunbc/rung_drop/parse_population_carrier_reader_beside_grammar.dag`): previous rung 4 (structurally
guaranteed — the one grammar route reads a data binding), temporary rung 1 (mitigatable — typed predicates with
a discriminating pair over hand-built slices, no executable reader of the real 170 rows), reason
`ReplacementStaged` naming route A (`rn_VDAPYNAW2T3U26OZ5JUHRBVC6`, plan PR #17116), and a restoration trigger
that names **THE CAPABILITY restored** (an emitted declared route that evaluates a named data binding and
returns its rows typed — meta-obligation 3), not the planning row itself; the carrier holds no copy of the text
(`single_authority`). Its consumer renders the row into a ledger line, and the witness guard is a discriminating
pair, not a presence check (the earlier `!= ""` guard was green by construction and was deleted): GREEN — the
rendered line contains the row identity and the capability text; RED — the shared extraction applied to a
planted `Retired` row (a representable forbidden state: retired while the reconciliation is unexecuted, with a
non-empty trigger text) yields `""`, so an extraction that ignored the standing arm would be caught.

### 4b. Emitted-roster reproduction — how the compiler already evaluates the named binding

This is the measurement input route A needs, run read-only by the worker, not an implementation decision.

Command (free compile gate, no heavy slot; the emitted directory is named and removed after):

```
gunbc compile --entry dag/gunbc/parse_population_baseline_roster.dag \
  --source-root dag --source-root src/v2 --target rust \
  --output-dir .pp-emitted-roster-measurement
rc=0   (0 blocking, 768 advisory)
```

Emitted roster file, measured from disk (this turn's run; `bytes` is NOT a stable contract — the emitted header
carries a `generated_at` timestamp, so the file size drifts a few hundred bytes run to run; the load-bearing
observation is the row/path content, not the byte count):

```
src/v1/stage0/src/gunbc_parse_population_baseline_roster.rs
  bytes = 41550            (read from disk; drifts with generated_at; order 41 KB, not a bound value)
```

**189 path literals ≠ 170 rows — and the count that is correct is 170, not 189.** A file-wide grep for
`"[^"]*\.dag"` returns **189**, but the roster models exactly **170** rows. The other **19** are *not* roster
rows: they are the selection-rule **discriminator fixtures** the carrier also emits —
`probe/tie_a.dag`(×3), `probe/tie_b.dag`(×3), `probe/small.dag`(×2), `probe/middle.dag`(×2),
`probe/large.dag`(×2), `probe/planted_extra.dag`(×1) — plus the real paths the hand-built
`parse_population_mistyped_real_row_slice` / `parse_population_correct_real_row_slice` slices repeat for the
reconcile discriminator (`roadmap_authority.dag`, `roadmap_belt_actuate.dag`, `e1_receipt_markings.dag`, ×2 each).
**189 − 19 = 170.**

**The exact matcher a route-A transport MUST use (so it cannot ship a roster 19 short or long):** count the
`path` column **scoped to the body of the single function `pub fn parse_population_baseline_rows()`**, delimited
by the next `pub fn` — not the whole file. Measured against the emitted `.rs`:

```
rows in parse_population_baseline_rows() (path column, fn-scoped)  = 170   <- CORRECT
file-wide "[^"]*\.dag" literals                                     = 189   <- WRONG (includes discriminator fixtures)
```

The column is `path` and the scope is the roster function only. `parse_population_expected_total_count()` already
emits `170` as a typed literal, so the transport should compare its fn-scoped `path` count against that emitted
constant rather than against any file-wide count.

The emitted rows carry the exact path in a single column, so the exact-path match cleanly separates the correct
row from the substring decoy (this is the discriminating pair the transport must carry, verified against the
emitted `.rs`) — the same trap as the count: a naive substring match on the correct path also hits the decoy:

```
exact-match "dag/gunbc/roadmap/roadmap_authority.dag"      -> 3 correct rows
exact-match "dag/test/claim/roadmap/roadmap_authority.dag" -> 1 decoy row
substring  "roadmap/roadmap_authority.dag"                 -> 4 (both)  <- the trap; match the full path column, not a substring
```

The emitted roster `.rs` is **NOT committed at main** (main builds the roster from the `.dag` carrier). These
counts are an observation of what the compiler already produces; they are input to the planning row, not a
bound contract of this PR.

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

### 9b. Later-stage acquisitions and the review-98443 code change

- **SPLIT (review 98809 F1–F4 / 98836, ruled by warm-fox-517) — SUPERSEDES the "committed runner" bullets below.**
  This merge lands only the `.dag` side: `dag/gunbc/parse_population_baseline_roster.dag`,
  `dag/test/claim/parse_population_baseline_roster_witness.dag`, and this report. The two shell scaffolds
  (`test/parse_population_baseline_roster/run_evidence.sh`, `…/stage_roster_roots.sh`) and their `awk`/`grep`/`sed`
  carrier reader and hand reconciliation were **removed from the PR**. Consequence for the honesty seam: the
  carrier↔tree reconciliation and the roster-driven staging are **gaps pending route A**, not executed checks —
  every "executed runner / committed runner / green-by-execution run link" claim in the review-98443 and
  review-98730 bullets below is historical (those runners are no longer in the diff). The rung drop
  The drop is ROSTERED per `gunbc.design_argument::guarantee_ladder` at
  `gunbc.rung_drop.parse_population_carrier_reader_beside_grammar`
  (`dag/gunbc/rung_drop/parse_population_carrier_reader_beside_grammar.dag`, review 98901 F2 — the carrier
  holds no copy of its text), stating plainly that no executed reader of the rows ships here;
  its claimed rung is that the typed `.dag` predicates model and discriminate the selection order over hand-built
  slices. **Route B (operator scaffold sanction) is declined** (neat-badger-614); route A (bold-cat-668's
  measurement-first planning row) is the only path, and §4b shows `gunbc compile` already emits the 170 rows as
  typed Rust, so route A needs no new Rust. No heavy acquisition was used for the split.

- **Instrument-check leg (tranche 1)**: one further srv1 heavy-slot acquisition (`HEAVY_OWNER=bright-hawk-891`,
  `heavy-slot: slot 1/12 acquired bright-hawk-891 @2026-11-25T21:47:48Z`) ran `/usr/bin/time -v` (getrusage,
  not perf) over the smallest population (roots_pop5, 5 modules, 0 refusals) to prove the non-perf counters are
  present and nonzero: `utime=28.92 stime=0.61 maxrss=653284KB minflt=160128 nvcsw=2 nivcsw=212`, `majflt=0`
  (legitimate: Major "requiring I/O" faults on a page-cache-warm CPU run; the run faults via 160128 minor faults).
  So productive acquisitions = **10** (9 baseline sweep + 1 instrument-check leg); total touched = **14**. This
  figure is authoritative and agrees with the lane-manager status messages (an earlier "five" was an understatement).

- **Review 98443 code change — used no heavy acquisition** (free `gunbc compile --target rust` gate only,
  `0 blocking errors`). What changed on the carriers, and what it implies for the run link:
  1. Deleted the four transitively-dead declarations: `parse_population_baseline_report`,
     `parse_population_five_largest_count_check`, `parse_population_one_hundred_sixty_five_count_check`, and
     `parse_population_total_count` (its only callers were those three). No consumer remained (§4a's reconciliation
     leg, not a String report, is now the load-bearing surface).
  2. Replaced the planted `byte_size(count: 999)` red control with a **real-wrong-row** red control
     (`parse_population_roster_red_control_mistyped_row_fails`) and added the discriminating comparison-predicate
     arms (`parse_population_reconcile_predicate_refuses_bad_row_holds`); the real carrier↔tree comparison moved to
     the executed runner `test/parse_population_baseline_roster/run_evidence.sh` (review 98730).
  3. Stripped the load-bearing status/authorization/review-number prose out of the carrier `//` comments to
     one-line pointers (`gunbc.design_argument::source_annotations`).
  **The claim name-set therefore changed** (old executed set included `parse_population_roster_red_control_refuses`,
  which no longer exists; the new set adds `..._reconcile_predicate_refuses_bad_row_holds` and renames the red control). The
  executed run link recorded earlier (`c426cbf322`, 8 names) is **superseded** by this change and must be
  re-executed at the new head. Both the BOOT-bearing srv1 run of the §4a reconciliation leg and the re-executed
  rostered claim at the new head are **pending a lane-manager acquisition GO** (tranche 2 re-offer).

- **Review 98730 — route (i)/(a), used no heavy acquisition** (free `gunbc compile --target rust` gate + host-independent
  runner validation only). The rung-inflation finding (the `.dag` "reconciles to tree" test compares two hand-built
  slices, reads neither the tree nor the 170 rows; the only tree tie was awk/sed/diff committed inside this markdown)
  was fixed by making the real checks executed named runners and renaming the `.dag` claim to what it runs:
  - `test/parse_population_baseline_roster/run_evidence.sh` — the last execution of the roster↔tree path (§4a): reads
    the pinned sha + rows out of the carrier, diffs against `git ls-tree`, drives its own red (mistyped row must fail
    by name or exit 1). SCAFFOLD dissolve-on names both retirement triggers; the reason prose is about this case (a
    `.dag` function cannot read the tree; CI executes no claims), mirroring the accepted precedent
    `test/compiler_gate_contract_stamp/run_evidence.sh`.
  - `test/parse_population_baseline_roster/stage_roster_roots.sh` — the measurement-level consumer (§4b, route (a)):
    builds each population's verb roots FROM the roster, byte+path-checked at the pinned sha, refusing by name on a
    mismatch so a wrong row turns the VERB INPUT red.
  - `.dag` test renamed to `parse_population_reconcile_predicate_refuses_bad_row_holds`; carrier comments no longer
    claim a tree read.
  - Author-side validation: run_evidence.sh rc 0 (170 rows reconcile, injected mistype on `roadmap_authority.dag`
    turns it red); stage_roster_roots.sh `five` rc 0 (staged 5, bytes 4575463), `--selftest-mistype` rc 1 (4 of 5);
    carrier + witness compile `--target rust` rc 0, 0 blocking.
  - Base updated by merging `origin/main a58ba3e3` (a merge commit, not a force push); my four/five files are
    branch-only, so no content collision.
  - Tranche 2 (three acquisitions declared to the lane-manager, executed on srv1 post-cutover at the final head):
    A = staging-plus-verb (`stage_roster_roots.sh` then the parse-population verb over the roster-staged roots,
    both populations); B = the reconcile pair run-link on srv1 (green rc 0 + count; mistype rc 1 naming the path);
    C = the rostered claim re-exec by name (set now includes `parse_population_reconcile_predicate_refuses_bad_row_holds`,
    no longer the planted name).
