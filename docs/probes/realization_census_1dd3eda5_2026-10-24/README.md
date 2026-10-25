# Realization-decision census

Pin `1dd3eda5696a` (origin/main, 2026-10-24). This census classifies; it does not redesign or fix anything.

The question it answers: **how much of gunbc decides for itself where its data lives, what it caches, how it reaches a resource, what authority it acts with, and which host it runs on**, instead of receiving those facts from the one place that should decide them.

## How to read the files

| file | one row per | identity column |
|---|---|---|
| `classes.tsv` | error class (read this first) | `class_id` |
| `modules.tsv` | hand-read module | `module` |
| `findings.tsv` | instance: one decision at one symbol | `finding_id` |
| `receipt.json` | the run: pin, script hashes, slice commits, populations, agreement figures | — |
| `method.md` | how every number was produced, every merge ruling, every caveat | — |

Each dimension has a **decision site**:

| site | meaning | problem? |
|---|---|---|
| None | the module decides nothing in this dimension | no |
| Home | the module is the one declared home of the fact | no |
| Passed | the module receives the fact as a parameter | no |
| Rederived | it recomputes or re-selects a fact it could have been handed | yes, mild |
| Literal | it hard-codes the fact (path, host, user, port, env name) with no home behind it | yes |
| Forked | it re-implements a shared mechanism beside the library (its own write, lock, mkdir, sweep, memo) | yes |

The L number (L0 to L5) is kept beside each site for sorting. A module's site in a dimension is its worst instance there. Every problem cell is backed by named rows in `findings.tsv`, except 7 reader cells listed in `merge_report.txt`.

## The reading, in plain language

**Most of the problem is storage, and most of storage is the same three habits.**

- Kept records are written with a bare file write where a store library already exists (`storage_raw_state_beside_library`: 57 modules, 135 instances).
- Store roots are spelled out as literal paths: `/var/lib/gunbc/...` appears in 22 modules (`storage_root_literal`: 62 modules).
- Receipts go to repo-relative `target/...` paths for which no home exists at all (`storage_relative_receipt_path`: 55 modules).

Beside these, modules make their own directories, delete their own trees and build their own create-once locks (`storage_hand_directory_setup`, `storage_hand_retention_sweep`, `storage_hand_exclusion`: 34, 31 and 37 modules).

Raw `Filesystem.Write` is the single most duplicated mechanism in the corpus: 79 modules.

**Placement is the second problem, and it is two habits.**

- Hosts, units and ports are named literally: `srv1` in 20 modules; unit names and `/etc/systemd/system/` in 35 (`placement_host_literal`, `placement_unit_name_or_dir`).
- An entry point's instance or host is re-selected further down, the class behind the wrong-instance bug: `srv1_live_dashboard_instance()` wrappers, one function per host, `if host == operator_host_srv1` guards (`placement_instance_repick`: 45 modules). The brief's 26 wrapper pins are a subset of these. The roadmap slice found about 15 more.

**Access is mostly ambient environment and literal secret paths.**

- Modules call `shell.Env.Get` directly to find a root, token or identity: 38 modules.
- Credential and key locations are written as literal paths, some beside a `SecretRef` for the same secret (`access_credential_location`: 37 modules).
- Kernel-interface paths (`/proc`, `/sys`) are read outside the `extdeps` models of those interfaces (`access_kernel_abi_path`: 15 modules).

**Authorization is small, but it is ambient root.**

- 12 modules elevate with `sudo` in place (`authz_ambient_sudo`) rather than receiving a capability.
- `gunbc.auth.privileged_effect_census` already classifies such sites, which is why that class sits at rung 1 rather than 0; it refuses none of them.

**Caching is a seed problem.**

- Production `.dag` code barely caches by hand (2 modules).
- The hand-written Rust seed has process-wide memo tables with hand-picked keys and no invalidation in 21 files (`caching_process_memo_table`).
- One emitter template wraps every generated constant in a memo: one finding, not 90 (`caching_generated_constant_memo`, `src/v1/05_emit_rust::emit_data_def`).

**The extdeps layer is mostly clean.** 70 of the 123 extdeps modules read are the faithful home of an upstream fact. 9 have a problem:

- 5 carry our own policy inside an upstream model (install paths under `/var/lib/gunbc/bin`, `/opt/actions-runner`, the `srv3` install-actuator label, the sccache install location);
- 3 hand-roll a temp-directory lifecycle;
- 1 re-spells its own binary path.

**By module class:**

| class | hand-read | with a problem | worst site Rederived / Literal / Forked |
|---|---:|---:|---|
| production | 681 of 2,226 | 318 | 31 / 187 / 100 |
| extdeps | 123 of 1,129 | 9 | 1 / 5 / 3 |
| seed (hand-written) | 71 of 315 | 42 | 0 / 2 / 40 |
| seed-generated | 90 | 0 | one template finding |
| compiler, instrument, ledger, test, roster | counted, not read | — | — |

## The shape of the fix, per decision site (the brief's wording, not a design)

- **Rederived: receive the value instead of re-deriving it.** The caller already holds the instance, store or host, so passing it removes the second derivation.
- **Literal: declare once and consume.** The fact gets one home (an existing authority where there is one, named in `owning_authority`) and every other spelling of it goes away. Where `owning_authority` says "none exists", the missing home is named and nothing is invented.
- **Forked: adopt the library and delete.** The module's own write, lock, mkdir, sweep or memo is replaced by the store, CAS, directory or materialization library it sits beside.

`classes.tsv` carries, per class:

- the current rung on DESIGN.md §4b's ladder;
- the attainable ceiling, with reason;
- the wall that would hold it (an `admit_callers` roster, a type, a declared `requires`, or derivation from one declaration);
- the red control that would prove that wall;
- the residual, and the next trigger.

## What this census does not say

- **A rung of 0 means "no wall observed by this census", not "proved unguarded".** The census looked for decision sites, not for gates that already refuse them.
- **The hand-read depth is uneven.** The large modules in slices 03, 04, 05, 06 and 09 were read by targeted search around every lead plus a declaration scan, not line by line. Their None/Home/Passed cells are lower confidence than their problem cells. Slice 06's specimens describe the mechanism and name the calls rather than quoting lines. `method.md` lists the depth per slice.
- **The no-signal remainder is bounded, not cleared.** All 82 modules matching the two shapes the mechanics missed were read: 12 have a problem. For the other 1,551 no-signal production modules, a sample of 51 found none, so their miss rate is under about 6% at 95% confidence. That is up to roughly 90 unread modules, of unknown shape.
- **The worst-module order is a reading order, not a severity model.** It sums problem levels across dimensions.
- **Seeds were recomputed, not copied.** The parent's precomputed seeds were unreachable across hosts, so every lead here is recomputed at the pin, and the cross-check against those seeds was never done.


---

# Generated tables (scripts/readme_tables.py)

## Counts: modules per decision site, per dimension, per module class

### production (681 modules)

| dimension | None | Home | Passed | Rederived | Literal | Forked |
|---|---:|---:|---:|---:|---:|---:|
| storage | 363 | 48 | 64 | 2 | 105 | 99 |
| caching | 653 | 12 | 11 | 1 | 2 | 2 |
| access | 505 | 11 | 71 | 13 | 80 | 1 |
| authz | 606 | 21 | 37 | 9 | 8 | 0 |
| placement | 322 | 73 | 153 | 37 | 96 | 0 |
| **worst of five** | 140 | 106 | 117 | 31 | 187 | 100 |

### extdeps (123 modules)

| dimension | None | Home | Passed | Rederived | Literal | Forked |
|---|---:|---:|---:|---:|---:|---:|
| storage | 77 | 36 | 3 | 0 | 4 | 3 |
| caching | 93 | 28 | 2 | 0 | 0 | 0 |
| access | 104 | 3 | 16 | 0 | 0 | 0 |
| authz | 117 | 3 | 3 | 0 | 0 | 0 |
| placement | 76 | 41 | 3 | 1 | 2 | 0 |
| **worst of five** | 25 | 70 | 19 | 1 | 5 | 3 |

### seed (71 modules)

| dimension | None | Home | Passed | Rederived | Literal | Forked |
|---|---:|---:|---:|---:|---:|---:|
| storage | 48 | 0 | 1 | 0 | 0 | 22 |
| caching | 46 | 0 | 3 | 0 | 0 | 22 |
| access | 56 | 0 | 6 | 0 | 8 | 1 |
| authz | 71 | 0 | 0 | 0 | 0 | 0 |
| placement | 59 | 0 | 0 | 0 | 12 | 0 |
| **worst of five** | 23 | 0 | 6 | 0 | 2 | 40 |

