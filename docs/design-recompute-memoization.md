# [DRAFT] Recompute pure-of-content — one fundamental error, transparent memoization

Work item: `node://adhoc-4a3d8313-94c` (sleek-bee-765) · CI-investigation tree.

**Status: DRAFT — ROADMAP alignment pass (2026-06-14).** Operator-confirmed §2/§10 spine
unchanged; vocabulary, carrier shape, ARC, and findings aligned to portfolio authority.
§1 centerpiece and §3 instance roster unchanged.

**Reader's map:** §1 proof it's real → §2 the law (+ ROADMAP vocabulary) → §3–4 instances
and fix → §5–7 enforcement + migration → §9–10 falsifiers + summary.

**Map / motivation only** — points at inline marks as durable authority; not a parallel
ledger for per-cache facts (operator standing principle, 2026-05-19).

**Portfolio authority:** [ctrl/ROADMAP.md — Realization pattern](https://github.com/gunb-ai/ctrl/blob/main/ROADMAP.md#cross-cutting-requirement--the-realization-pattern)
(ctrl #1609). ROADMAP cross-links this doc as its design root; this doc cross-links
ROADMAP for shared vocabulary — reference, do not duplicate.

**Durable solution home:** `dsl/std/{cache_identity,cache_interface,compute_fabric}.dag`.

---

## 1. Centerpiece — resolve-cost PR1 (#4867): cache revealed impurity, did not cause it

**Canonical example.** When resolve-cost PR1 (#4867, merged) added a per-module typed cache
on `MultiEntryIndex`, claim verdicts **flipped depending on entry order**. A cache that
changes a verdict is **not a cache bug** — it is a **purity bug the cache exposed**.

**What was wrong structurally:** `build_type_env` interns kernel type names into a *local
clone* of the ambient intern table at type time. Kernel-type ids therefore depended on how
many tokens happened to precede them in that table — table-**size**-dependent, not
content-stable. A module typed for an early entry baked ids at one table size; a later entry
reused that binding under different ids; lookups missed and types collapsed to the `Json`
fallback — an order-dependent verdict flip.

**Manual discharge today (not automatic enforcement):**

- `seed_kernel_intern_names` pre-seeds the shared table so kernel names get stable ids across
  every entry in an index (`src/v1/stage0/src/cli_run.rs:306-347`).
- `resolve_typed_cache_equivalence_test` is a **standing purity-oracle gate**: cached resolve
  must be byte-identical to the no-cache cold oracle in every entry order
  (`src/v1/tests/src/resolve_typed_cache_equivalence_test.rs:1-19`).

**First inhabitant on the substrate path:** PR1 is also the live first chapter of the
`cache_interface` story — resolve-cost PR2 (in flight) grounds the staged
dims as the first real consumer; PR1's manual seed + falsifier is what exists today.

Model authority for the type-time interning shape:
`src/v1/04_infer.dag:5453-5460` (`build_type_env` folds `kernel_type_set` into the table).

**Distinct from infer reconcile cache (#4282):** that lever memoizes lookup/reconcile with
span-only keys across `TypeEnv` updates — a different stage, different impurity class. PR1
is resolve-stage typed-graph cache + intern-id content-stability; do not collapse the two.

---

## 2. Thesis — one missing carrier, two invariant faces *(operator-confirmed spine)*

### Root cause: no reified execution receipt at the model→host boundary

The root defect is **not** “the language has no notion of pure-of-content.” The v2
five-`Behavior` kernel already has **purity by construction** at the model layer. The gap
is at the **execution boundary**: a pure model is run by an impure host, and nothing
**reifies the receipt** of that run — no carrier where declared purity meets host
impurity and records what was actually executed, on what inputs, with what result.

Without that receipt carrier, every consumer that needs an already-computed result either:

1. **Recomputes from scratch** — redundant work, or
2. **Hand-rolls a local cache** — necessary but **unsafe** when the host run was never
   reconciled to the model’s purity claim.

These are not two bug classes. They are **one missing carrier** surfacing on **two
invariants**:

| Face | Invariant | Symptom |
| --- | --- | --- |
| **Redundancy** | **Performance / Facts-Flow-Forward** | Pure-of-content work repeated when the content key is unchanged — facts recomputed instead of projected forward from a receipt |
| **Un-enforced purity** | **Fail-Closed** | Hand-rolled cache returns wrong answers silently; **verdict flips** when hidden host state leaks in (§1 PR1 #4867) |

Do not collapse both faces under “no duplicate representations” alone — that is the weakest
joint. The unification is **one missing execution-receipt carrier**, visible on two
existing invariant axes.

**Named error class:** *recompute pure-of-content* — executing or re-deriving work whose
result is already determined by declared **content facts** when a reified receipt (or
memo/cache keyed on it) would make recomputation unnecessary and falsify impurity. Per
[ctrl/ROADMAP.md](https://github.com/gunb-ai/ctrl/blob/main/ROADMAP.md#cross-cutting-requirement--the-realization-pattern):
keying on anything other than the **content of declared inputs** (PR1 #4867 keyed on
intern-table *size*).

### Portfolio vocabulary — three primitives + carrier *(ctrl/ROADMAP.md)*

The §2 spine (missing receipt carrier → two invariant faces) maps onto ROADMAP's shared
vocabulary without respining. **Three primitives ↔ three §4.3 TARGET properties — 1:1,
no fourth primitive, no parallel digest/receipt type:**

| ROADMAP primitive | §4.3 TARGET | Substrate attach |
| --- | --- | --- |
| **Content identity** | **(c) Content-keyed invalidation** | `ContentHash` / `ExecutionReceiptRef<T> { receipt_digest }` digest seam (`cache_interface.dag:203-204`) — memo-hit-vs-recompute key |
| **Hermetic realization** | **(b) Enforced purity at boundary** | Equivalence falsifier (§5) — **purity IS the falsifier**, not a new mechanism |
| **Receipt + change-driven reconcile** | **(a) Automatic memo** | **Persistence:** `cache_interface` store/lookup + `ExecutionReceipt<T>` (`key → output` on the compute_fabric lane). **Change-consequence:** `RecomputePlan` / `AffectedSet` (`v2.std.change` — v2-owner confirmed: rerun-scope / what dependencies must rerun **only**; does **not** hold persistence). **TARGET composition:** `(ExecutionReceipt persistence) × (AffectedSet/RecomputePlan rerun-scope)` — not a field on `AffectedSet` today |

**Substrate carrier (the N+M move):** `Realization<Spec, Effect>` names the **existing**
`compute_fabric` ↔ `cache_interface` composition — **not** new types. Abstraction over the
pair where `ExecutionReceipt` **is** a cache entry on the compute timeline (§4.4):

| Parameter | Existing substrate attach |
| --- | --- |
| **Spec** | `WorkUnit<T>` / `ArtifactSpec<T>` — what to compute (`compute_fabric.dag:398-403`), keyed by `ContentHash` |
| **Effect** | `ExecutionReceipt<T>` / `Outcome<ArtifactRef<T>>` — realized outcome (`compute_fabric.dag:448-451`) |
| **Handler** | `WorkDemand.effects: List<EffectBoundary>` — std effect/capability discipline (`compute_fabric.dag:274`; `EffectBoundary` forward-stub; inhabited pattern `v2.std.runtime.ResourceEffectBoundary`) — **not** a bespoke effect type |
| **Digest seam** | `ContentHash` / `ExecutionReceiptRef<T>` — content-addressed reconciliation across the acyclic firewall (`cache_interface.dag:203-204`) |

One kernel, N handlers (compute, build, provision, migrate, schedule). Partial pieces
already scatter in `cache_identity`, `cache_interface`, `compute_fabric`, and
`v2.std.change`.

**Inhabitation target (staged):** `CacheInterfaceFacts`, staged dims (`PersistenceLocality`,
`ArtifactKindId`, …), + `ExecutionReceipt<T>`. ARC inhabitant #1: `#4878` (resolve-cost).

**Staging guard (hard):** `Realization<Spec, Effect>` is a **TARGET / design-of-record
carrier name** — **not** a minted realized kernel type — until ≥2 **cross-layer**
inhabitants prove it (resolve #1, sccache #2, provisioning #3). Same ≥2 restraint as
`CacheFabric<T>` (§4.4). Frame as target, not built type.

**Sharpened findings (portfolio review, 2026-06-14):**

- **A — Open vs closed world.** One-door hermeticity holds for the compiler; provisioning /
  DB / cloud have a second door (external state drifts unobserved). The carrier needs two
  handler classes.
- **B — Oracle splits on reversibility.** Re-execution oracle (byte-identical cached vs cold)
  is sound only when realization is free to redo. Irreversible handlers need durable receipt
  + idempotency key (§5; §3.3 row 11 vs future provisioning).
- **C — Same-code scope.** Byte-identical requirement applies to the **identity + receipt
  kernel**, not the realize step or oracle (legitimately handler-parameterized). Matches the
  operator rule: same code among our own code for the kernel; handlers may differ.

**ARC — critical path sequencing** *(authority: [ctrl/ROADMAP.md](https://github.com/gunb-ai/ctrl/blob/main/ROADMAP.md#cross-cutting-requirement--the-realization-pattern); threaded in [gunbc ROADMAP.md](../ROADMAP.md) §Cross-cutting — superseded 2026-06-14 the prior “cross-layer #2 = §10 only” line):*

1. **Resolve (inhabitant #1).** `resolve-cost PR2` (#4878) — receipt + falsifier + reconcile
   on the compute handler; grounds staged `cache_interface` dims. PR1 (#4867) merged.
2. **sccache / build cache (inhabitant #2, de-risking rung).** Closed-world + reversible —
   proves the identity kernel carries across realize-steps; does **not** alone prove
   open-world layer-agnosticism (§3.3 row 11).
3. **§10 / container-runtime MVP provisioning (inhabitant #3, stress test).** Open-world +
   irreversible — idempotency ("has this image been realized on this host?"). Requirement
   met only when this handler carries on the same kernel (Finding A).
4. **Confirm kernel carries** across #1–#3 (same identity+receipt kernel; realize/oracle may
   differ per A/B).
5. **Dissolve hand-rolls** — §3.3 sunny-lynx census trends to 0; delete in same PR as each
   inhabitant (§7).
6. **Endpoint — Realization Lens** — substrate forbids hand-rolled-realization shapes.

### Why v2 recurs — staged carrier, not v2 sloppiness

The pattern is **not** “v2 was sloppy and v2 will be clean.” v2 **already** stages the
same hand-rolled compute-once-store pattern because the receipt carrier is
**staged-not-inhabited**:

- `ParseTable` — memoized `(position × production)` at `src/v2/compiler/02_parse.dag:155-161`
- `TestClaimCacheKey` / `interpretation_hash` — eval cache boundary at `src/v2/compiler/05_eval.dag:526+`

Eleven v2 hand-rolls (§3.3) plus these v2 instances are **the same theorem recurring**:
without an inhabited execution receipt, each layer re-implements store-and-project locally.
That recurrence is **systemic**, not a cleanup backlog.

### Anchor — recurrence is a theorem (Build Systems à la Carte)

Per *Build Systems à la Carte* (Mok et al.): build/caching recurrence across layers is a
**structural consequence** of missing a shared, content-addressed reconciliation of
**spec → effect** — not an accident of engineer discipline. gunbc’s version is the
**Realization** pattern: content-addressed reconciliation of model spec to host effect
across an impurity boundary. Portfolio authority:
[ctrl/ROADMAP.md](https://github.com/gunb-ai/ctrl/blob/main/ROADMAP.md#cross-cutting-requirement--the-realization-pattern)
(operator-elevated 2026-06-14; supersedes ctrl#1607 dep-graph trees). **ARC inhabitant #1:**
`resolve-cost PR2` (#4878, in flight) — the deepest place is resolve.

**Systemic fix (TARGET, staged):** transparent memoization = TARGET
`Realization<Spec, Effect>` over `cache_identity` / `cache_interface` / `compute_fabric`
(§4.4 M9 attach) — the receipt carrier that makes the run reifiable, keys results by
content identity, and wires the purity-oracle falsifier as standing enforcement (§5–§6).
Not claimed working today (§4.2).

**Sharpest operator rule:** sharing must be the **same code among our own code**, not two
abstractions that happen to agree — dogfood the fabric by dogfooding the compiler.

**This doc must not duplicate rosters.** §3 is the **single standing instance roster**;
it subsumes the sunny-lynx eleven-cache census. Lens by failure mode; marks are authority.

---

## 3. Instance roster — single catalog *(subsumes sunny-lynx eleven-cache census)*

§3 is the **only** instance catalog in this doc. It **subsumes** the sunny-lynx eleven-hand-
rolled-cache census: every census inhabitant appears in **§3.3**. **§3.1** adds non-cache
redundancy instances; **§3.2** adds the PR1 (#4867) purity-exposed instance; **§3.4** adds
parallel-derivation instances (not caches, same error class). Per-cache dimension facts
remain in source marks — this section cites them, does not mirror them.

### 3.1 Redundancy — pure-of-content work recomputed (no cache yet)

| Instance | Structural redundancy | Cite |
| --- | --- | --- |
| **`call_function` static param list** | Every call re-derives value-param names from `fn_node.params` via `authored_name_at`, `Vec` + `HashMap` alloc — pure function of static fn shape | `src/v1/stage0/src/v1_interpreter.rs` `param_names` (`:1286-1317`); eval profiler — `eval_profile_enabled` (`:1339`), `eval_profile_snapshot`/`eval_profile_reset` + `EVAL_COUNTS`/`EVAL_SELF_NANOS` (`:4334-4381`) (#4865) |
| **Resolve overlapping closure** | Each entry re-ran tokenize→parse→resolve→reconcile over overlapping modules before shared index | `cli_run.rs:270-303`; cold oracle vs `resolve_entry_with_index` |
| **Cost/complexity lens AST folds** | Lenses fold over AST during witness eval with no shared memo | `src/v2/lens/complexity.dag`; `src/v1/stage0/src/v1_interpreter.rs` lens-eval wall (`:4253-4269`); `eval_profile_snapshot`/`eval_profile_reset` + `EVAL_COUNTS`/`EvalProfile` (`:4334-4381`) |

`src/v1/stage0/src/v1_interpreter.rs:1286-1297`

```rust
    let param_names: Vec<String> = fn_node
        .params
        .iter()
        .filter(|p| {
            let name = authored_name_at(ctx.si(), (*p).clone());
            match p.children.first() {
                Some(type_expr) => authored_name_at(ctx.si(), type_expr.clone()) != name,
                None => false,
            }
        })
        .map(|p| authored_name_at(ctx.si(), p.clone()))
        .collect();
```

### 3.2 Purity exposed by cache — resolve-cost PR1 (#4867) *(distinct instance)*

| Instance | What the cache revealed | Cite |
| --- | --- | --- |
| **Typed-module cache + latent intern-id impurity** | Module typed result claimed content-pure but depended on ambient intern-table **size** at `build_type_env` kernel interning; cache made order-dependent verdict flips visible; `seed_kernel_intern_names` + byte-identical falsifier restore content-stability | `cli_run.rs:306-347`; `v1_compiler_infer.rs:11328`; `resolve_typed_cache_equivalence_test.rs:1-19`; `04_infer.dag:5460` |

This row is §1 in table form — the strongest evidence for the un-enforced-purity face.

### 3.3 The eleven hand-rolled caches — sunny-lynx census *(THE migration roster)*

Facts-before-abstraction: eleven real compute-once-store re-implementations warrant ONE
interface. **This table is the sunny-lynx census** — do not maintain a second catalog
elsewhere.

| # | Hand-roll | What it caches (structurally) | Authority cite |
| --- | --- | --- | --- |
| 1 | `pure_call_memo` | Structural pure fn results by fn identity + arg sharing | `v1_interpreter.rs:1022, 2180-2250` |
| 2 | `data_cache` | Nullary / data CAF values by node identity | `v1_interpreter.rs:1019, 1516-1520` |
| 3 | `parse_cache` | File path → parse result + newline index | `cli_run.rs:280-282` |
| 4 | `SymbolInterner` | String → `Symbol` dedup (identity table) | `v1_interpreter.rs:108-118, 1028` |
| 5 | `typed_module_cache` | Module name → typecheck result (**PR1 #4867** — see §3.2 for purity story) | `cli_run.rs:287-303, 539+` |
| 6 | `COMPILE_CACHE` | Per-(source, file) `compile_to_dag` outcome | `cached_compile.rs:63-70` |
| 7 | Bootstrap snapshots | Committed `Dag` snapshot reuse | `src/v3/compiler/src/lib.rs` (`std_fixture_bootstrap_snapshot`); `src/v3/compiler/tests/integration/pb1_bootstrap_full_snapshot_test.rs` |
| 8 | `.claim-map` | Claim corpus execution map artifact | `scripts/v2-claim-corpus-execution-map.sh:28`; `.gitignore:101` |
| 9 | Discovered-owned-data manifest | Host scan → manifest DAG | `discover_owned_data.rs:8-12`; `host_discovered_owned_data_manifest.dag:2-3` |
| 10 | `.freshness-check` | Stage0 regen verify temp tree | `dsl/gunbc/tools/freshness.dag:68-89`; `Makefile:56` |
| 11 | sccache (interim CI) | **ARC inhabitant #2** — de-risking *handler instance* of the carrier (not a rows 1–10 dissolve target; §7 row 11 governs migration); rustc invocation → object file; closed-world + reversible | `.github/workflows/ci.yml:70-91`; `cache_interface.dag:280-321` |

**Footnote — same class in newer substrate (not roster expansion):** v2 `ParseTable`
(`src/v2/compiler/02_parse.dag:155-161`) and `TestClaimCacheKey` (`src/v2/compiler/05_eval.dag:526+`) — pattern recurs in
v2; not added to the canonical eleven.

### 3.4 Parallel re-derivation — duplicate authority (not a cache)

When host code recomputes facts already owned by `.dag` authority — a second affected-set
detector, timing math duplicated outside the single ledger function — that is the same
error class at the CI boundary. Fix direction: **one authority produces the fact; transport
projects it** (`tools/ci_affected_components/src/receipt.rs:11-13`).

---

## 4. Systemic fix — transparent memoization (TARGET, staged)

**Transparent memoization** = TARGET `Realization<Spec, Effect>` (§2 M9 attach; staging
guard §4.4): first-class pure-content-addressed computation keyed on **content identity**,
hermetically realized, with **receipt + change-driven reconcile** — observationally
equivalent to full re-execution, inserted when purity + content key are known — not part of
the user-facing contract. Effect handler parameterizes the realize step via
`WorkDemand.effects`; identity + receipt kernel is shared (Finding C).

### 4.1 Two scopes, one mechanism

| Scope | Mechanism | Substrate hook |
| --- | --- | --- |
| **Within-run** | In-process memo (evaluator, parse table, import closure, pure-call memo) | `EvalMemoKey`, `ParseTable`, v2 `pure_call_memo` |
| **Cross-run** | Content-addressable store + declared invalidation | `cache_interface` facts + `ExecutionReceipt` linkage |

Cross-run caching is within-run memoization with persistence and explicit invalidation
triggers — not a separate product idea.

### 4.2 What is staged vs realized today

| Layer | Today | Notes |
| --- | --- | --- |
| **Substrate types** | Staged, self-authoritative | `.dag` marks are authority |
| **Structure on v2** | Partially resolves | Type shapes load; full claim-**run** not wired end-to-end |
| **End-to-end eval on self-hosted compiler** | Not yet wired | Three resolve gaps (workers in flight): optional fields; one-of-one enum tags; multi-hop type-alias chains (money/Measure carrier) |
| **PR1 (#4867)** | Manual discharge | `seed_kernel_intern_names` + equivalence falsifier |
| **PR2 (resolve-cost #4878, in flight)** | First real consumer / inhabitant #1 | Structural grounding + Rust behavioral proof; substrate eval honest-dormant |

**Do not read this doc as "the fix already works."** TARGET properties below are what the
substrate is built **toward**.

### 4.3 Three TARGET properties = three ROADMAP primitives *(1:1, substrate-attached)*

| Property | ROADMAP primitive | Target meaning | Substrate attach | Today |
| --- | --- | --- | --- | --- |
| **(a) Automatic memo** | Receipt + change-driven reconcile | Compiler/runner inserts memo; authors don't hand-roll | **Persistence:** `cache_interface` + `ExecutionReceipt<T>`. **Rerun-scope:** `RecomputePlan` / `AffectedSet` (`v2.std.change`) — composed at TARGET, not one substrate field | Eleven hand-rolls; v2/v2 local memos |
| **(b) Enforced purity at boundary** | Hermetic realization | Non-pure units cannot be soundly memoized | Equivalence falsifier (§5) — purity **is** the falsifier | Manual seeds + standing gates (PR1) |
| **(c) Content-keyed invalidation** | Content identity | Merkle/content-hash invalidation, not heuristic deps | `ContentHash` / `ExecutionReceiptRef` digest seam | Partially in `CacheInterfaceFacts`; sccache interim host |

### 4.4 Acyclic composition law *(quoted from substrate headers)*

`compute_fabric.dag` and `cache_interface.dag` form a symmetric pair — one digest seam, two
authorities. Do not re-architect; motivate what is already landed:

```text
// compute_fabric.dag
// Composition: cache via ExecutionReceipt.output only — MUST NOT import cache_interface.

// cache_interface.dag
// Composition: compute via ExecutionReceiptRef<T> digest only — MUST NOT import compute_fabric.
```

- **Compute side:** demand, lease, `ExecutionReceipt<T>`; `ToolchainCapability.linked_cache_interface`
- **Cache side:** `ExecutionReceiptRef<T> { receipt_digest: ContentHash }` as digest-only cross-link

`ExecutionReceipt` **is** a cache entry on the compute timeline; `PersistenceLocality` aligns
with `ComputeArtifactLocality` — one placement-with-eviction kernel, two views.

**Do not claim** unified `CacheFabric<T>` or minted `Realization<Spec, Effect>` kernel
types are built. Both are TARGET names under the same **≥2 cross-layer inhabitant** restraint
(resolve #1, sccache #2, provisioning #3). Extract shared kernel only when inhabitants
prove the identity+receipt shape carries — PR2 is the first trigger to watch.

---

## 5. Enforcement mechanism — the cache as purity oracle

§4.3 **(b) / hermetic realization** in operational detail:

1. **Content identity (§4.3 (c))** — memoized unit **declares** its input content-set;
   **content-key** = `content-hash(f-identity, input-content-hashes)` over the transitive
   closure of those inputs only — realized as `ContentHash` / `ExecutionReceiptRef` digest.
2. **Hermetic realization (§4.3 (b))** — read **outside** declared set = **impurity**
   (ambient intern table, mutable env, unmodeled host state). Discharge: **(a)** effect
   discipline (`WorkDemand.effects`) makes read impossible; **(b)** key misses hidden input
   → cached-vs-cold divergence → falsifier fires.
3. **Receipt + reconcile (§4.3 (a))** — two halves, distinct authorities:
   - **Persistence** — `key → output` via `ExecutionReceipt<T>` + `cache_interface` store/lookup
     (compute_fabric / cache_interface lane; snappy substrate co-sign).
   - **Change-consequence** — on content-key change, `RecomputePlan` / `AffectedSet`
     (`v2.std.change`; v2-owner confirmed) names **what dependencies/layers must rerun** —
     not `key → output` persistence. Gap selection is the TARGET composition
     `(ExecutionReceipt persistence) × (AffectedSet/RecomputePlan rerun-scope)` (§6).

**Finding B — oracle splits on reversibility:** the byte-identical cached-vs-cold falsifier
(reversible handlers — compute, build/sccache) **is** the enforcement for closed-world
realization. Irreversible handlers (provisioning, ARC inhabitant #3) need durable receipt +
idempotency key instead of re-execution oracle alone.

**The equivalence falsifier IS the enforcement** for reversible handlers — not a separate
mechanism. PR1 (#4867) is the canonical receipt on the compute handler.

---

## 6. Receipt + change-driven reconcile vs spurious recompute

**Not every recompute is the error class.** When the content key **changes** — source edit,
dependency change, invalidation trigger — re-executing dependent work is **correct**
receipt + change-driven reconcile: realize only the gap. **`RecomputePlan` / `AffectedSet`**
(`v2.std.change`; v2-owner confirmed) model **change-consequence** — what dependencies/layers
must rerun after change — **not** `key → output` persistence (that is `ExecutionReceipt` +
`cache_interface`; §5.3). That is incremental execution done right.

*Footnote:* `v2.lens.affected_set` projects `AffectedSet` → `RerunNodeSet` for CI witness
selection (`affected_set_selection.dag`) — same authority family, different projection.

**Spurious recompute** is the error: same content key, full re-derivation anyway — or a
cache hit that diverges from cold oracle because purity was violated. Caching does not mean
never recompute; it means **never recompute without a content-key change** (unless the
change-driven plan says so).

---

## 7. Migration path — target inventory, facts-first dissolution

Each §3.3 roster row dissolves only when a **real provider grounds** it: add
`cache_interface` row + delete hand-roll. Falsifier: hand-roll count → 0; cached-vs-cold
green.

**Three maturity states** per row:

| State | Meaning |
| --- | --- |
| **(a) Structurally grounded** | `cache_interface` row + dim inhabited (staged) |
| **(b) Realized in host** | Rust enforces + behavioral falsifier green |
| **(c) Realized in substrate** | `compute_fabric` evaluates cache verdict on v2 (gated) |

| # | Cache | Dissolve move (when grounded) | Maturity today |
| --- | --- | --- | --- |
| 1–4 | eval memos / interner | Content-keyed eval boundary; delete `HashMap` stores | — / partial (b) / — |
| 5 | `typed_module_cache` | PR2 resolved-graph row + delete hand-roll | (a) partial / **(b) PR1** / (c) PR2 |
| 6–10 | test/bootstrap/manifest artifacts | Artifact kind + parity receipt pattern | varies — see marks |
| 11 | sccache | **ARC inhabitant #2** — `sccache_local_facts` + CI projection; delete `ci.yml` hand-wire; closed-world + reversible de-risking rung | **(a)** / (b) interim / (c) gated |

PR2 = **(a)+(b)** for row 5 (ARC #1); row 11 = ARC #2. **ARC inhabitant #3** (open-world +
irreversible provisioning) is not a §3.3 hand-roll — it is the stress test where Finding A
is met; see [ctrl/ROADMAP.md ARC §3](https://github.com/gunb-ai/ctrl/blob/main/ROADMAP.md#cross-cutting-requirement--the-realization-pattern).

---

## 8. Non-goals

- Not a parallel ledger (marks win; §3 is one roster, not a second census).
- Not claiming automatic enforcement, `CacheFabric<T>`, minted `Realization<Spec, Effect>`
  kernel types, or full collapse before inhabitation.
- Not citing consumed scaffolding worksheets from early `.dag` headers.
- Not expanding eleven → thirteen (v2 footnotes only).
- Not benchmark-driven proof.

---

## 9. Falsification probes

| ID | Probe | Receipt |
| --- | --- | --- |
| **F1** | Cached-vs-cold byte-identical on fixed fixtures | PR1 pattern; #4171 artifact parity |
| **F2** | Cache keys avoid mutable context ids when meaning changes | #4282-class span-key law (infer — distinct from PR1) |
| **F3** | Order-permutation independence for shared-index caches | `resolve_typed_cache_equivalence_test` |
| **F4** | Host transport projects modeled facts, does not recompute | No second detector / duplicate timing authority |
| **F5** | Substrate acyclic import law preserved | `handwritten_parser_accepts_compute_fabric_dag` / `cache_interface_dag` |
| **F6** | Hand-roll count trends down per §3.3 row as interface inhabits | Diff review; `HashMap`/`OnceLock` stores deleted |

---

## 10. One-line summary

**Recompute pure-of-content** is what happens when there is **no reified execution receipt**
at the model→host boundary: redundancy violates Performance / Facts-Flow-Forward; silent
impurity violates Fail-Closed (PR1 #4867). v2 recurs the pattern because the carrier is
staged-not-inhabited, not because v2 was sloppy. **Transparent memoization** = TARGET
`Realization<Spec, Effect>` ([ctrl/ROADMAP.md](https://github.com/gunb-ai/ctrl/blob/main/ROADMAP.md#cross-cutting-requirement--the-realization-pattern);
staged in `cache_identity` / `cache_interface` / `compute_fabric`; TARGET carrier name only
per §4.4 staging guard) is the receipt carrier — ARC #1 resolve (#4878), #2 sccache,
#3 provisioning.
