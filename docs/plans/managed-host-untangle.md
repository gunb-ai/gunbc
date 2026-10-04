# Managed-host untangle: mtcollins1 → unit / platform / silicon / BMC stack / fleet procedure

*Work item adhoc-81aa03f8-bc8 (session warm-crane-577), measured at `26e99a9c7f`. Governing doctrine: DESIGN §3 (single authority; replacement migrations cut over at the root, consumers enumerated before deleting a root that is outside the required gate), §3b, §3c, and [the replacement migration doctrine](replacement-migration-doctrine.md).*

## Why

A second Ampere Altra platform, Mt. Jade 2U (unit `mtjade1`), is arriving. Most of what is named for `mtcollins1` is not specific to Mt. Collins: it is Ampere silicon knowledge, AMI MegaRAC knowledge, or fleet procedure for any BMC-managed host, with the unit hardcoded through a handful of constants. As long as the procedure is `mtcollins1_*`, Mt. Jade has two options: fork the procedure (a §3 nickname) or wait. The fix is the root seam below, plus one cut per authority.

## How the census was taken (the instrument)

This is an import census over every `.dag` module under `dag/` and `src/v2/`, not a grep for consumers:

- **Population**: every module whose name or text matches `mtcollins`. That is 269 modules: 81 witnesses and 188 production/reference modules.
- **Consumers**: the reverse `import` edges into each module.
- **Gate**: transitive import closure from `v2.workflow.required_floor` `required_gate_prefixes`.
- **Constants**: the `mtcollins|mt_collins|MtCollins` symbols each module imports from another module. These are the unit/platform constants it depends on.

The method is under **Census method** below. It is a one-off that each cut re-runs at its own head.

### Findings that shape the plan

1. **No layer-1/2/3/5 machine-intake module is in the required gate.** The 12 gate-reachable members are generic modules carrying prose or a single row:
   - `extdeps.bmc.megarac`
   - `extdeps.boards.types`
   - `gunbc.fleet_intent_network`
   - `gunbc.secret_provision`
   - `gunbc.ci_layer_roots`
   - `gunbc.durable_cas_file_store`
   - `gunbc.floor_demand`
   - `gunbc.roadmap_authority`
   - `gunbc.runner_registration_labels`
   - `gunbc.runner_throughput_qualification`
   - `v2.workflow.floor_route_gap`
   - `test.claim.action_use_admission_witness`

   **So every cut below deletes a root outside the gate.** Per §3, each child brief carries its consumer list by name, from the tables below, and must run each named consumer's witnesses itself. A green required run proves nothing about these modules (`docs/rung-drops/required_lanes_do_not_resolve_product_layer_modules_2026-09-20.txt`).

2. **The unit enters through about ten roots.** These are the most-imported unit symbols:

   | Symbol | Home | Importers |
   |---|---|---|
   | `mtcollins1_endpoint` | `gunbc.machine_intake_mtcollins1_access_observation` | 10 |
   | `operator_host_mtcollins1` | `gunbc.fleet_intent_network` | 7 |
   | `mtcollins1_firmware`, `mtcollins1_capability_row`, `mtcollins1_access_observation` | access observation | 7 / 5 / 4 |
   | `mtcollins1_bmc_gunbc_secret_ref` | `gunbc.secret_provision` | 6 |
   | `mtcollins1_secured_account` | `gunbc.machine_intake_mtcollins1_bmc_secure_observation` | 6 |
   | `mtcollins1_unit_hold_key`, `mtcollins1_unit_hold_store_host` | `gunbc.machine_intake_mtcollins1_maintenance_hold` | 6 / 4 |
   | `mtcollins1_cdrom_selection` | `gunbc.machine_intake_mtcollins1_actuate` | 5 |
   | `mtcollins1_host_nic_mac` | `gunbc.machine_intake_mtcollins1_netboot_client_observation` | 7 |
   | `mtcollins1_boot_export_dir` | `gunbc.machine_intake_mtcollins1_boot_artifact` | 6 |
   | `mt_collins_channel_population_roles`, `mt_collins_channel_connector_pairs`, … | `extdeps.ampere.mt_collins_*` (the board figure) | 10 / 5 |

   **Fleet procedure is unit-specific only through these roots.** Thread a host record through them and the procedure modules become host-generic without restating their logic.

3. **`gunbc.host_reset_subject_roster` `ResetSubjectCandidate` is already the per-host record in embryo.** It carries:
   - `host` (a `HostIdentity`)
   - `observation` (a `BmcAccessObservation`, which carries the endpoint)
   - `capability_row`
   - `username`
   - `boot_selection`
   - `unit_hold`

   Its note says a second host joins "the day it authors an observation". But it is named and scoped for *reset*, and the boot, hold and census procedures each re-import the same unit constants independently. Those are four consumers holding four copies of one row.

4. **`gunbc.machine_intake_subject` is not the home.** `MachineIntakeSubject` / `QualificationSubject` is the *identity of a unit under qualification*: unit key, assembly manifest, firmware manifest and attempt. It has no BMC endpoint, credential or stack. It stays what it is, and the new record references it rather than absorbing it.

5. **`mtjade1` has no `HostIdentity` row.** `gunbc.fleet_intent_network` names `operator_host_mtcollins1` but no `mtjade1`. What exists for the unit:
   - `gunbc.machine_intake_mtjade1_access_observation` carries `FamilyUnobserved`, a refused unit key, and `SecretLocusReservedNotOnAccessorRoster`.
   - `gunbc.machine_intake_bmc_firmware_family_discriminator` and `bmc_first_contact_standing` already model "BMC family not yet observed".

6. **The platform home exists.** `extdeps.boards.types` `BaseboardModel` / `ServerBaseboard` is in the gate, has 25 importers, and is where the board figure is keyed.

7. **`.github/workflows/fleet-converge.yml` is a projection.** Its authority is `gunbc.fleet_converge_workflow` (271 references, 4,446 lines). It carries 8 `mtcollins1_*` mode literals in conditions and about 30 `mtcollins1_*` step/job identifiers. Renaming any mode needs operator sign-off.

## The root seam (revised after the #13046 review)

**Name.** The name was chosen by DFS of existing vocabulary. The corpus already says *managed* for "a host this fleet drives through its baseboard controller" (`gunbc.host_reset_subject_roster` contrasts it with "executor fleet host"). No `ManagedHost` type exists, and none of the existing `Managed*` types is the same concept.

**The record is a projection over standings that already exist, not a new bundle.** Two joined authorities already cover most of it:

- `gunbc.machine_intake_access` `BoundBmcAccessContextStanding` binds the `MachineIntakeSubject`, endpoint, observed identity (which includes baseboard and BMC family), capability row, profile, observation and evidence manifest.
- `gunbc.machine_intake_bmc_secure` `BmcSecureStanding` / `ManagedCredentialReference` carry the secured account, role, `SecretRef` and credential epoch.

Restating any of that as loose `baseboard` / `bmc_stack` / `credential` fields would be a second authority and would lose the capability row that `reset_subject_admission` needs. So:

```
module gunbc.managed_host
type ManagedHost {
  host: HostIdentity                       // gunbc.fleet_intent_network, the sole naming authority
  access: BmcAccessObservationStanding     // endpoint and observed identity (baseboard, BMC family)
  capability_row: BmcFirmwareReleaseCapabilityRow  // the pair oob_boot_handoff_admission consumes today
  secure: BmcSecureStanding                // managed credential: account, role, SecretRef, epoch
  unit_hold: ResetUnitHold                 // moved here from host_reset_subject_roster
}
fn managed_hosts() -> List<ManagedHost>
fn managed_host_binding(h: ManagedHost) -> ManagedHostBindingStanding   // THE join; effectful consumers read only this
```

**The seam is a join, not a bundle** (side-chat review, 2026-10-03). There are two invariants.

1. **One joined standing.** The subject is derived from `secure.subject` and never stored again. `managed_host_binding` refuses, with a typed and located cause, unless all three hold:
   - `access` is observed;
   - the observed endpoint equals the `BmcSecured` endpoint;
   - the secured subject's unit is bound to the row's `host`.

   Every effectful consumer (reset, boot, hold) reads the credential and the endpoint only through the bound result. Without the join, unit A's secured account could be used against unit B's controller. The discriminating RED uses a secure standing for endpoint A and an access observation for endpoint B.
2. **Operation admission includes the operation's policy.** A host is a member of an operation's roster only if that operation's policy is decided for it. For reset, one reset-specific authority takes the bound `ManagedHost` plus the reset boot selection and produces an admitted reset subject. The roster lists exactly those hosts, and `gunbc.host_reset_return_run` consumes the selection from that admitted subject with no second lookup. The RED is a capable host with no reset selection, which must be absent from the roster. Every later cut that adds an operation (boot, hold) follows the same shape.

Rules for the record:

- **Baseboard and BMC family are read through `access` (its observed identity), never stored beside it.** A procedure that needs MegaRAC matches the bound identity and refuses otherwise.
- **The silicon field arrives in cut 1** as a standing authored there, not as a placeholder in cut 0.
- **Cut 0 has exactly one row, `mtcollins1`.** Its `subject` and `secure` come from `gunbc.machine_intake_mtcollins1_bmc_secure_observation` (`mtcollins1_intake_subject`, `mtcollins1_bmc_secure_standing`). Its `access` and `capability_row` are the unit's existing `mtcollins1_access_standing` and `mtcollins1_capability_row`.
- **Why `access` is the observation standing plus the capability row, not `BoundBmcAccessContextStanding`** (measured by cut 0, 2026-10-03):
  - `gunbc.machine_intake_access` `bind_bmc_access_context` has no production caller.
  - There are no production `AccessObservationReceipt` values.
  - No `BmcAccessProfileCatalogRow` exists outside type declarations.
  - Binding mtcollins1 would require authoring a catalog row whose provenance cannot be honestly decided today. `ProfileFromExternalAuthority` has no source, and `ProfilePromotedFromObservation` binds only beside a `ProfilePromotionVerified` that no producer makes.

  So the row carries the pair that the consumed admission (`oob_boot_handoff_admission`) actually reads today. This is a **declared frontier, not a second authority**. Its trigger: the first production producer of a `BoundBmcAccessContextStanding` for a managed host, which needs a catalog row with a decided provenance. In that change, `access` and `capability_row` collapse into the bound context and these two fields are deleted.

- **`mtjade1` gets no row yet.** It has no `MachineIntakeSubject` (its unit key is refused), no endpoint, no family and no hold decision. Fabricating any of them to populate a row is the §5 defect the review named.
- **mtjade1's absence is the honest standing.** Every procedure refuses a host that is not in `managed_hosts()`.
- **mtjade1 joins on a trigger, not by edit.** It becomes a row the day first contact authors its subject and access context standing (`gunbc.machine_intake_jade_first_contact_frontier`) and a hold decision is made. That is the trigger, and it is not a cut of this program.
- **`ResetSubjectCandidate` dissolves.** `gunbc.host_reset_subject_roster` becomes a filter over `managed_hosts()` with the same admission fold, and `boot_selection` returns to the reset arm as reset policy.

**Operation-keyed policy is not host identity** (ruling (b), 2026-10-03). The boot federation is policy for the pair (managed host, operation):

```
ManagedHostBootFederationStanding {host, operation, pool, provider, service_account, environment, claim_pins, secret_grants}
```

- `mtcollins1` is bound to the existing live names (`github-mtcollins1-boot` pool and provider, the `mtcollins1-boot` service account and GitHub environment, and the grants), which are not renamed.
- Any other host is explicitly unprovisioned, and the generic job refuses it.
- None of these ids appear on `ManagedHost`.

## Rulings recorded (operator, via eager-gull-22 side-chat review, 2026-10-03)

- **(a)** The fleet-converge mode recut is approved in principle as the **last** cut: one atomic transition, no aliases, the eight old literals deleted in the same change.
  - Do **not** overload the existing `host` input, which is the executor host. Add a distinct managed-subject input and keep the executor axis.
  - Names are derived after cuts 2–6 expose the axes. Census-QEMU modes are executor-toolchain operations, census image publish/readback are artifact operations, and boot/fan/UI/KVM are managed-host operations.
  - The final names still need operator sign-off.
- **(b)** Keep the live GCP IAM names. Model them as the operation-keyed standing above.

## Ordered cut list (re-cut vertically)

**The governing rule** (review item 3): *a module may not acquire a generic name while its answer is still transitively fixed to mtcollins1.* Each cut therefore moves a module only after everything it transitively imports from the `mtcollins1` population is either already generic or is a layer-1 unit observation that the generic module now receives as a **parameter** or reads off the `ManagedHost` row.

The boot vertical is large. Its transitive closure from `gunbc.machine_intake_mtcollins1_boot_run` reaches 50 census modules, including:
- the layer-1 `mtcollins1_memory_census_observation`;
- the layer-3 `mtcollins1_smpro_observation`;
- the layer-4 `mtcollins1_kvm_still`, `mtcollins1_bmc_sensor_observation` and `mtcollins1_bmc_secure_observation`.

So the BMC and silicon leaves it needs move first.

