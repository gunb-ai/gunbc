# Probe methodology note: RUSTC_WRAPPER cache impurity produced a false-green diagnostic count (2026-08-19)

Session `vivid-pike-765`. Recorded because it cost a wrong conclusion that was reported and then
retracted, and the failure mode has no local failure arm to notice it on its own — DESIGN.md's
"cache impurity" and "empty-observation narrow" (⊥-as-answer conflated with ⊥-as-ignorance) apply
directly.

## What happened

Probing `src/v2/compiler/06_translate.dag` at `e0c5e254445c0c39021a6306e6589198fa40219e` via the
standard `gunbc compile` → `cssl_assemble` → `cargo build --release --lib` route (see
[`e0308_root_partition_2026-08-18.md`](e0308_root_partition_2026-08-18.md) Method table) produced
**5 rustc errors, 0 × E0063** — apparently contradicting a peer measurement and an earlier census
pin that both showed 16 × E0063 in the same file (`v2_std_compilers_target_model.rs`).

The build did not fail, did not warn, and printed an honest rustc completion line:
`error: could not compile \`v1_compiled\` (lib) due to 5 previous errors; 1 warning emitted`.
That line was TRUE of what rustc actually did. It was not a compile of the crate's real content —
`RUSTC_WRAPPER=sccache` was still active (this environment's session default, alongside
`CTRL_BUILD_MODE=remote`) and sccache served a stale cached result for the crate.

Re-running **only the cargo build step**, on the same already-assembled crate (nothing re-emitted,
nothing re-compiled by `gunbc`/`cssl_assemble`), with `RUSTC_WRAPPER=` cleared and a fresh empty
`target/` dir, produced **268 errors**, including exactly 16 × E0063, all in
`v2_std_compilers_target_model.rs` — matching the peer measurement and the census exactly.

## The trap, stated plainly

**A false-green diagnostic count is more dangerous than a false-red one.** A smaller error count
reads as progress and nobody interrogates good news. There is no failure arm here at all — sccache
does not error on a cache hit, it succeeds, quickly, with a plausible completion line. The class is
`⊥-as-answer conflated with ⊥-as-ignorance`, DESIGN's own named failure mode, occurring in a probe
harness rather than in the compiler under test.

## What is necessary but not sufficient, and what closes the gap

Two checks used before this incident are real and worth keeping, but neither is sufficient alone:

1. **File-count match** against a prior/peer measurement (rules out a truncated/stunted crate).
2. **rustc's own total-error completion line**, taken from a route that reaches a genuine full pass
   (rules out early abort mid-compile).

Both of these can be honest and complete for a **cache-served** answer — sccache reports a real,
non-aborted rustc-shaped result, just not one produced by compiling this content. Add, as the
condition that actually closes the gap:

3. **State `RUSTC_WRAPPER` and `CTRL_BUILD_MODE` explicitly in every probe invocation, and clear
   `RUSTC_WRAPPER` for any build whose diagnostic count is being trusted as ground truth**, rather
   than assuming a `CTRL_BUILD_MODE=local` override alone is sufficient (it addresses the remote/arm64
   vs amd64 mismatch; it does not touch the wrapper).
4. **Before trusting a "clean" build result that looks smaller than a prior or peer measurement,
   run the byte-identical cached-vs-cold check** — DESIGN.md's own purity oracle for exactly this
   class of defect — rather than accepting the smaller number as an improvement.

## Repro (for a future probe author who hits the same shape of surprise)

```
# crate already assembled at $OUT from a prior gunbc compile + cssl_assemble run
cd "$OUT"
RUSTC_WRAPPER= CTRL_BUILD_WRAP_CARGO=0 CTRL_BUILD_MODE=local /opt/cargo/bin/cargo build --release --lib
```
versus the run that produced the false-green count:
```
cd "$OUT"
CTRL_BUILD_MODE=local /opt/cargo/bin/cargo build --release --lib   # RUSTC_WRAPPER left at session default: sccache
```
Same assembled source tree, same SHA, only the wrapper variable changed: 5 errors vs. 268.

