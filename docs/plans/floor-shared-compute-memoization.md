# Floor shared-computation memoization — design sketch

**Status:** DESIGN SKETCH — no implementation. Returns to stern-moth-225 + operator for approval before any code.

**Grounded 2026-06-29.** Root measurement: 537s clean-tree compile. Design principle: DESIGN.md §2 (minimize redundancy), §5 (correctness by construction, not validation).

---

## 1. Root — what is actually paid twice (and more)

The CI floor runs one `claim_executor` process per CI job, executing the plan as a sequence of batches. Within a batch, `SharedClaims` groups already deduplicate: all `RunnableSingleClaim` items sharing one `entry` file resolve that file's closure exactly once. This is correct and already lands.

The gap is **cross-batch and subprocess**: two independent dimensions of redundant work.

### Axis A — subprocess compiles (the expensive ones: ~537s each)

Three gates invoke compile-like subprocesses, each with a **distinct (source_roots, binary) tuple**:

| Gate | Binary | Source roots | Notes |
|---|---|---|---|
| `DslCompileCleanGate` | `gunbc compile` | `dsl src/v2` | `witness_layer_roots` via `dsl_compile_clean_transport.dag` |
| `EmitDeterminismGate` | `gunbc compile` | `dsl` only | `emit_determinism_corpus_roots = ["dsl"]` — intentionally narrower scope |
| `RegenVerifyGate` | `regen_stage0 --verify` | `src/v1 + dsl` (internal) | Different binary; verifies committed v1 seed against a fresh regen |

Because each gate uses a **different (source_roots OR binary) tuple**, none of the three produce byte-identical output. There is no artifact sharing opportunity between them today. The four compile invocations are not four calls to the same pure function — they are three distinct functions.

The **only true redundancy** on Axis A is `EmitDeterminismGate`'s intentional oracle pair: it runs its own tuple twice to catch non-reproducible emit. That pair must survive: emit is KNOWN-NONDETERMINISTIC today (Rust emitter HashMap iteration order; `v2.std.determinism` at P1), making the empirical x2 diff the only live check for that bug class. Content-addressing the output would assume determinism, not prove it.

**Axis A net today: no redundancy to remove.** The four compiles are four distinct necessary operations. The value of M2 (see §3) is therefore not present-day savings but **future-proofing**: the model makes it unwritable for a new gate to accidentally add a second compile of an existing tuple without declaring it. If a new gate is added that uses the same tuple as an existing gate, the structural dependency is explicit and the executor enforces sharing.

### Axis B — in-process `resolve_entry_graph` cross-batch (~30–40s per call)

`claim_executor`'s `run_walk` runs batches sequentially. The current batch-local `SharedClaims` deduplication does not persist across batch boundaries.

Eight of nine gates use `floor_gate_witness_entry = "dsl/tools/floor_effect_gate_witness.dag"` as their `.dag` claim entry (`ci_floor_plan.dag:139-143,150-151`). Only `SourceRootIngestGate` uses a different entry (`source_root_ingest_gate.dag`). The heavy-resolve serialization (`floor_heavy_resolve_chain_resource_edges`) places `RegenVerifyGate` and `EmitDeterminismGate` each in their own batch, alone, so `resolve_entry_graph` is called **4 times** on the same `floor_effect_gate_witness.dag` closure (~106 modules) per CI run:

| Call | Batch | Claim(s) | Resolve shared? |
|---|---|---|---|
| 1 | Batch 1 (compile anchor, alone) | `dsl_compile_clean_gate_passes` | — (first call) |
| 2 | Batch 2 Group A (5 negligible gates) | rust, emit_host, layering, extdeps, drift | SharedClaims within batch → 1 call for 5 claims |
| 3 | Later batch (alone, serialized) | `regen_verify_gate_passes` | re-resolves; no cross-batch sharing |
| 4 | Later batch (alone, serialized) | `emit_determinism_gate_passes` | re-resolves; no cross-batch sharing |

Calls 2–4 are redundant: same `(source_roots, entry)` pair, same pure function, same output. Three resolves are wasted across three separate batch boundaries within one `claim_executor` process.

