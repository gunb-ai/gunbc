# CI floor OOM + throughput — state of the world (handoff)

Snapshot 2026-06-26, by calm-carp-204. For a fresh planning session.

## The problem

1. **The CI `ci` (floor) job OOMs (exit 137) on every PR** — fleet-wide, including
   docs-only PRs. `claim_executor` exceeds the 8 GiB runner cgroup. This reds the
   `ci` signal everywhere and *masks real floor-gate failures*.
2. **CI wall-clock is 15-25 min** — unsustainable. Same root: per-shard memory
   forces narrow (low-parallelism) scheduling.

Both trace to **one root: the per-shard resolve is memory-bloated.**

## Root cause — measured by execution (this is the fix target)

- **Per-shard ≈ 2.75 GiB** (local run, width=6: 16.5 GiB total ÷ 6). At CI's
  width=3: `3 × 2.75 = 8.25 GiB > 8 GiB cap` → OOM. The arithmetic matches the
  observed kill exactly.
- **A single witness resolve is tiny: 13 MB (4 sources) to 41 MB (48 sources).**
  So the bloat is NOT one witness — it is the shard resolving its witnesses'
  combined module closure (hundreds of modules).
- **~73% of a resolved graph is per-module env-merge DUPLICATION** (`type_env`
  54% + `func_env` 19%): every module stores a *fully-merged copy* of every
  binding it transitively imports. Measured previously on a 233 MB artifact;
  consistent with the per-witness scaling here.
- **The fix:** replace the merged env with a **scope-chain env** (local bindings
  + `Rc` references to imported modules' envs; lookups walk the chain). Removes
  the ~73% → per-shard drops toward **~750 MB** → fits width=3 with headroom to
  go *wider* (which also fixes throughput). This is **"Layer A."**

NOTE on the measurement environment: `claim_executor` reads the **live cgroup
`memory.max`** at runtime. CI runner = 8 GiB (→ width 3). Local dev box reads
~31 GiB (→ width 6) — local runs are NOT clamped to 8 GiB. Re-measure under an
8 GiB-limited cgroup to replicate CI exactly. (Operator: local→BuildBuddy reroute
or a local MemoryMax clamp was intended but may not be in place.)

## Landed (merged to main)

- **#5833** — precompute scoping, 8.7× in `claim_batch` (a *different* binary than
  the floor gate; helped the per-shard runner, not the `claim_executor` floor).
- **#5836** — budget correctness: floor sizes width off the real 8 GiB cap, not a
  buggy ~13.9 GiB. Necessary but **insufficient** (per-shard still 2.75 GiB).
- **#5828** — orchestration-intent→Bash emit fold (neat-dove's de-fork prereq).

## In flight

- **#5837** (calm-carp-204) — emit-host isolation: moves `EmitHostGate`'s rustc
  children off the wide discovery batch. Proven by execution, draft. Helps, but
  the floor still OOMs at width=3 because the 2.75 GiB discovery shard is the
  binding constraint. **Needs Layer A to actually green.**
- **Layer A** (tidy-wren-707) — THE fix. Status: scaffolding in place —
  equivalence gate #5835 (proven to bite RED on import-loss), Layer B cache
  intern #5834 (disk only, not RSS), `parents` field-add (verified clean,
  shipping field-only), accessors (authoring, typed-init). **The value-delivering
  FLIP — de-merge `build_type_env`, hoist the kernel to a shared `Rc`, route the
  resolver spine + 3 enumerators + `04_resolve:80` through one chain-walk
  accessor, drop `merge_envs` — is NOT done. It is operator-gated.** This is the
  single most important pending action.
- **#5831** (neat-dove) — the per_shard=3.5 / width-1 *serialize* interim. We
  **dropped** it (operator did not want to cement serialism). Parked; a candidate
  temporary unblock IF a clamped measurement shows width=1 actually fits (unknown
  — a single shard resolving all 128 witnesses might OOM harder).

## Open decisions / blockers

1. **(A) Confirm the Layer A spine flip.** The load-bearing resolver-spine edit
   that actually cuts the 2.75 GiB. Held for explicit operator sign-off. THE fix
   is blocked here. The measurement now *justifies* it: per-shard is dominated by
   exactly the env-dup the flip removes.
2. **Interim?** width-1 serialize to restore a green CI signal while Layer A
   lands. Viability unmeasured. Operator declined cementing it earlier.
3. **§7 regen breakage** (separate, pre-existing): main's compiler seed does NOT
   self-reproduce — clean write-mode regen drifts 85 seed files, aborts on an
   unregistered `extdeps_uri_path.rs`, and emits 18× E0425 (`v1_rt::starts_with`
   undefined). Blocks clean stage0 regen; needs an owner.
4. **Throughput beyond OOM:** slow nextest tests (>600s individual cases),
   build/sccache flakiness, and the **resource-aware scheduler** (`width =
   memory / per_shard`, self-calibrating) — currently **unowned** (gentle-newt-542
   archived).
5. **Emitter gap** (filed separate): `fold` with a bare nullary-coproduct `init`
   emits nonexistent `Optional<...>` (E0412). Worked around with a typed init.

## The actual fix path (for the fresh session)

1. **Clamp/measure** one shard under an 8 GiB cgroup; confirm per-shard 2.75 GiB
   and the post-flip target (~750 MB). Test width=1 viability for an interim.
2. **Land Layer A's flip** (de-merge env) — guarded by #5835, with before/after
   `claim_executor` RSS as the proof. This is THE OOM fix.
3. **Re-confirm the floor greens** (#5837 emit-host isolation + Layer A together)
   at width=3 with headroom.
4. **Resource-aware scheduler** for throughput — only *after* per-shard is honest.
5. **Parallel, independent:** slow-test profiling (L3), build/sccache (L4), and
   the §7 regen fixed-point.

## Children

- **tidy-wren-707** — owns Layer A; ready for the flip. KEEP.
- **neat-dove-797** — de-fork lane (partly unblocked by #5828; a String→List<Int>
  codepoint straddle keystone remains). #5831 parked.

## Related docs

- `docs/plans/ci-throughput-fractal-profiling.md` — the profiling method + levels.
- `docs/plans/emit-host-batch-isolation.md` — #5837.
- `docs/plans/resolved-graph-representation-minimization.md` — Layer A/B detail.
