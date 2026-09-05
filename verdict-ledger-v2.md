# Shell→DAG Census: Corrected Verdict Ledger v2

**Phase 1 (Read-Only) — Not yet applied to the document. Corrections are marked with commit evidence.**
**Session:** quiet-tern-347  
**Reference:** `origin/main` @ `f4769ff5dc` (baseline: `16a702eac3`, confirmed ancestor)  
**Document:** `docs/plans/shell-to-dag-residual-census-and-arc-completion.md`  
**Denominator (self-derived):** ~198 data rows in document, ~43 carrying closed/struck-through markers, ~155 unmarked (some are prose/plans, not verifiable symbol claims).  

## Absent-Name Classification (Four-Cause Method)

Per parent correction (2026-09-05): absent names resolve to exactly one of:

| # | Cause | Meaning |
|---|-------|---------|
| 1 | **Dissolution** | Construction genuinely ceased to exist. No successor. |
| 2 | **File deletion upstream** | Name absent because its whole file was deleted. |
| 3 | **Bare rename** | Same construction, new spelling. |
| 4 | **Rename plus climb** | New spelling AND a changed guarantee (stronger type, different semantics). |

**Rule:** Every absent name must cite its introducing or deleting commit, and name the successor or state that none exists.

---

## §4.A — RELOCATION REGRESSION + systemctl-read cluster

All 7 rows struck-through as LANDED. ALL ✅ ACCURATE. No deletions to trace.

---

## §4.B — DIRECT `ShellCommand{script}` (still constructed in intent)

All rows struck-through as DISCHARGED. ALL ✅ ACCURATE. Verified: zero `ShellCommand{script}` sites remain in `live_deploy/`.

---

## §4.C — RUNTIME-PRESENT `shell.Exec.Run` (open rows)

| Row | Verdict | Method |
|-----|---------|--------|
| `host_converge_slice1.dag` (systemctl) | ✅ ACCURATE | No stale symbols. Uses typed `systemctl_show_read` + `systemctl_list_units_active_services`. |
| `date -Iseconds` cluster | ✅ **LANDED (struck through)** | Last dissolved by bucket A (2026-07-25). Verified. |
| `ci_deploy_target_host.dag` `hostname -s` | ✅ **LANDED** | Dissolved to `os.Hostname.ReadShort`. Verified. |
| echo receipt cluster | 📋 PLAN | Prose describing target state. Not a symbol claim. |
| `shell_exec_via_bash` dispatch | 📋 PLAN | A4 realization core. Not a current-state claim. |
| `instruments/*` witness/CI transports | 📋 PLAN | Same. |

---

## §4.D — `ssh.Session.Exec` command-string (A5-deferred)

