# Space-lens minimal calibration — floor memory lower-bound pairs

**Status:** INTERIM seed scaffold (2026-07-10). Emits labeled calibration triples on the live CI/falsifier floor path until resolver graph-major + skip-before-resolve shrink the resident set the pairs measure.

**Authority:** `roster_import_closure_nodes_pre_resolve` in `src/v1/stage0/src/cli_run.rs` — the deduped transitive import-closure of every discovery row plus prefix-context entries (`FLOOR_RUNNER_ENTRY`, `ENTRY_SELECTION_ENTRY` when selection is on), counted at module-path grain via the pure import walk (no typecheck). The space-lens predictor and floor calibration emission MUST bind here; no re-derivation in comments or parallel counters.

**Calibration triple (paired observations):**

| Label | Carrier | Meaning |
|---|---|---|
| `roster_import_closure_nodes` | stderr, pre-resolve | Import-closure module count (not roster row count) |
| `floor_peak_pre` / `floor_peak_post` | cgroup `memory.peak` steps in `gunbc.ci_workflow` | Job-scoped memory peak; `floor_outcome=success` is exact, killed runs are censored lower bounds |
| `floor_outcome` | post step | GitHub Actions step outcome paired with peak |

On a host-OOM kill, the last `[gantt] rss_mib` sample plus `roster_import_closure_nodes` (emitted before heavy resolve) and `floor_peak_post` (emitted in `always()` post step) form the lower-bound receipt. Bytes-per-node = peak ÷ closure_nodes; censored-at-cap observations are strictly greater than the read.

**Width-1 drift oracle:** On a completed width-1 run, `roster_import_closure_nodes_pre_resolve` must equal `DiscoverySummary.roster_closure_nodes` (post-resolve union). Mismatch localizes resolve seeding or import-walk definition drift.

**Dissolve-on triggers:**

1. **Skip-before-resolve** ([affected-set precompute](affected-set-precompute-pruning.md) step 4): when selection skips rows before resolve, the pre-resolve closure walk must count the *selected resident subset*, not all discovered rows — otherwise the pair overcounts.
2. **Bash-emit (#5828 / ROADMAP `6-shell-slice0`):** cgroup peak locate/reset/read shell in `gunbc.ci_workflow` retires when orchestration emit realizes the calibration steps through `RealizationDispatch` (dissolution triggers on `ci_cgroup_peak_locate_shell`, `ci_floor_peak_pre_script`, `ci_floor_peak_post_script`).
3. **Resolver graph-major** ([design](resolver-graph-major-design.md)): persistent node-keyed store + module-grained resolve shrinks the closure the calibration measures; this doc's interim seed hooks dissolve into the store-backed predictor.

**ROADMAP:** §2 *space-lens minimal calibration* row.

**Not in scope here:** selection skip-rate receipts for entry-file-touched diffs (substrate-not-whole-tree widen — loyal-wren lane); falsifier predict_only control bin wiring.
