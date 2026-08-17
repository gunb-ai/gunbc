# Finding: a record field initializer is not checked against its declared type (main)

**Status: OPEN. Against `main`, not against the namespace cut. Not repaired in
the namespace-cut lane.**

## The observation

On `origin/main` (measured at `b56c8996bd`, srv1, release `gunbc`), this module
compiles with `0 diagnostics` and exit `0`:

```
module test.probe.q

import std.nat { Nat }

type QualOnly {
  n: Nat
}

data qual_specimen: QualOnly = QualOnly { n: 1280 }
data planted: QualOnly = QualOnly { n: "definitely not a number" }
```

`planted` initializes a field declared `Nat` with a string literal. The compiler
emits the file and reports success.

Command:

```
gunbc compile --source-root probe --source-root dag --source-root src/v2 \
  --entry probe/q.dag --output-dir <out> --target dag
```

## Why this is filed separately, and at this severity

DESIGN names *"values inhabit declared types, fields exist"* as part of the
**ordinary compiler floor**, and states that a failure there is a below-baseline
safety regression that higher-order capability never compensates for. This sits
below the floor: no diagnostic, no refusal, an emitted artifact.

It is also §5's forbidden class rather than a rung on the §4b ladder — a path
that succeeds while producing a program the declaration says is wrong.

## How it was found, and why it nearly wasn't

It surfaced as the CONTROL half of a namespace-cut comparison. The branch
rejects an integer literal against a qualified `std.nat.Nat`; I proposed that as
a cut regression and ran main as the control. Main accepted a *string* in the
same position, which withdrew the regression claim — the pair does not
discriminate, because main does not check that position at all.

The finding is the control's own result. Withdrawing the branch claim was
correct and would have discarded it as a side effect; that is the recording
hazard worth naming, not just the defect.

## What is NOT established

- Whether the branch's stricter behavior at this position is correct-by-accident
  or a different mechanism. The branch's refusal is real and reproduces, but its
  cause has not been traced.
- The population on main. One specimen is proven; no census has been run for
  how many field initializers are unchecked, or whether this is specific to
  literals, to `Nat`, to imported types, or general.
- Whether any emitted artifact on main is wrong as a result.

## Reproduction cost

Seven lines and one compile. No fixture, no corpus, no host state.
