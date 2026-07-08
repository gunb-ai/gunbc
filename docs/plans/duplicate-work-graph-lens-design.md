# Codebase-wide duplicate-work lens — design

**Status:** DESIGN (pre-implementation). Grounds entirely on authorities that exist today; **no S2 / execution-spine dependency** (see §7). Discriminant must be proven by the four-case witness (§6) before any corpus fold is written — if the discriminant is wrong the lens is noise, and that is the whole risk.

**Origin:** the operator's original framing — "detect a redundant dependency and force a rewire; kill compile → … → recompile" — generalized from the point-instance `v2.lens.duplicate_computation` (shell/argv transports, #6372) to the whole codebase. This doc is the general mechanism; the existing shell lens becomes one instance of it (§5).

**Principle roots:** DESIGN.md §2 (minimize redundancy — the *horizontal* direction: one computation, one producer), §3 (single authority — a forked realization is a §3 violation at the realization layer), §5 (correctness by construction: Delete > Share > Cache; hash-consing makes the pure fork *unwritable*), §6 (priced in displaced cost, not elegance — the cost floor).

---

## 1. The question — "how could we possibly catch redundant work in the graph?"

Redundant work is **the same computation demanded more than once as independently-produced nodes**, rather than produced once and shared. The substrate makes this decidable because of two facts, both re-verified against the live tree (2026-07-08):

- **`content_hash` is occurrence-independent** (`src/v2/std/node.dag:1449`; `canonicalize_node_for_content_hash` stamps `SyntheticOccurrence` at `:594`). It folds `kind` + edge-labels + child-hashes and never reads `occurrence_id`. So two nodes that are *the same operation applied to content-identical input subgraphs* hash-equal, at any position. **content_hash is the intent identity** — "same expected inputs and outputs at an intent level," made a decidable structural fact.
- **The substrate does not hash-cons.** `node_with_occurrence_id` (`node.dag:115`) mints an explicit, distinct `occurrence_id` per constructed node. Structural twins therefore genuinely exist as separate nodes. If construction auto-deduped there would be nothing to catch; because it does not, forks are real and findable.

So the detector is: over the whole-corpus `Node` graph, `content_hash` every node, group by hash. Each group of size > 1 is candidate duplication. Everything after is *discrimination* — which groups are real, fixable redundancy (§2), and which are noise.

## 2. The discriminant — four conditions, all required

A content-hash group is a **fixable-redundancy wall** only if all four hold. This is the crux; each condition is grounded on an existing authority.

### D1 — Same computation *(content-hash equal)*
The grouping key itself (§1). Sound and decidable: zero false positives on "is this the same work." The only strength question is *how much* structural normalization we ground into the key — see §4.

### D2 — Fork, not reuse *(distinct occurrences)*
This is the condition `occurrence_id` exists to draw, and it is the difference between DRY and redundant:

- **Reuse** = **one** node, **N in-edges** — N consumers point at one shared producer. Already DRY, correct, the goal state. **Never flagged.**
- **Fork** = **N distinct-occurrence** nodes, one content-hash — the same computation authored/demanded N separate times. **This is the redundant work.** The fix turns a fork into reuse: delete N−1, rewire consumers to the survivor.

Mechanically: within a content-hash group, count distinct `occurrence_id`s. Group of one occurrence with many parents → reuse. Group of many occurrences → fork. `SyntheticOccurrence` twins (freshly built, e.g. by a fold) need care — see §6 case notes; the corpus signal is *minted* occurrences (`MintedOccurrence`) that repeat a hash.

### D3 — Safe to collapse *(purity is the license, idempotency is the effect-case proof)*
The operator's key refinement: **purity is not what makes work duplicated — D1 already settled that. Purity is what makes the fix provably safe.**

- **Pure node** → deleting one twin and rewiring its consumers to the other is behavior-preserving *by construction*. Certain. This is "certainly duplicated work." Purity is read from the model's own effect boundary — a node whose realization carries **no effect** (the empty-`uses` boundary the recompute-trace already keys on, `v1_interpreter.rs`; the modeled surface is `std.effects`).
- **Effectful node** → deleting one twin is safe **only if the effect is idempotent / collapsible**, which is a proof obligation `dag/std/effects.dag` already discharges: `is_idempotent_effect` (`:31`), and pairwise `create_double_init_collapsible` (`:73`) / `create_effect_is_dedupable` (`:62`). Two content-identical `AppendEffect`s (non-idempotent) genuinely happen twice — *not* redundant, must not be flagged. Two `CreateIfAbsent` with equal key-source collapse — flagged.

