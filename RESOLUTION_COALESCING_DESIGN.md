# Resolution coalescing — where we are, where we'll be

> Review artifact (not a committed parallel-ledger doc). This is the design rationale for
> the scope-(a) fire-fix PR and the scope-(c) cure direction; its durable home is the PR
> body + a `.dag` dissolution mark, never a standing `.md`. Grounded against origin/main
> `608ef66c71` (the #5122 merge that lit the fire).

---

> **Status reconciliation (read first).** The "CI never finishes / 160m" hang was a
> **different bug** — an infinite interpreter loop in `ci_gates` (commit `83867956ae`,
> *"break ci_gates interpreter loop blocking #5138"*), now fixed by witty-stag. CI completes
> GREEN at 23m44s. The per-claim re-resolve below is real and on the active CI path, but it is
> a **bounded inefficiency + the recurrence pattern + a §5 hazard scope (a) would introduce** —
> **not** a liveness emergency, and the *smallest* of the three CI wall-time prongs (compile
> ~13m and dsl_compile_clean ~6m dominate). So this is a correctness-thesis / efficiency effort,
> not a fire. Treat scope (c) — the cure — as the real payoff; scope (a)'s throwaway-seed
> band-aid is no longer justified by urgency.

## TL;DR

- **#5122 shipped dependency *ordering*, not *coalescing*.** On the active CI path
  (`claim_executor`, `ci_spec.dag:113`), each `SingleClaim` re-resolves its full closure in an
  isolated `!Send` thread; the `DiscoveryBatch` runnable collapses the ~199 corpus rows into
  one shared resolve (a SCAFFOLD that *admits* the problem by special-casing it away). So the
  re-derivation waste is bounded to the `SingleClaim` entries — not "hundreds" — and is the
  recurrence pattern, not the hang.
- **The cure you named is the right one**: coalescing should be a property of the
  *representation* (hash-consing), not a runtime cache. The building block already exists —
  `content_hash(Node)` (`src/v2/std/node.dag:1541`, 271 uses, the self-host fixed-point
  authority). **The seed just declines to use it.**
- **Two deliverables, sequenced and honest:**
  - **(a) fire fix** — share one resolve within the sweep instead of per-claim. Throwaway
    seed code. Buys runway. Carries its own §5 constraint (below) — it is *not* hazard-free.
  - **(c) the cure** — content identity at `Node` construction, so resolve/parse/typed-module
    memoization become **one lookup against node identity with no key to get wrong.** This
    dissolves the fire, the 6+ bespoke caches, *and* the modeled-walk OOM together.
- The "kernel that unifies the caches" idea recedes from *the answer* to *a stepping stone we
  may not need* once (c) lands — because it is still bolting onto the seed.

---

## 1. Where we are — the fire, located

The path a `claim_batch --roster-from-discovery` sweep actually takes:

```
claim_batch --roster-from-discovery
  └─ DiscoveryBatch over ~199 corpus rows          (self-labeled SCAFFOLD)
       └─ for each claim:  thread::spawn            ← one OS thread per claim
            └─ run_single_claim
                 └─ resolve_entry_graph(roots, entry)   ← FULL cold re-resolve, per claim
                      └─ parse → typecheck → infer the entire ~40-module closure
```

Three failure points, each verified in the tree:

1. **Per-claim cold re-resolve.** `src/v1/stage0/src/bin/claim_executor.rs:282` calls
   `resolve_entry_graph` inside `run_single_claim`; `:408` does one `thread::spawn` per
   runnable; `:248` documents the reason: *"The resolved graph is Rc-based (!Send), so each
   claim resolves and runs entirely within its own thread."* There is **zero shared state**
   between claims — which is exactly why it is thread-safe, and exactly why it re-resolves
   everything N times.

2. **The only sharing is gated off.** `resolved_graph_cache` (the one correctly content-keyed
   cache — verify-on-read, atomic temp-rename, sorted-stable closure digest) is reached only
   through `resolved_graph_cache_root_from_env()` (`cli_run.rs:435/551`), which returns `Some`
   only when `GUNBC_RESOLVED_GRAPH_CACHE_DIR` is set — **set nowhere in CI.** Even when live,
   it keys on the *whole closure*, so it can never hit on a *shared sub-module*.

3. **The scheduler models order, not sharing.** `src/v2/workflow/ci_floor_plan.dag` emits only
   `floor_depends_on_compile` edges onto a single compile root; its header: *"The ONLY fact
   added here is the dependency TOPOLOGY."* The witnesses assert order + parallelism — **none
   asserts that a shared resolve ran once.** The plan is correct as far as it goes; it stops
   one concept short of what was asked.

**Net (corrected):** the headline "never finishes" symptom was the `ci_gates` interpreter loop
(`83867956ae`), now fixed — CI is GREEN at 23m44s. The per-claim re-resolve is genuine waste on
this path, but bounded (DiscoveryBatch shares the corpus) and the smallest of the three prongs.
Its importance is as the **recurrence pattern** (§2) and the **§5 hazard a naive fix introduces**
(§4), not wall-time.

### A measurement question, already discharged statically

The natural worry — *is the cost resolve, or the O(total-source) text digest?* — is settled
without a profiler. `subject_digest_for_closure → closure_content_digest`
(`resolved_graph_cache.rs:77`, confirmed an `atom_identity_hash(content.to_string())` over
every file's **raw source text**) is called **only inside the two env-gated blocks**
(`cli_run.rs:435/551`). Env unset in CI → the digest is **never computed in the fire**. So
the resolve-vs-digest split in CI is **100% resolve, 0% digest** — a stronger result than a
timing (the function isn't reached). The super-linear blowup that *was* real was bisected
during prong-3 to import/interface resolution and fixed by #5135 (target_model 288s→1s,
body_producer 312s→28s). **What remains in CI is plain N×-redundant re-resolve of shared
modules — linear waste × N claims — which is exactly what scope (a) erases.**

---

## 2. Why it recurs (the operator's real question)

*"Won't this just happen with another area of the code later?"* — Yes, and the tree already
proves the pattern: **6+ hand-rolled re-derivation caches, each invented locally, three keyed
on non-content, one already a latent miscompile:**

| site | key | status |
|---|---|---|
| `resolved_graph_cache` | content digest | the one done right (but env-gated, version-string hazard) |
| `parse_cache` (`cli_run.rs:287`) | module path | local-proven |
| `typed_module_cache` (`cli_run.rs:308`) | module **name** | **surfaced a latent miscompile** (intern-id order bug, `cli_run.rs:334-343`) |
| `data_cache` | path | local-proven |
| `complexity cache_summary` / `CostInternTable` | name | local-proven |
| `v2 ParseTable` | — | the §2 carrier is staged, not inhabited |
| `pure_call_memo` (`v1_interpreter.rs:749`) | **Rc address / usize** | the genuine exception — identity, not content |

Every time someone needs to not-recompute, they mint a fresh bespoke cache with a private key
and a hand-written soundness proof — and a third of the keys carry preconditions the *next*
feature silently violates (see §4: perturbation is the live counterexample to the name key).
**That is the recurrence. It is the precise inverse of this language's thesis.**

---

## 3. The principle — coalescing is a representation property

In a content-addressed DAG, **identical subgraphs *are* the same node** (deduplicated by
content hash). Coalescing isn't a runtime cache you bolt on; it's hash-consing — a property
of how the graph is built. The machinery exists today: `content_hash` (a provenance-free
merkle catamorphism, `src/v2/std/node.dag:1541`) and `combine_hash` (`:732`), folded as
`combine_hash(a: acc, b: content_hash(n: n))` (`05_eval.dag:609`), and trusted as the
self-host fixed-point authority (`self_host.dag:152-162`).

**The gap is that the v1 *seed* resolver does not route the resolve step through content
identity.** It runs `resolve_entry_graph` as an imperative Rust function, per claim, and keys
its one cache on a raw text re-hash that never touches `content_hash(Node)`. So
coalescing-by-construction is the spec; the seed has not inhabited it *here* yet. Closing this
is the seed shrinking, exactly per §7 — and it is not a one-site theory: the modeled recursive
filesystem walk OOM-kills the same interpreter on a ~1k-file tree (quiet-swift). **Two
symptoms, one root: the seed doesn't share identity, so it re-resolves and re-walks until it
blows up.**

---

## 4. Where we'll be — two scopes

### Scope (a) — within-run coalescing (the fire fix)

Resolve each module **once per sweep** and share, instead of once per claim. `claim_batch`
*already* shares one `MultiEntryIndex` (`parse_cache` + `typed_module_cache`) across its
entries (`cli_run.rs:287/308`); `claim_executor` is the one path that doesn't. Route
`claim_executor`'s gate path through the same shared index, pre-populated in dependency order
before fan-out. No disk, no cache key, no version string, no env var.

**Shape: (i) single-threaded shared index, labelled throwaway.** Not "(i) vs (ii) by profile."
`MultiEntryIndex` is `RefCell`-backed → `!Sync` → cannot be borrowed across `thread::spawn`
even read-only. The two sound shapes are (i) sequential-within-a-readiness-layer sharing the
index single-threaded, or (ii) freeze into an `Arc` `Send+Sync` snapshot of `ResolvedGraph` +
`NewlineIndex`. **(ii) is capitalizing the seed §7 says shrinks to zero — brass on a sinking
ship.** `parse_table_memo_amortization_test` already proves within-context memo goes
sub-linear, so (i) suffices. If a profile ever says parallelism pays, the answer is *pull the
`.dag` resolver forward* (which coalesces by construction once (c) lands), never refactor the
doomed Rust.

#### The §5 constraint scope (a) MUST carry (the perturb fail-open)

Scope (a) is **not** hazard-free. The `typed_module_cache` born-mark (`cli_run.rs:293-307`)
states the name→typed-result key is sound *"ONLY over a pure unit,"* its precondition being
*"a module name maps to exactly one immutable source file."* **Perturbation violates this
directly**: `perturb_function_to_false` (`claim_executor.rs:483`, remapped into a
self-consistent temp tree at `:566-586`) rewrites content under an unchanged module name. A
shared name-keyed index spanning the clean **and** perturbed walks serves the clean typed
result to the perturbed module → the planted `->false` is masked → the discriminating RED
witness passes green. **Fail-open on the §5 machinery whose whole job is to forbid it.** The
existing `resolve_typed_cache_equivalence_test` permutes entry *order* only (`cli_run.rs:331,346`)
and **cannot** catch this.

Therefore scope (a) ships with three non-negotiables:
1. **Any content-mutating walk (every perturb receipt) gets a fresh index** — never reads the
   clean sweep's name-keyed cache. (Content-keying the typed cache by `(name, content-digest)`
   also fixes it, but that is scope (c); for throwaway seed code, fresh-index-for-perturb.)
2. **Add a content-under-same-name RED witness** — the order-permuted one cannot go red here.
3. **Verify first** whether perturb receipts currently route through `claim_batch`'s shared
   index (already exposed on main) or `claim_executor`'s isolated per-claim resolve (safe
   today). Belief: the latter — so the hazard is one scope (a) *introduces*, not pre-existing —
   but confirm, don't assume.

