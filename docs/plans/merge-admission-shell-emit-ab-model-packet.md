# Merge-admission shell→DAG A/B model packet

> **Status: MODEL PACKET @ `611fd701` (eager-boar-42).** Investigation-only deliverable for
> roadmap `2-emit-partition` PR-2 (merge-admission cluster). No code migration in this packet —
> model-before-implement: carriers land before emit migrations. Authority: DESIGN shell→intent open
> thread, ROADMAP `2-emit-partition`, census
> [§4.J.A](shell-to-dag-residual-census-and-arc-completion.md#4j--bucket-d-foreign-executor-emit-punch-list--post-7216-wise-crane-222-2026-07-26).

---

## 0. Scope and rejection rule

**In scope:** concat builders `ci_documentation_only_gate_skip_prefix`,
`ci_merge_admission_stamp_script`, `ci_merge_admission_gate_script`; raw floor-tail leaves
`ci_floor_stamp_ambient_exit_command`, `ci_floor_stamp_root_command`, `merge_admission_stamp_command`;
materialization receipt composers `ci_floor_materialization_receipt_gate_script`,
`ci_floor_resolve_receipt_gate_script`.

**Out of scope (separate §4.J rows):** `gunbc.ci_spec` composer bodies (charter: do not touch
`ci_spec` implementation), receipt-gate `sed` parsing internals beyond skip-prefix sharing,
`ci_deploy_access_emit` runtime regression.

**Rejection rule (operator, shell→intent):** a plan that merely moves shell into `Run.command` /
`String` concat is **not** completion. Typed orchestration intent must be emitted through
`v2.compiler.emit_orchestration`; foreign-executor `gunbc run` invocations remain as modeled
`Do{Run{…}}` leaves with typed env/capture, not hand-concatenated script strings.

---

## (a) Domain types that must land first (`std/` / `extdeps/`)

| type / carrier | home | why before migration | live precedent |
| --- | --- | --- | --- |
| **`Predicate.FileExists { path }`** | `v2.std.orchestration.Predicate` + bash emit row | skip-prefix opens with `test -f target/ci-floor-disposition.txt`; today only `ExitZero{run: Run{command: "test -f …"}}` is possible — that is shell-in-`Run.command`, not a predicate | named PR2b residue in `ci_merge_admission_emit.dag`; sketched in [orchestration-as-intent-design.md](orchestration-as-intent-design.md) §3 |
| **`CiFloorDisposition` enum** (or typed path content) | `gunbc.merge_admission_produce` (domain) projected to `StrEq` at emit edge | skip compares `$_disp` to `documentation_only_skipped`; disposition constants already exist as `data` rows — need a typed read arm, not raw `$(cat …)` cmdsubst | `merge_admission_floor_disposition_documentation_only_skipped`, `merge_admission_floor_disposition_floor_running` |
| **`CaptureSpec.PriorExitStatus { binder }`** (or pipeline-sequenced `ExitStatus` on prior step) | `v2.std.orchestration` | `FLOOR_EXIT=$?` is ambient prior-command capture **outside** the floor-tail `Pipeline` today; PR2a explicitly declined `CaptureSpec-on-Run` for this | `ci_floor_stamp_pr2b_residue_note` |
| **`ReceiptFieldParse` predicates** (keyed file parse) | `gunbc.ci_materialization` domain + orchestration `Let`/`StrEq`/`IntNe` | materialization/resolve gates use `sed -n` + empty-string refusal + numeric compare — **not** movable until a typed parse carrier exists | separate §4.J.B row (high complexity) |
| **`git_fetch` as orchestration `Do`** | reuse `extdeps.git` shell helpers as **parameterized** `Run` spellings inside emit intent, not new extdeps op | gate_script prepends fetch; `git_fetch_script_for_gate` already grounds on `GitRef` parts | `merge_admission_produce.git_fetch_script_for_gate` |

**Not new types (reuse as-is):** `RedirectSpec.ToFile`, `CaptureSpec.ExitStatus`, `Predicate.IntNe`,
`PipelineStep.Exit`, `EnvSet`, `Comment` — landed #7293 with floor-tail stamp consumer.

**Orphan to delete (no new type):** `ci_merge_admission_stamp_script()` has **zero production
consumers** @ `611fd701`. Stamp lives in `ci_floor_stamp_merge_admission_script` (emit module).
Census §4.J.A row for `ci_merge_admission_stamp_script` is stale.

---

## (b) Canonical Bash emit boundary

```
gunbc.merge_admission_produce     intent/domain (receipt wire, disposition constants, gunbc run argv)
        │
        ▼
v2.workflow.ci_merge_admission_emit   foreign-executor bash binding (floor marker init, floor tail)
v2.workflow.ci_workflow_run_emit      selection-control bash binding
v2.workflow.ci_materialization_emit   (sccache only today; receipt gates NOT yet migrated)
        │
        ▼ orch_bash_emit_pipeline / emit_orchestration
v2.compiler.emit_orchestration + bash grammar rows
        │
        ▼
gunbc.ci_workflow RunStep.run  ──►  gunbc.ci_yaml_emit  ──►  .github/workflows/ci.yml
```

**Rule:** producer modules (`merge_admission_produce`, `ci_materialization`) retain **policy and
constants**; **all new emit work** extends the `v2.workflow.*_emit` modules already on the
`realization_vocabulary_containment` roster. `gunbc.ci_spec` **imports emit surfaces only** (already
true for `ci_floor_disposition_marker_init_script`, `ci_floor_stamp_merge_admission_script`) —
charter forbids editing `ci_spec` composer logic in this wave.

**Refused-poison pattern:** each emit module carries `__GUNBC_ORCH_EMIT_REFUSED__` poison so emit
refusal reds `ci.yml` drift gate (DESIGN §5).

---

## (c) Production consumers → `ci.yml` surfaces

### `ci_documentation_only_gate_skip_prefix(gate_name, skip_reason)`

| # | caller module | immediate consumer | `ci_workflow.dag` step | `ci.yml` step name |
| --- | --- | --- | --- | --- |
| 1 | `merge_admission_produce` | `ci_merge_admission_gate_script()` | `ci_merge_admission_gate_step()` | Merge-admission gate |
| 2 | `ci_materialization` | `ci_floor_receipt_gate_documentation_only_skipped_prefix` → `ci_floor_resolve_receipt_gate_script()` | `ci_floor_resolve_receipt_gate_step()` | Floor resolve receipt gate |
| 3 | `ci_materialization` | same wrapper → `ci_floor_materialization_receipt_gate_script()` | `ci_floor_materialization_receipt_gate_step()` | Floor materialization receipt gate |
| 4 | `ci_workflow_run_emit` | `ci_selection_control_skip_prefix_command` → `ci_selection_control_script()` | `ci_selection_control_step()` | Affected-set selection control |

**Emission chain:** rows 1–3 still read concat from `gunbc.merge_admission_produce` /
`gunbc.ci_materialization` directly into `ci_workflow` (not yet emit-migrated). Row 4 reads from
`ci_workflow_run_emit` (partially migrated module).

### `ci_merge_admission_gate_script()`

| caller | step | `ci.yml` |
| --- | --- | --- |
| `gunbc.ci_workflow.ci_merge_admission_gate_step` | sole consumer | `ROOT=…`, `git fetch`, skip-prefix, `gunbc run … merge_admission_gate.dag` |

### `ci_merge_admission_stamp_script()` — **orphan**

No importer. Superseded by floor step tail:

| caller | step | `ci.yml` |
| --- | --- | --- |
| `gunbc.ci_spec.gunbc_ci_floor_only_script` → `ci_merge_admission_emit.ci_floor_stamp_merge_admission_script` | `ci_floor_step` (inside floor `run:`) | `FLOOR_EXIT=$?` … `CI_FLOOR_EXIT=$FLOOR_EXIT gunbc run … merge_admission_stamp.dag` … `STAMP_EXIT` propagate |

### Raw floor-tail leaves (`ci_merge_admission_emit`)

| symbol | role | `ci.yml` locus |
| --- | --- | --- |
| `ci_floor_stamp_ambient_exit_command` | `FLOOR_EXIT=$?` before tail pipeline | floor step |
| `ci_floor_stamp_root_command` | `ROOT=$(git rev-parse …)` | floor step |
| `merge_admission_stamp_command()` | gunbc stamp argv (in `Do{Run}` + `EnvSet CI_FLOOR_EXIT`) | floor step |

### Materialization receipt gates (sibling §4.J.B)

| symbol | step | `ci.yml` |
| --- | --- | --- |
| `ci_floor_resolve_receipt_gate_script` | `ci_floor_resolve_receipt_gate_step` | resolve gate `run:` |
| `ci_floor_materialization_receipt_gate_script` | `ci_floor_materialization_receipt_gate_step` | materialization gate `run:` |

Both **prefix** with `ci_documentation_only_gate_skip_prefix` via
`ci_floor_receipt_gate_documentation_only_skipped_prefix`. Bodies remain concat (`sed`, numeric
compare) — **not** part of PR-2a/b merge-admission slice; share only the skip-prefix migration.

---

## (d) Injection REDs and refusal receipts

| guard | what it catches | relevant surface |
| --- | --- | --- |
| **`ci_spec_witness_test` dissolve-on markers** | skip-prefix / marker-init scaffolds present in all four skip sites | `witness_documentation_only_gate_skip_*`, `witness_floor_disposition_marker_*` |
| **`merge_admission_producer_witness_test`** | floor tail exports `CI_FLOOR_EXIT`, propagates `STAMP_EXIT`, stamp uses `--source-root dag` | emit golden path |
| **`realization_vocabulary_containment` lens** | bash AST vocab outside realization edge | `ci_merge_admission_emit`, `ci_workflow_run_emit` enrolled |
| **`host_language_transport_script` lens** | raw literal at `Run` positions (ShellOnHost route) | merge-admission emit uses `orch_bash_do(command: …)` — literals in emit output are grammar-owned; **new** raw concat in producer modules would bypass this |
| **`orch_emit_*_invalid` refusals** | metachar / newline injection in predicates, binders, paths | any `FileExists` / `StrEq` migration must keep emit-time alphabet walls |
| **`ci_merge_admission_emit_refused_poison`** | emit_orchestration rejection must not fall back to hand shell | drift gate goes red |
| **RED controls (existing)** | documentation-only → `Skipped` not `Success`; missing `CI_FLOOR_EXIT` → Failure | `merge_admission_producer_witness_test`, `merge_admission_witness_test` |

**New RED needed for PR-2b:** golden byte test for skip-prefix **after** emit migration (mirror
`ci_merge_admission_emit_test` pattern) — assert emitted bash matches hand golden independent of
grammar row spellings (#6467 pattern).

---

## (e) Conflict-aware minimal PR boundaries

| PR | scope | touches `ci_spec`? | touches `ci.yml`? | conflicts / ordering |
| --- | --- | --- | --- | --- |
| **PR-2a′ (cleanup)** | delete orphan `ci_merge_admission_stamp_script`; reconcile census §4.J.A row | no | no | land first — zero behavioral change |
| **PR-2b-1 (carriers)** | `Predicate.FileExists` + emit row + unit test; optional `PriorExitStatus` or document pipeline-sequencing fix for `FLOOR_EXIT` | no | no | blocks all emit migrations; coordinate with regen rustfmt-path consumer (shared FileExists trigger) |
| **PR-2b-2 (skip-prefix)** | model `DocumentationOnlySkip` intent in `ci_merge_admission_emit`; migrate 4 call sites; delete `ci_documentation_only_gate_skip_prefix` concat | no (imports unchanged) | **yes** — regen `ci.yml` | conflicts with any concurrent `ci_workflow` / `ci_materialization` / `ci_workflow_run_emit` edit |
| **PR-2b-3 (gate)** | `ci_merge_admission_gate_script` → emit intent in `ci_merge_admission_emit` | no | yes | depends on PR-2b-2 (shared skip intent) |
| **PR-2b-4 (floor-tail residue)** | replace raw `merge_admission_stamp_command` string with typed gunbc-invoke intent; ambient `FLOOR_EXIT` carrier | no | yes | can parallel 2b-2 if carriers landed; golden test update in `ci_merge_admission_emit_test` |
| **PR-3 (materialization gates)** | `ci_floor_*_receipt_gate_script` emit migration + `ReceiptFieldParse` | no | yes | **separate** — high complexity; do not fold into merge-admission PR |

**Explicit non-goals for this wave:** editing `gunbc.ci_spec` composer bodies; moving receipt `sed`
ladders; runtime-present `ci_deploy_access_emit` correction (owned `silent-gull-602`).

---

## A/B split summary

| lane | symbols | complexity | prerequisite |
| --- | --- | --- | --- |
| **A** | floor-tail raw leaves + orphan cleanup | low | PR-2b-1 optional for stamp command |
| **B** | `ci_documentation_only_gate_skip_prefix` + dependents | medium | `FileExists`, typed disposition read |
| **B′** | `ci_merge_admission_gate_script` | medium | B |
| **C** | materialization receipt gates | high | `ReceiptFieldParse` + B for shared skip |

**Dissolve trigger (census §4.J):** typed orchestration intent emitted through canonical Bash medium;
concat builders and raw `Run.command` leaves delete; `ci.yml` drift+parse stays green.
