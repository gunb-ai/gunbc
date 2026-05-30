# v4 CI Schema Worksheet — Phase 1.5 (`CiUpsertStep<T>` + `UpsertInputRef`)

> **Status:** WORKSHEET DRAFT — Modeling DFS Manager approval required before any substrate worker dispatch.
> **Date:** 2026-05-30
> **Author:** smart-seal-842 (worker under proud-pike-680)
> **Dispatch anchor:** PR #3959 Phase 1 dispatch — **step 1** (author schema worksheet) + **step 3a** (typed carrier catalog / coproduct inhabitants)
> **Authority doc:** `docs/planning/v4-ci-overhaul-2026-05-30.md` §5–§6 (operator pre-dispatch edits 2026-05-30 on `pr3959`)
> **Prerequisite:** Phase **1.4** lands generic `Upsert<T>` in `dsl/std/patterns.dag` (parser/substrate worksheet separate). Phase **1.5** implementation workers remain **BLOCKED** until 1.4 completes; this worksheet may be approved in parallel.

---

## Mechanical dispatch rule

> **No CI Phase 1.5 substrate worker may be dispatched until this worksheet is complete and Modeling DFS Manager–approved.**

Same discipline as PR #3938 §10.0: the worksheet is reviewable authority; the worker brief is downstream. Acceptance is **structural schema correctness**, not YAML line-count reduction or wall-clock CI wins.

---

## §10.0-adapted worksheet

```text
Schema class:           CI-PHASE-1.5 (CiUpsertStep<T> + UpsertInputRef + receipt/carveout carriers)
Representative failure:  B3 in v4-ci-overhaul §4 — ci.dag has CiJob/CiCommand but no per-step typed
                         input declaration; ci.yml bucket-coarse `if:` cannot gate minimal CI.
Immediate local patch:   Add `dependency_set: List<Symbol>` (or `FileGlob { glob: Symbol }`) on CiJob;
                         add `cache_key: Hash` payload field; add `CiRunPolicy = Always | AffectedOnly`
                         on UpsertInputRef or CiGate; keep CiGateRunPolicy::Always as step policy.
Why forbidden:           Parallel string-keyed authority (P2); parallel cache payload vs B1 content_hash
                         (Practice 10 row 5 + Practice 11 parallel-payload); heuristic run-mode enum
                         forbidden per operator 2026-05-30 (recoverable to missing structural facts — P1).
DFS path:
  std/ authority:
    - dsl/std/patterns.dag — UPSERT<T> canon (verify → deps → create → cache); Phase 1.4 lands type
    - dsl/std/types.dag — GlobPattern, FilePath, ContentHash (file-set selector spine)
    - dsl/std/effects.dag — UpsertEffect (lattice meet) witnesses keyed convergent writes
  v4 workflow authority:
    - src/v4/workflow/ci.dag — CiPipeline, cache digests via content_hash projections (T-22 pattern)
  v4 lens / change authority:
    - src/v4/lens/edit_locus.dag — RepoPath, GitDiffNameOnly (path ingress)
    - src/v4/lens/affected_set.dag — AffectedSet, rerun frontier (Layer C verify-first input)
    - src/v4/std/change.dag — ChangeSet, AffectedSet coproduct
  v4 eval authority:
    - src/v4/compiler/eval + std/verification — TestClaim, test_claim_interpretation_cache_digest
  extdeps:               (none — CI schema is internal closed-system modeling)
  compiler stage:         workflow emission / T-22 interpreter consumers (not 04_infer/05_emit)
  scaffold notes:
    - ci.dag: CiGateRunPolicy::Always is 🟡 interim GHA projection — NOT reused for UpsertInputRef
    - patterns.dag: upsert<> pattern bodies commented (ROADMAP parser generics) — Phase 1.4 gate
Deepest unsound boundary:
  Missing typed per-step Upsert specialization + UpsertInputRef coproduct as single authority for
  "what facts does verify-first read?" — today only coarse component flags (CiComponentAffected).
Systemic fix:
  Land §1 schema in src/v4/workflow/ci.dag (after Phase 1.4 Upsert<T> primitive); derive cache keys
  only via content_hash(complete step subgraph); route always-run via ci_always_run_carveouts data,
  not input-ref policy variants; emit CiSelectionReceipt before trusting active skip.
Non-goals:
  - Hand-editing ci.yml as authority; bucket `if:` heuristics; dependency_set / DependencySource vocab
  - cache_key payload on CiUpsertStep; inputs: List<Symbol>; UpsertInputRef::Always
  - Dispatching step migration before Upsert<T> usable (Phase 1.4)
Falsification probe:
  (1) Change verify/create/resolve on a step with fixed inputs — cache_digest MUST change
      (content_hash sensitivity; mirrors test_claim cache receipt tests).
  (2) Two steps with identical inputs but different verify — MUST NOT share cache entry.
  (3) Grep worker diff for forbidden patterns (§4 register) — expected zero.
Metric allowed only as secondary:
  Skipped step count / wall-clock — evidence after receipt correctness, not acceptance.
```

