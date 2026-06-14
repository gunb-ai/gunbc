# [DRAFT] Recompute pure-of-content — one fundamental error, transparent memoization

Work item: `node://adhoc-4a3d8313-94c` (sleek-bee-765) · CI-investigation tree.

**Status: DRAFT — for operator framing review.** Not done. Structure and verified content
first; prose polish second. Awaiting operator confirmation of the **spine** (§2 thesis +
two-faces framing) before closeout. Centerpiece (§1) and instance catalog (§§3–4, §7) are
evidence sections — thesis reframe should not require rewriting them.

**Map / motivation only** — points at inline marks as durable authority; not a parallel
ledger for per-cache facts (operator standing principle, 2026-05-19).

**Durable solution home:** `dsl/std/{cache_identity,cache_interface,compute_fabric}.dag`.

---

## 1. Centerpiece — resolve-cost PR1 revealed impurity, did not cause it

When resolve-cost PR1 (#4867) added a per-module typed cache on `MultiEntryIndex`, claim
verdicts **flipped depending on entry order**. The cache did not introduce a new bug — it
made a **pre-existing content-impurity** observable.

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

**Moral:** A cache that changes a verdict is not a cache bug — it is a **purity bug the
cache exposed**. Making a computation content-addressable *requires* it be pure-of-content;
without enforced purity, manual caching is both necessary (redundancy face) and dangerous
(correctness face).

Model authority for the type-time interning shape:
`src/v2/04_infer.dag:5453-5460` (`build_type_env` folds `kernel_type_set` into the table).

---

## 2. Thesis — one root, two faces *(spine; subject to operator refinement)*

> **Modular thesis block.** If the operator sharpens the root articulation, revise this
> section (and §10 summary) only. §1 centerpiece, §§3–4 instances, §§5–7 substrate/migration
> evidence stay.

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

---

## 3. Structural redundancy instances (qualitative)

These illustrate the error class structurally. They are not benchmark arguments.

### 3.1 `call_function` re-derives static parameter metadata every call

On **every** invocation, `call_function` filters `fn_node.params`, runs
`authored_name_at` substring extraction per parameter, allocates a `Vec` and `HashMap`, and
re-derives which parameters are value params — work that is a **pure function of `fn_node`
shape** (static for the function's lifetime).

```1282:1293:src/v2/stage0/src/v2_interpreter.rs
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

Surfaced by the #4865 interpreter profiler context (`v2_interpreter.rs:4187-4199`):
witness eval routes through `eval_expr` for cost/complexity lens folds; redundant per-call
setup amplifies tree-walk cost.

### 3.2 Resolve re-walks overlapping module closure

Before PR1, each entry re-ran tokenize → parse → resolve → reconcile over heavily
overlapping module closures. PR1's `typed_module_cache` on `MultiEntryIndex` is a
**hand-rolled** compute-once-store for the shared core (`cli_run.rs:273-303`,
`reconcile_with_typed_cache` at `:539+`). PR2 (swift-lark-563, in flight) extends the lever
cross-process — first real consumer grounding `cache_interface` dims.

### 3.3 Cost/complexity lens folds re-traverse the AST

The cost and complexity lenses fold symbolically over the AST during witness evaluation with
no shared memo across nodes — full re-traversal per lens application (`src/v2/complexity.dag`
structural `fold` chains; eval cost acknowledged `v2_interpreter.rs:4187-4188`).

### 3.4 Host transport that re-derives modeled facts (brief)

When host code recomputes facts already owned by `.dag` authority — a second affected-set
detector, timing math duplicated outside the single ledger function — that is the same
error class at the CI boundary: duplicate derivation instead of projection. The fix direction
is always: **one authority produces the fact; transport projects it**.

---

## 4. The eleven hand-rolled caches — same pattern, eleven localities

The sunny-lynx census (11 inhabitants) is facts-before-abstraction satisfied: eleven real
compute-once-store re-implementations warrant abstracting ONE interface — not premature
generality. Each row below cites where the hand-roll lives; **per-cache dimension facts
remain in those marks** — this section names the pattern only.

| # | Hand-roll | What it caches (structurally) | Authority cite |
| --- | --- | --- | --- |
| 1 | `pure_call_memo` | Structural pure fn results keyed by fn identity + arg sharing | `v2_interpreter.rs:1022, 2180-2250` |
| 2 | `data_cache` | Nullary / data CAF values by node identity | `v2_interpreter.rs:1019, 1516-1520` |
| 3 | `parse_cache` | File path → parse result + newline index | `cli_run.rs:280-282` |
| 4 | `SymbolInterner` | String → `Symbol` dedup (identity table) | `v2_interpreter.rs:108-118, 1028` |
| 5 | `typed_module_cache` | Module name → typecheck result (resolve-cost PR1) | `cli_run.rs:287-303, 539+` |
| 6 | `COMPILE_CACHE` | Per-(source, file) `compile_to_dag` outcome | `cached_compile.rs:63-70` |
| 7 | Bootstrap snapshots | Committed `Dag` snapshot reuse | `lib.rs` (`std_fixture_bootstrap_snapshot`); `pb1_bootstrap_full_snapshot_test.rs` |
| 8 | `.claim-map` | Claim corpus execution map artifact | `scripts/v4-claim-corpus-execution-map.sh:28`; `.gitignore:101` |
| 9 | Discovered-owned-data manifest | Host scan → manifest DAG | `discover_owned_data.rs:8-12`; `host_discovered_owned_data_manifest.dag:2-3` |
| 10 | `.freshness-check` | Stage0 regen verify temp tree | `dsl/gunbc/tools/freshness.dag:68-89`; `Makefile:56` |
| 11 | sccache (interim CI) | Rustc invocation → object file | `.github/workflows/ci.yml:70-91`; modeled row `cache_interface.dag:280-321` |

**Footnote — same class in newer substrate (not roster expansion):** v4 `ParseTable`
(`02_parse.dag:155-161`) and `TestClaimCacheKey` (`05_eval.dag:526+`) show the pattern
recurring even where the compiler pipeline is being rebuilt — evidence the root issue is
fundamental, not v2-specific.

---

## 5. Systemic fix — transparent memoization (TARGET, staged)

**Transparent memoization** = first-class pure-content-addressed computation: a memo or
cache layer keyed on declared content identity such that hits are observationally equivalent
to full re-execution, inserted by the compiler/runner when purity + content key are known —
not part of the user-facing contract.

### 5.1 What is staged vs realized today

| Layer | Today | Notes |
| --- | --- | --- |
| **Substrate types** | Staged, self-authoritative | `cache_identity.dag`, `cache_interface.dag`, `compute_fabric.dag` — marks are authority; do not cite consumed scaffolding worksheets |
| **Structure on v2** | Partially resolves | Type shapes load; full claim-**run** (evaluation) not wired end-to-end |
| **End-to-end eval on self-hosted compiler** | Not yet wired | Three resolve gaps remain (each has a worker in flight): optional fields don't resolve on v2 yet; single-case catalog tags (one-of-one enums) don't resolve on v2; reading values through a chain of type aliases (money/Measure carrier) doesn't resolve on v2 |
| **PR1 (#4867)** | Manual discharge | `seed_kernel_intern_names` + equivalence falsifier — not automatic enforcement |
| **PR2 (swift-lark-563)** | First real consumer | Structural grounding + Rust-side behavioral proof; compute_fabric-eval witness honest-dormant until (c) below |

**Do not read this doc as "the fix already works."** The three TARGET properties below are
what the substrate is built **toward**.

### 5.2 Three TARGET properties (not claimed working)

| Property | Target meaning | Today |
| --- | --- | --- |
| **(a) Automatic memo** | Compiler/runner inserts memo at purity+cost boundaries; authors don't hand-roll | Eleven hand-rolls; v2/v4 local memos |
| **(b) Enforced purity at boundary** | Non-pure computation cannot be soundly memoized; impurities surface as errors or falsifier failures | Manual seeds + standing gates (PR1); purity oracle mechanism below |
| **(c) Content-keyed invalidation** | Minimal invalidation via Merkle/content hashes, not heuristic file deps | Partially modeled in `CacheInterfaceFacts` rows; host transport still interim for sccache |

### 5.3 Substrate motivation (do not re-architect)

`compute_fabric` and `cache_interface` form a **symmetric pair** with a bidirectional
firewall (each module's header marks forbid importing the other; they meet at a digest seam
only):

- **Compute side:** demand, lease, `ExecutionReceipt<T>`; `ToolchainCapability.linked_cache_interface`
- **Cache side:** store semantics per backend; `ExecutionReceiptRef<T>` as digest-only producer witness

`ExecutionReceipt` **is** a cache entry on the compute timeline; `PersistenceLocality` on
the cache side aligns with `ComputeArtifactLocality` on the compute side — one
placement-with-eviction kernel viewed from two angles.

**Do not claim** a unified `CacheFabric<T>` is built. Per modeling discipline: extract the
shared kernel only when ≥2 inhabitants provably share content-key / eviction / locality
shape. PR2 grounding `cache_interface` as first consumer is the ≥2 trigger to watch.

---

## 6. Enforcement mechanism — the cache as purity oracle

Assertion is not enough. The enforcement mechanism is:

1. A memoized unit **declares** its input content-set (the facts that may influence the
   result).
2. The **content-key** is derived from exactly those declared inputs:
   `content-hash(f-identity, input-content-hashes)`.
3. Any read **outside** the declared set — ambient intern table size, mutable env generation,
   host state without modeled invalidation — is an **impurity**.
4. Discharge paths:
   - **(a) Impossible:** effect/capability discipline prevents the ambient read at the
     authoring boundary.
   - **(b) Caught:** the key fails to capture the hidden input → **cached-vs-cold
     divergence** → the standing falsifier fires.

**Enforced purity at the boundary is not a separate mechanism from the falsifier — the
equivalence falsifier IS the enforcement.** A cached-vs-cold divergence means some
determinant of the output is not in the content key: a hidden input = an impurity. Wire
this gate continuously (not only when someone remembers to add a test).

PR1 is the canonical instance: hidden intern-table-size → order-dependent verdict →
`seed_kernel_intern_names` restores content-stable ids → byte-identical oracle restored.

Open design work remains where the substrate does not yet close effect-boundary checking
for every ambient read — but the mechanism is named: **declare inputs, derive key, falsify
equivalence**.

---

## 7. Migration path — target inventory, facts-first dissolution

Each roster cache dissolves only when a **real provider grounds** it: add
`cache_interface` row (or link from `compute_fabric`) + delete the hand-rolled impl.
Falsifier for dissolution: hand-roll count trends to zero; cached-vs-cold gate stays green.

**Three maturity states** (per row — stops reading the table as "done"):

| State | Meaning |
| --- | --- |
| **(a) Structurally grounded** | `cache_interface` row + dim inhabited in substrate (staged) |
| **(b) Realized in host** | Rust host enforces + behavioral falsifier green (bootstrap seed today) |
| **(c) Realized in substrate** | `compute_fabric` evaluates cache verdict on v2 (end goal; gated on resolve gaps) |

| # | Cache | Dissolve move (when grounded) | Maturity today |
| --- | --- | --- | --- |
| 1 | `pure_call_memo` | Eval memo keyed by declared `EvalMemoKey` / content-hash subject; delete interpreter `HashMap` | — / — / — |
| 2 | `data_cache` | Same eval-cache boundary as (1) | — / — / — |
| 3 | `parse_cache` | Cache row for parse artifact + subject digest; delete `parse_cache` map | — / — / — |
| 4 | `SymbolInterner` | Identity intern as cache locality on string content-key | — / (b) partial / — |
| 5 | `typed_module_cache` | PR1 manual; PR2 cross-process resolved-graph row + delete hand-roll | (a) partial / **(b) PR1** / (c) PR2 |
| 6 | `COMPILE_CACHE` | `compile_to_dag` artifact kind + parity receipt (#4171 pattern) | — / (b) test-only / — |
| 7 | Bootstrap snapshots | Content-hash pinned bootstrap plan (T-15 dissolution marks) | (a) partial / (b) / — |
| 8 | `.claim-map` | Corpus execution receipt as cache artifact | — / — / — |
| 9 | Discovered-owned manifest | Producer fold authority (no parallel scanner) | — / (b) / — |
| 10 | `.freshness-check` | Freshness as cache miss on regen artifact | — / (b) / — |
| 11 | sccache | `sccache_local_facts` row + CI projection; delete `ci.yml` hand-wire | **(a) row exists** / (b) interim / (c) gated |

PR2 resolved-graph cache targets **(a)+(b)** for row 5; **(c)** remains honest-dormant
until compute_fabric evaluates end-to-end on v2.

---

## 8. Non-goals

- Not a parallel ledger for per-cache dimension facts (marks win; debate in PR review).
- Not claiming automatic enforcement, unified `CacheFabric<T>`, or collapse of all eleven
  caches before real inhabitation.
- Not reconstructing consumed scaffolding worksheets referenced by early `.dag` headers.
- Not expanding the canonical eleven into thirteen (v4 footnotes only).
- Not using benchmark numbers as proof (scale may appear once; structure is the argument).

---

## 9. Falsification — standing gates, not one-off tests

| Probe | Receipt |
| --- | --- |
| Cached-vs-cold byte-identical on fixed fixtures | Equivalence tests (PR1 pattern; #4171 parity for artifacts) |
| Order-permutation independence for shared-index caches | `resolve_typed_cache_equivalence_test` |
| Hand-roll count trends down per migration row | Diff review + deletion of cited `HashMap`/`OnceLock` stores |
| Host transport projects modeled facts | No second detector re-deriving frontier/timing |
| Substrate acyclic import law preserved | `handwritten_parser_accepts_compute_fabric_dag` / `cache_interface_dag` |

---

## 10. One-line summary

**Recompute pure-of-content** is duplicating work that declared content facts already
determine — a duplicate-representation violation with a scary correctness face when
hand-rolled caches hide impurity. **Transparent memoization** (staged in
`cache_identity` / `cache_interface` / `compute_fabric`) is the TARGET fix: purity oracle
falsifiers today, automatic content-keyed memo tomorrow; PR1/manual, PR2/first consumer,
substrate eval end goal.
