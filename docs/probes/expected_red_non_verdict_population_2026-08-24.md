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

## 2. Where the 99 came from: a baseline that carried the fix

The figure is real and the sign is inverted. The brief was differenced against **#9020's branch
head**, not against `330f63c514` — and that branch carries the resolution repair.

| baseline used | run | `passed` | `known_red_runtime_errored` |
|---|---|---|---|
| `5482863b4e` (the actual base) | `32681700364` | 10520 | 142 |
| `c767f4c962` (#9020 branch head) | `32680784546` | 10619 | 43 |

The delta is exactly **−99 runtime-errored / +99 passed**, and it is #9020's repair *doing its job*:
99 witnesses that threw now answer. Read in the other direction — fixed branch as the normal state,
unfixed base as the degraded one — a repair reports as a regression with the sign flipped.

`planned` is **10791 on both runs**, which is what let the mistake survive a sanity check: an equal
planned population reads as an equal baseline, and it is not one. That is the generalizable failure
and it is worth more than the incident: **a comparison anchored on a feature branch inherits that
branch's effect as its zero.** The control that would have caught it is cheap — pull the run for the
exact base SHA and diff the *identity sets*, not the counts.

(An earlier revision of this document attributed the 99 to the `HELD → RUNTIME-ERRORED`
reclassification in `ec2f3c318fe` (#8959) of 2026-08-23, where `known_red_held` fell 208 → 36 as
`runtime_errored` appeared at 164. That reclassification is real and is why the population is
*visible* at all — it is how `Held` stopped absorbing witnesses that had rotted — but it is **not**
where the brief's 99 came from, and this document said it was. Corrected on the authority of the
dispatching session, then verified here against run `32680784546` directly.)

The half of the brief that survives contact with the evidence is its second clause, and it is worth
keeping: **142 enrolled identities produce no verdict on every run, and the floor reports `failed=0`
and exits green.**

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

## 7. The population is about to move, and the carrier is not pre-adjusted

`gunbc#9020` resolves 99 of the 142 — measured on its branch head `c767f4c962` (run
`32680784546`), which reports 43 where the base reports 142 over an identical planned population.

The carrier keeps the **142**. Writing 43 into it today would be a count copied from a branch that
has not landed, which is precisely the fix-carrying-baseline error described in §2 — committed a
second time, into the artifact that documents it. The `PopulationBasis` arm names the run and the
commit it was measured on, so the row is re-measured when that PR merges rather than quietly
carrying a figure from a tree nobody is running.

## 8. The no-import story does not cover the whole bucket (added 2026-08-24, after independent census)

§5 checked four names by hand and found every one referenced a declaration that exists while its
witness declared **no** import. That is accurate and it is a sample of four. An independent census
(valiant-lynx-227, run `32743601436`) partitions the erroring modules **53 declaring no import** to
**8 that declare imports and error anyway** — and cross-joining that against the cause census here
puts **six of those eight inside the `no declaration named` bucket**. The bucket is not homogeneous.

One read in full, so the second shape is evidence rather than inference: `v2.lens.vacuity_test`
fails on `nat_add_left_identity_input` while declaring ten imports, among them
`v2.test.nat_semiring.rung_5` — a module that *references* that name but is not the module that
*declares* it (`v2.std.algebra_laws.nat_semiring`). **Importing a module does not transitively
supply the names its own declarations reach for.** So the second shape is an *incomplete* import
set, not an absent one.

The load-bearing conclusion is unchanged and slightly strengthened: both shapes are authored
defects, both are repaired by adding the import that declares the referenced name, and neither is a
floor scope defect — so "widen the scope" remains the wrong trigger. What moves is the repairer's
expectation. The other five are established only as "declares imports AND lands in this bucket";
their specific missing imports are unread.

### A note on how this was nearly reported wrong

The census that produced the 53/8 split was first relayed as "142 rows but 133 distinct identities,
9 appearing twice" — a claim that, if true, would have meant this PR's roster carried nine stale
exemptions. It was false in a specific and instructive way: the floor **column-pads** the identity,
so short names are followed by spaces before `ERROR in`, and an extraction using `[^ ]*` cannot
cross that padding. The pattern silently selected for long identifiers and dropped exactly nine
rows. The nine were **missed, not duplicated** — the sign was inverted, which is the same shape as
the original brief this document exists to correct.

Two independent diagnoses converged on the padding cause, and the retraction is recorded here
rather than only in message traffic. The durable lesson is §7's, sharpened: **a distinct-vs-total
discrepancy is a tell about the reader before it is a tell about the population**, and the control
is to count with a *different* pattern than the one that extracts. It is also the strongest
argument yet for the instrument gap below.

## 9. The cause census in this document is a claim, not a receipt

The floor log carries **no cause text beside the ERROR row** — only identity and duration. So the
119/13/9/1 partition in §4 cannot be reproduced or checked by anyone unwilling to re-run the floor,
which OOMs in a session container. That makes it unverifiable *by construction*, not by anyone's
laziness, and this document says so rather than letting the table read as measured-and-checkable.

Printing the cause beside the identity is a smaller change than the wall this PR lands, and it
would have made the false alarm above impossible: a reader could have checked causes without
re-deriving identities by grep at all. It is not folded into this PR — it is a separate change with
its own subject — but until it lands, §4 is a claim.
