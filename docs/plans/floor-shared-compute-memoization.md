# Floor shared-computation memoization — M1 receipt + M2 tracker

**Status:** M1 LANDED + verified (2026-07-02). M2 remains a forward-only tracker (no present-day sharing opportunity — see §3). This file is no longer a pre-implementation sketch; it is the M1 receipt + M2 open-thread record. Any M2 code still returns to the operator for approval.

**M1 — within-walk resolve memo — is implemented as construction, not the sketch's proposed validation.** `claim_executor.rs` `run_walk` maintains a cross-batch `walk_memo: HashMap<entry, InterpContext>` keyed directly off `RunnableResourceProfile.heavy_whole_tree_resolve` (see §3-M1's `use_walk_memo`), so a heavy whole-tree resolve that runs isolated per batch is **unwritable** — stronger than the sketch's proposed `ResolveScope = Isolated | Shared` field, which would have needed a lens to forbid the heavy+isolated combination. The construction guarantee lives at the carrier; no `ResolveScope` field and no lens were added. Receipt: two purity oracles in the `claim_executor` bin tests — `memo_warm_cold_results_are_identical` (warm memo path is byte-identical to the cold `run_shared_entry_claims` path — the §5 purity oracle) and `memo_deduplicates_resolve_count` (resolve fires exactly once across batch boundaries; goes RED if the memo is bypassed). Run: `cargo test -p v1-compiler --bin claim_executor memo_` — 2 pass, 0.37s (2026-07-02).

**Grounded 2026-06-29.** Root measurement: 537s clean-tree compile. Design principle: DESIGN.md §2 (minimize redundancy), §5 (correctness by construction, not validation).

---

## 1. Root — what is actually paid twice (and more)

The CI floor runs one `claim_executor` process per CI job, executing the plan as a sequence of batches. Within a batch, `SharedClaims` groups already deduplicate: all `RunnableSingleClaim` items sharing one `entry` file resolve that file's closure exactly once. This is correct and already lands.

The gap is **cross-batch and subprocess**: two independent dimensions of redundant work.

### Axis A — subprocess compiles (the expensive ones: ~537s each)

Three gates invoke compile-like subprocesses, each with a **distinct (source_roots, binary) tuple**:

| Gate | Binary | Source roots | Notes |
|---|---|---|---|
| `DagCompileCleanGate` | `gunbc compile` | `dag src/v2` | `witness_layer_roots` via `dag_compile_clean_transport.dag` |
| `EmitDeterminismGate` | `gunbc compile` | `dag` only | `emit_determinism_corpus_roots = ["dag"]` — intentionally narrower scope |
| `RegenVerifyGate` | `regen_stage0 --verify` | `src/v1 + dag` (internal) | Different binary; verifies committed v1 seed against a fresh regen |

Because each gate uses a **different (source_roots OR binary) tuple**, none of the three produce byte-identical output. There is no artifact sharing opportunity between them today. The four compile invocations are not four calls to the same pure function — they are three distinct functions.

The **only true redundancy** on Axis A is `EmitDeterminismGate`'s intentional oracle pair: it runs its own tuple twice to catch non-reproducible emit. That pair must survive: emit is KNOWN-NONDETERMINISTIC today (Rust emitter HashMap iteration order; `v2.std.determinism` at P1), making the empirical x2 diff the only live check for that bug class. Content-addressing the output would assume determinism, not prove it.

**Axis A net today: no redundancy to remove.** The four compiles are four distinct necessary operations. The value of M2 (see §3) is therefore not present-day savings but **future-proofing**: the model makes it unwritable for a new gate to accidentally add a second compile of an existing tuple without declaring it. If a new gate is added that uses the same tuple as an existing gate, the structural dependency is explicit and the executor enforces sharing.

### Axis B — in-process `resolve_entry_graph` cross-batch (~30–40s per call)

