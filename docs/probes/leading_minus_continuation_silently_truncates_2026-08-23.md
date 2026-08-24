# A continuation line beginning with `-` silently truncates the expression

**Found:** 2026-08-23, while adding an arithmetic consistency check to
`extdeps.cpu_attachment.lotes_azifa072` (gunbc#9061).
**Class:** silent wrongness — below the guarantee ladder's floor, not a rung on it (DESIGN §4b/§5).
**Live corpus exposure:** zero. The only instance was the one that found it, now fixed.

## The defect

An expression continued onto a following line whose first token is `-` is not continued. The
parser ends the expression at the newline and reads the continuation as a **fresh statement**
applying unary minus. One operand is discarded. No diagnostic is produced at any severity.

```
fn minus_leading() -> Int {
  100
    - 1
}
```

`minus_leading()` returns **-1**. The `100` is evaluated and dropped; the value of the function is
the last statement, `-1`.

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

`+` and `*` continue correctly, and a **trailing** `-` continues correctly. Only a **leading** `-`
fails, because only `-` has a unary form that can legally begin a statement. This is the classic
automatic-semicolon-insertion hazard: the parser continues an expression across a newline exactly
when the next token *cannot* start a statement, and `- x` can.

## Why this is below the floor rather than a low rung

The failure is not a refusal, a wrong diagnostic, or a degraded answer. It is a **fabricated
plausible output**: a total, type-correct function that returns a confidently wrong value. It
type-checks, it compiles with zero diagnostics at any severity, and it reads correctly to a
reviewer — the wrong parse is invisible in the source that produced it. DESIGN §5 places silent
wrongness outside the ladder entirely.

It was caught only because the check was **executed** and its answer disagreed with hand
arithmetic. A `.contains()` grep, a typecheck, or a review pass would each have let it through,
which is the specification-without-execution trap in its purest live form.

## Blast radius, measured

A grep for continuation lines beginning with `-` across `dag/` and `src/v2/` returns 13 hits. Twelve
are prose bullets inside string literals — markdown lists in `std/realization_schedule.dag`,
`gunbc/tools/review_codex.dag`, `gunbc/ci_release_bins.dag` — where the text is data and the parse
is irrelevant. The thirteenth was the instance in this PR. So the mechanism is confirmed and the
current corpus exposure is zero.

Zero exposure is not zero risk. The corpus writes multi-line boolean chains with leading `&&`
constantly, so the *style* the hazard punishes is house style; it is only the operator that is rare.
The first arithmetic expression someone splits that way will be silently wrong.

## What was NOT done, and why

The parser is not fixed here. This is `src/v1`, whose admission test is service to the v2 self-host
program (DESIGN §3, v1 maintenance standing) — a lexer or statement-boundary change is neither that
nor in the scope of the pull request that found it, and it needs its own review against v2's grammar
rather than a drive-by in a socket-geometry change.

The instance was repaired by parenthesising the subtraction on one line, with the reason recorded at
the call site rather than left as unexplained formatting. That is a repair of the instance, not of
the class, and naming the difference is the point of this probe: **the class is open.**

## Next-rung trigger

A statement-boundary rule that refuses an ambiguous continuation rather than silently choosing one
of its two readings. The fail-closed form is available and cheap: where a newline is followed by a
token that could either continue the expression or begin a statement, refuse with a located
diagnostic naming both readings and requiring parentheses or an explicit terminator. That converts
this from silent wrongness to a typed refusal — from below the floor to *structurally guaranteed* —
and, unlike a precedence tweak, cannot change the meaning of any program that currently parses the
way its author intended.
