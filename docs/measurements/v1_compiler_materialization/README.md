# First fresh-process shadow experiment

The first observation is `v1.std.core::build_newline_index` as consumed on ordinary `gunbc run`. The result contains the logical file name, newline offsets (in Unicode scalar positions) **and all source character codes**. An ASCII edit `1` → `2` leaves every newline offset fixed and still changes this result. This is a discriminating test of the proposed result identity.

## Current acceptance for explicitly requested observations

When `GUNBC_MATERIALIZATION_SHADOW_DIR` is set, a cold miss records the fresh answer and an equal hit confirms it. Both permit entry evaluation. A valid prior that disagrees emits `hit/different`. Damaged bytes emit `corrupt/unavailable`. Either event refuses the invocation before entry evaluation and preserves the record for inspection. A directory or compiler-identity error, incomplete observation, or trace delivery failure also refuses the invocation. With the variable unset the entry runs normally.

The predictions and captured results below describe the initial observation-only experiment. The integration test `fresh_process_predictions_match_observed_compiler_results` exercises the stricter acceptance above.

### Binding an observation to its producer

Each event includes `compiler_image_sha256`, the same image digest the observer uses in the question key. On Linux the observer opens `/proc/self/exe`. The fresh-process harness compares the events to the SHA-256 of the executable delivered by its own build. An empty trace cannot establish the pairing. The negative control executes a byte-distinct copy with identical code and compares the two fresh answers before requiring rejection of the image pairing. Set `GUNBC_SHADOW_STALE_CONTROL_IMAGE` to the absolute path of an independently built, image-reporting seed to run the same control across real builds.

A producer may compare the image in this trace with the image it recorded at the end of a controlled build. The separately measured Cargo source/configuration candidate is experimental. Its dependency coverage and the original two-build result are described in [the compiler-rebuild measurement](RESULTS.md#compiler-rebuild). The executable digest remains the compiler input to the shadow key.

## Predictions, before observations

Use one private workspace, the same compiler executable, two reached source files (entry and helper), and a fresh OS process for each invocation. Each edit starts from the named baseline. Compare the same logical file and compiler question across processes. Start with an empty private shadow directory. No production read of a saved result is permitted in this experiment.

| Condition | Entry newline index | Helper newline index | Command behavior |
| --- | --- | --- | --- |
| Empty directory | Miss, fresh | Miss, fresh | Executes the selected entry |
| Identical inputs | Hit, equal to fresh | Hit, equal to fresh | Same return and output |
| Edit only a file outside the entry's demands | Hit, equal | Hit, equal | Same return and output. No index observation for the unrelated file |
| Change the helper's literal `1` to `2` | Hit, equal | Miss, fresh output differs from baseline | Caller observes the new value, including with shadow disabled |
| Revert the helper | Hit, equal | Hit, equal to original | Original outcome |
| Start with a second empty store on the **same** host | Miss, fresh output equal to baseline | Miss, fresh output equal to baseline | Original outcome. This is the missing-record control, not a second-worker result |
| Damage the previously stored helper record | Hit, equal | Refuse that stored record | The ordinary compiler still supplies the fresh answer in shadow mode |

A hit with a different fresh result fails the experiment, regardless of the predicted outcome. A miss is a lookup fact. Calling its recomputation equal requires a comparable, independently produced baseline result. A compiler stage the request never reached has no hit/miss outcome. This experiment predicts no Rust emission on the interpreted `run` path.

The integration test owns the executable assertions. The final criterion for a shared store is a real second worker with an empty local store, admitted shared bytes, fresh execution of the checkout check, and a deliberately mismatched checkout refusal. Copying a directory on one host cannot establish that criterion. For a later serving change, compare ordinary outcomes to forced-cold outcomes and measure full wall time including lookup, verification and hydration. Shadow mode recomputes every product; producer calls avoided in this experiment are zero.

## Independent compiler-section predictions

For the next bounded comparison, call the existing stage0 front end and normalizer with a single input `fn answer() -> Int { 1 }`, then replace the same-width literal with `2`. Compare the actual answers the compiler returns on those inputs. This experiment asks about *answer equality*. It does not claim a store lookup for questions with no shadow hook.

| Compiler question | Candidate dependency or context | Prediction for `1` → `2` |
| --- | --- | --- |
| Lexing | Source bytes, logical file, grammar | Token answers differ in the literal |
| Complete newline index | Logical path and source bytes | Offsets equal, character codes differ |
| Intern table after parsing | Ordered internable token spellings, initial table, parse context | Equal. Integer literals are neither identifiers nor keywords |
| Authored token ordinal allocation | Initial allocator state, ordered parse shape and allocation decisions | Equal for this fixed-shape edit |
| Normalization diagnostics | Resolved module, source index and normalizer semantics | Equal (both empty). This predicts only the diagnostic projection, not equality of the entire graph |
| Leaf-to-owner index produced by reconciliation | Set of `(declaration leaf, owning module)` pairs | Equal; add `other` with another `answer` and the same leaf should become ambiguous |

The signature and checked body of `answer` are separate proposed questions. A body-only change can leave the former equal while changing the latter. Rust emission needs an explicit emit request; `gunbc run` provides no emission sample. A full cross-worker experiment additionally needs the producer/toolchain and result-format compatibility established for that host.

## Deliberately wrong prior answer

After the original helper has been restored, place the digest of the real edited helper's answer under the original input key, with a valid integrity check for that key. The caller should still report an equal hit. The helper should report a hit with a different fresh answer, return the original fresh answer digest, and preserve the disagreeing record for inspection. The program should exit 0. Flipping an integrity byte in that same record should instead report a corrupt observation and the same program result. This control tests whether a wrong prediction is observable through the CLI, even when command success alone would hide it.

## An unchanged file with a different occurrence allocator

On a full fresh front-end call, changing the helper's one-digit integer literal should preserve the complete authored occurrence transport for this fixed-shape edit. Prepending a second file with a named declaration should move the helper's occurrence IDs, even though its bytes, logical path and full newline index stay equal. Compare the same ordered occurrences in the helper by authored spelling and source span. The second prediction challenges using file contents alone as the key for occurrence allocation.

The recorded observations, including the first mismatched diagnostic expectation, are in [RESULTS.md](RESULTS.md).

## Cache-key fixtures for the compiler reuse series (S1 / S4)

These fixtures decide what a compiler-result cache key must bind, for the question
`v1.symbol.index::build_symbol_index_census` (the census the serving lane reuses). Two key
derivations are compared:

* **S1 — bytes-only key**: `SHA-256(running compiler image, module path, module source bytes)`.
  The wrong key under test: it binds only the consumer file, not the declarations its references
  resolve to.
* **S4 — corrected key**: the complete key over a **sorted, source-derived** dependency set —
  the compiler image, the sorted closure roster, and every discovered module path with its source
  bytes. One authority: `v1_compiler::closure_identity::symbol_census_key` (seed library, declared
  in `dag/gunbc/materialization_shadow_seed_growth.dag`).

Scenario ids used in the harness: **E** the bytes-only red (provider added, consumer bytes fixed →
intact hit whose answer disagrees); **F** the corrected key notices the provider and agrees on a
warm repeat; **G** a body-only, reference-preserving edit recomputes to an equal answer; **H** a
duplicate module is refused before any answer; plus a determinism control (the same tree keyed in
two separate processes gives the same key).

Predictions (committed before any run, left unedited) are in
[cache-key-predictions.md](cache-key-predictions.md); the observations recorded beside them are in
[cache-key-observed.md](cache-key-observed.md). The harness is
`src/v1/stage0/tests/materialization_shadow_cache_key.rs`.
