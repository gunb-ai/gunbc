# [DRAFT] Recompute pure-of-content — one fundamental error, transparent memoization

Work item: `node://adhoc-4a3d8313-94c` (sleek-bee-765) · CI-investigation tree.

**Status: DRAFT — for operator framing review.** Not done. Structure and verified content
first; prose polish second. Awaiting operator confirmation of the **spine** (§2 thesis +
two-faces framing) before closeout. §1 centerpiece and §3 instance roster are evidence
sections — thesis reframe should not require rewriting them.

**Map / motivation only** — points at inline marks as durable authority; not a parallel
ledger for per-cache facts (operator standing principle, 2026-05-19).

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
  every entry in an index (`src/v2/stage0/src/cli_run.rs:306-347`).
- `resolve_typed_cache_equivalence_test` is a **standing purity-oracle gate**: cached resolve
  must be byte-identical to the no-cache cold oracle in every entry order
  (`src/v2/tests/src/resolve_typed_cache_equivalence_test.rs:1-19`).

**First inhabitant on the substrate path:** PR1 is also the live first chapter of the
`cache_interface` story — resolve-cost PR2 (swift-lark-563, in flight) grounds the staged
dims as the first real consumer; PR1's manual seed + falsifier is what exists today.

Model authority for the type-time interning shape:
`src/v2/04_infer.dag:5453-5460` (`build_type_env` folds `kernel_type_set` into the table).

