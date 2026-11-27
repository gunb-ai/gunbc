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

The selection rule, membership, per-member byte size, population rank and pinned tree sha have their **single home on the typed `.dag` carrier** `gunbc.parse_population_baseline_roster` (`dag/gunbc/parse_population_baseline_roster.dag`); its rostered claim `test.claim.parse_population_baseline_roster_witness` exercises the rule's self-consistency (count, ordering, uniqueness, partition) and the comparison predicate's discriminating arms. **The selection ALGORITHM is itself modelled and executed, not only commented** (review 98809 F1): `parse_population_selection_precedes` (byte size descending, then path ascending for the tiebreak) with its top-N window `parse_population_rows_outside_top_n` is checked by a discriminating pair (top-2 of `[30,20,10]` leaves exactly one outside; top-3 leaves none) and an equal-size path-tiebreak discriminator, so any shell application of it (the `sort -k1,1nr -k2,2 | head` runner) was transport only, and that runner was withdrawn from this PR (review 98836, route B declined) — the carrier is and remains the sole authority. Per `gunbc.design_argument::planning_has_one_home` this report holds no copy of the rule. The populations measured below were staged from the tree at the pinned sha (the instrument run's own module lists); they agree with the carrier's declared benchmark by construction — same pinned sha, same selection rule — and the **wall/RSS of every measured cell is timed by running the staged roots** (a row list cannot itself be timed). **The tie between the declared rows and the tree is NOT established in this merge.** A `.dag` function cannot read the tree, and CI executes no claims, so no `.dag` claim can do it; the shell runner that would (`run_evidence.sh`, which reads the pinned sha and rows out of the carrier and diffs them against `git ls-tree`) was **withdrawn from this PR** in the split ruled by warm-fox-517 (review 98836) — the `.dag` model, predicates and witness land here, the executed tree-reading consumer does not. Per `gunbc.design_argument::consumption` this is a named **gap**, not a claim of execution: the roster's 170-to-tree reconciliation is the open item that route A (bold-cat-668's measurement-first planning row, owner compiler-self-host) closes by reading the rows **through** `gunbc compile` rather than beside it. §4a records the author-side observation of the reconciliation and why it is not a committed check. Both roots (`dag/` and `src/v2/`) are required because Population 1 spans both (four `dag/` data files + one `src/v2/` roster).

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
sets they prove only that the comparison predicate DISCRIMINATES — `parse_population_reconcile_predicate_refuses_bad_row_holds`
and, over a real carrier-shaped row with one byte size mistyped, `parse_population_cardinality_refuses_mistyped_real_row_holds`
(warm-fox F5: the earlier red control was a plain `fn` with zero call sites that no claim route could ever execute,
and its assertion was inverted for a refusal leg — it asserted the unreconciled count was `0`, which is false by
construction, so as a leg it could only ever FAIL. It is now a `test fn` asserting the count is NOT zero, so every
leg in the batch HELDs and the negative is carried by an assertion inside a held leg over a named planted input).
Current leg set: 14 `test fn`s, zero plain-fn legs. Status for all of them: **parses only** — the module
`test.claim.parse_population_baseline_roster_witness` has NO row in
`v2.workflow.required_floor::required_gate_authored_modules` (verified by grep over that file: zero hits), so
no lane executes it and none of these legs is gated; an operator ruling would be needed to enrol the module.
Execution on srv1 through the limiter is the open item. Reading the real tree and failing by name on the real 170 rows is
likewise open, not done here.

**Route A is the merge path.** `gunbc compile --target rust` over this carrier already emits all 170 rows as
typed Rust literals to `src/v1/stage0/src/gunbc_parse_population_baseline_roster.rs` (reproduced in §4b), so the
tree reconciliation can be driven from **compiler output** rather than a regex reader beside the grammar — no new
Rust, no seed. Whether the consumer should instead read the binding through a proper emitted route is exactly
the question bold-cat-668's measurement-first planning row (owner compiler-self-host) answers before anything
becomes a new emitted verb; anything reaching the seed goes to the operator first. **Route B (operator scaffold
approval outside the diff) is DECLINED** (neat-badger-614, 2026-11-26) and is not a path forward.


