# Scoped census run — design

Status: accepted (side-chat design review 5499015152; reviews 79199, 79204). Owner: census lane
(mandate: keep the v2 roadmap grounded in measured evidence). The lane's first delivery is census
improvements #1 (scoped run), #2 (source locations) and #3 (run record + diff). This note covers #1
and the run record #3 reads. It deletes itself when #1–#3 land.

## Why the census reads a population today

`native_lane_census_read` (`v2.compiler.compile`) states the rule: a census reports what is wrong
with every member, so it cannot be driven by the acquirer, whose first refusal refuses the whole
acquisition (`acquisition_loop` → `AcquisitionRefused`). The subject closure
(`native_census_subject_closure`) is therefore computed after the whole-corpus ingest, from the
import edges of accepted trees, and only annotates roots. Every census question about native-seven's
closure pays for the whole corpus.

The rule is right about the failure it prevents: a truncated population reported as complete. It is
wrong that reading everything is the only way to prevent it.

## Acquisition that records a block instead of stopping

One acquisition authority (`v2.compiler.module_acquisition`), a second loop over the same
`acquisition_lookup` and `acquisition_verdict_step`:

- `acquisition_loop` (unchanged): the compiler's door. Any refusal refuses.
- `acquire_census_closure_of`: a refusal local to one member or one edge is recorded and the loop
  continues with the rest of the frontier. A refusal local to neither (grammar unprepared, entry
  underivable) still refuses.

```
type CensusAcquisition
  = CensusAcquisitionClosed  { entries, modules, reads }                // every request followed
  | CensusAcquisitionBlocked { entries, modules, blocked: non-empty CensusBlock list, reads }
  | CensusAcquisitionRefused { refusal }

type CensusBlock
  = CensusMemberBlock { module, refusal, content, host_directory, requesters }
  | CensusEdgeBlock   { refusal }
```

**A blocked member is observed once, with every requester.** The census loop keeps, per attempted
member, its terminal refusal, the bytes the lookup's read returned, and every request that reached it.
A diamond onto one malformed member yields one block with two requesters: not two reads, not two
rows. The member is then marked known, so later requests reach it through the lookup's known-member
shortcut. That shortcut runs after the lookup's cross-tree check, so sharing the observation never
shares an edge's permission. A requester whose tree may not import the member is still denied after
another requester reached it legally, and the other way round.

**The refusal is carried whole. Its arm is its meaning, and the census adds no second classification.**

| arm | what it establishes |
|---|---|
| `AcquisitionMemberRefused` | the member's front end refused; its outgoing requests are unobserved |
| `AcquisitionMemberSitesUnreadable` | the front end may have passed; reference discovery is incomplete. Not a front-end-stage refusal |
| `AcquisitionUnknownModule` (import / member / entry) | a stated requirement names no module; no target or descendants are invented |
| `AcquisitionLookupRefused` with `ReferenceCrossTreeDenied` | this edge is forbidden: not probed through, not routed around, target not marked bad |
| `AcquisitionLookupRefused` with another `ReferenceLookup` cause | that cause's own meaning (unreadable file, module line missing or disagreeing) |
| `AcquisitionTreeUnbound` | the invocation lacks a root binding; not evidence about repository source |

A qualified reference that names no module still requests nothing (`acquisition_unknown_step`). The
continuing loop does not turn it into a missing-import block.

Behind a member block, whatever that member would have requested is **unobserved**, not absent. Only
the Blocked arm can say so, and a blocked population is never reported as a closed one.

## The census reads acquisition's bytes

The scoped population joins the census as `census_acquisition_ingest`: the admitted members'
witnesses (`acquisition_ingest`), then each blocked member's own read, once. Both are built by one
witness shape, `acquisition_read_witness`. It is never fed through `read_population`, which probes
the filesystem again. A file edited between acquisition and census would then make the membership
and the stage results describe different bytes. An edge block read nothing and enters nothing; it is
reported from the acquisition. The whole-corpus run keeps its `read_population` door.

Downstream of the ingest, the census pipeline is unchanged: resolve, infer, cause grouping. Only the
population differs.

## Closure membership has one authority

In a scoped run, "in the subject closure" is the acquired set. `native_census_subject_closure`'s
import-only walk is not consulted: it misses the qualified-reference edges the acquirer follows, so
two closure definitions would be a §3 fork. The whole-corpus annotation keeps its import-only walk,
labelled as such, until it can be re-derived from the same acquisition (follow-on).

