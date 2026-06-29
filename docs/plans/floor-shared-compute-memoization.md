# Floor shared-computation memoization — design sketch

**Status:** DESIGN SKETCH — no implementation. Returns to stern-moth-225 + operator for approval before any code.

**Grounded 2026-06-29.** Root measurement: 537s clean-tree compile. Design principle: DESIGN.md §2 (minimize redundancy), §5 (correctness by construction, not validation).

---

## 1. Root — what is actually paid twice (and more)

The CI floor runs one `claim_executor` process per CI job, executing the plan as a sequence of batches. Within a batch, `SharedClaims` groups already deduplicate: all `RunnableSingleClaim` items sharing one `entry` file resolve that file's closure exactly once. This is correct and already lands.

The gap is **cross-batch and subprocess**: two independent dimensions of redundant work.

### Axis A — subprocess `gunbc compile` (the expensive one: ~537s each)

Three gates independently invoke `gunbc compile --target rust` over the same corpus:

| Gate | Compiles |
|---|---|
| `DslCompileCleanGate` | 1× `gunbc compile --source-root dsl --source-root src/v2` |
| `EmitDeterminismGate` | 2× `gunbc compile --source-root dsl` in sequence (to diff output trees) |
| `RegenVerifyGate` | 1× via `regen_stage0 --verify` (internally compiles to compare against committed seed) |

On a clean tree with the same compiler binary: compile-1 and compile-2 (and compile-3 and compile-4) are pure functions of `(source_content, compiler_binary)` and produce byte-identical output. Running the same pure function 4 times is a §2 violation; the 3 extra runs are each ~537s of wasted wall time.

`EmitDeterminismGate` is particularly telling: it proves determinism by running the SAME compile TWICE and diffing — but if the compile is content-addressed, the proof is already in the hash (same key → same output; re-running is redundant). The gate's intent (verify determinism) can be satisfied by the content-address invariant, not by re-execution.

### Axis B — in-process `resolve_entry_graph` cross-batch (~30–40s per call)

`claim_executor`'s `run_walk` runs batches sequentially. The current batch-local `SharedClaims` deduplication does not persist across batch boundaries. The `floor_effect_gate_witness.dag` closure (~106 modules) is resolved in:

- Batch 1: evaluate `dsl_compile_clean_gate_passes` → 1 call to `resolve_entry_graph`
- Batch 2: evaluate all Group-A gate witnesses → 1 call to `resolve_entry_graph` for the SAME entry

These are the same `(source_roots, entry)` pair; same pure function; same output. Both calls happen in the same OS process within ~52s of each other. The second pays a full ~30–40s re-resolve for no reason.

Any future `heavy_whole_tree_resolve` gate added to `floor_effect_gate_witness.dag` will silently add another ~30–40s re-resolve. The behavior is not unwritable — it just happens automatically.

---

## 2. Current model has no shared-output node

The plan in `std.realization_schedule` represents runnables as independent units. `RunnableSingleClaim` carries `entry`, `function`, and `profile` — but has no concept of a produced artifact that other runnables consume. `DataDependsOn` edges declare ordering (compile gate must pass before corpus runs) but they carry no payload (the resolved graph or compiled artifact).

This means the executor cannot share the output of one runnable as the input of another. Each runnable independently recomputes from source. The double-payment is not a bug in the executor — it is an honest consequence of a model that has no shared-output primitive. Fixing it by adding a memo to `claim_executor` without changing the model would be a §5 validation standing where construction was available: a future gate added to `floor_effect_gate_witness.dag` would again silently double-pay.

---

## 3. Proposed design — two complementary mechanisms

### M1 — Within-walk resolve memo (Axis B)

**Mechanism:** `run_walk` in `claim_executor` maintains a `HashMap<(source_roots_hash, entry), Arc<ResolvedGraph>>` across all batch executions. When a `SharedClaims` group resolves its entry, the result is stored. If a later batch resolves the same `(source_roots, entry)`, it gets the `Arc` clone — zero re-parse, zero re-typecheck.

