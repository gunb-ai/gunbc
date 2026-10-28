# Mt. Collins memory screen, 2026-09-12

**Purpose:** preserve the sequence of tested configurations without promoting the early diagnosis. “Trains” in the original table was later narrowed by the [outcome correction](mtcollins1_2rx4_outcome_correction_2026-09-12.md).

| Population | Recorded observation |
|---|---|
| Original sixteen, 8+8 odd connectors | 12m30s without restart, 8 W per socket |
| Thirty-one, originals plus spares | Restart ~5m11s, `0fde70330102` |
| Fifteen, 8+7 | Socket 1 disabled; continued degraded with `Boot_Progress` |
| Thirty-two, mismatched same-channel pairs | Restart ~5m04s, `0fde70330502` |
| Sixteen spares, 8+8 | Restart 4m50s, `0fde70331102` |
| Twelve originals + 2DRx4 quartet, socket 1 | Restart 4m50s, `0fde703b1102` |
| Same substitution, socket 0 | Restart 5m20s, `0fde70331102` |

**Discrimination:** original-module operation and unsuccessful reseating weakened shape/seating-only explanations. Reported equal capacity satisfied that vendor requirement; same-channel mixing rules did not discriminate subsequent 1DPC rounds. Working 2133 parts elsewhere disproved a blanket Altra speed prohibition, not board-specific qualification. srv2's failed Samsung upgrade despite other hosts' success remains a counterexample to transferring qualification across machines.

**Correction:** the [firmware finding](mtcollins1_dram_training_root_cause_2026-09-12.md) identified a rejected package-byte mixture. It did not prove every module healthy, a universal unequal-byte rule or uniform ADS support. The old “16 spares” population mixed ADS and ASF; a uniform ADS population remained untested. A proposed 1Rx4 substitution duplicated an already-observed monolithic configuration and was withdrawn. Seller labels were not independent organization/speed authority; the socket-bit reading remained bounded and could not identify a DIMM.

[Original dated account](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_micron_module_screen_state_2026-09-12.md). Raw artifacts and modeled evidence are unchanged.