**The gap is a rostered STALL, and its guard can fail.** The authoritative status is the carrier row
`gunbc.guarantee_stall.parse_population_carrier_reader_beside_grammar_stall`
(`dag/gunbc/guarantee_stall/parse_population_carrier_reader_beside_grammar_stall.dag`) — current rung, ceiling,
blocker, the next-rung trigger naming THE CAPABILITY, and the awaiting route-A referent are all read there,
not restated here (a second copy of stall status in prose is a fork; `source_annotations`). What this report
keeps is only why the row exists in that form: it was first filed as a rung DROP whose `previous` named a rung
that was **never held**, which the ladder forbids as rung inflation in reverse — the honest record is a
below-ceiling class with a named trigger ("no untracked stall"). The row is consumed through the carrier's
typed capability accessor (`parse_population_carrier_reader_beside_grammar_stall_capability`) plus the
discriminating pair in
`dag/test/claim/parse_population_baseline_roster_witness.dag`; the history of how the guard reached that form
(the green-by-construction `!= ""` guard deleted; the first red control replaced because it failed under both
worlds) is event record — it lives in the PR #17037 review thread, not in this file.

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

**Typed home (review 98937 F4).** These cells are projections; their one typed home is
`parse_population_measurement_receipts` in `dag/gunbc/parse_population_baseline_roster.dag` (one
`ParsePopulationMeasurementReceipt` per cell: printed counters, real rc, binary sha256, tree sha, host,
boot id), consumed by the witness legs `parse_population_measurement_receipts_record_the_sweep_holds`
and `parse_population_receipt_publishability_refuses_planted_failed_cell_holds` (parses only — the module is
not enrolled on the required floor; execution pending A/B/C). `planning_has_one_home` bars this file from being a **plan** home — the plan
(roadmap row, stall, route-A referent) lives in `.dag` carriers; what sits here is a measurement
**transcript** — raw runner output no `.dag` route can produce — and per `source_annotations` the receipt
FACTS now live typed, with this section rendering them. Retyped per review 98981: peak RSS is a
`ByteSize` (KiB x 1024), every duration a `std.measure` `Microsecond` (printed seconds x 1,000,000, exact
at the printed 2 decimals), and the two timing kinds are sealed variants — `WallStat {min_wall, median_wall,
max_wall}` for the repeated runs, `ProcessTimes {utime, stime, minor_faults, major_faults,
voluntary_ctx_switches, involuntary_ctx_switches}` for the getrusage leg — so cell E's RSS fact is stated
once, and the witness adds a second refusal leg: a planted reversed `min > max` triple is refused by
`parse_population_receipt_durations_are_ordered` (a state prose in a String could not express).

`rc` is taken from the command directly (`RC=$?` immediately after `/usr/bin/time … "$BIN" "$@"`), never through a pipe. The verb counts a few more modules per population than the staged file count — a stable offset from the enumeration; the per-cell counts are the `runs`/`modules` fields of the binding rows, not repeated here.

Machine: srv1, 48–50 concurrent sessions, load average 15–21 during the sweep (other lanes running `claim_batch` over the floor and a `--regen-round-cost` converge; slots 2,3,6,7,11 held by four other owners).

**No cell table here (review 99002).** Cell values — runs, wall statistics, peak RSS, module counts,
declared refusals, rc, artifact shas, host and boot — have exactly one home: the typed binding
`parse_population_measurement_receipts` (`gunbc.parse_population_baseline_roster`). A hand-built table here
was a second, independently-drifting copy (it kept KiB where the binding keeps `ByteSize`); it is deleted
rather than corrected. What this section keeps is only what no typed carrier holds: the run environment and
the verbatim prints below.

Refusal comparability: every cell reports the same declared-refusal count as the reference run (the
values are in the typed receipts; the publishability witness leg asserts exactly this), so all cells measure
the same program and are publishable. Load averages at run time were recorded in the run logs behind the
acquisitions and are deliberately not re-typed here — no claim consumes them.

Verdict-only control (cell C) printed:
```
parse-verdict reuse: ancestor=none reused=0 miss_identity_absent=166 miss_identity_mismatch=0 miss_entry_absent=0 miss_digest_unseen=0 miss_declared_refused=0
```