**Correctness:** Pure function of `(source_roots, entry)` — the source files don't change during a single `run_walk`. The Batch 1 → Batch 2 execution is sequential (no concurrent mutation), so `Arc` is safe with no locking.

**Memory:** The resolved graph for `floor_effect_gate_witness.dag` is ~0.9 GiB. Since the heavy-resolve chain already serializes these batches (Batch 1 finishes before Batch 2 starts), both batches hold the graph at different times — the memo doesn't increase peak memory, it keeps the Batch 1 graph alive slightly longer. This is acceptable.

**By-construction property:** Any gate in any entry that `run_walk` has already resolved in this run gets the memo automatically. A future gate added to `floor_effect_gate_witness.dag` cannot accidentally double-pay — the walk owns the resolve decision.

**Model surface needed (DESIGN.md §5 "model before implement"):** The `RunnableResourceProfile` already carries `heavy_whole_tree_resolve: Bool`. A new field `resolve_scope: ResolveScope` where `ResolveScope = ResolveScopeIsolated | ResolveScopeShared` lets the model explicitly declare which runnables participate in the shared resolve pool. The executor enforces: `ResolveScopeShared` entries may NOT re-resolve; a future runnable without this field defaults to `ResolveScopeIsolated` (safe, fail-closed — it re-resolves rather than silently sharing stale data). The lens verifies that every `heavy_whole_tree_resolve: true` runnable declares `ResolveScopeShared`.

### M2 — Compile artifact as a first-class plan node (Axis A)

**Mechanism:** Introduce `RunnableCompile` as a variant of `Runnable` in `std.realization_schedule`:

```
type Runnable
  = RunnableSingleClaim { ... }        -- existing
  | RunnableDiscoveryBatch { ... }     -- existing
  | RunnableCompile {                  -- NEW
      source_roots: List<String>
      target: EmitTarget
      artifact_id: ContentHash         -- the content-address key
      profile: RunnableResourceProfile
    }
```

A `RunnableCompile` is a plan node that:
1. Invokes `gunbc compile --source-root ... --target ...`
2. Content-addresses its output (directory tree hash)
3. Stores the artifact under `artifact_id` in a within-run artifact store
4. Exposes a typed `CompiledArtifact` handle to dependent nodes via `DataDependsOn`

**CRITICAL DEPENDENCY:** M2 is GATED on emit being deterministic. Today's Rust emitter has known non-determinism (HashMap iteration order in the emit path; `v2.std.determinism` is only at P1 as of 2026-06-29). Content-addressing a non-deterministic compile is unsound: two cold runs produce DIFFERING outputs, so a content-address key would collide on differing artifacts and serve an arbitrary one to consumers — the exact "key went green while the realizer faked the key" §5 fail-open trap. **M2 must not land until the determinism mechanism (#5941, `v2.std.determinism`) closes the non-determinism gap.** Sequence: M1 → determinism mechanism → M2.

**The determinism gate is the ORACLE, not a redundancy.** `EmitDeterminismGate`'s two cold, independent compiles are the empirical oracle for non-reproducible emit. Content-addressing does not prove determinism — it assumes it. Memoizing the pair to one artifact would delete the only live check for a known bug, making the gate inert (§5 coverage-by-illusion). The x2 compiles REMAIN in M2.

Dependent nodes change their contract under M2:
- `EmitDeterminismGate` → **unchanged**: still runs 2× cold, independent `gunbc compile` and diffs. It is the oracle that LICENSES the artifact as canonical. Both runs benefit from the M1 in-process resolve memo (same source closure, so each resolves once). Net: 2× compile (same as today for this gate), ~30–40s of resolve saved.
- `DslCompileCleanGate` → consumes `EmitDeterminismGate`'s proven-canonical `CompiledArtifact` via `DataDependsOn`. Checks that the compile exits 0. Does NOT re-compile. (Previously: 1× compile. After M2: 0× compile.)
- `RegenVerifyGate` → consumes the same proven-canonical `CompiledArtifact`. Verifies committed seed matches it. Does NOT re-compile. (Previously: 1× compile. After M2: 0× compile.)

