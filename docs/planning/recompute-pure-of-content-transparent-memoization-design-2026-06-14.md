# Recompute pure-of-content — one error class, transparent memoization, compute/cache substrate

Work item: `node://adhoc-4a3d8313-94c` (sleek-bee-765) · CI-investigation tree (swift-stag-552).

**Status: DESIGN / MAP ONLY.** No substrate lands from this doc. It names one fundamental
error class, catalogs code-verified instances, states the systemic fix (transparent
memoization), and motivates why `std.compute_fabric` + `std.cache_interface` are the
cross-run landing zone — not a third parallel cache story.

Companion receipts: RR-L (`docs/planning/v4-incremental-bootstrap-ci-perf-rr-l-worksheet-2026-06-02.md`
cache-key law), RR-K (`docs/planning/v4-affected-set-selected-execution-rr-k-worksheet-2026-06-02.md`
"projects, does not recompute"), thesis automatic memoization
(`docs/thesis/what-else-falls-out.md` §Automatic memoization / §Incremental cross-run execution).

---

## 1. The one error class

**Name:** *recompute pure-of-content*.

**Shape:** A consumer executes (or re-derives) work whose result is already determined by
declared **content facts**, when an observationally equivalent answer exists from memoization
or cache lookup keyed on those same facts.

**Pure-of-content** (the positive predicate the error violates):

