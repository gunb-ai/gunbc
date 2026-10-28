# Mt. Collins firmware mixture refusal, 2026-09-12

**Finding:** the repaired SOL collector captured firmware 210525 reporting `CHANNEL Mismatch Byte 6 SLOT0[00] EXP[91] @MCU[4]` and rejecting a non-identical DIMM mixture. This is a configuration-level counterexample scoped by `gunbc.host.mtcollins_memory_change.mtcollins_training_mixture_finding`, not a universal unequal-byte law, module-defect diagnosis or proof that uniform ADS modules train.

| Artifact under `artifacts/bmc/` | Evidence it owns |
|---|---|
| `mtcollins1-sol-dram-training-2026-09-12.txt` | Firmware refusal and complete positioned inventory |
| `mtcollins1-sel-full-2026-09-12T0355Z.txt` | Controller/host clock comparison and SEL records bracketing the boot |
| `mtcollins1-sel-full-2026-09-12.txt` | Earlier payload-family/co-emission census; ends before the SOL boot |
| `mtcollins1-controller-fru-2026-09-12.txt` | Unit serials independently agreeing with host SMBIOS |

**Interpretation:** SPD byte 6 carries package type, die count and signal loading. The observed expected/actual pair must stay distinct from catalog declarations. The captured Micron/Hynix mixture also differed in rank and speed; “everything else matched” is withdrawn. The Micron sibling comparison is a different subject. Uniform-part procurement is a conservative policy, not qualification or a general firmware rule. Later [multiple findings](mtcollins_memory_multiple_findings_2026-09-12.md) retain independent capacity obligations.

**Timing:** the later SEL capture measures a four-hour controller offset. Correcting that offset places records `518`, `51e`, `51f` around the SOL session; the same-boot join remains inference because neither stream carries a shared attempt identifier. The earlier capture could not support it. Socket 0 is explicit in the firmware output; MCU4 versus payload nibble 3 prevents a direct channel-index decode.

**Collector failure:** old captures were the client's banner after stdin EOF, despite an outer wrapper waiting. A pty with stdin held open exposed the real host output and inventory. Exit zero, repeated hashes, a quiet console or wrapper lifetime alone are not covered observation. `gunbc.recurring_failure_mode.empty_capture_read_as_clean_result` names the class; historical filing claims and corrections remain in Git history.

**Validate:** replay the hash-verified bytes through `assess_mtcollins_memory_change` and the scoped witnesses described in [prospective assessment](prospective_memory_change_evaluation_2026-09-12.md). Do not generalize the refusal beyond its captured platform, firmware, settings, socket and population, or confuse replay with another boot.

[Original dated account](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_dram_training_root_cause_2026-09-12.md). Raw artifacts and modeled evidence are unchanged.