**Compile count:** 4× → 2× (the determinism oracle pair). Saves ~537s × 2 ≈ 18 min.

**By-construction property:** The plan has exactly ONE `RunnableCompile` node per `(source_roots, target)` tuple. The executor raises a typed error if it encounters two `RunnableCompile` nodes with the same key — the plan is malformed, not silently redundant. A future gate that needs the compiled artifact adds a `DataDependsOn` on the existing `RunnableCompile` node; it cannot accidentally introduce a second compile without explicitly creating a second `RunnableCompile` node, which the lens will flag.

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
Yes, it survives unchanged as the oracle. Its x2 compiles are not a cost to eliminate — they are the discriminating witness for non-reproducible emit, a known live bug. The gate dissolves only when the determinism mechanism (`v2.std.determinism`, #5941) closes the bug and makes non-determinism unwritable by construction; at that point the two cold compiles become definitionally redundant and the gate can be replaced by a construction-side check. Until then, deleting or memoizing the pair is a §5 coverage-by-illusion.
**Recommendation: keep both compiles; gate M2 on the determinism mechanism landing.**

**D4 — Scope of `RunnableCompile.source_roots`.**
`DslCompileCleanGate` compiles `dsl + src/v2` (the full floor corpus). `EmitDeterminismGate` today compiles only `dsl`. These are different `(source_roots, target)` tuples — two `RunnableCompile` nodes. Is this the intended split, or should both use the same full-corpus compile? (The difference is intentional today because emit_determinism targets only `dsl`.)
**Recommendation: Preserve the existing split; document it explicitly in the plan as two separate `RunnableCompile` nodes with different `source_roots`.**

---

## 6. Displacement analysis

| Cost today | Cost after M1 | Cost after M1+M2 (gated on determinism) |
|---|---|---|
| `resolve_entry_graph` × 2 per run (~30–40s each) | × 1 (memo deduplicates Batch 1 → Batch 2) | × 1 |
| `gunbc compile` × 4 per run (~537s each) | × 4 (M1 doesn't help subprocess) | × 2 (determinism oracle pair survives) |
| **Total compile cost per run** | ~2148s – ~37s ≈ same subprocess cost, saved resolve | **~1074s + ~37s → 2× compile, 1× resolve** |

M1 saves ~30–40s per run (the Axis-B double-resolve). Small absolute, but the mechanism is a construction wall. Unblocked today.

M2 saves ~537s × 2 — the other two compiles (DslCompileCleanGate and RegenVerifyGate) consume the determinism oracle's proven-canonical artifact. That is ~18 minutes of CI wall time removed per run. **Gated on `v2.std.determinism` (#5941) closing the non-determinism gap before content-addressing compile output is sound.**

The combined displacement: **~2148s → ~1074s for the compile-related floor costs** (4→2 compiles, 2→1 resolve), roughly a 2× improvement. The remaining 2× (the determinism oracle pair) is load-bearing and dissolves only when emit is provably deterministic.

---

## 7. Dissolution triggers

M1 (`ResolveScopeShared` memo):
- Dissolves into M2 (`RunnableCompile`)'s pre-resolve: once the compile node holds the compiled+resolved graph as a typed artifact, the within-walk memo becomes a special case of "artifact produced by Batch-1 consumed by Batch-2" — the general mechanism subsumes it.
- Intermediate milestone: lens verifies all `heavy_whole_tree_resolve: true` runnables carry `ResolveScopeShared`.

M2 (`RunnableCompile`):
- **Gated on**: `v2.std.determinism` (#5941) closing non-reproducible emit. Content-addressing the artifact is unsound until emit is deterministic. Do not land M2 before this gate closes.
- After determinism closes: `EmitDeterminismGate` dissolves from an empirical oracle into a construction-verified witness; the 2× oracle pair becomes 1× (and M2 can collapse further to 1× total compile).
- Dissolves into the v2 scheduler (resource-aware-scheduler.md Node B/C) once the scheduler derives `Runnable.cost` from the compile node's output rather than static measurement rows.
- Dissolves into the Realization pattern (#4867) for cross-run caching once M2's content-addressed artifact is stored on disk between runs.
