# Parse-Population Baseline — Measurement Report

Scope: measured evidence for the `emission-real-consumer-profiles` row. The attribution element is discharged in **counter units** per the reviewer's ruling (§7, §7a) — labelled counter attribution, not time sampling; no time-share claim is made anywhere in this file.

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

No hardware *or* software event samples at paranoid 4. valgrind, strace, ltrace, gdb are not installed (installing any is a host change requiring the operator). Therefore **the row's `first_slice` "perf self time" is not obtainable on srv1 under current policy.** This measurement substitutes whole-process wall clock and peak RSS via GNU `/usr/bin/time`, and §7 states plainly why that substitute cannot carry the attribution the row wants. The perf-sample route was subsequently **retired for this node** by the reviewer's ruling (neat-badger-614, relayed by warm-fox-517 2026-11-27T19:5xZ): route (b), per-phase counters, discharges the attribution element in counter units (§7a), so nothing here waits on a host change. The §2 capability measurements stand as recorded evidence of why sampling was not used.

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
Leg set (count stated BY METHOD, not as a copied literal — a bare count drifted here once): `grep -c '^test fn ' dag/test/claim/parse_population_baseline_roster_witness.dag` outputs `19`; zero plain-fn legs. Execution status: **green by execution** for all 19 legs in run `pp_batch2` (srv1, BOOT c095c0cd, heavy-slot 11/12 acquired by bright-hawk-891 at 2026-11-27T11:15:06Z, instrument sha256 5aafd4fb6bc9e92bc4722c8fd85407351f9f7f235fc269f84d2afad90a86420e built-from-head 0d77469ccb92dc56de9b79c93b608adda4ab804b, claim route, one process, 19 operands one-per-operand: `19 claim(s): 19 held, 0 did not hold, 0 unobserved`; acquisition log /home/briansrls/tmp-bright-hawk-891/pp_batch2.log). The module still has NO row in
`v2.workflow.required_floor::required_gate_authored_modules` (verified by grep over that file: zero hits), so
no lane executes it and none of these legs is gated; an operator ruling would be needed to enrol the module.
Reading the real tree and failing by name on the real 170 rows is
likewise open, not done here.

### 5a. Red controls on the witness (claimed-verdict falsifiability)

Both controls follow the same shape: one acquisition each, a baseline leg over the unmutated tree and
a mutated leg, both in-slot as separate logs, each rc on its own line. The verdict is **derived** as the
set difference of per-label verdict maps built from the runner's own per-label lines (19/19 labels
required on both sides, baseline all-HELD enforced, empty label symmetric difference), compared to a
PRE-NAMED expected set by exact equality — a larger or smaller set is a finding, never absorbed, and no
token grep decides anything. The file swap is a printed `sed -i` with line-numbered read-back, with the
mutated file's sha256 required to differ from the committed sha256 and `git diff --name-only` printed at
the moment the mutated batch is invoked; restore is `git checkout --` with sha256-after == sha256-before
and porcelain back to zero lines. Instrument identical to the `pp_batch2` batch; the runner's own
freshness line renders `inputs=268` beside `inputs=268` on every leg, so the provenance claim
(binary built-from-0d77469ccb92, executed at b0fff7396d2, `dag/` byte-identical between them — full delta
one `.md` file, printed in-log) is backed by a number rather than prose.

* **Mutation A** — plant `planted` into the stall row's `capability` string
  (`dag/gunbc/guarantee_stall/parse_population_carrier_reader_beside_grammar_stall.dag`, line 35).
  Pre-named expected flip set: {`parse_population_stall_trigger_capability_declared_holds`}.
  Observed flip set: equal (baseline `19 held, 0 did not hold, 0 unobserved` rc 0; mutated
  `18 held, 1 did not hold, 0 unobserved` rc 1). The sibling
  `parse_population_stall_capability_extraction_distinguishes_planted_values_holds` stayed HELD on both
  sides — the flip is specific to the leg that reads the module's capability, which is the claim under
  test. `VERDICT_MUTATION_A=DID_NOT_HOLD_FIRED_SPECIFICALLY` (run `pp_mutA3`, acquisition
  2026-11-27T13:34:36Z, srv1 BOOT c095c0cd, logs `/home/briansrls/tmp-bright-hawk-891/pp_mutA3.log`
  and `mutA_out3/`; superseded attempts with broken observers retained as
  `pp_mutA.log.attempt1_unexpected` / `pp_mutA.log.attempt2_grepverdict`, never counted).
