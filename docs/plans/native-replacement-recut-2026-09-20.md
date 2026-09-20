# Native replacement recut (2026-09-20)

> **Status:** DRAFT FOR OPERATOR REVIEW. Nothing here is a roadmap row yet, and no row, witness or
> generated artifact changes with this file. It records a proposed recut of the public program, the
> one-day observations it rests on, and the decisions still open. Adopting it means editing
> `gunbc.roadmap_authority`; until then this file governs nothing.
> **Authority it answers to:** `DESIGN.md` §3 (*replacement migrations cut over at the root*), §2
> (*minimize the demand graph before materializing its answers*), §4b and §5; `std.realization`,
> `std.materialization_ladder` and `std.decision` for the runtime milestone;
> `v2.std.compilers.compilation_unit` and `v2.workflow.rust_crate_partition` for compilation units;
> `gunbc.v1_maintenance_standing` for what may still be done to the seed.
> **Framing:** a *smaller* native v2 system that replaces the v1 seed and the external session
> manager, with the capabilities that enlarge the retained closure deliberately deferred. It is not
> a v1 modernization program and not a feature-parity port.
> **What this file is not:** a milestone authority. The program's milestones, their prerequisites
> and their standing are declarations (`gunbc.compiler_frontend_program_interlock`,
> `gunbc.compiler_frontend_program_status`, `gunbc.namespace_cut_stage`), read with `where_are_we`;
> this file proposes DELTAS to them and owns none. Nor is it the recut of the private plan (that
> lives in the private overlay), and
> not a measurement authority. Every number below is a dated one-off observation with its method
> stated; none has an entry point, so none is an instrument (`DESIGN.md` §6). Where an instrument
> exists it is named instead.

---

## 1. Why recut

The binding constraint is **verification capacity against the work being generated**, and the
system has been answering it by cutting verification rather than demand.

- **The required floor stopped gating.** Under `--measurement-receipt`, `claim_executor
  --required-ci` writes its blockers into the receipt and returns success, and the workflow emitted
  by `gunbc.compiler_gate_workflow` bound no adjudicator. Landed merge-queue runs on 2026-09-20
  printed `floor refused` and `phases_failed=2` under a green check. Parse failures and a shifting
  set of type errors accumulated on main behind it. No row on main carries this yet: the node
  `required-floor-passes-with-a-failed-phase` is proposed in #11853, open at this writing.
- **The floor's cost is preparation, not claims and not `cargo build`.** The header of
  `gunbc.compiler_gate_workflow` records a run whose claims took under a minute while strict
  preparation and reach probes took most of the job; `gunbc.floor_demand` carries the memory
  receipts. Parallelising claim evaluation addresses a small share of that run.
- **v2 does not change that cost shape as it stands.** `gunbc.rung_drop`
  `v2_native_route_off_the_merge_path` records why the native route left the merge path; the route
  ingests the whole corpus exactly as the seed does.
- **The seed is growing while declared frozen.** `gunbc.v1_maintenance_standing` states its own
  rung: the purpose test is consumed by review diligence, not by any gate. Most recent growth is
  floor and measurement apparatus — the seed growing to police the seed's cost.
- **Demand is the other half.** A one-off query over the workflow's runs on 2026-09-19/20 showed
  roughly ten required runs per landing, because every pushed work-in-progress head fires the floor.

## 2. Objective

> A declared native v2 profile builds and verifies its own retained source, produces a qualified
> native successor, and carries one real daily development task loop. **The retained system derives
> its compilation units from its first supported build, and its runtime owns realization,
> materialization and resource-bounded concurrent execution.**

Two rules bound it.

- **No fallback, in the new system.** A capability outside the retained profile is unavailable
  there; it never resolves through v1 (`DESIGN.md` §3: *Y never falls back to X*).
- **No silent retirement, of the old one.** v1 keeps serving its existing consumers, frozen, until a
  by-name consumer census gives each a disposition. Outside the required gate a dependent can stop
  resolving while every lane reports success, so the census is enumerated, not inferred from what
  breaks.

## 3. Observations of 2026-09-20