### seed-generated (90 modules)

| dimension | None | Home | Passed | Rederived | Literal | Forked |
|---|---:|---:|---:|---:|---:|---:|
| storage | 90 | 0 | 0 | 0 | 0 | 0 |
| caching | 90 | 0 | 0 | 0 | 0 | 0 |
| access | 90 | 0 | 0 | 0 | 0 | 0 |
| authz | 90 | 0 | 0 | 0 | 0 | 0 |
| placement | 90 | 0 | 0 | 0 | 0 | 0 |
| **worst of five** | 90 | 0 | 0 | 0 | 0 | 0 |

### roster (3 modules)

| dimension | None | Home | Passed | Rederived | Literal | Forked |
|---|---:|---:|---:|---:|---:|---:|
| storage | 3 | 0 | 0 | 0 | 0 | 0 |
| caching | 3 | 0 | 0 | 0 | 0 | 0 |
| access | 3 | 0 | 0 | 0 | 0 | 0 |
| authz | 3 | 0 | 0 | 0 | 0 | 0 |
| placement | 3 | 0 | 0 | 0 | 0 | 0 |
| **worst of five** | 3 | 0 | 0 | 0 | 0 | 0 |

## Instances per dimension and decision site (hand-read classes)

| dimension | Rederived | Literal | Forked | modules with any |
|---|---:|---:|---:|---:|
| storage | 5 | 155 | 339 | 234 |
| caching | 1 | 2 | 76 | 27 |
| access | 18 | 126 | 2 | 103 |
| authz | 9 | 8 | 0 | 16 |
| placement | 46 | 138 | 0 | 143 |

## The 25 worst production modules (of 318 with any problem)

Ordered by the sum of their problem levels across the five dimensions (Rederived 3, Literal 4, Forked 5), then by instance count. This is a reading order, not a severity model.

