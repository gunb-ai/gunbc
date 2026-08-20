# materialization_carriers arm A: 41 at `main@4cec10f66a`, emitter WITHOUT #8614 (2026-08-20)

**Result: 41 error diagnostics, 34 distinct `(code, primary span, message)`.**

**Session:** `swift-moth-294` (sole write ownership of `src/v1/05_emit_rust.dag` /
`trait_derive_emit.dag` and their stage0 projections), dashboard node `adhoc-497a24aa-140`.
**Subject:** `src/v2/compiler/materialization_carriers.dag`.
**This does not supersede** `docs/probes/materialization_carriers_rebaseline_2026-08-19.md`
(`eager-ant-366`, the measurement authority for this module). It is a *second arm* taken for a
different purpose — see §5 — and is filed beside that document rather than replacing it.

## 1. Why this arm exists, and why it was only available today

#8614 changed `src/v1/05_emit_rust.dag` and **did not regenerate its stage0 mirror**. Every other
emitter change since the 51-era baseline regenerated in the same commit (measured: 7 commits touch
the authority, 7 touch the mirror, and only #8614 is unmatched). So `main` today builds a `gunbc`
whose emitter contains #8528 and #8570 but **not** #8614.

That is a *clean earlier emitter*, not a corrupt one — a distinction worth stating because
"the mirror does not match the authority" is easy to read as "the mirror is corrupt", and the two
states have opposite consequences. It makes a **pre-#8614 arm** available that disappears
permanently once the regen closure (#8652) lands:

| arm | emitter | attributable |
|---|---|---|
| **A** (this document) | #8528 + #8570, **no** #8614 | — |
| **B** (after #8652) | + #8614 | **B − A = #8614 alone** |

#8614 is a change that can move the population in *both* directions at once — a new compile-time
refusal can create sites while deep type-surface collection retires others — so a single
post-regen measurement cannot separate creation from retirement. Two arms can.

## 2. Instrument

Unchanged from the predecessor document §1 (single authority, not re-derived): `gunbc compile` →
`cssl_assemble` → `cargo build --release --lib --message-format=json`, `CSSL_STD_SEED_LINK=1`,
manifest rendered by `docs/probes/lib/render_cssl_probe_lib_cargo_toml.sh`. Counts are error-level
diagnostics as rustc reports them in the JSON stream, at `(code, primary span, message)` grain —
never a keyword scan.

One dispatch, remote (BuildBuddy), binaries built from the measured tree in the same dispatch, so
the binaries and the tree cannot disagree.

## 3. Provenance carried *inside* the run

Printed before any build, so the run proves which emitter it measured rather than relying on this
document to assert it:

```
HEAD: 4cec10f66a3d84cbde631e20ca9feaa31457d88c
authority 05_emit_rust.dag  marker count: 1
mirror v1_compiler_emit_rust.rs marker count: 0
guard passed: mirror marker count = 0
built gunbc marker count: 0  (arm A expects 0)
positive control (string table is readable): 25 'materialization' hits
```

The marker is #8614's refusal string (`is neither a resolved callable receiver field nor a
registered v1_rt bridge function`). It discriminates cleanly: authority 1 / mirror 0 at
`origin/main`, authority 1 / mirror 1 at `pull/8652/head`. The **binary** check is the load-bearing
one — it is freshness, not existence, and it would have aborted the run had the built compiler
contained #8614.

Frontend, same run: `indexed 3733 modules from 2 source roots`, `resolved 48 sources (transitive
import closure)`, `compiled: 52 files emitted, 147 diagnostics` (advisory), `CSSL_ASSEMBLE: PASS`.

## 4. The count

```
manifest bytes: 455
cargo exit: 101
json stream lines: 188
TOTAL error diagnostics: 41
DISTINCT (code, primary span, message): 34
```

| code | n | shape |
|---|---|---|
| E0308 | 15 | mismatched types |
| E0277 | 13 | trait bound not satisfied (`Clone`, mostly) |
| E0599 | 4 | no method named `clone` on a type parameter |
| E0425 | 3 | cannot find type `NonEmptyStr` |
| E0369 | 2 | binary `==` on `Rc<im::Vector<T>>` |
| E0310 | 2 | `impl Fn(..) + Clone` may not live long enough |
| E0560 | 1 | `Rc<Measure<(), (), i64>>` has no field named `count` |
| E0282 | 1 | type annotations needed |

## 5. Two cross-checks that the instrument agrees with #8528's

Neither was set up in advance; both could have failed.

- **E0422 is absent.** #8528 removed exactly 2 × E0422 to go 51 → 49.
- **E0425 `NonEmptyStr` appears exactly 3 times**, at `v2_compiler_materialization_carriers.rs`
  lines 141, 145, 149. #8528 declared "E0425 NonEmptyStr trio is declared residue".

A trio nobody told this run to expect, landing at three adjacent sites, is agreement that is hard
to obtain by accident. The instruments are treated as comparable on that basis — not on the fact
that both invoked `cargo`.

## 6. THE DELTA IS NOT ATTRIBUTABLE, and that is the most load-bearing sentence here

49 (at `b2dd729f92`, #8528) → 41 (here) is **−8 across five merged commits with two different
causes**:

| side | commits |
|---|---|
| emitter | #8537, #8539, #8570 |
| **subject** | **#8592** → `dag/std/algebra.dag` · **#8505** → `src/v2/std/node.dag` |

This is not a footnote. `v2_std_algebra.rs` carries 4 of the E0277 and both E0369; `v2_std_node.rs`
carries 3 of the E0308. The two subject-side commits landed **directly in the files where much of
the surviving population lives**. Any sentence of the form "the emitter took 49 to 41" is
unsupported by this run.

**The supported statement is:** 41 at `4cec10f66a`, down 8 from 49 at `b2dd729f92`, across a window
containing both emitter and subject changes, **unattributed**. Arm B is what buys attribution,
because it differs from arm A by #8614 and nothing else.

## 7. Harness defects found and fixed, recorded because one nearly produced a false result

1. **A guard that aborted because its condition held.** `grep -c` exits 1 on a zero count; under
   `set -o pipefail` that failure propagates through the pipe before the downstream comparison can
   succeed. The run printed `mirror marker count: 0` and then `ABORT: mirror carries #8614` on
   adjacent lines. Fixed by capturing the count into a variable and comparing it as a value — and
   by echoing the *passing* branch, since a guard silent on success is indistinguishable from one
   that never ran.
2. **`render_cssl_probe_lib_cargo_toml.sh` defines a function and writes via `gunbc run`.** It emits
   nothing on stdout. Redirecting it into `Cargo.toml` wrote an **empty manifest**, so `cargo`
   failed at manifest parse having compiled nothing.
3. **Consequence of 2:** `exit 101` with an empty JSON stream, which the first parse rendered as
   `TOTAL error diagnostics: 0`. Reported uncritically that is **"N = 0, down from 49"** — every
   individual number true, the conclusion fabricated, from a build that never compiled a line.

**The generalizable rule, now enforced by the script:** when a step can fail *before* producing
output, assert the output is non-empty as a condition **separate from the exit code**, and make
that arm print `VOID` rather than a count. A zero from an empty stream and a zero from a clean
build are different states; only one of them is a measurement.

## 8. The 34 distinct sites

```
E0277      src/v2_compiler_materialization_carriers.rs:184      the trait bound `A: Clone` is not satisfied
E0277      src/v2_compiler_materialization_carriers.rs:184      the trait bound `B: Clone` is not satisfied
E0277      src/v2_std_algebra.rs:43                             the trait bound `T: Clone` is not satisfied
E0277      src/v2_std_algebra.rs:45                             the trait bound `T: Clone` is not satisfied
E0277      src/v2_std_algebra.rs:84                             the trait bound `T: Clone` is not satisfied
E0277      src/v2_std_algebra.rs:88                             the trait bound `T: Clone` is not satisfied
E0277      src/v2_std_staging.rs:16                             the trait bound `C: Clone` is not satisfied
E0282      src/std_realization_measurement.rs:197               type annotations needed
E0308      src/extdeps_realization_compile_stage_memo.rs:102    mismatched types
E0308      src/extdeps_realization_compile_stage_memo.rs:96     mismatched types
E0308      src/extdeps_realization_parse_table_memo.rs:117      mismatched types
E0308      src/extdeps_realization_parse_table_memo.rs:123      mismatched types
E0308      src/extdeps_uri.rs:752                               mismatched types
E0308      src/extdeps_uri.rs:756                               `match` arms have incompatible types
E0308      src/std_cache_interface.rs:638                       mismatched types
E0308      src/std_cache_interface.rs:695                       mismatched types
E0308      src/std_cache_interface.rs:699                       mismatched types
E0308      src/std_cache_interface.rs:703                       mismatched types
E0308      src/v2_std_algebra.rs:118                            mismatched types
E0308      src/v2_std_node.rs:69                                mismatched types
E0308      src/v2_std_node.rs:78                                mismatched types
E0308      src/v2_std_node.rs:84                                mismatched types
E0308      src/v2_std_staging.rs:31                             mismatched types
E0310      src/v2_std_staging.rs:44                             the parameter type `impl Fn(A) -> Rc<CacheProbe<B>> + Clone` may not l
E0310      src/v2_std_staging.rs:44                             the parameter type `impl Fn(A) -> Rc<Outcome<B>> + Clone` may not live
E0369      src/v2_std_algebra.rs:45                             binary operation `==` cannot be applied to type `Rc<im::Vector<T>>`
E0369      src/v2_std_algebra.rs:88                             binary operation `==` cannot be applied to type `&Rc<im::Vector<T>>`
E0425      src/v2_compiler_materialization_carriers.rs:141      cannot find type `NonEmptyStr` in this scope
E0425      src/v2_compiler_materialization_carriers.rs:145      cannot find type `NonEmptyStr` in this scope
E0425      src/v2_compiler_materialization_carriers.rs:149      cannot find type `NonEmptyStr` in this scope
E0560      src/std_verification.rs:22                           struct `Rc<Measure<(), (), i64>>` has no field named `count`
E0599      src/v2_compiler_materialization_carriers.rs:184      no method named `clone` found for type parameter `A` in the current sc
E0599      src/v2_compiler_materialization_carriers.rs:185      no method named `clone` found for type parameter `A` in the current sc
E0599      src/v2_std_staging.rs:16                             no method named `clone` found for type parameter `A` in the current sc
```
