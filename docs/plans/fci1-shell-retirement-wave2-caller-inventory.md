# FCI-1 shell retirement · Wave 2 caller inventory (read-only)

Lane: gentle-otter-877 (FCI shell retirement), from stern-bat-401. Authority for the evidence
side: `gunbc.instruments.fabric_ci_evidence`. No code change made — inventory only.

Scope: every caller of the two shell controls, plus the shared driver and the sibling context
env, by module and symbol (not file:line). For each control, the process boundaries it observes,
mapped to the brief's families (value files vs diagnostics; Alpha vs Beta transport; altered
bytes; failed/ignored writes; projection-arm removal). Which callers are inside the required
gate vs outside it.

> The `retained_srvn` helpers in `gunbc.host.host_effect_realize` (snappy-bat-554) are NOT
> touched here. They share a name family but are a different module and are out of scope.

## 1. The four controls and the one sibling

| file | role | dissolve-on (own header) |
|------|------|---------------------------|
| `tools/fabric_ci_fci1_live_instrument.sh` | FCI-1 reservation instrument (the wet live run). Starts no Work command. | bash target grammar gains `[[ ]]`/glob, fn-with-local, `$(<file)`, subshell-free `a && b \|\| c` → emit as `bash_build`/`bash_emit` or typed `host_effect_apply` |
| `tools/fabric_ci_fci1_bounded_execution_controls.sh` | FCI-1 bounded-driver one-axis controls (cgroup/memory/workspace negative rows). Starts no Work, touches no reservation. | same bash-grammar trigger as above |
| `tools/fabric_ci_evidence_driver.sh` | shared wet-gate driver: status + exact named value files only; "diagnostics are never data". | same bash-grammar trigger |
| `tools/fabric_ci_evidence_calibration.sh` | realizes the ordered typed rows emitted by `fabric_ci_calibration_write_plan` (the evidence contract). | same bash-grammar trigger |
| `tools/fabric_ci_fci1_bounded_execution_context.env` | **sibling, generated** by `gunbc.fabric.fci1_bounded_execution_context_emit`. DO-NOT-HAND-EDIT. Defines `FCI1_MEMORY_MAX_BYTES`, `FCI1_MEMORY_HIGH_BYTES`, `FCI1_DRIVER_UNIT` and the four `fci1_run_*` helpers. | medium-as-string carrier in the emit dag retires on the same trigger as the two shell files it feeds |

The two *target-language* shells (live_instrument, bounded_execution_controls) are the retirement
subject. The driver + calibration + context.env are their support and share the same dissolve-on
trigger, so a complete retirement removes all five together.

## 2. Caller graph (by module / symbol)

Every non-`.git` reference to each name, grouped by what actually *calls* it.

### `tools/fabric_ci_fci1_live_instrument.sh`
- **`gunbc.instruments.fabric_control_plane_live_probe`** — the entry the instrument drives.
  `FABRIC_CI_ENTRY` is set to `dag/gunbc/instruments/fabric_control_plane_live_probe.dag`; the
  script shells out to `gunbc run --entry <probe> --function <symbol>`. Symbols invoked by symbol:
  - transport producers (via `fabric_ci_capture_transport` / `capture_submitter_transport`):
    `fci1_live_reserve_and_observe_available` (×3: positive/generation/dependent),
    `fci1_live_cell_observation`, `fci1_live_allocation_prestate`, `fci1_live_release_with_receipt`,
    `fci1_live_release`
  - asserts (via `fabric_ci_run_assertion`, 24 calls): `fci1_invalid_before_wire_refuses`,
    `fci1_assert_slot_absent_admitted`, `fci1_assert_unreadable_slot_refused`,
    `fci1_assert_exact_absent_to_free_control`, `fci1_assert_exact_free_to_free_control`,
    `fci1_assert_extra_generation_refused_control`, `fci1_assert_frozen_work_key`,
    `fci1_assert_host_preconditions`, `fci1_assert_clean_cell_prestate`,
    `fci1_assert_cell_admitted`, `fci1_assert_allocation_available`, `fci1_assert_replace_held`,
    `fci1_assert_lifetime_held`, `fci1_assert_generation_changed`, `fci1_assert_checkpoint_token`,
    `fci1_assert_cell_unchanged`
  - receipts/writes: `fci1_grade_and_write_cell_receipt`, `fci1_write_zero_work_receipt`,
    `fci1_write_canonical_cleanup_receipt`, `fci1_write_disposable_cleanup_receipt`,
    `fci1_grade_and_write_allocation_receipt`, `fci1_write_generation_changed_receipt`,
    `fci1_write_lifetime_dependent_receipt`, `fci1_checkpoint_reached`,
    `fci1_write_checkpoint_terminal_receipt`