#### Honest accounting for scope (a)

Scope (a) coalesces at **name** grain; the kernel/(c) at **content/node** grain. The cure does
not *subsume* (a)'s index — it **replaces** it at finer granularity. So in honest terms: **the
shared index is cache #7 — to be DELETED when content identity lands, not folded in.** The PR
says that plainly, with a dissolution mark pointing at (c), or it is the §3 violation it claims
to end.

### Scope (c) — content identity at `Node` construction (the cure)

Make `content_hash(Node)` the identity under which Nodes are interned *as they are built* —
true hash-consing. Then resolve, parse, and typed-module memoization stop being three keyed
caches with three hand-written soundness proofs and become **one lookup against node identity,
with no key to get wrong** (and so no per-site soundness adventure — perturbation can't alias,
because mutated content is a different node). This is the §1/§2/§7 answer: performance *falls
out* of the representation instead of being *fought for* per site. It dissolves:
- the fire (per-claim re-resolve — identical closures are the same nodes),
- 5 of the 6 bespoke caches (parse/typed/data/complexity/resolved-graph collapse to identity
  lookup; `pure_call_memo` stays out — it is address-identity, not content, deliberately),
- quiet-swift's modeled-walk OOM (the walk stops re-deriving shared subtrees).

`pure_call_memo` is the one genuine non-fit and is correctly carved out (`v1_interpreter.rs:749`,
`HashMap<(usize, Vec<usize>), Value>`, keepalive vecs load-bearing for the *address* keys).

