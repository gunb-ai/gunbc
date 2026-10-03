# Power-on sequence model and the "best understanding" diagnostic fold (Mt. Collins first)

Status: PLAN, for review before any code lands. Owner: calm-lynx-884, under eager-gull-22 (hw-boot).

## 0. The requirement

Operator, 2026-10-03, verbatim: "take ALL of the information we've gotten today (and in our existing models) - and construct a monolithic 'this is what actually happens when you turn on the computer' workflow - meaning, our diagnostic process will be 'this is our best understanding of what happened' - basically, the end of the workflow (error, no error, any information we have) - and then, as we add more platforms (mt jade, x86), we will abstract the workflow and plug in the differing pieces as we need - this will give us an idea of how to onboard new platforms".

This plan defines two artefacts:

1. **The stage chain.** This is one ordered model of what happens on a Mt. Collins (Ampere Altra, AMI MegaRAC BMC) between AC applied and the Linux census. Each stage names the component that runs it, the evidence that observes it, and what success, stated failure and silence look like. It also names the existing typed reader that covers it, or says that none does.
2. **One diagnostic fold.** Given one boot's captured evidence, it returns a `BootUnderstanding`. That value contains the furthest stage the evidence establishes, how progress ended, every decoded error, and every remaining unknown as a typed open question. It is never a pass/fail bit, and it never asserts a stage the evidence does not establish (§4d, §5).

The chain is monolithic for Mt. Collins. §6 records the seams for Mt. Jade and x86, but this plan does not abstract over them. The operator's sequencing is that abstraction follows the second platform, and the differences found then become the onboarding checklist.

## 1. What already exists and is reused (§3, §3b)

The survey found most of the evidence readers already typed. What is missing is the ordering and the join. Each reader today ends in its own `List<String>` of findings.

