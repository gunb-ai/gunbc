# v4 Go Leaf-Model CI Runner Worksheet — `go build` / `go test` host → `CiUpsertStep`

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`); CI Manager coordinates row registration. Post-#4149 reconciliation: R2b claim id `leaf_model_claim_go_r2b_int64_silent_overflow_truncates`.
> **Date:** 2026-06-01
> **Dispatch anchor:** `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §11.6 category C/D; #4091 ratified elastic CI / `CiUpsertStep` pattern (#4115 positive-Y replacement).
> **Sibling:** `docs/planning/v4-go-leaf-model-verification-worksheet-2026-06-01.md`

---

## Mechanical dispatch rule

> **No new `scripts/v4-leaf-model-go-*.sh` may be added to ci.yml without a modeled `CiUpsertStep` row in the same PR** (standing `project_no_new_shell` + #4115 pattern).

Interim shell bridges are allowed **only** with dissolve-on-arrival headers matching `scripts/v4-leaf-model-python-r1-verify.sh` until T-22 `run_target_verification` owns `go` invocation.

---

## §10.0-adapted worksheet

```text
Migration class:        GO-LEAFMODEL-CI-RUNNER (go build / go test boundary steps)
Representative failure:  Leaf-model Go verification exists only as ad hoc shell; CI category C
                         lists Python/Rust boundaries but no Go rows (dep graph §11.6).
Immediate local patch:   Wire go verify scripts into ci.yml without CiUpsertStep backing.
Why forbidden:           project_no_new_shell; no receipt parity; cannot skip-verify on unchanged claims.
DFS path:
  std/workflow authority:
    - src/v4/workflow/ci.dag — CiUpsertStep rows (47 on main post-#4115)
    - ci_pipeline_step_ids_shadow bijection for receipt parity
  host transport (interim):
    - scripts/v4-leaf-model-go-{r1,r2a,r2b,r3-external}-verify.sh
    - dissolves when T-22 modeled go toolchain transport lands
Deepest unsound boundary:
  Go leaf-model claims have no CI Upsert row; Go L0 cannot gate PRs touching go.dag.
Systemic fix:
  For each Go leaf-model claim_id, add CiUpsertStep<VerificationReport> row:
    inputs: FileSet(go.dag + claim .dag + lens fixture deps)
    verify: cached_verification_holds(claim)
    create: run_target_verification(go_fixture, go_toolchain{build, test?})
    resolve: latest_verification_report(go.dag, claim_phase)
  Map YAML category-C "leaf-model boundary" steps to these rows (gunbc#846 bypass retirement).
Non-goals:
  - Full v4-bootstrap-viability replacement (Class A exception per dep graph §11.6).
  - Four-compile redundancy collapse (CI Manager scope).
Falsification probe:
  Touch only a comment in go_r1.dag; affected set runs exactly Go R1 step, not full ci_v4 compile farm.
Metric allowed only as secondary:
  ci_v4 wall time — not acceptance.
```

---

## §4 Modeled step inventory (target)

| Step id (sketch) | Toolchain | Claim / scope | Interim shell |
|---|---|---|---|
| `verify_go_dag_r1` | `go build` | `leaf_model_claim_go_r1_int_surface_spelling` | `v4-leaf-model-go-r1-verify.sh` |
| `verify_go_dag_r2a` | `go build` | `leaf_model_claim_go_r2a_int_algebra_operations` | `v4-leaf-model-go-r2a-verify.sh` |
| `verify_go_dag_r2b` | `go run` or `go test` | `leaf_model_claim_go_r2b_int64_silent_overflow_truncates` | `v4-leaf-model-go-r2b-verify.sh` |
| `verify_go_dag_r3_external` | `go build` | `leaf_model_claim_go_r3_external_symbol_projection` | `v4-leaf-model-go-r3-external-verify.sh` |

**FileSet trigger:** `src/v4/extdeps/languages/go.dag`, `src/v4/test/claim/language_model/go_*.dag`, `src/v4/lens/leaf_model_verification.dag` (go_* sections).

---

## §8 Modeling DFS Arbiter approval checklist — CLOSED 2026-06-01

- [x] Claim IDs match leaf-model verification worksheet
- [x] Positive-Y `CiUpsertStep` only — no `ci.yml` shell without modeled row (operator hold)
- [x] CI Manager: `ci_pipeline_step_ids_shadow` registration in impl PR
- [x] **READY-FOR-WORKER-DISPATCH** (`proud-fox-405`)

---

## Related artifacts

- `docs/planning/v4-w2.3-ci-upsert-step-migration-worksheet-2026-05-30.md`
- `docs/planning/v4-p5-structural-bridge-replacement-worksheet-2026-05-30.md` — #4115 pattern
- `docs/planning/elastic-ci-redesign-exploration-2026-05-31.md` — #4091 substrate
