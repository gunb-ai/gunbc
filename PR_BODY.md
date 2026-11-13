
Part 2 of the `gunbc.plans.closure_consolidation` fleet-mode decomposition, far-modes lane. `gunbc.ci.ci_spec` still carried, beside the shared run-step machinery, one `data ..._target` + one `fn ..._invoke` per mode. Four modes here, each now living beside its own domain in a step module the domain owns: four run steps out of ci_spec into the modules that own them.

## What moved (10 declarations, verbatim)

| mode | target | invoke | new home |
|---|---|---|---|
| NamecheapObserve | `gunbc_ci_namecheap_observe_target` | `gunbc_ci_namecheap_observe_invoke` | `gunbc.namecheap.observe_step` |
| PrinterWorkflow | `gunbc_ci_printer_workflow_target` | `gunbc_ci_printer_workflow_invoke` | `gunbc.fleet.printer_workflow_step` |
| ReviewSheetConverge | `gunbc_ci_review_sheet_converge_target` (+ its two-line comment) | `gunbc_ci_review_sheet_converge_invoke` | `gunbc.review_sheet_converge_step` |
| ArrivalConverge / arrival_admit | `gunbc_ci_arrival_admit_target`, `gunbc_ci_arrival_converge_target` | `gunbc_ci_arrival_admit_invoke`, `gunbc_ci_arrival_converge_invoke` | `gunbc.machine_intake.arrival_step` |

Every moved declaration was byte-diffed against its main-side source (`diff` clean on all 10), the review-sheet comment block travelling with its target.

## Why each lives there

