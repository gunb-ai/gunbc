# S1/S4 cache-key fixture: observations, beside the predictions

Recorded **beside** [cache-key-predictions.md](cache-key-predictions.md); the predictions
are left unedited. This file records what was actually observed.

## Provenance of these readings

Taken on this branch (merged onto `origin/main` after #17126 landed as `16dfccbdf90`; merge
commit `cef11d4c797`) plus this PR's key/test changes. #17177 (the v1 source-to-chars /
newline-index change) is **not** in `origin/main`, so the census front end these readings describe
is unchanged from the #17126 base; no rows are spliced across #17177. The fixture is dev-profile.
These keys were re-recorded after a review fix that routes the key's compiler-image term through
`transform_content_digest` (the single authority) rather than re-reading the executable; the census
answer digests are unaffected by that change.

After consolidating the stored-record format into a single seed authority in
`closure_identity` (`shadow_record`/`shadow_read_prior`/`shadow_compare_and_record`, commit
`6b2be4127d0`) the fixture was re-run on that head and stayed green: every row **relation** below
reproduced — E1 key == E2 key (the bytes-only red), F1→F2 key move, F3 == F2 (warm hit/equal),
D1 == D2 (cross-process determinism), G2 key moved with answer equal to G1, H refused before any
record — and the build-stable census answer digests (`89f2a524…`, `c7820b86…`) were unchanged. The
left-hand key hexes are build-specific (they embed the running-image identity, as noted below); the
record-format change does not touch the key preimage, so only a rebuild that re-links the binary
moves them, exactly as stated.

Two key facts about the numbers:

* The **census answer digest** (right-hand column) depends only on the compiled answer, so it is
  stable across builds: `89f2a524…` is the consumer-only census, `c7820b86…` is the census with a
  competing `Banner` provider. A reference-preserving body edit reproduces `89f2a524…`.
* The **key** (left-hand column) embeds the compiler-image identity digest consumed from
  `transform_content_digest` (the single authority), so the key hex changes when the binary is
  rebuilt; within one build the key is byte-identical across separate processes (the determinism
  control). The values below are all from the single build recorded above, so they are directly
  comparable to each other.

## Observed table

| id | Scenario | key derivation | observed lookup/comparison | key (input_digest) | fresh answer (output_digest) |
| --- | --- | --- | --- | --- | --- |
| E1 | Cold, consumer only, empty store | S1 bytes-only | **miss / absent** | `c5ea7c98…e4f047` | `89f2a524…648d3` |
| E2 | Provider added, consumer bytes unchanged | S1 bytes-only | **hit / different** | `c5ea7c98…e4f047` (unchanged) | `c7820b86…85e43` |
| F1 | Cold, consumer only | S4 corrected | miss / absent | `efaa375d…abb0b8` | `89f2a524…648d3` |
| F2 | Provider added | S4 corrected | miss / absent (recomputed) | `fa7fd4cf…4b04a7` (moved) | `c7820b86…85e43` |
| F3 | Warm repeat, same provider tree | S4 corrected | **hit / equal** | `fa7fd4cf…4b04a7` | `c7820b86…85e43` |
| D1 | Same provider tree, process A | S4 corrected | miss / absent | `fa7fd4cf…4b04a7` | `c7820b86…85e43` |
| D2 | Same provider tree, process B | S4 corrected | hit / equal | `fa7fd4cf…4b04a7` (equal to D1) | `c7820b86…85e43` |
| G1 | Consumer only, literal `1` | S4 corrected | miss / absent | `efaa375d…abb0b8` | `89f2a524…648d3` |
| G2 | Body edit `1`→`2`, references unchanged | S4 corrected | miss / absent (recomputed) | `c45e6ab3…f07a67` (moved) | `89f2a524…648d3` (**equal to G1**) |
| H1 | Duplicate module, S1 | S1 bytes-only | refused / duplicate-module | `c5ea7c98…e4f047` | — (no answer, no record) |
| H2 | Duplicate module, S4 | S4 corrected | refused / duplicate-module | `b4a3896e…469633` | — (no answer, no record) |

## What was observed, against each prediction

* **The decisive red (E1→E2).** Holding the consumer bytes fixed and adding a previously absent,
  valid provider module (`census.other` declaring a competing `Banner` leaf) left the S1 bytes-only
  key **unchanged** (`c5ea7c98…` in both steps). The stored record was **intact** (valid 64-byte
  integrity) and held the pre-provider answer `89f2a524…`, while the fresh answer after the
  provider is `c7820b86…`. The observation is `hit / different` — a naive bytes-only reuse would
  serve a wrong answer. This matches the prediction exactly. The provider is genuinely
  resolution-dependent: the leaf-to-owner resolution for `Banner` goes from `SingleOwner` to
  `LeafAmbiguous` (asserted in the test).
* **The corrected key notices the provider (F1→F2→F3).** The S4 key moved from `efaa375d…` to
  `fa7fd4cf…` when the provider was added, so it **missed** and recomputed to the changed answer
  `c7820b86…` (distinct from the cold `89f2a524…`), and a warm repeat in another process was a
  `hit / equal`. Prediction matched.
* **Determinism (D1=D2).** The same provider-bearing tree keyed in two separate processes produced
  the identical S4 key `fa7fd4cf…`. This is the sorted-set control the reviewer asked for: no
  `HashSet` iteration order leaks into the key.
* **Body-only edit (G1→G2). Prediction S4f FALSIFIED.** The committed prediction expected the
  corrected S4 key to stay stable under a reference-preserving edit — "references unchanged → key
  **stable** → **hit, equal**". The observation is different: the same-width literal `1`→`2`
  **moved** the S4 key (`efaa375d…` → `c45e6ab3…`), so the lookup was a **miss / absent** (not a
  hit) and the answer was recomputed. What *did* hold is the load-bearing part the brief requires:
  the recomputed answer is **equal** to the baseline (`89f2a524…`). Correction recorded here,
  beside the unchanged prediction: `symbol_census_key` **over-approximates** its dependency set — it
  binds every discovered module's full source **bytes**, not the reference/dependency facts the
  answer depends on. So it is never wrong (it can only miss, never serve a stale answer) but is less
  reuse-efficient than the prediction assumed: any byte change inside the closure busts the key, even
  a reference-preserving body edit. The prediction over-claimed key stability; that claim is false.
* **Duplicate provider (H1/H2).** A second file declaring the same module path was refused
  (`duplicate-module`) **before** any census answer was computed, for both key derivations, with no
  record written. Prediction matched; a duplicate is not the right provider for the red.

Except for **S4f**, every observed hit/miss/disagree matched the committed predictions. One
correction is recorded above, beside the prediction rather than in its place: S4f claimed the
corrected S4 key stays stable under a reference-preserving body edit, but `symbol_census_key` binds
full source bytes, so the key moved and the lookup was a miss (the recomputed answer was still
equal). The predictions file is left unedited.
