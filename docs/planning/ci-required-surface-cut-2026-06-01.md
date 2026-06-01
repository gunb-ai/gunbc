# CI Required Surface Cut — Wave 1 Honesty Ledger

**Status:** ACTIVE — operator-ratified per `#4137` §11.7 (2026-06-01).  
**Authority:** `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §11.7.1–§11.7.6.  
**Implementation:** `.github/workflows/ci.yml` (Wave 1 PR).

Reduced required confidence is explicit. This is triage, not a modeling victory.

---

## Required on every non-draft PR (Class A — five gates)

| # | Gate | Mechanism in `ci.yml` | Notes |
|---|------|----------------------|-------|
| 1 | `fmt --check` | `fmt` job | ~30s |
| 2 | Bootstrap minimal viability | `ci_floor` → `v4-bootstrap-viability.sh` | v2→v4 `--target dag`; Class A shell exception until Wave 2 `CiUpsertStep` |
| 3 | M1 v4 full-tree rust emit probe | `ci_floor` → `v4-m1-rust-emit-probe.sh` with `emit_preconditions_block_required_path=true` and `V4_M1_RUST_EMIT_PROBE_STRICT=0` / `rustc_residuals_block_required_path=false` | **One** rust emit path; T-22 corpus rust **not** required; missing compiler, v2 emit failure, and skipped cargo-check preconditions fail closed; rustc residuals are a required receipt until P3 reaches binary-build threshold |
| 4 | `ci.dag` structural receipt + `CiUpsertStep` schema | `ci_floor` → `v4_workflow_ci_*` integration prefix filters | Modeled-positive-Y via parse harness |
| 5 | no-new-shell ratchet | `ci_floor` → `check-ci-no-new-shell.sh` | Allowlist of shell scripts on required path |

**Branch-protection check names:** `fmt`, `ci` (aggregator over `ci_floor` only — **not** `affected`).  
**Cold PR target:** ≤10min wall (was ~30–75min pre-cut).

### P5(b) receipt — `v4_workflow_ci_wave1_*` integration harness (this PR)

| Field | Value |
|-------|--------|
| **Deleted scaffold** | None — net **shrink** of required CI surface; new assertions bind existing `ci.dag` ↔ `ci.yml` for Wave 1 cut only |
| **SG-0 hand-Rust delta** | `0` (no new `src/v3/compiler/src/**` product code; two prefix-filter tests in existing `v4_workflow_ci_runner_dag_smoke_test.rs` integration harness) |
| **ROADMAP / charter row** | `#4137` §11.7.7 Wave 1 deliverable — honesty ledger + five-gate floor |
| **Deferral / dissolution** | `v4_workflow_ci_wave1_*` + updated `bankruptcy_tier0_*` YAML assertions dissolve when A15 Shape-B emits `ci.yml` from `ci_pipeline` (same posture as file header on smoke test) |

**Four-compile collapse (Wave 1):** Removed duplicate compiles from retired `ci_v4` path (Lens-CI semantic compile, MVP-1, T-15 harness, T-22 corpus eval, phase1 rung gate). Required path now runs **one** v2 `gunbc` build + **two** full-tree compiles (M1 `--target rust`, bootstrap `--target dag`) — not four. The M1 Rust probe must run and publish its residual surface, but does not fail the required path on the known rustc residual population while P3 remains below the binary-build threshold.

---

## Deleted from required CI (Class D — permanent)

| Item | Restoration |
|------|-------------|
| `discipline` job and all `scripts/check-*` discipline steps (SG-0, R4-carve, fabrication, release-doc, manager-brief, test-timeout, rust-toolchain, T-19 py self-test, install.sh smoke) | Pre-commit / nightly / manual; or `CiUpsertStep` when affected-set selects them (Wave 3+) |
| **All v3 integration tests** — `determinism_test`, `self_host_fixed_point`, v3 bootstrap `--verify`, v2 stage0 `--verify` on required path | **Permanent delete** per operator 2026-06-01; v3 lane retired |
| `ci_integration` job (Gate #103 integration, leaf-model bypass tests, RELEASE §5 binding, Lens-CI registry smoke, T-15 bin smoke) | Wave 3+ via TestClaim/affected-set; structural receipts stay in integration crate for local/scheduled |
| `ci_v4` job — T-15 harness, MVP-1, Lens-CI semantic compile, phase1/nat_semiring, T-22 corpus eval, bootstrap two-step advisory pattern | Same |
| Top-level legacy jobs `v2`, `v3`, `v4`, `self_host_ratchet` | Already deleted (B0); unchanged |

---

## Demoted to shadow / manual / scheduled (Class C)

| Item | Notes |
|------|-------|
| `check-r4-carve-dissolution-discipline.sh` | Prose-grep doc hygiene |
| `check-rust-toolchain-single-authority.sh` | Dissolve when extdeps generates toolchain |
| `check-test-timeout.sh` | Dev discipline |
| `check-manager-brief-authority.sh`, `check-release-doc-authority.sh` | Pre-commit material |
| Leaf-model verify shells (`v4-leaf-model-*-verify.sh`) | Substrate landed; integration tests demoted with `ci_integration` |
| T-22 TestClaim corpus eval (`v4-testclaim-corpus-eval.sh`) | Modeled in `ci.dag`; **not** on required path; duplicates M1 signal per §11.7.1 |
| T-15 self-host harness in CI | Modeled `V4T15SelfHostFixedPointCommand`; scheduled/main-push only when re-enabled |
| Gate #103 path-regex + affected-set integration tests in CI | Stay in crate; run locally / Wave 3 shadow |
| `affected` job (`detect-ci-affected-components`) | Runs with `continue-on-error: true`; **not** in `ci`/`ci_floor` `needs:` — Wave 3 shadow receipts only per §11.7.2 |

---

## Cut until modeled (Class D backlog)

| Item | Restoration |
|------|-------------|
| `check-pr-sg0-net-shrink-discipline.sh` | Retired with v3 lane |
| `check-compiler-std-ratchet.sh`, `l1-ratchet.sh` | v3 `*.dag` retired |
| `r3-debt-velocity.sh`, `r1_p0_no_fabrication_sentinel.sh` | Reporting / retired tree |

---

## Temporary shell exception table (§11.7.5)

| Script | Protects | Dissolution |
|--------|----------|-------------|
| `v4-bootstrap-viability.sh` | P2/P3 bootstrap path | `V4BootstrapStageCompile` `CiUpsertStep` + emitter |
| `v4-m1-rust-emit-probe.sh` | M1 residual surface | `M1RustEmitProbeCommand` projected run |
| `check-ci-no-new-shell.sh` | `project_no_new_shell` | Emitter reads `ci_pipeline` only; ratchet deleted |

---

## Explicit non-goals (this PR)

- Affected-set **exclusive** skip (Wave 4; requires Wave 3 shadow receipts)
- Migrating every YAML step to `CiUpsertStep` for completeness
- Updating `dsl/gunbc/ci_github_actions_workflow.dag` consumers beyond generator regen
