# Machine intake: hardware qualification, provisioning, placement commissioning

Operator ruling of 2026-08-29, transcribed as the authority for the `gunbc.machine_intake_*`
lane. The `.dag` modules cite this document; this document does not restate what they declare.
Where a section below names a type, the type is the authority and this is its rationale.

Status: **INTAKE-0 and BOOT-DELIVERY-0 modeling in review** (gunbc#9690), reworked under the
operator's second ruling of 2026-08-29, the third ruling of 2026-08-30
([machine-intake-ruling-3.md](machine-intake-ruling-3.md)) and the fourth ruling of 2026-08-30
([machine-intake-ruling-4.md](machine-intake-ruling-4.md), whose ten-item final approval gate is
the merge gate: neither slice is recorded complete until they hold). Under ruling 4 the request's
`BootDeliveryTarget` (subject × protocol-neutral `BmcControllerEndpoint`) is BOUND to the access
context before any candidate is judged (`bind_boot_delivery_target`; subject and controller
mismatches are separate refusals), every transport-local standing (configfs via
`TargetBoundReinstallPath`, UEFI HTTP, network boot) carries the target it was established for
and is refused as `CandidateEvidenceForOtherTarget` otherwise; the profile's capability row is
the exact-release `BmcFirmwareReleaseCapabilityRow` so raw "0.32" inhabits a valid profile (a
positive control proves it); a promoted profile binds only beside `ProfilePromotionVerified`,
which no producer can construct until INTAKE-AGENT-0A's evidence store — so every promoted
profile refuses today; the Redfish probe receipt has one protocol authority (nonempty
`admitted_protocols` is the plan floor; the ladder arm is payload-free) and the observation
receipt carries an evidence MANIFEST that must name both the discovery bytes and the nested
probe's bytes; the plan's provenance preserves the selected candidate's own evidence. The
changed-witness execution sublane #9717 requires is dispatched as its own CI PR (child work
item of this lane). Under ruling 3 the solver reads a
`BootDeliveryRequest` (target unit and attempt, exact `BootArtifact`, staging offers each naming
the digest they serve), a `BoundBootDeliveryEvidence` whose access context
`gunbc.machine_intake_access` has already bound to this attempt (profile ∩ observation, capability
row read off the profile, observation receipt digest checked), and a `BootDeliveryPolicy`; it
answers with a `BootDeliveryPlan` that repeats target and artifact, derives the Redfish locator and
protocol from the probe RECEIPT, requires the MegaRAC route's `image_redirection` fact, and names
the profile provenance and observation receipt its eligibility came from. The capability-only
legacy solver, its `FirmwareUpdateThenVirtualMedia` shortcut, `srv3_install_mechanism`,
`fleet_install_server_specs` and the ProxyDHCP artifact path are deleted; `gunbc.os_install_mechanism`
is a compatibility projection that refuses at `InstallAccessProfileNotLookedUp`. Firmware release
identity is the raw vendor string (`BmcFirmwareReleaseIdentity`, so "0.32" is representable);
`FirmwareTransition*` lives in `gunbc.bmc_firmware_transition` and `NetworkBoot*` in
`gunbc.network_boot_delivery`. Under ruling 2: the Mt. Collins family binding is an EXPECTATION
(`MtCollinsBmcImplementationExpected`) until a workflow-layer `BmcImplementationObserved`
receipt exists; the one solver reads `BmcAccessProfile × BmcAccessObservation` through a total
profile lookup (`BmcAccessProfileStanding`), plans the Redfish arm only at
`TransferProtocolAdmitted` or above on the probe ladder, and takes its transport preference as an
explicit policy input; `derive_fleet_admission` consumes only a `ValidatedIntakeReceiptLedger`
(one genesis, linked, one subject and attempt, monotone); the in-substrate receipt hash is named
a structural fingerprint, with SHA-256 reserved for the durable producer; and
`gunbc.bmc_onboarding` is quarantined as non-authoritative legacy vocabulary. Executed
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
`ChassisApplicability` (an explicit arm, never absence-means-all) × implementation × exact raw
firmware release (`BmcFirmwareReleaseIdentity`), carrying its routes as inhabitants
(`BmcAccessRoute`: Redfish locators, IPMI lanplus, the MegaRAC REST route with its typed
`image_redirection` fact, the OpenBMC shell, the OpenBMC NBD WebSocket path) so a no-Redfish
build is representable without inventing locators, plus the capability row that release carries
(a catalog row whose row firmware disagrees with its key is refused at lookup). Provenance is
`ProfileFromExternalAuthority | ProfilePromotedFromObservation`, never the rendering declaration.
`BmcAccessObservation` is per endpoint with its raw response digest and a resource-bound
`RedfishVirtualMediaProbeReceipt?`; the lookup answers `Known | Uncatalogued | Ambiguous |
CatalogRowRefused` (external facts only). The controller is named protocol-neutrally
(`BmcControllerEndpoint { host }`, ruling 4 §3): one Mt. Collins boot drives one controller over
MegaRAC REST for delivery and IPMI for boot control, so an identity carrying a protocol has
already selected a route. The capability row a profile carries is the exact-release
`BmcFirmwareReleaseCapabilityRow` (ruling 4 §2), so a release with no semantic reading can be
catalogued. Which attempt the observation belongs to, whether the lookup ran, whether the
evidence manifest names every blob read, and whether a promoted profile's receipts were
verified are WORKFLOW standings: `gunbc.machine_intake_access.bind_bmc_access_context` produces
the one `BoundBmcAccessContext` the solver may read, and `ProfilePromotionVerificationStanding`
refuses every promoted profile until the INTAKE-AGENT-0A evidence store produces
`ProfilePromotionVerified` (ruling 4 §4).

