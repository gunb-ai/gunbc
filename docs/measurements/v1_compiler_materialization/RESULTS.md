# What matched the predictions

In this initial capture, before explicitly requested observations were made mandatory, the observer compares the ordinary compiler's complete `NewlineIndex` with an independently stored answer digest. Every invocation that reached the observer executed the producer and evaluated the entry function. See the [current acceptance criteria](README.md#current-acceptance-for-explicitly-requested-observations) for the behavior now expected on bad evidence. The original predictions and the failed diagnostic assertion are preserved so that the result can be checked against the expectations in force at the time.

## Fresh processes

The same two-file experiment ran under the repository's dev and release profiles on Linux x86_64. Both builds matched the same predeclared outcomes. The observer hashes the running executable once per enabled process. The table gives elapsed time for the entire command. Each timing is a single observation.

| Change | Entry | Helper | Exit | Dev wall | Release wall |
| --- | --- | --- | ---: | ---: | ---: |
| Empty store | Miss | Miss | 0 | 1.834 s | 95.4 ms |
| Repeat | Equal hit | Equal hit | 0 | 1.680 s | 67.2 ms |
| Edit an unrequested file | Equal hit | Equal hit | 0 | 1.896 s | 82.6 ms |
| Change helper literal from 1 to 2 | Equal hit | Miss, changed answer | 7 | 2.655 s | 70.8 ms |
| Same changed helper with observation disabled | No observation | No observation | 7 | 0.050 s | 18.3 ms |
| Revert the literal | Equal hit | Equal hit | 0 | 2.721 s | 75.6 ms |
| Empty second store, same machine | Miss, original answer | Miss, original answer | 0 | 1.728 s | 69.8 ms |
| Plant the valid digest of the edited answer under the original key | Equal hit | Unequal hit | 0 | 1.913 s | 72.8 ms |
| Damage that stored record | Equal hit | Corrupt | 0 | 1.891 s | 66.2 ms |

The two failure controls preserved the planted bytes. The fresh helper answer in both cases matched the independently established original. Each profile produced 16 observations and avoided zero producer calls. Reported peak RSS across the nine runs was 32,148 KiB in dev and 20,792 KiB in release. The 52.53 ms difference for the release edited-input pair includes all observer overhead. [Dev measurements](observed/newline-index.json) · [Release measurements](observed/newline-index-release.json)

Adding a second file declaring the demanded module under a second admitted root changed both warm and cold commands to exit 1. Each refusal named both providers, and neither process reached the observed compiler question. Removing the extra declaration restored both original hits and the original command outcome. The first comparator also expected every byte of stderr to agree. That expectation failed because the timing and RSS fields varied. The permanent test compares status, stdout, the complete located cause and probe events. [Provider cases](observed/competing-provider.json)

## A timing prediction that failed

Five files were selected before measurement by sorting the SHA-256 digests of their tracked paths at the historical revision. The checkout guard's own file was the sixth control. Across 18 fresh harness processes and 576 calls, all complete fresh answers agreed. We predicted that streaming the canonical answer through SHA-256 would take longer than constructing the index for at least four of the six files. It was faster on all six. On this dev build the median hashing time was 89–97% of the median producer time. For the guard source the medians were 5.413 ms and 6.060 ms respectively, on an input of 25,833 bytes and a canonical answer of 210,128 bytes.

A second, frozen forecast used the dev result to predict the optimized build would hash faster on at least four sources. It did so on six of six over a further 576 calls, and all six answer digests matched the dev results. Hashing now took 20.2–26.7% of fresh construction. For the guard source the release medians were 0.357 ms for hashing and 1.469 ms for construction. The direction transferred to a new build profile. The cost ratio changed substantially.

The first forecast remains a failure. A serving benchmark still needs transfer, validation and reconstruction of usable values, plus independent fresh-result and command-outcome checks. [Selection, prediction and measured costs](observed/newline-index-cost.json)

## Independent compiler outputs