| # | module | problem dimensions | instances | worst finding |
|---:|---|---|---:|---|
| 1 | `gunbc.roadmap.roadmap_belt_actuate` | storage Forked, access Literal, authz Literal, placement Literal | 8 | storage Forked: `belt_tick_receipt_write_for_instance` about 30 direct Filesystem.Write persist belt state files with no store library |
| 2 | `gunbc.host.host_effect_realize` | storage Forked, access Literal, authz Rederived, placement Literal | 12 | storage Forked: `Srv3BmcwebTokenExtract arm (realize_bmcweb_token_extract_bod` Filesystem.Write(path: srv3_bmcweb_token_path, content: token) |
| 3 | `gunbc.auth.approval_ntfy_converge` | storage Literal, access Literal, authz Literal, placement Literal | 5 | access Literal: `ntfy_publisher_token_path` publisher token and key read by literal /etc paths |
| 4 | `gunbc.auth.approval_device_redemption` | storage Literal, caching Rederived, authz Rederived, placement Rederived | 5 | storage Literal: `approval_device_root` /var/lib/gunbc/approval-devices |
| 5 | `gunbc.fabric.workspace_source` | storage Forked, access Literal, placement Literal | 4 | storage Forked: `workspace_pack_archive` shell.Mkdir.Parents(path: placeholder_root as FilePath); Filesystem.Write(path: list_path, |
| 6 | `gunbc.fleet.fleet_host_key_enrollment` | storage Forked, access Literal, placement Literal | 4 | storage Forked: `fleet_host_key_enroll_apply;self_enroll_apply` Filesystem.WriteCreateNew(path: cfg); mkdir_parents_command; chmod_set_mode_command; Files |
| 7 | `gunbc.fabric.workspace_checkpoint_instrument` | storage Forked, access Literal, placement Literal | 3 | storage Forked: `workspace_restore_after_barrier` shell.Remove.RecursiveForce(path: dir as FilePath); workspace_write_authored: shell.Mkdir. |
| 8 | `gunbc.harness.harness_operator_route` | storage Forked, access Literal, placement Literal | 3 | storage Forked: `operator_open_header` hand-rolled bearer-header lifecycle (Mktemp.Dir+WriteOwnerOnly+Remove.RecursiveForce) |
| 9 | `gunbc.runner.runner_revision_job_main` | storage Forked, access Literal, placement Literal | 3 | storage Forked: `revision_job_await` Filesystem.WriteCreateNew(path: results_out / runtime_out, content: ...) |
| 10 | `gunbc.package_delivery` | storage Forked, access Literal, placement Rederived | 6 | storage Forked: `append_text_to_file` Filesystem.Read then Filesystem.Write(existing+extra) |
| 11 | `gunbc.runner.runner_microvm_slot_unit` | storage Literal, access Literal, placement Literal | 5 | access Literal: `workspace_launch_credential_source_path` join(["/etc/credstore/", workspace_launch_credential_prefix, instance]) |
| 12 | `gunbc.fleet.fleet_multi_principal_probe` | storage Literal, access Literal, placement Literal | 4 | access Literal: `multi_principal_probe_principals` ["ubuntu", "briansrls", "ghrunner", "root", "gunbc-automation"] |
| 13 | `gunbc.fleet.host_credential_custody_converge` | storage Literal, access Literal, placement Literal | 4 | access Literal: `custody_mount_table_path` /proc/self/mountinfo; argv ["cat", path] |
| 14 | `gunbc.fleet.organization_runner_roster_read` | storage Literal, access Literal, placement Literal | 4 | access Literal: `gh_token_env_name` shell.Env.Get("GH_TOKEN") beside org_admin_token_env_name |
| 15 | `gunbc.runner.runner_microvm_network` | storage Literal, access Literal, placement Literal | 4 | access Literal: `runner_microvm_ip_forward_proc_path` /proc/sys/net/ipv4/ip_forward |
| 16 | `gunbc.runner.runner_microvm_network_converge` | storage Literal, access Literal, placement Literal | 4 | access Literal: `converge_generation_reading` shell.Env.Get(name: github_run_id_variable_name) |
| 17 | `gunbc.actions_run_binding` | storage Literal, access Literal, placement Literal | 3 | access Literal: `OWNER_ENV` GITHUB_REPOSITORY_OWNER |
| 18 | `gunbc.auth.branch_run_grant_broker` | access Literal, authz Literal, placement Literal | 3 | access Literal: `branch_run_grant_executor_host` srv1 |
| 19 | `gunbc.fleet.fleet_converge_plan` | storage Literal, authz Literal, placement Literal | 3 | authz Literal: `timer_retirement_disable_line;timer_retirement_remove_line;f` sudo -n systemctl disable --now ; sudo -n rm -f /etc/systemd/system/ ; sudo -n -u |
| 20 | `gunbc.srv3.srv3_actuate_shell_paths` | storage Literal, access Literal, placement Literal | 2 | access Literal: `srv3_bmcweb_token_path` /tmp/srv3_bmcweb_token |
| 21 | `gunbc.auth.approval_gate` | storage Forked, access Rederived, placement Rederived | 3 | storage Forked: `approval_submit` ambient Filesystem.Read/WriteOwnerOnly/Delete of the submission body file |
| 22 | `gunbc.auth.approval_keyring_converge` | storage Literal, authz Literal, placement Rederived | 3 | authz Literal: `sudo_elevate` 2 direct sudo_elevate sites without a received grant row |
| 23 | `gunbc.auth.deployment_gate_authorization` | access Literal, authz Literal, placement Rederived | 3 | access Literal: `srv1_tailscale_https_url` tailscale relay URL literal + DEPLOYMENT_GATE_TOKEN env-name literal |
| 24 | `gunbc.auth.gcp_iam_bootstrap` | storage Literal, access Literal, authz Rederived | 3 | access Literal: `gcp_iam_bootstrap_journal_save` direct GCP IAM/SM calls through imported extdeps, no injected seam |
| 25 | `gunbc.host.host_reach_identity_probe` | storage Literal, access Literal, placement Rederived | 3 | access Literal: `probe_fleet_reach_wet` shell.Env.Get(name: "HOME") |

## The 9 worst extdeps modules (of 9 with any problem)

| # | module | problem dimensions | instances | worst finding |
|---:|---|---|---:|---|
| 1 | `extdeps.bmc.os_install_actuator_toolchain` | storage Literal, placement Literal | 2 | placement Literal: `data srv3_os_install_actuator_host_label` install-actuator-host |
| 2 | `extdeps.printing.orca_slicer` | storage Forked | 1 | storage Forked: `fn orca realization (temp.path; Remove.RecursiveForce)` shell.Mktemp.Dir then chmod 0500 then shell.Remove.RecursiveForce |
| 3 | `extdeps.provisioning.ubuntu_seeded_install_media_remaster` | storage Forked | 1 | storage Forked: `data install_media_remaster_work_prefix; fn install_media_re` /tmp/gunbc-seeded-media. |
| 4 | `extdeps.tools.sha256sum` | storage Forked | 1 | storage Forked: `fn sha256sum_verify_via_shell` Filesystem.Write(checksum_path) inside shell.Mktemp.Dir + Remove.RecursiveForce |
| 5 | `extdeps.github.actions_runner` | storage Literal | 3 | storage Literal: `data actions_runner_base_dir` /opt/actions-runner |
| 6 | `extdeps.cache.sccache` | placement Literal | 1 | placement Literal: `sccache_installed_binary_path` /usr/local/bin/sccache |
| 7 | `extdeps.provisioning.ubuntu_install_media` | storage Literal | 1 | storage Literal: `fn ubuntu_install_media_artifacts_dir` /var/lib/gunbc/artifacts |
| 8 | `extdeps.tools.websocat` | storage Literal | 0 | our install root literal "/var/lib/gunbc/bin/websocat" in an extdeps module; no directory authority enrols it |
| 9 | `extdeps.tools.stat` | placement Rederived | 1 | placement Rederived: `fn stat_allocated_blocks_command` /usr/bin/stat |

## The 10 worst seed modules (of 42 with any problem)

| # | module | problem dimensions | instances | worst finding |
|---:|---|---|---:|---|
| 1 | `src/v1/stage0/src/v1_interpreter.rs` | storage Forked, caching Forked, access Literal, placement Literal | 66 | caching Forked: `ACTIVE_SUBJECT` static ACTIVE_SUBJECT: RefCell<Option<String>> = const { RefCell::new(None) }; |
| 2 | `src/v1/stage0/src/v1_rt.rs` | storage Forked, caching Forked, access Literal, placement Literal | 16 | caching Forked: `RESOLUTION_SILENT_PICK` static RESOLUTION_SILENT_PICK: RefCell<SilentPickTelemetry> = RefCell::new(SilentPickTelem |
| 3 | `src/v1/stage0/src/cli_run.rs` | storage Forked, caching Forked, access Literal, placement Literal | 13 | caching Forked: `CHECKOUT_ROOT; PROCESS_WORKSPACE_ROOT; OBSERVATION_SOURCE_RO` static X: OnceLock<...> |
| 4 | `src/v1/stage0/src/memory_governor.rs` | storage Forked, access Forked, placement Literal | 20 | access Forked: `apply_memory_cgroup_bind` let _ = std::fs::write("/sys/fs/cgroup/cgroup.subtree_control", "+memory"); |
| 5 | `src/v1/stage0/src/cli_run/native_product_cache.rs` | storage Forked, access Literal, placement Literal | 13 | storage Forked: `commit` _ => std::fs::remove_dir_all(&dest) |
| 6 | `src/v1/stage0/src/bin/bootstrap_witness.rs` | storage Forked, caching Forked | 33 | caching Forked: `CI_PASS1` static CI_PASS1: LazyLock<Pass1Output> = LazyLock::new(// { |
| 7 | `src/v1/stage0/src/cli_run/native_lane_runner.rs` | storage Forked, placement Literal | 41 | storage Forked: `copy_directory_tree` std::fs::copy(entry.path(), &target).map_err(/e/ { |
| 8 | `src/v1/stage0/src/required_regen_host.rs` | storage Forked, access Literal | 36 | storage Forked: `admit_install_destination` InstallDestinationNotARegularFile: {basename} is a symlink. fs::copy follows the \ mechani |
| 9 | `src/v1/stage0/src/target_invocation_host.rs` | storage Forked, placement Literal | 15 | storage Forked: `run_ambient_name_census` let _ = std::fs::remove_dir_all(scratch.join(name)); |
| 10 | `src/v1/stage0/src/emitted_closure_compile_host.rs` | storage Forked, placement Literal | 6 | storage Forked: `create_private_probe_root` std::fs::create_dir_all(base).map_err(/e/ { |

## Most duplicated literal or mechanism, per dimension

A literal is a path-shaped token (first two segments under a system root, or a repo-relative target/ or artifacts/ name), a host name or an env-var name, taken from the specimen and note; a mechanism is an effect call. Counted over distinct modules.

| dimension | most duplicated literal | modules | most duplicated mechanism | modules |
|---|---|---:|---|---:|
| storage | `/var/lib/gunbc/` | 22 | `Filesystem.Write` | 79 |
| caching | `WITNESS_EVALUATION_FRAMES` | 1 | `OnceLock` | 12 |
| access | `/sys/fs/cgroup` | 6 | `shell.Env.Get` | 38 |
| authz | `/etc/systemd/system/` | 1 | `sudo_elevate` | 6 |
| placement | `srv1` | 20 | `env::var` | 10 |

## Error classes (classes.tsv), by modules affected

| class_id | dimension | modules | instances | current rung | owning authority |
|---|---|---:|---:|---:|---|
| `storage_root_literal` | storage | 62 | 66 | 0 | gunbc.owned_directory; gunbc.roadmap.dashboard_instance_directories |
| `storage_raw_state_beside_library` | storage | 57 | 135 | 0 | std.artifact_store::ArtifactStore; gunbc.durable_cas_file_store |
| `storage_relative_receipt_path` | storage | 55 | 63 | 0 | none exists: receipt/artifact sink |
| `placement_instance_repick` | placement | 45 | 46 | 0 | gunbc.roadmap.roadmap_dashboard_instance::HostDashboardInstance |
| `access_credential_location` | access | 37 | 39 | 0 | gunbc.auth.gcp_secret_access::SecretAccessGrant; gunbc.auth.access_token_source::select_access_token_source |
| `storage_hand_exclusion` | storage | 37 | 43 | 0 | std.durable_compare_and_set::CasExpectation; gunbc.durable_cas_file_store::file_compare_and_set |
| `access_ambient_env` | access | 35 | 35 | 0 | std.resources::ResourceHandle |
| `placement_unit_name_or_dir` | placement | 35 | 37 | 0 | none exists: unit-directory choice (extdeps.systemd lists legal locations) |
| `storage_hand_directory_setup` | storage | 34 | 83 | 0 | gunbc.owned_directory::OwnedDirectoryTarget; gunbc.roadmap.dashboard_instance_directories |
| `storage_hand_retention_sweep` | storage | 31 | 78 | 0 | gunbc.managed_directory |
| `placement_host_literal` | placement | 29 | 32 | 0 | gunbc.fleet.*; gunbc.roadmap.roadmap_dashboard_instance |
| `caching_process_memo_table` | caching | 23 | 75 | 0 | std.materialization_ladder; std.materialization_provider |
| `placement_upstream_binary` | placement | 22 | 26 | 0 | gunbc.roadmap.roadmap_dashboard_instance execution_environment; dispatch_capability_program |
| `storage_tmp_literal` | storage | 22 | 26 | 0 | std.artifact_store_scratch_grant |
| `access_rederived` | access | 17 | 18 | 0 | gunbc.fleet.fleet_host_identity |
| `access_kernel_abi_path` | access | 15 | 45 | 0 | extdeps.linux.* |
| `placement_env_branch` | placement | 13 | 25 | 0 | gunbc.ci.* (no CI-environment authority consumed by the seed) |
| `authz_ambient_sudo` | authz | 12 | 13 | 1 | gunbc.auth.authorization_pattern_selection::select_authorization_pattern; gunbc.auth.privileged_effect_census |
| `placement_port_address` | placement | 11 | 13 | 0 | none exists: served-endpoint declaration |
| `access_identity_literal` | access | 9 | 9 | 0 | gunbc.fleet.fleet_posix_accounts; gunbc.auth.access_token_source |
| `placement_device_node` | placement | 5 | 5 | 0 | host or instance declaration naming the device |
| `storage_root_rederived` | storage | 5 | 5 | 0 | gunbc.roadmap.dashboard_instance_directories |
| `authz_other` | authz | 4 | 4 | 0 | std.scoped_authorization::ScopedAuthorization; gunbc.auth.approval_capability::ApprovalCapability |
| `caching_root_literal` | caching | 3 | 3 | 0 | std.cache_interface::StorageSurface |
| `caching_generated_constant_memo` | caching | 1 | 1 | 0 | std.materialization_ladder |

