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

The full receipt and method are under **Instrument** below.

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
| **4a** | **Boot subject leaves**: artifact, image fetch, milestone, phase timing, admission, authorization, actuate. `MtCollins1BootSubject` becomes a boot subject over `ManagedHost`. | `MtCollins1BootSubject`, `mtcollins1_boot_subject*`, `mtcollins1_cdrom_selection`, `mtcollins1_boot_export_dir` as procedure inputs | After 2, 3, and #13025 (it edits the phase-timing witness). |
| **4b** | **Census boot image chain**: the 7 `mtcollins1_census_*` modules. Toolchain and QEMU modules are executor-host operations, so they are keyed by executor host, not managed host. | the `mtcollins1_census_*` procedure roots | After 4a. |
| **4c** | **Boot federation standing**: `ManagedHostBootFederationStanding` replaces `gunbc.auth.mtcollins1_boot_federation*`, live names kept as the `mtcollins1` binding (ruling b). | `mtcollins1_boot_*` federation decls | After 0. Independent of 4a/4b. |
| **4d** | **Boot run, dry realization, diagnostic bundle**: the vertical's root. Unit observations it reads today (`mtcollins1_memory_census_observation`, `mtcollins1_access_observation`) become row reads or parameters, so the dependency direction is procedure ← unit. | `mtcollins1_boot_run`, `_boot_dry_realization`, `_boot_diagnostic_bundle` | After 1, 4a, 4b, 4c. Likely split run/dry and bundle at dispatch. |
| **5** | **Remaining BMC-stack observations**: fan observe, UI bundle observe, KVM observer, SOL notice, served-UI catalog, BMC fan, which the boot does not need. | those `mtcollins1_` procedure roots | After 3. |
| **6** | **Platform**: physical orientation, DIMM connector and platform observation become logic over the baseboard read through `ManagedHost.access`. The Mt. Collins figure stays in `extdeps.ampere.mt_collins_*`. | the board bindings in those modules | After #13025 (it edits `mtcollins1_physical_orientation`). |
| **7** | **Workflow modes**, per ruling (a). | the 8 `mtcollins1_*` literals and their steps in `gunbc.fleet_converge_workflow` and its projection; `mtcollins-canary.yml` dispositioned in the same change | Last. Operator sign-off on names first. |

### Terminal receipt owed by every cut (review item 5)

These modules are outside the required gate, so a green required run proves nothing about them. Each PR's description carries all of the following:

1. **Consumer witnesses, run by name.** The cut's complete consumer roster, recomputed at the PR head with the instrument below (not copied from this receipt). Each named witness is executed with `gunbc run` against the head, with binary path and build sha printed.
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

Exact token `mtcollins1_boot`: 19 files at `26e99a9c7f`, all inside the populations above.

## Instrument

The import census in [managed-host-untangle-census.tsv](managed-host-untangle-census.tsv) is the receipt. It lists every module with its layer, path, required-gate membership, imported unit symbols, and its **complete** importer list. The method:

1. Walk `dag/` and `src/v2/` for `.dag` files.
2. Read each `^module` and each `^import <name>`.
3. The population is the modules whose name or text matches `mtcollins` (case-insensitive).
4. Importers are the reverse import edges.
5. Gate membership is the transitive import closure from the modules matching `v2.workflow.required_floor` `required_gate_prefixes`.
6. Imported unit symbols are the names inside each module's `import … { … }` lists that match `mtcollins|mt_collins|MtCollins`.

Each cut recomputes its own root's roster at its head this way. The table below is a reading aid. It truncates consumer lists, and the TSV does not.

## Census

The layer is assigned per module from its subject.
- **R** = mentions the unit only in prose or receipts.
- **W** = witness.

Consumers list production importers by name, truncated after six with a count, plus the number of witness importers. **The complete lists are in the TSV.**

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