So D3 splits exactly on the S2 boundary (§7): the **effect** case is a static structural fact (two producer nodes, one hash, effect idempotent) decidable now; the **pure** case's *runtime* elimination (Share/memoize the eval) needs the interpreter's spine — but the pure *fork* is still statically detectable and statically rewireable (delete the duplicate node), which is what the operator wants. Pure-value runtime Share is the deferred part; pure-node static rewire is not.

> Generalization debt: `create_double_init_collapsible` is **pairwise**. A content-hash group is N-ary. The lens needs a group-collapse predicate `effect_group_collapsible(shapes)` = all shapes idempotent *and* mutually collapsible — a fold over the group reusing the pairwise authority, not a new authority. Named here as the one real extension D3 requires.

### D4 — Worth collapsing *(cost floor — §6 priced in displaced cost)*
Every `[]`, `True`, `Int(0)`, bare atom is a content-twin. Flagging them is the purity-trap flood (§6): infinite, elegant, worthless. Only a fork whose **shared computation clears a cost floor** is a wall. Grounds on the existing cost lens (`src/v2/lens/cost.dag`): a fork of compile-whole-tree (~500s) is a wall; a fork of `[]` is noise. The floor is a declared threshold on the group's shared-subtree cost — and per §5/§6 a threshold is a smuggled heuristic *only if it selects a degradation arm*; here it selects *what to report*, never *whether to fail open*, so it is a reporting scope, not an absorbing fallback.

## 3. The fix hierarchy — Delete > Share > Cache (why the operator prefers rewire over cache)

The operator's ordering, and it is DESIGN §2/§5 exactly:

1. **Cache / Memoize — dispreferred.** Keep the duplicate nodes, memoize the result. This is the *confession that the redundancy was not removed* (§2's "eleven hand-rolled `HashMap` caches"). Sharp corollary: **every cache in the tree is a map of known redundancy** — `pure_call_memo`, `ParseTable` memo, `resolved_graph_cache`, sccache each exist because someone knew the work would be redone. The caches are therefore this lens's own candidate-site list.
2. **Share / rewire — preferred.** One node, N in-edges. The lens's actionable output: "these N are one computation; wire consumers to one, delete the rest." This is the "reorganize / delete the redundant work" the operator asked for, over a cache.
3. **Construction / hash-cons — endgame (§5 strongest).** Content-addressed construction returns the existing node, so a pure fork is **unwritable** — you never reach for the cache because the duplicate cannot be created. This is a substrate change (share content, keep `occurrence_id` as provenance). The lens finds forks now; hash-consing makes them impossible later. The lens is *sizing* that endgame.

## 4. Honesty boundary — which strengths are walls, which is the §5 "never" trap

"Same inputs/outputs at an intent level" has three reachable strengths. Naming which are walls is a §5 obligation (do not let a ratchet masquerade as a wall):

- **Structural** (content-hash equal): decidable, sound, zero false positives. **A wall.** This is what ships.
- **Normalized** (canonicalize commutative ops; apply the algebra-grounded rewrites the codebase already has, e.g. `a+b` ≡ `b+a`): decidable *up to the chosen normalization*. Each normalization is a bounded, deliberate step — a wall relative to its declared normalizer. Additive, later.
- **Semantic** (same input→output function regardless of structure): **undecidable (Rice's theorem)** — can *never* be a wall. This is the §5 "never" trap: a lens finds *some* semantic dupes, never all. Honestly the permanent ratchet, never promised as complete.

So: the wall is **structural + declared normalization**; full "intent-level equivalence" is the honestly-named ratchet. The lens's `ConstructionJustification` must say `WallAfterGrounding` for the structural core and carry the ratchet residue explicitly.

## 5. Relationship to the existing shell lens — it becomes an instance

`v2.lens.duplicate_computation` (#6372) already models exactly this shape at one grain: `ComputationDemandFact { computation_key, guard, replication }`, grouped by key, refused when N unguarded demands share a key, with a declared-`ReplicatedOracle` escape for intentional duplication (the emit-determinism x2 oracle). Its `computation_key` is an argv projection; the general lens's key is `content_hash`. Its guard/replication discriminants (guarded fallback never duplicates; declared oracle is admissible) are **exactly D2's reuse-vs-fork and the declared-exception escape D3 needs**. So the general lens **subsumes** it: the shell/argv key and the CI-job key both become "compute a `content_hash`-grade identity for this demand." Do not build the CI-job lens as a separate point-thing — it is this lens at job grain (where Share is a file download, hence shippable today; §7).

The declared-exception escape carries over verbatim: `ReplicatedOracle { runs: N }` at the site admits a genuinely-intentional duplication (known-nondeterministic emit), refused if observed ≠ declared. Same §5 discipline — the exception is *declared and counted*, never a silent widen.

## 6. De-risking — the four-case discriminating witness (build FIRST)

Before any corpus fold, one witness proves the discriminant is right. If it discriminates correctly the corpus lens is a mechanical fold over it; if it cannot, the lens would be noise. Four cases, each a `test fn` with a discriminating input that goes **red** when the behavior is wrong:

1. **GREEN — reuse.** One node (single occurrence), two parents/in-edges → **must not flag.** Red control: if the lens flags reuse, D2 is broken.
2. **RED — pure fork.** Two distinct-occurrence, content-identical **pure** nodes above the cost floor → **must flag exactly one violation**, naming the survivor as rewire target. Red control: perturb one node's subtree so hashes differ → violation disappears (proves it is content-identity, not position).
3. **ADMISSIBLE — declared oracle.** Two content-identical nodes carrying a declared `ReplicatedOracle` (the emit-determinism case) → **not flagged**; observed ≠ declared → flagged. Same escape as the shell lens.
4. **EFFECT-GATED.** Two content-identical `AppendEffect` nodes → **not flagged** (genuinely twice, non-idempotent); two content-identical `CreateIfAbsent`-equal-key nodes → **flagged** (collapsible). Proves D3 reads `effects.dag`, not just purity.

A fifth guard (cost floor, D4): two content-identical `[]`/`Int(0)` nodes below the floor → **not flagged**, proving D4 suppresses the purity-trap flood. Red control: drop the floor to zero → they flag, confirming the floor is what suppressed them (not an accident of the fold).

## 7. Why no S2 dependency

The confusion that chained this to S2 was conflating three threads; only the third needed the spine:

| Thread | Needs spine? | Where it lives |
|---|---|---|
| Detect/enforce duplicate **dependencies** (the original ask) | **No** — static structural fact over `content_hash` + `occurrence_id` | this lens |
| **Measure** re-evaluation (recompute-trace) | No — already a diagnostic (#6372) | landed |
| Make **eval-grain** duplication cheap (Share/memoize the pure runtime) | **Yes** | deferred to execution-spine `materialize`/Share |

D3 is exactly this split: the **effect half** is a static wall (two producer nodes, one hash, effect idempotent — decidable now, no interpreter). The **pure-value runtime** half (the ~135k eval-duplicates the recompute-trace found) is the spine's `Share`, correctly deferred. The half the operator actually wants — "kill compile → recompile" — is the effect half, a static wall today. And effect/producer nodes are **sparse** (hundreds corpus-wide, not the millions of eval nodes), so a one-pass content-hash grouping over the effect set does **not** inherit the enforcement-witness's whole-corpus N² eval cost — cheap-interpreted, no S2.

This is the spine's own **materialize: Recompute-vs-Share** law delivered at the one grain where it is a static wall today (job/effect grain = sharing is a rewire/file), with the eval grain (sharing is runtime memoization) left to the spine. **One law, two grains, one shippable now.**

## 8. Scope and sequence

- **In scope now:** the structural (D1) + fork (D2) + effect-idempotency (D3-effect) + cost-floor (D4) lens over the corpus `Node` graph, with the declared-oracle escape. The four-case witness first.
- **Deferred to the spine:** pure-value *runtime* Share (eval-grain memoization). The lens still statically detects and rewires pure *node* forks — only the runtime elimination waits.
- **Sequence:** (a) four-case discriminating witness proving the discriminant; (b) the `effect_group_collapsible` N-ary fold over the pairwise `effects.dag` authority; (c) corpus fold grouping `content_hash` over the effect/producer node set, gated by D4 cost floor; (d) fold the shell lens (#6372) in as the argv-grain instance.

## 9. Dissolution triggers

- **Endgame — hash-consing (§3):** when construction is content-addressed (share content, keep `occurrence_id` as provenance), pure forks become unwritable and the *pure* arm of this lens dissolves into construction (§5 strongest form). The effect arm persists (effects are not value-shared by hash-consing).
- **Spine `Share`:** when demands are execution-spine `DependencyView` nodes and `materialize` dedups by construction, the eval-grain residue this lens does not cover retires to the undeclared-demand residue — the same dissolution the shell lens (#6372) already declares (`WallAfterGrounding → RealizationDispatch`).
- **Normalization ratchet (§4):** never fully dissolves — semantic equivalence is undecidable. Honestly permanent; new normalizers are additive walls, not a path to "never."
