# Prospective memory-change assessment

The assessed subject is `gunbc.memory_configuration::MemoryChangeProposal`: the initial and proposed positioned populations, platform and firmware/settings, required operating questions, and the planned actions. `product.build_fulfillment::NodeRecipe` consumes this subject; its DIMM bill of materials counts acquisition positions and groups them by catalog declaration identity. Retained modules participate in the checks but do not become new purchases.

`gunbc.memory_configuration_evaluation::evaluate_memory_configuration` joins catalog/package authorities, the observing receipt, and independent qualification standings. Each obligation uses `std.goal_assessment::GoalAssessment`. Its fixed minimum cannot be removed by supplying an empty list, and unsupported additional obligations return an explicit missing-model fact. Comparison-scope coverage is separate from the observed mixture requirement. Within the witnessed socket, uniform package bytes satisfy this mixing check; that does not establish class support, part qualification, successful training, boot, or applicability on another socket.

The observed rule is deliberately bounded to the captured platform, DRAM firmware 210525, settings, socket 0, and the 0x00/0x91 pair. It is not a vendor rule for all Altra firmware, an electrical-load theory, a universal unequal-byte prohibition, or an ADS-family defect finding. `gunbc.memory_observation::MemoryMixtureFinding` retains the attempt, captured configuration, mismatch coordinate, and collector provenance. Raw SPD package layout comes from the pinned EDK2 DDR4 definition; Micron's published package bytes live with its catalog authority. The Hynix byte is an observation, not a new manufacturer guarantee.

For procurement, each position names allowed delivery alternatives. Equal package values across all allowed alternatives discharge the package question even when the RCD vendor is absent. A possible but non-certain conflict names the needed supplier restriction, sample qualification, or receiving inspection; it is unresolved, rather than a claim that undelivered hardware failed. The whole alternatives set remains bound into the assessment.

`gunbc.memory_change_evaluation::evaluate_memory_change` checks initialization snapshots at their action positions. A mixed population while powered off is distinct from initializing it. Its values cannot serve as collector or qualification receipts. Production policy withholds admission on any mandatory conflict or unresolved obligation. Qualification policy carries a finite population bound and explicit completion obligations. Every resolved fulfillment line retains this purpose and assessment, and selection does not promote qualification into production readiness.

## Consumption and limits

`product.build_fulfillment::resolve_build` and `max_constructible` execute the assessment before resolving supply or system inclusions. The training-failure screen consumes `gunbc.host_memory_qualification::part_implication`; `compatibility_of` is now called by the fulfillment fold. PR #11140 supplies `PartEvidenceUnresolved`, which remains unresolved through `CompatibilityEvidenceUnread`. This change specializes the shared goal-assessment payload instead of inventing another general outcome algebra or changing `HostMemoryStanding`'s four axes.

The originally assigned order-actuator and physical-memory-convergence completion conditions were withdrawn by the assigning parent after a caller census established that those routes do not exist. This increment does not introduce them. `memory_change_consumer_frontiers` returns both absent consumers and their required triggers beside each assessment: an order actuator must consume exact proposal/evidence-bound admission; physical convergence must independently read actual identities, catalog bindings, placements and firmware/settings, and re-evaluate changed inputs. A planned readback is not an observed readback. These frontiers are not claims of physical enforcement.

The surviving `PopulationOption` path is explicitly frozen for host population answers: no new host rows or consumers. `derive_populations` still enumerates homogeneous candidates before placement selection; it does not define the selected configuration. `product.node_power_envelope` currently lacks a per-connector mixed-population power model. Its returned frontier requires that model to consume `MemoryConfiguration.population`, preserve existing refusals, and delete the aggregate input in one cutover. Its present envelope is not a power assessment for this proposal. The evaluator never calls `effective_data_rate` or consumes `altra_two_dpc_ceilings`; an added data-rate obligation is currently missing-model/UNRESOLVED. Uncited 2DPC ceilings do not become constraints through this assessment.

Review 64358's unresolved-sample case is checked through `resolve_build`: even a market row marked `CompatibilityEstablished` cannot discharge the mandatory training-failure obligation. Production refuses; a qualification purchase requires that named completion obligation and remains a selection gap. The later supply fold now explicitly narrows unresolved findings for production and only permits the declared qualification exception. `PartNotImplicated` supplies no positive evidence: the independent class, specific-part, training and boot assessments remain mandatory.

The replay's board declaration now names `gunbc.machine_intake_mtcollins1_access_observation::mtcollins1_identity.baseboard`, an observed board identity, rather than a memory geometry. No full vendor baseboard catalog is fabricated. Inventory token bindings are observing receipts tied to the capture and catalog declaration identities; they do not assert a general firmware print convention. Missing or ambiguous bindings refuse explicitly. Qualification receipts must also cover their claimed positioned inventory; healthy process metadata and hand-authored positive standings cannot compensate for a partial capture. The bounded attempt's expected population count derives from the documented channel-pair rosters; generic positioned-inventory coverage has no sixteen-DIMM assumption.

## Replay

The artifact is consumed from evidence PR #11141 (head `c9ac696e3e1`); its original parent evidence commit is `59355770acb4648118bedf2a721dca719298c35e`. It is not independently re-authored by this evaluator:

- `artifacts/bmc/mtcollins1-sol-dram-training-2026-09-12.txt`
- SHA-256 `e5044f71cde73fe9b2ea1e76c9b894c1ab2f007577b9cbb7e8d50650c2b78a4c`

The replay checks read success and the artifact hash before classifying the transcript. Its collector duration is the operator's repaired-collector report (645 seconds), not the script wrapper lifetime. The artifact does not establish module serials, which remain absent. Collector rejection combines process status, expected stimulus, interval coverage and positive positioned inventory. Identical capture hashes only trigger suspicion; they do not prove self-observation.

```sh
ctrl-build --local -- /cargo-target/release/gunbc run \
  --source-root dag --source-root src/v2 \
  --entry dag/gunbc/host/mtcollins_memory_change.dag \
  --function assess_mtcollins_memory_change
```

The expected command result is a production admission refusal with the located byte-6 counterexample. This is a read-only replay, not a new boot attempt. `explain_mtcollins_memory_change` returns the full structured assessment and rendered explanation to model consumers; the command wrapper maps the admission result to `ProcessExit`.

Scoped controls live in `test.claim.memory_change_evaluation_witness`, alongside the migrated fulfillment and selection witnesses. Controlled transcripts there are simulation inputs. They are not evidence that this test session operated the host. Required CI remains a separate outcome: a refusal before claim evaluation is UNEVALUATED, not an assertion failure and not a pass.
