# Machine intake: hardware qualification, provisioning, placement commissioning

Operator ruling of 2026-08-29, transcribed as the authority for the `gunbc.machine_intake_*`
lane. The `.dag` modules cite this document; this document does not restate what they declare.
Where a section below names a type, the type is the authority and this is its rationale.

Status: **INTAKE-0 and the BOOT-DELIVERY-0 modeling landed** (gunbc#TBD). Executed
Mt. Collins and ASRock boots reaching the nonce-bound intake-agent callback are BOOT-DELIVERY-0's
acceptance and are still owed; INTAKE-AGENT-0, MEMORY-0 and SOAK-ADMISSION-0 are unbuilt.

## 0. The ruling, in one paragraph

The correct system is one hardware-independent qualification transaction with multiple
platform-specific access and boot-delivery realizations. "Virtual CD" is not a phase. Redfish,
OEM NBD/WebSocket, OpenBMC configfs USB gadget, UEFI HTTP, and PXE are alternative constructors
for one obligation: *deliver this content-addressed boot artifact to this identified host for one
boot, prove that the intended artifact executed, and restore the prior boot-control state.*
(`gunbc.boot_artifact_delivery`.)

Four corrections to the original brief:

1. Board serial is the durable unit key but is not enough to key a qualification. A repaired
   machine keeps its board serial while every DIMM changes. Qualification is keyed to the board
   serial plus an assembly-manifest digest (`gunbc.machine_intake_subject`).
2. Hardware qualification, OS provisioning, and placement commissioning are separate
   transactions. They share boot delivery and receipts; success in one must not fabricate success
   in another (`gunbc.machine_intake_phase`).
3. NFS traffic, a redirection session, boot-rail activity, KVM pixels, and a consumed boot
   override prove activity, not test success. A memory verdict needs a machine-readable report or
   an intake-agent receipt (`attest_diagnostic_boot`).
4. BMC-reported watts are not automatically wall watts, and EDAC reports are not automatically
   DIMM-attributed. Both need explicit provenance types (SOAK-ADMISSION-0, MEMORY-0).

Operationally: run all return-window-sensitive hardware testing immediately on arrival, before
shelving; run a second placement-specific commissioning soak after installation in its actual
power, cooling, and network environment; nothing serves until both have succeeded. srv2 is the
current candidate realization of an abstract staging service, not its identity
(`gunbc.machine_intake_staging`).

## 1. The subject being qualified

```
QualificationSubject = UnitKey × AssemblyManifestDigest × FirmwareManifestDigest
UnitKey              = validated unique baseboard serial
AssemblyManifest     = chassis identity × CPU × DIMM × accelerator × storage × NIC × PSU identities
FirmwareManifest     = BMC implementation/version × BIOS/UEFI × CPLD/SCP/platform firmware × configuration digests
MachineIntakeSubject = QualificationSubject × IntakeAttemptId
```

| Event | Consequence |
|---|---|
| BMC IP changes | Same unit; no identity change |
| Machine moves racks | Same unit and assembly; placement receipt becomes stale |
| One DIMM changes | New assembly subject; previous admission cannot apply |
| BIOS/BMC firmware changes | New firmware subject; affected qualification must be repeated |
| Motherboard changes | New unit key |
| Board serial absent, duplicated, or contradictory | Identity refuses; no fleet key is fabricated |

The chassis part number is a first-class identity fact (platform-variant discriminator), not the
unit key. A physical DIMM label is bound to its SPD identity (`PhysicalDimmLabelBinding`); an
absent or duplicated SPD serial is preserved as a limitation, never papered over by the label.

## 2. Platform facts vs live access observations

`extdeps.bmc.access_profile`: `BmcAccessProfile` is platform/catalog data keyed by baseboard ×
chassis part number × implementation × exact firmware, carrying candidate routes and OEM quirks
(for Mt. Collins: the MegaRAC bootstrap credential kind and `image_redirection: 1`; for
ASRock/OpenBMC: the OEM WebSocket locator or configfs/UDC parameters). `BmcAccessObservation` is
per endpoint, per attempt. The solver selects a path only from their intersection.

`BmcFirmwareFamily` gained `AmiMegaRac` (`extdeps.bmc.endpoint`); the implementation module is
`extdeps.bmc.megarac` and the vendor row `extdeps.vendor.ami`. The family-keyed dispatchers moved
out of `extdeps.bmc.openbmc` into `gunbc.bmc_implementation_dispatch` so a MegaRAC change never
edits the OpenBMC module (DESIGN §3 external upstream decomposition). No MegaRAC firmware
version, capability roster, factory login or surface row is authored: each lands from an
observed unit.

## 3. The boot boundary

Boot control (set one-shot target × reset × observe override consumption × restore) and artifact
delivery (visible to firmware × bytes proven × intended host consumed it × detach) are
independent obligations. `BootArtifactDelivery` is the coproduct
`RedfishVirtualMediaPull | OpenBmcNbdProxyWebsocket | OpenBmcConfigfsUsbGadget | UefiHttpBoot |
PxeChainBoot`; `extdeps.bmc.virtual_media` gained the `RedfishVirtualMediaTransport` arm and
`extdeps.bmc.redfish_virtual_media` models the DMTF shape. Boot purpose is data
(`BootPurpose`); the artifact set is per architecture (`IntakeArtifactSet`), never a bi-arch
invariant.

Platform mapping:

| Platform observation | Candidate delivery |
|---|---|
| Mt. Collins MegaRAC with live-observed Redfish VirtualMedia | RedfishVirtualMediaPull |
| ASRock/OpenBMC with live-observed OEM NBD WebSocket | OpenBmcNbdProxyWebsocket |
| ASRock/OpenBMC with BMC SSH, NBD client, configfs gadget, usable UDC | OpenBmcConfigfsUsbGadget |
| Firmware with observed UEFI HTTP support and reachable artifact service | UefiHttpBoot |
| Firmware with all network-boot requirements established | PxeChainBoot |

`gunbc.os_install_mechanism` is now a frozen projection of the delivery solver; its consumers
migrate to `BootDeliverySolution` and it is then deleted (its frozen note carries the trigger).

## 4. The workflow: one `MachineIntake`, three transactions

**A — arrival hardware qualification** (immediately after delivery or repair, before shelving):

| Phase | Required result |
|---|---|
| AccessDiscover | BMC implementation, exact firmware, supported protocol surfaces, BMC clock observed |
| IdentityBindProvisional | Board/chassis/BMC identities collected without contradiction |
| PriorLifeBoundary | Existing logs archived, then cleared or a precise immutable baseline cursor established |
| BmcSecure | Unique managed credential works; published factory credential no longer works |
| BootDeliveryEstablish | One eligible delivery transport selected from live observations |
| DiagnosticBootAttest | Intended intake artifact calls back with the attempt nonce and repeats machine identity |
| InventoryConform | BMC, SMBIOS/OS, and platform slot-table observations reconcile |
| MemoryFastQualify | Parseable pre-OS memory-test result |
| MemoryAuthoritativeQualify | Linux stress plus ECC/RAS before-and-after delta |
| SubsystemQualify | CPU, storage, network, accelerator, PSU, cooling obligations for the expected component set |
| AcceptanceSoak | Return-window heat/load test succeeds |
| ArrivalCleanup | Media detached, override cleared, temporary access revoked, desired power state restored |

A machine that completes this is HardwareQualified. It still cannot serve.

**B — provisioning:** OsArtifactDelivered → OsInstalled → InstalledSystemBootedFromLocalTarget
→ RuntimeIdentityMatchesIntakeSubject. OsInstalled needs a post-install producer (the installed
agent calling back after a normal local boot); a consumed override, a read ISO, or a DHCP lease
is not sufficient.

**C — placement commissioning** (in the serving location): PlacementIdentityBound →
ActualCoolingAndFanPolicyObserved → ActualNetworkPathsQualified → PlacementLoadSoakQualified →
WallPowerMeasured → ServiceRuntimeAdmitted → FleetAdmitted. Moving the machine, changing its PSU
feed, cooling, or fan policy invalidates this receipt without invalidating memory qualification.

`gunbc.bmc_onboarding` combines factory credentials, rotation, OsInstalled and FabricJoined in
one vocabulary; extracting those into the three transactions is a follow-up, not a second
lifecycle.

## 5. Identity and prior-life handling

Identity is provisional until the Linux intake environment repeats collection from the host
side and BMC identity agrees with host-visible SMBIOS/SPD/PCI identity agrees with platform
expectations. Disagreement is a named finding: BoardSerialDisagrees, ChassisPartNumberDisagrees,
BmcAndHostMemoryPopulationDisagree, DuplicateDimmSerial, ExpectedComponentAbsent,
UnexpectedComponentPresent (InventoryConform, unbuilt).

Prior-life boundary: before clearing anything, preserve BMC clock, SEL/event logs, Redfish log
collections, audit/account events, current boot override, virtual-media state, power state, and
a sensor snapshot as an immutable content-addressed capture. Then
`PriorLifeBoundaryEstablished = LogsArchivedAndCleared | LogsArchivedWithBaselineCursor`; the
second arm only where platform policy explicitly admits an uncleared append-only boundary. The
receipt carries the pre-clear archive digest and the post-clear observation.

## 6. Credential qualification

Factory credentials are a bootstrap mechanism, not an ordinary credential state. BmcSecure:
probe published bootstrap credential → authenticated session → create/rotate unique per-unit
credential → verify new works → verify published fails → record secret reference and credential
epoch. The receipt never contains the password. A non-working factory credential is an access
state (CredentialStateUnknown, PreviouslyRotatedCredentialRequired,
AccountManagementUnavailable) → PartsHold, not ReturnWindow, absent other evidence.

## 7. Proving the diagnostic environment booted

Each attempt mints a nonce; the intake agent returns `IntakeAgentBootReceipt { attempt_nonce,
artifact_digest, architecture, board_serial, assembly_observation, firmware_observation,
booted_at }`. That — and only that — constructs DiagnosticBootAttested
(`gunbc.machine_intake_receipt.attest_diagnostic_boot`). NFS plateaus, redirection sessions,
power signatures, reset completion, SOL streams and KVM pixels are supporting observations that
ride in the refusal. KVM is optional forensic capture.

## 8. Memory qualification (MEMORY-0, unbuilt)

Fast gate: `FastMemoryQualified { artifact, report_digest, pass_count, tested_bytes, error_count }
| FastMemoryRefused`; a delivery session plus power activity without a report is
`MemoryExerciseObserved`. Authoritative gate: before snapshot, test plan, after snapshot
(SMBIOS/DMI, SPD, Redfish Memory, EDAC topology, EDAC CE/UE before/after, rasdaemon, MCE/APEI/
GHES, stressapptest, stress-ng); deltas during the interval, not lifetime counters; thresholds in
policy rows (new UE → defect; new CE → refuses, swap diagnosis; zero → gate may hold).
Attribution is a separate standing: `DimmAttributed | RankAttributed | ChannelAttributed |
ControllerAttributed | Unattributed` — a channel-level error fails the gate but cannot construct
DimmDefective. Swap-pass diagnosis binds every stick label to SPD first; every physical swap
changes the assembly digest and starts a new subject.

## 9. Beyond memtest (SOAK-ADMISSION-0, unbuilt)

`ExpectedSubsystemSet` derives from platform and assembly inventory; an expected component cannot
silently disappear from testing, and a platform with no accelerator derives no accelerator
obligation rather than a Skipped row. Per-subject observations: CPU topology/load/throttling/RAS
deltas; memory population/stress/CE-UE/temperature; storage identity/SMART/self-test (destructive
only on declared-disposable storage); NIC MAC/link/errors/throughput to staging; accelerator
PCIe/device memory/compute/ECC-Xid-AER/thermal; fans tach presence and response; PSU inventory,
telemetry, external wall measurement where required; BMC sensor completeness, clock, log service,
account state.

## 10. Load soak and power (SOAK-ADMISSION-0, unbuilt)

Two policy rows: ArrivalAcceptanceSoak (return-window defects) and PlacementCommissioningSoak
(environment-specific). Both record load recipe digest, duration, ambient, sample interval,
temperature maxima, fan extrema, throttling, RAS deltas, power samples, cooldown.
`PowerMeasurementPoint = WallAcInput | PsuAcInput | PsuDcOutput | BoardRail | BmcUnspecified`.
Only a WallAcInput observation (or a justified calibrated conversion from a precisely identified
point) may retire the economics model's 350 W wall-power assumption; without one, emit
WallPowerUnestablished. `PowerTelemetryAgreement { wall_ac, bmc_reported, difference }`.

## 11. Disposition

`MachineIntakeDisposition = Admitted | ReturnWindow | PartsHold` with
`QualificationRefusalOwner = UnitHardware | BmcAccess | StagingInfrastructure | PlatformModel |
QualificationPolicy | OperatorAction`. Mapping (`gunbc.machine_intake_disposition`): observed
hardware defect ∧ window open → ReturnWindow; defect ∧ window closed → PartsHold{RepairOrCull};
qualification unestablished → PartsHold{cause, owner}; placement pending →
PartsHold{PlacementPending}; all gates ∧ subject current ∧ cleanup confirmed → Admitted.
`FleetAdmissible` is derived from receipts; a hand-authored Admitted row cannot substitute.

## 12. Receipt architecture

Every phase emits one normalized envelope (`IntakeReceiptEnvelope`) plus evidence refs by
digest. Wire schema `gunbc.machine-intake.receipt/v1`; raw archive
`/srv/bmc/intake/artifacts/<sha256>/`, `attempts/<id>/manifest.json`, `phase/*.json`,
`evidence/<blobs>`. The JSON is an observation transport, not a second decision authority.
Required properties: canonical digest; attempt identity and exact subject; tool/agent/artifact/
policy identities; start/end; evidence by digest; success or typed refusal; hash-chain link;
staging identity; no credentials or one-time tokens.

## 13. The `intake <bmc-ip>` executable (unbuilt)

One operator command: acquire endpoint lock → discover access surface → bind provisional identity
→ acquire unit/attempt lock → ask the modeled solver for the next phase plan → execute through the
selected adapter (redfish-standard, ipmi-fru, ami-megarac, openbmc-nbdproxy, openbmc-configfs,
uefi-http, pxe) → append receipt → resume until terminal disposition. Phase order and disposition
live in `.dag`, not in `if chassis == ...` branches. Needs idempotent resume, per-endpoint then
per-unit locks, one-shot callback tokens, content-addressed artifacts, cleanup/compensation after
every failed delivery, no secrets in logs, a dry plan mode, a typed terminal receipt on staging
failure. It emits observations; it does not author `.dag` rows. Dissolution: delete it when the
modeled fleet-ops executor consumes the same plan, performs every effect, and produces
byte/schema-equivalent receipt envelopes for both the Mt. Collins and ASRock transport fixtures.

## 14. Integration map

| Existing authority | Change |
|---|---|
| `extdeps.bmc.endpoint` | `AmiMegaRac` arm; `FactoryLoginStanding` (landed) |
| `extdeps.bmc.megarac`, `extdeps.vendor.ami` | new implementation and vendor modules (landed, no per-release rows yet) |
| `extdeps.bmc.access_profile` | profile/observation carriers and intersection (landed) |
| `extdeps.bmc.redfish_virtual_media`, `extdeps.bmc.virtual_media` | DMTF shape; `RedfishVirtualMediaTransport` arm (landed) |
| `gunbc.boot_artifact_delivery` | the one delivery solver (landed) |
| `gunbc.os_install_mechanism` | frozen projection of the solver (landed) |
| `gunbc.bmc_implementation_dispatch` | family dispatch moved out of `extdeps.bmc.openbmc` (landed) |
| `gunbc.machine_intake_*` | subject, phase, receipt, disposition, staging (landed) |
| `gunbc.bmc_onboarding` | extract BMC access/security, provisioning, fabric joining (follow-up) |
| `gunbc.nbd_proxy_virtual_media_install` | realize `ActuatorRedfishInsertMedia` through the Redfish arm (follow-up) |
| `extdeps.bmc.capability` | inventory, log, telemetry, cleanup capabilities (follow-up) |
| private `strategy.*` | per-unit normalized receipts and return-window facts (follow-up) |

srv1 vs srv2 as staging host is resolved by observation (`intake_staging_realization_today`
is `StagingUnobserved`), not by prose.

## 15. Implementation sequence

- **INTAKE-0 (landed):** subject, manifests, attempt, phase standing, refusal owner,
  disposition, fleet-admission derivation, receipt envelope; controls in
  `test.claim.machine_intake_subject_witness_test` and
  `test.claim.machine_intake_disposition_witness_test`.
- **BOOT-DELIVERY-0 (modeling landed; execution owed):** MegaRAC identity, Redfish VirtualMedia
  transport, solver consuming configfs/WebSocket, UEFI HTTP split from PXE; controls in
  `test.claim.boot_artifact_delivery_witness_test`. Acceptance: one executed Mt. Collins boot and
  one executed ASRock boot reaching the same nonce-bound intake-agent callback.
- **INTAKE-AGENT-0:** minimal multi-arch artifact set (identity, inventory, receipt callback,
  EDAC/RAS snapshot, sensors, cleanup coordination).
- **MEMORY-0:** fast report ingestion, authoritative plan, EDAC/Redfish deltas, attribution,
  label/SPD binding, swap-pass diagnosis.
- **SOAK-ADMISSION-0:** subsystem recipes, thermal/fan policy, wall-power producer, two soak
  profiles, final fleet-admission consumer.

The smallest useful MVP reaches the Linux agent, inventory conformance, and authoritative memory
evidence; automating media attachment while the verdict stays manual is not it.

## 16. Discriminating controls

| Perturbation | Required result | Where |
|---|---|---|
| Change one DIMM while retaining the board serial | Old admission no longer applies | subject + disposition witnesses |
| Leave factory credential working after rotation | BmcSecure refuses | BmcSecure (unbuilt) |
| Remove Redfish VirtualMedia from live observation, keep it in the profile | Redfish transport not selected | boot delivery witness |
| Remove virtual media and provide no network-boot evidence | No implicit PXE | boot delivery witness |
| Preserve NFS/media activity, remove the report/callback | Boot verdict refuses | disposition witness |
| Make EDAC unavailable | Memory standing unestablished, not zero-error | MEMORY-0 |
| Channel-attributed error only | Gate fails; no DIMM convicted | MEMORY-0 |
| BMC watts without wall meter | Wall power unestablished | SOAK-ADMISSION-0 |
| Fail the staging artifact service | PartsHold{StagingInfrastructure}, never ReturnWindow | disposition witness |
| Leave media attached or override active | Cleanup refuses; admission impossible | ArrivalCleanup (unbuilt) |
| Re-run from a stale attempt after a repair | Receipt subject mismatch refuses | disposition witness |
| Manually write Admitted without receipts | Derivation refuses | disposition witness |
