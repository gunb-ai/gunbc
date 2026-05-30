# Compiler Spine — CiSelectionReceipt shadow mode (Phase 2.0)

**Authority:** PR [#3938](https://github.com/gunb-ai/gunbc/pull/3938) §11.4 dispatch step 2; PR [#3959](https://github.com/gunb-ai/gunbc/pull/3959) §5 (`CiSelectionReceipt`); Modeling DFS Step 1 approval (`proud-pike-680`, 2026-05-30).
**Owner:** Compiler Spine (`smart-stag-871`).
**Consumers:** operator review, Close/Receipt (structural gate), Phase 2.5 activation (gate workflow on `selected`).
**Out of scope:** `CiUpsertStep<T>` implementation (blocked on Phase 1.4 `Upsert<T>` substrate); `CiRunPolicy` / `dependency_set` naming; active step skipping in GitHub Actions (shadow only).

---

## 1. Purpose

Produce a **per-PR `CiSelectionReceipt`** that records, for every modeled CI step, a **justified** `Run | Skip | CarvedOut` decision from structural facts (`ChangeSet`, `AffectedSet`, carve-out list). In **shadow mode** the existing workflow **still executes all steps**; the receipt is emitted first (or alongside) for human and manager review. Transition to **active** minimal CI (Phase 2.5 per `docs/planning/v4-ci-overhaul-2026-05-30.md`) gates on **`receipt.selected` = every `CiStepSelection` whose `decision` is `Run | CarvedOut`** (executable projection). `skipped` holds `Skip` only; `carved_out` is the carveout-config snapshot (`List<CiCarveout>`), not a substitute for the runnable set.

**Structural success criterion** (operator 2026-05-30): correct iff every step's `decision` is justified by (a) non-empty `inputs ∩ affected_set` evidence (`Run`), (b) empty intersection plus `cache_digest` projection `Accepted { value: … }` (`Skip`), or (c) explicit `ci_always_run_carveouts` match (`CarvedOut`). Wall-clock reduction is downstream, not the gate.

---

## 2. Approved substrate (Modeling DFS — do not re-litigate)

| Symbol | Shape | Notes |
| ------ | ----- | ----- |
| `CiUpsertStep<T>` | `Upsert<T> { inputs, verify, create, resolve }` | **Not implemented** until Phase 1.4 lands usable `Upsert<T>` |
| `UpsertInputRef` | `FileSet \| SubstrateNodeSet \| LensOutputRef \| TestClaimRef \| UpstreamUpsert` | **No `Always` variant** |
| `ci_always_run_carveouts` | `List<CiCarveout>` | `reason_code`, `reason_detail`, `dissolution_target` |
| `CiSelectionReceipt` | See §3 | First observable artifact |
| Forbidden | `List<Symbol>` inputs; `CiRunPolicy`/`CiRunMode`; `DependencySource`/`dependency_set` | P1 closed-system / P2 single-authority |

---

## 3. Receipt carriers (canonical)

Aligned with `docs/planning/v4-ci-overhaul-2026-05-30.md` §5. `Outcome<ContentHash>` is `std.diagnostic.Outcome` (Practice 10 — no parallel projection carrier).

```dag
type CiSelectionReceipt {
  pr: ChangeSet
  affected: AffectedSet
  selected: List<CiStepSelection>
  skipped: List<CiStepSelection>
  carved_out: List<CiCarveout>              // snapshot of matching carveouts for this receipt
}

type CiStepSelection {
  step_id: CiStepId
  inputs_consulted: List<UpsertInputRef>    // shadow: bridge rows until CiUpsertStep lands
  affected_intersection: List<AffectedNode>
  decision: SelectionDecision
  cache_digest: Outcome<ContentHash>      // Accepted = projected hash; Rejected = diagnostics — no sentinel (P3)
  reason: Symbol
}

type SelectionDecision
  = Run
  | Skip
  | CarvedOut { carveout_reason: Symbol }
```

`CiStepId` — typed step identity (Phase 2.0: alias `Symbol` job/gate id from `ci_pipeline` with explicit registry; Phase 1.5: dedicated carrier when `CiUpsertStep` rows land).

---

## 4. Shadow mode semantics

| Mode | Workflow behavior | Receipt |
| ---- | ----------------- | ------- |
| **Shadow** (this deliverable) | GitHub Actions / hand `ci.yml` runs **unchanged** (all jobs that today's `if:` allows) | Compute + emit `CiSelectionReceipt` (structured log / artifact) |
| **Active** (Phase 2.5) | Gate each step on `decision == Run ∨ CarvedOut` | Same receipt; workflow trusts it |

**Fail-closed on receipt construction:** if `ChangeSet` read is `Rejected` or `AffectedSet` is fail-closed, every step is `Run` in the receipt **and** `reason` documents superset selection (mirrors `ci_component_affected_fail_closed` today). Receipt must not claim `Skip` unless intersection is empty **and** the projected `cache_digest` is valid (§5); invalid projection → `Run` with `cache_digest_projection_fail_closed`.

**Cache projection carrier (Phase 2.1).** `project_cache_digest(row, pipeline) -> Outcome<ContentHash>` projects the **complete step subgraph** (interim: `ci_job_cache_digest` / `ci_gate_cache_digest` on the modeled `CiJob`/`CiGate` projection node — B1 / T-21 discipline, not inputs-only). `CiStepSelection.cache_digest` stores that **full `Outcome`** — never a fabricated hash when projection fails. **Valid for `Skip`** iff `cache_digest` is `Accepted { value: digest }`; `Rejected { diagnostics }` forces `Run` (fail-closed). Close/Receipt consumers must not treat `Rejected` as a hash-shaped fact.

---

## 5. Selection algorithm (spine contract)

Aligned with §1 and `docs/planning/v4-ci-overhaul-2026-05-30.md` §5 structural gate — `Skip` only when intersection is empty **and** cache projection is valid; otherwise fail-closed to `Run`.

```text
ci_selection_receipt_shadow(
  pr: ChangeSet,
  pipeline: CiPipeline,
  carveouts: List<CiCarveout>,
  registry: List<CiShadowStepRow>,
) -> CiSelectionReceipt

  affected := affected_set_from_diff(pr)   // v4.lens.affected_set authority
  if change_set_fail_closed(pr) || affected_set_fail_closed(affected):
    return superset_run_receipt(pr, affected, registry, reason: receipt_inputs_fail_closed)

  for each row in registry:
    projected := project_cache_digest(row, pipeline)   // Outcome<ContentHash>

    if row.step_id ∈ carveouts.step_ids:
      decision := CarvedOut { carveout_reason := matching.reason_code }
      reason := carveout_matched
    else if intersect(row.inputs, affected) ≠ ∅:
      decision := Run
      reason := affected_intersection_nonempty
    else match projected {
      Accepted { value: digest, diagnostics: _ } =>
        decision := Skip
        reason := affected_intersection_empty_cache_valid
      Rejected { diagnostics: _ } =>
        decision := Run                                // P3 fail-closed — cannot skip safely
        reason := cache_digest_projection_fail_closed
    }

    append CiStepSelection { ..., decision, cache_digest := projected, reason }
    partition per decision:
      Run | CarvedOut  → selected    // Phase 2.5 runnable set (workflow executes these)
      Skip             → skipped
    (carved_out on receipt = snapshot of matching ci_always_run_carveouts rows, not step partition)
```

**Registry** (`CiShadowStepRow`): spine-owned bridge table mapping each `ci_pipeline` job/gate to provisional `List<UpsertInputRef>` until Phase 1.5. Rows are **data**, reviewable, versioned with `ci.dag`.

---

## 6. Phase 2.0 shadow registry (interim bridge)

Maps current `ci_pipeline` jobs (`src/v4/workflow/ci.dag`) to **typed** `FileSet` selectors (not bare path strings). Component flags (`CiComponentAffected`) remain a **coarse GitHub `if:` bridge** only — not authoritative for per-step receipt; receipt uses node-level `AffectedSet` from T-21 lens.

| `CiStepId` (job id) | Provisional `inputs_consulted` | Shadow notes |
| ------------------- | ------------------------------ | ------------ |
| `v2_compile_src_v4` | `FileSet { src/v2/**, Cargo.toml, Cargo.lock }` + `SubstrateNodeSet` over v4 compiler bootstrap nodes | Carveout candidate: v2 circular-dep (see §7) |
| `lens_ci_registry_execution` | `FileSet { src/v4/**, dsl/std/** }` + `LensOutputRef` for declared lenses | |
| `m1_rust_emit_probe_execution` | `FileSet { src/v4/**, scripts/v4-m1*, Cargo.* }` | |
| `testclaim_corpus_eval_execution` | `TestClaimRef` roster + `SubstrateNodeSet` rerun frontier from `ci_select_from_affected_set` | Selection fn must stay `ci_select_from_affected_set` |

Gates with interim `CiGateRunPolicy::Always` (`lens_ci_registry_signal`, `m1_rust_emit_probe_signal`) appear in receipt as **CarvedOut** once listed in `ci_always_run_carveouts`, not as a permanent policy enum.

---

## 7. Initial `ci_always_run_carveouts` (shadow seed)

Honest seed list for operator review (expand only with `reason_code` + `reason_detail` + `dissolution_target`):

```dag
// data ci_always_run_carveouts — lands in ci.dag with receipt types (Phase 2.1)
{ step_id: v2_compile_src_v4,
  reason_code: v2_substrate_circular_dep,
  reason_detail: "v2 compile gate; affected-set lens imports v2 substrate — circular dependency unmodeled",
  dissolution_target: ModelMissingSubstrate { what: v2_substrate_dependency_modeled_in_affected_set }
}
{ step_id: lens_ci_registry_signal,
  reason_code: integrity_always_run_interim,
  reason_detail: "interim Always gate policy until Phase 2.5 activates receipt-driven gating",
  dissolution_target: ModelMissingSubstrate { what: ci_gate_run_policy_dissolved_to_receipt }
}
{ step_id: m1_rust_emit_probe_signal,
  reason_code: integrity_always_run_interim,
  reason_detail: "interim Always gate policy; non-blocking probe",
  dissolution_target: ModelMissingSubstrate { what: ci_gate_run_policy_dissolved_to_receipt }
}
```

---

## 8. Implementation phases (spine-owned)

| Phase | Deliverable | Blocked on |
| ----- | ----------- | ---------- |
| **2.0** (this doc) | Contract + registry + carveout seed | Modeling DFS Step 1 ✓ |
| **2.1** | `CiSelectionReceipt` types + `ci_selection_receipt_shadow` in `ci.dag`; unit claims on fixture `ChangeSet` | — |
| **2.2** | Host transport: CI job writes receipt JSON (shadow artifact) before existing steps | 2.1 |
| **2.5** | Active gating on `selected` | Operator sign-off on shadow stability |

**Worker brief guard:** no worker may add `CiUpsertStep` bodies or `dependency_set` fields; shadow work consumes this doc + `v4-ci-overhaul-2026-05-30.md` only.

---

## 9. Cross-lane

| Lane | Action |
| ---- | ------ |
| Modeling DFS | Phase 1.4 `Upsert<T>` substrate; then Phase 1.5 worksheet |
| Close/Receipt | Consume receipt for structural close predicates (not wall-clock) |
| Self-host/Release | Phase 2 YAML emit continues; shadow receipt does not change `ci.yml` yet |

---

## 10. Related artifacts

- `docs/planning/v4-ci-overhaul-2026-05-30.md` — operator-ratified target + sequencing
- `src/v4/workflow/ci.dag` — `ci_pipeline`, `ci_select_from_affected_set`, component affected bridge
- `src/v4/lens/affected_set.dag` — `AffectedSet` authority
- `src/v4/std/change.dag` — `ChangeSet`, `AffectedNode`
