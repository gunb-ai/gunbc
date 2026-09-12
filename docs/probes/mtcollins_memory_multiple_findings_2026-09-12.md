# Multiple bounded memory-mixture findings

This follow-up extends PR #11144 without replacing its first observation. Firmware 210525 reported expected byte 0x91 against found 0x00 in the first configuration, and expected byte 0xb2 against found 0x00 in the second. Each finding retains its configuration, comparison socket, mismatch coordinate, attempt and collector. Agreement remains a raw-byte question; neither two observed pairs nor the catalog stacking classification establish a general unequal-byte rejection rule.

The second source is `artifacts/bmc/mtcollins1-sol-3ds-mixture-refusal-2026-09-12.txt`, supplied by PR #11189, commit `4f9f6e7c305`. Its SHA-256 is `de1af941fee140e0280b77722b68c338111441c567fff570ce9065f6a72258d8`. The capture contains two complete sequences from the firmware notice through inventory and the mixture refusal. This supports complete captured stimulus cycles, not continuous attachment for the wrapper's 606-second lifetime. The collector's live banner/flush controls and cycle evidence must not be collapsed into that lifetime.

The second inventory prints `M393A8G40D40-CRB`. It does not establish the `0Q` suffix carried by `extdeps.memory.samsung::m393a8g40d40_crb0q_catalog`. Agreement of the printed capacity, rate, ranks and width is corroboration, not a SKU join. An SPD/physical-label read or Samsung's suffix decoder is needed to resolve the identity. The configuration therefore retains an observed module identity rather than inventing an external catalog row or suffix binding.

The diagnostic reports expected 0xb2 for the comparison group and found 0x00 at MCU4/slot0. It does not identify the mechanism selecting the expected value or provide each Samsung DIMM's raw SPD byte. The witnessed failure is a configuration-level counterexample; individual Samsung package evidence remains unresolved. A uniform 0xb2 training result is unobserved. The comparison iteration domain and first-mismatch stopping behavior are also unestablished.

Capacity is an independent vendor requirement. The existing GSG Issue 1.05 authority was downloaded and its pinned digest verified as `1c41b4f088fa9c8b760d21d57ae93b0d1a68ee67a740d4eb7ac893753fc580b3`. Page 10, immediately above Table 10, requires identically sized DIMMs within a socket while permitting different sizes between sockets. `mt_collins_socket_dimm_sizes_supported` projects that rule in the existing extdeps population module. This adds no rank or package restriction. The second captured configuration violates both capacity and package acceptance; firmware reporting byte 6 does not establish that capacity passed.

Evaluation joins all applicable established findings before classifying a proposed variant pair. An unrelated observed pair does not make a proved rejection unresolved; an unobserved pair remains unresolved. Repeated findings retain their provenance but do not multiply the number of failing delivery combinations. Scope coverage is the union of valid bounded scopes, rather than the first finding's scope. No valid finding means unresolved, never a vacuous pass.

The complete inventory and required printed descriptors must be present in each captured cycle. A full first cycle cannot hide an incomplete second inventory. The collector read is shared by package and scope evaluation and carried into the report, which lists retained observations and excluded-receipt reasons separately from requirement outcomes.

## Replay and validation

The read-only entry point assesses both captured configurations against the combined evidence:

```sh
gunbc run --source-root dag --source-root src/v2 \
  --entry dag/gunbc/host/mtcollins_3ds_memory_change.dag \
  --function assess_mtcollins_two_configurations
```

The committed-file replay verified both hashes and exited 1 with the expected production-admission refusal. It retained expected values 145 and 178 with their separate sources and coverage, located found 0 at J9/MCU4, reported the independent per-socket capacity violation (68719476736 versus 17179869184 bytes), and left the Samsung catalog binding unresolved. No hardware action occurred.

The controls live in `dag/test/claim/host/memory_multiple_findings_witness_test.dag`, with the original assessment, fulfillment and selection controls in their sibling witness modules. They cover both known pairs, an unobserved pair, order and duplicate findings, missing/failed observations, every-cycle inventory coverage, firmware and socket applicability, unbound identity, independent capacity, and the real production fulfillment refusal. For example, dispatch the identity-join and production-consumer controls through the existing instrument:

```sh
claim_batch --source-root dag --source-root src/v2 \
  --entry dag/test/claim/host/memory_multiple_findings_witness_test.dag \
  --functions w_printed_token_standings_cannot_qualify_unbound_modules,w_second_configuration_withholds_actual_production_fulfillment
```

The source-carried hypothesis controls are explicitly not new machine observations. Required CI remains separate from these scoped controls. The collection reader uses map/flatten rather than repeatedly copying an accumulator as observation history grows; its unavailable arm carries only unresolved facts.