---

## §1 Authoritative schema catalog (step 3a — typed carriers)

Phase 1.4 lands the generic pattern; Phase 1.5 specializes in `v4.workflow.ci`. **Canonical spelling** below matches operator ratification on PR #3959 (`FileSet`, not `FileGlob`; `CiStepId`, not `Symbol` step ids).

### 1.1 Generic prerequisite (Phase 1.4 — not re-defined here)

```dag
// dsl/std/patterns.dag — Phase 1.4 delivers usable:
// Upsert<T> { inputs, verify, create, resolve }  // exact field names follow 1.4 worksheet
// VerifyCheck, CreateAction, ResolveExpr — typed nodes/expressions, not Symbol strings
```

### 1.2 `CiUpsertStep<T>` — CI specialization of `Upsert<T>`

```dag
type CiUpsertStep<T> = Upsert<T> {
  inputs: List<UpsertInputRef>
  verify: VerifyCheck
  create: CreateAction
  resolve: ResolveExpr
  // NO cache_key field.
  // Cache authority: content_hash(projection_node(CiUpsertStep<T>)) at emission / lookup time
  // (B1 Merkle catamorphism — modeling-discipline.md Practice 10 row 5).
  // Aligns: ci_command_cache_digest / ci_job_cache_digest pattern in ci.dag today;
  //          test_claim_interpretation_cache_digest (T-21) for eval cache scope.
}
```

**P2 / Practice 11:** `cache_digest` appears only on **receipts** (`CiStepSelection.cache_digest`) as a **projection** of the step subgraph hash, never as authored step payload.

### 1.3 `UpsertInputRef` — typed verify-first input coproduct

```dag
type UpsertInputRef
  = FileSet { selector: FileSetSelector }
  | SubstrateNodeSet { selector: NodeQuery }
  | LensOutputRef { lens: LensIdV0, ports: List<Port> }
  | TestClaimRef { claim_id: Symbol }
  | UpstreamUpsert { step_id: CiStepId }
```

| Variant | Authority | Notes |
| -------- | --------- | ----- |
| `FileSet` | `FileSetSelector` + `RepoPath` ingress | GitHub path strings normalize to `FileSetSelector` at ingress; no bare glob `Symbol` in model. |
| `SubstrateNodeSet` | **`NodeQuery` (NEW)** | Must query structural node sets (module/symbol/locus), not string paths. Concept home: **`v4.std.node`** or **`v4.lens.application`** — manager picks one in 1.4/1.5 landing PR; no third path table. |
| `LensOutputRef` | `v4.lens.registry` `LensIdV0` | Use landed registry ID type; do not coin parallel `LensId` for CI-only. `Port` may be `Symbol` until port substrate exists — document as 🟡 if so. |
| `TestClaimRef` | `v4.std.verification` `TestClaim` | `claim_id` keys existing claim data rows; pairs with T-22 eval + `ci_select_from_affected_set`. |
| `UpstreamUpsert` | `CiStepId` | Recursive UPSERT dep resolution; `step_id` is typed step identity, not job name string. |

**Forbidden inhabitants:** `Always`, `RunRegardless`, `Policy`, `FileGlob`, `dependency_set`, raw `List<Symbol>` inputs.

### 1.4 Supporting carriers (same landing PR family)

