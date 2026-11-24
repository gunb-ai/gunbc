# Parse-Population Baseline Workloads

This document defines the two fixed benchmark populations for the
`emission-real-consumer-profiles` row.

Pinned SHA: `65230868362de3efc0ddfeeaeef5d83eeb2fa385` (origin/main at time of writing)

The row's brief references `56d460116d6252f6ec8dfe889ea358c804685861`; the top-170
selection is verified identical at both shas (`diff` empty).

## Selection rule

**Live roots.** The parse-population verb in CI walks
`--source-root dag --source-root src/v2` (witnesses.yml, the
"Walk parse-population reusing the nearest published ancestor's receipts"
step). These are the two live roots; `src/v1/` is the seed compiler's
hand-written source and is **out of scope** — the row profiles the
emitted v2 binary, not the v1 seed.

**Population universe.** All files matching `*.dag` under `dag/` and
`src/v2/` in the git tree at a pinned SHA.  At the pinned SHA this is
10,562 files (8,704 under `dag/`, 1,858 under `src/v2/`).

**Ranking.** Sort the population by **descending byte size** (blob size
from `git cat-file -s`). Ties broken by lexicographic path ascending for
determinism.

**Exclusions.** None beyond the root restriction.  Every `.dag` file in
the two roots is eligible.

**Population 1 — Five Largest.** Positions 1–5 in the sorted list.
These are the five largest inputs the parser must handle.  All five are
data-shaped or generated files (roadmap rows, receipt markings, Wikipedia
hardware tables, a generated roster).  This population measures the parser
on the largest inputs that exist, which is what the row asked for.  They
are not chosen for algorithmic interest.

**Population 2 — 165-Module Sample.** Positions 6–170 in the sorted list
(disjoint from Population 1).  This is the next 165 modules by descending
size, giving a sample that covers the upper size distribution while
remaining complementary to Population 1.  It contains a mix of code
(`src/v2/compiler/`, `src/v2/std/`) and data (`dag/`) modules; the mix is
the natural consequence of size ordering, not a selection for interest.

**Combined coverage.** 170 of 10,562 modules (1.6%).
Population 1: 5 modules, 4,575,463 bytes (avg 915,092).
Population 2: 165 modules, 29,960,631 bytes (avg 181,579).
Total: 34,536,094 bytes.  Combined: 139 from `dag/`, 31 from `src/v2/`.

## How to reproduce

```bash
SHA=65230868362de3efc0ddfeeaeef5d83eeb2fa385
git ls-tree -r --name-only "$SHA" -- dag/ src/v2/ \
  | grep '\.dag$' \
  | while read f; do
      echo "$(git cat-file -s "$SHA:$f") $f"
    done \
  | sort -t' ' -k1,1rn -k2,2 > all_dag_sorted.txt

# Population 1: lines 1-5
head -5 all_dag_sorted.txt

# Population 2: lines 6-170
sed -n '6,170p' all_dag_sorted.txt
```

## Population 1: Five Largest (positions 1–5)

| # | Bytes | Path |
|---|-------|------|
| 1 | 1421761 | `dag/gunbc/roadmap/roadmap_authority.dag` |
| 2 | 947773 | `dag/gunbc/roadmap/roadmap_belt_actuate.dag` |
| 3 | 750634 | `dag/product/p4261/e1_receipt_markings.dag` |
| 4 | 748625 | `dag/extdeps/wikipedia/list_of_nvidia_graphics_processing_units.dag` |
| 5 | 706670 | `src/v2/workflow/floor_grandfathered_roster.dag` |

## Population 2: 165-Module Sample (positions 6–170)

