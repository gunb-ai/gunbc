# v2 native compiler memory: what one source byte costs

> **Status:** Analysis for review, 2026-10-08. No source change. Every figure below is a dated observation from one host (srv2, aarch64, `main` at `6d82ac7721`), not an instrument; the reproduction section names how to re-derive each one.
> **Authority it answers to:** DESIGN §2 (minimize the demand graph before materializing), §6 (bare minimum cost), §6b (chain re-derivation; measurement as the adversary of a reading).
> **Framing:** the v2 native CLI (`v2.cli.compile_cli`, generation one from `//gunbc/instruments:v2-native-cli`) cannot emit any entry against the real roots: `emit --entry std.integer --source-root dag --source-root src/v2` ran 900 s, reached 15.7 GiB still rising, and produced nothing. This document dissects where the memory goes and names the boundary that owns it.

## Answer

Peak memory is set by the **parse of the single largest file**, and inside that parse about 60% of the peak is **old versions of the span index kept alive by packrat memo entries**. Each `ParseMemoAccepted` in `v2.compiler.parse` stores a whole `ParseProvenanceState`, whose `index: SpanIndex` is the corpus-so-far span map at the moment the memoized production finished. Every later `span_index_record` therefore finds the map shared and path-copies it, and each memo entry pins its own copy. A memo hit needs only the entries for the ids in its own frame (`parse_prov_replay_frame` reads nothing else), so almost all of that retained state is never read.

## The chain, re-derived

1. `v2_cli_main` reads every file under the source roots, then `emit_closure_from_ingest_located` calls `closure_ingest`, which parses and censuses **every** file before the entry is consulted. A nonexistent entry costs exactly as much as a real one. (§2: the demand is the entry's closure; the census materializes the corpus.)
2. Per file, `closure_ingest_step` keeps `parse_tree` and `census` in `ClosureIngestFile` and merges the file's spans into one `SpanIndex` with `span_index_merge`. Both live until exit.
3. Inside the parse, `parse_nonterminal_memoized` on a miss runs the production in `parse_prov_enter_frame(prov)` — which keeps `prov.index`, the whole map — and stores the result with `parse_memo_entry_from_expr`, which copies that `prov` into the memo entry.
4. `span_index_record` (`v2.std.provenance`) inserts through `map_insert`. In the emitted Rust (`v1_rt::rc_map_insert`) this is `Rc::make_mut` on `index.entries.clone()` while `index` still holds the map, so the refcount is always ≥ 2 and every insert copies an `im` HAMT path — whether or not a memo entry holds the old version. Where a memo entry does hold it, the copy is retained.

## Measurements

Single-file roots isolate the parse; `std.algebra` is `dag/std/algebra.dag`, the big file is `src/v2/compiler/body_lowering_fold.dag`.

| Input | Source | Peak RSS | Span records | Final span entries | Memo inserts | Memo hits |
|---|---|---|---|---|---|---|
| `std.integer` | 13.6 KB | 31 MiB | 3,905 | 3,606 | 968 | 5 |
| `std.algebra` | 64.5 KB | 171 MiB | 52,081 | 48,419 | 12,277 | 73 |
| `body_lowering_fold` | 594 KB | 1.65 GiB | 485,137 | 426,161 | 121,308 | 1,284 |
| root `src/v2/compiler` (97 files) | 3.57 MB | 2.52 GiB | — | — | — | — |

- A process with an empty root still makes 12.6 M allocations and takes 4.5 s preparing the grammar and lexer.
- Peak per source byte grows with file size (≈ 2.3 KB/B for `std.algebra`, ≈ 2.8 KB/B for the big file); a 48 k-entry map should cost a few MB, so the 171 MiB is versions, not entries.
- `std.algebra` allocates ≈ 2 GB in total for a 171 MiB peak; the largest bucket is 544-byte blocks (2.2 M of them), consistent with `im` chunk and HAMT node copies.
- On the 97-file root the largest file alone reaches 1.65 of the 2.52 GiB: cross-file retention is the smaller term.

### Probes (measurement-only edits to a copy of the emitted crate)

Each probe changed one site; output JSON was compared byte-for-byte with the unpatched binary.

| Probe | `std.algebra` | big file | 97-file root | Output |
|---|---|---|---|---|
| Unpatched | 171 MiB | 1.65 GiB | 2.52 GiB | — |
| Memo entry keeps an **empty** span index | 95 MiB | **0.65 GiB** | 2.26 GiB | identical on all three |
| Memo entry keeps a span index **restricted to its frame** (replayed through `parse_prov_replay_frame`) | 274 MiB | — | 4.16 GiB | identical |
| Census does not merge per-file spans | — | — | 2.12 GiB | not compared |
| … and keeps no `ClosureIngestFile` | — | — | 1.60 GiB | not compared |

The restricted-frame probe falsified the first reading: building a fresh map per memo entry costs more than sharing structure with the live map, because frames nest and each entry re-copies its children's ids. So the remedy is not a smaller snapshot; it is no snapshot.

## Proposed changes, ranked

1. **The span index is not backtracked state.** Occurrence ids are allocated monotonically and never reused, so recording a span for a branch that is later abandoned is harmless. Move `SpanIndex` out of `ParseProvenanceState` into a single append-only accumulator threaded beside it; a memo entry then stores only its frame (the ids), and a hit adopts nothing. Expected: the 60% above, with no snapshot cost. Open question for review: whether the final index may carry entries for abandoned ids, or whether those are filtered once at the end.
2. **Census only the entry's closure.** Walk references from the entry and parse files as the walk reaches them, so cost scales with the closure, not the corpus (§2). Also replace `closure_ingest_declares`, a linear scan per file, with the `index` it already builds.
3. **Move, don't clone, into `map_insert`.** The emitted `index.entries.clone()` from a borrowed parent defeats `Rc::make_mut` at every update site, so persistent maps are path-copied even when unshared. This is an emitter ownership defect (clone-vs-move at value use sites), shared by every v2 accumulator.
4. **Keep one tree per file.** `ClosureIngestFile` holds both the parse tree and the census tree (≈ 0.5 GiB on the 97-file root).
5. **A memory floor for the native door.** The `v2-native-cli` instrument only walks a tiny fixture root, which is why none of this surfaced. Enrol one real entry with a declared peak budget so a regression fails loudly.

## Reproduction

- Build generation one: `gunbc test //gunbc/instruments:v2-native-cli` (prints the kept `v2-native-cli-gen-one-<sha>` path).
- Peak and time: `/usr/bin/time -f "%e %M" <gen-one> emit --entry nope.x --source-root <dir>` with a single-file directory per sample.
- Attribution: `perf record -e page-faults -c 1 --call-graph dwarf` on the same command (needs `kernel.perf_event_paranoid ≤ 1`); `heaptrack` records counts and size histograms but did not unwind on this aarch64 host.
- Probes: copy the emitted crate while the instrument's cargo build runs (it is deleted afterwards), edit the named site, `cargo build --release --offline`.
