# v2 native memory: supporting analysis and review handback

> **Status, 2026-10-08:** Planning changes only. The accepted direction and scoped operator rulings now live in [demand-engine program D15](demand-engine-program.md#d15--native-memory-decisions-and-milestone-bindings), authored in `gunbc.plans.demand_engine_program`. The actual milestone members live in `gunbc.roadmap.roadmap_authority` under `demand-engine` and project to [ROADMAP.md](../../ROADMAP.md). This document preserves the dated analysis and review evidence; it is not another scheduling or decision authority.
>
> **Working pattern:** Work resumes one bounded slice at a time. The earlier year-long program estimate and subsequent week ranges are withdrawn as a schedule. There is no continuous-work or staffing commitment. At each handback, update the owning roadmap member with the landed revision, relevant evidence, remaining uncertainty and next action, then pause.
>
> **Measurement scope:** Every numerical finding below is a historical observation from srv2 (aarch64) at main `6d82ac7721` / `c48092a621`, using measurement-only edits to an emitted-crate copy. These measurements have not been rerun for this planning update. The reproduction section names the historical probes; D15 and the actual milestone choose the next bounded qualification.

## Where to resume

The roadmap members carry outcomes, acceptance controls and stopping points. Use their stable IDs when recording progress:

| Roadmap member | Decision home | Current handback |
|---|---|---|
| `native-memory-parse` | D15 / R2, scoped seed cleanup | Current delivery: [#13577](https://github.com/gunb-ai/gunbc/pull/13577) followed by [#13582](https://github.com/gunb-ai/gunbc/pull/13582). Both remain unmerged at this snapshot; the review below names the remaining child work. |
| `native-memory-envelope` | D15 / R5 and scoped R4/R3 | Planned. First select the exact native must-complete invocation and tiny-grant refusal case; no successful 1 GiB workload has been selected or demonstrated here. |
| `native-memory-cli-retention` | D15 / R1 | Planned. Name the post-census consumers and preserve their information in a sufficient compact carrier. |
| `native-memory-driver-retention` | D15 / R1; R6–R8 only as needed | Planned. Choose one payload/context family and establish every real reader before releasing it. |
| `native-memory-useful-run` | D15 / R1, consuming the envelope | Planned. Depends on the envelope; add only the retention or live-data handling prerequisite the selected run actually demonstrates. |

The settled R2, R5, scoped R4/R3 and practical R1 rulings, open R6–R12 decisions, and triggers for later tiers/identity/value-grain work are recorded once in D15. Heap accounting and dead-data release do not wait for the full M1.b-to-M5 sequence. Release of dead data and spill/recomputation of still-live data remain distinct capabilities, including within one run. Compile-stage memo work is a separate PR; the parse milestone does not close it.

## Parser review handback — 2026-10-08

- **#13577, `f0e5e776bfaccc056049c15d0dacfff355e69808`:** Reviewed as ready to queue. It is already out of draft. Its PR CI is green; the Rust unit lane is skipped on PRs under the existing workflow. This is review evidence, not a merge or roadmap acceptance event.
- **#13582, `be39cab5e912c7a17ea2abd6a2c9777966391fcd`:** Draft stacked on #13577. The deletion review found the parser/seed removal coherent. Tighten `v2.test.provenance.parse_memo_frame_replay` so the selected terminal's own occurrence id resolves to its expected span; checking only that the expected span exists in the collected loci is not discriminating against swapped terminal spans. Keep occurrence uniqueness and index coverage checks.
- **Qualification still owed by the child:** Restack/push and verify CI on its final `main`-based head. The workflow's PR activity types include `opened`, `synchronize` and `reopened`, so base retargeting alone is not evidence that a run started. No new CI job or blanket witness enrollment is needed for this review finding.
- **Local results supplied by the author:** Build, clippy, fmt, affected module runs, build lane and the native CLI instrument passed; `logic.dag` no longer has the memo-induced refusal. These do not establish whole-root success or a successful 1 GiB workload.
- **Unresolved witness:** The extra `v41_source_patch_converge` red5 refusal also reproduced on #13577's head. That supports treating it separately from the deletion; it does not establish a host/timing cause. Preserve the next concrete action if it repeats: capture the first failed operation/assertion, including its history/patch-blob inputs. The 15 other local host refusals are the author's reported baseline.

## Findings at the measurement revisions

**The packrat parse memo produces wrong answers.** In `v2.compiler.parse`, backtracking restores the whole `ParseProvenanceState` (allocator, span index, frame), so occurrence ids are re-minted along the surviving path. The memo table survives rejection, so a hit (`parse_nonterminal_memoized`) serves a capture minted on another path. `parse_prov_merge` only advances the allocator to the maximum, and `v2.std.provenance` `span_index_adopt` silently keeps an existing entry. Ids therefore collide, and served nodes take other nodes' spans.

Over all of `dag` + `src/v2` (7,932 files whose parse is accepted), comparing memo-on with memo-off (`materialization_allows_memo_store` → false, i.e. a `Recompute` verdict):

| Observation | Memo on | Memo off |
|---|---|---|
| Tree-structure digest equal | all 7,932 files | (reference) |
| Census refusals | 2,032 | 134 |
| Refused only with the memo | **1,898 files**, all `normalize_reason_minted_occurrence_duplicated` | — |
| Nodes resolving to a different source span | **1,011,967 of 84,098,594 (1.2%), in 4,795 files** | (reference) |
| Total parse time | 1,146 s | **934 s** |
| Lookups / hits | 16,491,677 / 112,337 (0.68%) | — |

- The census is total and refuses on any file (`v2.compiler.self_host.closure_emission`), so these false refusals alone keep the native CLI from emitting anything against the real roots.
- **Concrete case:** in `dag/test/fixture/cross_shard_seam/importee.dag`, the authored token `SeamBeta` at bytes 173..181 resolves to that range without the memo. With the memo it resolves to a derived parser event with no source location: its id collided with a wrapper minted on the surviving path.
- **Hits come from one production.** Only 3 of 86 productions ever hit; `dag_production_pattern` has 112,327 of the 112,337 hits.
- **Memory:** on `src/v2/compiler/body_lowering_fold.dag` (594 KB), 28.0 s / 1.61 GiB with the memo and 23.9 s / 0.53 GiB without.
- **The tail is real.** 103 files (1.3%) parse more than 1.5× slower without the memo, adding 7 s in total against the 212 s saved (worst case +265 ms). The cause is not match arms, which are less dense than average in those files. It remains unexplained. D15 / R2 retains stress controls as regression evidence and explicitly gives up the packrat worst-case guarantee; it does not claim a replacement bound.
- A disposition row on main already scheduled this deletion: `gunbc.primitive_egress` `dispositions_collection` row `parse_table_insert` reads "the prepared-choice plan (gunbc#11422) replaces the per-token memo; then both arms delete". #11422 merged on 2026-09-17.

**At corpus scale the CLI's memory is cross-file retention, not the memo.**
- On root `dag/extdeps` (1,028 files, 8.5 MB) the peak is 4.81 GiB as is, and 1.40 GiB when the census keeps neither the merged span index nor per-file trees. About 70% is retention across files, roughly 0.4 GiB per MB of source.
- Over the full 104 MB of roots this extrapolates to tens of GiB. `emit --entry std.integer --source-root dag --source-root src/v2` reached 15.7 GiB at a 900 s timeout with no output.

**The driver's demand engine keeps every settled value for the whole run.**
- This applies to the `adjudicate` verb of the binary emitted from `v2.compiler.compile`. The CLI has no engine.
- `v2.std.demand_engine` has no release operation. #12401 turned the old loop's per-module drop into whole-run accumulation, declared only in a comment inside a rendered-main Rust string. There is no §4b(3) rung-drop row.
- Separately, `native_demand_execute` discards the returned resolution context, so every accepted module re-runs a closure-wide resolve. This is a structural reading; it has not been measured.

**There is no envelope.**
- Neither native binary reads or binds a memory bound, and the harness spawns them with no limits.
- Only OS cgroups enforce anything, and an OOM kill is not a typed refusal.
- `std.materialization_ladder` `retention_admission` admits `ReleasedAtProviderScopeExit` providers at any capacity. `std.cache_interface`'s own comment says scope release bounds growth, never peak.

## Corrections to the first revision

1. **Occurrence ids are not monotone.** Backtracking re-mints them, so the proposed append-only span index is unsound on its own. It also needs a monotone allocator. That version was prototyped by a reader: zero location mismatches, 14 of 15 existing claims green. It is historical exploration, not the selected implementation: D15 / R2 chooses deletion. Reopening a repaired memo would require new evidence and a new disposition.
2. **The census must stay total.** Under the default `v2.std.resolution_policy` `NamespaceOnlyY`, a bare name may bind anywhere, and `closure_emission` states that the census is TOTAL for that reason. "Census only the entry's closure" is withdrawn. The `native-memory-cli-retention` milestone must preserve the declaration and source information later consumers need using sufficient compact projections.
3. **Each memo snapshot pins the span index for the current file so far, not the corpus so far.** Every parse starts from `span_index_empty`; only the allocator crosses files.
4. **The empty-index probe was not a proof of semantics.** Its byte-identical CLI output only attributed the memory: spans for replayed ids were silently dropped.

## How the historical target relates to the recut

The gap table and full-program checklist below preserve the analysis that motivated the direction. Their size estimates are historical, not a landing plan. The five roadmap milestones have their own bounded exits. In particular, the complete per-judgment seal, value-grain coverage and additional persistent/tier families are not extra gates on parser deletion, independent heap accounting or one measured retention cut. Their continuation follows D15's named triggers and existing M1.b/M1.c/M4/M5/M6 ownership.

## Historical gap analysis

| Requirement | Observed at the measurement revisions | Gap identified then | Original size estimate |
|---|---|---|---|
| Mandatory: every derived computation's retention is governed by a decision | Only the parse memo consumes a verdict, once per grammar. 25 retention mechanisms across both native closures; 13 ways to bypass governance | A derived verdict per judgment kind, with the seal refusing a missing one; a closed inventory census, executed | L |
| Global: both binaries, every route | The CLI holds every file's trees and spans to exit. The engine runs only in `adjudicate`. Rendered mains decide release in Rust strings | One allocator counter in every native main; CLI judgments keyed by `std.judgment_contract` `DemandIdentity`; one default fold | L |
| A default policy with no annotations | No policy carrier; capacity literals on each provider; `admitted.first()` | One total default fold in `std.materialization_ladder`: release at the last consumer, recompute only with a typed warrant, `Discharged` only for unavoidable recurrence | L |
| Override that cannot bypass accounting | No run-level carrier | A run-root override selecting among admitted handlers through `std.decision` `select_realization`, with no field for a grant or capacity | M |
| One envelope shared by execution and caching | OS cgroups only; the native binaries bind nothing | Grant resolution via `gunbc.host_budget_source` `effective_planning_budget` (narrow-only); a live-heap counter; a typed wall; spawners bound to the same grant; typed refusal at safe points | L–XL |
| Bounded retention where it dominates | CLI cross-file retention (about 70% at 8.5 MB); memo snapshots; engine whole-run retention | Delete the memo; interval table instead of the merged span index; projections instead of trees; context carry; exact last-consumer release | L–XL |
| Online hit/miss policy with RAM → disk tiers | LRU only (O(n) per get in `std.artifact_store`); the disk store is witness-only on main | Store on the native route; reuse-distance instrument; S3-FIFO as its own cited extdeps handler; admission-filtered demotion | L–XL; cross-run serving uses M1.c; in-run spill has its own measured trigger |
| Computation identity separate from content identity | `EvaluationIdentity` / `ResultContentIdentity` / `EvaluationResultBinding` are witness-only. SHA-256 already executes natively (`//gunbc/instruments:native-crypto-vectors`) | Mint native hashing for identity (measure first); split identity for one family using the §2.8 manifest protocol | L |
| Modules never tune caching locally | Literal capacities (EvalCallMemo 1M, CrossClaimPureMemo 256 MiB, family budgets, unbounded ParseTable) | Delete inert carriers; capacities only from the run grant and `gunbc.materialization_store_budgets` | M |

## Deferred full-program acceptance checklist

1. **One envelope.** Each native run resolves exactly one grant per conserved axis. One allocator counter counts execution and retention, and lease seats are derived.
2. **Typed adherence and useful execution.** A named must-complete roster finishes within its grant. A named may-refuse roster ends in a typed, located refusal with no partial stdout. Neither silently exceeds the envelope, and there are no OOM kills. The instrument publishes each workload's minimum viable grant.
3. **Mandatory.** Every judgment identity carries a derived verdict, and the seal refuses a missing one. Nothing outlives its judgment except through four routes:
   - an engine value released at its last consumer;
   - a ladder-discharged provider entry;
   - a charged process constant;
   - a declared residual with a capability trigger.

   The closed inventory joins the population discovered from both emitted crates with zero unclassified.
4. **Global, default plus override, no local tuning.** One counter, one ladder, one default fold, and zero String demand identities. The override selects only admitted handlers and records a `SelectionReceipt`.
5. **Evidence stays enrolled.** Forced-miss and evict-all runs are byte-identical; there are reclaimability, pressure-case and tier-failure controls; and the enrolled red tests stay after they go green (§4b(4)).

Envelope adherence is rung 1 (typed mitigation) and engine release rung 2. Neither is claimed higher.

## Reproduction of the historical measurements

- **Generation one:** `gunbc test //gunbc/instruments:v2-native-cli` prints the kept `v2-native-cli-gen-one-<sha>` path. Peak and time come from `/usr/bin/time -f "%e %M" <bin> emit --entry nope.x --source-root <root>`; a bogus entry costs exactly as much as a real one, because the census runs first.
- **Probes:** copy the emitted crate while the instrument's cargo build runs (it is deleted afterwards), edit one site, and run `cargo build --release --offline`.
  - Memo off: `materialization_allows_memo_store` returns false.
  - Per-file digest: hash the accepted tree with each node's occurrence resolved to its source extent, comparing structure-only and span-resolved digests between variants.
  - Per-production hits: count in `parse_nonterminal_memoized`'s hit arm.
  - Cross-file retention: no `span_index_merge` and no `ClosureIngestFile` in `closure_ingest_step`.
- **Attribution:** `perf record -e page-faults --call-graph dwarf` (needs `kernel.perf_event_paranoid ≤ 1`) placed about 60% of new heap pages under `span_index_record` during a large-file parse. heaptrack recorded counts and size histograms but did not unwind on this aarch64 host.
- **Milestone evidence:** `native-memory-parse` owns the permanent discriminating parser control; `native-memory-envelope` and the selected retention milestone own bounded allocator/phase receipts. Select or add the concrete instrument with that slice; `//gunbc/instruments:native-memory-baseline` was a proposal, not an existing instrument established by this analysis.
