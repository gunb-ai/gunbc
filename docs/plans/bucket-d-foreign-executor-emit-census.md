# Bucket D re-census — foreign-executor emit cluster

> **Status:** fresh census @ `origin/main` `95e4bbad69` (post-#7216 merge, 2026-07-26). Supersedes stale §4.E rows in `shell-to-dag-residual-census-and-arc-completion.md` for CI placement files. Authority: `shell-intent-emit-realization-design.md`, `host-effect-orchestration.md`.

## Scope

Foreign-executor shell (GHA `run:`, cron, git hooks, pre-runtime bootstrap) **legitimately stays shell**. Bucket D routes each site from hand-built `concat` command strings onto `Pipeline`/`EffectPlan` intent emitted through `orch_emit_pipeline(medium: bash_orchestration_emit_medium())` — a **bounded roster**, not a growth surface. Invariant: intent imports no `bash_build` / `ShellProgram` / `serialize_bash` (`realization_vocabulary_containment`).

## Already on emit (do not re-migrate)

| module | symbols | executor |
| --- | --- | --- |
| `v2.workflow.ci_workflow_run_emit` | `ci_isolate_toolchain_script`, `ci_pin_rustup_default_script`, `ci_selection_control_script` | GHA `run:` |
| `v2.workflow.ci_floor_peak_emit` | `ci_cgroup_peak_locate_shell`, `ci_floor_peak_pre_script`, `ci_floor_peak_post_script` | GHA `run:` |
| `v2.workflow.ci_retry_emit` | `ci_cargo_eagain_retry_script` | GHA `run:` (via `ci_release_build_emit`) |
| `v2.workflow.ci_release_build_emit` | `ci_release_build_script`, `gunbc_ci_run_script` | GHA `run:` (partial — still concat-wraps verify script) |
| `gunbc.assimilate.bmc_token_federation` | `gcp_token_smoke_script` | GHA `run:` |
| `gunbc.live_deploy.emit` | `expected_live_deploy_apply_script`, `expected_live_deploy_retract_script` | GHA deploy `run:` |
| `gunbc.host_effect` | `fresh_standup_bootstrap_intent` → `EmitArtifactThenThinRun` bootstrap arm | pre-runtime bootstrap |

## Typed-op candidates (route to extdeps op, NOT bash emit)

| site | current | target op | notes |
| --- | --- | --- | --- |
| `ci_spec.dag` · `ci_fmt_gate_line` | `"$CARGO_BIN" fmt --all --check` | `cargo.Build.Fmt` | `ci_fmt_gate_note` already names dissolve-on; **load-bearing ci.yml** |
| `ci_deploy_access_emit.dag` · `deploy_access_emit_principal_read_script` | `"whoami"` | `os.Id.Lookup` / effective-principal read | deploy preflight; Wave C typed argv |
| `fleet_posix_accounts.dag` · `declared_probe.probe_command` | `"id root"` etc. | **NOT a shell emit site** | authored provenance metadata only; live proof is `deploy_access_check_observed` via typed `SudoNopasswdExecuteProbe` |

## Deferred / out of bucket D

| site | reason |
| --- | --- |
| `roadmap_static_site.dag` body fns | HTML/JSON content emit, not shell — belt B (`gunbc serve`) |
| `runner_host_deploy.dag` · `runner_host_docker_provision_script` | fleet manual-gap lane |
| `bmc_virtual_media.dag` srv4 gadget scripts | BMC provisioning, separate lane |
| Runtime-present `host_effect_apply` / srvN tails | bucket B (typed effects), not foreign-executor emit |

## Remaining concat-built foreign-executor punch-list

### A — `merge_admission_produce.dag` (5 script surfaces)

| symbol | GHA consumer | emit complexity |
| --- | --- | --- |
| `ci_floor_disposition_marker_init_script` | floor opener (via `gunbc_ci_floor_only_script`) | low (comments + mkdir + redirect) |
| `ci_documentation_only_gate_skip_prefix` | receipt gates, merge gate, selection control | medium (nested if/test + cmdsubst) |
| `ci_merge_admission_stamp_script` | merge-admission stamp step | low |
| `ci_merge_admission_gate_script` | merge-admission gate step | medium |
| `ci_floor_stamp_merge_admission_script` | floor tail | medium (`$?` capture) |

### B — `ci_materialization.dag` (3 script surfaces)

| symbol | GHA consumer | emit complexity |
| --- | --- | --- |
| `ci_sccache_provider_shell_injection` | isolate-toolchain pipeline | medium (`if` + env append + printf receipt) |
| `ci_floor_materialization_receipt_gate_script` | ci job gate | high (sed parses + if ladder) |
| `ci_floor_resolve_receipt_gate_script` | ci job gate | high (sed + numeric compare) |

### C — `ci_spec.dag` (~15 distinct script composers feeding GHA)

| symbol | GHA consumer | route |
| --- | --- | --- |
| `ci_release_build_line` | build (via retry emit) | emit (partially done) |
| `ci_fmt_gate_line` | build fmt step | **typed op** |
| `ci_floor_build_verify_script` | build verify + unpack | emit (build_step transport pattern) |
| `ci_release_bins_pack_script` | build pack artifact | emit |
| `ci_release_bins_unpack_verify_script` | ci unpack step | emit |
| `gunbc_ci_floor_only_script` | ci floor `run:` | emit (composes A rows) |
| `ci_regen_floor_skip_shortcut_script` | regen skip | emit (`if` + cmdsubst) |
| `gunbc_ci_regen_floor_only_script` | regen job | emit (composer) |
| `gunbc_ci_deploy_invoke` | deploy job | emit (thin gunbc run) |
| `gunbc_ci_heal_regen_invoke` | heal regen | emit |
| `gunbc_ci_heal_commit_push_script` | heal commit/push | emit (git verbs; future typed git ops) |
| `scheduler_invoke` / `scheduler_invoke_with` | floor/regen | emit (thin claim_executor) |
| `git_fetch_script` | floor/regen/deploy | emit (`git fetch` + fallback) |

**Load-bearing:** every C-row change regenerates `.github/workflows/ci.yml`; drift+parse gate is the byte-oracle.

### D — permanent roster (emit target, never typed-op dissolve)

| site | executor |
| --- | --- |
| `gunbc.githooks_pre_push_emit` · `expected_githooks_pre_push_sh` | git pre-push hook |
| cron entry lines (REST transport witness corpus) | cron |

## Proposed batching (3 PRs)

### PR 1 (this session) — census + materialization/merge-admission emit seed

- This document (re-census authority for bucket D).
- `ci_sccache_provider_shell_injection` → `v2.workflow.ci_materialization_emit` (If-band, byte golden).
- `ci_floor_disposition_marker_init_script` → `v2.workflow.ci_merge_admission_emit` (byte golden).
- Witness tests + `realization_vocabulary_containment` roster rows.
- Regenerate `ci.yml`; prove drift gate green.

**Does not touch** `ci_spec.dag` script composers beyond import delegation through existing call graph.

### PR 2 — merge-admission cluster completion + receipt gates

- `ci_documentation_only_gate_skip_prefix`, stamp/gate/stamp scripts, both receipt gate scripts (B rows).
- Depends on PR 1 emit helpers (`emit_pipeline_or_poison` pattern).

### PR 3 — `ci_spec` composer migration (operator review gate)

- Pack/unpack/verify, regen skip, deploy/heal invokes, floor/regen composers.
- `ci_fmt_gate_line` → typed `cargo.Build.Fmt` (separate commit within PR).
- Full `gunbc ci` regen + `ci_spec_witness_test` + committed `ci.yml` byte-oracle.

**Operator review required before PR 3** — load-bearing CI generator.
