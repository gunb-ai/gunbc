# Machine-intake ruling 4 (2026-08-30) — reviewer second pass on head cdbf4611bb

Transcribed verbatim from the side chat (https://chatgpt.com/c/6a933560-5590-83ea-bc5c-731d17bfa2e3). The two short interim notes precede the ruling.

> The structure is much stronger, but one splice remains: `request.target` is not proved equal to the bound access subject/endpoint. I’m checking transport-local evidence for analogous joins.
> The deleted solver chain and module cut look sound; keep `LedgerPhaseUnranked` and the fail-closed projection. #9717 instead exposes that this PR’s changed witnesses never ran.

## Ruling

**HOLD / request changes on exact head `cdbf4611bbc472275130722cd6dc3da3d12fea42`.** This is substantially closer than the prior head: the second solver is gone, the compatibility layer is honestly fail-closed, the ledger now enforces lifecycle ordering, and the authority split is mostly right. The remaining blockers are narrow joins and provenance gaps rather than another redesign.

Also, all seven gate items are not yet present on this exact head. The onboarding dependency wall and Mt. Collins prose correction are still absent; the current witness remains the earlier “legacy terminal plus empty ledger” test, and the current Mt. Collins module still describes the unpreserved run as an executed observation and a fallback platform default. fileciteturn76file0L2-L2 fileciteturn77file0L2-L2

## What is accepted

The deletion decision is accepted. `solve_install_mechanism`, `FirmwareUpdateThenVirtualMedia`, the generated proxy-DHCP path, and the capability-only default-to-PXE chain are gone. `gunbc.os_install_mechanism` now contains only projections of `BootDeliverySolution` and per-host fail-closed standings. **Keep that projection until its two real consumers migrate; do not delete it merely because it currently refuses.** It is no longer a second solver. fileciteturn65file0L2-L2

The module cut is accepted. `gunbc.network_boot_delivery` is the right home for network-boot establishment, and `gunbc.bmc_firmware_transition` correctly states that a transition is not a delivery and is not consumed by the delivery solver. Its unbuilt producer can remain a declared next rung because nothing uses an `Established` value to authorize delivery today. fileciteturn62file0L2-L2 fileciteturn73file0L2-L2

The ledger climb is accepted at its stated rung. It now checks one subject and attempt, one genesis, links, interval direction, nondecreasing start time, phase rank, policy identity, and producer identity. Admission preserves the terminal ledger fingerprint, policy digest, and exact receipt selected for every phase. The source is also explicit that this is structural-fingerprint validation, not the future durable SHA-256 ledger. fileciteturn68file0L2-L2 fileciteturn70file0L2-L2

**Keep `LedgerPhaseUnranked`.** Its current refusal is unreachable because every present `IntakePhase` is enrolled, but it is the fail-closed arm that becomes reachable when a future phase variant is added without being placed in the required ordering. That is a legitimate totality guard, not speculative product state. Add a positive witness that every current phase has exactly one rank; an authorable negative fixture is not required. fileciteturn69file0L2-L2

## 1. The request target is still not bound to the access context

`BootDeliveryRequest.target` carries a subject and endpoint. `BoundBmcAccessContext` independently carries a subject and endpoint. The access binder correctly joins the observation receipt to the subject it was asked to bind, but the delivery solver does not compare the resulting context with `request.target`; it uses the context to select a transport and then copies the request target into the output plan. The existing witnesses always build both sides from the same helper, so they do not discriminate this splice. fileciteturn59file0L2-L2 fileciteturn58file4 fileciteturn58file5 fileciteturn64file0

As written, this can plan:

```text
access observation and profile for subject/endpoint A
+
boot-delivery request for subject/endpoint B
=
plan that names B but was admitted using A
```

Add a binding step before candidate evaluation:

```dag
type BoundBootDeliveryTargetContext
  = DeliveryTargetContextBound {
      target: BootDeliveryTarget
      access: BoundBmcAccessContext
    }
  | DeliveryTargetSubjectMismatch { ... }
  | DeliveryTargetControllerMismatch { ... }
```

The solver should consume only the bound arm. Required REDs:

1. same endpoint, different attempt or qualification subject;
2. same subject, different BMC controller.

The same target binding is needed for the other evidence inputs. `ReinstallPathEstablished` is currently accepted while its `qualified_by` population is ignored; UEFI HTTP evidence carries an artifact digest but no target; network-boot establishment carries four evidence references and a digest but no unit, endpoint, or attempt. Evidence from one machine can therefore authorize another machine’s request even after the access-context join is repaired. fileciteturn58file0 fileciteturn62file0L2-L2

Each established transport standing should carry the `BootDeliveryTarget`, or consume a target-bound observation type whose sole producer performs that join. Artifact-digest agreement is necessary, but it does not bind the host.

The plan’s positive provenance should also preserve the evidence particular to the selected candidate. The generic `profile_provenance + observation_receipt + considered` product is sufficient for the profile-shaped candidates, but it drops configfs `qualified_by` evidence and UEFI HTTP’s evidence list. A `CandidateEligible` verdict alone does not tell an executor or reviewer what made that candidate eligible.

## 2. `"0.32"` is representable but still cannot inhabit a valid profile

The opaque `BmcFirmwareReleaseIdentity` repair is right. `"0.32"` remains exactly `"0.32"` and has no fabricated semantic patch component. However, every `BmcAccessProfile` still embeds a semantic `BmcFirmwareCapabilityRow`, and catalog admission requires the raw release’s optional semantic view to equal that row’s semantic firmware. For `"0.32"`, the semantic view is absent, so the match always refuses. fileciteturn72file0L2-L2 fileciteturn60file0L2-L2

The new witness demonstrates exactly that: `"0.32"` is representable, but both profile lookup and access-context binding refuse it. That is a useful RED, but it means the observed Mt. Collins firmware can never progress to `AccessProfileKnown` under this model. fileciteturn64file1 fileciteturn64file2

Introduce an exact-release capability carrier:

```dag
type BmcFirmwareReleaseCapabilityRow {
  release: BmcFirmwareReleaseIdentity
  capabilities: List<BmcCapability>
}
```

and make the new access-profile path consume it. The legacy semantic capability row can remain for existing OpenBMC consumers.

The required positive control is:

```text
raw release "0.32"
+ exact-release OEM-remote-media capability row
+ matching profile
+ matching synthetic observation receipt
→ bound access context and MegaRAC delivery plan
```

That is a fixture proving inhabitability, not a seeded fleet observation.

## 3. Controller identity is still fused to one protocol

The profile now correctly carries route inhabitants, including Redfish, IPMI lanplus, and MegaRAC REST. But `BmcEndpoint` itself remains `{ host, protocol: Redfish | Ipmi }`, and that route-specific value is used as the intake target and access-context endpoint. fileciteturn71file0L2-L2

One Mt. Collins boot needs at least two routes against one controller:

```text
MegaRAC REST for artifact delivery
IPMI lanplus for boot control/reset
```

A single `protocol` field cannot identify that controller without selecting one route in advance. The current fixture demonstrates the problem indirectly: the supposedly no-Redfish OEM profile is observed through a `BmcEndpoint` constructed with `protocol: Redfish`. fileciteturn64file0L2-L2

Do not expand `BmcProtocol` with another arm. Introduce a protocol-neutral intake identity, for example:

```dag
type BmcControllerEndpoint {
  host: NonEmptyStr
}
```

Use it in `BootDeliveryTarget`, `BmcAccessObservation`, and `BoundBmcAccessContext`. The profile’s `BmcAccessRoute` population remains the authority for how that controller is reached. Existing legacy users of `BmcEndpoint` need not migrate in this PR.

## 4. Promoted profiles must refuse while their provenance is unverified

You are right not to fabricate a receipt store. The conclusion, however, is not that `ProfilePromotedFromObservation` may produce `AccessProfileKnown` while the gap is merely described.

At present, two arbitrary hash-shaped values can be authored as `observation_receipt` and `promotion_receipt`, placed on a catalog row, and accepted by lookup and access binding. The positive fixtures do exactly this with synthetic content hashes. No producer establishes that either receipt exists or that the observation describes the profile being promoted. fileciteturn60file0L2-L2 fileciteturn64file0L2-L2

No store is required to fail closed now. Put verification in the workflow layer:

```dag
type ProfilePromotionVerificationStanding
  = ProfilePromotionVerified { ... }
  | ProfilePromotionUnverified {
      observation_receipt: ContentHash
      promotion_receipt: ContentHash
    }
```

`bind_bmc_access_context` may bind:

- a profile grounded directly in an external authority; or
- a promoted profile accompanied by `ProfilePromotionVerified`.

Without the future store, every promoted profile produces `ProfilePromotionUnverified` and refuses. That gives the missing discriminating control today without pretending that receipt lookup exists.

INTAKE-AGENT-0A can later introduce the store-backed producer that constructs `ProfilePromotionVerified`.

## 5. The Redfish probe still has two protocol authorities

`RedfishVirtualMediaProbeReceipt` carries both:

```text
progress = TransferProtocolAdmitted { protocol: ... }
admitted_protocols = [...]
```

The solver uses the progress arm only to decide whether the plan floor was reached, then decides protocol eligibility from `admitted_protocols`. Consequently, the receipt can claim `TransferProtocolAdmitted { NFS }`, list only HTTPS, and admit an HTTPS offer. fileciteturn63file0L2-L2 fileciteturn58file1

Use one representation:

- make the progress arm payload-free and let a nonempty admitted-protocol population establish the plan floor; or
- put the admitted protocol population directly on the progress arm and remove the sibling field.

The nested probe’s `raw_response` also needs to be part of the observation receipt’s verified evidence manifest. The access binder currently checks the overall observation `raw_response` against one `EvidenceRef`, but the transport plan derives its locator and protocols from the nested probe receipt’s separate hash. There is no structural statement that this second blob was among the captured evidence. A receipt manifest containing all referenced evidence digests is the clean correction.

## Ledger and durable-receipt ruling

`IntakeReceiptRecord` may remain deferred to **INTAKE-AGENT-0A**. The current code is honest that `ValidatedIntakeReceiptLedger` is an in-substrate structural ledger and cannot validate the future SHA-linked wire ledger. That is sufficient for the INTAKE-0 semantic model, but it is not sufficient for a live admission producer. fileciteturn68file0L2-L2

The PR body is now stale: it still claims a “canonical digest and hash-chain check” and still describes the superseded live-surface refusal rather than `InstallAccessProfileNotLookedUp`. Update it only after the final code head is fixed. fileciteturn79file0L8-L8

The gate before any effectful intake/admission executable is:

```text
IntakeReceiptRecord {
  envelope
  canonical_bytes_identity: Sha256Digest
}
```

with links checked against those supplied record identities, not recomputed FNV fingerprints.

## Onboarding and Mt. Collins worker

The proposed worker direction remains correct, but it must land before this review can close.

The dependency wall must inspect the live producer/consumer relation—no path from `BmcOnboardingPhase` or `BmcLifecycleState` into intake standings, receipts, disposition, or admission. The exact-head witness currently proves only that the legacy state reaches `FabricJoined` and that an unrelated empty receipt list does not validate; adding a conversion function would not make that witness red. fileciteturn76file0L2-L2

For Mt. Collins, retain `MtCollinsBmcImplementationExpected`, but rewrite or delete the adjacent rows that currently say:

- the stack was “established by execution”;
- `0.32` and no-Redfish are an “observation”; and
- workflows may fall back to this platform default.

The raw producer was not preserved, so these may survive only as an explicitly unverified historical operator report with no authority path into lookup or planning. fileciteturn77file0L2-L2

## Namespace ruling

The final module cut is approved, but do not transcribe admissions from the present intermediate head. The exact head still carries the earlier 31-entry roster describing the old move directly into `boot_artifact_delivery`. Those are not the final targets after the split into `network_boot_delivery` and `bmc_firmware_transition`. fileciteturn74file0

The sequence is:

1. land the target/context, raw-release, endpoint, provenance, probe, and onboarding corrections on the branch;
2. merge current `main`;
3. run the namespace instrument against that exact integrated head;
4. transcribe only the resulting final `TargetChanged` identities;
5. rerun once to prove no unadjudicated or stale admissions.

`main` has already advanced to `61c6dfde780430ea6fd20ac3985a72f5cdbd9384`, well beyond the PR’s recorded base, so the final census cannot be taken from `cdbf4611bbc472275130722cd6dc3da3d12fea42`. fileciteturn80file0L2-L2

## #9717 is not an unrelated CI incident

The red is truthful. #9717 was specifically built because prior PRs added witnesses that passed remotely but executed **zero** times in the required floor. It now blocks every changed witness whose required-floor disposition is declined, missing, or nonterminal. fileciteturn81file0L8-L8

The 2026-08-29 compiler-floor ruling and #9717 can coexist:

```text
static global required-gate roster = compiler floor only
dynamic per-PR changed-witness lane = exactly the changed witness identities
```

The honest repair is a changed-witness execution sublane:

1. reuse #9717’s already-derived changed identity set;
2. execute exactly that set, without adding product prefixes to the global compiler gate;
3. write terminal outcomes into the same disposition ledger;
4. let #9717 project those outcomes.

Do **not**:

- weaken #9717;
- classify `DeclinedOutsideGateClosure` as green;
- add a blanket exception for #9690;
- expand the standing global compiler gate with every product namespace;
- convert the 53 identities to expected-red or transition admissions.

The reported 74 remote greens are valuable development evidence, but they are not yet an exact-head required-CI receipt. They could substitute only if imported as subject-bound, identity-complete execution receipts that the required gate itself consumes.

At the time I checked, the exact-head GitHub run was still nonterminal: Rust unit tests had succeeded, while the required build and floor jobs were still running. Thus there is not yet a final CI verdict on `cdbf4611bbc472275130722cd6dc3da3d12fea42` to classify. fileciteturn82file0L2-L2

The changed-witness execution repair can land as its own CI PR ahead of #9690. Until it does, #9690 remains correctly blocked rather than “green except for infrastructure.”

## Next slice

After these corrections and #9690’s merge, **INTAKE-AGENT-0A remains next**.

That slice should produce the missing common substrate:

1. deterministic x86-64 and AArch64 artifacts;
2. canonical SHA-256 `IntakeReceiptRecord` identities;
3. an evidence manifest/store capable of resolving observation and promotion receipts;
4. nonce-bound callback with artifact, architecture, unit, assembly, firmware, and attempt identity;
5. replay and duplicate-callback refusal;
6. raw inventory, EDAC/RAS, sensor, and access-observation captures without memory verdicts;
7. local callback-contract execution before hardware use.

That slice supplies the producer needed to verify `ProfilePromotedFromObservation` rather than forcing #9690 to invent a generic receipt store. Then execute the Mt. Collins and ASRock access/boot callbacks. **MEMORY-0 follows those real agent receipts.** The full `bmc_onboarding` split remains required before the general effectful intake executor or live `BmcSecure`, `OsInstalled`, or `ServiceRuntimeAdmitted` producers, not before the agent artifact.

## Final approval gate

Approval requires one final head containing:

1. request target ↔ access-context subject/controller binding;
2. target-bound configfs, UEFI HTTP, and network-boot standings;
3. a positive exact-raw `"0.32"` capability/profile path;
4. protocol-neutral BMC controller identity;
5. one Redfish protocol authority plus bound probe evidence;
6. fail-closed unverified promoted-profile provenance;
7. the real onboarding dependency wall and historical-report rewrite;
8. namespace admissions generated from the final current-main composition;
9. updated PR body;
10. terminal required CI in which every changed witness has an executed disposition.

The architecture no longer needs another broad rewrite. These are the remaining seams where independently valid values can still be paired to manufacture a plan about the wrong subject, controller, release, route, or producer.