Subject: main `31bbd220f9d` **plus the diff of #11875** (the in-body annotation repair), in an
isolated worktree with a freshly built seed. Logs, the applied patch, both module lists, binary
digests and the toolchain identity were preserved outside the worktree by the operator's session.
Instrument used where one exists: `gunbc test //gunbc/instruments:v2-native-cli`. Everything else
was a throwaway script over `module` / `import` lines and the emitted crate; if any of it is worth
re-deriving it is owed an entry point.

1. **Generation one was red on unmodified main.** The instrument refused `EmissionRefused` on the
   annotation parse errors in three files, none of which is in the entry's closure. The seed's
   emission ingests whole source roots. With the repair applied the instrument held: the seed
   emitted the closure and the emitted crate built with zero warnings.
2. **Scoped input reaches a different, relevant refusal.** Over the full roots the built native CLI
   refuses at a `service` declaration in a file outside its closure. Over roots containing only its
   closure it clears that and refuses at a coproduct written with a leading `|`
   (`extdeps.units.iso_80000_3`).
3. **A first-failure census, per file.** Each closure file was offered alone to the native binary.
   Most clear the whole front end and stop at unbound imports, which isolation guarantees. The rest
   stop at: a handful of parse forms (leading-pipe coproduct, positional-payload variant, record and
   generic type aliases, some `fn` / `data` declarations), postfix field access at body lowering,
   two normalization refusals, `infer_grounding_not_derived`, and a group at resolve ambiguity that
   isolation may or may not explain.
   *What this does not establish:* the remaining capability set. These are first observed failures;
   several may share a cause, and repairing one may expose another. Whole-closure resolve, infer and
   emit have never completed, and the small footprint observed belongs to an early refusal, not to a
   successful succession.
4. **Import lines understate the dependency relation.** The seed emitted more modules for this entry
   than `import` lines reach (six more: `v2.std.host_run`, `layer`, `refinement`,
   `subject_evidence`, `test_claim_falsification`, `verdict`), and a substantial number of emitted
   module-to-module links have no `import` line behind them, mostly to `std.types` and
   `v2.std.optional`. The hand-selected population in (2) was therefore incomplete; it did not
   matter only because the run refused at parse first.
5. **The emitted artifact is one package, already one file per module.** Cross-module references are
   uniformly `crate::<module>::…`, and no emitted module implements a trait for another module's
   type. There is exactly one cyclic group — `std.content_hash`, `std.occurrence_identity`,
   `v2.std.node` — and the emitter introduces it (a re-export of `Hash` into `std_content_hash`);
   the `.dag` import lines among the three are acyclic. Ownership wraps in the emitted crate are
   `Rc` throughout; its persistent collections come from the `Arc`-based `im`.
6. **The existing workspace entry is large.** The closure of `gunbc.roadmap_serve` is several times
   the compiler's, contains the `service` and `resource` families, and uses `Filesystem`, `Clock`
   and `Process` effects. That measures the *existing* entry, not the requirement (§4, R3).
7. **Cheap preconditions are checked last.** The instrument discovers an unwritable probe root and
   an unresolvable `rustc` only after paying the whole emission. The local probe root is a fixed
   path in the host's shared temp; `RUNNER_TEMP` and `RUSTC` are the declared selectors.
8. **The native verification lane has an empty positive population at this subject.** The binary
   built by `gunbc test //gunbc/instruments:self-host`, run in its `adjudicate` mode over the full
   roots, refused every identity it enrolled — the majority at `resolve_reason_ambiguous_symbol`,
   then unbound symbols, then the front-end refusals of (3) — and its own terminal reported
   `positive_population_empty` and `required_native_pass_regressed`. A scoped population gave the
   same proportions, and repeating the scoped run with the lane's own invocation shape (relative
   roots from the population root) gave identical counts, so neither the population nor the
   invocation explains it. The same binary's `census` mode continues past per-file refusals and
   rosters the whole front-end wall in one run, which makes it the instrument for that wall.
   *What this does not establish:* when the passes were lost or why. The run is a cost and outcome
   observation, not an admitted receipt: no malformed control was run, and the host facts were
   written by hand from true values with the old route withdrawn for the window.
