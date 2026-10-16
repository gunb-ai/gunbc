# Cut 2 — decompose ManagedHost (snappy-seal-211)

Goal: decompose `gunbc.host.managed_host.ManagedHost` into per-fact owners; shrink to `{host, access}`; add NO new fields to it. DRAFT PR #14063, branch `session/snappy-seal-211`. Operator/manager merges; never merge myself.

## Stage A — DONE + approved
- `gunbc.host.bmc_control_binding`: `BmcControlBinding{endpoint,unit,secured}` + join + endpoint-mismatch refusal (RED3 holds).
- `ControlledHost` (host_control_route) consumes it; `bmc_controlled_host(host, standing: BmcControlStanding)`.
- `ManagedHostLookup` found arms carry `control: BmcControlStanding` beside legacy `standing` (legacy deleted in stage D — 8 prod consumers listed in PR body).

## Stage B — IN PROGRESS (manager approved derivation: typed refusal; family-set board edge {Altra,AltraMax}; all-agree-or-unestablished; pxe="selectable via override"; board edge cited in board module)

DONE + pushed (e5d277c5c42), each typechecked 0 blocking:
- `extdeps.cpu.types`: `SoCFamily{name,models}` + `soc_family_architecture` (all models agree else none).
- `extdeps.cpu.ampere`: `ampere_altra_soc_family` (Q64-30,Q80-30) + `ampere_altra_max_soc_family` (M128-30).
- `extdeps.board_soc`: `BoardSocEdge{board,soc_families,citations}` + `BoardSocArchitectureStanding{Established{architecture}|Unestablished}` + `board_soc_architecture` (all admitted families agree else Unestablished, never first-wins).
- `extdeps.ampere.mt_collins_board`: `mt_collins_board_soc_edge` (families {Altra,AltraMax}, cited product brief `platform_note`).
- `gunbc.host.board_platform_facts`: `board_soc_edges=[mt_collins_board_soc_edge]`; `host_board_architecture(access: BmcAccessObservationStanding) -> BoardSocArchitectureStanding` (observed board FRU -> `board_authority_for` -> `board_facts_authority` -> edge -> arch; any missing link Unestablished).
- `gunbc.host.controller_capability`: `CapabilityFactStanding{CapabilityEstablished|CapabilityUnestablished}`; `ControllerCapability{power_read_write, one_shot_boot_selection, persistent_boot_selection, virtual_media, pxe_selection, serial_console, graphical_console}`; `controller_capability(row)` (power<-PowerControl; one_shot+pxe<-BootSourceOverride; virtual_media<-VirtualMedia|NbdProxyVirtualMedia|OemRemoteMedia; serial<-SerialConsole; persistent+graphical=CapabilityUnestablished, NOT defaulted).
- `gunbc.host.smpro_platform_capability`: `SmproReading` re-homed verbatim from the boot diagnostic bundle (`SmproProbed{observation:SmproPassObservation}|SmproNotTaken{reason}`) + `smpro_capability(architecture, reading) -> SmproCapability` (x86 = `SmproCapabilityAbsent`, not empty/refused). Unblocks merry-otter-837 4d-ii; otter told.

