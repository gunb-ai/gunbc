# Shell→DAG PR-2 A/B model packet (consolidated)

> **Status: MODEL PACKET @ `611fd701` (eager-boar-42).** Investigation-only — no code migration.
> Authority: DESIGN shell→intent open thread, ROADMAP `2-emit-partition`, census
> [§4.J](shell-to-dag-residual-census-and-arc-completion.md#4j--bucket-d-foreign-executor-emit-punch-list--post-7216-wise-crane-222-2026-07-26).
> Rejection rule: plans that merely move shell into `Run.command` / `String` concat are not
> completion.

---

## 0. Scope

**Merge-admission cluster (§4.J.A):**

| symbol | def module | lines |
| --- | --- | --- |
| `ci_documentation_only_gate_skip_prefix` | `gunbc.merge_admission_produce` | 216–236 |
| `ci_merge_admission_gate_script` | `gunbc.merge_admission_produce` | 200–214 |
| `ci_merge_admission_stamp_script` | `gunbc.merge_admission_produce` | 193–198 (**orphan**) |
| `ci_floor_stamp_ambient_exit_command` | `v2.workflow.ci_merge_admission_emit` | data |
| `ci_floor_stamp_root_command` | `v2.workflow.ci_merge_admission_emit` | data |
| `merge_admission_stamp_command` | `gunbc.merge_admission_produce` | 152–157 |

**Materialization receipt gates (§4.J.B):**

| symbol | def module | lines |
| --- | --- | --- |
| `ci_floor_materialization_receipt_gate_script` | `gunbc.ci_materialization` | 217–244 |
| `ci_floor_resolve_receipt_gate_script` | `gunbc.ci_materialization` | 246–273 |
| `ci_floor_receipt_gate_documentation_only_skipped_prefix` | `gunbc.ci_materialization` | 210–215 (wrapper) |

**Out of scope:** `gunbc.ci_spec` composer bodies (charter); `ci_deploy_access_emit` runtime regression.

---

## (a) Domain types that must land first

### Shared across merge-admission + materialization (PR-2b carriers)

| type | home | consumers | notes |
| --- | --- | --- | --- |
| **`Predicate.FileExists { path }`** | `v2.std.orchestration.Predicate` + bash emit row | skip-prefix (`test -f disposition`), materialization missing-file arms (`test -f receipt`) | Sketched [orchestration-as-intent-design.md](orchestration-as-intent-design.md) §3; **not yet in `Predicate` coproduct** |
| **`CiFloorDisposition` read** | `gunbc.merge_admission_produce` domain → `StrEq` at emit | skip-prefix compares to `documentation_only_skipped` | Constants exist; need typed `Let` from file content, not `$(cat …)` cmdsubst |
| **`CaptureSpec.PriorExitStatus { binder }`** | `v2.std.orchestration` | `FLOOR_EXIT=$?` floor-tail opener | PR2a declined ambient capture outside Pipeline; alternative: restructure floor step so claim_executor + stamp are one Pipeline with sequenced `ExitStatus` |

### Materialization-specific (PR-3 carriers)

| type | home | replaces | notes |
| --- | --- | --- | --- |
| **`FloorResolveReceipt`** | `gunbc.ci_materialization` (domain record) | `sed -n 's/^resolves_total=//p'` | Writer: `claim_executor::write_resolve_receipt` (`resolves_total=`, `resolve_ms_total=`, …). Gate checks: file exists, field parseable, `resolves_total == ci_floor_declared_resolve_count` (currently `1`) |
| **`FloorMaterializationReceipt`** | `gunbc.ci_materialization` (domain record) | three `sed` field extractions | Writer: `claim_executor::write_materialization_receipt` (`keyed_calls=`, `unkeyed_calls=`, `duplicated_keys=`, …). Gate checks: file exists, fields non-empty, `keyed_calls > 0` |
| **`ReceiptFieldParse` step** (or `PipelineStep.ReadKeyValue`) | `v2.std.orchestration` + `gunbc.ci_materialization` projection | `k=$(sed -n 's/^keyed_calls=//p' path)` | **sed must NOT survive as a Bash token** — becomes typed key-value read intent emitted via grammar row, or a **host read effect** at runtime-present boundary. Foreign-executor path (GHA `run:`) → orchestration `Let { name, value: ExprCmdSubst{…} }` is still medium-as-string; correct shape is either (1) typed `ReadReceiptField { path, key }` predicate/`Let` with dedicated emit, or (2) move gate into `claim_executor` post-walk (runtime-present — typed effect, no bash) |
| **`Predicate.IntEq` / `IntGt`** | `v2.std.orchestration.Predicate` | `[ "$k" -eq 0 ]`, `[ "$n" -ne 1 ]` | `IntNe` exists (#7293); resolve gate needs equality-to-declared-count |

**Verdict on sed:** `sed` parsing is **not** a permanent Bash token. It is an **interim transport** for a keyed line-file format both receipts already define in Rust (`claim_executor`). The typed model is the **receipt record** at the domain layer; bash emission is a **lossy projection** for the foreign executor, dissolving when either (a) receipt gates become in-process checks on the walk result, or (b) `ReadReceiptField` lands in orchestration with a grammar row that does not expose `sed` as authorable string.

**Not new types:** `ToFile`, `ExitStatus`, `IntNe`, `Exit`, `Comment`, `EnvSet` — landed #7293.

---

## (b) Canonical Bash emit boundary

```
DOMAIN (policy + constants + receipt schemas)
  gunbc.merge_admission_produce
  gunbc.ci_materialization          ← receipt paths, declared counts, gate policy notes

EMIT (foreign-executor bash binding)
  v2.workflow.ci_merge_admission_emit     floor marker init, floor tail (PARTIAL #7293)
  v2.workflow.ci_workflow_run_emit        selection control (skip_prefix caller)
  v2.workflow.ci_materialization_emit     sccache only today; receipt gates NOT migrated

EMIT ENGINE
  v2.compiler.emit_orchestration + bash grammar rows

WORKFLOW PROJECTION
  gunbc.ci_workflow  →  gunbc.ci_yaml_emit.expected_ci_yml()  →  .github/workflows/ci.yml
```

**Today:** receipt gate scripts and merge-admission gate script are still **concat in domain modules** (`ci_materialization.dag`, `merge_admission_produce.dag`) wired directly into `ci_workflow.dag` — not through `*_emit` modules. Only `ci_sccache_provider_shell_injection` and floor-tail/marker-init migrated (#7265/#7293).

**Refused-poison:** each `*_emit` module carries `__GUNBC_ORCH_EMIT_REFUSED__` so emit refusal reds drift gate.

---

## (c) Production consumers → `ci.yml` surfaces

### `ci_documentation_only_gate_skip_prefix` — 4 GHA sites

| # | caller | wrapper | `ci_workflow` step | `ci.yml` step |
| --- | --- | --- | --- | --- |
| 1 | `merge_admission_produce` | inside `ci_merge_admission_gate_script` | `ci_merge_admission_gate_step` | Merge-admission gate |
| 2 | `ci_materialization` | `ci_floor_receipt_gate_documentation_only_skipped_prefix` → resolve script | `ci_floor_resolve_receipt_gate_step` | Floor resolve receipt gate |
| 3 | `ci_materialization` | same → materialization script | `ci_floor_materialization_receipt_gate_step` | Floor materialization receipt gate |
| 4 | `ci_workflow_run_emit` | `ci_selection_control_skip_prefix_command` | `ci_selection_control_step` | Affected-set selection control |

**Exact skip-prefix bash shape** (all four sites, parameterized `gate_name` / `skip_reason`):

```bash
# 🟡 dissolve-on: ci_documentation_only_gate_skip_prefix …
if test -f target/ci-floor-disposition.txt; then _disp=$(cat target/ci-floor-disposition.txt)
if test "$_disp" = "documentation_only_skipped"; then echo "<gate_name>: skipped — CI_FLOOR_DISPOSITION=$_disp (<skip_reason>)"; exit 0; fi; fi
```

### `ci_merge_admission_gate_script` — 1 GHA site

**Definition** (`merge_admission_produce.dag:200`):

```
ROOT=$(git rev-parse …)
git fetch --no-tags <remote> <ref>
<skip_prefix gate_name="merge-admission gate" skip_reason="floor not run; merge-admission stamp already recorded Skipped disposition">
gunbc run … merge_admission_gate.dag --function main
```

| consumer | `ci.yml` |
| --- | --- |
| `ci_merge_admission_gate_step` | step "Merge-admission gate (receipt required; …)" lines ~261–268 |

### `ci_merge_admission_stamp_script` — orphan reconciliation

| fact | detail |
| --- | --- |
| **Importers** | **none** — zero `import` / call sites outside definition |
| **Live stamp path** | `ci_spec.gunbc_ci_floor_only_script` → `ci_merge_admission_emit.ci_floor_stamp_merge_admission_script` → floor step `run:` |
| **Census §4.J.A** | lists `ci_merge_admission_stamp_script` as open — **stale** |

**Reconciliation protocol (deletion ≠ migration):**

1. **Prove zero consumers** — grep/import closure (done @ `611fd701`).
2. **Delete** `fn ci_merge_admission_stamp_script` from `merge_admission_produce.dag` — no emit migration, no `ci.yml` change (script never reached workflow).
3. **Reconcile census** — remove §4.J.A row; add footnote under `ci_floor_stamp_merge_admission_script` / raw leaves.
4. **Witness** — no new golden; optional RED: `merge_admission_producer_witness_test` must not reference deleted symbol.
5. **Do not** route deletion through `*_emit` — there is nothing to emit.

### Raw floor-tail leaves — 1 GHA site (floor step)

| symbol | emitted role | typed today? |
| --- | --- | --- |
| `ci_floor_stamp_ambient_exit_command` | `FLOOR_EXIT=$?` | no — opaque `orch_bash_do` |
| `ci_floor_stamp_root_command` | `ROOT=$(git rev-parse …)` | no — opaque `orch_bash_do` |
| `merge_admission_stamp_command()` | gunbc stamp argv in `Do{Run}` + `EnvSet CI_FLOOR_EXIT` | partial — `Do`/`EnvSet` typed; argv string concat not |

### `ci_floor_resolve_receipt_gate_script` — 1 GHA site

**Definition** (`ci_materialization.dag:246`): skip_prefix + body:

| clause | bash | typed intent target |
| --- | --- | --- |
| missing file | `if ! test -f target/floor-resolve-receipt.txt; then echo "…missing…"; exit 1` | `Not(FileExists)` → `Exit 1` |
| read field | `n=$(sed -n 's/^resolves_total=//p' target/floor-resolve-receipt.txt)` | `Let` + `ReadReceiptField{key: resolves_total}` |
| malformed | `if test -z "$n"; then … exit 1` | `StrEmpty` → `Exit 1` |
| count wall | `if test "$n" -ne 1; then … exit 1` | `IntNe`/`IntEq` vs `ci_floor_declared_resolve_count` |
| ok | `echo "floor resolve count $n matches declared 1 …"` | `Comment` or diagnostic `Run` |

| consumer | `ci.yml` |
| --- | --- |
| `ci_floor_resolve_receipt_gate_step` | "Floor resolve receipt gate (declared cold-resolve count)" ~227–236 |

**Receipt producer (not bash):** `claim_executor` writes `target/floor-resolve-receipt.txt` at floor walk end.

### `ci_floor_materialization_receipt_gate_script` — 1 GHA site

**Definition** (`ci_materialization.dag:217`): skip_prefix + body:

| clause | bash | typed intent target |
| --- | --- | --- |
| missing file | `test -f target/floor-materialization-receipt.txt` | `FileExists` |
| read k,u,d | three `sed -n 's/^<field>=//p'` | three `ReadReceiptField` |
| malformed | empty-string tests on k,u,d | `StrEmpty` × 3 |
| keyed nonzero | `[ "$k" -eq 0 ]` → exit 1 | `IntEq` / zero-refusal |
| ok | disclosure echo line | diagnostic (counts not pinned per `ci_floor_materialization_receipt_note`) |

| consumer | `ci.yml` |
| --- | --- |
| `ci_floor_materialization_receipt_gate_step` | "Floor materialization receipt gate (demand ledger; …)" ~238–251 |

**Receipt producer:** `claim_executor::write_materialization_receipt`.

### `ci.yml` job step order (ci job, materialization + merge-admission context)

From `ci_workflow.dag:584`:

```
… floor_step → floor_peak_post → resolve_receipt_gate → materialization_receipt_gate
  → selection_control → merge_admission_gate
```

---

## (d) Injection REDs and refusal receipts

| guard | surface | what it catches |
| --- | --- | --- |
| **`ci_spec_witness_test`** | `witness_documentation_only_gate_skip_emits_dissolve_on_in_runners` | dissolve-on marker in all 4 skip-prefix scripts |
| **`merge_admission_producer_witness_test`** | floor-tail golden properties | `CI_FLOOR_EXIT` export, `STAMP_EXIT` propagate, `--source-root dag` |
| **`ci_materialization_witness_test`** | `ci_materialization_ladder_holds_live` | ladder over **live** `ci_workflow` (behavioral; not receipt-gate bytes) |
| **`realization_vocabulary_containment`** | enrolled `ci_merge_admission_emit`, `ci_workflow_run_emit`, `ci_materialization_emit` | bash AST vocab outside realization edge |
| **`generated_artifact_drift_gate`** | `CiYamlArtifact` → `.github/workflows/ci.yml` | committed `ci.yml` == `expected_ci_yml()` byte-for-byte |
| **`ci_yaml_validate.ci_yml_parses`** | yaml ingest on committed path | parse gate (sibling to drift) |
| **`orch_emit_*_invalid`** | emit-time | metachar in predicates, binders, paths |
| **`*_emit_refused_poison`** | each emit module | emit_orchestration refusal → invalid shell → drift red |
| **`ci_floor_declared_resolve_count` wall** | resolve gate | count change without conscious `.dag` update — **semantic RED** on enrollment |

**New REDs needed post-migration:**

- Golden byte tests for receipt gates + gate script ( `#6467` pattern in `*_emit_test.dag`).
- Perturbation RED: declared resolve count mismatch must red (already live via gate execution).
- Orphan deletion: compile-time — removed symbol must not appear in import graph.

---

## (e) Conflict-aware PR boundaries (A / B / B′ / materialization / cleanup)

| PR | lane | symbols | touches `ci_spec`? | touches `ci.yml`? | depends on | conflicts with |
| --- | --- | --- | --- | --- | --- | --- |
| **0 — orphan cleanup** | hygiene | delete `ci_merge_admission_stamp_script`; census reconcile | no | **no** | — | none |
| **1 — shared carriers** | prerequisite | `FileExists`, `ReadReceiptField` (or receipt records), `IntEq`; optional `PriorExitStatus` | no | no | — | regen rustfmt-path (#7290) shares `FileExists` trigger |
| **2a — floor-tail residue** | A | ambient `FLOOR_EXIT`, `merge_admission_stamp_command` string → typed `Do` | no | yes | PR-1 optional | any `ci_merge_admission_emit` editor |
| **2b — skip-prefix** | B | `ci_documentation_only_gate_skip_prefix` → emit; 4 call sites | no | yes | PR-1 `FileExists` + disposition read | `ci_materialization`, `ci_workflow_run_emit`, `merge_admission_produce` |
| **2c — merge gate** | B′ | `ci_merge_admission_gate_script` → `ci_merge_admission_emit` | no | yes | PR-2b (shared skip intent) | `ci_workflow` |
| **3a — materialization emit shell** | mat-1 | migrate receipt gate concat → `ci_materialization_emit` Pipeline (still sed leaves until PR-3b) | no | yes | PR-2b skip-prefix wrapper can land first | `ci_materialization.dag` |
| **3b — receipt parse model** | mat-2 | `Floor*Receipt` records + `ReadReceiptField`; delete sed concat | no | yes | PR-1 `ReadReceiptField` | `claim_executor` receipt format (coordinate if fields change) |
| **3c — in-process gate (optional)** | mat-3 | move receipt checks from GHA bash to `claim_executor` post-walk refusal | no | yes (delete steps or no-op) | PR-3b | floor plan / gate enrollment |

**Sequencing rule:** PR-0 anytime. PR-1 before any emit migration. PR-2b before PR-2c. PR-3a can parallel PR-2c after PR-2b. PR-3b is independent of merge-admission except shared PR-1 carriers.

**Explicit non-goals:** `ci_spec` composer edits; wrapping sed in `Run.command` without `ReadReceiptField`; treating orphan deletion as emit migration.

---

## Dissolve triggers (census §4.J)

| row | trigger |
| --- | --- |
| Merge-admission foreign executor | typed orchestration intent → canonical Bash emit → delete concat builders + raw floor-tail `Run.command` leaves |
| CI materialization foreign executor | receipt parsing as typed predicates/read effects, not `sed`/test strings; emit through `ci_materialization_emit` |
| Orphan `ci_merge_admission_stamp_script` | delete + census reconcile (not a migration) |

**Byte oracle:** any PR touching `ci_workflow.dag` step `run:` fields must regen `ci.yml` and keep `generated_artifact_drift_gate` + `ci_yml_parses` green.