`claim_executor`'s `run_walk` runs batches sequentially. The current batch-local `SharedClaims` deduplication does not persist across batch boundaries.

Eight of nine gates use `floor_gate_witness_entry = "dag/tools/floor_effect_gate_witness.dag"` as their `.dag` claim entry (`ci_floor_plan.dag:139-143,150-151`). Only `SourceRootIngestGate` uses a different entry (`source_root_ingest_gate.dag`). The heavy-resolve serialization (`floor_heavy_resolve_chain_resource_edges`) places `RegenVerifyGate` and `EmitDeterminismGate` each in their own batch, alone, so `resolve_entry_graph` is called **4 times** on the same `floor_effect_gate_witness.dag` closure (~106 modules) per CI run:

| Call | Batch | Claim(s) | Resolve shared? |
|---|---|---|---|
| 1 | Batch 1 (compile anchor, alone) | `dag_compile_clean_gate_passes` | — (first call) |
| 2 | Batch 2 Group A (5 negligible gates) | rust, emit_host, layering, extdeps, drift | SharedClaims within batch → 1 call for 5 claims |
| 3 | Later batch (alone, serialized) | `regen_verify_gate_passes` | re-resolves; no cross-batch sharing |
| 4 | Later batch (alone, serialized) | `emit_determinism_gate_passes` | re-resolves; no cross-batch sharing |

Calls 2–4 are redundant: same `(source_roots, entry)` pair, same pure function, same output. Three resolves are wasted across three separate batch boundaries within one `claim_executor` process.

M1 (landed) eliminates calls 2, 3, and 4 by memoizing the resolved graph from call 1 across the walk. A future `heavy_whole_tree_resolve` gate added to `floor_effect_gate_witness.dag` in its own serialized batch **cannot** silently re-resolve: because the memo keys off `heavy_whole_tree_resolve` itself, any such gate joins the shared memo by construction (see §3-M1).

---

## 2. Current model has no shared-output node

The plan in `std.realization_schedule` represents runnables as independent units. `RunnableSingleClaim` carries `entry`, `function`, and `profile` — but has no concept of a produced artifact that other runnables consume. `DataDependsOn` edges declare ordering but carry no payload.

For Axis B (in-process resolve): before M1, two batches in the same `claim_executor` process could call `resolve_entry_graph` for the same `(source_roots, entry)` pair without the executor knowing they're the same work. **M1 closed this** — but not by adding an implicit memo with no model surface (which would concede the bad state is writable). It keys the memo off the *existing* model field `RunnableResourceProfile.heavy_whole_tree_resolve`: a heavy resolve is the exact class that must share, so declaring the field IS the declaration to share, and heavy+isolated is unwritable. The model surface was already there; M1 read it.

For Axis A (subprocess compiles): the current gate set has no sharing opportunity (each gate uses a distinct tuple), so the missing shared-output primitive has no displacement value today. Its value is forward: if a future gate uses the same compile tuple as an existing gate, the model should make the sharing explicit and the duplication unwritable.

---

## 3. Design — M1 (landed) and M2 (forward-only tracker)

### M1 — Within-walk resolve memo (Axis B) — **LANDED**

**Mechanism (as landed):** `run_walk` in `claim_executor` maintains a `walk_memo: HashMap<entry, InterpContext>` across all batch executions. Keying by `entry` alone is sufficient — a given entry always resolves against the same `source_roots` within one walk (documented at the `walk_memo` decl). On the first heavy-resolve batch the entry is resolved once and its `InterpContext` stored; a later batch for the same entry reuses the cached context — zero re-parse, zero re-typecheck. (The original sketch proposed `HashMap<(source_roots_hash, entry), Arc<ResolvedGraph>>`; the landed form is the simpler entry-keyed context.)

**Correctness:** Pure function of `(source_roots, entry)` — the source files don't change during a single `run_walk`, and within a walk `entry` determines `source_roots`. The batches on the memo path run sequentially on the main thread (no concurrent mutation). Verified by `memo_warm_cold_results_are_identical` (warm==cold byte-identity, the §5 purity oracle).

