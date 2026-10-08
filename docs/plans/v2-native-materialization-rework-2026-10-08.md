# v2 native memory and materialization: gap analysis and rework plan

> **Status:** Draft for operator review, 2026-10-08. No source change. This replaces this PR's first revision, which proposed two unsound changes (see "Corrections to the first revision").
> **Authority it answers to:** DESIGN §2 (minimize demand before materializing), §3 (delete-first replacement), §3b (conformance), §4b (rung honesty), §5 (fail-closed), §6b (earliest unjustified boundary). It would amend docs/plans/demand-engine-program.md (a new D15) rather than stand beside it.
> **Framing:** The operator wants materialization mandatory and global on the native path. That means a default policy, overrides that cannot bypass accounting, and one envelope shared by execution and caching (for example 1 core, 1 GiB). It also means online hit/miss-driven tiering, computation identity separate from content identity, and no module-local tuning. Every figure below is a dated observation from srv2 (aarch64) at main `6d82ac7721`/`c48092a621`, made with measurement-only edits to a copy of the emitted crate. None is an instrument yet; "Reproduction" says how to re-derive each, and items P1.1 and C2.3 turn them into instruments.

## Answer

The first revision's year-long figure bundled three projects into one estimate:
- fixing incorrect parsing;
- removing avoidable retention;
- building the global materialization system.

It also priced them at historical landing times with weeks of ruling latency. The operator ruled on 2026-10-08 (see "Operator rulings"), and the plan below delivers in checkpoints:
- the parser fix in days;
- a resource envelope in weeks 2–4;
- bounded retention, then a first complete governed example, by about weeks 6–8.

