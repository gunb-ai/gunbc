# Witness bridge — re-observation receipt (2026-08-08, valiant-boar-65)

Scope ruling (witty-raven-412 msg_e903115c, operator-confirmed): the next
producer slice is the **witness bridge** — lowered bodies → witness
`TargetModel`s → native bundle members — because it is the only piece that
moves **v1-executions-remaining** (the product metric; floor = the
InterpreterSubjectTest roster size). Slice 4 (dissolve
`body_lowering_fold`'s interim hand-walkers into `body_producer_forward`'s
row-selected slot-map apply) follows the first bridge stage, unless a bridge
stage is blocked by a specific hand-walker deficiency.

## Standing, measured against main `ad4e39b232` (the docs are behind the tree)

- `v2.compiler.body_producer_forward` EXISTS: `BodyProducerForwardRow` rows
  for fn_decl / fn_body / match / if / let / loop; first consumer of
  `ObligationForwardDeterminism`; exactly-one selection via
  `grammar_relation_row_forward_production_selection`.
- `v2.compiler.body_lowering_fold` is WIRED into `03_normalize`
  (`body_lower_finish_for_normalize`, two call sites): fn_decl→Arrow+domain,
  infix→Transform, let→Bind; typed wrapper-retained frontier live
  (`^body_lowering_reason_wrapper_retained_emitted`, counted); fold→Loop
  stays delegated to `fold_lowering` (one authority). Its note names itself
  an INTERIM scaffold dissolving at Slice 4; match/if/body_try are
  hand-walkers inside it; `^dag_surface_fn_body` shells are deferred until
  the parent fn_decl lowers.
- Therefore `general-body-producer-design.md` Stage A "remaining" and
  `body-lowering-design.md`'s keystone framing are STALE (within-body core
  landed via the wave2_prep lane) — updating both is part of this PR's
  intent, per the ruling.

## Bridge constraints (rulings, binding)

1. The bridge consumes the LOWERED SUBSTRATE OUTPUT (Arrow / Bind /
   Transform / Branch / Match / Loop nodes) — never the producing mechanism.
   If a bridge stage needs to know WHICH mechanism lowered a body, stop:
   that coupling is the §3 fork smell.
2. Wrapper-retained discipline extends to the bridge: a lowered body the
   bridge cannot yet carry is a typed, counted **bridge-retained cause**,
   never silently interpreted.
3. Census-priced staging: each stage names the
   `NativeBlocked{GeneralWitnessTargetModelUnavailable}` bucket slice it
   zeroes (construct sub-census in `ci0-witness-execution-census.md`: named
   calls 95%, records 46%, `=>` sugar 42%, match 39%, projection 33%, let
   31%) and reports v1-executions-remaining.
4. `SymbolIndex` consumed read-only; changes escalate to witty-raven-412.
5. Roadmap row: `general-witness-body-producer`, riding PR #8034 (head
   `943bc613e1`, at its merge bar). Cite the row by id.

## Existing bridge-shaped machinery to consume, not re-mint

- The 3-member bundle path: `EmitFamilyMember { tree: InferredTree, target:
  TargetModel }` (`v2.compiler.emit_module`), members built in
  `v2.test.execution.native_selected_witness_bundle`
  (`selected_logic_plan_for` → `SelectedWitnessPlan`), executed via
  `claim_executor` `run_native_bundle_unit` with `SelectedWitnessIdentity
  { entry, function, subject_digest }`.
- `gunbc.witness_execution_class` `EmitterCarriage.CarriedInEnrolledBundle`
  is the classification-side join key the bridge population feeds.
- Open question for the first bridge stage (next session's first read): what
  a `TargetModel` actually requires per member (binding_spellings + lex rows,
  per the fixture family) and which of those facts are derivable from a
  lowered Arrow body + `SymbolIndex` alone — the first stage is the smallest
  real witness (a hermetic fn whose body is already fully lowered, no
  wrapper-retained residue) carried end-to-end: lowered body → derived
  TargetModel → bundle member → native execution equal to the interpreter
  oracle, with a planted-red control.

## Correction round 2 (msg_564240d4): existing machinery to CITE, not remodel

- `v2.std.live_read` (G2 call-reachability classification, `G2FactSubject`
  law) IS the subject-binding story for `NativeLiveObservation` — the census
  axis grounds on it; nothing to build there.
- `v2.std.witness_execution_routing` + `witness_bulk_routing` already carry
  `WitnessExecutionDisposition` (NativeRouted | InterpretedRetained{reason,
  dissolve_on} | EmitIneligible{cause}) at ENTRY grain — 744-entry roster,
  zero NativeRouted by design, measured ceiling 77 ReadsLiveTree + 35
  host-effect retained "until native host-effect transport is its own lane".
  **Reconciliation obligation (first bridge-PR task):**
  `gunbc.witness_execution_class` (fn grain) and this disposition vocabulary
  must resolve to ONE authority — consume/extend, or declare the fn↔entry
  grain join explicitly on the carrier. Two entry-grain classification
  vocabularies is the fork the census exists to catch.
- `host_effect_plan`/`host_effect_apply` + effect_plan real-execution
  witnesses: EffectObligation needs enrollment only, no new machinery.
- Blockers therefore reduce to exactly two: (a) the witness bridge (this
  slice), and (b) **native host-effect/live-read transport** for emitted
  witnesses — the un-opened "own lane" the July ceiling named; it, not
  missing modeling, holds the ~2,960 live-readers. (b) is NOT started
  without a check-in; it may deserve its own row/owner.