The subject module is derived from the label by one function, `native_census_subject_module`. It uses
the target-pattern authority, and the native lane's module ↔ package correspondence read backwards.
The whole-corpus annotation uses the same function. Only a package label names one module; a single
target or a subtree names none, and the census says so.

## Fan-out is labelled with the relation it measured

`native_census_closure_index` builds both graph directions from `normalized_tree_import_bindings`.
When A reaches B only through a qualified reference, the scoped population includes B, but the
reverse graph can still omit A from B's fan-out. The first delivery therefore reports fan-out as
**import-edge reachability over the observed population, coverage incomplete**. It is not demand
fan-out, and not evidence that two repairs are independent. Follow-on (preferred): feed the
reachability fold the acquisition's observed request → selected-target relation.

## Subject and entry point

- The subject is a target label bound by the instrument. The first subject is native-seven; later
  subjects are compiler entry closures, each as a further binding, never a changed meaning of an
  existing target.
- New driver verb, because it is a distinct acquisition request: `census-scoped <label>
  <source-root>...`. The label operand is the transport of the instrument's bound subject. The run
  record names the subject observed, and the consumer checks it against the binding.
- The instrument target is `//gunbc/instruments:v2-native-census-scoped`, in the existing
  `V2NativeCensusProducer` family; it is not a parallel producer.
- Preparation consumes the existing verified native-product reuse route; there is no census-specific
  cache. Cold and eligible-warm whole-command costs are reported separately. A compiler-source repair
  legitimately changes the product identity, so warm reuse cannot promise to skip preparation.

## Run record (the part of #5 that #3 needs)

- Maps to `gunbc.repository_census_observation` (population / subject revision / method run). It is
  not a new census-observation type.
- Identity is the inputs actually read (captured content digests, with the revision recorded as
  claimed) and the executed compiler artifact (its verified preparation receipt), not only a
  source-closure digest.
- Contents: the subject; the scope (whole-corpus or scoped); the population and its digest; the
  acquisition status (closed or blocked, with the blocks); per module × stage the outcome: passed,
  refused (reason, and location once #2 lands), unobserved behind an earlier stage, or unobserved
  behind a block; the existing `NativeCensusCompleteness` facts; completion.
- Publication: an incomplete, run-identified record is written before preparation, and the completed
  one replaces it atomically. Without an observed terminal event the run reads incomplete, never a
  fabricated cause. An output failure cannot leave an older file readable as this run.

## Diff semantics (#3)

The comparison grain for the first delivery is the module × stage obligation, keeping both runs'
diagnostics. Root-level continuity is claimed only where an identity relation supports it; ambiguous
matches stay unmatched. Different revisions are recorded, not refused, since comparing them is the
point. Different subjects are refused.

| before → after | reported as |
|---|---|
| resolve refused → resolve passed, infer refused | cleared, and newly exposed |
| infer refused → resolve refused | infer now unobserved behind resolve (not "cleared") |
| member present → absent, after-run Closed (exclusion established) | scope changed, member named |
| member present → absent, after-run Blocked | unobserved, not "scope changed" |
| member newly visible after a repair, and refusing | newly observed, not a regression |
| member unobserved (blocked or stopped run) | incomplete evidence |
| only location or occurrence id moved | location update, not churn |

When the closure digest changes, the diff names the added and removed members and compares the
common population.

## Controls

- Planted closure outside every corpus root (`fixtures/census_scoped_acquisition`): acquisition around
  a member block and an edge block, every block named, nothing behind a block or outside the closure
  acquired, a closed case, and a diamond onto one malformed member (one block, two requesters,
  sibling observed). The compiler's door still refuses the same closure.
- A denied edge beside a permitted use of the same target, in both orders.
- The census ingest holds the acquired and blocked reads, each once, and nothing behind a block.
- The subject module is derived from the label; single-target and subtree labels name none.
- Still to add with the route: qualified-reference-only reachability, location movement,
  earlier-stage masking, and interruption before terminal publication.

## Acceptance

One real repair measured before and after through the normal entry point, producing a located
comparison that names the cleared blocker and the next observation, plus the roadmap revisions it
justifies. Wall-time and peak figures are motivation only. The carrier that lands names the
instrument producing them, `//gunbc/instruments:v2-native-census` and its scoped sibling, rather than
restating them (review 79199).
