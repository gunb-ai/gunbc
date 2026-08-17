# A qualified constructor cannot be a `match` scrutinee

**Status:** open language-layer defect, reproduced on `main`, blocking for the
namespace cut. Found 2026-08-17 while validating the namespace-cut
qualification pass.

## The defect

A record literal used as a `match` scrutinee parses when its constructor is
named bare, and fails to parse when the same constructor is named by its
qualified path.

Minimal specimen, run against a `gunbc` built from `main` at `bbb5213`:

```
# REFUSES -- expected FatArrow, found Colon
module probe.u
fn f() -> Bool {
  match probe.carrier.Yes { n: 1 } {
    probe.carrier.Yes { n: _ } => true
    probe.carrier.No => false
  }
}

# COMPILES CLEAN -- positive control, same program, bare constructor
module probe.u
import probe.carrier { Choice, Yes, No }
fn f() -> Bool {
  match Yes { n: 1 } {
    Yes { n: _ } => true
    No => false
  }
}
```

The two differ only in how the scrutinee's constructor is spelled. The arms are
irrelevant to the failure: qualified constructors in ARM position parse fine
(separately witnessed -- a qualified variant in pattern position, in type
annotation position, in construction position, across source roots, and under
`--entry` with no import edge, all compile clean with three negative controls
refusing).

## Mechanism

After a dotted path the parser appears to treat the following `{` as opening the
match's ARM BLOCK rather than continuing a record literal. It then reads the
first field name as an arm pattern and demands `=>`, finding `:` instead. With a
bare constructor the constructor and its literal are consumed as one expression,
so the second `{` is unambiguously the arm block.

The reported position is a character offset, not a line number.

## Why it blocks the namespace cut

The branch's own parser refuses `import` with:

```
the import statement is deleted; name the container instead (a cross-module
reference is written container.member, and that reference IS the dependency edge)
```

So `container.member` qualification is the branch's terminal reference form. This
defect makes `match <qualified constructor literal> { ... }` unwritable under
that form: the bare spelling is being removed, and the qualified spelling does
not parse. It is not a property of the cut -- it reproduces on `main` -- but the
cut is what makes it reachable, because on `main` the construct is always
written bare behind an import.

## Population

Three sites corpus-wide at the time of writing. All three are currently left in
their bare (unqualified) form, so they parse and carry ordinary unresolved-name
diagnostics rather than aborting the compile.

## Deliberately NOT worked around

The obvious dodge is to bind the scrutinee first:

```
let x = extdeps.cloud.cloud.SigV4 { region: ..., service: ... }
match x { ... }
```

That is semantically identical and would make the tree green. It is not applied,
because it would drive this defect's observed frequency to zero while leaving
the parser gap in place -- the absorbing-fallback shape executed by an author,
and the exact class DESIGN names as the line-stop signal. The defect is recorded
loudly and counted instead.

## A measurement hazard this exposed, worth keeping

An unparseable `.dag` file does not produce a diagnostic. It PANICS in
`for_each_parsed_module_binding` and aborts the entire compile. So a corpus that
reports `1` may be strictly worse than one reporting `165` -- the run died
before measuring. During this investigation a corpus went from 165 diagnostics
to "1" and the naive reading was a 99% improvement; the truth was a hard abort
on the first file.

This is the same shape as the empty-observation narrow: bottom-as-answer
conflated with bottom-as-ignorance. Any corpus diagnostic count is only
meaningful alongside its parse-error count, and every measurement in the
namespace-cut lane should report both.

## Next rung

Fix is in the parser: after a dotted path in scrutinee position, continue the
record literal rather than opening the arm block. That touches load-bearing v1
parse code and requires a regen, so it is flagged for an explicit decision
rather than taken unilaterally.