#### The cross-run cache (former "scope b") is gated behind a real hazard

If durable cross-run reuse is ever wanted, `resolved_graph_cache`'s key folds two **hand-bumped**
strings (`RESOLVE_LOGIC_VERSION="v2-resolve-1"`, `KERNEL_INTERN_SEED_VERSION="kernel-seed-1"`,
`resolved_graph_cache.rs:19-23`). A resolver-semantics change without a manual bump serves a
stale-but-content-matching verdict — the exact fail-open §5 forbids, uncatchable by
verify-on-read (source content is unchanged; only the logic changed). **Keep the durable cache
off the floor until the version *derives* from `content_hash` of the resolve stage** — itself a
(c)/§7 milestone. Scope (a) is immune (no version key at all). And when it does land: *remove*
the env-gate, never merely default it (two activation modes = the §3 fork it is trying to kill).

---

## 5. What ships, in order

1. **(a) now** — single-threaded shared index in `claim_executor`, the three §5 non-negotiables,
   the honest "cache #7, delete-not-fold" mark. No env var, no digest, no Arc-refactor. Owns the
   fire and buys runway.
2. **(c) the cure** — `content_hash(Node)`-keyed interning at construction in the seed. This is
   where the instability actually ends. Two witnesses already point at it (per-claim re-resolve;
   modeled-walk OOM).