* **Mutation B** — reverse the roster's CellA `WallStat` triple in the REAL receipt data
  (`dag/gunbc/parse_population_baseline_roster.dag`, three `sed` expressions: min 20160000 → 23300000,
  max 23300000 → 20160000, so `min_wall > max_wall`; sha256 before 2bc74aef…, during a6e0843e…, after
  2bc74aef…). Pre-named expected flip set: {`parse_population_measurement_receipts_record_the_sweep_holds`}
  — the only leg feeding real receipts through `parse_population_receipt_durations_are_ordered`.
  Observed flip set: equal (rc 0 / rc 1, same summary shape). `VERDICT_MUTATION_B=DID_NOT_HOLD_FIRED_SPECIFICALLY`
  (run `pp_mutB1`, acquisition 2026-11-27T14:20:14Z after six correct STAND_DOWNs including one at
  load1=50.54, logs `pp_mutB1.log` and `mutB_out/`). The immunity of the two named negatives was
  **printed, not asserted**: the reversed-triple and failed-cell legs consume their own planted receipts
  (`parse_population_planted_reversed_stat_receipt`, `parse_population_planted_failed_receipt`, printed
  from disk with their values beside the mutated roster lines), and the render legs match
  `min_wall/median_wall/max_wall` with `_` patterns, so none of them can see the swap — and none did.

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

### 4c. Literal-count reconciliation — 191 = 170 + 20 + 1 (review follow-up, method stated)

A reviewer count found **191** `ParsePopulationRow` literals in the carrier where the report claims **170** fixed rows. The reconciliation, produced by `awk` over the **enclosing binding** (attribute each literal to the nearest preceding top-level `data`/`type`/`fn`/`let` header line, not by a substring count — a substring count also catches the type declaration and every `ParsePopulationRow` annotation site, which is the source of the overcount):

