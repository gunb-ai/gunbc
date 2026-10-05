# The Mt. Collins power-on account (power-on sequence model, Mt. Collins first)

Status: PLAN. It encodes the side-chat rulings R1–R3 and Q1–Q2 and the corrections from the reviews of `e4e8a4a4d4` and `75f0bd113c`, all before any implementation. Owner: calm-lynx-884, under eager-gull-22 (hw-boot).

## 0. The requirement

Operator, 2026-10-03, verbatim: "take ALL of the information we've gotten today (and in our existing models) - and construct a monolithic 'this is what actually happens when you turn on the computer' workflow - meaning, our diagnostic process will be 'this is our best understanding of what happened' - basically, the end of the workflow (error, no error, any information we have) - and then, as we add more platforms (mt jade, x86), we will abstract the workflow and plug in the differing pieces as we need - this will give us an idea of how to onboard new platforms".

The product is one **power-on account** per attempt: `gunbc.machine_intake_mtcollins1_power_on_account`. It covers success, degraded success, stated failure, repeated restarts, transport failure, partial observation and unresolved cause. It is never a pass/fail bit, and it never asserts past the evidence (DESIGN §4d, §5). It is monolithic for Mt. Collins. §9 records the seams for Mt. Jade and x86, and abstraction follows the second platform.

## 1. Rulings this plan encodes

- **R1 (side chat, decided): #13041 lands first, and slice A replaces it atomically.** Slice A deletes `gunbc.machine_intake_boot_outcome` `HostBootObservation` and its authored datum `mtcollins1_post_firmware_set_boot_2026_10_03`. In the same change, it makes firmware convergence (`gunbc.fleet.mtcollins_firmware_converge` `boot_verdict`) consume an operation-specific readback projected from the **whole** session (§6). Reaching UEFI is a milestone, not an ending, so the projection never reads the ending alone. The former slice E is folded into A.
- **R2 (decided): the opaque `CP:` reader is public, and it establishes only an opaque sequence relation.** It may say "cycle C of capture S ended after token X; reference capture R (digest, firmware and configuration scope) contains X followed by Y". It may not establish `DdrTraining` or any other `AmpereBootStage`, may not call Y a stage success, and may not locate an ending "at DdrTraining @ X". Semantic attribution belongs to the gunbc-private refinement slice (§8).
- **R3 (decided): the refused SMpro pair belongs to the secondary-join subject and is not decoded there.** Socket 1's `CUR_BOOTSTAGE 0x0001` / `BOOTSTAGE 0x03ff` is `extdeps.ampere.smpro_register` `SmproBootProgressRefused`. It licenses no "stuck at PMpro" claim, no DDR claim, and no other last-stage claim. It is carried inside `gunbc.machine_intake_mtcollins1_socket1_investigation_observation` `MtCollins1SecondaryJoinOutcome` (#13025, on main), which this plan reuses and does not re-derive. It also gets an open question on the join subject.
- **Q1 (decided): a component-owned console line establishes only that component's console milestone.** Examples: AMI `Press <DEL> or <ESC>` gives `UefiConsolePromptObserved`; `UEFI Interactive Shell v2.2` gives the stronger `UefiShellObserved`; `GNU GRUB`, `EFI stub:` and `Linux version` give their exact component milestones. A milestone never establishes an `AmpereBootStage`, stage completion, health or a successful handoff. It carries the exact line and its parser, the cycle, the capture digest and its ordering evidence.
- **Q2 (decided): the account consumes one frozen, attempt-bound `AttemptConfigurationReceipt` taken before power-on** (§3). Historical rows supply topology only, never current occupancy: `mtcollins1_slots` is the 2026-09-10/11 16-DIMM census, and the `mtcollins1_physical_orientation` CPU rows are a 2026-09-23 operator report.
- **Fact correction.** **Socket 0's** CCIX record is GPI-pending (gated): in `smpro-err-20261003T005609Z`, slave 0x9E (socket 0) reads `GPI_RAS_ERR 0x7E = 0x0002` beside the ERR_CCIX_RCA_LINKUP_FAIL record (`0xA6 = 0x0074`, `0xA7 = 0x2244`). **Socket 1's** record is ungated per the retained hand read the side chat cites. In that capture, socket 1's (0x9C) error registers mostly refused (`rsp=0xff`), so the ungated reading rests on the hand read. The first draft had this reversed.

## 2. What already exists and is reused (§3, §3b)

