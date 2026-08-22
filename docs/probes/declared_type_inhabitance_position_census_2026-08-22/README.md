# Declared-type inhabitance, measured at every grammar type position

The class: **a value accepted at a declared-type position does not inhabit that declared type.**
DESIGN section 4b puts *values inhabit declared types* in the ordinary compiler floor, so a
failure here is a BELOW-BASELINE safety regression, not a class sitting at mitigatable.

Two seams of it were known when this census started (gunbc#8854 direct-call arguments,
gunbc#8865 / gunbc#8876 record-literal fields) and gunbc#8868 records the enumeration as
UNFINISHED — "assume a third seam exists until someone enumerates them". This is that
enumeration, and it is taken from the parser rather than from recollection: see
[positions.md](positions.md).

## Instrument

`gen_arms.sh` writes one single-module `.dag` program per (position, specimen) pair. Every arm
shares one prelude declaring a coproduct, one of its payload types, a generic carrier and a
record, and differs only in the marked expression. Each arm is compiled on its own source root.

Four specimens per position, and each answers a different question:

| specimen | expression at the declared position | what its result means |
|---|---|---|
| `pos` | `SameRev` — a member of the declared coproduct | ACCEPT expected. A wall that refused this would be a fabricated refusal. |
| `nega` | `7` — a plain kernel value | REFUSE expected. This is the floor. |
| `negb` | `mk_inner()` — a value of one arm's PAYLOAD type at the PARENT position | REFUSE expected. gunbc#8865's shape. |
| `reach` | `nosuchname_zzz` — an undefined name | REFUSE expected, and it is the REACHABILITY control: it proves the position is judged at all, so an `accepted` on `nega`/`negb` is a hole in a live judgment rather than an artifact of a skipped fixture. |

The `reach` control is what makes this evidence rather than an assertion. Without it, "no
diagnostic appeared" is equally consistent with the arm never being compiled.

## What was measured

The verdicts and the site-grain fold are in [measured.md](measured.md); the design that follows
from them is in [design.md](design.md). Headline, from this session's own run and not transcribed
from another session's report: **seven of the fourteen grammar sites accept a plain kernel value
where a coproduct is declared** (the `as` cast is an eighth, excluded for gunbc#8925's stated
reason), **the arm-payload-at-parent specimen is accepted at all twelve reached positions**, and
two cells no census had recorded — a parameter's default-value expression, which is resolved
but never inferred (so an undefined name is accepted there), and the map-key position, whose refusals are the grammar's and not a type
judgment's.

## Two instrument failures, recorded because both fail toward ZERO

Both produced a table of all-zero diagnostics that looks exactly like "nothing refuses anywhere",
which is the finding this census would report if it were true. Neither was detectable from the
table alone — only from the raw output.

1. **A whole-root compile refuses on the memory budget and exits 0.**
   `WholeCorpusCompileBudgetBelowMeasuredDemand` on a 7 GiB runner: the run never starts, the
   message goes to stderr, and the exit status is 0. Grepping such a run for `error[` yields
   zero. Remedy: `--entry <file.dag>` scopes the compile past the whole-corpus admission.
2. **A `--source-root` outside the workspace root PANICS.**
   `repo_relative_path_normalized: path /tmp/... is not under process workspace root` — so arms
   written to `/tmp` produce a panic, not a compile. The arms must live under the repo root
   (or use the legacy `--source-dir`, which has no such check).

The paired-nonzero rule is what caught both: a table in which the reachability control also reads
zero is an instrument report, not a compiler report.
