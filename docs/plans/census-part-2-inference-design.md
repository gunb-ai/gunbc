# Census part 2 — inference (and emit) census below resolve

**Status: design only.** The build waits until census part 1 lane A lands: #13026,
deep-badger-684, reconciled here against its final head `9b7d2adc5c`. This doc does not mint a row
format. A's member already carries a stage, so part 2 **adds two arms to A's stage coproduct** and
adds no new field. Items marked **[lane C]** are a declared frontier, because lane C has not been
dispatched (§8).

Governing sections: DESIGN §3 (one row format, not two), §3c (every declaration added here names its
consumer), §4b (rung honesty at a declared subject grain), §5 (a refusal is a row, never a widened
pass), §6 (name the instrument; never transcribe its output).

## 1. What part 1 already establishes (reused, not restated)

- **The population authority.** `v2.compiler.compile` `native_census_modules` maps an ingest to
  `NativeCensusModule { module, path }`, one row per file that declares a module line.
- **The resolve decision.** `native_census_module_resolution` returns one of
  `NativeCensusModuleFileRefused | NativeCensusModuleResolveRefused { first, rest, observation } |
  NativeCensusModuleResolved`. The lane and the census both read this one decision.
- **The residual grain.** `NativeCensusResidualRow { module, path, chain }` has one row per failure
  chain, so one module can produce several rows. The fatal is *derived* with
  `v2.std.diagnostic diagnostics_fatal(chain)` and is never stored.
- **The file-refusal row.** `v2.compiler.native_test_vocabulary` `NativeTestFileRefusal { path,
  module, head_reason, fatal_reason }`. The rendering in swift-lynx-592's #13005 groups by **fatal**
  and treats **head** as an advisory field, because `parse_grammar_choice_overlap_residue` heads
  every file.
- **Locus.** swift-lynx-592's path renders a chain's located fatal through
  `lens_verdict_diagnostics_located_chain_text_in_spans`. Part 2 uses that renderer and adds no
  second one.
