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

## Classification: R2 stale-artifact (suspected primary class)

**Hypothesis:** srv2-05 retained a **toolchain-keyed native cache artifact** from the pre-#7069 window. After #7069 keyed `GUNBC_NATIVE_CACHE_ROOT` to the rustc/toolchain identity, the runner could still serve **byte-identical compile_skipped hits** while the **emitted family source or member dispatch surface** no longer matched the interpreted eval oracle.

Evidence supporting R2 stale-artifact over cold-cache miss:

| Signal | Observation | Reading |
|--------|-------------|---------|
| `compile_skipped=true` ×3 | All members warm | Not a cold-build failure |
| 496ms wall | Fast | Interpreter + cached native only |
| srv2-05 only (initial report) | Divergence on one runner class | Persistent local cache state |
| #7069 landed toolchain-keyed root | Cache root semantics changed | Stale hit class opens |
| srv1 4-run hermetic probe (parity_receipts_local) | Runs 2–4 green on same key | Repro is host-state dependent, not deterministic code bug |

**R2 definition (this receipt):** A native cache entry whose **key still matches** (compile skipped) but whose **stored artifact is semantically stale** relative to the current emitter/eval contract — the absorbing-failure mode where the transport reports success (cache hit) while the agreement oracle redds.

## Alternate hypotheses (lower prior)

1. **Genuine semantic divergence** — interpreted eval and native disagree on a member (meet/join/complement) under warm cache. Would be reproducible on a clean root; srv1 probe argues against this as the steady-state main-line failure.
2. **Member dispatch run failure** — stdout octet mismatch masked as agreement `false`. Loudness fix (member + both values) discriminates this on the next red.
3. **Workspace path collision** — `pr_native` test_id workspace shared across jobs without eviction. Less likely given hermetic no-evict design and distinct keys per family digest.

## Remediation sequence (mandate order)

1. **Agreement loudness (LANDING THIS PR):** `v2.std.native_agreement` + `*_failure_receipt` companion — failures name `member=` and `interpreted=` / `native=` octet labels.
2. **Do not re-flip** complement/meet_join until warm agreement greens on **≥2 distinct runner hosts** with loud receipts.
3. **srvN hygeine:** Evict `GUNBC_NATIVE_CACHE_ROOT` for key `8f21f541808ddb22` on affected runners before re-flip experiment; falsifier cold control continues exercising both legs.
4. **744-entry census:** Mechanical roster in `docs/probes/witness_entry_eligibility_census.tsv`; per-entry cssl first-error sweep on srvN names remaining `EmitIneligible` classes before bulk flip.

## Receipt hooks

- Frontier revert: `src/v2/compiler/self_host/native_routing_frontier.dag` `routed_flip_reverted_note`
- Loudness authority: `src/v2/std/native_agreement.dag`
- Failure companion: `emit_on_demand_family_crate_pr_native_agreement_failure_receipt()` in `emit_on_demand_family_crate_witness_test.dag`
- Local green control: `src/v2/compiler/self_host/parity_receipts_local.dag` (srv1, key `8f21f541808ddb22`, runs 2–4 warm PASS)

## Status

**Diagnosis-complete, re-flip blocked.** Next red on agreement must carry located member + both values; if R2 stale-artifact reproduces, eviction receipt is the counted remediation before any `NativeRouted` data flip.