## A second, unrelated trap found the same session: editing `src/v1/*.dag` has zero effect on the
## probe until a separate regen+rebuild step runs (2026-08-20)

Recorded beside the cache item because it produces the identical symptom class -- a probe that looks
authoritative and is silently answering the wrong question -- via a completely different mechanism, and
because the standard probe's `--source-root` list (`dag`, `src/v2`) never includes `src/v1`, which makes
the trap invisible from the invocation alone.

**What happened.** A fix was made to `src/v1/05_emit_rust.dag` -- part of the v1 seed compiler, itself
written in `.dag`. The standard probe route (`gunbc compile` -> `cssl_assemble` -> `cargo build`) was
re-run expecting the fix to be reflected. It was not, because `gunbc`/`cssl_assemble` are ordinary
compiled Rust binaries built via `cargo build` from the COMMITTED `.rs` files under
`src/v1/stage0/src/*.rs` -- they are not interpreted live from `.dag` source at invocation time. A
`.dag` edit under `src/v1` only reaches the running binary after (1) the corresponding generated `.rs`
mirror is regenerated/synced from the edited `.dag`, and (2) `gunbc`/`cssl_assemble` are rebuilt.

**How to tell whether a given `src/v1/stage0/src/*.rs` file is a generated mirror at all.** Its own
header answers this, per the fleet-wide "Check 1" convention converged on this session (see
`smart-ram-730`'s dashboard messages, 2026-08-19/20): a generated mirror carries, on line 1-2,
`// Generated by v1 compiler -- do not edit.` / `// Source module: <dotted.module.path>`. 129 of 167
files in that directory carry this header; the other 38 are genuinely hand-maintained host code (e.g.
`cli_run.rs`) that is SUPPOSED to diverge from any `.dag` and must never be treated as a mirror. Do not
infer the mapping from filename similarity or from `git grep -l <symbol>` -- both are demonstrated
defective (filename inference misses multi-mirror-per-authority cases; symbol grep over-collects across
unrelated authorities sharing a symbol name, and under-collects files declaring the type generically
without naming it verbatim). Read the header; trust nothing else.

**Getting a `.dag` fix to actually take effect for measurement, without a full corpus regen.**
`claim_executor --required-regen` demands POPULATION equality between what the current binary would
emit and what is committed, across the WHOLE stage0 tree, before it ever writes a candidate directory --
so a full regen attempt fails outright on any pre-existing unrelated drift elsewhere in that population
(confirmed: 8 unrelated basenames caused a refusal with no candidate directory written). When only a
narrow, already-understood function-level change needs to reach the binary for verification, and a full
regen is blocked by unrelated drift out of scope for the task at hand: compile the edited `.dag` module
standalone (`gunbc compile --entry src/v1/<module>.dag --source-root src/v1 --source-root dag ...` --
valid because `gunbc` is a general-purpose `.dag`->Rust compiler, independent of its own build history),
extract just the touched function bodies from that fresh output, splice them into the committed mirror
at the corresponding function boundaries, run `rustfmt` to normalize formatting (a raw single-module
emission is NOT formatted identically to the full-corpus regen pipeline's output -- dense single-line vs.
multi-line pretty -- and a naive whole-file diff between the two is dominated by this cosmetic noise, not
logic differences), then `cargo build --release -p v1-compiler --bin gunbc --bin cssl_assemble` from the
synced mirror. Diff the spliced mirror function-by-function against the fresh emission before trusting
it -- confirm the change is exactly the intended logic delta, nothing else moved.

**The tell that this trap has been hit:** a `src/v1/*.dag` edit that should visibly change probe output
(a new diagnostic, a cleared error, a different emitted call) produces byte-identical results to the
pre-edit probe run. Silence here is not confirmation the fix is wrong or irrelevant -- it is usually
confirmation the binary under test predates the edit.