## 6. What the whole-invocation figures do show

- A large **fixed process startup** dominates small inputs: the A (five-largest) wall receipts in
  `gunbc.parse_population_baseline_roster::parse_population_measurement_receipts` are the same order as the B
  (165-module) receipts in the same binding — every value lives once in the typed receipts and is not retyped
  here (`single_authority`). `PopulationReport{text,held,detail}` carries no per-phase counters, so this fixed
  cost cannot be split into grammar acquisition / closure load / enumerate / lex / parse from inside the verb.
  The wall figure is **process-level**, not parse cost.
- **Per-byte** wall: the division of each population's median wall receipt by its declared byte total
  (`parse_population_five_largest_bytes` / `parse_population_one_hundred_sixty_five_bytes` in the same carrier)
  shows no order-of-magnitude split between the two fixed populations. Both operands are binding values, so
  this paragraph adds no measurement of its own and quotes no figures. Per the row's own rule this stays a
  **scaling warning, not a confirmed scaling finding** — the two populations have different file-size mixes
  and both include the same fixed startup; a controlled size series of one source shape would be needed to
  confirm any per-byte claim, and whole-invocation timing cannot deliver that series. **Additionally**, the 165
  population is a deliberate **size-tail** sample (next-165-largest by blob size), not a representative corpus
  sample — declared in the `gunbc.parse_population_baseline_roster` header comment (review 98282/98328;
  annotation only, the substantive size-descending selection is enforced by
  `parse_population_check_rows_ordered_descending`, the 170-size by the cardinality probes)
  (`gunbc.design_argument::epistemic_humility`). Both populations sit in the heavy file-size tail, so this
  comparison is tail-vs-tail and asserts nothing about the whole-corpus size mix.
- Peak RSS: the receipts' `peak_rss` `ByteSize` fields put both fixed populations in the same half-gigabyte
  band, with the 5-module full-roots cell the higher of the pair — allocator-resident peak, whole-process
  (values in the binding, not retyped).

## 7. Attribution findings — the honest negative

The row's `boundary` asks each hot spot be attributed to a building-block/composite contract it violates, **or to the algorithm itself** (realization vs algorithm). On srv1 under current policy this attribution **cannot be produced**, for two independent reasons, stated plainly so the numbers are not read as the ones the ticket asked for:

1. **The named instrument is unavailable.** `first_slice` names "perf self time"; perf cannot sample at `perf_event_paranoid=4` (§2), and valgrind/strace/gdb are absent. A self-time histogram by collection/allocation/provenance/lexer/parser — the split the row wants — requires one of those and was not produced.