9. **The serial native run is per-file context ingest.** By the instrument's own
   `[native-cost-partition]`, nine tenths of the full-root run is `context`, with receipt admission
   and universe derivation making up nearly all the rest. Within `context`, normalize costs roughly
   twice parse, and one data roster, `v2.workflow.floor_grandfathered_roster`, sets the critical
   path with a normalize cost far out of proportion to its size — a cost-shape candidate under
   `DESIGN.md` §6 (*bare minimum cost*), stated as a hypothesis. The driver already restricts what
   it ingests to the import closure of the modules it enrolls. *What this does not establish:* the
   cost of prepare and eval. Almost nothing reached them, so both are unmeasured, and the historical
   cost of a passing fold cannot be read off this run.
10. **The front-end wall is attributed, and most of it is grammar rather than corpus.** v2's
   productions (`v2.extdeps.languages.dag` `dag_grammar_*_expr` under the ordered-`Choice`,
   greedy-repeat semantics of `v2.compiler.02_parse`) were mirrored outside the compiler and run
   over the same 6,378 files; the mirror rejects 673 of 673 census parse refusals, over-rejects one
   and under-rejects none. At that agreement each refusal is attributable to a first blocking
   construct rather than correlated with one. The distribution is not what "teach the grammar the
   missing forms" suggests:
   - The **largest single cause is an ordering defect, not a missing feature** — 194 files.
     `dag_grammar_expr_expr` offers `match_expr` before `binary_expr`, so a `match`/`if` block
     heading a `&&` or `==` chain is consumed alone and the operator is orphaned, although
     `dag_grammar_primary_expr_core` already reaches those forms. One reordering, no corpus edits.
   - The **effect and transport declaration families** — `uses`, `service`, `admit_callers`,
     `pattern`, `resource` — are 267 files and **none of them is under `src/v2`**. They do not block
     self-hosting, which settles the sequencing question §3(2) raised: the wall the CLI hits over
     full roots belongs to the corpus the retained profile does not contain.
   - `src/v2`'s own parse refusals are dominated by two cheap items: keyword-spelled binding
     *references* (the declaration and expression sites use different terminals) and the ordering
     defect above.
   - A **leading-`|` coproduct is exactly equivalent** to the accepted spelling, so either side
     works and one optional-pipe row beats editing 87 files; **positional variant payloads**,
     `if` without `else`, and numeric generic arguments have no meaning-preserving rewrite, and the
     last of those is DESIGN §2's own one-axis integer and float projection.
   - Taught in order, the top three forms clear 392 of 673 and all nineteen clear 648.
   *What this does not establish:* the 156 later-stage refusals. Census rows carry no byte offset,
   so postfix field access, wrapper retention and the rest correlate with candidate spellings but
   are not localized; that needs offsets the census does not emit.

## 4. The program is already carried; this document proposes deltas to it

**An earlier revision of this file carried a milestone table of its own (R0…R4). That table is
withdrawn, and its withdrawal is the point of this section.** The program it described already
exists as declarations, with a prerequisite relation and derived standing:

- `gunbc.compiler_frontend_program_interlock` `ProgramMilestone` — fifteen arms across the
  self-host and namespace programs — and `milestone_prerequisites`, which is the sequencing.
- `gunbc.compiler_frontend_program_status` `milestone_status` — `Clear` / `Outstanding` /
  `NotDerivable` per arm, with `MissingInstrument` naming what would settle an underivable one and
  `InstrumentObstacle` separating *not built* from *refused by authority* from *refusal falsified*.
- `gunbc.namespace_cut_stage` — the same shape applied to the namespace cut, serial, standings
  derived.

A second milestone vocabulary beside those is a §3 fork: one program, two names, consolidated later
at interest. So this document owns no milestones. **What the frontier is, is read — not restated
here** (`DESIGN.md` §6, *name the instrument*): `gunbc.compiler_frontend_program_status`
`where_are_we`, run against a built seed, derives the standing, the startability of each arm, and
the blocker set where it is derivable. A reading taken on 2026-09-20 is in §3's evidence
directory; a number copied into this prose would rot away from the carrier that owns it.