A step module imports the shared machinery, never `ci_spec` (the precedent: `gunbc.machine_intake.mtcollins1_boot_step` #16348, `gunbc.auth.approval_ci_step`). Each new module imports `gunbc_run_step_script` from `v2.workflow.gunbc_invoke_step_emit`, `witness_layer_roots` from `gunbc.ci.ci_layer_roots`, `GunbcRunStepTarget` from `gunbc.ci.ci_run_step_target`, and — where the invoke writes a receipt — its domain's receipt path (`namecheap.observation_artifact`, `fleet.printer_workflow_artifact`).

- **namecheap** → `gunbc.namecheap.observe_step`, in the namecheap dir beside `observe.dag` (its `entry`).
- **printer** → `gunbc.fleet.printer_workflow_step`, in the fleet dir beside `printer_workflow_run.dag` (its `entry`) and `printer_workflow_artifact.dag` (its receipt path); the printer domain's home.
- **review sheet** → `gunbc.review_sheet_converge_step`, beside the `review_sheet_converge_*` family (`review_sheet_converge_cli.dag` is its `entry`).
- **arrival** → `gunbc.machine_intake.arrival_step`, a sibling of `mtcollins1_boot_step` — it is arrival, not mtcollins1.

## ci_spec

Keeps the roster `gunbc_run_step_targets`; it now imports the five targets from the four step modules (the way it already imports `mtcollins1_boot_step`'s targets). It dropped the two receipt-path imports (`namecheap_observation_receipt_path`, `printer_workflow_receipt_path`) whose only consumer was the invoke that moved with them.

## fleet_converge_workflow

Its `ci_spec` import cluster is trimmed to the five admit/workspace invokes still defined there (`workspace_allocation_plan`, `admit_plan_run_pointer`, `admit_apply_run_pointer`, `admit_dashboard_deploy_inputs`, `assert_dashboard_deploy_receipts`); four new imports pull the five mode invokes directly from the step modules. `host_budget_preflight_invoke` is no longer on that line — after merging `origin/main` it correctly resolves from `gunbc.host.host_fleet_run_step` (host lane #16447). The two image-build invokes likewise resolve from `gunbc.image_ci_step` (#16440), not ci_spec; the merge keeps them there. The per-mode match functions are untouched (the parent's next step).

## Citations of the moved symbols

The only references to these names outside `ci_spec` and `fleet_converge_workflow`, both checked:

- `test.claim.printer_state_parent_reown_route_witness` — imported `gunbc_ci_printer_workflow_target` + `..._invoke` from `ci_spec`; repointed to `gunbc.fleet.printer_workflow_step`. Its `import gunbc.ci.ci_spec` line is now gone.
- `gunbc.recurring_failure_mode.receipt_consumer_without_producer` — a historical PR-narrative string inside an `examples` row that names `gunbc_ci_namecheap_observe_invoke` by bare name describing PR #12421. Not a module-qualified citation (`replacement_migration`); the symbol name is unchanged, so the sentence still reads true and it stays put.

No `gunbc.design_argument::replacement_migration` row cites any moved symbol. No untracked file references them.

## Unconsumed invokes

None. All five invokes are called by `fleet_converge_workflow`'s per-mode job-body builders, so none is stranded for the P5 deletion sweep.

## Witness closure

`test.claim.printer_state_parent_reown_route_witness` no longer imports `ci_spec` directly (the sole `import gunbc.ci.ci_spec` line was the one repointed). It still reaches `ci_spec` transitively, through `import gunbc.fleet.fleet_converge_workflow`. Closure **before 2297 → after 2301**; the +4 are exactly the four new step modules, now reachable through `ci_spec` (which imports the targets) and `fleet_converge_workflow` (which imports the invokes). No module dropped out of the closure. (Measured head-vs-base at the current merge base `3b589d2336d`; the other two touched witnesses move +4 the same way: `namecheap_workflow_witness` 2303→2307, `printer_workflow_binding_witness` 2298→2302.)

## Re-merge (import-block union, head `8612d3dd9c8`)

Main advanced past the original head (`ab849f00c90`) with host #16447, runner #16444, PXE #16335/#16446 and credentials #16190, which rewrote the `ci_spec` and `fleet_converge_workflow` import blocks. Per the parent's guidance I merged `origin/main` (not rebase) and took the union of the step-module imports. **Both conflicts were in import blocks only** — I re-verified all ten moved declaration bodies against the new `origin/main` `ci_spec` and they are byte-for-byte identical, so the render proof below stands unchanged.

The resolution took main's authoritative import blocks and re-applied this lane's delta:
- `ci_spec`: drops the 10 far-mode decls, imports the 5 targets from the 4 step modules, drops the 2 now-unused receipt-path imports.
- `fleet_converge_workflow`: pulls the 5 invokes from the step modules; the 5 admits stay on `ci_spec`; `host_budget_preflight_invoke` now comes from `gunbc.host.host_fleet_run_step`.

Re-run checks on the merged head (base `3b589d2336d`):
- **`scc.py`: 0 cycles** over 10,337 modules (count grew with the merged lane modules).
- **`unbound.py`: zero regression** — `ci_spec` `[]` on both trees; `fleet_converge_workflow`'s unbound set on head is **byte-identical** to main's (the same six pre-existing host-lane/heuristic names). My import edits introduced no new unbound names.
- **`unused.py`: zero regression** — counts match main exactly (`ci_spec` 16, `fleet_converge_workflow` 17); the two receipt-path imports I removed are absent from both (they were used by the moved invokes on main, gone here).
- **`clo.py`: step modules `ci_spec=False`** — `observe_step` 402, `printer_workflow_step` 413, `review_sheet_converge_step` 401, `arrival_step` 401, none reaching `ci_spec`.
- **Stale-import symbol check** (covers what `unbound.py` cannot): every symbol the new imports pull still exists in its source module at the head — the 5 admits are still `fn`-defined in `ci_spec`, and `gunbc_run_step_script`, `witness_layer_roots`, `GunbcRunStepTarget`, `printer_workflow_receipt_path`, `namecheap_observation_receipt_path` all resolve in their modules.
- `git diff --stat origin/main HEAD` is exactly the 7-file lane delta (ci_spec, fleet_converge_workflow, the 4 step modules, the reown witness), confirming no other lane's content was touched.

## Evidence

**Render — byte-identical.** `test.manual.bo603_render` renders all eight committed workflows through the production `artifact_generate` on both trees.

| tree | bo603 bytes | sha256 |
|---|---|---|
| base `c6a0844d791` | 550,079 | `fa710d057f0c1ec8…` |
| head `ab849f00c90` | 550,079 | `fa710d057f0c1ec8…` |

`cmp` result: **identical** (`RENDER_IDENTICAL=yes`). The head tree is `origin/p2-far-modes-steps` and the base is the PR merge base `c6a0844d791`, so the comparison is exactly this lane's 7-file delta. Both `gunbc run` exits 2 (it refuses a `String` return — the value lives in stderr between the `=====BO603` markers); the captured render is the eight committed workflow artifacts, including this lane's `namecheap-observation`, `printer-operation` and `arrival_converge` — so the byte-identity is over the workflows the moved steps feed. The re-merge changed import lines only and no moved declaration body (re-verified verbatim against the new main), so this byte-identity carries to head `8612d3dd9c8`.

**Witnesses — green on head** (`claim_batch --hermetic`, run on the head tree):

| witness | claims | result |
|---|---|---|
| `namecheap_workflow_witness` | 4 | **PASS** (4/0, EXIT=0) |
| `printer_workflow_binding_witness` | 6 | **PASS** (6/0, EXIT=0) |
| `printer_state_parent_reown_route_witness` | 1 | **PASS** (1/0, EXIT=0) — the witness whose `ci_spec` import was repointed to `gunbc.fleet.printer_workflow_step` |
| `review_sheet_converge_workflow_witness` | 5 | **PASS** (5/0, EXIT=0) |
| `workflow_dispatch_input_witness` | 42 | known red, **identical head vs main** (below) |

`workflow_dispatch_input_witness` is one of the two reds owned by another session. Run with its full 42-function list on both trees: head `EXIT=1`, 0 PASS / 42 FAIL; main control `EXIT=1`, 0 PASS / 42 FAIL; the FAIL sets are byte-identical. Every failure is the same pre-existing diagnostic — `workflow_dispatch_input_witness.dag:656:8: error: effect summary incomplete: call to 'unique' ... has no established callee identity` — an unimported `unique` in `the_password_session_tool_mode_is_on_the_roster_with_distinct_wires`, a mode outside this lane. Not introduced here.

**Import graph.** `scc.py` on the merged tree: **0 cycles** (10,337 modules). None of the four new step modules reaches `ci_spec` (see Re-merge). No new unused or unbound imports: `ci_spec` stays at its pre-existing count and `fleet_converge_workflow`'s unused/unbound sets are identical to the merge base.
