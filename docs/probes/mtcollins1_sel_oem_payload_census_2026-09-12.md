# Mt. Collins OEM SEL census, 2026-09-12

**Evidence:** `artifacts/bmc/mtcollins1-sel-full-2026-09-12.txt` owns the complete records, counts and timestamps. The controller returned an unknown advanced description; that is a controller limit, not proof that vendor documentation is absent.

**Finding:** `0fde7033ff10`/`0fde703bff10` co-occur at adjacent record IDs in the same second on three occasions (`ad/ae`, `b4/b5`, `bb/bc`). This independently supports the socket-bit hypothesis from the crossover. The captured failure payloads are unpaired. Neither observation proves whether the emitting socket is failing or reporting, or decodes stage/status/tail to a DIMM. The `caee` pairing axis differs between dated populations and cannot inherit the `0fde` interpretation.

**Retry:** power-up 21:57:57Z, restart 22:03:26Z (5m29s), `0fde70331102`, then a `caee` pair 23 seconds later. Video and SOL recorder lists stayed empty despite confirmed power/reset/critical/non-recoverable/watchdog triggers and pre-event enablement. Arming these recorders did not capture the training window on this controller.

**Access defects:** HTTP login returned a token/session but subsequent authenticated API calls refused; the equivalent HTTPS sequence worked. The rendered power widget said off while IPMI, `/api/chassis-status` and live CPU rails indicated on. Read controller observations, not the widget. The [SMpro source survey](mtcollins1_ampere_smpro_access_paths_2026-09-12.md) adds vendor evidence without erasing the payload-layout mismatch.

[Original dated account](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_sel_oem_payload_census_2026-09-12.md). Raw artifacts and modeled evidence are unchanged.
