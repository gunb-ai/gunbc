# v4 UPSERT stray catalog — 2026-05-30

**Node:** `adhoc-74470445-f54`

**Parent workstream:** `adhoc-342cfea2-653` workstream 2 kickoff.

**Scope:** `src/v4/compiler`, `src/v4/std`, and `src/v4/lens`.

**Purpose:** scan the v4 compiler/std/lens surface for places where the
UPSERT canon from `docs/audit/upsert-pattern-compiler-stray-2026-05-29.md`
is either implemented, blocked, or at risk of drifting into blind create /
blind overwrite semantics. This is a point-in-time catalog, not a maintained
ledger. Inline `🟡` marks remain authoritative; this file only cross-references
them against the v4 deferral audit.

**UPSERT canon:** verify-first, satisfy dependencies recursively,
create-if-missing, cache-outcome.

**Dissolution trigger:** delete this file when the rows in §2 are either
dissolved by their owning tasks or explicitly dismissed in review. Future
changes should update the inline marks, not this catalog.

---

## §0. Reproducibility

Run from repo root:

```bash
rg -n "Upsert|upsert|UPSERT|ensure|create|write|overwrite|cache|memo|idempotent|EffectClassification|signature-deferred|deferred" \
  src/v4/compiler src/v4/std src/v4/lens -S
```

Deferral cross-check:

```bash
rg -n "feature:T22-EVAL-CACHE-HASHES|EffectClassification|signature-deferred|T-13-effect-signature-deferred|T-13-idempotency|config-patch-record-projection|UpsertEffect|idempotent-operation" \
  src/v4/compiler src/v4/std src/v4/lens docs/audit/v4-deferral-audit-2026-05-29.md -S
```

There are no literal `STRAY-FROM-UPSERT` inline markers in the scoped source
tree at this snapshot. The table below is therefore authored from the scan,
not harvested from existing marker text.

---

## §1. Conforming upsert-shaped surfaces

| Location | Why it conforms | Notes |
| --- | --- | --- |
| `src/v4/std/effects.dag` `UpsertEffect` | Models upsert as an idempotent effect arm under `EffectShape = IsIdempotent(...)`. | Aligns with `INVARIANTS.md` P1 worked example: upsert is lattice-meet-shaped idempotency. |
| `src/v4/lens/testgen.dag` idempotent-operation generator | Schedules `sample_upsert_composable` and emits an `f(f(x)) == f(x)` TestClaim. | Conforming as a structural receipt generator for `std.effects`; not a runtime filesystem upsert. |
| `src/v4/std/patch.dag` `FieldPatch<T>` + `config_patch_layer` | Per-field overlay verifies presence via `Override` vs `Inherit`, then converges to the intended config value. | `ConfigPatchRecord` projection is now green in this file; formatter consumers outside this scan may still carry older consumer marks. |
| `src/v4/compiler/05_eval.dag` `TestClaimCacheKey` / `TestClaimCacheReceipt` | Has a cache-outcome boundary: returns a stable receipt for a claim + evaluation subject. | Conforming at the boundary, but the digest substrate remains deferred; see row U2. |

---

## §2. STRAY-FROM-UPSERT table