| # | Bytes | Path |
|---|-------|------|
| 6 | 662404 | `src/v2/compiler/body_lowering_fold.dag` |
| 7 | 648213 | `src/v2/std/compilers/target_model.dag` |
| 8 | 642229 | `dag/gunbc/host/host_effect_realize.dag` |
| 9 | 631378 | `src/v2/compiler/infer.dag` |
| 10 | 597494 | `dag/gunbc/fleet/fleet_converge_workflow.dag` |
| 11 | 571378 | `dag/extdeps/wikipedia/list_of_amd_graphics_processing_units.dag` |
| 12 | 560090 | `dag/gunbc/recurring_failure_mode/bare_name_resolved_by_proximity_standing_in_for_resolution.dag` |
| 13 | 528365 | `dag/extdeps/nvidia/linux_driver_supported_chips.dag` |
| 14 | 496502 | `dag/product/p4261/e1_b.dag` |
| 15 | 437107 | `dag/product/p4261/e1_board_sightings.dag` |
| 16 | 375922 | `src/v2/extdeps/languages/dag.dag` |
| 17 | 368266 | `src/v2/workflow/roster_import_sweep.dag` |
| 18 | 357665 | `dag/gunbc/runner/runner_host_file_converge.dag` |
| 19 | 345680 | `dag/test/claim/ubuntu_autoinstall_seed_witness.dag` |
| 20 | 329294 | `dag/gunbc/ci/ci_layer_roots.dag` |
| 21 | 321770 | `src/v2/compiler/resolve.dag` |
| 22 | 321632 | `src/v2/compiler/source_root_eval.dag` |
| 23 | 321452 | `dag/gunbc/machine_intake/bmc_recovery_route.dag` |
| 24 | 315112 | `dag/extdeps/cpu/ampere_altra_package_table4_raw.dag` |
| 25 | 308544 | `dag/gunbc/design_ruling.dag` |
| 26 | 303095 | `dag/gunbc/compute/work_provider_local.dag` |
| 27 | 297544 | `dag/gunbc/provisioning/ubuntu_autoinstall_seed.dag` |
| 28 | 292814 | `src/v2/workflow/local_repo_wet_terminal.dag` |
| 29 | 291271 | `dag/gunbc/machine_intake/bmc_secure.dag` |
| 30 | 278925 | `dag/gunbc/scm/repository_envelope.dag` |
| 31 | 272046 | `dag/gunbc/live_deploy/srv1_residue_rehearsal.dag` |
| 32 | 238709 | `dag/gunbc/roadmap/roadmap_dashboard_instance.dag` |
| 33 | 236738 | `dag/test/claim/product/p4261_carrier_layout_witness.dag` |
| 34 | 236386 | `dag/gunbc/spark/arm_memory_fit.dag` |
| 35 | 230901 | `dag/gunbc/machine_intake/mtcollins1_boot_run.dag` |
| 36 | 228849 | `dag/gunbc/machine_intake/megarac_boot_diagnostic_bundle.dag` |
| 37 | 218906 | `dag/gunbc/runner/runner_gpu_container_toolchain.dag` |
| 38 | 217939 | `dag/gunbc/fleet/site_pxe_edge_converge.dag` |
| 39 | 215655 | `src/v2/compiler/parse.dag` |
| 40 | 214558 | `dag/test/claim/firmware_network_boot_standing_witness.dag` |
| 41 | 213007 | `dag/gunbc/roadmap/roadmap_page.dag` |
| 42 | 211221 | `dag/gunbc/roadmap/roadmap_dispatch_actuator.dag` |
| 43 | 204619 | `dag/test/claim/long/roadmap_page_witness.dag` |
| 44 | 201833 | `src/v2/extdeps/languages/rust.dag` |
| 45 | 201652 | `src/v2/workflow/floor_route_gap.dag` |
| 46 | 193318 | `dag/gunbc/fleet/fleet_converge_plan.dag` |
| 47 | 192268 | `dag/gunbc/roadmap/roadmap_event_log.dag` |
| 48 | 192156 | `dag/gunbc/firmware_network_boot_standing.dag` |
| 49 | 190608 | `dag/gunbc/compute/remote_build_serving_converge.dag` |
| 50 | 187914 | `dag/gunbc/roadmap/roadmap_component.dag` |
| 51 | 186139 | `dag/test/claim/roadmap/roadmap_belt_actuate_witness.dag` |
| 52 | 183407 | `dag/test/claim/machine_intake/mtcollins1_boot_acceptance_matrix.dag` |
| 53 | 181414 | `dag/gunbc/machine_intake/bmc_first_contact_reading.dag` |
| 54 | 181053 | `src/v2/workflow/required_floor.dag` |
| 55 | 180757 | `dag/gunbc/serving/srv1_serving_replica_layout.dag` |
| 56 | 177226 | `dag/gunbc/auth/privileged_effect_registry.dag` |
| 57 | 176113 | `dag/gunbc/runner/runner_slot_allocation.dag` |
| 58 | 174168 | `dag/gunbc/fabric/engram_materialization.dag` |
| 59 | 171535 | `dag/gunbc/spark/pair_serving_authority_log.dag` |
| 60 | 169348 | `dag/gunbc/instruments/known_red_derivation.dag` |
| 61 | 166853 | `dag/gunbc/witness/witness_floor_workflow.dag` |
| 62 | 166629 | `dag/gunbc/instruments/fabric_control_plane_live_probe.dag` |
| 63 | 162998 | `src/v2/extdeps/languages/rust_fixtures.dag` |
| 64 | 161158 | `dag/gunbc/machine_intake/megarac_media_attach.dag` |
| 65 | 160464 | `dag/gunbc/roadmap/roadmap_serve.dag` |
| 66 | 158795 | `src/v2/workflow/floor_pure_producer_share.dag` |
| 67 | 158710 | `dag/product/p4261/e1_photographs.dag` |
| 68 | 158493 | `dag/test/claim/deployment_gate_operator_steps_witness.dag` |
| 69 | 158356 | `dag/test/claim/fleet/fleet_converge_plan_witness.dag` |
| 70 | 157785 | `dag/test/claim/compute/remote_build_serving_converge_witness.dag` |
| 71 | 157446 | `dag/gunbc/roadmap/roadmap_session_placement.dag` |
| 72 | 155814 | `dag/extdeps/realization/materialization_store_local.dag` |
| 73 | 152188 | `src/v2/compiler/eval.dag` |
| 74 | 151441 | `dag/gunbc/harness/harness_turn.dag` |
| 75 | 151208 | `dag/test/claim/fleet/site_pxe_edge_converge_witness.dag` |
| 76 | 151123 | `dag/test/claim/runner/runner_host_file_converge_witness.dag` |
| 77 | 150577 | `dag/gunbc/live_deploy/emit.dag` |
| 78 | 150025 | `src/v2/std/grammar.dag` |
| 79 | 149175 | `dag/gunbc/fleet/fleet_release_bins_key.dag` |
| 80 | 148257 | `src/v2/std/node.dag` |
| 81 | 147953 | `dag/gunbc/roadmap/roadmap_dashboard_instance_apply.dag` |
| 82 | 146355 | `dag/gunbc/spark/runtime_architecture_registry_reading.dag` |
| 83 | 146143 | `dag/gunbc/auth/deployment_gate_authorization.dag` |
| 84 | 143044 | `dag/test/claim/spark/engram_materialization_witness.dag` |
| 85 | 142480 | `src/v2/extdeps/languages/bash.dag` |
| 86 | 141986 | `dag/gunbc/harness/harness_seat.dag` |
| 87 | 140540 | `dag/gunbc/live_deploy/apply_legs.dag` |
| 88 | 138686 | `dag/extdeps/git.dag` |
| 89 | 137370 | `dag/test/claim/machine_intake/machine_intake_arrival_converge_witness.dag` |
| 90 | 135855 | `dag/std/materialization_provider.dag` |
| 91 | 135557 | `dag/gunbc/commit_workflow.dag` |
| 92 | 135040 | `dag/gunbc/package_delivery.dag` |
| 93 | 132233 | `dag/gunbc/roadmap/roadmap_decomposition.dag` |
| 94 | 130660 | `dag/test/claim/roadmap/session_placement_wet_witness.dag` |
| 95 | 130529 | `dag/gunbc/auth/gcp_iam_bootstrap.dag` |
| 96 | 129692 | `dag/test/claim/runner/runner_host_cap_bracket_witness.dag` |
| 97 | 127454 | `dag/extdeps/electronic_component/intel_10cx085f672_boundary_scan.dag` |
| 98 | 127408 | `src/v2/workflow/floor_cost_debt.dag` |
| 99 | 126710 | `dag/gunbc/plans/demand_engine_program.dag` |
| 100 | 126241 | `src/v2/std/compilers/semantic_decl_emission.dag` |
| 101 | 125288 | `src/v2/workflow/reference_migration.dag` |
| 102 | 125238 | `dag/gunbc/floor/floor_demand.dag` |
| 103 | 124833 | `dag/gunbc/self_host_compile_phase_frontier.dag` |
| 104 | 124487 | `src/v2/compiler/module_acquisition.dag` |
| 105 | 123443 | `dag/test/claim/roadmap/roadmap_presentation_witness.dag` |
| 106 | 123021 | `dag/gunbc/fleet/host_credential_custody_converge.dag` |
| 107 | 122956 | `dag/gunbc/scm/object_store.dag` |
| 108 | 122819 | `dag/gunbc/compute/remote_build_serving_setup.dag` |
| 109 | 122136 | `dag/gunbc/machine_intake/mtcollins1_ui_status_asset_observation.dag` |
| 110 | 121638 | `src/v2/extdeps/languages/typescript.dag` |
| 111 | 121508 | `dag/gunbc/witness/compiler_gate_contract_stamp.dag` |
| 112 | 120416 | `dag/gunbc/site/web_fonts.dag` |
| 113 | 118672 | `dag/gunbc/roadmap/roadmap_presentation.dag` |
| 114 | 116098 | `dag/std/occurrence_binding_candidates.dag` |
| 115 | 115465 | `dag/gunbc/instruments/router_proxy_falsifier.dag` |
| 116 | 113274 | `dag/gunbc/design_document.dag` |
| 117 | 113019 | `dag/gunbc/emit_subject_clean_frontier.dag` |
| 118 | 112625 | `dag/test/claim/roadmap/roadmap_authority.dag` |
| 119 | 112316 | `dag/test/claim/machine_intake/mtcollins1_boot_run_witness.dag` |
| 120 | 112277 | `dag/test/claim/gunbc_instruments_known_red_derivation_witness.dag` |
| 121 | 111533 | `src/v2/workflow/floor_expected_red.dag` |
| 122 | 111530 | `dag/test/claim/scm/scm_repository_envelope_witness.dag` |
| 123 | 111396 | `dag/product/printed_chassis/card_cooling_route.dag` |
| 124 | 111262 | `dag/gunbc/live_deploy/spec.dag` |
| 125 | 110449 | `dag/test/claim/review_sheet_converge_witness.dag` |
| 126 | 110125 | `dag/gunbc/spark/serving_arm.dag` |
| 127 | 108099 | `dag/test/claim/machine_intake/mtcollins1_ui_status_asset_observation_witness.dag` |
| 128 | 107422 | `dag/test/claim/dispatch_credential_acquisition_witness.dag` |
| 129 | 107398 | `dag/gunbc/machine_intake/arrival_converge.dag` |
| 130 | 107238 | `dag/test/claim/machine_intake/megarac_media_convergence_witness.dag` |
| 131 | 106297 | `dag/gunbc/floor_cost_distribution.dag` |
| 132 | 105968 | `src/v2/compiler/target_serialize.dag` |
| 133 | 105835 | `dag/gunbc/live_deploy/apply.dag` |
| 134 | 105301 | `dag/gunbc/roadmap/roadmap_effort_estimate.dag` |
| 135 | 104727 | `dag/gunbc/runner/runner_host_cap_bracket.dag` |
| 136 | 102662 | `dag/test/claim/roadmap/worker_git_subtree_apply_spawn_wet_witness.dag` |
| 137 | 102639 | `dag/test/claim/host/host_name_resolution_witness.dag` |
| 138 | 102596 | `dag/gunbc/github_app_acquisition.dag` |
| 139 | 102421 | `dag/gunbc/recurring_failure_mode/accepted_source_emits_uncompilable_target.dag` |
| 140 | 102271 | `dag/gunbc/fleet/fleet_converge_plan_cli.dag` |
| 141 | 102155 | `dag/test/claim/roadmap/worker_dedicated_account_witness.dag` |
| 142 | 101858 | `dag/test/claim/fleet/runtime_identity_match_witness.dag` |
| 143 | 101327 | `dag/gunbc/machine_intake/prior_life_boundary.dag` |
| 144 | 100867 | `src/v2/compiler/translate.dag` |
| 145 | 98992 | `dag/test/claim/github_app_manifest_witness.dag` |
| 146 | 98389 | `dag/test/claim/floor_cost_distribution_witness.dag` |
| 147 | 98304 | `dag/test/claim/machine_intake/megarac_boot_diagnostic_bundle_witness.dag` |
| 148 | 98271 | `dag/gunbc/explicit_witness_admission.dag` |
| 149 | 97965 | `dag/product/printed_chassis/gpu_duct.dag` |
| 150 | 97728 | `dag/gunbc/serving/router_dev_policy.dag` |
| 151 | 97506 | `dag/test/claim/machine_intake/megarac_boot_record_mapping_witness.dag` |
| 152 | 97431 | `dag/gunbc/fleet/fleet_known_hosts_anchor.dag` |
| 153 | 97158 | `dag/test/claim/deployment_gate_authorization_witness.dag` |
| 154 | 96976 | `dag/test/claim/live_deploy/emit.dag` |
| 155 | 96945 | `src/v2/compiler/source_authority.dag` |
| 156 | 96430 | `dag/test/claim/host/approval_broker_tree_install_witness.dag` |
| 157 | 95650 | `dag/gunbc/site/element/build_meter.dag` |
| 158 | 95582 | `dag/gunbc/roadmap/roadmap_event_carrier.dag` |
| 159 | 95568 | `dag/test/claim/network_boot_delivery_join_witness.dag` |
| 160 | 95247 | `dag/std/algebra.dag` |
| 161 | 95210 | `dag/gunbc/roadmap/roadmap_style.dag` |
| 162 | 94832 | `src/v2/test/claim/native_route_progress_guard_witness.dag` |
| 163 | 94159 | `src/v2/workflow/native_route_progress_guard.dag` |
| 164 | 94155 | `dag/gunbc/compute/host_capacity.dag` |
| 165 | 94133 | `dag/std/measure.dag` |
| 166 | 94001 | `dag/test/claim/machine_intake/bmc_first_contact_reading_witness.dag` |
| 167 | 93103 | `dag/test/claim/spark/pair_serving_authority_log_witness.dag` |
| 168 | 92781 | `dag/gunbc/v1/v1_interpreter_primitive_surface.dag` |
| 169 | 92614 | `dag/gunbc/site/gunb_ai_css.dag` |
| 170 | 92576 | `src/v2/extdeps/languages/swift.dag` |

## Notes

- **All five of Population 1 are data-shaped.** Four are tables or
  rosters generated from external data (Wikipedia GPU lists, product
  receipt markings) and one is a generated floor roster.  They measure
  the parser under maximum input pressure, not under algorithmic variety.
  This is what the row's first_slice specified ("the 5 largest dag
  modules").
- Population 2's mix (135 dag/ + 30 src/v2/) arises from the size ordering
  and is not curated.  It includes compiler-internal code modules
  (`body_lowering_fold`, `infer`, `source_authority`) alongside data
  modules, which gives better algorithmic coverage than Population 1.
- The 165-module count matches neither fleet shard counts nor any CI
  parameter.  It is the row's own declaration; this document makes it
  reproducible.
