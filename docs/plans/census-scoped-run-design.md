# Scoped census run — design

Status: proposal. Owner: census lane (mandate: keep the v2 roadmap grounded in measured evidence).
First delivery of that lane is census improvements #1 (scoped run), #2 (source locations),
#3 (run record + diff). This note covers #1 and the run record #3 reads. Deletes itself when
#1–#3 land.

## Why the census reads a population today

`native_lane_census_read` (src/v2/compiler/compile.dag) states the rule: a census reports what is
wrong with every member, so it cannot be driven by the acquirer, whose first unparseable member
refuses the whole acquisition (`acquisition_loop` → `AcquisitionRefused`). The subject closure
(`native_census_subject_closure`) is therefore computed *after* the whole-corpus ingest, from the
import edges of accepted trees, and only annotates roots. A whole run costs ~2 h over ~8.4k modules
to answer a question about ~135.

The rule is right about the failure it prevents (a truncated population reported as complete).
It is wrong that the only way to prevent it is to read everything.

## The change: acquisition that records a blocked member instead of stopping

One acquisition authority (`v2.compiler.module_acquisition`), a second loop over the same
`acquisition_step`:

- `acquisition_loop` (unchanged) — the compiler's door; any refusal refuses.
- `acquisition_census_loop` — a member-local refusal (`AcquisitionMemberRefused`,
  `AcquisitionMemberSitesUnreadable`) or an edge-local one (`AcquisitionUnknownModule`,
  `AcquisitionLookupRefused`, `AcquisitionTreeUnbound`) is *recorded* and the loop continues with
  the rest of the frontier. Whole-run refusals (grammar unprepared, entry underivable) still refuse.

Result:

```
type CensusAcquisition
  = CensusAcquisitionClosed  { entries, modules, reads }                 // every edge followed
  | CensusAcquisitionBlocked { entries, modules, blocked: NonEmpty<AcquisitionRefusal>, reads }
  | CensusAcquisitionRefused { refusal }
```

`Blocked` is the honest statement "acquisition blocked at these members/edges; whatever lies
behind them is unknown". A blocked member that has a file is still a census member (its file
refusal is a row); its outgoing edges are unknown, and the run record says so.

The scoped population is then fed to the **existing** census pipeline through the same
`read_population` door the whole-corpus run uses — resolve, infer, cause grouping, fan-out are
untouched. Only the population differs.

## Closure membership has one authority

In a scoped run, "in the subject closure" *is* the acquired set; `native_census_subject_closure`'s
import-only walk is not consulted (it misses qualified-reference edges the acquirer follows —
two closure definitions would be a §3 fork). The whole-corpus run keeps its annotation until it
can be re-derived from the same acquisition (follow-on, not in this delivery).

Fan-out in a scoped run is fan-out *within the scope*, and is labelled so; it is not comparable to
whole-corpus fan-out.

## Subject and entry point

- Subject is a target label, parsed by the one target-pattern authority, as `native_census_subject_label`
  does today. First subject: native-seven. Later subjects: compiler entry closures.
- New driver verb, not a flag on an existing one (the verbs are the census's modes; cf.
  `census-infer`): `census-scoped <label> <source-root>...`.
- Instrument target: `//gunbc/instruments:v2-native-census-scoped` (subject bound in the target,
  not on a CLI, matching the "label is the instrument's own fact" rule).

## Run record (the part of #5 that #3 needs)

One JSON object written to a named output file (not interleaved in the log), read by the summary
and the diff:

- `subject` (label), `scope` (`whole-corpus` | `scoped`), revision (full sha), producer identity
  (emitted-compiler closure digest, already printed by the native route);
- `population`: member list + digest; `acquisition`: `closed` | `blocked` with the blocked list;
- per module × stage outcome: passed / refused (reason, location once #2 lands) / unobserved
  (behind an earlier stage) / unobserved (behind blocked acquisition);
- `completeness`: the existing `NativeCensusCompleteness` facts (infer stops at first fatal, etc.);
- `completion`: finished | stopped (a run that dies leaves a record marked stopped, not absent).

## Diff semantics (#3)

Comparisons are refused across different `scope`/`subject`. Within one subject:

| before → after | reported as |
|---|---|
| resolve refused → resolve passed, infer refused | cleared + newly exposed |
| infer refused → resolve refused | infer now unobserved behind resolve (not "cleared") |
| member left the population | scope changed, member named |
| member unobserved (blocked/stopped) | incomplete evidence |
| only location / occurrence id moved | location update, not churn |

## Controls

Scope exclusion (a module outside the closure never ingested); blocked acquisition (planted
unparseable member → `Blocked`, remainder reported unknown, never a smaller complete population);
location movement; earlier-stage masking; stopped run.

## Acceptance

One real repair measured before/after through the normal entry point, producing a located
comparison naming the cleared blocker and the next observation; plus the roadmap revisions it
justifies. Also measure whole-command wall time including compiler preparation, since emission
(~18 min observed) may dominate a ~135-module census.