```dag
type FileSetSelector {
  root: RepositoryRoot      // canonical repo root — NOT raw path string at authority boundary
  pattern: GlobPattern      // dsl/std/types.dag — NOT bare Symbol
}

type RepositoryRoot = RepoRoot | AbsolutePath { path: FilePath }
// Proposal: prefer single data authority `repo_root: RepositoryRoot = RepoRoot`
// and reuse v4.lens.edit_locus.RepoPath for path literals — manager may collapse to RepoRoot only.

type CiStepId {
  job: Symbol
  gate: Symbol              // optional gate arm when step maps 1:1 to CiGate; manager confirms mapping
}
// Alternative (if gate-less jobs only): newtype wrapper over Symbol with brand — still NOT bare Symbol on UpsertInputRef.

type CiCarveout {
  step_id: CiStepId
  reason_code: Symbol
  reason_detail: String
  dissolution_target: DissolutionTarget
}

type DissolutionTarget
  = ModelMissingSubstrate { what: Symbol }
  | UnknownYet { investigation_owner: Symbol, review_due_by: Symbol }

data ci_always_run_carveouts: List<CiCarveout> = [ /* small honest list */ ]

type CiSelectionReceipt {
  pr: ChangeSet
  affected: AffectedSet
  selected: List<CiStepSelection>
  skipped: List<CiStepSelection>
  carved_out: List<CiCarveout>
}

type CiStepSelection {
  step_id: CiStepId
  inputs_consulted: List<UpsertInputRef>
  affected_intersection: List<AffectedNode>   // NEW or alias of AffectedDependency — manager ties to change.dag
  decision: SelectionDecision
  cache_digest: ContentHash                  // projected — std/types ContentHash
  reason: Symbol
}

type SelectionDecision
  = Run
  | Skip
  | CarvedOut { carveout_reason: Symbol }
```

**Selection rule (authority, not heuristic):**

```text
step_runs ⟺ step ∈ ci_always_run_carveouts
           ∨ intersect(step.inputs, affected_set(PR)) ≠ ∅
```

### 1.5 Cache-key boundary (explicit)

| Question | Answer |
| -------- | ------ |
| Where is cache key stored? | **Nowhere on the step.** Derived at use site. |
| What is hashed? | Complete `CiUpsertStep<T>` subgraph (inputs + verify + create + resolve + T specialization). |
| Why not inputs-only? | Two steps with same inputs and different verify/create would collide — P2 cache-scope violation (v4-ci-overhaul §5). |
| Existing pattern | `ci_*_cache_digest` fns; `test_claim_interpretation_cache_digest`; B1 `content_hash` |

---

## §2 DFS concept-home map (M9)

```text
Concept                    | Home (authoritative)              | CI schema action
---------------------------|-----------------------------------|------------------
Upsert operational canon     | dsl/std/patterns.dag              | Consume (1.4), specialize
Effect witness (meet)        | dsl/std/effects.dag UpsertEffect    | Classify CI writes
File path / glob structure   | dsl/std/types.dag GlobPattern       | FileSetSelector.pattern
Git path ingress             | v4.lens.edit_locus RepoPath         | Normalize → FileSetSelector
Affected PR set              | v4.std.change AffectedSet           | Layer C verify input
Rerun frontier               | v4.lens.affected_set                | intersect() authority
Lens identity                | v4.lens.registry LensIdV0           | LensOutputRef.lens
TestClaim identity           | v4.std.verification                 | TestClaimRef
CI pipeline shell            | v4.workflow.ci CiPipeline           | Host CiUpsertStep rows
Step cache projection        | v4.std.node content_hash            | Derive digest fns
```

**New concepts requiring explicit home (manager must pick before implementation):**

1. `NodeQuery` — substrate node set selector (for `SubstrateNodeSet`).
2. `RepositoryRoot` / `RepoRoot` — repo boundary for file sets (or justify `RepoPath` only).
3. `CiStepId` — step identity (replaces raw `Symbol` in upstream refs and receipts).
4. `AffectedNode` (receipt field) — projection of intersection evidence; likely alias/subset of `AffectedDependency` or `Change.subject`.

---

## §3 Consumption of existing substrate (no parallel authorities)

| Existing artifact | How CI schema consumes it |
| ----------------- | ------------------------- |
| `CiJob` / `CiCommand` / `CiGate` | Migration source rows; each becomes a `CiUpsertStep<_>` instance; old `CiGateRunPolicy::Always` does **not** migrate to `UpsertInputRef` |
| `ci_component_affected_from_git_diff` | Coarse layer until per-step inputs wired; receipt may still compute component flags for shadow mode |
| `ci_select_from_affected_set` | TestClaim roster narrowing — `TestClaimRef` + `SubstrateNodeSet` must align with this frontier |
| `content_hash` / `ci_*_projection_node` | Template for step-level `ci_upsert_step_projection_node` |
| `dsl/std/patterns.dag` UPSERT header | Operational semantics for verify-first + recursive deps |

