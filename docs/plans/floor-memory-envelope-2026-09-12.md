# The floor cannot account for the phase memory it refuses on

The floor refuses on memory, but its cost receipts publish evaluation time and
no resident-memory reading for the terminal-ledger phase. The refusing run
publishes neither cost artifact. **The exact terminal-ledger typecheck peak and
its run median are UNOBSERVED.** This is an instrumentation gap: the available
cgroup readings establish throttle contact but cannot account for the phase's
memory demand. No cap change is justified by this finding.

The strongest available evidence is the natural experiment: the assignment
reports #11115 refused twice before a genuine floor SUCCESS at 42m23s, while
#11139 refused three times and never reached a verdict. The inspected jobs below
independently confirm a same-merge-subject refusal/success pair for #11115 and all
three refusals for #11139. The workload is marginal, not universally unable to
complete. These selected outcomes are not an estimate of the fleet's pass rate.

This is a stopped-line measurement report under DESIGN §§2, 4d and 5, not a repair
or a claim that the requested phase measurement has been completed. No refusal,
merge-admission wiring, OOM classification, or runner allocation changes here.

## Throttled slot envelopes: producer and subject

The observations below come from `floor_cgroup_envelope` in
`src/v1/stage0/src/cli_run/required_floor_runner.rs`: its `[floor-cgroup]` rows
read `memory.high`, `memory.max`, `memory.current`, `memory.peak`, and event
counters from the named cgroup and its ancestors. Only the **runner service's**
rows enter this table, never the aggregate parent slice's peaks.

| PR / attempt within run | CI job (raw log producer) | runner | largest printed service `memory.peak`, bytes | terminal result / refusal site |
| --- | --- | --- | ---: | --- |
| #11115 / 1 | [103493724795](https://github.com/gunb-ai/gunbc/actions/runs/34670663495/job/103493724795) | srv3-16 | 16106778624 | refused / `extdeps.git.object_store` |
| #11115 / 2 | [103498744149](https://github.com/gunb-ai/gunbc/actions/runs/34670663495/job/103498744149) | srv1-13 | 16114794496 | success |
| #11139 / 1 | [103491056928](https://github.com/gunb-ai/gunbc/actions/runs/34670582182/job/103491056928) | srv4-19 | 16106856448 | refused / `v2.workflow.floor2_prepared_subject` |
| #11139 / 2 | [103496272864](https://github.com/gunb-ai/gunbc/actions/runs/34670582182/job/103496272864) | srv4-21 | 16106946560 | refused / `v2.workflow.floor2_prepared_subject` |
| #11139 / 3 | [103505983106](https://github.com/gunb-ai/gunbc/actions/runs/34670582182/job/103505983106) | srv4-10 | 16107003904 | refused / `std.primitives` |

All five read `memory.high=16106127360` (15 GiB) and
`memory.max=17179869184` (16 GiB). Their service-local high events grow;
the printed service OOM and OOM-kill counters remain zero. The refusal is
`MemoryStallRefusedPageThrash`, not evidence of an OOM kill.

The checked subjects are the merge commits, verified from each job's checkout:

- #11115: `fb82d4c10374e61a135f4038ef4f7b414bbcaa7d`, head
  `53fc8e668b7812bd5f64f3d86feab18c6bb8b52b`.
- #11139: `45a4956c998e9b4ce11ca0c41e9dbb9907fdfbca`, head
  `c54441d2a45d0e67c2162be5f01f7c45d276bde0`.
- Both merge subjects use base `6cdebf53c4b1648e282de2f5ed1896b434b15b34`.

This is a five-job convenience sample, not a census of every attempt mentioned in
the assignment. In particular the selected #11115 run contains one refusal and
one success; this report does not claim to have found its other reported refusal.

The median of the five **largest printed service high-water readings** is
16106946560 bytes, or 15.000763 GiB: 0.78125 MiB **above** `memory.high`.
The range is 15.000607–15.008072 GiB. These are observations of a reclaim-throttled
slot, not a distribution of uncensored working-set demand. `memory.high` is a
throttle and can be crossed. Calling this median “the typecheck fits within
0.8 MiB” would invert the evidence.

There is no reset at the terminal-ledger boundary, no terminal-boundary cgroup
read in these logs, and no process RSS peak specific to this phase. The last
printed peak can even include work after ledger publication. Therefore neither
subtracting the threshold from this peak nor subtracting two cumulative peaks
answers the requested phase-memory question. Even the successful job is not an
uncensored measurement of demand on an unconstrained host.

## What the cost receipts can and cannot decide

For successful run `34670663495`, downloaded artifacts:

- `required-floor-claim-cost` (artifact `10292490653`):
  `required_floor_claim_cost.tsv`, summary `executed=3771`, columns for identity,
  outcome, observed/censored wall and CPU time, eval steps, and safety limits.
- `required-floor-cross-claim-demand` (artifact `10292285976`):
  `required_floor_cross_claim_demand.tsv`, summary `claims_absorbed=3771`,
  `claim_cpu_observed_total_ms=42837`, `claim_cpu_censored_rows_excluded=0`.
  Its header explicitly says `cost_columns=inclusive_of_callees_do_not_sum`.

Neither schema has an RSS column, and neither file contains
`terminal_ledger` or `floor2_prepared_subject`. They observe claim evaluation,
not the later ledger resolve. The artifact list for refusing run `34670582182`
contains measurement and outcome receipts but neither of these cost artifacts.
An absent artifact is unavailable evidence, not zero cost. No cost-partition
instrument was run; no `OverAttributed` result is asserted here.

The passing job does carry a phase-local **time** producer:
`publish_terminal_ledger` in `cli_run/terminal_ledger_publish.rs` prints
`phase=terminal-ledger-publish state=completed resolve_ms=31100 total_ms=32485
rows=3771` at `2026-09-12T05:07:51.2992978Z`.
That measures one resolve, not 3,771 repeated resolves.

## Why it costs what it costs: established facts and remaining discriminator

`build_ledger_wire_ctx` uses `process_shared_index(source_roots)` and
`resolve_entry_with_index_for_discovery_corpus` for the exact entry
`src/v2/workflow/floor_terminal_ledger_wire.dag`. `run_required_floor` invokes
publication once after its claim fold. Its prepared graph is still live;
`prepared.subject_digest` also has later consumers. The shared parse/typecheck
index survives. This is **a fresh context beside retained floor state**, not a
fresh empty process. The source explicitly chose rebuilding over retaining an
additional context across the fold; whether that tradeoff still saves memory
needs measurement, not reliance on that comment.

The named module is where the governor detects the stall. The four refusals
name three different modules, including `std.primitives`; this rules out treating
`floor2_prepared_subject` as a demonstrated unique allocation culprit. It does
not rule out a costly common dependency or retained state elsewhere.