| Concern | Owning authority (reused, not forked) |
|---|---|
| Firmware boot stages 0–9 and their status | `extdeps.ampere.scp_diagnostic` `AmpereBootStage` (`Smpro`, `Pmpro`, `AtfBl1`, `DdrInitialization`, `DdrTraining`, `AtfBl2`, `AtfBl31`, `AtfBl32`, `Uefi`, `OperatingSystem`) and `AmpereBootStatus` |
| SMpro stage registers 0xB0–0xB3 | `extdeps.ampere.smpro_register` `smpro_bootstage_of`, `smpro_cur_bootstage_of`, `SmproBootStage` |
| Per-socket SMpro reading | `gunbc.machine_intake_mtcollins1_smpro_observation` `SmproSocketStage`, `SmproProbeOutcome` |
| SMpro/PMpro internal error records (PR #13058) | `extdeps.ampere.smpro_internal_error` `SmproInternalRecord`, `SmproRecordGate` |
| DRAM console block and roster | `gunbc.machine_intake_ampere_dram_console_observation` `AmpereDramConsoleObservation` |
| Firmware boot summary, inter-socket links, CPER | `gunbc.machine_intake_ampere_socket_console_observation` `AmpereFirmwareSummaryObservation` |
| NVPARAM dump (PR #13059) | `gunbc.machine_intake_ampere_nvparam_console_observation` `NvParamConsoleObservation` |
| Firmware console statements, SEL boot cycles | `gunbc.machine_intake_pre_os_bringup_verdict` `FirmwareConsoleReport`, `boot_cycles` |
| SEL event vocabulary | `extdeps.bmc.ipmi_sel` `SystemBootInitiatedOffset`, `SystemFirmwareProgressOffset` |
| Ampere OEM SEL records (manufacturer 0xCD3A) | `extdeps.ampere.oem_sel` `AmpereOemSelRecordMatch`. It is in the repo but not on the boot path today. |
| BMC sensors | `gunbc.machine_intake_mtcollins1_bmc_sensor_observation` |
| GRUB | `extdeps.bootloader.grub` `GrubConsoleReport`, `GrubError` |
| Kernel and initrd modules | `gunbc.machine_intake_kernel_module_decompression_observation` |
| SOL, KVM, media | `extdeps.bmc.megarac`, `extdeps.bmc.serial_console`, `gunbc.machine_intake_mtcollins1_kvm_still` |
| The run and its phases (the harness's steps, not firmware stages) | `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_boot_phase_timing` `MtCollins1BootPhase` |
| Authored post-update outcome (PR #13041) | `gunbc.machine_intake_boot_outcome` `HostBootObservation` (`PostDdrProgressLost`, …) |
| The host | `gunbc.managed_host` `ManagedHost` (PR #13055) |

Run phases and firmware stages are different facts. `MtCollins1BootPhase` (sol_acquire … sol_release) is what *our harness* did. The stage chain is what *the machine* did. The fold reads the harness phases only as provenance, for example to establish when power-on happened.

## 2. The stage chain (Mt. Collins)

The chain is ordered, and every stage below is a row. Stages 0–9 are exactly `AmpereBootStage` and are not re-coined. The new vocabulary is only the stages outside the SCP's numbering, which are platform and BMC stages before stage 0 and OS-side stages inside `OperatingSystem`.

These are the evidence carriers each stage cites:

- **CON**: the SOL console, with lines timestamped on receipt.
- **SMP**: the SMpro registers over the BMC's private I2C, per socket.
- **ERR**: the SMpro/PMpro error records 0x7E and 0xA0–0xAD.
- **SEL**: the BMC system event log, before and after.
- **SDR**: BMC sensors.
- **CHS**: chassis power status.
- **KVM**: screen stills.

| # | Stage | Runs on | Evidence | Success | Stated failure | Silence / timeout | Reader today |
|---|---|---|---|---|---|---|---|
| P0 | AC / standby → BMC up | BMC (MegaRAC) | BMC reachability, `mc info`, firmware version | BMC answers IPMI and Redfish | BMC refuses or fails authentication. After a reflash, users and SEL are reset (self-observed, 0.45.3). | BMC unreachable | `BmcReachability`, `BmcAccessHeld`, `mtcollins1_boot_diagnostic_bundle` |
| P1 | Chassis power-on | BMC → board power sequencer | CHS, SDR rails, SEL `Power Unit` | Power is on and the rails are in range | A rail is out of range, or SEL shows power-down | Power never reads on | `PowerAfterAttempt`, `mtcollins1_bmc_sensor_observation` |
| 0 | `Smpro` | SMpro, per socket | SMP B0/B2 | B0 high byte 0, status Completed (`0002`) | status Failed | The register never answers | `SmproSocketStage` |
| 1 | `Pmpro` | PMpro, per socket | SMP, ERR | `0102` | ERR record present | — | `SmproSocketStage`, PR #13058 |
| 1a | Inter-socket (CCIX/2P) link bring-up | PMpro, both sockets | ERR on each socket, SMP of socket 1, CON roster (later), firmware summary (stage 8) | Socket 1 progresses beside socket 0. Summary shows 2 active sockets and Inter Socket Connection lines. | `ERR_CCIX_RCA_LINKUP_FAIL` (location 68, code 116). Self-observed on 2026-10-03 on both sockets, GPI-gated on socket 1 and ungated on socket 0. | Socket 1 SMpro stuck (`CUR=0x0001`, `BOOTSTAGE=0x03ff`, an inconsistent pair) | PR #13058 decoder. Nothing joins it to the chain today. |
| 2 | `AtfBl1` | AP core 0, socket 0 | SMP (`0201`), CON | Proceeds to stage 3 | — | — | `SmproSocketStage` |
| 3 | `DdrInitialization` | DRAM firmware | CON `DRAM FW version`, the `DRAM populated DIMMs:` roster, `CP:` words | Roster lists every populated DIMM | `FirmwareConsoleReport.TrainingRefusedByFirmware` | — | `AmpereDramConsoleObservation`, `FirmwareConsoleReport` |
| 4 | `DdrTraining` | DRAM firmware | CON `CP:` sequence | The sequence reaches the reference continuation after `00001a0a` (`00001a0b`, `00001a0c`, …) | A stated training refusal | The last `CP:` is followed by silence, power stays on, and the SoC restarts (see the note below) | **No `CP:` reader.** This gap is filled by new module A. |
| 5–7 | `AtfBl2`, `AtfBl31`, `AtfBl32` | ATF | SMP, CON | Reaches UEFI | — | — | `SmproSocketStage` |
| 8 | `Uefi` | AMI Aptio | CON firmware summary ("Number of active sockets", "Inter Socket Connection"), NVPARAM dump, POST codes, KVM | The summary is printed with active sockets equal to the populated sockets | CPER/BERT records | — | `AmpereFirmwareSummaryObservation`, PR #13059 |
| 8a | Boot loader (GRUB) | GRUB, from virtual media | CON | The kernel is loaded | `GrubBootFailed{GrubError}` | `GrubStillTrying` | `GrubConsoleReport` |
| 9a | Kernel EFI stub → initrd | Linux EFI stub | CON | The initrd is loaded | `Failed to load initrd: 0x8000000000000001` (EFI_LOAD_ERROR, intermittent read from the virtual CD), then a VFS panic | — | Only the authored `MtCollins1BootFinding.InitrdLoadFailedKernelPanic`. **No reader.** This gap is filled by new module A. |
| 9b | Linux userspace | Linux | CON, module decompression | Modules load | `ModuleInsertFailure` | — | `kernel_module_decompression_observation` |
| 9c | Census capture | our workload | host capture sections | `TerminalAccepted` | `TerminalRefused{causes}` | `TerminalPending` | `mtcollins1_boot_terminal_verdict` |

### Note: the stage 4 stop, public statement only

These are self-observed facts.

- **The stop.** In all five baseline0 cycles and all six step1 cycles, with J1+J17 and both CPUs fitted, the last console word is `CP: 00001a0a`. Only `SK0 MC0` appears in the roster.
- **What follows.** About 1.1 s after that word, SMpro stops answering. Chassis power stays on, and the console is silent until the next `DRAM FW version` banner. The restart period is about 22–26 s from the last checkpoint. The next cycle's SEL shows `S5/G2 soft-off` + `System Boot Initiated: System Restart`, with Ampere OEM records (`c0`, manufacturer `00cd3a`) beside them.
- **The same loop across layouts.** It repeated identically, with 6 cycles each, `CP: 00001a0a` last, a ~25.8 s period, `SK0 MC0` only and socket 1's SMpro B0 stuck at `0002`, in three layouts:
  - step 1: J1+J17 with both CPUs;
  - step 2: a different matched DIMM pair in J1+J17;
  - step 2b: J1 only with the socket-1 CPU installed. This exact layout booted under the old firmware on 2026-10-01.
- **No error was stated.** Across the 5 post-update captures, the console printed **zero** error lines: no `ERR:`, `fail`, `timeout` or window lines. "No stated failure" is therefore a literal observation, not an absence of reading.
- **Under old firmware** (GitHub run 37062170720), the same stop occurred on the first pass. A second pass continued `00001a0a → 00001a0b`, then on to UEFI.

The *mechanism* of the restart, and the *meaning* of the `CP:` word's fields, come from proprietary or disassembly-derived sources. Both stay in gunbc-private. The public model treats a `CP:` word as an opaque 32-bit value. Its observed order is compared against our own observed reference sequences, which are self-observed and public. The public fold may say "progress stopped after `00001a0a`; the reference continues with `00001a0b`; the restart cadence observed is ~22–26 s". It may not say why.

Private evidence is referenced by name only: gunbc-private, after #202 lands.

## 3. The fold: `BootUnderstanding`

The fold is one fold, `boot_understanding(host: ManagedHost, evidence: BootEvidence) -> BootUnderstanding`, with this shape:

```
BootEvidence {                      // all existing typed readings; nothing re-read
  bmc, chassis, sel_before, sel_after, sensors,
  smpro: per-socket SmproSocketStage at named instants,
  smpro_errors: per-socket SmproRecordGate,        // #13058
  console: ConsoleReading,                          // DRAM block, CP sequence, summary, NVPARAM, GRUB, EFI stub, modules
  terminal: MtCollins1BootTerminalVerdict,
  timing: MtCollins1BootPhaseTiming                 // provenance of instants only
}

BootUnderstanding {
  host: HostIdentity,
  reached: StageEstablished { stage: PowerOnStage, by: List<EvidenceCitation> },
  ending: BootEnding,
  errors: List<DecodedBootError>,
  open: List<OpenQuestion>
}

BootEnding =
    ReachedCensus { verdict }                              // no error found at any stage
  | StoppedWithStatedFailure { stage, error: DecodedBootError }
  | ResetWithObservedCadence {                        // console silent, a restart cadence IS observed; what #13041 calls PostDdrProgressLost
        at: StageEstablished,                             // located at the sub-step, e.g. DdrTraining @ last CP 00001a0a
        last_checkpoint?: CheckpointWord, reference_next?: CheckpointWord,   // compared opaquely
        restart_cadence: std.measure Second,              // observed ~25.8 s; no timeout values in public
        console_errors: Nat,                              // observed 0, not "unread"
        power_during_silence: ChassisPowerReading,
        restart: RestartObservation,                      // SEL S5 + System Restart
        reset_cause: ResetCauseObservation }
  | StoppedSilently { at: StageEstablished, last_checkpoint?: CheckpointWord, console_errors: Nat }   // silent, no cadence observed

ResetCauseObservation =
    Unobserved { searched: List<EvidenceCitation>, would_close: EvidenceCarrier }
  | Observed { cause: ResetCauseWord, raw: Word }
ResetCauseWord = ResetCauseUndefined { raw }       // public carries the raw word only; the decoded enum is PRIVATE (gunbc-private, after #202)
    // Today: Unobserved. No public register window reaches the SCP's reset-cause record (the 0x00-0xFF SMpro sweep found none).
  | EndingUnobservable { missing: List<EvidenceCarrier> }  // we cannot say where it ended

DecodedBootError =
    SmproRecord { socket, record: SmproInternalRecord }
  | FirmwareStatement { report: FirmwareConsoleReport }
  | BootLoader { error: GrubError }
  | InitrdLoadFailed { status_word }
  | KernelModule { failure: ModuleInsertFailure }
  | SummaryMismatch { difference: AmpereSocketConsoleDifference }

OpenQuestion = { subject: PowerOnStage, question: OpenQuestionKind, would_close: EvidenceCarrier }
```

`PowerOnStage` is `PlatformStage { P0 | P1 } | Firmware { AmpereBootStage } | InterSocketLink | BootLoader | KernelStub | Userspace | Census`. It wraps `AmpereBootStage`, it does not copy it.

These laws are enforced in the fold and each one gets a discriminating witness:

1. **No stage past the evidence.** A stage is `reached` only when a cited reading establishes it. A missing carrier yields `EndingUnobservable` or an `OpenQuestion`. It never yields a default "reached UEFI".
2. **Every refused or contradictory reading becomes an open question.** One example is socket 1's `CUR=0x0001` with `BOOTSTAGE=0x03ff`. Another is a GPI-gated error record whose gate reads `GpiUnavailable`. Neither may be dropped or coerced.
3. **No pass/fail projection is exported.** The receipt renders the whole understanding. Exit status stays the existing terminal verdict, and that verdict is no longer the carrier of the diagnosis.
4. **Independent findings stay independent.** The fold reports a `StageStanding` for every stage the evidence touches, not just one chain ending. Socket 0's progress gives the `ending`. A finding on another stage is its own standing and is not merged into the ending's story. The motivating case: socket 1's `CUR=0x0001` / `BOOTSTAGE=0x03ff` pair and the CCIX `ERR_CCIX_RCA_LINKUP_FAIL` record belong to `InterSocketLink`, and the private trace shows they are a separate finding from the DDR-training stall. Hence `BootUnderstanding` also carries `stages: List<StageStanding>`, where `StageStanding = { stage, standing: Reached | StatedFailure{error} | Anomalous{readings} | Unobserved }`.
5. **Retry state is a first-class unknown.** `OpenQuestionKind.RetryStatePersistence` asks whether any retry or failsafe state survives the reset. Its answer decides between bounded retries and an infinite loop. It is open on every `ResetWithObservedCadence` ending (public standing: unanswered) until evidence closes it.
6. **"Why" stays private.** `OpenQuestionKind` includes `RestartMechanismNotPubliclyModeled` and `CheckpointMeaningNotPubliclyModeled`. They are honest standing unknowns in the public repo. A gunbc-private fold may consume `BootUnderstanding` and close them. The public repo never imports private.

### Relation to PR #13041 `HostBootObservation`

`PostDdrProgressLost{last_checkpoint, soc_silent, chassis_power_on_while_silent, next_cycle_logged_s5_and_system_restart, …}` is the same fact as `BootEnding.ResetWithObservedCadence`, but it is *authored* rather than derived. Two types for one fact would be a §3 fork. Proposal: once this lands, `HostBootObservation` becomes a projection of `BootUnderstanding`, or is replaced by it. #13041's authored 2026-10-03 datum is then derived from the committed capture excerpt instead of typed in by hand. **Review question R1**: should #13041 land first and be cut over, or rebase onto this?

## 4. Consumption (§3c): what the fold replaces

- `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle` `mtcollins1_boot_bundle_findings` (`List<String>`) is **deleted**. The bundle carries `understanding: BootUnderstanding` instead. The per-reader `*_findings` string producers are deleted with it: `mtcollins1_smpro_findings`, `ampere_dram_findings`, `mtcollins1_sensor_findings`, `socket_summary_findings` and `kernel_module_decompression_findings`. Their content is now typed fields of the understanding.
- `mtcollins1_boot_outcome_with_findings` stops string-appending into `ExitFailure.reason`. In `gunbc.machine_intake_mtcollins1_boot_run`, the `findings=` receipt line in `mtcollins1_boot_conclude` becomes a rendering of the understanding through `std.observation` `ObservationPresentation`, which is the reporting home in §3b.
- `mtcollins1_socket1_investigation_observation` `MtCollins1BootFinding` (an authored ledger) stays as history. Its `InitrdLoadFailedKernelPanic` arm now has a reader, so new occurrences come from the fold and not from hand entry.

This is a replacement migration (§3): the string findings are deleted in the same change that introduces the fold, with no period where both exist.

## 5. Host as data

The fold takes `ManagedHost` from PR #13055, not `mtcollins1_*` constants. Socket count, SMpro addresses (0x4F and 0x4E) and the reference `CP:` sequences become per-platform rows keyed from the host's platform. For this plan the only row is Mt. Collins, as `mtcollins1`.

## 6. Seams (noted, not abstracted)

| Seam | Mt. Collins | Mt. Jade (#13065) | x86 |
|---|---|---|---|
| BMC family | AMI MegaRAC 0.45.3 | AMI MegaRAC, an Ampere JADE LTS SPX12 build | Varies: OpenBMC, iDRAC, iLO, … |
| Pre-OS stage authority | `AmpereBootStage` via SMpro | Same silicon, same registers. To be verified that the BMC exposes the same SMpro I2C route. | No SMpro. Uses BIOS POST codes (port 80) and IPMI POST codes, so it needs a different stage authority. |
| Inter-socket link | CCIX/2P, PMpro | Same | UPI/xGMI. Different error source (MCA). |
| DRAM training evidence | Ampere DRAM console block, `CP:` | Likely the same; reference sequences per platform | MRC messages, vendor-specific |
| Firmware summary and NVPARAM | AMI Aptio on Ampere, NVPARAM | Likely the same | None |
| Console route | MegaRAC SOL, which dies on reset (`DiesOnReset`) | MegaRAC SOL | Varies |

The column differences, once verified on the second platform, are the onboarding checklist the operator asked for.

## 7. Implementation slices (child PRs)

1. **A: the chain, the `CP:` and EFI-stub readers, and the fold, consumed by the bundle and the receipt.** This uses only what is on main: the SMpro stage, the DRAM block, the summary, GRUB, modules, SEL (including `oem_sel`) and chassis. It deletes the string findings in the same PR. It commits public-safe excerpts from the `mtcollins1-captures-2026-10-03` evidence branch as `artifacts/bmc/` fixtures. Witnesses: the step1, step2 and step2b captures each fold to `ResetWithObservedCadence{DdrTraining @ 00001a0a, console_errors 0, reset_cause Unobserved}` plus an independent `InterSocketLink` `Anomalous` standing; the 2026-09-13 two-socket good boot folds to `ReachedCensus`; run 37069907299 folds to `StoppedWithStatedFailure{InitrdLoadFailed}`. A deleted-carrier control yields `EndingUnobservable`, not a stage.
2. **B: SMpro/PMpro error records into the fold.** This is gated on #13058 landing. It adds the `InterSocketLink` stage's stated failure and gate standing.
3. **C: NVPARAM as stage-8 context.** This is gated on #13059.
4. **D: `ManagedHost` parameterisation.** This is gated on #13055. If #13055 lands first, it folds into A.
5. **E: cut over `HostBootObservation` (#13041).** Ordering depends on R1.

## 8. Review questions

- **R1 (working answer, from eager-gull-22, pending side chat).** #13041 lands first, and slice E absorbs `PostDdrProgressLost` into `ResetWithObservedCadence`. Original question: #13041's `HostBootObservation`: land it, then cut it over (E), or have #13041 consume `BootEnding` directly?
- **R2 (working answer, from eager-gull-22, pending side chat).** Yes, the opaque `CP:` reader is public. Original question: is a public `CP:` reader that treats words as opaque values, compared against self-observed reference sequences, inside the public/private line? I believe yes, because the words and their order are our own console observations and no field is decoded.
- **R3 (resolved by review, 2026-10-03).** `InterSocketLink` owns the socket 1 pair and the CCIX record, as a finding independent of the DDR stall (law 4).
- **Naming (deliberate departure from the proposal).** The operator's session proposed `ResetByPlatformWatchdog`. The plan names the arm `ResetWithObservedCadence`, because a public arm named for a watchdog asserts the restart mechanism. That mechanism is disassembly-derived and private, and the public evidence shows only the cadence (§4d). A private fold may refine it.
- **R4 (resolved).** Reset cause is an explicit slot, `ResetCauseUnobserved` today. Retry-state survival is a typed open question. "No stated failure" is grounded in an observed zero-error-line count.
