# Machine-intake ruling 3 — review of #9690 @ d4b1e6a104 (2026-08-30)

Transcribed verbatim from the design side-chat (turn c6f87ffa). Citations stripped.

# Review verdict

**REQUEST CHANGES / HOLD at `d4b1e6a104ce8bf78e147642742c259e98994a59`.** The major repairs are real, but ruling 2 is not fully discharged. I would not yet record either **INTAKE-0 complete** or **BOOT-DELIVERY-0 modeling complete**.

The following parts now satisfy the ruling:

- `MtCollinsBmcImplementationExpected` no longer presents the family as an Ampere-authored platform fact.
- The solver returns `BootDeliveryPlanned`, not an execution establishment.
- `derive_fleet_admission` accepts only `ValidatedIntakeReceiptLedger`.
- `intake_staging_realization_today` remains `StagingUnobserved`.
- The `bmc_onboarding` split may remain a follow-up. fileciteturn35file0L2-L2 fileciteturn31file0L2-L2 fileciteturn46file0L2-L2 fileciteturn50file0L2-L2

Four ruling-level blockers remain.

## 1. The new solver is not bound to **this artifact on this host**

`BootArtifact` correctly names purpose, architecture, digest, format, entrypoint, and build identity. But `BootDeliveryEvidence` contains no `BootArtifact`, target unit, target host identity, or intake attempt, and `BootDeliveryPlanned` contains only a transport plus the considered verdicts. The selected Redfish URI, MegaRAC image name, NBD export, or network-boot evidence is therefore not proven to represent the `BootArtifact` defined earlier in the same module. The plan also drops the endpoint from which eligibility was derived. fileciteturn51file0L2-L2 fileciteturn31file0L2-L2 fileciteturn52file0L2-L2

This admits a straightforward false plan:

```text
requested artifact digest = A
Redfish image URI actually serves digest = B
profile + probe + params are otherwise eligible
result = BootDeliveryPlanned
```

Nothing in the solver compares A and B because A is not an input.

The solver needs a request and a result closer to:

```dag
type BootDeliveryRequest {
  target: BootDeliveryTarget
  artifact: BootArtifact
  access: BoundBmcAccessContext
  staging: BootArtifactStagingBinding
}

type BootDeliveryPlan {
  target: BootDeliveryTarget
  artifact: BootArtifact
  delivery: BootArtifactDelivery
  boot_control: BootControlPlan
  eligibility_provenance: BootDeliveryEligibilityProvenance
}
```

Every transport-specific staging binding must name or derive the exact artifact digest. The eventual callback then repeats the digest from the same plan.

### The access evidence can also be spliced across identities

`BootDeliveryEvidence` independently accepts:

```text
BmcFirmwareCapabilityRow
BmcAccessProfileStanding
BmcAccessObservationStanding
```

The capability test reads `ev.row`; surface eligibility reads only the profile and observation. There is no check that:

```text
row.firmware
= profile.key.firmware
= observation.identity.firmware
= observation.firmware
```

A capability row for firmware A can therefore contribute `CapabilityVirtualMedia` while a profile and observation for firmware B establish the surface. `BmcFirmwareCapabilityRow` itself carries only firmware and a capability list—not board identity or source provenance. fileciteturn31file0L2-L2 fileciteturn47file0L2-L2

`BmcAccessObservation` also carries the firmware twice—inside `identity.firmware` and again as `firmware`—with no constructor proving they agree. It is described as per-attempt, but carries neither an attempt ID nor a qualification subject, and it contains no raw-response evidence digest. An observation from a previous attempt can therefore be handed to the current solve as long as its interpreted identity still fits. fileciteturn30file0L2-L2

Required shape:

```dag
type BoundBmcAccessContext {
  subject: MachineIntakeSubject
  endpoint: BmcEndpoint
  identity: BmcObservedIdentity
  capability_row: BmcFirmwareCapabilityRow
  profile: BmcAccessProfile
  profile_provenance: BmcAccessProfileProvenance
  observation_receipt: EvidenceRef
}
```

Its producer must prove all identity joins before the delivery solver can run. Better still, place the exact capability row inside `BmcAccessProfileCatalogRow`, with a constructor proving that its firmware equals the profile key. Then remove the independently supplied `row` from `BootDeliveryEvidence`.

Add RED controls for:

- capability-row firmware different from the profile;
- `observation.firmware != observation.identity.firmware`, if the duplicate field remains;
- a live observation from another attempt;
- a profile whose declaration exists but whose underlying observation receipt is absent.

### The real Mt. Collins firmware cannot currently inhabit the key

The retained historical report calls the Mt. Collins firmware `0.32`, while `bmc_firmware_version_from_wire` accepts exactly three numeric components and every profile/observation key requires `BmcFirmwareVersion`. A live wire value of `0.32` therefore either refuses or must be fabricated as `0.32.0`. fileciteturn35file0L2-L2 fileciteturn47file0L2-L2 fileciteturn30file0L2-L2

Preserve an opaque exact vendor release identity:

```dag
type BmcFirmwareReleaseIdentity {
  family: BmcFirmwareFamily
  raw_version: NonEmptyStr
  semantic_version: FirmwareSemanticVersion?
}
```

Exact profile lookup should use the raw vendor identity. Semantic parsing is an optional derived view, never a prerequisite for observing the firmware.

## 2. A planned transport can disagree with the route that was observed

### Redfish protocol and resource are unbound

`TransferProtocolAdmitted` carries the admitted protocol, but `redfish_virtual_media_candidate` discards that payload and merely checks that separate `RedfishVirtualMediaParams` are present. Those params independently carry a media locator and protocol. Thus:

```text
probe admitted NFS
params request HTTPS
```

still produces `BootDeliveryPlanned`. The same is true of the media member: the probe carries no locator, while the params may name any member. `InsertSucceeded` and `EjectSucceededAndDetached` are also accepted as plan-floor evidence even though those arms have lost both locator and protocol. fileciteturn32file0L2-L2 fileciteturn31file0L2-L2 fileciteturn44file0L2-L2

The probe must be resource-bound:

```dag
type RedfishVirtualMediaProbeReceipt {
  locator: RedfishVirtualMediaLocator
  progress: RedfishVirtualMediaProbe
  admitted_protocols: List<RedfishVirtualMediaTransferProtocol>
  raw_response: EvidenceRef
}
```

The solver should derive the locator and protocol from that receipt. It should accept only the artifact URI from staging, rather than accepting complete final params that can contradict the observation.

### The MegaRAC constructor omits the parameter its own authority calls required

`extdeps.bmc.megarac` says the start-media request **must** carry `image_redirection: 1`. The design document likewise says that OEM quirk belongs in the access profile. Yet `MegaRacRemoteMediaParams` has no such field, the solver never examines `profile.oem_parameters`, and the positive Mt. Collins control uses `oem_parameters: []` while still selecting `MegaRacRestRemoteMedia`. fileciteturn36file0L2-L2 fileciteturn37file0L2-L2 fileciteturn44file0L2-L2 fileciteturn34file0L2-L2

That is a direct false-green against the observed operation contract. Encode the requirement structurally, preferably as a typed field rather than a generic name/value pair:

```dag
type MegaRacRemoteMediaBinding {
  ...
  image_redirection: MegaRacImageRedirectionEnabled
}
```

The positive test must fail when that binding is absent.

### The access-profile shape cannot honestly represent the no-Redfish platform

Every `BmcAccessProfile` is required to provide three nonempty Redfish locators, even for the Mt. Collins build whose retained report says Redfish is absent. The synthetic control therefore invents `/redfish/...` locators for an OEM-only profile. Meanwhile `BmcEndpoint.protocol` has only `Redfish | Ipmi`, and the no-Redfish MegaRAC OEM-REST fixture is labeled `protocol: Redfish`. fileciteturn30file0L2-L2 fileciteturn33file0L2-L2 fileciteturn34file0L2-L2

Replace mandatory Redfish fields with route inhabitants:

```dag
type BmcAccessRoute
  = RedfishRoute { ... }
  | IpmiLanplusRoute { ... }
  | MegaRacRestRoute { ... }
  | OpenBmcShellRoute { ... }
  | OpenBmcNbdWebsocketRoute { ... }
```

The BMC host identity should be separate from the routes it exposes; one controller may expose IPMI and OEM REST simultaneously. The boot-control target should also be protocol-neutral. Presently every delivery, including the IPMI-controlled no-Redfish MegaRAC path, projects to `RedfishBootSourceOverrideTarget`. fileciteturn31file0L2-L2

