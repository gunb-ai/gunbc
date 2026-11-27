# S1/S4 cache-key fixture: predictions, before any observation

These predictions are written and committed **before** the fixture runs. They are left
unedited afterwards; corrections are recorded beside them in
[cache-key-observed.md](cache-key-observed.md), never in place of a prediction.

## Provenance discipline (parse-lane constraint)

#17177 changes v1 source-to-chars conversion, which feeds `build_newline_index`. Every
digest or measurement recorded for this fixture names the `gunbc` commit it was taken at.
The predictions below are design predictions and carry no digest, so they predate any
particular front end. If #17177 (or any front-end change) lands mid-observation, the
affected observed rows are re-run at the new sha; readings are never spliced across that
change in one row. The fixture never edits `v1.std.core`'s char conversion or
`build_newline_index`; it only observes.

## Working definitions (recorded with the predictions, before observation)

A *consumer* is one source file whose own bytes are held fixed across the decisive
scenario. A *provider* is a previously absent, valid module that changes what one of the
consumer's references resolves to. The fixture asks about **the cache key for a
front-end answer that depends on a resolved reference**, keyed two ways:

- **S1 bytes-only key** — derived only from the consumer: SHA-256 over the compiler image,
  the consumer's logical path, and the consumer's bytes. This is the shape the production
  `build_newline_index` observer already uses (`input_key`). It does not see any other
  module in the tree.
- **S4 corrected key** — derived from the consumer plus a **sorted**, source-derived set
  of the reference/dependency facts the answer depends on (so a newly present provider
  changes the set, and therefore the key). The set is iterated in **sorted order**, never
  `HashSet` iteration order, so the key is reproducible across processes.

The decisive scenario holds the consumer bytes unchanged and adds a **distinct** valid
provider module declaring the same leaf the consumer references, which changes that
reference's resolution from a single owner to ambiguous. A *duplicate* module (a second
declaration of the same module path) is refused before the probe runs
(`a_new_provider_under_an_admitted_root_refuses_before_shadow_observation`), so a duplicate
is not the right provider — it never reaches S4.

The experiment never supplies an answer to the compiler: the compiler always recomputes
fresh; the store only observes.

## Predictions (hit / miss / disagree), per key

| id | Scenario | S1 bytes-only key | S4 corrected (sorted) key |
| --- | --- | --- | --- |
| S1a | Cold, consumer only, empty store | **miss** (absent), store fresh answer | **miss** (absent), store fresh answer |
| S1b | Warm repeat, identical inputs | **hit**, **equal** | **hit**, **equal** |
| S4c | Determinism control: same tree keyed twice, in two separate processes | (not applicable) | the two computed keys are **equal** (sorted order is deterministic) |
| S4d | Add the valid provider, consumer bytes unchanged — the decisive case, bytes-only key | **hit** is intact (the key did not move) **but the stored answer disagrees with the fresh answer** → `hit/different` (the red) | (see S4e) |
| S4e | Same provider, corrected key | (not applicable) | provider changes the sorted set → **miss** → answer **recomputed** to the fresh, changed value; a warm repeat with the same provider set is a **hit, equal** |
| S4f | Body-only edit that leaves the consumer's references unchanged | key moves (bytes changed) → miss → recompute; recomputed answer **equal** to baseline | references unchanged → key **stable** → **hit, equal**; recomputed answer **equal** |
| S3g | Duplicate module as the "provider" (control) | refused **before** S4 runs; no probe observation | refused **before** S4 runs; no probe observation |

### What each prediction is testing

- S4d is the load-bearing claim in the task title: under a bytes-only key the stale stored
  answer is still *intact* (valid integrity bytes) yet *wrong*, because the consumer's
  reference now resolves differently. A naive reuse would serve the wrong answer.
- S4e/S4c say the corrected sorted key both notices the provider and is reproducible across
  processes.
- S4f confirms the corrected key is not over-sensitive: an edit that leaves the reference
  set unchanged recomputes to an equal answer.
- S3g confirms the duplicate module is the wrong tool: it is refused before the probe, so it
  cannot produce the red.

## Open design question recorded with the predictions

The exact front-end *answer* this fixture keys (leaf/reference resolution vs. a wider
resolved-reference digest) and whether the corrected key is introduced inside the production
observer or as a parallel probe are recorded here as the one design choice being confirmed
with the owning session. The hit/miss/disagree structure above is independent of that
choice and is the prediction being preserved.

## Rename control (added after review 99286; committed before running)

`symbol_census_key` binds (module path, source bytes); it never binds the logical source-file
name. But `build_symbol_index_census` consumes the per-file `NewlineIndex` map (keyed by file) and
the resolved graph carries source-located nodes, so the `SymbolIndex` the census returns may
record the file identity. A cached answer keyed without file identity could then be reused when
the file is renamed.

Scenario R1: rename the consumer file `helper.dag` -> `renamed.dag`, keeping the bytes and the
`module census.helper` declaration identical, then recompute under the S4 key with the same store.

| id | Scenario | S4 corrected key | census answer digest |
| --- | --- | --- | --- |
| R1a | Cold, consumer as `helper.dag`, empty store | miss (absent), store fresh answer (the baseline digest `89f2a524…`) | baseline |
| R1b | Rename to `renamed.dag` (identical bytes + module), same store | **unchanged** (module path + bytes unchanged; the key never sees the file name) | **PREDICTION: changes** — the `SymbolIndex` carries `index.file` and source-located nodes, so a different file name yields a different canonical answer |

Consequence predicted for R1b: the intact record under the unchanged key now disagrees with the
fresh answer → `hit/different`. That is a cache-key defect: the key omits a source-file identity
the answer depends on. If instead the fresh answer digest comes out **equal**, the key is correct
as-is and this becomes a regression guard rather than a defect.
