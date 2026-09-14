# Optional replacement migration B — draft recovery relay

NOT FOR MERGE. Source base: ed853cf430e8004b937b9017ca1fb6742901c00e.
The patch preserves WIP without publishing a source/mirror mismatch on the production branch. No generated mirrors are included. This is not yet a regeneration request or a validated implementation.

Gatekeeper accepted the declaration-level plan in msg_9ea2e6dd, following side-chat turn 832a0c6e. Canonical resolved/inferred Optional is the existing v2.std.optional.Optional owner application with instantiated payload and Required cardinality. The single syntax ingestion point is resolve_node_bounded. Comparator equivalence is forbidden.

The consumer census is the prior cb952 census with the admitted classify_field_recursion disposition corrected; the remaining rows still need reconciliation against this main-based implementation. No complete-coverage claim is made by this draft.

Two bounded divergences must appear in the final PR: flatten already-Optional results only at lookup_field_type_node and map_lookup_result_type, preserving main behavior until nested pattern/emitter consumers are proven; retain classify_field_recursion as authored-syntax analysis per msg_1fb0b46d. The latter receives raw module declarations through build_item_inductive_fields in build_type_env/build_type_env_unresolved and emits no type evidence. Next rung: split the resolved-evidence carrier so it cannot contain CardOptional, placing this analysis on the syntax carrier by construction.

Validation so far: isolated exact-main claim_executor/claim_batch build succeeded; a parse-only probe accepted six edited modules. No semantic pass, fixed point, or native receipt claimed. Pending: producer-boundary audit, controls, main-seed transaction, regeneration and native closure. B is falsified if it requires #11277-specific canonicalization; do not add a compatibility bridge.

Executed discriminator: authored_optional_field_resolves_to_instantiated_owner_application compiles authored `type Sample { value: Int? }` through compile_to_resolved, then inspects the actual field binding. Exact-main committed seed ed853cf430e refuses its Required assertion: actual CardOptional. Remaining assertions require one ordered Int argument and a Disj Optional payload whose Present.value is instantiated to Int. The test is enrolled in infer_semantics_witness; the isolated diagnostic runner executed that function verbatim against the main-seed library under the 22 GiB virtual-memory bound. See authored-main-red.log. This is baseline RED evidence, not a candidate GREEN.

Latest audit: main has 83 cardinality-consuming declarations, compared with 85 on the earlier #11277 head; main-consumer-census.md records the current-base dispositions. Authored alias RHS and named-alias grounding now route marked inputs into the same resolver boundary and preserve diagnostics. The three source regression claims completed on the exact main seed but exceeded the 500ms CPU ceiling (see main-claims.log); no floor pass is claimed. Cost reduction remains required.

The informational #11277 payload transaction returned as origin/relay/regenA-cb952-payload-gen1 (8639bd02a847): all five Primitive receiver/method refusals closed, but generation 2 still refused twelve Optional joins (seven old, five newly reached). This is not a B receipt. B and the later recomposed #11277 must close that join class; no mirrors from that relay are installed here.
