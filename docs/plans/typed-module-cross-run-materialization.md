# Typed-module materialization across runs

> **Status:** IMPLEMENTED, UNMEASURED (2026-09-20). The declarations live in
> `gunbc.floor_materialization` (rows `floor_typecheck_store_persist_*`) and the realization in
> `v1_compiler.shared_typecheck_store` (`PersistentTypedStore`). This file carries only what
> neither of those can: the operating procedure for the measurement, the four operational choices
> and their reasons, and the deletion trigger. Nothing here restates a declaration — a number or a
> verdict quoted here would be a second authority for it.
> **Authority it answers to:** `DESIGN.md`; the ladder's admission rules in
> `std.materialization_ladder`; the retention vocabulary in `std.cache_interface`.
> **Framing:** relief during the v1 → v2 migration. Not a parallelism project.

---

## 1. What this is

The typed-module snapshots the in-process tiers already produce now have a host-local,
content-addressed backing, so the value survives the process that computed it. It is one ladder
rung — a second provider over the identity `union-typecheck-store(dag,src/v2)` that the in-process
provider already covers — and not a new cache: the byte transport, the content key and the seam
are the ones that were already there, and only the backing changed.

## 2. What it delivers, and what it does not

**Delivered: same-build, cross-process reuse.** Two processes running ONE built binary over one
corpus — the witness fold and the generated-artifact drift gate — each pay typecheck preparation
in full today. The second one now reads the first one's results.

**Not delivered: cross-push reuse.** The typed-module content key's compiler-identity term is
`resolved_graph_cache::transform_content_digest()`, the content hash of the running binary, and
`src/v1/stage0/build.rs` embeds `GUNBC_BUILD_IDENTITY` — the checkout's commit — into that binary
through `cargo:rustc-env`. A rebuild at any new commit, INCLUDING a corpus-only `.dag` commit,
therefore re-keys every entry. Any claim that an unchanged module survives a push is unsupported
until compiler identity is derived from something invariant under a corpus-only commit, which is a
change to that derivation and not to this store. **The design this file replaces asserted that
relief; that assertion does not follow from the key it proposed, and is withdrawn.**

The bound is also narrower than "preparation" within one run: a hit skips `collect_parent_envs`
and the typecheck compute, which is where the preparation wall is spent, but the module is still
parsed and resolved, because that is what derives the key. Parse is not relieved.

## 3. The measurement that decides whether this was worth it

Not yet run. It is three demonstrations, and the first two are what "working" means here:

1. **Cold → warm, separate processes.** Run the fold and then the drift gate over one corpus under
   one built binary with the store armed, against the same pair with it unarmed. Compare end-to-end
   wall, peak RSS, and the fold's subject digest — which must be EQUAL, because a store that
   changes a verdict is a defect and not a saving. Read the `[typed-store-persist]` line on the
   floor receipt for hits, misses, rejections, refusals, throughput and LIVE OCCUPANCY.
2. **An invalidation control.** Edit one module in the corpus and confirm that module and its
   dependents miss while the rest hit, and that no obsolete result is served.
3. **A failure control.** Point the store at an unwritable directory, truncate an entry, and fill
   it past its ceiling. Each must produce a counted cold or refused path. A verification mismatch
   must be reported as `persist_rejected`, never folded into an ordinary miss.

The Rust-level evidence for (3) and for the store's own properties is
`persistent_typed_store_tests` in `v1_compiler.shared_typecheck_store`, including the ceiling
refusal, the truncation rejection, the key-mismatch rejection and the residency-versus-throughput
distinction. Those are unit-level; (1) and (2) are the production-path receipt and are outstanding.

## 4. How to arm it

Off by default. Three environment variables, read once per process:

- `GUNBC_TYPED_STORE_PERSIST=<dir>` — the host-local directory. Arming is naming it.
- `GUNBC_TYPED_STORE_PERSIST_MAX_BYTES=<n>` — overrides the modeled ceiling
  (`floor_typecheck_store_persist_cap_bytes`).

The SOURCE-ROOT SET is not an environment variable: it is taken from the index that asks, because
resolution is root-relative and the roots are a fact about the computation rather than an
operator's assertion about it. A second root set inside one process refuses.

## 5. The four operational choices, and why

| Choice | Settled as | Why |
|---|---|---|
| Location and ownership | One host-local directory named by the operator, outside the checkout. No daemon, no network service. | The tier is `ArtifactTier`/`LocalFilesystem` by `tier_axes`; anything more is a different provider with a different correctness surface. |
| Capacity and replacement | `ByteCapacity` with an exact limit, `RefuseNewStore`. | The ladder refuses `ReleasedNever` without a ceiling AND without observed occupancy, so both are obligations. `RefuseNewStore` over LRU because replacement is a second mechanism this bridge does not need: a full store declines, counts, and the run is merely cold. An entry-count bound over a variable-size payload establishes no byte bound. |
| Warm-seeding | Ordinary runs populate it. No seeding workflow. | A seeding job is a standing cost defended by a benefit nobody has measured yet. |
| Verification | The cold/warm procedure in §3, operator-invoked. | A scheduled job costs a full preparation by definition and is not a prerequisite for the first relief. |

## 6. Deletion trigger

This is a bridge over the v1 seed, and it ends with its consumer, not with a date: **when the
authoritative native route replaces the v1 preparation this store sits behind, its persistence
realization, its arming switch and its provider row are removed together.** v1 deletion does not
wait for this optimization to become a general platform. The rows are all named
`floor_typecheck_store_persist_*` in one module and the realization is one section of one file, so
the removal is a deletion rather than an untangling.

## 7. Non-goals

Threading the resolver, re-sharding the fold, and moving the fold to a hosted runner are all out
of scope. This change removes repeated work rather than distributing it.