Each checkpoint is measured and the remainder re-estimated before the next is funded. The full program (Phase 5's tiers and identity, value-grain governance) stays provisional.

"Mandatory" is read as a mandatory decision, which may be "do not retain", not as mandatory caching. Every allocation is charged to one budget regardless.

Exact release and tiering solve different problems:
- release data when nothing needs it again;
- spill or recompute data that is still needed but cannot stay resident.

Exact release needs a graph closed over every consumer, which has to be established, not assumed. Tiering can matter within one run as well as across runs.

## What is broken today

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
- **The tail is real.** 103 files (1.3%) parse more than 1.5× slower without the memo, adding 7 s in total against the 212 s saved (worst case +265 ms). The cause is not match arms, which are less dense than average in those files. It is unexplained, and this is why the plan keeps a replacement bound for packrat's worst case (P1.2).
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

1. **Occurrence ids are not monotone.** Backtracking re-mints them, so the proposed append-only span index is unsound on its own. It also needs a monotone allocator. That version was prototyped by a reader: zero location mismatches, 14 of 15 existing claims green. It is now only the fallback (P1.5); deletion is the primary route.
2. **The census must stay total.** Under the default `v2.std.resolution_policy` `NamespaceOnlyY`, a bare name may bind anywhere, and `closure_emission` states that the census is TOTAL for that reason. "Census only the entry's closure" is withdrawn. The cut is to keep each file's declaration projections, not its trees and spans (B3.5, B3.6).
3. **Each memo snapshot pins the span index for the current file so far, not the corpus so far.** Every parse starts from `span_index_empty`; only the allocator crosses files.
4. **The empty-index probe was not a proof of semantics.** Its byte-identical CLI output only attributed the memory: spans for replayed ids were silently dropped.

## Gap analysis

| Requirement | Have today | Gap | Size |
|---|---|---|---|
| Mandatory: every derived computation's retention is governed by a decision | Only the parse memo consumes a verdict, once per grammar. 25 retention mechanisms across both native closures; 13 ways to bypass governance | A derived verdict per judgment kind, with the seal refusing a missing one; a closed inventory census, executed | L |
| Global: both binaries, every route | The CLI holds every file's trees and spans to exit. The engine runs only in `adjudicate`. Rendered mains decide release in Rust strings | One allocator counter in every native main; CLI judgments keyed by `std.judgment_contract` `DemandIdentity`; one default fold | L |
| A default policy with no annotations | No policy carrier; capacity literals on each provider; `admitted.first()` | One total default fold in `std.materialization_ladder`: release at the last consumer, recompute only with a typed warrant, `Discharged` only for unavoidable recurrence | L |
| Override that cannot bypass accounting | No run-level carrier | A run-root override selecting among admitted handlers through `std.decision` `select_realization`, with no field for a grant or capacity | M |
| One envelope shared by execution and caching | OS cgroups only; the native binaries bind nothing | Grant resolution via `gunbc.host_budget_source` `effective_planning_budget` (narrow-only); a live-heap counter; a typed wall; spawners bound to the same grant; typed refusal at safe points | L–XL |
| Bounded retention where it dominates | CLI cross-file retention (about 70% at 8.5 MB); memo snapshots; engine whole-run retention | Delete the memo; interval table instead of the merged span index; projections instead of trees; context carry; exact last-consumer release | L–XL |
| Online hit/miss policy with RAM → disk tiers | LRU only (O(n) per get in `std.artifact_store`); the disk store is witness-only on main | Store on the native route; reuse-distance instrument; S3-FIFO as its own cited extdeps handler; admission-filtered demotion | L–XL, gated on M1.c |
| Computation identity separate from content identity | `EvaluationIdentity` / `ResultContentIdentity` / `EvaluationResultBinding` are witness-only. SHA-256 already executes natively (`//gunbc/instruments:native-crypto-vectors`) | Mint native hashing for identity (measure first); split identity for one family using the §2.8 manifest protocol | L |
| Modules never tune caching locally | Literal capacities (EvalCallMemo 1M, CrossClaimPureMemo 256 MiB, family budgets, unbounded ParseTable) | Delete inert carriers; capacities only from the run grant and `gunbc.materialization_store_budgets` | M |

## Plan

| When | Deliverable | Exit evidence |
|---|---|---|
| Days 1–3 | The parse demand is priced below the cost floor (`Recompute`) through the ladder's own roster (v2-only, no stage0). The tests that pinned `Memoize` are updated, and a colliding-grammar regression asserts distinct ids and the expected span, independently of any memo | The new claim is red with the old verdict and green with the new; the CLI accepts `src/v2/std/logic.dag` |
| Week 2 | Delete the memo carrier atomically, with its seed half: driver memo counters and the interpreter arm | Nothing references the deleted symbols; the regression control stays enrolled |
| Weeks 2–4 | Envelope: one grant per run (cgroup bound, or a declared default ceiling, narrowable only); a counting allocator in both native mains; a wall that allocates nothing, with reserved headroom; typed refusal at per-file and per-demand safe points; a cgroup backstop; pressure tests | Planted tiny grants refuse typed; no OOM kills; overhead measured |
| Weeks 3–6 | Retention: carry the resolution context; typed fault on an absent payload; driver readers moved into the graph; release at the last consumer where coverage is established; CLI keeps declaration projections and an interval table instead of trees and the merged span index | Per-phase allocator receipts; **re-estimate here** |
| Weeks 6–8 | The first complete governed example: one native workload with in-run reuse under memory pressure, completing within its grant under the default policy with no module tuning. Plus practical mandatory governance: capacity-checked ladder admission, no capacity literals on the native path, and a census claim that reds when a new private cache appears | The named workload completes at its grant; the census is green |

Acceptance criteria name three populations separately:
- workloads that must complete within 1 GiB;
- workloads permitted to return a typed resource refusal;
- the guarantee, for both, that nothing silently exceeds the envelope.

Prompt refusal alone does not pass. Even with the memo gone, the census still refuses 134 files on the real roots (genuine parse and lowering refusals), so a full-roots CLI run is not yet a must-complete workload.

Deferred, each with its trigger:
- an online replacement policy, until a cache with measured reuse distance exists;
- spill of live data, until a measured workload's live set exceeds its grant;
- computation and content identity, and handles, until M1.c's identity contract lands;
- compiler-derived value-grain boundaries, as the follow-on to judgment-grain governance.

## Operator rulings (2026-10-08)

- **R2.** The parse memo is priced `Recompute`, then deleted. Packrat's linear-time worst case is given up; the parser stress tests remain as regression guards, not as a claimed bound.
- **R5.** With no observable OS limit, a run uses a declared default ceiling that a request can only narrow.
- **R4/R3, scoped seed exception.** Allowed: the allocator counter and wall rendered only into the native mains (no new public `v1_rt` item, nothing shipped into other seed binaries), a live-heap reading at the composition root, removal of the memo counters, and the driver's reader moves.
- **R1, practical first.** Every allocation is charged to one budget. A cache exists only through the ladder, enforced by a census claim. The default is release at last use or at end of scope. Compiler-derived boundaries come later, with their own trigger.

Still open: R6 (share restoration), R7 (engine ready order), R8 (`NativeTestContext` derivation), R9 (store budgets under one envelope), R10 (native hashing), R11 (CI placement) and R12 (owners).

## Do not do

- Do not put any memo under a capacity, LRU or eviction controller. Delete it unless the comparison gate falsifies deletion.
- Do not narrow the CLI census, and do not adopt an append-only span index without a monotone allocator.
- Do not render the counting allocator or new public builtins into `v1.compiler.runtime_rust` / `v1_rt`. They would ship into every seed binary.
- Do not let .dag code, an environment variable or a flag arm, disarm or widen the wall. There is no "envelope off" mode (§5).
- Do not report an OOM kill, throttle, abort or timeout as a typed refusal.
- Do not use LRU or S3-FIFO for within-run retention when the sealed demand graph knows the last consumer. Do not pick a replacement policy before reuse evidence exists, and do not add it as a new arm of `std.cache_interface` `ReplacementStrategy`.
- Do not derive per-entry byte weights under `im`/`Rc` sharing. Measure at the allocator and re-measure after each release.
- Do not reuse `product.capacity.pool`'s durable lease kernel or per-put encumbrance entries in-process.
- Do not put a full-roots run (about one CPU-hour) on any per-PR gate, and do not add a CI job without sign-off.
- Do not run two stage0-regenerating PRs concurrently.

## Definition of done

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

## Reproduction

- **Generation one:** `gunbc test //gunbc/instruments:v2-native-cli` prints the kept `v2-native-cli-gen-one-<sha>` path. Peak and time come from `/usr/bin/time -f "%e %M" <bin> emit --entry nope.x --source-root <root>`; a bogus entry costs exactly as much as a real one, because the census runs first.
- **Probes:** copy the emitted crate while the instrument's cargo build runs (it is deleted afterwards), edit one site, and run `cargo build --release --offline`.
  - Memo off: `materialization_allows_memo_store` returns false.
  - Per-file digest: hash the accepted tree with each node's occurrence resolved to its source extent, comparing structure-only and span-resolved digests between variants.
  - Per-production hits: count in `parse_nonterminal_memoized`'s hit arm.
  - Cross-file retention: no `span_index_merge` and no `ClosureIngestFile` in `closure_ingest_step`.
- **Attribution:** `perf record -e page-faults --call-graph dwarf` (needs `kernel.perf_event_paranoid ≤ 1`) placed about 60% of new heap pages under `span_index_record` during a large-file parse. heaptrack recorded counts and size histograms but did not unwind on this aarch64 host.
- **Becoming instruments:** P1.1 makes the memo comparison an instrument with pre-registered predictions; C2.3 adds `//gunbc/instruments:native-memory-baseline` for the per-phase memory receipts.
