# The expected-red non-verdict population, and the falsification of the brief that found it

**Date:** 2026-08-24 · **Subject:** `v1_compiler.cli_run` `run_required_floor`, `v1_compiler.claim_executor` `required_floor_outcome_is_clean` · **Carrier:** `gunbc.floor_non_verdict_enrollment`

## 1. The brief, and why it is false as stated

The brief this work was dispatched under reads:

> MAIN REGRESSION, non-gating: 99 witnesses went from PASSED to RUNTIME-ERRORED across
> `330f63c514..5482863b4e` and the floor still reported `failed=0`.

Measured against the runs those two commits actually produced, **no witness changed disposition in
that range at all.** Both runs are green, both roster joins are identical, and the runtime-errored
population is byte-identical in both directions.

| | `330f63c514` (run `32677515653`) | `5482863b4e` (run `32681700364`) |
|---|---|---|
| `planned` | 10789 | 10791 |
| `passed` | 10518 | 10520 |
| `known_red_held` | 35 | 35 |
| `failed` | 0 | 0 |
| `known_red_runtime_errored` | 142 | 142 |
| `route_gap_held` | 94 | 94 |

The per-identity populations were extracted from both logs and diffed: the 142
`KNOWN-RED-RUNTIME-ERRORED` identities are **the same 142 identities**, with no additions and no
removals. The `expected-red-roster-join` artifacts differ in exactly one line — the `run_head`
header — and both report `roster=177 still_red=177 now_passes=0 not_evaluated=0`. The diff across
the range does not touch `src/v2/workflow/floor_expected_red.dag`.

So there is no regression at those commits, and there could not have been one of the shape
described: a witness that was **PASSED** is not enrolled, and an unenrolled runtime error is pushed
to `outcome.failures`, which **is** a gating conjunct. `PASSED → RUNTIME-ERRORED` stops the line
today. The one runtime-error arm that does not stop the line is the *enrolled* one, and an enrolled
identity that passes reports as `known_red_now_passing`, which is also gating.

## 2. What actually happened, on a different day and with a different transition

The event the brief is a distorted echo of is real, and it is a **`HELD → RUNTIME-ERRORED`**
reclassification that landed on 2026-08-23 in `ec2f3c318fe` (#8959, "Enrollment cannot hold a claim
that was never decided: the other two non-verdict outcomes").

| run | head | `known_red_held` | `known_red_runtime_errored` |
|---|---|---|---|
| `32603180703` (2026-08-22T22:42Z, before) | `f2d1ae7e1` | 208 | *(counter did not exist)* |
| `32644028606` (2026-08-23T13:59Z, after) | `50bac70dc` | 36 | 164 |
| `32681700364` (2026-08-24T02:02Z) | `5482863b4e8` | 35 | 142 |

That commit did not break anything. It **revealed** something: 164 identities that had been counted
as "known red, behaving as enrolled" had in fact been throwing, and `Held` was absorbing them. Its
own commit message states the harm exactly — "a known-red witness which ROTS … is indistinguishable
from one still doing its job, forever, and its enrollment never comes up for review."

The half of the brief that survives contact with the evidence is therefore its second clause, and it
is worth keeping: **142 enrolled identities produce no verdict on every run, and the floor reports
`failed=0` and exits green.**

## 3. Why the line does not stop, which is a decision and not a defect

`required_floor_outcome_is_clean` has seven conjuncts. `known_red_runtime_errored` and
`known_red_observation_unreadable` are deliberately not among them; the function's own comment
records the reasoning:

> they are REPORTED and deliberately NOT gating … Making them block reds lanes with no connection to
> the defect, which needs an approved design and a shadow phase rather than an author's judgement.

That is a defensible call and this probe does not dispute it. What was missing beside it is the
DESIGN §4b(2) obligation that comes with any class parked below its ceiling: a **named next-rung
trigger** and a **sized population**. Neither existed. `gunbc.floor_non_verdict_enrollment` now
carries both.

## 4. The population, measured

From run `32681700364` (`5482863b4e8`), 142 identities across 65 distinct error signatures:

| cause | identities |
|---|---|
| name not in the run's loaded index | 119 |
| type error | 13 |
| undefined variable | 9 |
| call contract mismatch | 1 |

`known_red_observation_unreadable` was 0 on the same run, so every non-verdict identity in this
population threw rather than answering a non-`Bool`.

The largest signatures:

```
18 × no declaration named 'srv3_install_hang_no_router_lease_ms' in this execution's loaded index
13 × no declaration named 'extdeps_cargo_build_module' in this execution's loaded index
11 × type error: atom_identity_hash requires exactly one string argument
 5 × no declaration named 'nat_add_left_identity_input' in this execution's loaded index
 5 × undefined variable: LocalInProcess
```

## 5. The dominant cause is an authored defect, not a floor scope defect

This distinction decides who owns the repair, so it was checked by hand rather than assumed. Four
unresolved names were looked up in the corpus, and **every one is declared**:

| name the run could not resolve | where it is declared |
|---|---|
| `design_section_1_argument`, `design_argument` | `gunbc.design_argument` |
| `srv3_install_hang_no_router_lease_ms` | `gunbc.srv3_os_install_diagnostic` |
| `nat_add_left_identity_input` | `v2.std.algebra_laws.nat_semiring` |
| `extdeps_cargo_build_module` | `v2.lens.extdeps_shape_transport_policy.module_refs` |

The witnesses referencing them declare **no import** that would bring the declaration into the run's
loaded index. `dag/test/claim/design_argument_witness_test.dag` is the clean specimen: it has a
`module` line, a `live_tree_disposition`, and then `test fn`s referencing `design_argument` — with no
import statement anywhere in the file.

The interpreter's refusal is therefore **correct**. The witness is unimportable, and the repair is to
give each witness the import its subject needs (or to delete the witness), not to widen the floor's
scope. That is why the next-rung trigger declares population repayment as a *precondition* rather
than a follow-up: adding the gating conjunct today converts 142 counted rows into a red with no
owner, which is precisely the outcome the arm's author reserved the change to avoid.

## 6. What this probe does not claim

- **It does not size future exposure.** The 142 is what one run measured at one moment. Nothing
  counts a witness that starts throwing tomorrow and nothing refuses one, so DESIGN §4b(3)'s
  bounded-population requirement is *not* satisfied here — the carrier says so in its own
  `PopulationBasis` arm rather than letting a figure imply a bound.
- **It does not establish that the 142 witnesses are wrong about their subjects.** They threw before
  reaching a subject; what they would have answered is unknown, and that is the whole content of
  "enrollment asserts a verdict this claim never produced."
- **It changes no gate.** The conjunct is named as the next rung, not added.
