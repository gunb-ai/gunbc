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
> **What this file is not:** the recut of the private plan (that lives in the private overlay), and
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

## 4. Milestones

Rows named in the last column exist in `gunbc.roadmap_authority` today and are extended rather than
re-minted. A citation of a row is not a claim that the row's current text requires what the
milestone does; where it does not, the row's first slice is what changes.

| Milestone | Acceptance | Existing rows |
|---|---|---|
| **R0 — Truthful gate** | On the real `floor` job, a planted-red candidate concludes failure and a clean one concludes success, and a run on main evaluates claims. One controlled candidate carries the corpus repair and the gate repair together, with ordinary queue intake paused. Receipt mode's lost-blocker defect is repaired wherever that mode remains an acceptance authority, or that use is withdrawn. | the `ci-control` lane; the node proposed in #11853 once it lands |
| **R1 — Safe development execution** | Any existing session submits an exact build/test subject to one designated, resource-admitted executor and receives its result and artifacts without running the heavy operation itself. Exact subject transport (including across git object stores), preflight before expensive work, per-request scratch, admission by measured envelope rather than a fixed slot, shared producers for equivalent requests, honest and distinguishable completion, artifact identity, cancellation and cleanup. A weaker-than-microVM isolation is stated as such. | `compute-exact-work-contract`, `compute-artifact-return-and-materialization`, `compute-deduplication-and-admission`, `ci-owned-execution`, `ci-remote-build-containment` |
| **C — Derived native compilation units** | The retained module population, its declaration interfaces and the target's constraints produce the compilation-unit assignment; that assignment produces the actual Cargo workspace, manifests, dependency edges and cross-crate references that are built. Every retained module has exactly one owning unit and the build uses those units. A missing interface, an invalid cross-unit dependency or an invalid layout refuses — no monolithic fallback. A no-change rebuild and representative localized edits show the real reuse. The successor reproduces through the same mechanism. | `v2-emitter-native-crate-partition` (its out-of-scope clause, which lets succession run on the one-package probe, is what this milestone overrules) |
| **R2a — Native succession** | The native compiler emits and builds its retained closure **through C**, and the result does so again with no seed available; a missing native ancestor refuses. The entry's population is produced inside the fold by a reference-derived relation with a stated validity condition on the index that selects it. Behavioural discriminators, not only "the output builds twice". Directory listing and typed read failure land here, because succession is their first consumer. | `v2-emitter-native-bootstrap`, `v2-emitter-production-compatible-corpus-module` |
| **R2b — Native verification** | The supported witness population is derived from the retained system's obligations and runs natively as a required lane, with must-fail controls, retiring `v2_native_route_off_the_merge_path` for that scope. A regex count of contained witnesses is a sizing observation, never the denominator. Narrowing may not discard a failing witness for behaviour the retained system still claims. | `v1-verification-ledger`, `v1-ci-floor-cutover`, `ci-cost-floor-preparation-cut` |
| **P — Runtime realization and materialization** | Starts now; delivery is judged on a real consumer. Independent ready demands overlap in execution; a dependent proceeds when *its* prerequisite completes, not when an unrelated one does; concurrent requests for one admitted pure computation share a producer or its result; unchanged inputs reuse a qualified value, changed semantic inputs invalidate it, and a missing or corrupt materialization never becomes success; execution and retained values fit the admitted envelope, with cancellation and release accounting for other consumers; results and required refusals agree with the reference, and ordered or unauthorized effects are not parallelized by accident. The three-arm measurement (in-process; one child; n children) is evidence toward this, not the acceptance. Process spawn lands here. | `docs/plans/native-route-parallel-realization.md` (a candidate realization), `compute-deduplication-and-admission`; qualify `gunbc.floor_materialization` before authoring another store |
| **N — Namespace replacement** | The shared correctness repairs start now: calls survive v2 body lowering, and resolve supplies the actual binding rather than a spelling-based substitute. The retained semantic cut follows the first native foothold and precedes any widening of the retained set, so each module admitted afterwards migrates once. Reference binding, dependency selection, evaluation and emission stop taking answers from the import-based authority; `import` refusing at parse is the last step, not the proof. | the thirteen `namespace-*` rows, re-scoped from whole-tree to retained-profile-first |
| **R3 — Minimum daily workspace** | One real task proceeds through a native-backed loop: open, understand constraints, continue the candidate, request admitted verification, inspect the exact result, publish, resume after interruption. Its closure is derived from that loop, not inherited from the existing serve entry. Publication and credential effects land here. Existing sessions consume the task / attempt / result interface before the surface exists, and journaling counts only where it transfers authority. | `roadmap-serve-emitted-realization`, `roadmap-verify-to-publish-phase`, `roadmap-receipt-continuity`, plus one new row binding a session and its pull request to a node through `gunbc.roadmap.roadmap_event_log` |
| **R4 — Predecessor retirement** | The by-name consumer census of the interpreter and of the session manager, and the preservation of the session manager's state, start now. A dependency is deleted when its retained replacement qualifies; an obsolete product is deferred or removed rather than ported. Undated by design. | `v1-interpreter-quarantine`, `v1-interpreter-delete`, `v1-products-v1-free`, `v1-zero-hand-maintained-rust`, `ci-actions-runner-retirement` |

