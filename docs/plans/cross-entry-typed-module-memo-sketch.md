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

Increment C already collapses term-1 redundancy *within a process across workers* — but it is **name-keyed** and it left eviction as an "honest trade" (increment C §7, open-decision-3), explicitly *increasing* co-resident retention. **Term-3 upgrades that open decision to a hard constraint:** a naive cross-entry/cross-worker memo *deepens* retention (term-3's failure axis) and is **net-negative on a constrained host**. So A wins only as the materialization-ladder instance — content-keyed with declared eviction — never a hand-rolled `HashMap` that cannot evict (ROADMAP ④'s "N hand-rolled caches" disease; a memo that can't evict is a term-3 regression wearing a term-1 win's clothes).

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

**Eviction grain = the `ComputationIdentity` entry.** When the store exceeds budget, evict least-recently-demanded identities. Dropping a pure typed-module fact is *always sound* — it only ever costs a recompute (ladder rule-3: "dropping pure facts is always sound"). That soundness is *why* `SpacePacked` is admissible here and why eviction can never produce a wrong answer, only a slower one — which the §4 aging metric then prices.

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

- **(a) Total resolve wall drops materially.** Term-1 redundant typecheck is discharged — the shared std/spec prefix + big coproduct modules are typechecked once per *process*, not once per entry/worker. Measured by resolve-split `typecheck_compute` nanos down (the #6535 instrument), fingerprints unchanged.
- **(b) The late/early aging ratio falls from ~2.9× toward ~1×.** The memo displaced recompute **without deepening retention** — proof that eviction (§2) held the store inside the host budget. Measured by the same per-third timing lively-heron used on the Pi log.

**The failure signature that makes (b) non-negotiable:** if (a) improves but (b) *worsens*, eviction is mis-tuned — A bought term-1 by paying term-3, exactly the net-negative outcome the constraint exists to prevent. (a)-alone is not acceptance; it is the trap. Both, or the budget (§2) is re-derived.

The Pi is the RED control for the whole lane: a name-keyed unevictable store (increment C as-is) would show (a) improve and (b) worsen on the Pi — which is *why* the plain increment C is not landed on constrained hosts, and why this refinement exists.

---

## 5. What lands, in order (post-sign-off)

Nothing here touches 04_infer/04_env until sign-off. Then, staged smallest-first, each stage priced by the receipt it turns green:

1. **Key (§1) as a pure function** — `computation_identity(module) = hash(source) ⊕ direct-import interface hashes ⊕ compiler-id`, unit-tested, no store yet. (Load-bearing files untouched; this is a new pure fn.)
2. **`CacheProvider` row** on `materialization_ladder.dag` via `provider_from_catalog` — `MemoTier{ContentKeyed}`, `SpacePacked{budget = host-envelope fraction}`, coverage = the typed-module identities. This is the *declaration*; it reds until §3's warm==cold receipt is live.
3. **Store realization** — the increment-C `Arc` store (its C1 Arc-migration is the prerequisite plumbing) re-keyed name→`ComputationIdentity`, with the eviction loop + counted evictions. **This is the first edit near retention machinery — gated on stages 1–2 green and operator sign-off of this sketch.**
4. **§3 oracle** wired as the discharge receipt + RED controls enrolled.
5. **§4 Pi acceptance** — cold/warm fingerprints + resolve-split wall + aging ratio, on the adversarial host.

**Dissolution trigger:** when the realization-measurement lane (P1) lands, the `SpacePacked` budget derivation swaps declared→measured (§2 terminal); when S2b cross-run persistence lands, this in-process content-keyed store becomes the backend S2b reads through (increment C §10) — same store, re-scoped, never re-invented.

---

## 6. The three questions this sketch asks the operator

1. **Key set (§1):** minimal (source ⊕ direct-import interfaces ⊕ compiler-id) — confirmed complete by the byte-identical oracle, not the conservative transitive-env fingerprint? *(recommend minimal.)*
2. **Budget denomination (§2):** interim = declared fraction of the host memory-envelope read (governor's own source), terminal = measured (P1)? Evictions counted, refuse-on-unavailable? *(recommend yes.)*
3. **Acceptance (§4):** both (a) wall-down and (b) aging-ratio-toward-1× required on the Pi; (a)-alone is the mis-tuned-eviction trap, not acceptance? *(recommend yes.)*