REMAINING (build + typecheck + commit):
1. NEW `gunbc.host.virtual_media_boot_selection`: `VirtualMediaBootSelectionRefusal{VirtualMediaUnestablished | ArchitectureUnsupported{architecture} | ArchitectureUnestablished}`; `VirtualMediaBootSelectionOutcome{Admitted{selection: IpmiBootSelection} | Refused{cause}}`; `virtual_media_boot_selection(arch: BoardSocArchitectureStanding, media: CapabilityFactStanding) -> Outcome` (arch gate first: Unestablished -> ArchitectureUnestablished; non-Aarch64 -> ArchitectureUnsupported; then media Unestablished -> VirtualMediaUnestablished; else Admitted `aarch64_uefi_once_boot_selection(IpmiBootCdrom)`).
2. EDIT `managed_host_boot`: `boot_selection(host: ManagedHost) -> VirtualMediaBootSelectionOutcome` = `virtual_media_boot_selection(host_board_architecture(host.access), controller_capability(host.capability_row).virtual_media)`. `BootSubjectNoSelection{host, cause: VirtualMediaBootSelectionRefusal}`; `admit_boot_subject` matches the outcome (Admitted -> AdmittedBootSubject with selection; Refused -> BootSubjectRefused{host, cause}). Drop `operator_host_mtcollins1` + `mtcollins1_virtual_media_boot_selection` imports.
3. EDIT `host_reset_boot_selection`: `reset_boot_selection(host) -> Outcome` (same generic derivation; stays a separate fn so the boot/reset arms cannot cross-wire).
4. EDIT `host_reset_subject_roster`: `ResetSubjectNoSelection{host, cause}`; `admitted_reset_subjects` match.
5. DELETE `data mtcollins1_virtual_media_boot_selection` (mtcollins1_access_observation).
6. WITNESSES: `managed_host_boot_witness` + `managed_host_witness` "unselected-host" fixture -> cap row lacking virtual-media (e.g. `[CapabilityPowerControl]`) so it refuses `VirtualMediaUnestablished` (arch still Aarch64). + NEW RED1 witness: 2nd machine (same Altra board -> Aarch64 + same cap row `[BootSourceOverride,PowerControl,OemRemoteMedia]`) gets the same `IpmiBootSelection` via the generic fn, no code change.

## Stage C
- unit_hold (ResetUnitHold) -> machine-operation (maintenance-exclusion) policy owner.
- SMpro -> `gunbc.host.smpro_platform_capability` (DONE in stage B). x86 = ABSENT, not empty/refused. RED2: x86 fixture no SMpro = valid controlled machine.

## Stage D
- Shrink `ManagedHost` -> `{host, access}`; migrate the 8 legacy consumers off secure/capability_row/unit_hold/smpro; DELETE legacy `ManagedHostBindingStanding`/`ManagedHostBinding` (no dual standing on main).
- OWNERSHIP with crisp-newt-630 (record carries the RungDrop VALUE, not a ref; classifier reads `record.covering_drop.standing` O(1)): I own the record field (`covering_drop: RungDrop?` value) + `member_under_declared_drop` param + the projection arm reading the standing + the 3 downstream `.drop` fields (managed_host_boot_federation:108, host_reset_subject_roster:102/116) + the mtcollins1 authoring site (L418) switching to the value. They own the seam: remove `import standing` (L20) + delete orphaned `covering_bmc_secure_drop`/`drop_is_standing_citation` (L205-214) + the witness decl_name->identity updates (managed_host_admission_witness:53/68/75/80/276) + `mtcollins1_bmc_secure_standing_drop_ref` (L195) + the 3 witness `drop_is_standing_citation` sites becoming value-based. Ping before either pushes (shared managed_host.dag L147-360, L437-472).

## Stage E (after D, measured before/after; memory lever)
- `managed_host_memberships()` stays NO-ARG (record carries the value). The whole lever = cut `import gunbc.rung_drop.standing` at L20; the per-host classifier reads the value off the record. I measure the real (compiler) boot_run closure before/after.
- Manager's success criterion: `gunbc.rung_drop.standing` + roster leave mtcollins1_boot_run's import closure; measure closure file count AND gunbc run max_rss before/after. Per-host path must NOT route through any function that calls `standing_rung_drops()`.

## Typecheck loop
`/cargo-target/release/gunbc compile --source-root dag --source-root src/v2 --entry <module> --target dag` — SEQUENTIAL (parallel OOMs the shared cgroup). 0 blocking errors = clean.
NOTE: the giant mtcollins1 `boot_diagnostic_bundle` closure has 13 PRE-EXISTING blocking errors in src/v2 (7) + srv3 (1) + host_effect_realize (2) + megarac_kvm_still (3) — confirmed via a base (unmodified) run; my edits add 0.

## Coordination
- cut 1 = crisp-newt-630 (admission-join -> PhysicalMachineBinding; owns the stage E seam).
- merry-otter-837 (neutral boot module gunbc.host.boot_attempt + 4d-ii megarac adapter): imported SmproReading from smpro_platform_capability (pushed).
- zesty-eagle-346 (census image chain), still-owl-732 (boot run, after otter).

## Rules
Model before implement; DFS existing vocabulary (extend not mint); delete-first; no host/board-name branch in generic code; no workflow-mode renames.