| Controlled change | Fresh outputs that remained equal | Fresh outputs that changed |
| --- | --- | --- |
| Change an integer literal from 1 to 2 | Newline positions, interned identifiers and their indices, authored token ordinals, complete occurrence transport for this fixture, normalize diagnostics, reconciled leaf owner | Token stream and full newline index |
| Prepend another declaration-bearing file ahead of the same helper | Original helper bytes, complete helper newline index, ordered helper names and their locations | Numeric occurrence IDs for those same helper locations |
| Add a second module that owns the same declaration leaf | Original declaration remains in its first module | The leaf lookup changed from one owner to ambiguous |
| Change only a helper body, preserving its exported type | Helper interface summary, caller checked items and caller Rust file bytes | Helper checked items and helper Rust file bytes |
| Change helper return type from Int to String, then repair the caller | Before repair, the caller's source is identical | Helper interface, caller acceptance and repaired caller's Rust file bytes |

The first type/emit trial matched 12 of its 13 comparisons. The compiler produced the expected caller error and spelled the diagnostic values `Primitive(Int)` and `Primitive(String)`. The original comparator used `Int` and `String`. Correcting that assertion gave 13 of 13 on the same inputs. Rust emission here is an explicit compiler request. The interpreted command does not request Rust emission. [Original prediction and observed correction](observed/compiler-questions.txt)

## Compiler rebuild

Two clean builds of the actual seed used the same Git tree and an empty commit between them. The embedded commit and executable digest changed. A candidate identity constructed from the selected Cargo build inputs, configuration and toolchain stayed equal. It covered 150 Cargo units, 4,170 input files and 40 toolchain files. Both physical images independently returned the same 1,214-byte result. The current image-keyed observer recorded a miss after rebuilding, as predicted. A stale image paired with a later checkout failed the build-manifest check. [Before/after identities and readbacks](observed/compiler-rebuild.json)

This validates one provenance-only rebuild for this question and host. A production identity also needs the build inputs that can alter behavior on its admitted hosts, including native compiler and linker inputs, build-script environment and target ABI. A question that reads revision metadata would include it among its own inputs.

The producer-side capture now pairs each measured build with the image reported by the running compiler. Edits outside the observed build inputs preserved the candidate. Changing one character mapping in the newline-index producer changed both the candidate and the actual answer while the fixture continued to pass. Pairing a successful run with the other build refused on its image digest even though both answers agreed. Rebuilding the original head restored the candidate, answer, and executable bytes. The active shadow key still uses the physical image. [Predictions, controls, and captured identities](observed/compiler-executable-binding.json)

## Running the behavioral assertions

Run the integration tests from the repository root in its supported build environment.

```sh
cargo test --locked -p v1-compiler --test materialization_shadow --test materialization_shadow_questions -- --nocapture
```

The fixtures create temporary source roots and start fresh compiler processes. The emitter and the internal comparisons run directly against the compiler library. The front-end observer is opt-in via `GUNBC_MATERIALIZATION_SHADOW_DIR`, an absolute path. Journal entries are 64 bytes. The field `bytes` in the trace is the size of the canonically encoded fresh answer.

## Evidence required from a second worker

The shared native-product transport currently admits a key directory containing an executable and its manifest. The new local experiment carries an answer digest and its integrity check. Its 64 bytes cannot reconstruct a `NewlineIndex`. The existing transfer and verification machinery is a precedent for carrying a separately declared intermediate.

A two-host experiment should link four independently recorded facts. The producer computed the question fresh with an empty local store. The shared store accepted a specific object version with a known byte digest. A consumer on a distinct physical host fetched that version with an empty local store. The consumer computed the same question fresh and compared the complete result. Demonstrating useful serving adds decoding the transferred value and passing forced-cold parity on the checkout guard. An unavailable or damaged object must leave an attributable refusal or fresh fallback. Two jobs scheduled on one machine do not establish the host boundary.