What this document adds is four things, and each is a change to those carriers rather than a
parallel program.

### 4a. Two arms that do not exist

| Proposed arm | Why it is not covered by an existing arm | Prerequisite |
|---|---|---|
| **derived compilation units** | The self-host arms take the emitted artifact as given. None of them asks whether the retained module population, its declaration interfaces and the target's constraints *produce* the unit assignment that is actually built. `v2.std.compilers.compilation_unit` `CompilationUnit` and `v2.workflow.rust_crate_partition` `PartitionPolicy` are the homes; today the policy-derived arm returns `PolicyResolverUnimplemented`, and `self_host/stage0_crate_layout` and `stage0_executable_assembly` carry the hand-assigned precedent. Its acceptance: every retained module owned by exactly one unit, the emitted workspace built *from* that assignment, a missing interface or invalid cross-unit dependency refusing rather than falling back to one package, and a localized edit shown to rebuild its owner rather than everything. | `SelfHostCorpusEmitsCleanly` |
| **runtime realization and materialization** | No arm covers the runtime's own concurrency and reuse. Its acceptance is the demand lifecycle, not a benchmark: independent ready demands overlap; a dependent proceeds when *its* prerequisite completes; concurrent requests for one admitted pure computation share a producer or its result; unchanged inputs reuse a qualified value and changed semantic inputs invalidate it; execution and retained values fit the admitted envelope with cancellation accounted; required refusals still agree with the reference and ordered effects are not parallelized by accident. Homes: `std.realization`, `std.materialization_ladder`, `std.decision`. | none — it is startable |

Its first target is named by §3(9): per-file context ingest, which is nearly all of the serial
native run and is independent across files. The same grain is the natural unit of materialization.

### 4b. One arm re-cut, per the 2026-09-20 operator ruling

`NamespaceWaveAdmissionEnrolled` is today satisfied only by a **merge-time wall**. That wall was
built, refused every merge_group run on a clean floor, was deleted, and is now
`gunbc.rung_drop` `namespace_wave_admission_wall_removed` — which the milestone reads, so the
namespace waves stay unreleased until a wall exists again. The ruling is that the wall is the wrong
mechanism: the delta should be established the way the self-host frontier establishes its own
standing — **a receipt produced on demand, off CI, by the stage carrier that already exists** —
rather than by a gate that charges every unrelated change at landing time. So the arm is re-cut to
be satisfied by that receipt, and the drop retires against that capability instead of against a
resurrected gate. `gunbc.namespace_cut_stage` is already that shape and already reports; what it
does not yet do is discharge this arm.

### 4c. Three subjects that are not compiler-frontend milestones

They were in the withdrawn table and do not belong in this carrier at all. Each goes to the lane
that owns it, and none of them gates the arms above:

- **the truthful required gate** — the floor concluding success while refusing (§1) is an incident
  with repairs already open, not a milestone. Its home is the `ci-control` lane. It is a
  precondition for *trusting* any measurement above, which is why §3 states its subject exactly.
- **safe development execution** — one designated, resource-admitted executor for every session's
  build and test. Home: the `compute` rows (`compute-exact-work-contract`,
  `compute-artifact-return-and-materialization`, `compute-deduplication-and-admission`).
- **the minimum daily workspace** — home: the roadmap lane
  (`roadmap-serve-emitted-realization`, `roadmap-verify-to-publish-phase`).

### 4d. Observations recorded as standing, not as prose

§3's findings are facts about arms that already exist, and belong on them:
`SelfHostNativeRouteRequiredLane` is outstanding for a reason it does not yet carry — the lane's
positive population is empty, every enrolled identity refusing — and the front-end refusal census
bears on the emit arms. Recording them as `MilestoneEvidence` is what keeps `where_are_we` the
single place the frontier is read.

## 5. Sequencing notes

