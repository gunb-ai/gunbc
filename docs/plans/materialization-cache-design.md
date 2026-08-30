# Materialization cache (L3) — design draft, NOT YET BUILDING

Status 2026-08-30: HOLD from bright-ram-778 (operator side-chat approval pending; operator also
deciding whether L1–L3 move to the `gunbc build`/`gunbc test` CLI program out of XL-N).
Depends on L1 (sharp-newt-558): the `ExecutionPlan` that binds a committed `GitObjectId` and
materializes an immutable checkout from it. Do not merge before that seam.

## Why a key was not previously possible, and what changed

`gunbc.instrument_targets` (paragraph above `instrument_targets()`) and its hand mirror
`src/v1/stage0/src/target_invocation_host.rs` (`InvocationOutcome`) rule out target-result caching,
cross-run comparison, remote execution and replay "until SOURCE BINDING lands as its own
capability", and forbid adding a digest field: hashing the tree before/after a run proves nothing,
because the tree can go A -> B -> A while the producer reads a MIXED population and both hashes
agree. L1's committed-tree subject dissolves that by construction — the producer no longer reads a
mutable namespace — so the paragraph is L3's precondition and L1 is its dissolution.

A consumed-input manifest was considered and REJECTED (manager ruling): it is a second identity
scheme for bytes git already names (DESIGN §3 nickname) and it re-admits the dirty worktree L1
refuses.

## Deliverable 1 — MaterializationKey

    MaterializationKey = fold(tree id, producer identity, toolchain identity, policy digest)

Each input, with the authority that owns it (§3: cite the symbol, not the position):

| input | authority |
|---|---|
| tree id | `extdeps.git.object_store` `GitObjectId` (`GitSha1ObjectId` / `GitSha256ObjectId`), carried on L1's `ExecutionPlan` |
| producer identity | `gunbc.target_binding` `TargetBinding` — `extdeps.bazel.label` `render_label` for the target, `TargetProducer` for the realization |
| toolchain identity | `extdeps.realization.emit_on_demand_host` `observed_tool_identity` per-tool rows (NOT the `toolchain_identity` aggregate fold — grain mismatch, review 44388 / #7444) |
| policy digest | `std.content_hash` `content_hash_of_value` over the producer's own modeled policy rows (e.g. `floor_prepared_subject_exclusions`, a producer's declared source roots) |

Unmanifested channels (env, clock, network) are NOT covered by the key and must not be absorbed:
they are `std.materialization_ladder`'s `RefusedUnmodeledWorldRead` / `DemandNature` gate.

Discriminating RED (the `cache_impurity` mode, `gunbc.recurring_failure_mode`): change one input,
the key must change. The live specimen to use as the first RED is
`compile_dag_rust_emit_check_memo_key` (`src/v1/stage0/src/cli_run/emit_host.rs`), which keys on
source + file_path + includes + excludes + inventory digest and OMITS COMPILER IDENTITY — sound as
an in-process memo (one process, one compiler), unsound the moment it is persisted across runs.

## Deliverable 2 — persistence through the existing build_cache model

No second cache authority. `gunbc.build_cache_instance` is the deployed grain (one row per host x
consumer class, carrying endpoint / unit / principal / storage / capacity / compile pool);
`std.materialization_ladder` `CacheProvider` is the ladder-facing declaration (ArtifactTier,
ContentKeyed, `ProviderRetention`), the way `gunbc.floor_materialization` already declares its two
ReferenceTier shares. The deletion incident (a rust-cache post-step removing shared-home files) is
addressed by construction rather than by policy: the store writes only inside the instance's own
`intended_storage` root.

### (a) CI seed build
The modeled route already exists and is DEAD: measured 2026-08-25 (recorded in
`build_cache_desired_instances_note`), all four hosts carry `/var/lib/ctrl/sccache-ci/server.sock`
with ZERO listeners and no `gunbc-build-cache-ci.service` anywhere, so every desired row refuses.
Setting `RUSTC_WRAPPER` unconditionally in the emitted workflow would auto-spawn a server inside a
runner cgroup — exactly the placement `gunbc.build_cache_provision_verdict` refuses. So the emission
is GATED: observe the endpoint; a listener present -> attach; otherwise proceed cold and print the
receipt line. Provisioning the unit is host actuation, never a merge.

### (b) Floor memos across runs
Key extended with compiler identity per Deliverable 1; store under the instance's storage root.

### (c) Regen installed tree — keyed by tree id, optional.

## Deliverable 3 — falsifiers
- stale cache (same key, different bytes): impossible by key, since every determining input is IN
  the key; the RED is the omitted-input fixture above.
- warm run and cold run produce byte-identical outputs, compared by identity.
- cache unavailable -> cold + typed receipt line. This is a legitimate typed degradation (slower,
  never wrong), NOT the absorbing fallback: it does not substitute a superset answer and it is
  counted and located.

## Deliverable 4 — receipts by instrument
CI job timing lines; the floor's `compile_dag_rust_emit_check_memo` / `compile_dag_diagnostic_census_memo`
hit/miss lines (`claim_executor`); `gunbc.regen_round_cost` phases — before/after on the same tree.