Historical runs exercised this distinction for a different product. One native compiler was produced on srv3 and restored on srv1, where it passed the same clean and malformed-input controls. In that same receiving job, a different restored executable tried to launch a toolchain path that existed only on its producer. Successful transfer and portable execution therefore get separate assertions. [Producer job](https://github.com/gunb-ai/gunbc/actions/runs/37905841741/job/113738968609) · [Consumer job](https://github.com/gunb-ai/gunbc/actions/runs/37927173506/job/113808803642)


## Checkout revision as a live input

The real `verify_checkout_at_plan_revision` function used the original plan receipt from run 45722153952 (artifact 15077460698). A matched checkout at `a58ba3e3` and an empty commit at `0f36c96f` have the same file tree. Each fresh process received the same native release binary, arguments and logical paths and read the receipt from its production path. Its observation log recorded the actual receipt read and Git revision lookup.

| Process | First observation of each key | Checkout verdict | Wall, s | CPU, s | Peak RSS, MiB |
| --- | --- | --- | ---: | ---: | ---: |
| Empty store, matching HEAD | 587 miss | exit 0 | 114.675 | 112.670 | 2,287.9 |
| Populated store, matching HEAD | 587 hit | exit 0 | 78.675 | 78.204 | 2,288.4 |
| Populated store, different HEAD | 587 hit | `CheckoutHeadMismatch`, exit 1 | 76.021 | 75.640 | 2,289.1 |
| Independent empty store, different HEAD | 587 miss | `CheckoutHeadMismatch`, exit 1 | 80.136 | 79.433 | 2,293.7 |

All four runs had the same 587 question, source, input and fresh-answer associations. Each computed 111,263,042 bytes of canonical answers; each complete journal occupied 37,568 bytes of file contents. The cold and warm refusals named the same expected and observed revisions. All four commands recomputed their answers. Single-run wall times reflect process scheduling and system caches, including overlap with other measurements.

The separately downloaded historical release binary also read the original receipt and Git HEAD and returned the located mismatch. That arm used ARM64 emulation. A preliminary local dev replay was stopped in the front end after approximately 32 minutes. Both are excluded from the native release timing table.

[Predeclared checkout cases](observed/checkout-guard-predictions.json) · [Release verdicts, phase times and independent host readbacks](observed/checkout-guard-release.json)

## The next reconciliation question on the checkout guard

A separate profiled cold run executed the matching checkout guard on the same native release image. It returned 0 and independently observed 587 initial misses. Sampling at 10 ms intervals gave these inclusive CPU times. The pass marks reported 32 seconds in the front end and 26 in reconciliation for that run.

| Named reconciliation candidate | Sampled inclusive CPU seconds |
| --- | ---: |
| `build_symbol_index_census` | 10.18 |
| `build_module_context` | 1.34 |
| `build_type_env` | 0.99 |
| `infer_items_demanded` | 0.22 |

The symbol census ranked first among the ten preregistered candidates for this invocation. The earlier ranking that placed module-context construction first came from a different workload. Inclusive samples overlap and can omit inlined frames. These times identify a candidate to test and cannot be added as independent savings. [Profile and complete candidate list](observed/checkout-guard-profile.json)

Within the census, 6.66 sampled seconds were attributed to resolving function signatures (`census_with_resolved_fn_sigs`) and 3.52 to collecting the raw nodes. Eleven predeclared comparisons on the actual eight-field `SymbolIndex` then exercised what that result preserves. Changing `1` to `2` in a helper body preserved the full census. Changing it to `12345` preserved the qualified names but shifted the following declaration’s span from [46, 50) to [50, 54). The `entries` and `global_bare` fields changed, and the other six fields remained equal. Adding one `Banner` type introduced a unique bare binding. Adding another in a different module made the bare name ambiguous while both qualified bindings remained available. A key that captures only the qualified name set is insufficient for consumers of this complete value. [Census predictions](observed/symbol-census-predictions.txt)
