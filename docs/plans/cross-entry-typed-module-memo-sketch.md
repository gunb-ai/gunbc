# A — cross-entry typed-module memo: content-key + host-denominated eviction (SKETCH FOR SIGN-OFF)

> **Status:** DESIGN SKETCH, 2026-07-15, session sleek-ram-450. **Not yet code.** Sign-off gate before any edit to retention machinery (04_infer/04_env are load-bearing; touching retention while retention is the live pathology is the model-before-implement class). This doc is put in front of the operator/lively-heron for the four decisions below; implementation follows sign-off.
>
> **Not a new authority (§3).** This is the **content-key + load-bearing-eviction refinement** of [cross-worker-typecheck-share-design](cross-worker-typecheck-share-design.md) increment C, keyed on the [duplicate-work](duplicate-work-graph-lens-design.md) `ComputationIdentity` lattice and realized as one `CacheProvider` row on the existing `dag/std/materialization_ladder.dag`. Every type it names already exists. It forks nothing.
>
> Parent lanes: [cross-worker-typecheck-share-design](cross-worker-typecheck-share-design.md) (increment C — name-keyed, in-process, eviction left as an honest trade), [duplicate-work-graph-lens-design](duplicate-work-graph-lens-design.md) (`ComputationIdentity` + `CacheProvider`/`EvictionClass`), [v1-run-stability-throughline](v1-run-stability-throughline.md) (the memory axis; term-3 owner).

---

## 0. Why this exists — the three-term decomposition, and why A now carries a hard constraint

The v1 resolve superlinearity is closed as three terms (memory note `v1-resolve-superlinearity-three-term-decomposition`):

1. **Fixed per-module hotspots** — big `HostEffect`-coproduct modules (`host_effect_realize` 10.4s, `host_identity_assimilation` 16.8s, `ci_spec` 8.3s) re-typechecked **once per containing closure** (closure-independent cost; the redundancy is the *repetition across entries/workers*, not any single typecheck). **This is A's displaced cost.**
2. Mild inherent lookup growth → namespace/SymbolIndex lane, not A.
3. **Retention aging** (lively-heron, from the Pi log) — same-size entries run **~2.9× slower in the last third** of the corpus run than the first; memory pressure on the ~6.25 GB retained store, not structural growth (corroborated: 130min wall vs 56min CPU ≈ swap stall).

**Denomination correction (2026-07-15, after reading the live store path).** Term-1's *cross-entry* redundancy is **already collapsed within a process**: `run_discovery_corpus` builds **one `process_shared_index`** whose `typed_module_cache` persists across every entry-group — at width=1 the inline drain reuses it (`cli_run.rs` ~8863), at width>1 the `cross_worker_store` shares it across workers. So `host_effect_realize` is already typechecked **once per process**, not once per entry. **A's remaining displaced cost is therefore term-3 (retention aging), not a term-1 wall win** (that is banked). The store is **name-keyed and unevictable — it grows to the whole union**, which *is* term-3; and that unbounded growth is precisely why the cross-worker store is **withheld at width=1** ("serde retention without cross-worker benefit breaks the CI budget", `cli_run.rs` ~8895). **Eviction is the deliverable, and it is what lets the store be armed at any width on a constrained host.** Content-keying is the *soundness license* for eviction (drop a pure fact, recompute by content key on re-demand, purity-oracle-proven no stale serve) and the S2b-ready backend — not itself a wall win.

So A wins only as the materialization-ladder instance — content-keyed with declared eviction — never a hand-rolled `HashMap` that cannot evict (ROADMAP ④'s "N hand-rolled caches" disease; a memo that can't evict is a term-3 regression).

**§3 reconciliation with M2 (throughline) — A coordinates, does not fork.** [v1-run-stability-throughline](v1-run-stability-throughline.md) **M2** ("retention strip at the cache grain") is *also* a term-3 mechanism, and it deliberately rejects LRU: "recompute reds the once-per-node contract." The two are **complementary, one law two grains** — and A must build them as such, not as a second eviction:

- **M2 env-strip = primary, zero recompute.** Once every importer of a module in the roster's remaining demand is typechecked, strip its `type_env`/`func_env` to shared empty singletons (keep `module`/`items`/`item_registry`). The stripped fields are provably no longer demanded, so this frees bytes **without recompute** — it respects once-per-node. This is the first line of defense and shrinks *per-entry* footprint.
- **A `SpacePacked` whole-entry eviction = backstop, counted recompute.** When strip is insufficient to hold the store inside the host budget, drop least-recently-demanded whole `ComputationIdentity` entries; a re-demand recomputes by content key. This *does* incur recompute (a counted, located deviation from once-per-node — **priced in §4**, not silent), and it is the *hard* memory bound strip cannot guarantee.

Together: `store bytes ≤ min(host-budget, N × stripped-entry)`. Eviction count is a §5-counted diagnostic, so the once-per-node deviation is observable and prioritizable — never absorbed. **A subsumes M2's strip as its no-recompute inner ring; A owns only the budget-enforcing outer ring.** (Coordination: M2 is a throughline milestone; whether A absorbs M2's strip or consumes it is an operator lane call — flagged, §6.)

The four sections below are the four decisions that make A retention-safe. They are not free choices.

---

## 1. The key is more than the module's content hash

**Decision: `ComputationIdentity` for a typed module = `hash(module source) ⊕ hash(each direct-import `ModuleInterface`) ⊕ compiler-identity`.** The store keys on this, not on module name (increment C's S2a name key) and not on module source alone.

Why each term is *necessary* (drop one → a stale hit → §5 fail-open):

| Key term | Why it must be in the key | If omitted |
|---|---|---|
| **module source hash** | typecheck reads the module's own text | edit body → stale typed module served |
| **direct-import `ModuleInterface` hashes** | typecheck consumes imports *at interface grain* — Inc B already narrowed cross-boundary consumption to `interface.env`/`interface.cache`, so the interface hash is the exact sensitivity surface | change an imported type → downstream module keeps its old inference |
| **compiler identity** | the typechecker itself is an input | seed regen changes inference → stale serve across a rebuild |

Why this set is **complete** (the minimal-vs-conservative choice, and why minimal is right here):

- **Minimal = direct-import interfaces** (not the transitive env fingerprint). Because Inc B made consumption interface-grain and interface-transitive (`transitive_interface_binding_test`): a module's typed result is a pure function of its source and its *direct* imports' interfaces; a transitive change reaches this module *only through* a direct import's interface hash changing. So direct-import interfaces already close over the transitive cone — the conservative transitive-env fingerprint would be a strictly larger key computing the same partition (more invalidation, no more safety). **Recommend minimal.**
- **Completeness is not argued, it is *witnessed*.** The receipt is §3's byte-identical cached-vs-cold oracle: if the key were incomplete, some input outside the key could change the typed result, and a memo-hit run would diverge in `corpus_fingerprint`/`emit_graph_fingerprint` from a cold run. Byte-identical across the whole corpus **is** the completeness proof (§5 purity oracle — key on declared-input content; byte-identical cached-vs-cold is the purity oracle). If the oracle ever reds, the key is provably missing a term — and the divergence *locates which module*, so the fix is a key term, never a widen.

This is exactly the `IdentityUnknown` drain discipline (duplicate-work §6): MVP lands `StructurallyIdentical` over this composed key; a hash we cannot yet compute (should any import interface not be content-addressable) is a typed `IdentityUnknown{MissingEnforcement}`, refused, never a name-key fallback.

---

## 2. Eviction needs a budget denominated per-host, not per-corpus

**Decision: the provider's `EvictionClass` is `SpacePacked { budget }`, where `budget` is read from the host memory envelope the governor already reads — not an authored constant, not corpus-denominated.**

The structural tell that this is mandatory, not optional: the typed-module store's *current* lifetime is ⊤ (lives to process exit). Run it through the existing catalog→ladder projection (`provider_from_catalog`, `dag/extdeps/cache/materialization.dag`): an in-memory store with `Never`/unbounded growth **outside a scope that exits** maps to **`ProjectionRefused`** — "a store with unbounded or human-gated growth cannot satisfy rule 3, and the refusal points at the catalog row that needs a declared budget." **Term-3 retention aging is that refusal, observed at runtime.** The ladder already says the fix is a declared budget; the operator's steer says *how to denominate it*.

- **Interim (this lane): declared budget derived from the host budget read.** The governor already reads the host memory envelope (`memory.max` / container cap — the same source that packs floor worker width). The eviction budget is a *fraction of that read*, so it scales with the host: a 31 GB container and a 6 GB Pi get different budgets from the same expression, without a per-site constant. Evictions are **counted** (a typed, located, per-eviction diagnostic — never a silent shrink; the absorbing-fallback wall, §5). `SpacePacked{budget}` carries the derivation string, not a literal.
- **Terminal: measured space.** When the realization-measurement lane lands (witness-plan P1, [realization-measurement-loop](realization-measurement-loop.md)), the budget is derived from *measured* per-entry store bytes rather than a declared fraction. Same `SpacePacked` cell; the `budget` derivation swaps declared→measured. Declared-first is the honest interim (loud, bounded, with its dissolution trigger = P1), not an authored pin.

**Eviction is two rings (the M2 reconciliation, §0):**

1. **Inner ring — env-strip (M2), zero recompute.** Strip `type_env`/`func_env` from an entry once its importers in the roster's remaining demand are all typechecked. Frees bytes without recompute → respects once-per-node → *not* counted against the once-per-node contract. This is the primary bytes-freeing mechanism.
2. **Outer ring — `SpacePacked` whole-entry eviction, counted recompute.** Only when the stripped store still exceeds the host budget: evict least-recently-demanded whole `ComputationIdentity` entries; re-demand recomputes by content key. Dropping a pure typed-module fact is *always sound* (ladder rule-3: "dropping pure facts is always sound") — eviction can never produce a wrong answer, only a slower one. Each eviction is a **counted, located diagnostic** so the once-per-node deviation is observable and priced (§4), never absorbed.

`SpacePacked{budget}` is the outer ring's declared bound; the inner ring is `ScopeExit`-shaped (strip = the demand-span exit for those fields). Both are the ladder's own vocabulary.

Refusal, never widen: if the budget read fails (host envelope unavailable), the provider **refuses** (typed, located) — it does not fall back to an unbounded store. An unevictable store is the term-3 regression; refusing to build one is the fail-closed floor.

---

## 3. The purity-oracle receipt plan

**Decision: the completeness/soundness receipt is a cold run vs a memo-hit run with byte-identical resolve fingerprints — reusing the oracle that already exists.**

`measure_whole_tree_resolve` already emits `corpus_fingerprint` and `emit_graph_fingerprint` over the whole corpus (`src/v1/stage0/src/bin/measure_whole_tree_resolve.rs:110-112`). The receipt:

1. **Cold run** — store empty, every module typechecked, record both fingerprints.
2. **Memo-hit run** — same corpus, store warm (or shared across entries), record both fingerprints.
3. **Assert byte-identical.** Any divergence = the key (§1) is incomplete OR eviction served a stale entry (§2) — both fail-closed reds, both *located* to the diverging module.

RED controls (the oracle must be able to go red, or it's inert — §5 discrimination gate):
- **Planted stale key** — drop the compiler-identity term (or a direct-import interface term) from the key, mutate that input → memo-hit fingerprint must diverge from cold. Proves the key terms are load-bearing.
- **Planted stale eviction** — serve an evicted-then-mutated identity without re-derivation → divergence. Proves eviction re-derives, never resurrects stale.

This is the same warm==cold purity oracle the catalog already witnesses (`extdeps/realization/cache_purity.dag`, `audit_warm_equals_cold`) — the store-tier discharge reuses it, does not re-mint it. A `MemoTier{ContentKeyed}` discharge without a warm==cold receipt reds by construction (duplicate-work forward-wall 3).

---

## 4. Acceptance on the adversarial host (the Pi)

**Decision: A is accepted only when, on the Pi (the host where term-3 lives), both hold — alongside the §3 fingerprint receipts:**

- **(b) The late/early aging ratio falls from ~2.9× toward ~1× — the primary prize.** Bounding the store (§2) removes the memory pressure that ages the run. Measured by the same per-third timing lively-heron used on the Pi log.
- **(a) Total resolve wall drops materially — as a *consequence* of (b), not of term-1.** Term-1's cross-entry redundancy is already banked within-process (§0); (a)'s wall drop is the **swap-stall relief** the bounded store buys (the 130min-wall vs 56min-CPU gap closes). Measured by total resolve wall + resolve-split `typecheck_compute`; fingerprints unchanged.

**The failure signature that makes both non-negotiable:** eviction (§2 outer ring) incurs recompute, so an over-tight budget *raises* wall (recompute cost) even as it lowers peak bytes — the counted eviction diagnostic is the tell. If (b) improves but (a) *worsens*, the budget is too tight (over-evicting → recompute dominates); if (a) improves but (b) is flat, the budget is too loose (no real bound). Both move together only when the budget is tuned to the host — that is the acceptance. Neither alone; the budget (§2) is re-derived until both hold.

The Pi is the RED control for the whole lane: a name-keyed unevictable store (increment C as-is, withheld at width=1 today) grows to the union and ages 2.9× on the Pi — which is *why* the plain increment C is not armed on constrained hosts, and why this refinement exists.

---

## 5. What lands, in order (post-sign-off)

Nothing here touches 04_infer/04_env until sign-off. Then, staged smallest-first, each stage priced by the receipt it turns green:

1. **Key (§1) as a pure function** — `computation_identity(module) = hash(source) ⊕ direct-import interface hashes ⊕ compiler-id`, unit-tested, no store yet. (Load-bearing files untouched; this is a new pure fn.) **LANDED — #6740** (`std.interface_summary.typed_module_key`).
2. **`CacheProvider` row** on `materialization_ladder.dag` via `provider_from_catalog` — `MemoTier{ContentKeyed}`, `SpacePacked{budget = host-envelope fraction}`, coverage = the typed-module identities. ~~This is the *declaration*; it reds until §3's warm==cold receipt is live.~~ **Re-staged under the no-demo-staged-landings ruling (operator, 2026-07-16, machine-shape ruling 11): a declaration may not land ahead of its real consumer, so the catalog-facts flip lands IN the store PR (PR-α below) and the `SpacePacked` provider projection lands with the eviction PR (PR-β).**
3. **Store re-key (PR-α)** — the live typed store (`MultiEntryIndex.typed_module_cache` + the width>1 `SharedTypecheckCaches`) re-keyed name→content key via the stage-1 fn (source hash recorded in the parse loop; import interface hashes = the Inc-B `ModuleInterface.summary.interface_hash`, chained in dependency order; compiler identity = `resolved_graph_cache::transform_content_digest`, the same authority the resolved-graph subject digest consumes). Catalog facts flip `HandAuthored→NativeInternal{[UpsertSubject, ToolchainSpec]}` in the same PR. Key-term RED controls by execution (`typed_module_content_key_tests` in cli_run.rs); warm==cold stays owned by `resolve_typed_cache_equivalence_test`. No eviction yet — the store is still `Never`-evicting and honestly cataloged as such.
4. **Eviction (PR-β)** — the two rings (§0: M2 strip inner, `SpacePacked{host-budget}` outer), counted evictions, interface-summary payload (§6 decision 4), catalog eviction flip, `CacheProvider` projection + §3 oracle wired as the discharge receipt.
5. **§4 Pi acceptance** — cold/warm fingerprints + resolve-split wall + aging ratio, on the adversarial host.

**Dissolution trigger:** when the realization-measurement lane (P1) lands, the `SpacePacked` budget derivation swaps declared→measured (§2 terminal); when S2b cross-run persistence lands, this in-process content-keyed store becomes the backend S2b reads through (increment C §10) — same store, re-scoped, never re-invented.

---

## 6. What the store-path read changed — the one decision that now gates code

Reading the live store path (`cli_run.rs` `run_discovery_corpus`) surfaced a load-bearing fact the sketch sign-off predates, and it re-denominates A. **This is the model-before-implement payoff: surfaced before any store code.**

**Finding 1 — term-1 wall is already banked** (§0): one `process_shared_index` per process shares the typed cache across all entries; `host_effect_realize` is typechecked once per process, not per entry. A's win is term-3 (retention), not term-1 wall.

**Finding 2 — naive whole-row eviction may free almost nothing on the target host.** Throughline M2 already found: "**not** whole-row eviction (Rc-pinned payloads make that free almost nothing)" and "**not** an LRU (recompute reds the once-per-node contract)." At width=1 (the constrained Pi) the store is `HashMap<name, Rc<TypecheckModuleResult>>` — the payload Rc may be pinned by the live resolved graph, so dropping the map slot frees little. And retention is **Node-dominated** (memory note `v1-resolve-retention-is-node-dominated`): the mass is the typed *body*'s Nodes (`module`/`items`), which **both** M2 env-strip **and** whole-row LRU leave pinned. The mechanism that provably frees Node mass is **M3 / Inc-B interface-summary retention** — store only `ModuleInterface`, drop the typed body.

**The re-denominated A (recommendation):** A's content-keyed host-budget-bounded store is right, **but the payload it retains must be the interface summary (M3/Inc-B shape), not the full `TypecheckModuleResult`.** Content-keying makes recompute-on-miss sound; the host budget bounds it; **retaining interface-summaries (not Node-heavy bodies) is what makes eviction actually free the dominant mass.** So A = increment-C store ⊕ M3 interface retention ⊕ host-budget `SpacePacked` ⊕ content key. The `SpacePacked` outer ring then bites on a store whose entries are *small* (interfaces), where dropping them frees real bytes; the width>1 serde store (`Arc<Vec<u8>>` snapshots) frees bytes on eviction regardless.

### The decisions this now asks the operator (code is gated on 4)

1. **Key set (§1):** minimal (source ⊕ direct-import interfaces ⊕ compiler-id), completeness witnessed by the byte-identical oracle? *(recommend minimal.)*
2. **Budget denomination (§2):** interim = declared fraction of the host memory-envelope read, terminal = measured (P1); evictions counted; refuse-on-unavailable? *(recommend yes.)*
3. **Acceptance (§4):** both (a) wall-down (swap-stall relief) and (b) aging-ratio-toward-1× on the Pi; over-tight budget shows as wall-up from counted recompute? *(recommend yes.)*
4. **The re-denomination (this section) — the gating call:** A retains **interface summaries** (M3/Inc-B shape) under the host budget, *not* full typed bodies — because naive whole-row eviction of Rc-pinned Node-heavy bodies frees ~nothing at width=1 (M2's documented finding). This **couples A to M3/Inc-B (resolver-graph-major lane)**. Does A absorb the M3 interface-retention shape into its store, or consume M3 as a prerequisite from that lane? *(recommend: A absorbs the interface-retention shape into the store payload — it is the same `ModuleInterface` authority Inc B already projects; coordinate landing with the Inc-B owner.)*