- **170** literals live in `data parse_population_baseline_rows` — the two fixed populations (five largest + 165 size-tail), the only rows the report counts.
- **20** literals are the `rse_population_*` helper fixtures (nine helper bindings, each carrying its own small `ParsePopulationRow` list used by the witness's named negatives).
- **1** is the declaration line `type ParsePopulationRow {` itself.

191 = 170 + 20 + 1. The five population shape names (`FiveLargest`, `Sample165SizeTail`, `VerdictOnly165Reuse`, `InstrumentCheckPop5`, `FullRootsReference`) exist only on the measurement receipts (`ParsePopulationMeasurementRow` side), not among the 170. The report's 170 stands.

## 5. Cells — raw output, real rc, load alongside every cell

**Typed home (review 98937 F4).** These cells are projections; their one typed home is
`parse_population_measurement_receipts` in `dag/gunbc/parse_population_baseline_roster.dag` (one
`ParsePopulationMeasurementReceipt` per cell: printed counters, real rc, binary sha256, tree sha, host,
boot id), consumed by the witness legs `parse_population_measurement_receipts_record_the_sweep_holds`
and `parse_population_receipt_publishability_refuses_planted_failed_cell_holds` (parses only — the module is
not enrolled on the required floor; both legs green by execution in the pp_batch2 19-operand run and the §7a step pair). `planning_has_one_home` bars this file from being a **plan** home — the plan
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
the same program and are publishable. **Load sample beside the median receipts (review follow-up):** the run2b cells (A/B/C re-run under the loosened gate) carry load1 sample `44.02/128` captured at run time; the run2 cells carry `not captured at run time; no retroactive sample exists` — stated literally, no retroactive sample was manufactured. Beyond these samples, load averages at run time were recorded in the run logs behind the acquisitions and are deliberately not re-typed here — no claim consumes them.

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

## 7. Attribution findings — the honest negative (sampling tier), superseded for the handback element by the §7/§7a counter ruling

The row's `boundary` asks each hot spot be attributed to a building-block/composite contract it violates, **or to the algorithm itself** (realization vs algorithm). On srv1 under current policy this attribution **cannot be produced**, for two independent reasons, stated plainly so the numbers are not read as the ones the ticket asked for:

1. **The named instrument is unavailable.** `first_slice` names "perf self time"; perf cannot sample at `perf_event_paranoid=4` (§2), and valgrind/strace/gdb are absent. A self-time histogram by collection/allocation/provenance/lexer/parser — the split the row wants — requires one of those and was not produced.

2. **Whole-invocation wall/RSS cannot carry per-operation attribution.** The ground for this limit is observable, not borrowed: the emitted verb's report type `PopulationReport { text: String, held: Bool, detail: String }` in module `v2.cli.population_verbs` (`src/v2/cli/population_verbs.dag`) carries no per-phase counters — there is nothing in the verb's output to decompose a single wall/RSS figure against, so wall/RSS stays a whole-process figure. If the reviewer relies on a standing ruling here, it is not in `gunbc.design_ruling` (grep for it returns no file) and would have to be recorded there to be citable; this report does not rest on any ruling it cannot name a carrier for. Cell C confirms the row's intended realization control is **not establishable locally**: `--reuse-from` (and `--receipts`/`--write-identity`) require `--compiler`, and reuse consumes a CI **published-ancestor** restore (ancestor/identity/receipts keyed by ancestor sha + compiler) that has no local write equivalent. With no such cache, the reuse print is the one in §5, and the cell-C wall receipt in the binding is statistically indistinguishable from the B (full-artifact) median receipt — i.e. the "verdict-only speedup cannot hide unchanged map construction" control cannot separate map construction from verdict computation on this host.

   **Control-as-designed result — read this before spending a slot on it.** The verdict-only control **could not answer the question it was designed to answer on this workload, and this is a result, not a failure of execution.** Its purpose (from the row's `boundary`: a verdict-only speedup must not hide unchanged map construction) requires the reuse path to actually hit (`reused > 0`), so that a verdict-only run reuses the map and any residual cost is verdict computation. On srv1 reuse cannot hit without a CI **published-ancestor** cache, so cell C re-measured full construction (the §5 reuse print; its wall receipt in the binding) against the B (full-artifact) median receipt in the same binding — the gap is run-to-run noise. The control is therefore **inconclusive by construction on this host**, not under-powered: repeating C with more samples will NOT separate the two, because the reuse path that carries the signal is absent. Do **not** spend an acquisition repeating C until either the per-phase-counters row (`v2-verb-reports-per-phase-counters`, pull 17071) lets the verb decompose its own cost, or a perf window (operator decision) makes sampling available — those change what the verb can report; more repeats of the whole-process figure do not.

**Seed-vs-emitted is not a usable realization cell either.** The v1 seed binary has no `parse-population` subcommand (`unrecognized subcommand`, exit 2); a seed cell is a missing capability, not a cost. The realization split the verb *can* express is `--reuse-from` vs reuse-OFF, which §7(2) shows is not establishable here.

**Therefore (superseding the earlier criteria-question framing):** the baseline is taken (fixed populations, byte-identity proven, refusal-comparable, wall/RSS min+median with load), and the attribution element the `boundary` requires — "each hot spot attributed to a building-block or composite contract it violates, or to the algorithm itself" — is discharged by the reviewer's ruling **in counter units**. Ruling, verbatim (neat-badger-614, relayed by warm-fox-517): *"Route (b) discharges 'per consumer, the attributed profile' as long as the attribution is stated in COUNTER units (per-phase steps, TEXT_LOOKUP_CHARS_WALKED, eval_steps per label, and cpu_ms as the runner reports it), labelled plainly as counter attribution and not time sampling. No time-share claim may be made that the counters don't measure."* The perf window (a) is retired for this node — the ruling made `perf_event_paranoid` irrelevant here; the CI receipts cache (c) remains relevant only to the cell-C control-as-designed result above, which stays exactly as recorded (inconclusive by construction on this host, do not repeat). The counter-unit attribution itself is §7a.

## 7a. Counter attribution — the fixed 19-operand batch (counter units, not time sampling)

Per the ruling (§7): every number here is in **counter units**, taken verbatim from the runner's own stream
(`eval_steps=<n> cpu_ms=<n>` per label from `src/v1/stage0/src/target_invocation_host.rs`, the same lines the
`HELD`/`DID-NOT-HOLD`/`UNOBSERVED` verdicts print beside). This section is labelled **counter attribution**;
it is not a profile and makes no time-share claim. Wall appears only as each leg's total (§7a-run), never
apportioned to a consumer.

**Run provenance (step-pair acquisition, 2026-11-27).** Poller `pp_acquire_step_pair.sh`, `POLLER_START host=srv1 BOOT=c095c0cd-1eaa-4bc7-861d-2bd9756c2bc4 PID=1188990 OWNER=bright-hawk-891 MAX=20 SLEEP=180s`; gate line `[2026-11-27T20:12:13Z] TRY=1/20 load1=39.18 load5=40.04 real_occupants=10 -> GATE_PASS`. In-slot: acquisition line `heavy-slot: slot 8/12 acquired by bright-hawk-891 at 2026-11-27T20:12:13Z`, `SLOTTED_PID=1189059 CGROUP=0::/user.slice/user-1000.slice/user@1000.service/app.slice/heavy-slot-1189017.scope`, `SLOT_OWNERSHIP_PRECONDITION=ok file=/home/briansrls/.heavy-slots/slot8.who`, `IN_SLOT HEAD=b0fff7396d295dc937dcc04a317a80196ff4c1ab`, `IN_SLOT disk_sha256=5aafd4fb6bc9e92bc4722c8fd85407351f9f7f235fc269f84d2afad90a86420e` — read again before the AFTER leg (`AFTER_LEG_PRECHECK disk_sha256_read_before_leg=` same value). Freshness line both legs: `gunbc test: binary fresh against its dep-info: /home/briansrls/tmp-bright-hawk-891/pp-build/target/release/gunbc inputs=268`. Provenance statement: **binary built-from-0d77469ccb92, executed at tree b0fff7396d2**, `git diff --name-only 0d77469ccb92 HEAD` prints exactly `docs/plans/parse-population-baseline-measurement-report.md` (rc 0) and the `dag/`-scoped diff is 0 files — the claim inputs are byte-identical across the whole chain, and the freshness inputs (268 dep-info entries, no `.dag`/`.md` among them) are unaffected. Both legs invoked from `cwd=/home/briansrls/tmp-bright-hawk-891/pp-build` with the relative binary path. The 19 operands were printed and the count asserted from the file (`OPERAND_COUNT=19 (asserted 19)`, `wc -l < leg_labels_final.txt` = 19).

**Named expectations printed BEFORE the legs** (verbatim from the log): `eval_steps` per label EQUAL across legs (equality; a nonzero delta makes the step meter untrustworthy — a finding, not a pass); batch total EQUAL; held_count 19/19 both legs; `cpu_ms` **no direction asserted** (reported only, wall noise); rc 0 expected, rc 124 = TIMEOUT_NOT_RESULT (report, no trim, no re-arm). Hot-spot targets named from the retained instrument cells: `…roster_is_unique_holds` (eval_steps 134401), `…roster_byte_invariants_hold` (10739), `…selection_rule_is_modelled_holds` (7470) — the pair brackets all three.

**Results (runner's own stream; logs retained at `/home/briansrls/tmp-bright-hawk-891/step_pair/{before,after}.log`).** BEFORE 20:12:13Z–20:19:06Z, AFTER 20:19:06Z–20:25:15Z, each leg's rc on its own line: `RC_BEFORE=0`, `RC_AFTER=0`; `SLOT_RC=0`. Summary lines, both legs verbatim: `19 claim(s): 19 held, 0 did not hold, 0 unobserved`. `STEP_MAP_COUNTS before=19 after=19 (expect 19 19)`, `EVAL_STEPS_IDENTICAL all 19 labels`, `HELD_BEFORE=19 HELD_AFTER=19`, `VERDICT_STEP_PAIR=STABLE — step meter trustworthy for the repair before/after pair`. Batch wall totals: BEFORE 413 s, AFTER 369 s (leg totals only; not apportioned). Per-label counters, `eval_steps` identical across legs on every label; `cpu_ms` shown before/after (noise, no direction asserted):

| label (suffix after `…witness:`) | eval_steps | cpu_ms B/A |
|---|---|---|
| roster_byte_invariants_hold | 10739 | 19/18 |
| roster_cardinality_holds | 2429 | 2/2 |
| roster_duplicate_key_refuses | 2102 | 6/5 |
| roster_is_unique_holds | 134401 | 91/89 |
| roster_rows_ordered_descending_holds | 4266 | 5/5 |
| cardinality_refuses_mistyped_real_row_holds | 211 | 0/0 |
| cell_measurement_pairing_is_constructed_not_checked_holds | 404 | 1/1 |
| cell_renderings_are_distinct_holds | 262 | 0/0 |
| duration_ordering_refuses_reversed_triple_holds | 53 | 0/0 |
| measurement_receipts_record_the_sweep_holds | 373 | 0/0 |
| ordering_accepts_descending_holds | 112 | 0/0 |
| ordering_refuses_ascending_holds | 111 | 0/0 |
| receipt_publishability_refuses_planted_failed_cell_holds | 47 | 0/0 |
| reconcile_predicate_refuses_bad_row_holds | 299 | 0/0 |
| selection_breaks_tie_by_path_holds | 249 | 0/0 |
| selection_rule_is_modelled_holds | 7470 | 9/8 |
| stall_capability_extraction_distinguishes_planted_values_holds | 45 | 0/0 |
| stall_trigger_capability_declared_holds | 33 | 0/0 |
| top_n_window_discriminates_holds | 187 | 0/0 |

Batch totals (wrapper arithmetic over the runner's own per-label `eval_steps`, computed by `awk` with rc 0 printed): `BATCH_TOTAL_EVAL_STEPS_BEFORE=163793`, `BATCH_TOTAL_EVAL_STEPS_AFTER=163793`.

**What the counter units attribute.** Three legs carry 93.2 % of the batch's `eval_steps` (134401+10739+7470 = 152610 of 163793; `awk 'BEGIN{printf "share=%.4f\n", 152610/163793}'` → 0.9317, rc 0 — a counter-share of the step budget, which the counters do measure; it is not a wall-time share): `roster_is_unique_holds`, `roster_byte_invariants_hold`, `selection_rule_is_modelled_holds`. All three are the whole-roster folds over the 170-row `data parse_population_baseline_rows` — their step cost is the folds visiting 170 rows × their predicate width, i.e. the algorithm's declared work over the fixed denominator, not an unplanned realization cost: no collection/allocation counter fires unusually (the batch runs on pre-built values; `minor_faults`/context-switch counters in the typed process-times receipts show no differential), and no building-block contract they exercise is violated (the witness's own named negatives — duplicate key, ascending order, reversed triple, planted receipt, mistyped row — all refuse, §5a). Attribution per the ruling's options: **algorithm**, for all three hot legs — the per-row predicate evaluation of key-uniqueness and byte-invariants over 170 fixed rows. Nothing here becomes a row for `emission-ownership-facts-survive-lowering` (no realization defect observed in counter terms); no constant-rule-per-character interpretation cost appears in the claim route (that cost lives in the verb route, whose counters are recorded in the typed receipts and §5).

**Disclosed limits and observer defects (kept, not deleted).** (1) `BEFORE_COUNTER_LINES`/`AFTER_COUNTER_LINES` printed `no counter lines in stream`: the `population parses:` / `meta-parse verdict reuse:` / `[cost-partition]` / `TEXT_LOOKUP_CHARS_WALKED` prints belong to the parse-population **verb** route, not the claim-test route this batch takes; those verb counters were captured earlier and live in the typed receipts (§5), and their absence here is route scope, not loss. (2) The wrapper's first total lines printed empty (`join: missing operand` — `join -o 1.1,1.2` misused with one file); totals were recomputed from the retained per-label step maps with plain `awk` (rc 0, printed above). The defect stays in `/home/briansrls/tmp-bright-hawk-891/step_pair.log` at its original position.

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
  lifecycle. All 19 witness legs are **green by execution**: the 19-operand batch (`19 claim(s): 19 held,
  0 did not hold, 0 unobserved`, pp_batch2 run — link in the PR #17037 evidence comments), reproduced
  identically by the §7a step pair, with the two mutation controls each flipping exactly the pre-named leg
  (§5a). No acquisition is currently held or pending on this node.

What remains in this file is deliberately only what no `.dag` carrier can hold: the premise finding (§1), the
host-capability readings with their real exit codes (§2), the emittedness proof (§3), the staging/byte-identity
procedure (§4), the verbatim runner prints (§5), the attribution reasoning and its honest negative (§6–§7), and
the relay observation (§8).