Also replace `chassis_part_number: None means match every chassis` with an explicit applicability:

```dag
type ChassisApplicability
  = ExactChassisPartNumber { part_number: NonEmptyStr }
  | AllChassisVariants
```

Absence must not silently mean universality.

## 3. There are still two delivery solvers

The frozen projection is not the offending path. The offending path is the still-live original solver:

```dag
fn solve_install_mechanism(...)
```

It selects virtual media from a capability row alone and defaults to PXE in its final `else`. `srv3_install_mechanism` still calls it, and `gunbc.os_install.fleet_install_server_specs` still consumes that result. The generated-artifact registry then consumes the install-server roster for `ProxyDhcpDnsmasqArtifact`. This remains an independent live modeled answer beside `solve_boot_artifact_delivery`. fileciteturn28file0L2-L2 fileciteturn29file0L2-L2 fileciteturn23file0L2-L2 fileciteturn25file0L2-L2

That chain must be deleted or migrated now:

```text
solve_install_mechanism
srv3_install_mechanism
fleet_install_server_specs_for
fleet_install_server_specs
the stale ProxyDHCP artifact path, if its consumer census remains empty
```

Do not retain a second solver merely because the generated registry cannot express a refusal. Given that the proxy-DHCP artifact is currently `NotCommitted`, deleting the dead consumer is preferable to growing an additional standing solely to preserve it. fileciteturn25file0L2-L2

There is a second bypass inside the purported projection: `firmware_transition_precursor` returns `FirmwareUpdateThenVirtualMedia` when a transition is established and **some** catalog row supports VM. It ignores the exact `to` version and performs no post-transition profile lookup or live observation. A transition to an unrelated, non-VM release can therefore select the combined mechanism. fileciteturn29file0L2-L2

A firmware transition is not a delivery. The result should be:

```text
FirmwareTransitionRequired / FirmwareTransitionPlanned
```

After executing it, the workflow must re-observe the endpoint and call the one delivery solver again. It may not preselect “then virtual media.”

### Ruling on `AccessProfileNotLookedUp`

**The refusing projection is honest and should not be deleted blindly.** `srv3_os_install_actuate_scope` and `gunbc.nbd_proxy_virtual_media_install` still consume its refusal-bearing solution and currently fail closed because of it. fileciteturn26file0L2-L2 fileciteturn27file0L2-L2

Keep that compatibility adapter until those consumers read `BootDeliverySolution` directly. Delete the capability-only total solver now. Also rename:

```text
InstallVirtualMediaLiveSurfaceUnestablished
```

because `AccessProfileNotLookedUp` is not a live-surface observation failure. `InstallBootDeliveryEvidenceUnestablished` or a specific `InstallAccessProfileNotLookedUp` arm would preserve the actual cause.

## 4. The receipt ledger is link-valid, but not yet lifecycle-valid

The raw-list admission defect is fixed: admission now receives only `ValidatedIntakeReceiptLedger`, and the validator rejects empty, disconnected, differently linked, mixed-subject/attempt, and decreasing-start-time ledgers. The FNV value is also honestly named a structural fingerprint. Those are substantial corrections. fileciteturn38file0L2-L2 fileciteturn39file0L2-L2 fileciteturn46file0L2-L2

But the prior ruling also required valid phase ordering or explicit retry semantics. The validator does not read the declared phase order at all. It checks only subject, predecessor, and nondecreasing `started_at`; admission later searches the entire ledger independently for each required phase. fileciteturn38file0L2-L2 fileciteturn39file0L2-L2 fileciteturn40file0L2-L2 fileciteturn46file0L2-L2

A counterexample that currently admits:

1. Put every required receipt in reverse phase order.
2. Give them increasing timestamps.
3. Recompute every predecessor fingerprint.
4. Validate the ledger.
5. Admission finds one success for every required phase and returns `FleetAdmissible`.

A subtler counterexample is a complete successful lifecycle followed by a new successful `AccessDiscover`. Last-wins makes the new access receipt current, while all downstream qualifications still predate that new discovery.

Add either a declared prerequisite relation or a monotone phase rank. For a serial first implementation, the simplest rule is:

- phase rank may remain the same for an immediate retry;
- phase rank may advance;
- phase rank may never regress after a later phase has been recorded;
- retrying an earlier phase requires a new attempt or explicitly invalidates all descendants.

The validator also needs:

- `started_at <= ended_at`;
- a declared time-order rule across receipts;
- policy-digest compatibility across the attempt;
- producer-version compatibility or a typed producer-transition receipt;
- a RED for each violation.

### The generic ledger still cannot validate the promised SHA chain

The comment says the eventual durable producer will emit SHA-256, but validation always calculates the expected predecessor with `intake_receipt_structural_fingerprint`. Cross-family comparison refuses. Consequently, a genuine SHA-linked ledger cannot pass this validator. The receipt note also still names the nonexistent `intake_receipt_digest` and calls the ordering immutable. fileciteturn38file0L2-L2

Either:

- explicitly name this `StructurallyValidatedIntakeFixtureLedger` and do not claim it is the durable admission ledger; or
- introduce `IntakeReceiptRecord { envelope, identity }`, have the producer establish the canonical SHA identity, and validate links against the supplied current-record identities.

Finally, `FleetAdmissionReceipt` drops the ledger’s terminal identity and the exact receipt identity for each satisfied phase. It carries only subject, phase names, and derivation time. The derivation reads the producer ledger, but its output does not preserve which producer ledger it read. Add the terminal ledger identity, policy identity, and phase-to-receipt bindings. fileciteturn46file0L2-L2

# Layer-placement rulings

## `BootDeliveryPreference`

**Correctly belongs in `gunbc`, not `extdeps`.** It is product/workflow policy, not a vendor or protocol fact. Keeping it in `gunbc.boot_artifact_delivery` is acceptable while there is only one local policy.

However, it should not be a field of a type named `BootDeliveryEvidence`. Separate fact from policy:

```dag
fn solve_boot_artifact_delivery(
  request: BootDeliveryRequest,
  evidence: BoundBootDeliveryEvidence,
  policy: BootDeliveryPolicy,
) -> BootDeliverySolution
```

Move it to `gunbc.boot_delivery_policy` only when a second workflow or policy row needs independent governance.

## `BmcAccessProfileStanding`

The current type fuses two layers. Split it:

**Keep in `extdeps.bmc.access_profile`:**

```text
BmcAccessProfile
BmcAccessProfileKey
BmcAccessProfileCatalogRow
BmcAccessProfileLookup =
  Known | Uncatalogued | Ambiguous
```

**Move to a workflow module such as `gunbc.machine_intake_access`:**

```text
ProfileLookupNotRun
AccessUnobserved
AccessObservationCaptured
ProfileBoundToObservation
ProfileObservationIdentityMismatch
```

`AccessProfileNotLookedUp` is workflow progress, not an external fact. `AccessProfileIdentityMismatch` is presently not produced by `bmc_access_profile_for` at all; the lookup produces only known, uncatalogued, or ambiguous, while a different function produces the profile-observation mismatch. That arm should either acquire a real producer or disappear from the lookup standing. fileciteturn30file0L2-L2

Also, `DeclarationRef` is not sufficient profile provenance. It identifies the catalog declaration—the renderer—not the observation or cited producer that made the profile admissible. Preserve both:

```dag
type BmcAccessProfileProvenance
  = ProfileFromExternalAuthority { authority: ExternalAuthority }
  | ProfilePromotedFromObservation {
      observation_receipt: ContentHash
      promotion_receipt: ContentHash
    }
```

# Namespace ruling

The 31 rows use the correct **mechanical disposition** for the movement that actually occurred: the referenced declarations changed target, so `TargetChanged` is the right classification. fileciteturn43file0L2-L2

The target cut is not final, however.

### `FirmwareTransition*`: move to its own module now

This one is mandatory. `boot_artifact_delivery` itself says a firmware transition is **not** a delivery, and the new delivery solver does not consume `FirmwareTransitionStanding`; only the frozen legacy projection does. It belongs in something like:

```text
gunbc.bmc_firmware_transition
```

That module should own the six requirements, the establishment producer, exact target-row binding, and execution receipt. fileciteturn31file0L2-L2 fileciteturn29file0L2-L2

### `NetworkBoot*`: separate module is the cleaner final cut

I would also move the network-boot establishment vocabulary to:

```text
gunbc.network_boot_delivery
```

Its four evidence axes have independent producers—client firmware mode, serving infrastructure, boot artifacts, and boot control—and form an independently useful standing consumed by the compositor. `gunbc.boot_artifact_delivery` should import that standing and select the `PxeChainBoot` constructor.