---

## §4 Spot-fix register (forbidden — grep gate for reviewers)

| Pattern | Why forbidden |
| ------- | ------------- |
| `inputs: List<Symbol>` | String-keyed file globs — not structural |
| `FileGlob { glob: Symbol }` | Superseded by `FileSet { selector: FileSetSelector }` |
| `cache_key:` on step type | Parallel payload vs derived `content_hash` |
| `UpsertInputRef::Always` / `CiRunPolicy` / `CiRunMode` | Heuristic enum; use `ci_always_run_carveouts` |
| `UpstreamUpsert { step_id: Symbol }` | Untyped step identity |
| `dependency_set` / `DependencySource` | Retired vocabulary (PR #3959) |
| Reusing `CiGateRunPolicy` for step selection | GHA interim projection only (🟡) |

---

## §5 Falsification probes (acceptance)

1. **Cache sensitivity:** For fixed `inputs`, mutate only `verify` → `content_hash(step)` changes (manual claim or TestClaim in `v4/test/claim/workflow/`).
2. **Collision rejection:** Two `CiUpsertStep` values with identical `inputs` and different `create` → different cache digests; cache lookup must not merge.
3. **Receipt shadow mode:** Pipeline emits `CiSelectionReceipt` with `decision` per step before CI active-skip; carved-out steps cite `ci_always_run_carveouts` entry.
4. **Forbidden-pattern grep:** Implementation PR must not introduce §4 patterns.
5. **Ingress normalization:** Raw GitHub path string in transport only; persisted model rows use `FileSetSelector` + `GlobPattern`.

Secondary metrics: skipped-step ratio, wall-clock — report only after 1–4 pass.

---

## §6 Downstream worker brief shape (after approval + Phase 1.4)

```text
Land §1 types in src/v4/workflow/ci.dag (and concept-home modules for NodeQuery / RepositoryRoot if split).

MUST:
  - Specialize Upsert<T> as CiUpsertStep<T> exactly per approved §1.2–1.3
  - Project cache_digest via content_hash(complete step projection node)
  - Add ci_upsert_step_projection_node + ci_upsert_step_cache_digest
  - Declare ci_always_run_carveouts data (may be empty initial)
  - Add CiSelectionReceipt construction in shadow mode (receipt before active skip)

MUST NOT:
  - Any §4 forbidden pattern
  - Migrate ci.yml by hand as authority
  - Block on wall-clock CI improvement

Escalate (do not spot-fix):
  - Any step whose inputs cannot be expressed as UpsertInputRef without new variant
  - NodeQuery expressivity gap for a real CI step
```

---

## §7 Non-goals (this worksheet / Phase 1.5 substrate PR)

- Phase 1.4 parser/generic `Upsert<T>` landing (separate worksheet).
- Phase 2.5 active skip / GHA `if:` dissolution (Compiler Spine; depends on receipt stability).
- Converting all 91 ci.yml steps (atom migration Phase 1b).
- `timeout: Duration` per step (noted in v4-ci-overhaul §9 — optional follow-on).
- Replacing `CiGateRunPolicy` GHA projection (remains 🟡 until gate #98/#100).

---

## §8 Manager approval checklist (proud-pike-680)

- [ ] §1.3 `UpsertInputRef` coproduct approved as sole input authority
- [ ] `NodeQuery` concept home selected
- [ ] `CiStepId` shape approved (job+gate vs branded Symbol)
- [ ] `RepositoryRoot` / `RepoRoot` resolution approved
- [ ] Cache derived-only rule accepted (no payload field)
- [ ] Carveout + `UnknownYet` owner/cadence accepted
- [ ] Worker dispatch authorized (still blocked on Phase 1.4 completion)

---

## Related artifacts

- `docs/planning/v4-ci-overhaul-2026-05-30.md` — architecture §5–§8
- `docs/planning/v4-correctness-ladder-2026-05-30.md` §10.0 — worksheet discipline
- `docs/planning/v4-leaf-model-verification-2026-05-30.md` §8 — consumer example `CiUpsertStep<VerificationReport>`
- `dsl/std/patterns.dag` — UPSERT<T> canon
- `src/v4/workflow/ci.dag` — current CI substrate + cache digests
- `docs/audit/upsert-pattern-compiler-stray-2026-05-29.md` — stray scan