| ID | Location | STRAY-FROM-UPSERT finding | Deferral-audit cross-ref | Classification | Dissolve-on |
| --- | --- | --- | --- | --- | --- |
| **U1** | `src/v4/lens/effect.dag` `EffectClassification {}` and `effect_signature_deferred_unresolved` | Upsert/idempotency effect facts cannot yet be verified from the operation signature. The lens writes an empty marker row and fails closed whenever dependencies exist, so it avoids fabrication but does not complete the verify-first phase for effect kind. | `v4-deferral-audit` §3.1 `TASKS.md` line 727, `EffectClassification` B3 signature-deferred; §3.3 Open Q10 for partiality/effects in `ModelCore`. | **NECESSARY.** Signature-derived effect-kind closure is upstream work, not a local blind-create bug. | T-22/T-23 and the #3468 follow-up close `InferredFacts` → closed effect-kind decoding; then replace the empty marker with the signature-derived carrier. |
| **U2** | `src/v4/compiler/05_eval.dag` cache digest helpers and `InterpretationCacheDigestAuthority` | Cache-outcome exists, but cache identity is built from hand-rolled digest folds and registered-slot sentinels. This is upsert-shaped only at the receipt API; the stable identity substrate is deferred. | `v4-deferral-audit` §3.1 `TASKS.md` lines 741/1522 and §3.2 `v4_evaluator` Wave 2; inline `feature:T22-EVAL-CACHE-HASHES`. | **NECESSARY.** The current code is fail-closed and tagged; replacing it requires the T-22 content-hash/evaluator substrate. | IRT-4 / TASKS T-22: `content_hash(TestClaim eval_subject Node)` and interpreter function-body hashes replace per-slot digest folds. |
| **U3** | `src/v4/lens/idempotency.dag` `RequiresAlgebraWitness` default | The idempotency lens does not infer positive upsert idempotency from dependency kind or from `std.effects.UpsertEffect`; it requires an algebra witness and reports unresolved otherwise. This is intentionally conservative, but it means upsert classification is not yet recursively satisfied from effect facts. | `v4-deferral-audit` §1.9 long-tail / T-13 family gates; related to §3.1 B3 signature-deferred effect semantics. | **NECESSARY.** The fail-closed default is correct until algebra/effect facts are available through `InferredFacts`. | T-13/T-22 effect signature closure plus algebra witness projection: feed `UpsertEffect` law witnesses into `AlgebraicIdempotenceProven`. |
| **U4** | `src/v4/compiler/00_compile.dag` local `CompileLens` gate adapter | The compile pipeline verifies required lenses before translate/eval, but the lens application surface is locally re-authored. That makes the verify-first phase upsert-shaped, while the dependency-satisfaction phase depends on a duplicated adapter. | `v4-deferral-audit` long-tail `feature:T-23-lens-application-migration`; inline comments in `00_compile.dag`. | **NECESSARY but at-risk.** It is tagged and bounded, but keeping the local adapter after T-23 would become parallel authority. | T-23: route compile gates through `v4.lens.application` and delete `CompileLens` / `run_compile_lens_gate`. |
| **U5** | `src/v4/std/dependency.dag` `classify_named_edge_usage` | Dependency usage classification uses edge-name sentinels until resolve stamps ground facts. This affects recursive prerequisite satisfaction for upsert-shaped analyses because dependency edges are inferred from labels instead of declared facts. | `v4-deferral-audit` long-tail `feature:dependency-usage-classifier-consumes-resolve-ground-facts`. | **NECESSARY.** T-9 ground facts are not present; the current classifier is explicitly tagged. | T-9 resolve stamps `dependency_binds_to_edge` / module / bootstrap labels as substrate facts and consumers stop reading label convention. |
| **U6** | `src/v4/std/patch.dag` direct `config_patch_layer` body | The function body returns `base`; real per-field upsert semantics are lowerer-expanded. Direct execution is rejected by a fallback diagnostic, so it is not a silent overwrite, but the source-level body is not the authority. | `v4-deferral-audit` §1.2 / §A7 `config-patch-record-projection`; note: this file now marks the projection green. | **AT-RISK, currently bounded.** It is acceptable only while direct execution remains fail-closed and expansion is the declared consumer. | Keep `config_patch_layer` as syntax-only until v4 can represent generic record-field projection bodies directly; direct execution must continue to reject. |

---

## §3. Non-strays checked

| Location | Reason not classified as stray |
| --- | --- |
| `src/v4/std/effects.dag` `CreateEffect` / `AppendEffect` | These are deliberately non-idempotent/breaking effect arms, not failed upserts. |
| `src/v4/std/verdict.dag` `Deferred` tally | This is verification-result accounting, not a task deferral or upsert write path. |
| `src/v4/std/diagnostic.dag` `append_outcome_value` and diagnostics append helpers | They compose diagnostics through the monoid surface; append semantics here are not a blind create/write operation. |
| `src/v4/lens/affected_set.dag` pending diagnostics folds | Existing `🟡 needs-more-work` marks are already covered by the deferral audit §2 as T-21 necessary scaffolds; no distinct upsert violation found. |

---

## §4. Summary

The scoped v4 source has **no untagged high-severity UPSERT stray** equivalent
to the DSL `content_upsert` stub called out in
`upsert-pattern-compiler-stray-2026-05-29.md`.

The meaningful risks are bounded and already yellow-tagged:

1. Effect/idempotency verification is fail-closed until signature-derived
   effect facts and algebra witnesses land.
2. Eval cache receipts are upsert-shaped at the API boundary, but stable
   cache identity is waiting on T-22 content-hash substrate.
3. Compile lens application and dependency classification are local bridges
   that must dissolve when their named substrate consumers land.

No implementation change is recommended from this catalog alone; the next
workstream should target the owning deferral rows rather than add another
parallel upsert abstraction.
