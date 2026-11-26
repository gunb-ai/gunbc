# What matched the predictions

The observer compares the ordinary compiler's complete `NewlineIndex` with an independently stored answer digest. Every invocation still executes the producer and evaluates the entry function. The original predictions and the failed diagnostic assertion are preserved so that the result can be checked against the expectations in force at the time.

## Fresh processes

The two-file case ran under the repository's default dev build on Linux x86_64. The table gives each process's elapsed time. Numbers include the entire ordinary command. It hashed a roughly 70 MB executable once per enabled process. The small timings are single observations with no isolated measurement of the newline-index producer.

| Change | Entry | Helper | Exit | Wall |
| --- | --- | --- | ---: | ---: |
| Empty store | Miss | Miss | 0 | 1.834 s |
| Repeat | Equal hit | Equal hit | 0 | 1.680 s |
| Edit an unrequested file | Equal hit | Equal hit | 0 | 1.896 s |
| Change helper literal from 1 to 2 | Equal hit | Miss, changed answer | 7 | 2.655 s |
| Same changed helper with observation disabled | No observation | No observation | 7 | 0.050 s |
| Revert the literal | Equal hit | Equal hit | 0 | 2.721 s |
| Empty second store, same machine | Miss, original answer | Miss, original answer | 0 | 1.728 s |
| Plant the valid digest of the edited answer under the original key | Equal hit | Unequal hit | 0 | 1.913 s |
| Damage that stored record | Equal hit | Corrupt | 0 | 1.891 s |

The two failure controls preserved the planted bytes. The fresh helper answer in both cases matched the independently established original. There were 16 observations, a maximum reported RSS of 32,148 KiB, and zero producer calls avoided. [Per-case measurements](observed/newline-index.json)

Adding a second file declaring the demanded module under a second admitted root changed both warm and cold commands to exit 1. Each refusal named both providers, and neither process reached the observed compiler question. Removing the extra declaration restored both original hits and the original command outcome. The first comparator also expected every byte of stderr to agree. That expectation failed because the timing and RSS fields varied. The permanent test compares status, stdout, the complete located cause and probe events. [Provider cases](observed/competing-provider.json)

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

## Running the behavioral assertions

Run the integration tests from the repository root in its supported build environment.

```sh
cargo test --locked -p v1-compiler --test materialization_shadow --test materialization_shadow_questions -- --nocapture
```

The fixtures create temporary source roots and start fresh compiler processes. The emitter and the internal comparisons run directly against the compiler library. The front-end observer is opt-in via `GUNBC_MATERIALIZATION_SHADOW_DIR`, an absolute path. Journal entries are 64 bytes. The field `bytes` in the trace is the size of the canonically encoded fresh answer.