The specific recurrence repaired by [#10992](https://github.com/gunb-ai/gunbc/pull/10992)
and [#11032](https://github.com/gunb-ai/gunbc/pull/11032) was fixture preparation
re-derived across isolated claim frames. That evidence does not transfer to this
once-per-publication resolve. Re-typechecking an already prepared fact is a
separate possible recurrence; the index's cache hits, misses and retained graph
ownership must distinguish it from new work. No allocation profile here proves
either “quadratic cost shape” or “genuinely irreducible large closure.”

There is an existing, more specific cost-shape lead:
`gunbc.recurring_failure_mode.entry_scoped_typecheck_rss_tracks_the_name_census`.
It records sleek-swift-789's 2026-09-11 arm64 `gunbc compile --entry
src/v2/workflow/floor2_prepared_subject.dag --target dag` observation: 16 resolved
sources, 5385 census-indexed modules, peak 9964140 KiB from `/proc/<pid>/status`
`VmHWM`, and exit 0. **That is prior evidence from its named producer, not a new
measurement of this entry or an amd64 requirement.** The row does not pin an exact
source SHA for its binary; this report does not promote its provenance. Its
isolated peak cannot be added to an independently observed floor RSS: shared
pages, caches and lifetimes overlap.

Current source confirms `tree_bare_census_for_root` builds the whole-root compile
closure's symbol census and caches it per root in `MultiEntryIndex`. The existing
`GUNBC_EDGE_INDEX_CENSUS_TRACE=1` miss trace identifies both root and index. Joining
those identities across preparation and ledger resolve is a concrete discriminator
for repeated census construction. The old isolated observation and current source
make this a stronger lead than assuming the small named module is intrinsically
large, but do not establish the allocation owner in these five runs.

The entry is **already scoped**. The wire imports identity, disposition and safety
vocabulary; `v2.workflow.required_floor` also imports roster/policy modules.
Narrowing further means separating those concepts at their existing authority
boundaries, not adding an `--entry` flag that is already present or omitting
required witnesses. A smaller import closure may still pay the whole-root census;
source size or an import grep is not a measured byte saving.

## Ranked repair experiments and costs

These are ranked by boundedness and expected diagnostic value. Savings are
unmeasured unless explicitly labeled otherwise; implementation costs are estimates.

| Rank | Candidate | Required discriminating observation | Cost / risk |
| --- | --- | --- | --- |
| 1 | Remove unnecessary co-residence at the terminal boundary | Read current RSS, cgroup charge and live graph/cache ownership immediately before resolve; compare the same subject after releasing only objects with no remaining consumers | Small ownership audit; potentially substantial memory benefit, presently unquantified. Do not clear reusable caches indiscriminately or retain another whole context across the fold. |
| 2 | Investigate the existing whole-root name-census cost-shape lead | Join census misses by root and index; attribute allocations and retained maps across preparation and this resolve | Small diagnostic exercise first; a compiler repair could be medium-to-large. Prior scoped evidence identifies a lead, not a measured saving on this floor. The earlier per-claim fixes are not its evidence. |
| 3 | Narrow the wire's dependency closure through existing concept boundaries | Exact compiler-produced closure census plus matched before/after phase peak and identical rendered ledger/refusals; establish that census cost also falls or is already shared | Small-to-medium model move with import consumers to update. Could reduce the measured 31.1s resolve and its memory increment; neither saving is established. |
| 4 | Size/place the runner for measured co-resident demand | Completed amd64 phase peaks and cgroup anonymous/file/kernel charge under an externally enforced, characterized envelope, including repeat variance | Operational allocation and reduced fleet concurrency; no supported GiB request yet. Legitimate if demand remains after ranks 1–3 are ruled out, not a cap-only green. |
| 5 | Split execution at the terminal boundary | Show that serial release plus a separately executed ledger producer preserves subject binding, all terminal identities and all refusal arms | Largest complexity: process/artifact boundary, startup and parse costs, ownership and failure propagation. Premature without proving retained co-residence is unavoidable. |

**Plain answer on sizing:** the current slot demonstrably reclaims at 15 GiB on
both heads. We cannot yet say how large a box this phase requires. There is no
evidence here for declaring 16, 18 or 20 GiB sufficient.

## Next-rung capability and reproduction

**Next-rung trigger:** the floor's own receipts publish per-phase resident-memory
readings on refusing runs as well as successful ones, sufficient to compare the
terminal-ledger typecheck phase's peak with the admitted budget. The readings must
carry subject identity, phase boundaries, resource/budget scope and unavailable
or censored standing. A named file or access grant alone does not discharge this
capability. The existing failure-mode row
`gunbc.recurring_failure_mode.instrument_blind_spot_anti_correlated_with_severity`
carries this specimen; it does not assert that the memory behavior is repaired.

Fetch each of the five jobs once, preserving its id in the filename:

```sh
gh api repos/gunb-ai/gunbc/actions/jobs/103498744149/logs --allow-escape-sequences > 103498744149.log
gh run download 34670663495 -n required-floor-claim-cost -D cost
gh run download 34670663495 -n required-floor-cross-claim-demand -D demand
```

For each job log, select `[floor-cgroup]` lines whose `level` ends in the runner
`.service`, take the maximum printed `peak`, then the median across the five job
values. Convert bytes to GiB by division by `2**30`; convert the difference from
`high` to MiB by division by `2**20`. Preserve the measurement grain stated above.
Read the actual refusal lines, not the echoed shell grep that classifies them.

A new measurement must retain the real floor's baseline, bracket **typecheck**
within this entry resolve (not all preparation), record current and peak process
RSS separately from cgroup charge, retain missing/censored distinctions, and
name exact source and binary identities. An isolated entry run is a useful
closure-increment control, but cannot replace the in-floor observation. Repeat
the same subject to report a phase-peak median; a refused attempt cannot supply
its unobserved completing demand. Keep the existing MemoryStall refusal active.

Two execution routes were checked in this session and could not provide that
measurement:

- [BuildBuddy envelope probe](https://app.buildbuddy.io/invocation/b6c1a5db-8f85-4197-a915-cea73a5745ee) reached amd64, but `/proc/self/cgroup` read `0::/` and both
  `/sys/fs/cgroup/memory.high` and `memory.max` were absent. The command stopped
  before building or running the workload. A separate
  [invalid-Cargo-flag control](https://app.buildbuddy.io/invocation/e56f48bc-d69a-4d8c-a6fc-d02fcf2c079d)
  reached remote Cargo and failed as expected; remote transport itself
  was working. No budget was invented to pass admission.
- The parent's bounded local alternative was
  `systemd-run --user --scope -p MemoryHigh=15G -p MemoryMax=16G ...`.
  This container has no `systemd-run`, its cgroup directory is not writable, and
  it has no built local interpreter. Its visible 31 GiB container limit is not
  permission to consume the shared host slice. No local heavy run was started.

An arm64 scoped control, if subsequently enabled, establishes a cost-shape lead;
it does not establish an amd64 cap. The exact runner phase peak and median remain
the outstanding measurement, explicitly reported to `bright-eagle-728`.