The session manager's feature growth is frozen for the duration: security, loss prevention, state
export, and hand-off onto the new execution and task interfaces only.

## 5. Sequencing notes

- **Inside R2a, finish one successful whole-closure emit first.** Interfaces, rebuild behaviour,
  the memory envelope and the behavioural discriminators all need a successful emission to measure
  against. The one-package binary remains a bootstrap and measurement artifact; it never satisfies
  C or R2a.
- **C's first policy needs no selection.** One unit per cyclic group of the reference-derived unit
  graph is derivable without choosing anything, and §3(5) found a single group of three. The
  unimplemented policy arm of `v2.workflow.rust_crate_partition` (`PolicyResolverUnimplemented`)
  therefore does not block the first build; coarsening is a later `std.decision` over measured axes.
  Crate-level build concurrency, `codegen-units`, and runtime concurrency are three different facts.
- **Scoped ingest needs a sound population boundary.** The identity-level home is
  `v2.std.dependency` `DependencyRelation<Subject>`; `DependencyView` carries `Node`s, which is the
  tree scoped ingest exists to avoid building. "An unparseable file outside the closure changes
  nothing" holds only for a file proven outside the admitted dependency **and name-resolution**
  domain — a competing declaration must not be ignored because yesterday's graph had no edge to it.
  Reuse cannot key on yesterday's edges when the namespace facts that justified them have changed.
- **The admitted source universe is a choice.** Either the whole repository, in which case the
  declaration index must be total over forms v2 cannot yet parse or the `service` wall returns
  through the index; or the retained roots physically hold only the retained population.
- **P's first consumer is the native witness fold** — wide, pure, independent claims over shared
  preparation — and its second is R1's executor. Line-weighted speedups computed over the import
  graph are proxy estimates of module-grain preparation and bound nothing about runtime demand
  graphs. Fork-after-context is a candidate, not the contract: an `Rc` clone writes its count, so
  reading shared context in a child dirties copy-on-write pages, and the measurement is shared and
  private-dirty memory rather than per-process resident size. `Arc<T>` does not make `T` safe.
- **Native effects arrive with their first consumer** (listing and typed read failure with
  succession, spawn with P, publication and credentials with R3), not behind one late ruling. No
  second handwritten semantic implementation is admitted; a line budget on the seed exposes growth
  and authorizes nothing.

## 6. Roadmap mechanics

- **A pause must bind dispatch.** `roadmap_focus_selection` filters `ROADMAP.md` only; its own
  annotation names the missing dimension. `gunbc.roadmap_launch_admission`, `gunbc.roadmap_page` and
  the spawner must read the new field or the recut is a display edit. `Unsized` is not a pause.
- **Keep three facts apart:** scheduling (continues / draining / paused with a reason and a restart
  obligation), evidence (a hypothesis falsified, with its receipt), and operations (a service keeps
  running with no new development admitted).
- **Drain means review, not land.** Each outstanding change is judged against the retained program:
  land what qualifies, preserve and close what is obsolete, withdraw what its evidence does not
  support.
- **A restart needs an unmet obligation of a retained consumer and an activation decision.**
  Completing R2b reopens nothing by itself, and naming a consumer is not enough.
- **Proposed dispositions.** Continue: `compute`, `ci-cost`, `ci-control`, the `self-host` rows named
  above, `roadmap`, `roadmap-runtime`, `harness`, `generated-artifact`. Continue re-scoped:
  `namespace`. Draining: the primitive-egress changes already open. Operating without investment:
  serving liveness and runner convergence, on which the harness and CI depend. Paused:
  `compiler-guarantee` except rows a retained obligation names, `source-intent`, `shell`,
  `toolchain`, the dashboard presentation rows, the small tooling lanes, and new fleet and serving
  work.
- **Integration.** One owner lands the colliding wind-down changes to `gunbc.roadmap_authority`
  serially, and repairs the `roadmap_authority_test` witnesses that are red and outside the gate in
  the same pass. Accepted rows are extended with `updated(...)`; editing a brief, red control or
  handback un-accepts the row.

## 7. Open operator decisions

1. Where the designated executor runs: a retired CI slot, or a host whose envelope is first
   established.
2. The admitted source universe (§5).
3. Whether v2's grammar admits the small parse forms of §3(3) or the corpus is normalised; the seed
   admits them today.
4. Whether "derived from the beginning" means from the first *supported* native build, as §5 reads
   it, or earlier.
5. Who integrates `gunbc.roadmap_authority`.

## 8. Not claimed

No estimate of duration for C, R2a, R2b, P or R3: each is unmeasured beyond the first failures
above. No claim that generation two works, that the native build's memory envelope is known, that
any speedup figure bounds the runtime, or that the first-failure census is complete.
