# Ampere pre-OS observation paths, 2026-09-12

**Purpose:** retain the source survey that motivated later SMpro reads. This survey performed no hardware probe; subsequent observations belong to `gunbc.machine_intake.mtcollins1_smpro_observation` and `mtcollins1_smpro_error_reading`.

**SEL:** Ampere's IANA identity and Altra BMC Interface Specification v1.43 identify the producer. The spec defines registers, not SEL layout; Ampere's OpenBMC training-log/error-monitor emitters supply `(channel << 4) | type | (socket << 3)`. That supports the socket hypothesis, but captured sensor `0xDE` versus documented `0xEB`, and type 3 versus 4, prevent using that layout to decode the tail or name a DIMM. MegaRAC's unknown description is a controller capability fact, not absent vendor knowledge.

**Register route:** OpenBMC's `0x3c/0x17` sysfs-backed command may collide with AMI Get Hardware ID. The survey instead identified standard IPMI Master Write-Read `0x06/0x52` and Mt. Jade's bus-2 SMpro addresses 0x4F/0x4E; applicability still needs observation. The v1.43/Linux register contract names current stage `0xB2`, failure bits/slot bitmap `0xB1`, slot select/syndrome `0xB4/0xB5`, and stage history `0xB0`. History advancement consumes data and goes last, after nondestructive evidence. Syndrome byte order must be established before decoding: B5 is absent from the spec's byte-swap exemption list.

**Console uncertainty:** family and board sources disagree between S0_UART1 and S0_UART3 for the SCP/SMpro console. Neither establishes a Mt. Collins UART header; the mux/header are reference-design facts. A physical investigation must resolve board routing before treating either candidate as established. The OpenBMC `0x3c/0xB0` mux command invokes vendor software not demonstrated on MegaRAC. SBMR support is expressly excluded for Mt. Jade in the cited manual.

**Probe order:** preserve SEL, inspect SOL instances, read current stage/failure bitmap, then slot syndromes with an established byte order; consume history last. Physical access is a separately authorized fallback. The original register bitfields, source locators and proposed probes remain in the historical survey; they are not a current actuation license or an invented decoder.

[Original dated account](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/probes/mtcollins1_ampere_smpro_access_paths_2026-09-12.md). Raw artifacts and modeled evidence are unchanged.