| Leg | Meaning | Authority already in tree |
| --- | --- | --- |
| **Pure** | No side effects; same inputs ⇒ same outputs (within the modeled evaluation strategy). | `std/effects.dag` partition; v2 interpreter `is_structural_pure_fn` gate; R3 auto-memoization claims |
| **Of-content** | The lookup identity is a **content fact** — structural form, `content_hash`, source span, declared input digests — not a mutable context id (intern-table slot, env generation, host path without modeled invalidation, frame count, declaration *name* alone). | `EvalMemoKey` / `EvalStateKey`; `ParsePositionKey`; `TestClaimCacheKey`; RR-L §2.1 cache-key law (#4282) |

**Why this is ONE class, not many bugs:** In a closed system, "I forgot to cache," "I cached on
the wrong key," "I rebuilt what an upstream carrier already held," and "the host transport
re-derived modeled facts" are the same structural defect viewed at different boundaries —
**duplicate derivation of a content-determined fact** (INVARIANTS P1 "No duplicate
representations," P5 "Progress Is Dissolution"). The perf symptom varies (O(n²), exponential
parse, CI wall-clock, false-green); the causal shape is identical.

**Concept-unification test** (`docs/thesis/concept-unification.md`): if fixing the next
instance requires inventing a new mechanism rather than applying transparent memoization with
a content key, investigate whether the instance is really distinct. A parallel cache authority
is evidence of a missed unification.

---

## 2. Systemic fix — transparent memoization

**Definition:** Insert a memo or cache layer keyed on the declared content identity of a
pure-of-content computation such that:

1. **Observational equivalence** — hits produce bit-identical results (and, where modeled,
   identical diagnostics) to a full re-execution. Performance caches are invisible rewrites
   (RR-L §2.1).
2. **Content-keyed** — keys cite immutable content authority; mutable context ids are rejected
   as primary identity (RR-L rejected pattern: intern-id keys across `TypeEnv` rebinds).
3. **Single authority** — one producer owns the fact; downstream **projects** it, never
   recomputes a parallel derivation (RR-K §2.4; `docs/perf/clone-elimination.md` Rule 3:
   after threading X, delete "recompute X").
4. **Transparent to the author** — memoization is not part of the user-facing contract
   (M0/M1 doctrine: lenses are pure readers; memo is a local concern if profiling demands it).
   The compiler/runner may insert it when purity + content key + cost facts are already known
   (thesis §Automatic memoization).

**Two scopes, one mechanism:**

| Scope | Mechanism | Substrate hook |
| --- | --- | --- |
| **Within-run** | In-process memo table (evaluator, parse table, import closure, pure-call memo) | `EvalMemoKey`, `ParseTable`, v2 `pure_call_memo` |
| **Cross-run** | Content-addressable artifact store with declared invalidation | `std.cache_interface` facts rows + `ExecutionReceipt` linkage |

Cross-run caching is not a separate product idea — it is within-run memoization with persistence
and explicit invalidation triggers (`InvalidationTrigger` on `CacheInterfaceFacts`). The thesis
already states both consequences (`docs/thesis/what-else-falls-out.md` lines 405–441).

---

## 3. Code-verified instances (qualitative catalog)

Each row was verified against the tree at design time (2026-06-14). "Consumer" names the
green test, claim, or operational receipt that would fail if the fix were wrong.

### 3.1 Within-run — missing or wrong memo

| Instance | Symptom | Content key (correct) | Wrong pattern | Consumer / receipt |
| --- | --- | --- | --- | --- |
| **Parse table** | Exponential re-walk on right-recursive grammars without `(position × production)` memo | `ParsePositionKey { position, production }` → `ParseTable` | Re-parse shared `(pos, prod)` cells | `src/v4/compiler/02_parse.dag:155–161`; claim `parse_production accepts right-recursive A = 'x' A? … via memoized ParseTable` (`grammar_validation.dag`) |
| **v2 structural pure calls** | Redundant re-traversal of `content_hash`, `fold_node`, `well_formed`, … in emit pipeline | Resolved fn identity + arg `Rc` identities (sharing-preserving) | Re-execute structural predicates on every call site | `src/v2/stage0/src/v2_interpreter.rs:2180–2232` (`pure_call_memo_*`, `is_structural_pure_fn`); comment names emit-pipeline collapse |
| **Module import closure** | Re-load / re-parse same module path in one compile | Module path string (parser is single authority) | Second load per import edge | `src/v2/tests/src/helpers.rs:96–101` (OnceLock module index; "loaded exactly once (memoized by module path)") |
| **v3 evaluator memo key** | Would cache on name/frame-count fingerprint | `EvalMemoKey { program, node, state_key: EvalStateStack, strategy }` | String digest, declaration name, frame count | `src/v3/std/runtime.dag:92–122` (TERMINAL marks: "Memoization identity is structural") |
| **v4 TestClaim eval cache** | Re-run identical claim interpretation | `TestClaimCacheKey` → `interpretation_hash` over claim + facts + `evaluator_input` `content_hash` | Parallel stored digest fields; identity-only keys | `src/v4/compiler/05_eval.dag:526–537`; `test_claim_cache_digest_sensitivity.dag` (distinct diagnostics ⇒ distinct hash; Node input participates) |

### 3.2 Within-run — cache key used mutable context (false hit / false miss)

| Instance | Symptom | Content key (correct) | Wrong pattern | Consumer / receipt |
| --- | --- | --- | --- | --- |
| **v2 infer lookup/reconcile** (#4282) | O(n²) reconcile or false cache hits when `TypeEnv` rebinds intern ids | Source span / content-stable scrutinee key | Memo keyed on intern id across env updates | RR-L §1 landed evidence (#4282); §2.1 accepted vs rejected patterns |
| **Byte-offset cache boundary** | Aliasing or false eligibility at eval cache boundary | `byte_offset_cache_key` from `ByteOffsetCacheDigestAuthority` (eligible vs ineligible tagged) | Bare `Hash` or Peano on magnitude without authority | `src/v4/std/node.dag:678,1431–1449`; `test_claim_cache_digest_sensitivity.dag` fingerprint inequality claims |

### 3.3 Cross-boundary — parallel re-derivation (second authority)

| Instance | Symptom | Content key (correct) | Wrong pattern | Consumer / receipt |
| --- | --- | --- | --- | --- |
| **Affected-set host transport** | Drift between modeled frontier and hand-maintained path buckets | Project `CiComponentAffected` / `AffectedSet` modeled facts | `detect-affected-components.sh`-style second detector; per-job `changed?` re-read of git diff | RR-K §2.1–2.4; `tools/ci_affected_components/src/receipt.rs:11–13` ("projects, does not recompute") |
| **CI timing ledger** | Re-derive timings from raw windows in multiple places | `job_windows_to_timings` single authority | Duplicate timestamp math in shell + Rust | `receipt.rs:11–12` |
| **merge_envs / fact-flow** | Downstream rebuilds fact already threaded on input | Return the authoritative input carrier | `merge_envs` reconstructs `InternTable` already on `TypeEnv` | `docs/perf/clone-elimination.md` Rule 3–4 (PR2 negative receipt) |
| **NodeArtifactProvenance** | Allowlist narrower than real compile closure | Fold `source_ir_node_artifact_provenance` over source-root ingest | Hand-maintained path→bucket table parallel to compiler closure | `docs/planning/diff-node-artifact-provenance-producer-design-2026-06-13.md` §Why this exists |
| **Bootstrap hash pins** | Placeholder digest aliases until fixed-point proves convergence | Merkle `content_hash` per stage (T-15 dissolution) | Symbolic `Hash` data aliases standing in for computed digests | `src/v4/workflow/bootstrap.dag:3,33,414` (needs-more-work marks) |

### 3.4 Cross-run — cache without modeled interface (interim transport)

| Instance | Symptom | Content key (correct) | Wrong pattern | Consumer / receipt |
| --- | --- | --- | --- | --- |
| **sccache in CI** | Hand-wired `RUSTC_WRAPPER=sccache` probe in `ci.yml` | `sccache_local_facts` row: `NativeInternalHash` over `RustcInvocation` + `ToolchainSpec`; `content_verified_on_read: true` | Ad-hoc socket probe + env flip without `cache_interface` projection | `.github/workflows/ci.yml:70–91`; `dsl/std/cache_interface.dag:280–321` |
| **GHA Actions cache** | Prefix fallback keys without typed miss/reject semantics | `gha_actions_cache_facts` (`PrefixFallback`, `InvalidationTrigger` list) | Untyped cache key strings in workflow YAML only | `cache_interface.dag:230–277` |
| **BuildBuddy CAS** | Remote CAS without producer receipt linkage | `buildbuddy_cas_facts` + `ProducerReceipt` / `CachedArtifactReceipt` | Raw remote hash without `CacheRejectReason` surface | `cache_interface.dag:324+` (row skeleton) |

### 3.5 Planned / gated — compiler-inserted memo (not yet territory)

| Instance | Expected content key | Status | Consumer / receipt |
| --- | --- | --- | --- |
| **R3 auto-memoization** | Repeated pure call sites emit cache scaffolding; one-shot sites do not | Gates in `r3_free_consequences_first_batch.dag`; repeated-call caching deferred | `r3_free_consequences_first_batch_test.rs` (`auto_memoization_*` claims) |
| **Dependency `RecomputePlan`** | `AffectedSet` + `ReadinessLayer` list — *what* to re-execute after change, not spurious full recompute | Wave-2 fixture | `dependency_recompute_plan.dag` + `v4.std.change.RecomputePlan` |
| **Emit hang diagnosis** | — | **Refuted:** emit hang was v2-interpreter runtime cost, not missing `fold_node` memo (INVARIANTS read-this-first receipt) | `INVARIANTS.md` lines 40–45; `docs/v4-compiler-migration.md` |

---

## 4. Why `compute_fabric` + `cache_interface` (not a third cache story)

The within-run fixes above are locally correct but do not compose into a **typed cross-run
contract**. CI already runs three cache surfaces (sccache, GHA cache, BuildBuddy) as interim
host transport. Without a modeled interface, each surface reinvents key derivation,
invalidation, and fail-closed reject reasons — the cross-run form of "recompute
pure-of-content" (re-execute rustc when `sccache_local_facts` says hit; or trust a prefix key
without `CacheRejectReason::SubjectDigestMismatch`).

**Composition law (already landed in std):**

```
compute_fabric.dag header:
  "Composition: cache via ExecutionReceipt.output only — MUST NOT import cache_interface."

cache_interface.dag header:
  "Composition: compute via ExecutionReceiptRef<T> digest only — MUST NOT import compute_fabric."
```

Acyclic on purpose — one digest seam, two authorities:

| Module | Owns | Cache touchpoint |
| --- | --- | --- |
| **`std.compute_fabric`** | *Demand* satisfied, *lease*, wall-clock/cost `ExecutionReceipt<T>` | `ToolchainCapability.linked_cache_interface: Option<CacheInterfaceId>` (`compute_fabric.dag:259–264`); receipt carries `output: Outcome<ArtifactRef<T>>` (`:448–456`) |
| **`std.cache_interface`** | *Store* semantics per backend (`CacheInterfaceFacts`, lookup/write/miss, eviction, reject reasons) | `ExecutionReceiptRef<T> { receipt_digest: ContentHash }` as producer witness only (`cache_interface.dag:203–210`); `CachedArtifactReceipt` on hit (`:215–220`) |

**Transparent memoization across runs** is then mechanical:

1. Pure-of-content work item `W` runs under a `ComputeLease` → `ExecutionReceipt<T>`.
2. `output` digest + declared `ArtifactKindId` + subject digest form the cache key per the
   linked `CacheInterfaceFacts.key_derivation` row (e.g. sccache `NativeInternalHash`,
   GHA `HandAuthoredString` + prefix fallback).
3. On cache hit, `CacheLookupResult::Hit` carries `CachedArtifactReceipt` — **project** the
   artifact, do not re-execute `W` (RR-K §2.4 pattern at the compute/cache boundary).
4. On mismatch, typed `CacheRejectReason` (fail-closed — no fabricated hit).

This is the dissolve target for every `ci.yml` "INTERIM sccache transport" block: the workflow
becomes a **projection** of `cache_store_view(sccache_local_facts)` (and sibling rows), not a
parallel cache authority.

**Why not fold cache into compute_fabric only?** Store semantics vary orthogonally from host
supply (Worksheet A vs B §1.6 grid — locality, eviction, consistency, auth). One
`ComputeHost` can link multiple `CacheInterfaceId`s; one sccache row serves many toolchains.
Merging would recreate the dual-representation defect the composition law prevents.

---

## 5. Decision procedure (for implementers)

When reviewing code or designing a new stage:

```
1. Is the computation pure-of-content?
   - Pure: check effects partition / evaluator purity gate.
   - Of-content: can you name the content authority (hash, span, digest, structural key)?
     If the only stable id is a mutable context slot → STOP; fix the key first (RR-L).

2. Is the result already available from an upstream carrier?
   - YES → project; delete recompute (clone-elimination Rule 3).
   - NO  → go to 3.

3. Will this run more than once with the same content key?
   - Within one process → transparent memo (table/map at the boundary).
   - Across runs → link `CacheInterfaceId` on the toolchain/host; record
     `ExecutionReceipt` + lookup via `cache_interface` semantics.

4. Does a host transport re-derive modeled facts?
   - Forbidden (RR-K). Transport projects `.dag` facts or fails safe.
```

---

## 6. Non-goals (this design doc)

- No new memo substrate in `std/node` or compiler stages from this PR — map only.
- No promotion of R3 auto-memoization gates to emitted scaffolding without a consumer slice.
- No `ci.yml` rewrite in this doc — dissolve-on-arrival when cache_interface projection
  consumer lands (existing comments in workflow).
- No conflation of **incremental re-execution after source change** (`RecomputePlan`,
  `AffectedSet`) with **spurious recompute** — change-driven recompute is correct when the
  content key *changes*; the error class is recompute when it has not.
- No claim that hash equality is semantic equality — lookup verification on read
  (`content_verified_on_read`, `CacheRejectReason::ContentDigestMismatch`) handles collision
  at the store boundary (thesis caveat, `what-else-falls-out.md` lines 434–437).

---

## 7. Falsification probes (before an implementation PR claims PROVEN)

| ID | Probe | Receipt |
| -- | ----- | ------- |
| F1 | Memo/cache hit is observationally identical to miss path on a fixed fixture | Equivalence test (RR-L R5 form for cross-run; eval cache claims for within-run) |
| F2 | Cache key does not use mutable context id for facts that change across env updates | #4282-class test / RR-L R1 |
| F3 | Host transport does not recompute facts owned by `.dag` authority | RR-K R7 diff review |
| F4 | Cross-run hit carries `CachedArtifactReceipt` with typed reject on mismatch | `CacheRejectReason` exhaustiveness + negative hit test |
| F5 | `compute_fabric` / `cache_interface` acyclic import law preserved | `handwritten_parser_accepts_*_dag` integration tests |
| F6 | One-shot pure calls emit no memo scaffolding (no spurious cache) | `auto_memoization_no_caching_for_one_shot` gate |

---

## 8. Suggested landing order (downstream workers)

1. **Within-run receipts first** — extend existing memo boundaries (`05_eval` cache key,
   parse table, v2 pure-call memo) with falsification tests F1–F2.
2. **cache_interface projection consumer** — CI workflow reads `sccache_local_facts` /
   `gha_actions_cache_facts` rows; retires hand-wire (F3, F4).
3. **compute_fabric linkage** — populate `ToolchainCapability.linked_cache_interface` on
   grounded supply rows; `ExecutionReceipt` output digests feed cache lookup planner (F5).
4. **R3 emitter auto-memo** — only after purity+cost lens can prove content keys at emit
   time (gates already staged; repeated-call half deferred).

---

## 9. One-line summary

**Recompute pure-of-content** is duplicating work that declared content facts already
determine; **transparent memoization** is the single fix (within-run table, cross-run
`cache_interface`, linked from `compute_fabric` receipts) — and the repo already contains
qualitative proof instances at every layer from parse memo through CI sccache interim transport.
