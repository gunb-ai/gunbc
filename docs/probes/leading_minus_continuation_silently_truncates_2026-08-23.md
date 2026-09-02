# A continuation line beginning with `-` silently truncates the expression

**Found:** 2026-08-23, while adding an arithmetic consistency check to
`extdeps.cpu_attachment.lotes_azifa072` (gunbc#9061).
**Class:** silent wrongness — below the guarantee ladder's floor, not a rung on it (DESIGN §4b/§5).
**Original live corpus exposure:** zero after the instance that found it was fixed. That containment
strategy was falsified by a recurrence on 2026-08-31 in gunbc#9839; see “Recurrence and wall” below.

## The defect

An expression continued onto a line whose first token is `-` is not continued: the parser ends the
expression at the newline and reads the continuation as a **fresh statement** applying unary minus.
One operand is discarded. No diagnostic at any severity.

```
fn minus_leading() -> Int {
  100
    - 1
}
```

`minus_leading()` returns **-1**: `100` is evaluated and dropped; the function's value is the last
statement, `-1`.

In a `let` binding the discarded half is the other one:

```
let multi = d.outline_fill_position_count
  - d.actual_pad_count
```

`multi` binds to `d.outline_fill_position_count` alone — measured at 5285 where 359 was intended —
because the `- d.actual_pad_count` line becomes a separate discarded statement.

## Only `-`, and the reason is the point

| reduced case | returns | correct |
|---|---|---|
| `100` ⏎ `- 1` | **-1** | ✗ (expected 99) |
| `100` ⏎ `+ 1` | 101 | ✓ |
| `100` ⏎ `* 3` | 300 | ✓ |
| `100 -` ⏎ `1` | 99 | ✓ |

`+`, `*`, and a **trailing** `-` continue correctly. Only a **leading** `-` fails, because only `-`
has a unary form that can legally begin a statement — the classic automatic-semicolon-insertion
hazard: the parser continues across a newline exactly when the next token *cannot* start a
statement, and `- x` can.

## Why this is below the floor rather than a low rung

Not a refusal, a wrong diagnostic, or a degraded answer but a **fabricated plausible output**: a
total, type-correct function returning a confidently wrong value. It type-checks, compiles with zero
diagnostics at any severity, and reads correctly to a reviewer — the wrong parse is invisible in the
source. DESIGN §5 places silent wrongness outside the ladder entirely.

It was caught only because the check was **executed** and disagreed with hand arithmetic. A
`.contains()` grep, a typecheck, or a review pass would each have let it through — the
specification-without-execution trap in its purest live form.

## Blast radius, measured

A grep for continuation lines beginning with `-` across `dag/` and `src/v2/` returns 13 hits. Twelve
are prose bullets inside string literals — markdown lists in `std/realization_schedule.dag`,
`gunbc/tools/review_codex.dag`, `gunbc/ci_release_bins.dag` — where the text is data and the parse
is irrelevant. The thirteenth was the instance in this PR. Mechanism confirmed; current corpus
exposure zero.

Zero exposure was not zero risk: the corpus constantly writes multi-line boolean chains with leading
`&&`, so the *style* the hazard punishes is house style; only the operator is rare. The next
arithmetic expression split that way was silently wrong eight days later.

## What was not done in the original change, and why

The parser was not fixed in that change. This is `src/v1`, whose admission test is service to the v2
self-host program (DESIGN §3, v1 maintenance standing) — a lexer or statement-boundary change was
neither that nor in scope for the PR that found it, and needed its own review against v2's grammar
rather than a drive-by in a socket-geometry change.

The instance was repaired by parenthesising the subtraction on one line, with the reason recorded at
the call site. That repaired the instance, not the class: **the class remained open at that point.**

## Recurrence and wall

The class recurred on 2026-08-31 in gunbc#9839, where it silently produced the negation of a count
inside a comparator. Both new witnesses simply reported failure; the author found the parser choice
only by printing intermediate values and observing that each was exactly one less than the truth.
The earlier “current exposure zero” observation was therefore a historical census, not containment.

The parser now refuses the exact ambiguous statement boundary: a complete expression followed by a
newline and the sole token (`-`) that has both prefix and infix roles. The located parse diagnostic
spans the preceding expression and the operator and names both explicit spellings: put infix `-`
before the newline, or parenthesize a fresh unary expression. Both explicit readings remain admitted
and are checked by value; the ambiguous spelling admits neither. This converts the class from silent
wrongness to a structural parse refusal without guessing the author's intent.