- **`gunbc.fabric.fci1_bounded_execution_context`** (the `.env` sibling) — sourced at line 14 for
  `FCI1_DRIVER_UNIT` / `FCI1_MEMORY_*` and `fci1_run_bounded_driver` / `fci1_run_bounded_submitter`
  / `fci1_run_bounded_dependent_submitter` (the `fci1_run_unbounded_memory_control` helper is
  present in the env but not used by this script).
- **`gunbc.instruments.fabric_ci_evidence`** — via the calibration it launches (below).
- **self** (line 26) — the outer process re-enters under the typed bounded envelope with
  `FCI1_BOUNDED_INNER=1`; not a cross-module caller.
- **prose** — `gunbc.rung_drop.fabric_evidence_gating` (the `FabricEvidenceGating` row) quotes the
  path in its authored text. Not a caller.

### `tools/fabric_ci_fci1_bounded_execution_controls.sh`
- **`gunbc.instruments.fabric_control_plane_live_probe`** — `run_pure_entry` array invokes
  `gunbc run --entry <probe> --function fci1_assert_checkpoint_token --arg token=none` as its
  pure entry (wrapped in the bounded cgroup check, the wrong-cwd and unbounded negative rows, and
  the exact-inner-status-86 row). Only symbol: `fci1_assert_checkpoint_token`.
- **`gunbc.fabric.fci1_bounded_execution_context`** — sourced for `FCI1_MEMORY_*`,
  `FCI1_DRIVER_UNIT`, `fci1_run_bounded_driver`, `fci1_run_bounded_submitter`,
  `fci1_run_unbounded_memory_control`.
- No other caller. (The live_instrument script does **not** reference the controls script.)

### `tools/fabric_ci_evidence_driver.sh`
- **`tools/fabric_ci_fci1_live_instrument.sh`** — `source`d; uses
  `fabric_ci_driver_init`, `fabric_ci_run_assertion`, `fabric_ci_capture_transport`.
- **`tools/fabric_ci_evidence_calibration.sh`** — `source`d; uses
  `fabric_ci_driver_init`, `fabric_ci_run_assertion`, `fabric_ci_observe_status`,
  `fabric_ci_capture_transport`.

### `tools/fabric_ci_fci1_bounded_execution_context.env`
- **generated by** `gunbc.fabric.fci1_bounded_execution_context_emit`
  (`dag/gunbc/fabric/fci1_bounded_execution_context_emit.dag`, the `Fci1BoundedExecutionContextEmission`
  / `expected_fci1_bounded_execution_context_env` path).
- **registered by** `gunbc.generated_artifact` (`Fci1BoundedExecutionContextArtifact`, the
  `ArtifactLocation { directory: "tools", name: "fabric_ci_fci1_bounded_execution_context.env" }`
  and the `decl_ref` to `expected_fci1_bounded_execution_context_env`).
- **merge attribute** — `.gitattributes` line 48: `merge=generated-artifact`.
- **conservation** — `src/v2/compiler.reference_conservation_census` lists the *witness* dag
  `dag/test/claim/fabric/fci1_bounded_execution_context_emit_witness.dag` in
  `reference_conservation_stratified_sample_paths` (the claim that the emitted content equals the
  expected). The `.env` file itself is a build artifact, not a census subject.
- **sourced by** `tools/fabric_ci_fci1_live_instrument.sh` and
  `tools/fabric_ci_fci1_bounded_execution_controls.sh` (the two target shells).

### `tools/fabric_ci_evidence_calibration.sh` (sibling, included for completeness)
- **`gunbc.instruments.fabric_ci_evidence`** — `FABRIC_CI_ENTRY` is set to
  `dag/gunbc/instruments/fabric_ci_evidence.dag`; the script realizes
  `fabric_ci_calibration_steps()` / `fabric_ci_calibration_write_plan` row by row.
- **`tools/fabric_ci_evidence_driver.sh`** — `source`d.
- **`tools/fabric_ci_fci1_live_instrument.sh`** — invoked directly (`bash .../fabric_ci_evidence_calibration.sh`)
  after the inner bounded admission, before any srv3 observation.
- **prose** — `gunbc.rung_drop.fabric_evidence_gating` quotes the path.

## 3. Process boundaries each control observes

