# Native obligation population: the built driver judges the v2 universe on the required path

**Governed by** [v2-compiler-architecture](v2-compiler-architecture.md) as a implementation program of the stage: the native route on the required path. Derived from gunbc.plan_governance; the body restates none of it.

> **Status: designed 2026-10-09 (operator ruling: every v2 task carries a design and is handed to a dispatch worker). No implementation lands from this document.** It is the model behind roadmap nodes `native-route-first-frontier` and `native-obligation-population`.

## 1. Goal and displaced cost

After the 2026-10-09 ruling the required path has one job, `witnesses`, which emits and builds the two retained products. Nothing on that path yet judges the `v2.test.*` universe with a binary the seed emitted, so `gunbc.rung_drop` `v2_native_route_off_the_merge_path` stands and the native verdict is reasoned rather than observed. The displaced cost is every landing that cannot tell whether the native compiler agrees with the tree it just built.

## 2. The model

- **Population, separate from the subject rows.** `gunbc.emitted_subject_build_gate` `emitted_subject_build_rows` name what is emitted and built (two products). The population is what the BUILT product then adjudicates; it is a different set with a different executor and is not appended to those rows.
- **Executor.** The self-host product's built `std.compiler_entry` `SourceRootEvalDriver`, run in adjudicate mode over a universe of identities. Not the interpreter, not the seed's Rust.
- **Denominator.** The denominator production admission reads is the derived universe: green identities plus exactly one observed frontier plus typed exclusions and refusals (`native_denominator_is_the_universe` joins them at identity grain). The green set alone is never a denominator.
- **Frontier subject.** The first frontier is observed over a subject named by a stable qualified identity, or by a derived single-member target with its cardinality asserted; "the smallest member" is not a subject (`native_frontier_subject_is_stable`).
- **Green set plus exactly one frontier.** `native_population_green_set` holds identities the driver adjudicated as agreeing; the frontier is the FIRST typed terminal the driver reports beyond it. A second frontier row refuses: the line stops at the first deficit (DESIGN section 5).
- **Eligibility is native adjudication only.** A member judged by the host (`NativeClaimProgramProducer`, `NativeServeProgramProducer`) is ineligible; admitting it would let the interpreter's verdict stand in for the native one.
- **Separately sized slot class (operator chat ruling 2026-10-10, relayed by smart-gull-336).** The 22 GiB / 21 GiB / swap-0 slot of `gunbc.runner_slot_desired` is the emit-and-build slot and stays so; every v2-native job (the built `SourceRootEvalDriver`, a native regen instrument) runs in its own class: a second `RunnerSlotDesired` row with its own MemoryMax, MemoryHigh, MemorySwapMax=0 and runs-on label, the `gunbc.runner_slot_allocation` math splitting hosts across the two, and the size fixed only after the native job's own MEMORY RECEIPT, never copied from the emit-build slot.
- **Reuse versus replace.** `gunbc.native_frontier_roster` and `gunbc.native_frontier_debt` stay the debt contract; the first green set is what mints the roster, so this plan feeds that carrier and does not fork it.

## 3. Milestones

1. **M1 native-route-first-frontier.** Makes true: `native_route_first_frontier` is `FrontierObserved` from one driver run over a one-identity universe named by a stable subject. Deletes in the same change: nothing. Production consumer: the ordering decision between `xl2-body-lowering` and `native-seven`. Qualification: the witness functions, green over supplied values. Exit: the row carries tested tree, executable identity and command.
2. **M2 native-obligation-population.** Makes true: a green set and the one frontier judged by a step in the `witnesses` job. Deletes in the same change: the `v2_native_route_off_the_merge_path` population it discharges. Production consumer: `native-production-admission`, whose denominator is the derived universe (green + one frontier + typed exclusions), not the green set alone. Qualification: the witness functions, green over supplied values. Exit: the first merge_group run printing the green set and the frontier.

## 4. Decisions

1. Ruled 2026-10-09: the frontier is observed from a run, never reasoned from source.
2. Ruled 2026-10-10 (operator, relayed by smart-gull-336): the denominator is the derived universe -- green identities, exactly one observed frontier and typed exclusions/refusals -- and the green set alone is never a production-admission denominator; the first frontier's subject is a stable qualified identity or a derived single-member target with a cardinality assertion, never "the smallest member".
3. Deferred: widening the universe from one identity to all of `v2.test.*`; trigger is M1 landed with a terminal that the driver can pass.

## Dissolution trigger (DESIGN §6)

Delete when the population is judged on every landing by the binary built from the seed's emission, retiring gunbc.rung_drop v2_native_route_off_the_merge_path and discharging v1_required_lanes_withdrawn member (1), and native_population_green_set has become the minted gunbc.native_frontier_roster. A terminal refusal is evidence and never dissolves this plan.