| Concern | Owning authority (reused, never forked) |
|---|---|
| Firmware boot stages 0–9 | `extdeps.ampere.scp_diagnostic` `AmpereBootStage`, `AmpereBootStatus` |
| SMpro register pair and its refusal rule | `extdeps.ampere.smpro_register` `smpro_boot_progress_of` → `SmproBootProgress` (`SmproBootProgressRead` / `SmproBootProgressRefused`) |
| Per-socket SMpro reading | `gunbc.machine_intake_ampere_smpro_observation` `SmproSocketStage`, `SmproProbeOutcome`, read through `SmproPassObservation` |
| Second socket's join | `gunbc.machine_intake_mtcollins1_socket1_investigation_observation` `MtCollins1SecondaryJoinReading`, `secondary_join_outcome` → `MtCollins1SecondaryJoinOutcome` |
| SMpro/PMpro error records and GPI gate (#13058) | `extdeps.ampere.smpro_internal_error` `SmproInternalRecord`, `SmproRecordGate` |
| DRAM console block and roster | `gunbc.machine_intake_ampere_dram_console_observation` `AmpereDramConsoleObservation` |
| Firmware summary, inter-socket lines, CPER | `gunbc.machine_intake_ampere_socket_console_observation` `AmpereFirmwareSummaryObservation` |
| NVPARAM dump (#13059) | `gunbc.machine_intake_ampere_nvparam_console_observation` |
| Firmware console statements, SEL cycles | `gunbc.machine_intake_pre_os_bringup_verdict` `FirmwareConsoleReport`, `boot_cycles` |
| SEL vocabulary, Ampere OEM SEL | `extdeps.bmc.ipmi_sel`, `extdeps.ampere.oem_sel` |
| GRUB | `extdeps.bootloader.grub` `GrubConsoleReport` |
| Kernel module refusals | `gunbc.machine_intake_kernel_module_decompression_observation` |
| SOL collector, KVM, virtual media, handoff | `gunbc.machine_intake_sol_collector_observation`, `gunbc.machine_intake_mtcollins1_kvm_still`, the bundle's `MtCollins1MediaAttachRecord` / `MtCollins1HandoffMedia`, `extdeps.bmc.megarac` |
| Harness phases | `gunbc.machine_intake_mtcollins1_boot_phase_timing` `MtCollins1BootPhase`. These are what our harness did, never firmware stages. |
| Slot topology (J-label to socket/MC/slot), never current occupancy | `gunbc.machine_intake_mtcollins1_memory_census_observation` `mtcollins1_slots` (the 2026-09-10/11 census) |
| Firmware versions | #13041 `gunbc.fleet.mtcollins_firmware_baseline` |
| Authored outcome being replaced | #13041 `gunbc.machine_intake_boot_outcome` `HostBootObservation` |
| The host | #13055 `gunbc.managed_host` `ManagedHost` |

## 3. Shape: one attempt, three orthogonal standings, cycles of subjects

These three things are independent, and none may stand in for another:

1. **What the host did** (`HostAccount`): its cycles, and per cycle the standings of each subject.
2. **What we observed** (`ObservationCoverage`): one standing per carrier, saying whether it was read, refused, empty, partial or not taken, and why. An empty SOL capture is a coverage gap, never SoC silence. A BMC authentication refusal is a coverage gap on the BMC carrier, never a failed "BMC came up" stage.
3. **Whether our workflow worked** (`CollectionOutcome`): media attach and handoff, override consumption, power-after, bundle write, credential residue. A media-read failure or a bundle-write failure is a collection outcome, never a host ending.

Beside these sits the **attempt configuration receipt** (`AttemptConfigurationReceipt`). It is frozen and bound to one attempt, it is taken before power-on, and the account consumes only it. The fold never queries mutable or historical global rows. The receipt separates:

- **expected configuration and policy**: the expected topology this attempt is judged against;
- **the requested stimulus** (for example "remove the socket-1 CPU", "J1 only", "firmware set 2026-10-02") **from the applied-stimulus receipt**. For manual CPU or DIMM work, the applied receipt is a human completion/inspection receipt the workflow consumes. A workflow-authored request is not evidence that the change happened;
- **the current physical population** per socket and per slot, each field with its standing: `LiveObserved { source }`, `OperatorAttested { receipt }`, `Conflicted { readings }` or `NotRecorded { reason }`;
- **live firmware readbacks** (BMC, SCP, UEFI, CPLD), taken after the physical configuration is set and before actuation;
- **provenance**: subject (the `ManagedHost` row), attempt, timestamp, artifact and digest.

Static slot and orientation modules may map a label such as `J1` to topology. They may never supply current occupancy. A question that needs a field whose standing is `NotRecorded` or `Conflicted` becomes an open question, never a default.

### Cycles

A power-on attempt is a **session of cycles**. The old firmware stopped on cycle 1 and reached UEFI on cycle 2 (run 37062170720). The new firmware repeats one stopped cycle 6 times (steps 1, 2 and 2b). One cycle can have socket 0 reaching Linux while socket 1 never joins. So:

- **Cycle segmentation does not depend on reaching DRAM.**
  - Cycle 1 opens on the admitted chassis-power-on effect.
  - Each later cycle opens on typed restart/reset boundary evidence: SEL `System Boot Initiated` / `S5` records, power or reset observation, and console evidence.
  - A `DRAM FW version` banner binds a console span to a cycle. It is not the sole authority that a cycle exists, because a host can stop in SMpro, PMpro or ATF before the banner, and an empty SOL can sit beside SEL and SMpro evidence of a boot.
  - Each cycle carries its identity (index, boundary evidence, console span if any) and the capture digests.
- **Within a cycle, standings are keyed by subject**, not by one total chain:
  - `Platform`: the BMC and chassis power;
  - `Socket { k }`: that socket's SMpro progress via `SmproBootProgress`, and its error records via #13058;
  - `SecondaryJoin`: `MtCollins1SecondaryJoinOutcome`;
  - `DramFirmware`: the console block, roster, and untrained sockets established only by a closed roster;
  - `Uefi`: summary, NVPARAM, POST;
  - `BootLoader`, `KernelStub`, `Kernel` and `Census`.

### Stage attribution rule (R2 applied)

A **firmware stage** (`AmpereBootStage`) is established only by a public `AmpereBootStage` reading, which is an SMpro register pair decoded by `smpro_boot_progress_of` (`SmproBootProgressRead`). Console output establishes **console milestones** (Q1): exact component-owned lines, such as the DRAM firmware block, the firmware summary, `UefiConsolePromptObserved`, `UefiShellObserved`, the GRUB banner, an EFI stub line, the kernel banner and the census marker. A milestone never implies an `AmpereBootStage`, completion, health or handoff. An opaque checkpoint token implies neither. This is structural: only the `Socket{k}` progress reading can carry `SmproBootProgressRead` and so an `AmpereBootStage`, while console subjects carry milestone readings that have no stage field (§5).

## 4. Mt. Collins: subjects, carriers, and what success, failure and silence look like

| Subject | Runs on | Carriers | Success | Stated failure | Silence or refusal |
|---|---|---|---|---|---|
| Platform: BMC | MegaRAC on standby | BMC reads, `mc info` | Answers. The reflash resets users and SEL (self-observed, 0.45.3). | Not applicable: a refused read is a coverage gap | Coverage gap |
| Platform: chassis power | BMC → sequencer | Chassis read, SDR rails, SEL `Power Unit` | On, rails ok | Rail not ok (`mtcollins1_sensor_anomalies`) | Power never reads on |
| Socket k: SMpro/PMpro progress | SMpro, PMpro | SMpro registers per pass | `SmproBootProgressRead` advancing | Reported status `Failed` | `SmproBootProgressRefused` is an open question, never a stage |
| Socket k: error records | SMpro, PMpro | 0x7E, 0xA0–0xAD (#13058) | No pending record | Only a `PendingAndDecoded` record. On 2026-10-03, socket 0's `ERR_CCIX_RCA_LINKUP_FAIL` (loc 68, code 116) is the one current stated error | A matching payload under `GpiUnavailable` or ungated (socket 1 on 2026-10-03) is an anomalous raw record plus an open question. It is never a second `DecodedBootError` or `PendingErrorRecord` |
| SecondaryJoin | PMpro, both sockets | Summary active sockets and inter-socket lines, socket-1 SMpro pair | `SecondaryJoined` | `SecondaryJoinTimeout` (a liveness finding, not a fault record) | `SecondaryJoinUnclassified { reason }` |
| DramFirmware | DRAM firmware | Console block, roster, opaque `CP:` tokens | Roster closed with every populated slot listed (against the `AttemptConfigurationReceipt` population) | `FirmwareConsoleReport` training refusal | Last token followed by restart or silence (the ending rule, §5) |
| Uefi | AMI Aptio | Summary, NVPARAM, POST, KVM | Summary printed, active sockets = expected | CPER/BERT records, summary mismatch | — |
| BootLoader | GRUB from virtual media | Console | Kernel loaded | `GrubBootFailed` | `GrubStillTrying` |
| KernelStub | Linux EFI stub | Console (`extdeps.linux.efi_stub`, v6.8) | Initrd loaded | `Failed to load initrd: 0x8000000000000001` (EFI_LOAD_ERROR, run 37069907299) | — |
| Kernel | Linux | Console (`extdeps.linux.boot_console`, v6.8) | Banner, no panic | `Kernel panic - not syncing: …`, module refusal | — |
| Census | Our workload | Host-capture markers | END marker | `TerminalRefused` | `TerminalPending` |

### Self-observed facts the fixtures carry (public)

- **The new-firmware loop.** Steps 1, 2 and 2b each ran 6 identical cycles. In each: last token `00001a0a`, only `SK0 MC0` in the roster, socket 1's SMpro B0 stuck at `0002`, zero console error lines (no `ERR:`, `fail`, `timeout` or window lines), SMpro unresponsive about 1.1 s after the last token while chassis power stays on, then a restart. The next cycle's SEL shows `S5/G2 soft-off` + `System Restart` plus Ampere OEM records (`c0`, manufacturer `00cd3a`). The observer-clock period was ~25.8 s. The mechanism is not public.
- **Step 3, positive control.** With the socket-1 CPU removed (J1 only), the new firmware reaches the firmware summary (1 active socket) and the AMI UEFI shell, and stays stable for 8+ minutes. One earlier run restarted once after the summary, and that did not recur.
- **Old firmware.** Run 37062170720: cycle 1 stops after `00001a0a`, cycle 2 continues to UEFI. Run 37069907299: reaches UEFI with 1 socket, GRUB, then initrd EFI_LOAD_ERROR and a VFS panic.
- **The UEFI summary prints `Failsafe status : N` and `Reset status : N`.** They are carried as observed, undecoded summary fields (§7).

## 5. The account's types

```
PowerOnAccount {
  attempt: AttemptIdentity,                       // run id, capture digests
  host: HostIdentity,                             // from ManagedHost (#13055); mtcollins1 is a row, not a constant
  configuration: AttemptConfigurationReceipt,       // frozen before power-on; the only configuration the fold reads
  account: HostAccount,
  coverage: ObservationCoverage,
  collection: CollectionOutcome,
  open: List<OpenQuestion>,
}

HostAccount {
  cycles: List<BootCycleAccount>,
  milestones: List<MilestoneReached>,            // e.g. UEFI reached in cycle 2 -- a milestone, never an ending
  ending: HostEnding,
  degradations: List<Degradation>,               // orthogonal to the ending
  errors: List<DecodedBootError>,                // every stated failure, with its cycle and subject
}

BootCycleAccount { index: Int, span: CaptureSpan, standings: List<SubjectStanding>, checkpoints: CheckpointObservation }

SubjectStanding                                   // the reading's type is fixed by its subject, so no subject can carry another's evidence
  = PlatformStanding { part: PlatformPart, reading: PlatformReading }
  | SocketProgress { socket: Int, reading: SocketProgressReading }        // the ONLY carrier of an AmpereBootStage
  | SocketErrorRecords { socket: Int, records: List<SocketRecordReading> }
  | SecondaryJoinStanding { outcome: MtCollins1SecondaryJoinOutcome }
  | ConsoleSubject { subject: ConsoleSubjectKind, reading: MilestoneReading }  // DramFirmware | Uefi | BootLoader | KernelStub | Kernel
  | CensusStanding { reading: CensusReading }

SocketProgressReading = ProgressRead { progress: SmproBootProgressRead, pass } | ProgressRefused { pair: SmproBootProgressRefused, pass }
                      | ProgressStatedFailure { progress: SmproBootProgressRead } | ProgressNotRead { coverage_ref }
SocketRecordReading = PendingDecoded { record: SmproInternalRecord }             // a stated error
                    | UngatedRaw { raw, gate: SmproRecordGate }                  // anomaly + open question, never an error
MilestoneReading = MilestoneObserved { milestone, line, parser, capture_sha256, order } | MilestoneStatedFailure { error, line }
                 | MilestoneAnomalous { anomalies } | MilestoneNotObserved

HostEnding                                       // what the HOST did last; says nothing about errors on the way
  = ReachedCensus                                 // may coexist with errors and degradations
  | StoppedWithStatedFailure { cycle, subject, error }
  | RestartedRepeatedly { cycles: Int, last_cycle_ended_after: CheckpointRelation?, cadence: RestartCadence,
                          console_errors: Nat, reset_cause: ResetCauseObservation }
  | StoppedWithoutRestart { last_cycle_ended_after: CheckpointRelation?, console_errors: Nat }
  | HostEndingNotEstablished                      // only when the remaining HOST evidence establishes no ending

Degradation = SocketNotJoined{outcome: MtCollins1SecondaryJoinOutcome} | ActiveSocketsBelowExpected{active, expected}
            | DimmNotTrained{slot}                                  // only against a receipt field LiveObserved/OperatorAttested
            | PendingErrorRecord{socket, record: SmproInternalRecord}  // PendingAndDecoded only

CheckpointObservation { capture_sha256, tokens: List<OpaqueToken>, malformed: List<String> }
CheckpointRelation {                              // R2: an opaque relation, nothing more
  cycle: Int, capture_sha256, last: OpaqueToken,
  reference: ReferenceCapture { capture_sha256, firmware: FirmwareSnapshot, configuration: ConfigurationSnapshot },
  continuation: ReferenceContinues{next: OpaqueToken, remaining: Nat} | ReferenceEndsThere | NotInReference | ReferenceUnread{detail}
}

ResetCauseObservation = ResetCauseUnobserved { searched } | ResetCauseObserved { raw: Word }   // decode is private
RestartCadence = CadenceTimed { period: Second, cycles } | CadenceCounted { cycles }           // production SOL is not timestamped

ObservationCoverage { carriers: List<CarrierCoverage> }
CarrierCoverage { carrier: Carrier, standing: Read | ReadEmpty | Partial{detail} | Refused{cause} | NotTaken{reason} }
Carrier = Sol | Kvm | SmproRegisters | SmproErrorRecords | Sel | Sdr | ChassisPower | RedfishInventory | VirtualMedia | HostCapture

CollectionOutcome { media, handoff, override, power_after, bundle_write, credential, phases: MtCollins1BootPhaseTiming }

OpenQuestion { subject: PowerOnSubject, question: OpenQuestionKind }
OpenQuestionKind = ResetCauseNotPubliclyReadable | RetryStatePersistence | CheckpointMeaningNotPubliclyModeled
                 | SmproPairRefused{socket} | RestartCadenceNotTimed | ConfigurationNotRecorded{field}
                 | CarrierNotRead{carrier} | UngatedRecordUnexplained{socket} | ConfigurationConflicted{field}
```

Laws, each with a discriminating RED in slice A:

1. **`ReachedCensus` is orthogonal to errors.** A GPI-pending CCIX record plus a one-socket census yields `ReachedCensus` with `degradations` (`SocketNotJoined`, `PendingErrorRecord`). It is never `StoppedWithStatedFailure`.
2. **Cycles are preserved.** Run 37062170720's console yields two cycles: cycle 1 ended after `00001a0a`, cycle 2 has the milestone "UEFI reached". The ending reflects the last cycle, and the milestone survives.
3. **Coverage and collection never choose the host ending.** An empty SOL, a BMC authentication refusal or a bundle-write failure is recorded in `coverage` or `collection` and nothing else. If another carrier establishes `ReachedCensus`, the UEFI shell or a reset, that host result stands beside the coverage defect. `HostEndingNotEstablished` applies only when the remaining host evidence establishes no ending. Converse RED: `SOL = ReadEmpty` with a valid host-capture END marker yields `ReachedCensus` plus an SOL coverage gap.
4. **No stage from a token.** No law or arm takes an `AmpereBootStage` from a `CP:` token. Mutating the last token changes only `CheckpointRelation`.
5. **The refused pair is not a stage.** `SmproBootProgressRefused` on socket 1 produces `SecondaryJoin` evidence and an `SmproPairRefused` open question, and no `Socket{1}` stage.
6. **Configuration-dependent questions need the receipt.** If the receipt's DIMM population is `NotRecorded` or `Conflicted`, "every populated DIMM trained" is an open question, never assumed. A historical row never stands in: a step-3 attempt whose receipt says "socket 1 CPU absent, J1 only" is judged against that receipt, never against `mtcollins1_slots` or the 2026-09-23 orientation report.
7. **A cycle exists without DRAM.** A chassis power-on plus SMpro progress or SEL boundary evidence, with no `DRAM FW version` banner, yields one partial cycle whose console span is absent. It never yields zero cycles or `HostEndingNotEstablished` on that ground alone.
8. **Ungated is not stated.** A `GpiUnavailable` or ungated record with the same payload as a pending one is `UngatedRaw` plus `UngatedRecordUnexplained`. It adds nothing to `errors` or `degradations`.

## 6. Consumption and the replacement migration (§3, §3c)

**Firmware convergence (R1).** `gunbc.fleet.mtcollins_firmware_converge` `boot_verdict` stops taking `HostBootObservation?`. It takes `FirmwareSetBootReadback`, projected from the whole account by `firmware_set_boot_readback(account)`:
- `BootReachedUefiAfterSet { cycle, milestone, ending: HostEnding, degradations, account_ref }`, when any cycle reached a UEFI milestone. The session ending and a reference to the whole account travel with it, so a UEFI milestone cannot erase a later `RestartedRepeatedly` ending. Convergence decides with both in hand.
- `BootProgressLostAfterSet { relation: CheckpointRelation, cadence, reset_cause }`, when no cycle reached UEFI and the ending is `RestartedRepeatedly` or `StoppedWithoutRestart`.
- `BootReadbackUnobservable { coverage }`, otherwise.

`BootProgressLostAfterFirmwareSet` keeps its role as the convergence refusal and takes the relation instead of an authored checkpoint integer. `HostBootObservation`, `mtcollins1_post_firmware_set_boot_2026_10_03` and the `gunbc.machine_intake_boot_outcome` module are deleted in the same change. The authored 2026-10-03 datum is replaced by an account derived from the committed capture excerpt (step 1), which is that witness's fixture.

**Consumer and semantic census for every deleted producer.** Each finding gets a typed destination or an explicit retirement:

| Deleted | Consumers today | Destination |
|---|---|---|
| `…bundle` `mtcollins1_boot_bundle_findings` | bundle `mtcollins1_boot_bundle_text`, `mtcollins1_boot_outcome_after_bundle`; `…boot_run` `mtcollins1_boot_conclude` (`findings=` receipt line); bundle witness | The `PowerOnAccount` rendered through `std.observation` presentation. Receipt `findings=` becomes `account=` (one summary line) plus `coverage=` and `collection=` |
| `…bundle` `mtcollins1_boot_outcome_with_findings` | `mtcollins1_boot_outcome_after_bundle`; bundle witness | Same role. It appends the account summary plus its errors and degradations to a refusal, and never flips the verdict (kept REDs) |
| `bmc_all_failed_text` aggregate (the "unreachable" finding) | bundle findings | `coverage`: BMC carriers `Refused { cause }`. Retired as a host statement |
| `processor_findings`, `memory_findings`, `socket_absent_finding` | bundle findings | `coverage` plus `Degradation` against the `AttemptConfigurationReceipt` (BMC's inventory view vs the expected topology) |
| `sel_findings` | bundle findings | Per-cycle SEL events: memory/processor events go to `DramFirmware` / `Socket` anomalies, and restart records go to cycle segmentation |
| `sensor_findings` → `…bmc_sensor_observation` `mtcollins1_sensor_findings` | bundle; sensor witness | `mtcollins1_sensor_anomalies` (typed, in the sensor module) on `Platform{ChassisPower}` |
| `smpro_findings` → `gunbc.machine_intake_ampere_smpro_observation` `smpro_pass_findings` | bundle (3 passes); SMpro witness | `Socket{k}` standings via `smpro_boot_progress_of`, and `SecondaryJoin` via `secondary_join_outcome`. The "sockets differ" string is retired: the difference is now the two typed standings |
| `console_findings`: `…dram_console_observation` `ampere_dram_findings` | bundle; DRAM witness | `ampere_dram_untrained_sockets` (closed-roster rule, in the DRAM module) feeds `DegradationDimmNotTrained` / `DramFirmware` anomalies; malformed rows become `DramFirmware` anomalies |
| `console_findings`: `socket_summary_findings` | bundle | `SecondaryJoin` via `secondary_join_outcome`, and `ActiveSocketsBelowExpected` against the receipt's expected configuration |
| `console_findings`: `…kernel_module_decompression_observation` `kernel_module_decompression_findings` | bundle; kernel-module witness | `Kernel` `StatedFailure { KernelModuleRefused }` |
| `media_attach_findings`, `handoff_media_findings`, `override_findings`, `power_after_findings` | bundle; `megarac_media_convergence_witness_test` | `CollectionOutcome` (typed records, rendered once) |
| credential-shred string | bundle findings | `CollectionOutcome.credential` |
| `HostBootObservation` and its datum (#13041) | `mtcollins_firmware_converge` `boot_verdict`; its witness | `FirmwareSetBootReadback` (above) |

**Consumers outside the required gate.** The production consumers are all in this closure: `mtcollins1_boot_run` (its fleet-converge entry) and the bundle. The test consumers are the witnesses of `mtcollins1_boot_diagnostic_bundle`, `ampere_smpro_observation`, `ampere_dram_console_observation`, `mtcollins1_bmc_sensor_observation`, `kernel_module_decompression_observation`, `megarac_media_convergence` and `mtcollins_firmware_converge`. Each one is migrated in slice A to the typed destination above, with each of its discriminating REDs kept. None is deleted without its claim moving.

## 7. Reset cause, retry state and the summary's status lines

The SCP keeps a reset-cause record, and no public register window reaches it: the full 0x00–0xFF SMpro sweep (`sweep-20261003T014255Z`) found none. So `ResetCauseObservation` is publicly `ResetCauseUnobserved { searched }`. `RetryStatePersistence` asks whether any failsafe or retry state survives the reset, which decides bounded retries versus an infinite loop. It is open on every `RestartedRepeatedly` ending. On cycles that reach UEFI, the summary's `Failsafe status` and `Reset status` fields are carried as raw observed values and cited as evidence on those two questions, never decoded publicly.

## 8. The gunbc-private refinement slice (named)

The private slice is **P: the Mt. Collins power-on account refinement**, in gunbc-private. It consumes the public `PowerOnAccount` and the private evidence (gunbc-private #202/#203: disassembly and SCP notes). It returns a private refinement that:
- attributes `CP:` tokens to stages and sub-steps (e.g. the DDR training eye sweep);
- names the restart mechanism (the SCP boot-complete wait and the watchdog);
- decodes `ResetCauseObserved` and the summary's status fields;
- answers `RetryStatePersistence` where the private evidence can.

It imports the public account, the public repo never imports it, and nothing private flows back into public.

## 9. Seams (noted, not abstracted)

| Seam | Mt. Collins | Mt. Jade (#13065) | x86 |
|---|---|---|---|
| BMC family | AMI MegaRAC 0.45.3 | AMI MegaRAC, Ampere JADE LTS SPX12 | Varies |
| Stage authority | `AmpereBootStage` via SMpro | Same silicon. To verify that the SMpro I2C route is exposed. | No SMpro: port-80/IPMI POST codes, a different authority |
| Second socket | CCIX/2P, PMpro, `MtCollins1SecondaryJoinOutcome` | Same, needs its own join row | UPI/xGMI, MCA |
| DRAM evidence | Ampere DRAM console block, opaque `CP:` | Likely the same, with references per platform | Vendor MRC messages |
| Summary and NVPARAM | AMI Aptio on Ampere | Likely the same | None |
| Console route | MegaRAC SOL (`DiesOnReset`) | MegaRAC SOL | Varies |

The differences found when Mt. Jade lands become the onboarding checklist.

## 10. Implementation slices

1. **A (#13117): the power-on account, replacing every string finding and `HostBootObservation`, after #13041 lands.** Former slice B (#13058's error records) is folded in, and §10a records the decisions. It includes:
   - the public readers: opaque `CP:` (`gunbc.machine_intake_ampere_checkpoint_console_observation`), and the EFI stub and kernel lines (`extdeps.linux.efi_stub`, `extdeps.linux.boot_console`, read by `gunbc.machine_intake_linux_boot_console_observation`);
   - the `AttemptConfigurationReceipt` (frozen before power-on, including the human completion/inspection receipt for manual changes), and coverage;
   - the account;
   - the convergence readback;
   - the bundle and receipt cutover;
   - every census row in §6, plus the REDs of §5.

   Fixtures are committed excerpts from the `mtcollins1-captures-2026-10-03` evidence branch, including step 3 as the new-firmware positive control, and the run 37062170720 / 37069907299 artifacts. A pre-review draft of the readers and a fold exists on `session/calm-lynx-884-slice-a-wip`. It predates these rulings (it uses a scalar ending, stage attribution from the DRAM banner, and its own SMpro pair rule) and will be reworked, not landed as-is.
2. **B: the attempt-configuration receipt's live inputs**: the applied stimulus, current population and pre-power-on firmware readback (decided Q4). The error records originally planned as B landed in A.
3. **C: NVPARAM** as UEFI-subject context, gated on #13059.
4. **D: `ManagedHost`**, gated on #13055. If #13055 lands before A, it is folded into A.
5. **P: the gunbc-private refinement slice** (§8).

## 10a. Decisions taken while implementing slice A (#13117)

These were settled while building slice A and through its side-chat reviews. Each one is enforced by a witness in #13117 and recorded here so this plan stays the authority.

- **Cycles open only on typed boundary evidence.** Each SEL boot record gets a `SelBootBoundaryStanding` from an ordered fold with no look-ahead:
  - the first record corroborates the confirmed power-on when it is a power-up;
  - the first record is `BoundaryAmbiguous` when it is a restart after a confirmed power-on. It opens no cycle and becomes an open question, because this MegaRAC can log the power-on's own first boot as "System Restart" (step 1);
  - every later record, including a second power-up, is a distinct boundary.

  Console banners only bind spans to cycles, and only when the counts agree; otherwise the binding is `ConsoleSpansUnbound`. The BMC clock is never joined to ours.
- **Stated failures are counted per occurrence.**
  - Panics and initrd failures are read per printed line.
  - Firmware statements, GRUB reports and module refusals are read by their owning readers over each console span's own lines.
  - Each error carries its cycle. Only a failure in the last cycle can choose the ending, and the furthest one there wins. Earlier and unbound errors stay as history.
- **A loop is distinguished from progress after a restart.** `RestartedRepeatedly` applies only when the last cycle got no further than an earlier one. Otherwise the ending is `StoppedWithoutRestart { restarts_before }`.
- **Topology, population and absence are judged only against the receipt.**
  - The expected topology is `ExpectedSockets`, or `ExpectedTopologyNotRecorded` (production today).
  - Processors are checked once per expected socket: missing, duplicated, or unreadable socket ID.
  - Memory is checked per socket, over the union of expected-populated and observed sockets, against `SlotRow { label, socket, populated }`.
  - Sensor absence counts only for an expected socket.
  - Health states reported by the BMC stay unconditional.
- **SMpro.**
  - Passes are reported per pass, not placed into cycles.
  - #13058's records are placed by their gate (pending → stated error plus degradation; ungated → anomaly plus open question).
  - Pending warnings are kept.
  - The error-record registers have their own coverage carrier, separate from the boot-stage pair.
- **Each milestone carries provenance.** It has its capture digest and a `DeclarationRef` to its reader. A one-variant nullary sum does not resolve in the substrate, so a declaration citation is used instead.
- **Convergence refuses UEFI followed by a loop.** It reads the whole-account readback. A UEFI milestone followed by a `RestartedRepeatedly` ending refuses as `BootReachedUefiThenRestartedRepeatedly`.
- **#13025's `secondary_checkpoints` is deleted (Q3).** The per-socket CP classification belongs to the private refinement slice.
- **The receipt's live inputs are slice B (Q4).** Slice B adds the boot-workflow input for the applied stimulus (with a human inspection receipt through the operator-attested route), the current population, and the pre-power-on firmware readback. Until it lands, those fields are `NotRecorded` and every question needing them is open.

## 11. Questions

Q1 and Q2 were decided by the side chat on `75f0bd113c` and are encoded in §1, §3 and §5. Q3 (delete `secondary_checkpoints`) and Q4 (add the receipt inputs, as slice B) were decided during slice A (§10a). Q5 and Q6 are decided; none is open:
- **Q5 (decided by eager-gull-22, 2026-10-03):** the plan and the inspection receipt are committed JSON under `artifacts/receipts/`, read from the checkout by a fail-closed typed reader that records the file's digest. One dispatch input names the file. Inline JSON in a dispatch input is refused.
- **Q6 (decided, 2026-10-04):** add the `attempt_receipt` input to the fleet-converge `mtcollins1_boot` mode. It was operator escalation msg_1b57e749, approved by default after 15 minutes with no answer, and relayed by eager-gull-22.

## 12. Slice B: the attempt receipt's live inputs (plan, for review before code)

Slice B fills the `AttemptConfigurationReceipt` fields that slice A records as `NotRecorded` (decided Q4). The boot run takes its host as a `ManagedHost` / `ManagedHostBinding` (`gunbc.managed_host`), not as mtcollins1 constants; cut 4d of the managed-host untangle will re-root the rest of the run.

**Two records, each minted only by its checks.**

```
AttemptConfigurationPlan (sole constructor) {
  subject: ManagedHost, attempt: AttemptPlanId,
  expected: ExpectedTopology, requested: StimulusRequest,   // expected and requested are stated ONCE, here
  fixed_at: OperatorAttestedTime,
}
AttemptInspectionReceipt (sole constructor) {
  plan: AttemptConfigurationPlan,
  applied: StimulusApplication, cpus: PopulationReading<SocketCpuRow>, dimms: PopulationReading<SlotRow>,
  completed_at: OperatorAttestedTime, evidence: ReceiptEvidence, witnessed_by: NonEmptyStr,
}
OperatorAttestedTime { rendering: NonEmptyStr, attested_by: NonEmptyStr }   // a time a person wrote, never an observer clock reading
ReceiptEvidence { path, digest: Sha256FileDigest, commit }   // the committed file the run read
```

**The attempt identity comes from the dispatch, not from the artifact.** The current run's attempt identity is minted outside the receipt: it is the fleet-converge dispatch's existing `transaction_nonce`, which the operator mints and which the run name echoes. Admission runs before any pre-power read or actuation and joins `receipt.plan.attempt` to that identity. It then consumes the identity in a **create-once slot**:
- The slot is keyed by (managed subject, `transaction_nonce`), in its own attempt-admission namespace, separate from the unit hold.
- `Absent -> Consumed` is the only successful transition, done by compare-and-set against `Absent` (`std.durable_compare_and_set`).
- No hold release or hold cleanup deletes or rewrites a slot.

So a used identity stays used: it is not blocked by a later one, and it does not block one. A valid receipt authored for attempt A cannot be selected by attempt B, and A cannot be replayed after B.

Controls, each an implementation RED:
- the first A succeeds;
- a concurrent or repeated A refuses;
- B then succeeds;
- A still refuses after B;
- releasing the unit hold does not erase A's consumed standing.

Implementation cut obligations:
- The workflow's `transaction_nonce` description changes from "not consumed by any step" to its admission role.
- A named receipt with an empty or absent `transaction_nonce` refuses.

The run mints an `AttemptInspectionReceipt` only after every check passes, and each failed check refuses as its own typed cause:
- the file's digest matches what was read;
- the subject is the host this run's `ManagedHostBinding` bound;
- the plan's attempt equals this dispatch's attempt identity;
- the attempt identity was not already consumed;
- the plan's identity is the one the receipt carries;
- the attested ordering holds: `fixed_at` is at or before `completed_at`, as attested.

**Times.** The plan's and receipt's times are what a person wrote, so they are `OperatorAttestedTime`, never `ObserverTimestamp`. They are ordered only among themselves. "Before the power write" is established STRUCTURALLY: admission is a step this run completes before it actuates. It is never established by comparing an attested time with an observer clock.

**The absent-receipt rule (decided by eager-gull-22).**
- **No receipt named:** the dispatch input is empty. The boot proceeds with the configuration `NotRecorded` and no topology judgement, as in slice A.
- **A receipt named but refused:** a missing file, a digest or parse failure, an empty `transaction_nonce`, a subject, host, attempt or plan mismatch, a consumed identity, or an attested ordering violation. The boot REFUSES before any pre-power read or actuation, with that typed cause.
- REDs, one per arm:
  - an empty input reaches actuation with the configuration `NotRecorded`;
  - a valid receipt for attempt A dispatched as attempt B refuses before actuation;
  - a replayed identity refuses;
  - a malformed file refuses.

The pre-power-on firmware readback is bound to the same plan (`FirmwareReadBeforeActuation { plan, rows }`), so a readback from another attempt cannot fill this one. A receipt that is absent or refused leaves the fields `NotRecorded` or refused with their cause. The boot never proceeds on a guessed configuration.

**The operator-attested route** follows `std.human_intervention`:
- The inspection is a `HumanIntervention` step with `HumanSurfaceOnly`, because a physical change has no API.
- Its `DischargedAt { evidence }` cites the evidence PRODUCER: the fail-closed reader that mints `AttemptInspectionReceipt` from the committed file, a `DeclarationRef` to that function. It does not cite a flag or a boolean standing.

**The pre-power-on Redfish population read is not live by default.** A controller reading taken before power-on may be the BMC's cache from the last POST, so it is `PopulationControllerReading { freshness: FreshnessEstablished | CachePossible, rows }`:
- It can corroborate the inspection or conflict with it. A conflict is `PopulationConflicted`, an open question, never resolved by preference.
- Only `FreshnessEstablished` counts as a live reading. Nothing on main establishes that freshness today, so today's reading is `CachePossible`.

**Field routes.**

| Field | Route |
|---|---|
| expected topology and requested stimulus | the plan, stated once |
| applied stimulus and population | the inspection receipt; the controller reading only corroborates or conflicts |
| firmware | a new pre-power-on `hpm check` read through `extdeps.bmc.ipmi` (none exists on main; `gunbc.fleet.mtcollins_firmware_converge` only renders a dry argv), plus the SMpro version word where it answers, both bound to the plan |

**Delivery (Q5, decided by eager-gull-22 on 2026-10-03).** The plan and the inspection receipt are committed JSON under `artifacts/receipts/`, reviewed and versioned like any change. eager-gull-22 authors them from what the operator reports about the physical change. One fleet-converge dispatch input names the receipt file, and the boot run reads it from the checkout with the fail-closed typed reader above. Inline JSON in a dispatch input is refused because it is not reviewable. Adding that input is Q6, decided.

### 12a. Slice B1 as built

Slice B1 is `gunbc.host_boot_attempt_admission`. It holds the plan and inspection records, the reader, the create-once attempt slot, and the projection into `AttemptConfigurationReceipt`. It is host-generic: mtcollins1 appears only as its route row (the receipt variables and the slot roster), in the `gunbc.host_maintenance_hold_reason` pattern. It returns one sealed `BootAttemptClearance`.

**Placement** (agreed with warm-crane-577): the module sits beside the boot authorization. `mtcollins1_boot_under_live_unit_hold` calls `admit_boot_attempt(proof, revision)` right after `UnitHeld`, so admission runs under the hold and **before any controller read**.
- **Baseline:** the SDR cache and SEL baseline (`mtcollins1_boot_baseline`) are taken only after clearance.
- **Refused receipt:** the boot reads nothing from the controller, releases the hold, writes nothing, and fails with the typed cause. The matrix control is `a_refused_named_receipt_reads_no_baseline_and_writes_nothing`.
- **Configuration record:** the frozen configuration travels on the attempt record into the bundle. Slice A's always-`NotRecorded` placeholder is deleted.
- **Untangle cuts:** cuts 4a and 4d move the call site, and O2 carries its refusals.

**The receipt is a tracked blob, digested.**
- **Revision binding:** admission reads the receipt at the boot's bound revision, not from the worktree. `extdeps.git.inspect` `ListTreeEntryAtPath` finds the entry, decoded by the existing `gunbc.namespace_step0_subject_collector` ls-tree reader. `git show <rev>:<path>` reads the content.
- **Refusals:** no entry is `ReceiptNotTracked`. A symlink, gitlink or directory is `ReceiptNotRegularFile`.
- **Evidence:** `ReceiptEvidence { path, digest: Sha256FileDigest, commit }` carries the SHA-256 of exactly those bytes, from `extdeps.tools.sha256sum`.
- **Follow-up:** the step0 decoder belongs in `extdeps.git`. Extracting it is left to the namespace lane rather than forked here.

**Times** are admitted only as canonical UTC instants, using `gunbc.auth.approval_capability` `utc_instant_is_canonical`, which checks calendar-valid fields. They are ordered with `utc_instant_before`, and equal instants are admitted.

**Populations join exactly, or the receipt refuses.**
- **CPUs:** the CPU rows name exactly the plan's expected sockets.
- **DIMMs:** the DIMM rows name exactly the host's slot roster. For Mt. Collins that is the Getting Started Guide's 32 connectors (`dimm_figure_banks`), labelled as the guide labels them (its `J` prefix and the connector number) and placed on their bank's socket. Every label must be known and on its roster socket, and every slot must appear, populated or not.
- **Refusal causes:** a missing socket, an unknown label, a label on another socket, and an omitted slot each refuse with their own cause.

**The slot store** is `/var/lib/gunbc/boot-attempts`, provisioned by `gunbc.runner_host_grants` `unit_hold_store_operations` beside the unit-hold store: same hosts, owner and mode. It is a separate directory, so a hold's release or recovery cannot reach it. Its executed control on the real store host is the first grant convergence followed by a receipt-carrying boot on srv1.

Where the build differs from the plan above, with reasons:
- **Slot key.** The key is `attempt-<len(host)>-<host>-<nonce>`, with the host and nonce admitted only over `[A-Za-z0-9_-]`. It is injective by construction and is not a hash: the corpus's `content_hash_of_value` is a 64-bit structural hash, which does not meet "collision-safe" against a chosen nonce.
- **The plan's identity.** The inspection names its plan by `plan_subject` and `plan_attempt`. Because an attempt identity admits one boot, that pair identifies the plan.

### 12b. Slice B2 as built (firmware), and B3 (controller population)

Slice B2 adds one pre-power read, `mtcollins1_boot_pre_power`. It is taken after admission and before `mtcollins1_boot_actuate_held`, and `gunbc.host_boot_attempt_admission` `attempt_configuration_with_pre_power` joins it to the frozen configuration.
- **The read.** `ipmitool hpm check` is a new `extdeps.bmc.ipmi` `HpmCheck` operation. It is parsed by `extdeps.bmc.ipmitool_hpm_check`, which landed with the CPLD route fix (#13254).
- **The rows.** Each component's active cell is kept as rendered, because the auxiliary bytes have no public decode.
- **Binding.** The readback is `FirmwareReadBeforeActuation { attempt, source, rows }`, where `attempt` is the attempt admission bound. An unread or refused table leaves firmware `NotRecorded` with its cause.
- **The acceptance matrix.** The dry BMC (`gunbc.bmc_dry_realization`) answers `HpmCheck` with the retained BMC 0.32 capture, as a layout fixture.

**B3: `PopulationControllerReading` (declared frontier).** B3 is the pre-power Redfish population read. It is labelled `CachePossible`, and is joined to the inspection as corroborated or conflicted per socket.
- **Why it is not in B2:** its route dispatches `shell.Mktemp.Dir` and `shell.Remove.FileForce` (for the netrc) and `redfish.Http.GetResourceByPath`. The boot dry world does not model these, and no mtcollins1 Redfish response is retained to model them from.
- **Trigger:** a retained mtcollins1 Redfish capture of the service root, Systems, the system, Processors and Memory. eager-gull-22's read-only probe takes it when the BMC is reachable.
- **Caveat:** after the 2026-10-02 reflash, gunbc gets 401 on Redfish because its role is missing. If the capture is 401 bodies, the trigger also needs the login convergence to restore that role.
- **Already written:** the join and folds are in WIP commit `16a67dfb99d` on `session/calm-lynx-884-slice-b2`.