3. The kernel-unifying-caches design recedes to a stepping stone — possibly unneeded once (c)
   lands, because hash-consing at construction *is* the unification, with no kernel to bolt on.

---

## Appendix — verification ledger (file:line, checked against `608ef66c71`)

- `claim_executor.rs:282` per-claim `resolve_entry_graph`; `:408` thread-per-runnable; `:248`
  the `!Send`/Rc rationale; `:483` `perturb_function_to_false`; `:566-586` temp-tree remap.
- `cli_run.rs:287` `parse_cache`; `:308` `typed_module_cache`; `:293-307` the name-key born-mark
  ("ONLY over a pure unit"); `:331,346` order-permute equivalence test; `:334-343` the latent
  intern-id miscompile; `:435/551` the env-gate (only call sites of the digest).
- `resolved_graph_cache.rs:77` `closure_content_digest` (raw text re-hash); `:95`
  `subject_digest_for_closure`; `:19-23` the two hand-bumped version strings.
- `src/v2/std/node.dag:1541` `content_hash`; `:732` `combine_hash`; `05_eval.dag:609` the merkle
  fold; `self_host.dag:152-162` the fixed-point authority.
- `src/v2/workflow/ci_floor_plan.dag` "ONLY fact added is the dependency TOPOLOGY"; witnesses
  assert order/parallelism, none assert shared-resolve-once.
- `v1_interpreter.rs:749` `pure_call_memo` (address-keyed, correctly excluded).
- `parse_table_memo_amortization_test.dag` within-context memo proven sub-linear (the basis for
  shape (i)).