**Memory:** The resolved graph for `floor_effect_gate_witness.dag` is ~0.9 GiB. Since the heavy-resolve chain already serializes these batches (Batch 1 finishes before Batch 2 starts), both batches hold the graph at different times — the memo doesn't increase peak memory, it keeps the Batch 1 graph alive slightly longer. This is acceptable.

**Model surface (as landed — supersedes the proposal below):** No new field was added. The memo keys directly off the existing `RunnableResourceProfile.heavy_whole_tree_resolve: Bool` — the executor treats every heavy whole-tree resolve as memo-shared, and a same-entry non-heavy claim in a later batch also joins the memo (clause (b) in `run_walk`). This is *stronger* than the proposal: the invariant "a heavy resolve is never correctly isolated" (which the proposed lens below would have enforced) is discharged by construction — heavy+isolated cannot be expressed. The `ResolveScope` design below is retained only as the rejected alternative.

**~~Proposed model surface~~ (rejected in favour of the direct key above):** A new field `resolve_scope: ResolveScope` where `ResolveScope = ResolveScopeIsolated | ResolveScopeShared` would have let the model explicitly declare which runnables participate in the shared resolve pool:
- `ResolveScopeShared` — the executor provides the memoized `ResolvedGraph` from the first resolve of this `(source_roots, entry)` in the walk; a second resolve is structurally impossible (the executor owns it). This is the construction guarantee.
- `ResolveScopeIsolated` (default) — the executor resolves independently; no sharing. Fail-closed: a new gate that omits `ResolveScopeShared` re-resolves rather than silently sharing stale data. The wrong behavior is safe (extra work), not silent (shared stale result).

Under the rejected proposal the memo would have applied only to `ResolveScopeShared` entries, backed by a lens requiring every `heavy_whole_tree_resolve: true` runnable to declare `ResolveScopeShared`. The landed design collapses that: since the combination heavy+isolated is never correct, the memo fires directly on `heavy_whole_tree_resolve` and no `ResolveScopeShared` field or lens exists — the invariant the lens would have checked is discharged by construction instead.

### M2 — Compile artifact as a first-class plan node (Axis A)

**Current gate set — no sharing opportunities today.** Each of the three compile operations uses a distinct `(source_roots, binary)` tuple (verified against source):

| Gate | Tuple |
|---|---|
| `DagCompileCleanGate` | `(["dag","src/v2"], gunbc compile)` |
| `EmitDeterminismGate` | `(["dag"], gunbc compile)` — ×2 oracle pair |
| `RegenVerifyGate` | `(["src/v1","dag"], regen_stage0 --verify)` — different binary |

No two gates share a tuple, so no artifact sharing is possible across gates today. Each `RunnableCompile` node is independent; `DagCompileCleanGate` and `EmitDeterminismGate` are different source-root sets and cannot share an artifact without re-introducing the narrower-vs-wider corpus skew.

**The EmitDeterminismGate oracle pair survives unchanged.** Its two cold compiles use the same tuple and are the empirical oracle for non-reproducible emit. Content-addressing assumes determinism; it does not prove it. Emit is KNOWN-NONDETERMINISTIC today (`v2.std.determinism` at P1), so memoizing the pair would delete the only live check for that bug (§5 coverage-by-illusion). The x2 compiles remain in any M2 shape.

**M2's value is forward-looking, not present-day displacement.** The structural mechanism `RunnableCompile` is introduced so that if a future gate needs to compile the same `(source_roots, binary)` tuple as an existing gate, it must declare a `DataDependsOn` on the existing node rather than adding a second `RunnableCompile`. That second node would be statically flagged by the lens as a duplicate tuple — the double-compile becomes **unwritable by construction** rather than a silent accidental addition.

**Mechanism sketch:**