M1 eliminates calls 2, 3, and 4 by memoizing the resolved graph from call 1 across the walk. Any future `heavy_whole_tree_resolve` gate added to `floor_effect_gate_witness.dag` in its own serialized batch would silently add a 5th call — the behavior is not unwritable today.

---

## 2. Current model has no shared-output node

The plan in `std.realization_schedule` represents runnables as independent units. `RunnableSingleClaim` carries `entry`, `function`, and `profile` — but has no concept of a produced artifact that other runnables consume. `DataDependsOn` edges declare ordering but carry no payload.

For Axis B (in-process resolve): this means two batches in the same `claim_executor` process can call `resolve_entry_graph` for the same `(source_roots, entry)` pair without the executor knowing they're the same work. Fixing it by adding a memo to `claim_executor` without model surface would concede the bad state is writable: a future gate added to `floor_effect_gate_witness.dag` would silently double-pay again.

For Axis A (subprocess compiles): the current gate set has no sharing opportunity (each gate uses a distinct tuple), so the missing shared-output primitive has no displacement value today. Its value is forward: if a future gate uses the same compile tuple as an existing gate, the model should make the sharing explicit and the duplication unwritable.

---

## 3. Proposed design — two complementary mechanisms

### M1 — Within-walk resolve memo (Axis B)

**Mechanism:** `run_walk` in `claim_executor` maintains a `HashMap<(source_roots_hash, entry), Arc<ResolvedGraph>>` across all batch executions. When a `SharedClaims` group resolves its entry, the result is stored. If a later batch resolves the same `(source_roots, entry)`, it gets the `Arc` clone — zero re-parse, zero re-typecheck.

**Correctness:** Pure function of `(source_roots, entry)` — the source files don't change during a single `run_walk`. The Batch 1 → Batch 2 execution is sequential (no concurrent mutation), so `Arc` is safe with no locking.

**Memory:** The resolved graph for `floor_effect_gate_witness.dag` is ~0.9 GiB. Since the heavy-resolve chain already serializes these batches (Batch 1 finishes before Batch 2 starts), both batches hold the graph at different times — the memo doesn't increase peak memory, it keeps the Batch 1 graph alive slightly longer. This is acceptable.

**Model surface needed (DESIGN.md §5 "model before implement"):** The `RunnableResourceProfile` already carries `heavy_whole_tree_resolve: Bool`. A new field `resolve_scope: ResolveScope` where `ResolveScope = ResolveScopeIsolated | ResolveScopeShared` lets the model explicitly declare which runnables participate in the shared resolve pool:
- `ResolveScopeShared` — the executor provides the memoized `ResolvedGraph` from the first resolve of this `(source_roots, entry)` in the walk; a second resolve is structurally impossible (the executor owns it). This is the construction guarantee.
- `ResolveScopeIsolated` (default) — the executor resolves independently; no sharing. Fail-closed: a new gate that omits `ResolveScopeShared` re-resolves rather than silently sharing stale data. The wrong behavior is safe (extra work), not silent (shared stale result).

The memo does NOT apply automatically to all same-entry resolves — only to `ResolveScopeShared` entries. This preserves the construction authority in the model rather than making the executor's behavior implicit. The lens verifies: every `heavy_whole_tree_resolve: true` runnable must declare `ResolveScopeShared` (the combination is never correct to isolate — a heavy resolve that runs independently in each batch is the violation this mechanism walls off).

### M2 — Compile artifact as a first-class plan node (Axis A)

**Current gate set — no sharing opportunities today.** Each of the three compile operations uses a distinct `(source_roots, binary)` tuple (verified against source):

| Gate | Tuple |
|---|---|
| `DslCompileCleanGate` | `(["dsl","src/v2"], gunbc compile)` |
| `EmitDeterminismGate` | `(["dsl"], gunbc compile)` — ×2 oracle pair |
| `RegenVerifyGate` | `(["src/v1","dsl"], regen_stage0 --verify)` — different binary |