- **Lane A (#13026).** The carrier is `v2.compiler.compile` `NativeCensusCauseMember
  { stage: NativeCensusCauseStage, path, module, head_reason, … }`, grouped into
  `NativeCensusCauseGroup { fatal_reason, members }` by `native_census_cause_groups_add`. The
  partition check is `native_census_cause_partition_holds`. The stage coproduct is
  `NativeCensusCauseFrontEnd | NativeCensusCauseResolve`. The instrument row is
  `//gunbc/instruments:v2-native-census` (`V2NativeCensusProducer`), and it runs the driver verb
  `census-resolve`. **Line:col is not part of A.** It waits on swift-lynx-592's path, which is held
  on #12506, so it is a dependency of this doc (§8) and not an A field.

## 2. Subject population

The population is **every module that `native_census_module_resolution` answers
`NativeCensusModuleResolved`** over the same ingest part 1 censused. The ingest is the same source
roots at the same revision. Below that boundary, a module is a subject of:

1. **infer**: `native_test_infer_resolved`, that is `infer_and_discharge`, over the module's
   `ResolvedTree`.
2. **emit**: `compile_inferred_translate`, that is `v2.compiler.emit` `emit`, over the
   `InferredTree` for the target the instrument row declares. Only modules that infer are subjects
   of emit.

The population is **derived, never re-enumerated**. Part 2 reads part 1's per-module decision and
filters to the resolved arm. A module that part 1 left in the file-refused or resolve-refused arms
is **not a part 2 subject**. It still appears in the receipt as an `UpstreamRefused` disposition
(§5), so the identity join over `native_census_modules` stays complete (DESIGN §5: completeness is
an identity join, not a count).


### 2a. The infer demand is shared (sharp-raven-357 ruling)

The per-module infer step is **one demand subject in the native drain**, not a walk owned by this
census. The working name is `NativeInferCensusSubject { entry }`. It resolves via
`native_demand_resolved_tree`, then runs `native_test_infer_resolved`, and yields
`Inferred { InferredTree } | InferRefused { chains }`. Two folds read that one result:

- This census reads the `InferRefused` arm: one row per chain, with cascade attribution (§4).
- M0 of the nominal-type plan (gunbc#13024) reads the `Inferred` arm: type declarations and
  `as` sites. On `InferRefused`, M0 records each of the module's `as` sites as
  `OperandTypeUndecided` with the infer cause, counted and never dropped.

Each fold measures a different fact, so each keeps its own line kind (refusal rows vs
classification rows) in one stdout. They share the subject. They do not share a row format.

The driver verb is **`census-infer`**. Widening `census-resolve` would make that word mean two
things, and §3 forbids a meaning fork. `census-resolve` stays part 1's verb. Its resolve rows and
`census-infer`'s rows are the one carrier of §5, distinguished by `stage`.

## 3. What a refusal at each stage means

| stage | refusal means | row stage |
|---|---|---|
| file (front end) | The file never parsed into a module. This is part 1's `NativeTestFileRefusal` and is not re-reported. | — |
| resolve | Some reference in the module did not bind. Part 1 reports this. | — |
| **infer** | The module resolved, but `infer_and_discharge` refused. Causes include no grounding derived, a fact-lookup miss, incoherent canonical grounding, a branch-shape mismatch, or an undischarged obligation. Each is a located `Diagnostic` from `v2.compiler.infer`. | `Infer` |
| **emit** | The module inferred, but the target realization refused. The refusal is the target's inability to realize a semantically legal program (DESIGN §4: the target never decides legality). | `Emit` |

**Part 2 needs a discriminator that does not exist yet.** The native lane folds resolve and infer
into one `NativeTestStagePrepare` (`native_test_prepare_module` binds them). A census whose stage
column read `Prepare` would attribute an infer refusal to resolve, which is the fabricated
attribution that the `NativeLaneModuleResolution` comment already refuses at the context/resolve
split. So the census does **not** read the lane's `Prepare` verdict. It calls the two halves
separately, using part 1's resolved tree, and stamps the stage from which call refused. Whether
`NativeTestStagePrepare` should itself be split into `Resolve | Infer` is a lane-vocabulary change
that is out of scope for this doc. It is flagged here because the census makes that conflation
visible.

An infer refusal is also **per chain**, as it is for resolve. If `infer_and_discharge` returns
several `NonEmptyDiagnostics`, the module yields one row per chain. If infer stops at the first
fatal, the row set for that module is a **lower bound**. The receipt says so with the same
`ObservationCompleteness` arm resolve uses (`ObservationIncomplete { reason }`), and the census does
not report it as complete.

## 4. Cascade attribution

Part 1 lane C collapses import cascades to their roots. If module M fails to resolve only because an
import target T was refused, the row belongs to T and M is a cascade member.

Below resolve, the same structure applies along a different edge. **Infer runs per module over a
resolved tree whose imported declarations come from other modules.** When M's infer refuses at an
occurrence whose fact is owned by an imported declaration in module T, and T itself refused infer,
M's row is a **cascade** of T's root row.

- **The attribution edge.** The refused fatal's locus resolves to a declaration. If that
  declaration's owning module is not M and that owning module has its own infer row, M's row is
  attributed `CascadeOf { root: <T's row identity> }`. Otherwise it is `Root`.
- **The rule is reused, not reinvented.** Lane C's collapse rule (root = earliest refused module on
  the import path) is applied with the infer-stage row table substituted for the resolve-stage one
  **[lane C]**. Part 2 adds no second cascade authority. If lane C's rule cannot take a stage
  parameter, that rule is the thing to generalize.
- **Cross-stage cascades are not collapsed.** A module whose import target refused at *resolve* is
  not an infer subject at all (§2), so it never produces an infer row to attribute.
- **Emit cascades.** Emit runs over one module's inferred tree. A cascade at emit is only possible
  if emit reads another module's emitted artifact. Until a measurement shows such an edge, every
  emit row is `Root`. A cascade reported at emit is then a **red signal** that the edge exists. It
  is never silently attributed.

## 5. Receipt shape — A's carrier, two more stage arms

```
NativeCensusCauseStage
  = NativeCensusCauseFrontEnd        -- A
  | NativeCensusCauseResolve         -- A
  | NativeCensusCauseInfer           -- part 2
  | NativeCensusCauseEmit            -- part 2
NativeCensusCauseMember              -- A's member, unchanged apart from the stage arms
NativeCensusCauseGroup { fatal_reason, members }   -- A's grouping key, unchanged
attribution: Root | CascadeOf{root}  -- [lane C] declared frontier
```

- **Grouping stays on `fatal_reason` alone,** as A has it. Stage is a member field and never a key.
  This is the same rule A applies to head: a fatal shared across stages is one cause group with
  members at several stages, and a reader filters on the field. Heads are carried and never tallied.
- **The partition check is A's.** `native_census_cause_partition_holds` covers infer and emit
  members without change. A member that lands in no group, or in two, breaks the partition at every
  stage.
- **Per-module disposition** is one coproduct per module, so the join over `native_census_modules`
  is total: `UpstreamRefused { stage: FrontEnd | Resolve }` | `InferRefused` | `EmitRefused` |
  `Emitted`. Rows hang off the refused arms.
- **Completeness** is a coproduct, as in part 1 (`Xl2CensusCompleteness` pattern). An incomplete
  infer observation or an unjoinable cascade root each refuses completeness. Neither is a zero.
- **No parallel carrier.** An `InferCensusRow` beside A's member would be the §3 fork. Adding
  arms makes every exhaustive match over `NativeCensusCauseStage` fail to compile until it handles
  them, which is the enrollment we want.

## 6. Instrument label — extend `v2-native-census`, no second label

Part 2 is **the same label**: `//gunbc/instruments:v2-native-census`. The label measures one fact,
the native census's cause groups over the self-host roots. Part 2 extends the stages the census
reaches; it does not measure a different fact. A second label would give one measurement two
routes, which DESIGN's "Building & checks" forbids (the `--self-host` precedent A also cites).

- **Realization.** `V2NativeCensusProducer` switches from running `census-resolve` to running
  `census-infer` (§2a). That verb reaches infer and emit through the shared demand subject and
  prints A's `cause_group` lines, whose members now include infer and emit members. `census-resolve`
  stays as a verb for part 1's grain. It is a realization, not a peer route, so this is not a
  second label.
- **Exit codes are A's, unchanged.** 0: the partition held. 1: the partition broke. 2: the run did
  not complete. Refusals are the census's data and do not change the exit code. Completeness below
  complete is reported on the status line, as A reports `cause_groups` and `partition_holds`.
- **Emit target.** The target is the instrument's own fact (the row names it), never a CLI option,
  so that one label cannot quietly measure a different target.
- **Cost.** This is a whole-tree run: CI or `ctrl-build --remote`, never in a session. The infer
  pass reuses the single `ResolutionContext` per ingest (#11401). Inferring per module must not
  rebuild that context.

## 7. Discriminating controls

Each control must go red when the behavior it guards is wrong. These are fixtures handed to the
compiler (DESIGN §4b: a compiler's probes are invalid programs).

1. **Stage discrimination.** In a two-module fixture, A resolves and fails infer (a branch-shape
   mismatch), and B fails resolve. The expected rows are A with `Infer` and B with `Resolve`.
   Rendering both as `Prepare`, or giving A `Resolve`, must red.
2. **Per-chain grain.** One module with two independent infer refusals yields two rows. Folding
   them to one must red. If infer cannot produce two, the control instead asserts
   `ObservationIncomplete`.
3. **Cascade.** T fails infer at a declaration that M uses, and M's only infer failure is at that
   use. Expected: M is `CascadeOf T` and T is `Root`. Removing T's defect must make **both** rows
   disappear. That is the route assertion, not only the answer (DESIGN §3 pairing).
4. **Non-cascade.** M has its own infer defect and also imports a refused T. M's own row must be
   `Root`. This catches over-collapse.
5. **Emit.** A module that infers but uses a construct the target refuses yields one `Emit` row.
   The positive control is a module that emits, with disposition `Emitted` and no rows.
6. **Join completeness.** Delete one module from the disposition table. The identity join must
   refuse, and a count comparison must not pass it.
7. **Inhabitance.** One control runs the real whole-tree route (the instrument row itself, on CI)
   and asserts that at least one `Infer`-stage row was produced **by the infer call** and not by a
   supplied fixture. Removing the infer call from the route must red it.

## 8. Open dependencies

- **Line:col on members.** swift-lynx-592 owns this path (#13005 follow-up), and it is held on
  #12506. A's member shape takes the field once that lands. Part 2's infer and emit members inherit
  it with no second locus path. Until then, a member's position is whatever its chain's diagnostics
  carry, rendered by nothing new.
- **[lane C], declared frontier.** The cascade carrier (`Root | CascadeOf`) and its collapse rule
  belong to lane C, which has not been dispatched. Its trigger is lane C landing a carrier that takes
  the stage as a parameter. If C's rule is resolve-only, it is generalized in C's module, not copied
  here. Until then, part 2 members carry no attribution, and the census reports `attribution
  unobserved` as a completeness refusal, never as `Root`.
- **The shared infer subject** (§2a), with tidy-koi-264's M0. Whichever lane builds first owns the
  subject and the `census-infer` verb arm.
- **`NativeTestStagePrepare` split.** This is a lane-vocabulary question (§3) that this doc raises
  and does not decide.