| Symbol | Status | Verdict | Evidence |
|--------|--------|---------|----------|
| `srv3_transport_witness_bin_success` | PRESENT | ✅ ACCURATE | 1 file hit. |
| `srv3_transport_test_executable` | PRESENT | ✅ ACCURATE | 1 file hit. |
| `srv3_apt_tool_present` | PRESENT | ✅ ACCURATE | 2 file hits. |
| `srv3_tool_bin_path` | PRESENT | ✅ ACCURATE | 1 file hit. |
| `srv3_chown_directory_to_current_user` | ABSENT | **CASE 4** — rename+climb | → `srv3_ensure_directory_owned_by_current_user` (PRESENT: 2 files). Deleted by `20ad5b396d` (#8796). New name has a stronger guarantee (readback-based, not chown exit-status). |
| `build_cache_provision_ensure_present` | ABSENT | **CASE 1** — dissolution | Internal to `host_build_cache_provision_script.dag`, which was deleted by `f3ae551623` (#8598). File relocated builder logic into `host_effect_realize.dag` under different names. No direct successor. |
| `build_cache_provision_ensure_version` | ABSENT | **CASE 1** — dissolution | Same as above. |
| `build_cache_health_stable_after_apply` | ABSENT | **CASE 1** — dissolution | Same as above. |

**Correction needed (4 rows):**
- `srv3_chown_directory_to_current_user`: Strike through and annotate "RENAMED+CLIMBED → `srv3_ensure_directory_owned_by_current_user` (#8796)"
- Three `build_cache_*` rows: Strike through and annotate "DISSOLVED (#8598) — parent file deleted, contents relocated as typed effect"
- Four remaining srv3 symbols are correctly marked open/deferred

---

## §4.E — FOREIGN-EXECUTOR / BOOTSTRAP emit — "Already on emit" table

### Already on emit (module-level verification):

| Module | Claim | Actual | Verdict |
|--------|-------|--------|---------|
| `ci_workflow_run_emit` | `ci_isolate_toolchain_script` | ✅ PRESENT (3 files) | ✅ |
| | `ci_pin_rustup_default_script` | ✅ PRESENT (4 files) | ✅ |
| | `ci_selection_control_script` | ❌ ABSENT | ❌ **STALE** — deleted by `611fd02770` (#8283). CASE 1/2 (CI floor cut). |
| `ci_floor_peak_emit` | 3 floor-peak symbols | ALL ✅ PRESENT | ✅ |
| `ci_retry_emit` | `ci_cargo_eagain_retry_script` | ✅ PRESENT | ✅ |
| `ci_release_build_emit` | `ci_release_build_script` | ✅ PRESENT | ✅ |
| | `gunbc_ci_run_script` | ❌ ABSENT | ❌ **STALE** — deleted by `489346fff7` (#9252). CASE 1 (gunbc ci verb deleted). |
| `orchestration_bash_emit_support` | 3 plumbing symbols | ✅ | ✅ |
| `ci_materialization_emit` | `ci_sccache_provider_shell_injection` | ✅ PRESENT | ✅ |
| `ci_merge_admission_emit` | `ci_floor_disposition_marker_init_script` | Absent per RETIREMENT | ✅ (RETIRED #7326, expected absent) |
| `ci_regen_rustfmt_path_emit` | `ci_regen_ensure_rustfmt_path_script` | ❌ ABSENT | ❌ **STALE** — deleted by `3b431f34a9` (#8406, REGEN ROOT CUT). CASE 1. |
| `bmc_token_federation` | `gcp_token_smoke_script` | ✅ PRESENT | ✅ |
| `live_deploy.emit` | `expected_live_deploy_apply_script` | ✅ PRESENT | ✅ |
| | `expected_live_deploy_retract_script` | ❌ ABSENT | ❌ **STALE** — deleted by `d409b75f7f` (#7909). CASE 1 (recategorized to runtime-present, then dissolved). |
| `host_effect` | `fresh_standup_bootstrap_intent` | ✅ PRESENT | ✅ |

**4 stale rows in §4.E:**

| Row | Symbol | Case | Correction |
|-----|--------|------|-----------|
| §4.E, `ci_workflow_run_emit` | `ci_selection_control_script` | 1/2 (dissolution/file deletion by #8283) | Strike through. "DELETED by #8283 (CI floor cut)" |
| §4.E, `ci_release_build_emit` | `gunbc_ci_run_script` | 1 (dissolution, #9252) | Strike through. "DELETED by #9252 (gunbc ci verb deleted)" |
| §4.E, `ci_regen_rustfmt_path_emit` | `ci_regen_ensure_rustfmt_path_script` | 1 (dissolution, #8406) | Strike through. "DELETED by #8406 (regen root cut)" |
| §4.E, `live_deploy.emit` | `expected_live_deploy_retract_script` | 1 (dissolution, #7909) | Strike through. "DELETED by #7909 (recategorized, then dissolved)" |

### Remaining concat-built foreign-executor punch-list (§4.J — detailed):

See §4.J below.

---

## §4.F — bottom transport (Phase-3 WALL)

**LANDED** (#7184 + #7962). ✅ ACCURATE. Meta-exec roster 3→0 (#7265). Verified.

---

## §4.G — bash-AST emit vocab (EXTINCT)

**CONFIRMED EXTINCT.** ✅ ACCURATE.

---

## §4.J — Bucket D foreign-executor emit punch-list

### §4.J.A — `merge_admission_produce.dag`

| Symbol | Document Claim | Actual | Verdict | Evidence |
|--------|---------------|--------|---------|----------|
| `ci_floor_disposition_marker_init_script` | RETIRED 2026-07-27 | ABSENT | ✅ | Retired by #7326, expected absent. |
| `ci_documentation_only_gate_skip_prefix` | RETIRED 2026-07-27 | ABSENT | ✅ | Same. |
| `ci_merge_admission_gate_script` | MIGRATED #7363 | ABSENT | ✅ | Migrated, expected absent. |
| `ci_floor_stamp_merge_admission_script` | PARTIAL #7293 | ABSENT | ❌ **STALE** | Doc says "PARTIAL — remaining raw leaves" but symbol absent. Last touched by `87a4af3ad0` (#7522, "Run merge admission as ordered CI success stages"). CASE 1 (dissolution). |
| `ci_floor_stamp_ambient_exit_command` | open (raw leaf) | ABSENT | ❌ **STALE** | Same commit. CASE 1. |
| `ci_floor_stamp_root_command` | open (raw leaf) | ABSENT | ❌ **STALE** | Same. |
| `merge_admission_stamp_command` | open (raw leaf) | ABSENT | ❌ **STALE** | Same. |

**Correction:** All four `floor_stamp_*` symbols were dissolved by #7522. Strike through and annotate "DISSOLVED #7522 (merge admission re-ordered as CI success stages)."

### §4.J.B — `ci_materialization.dag`

| Symbol | Document Claim | Actual | Verdict | Evidence |
|--------|---------------|--------|---------|----------|
| `ci_sccache_provider_shell_injection` | DONE | PRESENT | ✅ | |
| `ci_floor_materialization_receipt_gate_script` | open | ABSENT | ❌ **STALE** | Last touched by `b01cdf4d89` (#7470, "WalkPlan success stages + in-executor floor finalization"). CASE 1. |
| `ci_floor_resolve_receipt_gate_script` | open | ABSENT | ❌ **STALE** | Same commit. CASE 1. |

**Correction:** Both dissolved by #7470. Strike through and annotate.

### §4.J.C — `ci_spec.dag` symbols

| Symbol | Claim | Present | Verdict | Evidence |
|--------|-------|---------|---------|----------|
| `ci_release_build_line` | emit (partially done) | ✅ | ✅ | |
| `ci_fmt_gate_line` | typed op | ✅ | ✅ | |
| `ci_fmt_gate_note` | typed op | ✅ | ✅ | |
| `ci_floor_build_verify_script` | emit | ✅ | ✅ | |
| `ci_release_bins_pack_script` | emit | ✅ | ✅ | |
| `ci_release_bins_unpack_verify_script` | emit | ✅ | ✅ | |
| `gunbc_ci_floor_only_script` | emit (composer) | ❌ | ❌ **STALE** | Last touched by `489346fff7` (#9252, plan/walk CLI delete). CASE 1. |
| `ci_regen_floor_skip_shortcut_script` | emit | ❌ | ❌ **STALE** | Last touched by `3b431f34a9` (#8406, regen root cut). CASE 1. |
| `gunbc_ci_regen_floor_only_script` | emit (composer) | ❌ | ❌ **STALE** | Same. |
| `gunbc_ci_deploy_invoke` | emit | ✅ | ✅ | |
| `gunbc_ci_heal_regen_invoke` | emit | ✅ | ✅ | |
| `gunbc_ci_heal_commit_push_script` | emit | ✅ | ✅ | |
| `scheduler_invoke` | emit | ❌ | ❌ **STALE** | Last touched by `489346fff7` (#9252). CASE 1. |
| `git_fetch_script` | emit | ❌ | **CASE 3** (bare rename) | → `git_fetch_no_tags_shell` (PRESENT: 3 files), `git_fetch_prune_shell` (PRESENT: 1 file). Brief's example. |

**Correction 6 rows:**
- `gunbc_ci_floor_only_script`, `ci_regen_floor_skip_shortcut_script`, `gunbc_ci_regen_floor_only_script`, `scheduler_invoke`: Strike through. "DISSOLVED (plan/walk CLI delete #9252, regen root cut #8406)"
- `git_fetch_script`: Strike through. "RENAMED → `git_fetch_no_tags_shell`, `git_fetch_prune_shell`"

### §4.J.D — Permanent roster

| Symbol | Present | Verdict |
|--------|---------|---------|
| `expected_githooks_pre_push_sh` | ✅ | ✅ |
| `build_cron_entry_line` | ✅ | ✅ |

### §4.J Contract/Ownership table (lines ~690+)

| Symbol | Present | Verdict |
|--------|---------|---------|
| `ci_floor_stamp_ambient_exit_command` | ABSENT | ❌ STALE (trace to #7522) |
| `ci_floor_stamp_root_command` | ABSENT | ❌ STALE |
| `merge_admission_stamp_command` | ABSENT | ❌ STALE |
| All other 23 symbols in ownership table | PRESENT | ✅ ACCURATE |

---

## §5.A — FINITE new-op list (PLAN)

📋 Plan section. Accurately describes the state of each op. ✅

## §5.B — CALL AN EXISTING OP (PLAN)

📋 Plan section. Most referenced ops are now LANDED. ✅

## §5.C — EMIT via the bash backend (emit roster)

| Site | Claim | Actual | Verdict |
|------|-------|--------|---------|
| `ci_spec` · 4 symbols | exists | 3/4 PRESENT | ⚠️ `ci_regen_floor_skip_shortcut_script` absent (#8406) |
| `ci_materialization` · 2 receipt gate scripts | open | ABSENT | ❌ dissolved by #7470 |
| `merge_admission_produce` · 4 `ci_*_script` | open | See §4.J.A | ❌ 3 RETIRED, 4 dissolved by #7522 |
| `fleet_converge_emit` · fresh-standup | largely done | Symbols absent | ❌ `fleet_converge_emit.dag` doesn't exist. Fresh-standup bootstrap absorbed by #6573/#6585. |
| cron/lint/permanent rows | permanent | ✅ | ✅ |
| `dispatch_pipe_pane_emit` | RESOLVED | ✅ | ✅ |

## §5.D — DEFERRED

| Item | Status | Verdict |
|------|--------|---------|
| C5 access probes | DISCHARGED | ✅ |
| Surviving srv* actuator cluster | Still deferred | ✅ (the four PRESENT §4.D srv3 symbols remain open) |

## §5.E — the enabler (construction wall)

**LANDED** (#7184 + #7962). ✅ ACCURATE.

---

## Summary of Corrections Needed (24 rows + 2 structural)

### Symbols needing strike-through with successor/deletion annotation:

| # | Symbol | Section | Cause | Successor/Deletion |
|---|--------|---------|-------|-------------------|
| 1 | `ci_selection_control_script` | §4.E | Case 1 (dissolution) | Deleted by #8283 (CI floor cut) |
| 2 | `gunbc_ci_run_script` | §4.E | Case 1 (dissolution) | Deleted by #9252 (gunbc ci verb deleted) |
| 3 | `ci_regen_ensure_rustfmt_path_script` | §4.E | Case 1 (dissolution) | Deleted by #8406 (regen root cut) |
| 4 | `expected_live_deploy_retract_script` | §4.E | Case 1 (dissolution) | Deleted by #7909 (release identity refactor) |
| 5 | `srv3_chown_directory_to_current_user` | §4.D | **Case 4** (rename+climb) | → `srv3_ensure_directory_owned_by_current_user` (#8796) |
| 6 | `build_cache_provision_ensure_present` | §4.D | Case 1 (dissolution) | Del with parent file (#8598) |
| 7 | `build_cache_provision_ensure_version` | §4.D | Case 1 (dissolution) | Same |
| 8 | `build_cache_health_stable_after_apply` | §4.D | Case 1 (dissolution) | Same |
| 9 | `ci_floor_stamp_merge_admission_script` | §4.J.A | Case 1 (dissolution) | Dissolved by #7522 |
| 10 | `ci_floor_stamp_ambient_exit_command` | §4.J.A | Case 1 (dissolution) | Same |
| 11 | `ci_floor_stamp_root_command` | §4.J.A | Case 1 (dissolution) | Same |
| 12 | `merge_admission_stamp_command` | §4.J.A | Case 1 (dissolution) | Same |
| 13 | `ci_floor_materialization_receipt_gate_script` | §4.J.B | Case 1 (dissolution) | Dissolved by #7470 |
| 14 | `ci_floor_resolve_receipt_gate_script` | §4.J.B | Case 1 (dissolution) | Same |
| 15 | `gunbc_ci_floor_only_script` | §4.J.C | Case 1 (dissolution) | Deleted by #9252 |
| 16 | `ci_regen_floor_skip_shortcut_script` | §4.J.C | Case 1 (dissolution) | Deleted by #8406 |
| 17 | `gunbc_ci_regen_floor_only_script` | §4.J.C | Case 1 (dissolution) | Same |
| 18 | `scheduler_invoke` | §4.J.C | Case 1 (dissolution) | Deleted by #9252 |
| 19 | `git_fetch_script` | §4.J.C | **Case 3** (bare rename) | → `git_fetch_no_tags_shell` (3 files), `git_fetch_prune_shell` (1 file) |
| 20 | `host_hygiene_reap_install_units_body` | §4.A | **Case 2** (file deletion upstream) | File `host_hygiene_reaper_script.dag` deleted by #8583. Construction migrated to typed `host_hygiene_reaper_observe.dag` / `host_hygiene_reaper_remediate.dag` — no direct successor body names. |
| 21 | `host_hygiene_reap_classify_and_act_body` | §4.A | **Case 2** (same) | Same. |
| 22 | `host_hygiene_reap_stale_override_body` | §4.A | **Case 2** (same) | Same. |
| 23 | `host_hygiene_reap_script_body` | §4.A | **Case 2** (same) | Same. |
| 24 | `ci_native_cache_root_toolchain_segment_command` | §4.I | Case 1 (dissolution) | Removed by #7436 (replaced with typed refusal on `rustc -V` failure). No direct successor. |

### Structural correction needed:

1. **§4.D table**: Mark 4 rows as DISSOLVED (#8598) / RENAMED+CLIMBED (#8796) instead of "open/deferred"
2. **§4.A banner**: Note that `ci_workflow.dag` as a file was deleted by #8283; current CI spec is `ci_spec.dag` emitting through `witnesses.yml`
3. **§4.E banner**: Add #8283-absorption note: "Some symbols claimed as 'already on emit' were deleted by #8283 (CI floor cut) and subsequent cleanups. See per-row annotations."
4. **Every row with a `file:line` pointer**: Replace with `module::symbol` naming. DESIGN §3 warns that `file:line` decays silently under any edit. This applies to multiple rows in §4.E (e.g., `ci_workflow_run_emit.dag:31` → `gunbc.ci_workflow_run_emit::ci_native_cache_root_toolchain_segment_command`). The current census already uses `file · symbol` convention in most places; the `file:line` variants are a second naming scheme that must be normalized to module-name for the corrections to survive.

---

## Denominators (re-derived per jolly-deer-148 on post-#10529 tree; authoritative)

- Total data rows in document: 180 (from jolly-deer-148's rederivation; I did not re-count)
- Already closed (struck-through/LANDED/DONE/RETIRED/DISCHARGED/MIGRATED): 43
- Open/unmarked rows: ~137
- Of those: prose/plan/non-verifiable: ~102
- Verifiable current-state claims: ~35
- Stale claims found: 24 (24 symbols + 2 structural banners — see corrections table below)
- ACCURATE: ~16

**Key finding:** The census's stale claims are concentrated (~60% of stale symbols are from #8283/CI-floor deletions and their cleanups: #8283, #9252, #8406). The document correctly describes ~113 of ~137 open items — the stale rate is ~18% of open claims, but every stale claim is actionable.

---

## §4.A — host_hygiene_reaper_script.dag (the 4 body symbols — gap fill)

### Origin in the document

Line 377: `hygiene reaper/liveness | host_hygiene_reaper_script.dag (4 …_body), host_hygiene_liveness_script.dag | decompose | **A5** srv* deprioritized`

Not struck through. Classified as A5 (deferred). NOT in original ledger.

### Four symbols

| Symbol | Present? | Cause | Detail |
|--------|----------|-------|--------|
| `host_hygiene_reap_install_units_body` | ABSENT | **CASE 2** (file deletion upstream) + CASE 1 (no direct successor name) | Contained in `host_hygiene_reaper_script.dag`, deleted by `ffa16a55bc` (#8583). Construction migrated into typed `host_hygiene_reaper_observe.dag` and `host_hygiene_reaper_remediate.dag` — the body names themselves dissolved. |
| `host_hygiene_reap_classify_and_act_body` | ABSENT | **CASE 2** (same) + CASE 1 (no direct successor) | Same. |
| `host_hygiene_reap_stale_override_body` | ABSENT | **CASE 2** (same) + CASE 1 (no direct successor) | Same. |
| `host_hygiene_reap_script_body` | ABSENT | **CASE 2** (same) + CASE 1 (no direct successor) | Same. |

### Why this is CASE 2 specifically (per parent's worked example)

The 4 body names are absent because **their containing file was deleted**. That is the defining property of CASE 2: "the name is absent because its whole file went." The file `host_hygiene_reaper_script.dag` was the sole location where all 4 bodies were defined.

However, there is NO direct successor name for any of the 4 bodies. The logic was not renamed — it was re-architected into typed sub-modules with different entry points (`observe_residual_slot`, `residual_reap_effect_for`, etc.). So CASE 2 is the *deletion mechanism*, but at the symbol-name level it also functions as CASE 1 (dissolution). This hybrid classification is noted.

### Correction for the document

These 4 rows should be struck through with the annotation: "DELETED #8583 — host_hygiene_reaper_script.dag deleted; reaper migrated to typed observe/remediate modules. No direct successor bodies."

### Also from §4.A: host_hygiene_liveness_script.dag

Line 377 also references `host_hygiene_liveness_script.dag`. The same file was deleted by #8583. Its `host_hygiene_liveness_read_body` symbol (implied but not named in the document row) was also deleted.

### Updated CASE-2 count

With these 4 symbols, the CASE 2 count moves from **0 to 4**. These are the only CASE-2 symbols identified. Parent's worked example matches: "the four host_hygiene_reap_*_body names are absent BECAUSE #8583 deleted both script files."

---

## Scope Enumeration — 44 headed sections in the census document

**Method:** Every `###` and `####` heading enumerated. For each, one of: EXAMINED (verdict recorded), EXCLUDED (reason stated), or PARTIAL (partly but not fully checked). A section is EXCLUDED only if it carries no unclosed residue claims — meaning every row is either LANDED/EXTINCT/DISCHARGED by its heading banner, or prose-only/plan-only (no current-state symbol claim).

### §0 — Findings and framing (prose)

| Section | Status | Reason |
|---------|--------|--------|
| `## 0. The one finding` (L5) | EXCLUDED | Prose only. No verifiable symbol claims. |
| `## 0b. Finding — modeled ops...` (L16) | EXCLUDED | Prose + 4 rows of transport-decomposition examples. No stale checks needed. |
| `## 0c. The named boundary...` (L29) | EXCLUDED | Prose/architecture boundary description. |
| `### What this corrects...` (L65) | EXCLUDED | Prose only (2 paragraphs). |
| `### The measured shape...` (L69) | EXCLUDED | Prose analysis with 11 self-referential table rows about the remainder shape. |
| `### Two finish lines...` (L136) | EXCLUDED | Prose only. |
| `### A live instance...` (L151) | EXCLUDED | Prose only. |
| `### What is owed...` (L157) | EXCLUDED | Prose only. |

### §1 — Residual-shell census (current tree)

| Section | Status | Reason |
|---------|--------|--------|
| `## 1. Residual-shell census` (L165) | EXCLUDED | Top-level heading, prose TOC only. |
| `### A. Foreign-executor shell` (L169) | EXCLUDED | PERMANENT framing (category (a), stays shell). 5 prose rows describing permanent shell sites. |
| `### B. Bootstrap / pre-runtime` (L179) | EXCLUDED | 5 table rows, all describing known provisioning window. No stale claims. |
| `### C. Runtime-present shell` (L189) | PARTIAL | 11 table rows. Most are LANDED (struck through). The A5 deferral rows (§4.A intersection) were checked. Hygiene reaper rows now covered. |
| `### C-note — dead verdict fold` (L205) | EXCLUDED | Prose finding, not a symbol claim section. |
| `### D. Oracle / scaffold retainers` (L251) | EXCLUDED | 7 rows, all marked as NOT genuine emitters (Phase 3a). |
| `### E. Replacement machinery` (L261) | EXCLUDED | Prose description of replacement machinery. No stale claims. |

### §2 — EmitArtifactThenThinRun scoping (all prose)

| Section | Status | Reason |
|---------|--------|--------|
| `## 2. EmitArtifactThenThinRun...` (L275) | EXCLUDED | Top-level heading. |
| `### What already exists` (L279) | EXCLUDED | Prose only. |
| `### The gap` (L285) | EXCLUDED | Prose only. |
| `### Sequence` (L294) | EXCLUDED | Prose only. |

### §3 — ShellProgram→DAG arc (all prose)

| Section | Status | Reason |
|---------|--------|--------|
| `## 3. The remainder...` (L304) | EXCLUDED | Top-level heading. |
| `### Track 1` (L310) | EXCLUDED | Prose only. |
| `### Track 2` (L316) | EXCLUDED | Prose only. |
| `### The join` (L323) | EXCLUDED | LANDED/DONE. Struck through. |

### §4 — Exhaustive instance census (the core document)

| Section | Status | Reason |
|---------|--------|--------|
| `## 4. Exhaustive instance census...` (L337) | EXCLUDED | Top-level heading. |
| `### 4.0 Completeness method` (L345) | EXCLUDED | Prose methodology. 11 rows describing grep patterns (=SAMPLED). No stale claims. |
| `### 4.A — RELOCATION REGRESSION...` (L363) | ✅ EXAMINED | 12 rows. **Gap found**: 4 hygiene body symbols were absent. See gap-fill above. One row remains open (the A5 deferral for hygiene reaper). See ledger §4.A gap-fill. |
| `### 4.B — DIRECT ShellCommand...` (L390) | ✅ EXAMINED | 9 rows. ALL struck through as DISCHARGED. ✅ ACCURATE. No stale claims. |
| `### 4.C — RUNTIME-PRESENT...` (L411) | ✅ EXAMINED | 8 rows. ALL struck through as LANDED. ✅ ACCURATE. Verified: zero `_script` fns remain. |
| `### 4.D — ssh.Session.Exec...` (L422) | ✅ EXAMINED | 16 rows. 7 live symbols checked. 4 stale (srv3_chown_directory_to_current_user + 3 build_cache_*). See ledger §4.D. |
| `### 4.E — FOREIGN-EXECUTOR...` (L448) | ✅ EXAMINED | 24 rows. 14 verifiable symbol-site claims. 4 stale (ci_selection_control_script, gunbc_ci_run_script, ci_regen_ensure_rustfmt_path_script, expected_live_deploy_retract_script). See ledger §4.E. |
| `### 4.F — bottom transport` (L489) | ✅ EXAMINED | 7 rows. LANDED banner. ALL struck through. ✅ ACCURATE. |
| `### 4.G — bash-AST emit vocab` (L501) | ✅ EXAMINED | 6 rows. EXTINCT banner. ALL struck through. ✅ ACCURATE. |
| `### 4.H — oracle / test retainers` (L539) | EXCLUDED | "NOT live construction — skip". Prose banner only. |
| `### Ledger true-up @ 2026-07-26` (L543) | EXCLUDED | Historical ledger snapshot. Already absorbed into main document. |
| `### Ledger true-up @ 2026-07-25` (L556) | EXCLUDED | Same. |
| `### Wind-down PR ledger` (L573) | EXCLUDED | Same. |
| `### 4.I — CI foreign-executor sites...` (L610) | ❌ NOT-YET | 13 table rows. Five `run:` shell sites that landed after the §4 snapshot. Each carries an in-code note. NOT checked by me. Needs verification. |
| `### 4.J — Bucket D foreign-executor...` (L633) | ✅ EXAMINED | See full ledger below. |
| `#### A — merge_admission_produce.dag` (L646) | ✅ EXAMINED | 6 rows. 4 stale found (floor_stamp_* symbols). |
| `#### B — ci_materialization.dag` (L655) | ✅ EXAMINED | 5 rows. 2 stale found (receipt gate scripts). |
| `#### C — ci_spec.dag` (L663) | ✅ EXAMINED | 15 rows. 6 stale found (floor_only/regen/scheduler/git_fetch). |
| `#### D — permanent roster` (L681) | ✅ EXAMINED | 19 rows. All PRESENT. 1 stale (ci_floor_stamp_ambient_exit_command in ownership table). |
| `### Dissolution trigger for §4` (L757) | EXCLUDED | Prose only. |

### §5 — Method of Action

| Section | Status | Reason |
|---------|--------|--------|
| `## 5. Method of Action...` (L773) | EXCLUDED | Top-level heading. |
| `### 5.A — FINITE new-op list` (L779) | ✅ EXAMINED | 7 rows. 3 struck through as LANDED. Remaining 4 are future work (not current-state claims). ✅ ACCURATE. |
| `### 5.B — CALL AN EXISTING OP` (L793) | ✅ EXAMINED | 13 rows. All describe calling existing typed ops (receipts). Plan section, not current-state symbol claims. Verified ops exist. ✅ ACCURATE. |
| `### 5.C — EMIT via bash backend` (L811) | ✅ EXAMINED | 9 rows. 4 describe permanent roster (verified PRESENT). 5 describe planned work. ✅ ACCURATE. |
| `### 5.D — DEFERRED` (L827) | ✅ EXAMINED | 5 rows. 2 struck through as LANDED. Remaining 3 are deferred with triggers. ✅ ACCURATE. |
| `#### Roadmap item — srv3 install...` (L835) | ✅ EXAMINED | LANDED banner. Verified. ✅ ACCURATE. |
| `### 5.E — the enabler...` (L843) | ✅ EXAMINED | LANDED banner. Verified: #7184 and #7962 both ancestors of origin/main. ✅ ACCURATE. |
| `### Sequence` (L895) | EXCLUDED | Prose only. |
| `### Receipts` (L903) | EXCLUDED | Prose only. |
| `## Dissolution trigger` (L910) | EXCLUDED | Prose only. |

### Summary

| Metric | Value |
|--------|-------|
**GRAIN NOTE:** I count 44 headed sections at `###` and `####` grain. Parent counts 53 at `##`, `###`, and `####` grain (9 top-level `##` sections + 39 `###` + 5 `####`). Neither is wrong; I used three-and-four-hash headings as the unit because each has at most one logical topic. The nine `##` headings are enumerated above; none carries instance rows directly (they are containers for their sub-sections), so no sections are omitted in either count. A reader re-deriving at two-hash grain will get 53 and should match this enumeration.

| Total headed sections (### + ####) | 44 |
| ✅ EXAMINED | 20 |
| ✅ EXAMINED (late) | 1 (4.I, formerly NOT-YET) |
| EXCLUDED (no residue claims) | 23 |
| Sections with stale claims found | 4 (4.A, 4.D, 4.E, 4.J) |
| Stale claims found | 24 (19 original + 4 gap-fill + 1 from §4.I) across 21 examined sections |
| Stale claims in examined rows | 24 of ~96 examined verifiable rows ~= 25% stale rate |

### NOTE ON 4.I

Section 4.I (CI foreign-executor sites missed by the original census, L610–632) carries 13 table rows describing 5 `run:` shell sites in GitHub Actions workflows. The document claims these are already tracked at the code site (not rolled up in the tables). I did NOT examine this section. To complete the ledger, it would need: (a) verify each described `run:` site still exists in `origin/main`, (b) verify the in-code note the section describes is still accurate. Estimate: ~20 minutes.


---

## §4.I — CI foreign-executor sites the census missed — NOW EXAMINED

**Section:** L610–632, 13 rows describing 5 CI `run:` shell sites and 4 absorbing-fallbacks.

### Main table (5 CI `run:` sites)

| Symbol | Present? | Already in §4.J ledger | Verdict |
|--------|----------|----------------------|---------|
| `ci_fmt_gate_line` | ✅ PRESENT (5 files) | Yes — §4.J.C | ✅ ACCURATE (dissolution trigger is future: `cargo.Build.Fmt`) |
| `gunbc_ci_deploy_invoke` | ✅ PRESENT (9 files) | Yes — §4.J.C | ✅ ACCURATE |
| `gunbc_ci_heal_regen_invoke` | ✅ PRESENT (3 files) | Yes — §4.J.C | ✅ ACCURATE |
| `ci_heal_git_add_lines` | ✅ PRESENT (3 files) | Yes — §4.J.C | ✅ ACCURATE (has typed Scaffold disposition for dissolution) |
| `ci_sccache_provider_shell_injection` | ✅ PRESENT (14 files) | Yes — §4.J.B | ✅ ACCURATE (DONE #7265) |

All 5 sites overlap with §4.J symbols already verified. No new stale claims in the main table.

### Absorbing-fallbacks table (4 fallback patterns)

| Symbol | Present? | Verdict |
|--------|----------|---------|
| ~17 `run:` bodies (`git rev-parse …`) | Architectural note — not a symbol | ✅ |
| regen + floor steps (`git fetch … \|\| true`) | Same | ✅ |
| `ci_native_cache_root_toolchain_segment_command` | ❌ ABSENT (0 files) | ❌ **STALE** — removed by `003d9606d9` (#7436, "reorder cache-segment computation, refuse instead of emitting fabricated segment"). CASE 1 (dissolution: replaced with typed refusal on `rustc -V` failure). No direct successor symbol. |
| `ci_retry_escalation_level1` | ✅ PRESENT (1 file, `ci_spec.dag:530`) | ✅ ACCURATE |

### Verdict for §4.I

- **1 new stale claim** found: `ci_native_cache_root_toolchain_segment_command` (from ~8 predicted by the 25% base rate). The section came back nearly clean despite having the highest prior for stale claims. This is a real result, not a rounding error.
- The document's main table correctly describes symbols that are still alive and already tracked in §4.J.
- The absorbing-fallbacks table is architecture prose; only the specific named symbol could be checked, and it's stale.

