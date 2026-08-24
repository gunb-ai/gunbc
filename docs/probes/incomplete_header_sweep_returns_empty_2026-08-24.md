# The incomplete-header sweep returns empty, and the empty is uninformative: the instrument cannot go red

**Subject.** A brief dispatched this lane to finish an *incomplete-header sweep* over the
`src/v2/compiler` modules, on the stated premise that a partial import header is **already broken** —
that each module carrying one is a live refusal when compiled alone, in the manner of
`v2.compiler.program_assembly` in gunbc#9083.

**Result: the premise does not hold on current main, and the reason is not that the headers are
complete.** Every module in the universe compiles alone with zero blocking errors, *and* a module's
import header can be deleted **in its entirety** without producing one — measured on a 33-import
module, against an undefined-name positive control that does refuse. There is nothing to repair by
this method, and no oracle to repair against.

**Evidence status.** Every figure below was produced by execution in this lane, by a binary built
from the tree it measures inside the same remote dispatch. Nothing here is relayed from another
session. The one claim that is *corroborated* rather than measured here — that closure membership
rather than the import statement is what admits a bare call — is attributed to `sleek-ant-767` in §5
and is not load-bearing for anything in §1–§4.

---

## 1. The instrument, and the two defects it had first

    gunbc compile --source-root dag --source-root src/v2 \
                  --entry <file> --output-dir <dir> --target rust

Built from HEAD in the same dispatch as every measurement, so no measurement is taken by a binary
older than the tree it reads. This is the shape of the gunbc#9083 receipt.

**Two instrument defects were found before any result was trusted, and both would have produced a
confident wrong answer.** They are recorded because each fails silently in the direction of good
news:

1. **`--output-dir` is required.** The first full pass omitted it. All 70 invocations exited `rc=2`
   at clap argument validation, having never reached the compiler — and a script grepping for
   `N blocking error(s)` finds nothing, which reads exactly like a clean sweep. **A zero produced by
   an argument error and a zero produced by a clean compile are the same bytes.**
2. **`--target dag` measures a different question.** Under it, three modules reported blocking errors
   naming `extdeps.filesystem.filesystem_io` — `'file' transport emission is not modeled … for target
   'dag'`. Those are emission-handler gaps introduced by the target choice, not header defects. The
   census is taken at `--target rust`.
3. **A `head -200` filter truncated 20 of 70 results before the summary line**, since the advisory
   block precedes it. Those 20 were re-run with a filter that keeps only `^error[` and the summary.
   Truncation fabricates a negative in exactly the direction that flatters the sweep.

## 2. The universe, which the brief had wrong by a third

The brief named **47** modules. The universe is **70** `.dag` files under `src/v2/compiler`:

| | count |
|---|---|
| top-level `src/v2/compiler/*.dag` | 42 |
| `src/v2/compiler/self_host/` | 27 |
| `src/v2/compiler/manual/` | 1 |
| **total** | **70** |

All 70 were measured. A closed-universe argument is only as closed as its denominator, and this one
was off by 23 modules before a line was read — the denominator error, stated loudly because a
one-sided sweep over the wrong universe is a confident answer to a question nobody asked.

## 3. The census

**69 of 70 report `0 blocking error(s)`.**

The single exception is `src/v2/compiler/manual/retry_eagain_bash.dag`, with **1** blocking error,
and it is **not** a header defect:

    error[dag/extdeps/cloud/gcp/gcp.dag]: 'file' transport emission is not modeled:
    operation 'gcloud.Auth.ReadADC' declared in 'extdeps.cloud.gcp.gcp' cannot be emitted
    for target 'rust' -- file transport output key 'client_id' has no modeled channel

That is a transport-realization gap in an `extdeps` module with a different owner. It is named and
excluded rather than folded in: folding it would make the closed universe not closed.

**So: zero live name-resolution refusals in the closed universe.**

## 4. Why that zero is not evidence, which is the actual finding

An empty sweep has two readings — *nothing is wrong*, and *the sweep cannot see* — and they are
indistinguishable from the output. They are distinguished by one command: **break something the
sweep is supposed to catch.**

`src/v2/compiler/00_compile.dag` declares and uses `parse`, and carries a 33-line import header.
Four arms, one binary, one entry, one dispatch, each mutation applied to the file on disk and
reverted before the next:

| arm | header | blocking | advisory |
|---|---|---|---|
| **A** baseline | as committed (33 imports) | **0** | 472 |
| **B** all imports stripped | `remaining import lines: 0` | **0** | 510 |
| **C** undefined-name control | `normalize(` renamed to `definitely_not_a_real_symbol_zz(` | **2** | 472 |
| **D** restored | as committed | **0** | 472 |

Arm **C** is the positive control and it establishes the instrument is not inert:

    error[src/v2/compiler/00_compile.dag:396:16]: function 'definitely_not_a_real_symbol_zz' not found in scope
    error[src/v2/compiler/00_compile.dag:482:20]: function 'definitely_not_a_real_symbol_zz' not found in scope
    2 blocking error(s)

Arm **B** is the finding. **A module whose import header has been deleted in its entirety compiles
alone with zero blocking errors.** The 33 import statements contribute nothing to whether the module
resolves; their entire measurable effect is 38 additional `unlisted import use` **advisories**
(472 → 510), which are non-blocking by policy.

A fifth arm, run earlier in a separate dispatch, deleted a *single* used import line:

    15d14
    < import v2.compiler.parse { parse }

with the diff printed inside the dispatch to establish the file on disk actually changed. That arm
returned `0 blocking error(s), 472 advisory diagnostic(s)` — identical to baseline in **both**
columns. **Why the single-line deletion moved the advisory count by zero while the full strip moved
it by 38 is not established here**, and it is left as an open question rather than explained: the
blocking result is the load-bearing one and is the same either way.

That is the mechanism `docs/probes/pool_fallback_is_the_resolution_mechanism_2026-08-24.md` describes,
observed from the deletion side rather than the construction side: after the namespace strip,
resolution is name-derived, so **a module's header can be arbitrarily wrong — up to and including
absent — without anything going red.** gunbc#9083's 26 refusals were the loud case, not the
representative one.

## 5. The corroboration, which arrived from the other direction

`sleek-ant-767` measured the same proposition today on a different instrument, different entry,
different binary, by construction rather than by deletion — four arms on one fixture:

| arm | blocking | verdict |
|---|---|---|
| listed (selective import naming the fn) | 0 | correct |
| selective-unlisted (import present, not naming it) | 0 | defect |
| pool-coincidence (provider never imported) | 0 | defect |
| absent-from-closure (nothing reaches it) | 1 | correct |

Their one-line result: **what admits a bare free call is closure membership, not the import
statement.** The name list contributes nothing — which arm B above reaches independently by deleting
the entire list and observing no change.

Two lanes, two methods, one conclusion, neither aware of the other. Their fourth arm also bounds the
worst reading of §4: a name that nothing reaches **does** refuse, so the instrument discriminates on
closure membership and is not simply inert. That is a more useful result than a bare inconclusive,
and it is why §4's zero is *blindness to headers* rather than *blindness to everything*. Arm C
establishes the same bound inside this lane's own instrument, so §4 does not depend on theirs.

## 6. What was not built, and why that is the deliverable

**No 47-header repair PR was opened.** Two independent reasons, either sufficient:

- **It would be unverifiable by construction.** No red before, no red after. That is the decoration
  failure named in DESIGN §4b: permanently green, carrying no information, and *worse than absent*
  because it would afterwards be cited as coverage of a class it never touched.
- **It is throwaway by construction.** `integration/namespace-cut` deletes import headers
  corpus-wide. Authoring 47 headers immediately upstream of a change that removes them spends
  author, reviewer and maintainer time on an artifact scheduled for deletion (DESIGN §6,
  intellectual sustainability).

## 7. The transferable rule

**Before trusting an empty sweep, break something the sweep should catch.**

The repository keeps re-deriving the distinction between *a check that returns nothing because
nothing is wrong* and *a check that returns nothing because it cannot see*. It is not a matter of
care or review attention — the two are byte-identical at the output. One planted red converts an
ambiguous zero into a demonstrated blindness, and it costs one command inside a dispatch that is
already running.

This lane needed it three times over: an argument error, a wrong target, and a truncated read each
produced a clean-looking zero before any real measurement had been taken.

## 8. Provenance

| | |
|---|---|
| lane | `bright-wren-428`, work item `adhoc-050e85db-087` |
| tree measured | `3985222df6e` (`main`) |
| binary | built from that tree per dispatch, `cargo build --release -p v1-compiler --bin gunbc` |
| corroborating lane | `sleek-ant-767` (§5), reported via `smart-ram-730` |
| related | `docs/probes/pool_fallback_is_the_resolution_mechanism_2026-08-24.md`; gunbc#9083 |
