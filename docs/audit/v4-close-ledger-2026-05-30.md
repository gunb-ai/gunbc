---
title: v4 Close Ledger — per-probe owner-manager mapping
date: 2026-05-30
manager: Close/Receipt (sharp-otter-407)
derived_from: docs/audit/v4-close-interrogation-validation-2026-05-30.md (PR #3941, merged)
authority: PR #3938 §11 (manager-lane architecture) + this lane's manager-pass receipt (docs/planning/v4-close-receipt-manager-pass-2026-05-30.md, this PR)
---

# v4 Close Ledger — 2026-05-30

Close/Receipt-lane per-probe close ledger. Each row in the 346-probe questionnaire validation (PR #3941) is re-emitted here with an added `owner_manager` per the §11.3 lane-to-section mapping in PR #3938, projected onto the questionnaire-section axis.

**Authority:** this ledger is read-only at the probe level — `ship_disposition`, `engineering_state`, evidence pointers, and `blocking_receipt` are taken verbatim from PR #3941. The Close/Receipt lane's contribution is the `owner_manager` column (primary + secondary) and the per-owner roll-ups in §2.

**Closure invariant restated:** a probe moves from `ship_disposition: GAP` to `PROVEN` only by an executable receipt that answers the exact probe plus a falsification receipt where the probe is explicitly adversarial. `engineering_state` is orthogonal to `ship_disposition`; SUBSTRATE_PRESENT is planning value, not close-readiness. See `docs/planning/v4-close-receipt-manager-pass-2026-05-30.md` §1 for the full vocabulary and the §2 close grades (`SUBSTRATE_CLOSED` / `GATE_CLOSED` / `RECEIPT_CLOSED`).

## §1. Headline

- **Total probes:** 346
- **`ship_disposition: PROVEN`:** 0 / 346
- **`ship_disposition: GAP`:** 346
- **Other `ship_disposition` values observed:** 
- **`engineering_state` distribution:** `SUBSTRATE_PRESENT` = 233, `NO_ARTIFACT_FOUND` = 68, `CENSUS_NOT_RUN` = 45

No probe is close-ready. The split between SUBSTRATE_PRESENT and NO_ARTIFACT_FOUND tracks substrate-rich / activation-poor (PR #3938 §3 diagnosis) and pre-substrate gaps respectively.

## §2. Per-owner-manager roll-up (primary ownership)

| owner_manager (primary) | probes | GAP | other ship_disposition | typical blocking_receipt class |
| ----------------------- | ------:| ---:| ---------------------- | ------------------------------ |
| Close/Receipt | 19 | 19 | — | manager-pass ratification (see §3 ratification status in receipt doc) |
| Compiler Spine | 15 | 15 | — | T-8/T-9/T-10 interface stability + Instantiation consumer wiring |
| Ladder/Fixture | 4 | 4 | — | rung-predicate gate firing on §7 fixture set |
| Modeling DFS | 138 | 138 | — | DFS worksheet + single-authority substrate fix (per PR #3938 §10.0) |
| Runtime/TestClaim | 79 | 79 | — | T-38 executable runner + falsification transcript |
| Self-host/Release | 42 | 42 | — | T-15 reproduction + T-36 round-trip receipts |
| Target Realization | 49 | 49 | — | TargetAtomRealization / TargetTypeExpressionProjection / TargetCollectionRealization rows + per-language emit |

**Reading note:** the primary owner is the manager whose gate-firing or worksheet approval is the first step that moves the probe toward PROVEN. The secondary owner (in the per-section ledger below) is the manager whose ratification or interface stability is downstream.

## §3. Per-section ledger (346 rows)

Within a section, `owner_manager` is uniform per the §11.3 mapping. Each section header names the owner pair; rows then list `probe_id | ship_disposition | engineering_state | blocking_receipt`.

### §1.1 Complexity

- **owner_manager (primary):** Runtime/TestClaim
- **owner_manager (secondary):** Modeling DFS
- **probes:** 9

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:75` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:76` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:77` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:78` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:79` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:80` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:81` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:82` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:83` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §1.2 Cost

- **owner_manager (primary):** Runtime/TestClaim
- **owner_manager (secondary):** Modeling DFS
- **probes:** 15

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:91` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:92` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:93` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:94` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:95` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:99` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:100` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:101` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:102` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:103` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:107` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:108` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:112` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:113` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:114` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |

### §1.3 Parallelism

- **owner_manager (primary):** Runtime/TestClaim
- **owner_manager (secondary):** Modeling DFS
- **probes:** 4

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:122` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:123` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:124` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:125` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §1.4 Effect enumeration

- **owner_manager (primary):** Runtime/TestClaim
- **owner_manager (secondary):** Modeling DFS
- **probes:** 4

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:133` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:134` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:135` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:136` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |

### §1.5 User-defined dimensions

- **owner_manager (primary):** Runtime/TestClaim
- **owner_manager (secondary):** Modeling DFS
- **probes:** 7

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:144` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:145` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:146` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:147` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:151` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:152` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:153` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |

### §1.6 Tier 1 mechanics (coercion = emission / ownership / grounding completeness)

- **owner_manager (primary):** Modeling DFS
- **owner_manager (secondary):** Compiler Spine
- **probes:** 13

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:165` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:166` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:167` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:171` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:172` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:173` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:174` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:178` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:179` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:180` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:181` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:182` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:183` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §1.7 Tier 2 runtime safety (proven safe or total)

- **owner_manager (primary):** Modeling DFS
- **owner_manager (secondary):** Close/Receipt (D1 Option C — out of v4 ladder; tracked under T-25)
- **probes:** 8

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:193` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:194` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:195` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:196` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:197` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:201` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:202` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:203` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |

### §2.1 Pure Bootstrap (zero hand-Rust)

- **owner_manager (primary):** Self-host/Release
- **owner_manager (secondary):** Modeling DFS
- **probes:** 5

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:217` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:218` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:219` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:220` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:221` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §2.2 Closed system / no escape hatches

- **owner_manager (primary):** Modeling DFS
- **owner_manager (secondary):** Self-host/Release
- **probes:** 5

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:229` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:230` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:231` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:232` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:233` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |

### §2.3 Single authority / cost-of-change = 1

- **owner_manager (primary):** Modeling DFS
- **owner_manager (secondary):** Compiler Spine
- **probes:** 5

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:241` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:242` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:243` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:244` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:245` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |

### §2.4 Fail-closed discipline

- **owner_manager (primary):** Compiler Spine
- **owner_manager (secondary):** Modeling DFS
- **probes:** 4

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:253` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:254` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:255` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:256` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §2.5 Impossible bugs by construction (THE META-PROMISE)

- **owner_manager (primary):** Modeling DFS
- **owner_manager (secondary):** Runtime/TestClaim
- **probes:** 45

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:266` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:267` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:268` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:269` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:270` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:276` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:277` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:278` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:279` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:280` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:281` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:287` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:288` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:289` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:290` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:291` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:297` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:298` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:299` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:300` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:301` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:311` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:312` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:313` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:314` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:315` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:316` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:317` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:323` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:324` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:325` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:326` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:327` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:331` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:332` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:333` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:392` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:393` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:394` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:395` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:396` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:400` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:401` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:402` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:403` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §2.6 Substrate-shape specifics (6 connectives + 5 behaviors + C1 stop-signal)

- **owner_manager (primary):** Modeling DFS
- **owner_manager (secondary):** Compiler Spine
- **probes:** 12

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:487` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:488` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:489` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:490` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:494` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:495` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:496` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:497` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:501` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:502` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:503` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:504` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §2.7 Modeling discipline

- **owner_manager (primary):** Modeling DFS
- **owner_manager (secondary):** Close/Receipt
- **probes:** 16

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:521` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:522` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:523` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:527` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:528` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:529` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:533` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:534` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:535` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:539` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:540` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:541` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:545` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:546` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:547` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:548` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §3.1 Omni-emission (R3 = 3 Shape-A targets: Rust / Python / Go)

- **owner_manager (primary):** Target Realization
- **owner_manager (secondary):** Compiler Spine
- **probes:** 4

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:560` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:561` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:562` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:563` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §3.2 Workflow-as-data

- **owner_manager (primary):** Compiler Spine
- **owner_manager (secondary):** Target Realization
- **probes:** 5

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:594` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:595` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:596` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:597` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:598` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §3.3 Tests-as-data

- **owner_manager (primary):** Runtime/TestClaim
- **owner_manager (secondary):** Compiler Spine
- **probes:** 4

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:606` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:607` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:608` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:609` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §3.4 Full-stack-from-one-`.dag` — visceral 4-layer omni-emission + R4 framework substrate (FORWARD POINTER)

- **owner_manager (primary):** Target Realization
- **owner_manager (secondary):** Compiler Spine
- **probes:** 4

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:628` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:629` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:630` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:631` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §3.5 L6 — every structural form compiles to every target

- **owner_manager (primary):** Target Realization
- **owner_manager (secondary):** Compiler Spine
- **probes:** 5

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:641` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:642` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:643` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:644` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:645` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §3.6 L7 — operations obey declared algebraic laws

- **owner_manager (primary):** Runtime/TestClaim
- **owner_manager (secondary):** Modeling DFS
- **probes:** 5

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:657` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:658` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:659` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:660` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:661` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §3.7 The verification-machinery promises (testgen / integration / mocks / dry-run)

- **owner_manager (primary):** Runtime/TestClaim
- **owner_manager (secondary):** Compiler Spine
- **probes:** 27

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:677` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:678` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:679` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:680` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:681` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:682` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:692` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:693` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:694` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:695` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:696` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:697` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:709` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:710` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:711` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:712` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:713` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:714` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:724` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:725` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:726` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:727` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:728` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:729` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:730` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:738` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:739` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |

### §3.8 Multi-program / network-coordinated emission from one `.dag` (FORWARD POINTER)

- **owner_manager (primary):** Target Realization
- **owner_manager (secondary):** Self-host/Release
- **probes:** 7

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:760` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:761` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:762` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:763` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:764` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:765` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:766` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §4.1 Lens self-application

- **owner_manager (primary):** Runtime/TestClaim
- **owner_manager (secondary):** Compiler Spine
- **probes:** 4

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:786` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:787` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:788` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:789` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §4.2 Self-host fixed point

- **owner_manager (primary):** Self-host/Release
- **owner_manager (secondary):** Compiler Spine
- **probes:** 4

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:797` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:798` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:799` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:800` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §4.3 Concept unifications

- **owner_manager (primary):** Modeling DFS
- **owner_manager (secondary):** Close/Receipt
- **probes:** 10

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:815` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:816` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:817` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:823` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:824` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:825` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:826` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:830` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:831` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:832` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §5.1 5 substrate-gap classes closed

- **owner_manager (primary):** Modeling DFS
- **owner_manager (secondary):** Compiler Spine
- **probes:** 6

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:846` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:847` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:848` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:849` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:850` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:851` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |

### §5.2 v2 fully retired

- **owner_manager (primary):** Self-host/Release
- **owner_manager (secondary):** Modeling DFS
- **probes:** 3

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:859` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:860` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:861` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |

### §5.3 BridgeLedgerZero

- **owner_manager (primary):** Self-host/Release
- **owner_manager (secondary):** Modeling DFS
- **probes:** 3

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:869` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:870` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:871` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |

### §5.4 Compiler-as-data residual — "is the compiler pure data yet?"

- **owner_manager (primary):** Self-host/Release
- **owner_manager (secondary):** Modeling DFS
- **probes:** 19

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:887` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:888` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:889` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:890` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:896` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:897` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:898` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:899` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:900` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:901` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:907` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:908` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:913` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:927` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:928` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:929` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:930` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:931` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:932` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |

### §5.5 Free consequences (when Tiers 1-2 close)

- **owner_manager (primary):** Modeling DFS
- **owner_manager (secondary):** Runtime/TestClaim
- **probes:** 11

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:960` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:961` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:962` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:963` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:967` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:968` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:969` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:970` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:974` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:975` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:976` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |

### §6.1 "Show the correct code"

- **owner_manager (primary):** Target Realization
- **owner_manager (secondary):** Compiler Spine
- **probes:** 4

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:990` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:991` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:992` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:993` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §6.2 Audience duality / opt-in depth

- **owner_manager (primary):** Compiler Spine
- **owner_manager (secondary):** Close/Receipt
- **probes:** 6

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:1005` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1006` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1007` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1011` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1012` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1013` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §6.3 Adoption model — economics, not enforcement

- **owner_manager (primary):** Close/Receipt
- **owner_manager (secondary):** (NOT_IN_V4 release-gate sense; tracked separately)
- **probes:** 10

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:1028` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1029` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1030` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:1034` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:1035` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:1039` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1040` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:1044` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:1045` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:1046` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |

### §7 Cross-doc ledger coherence (structural — keep from v0)

- **owner_manager (primary):** Close/Receipt
- **owner_manager (secondary):** Self-host/Release
- **probes:** 9

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:1056` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:1057` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:1058` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:1059` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:1060` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:1061` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:1062` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:1063` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |
| `docs/v4-close-interrogation.md:1064` | `GAP` | `CENSUS_NOT_RUN` | Complete targeted audit/census run for this probe. |

### §8 Per-gate predicate execution at close

- **owner_manager (primary):** Ladder/Fixture
- **owner_manager (secondary):** Close/Receipt
- **probes:** 4

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:1072` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1073` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1074` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1075` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §10 Close ceremony

- **owner_manager (primary):** Self-host/Release
- **owner_manager (secondary):** Close/Receipt
- **probes:** 8

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:1096` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:1097` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:1098` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:1099` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:1100` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:1101` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:1102` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:1103` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |

### §13 Arbitrary ingestion — bidirectional substrate (NEW for v4, was R4)

- **owner_manager (primary):** Target Realization
- **owner_manager (secondary):** Compiler Spine
- **probes:** 9

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:1140` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1141` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1142` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1143` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1144` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1145` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1149` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1150` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1151` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §14 Additional Shape A languages — C, C++, LLVM IR, TypeScript (NEW for v4, was R4.A/R4.C)

- **owner_manager (primary):** Target Realization
- **owner_manager (secondary):** Compiler Spine
- **probes:** 5

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:1191` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1192` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1193` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1194` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1195` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §15 Framework substrates — React, future UI/server frameworks (NEW for v4, was R4 canvas)

- **owner_manager (primary):** Target Realization
- **owner_manager (secondary):** Compiler Spine
- **probes:** 4

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:1219` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1220` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1221` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1222` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |

### §16 Multi-program / network coordination (NEW for v4, was §3.8 forward-pointer)

- **owner_manager (primary):** Target Realization
- **owner_manager (secondary):** Self-host/Release
- **probes:** 7

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:1257` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:1258` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1259` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1260` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1261` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1262` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1263` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |

### §17.1 C4 — Additional MachineConstraint axes

- **owner_manager (primary):** Modeling DFS
- **owner_manager (secondary):** Compiler Spine
- **probes:** 3

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:1287` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1288` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1289` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §17.2 C5 — Rounding-mode product-shape extension

- **owner_manager (primary):** Modeling DFS
- **owner_manager (secondary):** Compiler Spine
- **probes:** 2

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:1298` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |
| `docs/v4-close-interrogation.md:1299` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

### §17.3 C6 — Aspect-axis (PointKind) for instant/duration/rate

- **owner_manager (primary):** Modeling DFS
- **owner_manager (secondary):** Compiler Spine
- **probes:** 2

| probe_id | ship | engineering_state | blocking_receipt |
| -------- | ---- | ----------------- | ---------------- |
| `docs/v4-close-interrogation.md:1308` | `GAP` | `NO_ARTIFACT_FOUND` | Delivery artifact or executable demo receipt for the requested probe. |
| `docs/v4-close-interrogation.md:1309` | `GAP` | `SUBSTRATE_PRESENT` | Executable `TestClaimRun` verdict plus adversarial falsification transcript for this probe. |

## §4. What this ledger is NOT

- **Not a re-audit.** Per-probe `ship_disposition`, `engineering_state`, evidence, and `blocking_receipt` are taken verbatim from PR #3941 (`docs/audit/v4-close-interrogation-validation-2026-05-30.md`). The Close/Receipt lane has not re-walked the 346 probes; the next validation run will re-emit under the two-axis vocabulary directly.
- **Not a dispatch instrument.** Owner-manager attribution names the lane responsible for the *next step* toward PROVEN; it does not authorize worker dispatch. Worker briefs go through the named manager's worksheet/gate process per PR #3938 §11.
- **Not a release gate.** Release gate is TASKS.md:801-815 (six v4-done predicates). This ledger is the standing close-readiness surface beneath that gate, and per receipt-doc §3 D4 the v4-done definition is operator-authority, not this lane's to amend.

## §5. Related artifacts

- `docs/audit/v4-close-interrogation-validation-2026-05-30.md` — source data (PR #3941, merged).
- `docs/v4-close-interrogation.md` — 346-probe questionnaire (probe sources cited per row).
- `docs/planning/v4-close-receipt-manager-pass-2026-05-30.md` — this lane's manager-pass receipt (same PR): vocabulary, close grades, anti-shelfware policy, ladder ↔ questionnaire complementarity.
- `docs/planning/v4-correctness-ladder-2026-05-30.md` — PR #3938 (open at ledger authoring time): lane architecture, §8 decisions, §10.0 vocabulary origin. **This ledger depends on PR #3938 landing.**