`BmcFirmwareFamily` gained `AmiMegaRac` (`extdeps.bmc.endpoint`, gunbc#9678); the implementation
module is `extdeps.bmc.megarac` and the vendor row `extdeps.vendor.ami`. The family-keyed
dispatchers moved out of `extdeps.bmc.openbmc` into `gunbc.bmc_implementation_dispatch` so a
MegaRAC change never edits the OpenBMC module (DESIGN §3 external upstream decomposition).
Family-grain MegaRAC facts are cited (admin/admin published default, IPMI-only protocol floor);
build-scoped facts live on the build binding `extdeps.ampere.mt_collins_product_brief.bmc` —
the observed Foxconn Mt. Collins build (BMC 0.32) serves NO Redfish and boots through the
proprietary REST remote-media surface, so the Mt. Collins delivery arm is
`MegaRacRestRemoteMedia`, not the DMTF pull. A profile/capability row per exact firmware is
still to be authored from that observation.

## 3. The boot boundary

Boot control (set one-shot target × reset × observe override consumption × restore) and artifact
delivery (visible to firmware × bytes proven × intended host consumed it × detach) are
independent obligations. `BootArtifactDelivery` is the coproduct
`RedfishVirtualMediaPull | MegaRacRestRemoteMedia | OpenBmcNbdProxyWebsocket |
OpenBmcConfigfsUsbGadget | UefiHttpBoot | PxeChainBoot`; `extdeps.bmc.virtual_media` gained the
`RedfishVirtualMediaTransport` and `MegaRacRemoteMediaTransport` arms and
`extdeps.bmc.redfish_virtual_media` models the DMTF shape. Boot purpose is data
(`BootPurpose`); the artifact set is per architecture (`IntakeArtifactSet`), never a bi-arch
invariant.

Platform mapping:

| Platform observation | Candidate delivery |
|---|---|
| MegaRAC build with live-observed Redfish VirtualMedia | RedfishVirtualMediaPull |
| Mt. Collins MegaRAC (observed build 0.32: no Redfish) with live-observed REST remote media | MegaRacRestRemoteMedia |
| ASRock/OpenBMC with live-observed OEM NBD WebSocket | OpenBmcNbdProxyWebsocket |
| ASRock/OpenBMC with BMC SSH, NBD client, configfs gadget, usable UDC | OpenBmcConfigfsUsbGadget |
| Firmware with observed UEFI HTTP support and reachable artifact service | UefiHttpBoot |
| Firmware with all network-boot requirements established | PxeChainBoot |

`gunbc.os_install_mechanism` is a compatibility projection of the delivery solver with no solver
of its own; its consumers migrate to `BootDeliverySolution` and it is then deleted (its frozen note
carries the trigger). A `BootDeliveryPlanned` result is a candidate PLAN bound to target and
artifact — insert, host consumption, eject and detach are established only by execution receipts.
The boot-control target is protocol-neutral (`BootControlTarget`); which BMC operation sets it is
realization bound to the route the profile exposes. The request's `BootDeliveryTarget` is bound
to the access context first (`BoundBootDeliveryTargetContext`, ruling 4 §1) and the transport-local
standings each carry their own target, so a plan can never name a subject or controller other
than the one its evidence was gathered for; the plan's `BootDeliveryEligibilityProvenance` names
the profile provenance, the evidence manifest, the selected candidate's own evidence
(`SelectedCandidateEvidence`) and every candidate's standing.

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
| `extdeps.bmc.endpoint` | `AmiMegaRac` arm (landed via gunbc#9678 and this PR) |
| `extdeps.bmc.megarac`, `extdeps.vendor.ami` | implementation module (gunbc#9678: cited admin/admin default, IPMI floor, proprietary REST remote-media operations, executed Mt. Collins boot recipe) and vendor row |
| `extdeps.bmc.access_profile` | profile/observation carriers and intersection (landed) |
| `extdeps.bmc.redfish_virtual_media`, `extdeps.bmc.virtual_media` | DMTF shape; `RedfishVirtualMediaTransport` arm (landed) |
| `gunbc.boot_artifact_delivery` | the one delivery solver: request × bound evidence × policy → plan (landed, ruling 3 §1-§2) |
| `gunbc.machine_intake_access` | attempt-bound access context producer; `BootDeliveryTarget`, evidence manifest, promotion verification standing (landed, ruling 3 §1, ruling 4 §1/§4/§5) |
| `extdeps.bmc.endpoint`, `extdeps.bmc.capability` | `BmcControllerEndpoint`; `BmcFirmwareReleaseCapabilityRow` (landed, ruling 4 §2-§3) |
| `gunbc.install_transport_qualification` | `TargetBoundReinstallPath` — the host transport standing joined to an intake target (landed, ruling 4 §1) |
| CI changed-witness execution sublane | dispatched as its own PR (child work item; ruling 4 "#9717 is not an unrelated CI incident") |
| `gunbc.bmc_firmware_transition`, `gunbc.network_boot_delivery` | standings cut out of the solver into their own authorities (landed, ruling 3 namespace ruling) |
| `gunbc.os_install_mechanism` | compatibility projection; capability-only solver and `FirmwareUpdateThenVirtualMedia` deleted (landed, ruling 3 §3) |
| `gunbc.os_install`, `gunbc.generated_artifact` | `fleet_install_server_specs`, `InstallServerSpec`, `gunbc.install_server_emit` and the `ProxyDhcpDnsmasqArtifact` path deleted (ruling 3 §3) |
| `gunbc.bmc_implementation_dispatch` | family dispatch moved out of `extdeps.bmc.openbmc` (landed) |
| `gunbc.machine_intake_*` | subject, phase, receipt, disposition, staging (landed) |
| `gunbc.bmc_onboarding` | extract BMC access/security, provisioning, fabric joining (follow-up) |
| `gunbc.nbd_proxy_virtual_media_install` | realize `ActuatorRedfishInsertMedia` through the Redfish arm (follow-up) |
| `extdeps.bmc.capability` | inventory, log, telemetry, cleanup capabilities (follow-up) |
| private `strategy.*` | per-unit normalized receipts and return-window facts (follow-up) |

srv1 vs srv2 as staging host is resolved by observation (`intake_staging_realization_today`
is `StagingUnobserved`), not by prose.

## 15. Implementation sequence

- **INTAKE-0 (modeling in review; ruling 3 gate 5 — lifecycle-valid ledger, admission
  provenance — in this PR):** subject, manifests, attempt, phase standing, refusal owner,
  disposition, fleet-admission derivation, receipt envelope; controls in
  `test.claim.machine_intake_subject_witness_test` and
  `test.claim.machine_intake_disposition_witness_test`.
- **BOOT-DELIVERY-0 (modeling landed; execution owed):** MegaRAC identity, Redfish VirtualMedia
  transport, solver consuming configfs/WebSocket, UEFI HTTP split from PXE; controls in
  `test.claim.boot_artifact_delivery_witness_test`. Acceptance: one executed Mt. Collins boot and
  one executed ASRock boot reaching the same nonce-bound intake-agent callback.
- **INTAKE-AGENT-0A (next, per ruling 3):** deterministic x86-64 and AArch64 artifacts with
  recorded content identities; a callback carrying attempt nonce, exact artifact digest,
  architecture, raw board serial, assembly/firmware observation digests and boot time; replay and
  duplicate-callback refusal; wrong-nonce/artifact/unit/architecture REDs; canonical durable receipt
  bytes and their SHA identity; raw EDAC/RAS, inventory and sensor capture only; local execution of
  the callback contract without a BMC. Then ACCESS-OBSERVE-0 / BOOT-DELIVERY-0E on hardware:
  observation → profile lookup → plan → execute → agent callback → cleanup, on Mt. Collins and ASRock.
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
| Stage a URI serving digest B for a request naming digest A | `CandidateStagedArtifactMismatch`, no plan | boot delivery witness |
| Hand the solve an observation receipt from another attempt | `AccessContextObservationForOtherAttempt`, no plan | boot delivery witness |
| Receipt manifest omits the observation's raw response | `AccessContextObservationEvidenceMismatch` | boot delivery witness |
| Receipt manifest names the discovery bytes but not the nested probe's | `AccessContextProbeEvidenceMismatch` | boot delivery witness |
| Request target for attempt two, context bound for attempt one (same controller) | `DeliveryTargetSubjectMismatch`, no plan | boot delivery witness |
| Request target for controller .61, context bound for .60 (same subject) | `DeliveryTargetControllerMismatch`, no plan | boot delivery witness |
| Configfs path / network boot established for another target | `CandidateEvidenceForOtherTarget` | boot delivery witness |
| Profile promoted from an observation, receipts unresolvable | `AccessContextPromotedProfileUnverified`, no plan | boot delivery witness |
| Raw release "0.32" with an exact-release OEM row and matching observation | context binds, MegaRAC plan (positive) | boot delivery witness |
| Every current phase in the three rosters | exactly one rank, all distinct (positive) | disposition witness |
| Catalog row whose capability firmware differs from its key | lookup refuses the catalog | boot delivery witness |
| Probe admitted NFS only, staging offers HTTPS | `CandidateRedfishProtocolNotAdmitted` | boot delivery witness |
| MegaRAC route with `image_redirection` Unset | `CandidateOemParameterUnset`, no plan | boot delivery witness |
| Remove virtual media and provide no network-boot evidence | No implicit PXE | boot delivery witness |
| Preserve NFS/media activity, remove the report/callback | Boot verdict refuses | disposition witness |
| Make EDAC unavailable | Memory standing unestablished, not zero-error | MEMORY-0 |
| Channel-attributed error only | Gate fails; no DIMM convicted | MEMORY-0 |
| BMC watts without wall meter | Wall power unestablished | SOAK-ADMISSION-0 |
| Fail the staging artifact service | PartsHold{StagingInfrastructure}, never ReturnWindow | disposition witness |
| Leave media attached or override active | Cleanup refuses; admission impossible | ArrivalCleanup (unbuilt) |
| Re-run from a stale attempt after a repair | Receipt subject mismatch refuses | disposition witness |
| Manually write Admitted without receipts | Derivation refuses | disposition witness |