No two gates share a tuple, so no artifact sharing is possible across gates today. Each `RunnableCompile` node is independent; `DslCompileCleanGate` and `EmitDeterminismGate` are different source-root sets and cannot share an artifact without re-introducing the narrower-vs-wider corpus skew.

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

## 5. Operator decisions required before implementation

**D1 — Where does `ResolveScope` live?**
Option A: Add `resolve_scope: ResolveScope` to `RunnableResourceProfile` in `std.realization_schedule`. Clean: the profile already describes resource semantics.
Option B: Keep it implicit — the executor memos all same-entry resolves with no model surface. Simpler but concedes the bad state is writable (a new gate won't know it's sharing).
**Recommendation: A** (model before implement, §5).

**D2 — Does M2 land in this PR or a follow-on?**
M1 (within-walk resolve memo) is lower-risk and immediately addressable. M2 (`RunnableCompile`) requires a new substrate type, a new executor code path, and a refactor of three gate transports to consume an artifact handle instead of shelling out. These are independently beneficial.
**Recommendation: M1 first (one PR, small, purely additive); M2 as a follow-on with operator approval of the substrate shape.**

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
| **Total compile cost per run** | ~2148s – ~37s ≈ same subprocess cost, saved resolve | **~1611s + ~37s → 3× compile, 1× resolve** |

M1 saves ~30–40s per run (the Axis-B double-resolve). Small absolute, but the mechanism is a construction wall. Unblocked today.

M2 saves **0 compiles today** — the current gate set has no duplicate `(source_roots, binary)` tuples (each gate uses a distinct one, verified). M2's value is forward-proofing: once the plan explicitly declares `RunnableCompile` nodes, a future gate that accidentally duplicates an existing tuple is caught by the lens before it silently adds another ~537s to CI.

**M2 displacement after determinism closes** (future state): if `EmitDeterminismGate`'s x2 oracle pair collapses to 1× once emit is provably deterministic, that frees 1× ~537s. That is the max M2 displacement in the current gate set — not 2×. Any further wins require a new gate that would otherwise duplicate an existing tuple.

| Mechanism | Compiles today | Compiles after | Resolve calls | Time saved |
|---|---|---|---|---|
| baseline | 4× | — | 2× | — |
| + M1 | 4× | 4× | 1× | ~35s |
| + M2 (gated on #5941) | 4× | 3× or 4× | 1× | ~537s (if oracle pair collapses) |

---

## 7. Dissolution triggers

M1 (`ResolveScopeShared` memo):
- Terminal within its scope: the within-walk resolve memo addresses Axis-B (in-process resolve deduplication across batches); it does not dissolve into M2, because M2 addresses Axis-A (subprocess compile declarations). The two are orthogonal.
- Dissolution trigger: when v2 streaming-infer (resource-aware-scheduler Node B/C) replaces the v1 whole-tree resolve with a dependency-batched resolve, per-entry memoization becomes a sub-case of the streaming boundary and M1's memo can be removed.
- Intermediate milestone: lens verifies all `heavy_whole_tree_resolve: true` runnables carry `ResolveScopeShared`.

M2 (`RunnableCompile`):
- **Gated on**: `v2.std.determinism` (#5941) closing non-reproducible emit. Content-addressing the artifact is unsound until emit is deterministic. Do not land M2 before this gate closes.
- After determinism closes: `EmitDeterminismGate` dissolves from an empirical oracle into a construction-verified witness; the 2× oracle pair becomes 1×, reducing total compiles from 4× to 3× (DslCompileClean, EmitDeterminism×1, RegenVerify each use distinct tuples and all remain necessary). Further reduction below 3× would require either collapsing two of the existing tuples to the same `(source_roots, binary)` or removing a gate — neither is in scope here.
- Dissolves into the v2 scheduler (resource-aware-scheduler.md Node B/C) once the scheduler derives `Runnable.cost` from the compile node's output rather than static measurement rows.
- Dissolves into the Realization pattern (#4867) for cross-run caching once M2's content-addressed artifact is stored on disk between runs.