This is less categorical than the firmware-transition move: keeping it beside its sole constructor would not violate ruling 2. But the separate module is the better final authority boundary and avoids turning the 605-line solver into the home of every candidate’s evidence lifecycle.

**Do not preserve the current 31 rows and then move the subjects again.** Make the final module cut, rerun the required namespace instrument, and transcribe the resulting final-target admissions.

# `bmc_onboarding` quarantine

Deferring the split remains approved. The header quarantine says the correct things. fileciteturn42file0L2-L2

The new witness is not discriminating, though. It proves:

```text
legacy lifecycle reaches FabricJoined
empty receipt list does not validate
```

Those are independent facts. A future function converting `FabricJoined` into an intake receipt could be added and the witness would remain green. fileciteturn41file0L2-L2

Replace or supplement it with a dependency/producer wall that derives and requires:

```text
consumers of BmcOnboardingPhase in gunbc.machine_intake_* = []
producers from BmcLifecycleState to intake receipt/standing/disposition = []
```

That correction is small and does not require doing the full split now.

The `MtCollinsBmcImplementationExpected` correction itself is accepted, but the adjacent strings still call the unpreserved `0.32` report an “observation,” say it was “grounded by execution,” and describe fallback to a platform default. The same module now admits that the raw producer was not preserved. Rename those to an explicitly unverified historical operator-report note and remove any claim that a workflow may use them as fallback authority. fileciteturn35file0L2-L2

# Next slice

**INTAKE-AGENT-0 is next. Not MEMORY-0, and not the full `bmc_onboarding` split.**

MEMORY-0 would otherwise model report ingestion, attribution, and verdicts before the producer capable of returning the report exists. The project’s own sequence places the minimal multi-architecture agent before MEMORY-0, and BOOT-DELIVERY-0 acceptance explicitly requires both physical transports to reach that common nonce-bound callback. fileciteturn45file0L2-L2

I would make the next serial gate:

## INTAKE-AGENT-0A — artifact and callback contract

Exit requires:

1. Deterministic x86-64 and AArch64 artifacts with recorded content identities.
2. A callback carrying attempt nonce, exact artifact digest, architecture, raw board serial, assembly-observation digest, firmware-observation digest, and boot time.
3. Replay and duplicate-callback refusal.
4. Wrong-nonce, wrong-artifact, wrong-unit, and wrong-architecture RED controls.
5. Canonical durable receipt bytes and their SHA identity.
6. Raw EDAC/RAS, inventory, and sensor capture only—no memory pass/fail verdict yet.
7. Local execution of the callback contract without needing a physical BMC.

Then run a narrow **ACCESS-OBSERVE-0 / BOOT-DELIVERY-0E** slice on hardware:

```text
Mt. Collins observation → profile lookup → plan → execute → agent callback → cleanup
ASRock observation      → profile lookup → plan → execute → same callback → cleanup
```

Only after those producers exist should MEMORY-0 consume the real agent evidence. The `bmc_onboarding` split remains a gate before the general effectful `intake` executor or before live `BmcSecure`, `OsInstalled`, or `ServiceRuntimeAdmitted` producers—not before building the diagnostic agent artifact.

## Merge gate from this review

Before approval of #9690:

1. Bind target and exact `BootArtifact` into the solve and plan.
2. Replace the independently supplied capability/profile/observation tuple with one validated, attempt-bound access context carrying real producer provenance.
3. Bind Redfish locator/protocol and MegaRAC OEM requirements into the selected transport.
4. Remove the capability-only solver and the `FirmwareUpdateThenVirtualMedia` shortcut; keep only the fail-closed compatibility projection until its real consumers migrate.
5. Add phase/prerequisite, interval, policy, and producer compatibility to ledger validation; preserve ledger provenance in admission.
6. Split `FirmwareTransition*`, preferably `NetworkBoot*`, and regenerate namespace admissions.
7. Replace the onboarding tautology with a dependency/producer wall and quarantine the unpreserved Mt. Collins prose as historical report only.

This review is against the requested exact head. Current `main` has since advanced to `1b59e7d3de3d11d1de7e1551cfbac092067660a7`, so after the architectural corrections the branch must be brought current and the namespace census and claims rerun on the final integrated head. fileciteturn2file0L12-L16 fileciteturn48file0L2-L2
