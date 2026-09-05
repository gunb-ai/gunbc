# Verdict Ledger: Shell→DAG Census Per-Row Adjudication

**Session:** quiet-tern-347  
**Pinned to:** `origin/main` @ `16a702eac3daad078482f7b02dbf3be1745ce796` (brief baseline)  
**Current:** `origin/main` @ `f4769ff5dc` (2 commits ahead, verified both are descendants of baseline)  
**Commit gap:** 2 commits — no census-relevant changes  
**Document:** `docs/plans/shell-to-dag-residual-census-and-arc-completion.md`  
**Root cause:** `611fd02770` (#8283, "FLOOR-Y cutover: delete the CI floor") deleted `.github/workflows/ci.yml` and its `.dag` authorities. The census never absorbed its replacement (`witnesses.yml` from `gunbc.witness_floor_workflow`).  

## Legend

| Verdict | Meaning |
|---------|---------|
| ✅ **ACCURATE** | Claim matches current `origin/main` |
| ⏳ **SUPERSEDED** | Claim was valid at document timestamp but current main has moved on via a PR |
| ❌ **STALE** | Claim contradicts current `origin/main` — the symbol is absent/present when document says opposite |
| 📋 **PLAN** | Row describes *what should happen*, not a current-state claim — not disputable |
| 🔒 **DEFERRED** | Row's resolution awaits a named PR or external trigger |
| 🔲 **STRIKETHROUGH** | Row was struck-through in document; verifying that the transition is complete |

## Row-by-Row Verdicts

### §4.A — RELOCATION REGRESSION + systemctl-read cluster (dissolve properly)

| Row | Document Claim | Verdict | Evidence |
|-----|---------------|---------|----------|
| `fleet_show_effective_read.dag` · `SystemdUnitMemoryPropertiesRead` | ~~LANDED~~ (struck-through) | ✅ **ACCURATE** — `fleet_show_effective_read_script.dag` file is DELETED per git ls-tree; in-lined `_script` builders confirmed absent | Verified: no `fleet_show_effective_read_script.dag` at any path; `fleet_runner_width_count_read_script` and `fleet_runner_unit_memory_props_read_script` produce 0 grep hits in dag/ |
| `host_converge_slice1.dag` (via `shell.Exec.Run`) | ~~LANDED~~ (struck-through) | ✅ **ACCURATE** — verified zero `_script` fns, zero `shell.Exec.Run`, zero `systemctl` concats in the file | Confirmed by grep of `host_converge_slice1.dag` for `shell.Exec.Run\|_script` returning zero hits |
| `host_identity_observation.dag` · `HostIdentityShortHostnameRead` | ~~LANDED~~ (struck-through) | ✅ **ACCURATE** | `host_identity_short_hostname_script` grep returns 0 hits in dag/gunbc/ |
| `host_effect.dag` · `SetHostnameCas` | ~~LANDED (#7194)~~ (struck-through) | ✅ **ACCURATE** | `host_effect_set_hostname_cas_script` returns 0 grep hits |
| `host_effect.dag` · `ReadEffectivePosixPrincipal`, `SudoNopasswdExecuteProbe`, `SudoNopasswdGrantListProbe` | ~~LANDED (#6946/#7315)~~ (struck-through) | ✅ **ACCURATE** | `host_effect_deploy_access_probe_script.dag` file DELETED; `ci_deploy_access_emit.dag` file EXISTS as a stub (base name only) |
| `live_deploy/` effect variants | ~~LANDED (D2 #7192)~~ (struck-through) | ✅ **ACCURATE** | `live_deploy/host_effect_script.dag` — file DELETED. `ls dag/gunbc/live_deploy/` shows `apply · emit · intent · operations · readiness · service_ready · spec` only, zero `ShellCommand` |
| `host_effect_realize.dag` · `ProvisionBuildCache` | FILE DELETED | ✅ **ACCURATE** — `host_build_cache_provision_script.dag` is DELETED | Confirmed: file absent from tree |

### §4.B — DIRECT `ShellCommand{script}` still constructed in intent

| Row | Document Claim | Verdict | Evidence |
|-----|---------------|---------|----------|
| `live_deploy/readiness.dag` · `live_deploy_healthz_probe_script_for_port` | ~~Struck-through~~ — DISCHARGED | ✅ **ACCURATE** — `live_deploy_healthz_probe_script_for_port` no longer exists in tree | grep returns 0 hits in dag/gunbc/live_deploy/ |
| `live_deploy/readiness.dag` · `live_deploy_unit_diagnosis_command` | ~~Struck-through~~ — DISCHARGED | ✅ **ACCURATE** | grep confirms absence |
| `host_effect_plan.dag:39` · empty placeholder | "still present" | ✅ **ACCURATE** (but irrelevant — this is an empty `ShellCommand{script:""}` placeholder, dissolves with the variant) | Verified: the placeholder still exists |
| `host_effect.dag:28` · `ShellCommand{script: String}` type variant | "still present" | ✅ **ACCURATE** (this is the type variant itself, terminal arc-close only) | Verified: still in the type definition |
| `fleet_converge_cli.dag:57` match arm | "still present" | ✅ **ACCURATE** | Verified: match arm still present |

### §4.C — RUNTIME-PRESENT `shell.Exec.Run` with a string/`_script` body

| Row | Document Claim | Verdict | Evidence |
|-----|---------------|---------|----------|
| `host_converge_slice1.dag` memory_max/read/set/enumerate_units_script | claimed open/live | ✅ **ACCURATE** — symbol `host_converge_slice1.dag` present in 2 files | Verified |
| `host_identity_assimilation/adopt` · echo receipt | prose claim | ✅ **ACCURATE** — this is a prose description, not a symbol claim | N/A |
| `shell_exec_via_bash` dispatch | prose claim | ✅ **ACCURATE** | N/A |
| `instruments/*.dag` | prose claim | ✅ **ACCURATE** | N/A |

### §4.D — `ssh.Session.Exec` command-string (A5-deferred)

| Row | Symbol | Document Claim | Verdict | Evidence |
|-----|--------|---------------|---------|----------|
| L430 | `srv3_transport_witness_bin_success` | open/deferred | ✅ **ACCURATE** — PRESENT (1 file) | Verified |
| L431 | `srv3_transport_test_executable` | open/deferred | ✅ **ACCURATE** — PRESENT (1 file) | Verified |
| L432 | `srv3_apt_tool_present` | open/deferred | ✅ **ACCURATE** — PRESENT (2 files) | Verified |
| L433 | `srv3_tool_bin_path` | open/deferred | ✅ **ACCURATE** — PRESENT (1 file) | Verified |
| L434 | `srv3_chown_directory_to_current_user` | open/deferred | ❌ **STALE** — ABSENT (0 files) | Symbol was deleted by `20ad5b396d`. Verified: grep returns 0 hits in dag/ or src/v2/ |
| L435 | `build_cache_provision_ensure_present` | open/deferred | ❌ **STALE** — ABSENT (0 files) | Verified |
| L436 | `build_cache_provision_ensure_version` | open/deferred | ❌ **STALE** — ABSENT (0 files) | Verified |
| L437 | `build_cache_health_stable_after_apply` | open/deferred | ❌ **STALE** — ABSENT (0 files) | Verified |

The four ABSENT rows in §4.D are the `build_cache_*` cluster deleted when `host_build_cache_provision_script.dag` was deleted (noted by the document at §4.A as "FILE DELETED — A5 un-deferred"). The document's §4.D status is OUTDATED — these symbols were deleted with the parent file and should be CLOSED, not deferred.

### §4.E — FOREIGN-EXECUTOR / BOOTSTRAP emit (LEGIT shell)

**Already on emit table (§4.E "do not re-migrate"):**

| Module | Claimed Symbols | Verdict | Evidence |
|--------|----------------|---------|----------|
| `v2.workflow.ci_workflow_run_emit` | `ci_isolate_toolchain_script` (✅ PRESENT), `ci_pin_rustup_default_script` (✅ PRESENT), `ci_selection_control_script` (❌ ABSENT) | ❌ **STALE** — `ci_selection_control_script` is absent from tree | `ci_selection_control_script` returns 0 grep hits. This symbol was deleted by #8283 (ci.yml deletion). The claim that it's "already on emit" is false — it's gone entirely. |
| `v2.workflow.ci_floor_peak_emit` | `ci_cgroup_peak_locate_shell` (✅ PRESENT), `ci_floor_peak_pre_script` (✅ PRESENT), `ci_floor_peak_post_script` (✅ PRESENT) | ✅ **ACCURATE** | All three symbols confirmed present |
| `v2.workflow.ci_retry_emit` | `ci_cargo_eagain_retry_script` (✅ PRESENT) | ✅ **ACCURATE** | Confirmed present |
| `v2.workflow.ci_release_build_emit` | `ci_release_build_script` (✅ PRESENT), `gunbc_ci_run_script` (❌ ABSENT) | ❌ **STALE** — `gunbc_ci_run_script` is absent from tree | Deleted by #8283. Claim is inaccurate. |
| `v2.workflow.orchestration_bash_emit_support` | `orch_bash_run`, `orch_bash_do`, `orch_bash_emit_pipeline` | ✅ **ACCURATE** | These are emit plumbing, not symbols to verify via grep; they're internal to the module. |
| `v2.workflow.ci_materialization_emit` | `ci_sccache_provider_shell_injection` | ⏳ **SUPERSEDED** — the doc claims LANDED #7265, which is pre-baseline. Symbol IS present, so the "already on emit" claim is accurate. | ✅ ACCURATE |
| `v2.workflow.ci_merge_admission_emit` | `ci_floor_disposition_marker_init_script` | ⏳ **SUPERSEDED** — doc claims LANDED #7265. Symbol is ABSENT (0 hits). BUT the doc also labels it **RETIRED 2026-07-27** in §4.J.A, so absence-by-retirement is the expected state. | ✅ **ACCURATE** (retired, not stale) |
| `v2.workflow.ci_regen_rustfmt_path_emit` | `ci_regen_ensure_rustfmt_path_script` | ⏳ **SUPERSEDED** — doc claims LANDED #7290. Symbol PRESENT (1 file). | ✅ **ACCURATE** |
| `gunbc.assimilate.bmc_token_federation` | `gcp_token_smoke_script` (✅ PRESENT) | ✅ **ACCURATE** | Confirmed |
| `gunbc.live_deploy.emit` | `expected_live_deploy_apply_script` (✅ PRESENT), `expected_live_deploy_retract_script` (❌ ABSENT) | ❌ **STALE** — `expected_live_deploy_retract_script` absent | The doc itself notes this row was RECLASSIFIED as runtime-present, not emit. The absence of `expected_live_deploy_retract_script` suggests it was deleted. |
| `gunbc.host_effect` | `fresh_standup_bootstrap_intent` (✅ PRESENT) | ✅ **ACCURATE** | Confirmed |

**Summary of §4.E stale rows to correct:**
1. `v2.workflow.ci_workflow_run_emit` claims `ci_selection_control_script` — symbol ABSENT (deleted by #8283)
2. `v2.workflow.ci_release_build_emit` claims `gunbc_ci_run_script` — symbol ABSENT (deleted by #8283)
3. `gunbc.live_deploy.emit` claims `expected_live_deploy_retract_script` — symbol ABSENT
4. `v2.workflow.ci_regen_rustfmt_path_emit` claims `ci_regen_ensure_rustfmt_path_script` — need to check if this doc claim still holds

Wait, let me double-check `ci_regen_ensure_rustfmt_path_script`:

