# srv2-05 native agreement divergence — root-cause diagnosis

**Stamp:** 2026-07-23  
**Incident:** GitHub Actions run `29963619401` (merge `2a226829f`, host `srv2-05`, 2026-07-22 23:28Z)  
**Witness:** `emit_on_demand_family_crate_pr_native_agreement_holds`  
**Outcome:** `Bool(false)` in 496ms, `compile_skipped=true` on all three family members  
**Native cache key:** `8f21f541808ddb22`  
**Frontier action:** complement + meet_join reverted to `InterpretedRetained` per `routed_flip_reverted_note` (#7091)

## What the log showed (before loudness fix)

- Warm native leg: all three members reported `compile_skipped=true` (cache hit).
- Agreement witness returned `false` with **no located member** and **no interpreted/native value pair**.
- CI failure line: `emit_on_demand_family_crate_pr_native_agreement_holds (...) returned Bool(false)`.

This is indistinguishable from a member-run failure, a semantic mismatch, or a harness false without further triage — exactly the loudness gap Lane B item (1) closes.

## Classification: R2 stale-artifact (confirmed construction defect)

The cache key was not a digest of the native realization. It combined the
inferred member-tree digest with constant labels (`"v2.compiler.emit"`,
`"rust"`, and `"native-artifact"`). `inferred_tree_digest` covers the source
tree and evaluated facts, but not the target model's runtime row: emitted
workspace suffixes, the family dispatcher, toolchain manifest, and build argv
were absent. The warm handler then treated the existence of a one-byte
`.native_ready` marker as sufficient and did not materialize the currently
requested files. A durable cross-branch workspace could therefore serve an
older family binary under the unchanged outer key.

Evidence supporting R2 stale-artifact over cold-cache miss:

| Signal | Observation | Reading |
|--------|-------------|---------|
| `compile_skipped=true` ×3 | All members warm | Not a cold-build failure |
| 496ms wall | Fast | Interpreter + cached native only |
| srv2-05 only (initial report) | Divergence on one runner class | Persistent local cache state |
| `toolchain_for_grain(NativeArtifact)` | Literal `"native-artifact"`, not a toolchain digest | The modeled key claim exceeded its construction |
| srv1 4-run hermetic probe (parity_receipts_local) | Runs 2–4 green on same key | Repro is host-state dependent, not deterministic code bug |

The fix retains the outer computation key but appends a derived
`artifact_realization_digest` over the exact materialized workspace paths/texts
and structured build argv. The marker now belongs to that realization child.
The executable `family_crate_dispatch_change_cold_rebuild_holds` control holds
the inferred trees and emitted member source constant, changes only the family
dispatcher, and requires cold → cold → warm. Before the fix its second leg was
a warm stale hit.

## Rejected alternatives

1. **Deterministic emitter/evaluator disagreement:** rejected by the clean srv1
   cold/warm sequence on the same source revision.
2. **Cold build or member-run failure:** rejected by three
   `compile_skipped=true` receipts and the 496ms wall time.
3. **Hash collision between distinct inferred trees:** unnecessary; the target
   realization was absent from the hashed inputs by construction.

## Remediation sequence (mandate order)

1. Agreement loudness landed in #7111 and names the member plus both values.
2. The effective workspace identity now includes exact realization inputs.
3. The same-tree/different-dispatch execution control prevents regression.
4. Complement/meet_join remain `InterpretedRetained` until the routing
   frontier's independent warm-host evidence requirement is met; this diagnosis
   does not silently re-flip policy.

## Receipt hooks

- Frontier revert: `src/v2/compiler/self_host/native_routing_frontier.dag` `routed_flip_reverted_note`
- Loudness authority: `src/v2/std/native_agreement.dag`
- Failure companion: `emit_on_demand_family_crate_pr_native_agreement_failure_receipt()` in `emit_on_demand_family_crate_witness_test.dag`
- Local green control: `src/v2/compiler/self_host/parity_receipts_local.dag` (srv1, key `8f21f541808ddb22`, runs 2–4 warm PASS)

## Status

**Root cause closed in construction; re-flip remains evidence-gated.**