| # | Cut (one authority) | Root deleted | Sequencing |
|---|---|---|---|
| **0** | **Seam**: `gunbc.managed_host` as above, `mtcollins1` row only; dissolve `ResetSubjectCandidate`. | `reset_subject_candidates`, and the roster's direct `mtcollins1_*` imports | Now. Disjoint from #13025 and #13041. |
| **1** | **Silicon**: Ampere-generic SMpro/PMpro, boot-stage, `CP:` and CCIX decoding moves out of `mtcollins1_smpro_observation` into `extdeps.ampere.*`, next to the existing `ampere_{dram,socket}_console_observation`. Adds the silicon standing to `ManagedHost`. | the generic decoders inside the unit module | After #13025, and after cool-ant-760's error-record decoder lands. Public-tree rule: no SCP UM, no CHANGELOG.txt, nothing disassembled. |
| **2** | **Maintenance hold**: host-generic over `ManagedHost.unit_hold`. | `mtcollins1_unit_hold_*`, `mtcollins1_maintenance_hold_{take,release}` | After 0. The env var `GUNBC_MTCOLLINS1_MAINTENANCE_REASON` is a workflow surface: keep it until cut 7. |
| **3** | **Boot-critical BMC leaves**: `mtcollins1_kvm_still`, `mtcollins1_media_attach` (the MegaRAC virtual-media half), `mtcollins1_bmc_sensor_observation`, and the secured-account projection of `mtcollins1_bmc_secure_observation`. These become MegaRAC-scoped modules that take a `ManagedHost` and refuse a non-MegaRAC bound identity. The unit's own readings stay unit-named. | those modules' `mtcollins1_` procedure roots | After 0. |
| **4a** | **Boot subject leaves**: artifact, image fetch, milestone, phase timing, admission, authorization, actuate. `MtCollins1BootSubject` becomes a boot subject over `ManagedHost`. | `MtCollins1BootSubject`, `mtcollins1_boot_subject*`, `mtcollins1_cdrom_selection`, `mtcollins1_boot_export_dir` as procedure inputs | After cuts 1 and 2. **Not after cut 3**: measured on main 2026-10-04, the 4a modules import none of the cut-3 modules. The census-image import in the boot authorization becomes the boot **medium parameter**, so 4a does not wait on 4b either. The runner-image medium arm and its terminal milestone (quiet-stag-623's lane) land on top of 4a. |
| **4b** | **Census boot image chain**: the 7 `mtcollins1_census_*` modules. Toolchain and QEMU modules are executor-host operations, so they are keyed by executor host, not managed host. | the `mtcollins1_census_*` procedure roots | After 4a. |
| **4c** | **Boot federation standing**: `ManagedHostBootFederationStanding` replaces `gunbc.auth.mtcollins1_boot_federation*`, live names kept as the `mtcollins1` binding (ruling b). | `mtcollins1_boot_*` federation decls | After 0. Independent of 4a/4b. |
| **4d** | **Boot run, dry realization, diagnostic bundle**: the vertical's root. Unit observations it reads today (`mtcollins1_memory_census_observation`, `mtcollins1_access_observation`) become row reads or parameters, so the dependency direction is procedure ← unit. | `mtcollins1_boot_run`, `_boot_dry_realization`, `_boot_diagnostic_bundle` | After 1, 4a, 4b, 4c. Likely split run/dry and bundle at dispatch. |
| **O1** | **Arrival convergence, factory state through an admitted managed host** (operator requirement and rulings, 2026-10-03). PRs: O1a route standing, O1b state-shaped `BmcSecure` and observer clock, then O1c in three parts (the shared step-sequence fold with its first consumer; the prior-life archive; `BmcSecure` wiring and the admission population). See its own section below. | the family-keyed rotation route; the rotation-event-only `BmcSecure` derivation; the supplied `BmcSecureStanding` in the firmware convergence; the source-roster `managed_hosts()` | After 4c. Does not need the generic boot run. Each unit's live credential write needs its own operator go-ahead; mtcollins1 goes first. |
| **O2** | **Boot enters through the arrival convergence** (replacement migration): the `mtcollins1_boot` mode is re-rooted onto the convergence in one transition, and the old boot entry is deleted. | the boot run's own entry (the root that assumes `BmcSecured`); the caller-supplied password path into the boot wrappers | **After 4d.** The convergence is host-generic, so it may not call a boot run whose answer is still fixed to mtcollins1. |
| **5** | **Remaining BMC-stack observations**: fan observe, UI bundle observe, KVM observer, SOL notice, served-UI catalog, BMC fan, which the boot does not need. | those `mtcollins1_` procedure roots | After 3. |
| **6** | **Platform**: physical orientation, DIMM connector and platform observation become logic over the baseboard read through `ManagedHost.access`. The Mt. Collins figure stays in `extdeps.ampere.mt_collins_*`. | the board bindings in those modules | After #13025 (it edits `mtcollins1_physical_orientation`). |
| **7** | **Workflow modes**, per ruling (a). | the 8 `mtcollins1_*` literals and their steps in `gunbc.fleet_converge_workflow` and its projection; `mtcollins-canary.yml` dispositioned in the same change | Last. Operator sign-off on names first. |

### Cut O — arrival convergence (plan delta, 2026-10-03; revised for the side-chat review of `b42944791e`)

**Requirement** (operator, via eager-gull-22). Boot is to be entered from onboarding, starting at **factory state**. Today's boot run assumes `BmcSecured` already holds. First subject by requirement: `mtjade1` (MegaRAC at 192.168.1.246, factory `admin`/`admin`, BMC clock reading the year 2000). First **wet** subject by ruling: `mtcollins1`.

**Framing (operator ruling, 2026-10-03): a consolidation, not a new mode beside the boot.** It is a replacement migration under DESIGN §3.
- The arrival convergence becomes the one entry for boot, and no second boot route survives beside it.
- No new mode, no new job, no renamed mode. The literal `mtcollins1_boot` and its job stay byte-identical until cut 7.
- Boot is the gap-intolerant boundary, so the staged form applies: the convergence is built and witnessed first (O1), then **one transition** re-roots the mode and deletes the old entry together (O2).

**Module name.** `gunbc.machine_intake_arrival_converge` (approved). It avoids "onboarding" because `gunbc.bmc_onboarding` is quarantined legacy (ruling 2026-08-29). That module is not imported, and the quarantine witness keeps holding.

#### Scope: a prefix of the existing `ArrivalPhase` order, exactly

`gunbc.machine_intake_phase` `arrival_phases_all` orders twelve phases. Cut O is the **prefix through `BootDeliveryEstablish`**: five phases, in the authority's own order, with none omitted and none added. The later phases (`DiagnosticBootAttest` through `ArrivalCleanup`) are not in this cut and keep their present standing. The fold is over `arrival_phases_all`, cut at `BootDeliveryEstablish` by `intake_phase_rank`. It is not a second, shorter list.

Each phase yields one result and one receipt. Supporting operations are **nested under the phase that consumes them**; they are not phases.

| Phase | Goal, read back independently | Effects, and whether each is a write | Nested support |
|---|---|---|---|
| `AccessDiscover` | access observation, firmware family, **and the sealed route standing** (below) | read-only probes | the controller's own clock reading, recorded as an observation and never used for ordering |
| `IdentityBindProvisional` | the exact `MachineIntakeSubject` (below) and its `HostIdentity` binding | read-only: FRU, firmware versions | subject construction; host allocation lookup |
| `PriorLifeBoundary` | `LogsArchivedWithBaselineCursor`: the prior-life carriers are archived as content-addressed evidence and a cursor is derived from that archive (below) | **read-only arm**: archive, derive the cursor, clear nothing. **`LogsArchivedAndCleared`**: a write with its own admission and operator sign-off, **not in this cut**. | per-carrier completeness standings |
| `BmcSecure` | the existing conjunction: the managed credential is accepted **and** the published credential is refused on every LAN member of the channel census | **write**: the typed account action, operator-gated per unit | secret generation, store and exact-version fetch **before** the write (below) |
| `BootDeliveryEstablish` | **one eligible delivery transport selected from live observations**: a `gunbc.boot_artifact_delivery` `BootDeliveryPlan`, bound to subject, controller and artifact. **This is not the boot run.** | read-only observations; the selection | managed-host admission (below) |

`PriorLifeBoundary` has no implementation today: the phase is declared and nothing produces it. A factory or repaired unit cannot skip it, so O1c builds its read-only arm. That is new work this plan had omitted.

**What the `PriorLifeBoundary` read-only arm is** (`docs/plans/machine-intake-design.md` §5: `PriorLifeBoundaryEstablished = LogsArchivedAndCleared | LogsArchivedWithBaselineCursor`). It is an **archive**, not a bare cursor.
- **Carriers archived**, each as content-addressed evidence with its own completeness standing: the BMC clock, the SEL and event logs, the Redfish log collections, audit and account events, the current boot override, virtual-media state, power state, and a sensor snapshot.
- **Cursor:** derived **from that archive**.
- **Nothing is cleared.**
- **Incomplete archive:** a required carrier that cannot be archived makes the phase refuse, or records a typed gap. It is never silently omitted.
- **Policy admission:** the design admits this arm "only where platform policy explicitly admits an uncleared append-only boundary". That admission is an authored policy row per platform. It is a decision this cut must state for Mt. Collins and Mt. Jade, not assume.
- **Control:** only the SEL final record id, with the other carriers absent → not established.

**What `BootDeliveryEstablish` is, and is not.** It is phase 5: a delivery plan or standing. The boot run is not part of it, and establishing it completes no later phase.
- **O2:** consumes the established delivery to enter the existing boot run.
- **Controls:**
  - delivery selected → `BootDeliveryEstablish` established **and `DiagnosticBootAttest` unestablished**;
  - no `BootDeliveryEstablish` → the boot entry cannot start.

**O1 and O2.** O1 runs the prefix through `BmcSecure` and ends at an admitted managed host. O2 adds `BootDeliveryEstablish` (the delivery standing) and re-roots the boot mode so that the existing boot run is entered only from an established delivery. It lands only after cut 4d has made the boot run host-generic. The boot run itself, and every phase after phase 5, stays what it is. mtcollins1's wet `BmcSecure` control needs no boot, so it runs at O1.

#### The subject (`IdentityBindProvisional`)

A FRU serial yields only a `UnitKeyStanding`. A `MachineIntakeSubject` is a `QualificationSubject` (unit key, assembly-manifest digest, firmware-manifest digest) plus an `IntakeAttemptId`. Each part has a named producer:

| Part | Producer | Evidence |
|---|---|---|
| unit key | `gunbc.machine_intake_subject` `bind_unit_key` over `BoardSerialObservation`s | committed FRU capture |
| provisional assembly manifest | `AssemblyManifest` of the `ComponentIdentity` rows readable before boot (board, BMC, and what the FRU and the BMC inventory expose), digested by `assembly_manifest_digest`. Provisional is the phase's own word: `InventoryConform` later re-derives it from the booted host. | the same captures |
| live firmware manifest | `FirmwareManifest` of the versions read back from the controller now, digested by `firmware_manifest_digest` | committed version readbacks |
| attempt identity | one `IntakeAttemptId` minted per convergence run | the run's receipt |
| subject | `qualification_subject_of`, then `MachineIntakeSubject` | the above |

**That exact subject flows through `BmcSecure`, managed-host admission and boot.** A firmware reflash changes the firmware-manifest digest, so `compare_qualification_subject` reports `FirmwareManifestStale`. A `BmcSecureStanding` bound to the old subject is then not current and is never reused: re-entry after a reflash re-derives the subject and re-runs the phases against it.

**Host identity.** A `UnitKey` is not a fleet-operation identity. The `HostIdentity` comes from `gunbc.fleet_intent_network` (naming) and `gunbc.hostname_allocation` `gunbc_fleet_hostname_allocations` (the allocation row). The binding of a `HostIdentity` to a subject is recorded in the admission record below. mtjade1 has neither a name row nor an allocation today, and both are authored decisions, not derived.

#### The route standing (`AccessDiscover`), corrected first (O1a)

`gunbc.bmc_implementation_dispatch` `rotation_route_for_family(AmiMegaRac)` answers `RotationRouteUnavailable` for a whole family. That is wrong in the direction the evidence shows, and the replacement must not be another family-keyed answer. The evidence is exactly this, and no wider:

- **mtcollins1**: an IPMI user-management rotation was **executed** on that controller and build (`artifacts/bmc/mtcollins1-bmc-user3-route-2026-09-13.txt`).
- **mtjade1**: a Redfish Manager **read** is committed (#13065). It proves a Redfish surface. It does not prove a Redfish AccountService write, nor an IPMI user-management request, on that controller.

So the route is a **sealed standing bound to an endpoint, a firmware build and evidence**, derived **inside** the convergence after `AccessDiscover`. It is not passed in, which withdraws this plan's earlier "selected before and passed in" sentence: that would have been a second observation authority. The arms:
- **grounded by an executed request on this controller and build** (carries the evidence);
- **grounded by a cited source for this build** (carries the citation);
- **ungrounded.**

An ordinary `BmcSecure` Apply consumes **only a grounded standing**. mtcollins1's IPMI route does **not** authorize mtjade1, whose route is ungrounded today.

**Grounding a route is its own effect, not the Apply.** The Apply cannot produce the premise that admits it, so there is a bootstrap with its own authorization:

`RouteUngrounded` → `RouteQualificationCandidate { endpoint, firmware_build, request shape, evidence }` → `RouteGroundedByExecutedRequest { endpoint, firmware_build, request receipt, response receipt }`

- The transition is made by a **separately authorized route-qualification effect**. It is classified as its own privileged effect through `select_authorization_pattern`, and it has its own operator go-ahead per controller.
- The candidate names the exact request shape and the evidence that makes it worth trying (an observed surface, a public standard's operation). It authorizes nothing by itself.
- Only the grounded arm enters ordinary `BmcSecure` Apply.
- **RED:** `RouteUngrounded` + operator approval of the rotation + ordinary Apply → refuse.

This is the MegaRAC rotation operation that the runner-bringup gap analysis lists first. The route standing is keyed by endpoint and build and carries request and response receipts, so the same model serves that backlog's other consumers without a second route vocabulary.

Controls:
- two `AmiMegaRac` controllers with different observed and executed surfaces produce different route standings;
- a Manager GET alone cannot construct an account-write route.

`rotation_route_consumption_frontier` is retired by the actuator in O1c reading this standing.

#### Request shapes, and what a grounding establishes (O1a-2, ruled 2026-10-04)

The route standing is per **request shape**, per controller and build. Two shapes exist.

- **Set User Password** (`IpmiSetUserPassword`). mtcollins1's committed execution grounds it **at BMC 0.32 only**. The controller is now on 0.45.3, so for the current build this route is another build's and the Apply refuses (`RouteForAnotherBuild`).
- **Set User Access, privilege-limit-only form** (`IpmiSetUserPrivilegeLimit`). It is needed because the `BmcSecure` conjunction requires the published credential refused on every LAN member, and a password write cannot set a channel privilege to no-access. mtcollins1 is a **candidate** only, and nothing is grounded.

What the model holds:
- **One shape never admits another.** A route grounded for one request shape does not admit an Apply of the other.
- **A grounding records what it established.** `EffectObserved`, or `AcceptedEffectUnobserved`, computed from before and after readings. A privilege-**lowering** write is admitted only over an effect-observed grounding.
- **The lockout guard runs before the first dangerous write, including the qualification write itself.** The live transport may be driven only from a sealed admitted qualification request. For the privilege-limit shape, that request requires a sealed, evidence-bound readback from the same run showing the goal's managed account at administrator on that channel. The readback must be for the same controller and build, with one unambiguous row.
- **mtcollins1's qualifying request is non-destructive.** It re-asserts the value already there. That grounds acceptance only, so it does **not** admit closing the factory administrator.

**What mtcollins1's wet transition therefore needs, in order.** Each item is an operator-gated act; eager-gull-22 batches them into one go-ahead.
1. A committed read-only post-reflash probe: firmware version, channel info for every channel, and user access and user name for every user id on each LAN channel. The last of these is what lets a candidate name a spare slot.
2. Qualify Set User Password at 0.45.3.
3. Converge the managed account and read it back.
4. Qualify the privilege-limit shape with a **discriminating, harmless** request: a spare user slot changed, read back, then restored.
5. Only then, the closing write on the factory administrator.
6. The independent re-read from which `BmcSecure` is derived.

The convergence refuses at each step that is not yet grounded. It never widens.

#### `BmcSecure` as a desired state (O1b)

Today `derive_bmc_secure` is rotation-event shaped. A rejected bootstrap credential is `PreviouslyRotatedCredentialRequired`, an accepted one with no rotation is `CredentialRotationNotApplied`, and only `RotationApplied` with an advanced epoch can mint `BmcSecured`. **So a correctly secured controller cannot be observed as a Noop through the existing fold.** The phase is therefore decomposed in `gunbc.machine_intake_bmc_secure`, with its conjunction unchanged:

1. **Observe** the current managed-account and published-account state: the managed probe, plus the published probes joined to the channel census. This is independent of whether this run rotated anything.
2. **Noop only when the full existing conjunction already holds.**
3. Otherwise **Apply** a typed account action over the grounded route.
4. **Re-read independently** and derive `BmcSecure` from that readback.

**Secret order, structurally.** The new credential generation is generated and stored under the unit's `SecretRef`, and its **exact resolved version is fetched**, before the account write and before the managed probe. Later consumers materialize that same `SecretRef` generation through `gunbc.auth.secret_ref_credential` `fetch_secret_ref_credential`. The earlier table's separate "managed secret materialized" row after `BmcSecure` was wrong: the rotation path cannot first obtain the secret after it has supposedly completed.

Controls:
- already secured → Noop;
- managed accepted and factory accepted → not Noop;
- reflash-restored factory access → the same Apply and readback path, with no second recovery authority.

**Ruling on mtcollins1's standing (eager-gull-22, 2026-10-04): staged, option C.**
- **Measured by O1b.** The committed `BmcSecured` standing is the 2026-09-13 observation on BMC 0.32. The 2026-10-02 reflash to 0.45.3 changed the subject and re-enabled the factory `admin`. No post-reflash readback is committed. So the honest standing for the current subject is not secured, and landing that alone would empty `managed_hosts()` and stop boot, reset, hold and federation admission.
- **O1b lands in shadow.** Boot is gap-intolerant, so O1b lands the state-shaped derivation, the carriers, the clock and the controls, with mtcollins1's live observation and the rotation-event path untouched and frozen.
- **One later transition** lands the honest re-expression and deletes the old path, **together with** a committed read-only post-reflash probe and the operator-approved wet `BmcSecure` Apply on mtcollins1.
- **Declared drop.** Until then, main's stale claim is a declared drop under `gunbc.rung_drop`, whose trigger is that capability.
- **What the state-shaped fold refuses** (side-chat reviews of #13221). The firmware build that admits a route is read from the observation, never supplied. The published account's replacement is the goal's exact break-glass generation, checked before the write, in the assessment and in the post-read. An unaddressed LAN channel, or channel access not closed, is refused with a typed cause naming the missing operation; it is not planned as a password write.
- **`rotation_route_consumption_frontier` stays open** until the planner is on a production path. This shadow cut supplies the consumer's code, not its production route.
- **Order on the hardware:** the operator's DIMM change, then cut 2 (#13131) landing, then the probe and the wet `BmcSecure`, then the census boot the runner lane needs, on a secured BMC.

**mtcollins1 is not assumed satisfied.** This withdraws the earlier sentence that it "enters at its satisfied state and every step is a Noop". After the 2026-10-02 reflash the `gunbc` user and IPMI admin were restored by hand and the factory `admin` was not disabled (eager-gull-22, 2026-10-03). The readback is therefore expected to find managed accepted and factory accepted: not Noop.

**Firmware re-entry.** `gunbc.fleet.mtcollins_firmware_converge` `BmcCredentialReestablishment.ipmi` is today a supplied `BmcSecureStanding`. It becomes this phase's result, for the post-reflash subject.

#### Managed-host admission: a durable, named population (O1c)

`managed_hosts()` is today a source roster with one construction-confined row. A runtime convergence cannot add a row, and the earlier sentence "mtjade1 joins by converging, not by edit" is withdrawn. Membership needs a durable authority, and the repository already has the mechanism: an intake fact is a **git-tracked observation consumed at mint time**. So:

- **`ManagedHostAdmission`**: one committed record per unit. It is authored from that unit's arrival receipts in the unit's own observation module, the way the mtcollins1 observations are today. It retains:
  - `HostIdentity` ↔ the exact `MachineIntakeSubject`;
  - the access and capability standing;
  - the `BmcSecureStanding`;
  - the unit-hold decision;
  - the receipt identity and its currency.
- **`managed_hosts()` becomes a projection** over the admission records. A unit is a member only when its record is complete, its standings are bound to the same subject, and that subject is current.
- **Absent, not defaulted.** A secured unit with no hold decision, with no host allocation, or with a receipt for another subject, is absent.
- The boot and reset consumers enumerate that same population (they already read `managed_hosts()`).
- `managed_host_binding` stays what it is: a pure join over a row. It is not the membership event.

So mtjade1 joins when its receipts are committed and its admission record is authored, which is an edit of evidence and never of the roster's logic.

#### Receipts and clock: domain carriers, not `gunbc.ensure` alone (O1b, O1c)

`gunbc.ensure` says of itself that its before and after observations and its satisfaction Booleans are caller-supplied, that one observation can be passed twice, and that it does not prove the decided plan is the effect that ran. So "ensure-style" is the decision shape only. For every effectful phase, the effect authority mints a **construction-confined domain carrier** that binds:

`subject + phase + admitted plan + application receipt + independent post-read + evidence`

The dry realization over `gunbc.bmc_model` `BmcWorld` exercises those carriers, not only the generic Noop / Apply / Refuse arms.

**Observer clock.** `bmc_secure` carries bare `EpochMs`, so a comment cannot make a controller-sourced time unconstructible. O1b introduces an observer-clock timestamp whose only producer is `gunbc.clock_read`, and every ordering field in the phase consumes it. The controller's year-2000 clock is preserved as an `AccessDiscover` observation and can never be the ordering clock. Control: a controller-sourced year-2000 timestamp cannot satisfy the rotation and readback ordering, and an observer timestamp can. Setting the BMC clock is a write and is out of scope.

#### The convergence fold: one shared home, landed with its first consumer (O1c-1)

**Why this is here** (eager-gull-22, 2026-10-03). The Spark serving bring-up (valiant-crab-775) needs the same thing cut O does: several effectful steps run as one convergence instead of separately dispatched modes that a human sequences. The operator pointed that lane at this one. The home lands **once**, with cut O and the Spark arm as its consumers. It does not go inside `gunbc.machine_intake_arrival_converge`.

**What already exists (DFS before naming anything).**
- **The per-step lifecycle has a consumed home.** `gunbc.host_convergence_protocol` is the canonical protocol from the CONVERGENCE-ONE program (`docs/plans/convergence-one-program.md`): select, observe goal-blind, assess, plan, admit, apply, independently read back, terminal verdict.
  - `HostEffectDomainProjection { observe, assess, decide }`, `inspect_via_host_effect_projection` and `reconcile_from_read_attempt` are the step.
  - `gunbc.ensure` is a projection of it.
  - That program's own rule: "anything that calls itself convergence without running that lifecycle is a second convergence algebra".
- **The roster of persistent host effects has a home.** `gunbc.host_convergence_census` `host_convergence_census_rows`. Each arrival phase's effect, and each Spark step, is a row there. It is not a parallel list.
- **The sealed receipt is the domain's.** `std.realization_reconcile` states that std cannot seal the relation decided plan ↔ performed plan ↔ evidence without accepting a law from the caller it would be checking, and that the binding "is the DOMAIN's obligation, discharged by a domain carrier minted only by the authority that performed the effect" (worked example: `gunbc.typed_remote_file_write` `RemoteFileConverged`). So there is **no generic sealed receipt** to build. O1b's `BmcSecure` receipt and Spark's receipts stay domain carriers.

**What is still open in that home, stated honestly** (side-chat review of `5044d42`). Ordered composition is not the only missing piece.
- **The protocol's apply side is an awaiting frontier.** `host_effect_decide_apply_frontier` is `FrontierConsumerAwaiting`: `converge_apply_for_host` bypasses `HostEffectDomainProjection` and calls `host_effect_apply` directly. That is CONVERGENCE-ONE's unbuilt C8.
- **`gunbc.ensure` `ensure_reconcile` does not execute the lifecycle.** It classifies reports.

**The boundary this plan takes: (B).** The fold **composes already-executed, domain-owned step outcomes**. It does not execute the protocol and does not claim to.
- Each domain runs its own step (observe, decide, apply, read back) and mints its own sealed receipt.
- The fold orders step instances, checks preconditions against those receipts, and produces the run's ledger.
- **C8 stays open.** It is discharged only by a production route through the projection's apply side. Nothing in cut O claims to discharge it. If an O1c PR does route a real apply through the projection, it says so and retires the frontier by its own trigger.

It goes in the workflow layer, `gunbc.fleet`. It does not go in std, because lanes, principals and resumability are fleet policy. The module is named by DFS when built.

**The runtime steps are joined to the census, so the census stays the roster.**
- `StepInstanceKey { run, step_identity, subject }` identifies a step instance.
- Every runtime step maps to **exactly one** `HostConvergenceCensusRow`.
- A missing, extra, duplicate or unresolved identity refuses.
- A duplicate `(run, step, subject)` also refuses.

Without that identity join the runtime list would become the real roster and the census would be commentary.

**Its shape, tested against two real consumers:** cut O's five arrival phases, and the Spark arm's steps S1–S9 (image produce and distribute, checkpoint seed and replica, sudo grants, claim recovery, supersede, launch). valiant-crab-775 reviewed the shape on 2026-10-04, and its six corrections are in the table.

| Requirement | Shape | Driven by |
|---|---|---|
| A step | an identity; a `HostEffectDomainProjection`; its domain receipt type; its preconditions. **A step instance is step × subject**: the Spark steps are per (host, artifact), the arrival steps per unit. | both |
| Preconditions | step identities **with a quantifier over subjects**: all-of or any-of, each over a stated subject set. So "3 of 4 hosts converged" is representable as partial progress and is not read as a refusal. | Spark S9 (S3, S5, S6 on all hosts); S5 (S4 verified on some peer) |
| Order | derived from preconditions by an identity join (an unknown identity or a cycle refuses). **Where an order authority already exists, each step is bound to one member of it and the derived order must agree with that authority's ranking, or refuse.** For cut O, each step is bound to one `IntakePhase` and checked with `intake_phase_rank`. The roster is a projection of the authoritative order (2 phases in O1c-1, 4 in O1, 5 with O2), not the 12-phase list itself. | arrival (authority), Spark (derived) |
| Per step | the canonical protocol, unchanged: Noop / Apply / Refuse, independent readback, the domain receipt minted by the effect authority | both |
| Precondition satisfied | only by a **readback in this run** of the precondition instance: a receipt for the same subject. Never by an earlier step's return value. **Which readback is sufficient is the domain's declaration**, and the fold does not force the most expensive one. A domain may declare a cheap identity readback that binds to an earlier full-verification receipt, and it must then state what that cheap readback does and does not establish (§4b rung honesty). | both; Spark S5, whose full verify re-hashes the whole checkpoint on every host |
| Refusal | names the unmet step by identity. The fold may take that step **only if it is in the roster and ungated**. A gated step stops with a typed refusal naming the discharge it needs. | Spark S9 naming S6/S7; the arrival credential write |
| Gate | a step is ungated, or gated on an authorization discharged for that subject and that run. The pattern comes from `select_authorization_pattern`, selected **before** the fold (§3d: the fold chooses nothing). | arrival `BmcSecure`; Spark S7 |
| Principal | per step, from the same selection. A step whose principal is not bound refuses. | Spark S6/S7 vs S9 |
| No fallback arms | a step with no admissible realization refuses; it never widens (§5) | Spark S5: no verified peer, no Hub fallback |
| Uncertain completion | an applied step whose readback does not yet ground is **pending**, distinct from converged and from refused. A rerun re-observes and reattaches. A pending instance never satisfies a precondition. **Only its dependents wait.** Independent instances proceed in the same run; otherwise every run degenerates to one step. | Spark S2/S4 (hours, resumable) while S3 and S6 proceed; arrival archive |
| Deadline | every effect leg carries a deadline. A leg with none is unconstructible. | Spark's ssh leg that hangs instead of refusing |
| Lane | the set of hosts a run may mutate is derived from its step instances' **mutated subjects**, one lane per affected host. **The executor host that dispatches the run is not a lane key.** | Spark, where every mode dispatches from srv1 and collides there; the arrival unit hold |
| Verdict | one ledger per run: each step instance converged, pending, refused, or not reached. **A refusal stops the line. A pending instance does not.** | both |
| Selected before the fold | everything that is a choice: the route, the authorization pattern, the principal, and **roles** such as seed versus replica. A role derived inside a step is how a silent fallback happens. | arrival route (O1a); Spark S4/S5 roles |
| Not a step | a **measurement** is not a persistent host effect. It consumes a converged verdict and produces an observation receipt, outside the fold and outside the census. Otherwise "converged" comes to mean "benchmarked". | Spark S10 (load) |

**Workflow surface.** Lane derivation changes concurrency groups in the generated workflow, which is an operator-visible surface. Cut O's use changes none: it runs inside the existing `mtcollins1_boot` job and its unit hold. The Spark arm's reshaping of its modes is that lane's proposal to the operator, not part of this cut.

**§3c, and the pairing obligation: each capability lands with its first REAL consumer.** A fold with no consumer is a dangling declaration, and a capability exercised only by a designed fixture has no inhabitance claim (`DESIGN.md` §3, "a witness discriminates at one interface"). So the fold is not built whole in one PR. Each row of the shape table arrives in the PR whose consumer really exercises it:

| PR | Real consumer | Capabilities it brings |
|---|---|---|
| **O1c-1** | the arrival prefix's two **read-only** phases, `AccessDiscover` (with the O1a route standing) and `IdentityBindProvisional` | the read-only scheduler core: `StepInstanceKey`; the census join and its refusals; step-to-`IntakePhase` binding and the rank check; preconditions satisfied by an in-run readback receipt; refusal naming the unmet step; the run ledger. **No apply, gate, lane, deadline or pending arm.** |
| **O1c-2** | `PriorLifeBoundary`'s archive (an effect that writes evidence and can be incomplete) | an applied step with its domain receipt; per-leg deadlines; incomplete-versus-refused |
| **O1c-3** | `BmcSecure` (O1b's receipt) and the `ManagedHostAdmission` population | the authorization gate and its discharge; principal per step; the mutation lane (the unit hold on the store host); apply followed by independent readback |
| **Spark arm** (valiant-crab-775's lane) | S1–S9 over four hosts | subject quantifiers (all-of, any-of); pending that blocks only dependents; the domain-declared cheap readback; multi-host lanes keyed on mutated subjects |

**Controls land with the capability.**
- O1c-1's are:
  - a step whose phase rank disagrees with the authority refuses;
  - an unknown precondition refuses;
  - a runtime step with no census row, or two rows, refuses;
  - a duplicate `(run, step, subject)` refuses;
  - a precondition is not satisfied by a receipt for another subject.
- The quantifier, pending and lane controls arrive with the consumer that inhabits them, not earlier with a fixture.

valiant-crab-775 reviews O1c-1's shape before it lands. `DESIGN.md` §3d names the fleet admission spine as this cycle's first consumer, so the shape must not preclude it, and O1c-1's PR says how it fits.

**What O1b changes.** Nothing in its scope. Its observe, assess and decide go through the canonical projection (`HostEffectDomainProjection`), and its effect is a census row. Its apply and readback are the domain's own, with its own sealed receipt, because the projection's apply side is the open C8 frontier.

#### Authorization and what lands

- **The rotation is a privileged effect.** It is classified by executing `gunbc.auth.authorization_pattern_selection` `select_authorization_pattern` over its real attributes, and it gets its row in `gunbc.auth.privileged_effect_census`. The pattern is what the selection returns.
- **The write is gated inside the single route.** The `BmcSecure` Apply arm refuses unless that authorization is discharged for that unit and that run. A run that finds the unit unsecured with no discharge stops with a typed refusal at `BmcSecure`. It never boots on a factory credential and never widens.
- **O2 preserves every refusal** the current boot entry makes (admission, authorization, maintenance hold, artifact, medium readback). Each is listed and witnessed in that PR.
- **The O1 PRs land** the route standing, the state-shaped `BmcSecure`, the observer clock, the convergence through `BmcSecure`, the admission population, and the dry realization with the controls above. **No live credential write** is part of any PR; each unit's write gets its own operator go-ahead, mtcollins1 first.
- **Password-path frontier.** The remaining wrappers (media attach, KVM, SOL, fan, UI bundle) cut over in cuts 3, 4a, 4d and 5. `mtcollins1_managed_secret_fetch_frontier` is retired when the last one refuses a caller-supplied path.
- **Workflow surface:** none added. O2's generated-workflow diff is shown in its PR.

#### Decisions (eager-gull-22 for the operator, 2026-10-03)

- **(i) Module name:** `gunbc.machine_intake_arrival_converge`. Approved.
- **(ii) Read-only reads of mtjade1 with the factory credential:** no separate sign-off; the operator approved hands-on access to the unit on 2026-10-03. **The rotation and any account write do need sign-off.**
- **(iii) First wet subject:** mtcollins1, with its unsecured state treated as likely real.
- **(iv) Workflow mode:** approved as a **re-root of the existing `mtcollins1_boot` mode**, not a new mode. The live credential write on each unit still gets its own go-ahead.

### Terminal receipt owed by every cut (review item 5)

These modules are outside the required gate, so a green required run proves nothing about them. Each PR's description carries all of the following:

1. **Consumer witnesses, run by name.** The cut's complete consumer roster, recomputed at the PR head with the census method below and included in full in the PR description. Each named witness is executed with `gunbc run` against the head, with binary path and build sha printed.
2. **A same-path RED.** On the acceptance path the cut's witness actually runs, remove the binding the cut introduced (the `ManagedHost` row, the standing arm, the parameter) and show the named witness refusing. Then restore it and show it accepted. A RED reached by a different route does not count.
3. **A corpus sweep.** `git grep` at the head for every deleted module name and symbol, across `.dag`, `.rs`, `.yml`, `docs/` and `artifacts/`. Every remaining hit is classified with the dispositions in the non-import census below.
4. **No aliases.** No module re-exports, wraps or renames-through the deleted root, and no `mtcollins1_*` function is a one-line call into the generic one.

## Non-import census (review item 4)

These occurrences do not refuse through an import edge, so the import census above cannot see them. They are measured at `26e99a9c7f` and dispositioned by surface. **migrate** means the owning cut changes it. **receipt** means it is deliberately unit-named and stays. **residue** means it is historical prose and stays as written.

| Surface | Population | Disposition |
|---|---|---|
| `.github/workflows/fleet-converge.yml` (projection of `gunbc.fleet_converge_workflow`) | 118 occurrences: 8 mode literals and about 30 step/job ids | **migrate**, cut 7 only, regenerated from the authority. Never hand-edited. |
| `.github/workflows/mtcollins-canary.yml` | a hand-authored scaffold by its own header; `runs-on: [self-hosted, mtcollins]` | **migrate or retire** in cut 7, and only with the operator's ruling. The runner label is a live registration (`gunbc.runner_registration_labels`). |
| `"mtcollins1_*"` string literals in `.dag` (mode names, step ids, effect rows) | 34 files. The largest are `gunbc.fleet_converge_workflow` (26), `gunbc.auth.privileged_effect_census` (11), `gunbc.ci_spec` (11), `mtcollins1_boot_run` (10), `mtcollins1_memory_census_observation` (10) | **migrate** in the cut that owns the named step. The privileged-effect rows follow cut 4c and 7. Unit-observation literals are **receipts**. |
| Entry-path strings `"dag/gunbc/machine_intake/mtcollins…"` in `.dag` | `gunbc.non_fold_residue` (45), `gunbc.ci_spec` (11), `v2.workflow.floor_unimported_bare_provider_debt_roster` (4), and 5 single occurrences | **migrate**: every cut that moves a file updates these rows in the same PR. Each is a path a deletion does not refuse. |
| `artifacts/bmc/mtcollins1-*` (17 files) | captured evidence; paths cited by unit observations | **receipt**: never renamed, because the content hashes bind the path. |
| `docs/probes/mtcollins*` (12), `docs/rung-drops/*` (1), `docs/design-rung-drops.md` | dated observations and declared drops | **receipt**. |
| `docs/plans/*` (14 files other than this one), `docs/recovered/*`, `ROADMAP.md` (2) | dated plans; the roadmap goal "Bring Mt. Collins unit 1 into service" is correctly unit-named | **residue / receipt**. Not edited by these cuts. |
| `src/v1/stage0/src/cli_run.rs` (1) | a comment | **residue**. |
| The IAM condition description string beginning `gunbc.auth.mtcollins1_boot_federation: the one version …` (kept by cut 4c in `gunbc.auth.managed_host_boot_federation`) | part of a **live** IAM binding condition | **receipt**: it must stay byte-identical until a deliberate re-provision. A name sweep must not "fix" it, although it names a deleted module. |

Exact token `mtcollins1_boot`: 19 files at `26e99a9c7f`, all inside the populations above.

## Census method: a one-off, re-run by each cut (not an instrument)

No entry point in the tree derives an import-graph consumer roster, and building one is outside this program. So the census is a **one-off measurement** (DESIGN §6), and no snapshot of its output is the authority for anything.

**Each cut re-runs it at its own PR head** over its own deleted roots, and puts the **complete** consumer roster in that PR's description, bound to that head sha. That roster is the §3 "enumerate the consumers by name before you delete" receipt for the cut, and it lives with the deletion it licenses, where it cannot go stale.

The method:
1. Walk `dag/` and `src/v2/` for `.dag` files.
2. Read each `^module` and each `^import <name>`.
3. The population is the modules whose name or text matches `mtcollins` (case-insensitive).
4. Consumers are the reverse import edges.
5. Gate membership is the transitive import closure from the modules matching `v2.workflow.required_floor` `required_gate_prefixes`.
6. Imported unit symbols are the names inside each module's `import … { … }` lists that match `mtcollins|mt_collins|MtCollins`.

It is a regex over source text, so it censuses *import shapes*. The terminal receipt's corpus sweep and the non-import census above cover what an import edge cannot see.

## Census

The layer is assigned per module from its subject.
- **R** = mentions the unit only in prose or receipts.
- **W** = witness.

Consumers list production importers by name, truncated after six with a count, plus the number of witness importers. **This table is a dated reading aid at `26e99a9c7f`, used to plan the order. It is not a consumer receipt: each cut recomputes its own roster at its head (see Census method).**

### Layer 3 — silicon (3 modules)

| module | lines | refs | unit/platform constants imported | consumers (import census) | gate |
|---|---|---|---|---|---|
| `gunbc.machine_intake_ampere_dram_console_observation` | 314 | 2 | — | `gunbc.machine_intake_ampere_socket_console_observation`, `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle` · 1 witness | no |
| `gunbc.machine_intake_ampere_socket_console_observation` | 503 | 2 | — | `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle` · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_smpro_observation` | 395 | 36 | `mtcollins1_endpoint` | `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle`, `gunbc.machine_intake_mtcollins1_boot_phase_timing`, `gunbc.machine_intake_mtcollins1_boot_run` · 3 witness | no |

### Layer 5 — fleet procedure (53 modules)

| module | lines | refs | unit/platform constants imported | consumers (import census) | gate |
|---|---|---|---|---|---|
| `extdeps.provisioning.ubuntu_seeded_install_media` | 196 | 1 | — | `extdeps.provisioning.ubuntu_seeded_install_media_remaster`, `gunbc.machine_intake_megarac_media_attach`, `gunbc.machine_intake_mtcollins1_boot_authorization`, `gunbc.machine_intake_mtcollins1_census_image`, `gunbc.machine_intake_mtcollins1_census_image_publish`, `gunbc.machine_intake_mtcollins1_census_medium_readback` +4 more · 7 witness | no |
| `extdeps.tools.xorriso` | 151 | 1 | — | `extdeps.provisioning.ubuntu_seeded_install_media_remaster`, `extdeps.provisioning.ubuntu_seeded_install_media_toolchain`, `gunbc.host_effect_realize`, `gunbc.machine_intake_mtcollins1_census_member_readback` · 2 witness | no |
| `gunbc.auth.fleet_secret_accessor_roster` | 95 | 6 | `mtcollins1_bmc_gunbc_secret_ref` | `gunbc.auth.approval_mac_key_provision`, `gunbc.spark.secret_access_ensure` · 1 witness | no |
| `gunbc.auth.gcp_iam_converge` | 893 | 4 | `mtcollins1_boot_dedicated_federation` | `gunbc.auth.gcp_iam_converge_run`, `gunbc.fleet_converge_workflow` · 1 witness | no |
| `gunbc.auth.gcp_iam_converge_federation` | 67 | 1 | — | `gunbc.auth.gcp_iam_converge`, `gunbc.fleet_converge_workflow` | no |
| `gunbc.auth.heal_publisher_provision` | 500 | 1 | — | `gunbc.auth.gcp_iam_converge`, `gunbc.auth.mtcollins1_boot_federation_provision`, `gunbc.namecheap.federation_provision` · 2 witness | no |
| `gunbc.auth.mtcollins1_boot_federation` | 157 | 43 | `mtcollins1_bmc_gunbc_secret_ref` | `gunbc.auth.gcp_iam_converge`, `gunbc.auth.mtcollins1_boot_federation_provision`, `gunbc.auth.privileged_effect_census`, `gunbc.fleet_converge_workflow` · 1 witness | no |
| `gunbc.auth.mtcollins1_boot_federation_provision` | 39 | 28 | `mtcollins1_boot_secret_grants`, `mtcollins1_boot_service_account`, `mtcollins1_boot_service_account_display_name`, `mtcollins1_boot_service_account_id`, `mtcollins1_boot_service_account_member` … | `gunbc.auth.gcp_iam_converge` · 1 witness | no |
| `gunbc.auth.privileged_effect_census` | 1060 | 54 | `mtcollins1_boot_service_account_member` | `gunbc.census_closure_frontier` · 3 witness | no |
| `gunbc.fleet_converge_workflow` | 4446 | 271 | `gunbc_ci_mtcollins1_boot_admit_invoke`, `gunbc_ci_mtcollins1_boot_invoke`, `gunbc_ci_mtcollins1_boot_job_backstop_timeout_minutes`, `gunbc_ci_mtcollins1_census_image_publish_invoke`, `gunbc_ci_mtcollins1_census_image_step_timeout_minutes` … | `gunbc.generated_artifact_emit` · 11 witness | no |
| `gunbc.fleet_workflow_steps` | 929 | 4 | — | `gunbc.fleet_converge_workflow` · 2 witness | no |
| `gunbc.host_reset_return` | 1034 | 2 | — | `gunbc.ci_spec`, `gunbc.fleet_converge_workflow`, `gunbc.host_reset_return_run` · 2 witness | no |
| `gunbc.host_reset_return_run` | 787 | 3 | — | `gunbc.ci_spec`, `gunbc.fleet_converge_workflow`, `gunbc.machine_intake_mtcollins1_boot_run` · 1 witness | no |
| `gunbc.host_reset_subject_roster` | 140 | 22 | `mtcollins1_access_observation`, `mtcollins1_bmc_username`, `mtcollins1_capability_row`, `mtcollins1_cdrom_selection`, `mtcollins1_unit_hold_key` … | `gunbc.fleet_converge_workflow`, `gunbc.host_reset_return_run` | no |
| `gunbc.hostname_allocation` | 111 | 3 | `operator_host_mtcollins1` | `gunbc.deployed_intent_v1`, `gunbc.host_reset_return_run`, `gunbc.network_identity_subsumption` | no |
| `gunbc.machine_intake_host_capture_envelope` | 749 | 5 | — | `gunbc.machine_intake_host_capture_historical_binding`, `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle`, `gunbc.machine_intake_mtcollins1_boot_milestone`, `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_census_image` · 6 witness | no |
| `gunbc.machine_intake_host_capture_historical_binding` | 348 | 14 | `mtcollins1_census_stages`, `mtcollins1_host_capture_artifact_path`, `mtcollins1_host_capture_byte_count`, `mtcollins1_host_capture_digest` | — · 2 witness | no |
| `gunbc.machine_intake_host_resource_observation` | 403 | 4 | — | `gunbc.machine_intake_mtcollins1_topology_goal` · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_actuate` | 196 | 41 | `MtCollins1BootStart`, `mtcollins1_access_observation`, `mtcollins1_boot_start_of`, `mtcollins1_capability_row`, `mtcollins1_secured_account` … | `gunbc.host_reset_subject_roster`, `gunbc.machine_intake_mtcollins1_boot_authorization`, `gunbc.machine_intake_mtcollins1_boot_run` · 3 witness | no |
| `gunbc.machine_intake_mtcollins1_boot_admission` | 52 | 15 | — | `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_kvm_observer_observe` · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_boot_artifact` | 58 | 5 | — | `gunbc.machine_intake_mtcollins1_boot_authorization`, `gunbc.machine_intake_mtcollins1_boot_image_fetch`, `gunbc.machine_intake_mtcollins1_census_image`, `gunbc.machine_intake_mtcollins1_census_image_publish`, `gunbc.machine_intake_mtcollins1_census_medium_readback`, `gunbc.machine_intake_mtcollins1_census_member_readback` +1 more · 2 witness | no |
| `gunbc.machine_intake_mtcollins1_boot_authorization` | 105 | 61 | `mtcollins1_boot_export_dir`, `mtcollins1_boot_image_sha256`, `mtcollins1_cdrom_selection`, `mtcollins1_census_image_stem`, `mtcollins1_census_volume_id` … | `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_census_medium_readback`, `gunbc.machine_intake_mtcollins1_census_member_readback`, `gunbc.machine_intake_mtcollins1_media_attach` · 9 witness | no |
| `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle` | 2204 | 214 | `MtCollins1BootPhaseTiming`, `mtcollins1_boot_ipmi_kill_after`, `mtcollins1_boot_ipmi_read_deadline`, `mtcollins1_boot_phase_timing_not_taken`, `mtcollins1_boot_phase_timing_text` … | `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_media_attach` · 8 witness | no |
| `gunbc.machine_intake_mtcollins1_boot_dry_realization` | 730 | 100 | `mtcollins1_sol_notice_ready_suffix`, `mtcollins1_sol_notice_token_env`, `mtcollins1_sol_notice_watcher_path` | — · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_boot_image_fetch` | 119 | 37 | `mtcollins1_boot_export_dir`, `mtcollins1_diskless_image_name` | `gunbc.machine_intake_mtcollins1_boot_authorization` | no |
| `gunbc.machine_intake_mtcollins1_boot_milestone` | 133 | 28 | `mtcollins1_nproc` | `gunbc.machine_intake_host_capture_historical_binding`, `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_census_image` · 4 witness | no |
| `gunbc.machine_intake_mtcollins1_boot_phase_timing` | 255 | 92 | — | `gunbc.machine_intake_mtcollins1_actuate`, `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle`, `gunbc.machine_intake_mtcollins1_boot_run` · 2 witness | no |
| `gunbc.machine_intake_mtcollins1_boot_run` | 2734 | 571 | `MtCollins1BootDiagnosticBundle`, `MtCollins1BootMedium`, `MtCollins1BootPhaseTiming`, `MtCollins1BootRoute`, `MtCollins1BootRunRecord` … | `gunbc.ci_spec`, `gunbc.fleet_converge_workflow`, `gunbc.machine_intake_mtcollins1_boot_dry_realization`, `gunbc.machine_intake_mtcollins1_sol_notice` · 8 witness | no |
| `gunbc.machine_intake_mtcollins1_census_image` | 220 | 76 | `mtcollins1_boot_export_dir`, `mtcollins1_census_stages`, `mtcollins1_diskless_image_name` | `gunbc.machine_intake_mtcollins1_boot_authorization`, `gunbc.machine_intake_mtcollins1_census_image_publish`, `gunbc.machine_intake_mtcollins1_census_medium_readback`, `gunbc.machine_intake_mtcollins1_census_member_readback`, `gunbc.machine_intake_mtcollins1_media_attach` · 9 witness | no |
| `gunbc.machine_intake_mtcollins1_census_image_publish` | 267 | 46 | `mtcollins1_boot_export_dir`, `mtcollins1_boot_export_host_target`, `mtcollins1_census_image`, `mtcollins1_census_image_stock_path`, `mtcollins1_census_pinned_image` | `gunbc.ci_spec`, `gunbc.fleet_converge_workflow` · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_census_medium_readback` | 260 | 74 | `MtCollins1BootMedium`, `MtCollins1CensusMedium`, `MtCollins1StockInstallerMedium`, `mtcollins1_boot_export_dir`, `mtcollins1_boot_export_host_target` … | `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_census_member_readback` · 3 witness | no |
| `gunbc.machine_intake_mtcollins1_census_member_readback` | 483 | 34 | `MtCollins1BootMedium`, `MtCollins1CensusMedium`, `MtCollins1StockInstallerMedium`, `mtcollins1_boot_export_host_target`, `mtcollins1_boot_medium` … | `gunbc.ci_spec`, `gunbc.fleet_converge_workflow`, `gunbc.machine_intake_mtcollins1_census_image_publish` · 2 witness | no |
| `gunbc.machine_intake_mtcollins1_census_qemu_host_observe` | 294 | 11 | — | `gunbc.ci_spec`, `gunbc.fleet_converge_workflow`, `gunbc.machine_intake_mtcollins1_census_qemu_toolchain_converge` · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_census_qemu_toolchain` | 172 | 1 | — | `gunbc.machine_intake_mtcollins1_census_qemu_host_observe`, `gunbc.machine_intake_mtcollins1_census_qemu_toolchain_converge` · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_census_qemu_toolchain_converge` | 41 | 8 | — | — | no |
| `gunbc.machine_intake_mtcollins1_census_refusal_hypotheses` | 356 | 13 | — | — · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_diskless_boot_receipt` | 144 | 12 | — | — | no |
| `gunbc.machine_intake_mtcollins1_handoff_probe` | 63 | 14 | `mtcollins1_access_observation`, `mtcollins1_capability_row`, `mtcollins1_firmware` | — | no |
| `gunbc.machine_intake_mtcollins1_maintenance_hold` | 763 | 58 | `operator_host_mtcollins1` | `gunbc.fleet_converge_workflow`, `gunbc.host_reset_return_run`, `gunbc.host_reset_subject_roster`, `gunbc.machine_intake_mtcollins1_actuate`, `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_kvm_observer_observe` +2 more · 5 witness | no |
| `gunbc.machine_intake_mtcollins1_media_attach` | 427 | 111 | `MtCollins1BootSubject`, `MtCollins1MediaAttachRecord`, `mtcollins1_boot_chassis_power`, `mtcollins1_boot_export_dir`, `mtcollins1_boot_ipmi_kill_after` … | `gunbc.machine_intake_mtcollins1_boot_run` · 4 witness | no |
| `gunbc.machine_intake_mtcollins1_memory_census_observation` | 916 | 172 | `MtCollins1ControllerFruIdentity`, `mtcollins1_controller_fru_identity` | `gunbc.machine_intake_host_capture_historical_binding`, `gunbc.machine_intake_mtcollins1_bmc_secure_observation`, `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle`, `gunbc.machine_intake_mtcollins1_boot_milestone`, `gunbc.machine_intake_mtcollins1_topology_goal` · 3 witness | no |
| `gunbc.machine_intake_mtcollins1_netboot_client_observation` | 73 | 11 | — | `gunbc.machine_intake_pre_os_capture_replay`, `gunbc.machine_intake_pre_os_observer`, `gunbc.site_uefi_arm64_pxe_edge` · 6 witness | no |
| `gunbc.machine_intake_oob_boot_handoff` | 542 | 2 | — | `gunbc.host_reset_return_run`, `gunbc.host_reset_subject_roster`, `gunbc.machine_intake_mtcollins1_actuate`, `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle`, `gunbc.machine_intake_mtcollins1_boot_phase_timing`, `gunbc.machine_intake_mtcollins1_boot_run` +1 more · 5 witness | no |
| `gunbc.machine_intake_pre_os_bringup_attempt` | 370 | 2 | — | `gunbc.machine_intake_mtcollins1_32dimm_bringup_observation`, `gunbc.machine_intake_mtcollins1_sixteen_module_restore_observation` · 1 witness | no |
| `gunbc.machine_intake_pre_os_bringup_verdict` | 633 | 1 | — | `gunbc.machine_intake_mtcollins1_32dimm_bringup_observation`, `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle`, `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_sixteen_module_restore_observation`, `gunbc.machine_intake_pre_os_capture_replay`, `gunbc.machine_intake_pre_os_observer` +2 more · 6 witness | no |
| `gunbc.machine_intake_pre_os_capture_replay` | 175 | 27 | `mt_collins_channel_population_roles`, `mtcollins1_first_own_image_boot`, `mtcollins1_host_nic_mac`, `mtcollins_3ds_capture_path`, `mtcollins_3ds_capture_sha256` … | — · 2 witness | no |
| `gunbc.machine_intake_pre_os_observer` | 139 | 1 | — | `gunbc.machine_intake_pre_os_capture_replay`, `gunbc.machine_intake_pre_os_inventory_assessment`, `gunbc.machine_intake_pre_os_timeline` · 3 witness | no |
| `gunbc.machine_intake_pre_os_timeline` | 392 | 3 | `mtcollins_dram_cycle_start` | — · 1 witness | no |
| `gunbc.machine_intake_sel_stimulus_listener` | 124 | 2 | — | `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle`, `gunbc.machine_intake_pre_os_capture_replay`, `gunbc.machine_intake_pre_os_replay` · 2 witness | no |
| `gunbc.machine_intake_terminal_transcript` | 18 | 2 | — | `gunbc.machine_intake_host_capture_envelope`, `gunbc.machine_intake_pre_os_capture_replay`, `gunbc.machine_intake_pre_os_observer` · 1 witness | no |
| `gunbc.seeded_install_media_policy` | 28 | 1 | — | `gunbc.machine_intake_mtcollins1_census_image`, `gunbc.srv3_seeded_install_media_artifact` | no |
| `gunbc.site_uefi_arm64_pxe_edge` | 339 | 7 | `mtcollins1_host_nic_mac` | `gunbc.census_closure_frontier`, `gunbc.site_pxe_edge_converge` · 2 witness | no |
| `gunbc.verified_archive_install` | 235 | 1 | — | `gunbc.machine_intake_mtcollins1_census_qemu_toolchain`, `gunbc.runner_browser_toolchain`, `gunbc.runner_microvm_host_ready` · 3 witness | no |

### Layer 4 — BMC stack (22 modules)

| module | lines | refs | unit/platform constants imported | consumers (import census) | gate |
|---|---|---|---|---|---|
| `extdeps.bmc.http` | 350 | 1 | — | `gunbc.fleet_health_observe`, `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle`, `gunbc.srv3_boot_once_cd`, `gunbc.tools.bmc_health_reader_converge`, `gunbc.tools.bmc_intake_first_contact`, `gunbc.tools.bmc_onboard` | no |
| `extdeps.bmc.ipmi` | 410 | 1 | — | `ctl.pos_emit`, `ctl.pos_service`, `gunbc.machine_intake_bmc_fan_observation`, `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle`, `gunbc.machine_intake_mtcollins1_boot_dry_realization`, `gunbc.machine_intake_mtcollins1_boot_run` +4 more · 1 witness | no |
| `extdeps.bmc.ipmi_sel` | 83 | 1 | — | `gunbc.machine_intake_mtcollins1_32dimm_bringup_observation`, `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle`, `gunbc.machine_intake_mtcollins1_sixteen_module_restore_observation`, `gunbc.machine_intake_pre_os_bringup_verdict`, `gunbc.machine_intake_sel_stimulus_listener` · 2 witness | no |
| `extdeps.bmc.ipmitool_observed_output` | 67 | 6 | — | `gunbc.bmc_dry_realization`, `gunbc.machine_intake_mtcollins1_boot_dry_realization` | no |
| `extdeps.bmc.megarac` | 664 | 5 | — | `extdeps.bmc.access_profile`, `extdeps.bmc.virtual_media`, `gunbc.bmc_implementation_dispatch`, `gunbc.bmc_megarac_web_adapter`, `gunbc.boot_artifact_delivery`, `gunbc.machine_intake_megarac_media_attach` +3 more · 7 witness | **yes** |
| `extdeps.bmc.megarac_observed_output` | 73 | 1 | — | `gunbc.bmc_dry_realization` | no |
| `extdeps.bmc.phosphor_fan_control` | 126 | 2 | — | — · 1 witness | no |
| `extdeps.bmc.redfish_memory_inventory` | 459 | 2 | — | `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle` · 1 witness | no |
| `gunbc.bmc_model` | 409 | 2 | — | `gunbc.bmc_dry_realization`, `gunbc.bmc_megarac_web_adapter`, `gunbc.machine_intake_mtcollins1_boot_dry_realization` · 5 witness | no |
| `gunbc.machine_intake_bmc_fan_observation` | 519 | 2 | — | `gunbc.machine_intake_mtcollins1_fan_observe` · 1 witness | no |
| `gunbc.machine_intake_bmc_rotation_route` | 117 | 1 | — | — · 1 witness | no |
| `gunbc.machine_intake_megarac_media_attach` | 2107 | 9 | — | `gunbc.bmc_dry_realization`, `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle`, `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_media_attach`, `gunbc.machine_intake_mtcollins1_ui_bundle_observe` · 5 witness | no |
| `gunbc.machine_intake_megarac_served_ui_catalog_observation` | 20 | 2 | — | `gunbc.machine_intake_megarac_ui_features`, `gunbc.machine_intake_mtcollins1_ui_bundle_observe` · 3 witness | no |
| `gunbc.machine_intake_megarac_ui_features` | 403 | 3 | — | `gunbc.machine_intake_megarac_media_attach`, `gunbc.machine_intake_mtcollins1_kvm_still` · 2 witness | no |
| `gunbc.machine_intake_mtcollins1_bmc_secure_observation` | 275 | 92 | `mtcollins1_bmc_gunbc_secret_ref`, `mtcollins1_controller_fru_identity`, `mtcollins1_endpoint`, `mtcollins1_firmware`, `mtcollins1_subject_binding` | `gunbc.machine_intake_mtcollins1_actuate`, `gunbc.machine_intake_mtcollins1_fan_observe`, `gunbc.machine_intake_mtcollins1_kvm_observer_observe`, `gunbc.machine_intake_mtcollins1_media_attach`, `gunbc.machine_intake_mtcollins1_platform_observation`, `gunbc.machine_intake_mtcollins1_ui_bundle_observe` · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_bmc_sensor_observation` | 96 | 25 | — | `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle` · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_fan_observe` | 264 | 39 | `mtcollins1_endpoint`, `mtcollins1_secured_account` | `gunbc.ci_spec`, `gunbc.fleet_converge_workflow`, `gunbc.machine_intake_mtcollins1_kvm_observer_observe`, `gunbc.machine_intake_mtcollins1_ui_bundle_observe` | no |
| `gunbc.machine_intake_mtcollins1_kvm_observer_observe` | 276 | 71 | `mtcollins1_bmc_firmware`, `mtcollins1_boot_host_admission`, `mtcollins1_endpoint`, `mtcollins1_fan_credential_path`, `mtcollins1_fan_credential_path_env` … | `gunbc.fleet_converge_workflow` · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_kvm_still` | 1180 | 43 | — | `gunbc.ci_spec`, `gunbc.fleet_converge_workflow`, `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle`, `gunbc.machine_intake_mtcollins1_boot_dry_realization`, `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_kvm_observer_observe` · 6 witness | no |
| `gunbc.machine_intake_mtcollins1_sol_notice` | 438 | 52 | `mtcollins1_boot_sol_capture_path_env`, `mtcollins1_boot_sol_pid_path_env`, `mtcollins1_sol_client_diagnostic_path`, `mtcollins1_sol_delivery_suffix`, `mtcollins1_sol_incident_path` … | `gunbc.ci_spec` · 2 witness | no |
| `gunbc.machine_intake_mtcollins1_ui_bundle_observe` | 364 | 61 | `mtcollins1_boot_ipmi_kill_after`, `mtcollins1_boot_ipmi_read_deadline`, `mtcollins1_endpoint`, `mtcollins1_fan_credential_path`, `mtcollins1_fan_credential_path_env` … | `gunbc.fleet_converge_workflow` · 1 witness | no |
| `gunbc.machine_intake_sol_hold` | 344 | 17 | — | `gunbc.ci_spec`, `gunbc.machine_intake_mtcollins1_boot_dry_realization`, `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_sol_notice`, `gunbc.owned_process` · 2 witness | no |

### Layer 2 — platform (16 modules)

| module | lines | refs | unit/platform constants imported | consumers (import census) | gate |
|---|---|---|---|---|---|
| `extdeps.ampere.mt_collins_1u_product_brief.specifications` | 228 | 23 | `MtCollins1UPublicProductBrief`, `mt_collins_1u_brief_authority`, `mt_collins_1u_public_product_brief` | `gunbc.machine_intake_mtcollins1_expansion_attachment`, `gunbc.machine_intake_mtcollins1_expansion_candidates`, `gunbc.machine_intake_mtcollins1_expansion_observation` · 2 witness | no |
| `extdeps.ampere.mt_collins_1u_product_brief.subject` | 102 | 8 | — | `extdeps.ampere.mt_collins_1u_product_brief.specifications` | no |
| `extdeps.ampere.mt_collins_2u_user_guide.subject` | 68 | 7 | — | `gunbc.mt_collins_dimm_physical_identity`, `gunbc.specification_citation_read_provenance` · 1 witness | no |
| `extdeps.ampere.mt_collins_getting_started_guide.dimm_layout` | 92 | 32 | `mt_collins_gsg_authority`, `mt_collins_gsg_model_scope` | `gunbc.machine_intake_mtcollins1_dimm_connector_observation`, `gunbc.machine_intake_mtcollins1_physical_orientation`, `gunbc.machine_intake_mtcollins1_socket1_investigation_observation`, `gunbc.mt_collins_dimm_physical_identity`, `gunbc.mtcollins_memory_placement` · 3 witness | no |
| `extdeps.ampere.mt_collins_getting_started_guide.subject` | 92 | 3 | — | `extdeps.ampere.mt_collins_getting_started_guide.dimm_layout`, `extdeps.ampere.mt_collins_product_brief.memory_population`, `extdeps.ampere.mt_collins_product_brief.platform` · 1 witness | no |
| `extdeps.ampere.mt_collins_product_brief.bmc` | 59 | 5 | `MtCollinsPublicProductBrief`, `mt_collins_public_product_brief` | — | no |
| `extdeps.ampere.mt_collins_product_brief.memory_population` | 387 | 85 | `MtCollinsPublicProductBrief`, `mt_collins_gsg_authority`, `mt_collins_public_product_brief` | `gunbc.machine_intake_mtcollins1_dimm_connector_observation`, `gunbc.machine_intake_mtcollins1_physical_orientation`, `gunbc.machine_intake_mtcollins1_spare_screen_prediction`, `gunbc.machine_intake_pre_os_capture_replay`, `gunbc.memory_configuration_evaluation`, `gunbc.mt_collins_dimm_physical_identity` +3 more · 9 witness | no |
| `extdeps.ampere.mt_collins_product_brief.platform` | 102 | 12 | `MtCollinsPublicProductBrief`, `mt_collins_gsg_authority`, `mt_collins_public_product_brief` | `gunbc.memory_provisioning` · 1 witness | no |
| `extdeps.ampere.mt_collins_product_brief.subject` | 77 | 8 | — | `extdeps.ampere.mt_collins_product_brief.bmc`, `extdeps.ampere.mt_collins_product_brief.memory_population`, `extdeps.ampere.mt_collins_product_brief.platform` · 1 witness | no |
| `extdeps.boards.types` | 89 | 1 | — | `extdeps.bmc.access_profile`, `extdeps.boards.asrock_rack`, `extdeps.boards.gigabyte`, `extdeps.cooling.dynatron`, `extdeps.cooling.types`, `extdeps.firmware.types` +13 more · 6 witness | **yes** |
| `gunbc.machine_intake_mtcollins1_dimm_connector_observation` | 105 | 22 | `MtCollinsDimmDiagramLegend`, `MtCollinsDimmPopulationIdentity`, `MtCollinsDimmPopulationRow`, `mt_collins_population_16_dimms` | `gunbc.machine_intake_mtcollins1_expansion_candidates`, `gunbc.machine_intake_mtcollins1_expansion_observation`, `gunbc.machine_intake_mtcollins1_physical_orientation` · 2 witness | no |
| `gunbc.machine_intake_mtcollins1_physical_orientation` | 856 | 293 | `MtCollins1ChassisEnd`, `MtCollins1CpuRelease`, `MtCollins1DiagnosisHypothesis`, `MtCollins1FirmwareSocketSummary`, `MtCollins1HypothesisStanding` … | — · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_platform_observation` | 225 | 33 | `mtcollins1_unit_key_standing` | — · 2 witness | no |
| `gunbc.machine_intake_mtcollins1_topology_goal` | 18 | 7 | `mtcollins1_nproc` | — · 1 witness | no |
| `gunbc.machine_intake_power_supply` | 25 | 1 | — | `gunbc.machine_intake_mtcollins1_platform_observation` · 1 witness | no |
| `gunbc.mt_collins_dimm_physical_identity` | 173 | 44 | `MtCollins2uMemoryTopologyCitation`, `MtCollinsChannelConnectorPair`, `MtCollinsChassisFigureOrientation`, `MtCollinsDimmDiagramLegend`, `MtCollinsDimmFigureBank` … | `gunbc.machine_intake_mtcollins1_dimm_connector_observation`, `gunbc.machine_intake_mtcollins1_physical_orientation` · 1 witness | no |

### Layer 1 — unit (17 modules)

| module | lines | refs | unit/platform constants imported | consumers (import census) | gate |
|---|---|---|---|---|---|
| `gunbc.machine_intake_mtcollins1_32dimm_bringup_observation` | 695 | 114 | `AmpereMtCollins`, `mtcollins1_firmware`, `mtcollins_training_capture_path`, `mtcollins_training_capture_sha256` | `gunbc.machine_intake_mtcollins1_sixteen_module_restore_observation` · 3 witness | no |
| `gunbc.machine_intake_mtcollins1_access_observation` | 294 | 45 | `AmpereMtCollins` | `gunbc.host_reset_subject_roster`, `gunbc.machine_intake_mtcollins1_32dimm_bringup_observation`, `gunbc.machine_intake_mtcollins1_actuate`, `gunbc.machine_intake_mtcollins1_bmc_secure_observation`, `gunbc.machine_intake_mtcollins1_boot_authorization`, `gunbc.machine_intake_mtcollins1_boot_diagnostic_bundle` +8 more · 3 witness | no |
| `gunbc.machine_intake_mtcollins1_connectx4lx_observation` | 108 | 19 | `operator_host_mtcollins1` | `gunbc.machine_intake_mtcollins1_expansion_candidates` · 2 witness | no |
| `gunbc.machine_intake_mtcollins1_expansion_attachment` | 112 | 6 | `MtCollinsUnitExpansionEvidence`, `mt_collins_1u_ocp_slot` | `gunbc.machine_intake_mtcollins1_expansion_candidates` · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_expansion_candidates` | 87 | 8 | `mt_collins_1u_ocp_slot` | — · 2 witness | no |
| `gunbc.machine_intake_mtcollins1_expansion_observation` | 186 | 18 | `MtCollins1UExpansion`, `mt_collins_1u_expansion`, `mt_collins_1u_ocp_slot` | `gunbc.machine_intake_mtcollins1_expansion_attachment`, `gunbc.machine_intake_mtcollins1_expansion_candidates` · 2 witness | no |
| `gunbc.machine_intake_mtcollins1_memory_prediction` | 156 | 8 | — | — · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_sixteen_module_restore_observation` | 304 | 44 | `mtcollins1_pre_os_capabilities`, `mtcollins1_sol_idle_capture`, `mtcollins1_sol_reading`, `mtcollins1_sol_session_banner_digest` | — · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_socket1_investigation_observation` | 389 | 139 | `MtCollinsDimmFigure` | `gunbc.machine_intake_mtcollins1_physical_orientation` · 1 witness | no |
| `gunbc.machine_intake_mtcollins1_spare_screen_prediction` | 247 | 16 | `MtCollinsDimmPopulationRow`, `mt_collins_population_16_dimms` | — · 1 witness | no |
| `gunbc.memory_configuration_evaluation` | 396 | 11 | `AmpereMtCollins`, `MtCollinsChannelPopulationRoleStanding`, `MtCollinsPopulationRoleRefusal`, `mt_collins_channel_population_roles`, `mt_collins_socket_dimm_sizes_supported` … | `gunbc.memory_change_evaluation`, `gunbc.memory_change_report` · 2 witness | no |
| `gunbc.mtcollins_3ds_memory_change` | 132 | 59 | `MtCollinsChannelPopulationRole`, `MtCollinsPopulationRoleRefusal`, `mt_collins_channel_population_roles`, `mtcollins_dram_cycle_start`, `mtcollins_hynix_part` … | `gunbc.machine_intake_pre_os_capture_replay` · 1 witness | no |
| `gunbc.mtcollins_memory_change` | 197 | 57 | `AmpereMtCollins`, `MtCollinsChannelPopulationRole`, `MtCollinsPopulationRoleRefusal`, `mt_collins_channel_population_roles`, `mt_collins_socket0_channel_pairs` … | `gunbc.machine_intake_mtcollins1_32dimm_bringup_observation`, `gunbc.machine_intake_pre_os_capture_replay`, `gunbc.machine_intake_pre_os_timeline`, `gunbc.mtcollins_3ds_memory_change` · 4 witness | no |
| `gunbc.mtcollins_memory_placement` | 22 | 4 | `MtCollinsChannelPopulationRole` | `gunbc.memory_configuration_evaluation`, `gunbc.mtcollins_memory_change` · 2 witness | no |
| `gunbc.rung_drop.mtcollins1_boot_accepts_socket1_absent` | 46 | 27 | — | `gunbc.rung_drop.roster` | no |
| `gunbc.rung_drop.mtcollins1_boot_matrix_enrolment_dead_band_observed_only` | 44 | 15 | `mtcollins1_boot_matrix_enrolment_dead_band_observed_only`, `mtcollins1_boot_matrix_native_witness_capability` | `gunbc.rung_drop.roster` | no |
| `gunbc.rung_drop.mtcollins1_boot_matrix_new_witness_eval_step_cost` | 58 | 11 | — | `gunbc.rung_drop.mtcollins1_boot_matrix_enrolment_dead_band_observed_only`, `gunbc.rung_drop.roster` | no |

### Layer R — reference only (keeps prose/receipt; no layer move) (77 modules)

| module | lines | refs | unit/platform constants imported | consumers (import census) | gate |
|---|---|---|---|---|---|
| `extdeps.linux.edac` | 214 | 1 | — | `gunbc.machine_intake_mtcollins1_memory_census_observation` · 2 witness | no |
| `extdeps.linux.mlx5_core` | 33 | 1 | — | `gunbc.machine_intake_mtcollins1_connectx4lx_observation` · 1 witness | no |
| `extdeps.network.dell_0r887v` | 23 | 1 | — | `gunbc.machine_intake_mtcollins1_expansion_candidates` | no |
| `extdeps.network.ibm_01ft753` | 40 | 1 | — | `gunbc.machine_intake_mtcollins1_expansion_candidates` | no |
| `extdeps.network.mcx4121a_acat` | 53 | 1 | — | `gunbc.machine_intake_mtcollins1_connectx4lx_observation`, `gunbc.machine_intake_mtcollins1_expansion_candidates` | no |
| `extdeps.network.mcx4411a_acan` | 24 | 1 | — | `gunbc.machine_intake_mtcollins1_expansion_candidates` | no |
| `extdeps.network.mcx4421a_acan` | 24 | 1 | — | `gunbc.machine_intake_mtcollins1_expansion_candidates` | no |
| `extdeps.network.mcx4621a_acab` | 24 | 1 | — | `gunbc.machine_intake_mtcollins1_expansion_candidates` | no |
| `extdeps.standards.nic_attachment` | 62 | 1 | — | `extdeps.ampere.mt_collins_1u_product_brief.specifications`, `extdeps.network.dell_0r887v`, `extdeps.network.ibm_01ft753`, `extdeps.network.mcx4121a_acat`, `extdeps.network.mcx4411a_acan`, `extdeps.network.mcx4421a_acan` +6 more · 3 witness | no |
| `gunbc.ci_layer_roots` | 2236 | 3 | — | `dag.test.claim.offline_local_recipe_witness`, `gunbc.ci_spec`, `gunbc.commit_workflow`, `gunbc.contributor_onboarding_path`, `gunbc.decoder_execution_identity`, `gunbc.derived_obligation_row` +34 more · 9 witness | **yes** |
| `gunbc.ci_spec` | 3825 | 194 | `mtcollins1_bmc_gunbc_secret_ref`, `mtcollins1_boot_sol_capture_path_env`, `mtcollins1_boot_sol_pid_path_env`, `mtcollins1_census_image_receipt_path`, `mtcollins1_fan_credential_path_env` … | `gunbc.compiler_gate_workflow`, `gunbc.fleet_converge_workflow`, `gunbc.fleet_release_bins_key`, `gunbc.fleet_workflow_steps`, `gunbc.generated_artifact_emit`, `gunbc.heal_publisher_workflow` +10 more · 10 witness | no |
| `gunbc.doc_graph_roots` | 1757 | 3 | — | `v2.test.lens_doc_reachability.doc_reachability_test` · 3 witness | no |
| `gunbc.durable_cas_file_store` | 856 | 1 | — | `gunbc.auth.approval_assertion_counter`, `gunbc.auth.approval_decision_store`, `gunbc.auth.approval_device_redemption`, `gunbc.auth.approval_keyring_converge`, `gunbc.auth.approval_store_receipt`, `gunbc.authorization_claim_slot` +9 more · 8 witness | **yes** |
| `gunbc.durable_exclusive_hold_file_store` | 369 | 1 | — | `gunbc.fabric_control_plane`, `gunbc.fabric_required_build_cell`, `gunbc.machine_intake_mtcollins1_maintenance_hold`, `tools.fabric_control_plane_live_probe` · 7 witness | no |
| `gunbc.fleet_intent_network` | 461 | 3 | — | `gunbc.approve_ios_project`, `gunbc.auth.approval_broker_endpoint`, `gunbc.auth.approval_device_enrolment_code_issue`, `gunbc.auth.approval_keyring_converge`, `gunbc.auth.approval_ntfy_deployment`, `gunbc.auth.approval_request_submission` +90 more · 101 witness | **yes** |
| `gunbc.floor_demand` | 1870 | 1 | — | `gunbc.ci_floor_measurement`, `gunbc.fabric_control_plane`, `gunbc.fabric_floor_dispatch`, `gunbc.fabric_witness_run`, `gunbc.floor_cold_build_receipt`, `gunbc.floor_memory_demand` +5 more · 8 witness | **yes** |
| `gunbc.guarantee_stall.pci_address_rendering_equivalence_stall` | 17 | 2 | — | `gunbc.guarantee_stall.roster` | no |
| `gunbc.non_fold_residue` | 2029 | 55 | — | — · 1 witness | no |
| `gunbc.owned_process` | 175 | 1 | — | `gunbc.bmc_megarac_web_transport`, `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_kvm_observer_observe`, `gunbc.machine_intake_mtcollins1_kvm_still` · 3 witness | no |
| `gunbc.recurring_failure_mode.a_derivation_key_that_does_not_pin_bytes_across_build_hosts` | 26 | 4 | — | — | no |
| `gunbc.recurring_failure_mode.a_pending_member_vanishes_from_a_terminal_refusal` | 28 | 3 | — | — | no |
| `gunbc.recurring_failure_mode.a_termination_contract_is_met_by_a_trimming_transport` | 51 | 5 | — | — | no |
| `gunbc.recurring_failure_mode.absent_reads_identically_to_never_looked` | 70 | 3 | — | — | no |
| `gunbc.recurring_failure_mode.ad_hoc_hardware_probe_produces_no_receipt` | 26 | 1 | — | — | no |
| `gunbc.recurring_failure_mode.an_unspawnable_program_aborts_the_run_instead_of_refusing` | 22 | 2 | — | — | no |
| `gunbc.recurring_failure_mode.collector_armed_after_the_transition_it_observes` | 14 | 2 | — | — | no |
| `gunbc.recurring_failure_mode.control_plane_acknowledgement_minted_as_effect` | 14 | 1 | — | — | no |
| `gunbc.recurring_failure_mode.else_less_if_statement_has_no_lowered_form` | 28 | 2 | — | — | no |
| `gunbc.recurring_failure_mode.empirical_association_promoted_to_decode` | 14 | 1 | — | — | no |
| `gunbc.recurring_failure_mode.enforcement_gate_positioned_after_the_effect_it_guards` | 15 | 1 | — | — | no |
| `gunbc.recurring_failure_mode.enrolment_dead_band_has_no_representable_standing` | 17 | 2 | — | — | no |
| `gunbc.recurring_failure_mode.gunbc_run_pays_a_large_fixed_pre_entry_cost_per_invocation` | 23 | 4 | — | — | no |
| `gunbc.recurring_failure_mode.identity_hashed_from_a_shared_mutable_path` | 44 | 2 | — | — | no |
| `gunbc.recurring_failure_mode.rest_response_nested_from_key_ignored` | 32 | 2 | — | — | no |
| `gunbc.recurring_failure_mode.subject_and_its_digest_as_independent_parameters` | 22 | 2 | — | — | no |
| `gunbc.recurring_failure_mode.the_checker_admits_a_cast_the_evaluator_refuses` | 17 | 2 | — | — | no |
| `gunbc.recurring_failure_mode.unbound_interpolation_in_service_argv_accepted_until_runtime` | 22 | 1 | — | — | no |
| `gunbc.recurring_failure_mode.upstream_non_endorsement_consumed_as_physical_prohibition` | 17 | 3 | — | — | no |
| `gunbc.recurring_failure_mode.wet_witness_keyed_to_the_runner_not_its_subject` | 28 | 1 | — | — | no |
| `gunbc.roadmap_authority` | 5415 | 4 | — | `gunbc.floor_non_verdict_enrollment`, `gunbc.generated_artifact_emit`, `gunbc.roadmap_belt`, `gunbc.roadmap_belt_actuate`, `gunbc.roadmap_closing_contract_authoring`, `gunbc.roadmap_forecast` +11 more · 15 witness | **yes** |
| `gunbc.rung_drop.roster` | 240 | 9 | `mtcollins1_boot_accepts_socket1_absent`, `mtcollins1_boot_matrix_enrolment_dead_band_observed_only_drop`, `mtcollins1_boot_matrix_new_witness_eval_step_cost` | `gunbc.design_ledgers`, `gunbc.jurisdiction_drop_accounting`, `gunbc.ledger_row_coherence`, `v2.workflow.floor_subject_seed_test` · 5 witness | no |
| `gunbc.runner.runner_bound_attempt_receipt` | 184 | 4 | — | — | no |
| `gunbc.runner.runner_cgroup_placement_receipt` | 169 | 1 | — | — | no |
| `gunbc.runner.runner_checkout_receipt` | 109 | 1 | — | — | no |
| `gunbc.runner.runner_first_job_receipt` | 237 | 3 | — | — | no |
| `gunbc.runner.runner_guest_egress_attempt` | 209 | 1 | — | `gunbc.runner.runner_cgroup_placement_receipt`, `gunbc.runner.runner_cpu_bandwidth_receipt` | no |
| `gunbc.runner.runner_guest_network_receipt` | 186 | 1 | — | — | no |
| `gunbc.runner.runner_host_first_boot_receipt` | 127 | 5 | — | `gunbc.machine_intake_pre_os_capture_replay` · 2 witness | no |
| `gunbc.runner.runner_host_hardware_observation` | 216 | 9 | — | — · 1 witness | no |
| `gunbc.runner.runner_host_jailed_kvm_receipt` | 200 | 2 | — | — | no |
| `gunbc.runner.runner_host_kvm_probe_receipt` | 122 | 5 | — | — | no |
| `gunbc.runner.runner_host_media_independence_receipt` | 137 | 10 | — | — | no |
| `gunbc.runner.runner_isolation_admission` | 292 | 2 | — | — · 1 witness | no |
| `gunbc.runner.runner_live_census_receipt` | 185 | 1 | — | — | no |
| `gunbc.runner.runner_observed_version_check` | 129 | 3 | — | — | no |
| `gunbc.runner.runner_qualification_receipt` | 177 | 4 | — | — | no |
| `gunbc.runner.runner_signed_envelope_receipt` | 114 | 3 | — | — | no |
| `gunbc.runner.runner_two_attempt_receipt` | 180 | 3 | — | — | no |
| `gunbc.runner_attempt_launch` | 1044 | 1 | — | `gunbc.runner_microvm_lifecycle`, `gunbc.runner_microvm_lifecycle_realize`, `gunbc.runner_microvm_slot_controller`, `gunbc.test.claim.runner.runner_microvm_lifecycle_witness_test` · 2 witness | no |
| `gunbc.runner_browser_toolchain` | 1381 | 3 | — | `gunbc.machine_intake_mtcollins1_boot_run`, `gunbc.machine_intake_mtcollins1_kvm_observer_observe`, `gunbc.machine_intake_mtcollins1_kvm_still`, `gunbc.wet_host_premise_readback` · 8 witness | no |
| `gunbc.runner_host_grants` | 479 | 5 | `mtcollins1_unit_hold_store_host` | `gunbc.fleet_converge_plan`, `gunbc.generated_artifact_emit`, `gunbc.host_credential_custody_converge`, `gunbc.runner_host_file_converge`, `gunbc.runner_microvm_network_apply`, `gunbc.runner_microvm_network_observe` · 4 witness | no |
| `gunbc.runner_jit_mint` | 159 | 1 | — | `gunbc.runner.runner_jit_perform`, `gunbc.runner_attempt_launch` · 1 witness | no |
| `gunbc.runner_registration_labels` | 94 | 1 | — | `gunbc.fleet_runner_connectivity`, `gunbc.runner_browser_toolchain`, `gunbc.runner_connectivity_recovery`, `gunbc.runner_unit_file` · 3 witness | **yes** |
| `gunbc.runner_throughput_qualification` | 1027 | 2 | — | `gunbc.runner_throughput_qualification_route`, `gunbc.runner_throughput_selection`, `product.fabric.node_qualification` · 1 witness | **yes** |
| `gunbc.runner_throughput_qualification_route` | 383 | 27 | `operator_host_mtcollins1` | — · 1 witness | no |
| `gunbc.secret_provision` | 603 | 6 | — | `gunbc.auth.approval_decision_store`, `gunbc.auth.approval_device_redemption`, `gunbc.auth.approval_mac_key_provision`, `gunbc.auth.approval_ntfy_deployment`, `gunbc.auth.approval_request_submission`, `gunbc.auth.approval_store_receipt` +13 more · 8 witness | **yes** |
| `gunbc.spark.pair_serving_d0_door` | 355 | 1 | — | `gunbc.fleet_converge_workflow` · 3 witness | no |
| `tools.approval_store_live_probe` | 236 | 1 | — | — | no |
| `v2.compiler.reference_conservation_census` | 816 | 1 | — | `v2.test.claim.namespace_xl0.reference_conservation_census` | no |
| `v2.test.floor_enrolment_margin` | 717 | 4 | `mtcollins1_boot_matrix_enrolment_dead_band_observed_only` | — | no |
| `v2.workflow.floor_cost_debt_admission` | 155 | 12 | — | `v2.workflow.floor_enrolment_margin` | no |
| `v2.workflow.floor_enrolment_dead_band` | 97 | 15 | — | `gunbc.rung_drop.mtcollins1_boot_matrix_enrolment_dead_band_observed_only`, `v2.test.floor_enrolment_margin`, `v2.workflow.floor_enrolment_margin` | no |
| `v2.workflow.floor_enrolment_margin` | 649 | 1 | — | `v2.test.floor_enrolment_margin` · 1 witness | no |
| `v2.workflow.floor_eval_step_cost_drop` | 416 | 31 | — | `gunbc.rung_drop.app_attest_interpreted_crypto_new_witness_eval_step_cost`, `gunbc.rung_drop.app_attest_verifier_new_witness_eval_step_cost`, `gunbc.rung_drop.dag_emit_round_trip_new_witness_eval_step_cost`, `gunbc.rung_drop.fleet_build_job_membership_new_witness_eval_step_cost`, `gunbc.rung_drop.live_deploy_apply_render_new_witness_eval_step_cost`, `gunbc.rung_drop.live_deploy_dark_install_render_new_witness_eval_step_cost` +7 more · 1 witness | no |
| `v2.workflow.floor_route_gap` | 1999 | 24 | — | `gunbc.native_frontier_ratchet`, `gunbc.witness_v2_native_route`, `v2.test.claim.local_repo_wet_terminal_test`, `v2.workflow.floor_terminal_ledger`, `v2.workflow.floor_terminal_ledger_wire`, `v2.workflow.floor_terminal_ledger_wire_test` +1 more · 1 witness | **yes** |
| `v2.workflow.floor_unimported_bare_provider_debt_roster` | 2209 | 7 | — | `v2.test.claim.floor_unimported_bare_provider_debt`, `v2.workflow.floor_unimported_bare_provider_debt` | no |
| `v2.workflow.local_repo_wet_terminal` | 2562 | 55 | — | `gunbc.wet_host_premise_readback`, `v2.test.claim.local_repo_wet_terminal_test`, `v2.workflow.required_floor`, `v2.workflow.witness_admission` · 1 witness | no |

### Witnesses (81 modules)

Each witness moves with the subject it imports; none is in the required gate except `test.claim.action_use_admission_witness` (3 prose refs only).

`test.claim.action_use_admission_witness`, `test.claim.approval_request_submission_witness_test`, `test.claim.authorization_pattern_selection_witness`, `test.claim.boot_artifact_delivery_witness_test`, `test.claim.build_fulfillment_witness`, `test.claim.build_selection_witness`, `test.claim.durable_exclusive_hold_witness`, `test.claim.fan_policy_standing_witness`, `test.claim.fleet.fleet_converge_checkout_pin_witness_test`, `test.claim.fleet.site_pxe_edge_converge_witness`, `test.claim.gcp_secret_access_witness_test`, `test.claim.heal_publisher_provision_witness`, `test.claim.host_memory_qualification_witness_test`, `test.claim.host_reset_return_witness_test`, `test.claim.install_media_remaster_grub_cmdline_witness`, `test.claim.linux_edac_topology_authority_witness`, `test.claim.machine_intake.ampere_dram_console_observation_witness_test`, `test.claim.machine_intake.ampere_socket_console_observation_witness_test`, `test.claim.machine_intake.host_capture_sol_crlf_witness_test`, `test.claim.machine_intake.host_capture_sol_nul_fill_witness_test`, `test.claim.machine_intake.host_resource_observation_witness`, `test.claim.machine_intake.kernel_module_decompression_observation_witness_test`, `test.claim.machine_intake.megarac_media_convergence_witness_test`, `test.claim.machine_intake.megarac_ui_features_witness_test`, `test.claim.machine_intake.mtcollins1_bmc_sensor_observation_witness_test`, `test.claim.machine_intake.mtcollins1_boot_acceptance_matrix_test`, `test.claim.machine_intake.mtcollins1_boot_authorization_witness_test`, `test.claim.machine_intake.mtcollins1_boot_diagnostic_bundle_witness_test`, `test.claim.machine_intake.mtcollins1_boot_phase_timing_witness_test`, `test.claim.machine_intake.mtcollins1_boot_run_witness_test`, `test.claim.machine_intake.mtcollins1_census_image_publish_witness_test`, `test.claim.machine_intake.mtcollins1_census_medium_readback_witness_test`, `test.claim.machine_intake.mtcollins1_census_member_readback_witness_test`, `test.claim.machine_intake.mtcollins1_census_qemu_host_observe_witness_test`, `test.claim.machine_intake.mtcollins1_census_refusal_hypotheses_witness_test`, `test.claim.machine_intake.mtcollins1_kvm_observer_observe_witness_test`, `test.claim.machine_intake.mtcollins1_kvm_observer_protocol_wet_witness`, `test.claim.machine_intake.mtcollins1_kvm_still_witness_test`, `test.claim.machine_intake.mtcollins1_maintenance_hold_witness_test`, `test.claim.machine_intake.mtcollins1_physical_orientation_witness`, `test.claim.machine_intake.mtcollins1_sixteen_module_restore_witness_test`, `test.claim.machine_intake.mtcollins1_smpro_observation_witness_test`, `test.claim.machine_intake.mtcollins1_stale_power_off_witness_test`, `test.claim.machine_intake.mtcollins1_ui_bundle_observe_witness_test`, `test.claim.machine_intake.mtcollins1_unit_hold_forged_probe_witness_test`, `test.claim.machine_intake.oob_power_action_witness`, `test.claim.machine_intake.pre_os_bringup_witness_test`, `test.claim.machine_intake.pre_os_sol_crlf_witness_test`, `test.claim.machine_intake.sol_hold_stdin_wet_witness`, `test.claim.machine_intake_ethernet_witness`, `test.claim.machine_intake_mtcollins1_bmc_secure_standing_witness_test`, `test.claim.machine_intake_predictive_claim_witness_test`, `test.claim.megarac_spx_ui_surface_artifact_integrity_witness`, `test.claim.memory_change_evaluation_witness`, `test.claim.memory_multiple_findings_witness`, `test.claim.micron_dram_module`, `test.claim.modeled_filesystem_witness_test`, `test.claim.mt_collins_dimm_physical_identity_witness`, `test.claim.mt_collins_population_role_witness`, `test.claim.mtcollins1_candidate_attachment_witness_test`, `test.claim.mtcollins1_census_image`, `test.claim.mtcollins1_census_image_local_wet`, `test.claim.mtcollins1_connectx4lx_observation_witness_test`, `test.claim.mtcollins1_expansion_witness_test`, `test.claim.mtcollins1_memory_census_witness_test`, `test.claim.mtjade1_access_witness`, `test.claim.nbd_proxy_virtual_media_install`, `test.claim.pci_identity_observation_witness`, `test.claim.pre_os_capture_replay_witness`, `test.claim.pre_os_observer_witness`, `test.claim.pre_os_replay_witness`, `test.claim.product.cohort_observation_witness`, `test.claim.refinement_cast_wall_witness_test`, `test.claim.runner_isolation_admission_witness`, `test.claim.runner_throughput_qualification_witness`, `test.claim.site_uefi_arm64_pxe_edge_witness`, `test.claim.srv3_os_install_actuate`, `test.claim.unit_hold_store_provision_witness`, `test.claim.workflow_dispatch_input_witness`, `test.claim.xorriso_path_list_witness`, `test.probe.unit_hold_proof_forged_probe`
