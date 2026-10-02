# Census part 2 — inference (and emit) census below resolve

**Status: design only.** The build waits until census part 1 lane A (deep-badger-684) merges. This
doc does not mint a row format. It extends A's row format by **one stage field**. Where A's PR is
still moving, the dependency is marked **[A]**, and §8 lists what has to be reconciled once it lands.

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
- **Lane A [A].** Lane A's row groups by `fatal_reason`, carries `head` as a field and the locus
  from that path, and is produced by one instrument row run via `gunbc test //gunbc/instruments:…`.

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

## 5. Receipt shape — one row format, extended by one field

```
row (lane A's shape [A]) + stage
  module:        String                 -- as NativeCensusResidualRow
  path:          Symbol
  stage:         CensusStage            -- Resolve | Infer | Emit   (the ONE new field)
  chain:         NonEmptyDiagnostics    -- fatal derived, head = advisory field, locus via
                                           swift-lynx-592's renderer
  attribution:   Root | CascadeOf{root} -- lane C's carrier [lane C]
```

- **Grouping** is by `(stage, fatal_reason)`, which is A's grouping keyed one level deeper. Heads
  are carried and never tallied (the #13005 rule).
- **Per-module disposition** is one coproduct per module, so the join over `native_census_modules`
  is total: `UpstreamRefused { stage: File | Resolve }` | `InferRefused` | `EmitRefused` |
  `Emitted`. Rows hang off the refused arms.
- **Completeness** is a coproduct, as in part 1 (`Xl2CensusCompleteness` pattern). An incomplete
  infer observation, a missing locus, or an unjoinable cascade root each refuses completeness. None
  of them is a zero.
- **Resolve is not re-reported.** If lane A's carrier already holds resolve rows, part 2's carrier
  *is* that carrier with `stage` added. Part 1's resolve rows become `stage: Resolve`. That is the
  "one row format" requirement taken literally. The alternative would be a parallel
  `InferCensusRow` beside it, which is the §3 fork.

## 6. Instrument label

There is one row in `gunbc.instrument_targets` and one arm in the `TargetProducer` dispatch (DESIGN,
"Building & checks": a new measurement is a row, never a flag). The proposed label is
`//gunbc/instruments:below-resolve-census`. **If lane A's label is a whole-pipeline census, part 2
is not a new label.** It widens A's subject to stages past resolve under A's label, and the two
renderings stay one instrument. That is decided once A's label is visible **[A]**. The default here
is to extend A's row rather than add a second label.

- **Exit codes** follow the label convention. 0: every subject emitted and completeness held.
  1: the census was observed, with refusals or incomplete completeness. 2: no observation, for
  example because the ingest refused.
- **Emit target.** The target is the instrument's own fact (the row names it), never a CLI option,
  so that one label cannot quietly measure a different target.
- **Cost.** This is a whole-tree run: CI or `ctrl-build --remote`, never in a session. The infer
  pass reuses part 1's single `ResolutionContext` per ingest (#11401). Inferring per module must not
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

## 8. Open dependencies (reconcile before building)

- **[A]** A's exact carrier name and fields, its label, and whether it stores the fatal or derives
  it. Part 2 conforms to whatever A merges. If A stores `fatal_reason`, part 2 does the same and
  cites A, because a disagreement there is A's decision, not this doc's.
- **[lane C]** Whether the cascade carrier is stage-parametric. If it is not, generalize it in lane
  C's module rather than adding a sibling here.
- **Locus field on `NativeTestFileRefusal`.** #13005 defers it to a change to `00_compile`
  `native_test_file_refusal`. Part 2 needs locus only on chains, which already carry it, so this
  does not gate part 2.
- **`NativeTestStagePrepare` split.** This is a lane-vocabulary question (§3) that this doc raises
  and does not decide.