- **The carrier already answers one question this document was about to get wrong.**
  `milestone_prerequisites` makes `NamespaceFixForwardComplete` require `SelfHostSeedRetirement` —
  the last arm. Read literally, namespace fix-forward cannot begin until the seed is gone, which
  contradicts the intent of starting namespace early so it does not drag across a growing retained
  set. Either that prerequisite is right and the intent is wrong, or it encodes an assumption the
  re-cut of 4b changes. It is a decision (§7), and it was invisible while the program lived in
  prose.
- **Finish one successful whole-closure emit before the derived-unit arm can be judged.**
  Interfaces, rebuild behaviour and the memory envelope all need a successful emission to measure
  against. The one-package artifact stays a bootstrap and measurement probe; it never satisfies 4a.
- **Scoped ingest needs a sound population boundary.** §3(4) is the reason: `import` lines are not
  the dependency relation the build has. The identity-level home is `v2.std.dependency`
  `DependencyRelation<Subject>`; `DependencyView` carries `Node`s, which is the tree scoped ingest
  exists to avoid building. "A file outside the closure changes nothing" holds only for a file
  proven outside the admitted dependency **and name-resolution** domain — a competing declaration
  must not be ignored because yesterday's graph had no edge to it. That is also why the namespace
  cut and materialization meet: reuse cannot key on yesterday's edges once the namespace facts that
  justified them have changed.
- **The admitted source universe is a choice.** Either the whole repository, in which case the
  declaration index must be total over forms v2 cannot yet parse or the refusals return through the
  index; or the retained roots physically hold only the retained population.
- **Native effects arrive with their first consumer** — directory listing and typed read failure
  with succession, process spawn with the runtime arm, publication and credentials with the
  workspace — not behind one late ruling. No second handwritten semantic implementation is
  admitted; a line budget on the seed exposes growth and authorizes nothing.

## 6. Roadmap mechanics

- **A pause must bind dispatch.** `roadmap_focus_selection` filters `ROADMAP.md` only; its own
  annotation names the missing dimension. `gunbc.roadmap_launch_admission`, `gunbc.roadmap_page`
  and the spawner must read the new field or the recut is a display edit. `Unsized` is not a pause.
- **Keep three facts apart:** scheduling (continues / draining / paused with a reason and a restart
  obligation), evidence (a hypothesis falsified, with its receipt), and operations (a service keeps
  running with no new development admitted).
- **Drain means review, not land.** Each outstanding change is judged against the retained program:
  land what qualifies, preserve and close what is obsolete, withdraw what its evidence does not
  support.
- **A restart needs an unmet obligation of a retained consumer and an activation decision.**
  An arm reaching `Clear` reopens nothing by itself, and naming a consumer is not enough.
- **Roadmap rows cite arms rather than restating them.** A row whose acceptance paraphrases a
  milestone is the same fork this section's §4 withdrew, one layer out.
- **Integration.** One owner lands the colliding wind-down changes to `gunbc.roadmap_authority`
  serially, and repairs the `roadmap_authority_test` witnesses that are red and outside the gate in
  the same pass. Accepted rows are extended with `updated(...)`; editing a brief, red control or
  handback un-accepts the row.

## 7. Open operator decisions

1. **The namespace prerequisite** (§5, first bullet): does `NamespaceFixForwardComplete` keep
   `SelfHostSeedRetirement` as a prerequisite, or does 4b's re-cut change it?
2. The admitted source universe (§5).
3. Grammar or corpus, now answerable per form rather than in aggregate (§3(10)): the roster says
   grammar for all but about thirteen files, one cause is a defect rather than a feature at all,
   and the large declaration families can be deferred without blocking self-hosting. What remains
   yours is whether the forms with no meaning-preserving rewrite are admitted or designed out.
4. Where the designated executor of 4c runs: a retired CI slot, or a host whose envelope is
   established first.
5. Who integrates `gunbc.roadmap_authority`.

## 8. Not claimed

No duration for any arm: each is unmeasured beyond the first failures of §3. No claim about the
cost of native prepare or eval, which §3(9) could not observe. No claim that generation two works,
that the native build's memory envelope is known, that any speedup figure bounds the runtime, or
that the first-failure census is complete. No standing is asserted here for any milestone — the
carrier derives it, and this file names the reader rather than copying its output.
