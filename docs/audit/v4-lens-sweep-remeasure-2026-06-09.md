# v4 Lens Sweep Re-Measure

**Date:** 2026-06-09
**Session:** bright-pike-883
**Work item:** node://adhoc-5d764618-b85
**Mode:** read-only measurement by execution

## Commands Run

```bash
ctrl-build -- cargo build --release -p v2-compiler --bin gunbc
ctrl-build -- target/release/gunbc compile --source-root src/v4 --output-dir /tmp/bright-pike-883-v4-compile-out --target rust+dag
ctrl-build -- scripts/v4-lens-ci-gate.sh --perturb-check
ctrl-build -- scripts/v4-affected-set-node-frontier-gate.sh --perturb-check
```

The affected-set/testgen command has two modeled row authorities in the same
host gate:

- `src/v4/test/claim/workflow/affected_set_ci_runner.dag`
  (`ci_runner_node_frontier_claim_run_rows`,
  `ci_runner_node_frontier_claim_run_row_count = "9"`)
- `src/v4/test/claim/workflow/affected_testgen_ci_runner.dag`
  (`affected_testgen_claim_run_rows`,
  `affected_testgen_claim_run_row_count = "6"`)

Raw local logs were captured at:

- `/tmp/bright-pike-883-v4-compile.log`
- `/tmp/bright-pike-883-v4-lens-ci.log`
- `/tmp/bright-pike-883-affected-testgen.log`

## Counts By Execution

| Surface | Execution result | Count |
|---|---:|---:|
| `src/v4` compile, `rust+dag` | passed | 423 emitted files, 0 diagnostics |
| `src/v4` compile, Rust target slice | passed | 422 emitted files, 0 diagnostics |
| `src/v4` compile, DAG target slice | passed | 1 emitted file, 0 diagnostics |
| Lens CI witness rows | passed | 4 discriminating witnesses |
| Lens CI perturb reds | expected fail | 4 forced-false perturb failures |
| Affected-set node-frontier witness rows | passed | 9 discriminating witnesses |
| Affected-set node-frontier perturb reds | expected fail | 9 forced-false perturb failures |
| Affected-testgen witness rows | passed | 6 discriminating witnesses |
| Affected-testgen perturb reds | expected fail | 6 forced-false perturb failures |

## SB-c/SB-d Resolved Error Check

The `src/v4` `rust+dag` compile completed with `0 diagnostics`.
By execution, the previously reported 14 SB-c/SB-d resolved errors are absent from this run.

## Nine Reds

The 9 reds are not live compile diagnostics. They are the intentional `--perturb-check`
failures from `scripts/v4-affected-set-node-frontier-gate.sh`, produced after the gate
copies `src/v4` to a temporary source root and rewrites each witness function body to
`false`. Each row passed in the unmodified source and failed only under perturbation.

| Red | Class | Root cause |
|---|---|---|
| `affected-set node-frontier phase-1 narrow selection` | perturbation receipt | Witness body rewritten to `false`; confirms the narrow rerun-node selection row is discriminating. |
| `affected-set node-frontier phase-2 fail-closed superset` | perturbation receipt | Witness body rewritten to `false`; confirms fail-closed selection widens to the full roster. |
| `affected-set node-frontier phase-3 shape collision` | perturbation receipt | Witness body rewritten to `false`; confirms shape-collision selection is exercised. |
| `affected-set node-frontier phase-4 inner frontier` | perturbation receipt | Witness body rewritten to `false`; confirms inner-frontier traversal is exercised. |
| `affected-set discovered==selected narrow frontier` | perturbation receipt | Witness body rewritten to `false`; confirms discovered narrow frontier equals selected claims. |
| `affected-set discovered==roster fail-closed widen` | perturbation receipt | Witness body rewritten to `false`; confirms fail-closed discovery equals the full roster. |
| `affected-set discovered==selected shape collision` | perturbation receipt | Witness body rewritten to `false`; confirms collision discovery equals selected claims. |
| `affected-set discovered==selected inner frontier` | perturbation receipt | Witness body rewritten to `false`; confirms inner-frontier discovery equals selected claims. |
| `affected-set node-frontier four-phase discrimination` | perturbation receipt | Witness body rewritten to `false`; confirms the aggregate four-phase discrimination row is wired. |

## Foundation Status

The lens-CI foundation that affected-set and affected-testgen rest on is green by execution:
`scripts/v4-lens-ci-gate.sh --perturb-check` reported 4 discriminating lens witnesses passed,
and every lens row failed under forced-false perturbation. The downstream
`scripts/v4-affected-set-node-frontier-gate.sh --perturb-check` also passed by execution
against both of its modeled rosters: 9 affected-set node-frontier witnesses projected from
`affected_set_ci_runner.dag`, and 6 affected-testgen witnesses projected from
`affected_testgen_ci_runner.dag`, with their corresponding perturb reds observed.