```
type Runnable
  = RunnableSingleClaim { ... }        -- existing
  | RunnableDiscoveryBatch { ... }     -- existing
  | RunnableCompile {                  -- NEW (future-proofing wall)
      source_roots: List<String>
      binary: CompileBinary            -- GunbcCompile | RegenStage0Verify | ...
      target: EmitTarget
      profile: RunnableResourceProfile
    }
```

A lens enforces: no two `RunnableCompile` nodes in the same plan share the same `(source_roots, binary, target)` key. A `RunnableSingleClaim` that invokes a compile operation must reference the plan's canonical `RunnableCompile` node for that tuple via `DataDependsOn`; it cannot shell out to compile independently.

**CRITICAL DEPENDENCY:** M2 is GATED on emit being deterministic. Content-addressing a non-deterministic compile output is unsound — two cold runs produce differing artifacts, so a content-address key would collide and serve an arbitrary one to consumers (the "key went green while the realizer faked the key" §5 trap). **M2 must not land until `v2.std.determinism` (#5941) closes the non-determinism gap.** Sequence: M1 → determinism mechanism → M2.

**Compile count:** unchanged today (4×). The benefit is prevention of future regression, not current displacement.

---

## 4. Scope — what this is NOT

**This is NOT the cross-run resolve cache (#4867).** The `resolved_graph_cache.rs` disk cache enables sharing across separate `claim_executor` invocations (separate runs, separate PRs). It is currently dormant in CI (compiler binary changes every commit → cold cache; the JSON I/O buffers multi-GiB into memory). That lane's fix is streaming serialization + a content-key that doesn't include the compiler binary for stable sub-closures. That work is independent and orthogonal.

**This is NOT a general memoization of all interpreter calls.** `pure_call_memo` and `ParseTable` memo are separate mechanisms with separate key derivations. This design only addresses the floor's cross-batch resolve cost and the subprocess compile cost.

**This is NOT a change to the CI schedule ordering.** The `DataDependsOn` / `ResourceDependsOn` dependency model stays. `RunnableCompile` becomes a new batch-1 node; all gates that consume its artifact become batch-2 (already the case via `floor_depends_on_compile`).

---

## 5. Operator decisions — D1/D2 RESOLVED (M1 landed); D3/D4 still open (M2)

**D1 — Where does `ResolveScope` live? — RESOLVED.**
Neither of the sketch's options was taken. Instead of adding a `resolve_scope` field (Option A) or keeping an implicit memo with no model surface (Option B), M1 keys directly off the *existing* `RunnableResourceProfile.heavy_whole_tree_resolve` field. That field already carries the exact semantics ("this runnable does the heavy whole-tree resolve"), and a heavy resolve is never correctly isolated, so declaring the field IS the declaration to share — a model surface without a new field, and the bad state (heavy+isolated) unwritable. This is stronger than Option A and avoids Option B's writable-bad-state concession.

**D2 — Does M2 land in this PR or a follow-on? — RESOLVED for M1.**
M1 landed on its own (small, purely additive, entry-keyed context memo). M2 (`RunnableCompile`) remains a separate follow-on requiring a new substrate type, a new executor code path, and a refactor of three gate transports — and stays gated on operator approval of the substrate shape (see D3/D4 and §7).

**D3 — Does `EmitDeterminismGate` survive M2?**
Yes, unchanged. Its x2 compiles are the empirical oracle for non-reproducible emit (a live bug). The gate dissolves only when `v2.std.determinism` (#5941) makes non-determinism unwritable by construction; at that point the pair becomes definitionally redundant and can be replaced by a construction-side check.
**Recommendation: keep both compiles; gate M2 on the determinism mechanism landing.**

**D4 — Distinct tuples per gate in M2.**
All three gates use different `(source_roots, binary)` tuples and therefore each gets its own `RunnableCompile` node with no artifact sharing. This is the correct model — it is a change to HOW compile operations are declared in the plan (explicitly, as named nodes) rather than a change to how many compiles are run. A future gate that reuses an existing tuple will automatically reuse the node; that is the mechanism's payoff.
**Recommendation: accept the three-node model; document the distinct tuples explicitly so a future author knows which node to declare a dependency on.**

---

## 6. Displacement analysis

| Cost today | Cost after M1 | Cost after M1+M2 (gated on determinism) |
|---|---|---|
| `resolve_entry_graph` × 4 per run (~30–40s each) | × 1 (memo deduplicates calls 2–4 to call 1) | × 1 |
| `gunbc compile` × 4 per run (~537s each) | × 4 (M1 doesn't help subprocess) | × 3 (oracle pair 2→1; other two tuples unchanged) |
| **Total compile cost per run** | ~2148s – ~105s ≈ same subprocess cost, saved resolve | **~1611s + ~105s → 3× compile, 1× resolve** |

M1 saves ~105s per run (3 redundant Axis-B resolves × ~35s each). The mechanism is a construction wall. **Landed** (keyed off `heavy_whole_tree_resolve`; oracles green).

M2 saves **0 compiles today** — the current gate set has no duplicate `(source_roots, binary)` tuples (each gate uses a distinct one, verified). M2's value is forward-proofing: once the plan explicitly declares `RunnableCompile` nodes, a future gate that accidentally duplicates an existing tuple is caught by the lens before it silently adds another ~537s to CI.

**M2 displacement after determinism closes** (future state): if `EmitDeterminismGate`'s x2 oracle pair collapses to 1× once emit is provably deterministic, that frees 1× ~537s. That is the max M2 displacement in the current gate set — not 2×. Any further wins require a new gate that would otherwise duplicate an existing tuple.

| Mechanism | Compiles today | Compiles after | Resolve calls | Time saved |
|---|---|---|---|---|
| baseline | 4× | — | 4× | — |
| + M1 | 4× | 4× | 1× | ~105s (3 calls × ~35s) |
| + M2 (gated on #5941) | 4× | 3× or 4× | 1× | ~537s (if oracle pair collapses) |

---

## 7. Dissolution triggers

M1 (landed — `heavy_whole_tree_resolve`-keyed `walk_memo`):
- Terminal within its scope: the within-walk resolve memo addresses Axis-B (in-process resolve deduplication across batches); it does not dissolve into M2, because M2 addresses Axis-A (subprocess compile declarations). The two are orthogonal.
- Dissolution trigger: when v2 streaming-infer (resource-aware-scheduler Node B/C) replaces the v1 whole-tree resolve with a dependency-batched resolve, per-entry memoization becomes a sub-case of the streaming boundary and M1's memo can be removed.
- No lens milestone: the sketch's planned "lens verifies all `heavy_whole_tree_resolve: true` runnables carry `ResolveScopeShared`" was obviated — with the memo keyed directly on `heavy_whole_tree_resolve`, the isolated-heavy state is unwritable, so there is nothing for a lens to check.

M2 (`RunnableCompile`):
- **Gated on**: `v2.std.determinism` (#5941) closing non-reproducible emit. Content-addressing the artifact is unsound until emit is deterministic. Do not land M2 before this gate closes.
- After determinism closes: `EmitDeterminismGate` dissolves from an empirical oracle into a construction-verified witness; the 2× oracle pair becomes 1×, reducing total compiles from 4× to 3× (DagCompileClean, EmitDeterminism×1, RegenVerify each use distinct tuples and all remain necessary). Further reduction below 3× would require either collapsing two of the existing tuples to the same `(source_roots, binary)` or removing a gate — neither is in scope here.
- Dissolves into the v2 scheduler (resource-aware-scheduler.md Node B/C) once the scheduler derives `Runnable.cost` from the compile node's output rather than static measurement rows.
- Dissolves into the Realization pattern (#4867) for cross-run caching once M2's content-addressed artifact is stored on disk between runs.