**Distinct from infer reconcile cache (#4282):** that lever memoizes lookup/reconcile with
span-only keys across `TypeEnv` updates — a different stage, different impurity class. PR1
is resolve-stage typed-graph cache + intern-id content-stability; do not collapse the two.

---

## 2. Thesis — one root, two faces *(spine; subject to operator refinement)*

> **Modular thesis block.** If the operator sharpens the root articulation, revise this
> section (and §10 summary) only. §1 centerpiece and §3 roster stay.

The language/compiler has **no first-class notion** that *a value is a pure function of its
inputs' content*. Because of that gap, every consumer that needs an already-computed result
either:

1. **Recomputes from scratch** — redundant work (perf face), or
2. **Hand-rolls a local cache** — necessary but **unsafe** when purity was never checked
   (correctness face).

These are not two bug classes. They are one root defect with two symptoms.

| Face | Symptom | Structural shape |
| --- | --- | --- |
| **Redundancy** | Pure-of-content work repeated when the content key is unchanged | Same content-determined value derived again instead of projected from a memo |
| **Un-enforced purity** | Hand-rolled cache returns wrong answers silently | Hidden input (ambient state) leaked into a result claimed to be content-keyed |

**Named error class:** *recompute pure-of-content* — executing or re-deriving work whose
result is already determined by declared **content facts** when memoization or cache lookup
on those facts would be observationally equivalent.

**Pure-of-content predicate** (the positive law the error violates):

- **Pure:** no side effects; same declared inputs ⇒ same outputs within the modeled strategy.
- **Of-content:** lookup identity is a **content fact** — structural form, `content_hash`,
  source span, declared input digests — not a mutable context id (intern-table slot without
  stability proof, env generation, host path without modeled invalidation, declaration name
  alone).

### Duplicate representation — enforcing an existing invariant, not inventing a rule

Per INVARIANTS **no duplicate representations**: maintaining the same content-determined
value in many recomputed or separately-cached forms is a **duplicate-representation
violation**. The redundancy face is not merely "slow" — it is the same fact represented
many times by recomputation. Transparent memoization enforces the invariant the thesis
already commits to: **one authoritative representation keyed by content**, projected
everywhere else.

**Sharpest statement (operator governing rule):** the sharing must be the **same code among
our own code**, not two abstractions that happen to agree. Dogfood the compute fabric by
dogfooding the compiler — one placement-with-eviction kernel, not eleven ad-hoc stores.

**This doc must not duplicate rosters.** §3 below is the **single standing instance roster**;
it subsumes the sunny-lynx eleven-cache census (internal work-item census = facts-before-
abstraction evidence for ONE interface). Lens by failure mode; one authority.

---

## 3. Instance roster — single catalog *(subsumes sunny-lynx eleven-cache census)*

§3 is the **only** instance catalog in this doc. It **subsumes** the sunny-lynx eleven-hand-
rolled-cache census: every census inhabitant appears in §3.2. §3.1 adds non-cache redundancy
instances; §3.3 adds parallel-derivation instances (not caches, same error class). Per-cache
dimension facts remain in source marks — this section cites them, does not mirror them.

### 3.1 Redundancy — pure-of-content work recomputed (no cache yet)

| Instance | Structural redundancy | Cite |
| --- | --- | --- |
| **`call_function` static param list** | Every call re-derives value-param names from `fn_node.params` via `authored_name_at`, `Vec` + `HashMap` alloc — pure function of static fn shape | `v2_interpreter.rs:1282-1317`; profiler context `:4187-4199` (#4865) |
| **Resolve overlapping closure** | Each entry re-ran tokenize→parse→resolve→reconcile over overlapping modules before shared index | `cli_run.rs:270-303`; cold oracle vs `resolve_entry_with_index` |
| **Cost/complexity lens AST folds** | Lenses fold over AST during witness eval with no shared memo | `complexity.dag`; `v2_interpreter.rs:4187-4188` |

### 3.2 Purity exposed by cache — resolve-cost PR1 (#4867) *(distinct instance)*

| Instance | What the cache revealed | Cite |
| --- | --- | --- |
| **Typed-module cache + latent intern-id impurity** | Module typed result claimed content-pure but depended on ambient intern-table **size** at `build_type_env` kernel interning; cache made order-dependent verdict flips visible; `seed_kernel_intern_names` + byte-identical falsifier restore content-stability | `cli_run.rs:306-347`; `v2_compiler_infer.rs:11328`; `resolve_typed_cache_equivalence_test.rs:1-19`; `04_infer.dag:5460` |

This row is §1 in table form — the strongest evidence for the un-enforced-purity face.

### 3.3 The eleven hand-rolled caches — sunny-lynx census *(THE migration roster)*

Facts-before-abstraction: eleven real compute-once-store re-implementations warrant ONE
interface. **This table is the sunny-lynx census** — do not maintain a second catalog
elsewhere.

| # | Hand-roll | What it caches (structurally) | Authority cite |
| --- | --- | --- | --- |
| 1 | `pure_call_memo` | Structural pure fn results by fn identity + arg sharing | `v2_interpreter.rs:1022, 2180-2250` |
| 2 | `data_cache` | Nullary / data CAF values by node identity | `v2_interpreter.rs:1019, 1516-1520` |
| 3 | `parse_cache` | File path → parse result + newline index | `cli_run.rs:280-282` |
| 4 | `SymbolInterner` | String → `Symbol` dedup (identity table) | `v2_interpreter.rs:108-118, 1028` |
| 5 | `typed_module_cache` | Module name → typecheck result (**PR1 #4867** — see §3.2 for purity story) | `cli_run.rs:287-303, 539+` |
| 6 | `COMPILE_CACHE` | Per-(source, file) `compile_to_dag` outcome | `cached_compile.rs:63-70` |
| 7 | Bootstrap snapshots | Committed `Dag` snapshot reuse | `lib.rs` (`std_fixture_bootstrap_snapshot`); `pb1_bootstrap_full_snapshot_test.rs` |
| 8 | `.claim-map` | Claim corpus execution map artifact | `scripts/v4-claim-corpus-execution-map.sh:28`; `.gitignore:101` |
| 9 | Discovered-owned-data manifest | Host scan → manifest DAG | `discover_owned_data.rs:8-12`; `host_discovered_owned_data_manifest.dag:2-3` |
| 10 | `.freshness-check` | Stage0 regen verify temp tree | `dsl/gunbc/tools/freshness.dag:68-89`; `Makefile:56` |
| 11 | sccache (interim CI) | Rustc invocation → object file | `.github/workflows/ci.yml:70-91`; `cache_interface.dag:280-321` |

**Footnote — same class in newer substrate (not roster expansion):** v4 `ParseTable`
(`02_parse.dag:155-161`) and `TestClaimCacheKey` (`05_eval.dag:526+`) — pattern recurs in
v4; not added to the canonical eleven.

### 3.4 Parallel re-derivation — duplicate authority (not a cache)

When host code recomputes facts already owned by `.dag` authority — a second affected-set
detector, timing math duplicated outside the single ledger function — that is the same
error class at the CI boundary. Fix direction: **one authority produces the fact; transport
projects it** (`tools/ci_affected_components/src/receipt.rs:11-13`).

---

## 4. Systemic fix — transparent memoization (TARGET, staged)

**Transparent memoization** = first-class pure-content-addressed computation: memo/cache keyed
on declared content identity, observationally equivalent to full re-execution, inserted when
purity + content key are known — not part of the user-facing contract.

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
| **PR2 (swift-lark-563)** | First real consumer | Structural grounding + Rust behavioral proof; substrate eval honest-dormant |

**Do not read this doc as "the fix already works."** TARGET properties below are what the
substrate is built **toward**.

### 4.3 Three TARGET properties (not claimed working)

| Property | Target meaning | Today |
| --- | --- | --- |
| **(a) Automatic memo** | Compiler/runner inserts memo; authors don't hand-roll | Eleven hand-rolls; v2/v4 local memos |
| **(b) Enforced purity at boundary** | Non-pure units cannot be soundly memoized | Manual seeds + standing gates (PR1); purity oracle (§5) |
| **(c) Content-keyed invalidation** | Merkle/content-hash invalidation, not heuristic deps | Partially in `CacheInterfaceFacts`; sccache still interim host |

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

**Do not claim** unified `CacheFabric<T>` is built. Extract shared kernel only when ≥2
inhabitants share content-key / eviction / locality shape — PR2 is the ≥2 trigger to watch.

---

## 5. Enforcement mechanism — the cache as purity oracle

1. Memoized unit **declares** its input content-set.
2. **Content-key** = `content-hash(f-identity, input-content-hashes)` from exactly those inputs.
3. Read **outside** declared set = **impurity** (ambient intern table, mutable env, unmodeled host state).
4. Discharge: **(a)** effect discipline makes read impossible; **(b)** key misses hidden input → cached-vs-cold divergence → falsifier fires.

**The equivalence falsifier IS the enforcement** — not a separate mechanism. PR1 (#4867) is
the canonical receipt.

---

## 6. Incremental re-execution vs spurious recompute

**Not every recompute is the error class.** When the content key **changes** — source edit,
dependency change, invalidation trigger — re-executing dependent work is **correct**.
`RecomputePlan` / `AffectedSet` (`v4.std.change`) model *what must rerun after change*; that
is incremental execution done right.

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
| 11 | sccache | `sccache_local_facts` + CI projection; delete `ci.yml` hand-wire | **(a)** / (b) interim / (c) gated |

PR2 = **(a)+(b)** for row 5; **(c)** honest-dormant until substrate eval lands.

---

## 8. Non-goals

- Not a parallel ledger (marks win; §3 is one roster, not a second census).
- Not claiming automatic enforcement, `CacheFabric<T>`, or full collapse before inhabitation.
- Not citing consumed scaffolding worksheets from early `.dag` headers.
- Not expanding eleven → thirteen (v4 footnotes only).
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

**Recompute pure-of-content** duplicates content-determined facts — a duplicate-representation
violation; hand-rolled caches hide impurity until a falsifier fires (PR1 #4867). **Transparent
memoization** (staged in `cache_identity` / `cache_interface` / `compute_fabric`) is the
TARGET: purity-oracle gates today; automatic content-keyed memo tomorrow.