2. **Whole-invocation wall/RSS cannot carry per-operation attribution.** The ground for this limit is observable, not borrowed: the emitted verb's report type `PopulationReport { text: String, held: Bool, detail: String }` in module `v2.cli.population_verbs` (`src/v2/cli/population_verbs.dag`) carries no per-phase counters — there is nothing in the verb's output to decompose a single wall/RSS figure against, so wall/RSS stays a whole-process figure. If the reviewer relies on a standing ruling here, it is not in `gunbc.design_ruling` (grep for it returns no file) and would have to be recorded there to be citable; this report does not rest on any ruling it cannot name a carrier for. Cell C confirms the row's intended realization control is **not establishable locally**: `--reuse-from` (and `--receipts`/`--write-identity`) require `--compiler`, and reuse consumes a CI **published-ancestor** restore (ancestor/identity/receipts keyed by ancestor sha + compiler) that has no local write equivalent. With no such cache, the reuse print is the one in §5, and the cell-C wall receipt in the binding is statistically indistinguishable from the B (full-artifact) median receipt — i.e. the "verdict-only speedup cannot hide unchanged map construction" control cannot separate map construction from verdict computation on this host.

   **Control-as-designed result — read this before spending a slot on it.** The verdict-only control **could not answer the question it was designed to answer on this workload, and this is a result, not a failure of execution.** Its purpose (from the row's `boundary`: a verdict-only speedup must not hide unchanged map construction) requires the reuse path to actually hit (`reused > 0`), so that a verdict-only run reuses the map and any residual cost is verdict computation. On srv1 reuse cannot hit without a CI **published-ancestor** cache, so cell C re-measured full construction (the §5 reuse print; its wall receipt in the binding) against the B (full-artifact) median receipt in the same binding — the gap is run-to-run noise. The control is therefore **inconclusive by construction on this host**, not under-powered: repeating C with more samples will NOT separate the two, because the reuse path that carries the signal is absent. Do **not** spend an acquisition repeating C until either the per-phase-counters row (`v2-verb-reports-per-phase-counters`, pull 17071) lets the verb decompose its own cost, or a perf window (operator decision) makes sampling available — those change what the verb can report; more repeats of the whole-process figure do not.

**Seed-vs-emitted is not a usable realization cell either.** The v1 seed binary has no `parse-population` subcommand (`unrecognized subcommand`, exit 2); a seed cell is a missing capability, not a cost. The realization split the verb *can* express is `--reuse-from` vs reuse-OFF, which §7(2) shows is not establishable here.

**Therefore:** the baseline is taken (fixed populations, byte-identity proven, refusal-comparable, wall/RSS min+median with load), but the realization-vs-algorithm attribution the `boundary` requires is a criteria decision for the reviewer. It needs one of: (a) an operator-approved `perf_event_paranoid ≤ 2` window on srv1 for true self-time sampling with frame-pointer call graphs, or (b) per-phase counters emitted by the parse-population verb (a carrier change), or (c) acceptance of a CI published-ancestor receipts cache to make the verdict-only control live. None of (a)/(b)/(c) is in this row's cleared scope.

## 8. Reproducibility observation (relay)

While building the measured artifact I observed that two emit builds of the same closure produced different files with an identical `.text` section. That section-by-section observation is relay data for the reproducibility row and is carried there, not duplicated in this report. The artifact this report measured is identified in §3.

## 9. What this merge lands, and where the record lives (review 99002 trim)

This merge lands exactly four files: `dag/gunbc/parse_population_baseline_roster.dag` (the carrier: the
170-row pin, the selection rule, and the typed measurement receipts),
`dag/gunbc/guarantee_stall/parse_population_carrier_reader_beside_grammar_stall.dag` (the stall row — the
authoritative status of the unexecuted tree tie),
`dag/test/claim/parse_population_baseline_roster_witness.dag` (the rostered claims, all `test fn`, with the
discriminating pairs), and this report. The shell scaffolds (`run_evidence.sh`, `stage_roster_roots.sh`) were
withdrawn (review 98836; route B declined) and ship in no merge.

Everything an earlier draft of this file restated by hand now has one home, and this section is a pointer
index rather than a copy (`single_authority`, `source_annotations`, `how_to_work`'s "hand-authored projection
the model should generate"):

- **Cell values** (runs, wall/RSS/module/refusal/rc counters, artifact sha, tree sha, host, boot): the typed
  binding `parse_population_measurement_receipts`; the witness publishability legs consume it.
- **Acquisition counts and slot accounting** (productive runs, instant-refused touches, the instrument-check
  leg): derived from the receipts (`runs` per cell plus the E leg) and the heavy-slot log lines quoted in the
  PR #17037 evidence comments — event record, not report prose.
- **Stall status and rungs**: the stall row (see §4a); the ladder renders it, this report does not.
- **Review and guard history** (which controls were deleted, renamed, or re-polarized at which review): the
  PR #17037 review thread — events belong to the PR, not to a projection.
- **Execution status of the claims**: honest tier labels sit on each claim in the witness and in the
  acceptance request; "parses + required lane" vs "green by execution in run <link>" per the work-item
  lifecycle. The srv1 re-execution (A/B/C) is the open acquisition, held by the lane manager until the head
  is final.

What remains in this file is deliberately only what no `.dag` carrier can hold: the premise finding (§1), the
host-capability readings with their real exit codes (§2), the emittedness proof (§3), the staging/byte-identity
procedure (§4), the verbatim runner prints (§5), the attribution reasoning and its honest negative (§6–§7), and
the relay observation (§8).
