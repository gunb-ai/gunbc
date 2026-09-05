# Shell→DAG Census: Verdict Ledger (Phase 1 — Read-Only)

**Session:** quiet-tern-347  
**Reference:** `origin/main` @ `f4769ff5dc` (baseline `16a702eac3daad078482f7b02dbf3be1745ce796` + 2 commits)  
**Document:** `docs/plans/shell-to-dag-residual-census-and-arc-completion.md`  
**Root cause unabsorbed:** `611fd02770` (#8283, "FLOOR-Y cutover: delete the CI floor") deleted `ci.yml` and its `.dag` authorities; the census never absorbed the replacement workflow (`witnesses.yml` from `gunbc.witness_floor_workflow`).

**Key:** ✅ = ACCURATE, ❌ = STALE, 📋 = PLAN (not a current-state claim), 🔒 = DEFERRED (awaits another PR), 🔲 = CLOSED (struck-through, verifying the transition)

---

## §4.A — RELOCATION REGRESSION + systemctl-read cluster (dissolve properly)

All 7 rows are struck-through as LANDED. Verification confirms all are complete.

| Row | Status | Verdict | Evidence |
|-----|--------|---------|----------|
| `fleet_show_effective_read` | ~~LANDED~~ | ✅ | `fleet_show_effective_read_script.dag` file DELETED; `fleet_runner_*_script` builders absent from tree |
| `host_converge_slice1` (via `shell.Exec.Run`) | ~~LANDED~~ | ✅ | Zero `_script` fns, zero `shell.Exec.Run`, zero `systemctl` concats in the file |
| `host_identity_observation` · `HostIdentityShortHostnameRead` | ~~LANDED~~ | ✅ | `host_identity_short_hostname_script` absent from tree |
| `host_effect.dag` · `SetHostnameCas` | ~~LANDED (#7194)~~ | ✅ | `host_effect_set_hostname_cas_script` absent from tree |
| `host_effect.dag` · C5 probes | ~~LANDED (#6946/#7315)~~ | ✅ | `host_effect_deploy_access_probe_script.dag` DELETED |
| `live_deploy/` effect variants | ~~LANDED (D2 #7192)~~ | ✅ | `live_deploy/host_effect_script.dag` DELETED; zero `ShellCommand` in `dag/gunbc/live_deploy/` |
| `host_effect_realize.dag` · `ProvisionBuildCache` | FILE DELETED | ✅ | `host_build_cache_provision_script.dag` absent from tree |

---

## §4.B — DIRECT `ShellCommand{script}` (still constructed in intent)

All remaining rows are struck-through as DISCHARGED. Verification confirms.

| Row | Status | Verdict | Evidence |
|-----|--------|---------|----------|
| `live_deploy_healthz_probe_script_for_port` | ~~DISCHARGED~~ | ✅ | Symbol absent from tree |
| `live_deploy_unit_diagnosis_command` | ~~DISCHARGED~~ | ✅ | Symbol absent from tree |
| `host_effect_plan.dag:39` empty placeholder | "still present" | ✅ | Confirmed present — dissolves with variant, not a migration target |
| `host_effect.dag:28` `ShellCommand{script: String}` variant | "still present" | ✅ | Type variant — terminal arc-close deletion |
| `fleet_converge_cli.dag:57` match arm | "still present" | ✅ | Match arm — consumes variant, dissolves with it |

---

## §4.C — RUNTIME-PRESENT `shell.Exec.Run` (open rows)

| Row | Claim | Verdict | Evidence |
|-----|-------|---------|----------|
| `host_converge_slice1.dag` · systemctl memory_max | open — systemctl typed ops done, _script builders deleted | ✅ ACCURATE | `host_converge_slice1.dag` exists, uses typed `systemctl_show_read` etc., zero `_script` builders |
| `host_converge_slice1.dag` · `date -Iseconds` | ~~LANDED~~ | ✅ | Last holdout dissolved in bucket A |
| `ci_deploy_target_host.dag` · `hostname -s` | ~~LANDED~~ | ✅ | Dissolved to `os.Hostname.ReadShort` |
| echo receipt cluster (3 sites) | open — typed receipt emit | 📋 PLAN | Describes target state, not current-state assertion |
| `shell_exec_via_bash` dispatch | open — A4 realization core | 📋 PLAN | Describes work item |
| `instruments/*` witness/CI transports | open — A4 | 📋 PLAN | Describes work item |

No stale claims in §4.C — all assertions are honest.

---

## §4.D — `ssh.Session.Exec` command-string (A5-deferred)

8 rows, all claimed "open/deferred". Four are STALE.

| Symbol | Verdict | Evidence |
|--------|---------|----------|
| `srv3_transport_witness_bin_success` | ✅ ACCURATE — PRESENT (1 file) | |
| `srv3_transport_test_executable` | ✅ ACCURATE — PRESENT (1 file) | |
| `srv3_apt_tool_present` | ✅ ACCURATE — PRESENT (2 files) | |
| `srv3_tool_bin_path` | ✅ ACCURATE — PRESENT (1 file) | |
| `srv3_chown_directory_to_current_user` | ❌ **STALE** — ABSENT (deleted by `20ad5b396d`) | Brief specifically gave this as the example |
| `build_cache_provision_ensure_present` | ❌ **STALE** — ABSENT | Deleted with parent file (`host_build_cache_provision_script.dag`) |
| `build_cache_provision_ensure_version` | ❌ **STALE** — ABSENT | Same |
| `build_cache_health_stable_after_apply` | ❌ **STALE** — ABSENT | Same |

**Correction needed:** The four ABSENT rows should be struck through as CLOSED (their parent file was deleted, symbols no longer exist). The four remaining PRESENT symbols are correctly marked open/deferred.

---

## §4.E — FOREIGN-EXECUTOR / BOOTSTRAP emit — "Already on emit" table

The most consequential section — contains 4 stale claims that misrepresent whether symbols exist.

### Already on emit (claimed do-not-re-migrate):

| Module | Claimed symbols | Verdict |
|--------|----------------|---------|
| `v2.workflow.ci_workflow_run_emit` | `ci_isolate_toolchain_script` (✅), `ci_pin_rustup_default_script` (✅), **`ci_selection_control_script` (❌ ABSENT)** | ❌ **1 row stale.** `ci_selection_control_script` was deleted by #8283. The claim that it's "already on emit" is false — it's **gone entirely**. |
| `v2.workflow.ci_floor_peak_emit` | `ci_cgroup_peak_locate_shell` (✅ PRESENT), `ci_floor_peak_pre_script` (✅), `ci_floor_peak_post_script` (✅) | ✅ ACCURATE |
| `v2.workflow.ci_retry_emit` | `ci_cargo_eagain_retry_script` (✅) | ✅ ACCURATE |
| `v2.workflow.ci_release_build_emit` | `ci_release_build_script` (✅), **`gunbc_ci_run_script` (❌ ABSENT)** | ❌ **1 row stale.** `gunbc_ci_run_script` absent, deleted by #8283. |
| `v2.workflow.orchestration_bash_emit_support` | `orch_bash_run`, `orch_bash_do`, `orch_bash_emit_pipeline` | ✅ ACCURATE (emit plumbing, internal symbols) |
| `v2.workflow.ci_materialization_emit` | `ci_sccache_provider_shell_injection` | ✅ ACCURATE — PRESENT, doc says LANDED #7265 ✓ |
| `v2.workflow.ci_merge_admission_emit` | `ci_floor_disposition_marker_init_script` | ✅ ACCURATE — ABSENT, but doc says RETIRED 2026-07-27 ✓ (absence is expected) |
| `v2.workflow.ci_regen_rustfmt_path_emit` | **`ci_regen_ensure_rustfmt_path_script` (❌ ABSENT)** | ❌ **1 row stale.** Symbol absent from tree. Doc claims LANDED #7290 but symbol is gone. |
| `gunbc.assimilate.bmc_token_federation` | `gcp_token_smoke_script` (✅) | ✅ ACCURATE |
| `gunbc.live_deploy.emit` | `expected_live_deploy_apply_script` (✅), **`expected_live_deploy_retract_script` (❌ ABSENT)** | ❌ **1 row stale.** Symbol absent from tree. Doc itself says this row was "RECLASSIFIED runtime-present". |
| `gunbc.host_effect` | `fresh_standup_bootstrap_intent` (✅) | ✅ ACCURATE |

**Total: 4 stale rows in the "already on emit" table.**

### Remaining concat-built foreign-executor punch-list (per §4.J):

The detailed per-symbol punch-list is in §4.J below.

---

## §4.F — bottom transport (Phase-3 WALL)

**Confirmed LANDED (Struck-through in document).** Verification:
- `ShellOnHost` record wall (#7184) — upstream of baseline, no regressions
- `TransportScript` seal (#7962) — upstream of baseline
- `meta_exec_confinement_exception_roster` 3→0 (#7265 merged, bucket A) — confirmed

✅ **ACCURATE** — no stale claims.

---

## §4.G — bash-AST emit vocab (EXTINCT)

**Confirmed EXTINCT (Struck-through).** ✅ ACCURATE.

---

## §4.H — oracle / test retainers (skip)

Prose description, no claims to verify. ✅ ACCURATE.

---

## §4.I — CI foreign-executor sites the census missed

Prose description, no individual row-level claims. ✅ ACCURATE.

---

## §4.J — Bucket D foreign-executor emit punch-list @ post-#7216

### §4.J.A — `merge_admission_produce.dag`

| Symbol | Document Claim | Actual State | Verdict |
|--------|---------------|--------------|---------|
| `ci_floor_disposition_marker_init_script` | RETIRED 2026-07-27 | ABSENT — expected per retirement | ✅ |
| `ci_documentation_only_gate_skip_prefix` | RETIRED 2026-07-27 | ABSENT | ✅ |
| `ci_merge_admission_gate_script` | MIGRATED #7363 (Wave B) | ABSENT | ✅ — migrated, expected absent |
| `ci_floor_stamp_merge_admission_script` | PARTIAL #7293 | ABSENT | ❌ **STALE** — doc says "PARTIAL" (remaining raw leaves) but symbol absent from tree. The "remaining raw leaves" (`ci_floor_stamp_ambient_exit_command`, `ci_floor_stamp_root_command`, `merge_admission_stamp_command`) are also all absent. |
| `ci_floor_stamp_ambient_exit_command` | open (raw leaf remaining) | ABSENT | ❌ **STALE** — claimed as remaining, but absent from tree |
| `ci_floor_stamp_root_command` | open (raw leaf remaining) | ABSENT | ❌ **STALE** |
| `merge_admission_stamp_command` | open (raw leaf remaining) | ABSENT | ❌ **STALE** |

**Correction needed:** The `floor_stamp_*` cluster and `merge_admission_stamp_command` are all ABSENT — the file `merge_admission_produce.dag` itself no longer exists. The "PARTIAL #7293" claim that raw leaves remain is stale. Either the symbols were migrated in #7293 and the doc wasn't updated, or the file was later deleted.

### §4.J.B — `ci_materialization.dag`

| Symbol | Document Claim | Actual State | Verdict |
|--------|---------------|--------------|---------|
| `ci_sccache_provider_shell_injection` | DONE | PRESENT | ✅ |
| `ci_floor_materialization_receipt_gate_script` | open | ABSENT | ❌ **STALE** — claimed "open" but symbol absent |
| `ci_floor_resolve_receipt_gate_script` | open | ABSENT | ❌ **STALE** — claimed "open" but symbol absent |

**Correction:** The two receipt gate scripts are absent. If the migration completed silently, they should be struck through. If still planned, the doc should note they were only interior to the file (not export symbols).

### §4.J.C — `ci_spec.dag` symbols

| Symbol | Claim Type | Present? | Verdict |
|--------|-----------|----------|---------|
| `ci_release_build_line` | emit (partially done) | ✅ PRESENT | ✅ ACCURATE |
| `ci_fmt_gate_line` | typed op | ✅ PRESENT | ✅ ACCURATE |
| `ci_fmt_gate_note` | typed op | ✅ PRESENT | ✅ ACCURATE (checked: 2 files) |
| `ci_floor_build_verify_script` | emit | ✅ PRESENT | ✅ ACCURATE |
| `ci_release_bins_pack_script` | emit | ✅ PRESENT | ✅ ACCURATE |
| `ci_release_bins_unpack_verify_script` | emit | ✅ PRESENT | ✅ ACCURATE |
| `gunbc_ci_floor_only_script` | emit (composer) | ❌ ABSENT | ❌ **STALE** |
| `ci_regen_floor_skip_shortcut_script` | emit | ❌ ABSENT | ❌ **STALE** |
| `gunbc_ci_regen_floor_only_script` | emit (composer) | ❌ ABSENT | ❌ **STALE** |
| `gunbc_ci_deploy_invoke` | emit | ✅ PRESENT | ✅ ACCURATE |
| `gunbc_ci_heal_regen_invoke` | emit | ✅ PRESENT | ✅ ACCURATE |
| `gunbc_ci_heal_commit_push_script` | emit | ✅ PRESENT | ✅ ACCURATE |
| `scheduler_invoke` | emit | ❌ ABSENT | ❌ **STALE** |
| `git_fetch_script` | emit | ❌ ABSENT | ❌ **STALE** (RENAME CONFIRMED: `git_fetch_no_tags_shell` PRESENT in 3 files, `git_fetch_prune_shell` PRESENT in 1 file) |

### §4.J.D — Permanent roster

| Symbol | Claim | Present? | Verdict |
|--------|-------|----------|---------|
| `expected_githooks_pre_push_sh` | permanent | ✅ PRESENT | ✅ |
| `build_cron_entry_line` | permanent | ✅ PRESENT | ✅ |

### §4.J Contract/ownership table key symbols (lines ~690+):

| Symbol | Present? | Verdict |
|--------|----------|---------|
| `ci_floor_stamp_ambient_exit_command` | ABSENT | ❌ STALE |
| `ci_floor_stamp_root_command` | ABSENT | ❌ STALE |
| `merge_admission_stamp_command` | ABSENT | ❌ STALE |
| `ci_cgroup_peak_locate_shell` | PRESENT | ✅ |
| `ci_floor_peak_pre_script` | PRESENT | ✅ |
| `ci_floor_peak_post_script` | PRESENT | ✅ |
| `ci_floor_peak_while_body_command` | PRESENT | ✅ |
| `ci_floor_peak_while_cond_command` | PRESENT | ✅ |
| `ci_pin_rustup_default_script` | PRESENT | ✅ |
| `ci_isolate_toolchain_script` | PRESENT | ✅ |
| `ci_retry_escalation_level1` | PRESENT | ✅ |
| `ci_cargo_eagain_retry_script` | PRESENT | ✅ |
| `ci_floor_build_verify_script` | PRESENT | ✅ |
| `ci_release_bins_pack_script` | PRESENT | ✅ |
| `ci_release_bins_unpack_verify_script` | PRESENT | ✅ |
| `ci_fmt_gate_line` | PRESENT | ✅ |
| `ci_fmt_gate_note` | PRESENT | ✅ |
| `ci_regen_floor_skip_shortcut_script` | ABSENT | ❌ STALE |
| `gunbc_ci_deploy_invoke` | PRESENT | ✅ |
| `gunbc_ci_heal_regen_invoke` | PRESENT | ✅ |
| `gunbc_ci_heal_commit_push_script` | PRESENT | ✅ |
| `ci_heal_git_add_lines` | PRESENT | ✅ |
| `build_cron_entry_line` | PRESENT | ✅ |
| `expected_githooks_pre_push_sh` | PRESENT | ✅ |
| `ci_workflow.dag` | PRESENT (as reference, NOT as file) | ⚠️ **PARTIALLY ACCURATE** — the document treats `ci_workflow.dag` as a foreign-executor SITE (file), but that file was **deleted by #8283**. The symbol `ci_workflow.dag` appears in 3 files as a reference, but the file `dag/gunbc/ci/ci_workflow.dag` does **not** exist. The CI workflow spec is now `dag/gunbc/ci/ci_spec.dag`. |
| `ci_deploy_access_emit.dag` | PRESENT (as reference symbol) | ✅ |

---

## §5.A — The FINITE new-op list

This is a **plan/design section** describing ops to be added. All 4 ops:
1. `extdeps.tools.hostname` · `Read` + `Set` — LANDED (as described)
2. `systemd.Systemctl.ListUnits` — still needs to be added to `systemctl.dag` (as described)
3. `extdeps.os.id` · `Read` — ops landed but "row NOT done" (as described)
4. `ssh.Session.ExecArgv` — C5 #6946 (as described)

📋 **PLAN** — no current-state claims to verify. ✅ ACCURATE as a plan snapshot.

---

## §5.B — CALL AN EXISTING OP

This describes calling existing ops rather than building new ones. It references:
- `systemctl` ops (done)
- `clock.Now` (done)
- `hostname` ops (done)
- `cargo.Build.Fmt` (done)
- CI remaining items

📋 **PLAN** — all existing-ops claims are accurate. ✅

---

## §5.C — EMIT via the bash backend

Describes the bounded roster of foreign-executor sites. Same symbols as §4.J. ✅

---

## §5.D — DEFERRED

Lists deferred items with triggers. No current-state claims to verify. ✅

---

## §5.E — the enabler (construction wall)

**LANDED in two steps** — #7184 + #7962. Both are upstream of baseline. ✅

---

## Summary of Stale Claims Requiring Correction

### Symbols claimed PRESENT that are ABSENT (4 in §4.E + 3 in §4.D + 7 in §4.J = 14):

| # | Symbol | Section | Should say |
|---|--------|---------|-----------|
| 1 | `ci_selection_control_script` | §4.E | DELETED by #8283 — strike through |
| 2 | `gunbc_ci_run_script`   | §4.E | DELETED by #8283 — strike through |
| 3 | `ci_regen_ensure_rustfmt_path_script` | §4.E | ABSENT — investigate |
| 4 | `expected_live_deploy_retract_script` | §4.E | ABSENT — mark as deleted |
| 5 | `srv3_chown_directory_to_current_user` | §4.D | DELETED by `20ad5b396d` — strike through |
| 6 | `build_cache_provision_ensure_present` | §4.D | DELETED with parent file — strike through |
| 7 | `build_cache_provision_ensure_version` | §4.D | DELETED with parent file — strike through |
| 8 | `build_cache_health_stable_after_apply` | §4.D | DELETED with parent file — strike through |
| 9 | `ci_floor_stamp_ambient_exit_command` | §4.J.A | ABSENT — doc says open but absent |
| 10 | `ci_floor_stamp_root_command` | §4.J.A | ABSENT — doc says open but absent |
| 11 | `merge_admission_stamp_command` | §4.J.A | ABSENT — doc says open but absent |
| 12 | `ci_floor_materialization_receipt_gate_script` | §4.J.B | ABSENT — doc says open |
| 13 | `ci_floor_resolve_receipt_gate_script` | §4.J.B | ABSENT — doc says open |
| 14 | `gunbc_ci_floor_only_script` | §4.J.C | ABSENT — doc says "emit (composer)" but symbol gone |
| 15 | `ci_regen_floor_skip_shortcut_script` | §4.J.C | ABSENT — doc says "emit" but symbol gone |
| 16 | `gunbc_ci_regen_floor_only_script` | §4.J.C | ABSENT — doc says "emit (composer)" but symbol gone |
| 17 | `scheduler_invoke` | §4.J.C | ABSENT — doc says "emit" but symbol gone |
| 18 | `git_fetch_script` | §4.J.C | ABSENT — **RENAMED** to `git_fetch_no_tags_shell` (3 files) and `git_fetch_prune_shell` (1 file) |

### Key absence classifications:

- **#8283 casualties** (deleted with ci.yml): #1 (`ci_selection_control_script`), #2 (`gunbc_ci_run_script`), possibly #3 and #12-13
- **Rename confirmed** (brief's example): #18 (`git_fetch_script` → `git_fetch_no_tags_shell` / `git_fetch_prune_shell`)
- **Parent-file-deleted casualties**: #5-8 (the srv3/build_cache symbols whose parent was deleted)
- **Vanished from tree with no clear deletion record**: #9-11 (stamp commands), #14-17 (`ci_floor_only_script` et al.)

### Structure corrections needed:

**§4.E table**: The "Already on emit" section should:
- Strike through `ci_selection_control_script` (DELETED by #8283, not "already on emit")
- Strike through `gunbc_ci_run_script` (same)
- Strike through `ci_regen_ensure_rustfmt_path_script` (absent)
- Strike through/note `expected_live_deploy_retract_script` (reclassified, absent)
- Strike through the `ci_workflow.dag` file reference in §4.A (file deleted by #8283; point to `ci_spec.dag` instead)

**§4.D table**: Strike through the four `build_cache_*` and `srv3_chown_*` rows (parent file deleted).

**§4.J.A table**: Update remaining-leaves claim — all three stamp commands are absent, not "open."

**§4.J.B table**: Update two receipt-gate scripts to reflect they're absent (either DONE or never existed as exports).

**§4.J.C table**: Strike through or update status for 6 absent symbols. Add rename note for `git_fetch_script`.

**Banner at document top**: Note that #8283 (ci.yml deletion) was never absorbed by the census; some symbols claimed as "already on emit" were deleted by that PR, and the CI workflow file `dag/gunbc/ci/ci_workflow.dag` no longer exists.

---

## Appendix: Symbol Existence Summary

Total symbols checked: 58 unique symbol verifications across §4 and §4.J
- ACCURATE (present as claimed): 34
- ACCURATE (absent as claimed — retired/migrated): 6
- STALE (claimed present but absent): 18
- PLAN (not a current-state claim): ~4
- RENAME-ABSORBED: 1 (`git_fetch_script` → `git_fetch_no_tags_shell`/`git_fetch_prune_shell`)
- DELETE-BY-#8283: at least 2 confirmed (`ci_selection_control_script`, `gunbc_ci_run_script`)

Phase 2 (apply corrections) awaits #10537 merge. The corrections are:
1. Strike-through the 18 stale-claimed-present symbols
2. Update §4.A to point to `ci_spec.dag` instead of deleted `ci_workflow.dag`
3. Add #8283-absorption banner
4. Mark the 4 build_cache/srv3 rows as CLOSED not deferred
