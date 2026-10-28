# Mt. Collins 2DRx4 group prediction, 2026-09-12

**Question:** does substituting the four 2DRx4 spares reproduce the restart signature? The preregistered configuration was twelve originals plus that quartet, eight modules per socket in odd connectors. The corrected spare inventory was 6×1Rx4, 4×2Rx4, 4×2DRx4 and 1×2Rx8; the six 1Rx4 modules' service on other hosts was an untested platform-transfer prior.

**Prediction:** low-confidence failure within roughly five minutes, with `Sys_Boot_Restart` and an `0fde7033…` payload. The unfamiliar organization motivated the guess; it was not established incompatibility.

**Assessment:** failure implicates the replacement configuration, not an individual defect; training would falsify the declared prediction for this population. Population shape alone had already proved insufficient, without proving its necessity. Neither result distinguishes a defective module from a combination restriction. See the [socket-swap outcome](mtcollins1_socket_swap_outcome_2026-09-12.md) and [bounded firmware finding](mtcollins1_dram_training_root_cause_2026-09-12.md) for subsequent evidence.

[Original dated account](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/probes/mtcollins1_2drx4_group_test_prediction_2026-09-12.md). Raw artifacts and modeled evidence are unchanged.
