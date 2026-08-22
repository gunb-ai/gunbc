# The v2 `Bool` fork: `!` on the coproduct bottom does not yield top

Executed evidence for the `Bool` fork row in
`docs/plans/compiler-guarantee-recovery-gap-analysis.md`. Recorded here rather than left in a
message because **a message is a measurement that expires**, and this one has already changed
once — see "Provenance" below.

## The probe

`src/v2/test/claim/bool_bottom_probe_test.dag`:

```
module v2.test.claim.bool_bottom_probe_test

import v2.std.logic { Bool, True, False }

fn a_control_comparison_negates() -> Bool {
  !(1 == 2)
}

fn b_bottom_negated_should_be_true() -> Bool {
  !False
}

fn c_top_is_itself_true() -> Bool {
  True
}

fn d_top_equals_top_control() -> Bool {
  True == True
}
```

Invocation:

```
./target/release/claim_batch --source-root dag --source-root src/v2 \
  --entry src/v2/test/claim/bool_bottom_probe_test.dag \
  --functions a_control_comparison_negates,b_bottom_negated_should_be_true,c_top_is_itself_true,d_top_equals_top_control
```

## The run

```
PASS a_control_comparison_negates
FAIL b_bottom_negated_should_be_true
FAIL c_top_is_itself_true (returned `Bool`, not Bool; --claim-run entries must return Bool)
PASS d_top_equals_top_control
```

## What each arm establishes

| arm | role | what it shows |
|---|---|---|
| **(a)** | positive control | `!(1 == 2)` negates correctly through the **host** representation |
| **(d)** | positive control | `True == True` passes, so the **coproduct** is not wholly inert |
| **(b)** | **the RED** | `!False` on the coproduct does not yield top — and does not refuse |
| **(c)** | the RED that names it | the diagnostic reads `` returned `Bool`, not Bool `` |

(a) and (b) are the discriminating pair §4b(1) requires: **same operator, two representations,
opposite results.** Without (a), (b) would only show that `!` is broken. (d) does the same job
for the coproduct: without it, (b) could mean the type is unusable rather than that negation
specifically is wrong.

(b) is the below-floor arm. It does not refuse — it silently produces a value that then fails a
different assertion. Silent wrongness is outside the ladder, not a rung on it.

## Arm (c): the census is UNBUILT, not impossible

The diagnostic writes **two distinct types, both spelled `Bool`, in one sentence** — one
backticked, one bare. A reader hitting this in CI cannot act on it: it says a thing is not
itself.

That is worse than opaque and better than it looks. A **grep** census is impossible, because the
spelling is identical on both sides of the fork in the source. But the compiler demonstrably
distinguishes them — it holds both types and renders them differently at the refusal site. So
the discriminator exists inside the compiler even though it does not exist in the text, and
anything reaching the comparison the diagnostic reaches can partition the corpus.

Under §4b(2) that makes this a **next-rung trigger**, not a permanent stall — only *cannot climb
further* is permanent.

## Scope limits — stated so the row does not overclaim

- **Four arms is a mechanism receipt, not a census.** It establishes that the class exists and
  its rung. It says nothing about population, and per (c) a grep cannot either. One known
  consumer on the dead side: `v2.std.logic` `bool_boolean_algebra` matching `True`/`False`.
- **One path only: source → interpretation, via `claim_batch`.** §4b(1) makes a class's rung the
  **minimum across in-scope paths**, and the emission path is unmeasured. Any citation of this
  as the class's rung must say *on the interpreted path*.
- **The fix is not proposed here.** Which representation survives is a §3 root question.

## Provenance — why this file exists rather than a quoted result

The arms were run twice. The first run was reported with
`d_bottom_equals_top_should_be_false` as **PASS**; re-running caught that the arm asserted
falsity while the harness passes only on true, so it was a mis-designed arm reporting the wrong
thing rather than a finding. It was replaced by `d_top_equals_top_control`.

**One of four lines in the original receipt was wrong, and only re-running found it.** Had the
first receipt been transcribed into the row, the row would have carried a fabricated PASS. That
is the argument for keeping executed evidence in a file with its invocation beside it — a
remembered receipt is not a receipt.

Probe by `bright-ferret-335`; recorded here by `fierce-lynx-647`.