The brief's boundary families are realized by the **calibration** against the evidence authority
(`gunbc.instruments.fabric_ci_evidence`), driven by the live_instrument's launch of the
calibration. The two live/controls shells then observe the *process* boundaries on srv3.

### Evidence-contract boundaries (calibration ↔ `gunbc.instruments.fabric_ci_evidence`)
Realized by the typed `FabricCiCalibrationStep` rows in `fabric_ci_calibration_steps()`:

| boundary family | step / operation | what it observes |
|-----------------|------------------|------------------|
| **value files vs diagnostics** | `diagnostic-not-channel` → `fabric_ci_calibration_diagnostic_forgery` (coordinate `diagnostic-only`) | a diagnostic string must NOT be readable through the value channel; the driver's `fabric_ci_capture_transport` refuses unless a real named value file exists with the `FCIE0X…` wire |
| **Alpha vs Beta transport** | `transport-assert` (alpha) + `transport-assert` (beta) + `transport-cross-red` (beta through alpha's assert) | Alpha and Beta are distinct transports; a Beta payload asserted through the Alpha assertion must red |
| **altered bytes** | `altered-byte-red` → `fabric_ci_calibration_transport_alpha` (coordinate `alpha-alter`) | flipping the final byte of a captured wire must red the transport assert (exact-bytes, not a prefix match) |
| **failed writes** | `write-failure-red` → `fabric_ci_calibration_transport_alpha` path `/proc/fci-evidence-write-refusal` | a producer whose value-file write fails must refuse, not emit |
| **ignored writes** | `ignored-write-red` → `fabric_ci_calibration_ignores_write_failure` (coordinate `ignored`) | a producer that silently ignores a write failure must be caught (no value file → no acceptance) |
| **projection-arm removal** | `delete-projection-arm-red` → subject `fabric_ci_evidence_projection_fixture.dag`, selector `FabricCiCalibrationBeta { payload: payload } =>` | deleting (or duplicating) the Beta projection arm in the fixture must make `fabric_ci_calibration_assertion_success` red — the two-arm projection is load-bearing |

Supporting refusals in the same plan: `fabric_ci_calibration_assertion_success` (status 0),
`fabric_ci_calibration_bool_must_refuse` (status 2),
`fabric_ci_calibration_raw_coproduct_must_not_cross` (status 2), and
`fabric_ci_calibration_collapsed_projection_must_red`.

### Process boundaries observed on srv3 (the two target shells)
These are the boundaries a *live* run observes, mapped to the families where they correspond:

- **bounded envelope / cgroup** (both shells, via the `fci1_run_*` helpers + `assert_unit_collected`):
  the driver/submitter units run under `FCI1_DRIVER_UNIT` with `MemoryMax`/`MemoryHigh` set to
  `FCI1_MEMORY_MAX_BYTES`/`FCI1_MEMORY_HIGH_BYTES`; wrong-`WorkingDirectory` and
  unbounded-`MemoryMax` rows must refuse (named `workspace root` / `process_workspace_root` and
  `HostBudgetUnreadable`). This is the "finite hard bound" the instrument re-enters under.
- **value vs diagnostic** (driver): `fabric_ci_run_assertion` tees to `FABRIC_CI_LOG` / stderr
  (diagnostics) but the *only* data path is `fabric_ci_capture_transport` reading an exact named
  value file (`FABRIC_CI_VALUE_ROOT/<coordinate>.wire`) with the `FCIE0X` wire; diagnostics are
  never data. The inner admission (live_instrument) re-checks `memory.max`/`memory.high` in the
  cgroup, the `WorkingDirectory`, and the driver's cgroup membership as independent fresh
  observations — "no value inherited from the outer marker is evidence".
- **value files / write lifetime** (live_instrument): reservation, release, generation and
  dependent-lifetime receipts are written to value files under the receipt root; the cleanup
  asserts `LoadState=not-found ActiveState=inactive` for each unit, the canonical allocation store
  is unchanged, and the `/run` disposable-root population is unchanged.
- **altered bytes / exact status** (controls): the exact-inner-status-86 row proves the bounded
  envelope surfaces the inner exit code unaltered.
- **projection / authority wire** (live_instrument, via `fci1_assert_*_control` and
  `fci1_assert_replace_held`): slot prestate and cleanup-authority wires (
  `fci1_allocation_prestate_wire`, `fci1_reservation_cleanup_authority_wire`) are exact; an extra
  generation or a wrong authority must refuse.

### Boundary that is NOT observed by any of these five
The **projection-arm removal** boundary's *subject* is the fixture
`fabric_ci_evidence_projection_fixture.dag`, which is mutated in-place by the calibration and
restored. None of the five shell files reads the fixture directly; only the calibration (via the
`fabric_ci_calibration_*` functions in `gunbc.instruments.fabric_ci_evidence`) touches it. So the
projection-arm-removal observation lives entirely inside the calibration's launch of the evidence
authority, not in the live instrument's reservation run.

## 4. Inside the required gate vs outside

The required gate is `gunbc.emitted_subject_build_gate.emitted_subject_build_rows` (the compiler
build steps in the required `witnesses` context). The FCI-1 lane was **rung-dropped 2026-09-04**
(operator ruling on runner capacity): the `fabric-evidence` job, the
`fabric_ci_evidence_calibration_step` step, and the `FABRIC_EVIDENCE` binding are absent from every
emitted workflow. `w_RED_the_deleted_lanes_do_not_return` guards that absence. This is recorded in
`gunbc.rung_drop.fabric_evidence_gating`.

Consequence for the caller graph:

- **OUTSIDE the required gate (all five shell files and their live callers):**
  - `tools/fabric_ci_fci1_live_instrument.sh` — hand-invoked by an operator on srv3; no CI trigger.
  - `tools/fabric_ci_fci1_bounded_execution_controls.sh` — hand-invoked; no CI trigger.
  - `tools/fabric_ci_evidence_driver.sh` — `source`d only by the two hand-invoked shells.
  - `tools/fabric_ci_evidence_calibration.sh` — hand-invoked / launched by the live instrument;
    its only remaining CI-era caller (the dropped `fabric-evidence` job) is gone.
  - Their target-language subjects `gunbc.instruments.fabric_control_plane_live_probe` and
    `gunbc.instruments.fabric_ci_evidence` are not members of `emitted_subject_build_rows`.
- **COMPILED into the required corpus, but not judged as a required verdict:**
  - `gunbc.generated_artifact` registers `Fci1BoundedExecutionContextArtifact`
    (`tools/fabric_ci_fci1_bounded_execution_context.env`).
  - `gunbc.fabric.fci1_bounded_execution_context_emit` is the emitter.
  - `src/v2.compiler.reference_conservation_census` (a compiler module the required path compiles)
    lists the *witness* `dag/test/claim/fabric/fci1_bounded_execution_context_emit_witness.dag` in
    its stratified sample paths.

  None of these is a *verdict* the required job runs: the required job on origin/main
  (`.github/workflows/witnesses.yml`) carries no generated-artifact drift step, and the required
  path compares no generated artifact (`gunbc.rung_drop v1_required_lanes_withdrawn`, DESIGN.md).
  The `GeneratedArtifactPhase` drift phase named in the `ci_spec.dag` roster model is **not
  executed by v2-bootstrap**.

**Critical for retirement — no required verdict catches a stray registration; enumerate consumers.**

- `reference_conservation_stratified_sample_paths` contains only `.dag` blobs; none of the five
  files is a census subject, so the reference-conservation claim is untouched by deleting any of
  them.
- The four shell files (`live_instrument.sh`, `bounded_execution_controls.sh`, `evidence_driver.sh`,
  `evidence_calibration.sh`) are **not** registered in `gunbc.generated_artifact`, and no required
  verdict reads them. Deleting them is invisible to the required path.
- The `.env` **is** registered: `Fci1BoundedExecutionContextArtifact` is
  `CommitRequired { consumer: HostReconciler }` (`generated_artifact.dag:260`). But the required job
  runs **no** generated-artifact drift step, and the required path compares no generated artifact
  (`gunbc.rung_drop v1_required_lanes_withdrawn`; the `GeneratedArtifactPhase` drift phase is in
  the `ci_spec.dag` roster model and is not executed by v2-bootstrap). **Deleting the `.env` while
  it stays registered would NOT go red — it passes silently.** That is exactly why the brief
  requires enumerating every consumer of the `.env`: a dangling registration arm, a stale
  producer decl_ref, a lingering `.gitattributes` merge line, or an orphaned census entry all
  survive the required gate unnoticed.
- Retirement must therefore remove the `.env` **and** its provenance in the **same** change: the
  registration arm (`generated_artifact.dag`), the producer decl_ref
  (`expected_fci1_bounded_execution_context_env`), the `.gitattributes` merge line, and the
  witness-dag census entry. Nothing on the required path refuses a merge if any one of them is left
  behind.

**No required verdict reads the `.env` at all** (three independent facts, verified against
origin/main):
- The conservation census sample
  (`v2.compiler.reference_conservation_census`
  `reference_conservation_stratified_sample_paths`, line 497) holds exactly
  `dag/test/claim/fabric/fci1_bounded_execution_context_emit_witness.dag` — a `.dag`, **not** the
  `.env`. The conservation check, if it runs, reads that dag's content, never the `.env`'s bytes.
- The fci1 emit-witness claim is **not** an admitted required-floor verdict:
  `v2.workflow.required_floor` `long_home_prefixes()` is six `*.long.` prefixes and the
  exact-grain admission list admits a fixed set of `test.claim.*` modules; neither
  `test.claim.fabric.fci1_bounded_execution_context_emit_witness` nor
  `v2.test.claim.namespace_xl0.reference_conservation` is admitted. (That witness reads the
  *registry*, not the file — deleting a file does not move the registration, so it cannot pin the
  file's presence regardless.)
- The required job (`.github/workflows/witnesses.yml`) runs no generated-artifact drift step and
  compares no generated artifact (`gunbc.rung_drop v1_required_lanes_withdrawn`).

Consequently the retirement change must be **self-consistent**: no in-gate check enforces that the
file, its registration, the producer decl_ref, the `.gitattributes` line and the census entry are
removed together. The only consumers that would notice a stray registration are the **non-required**
`gunbc.drift_watch_workflow` / `gunbc.heal_workflow` lanes (`gunbc.tier_o_artifact_emit`
`tier_o_artifacts`), which do not block a merge.

Nothing on the required path reads any member of the FCI-1 set: the conservation claim's subject is
the *witness dag* (not the `.env`, not the emit dag's emitted content) and it is not admitted to the
required floor; the emit dag's expected content is compared only by the drift phase that v2-bootstrap
does not execute; and the required job's steps never touch `tools/` artifacts. The only corpus
references to the FCI-1 set are the generated-artifact registration and the emit dag — the `.env`'s
own provenance — plus the documentary rung-drop prose row (`fabric_evidence_gating`).

## 5. Deletion set vs coupled surface (wave-1 input)

### 5.1 Dead with the shells (no consumer other than the shells)
- **The five files** — `tools/fabric_ci_fci1_live_instrument.sh`,
  `tools/fabric_ci_fci1_bounded_execution_controls.sh`, `tools/fabric_ci_evidence_driver.sh`,
  `tools/fabric_ci_evidence_calibration.sh`,
  `tools/fabric_ci_fci1_bounded_execution_context.env`.
- **Probe-dag `fci1_*` surface (79 of 81 exports)** of
  `gunbc.instruments.fabric_control_plane_live_probe`: every `fci1_*` fn/type exported by the probe
  dag has its only real consumer in the two shell files. (Orphaned but not harmful — the
  unused-declaration lens covers unused *parameters*, is not in the required floor, and an unused
  top-level fn is legal.)
- **Evidence-dag `fabric_ci_calibration_*` surface (27 of 34 exports)** of
  `gunbc.instruments.fabric_ci_evidence`: the calibration step plan, the transport/forgery/wire
  calibration fns and their coproduct arms have no consumer but the calibration shell.
- **The projection fixture** `fabric_ci_evidence_projection_fixture.dag` — imported only by the
  evidence dag (line 11); dies with it.
- **The emit dag** `gunbc.fabric.fci1_bounded_execution_context_emit` — its only consumers are the
  `generated_artifact_emit` / `tier_o_artifact_emit` arms that serve the `.env` registration
  (tier-o lanes, not required) and the `decl_ref` binding. Dies with the registration.
- **The `.env` provenance arms, all in one change** — `generated_artifact.dag` lines
  175 (`ArtifactLocation`), 260 (`CommitRequired { consumer: HostReconciler }`), 312 (flag table),
  390 (equality table), 468 (`decl_ref` to `expected_fci1_bounded_execution_context_env`), plus the
  `Fci1BoundedExecutionContextArtifact` decl itself; `.gitattributes` line 48
  (`merge=generated-artifact`); `reference_conservation_stratified_sample_paths` line 497 (the
  witness dag entry) — *if* the witness dag also retires (see 5.2: the census entry must be removed
  whenever the listed path stops existing, since a census whose sample path is missing is not
  self-consistent).

### 5.2 NOT dead — the coupled shared surface
- **The probe dag survives.** `fabric_control_plane_live_probe.dag` is a multi-family instrument
  (135 top-level exports across `cp_live_*`, `probe_*`, `nfr_*`, `fci1_*`). Two `fci1_*` decls have
  non-shell consumers and **must not be deleted with the shells**:
  - `fci1_assert_checkpoint_token` — driven by both shells AND named (as a string-literal entry
    name) by two tests in `src/v1/stage0/src/cli_run.rs` (lines ~3358, ~3404); the string literal is
    not a compile dependency, but the decl is the shared symbol.
  - `fci1_checkpoint_token` + the coproduct arms
    `Fci1CheckpointAfterCanonicalCommitBeforeTransport`,
    `Fci1CheckpointAfterReleaseBeforeGrading`, `Fci1CheckpointGenerationHeldTwo` — imported by
    `dag/test/claim/fabric/fabric_ci_fci1_allocation_lifecycle_witness.dag` (its `--function`
    invocation and the `Fci1Checkpoint` import).
  Wave 1 therefore either (a) trims the probe dag to its non-`fci1` surface plus the five shared
  checkpoint decls, trimming the now-dead `import gunbc.fabric.fabric_required_build_cell { fci1_allocation_slot_admission, fci1_exact_allocation_restoration }` import along with the orphaned fns
  that call them, or (b) migrates the lifecycle witness.
- **The lifecycle witness** `fabric_ci_fci1_allocation_lifecycle_witness.dag` imports probe-dag
  `fci1_*` decls AND `gunbc.fabric.fci1_bounded_execution_context` (`Fci1BoundedExecutionContext`,
  `fci1_bounded_execution_context`, `fci1_bounded_execution_context_witness`,
  `Fci1BoundedExecutionContextWit`). It is a non-required claim (not admitted to the required
  floor), but it is inside the corpus fold (`dag/test/claim/…` under the `dag` source root), so a
  dangling import after the probe trim **breaks the fold compile** — the real failure mode of the
  retirement change (no verdict goes red; the corpus just does not build). It must be retired or
  rewritten to the surviving surface in the same change.
- **The evidence-dag wire surface survives.** `fabric_ci_evidence_wire`,
  `decode_fabric_ci_evidence_wire`, `fabric_ci_write_wire`, `FabricCiEvidenceDecode`,
  `fabric_ci_shell_crossing_coordinates_hash` are imported by the probe dag (line 133) and the
  `fabric_ci_evidence_wire_decode_witness.dag` claim. (The `fabric_ci_calibration_assertion_success`
  mention in `materialization_kernel_closing_frontier_audit.dag` is a prose string literal in a
  `NotDelivered` reason, not a dependency.) Whether the evidence dag itself retires depends on
  whether the wire surface is still wanted post-retirement — wave-1 decision.
- **`gunbc.fabric.fci1_allocation_slot_admission` / `fci1_exact_allocation_restoration` live in
  `gunbc.fabric.fabric_required_build_cell`, a shared product module** (imported by
  `workspace_allocation_supply`, `runner_microvm_slot_controller` and their witnesses) — NOT in the
  probe dag. The probe's import of them is the probe's consumption of a shared module; trimming the
  probe's `fci1_allocation_*` fns drops that import but does not touch the source module.
- **The context-twin dag** `gunbc.fabric.fci1_bounded_execution_context` (+ its
  `fci1_bounded_execution_context_witness.dag`) is imported only by the lifecycle witness — dies
  with the witness unless the witness is kept.
- **The two `cli_run.rs` entry-name string literals** (`fci1_assert_checkpoint_token`) are not
  compile dependencies; they survive the file deletion but name the one probe decl that must stay.

### 5.3 Required-gate impact of the whole change set
None. No required verdict reads any member of this set (see §4); the required job compiles the
corpus and runs its own steps only. The retirement change must be **compile-consistent across the
fold** (witness imports, decl_refs, census sample paths, registration arms all removed or
re-pointed together) — that, not a verdict, is the failure mode. The non-required `drift_watch`/
`heal` lanes are the only consumers that would later flag a stray `.env` registration.

## 6. Open / needs confirmation
- Nothing blocked. This is read-only. Wave 1 (royal-otter-526) landing is what gates any *code*
  change; this inventory is the read-only prep and is independent of that.
- The `fci1_run_unbounded_memory_control` helper is defined in the context.env and used by the
  controls script (negative row) but not by the live instrument; a retirement that drops the env
  must keep the controls' negative rows covered by whatever replaces the helper.
