# Calibration for the terminal-ledger mirror: one row per disposition, five mutations

Design for the population that converts `TerminalDispositionWireAgreement` from *enforced for
what ran* into *enforced for every declared arm*, and gives
`SeedDagTerminalSchemaDeclarationLockstep` something that can distinguish it from its sibling.

Not yet built. This document is the design; the branch waits on the current file hold.

## Why the production join is not enough

The comparator runs over every row of every run. That is the **observed** population, not the
set of mapping **arms**. An arm no live row reaches is unexercised, and the join is silent about
it — so the honest scope today is "every disposition the floor happened to produce", which on a
green run is roughly three of nine.

§4b(1) makes a class's rung the minimum across its in-scope paths. An unexercised arm is on no
path at all, so it contributes nothing, and claiming the join covers the vocabulary would be
rung inflation.

## The population: nine rows, independently authored

One row per `ClaimDisposition`:

| # | disposition | terminal tag | expectation |
|---|---|---|---|
| 1 | `Passed` | `returned-true` | `expect-hold` |
| 2 | `Failed` | `returned-false` | `expect-hold` |
| 3 | `KnownRedHeld` | `returned-false` | `expect-red` |
| 4 | `KnownRedNowPassing` | `returned-true` | `expect-red` |
| 5 | `BudgetRefusedBeforeVerdict` | `budget-refused` | `expect-hold` |
| 6 | `HostToolUnresolvedBeforeVerdict` | `host-tool-unresolved` | `expect-hold` |
| 7 | `RouteGapBeforeVerdict` | `route-gap` | `expect-hold` |
| 8 | `RuntimeErroredBeforeVerdict` | `runtime-errored` | `expect-hold` |
| 9 | `ObservationUnreadableBeforeVerdict` | `returned-unreadable` | `expect-hold` |

**Independently authored is the whole point** (§5's oracle rule). These rows are written from the
declared vocabulary, not harvested from a run — a population measured out of the current tree is
not an oracle, and rows collected from live output would agree with the mappings by construction.

Rows 3 and 4 matter disproportionately: they are the two where the *expectation* changes the
disposition without changing the tag, so they are the only rows that exercise the expectation
input to the decode at all.

## The five mutations, and what each proves

A calibration that only goes green proves nothing. Each mutation is applied alone, to an
otherwise-unmodified tree, and the expected outcome is stated before it runs.

| # | mutation | expected | what it establishes |
|---|---|---|---|
| 1 | change one Rust `ClaimOutcome → tag` mapping | **RED** | the seed half is load-bearing |
| 2 | change one `.dag` `tag → disposition` mapping | **RED** | the module half is load-bearing, independently |
| 3 | add an unknown tag | **decoder refuses** | the vocabulary is closed, not best-effort |
| 4 | remove one arm from either mapping | **typecheck or calibration refuses** | arm existence is covered |
| 5 | keep every tag, change a payload contract | **must NOT red** | ← the discriminator |

Mutations 1 and 2 must be applied **independently**, not together. Applied together they cancel:
both sides move the same way, the comparator agrees, and a green run would be read as a passing
calibration when it is exactly the jointly-wrong case Class 1 declares it cannot catch.

### Mutation 5 is the one that makes two rows into two claims

Every other mutation reds. Mutation 5 must stay green, and that is what proves the two standing
rows are two claims rather than one claim written twice.

Change a payload's contract while leaving every tag and every disposition mapping untouched.
`TerminalDispositionWireAgreement` is silent — correctly, since no tag moved and no disposition
disagrees. `SeedDagTerminalSchemaDeclarationLockstep` is the only class that could have caught
it, and today it cannot, because it is held by author diligence.

So mutation 5 is simultaneously:

- the **positive control** on Class 1, showing its scope is genuinely the mapping join and not
  the declarations; and
- the **standing RED** for Class 2, which stays open until the schema lockstep gets a mechanism.

Without it, the two rows are indistinguishable by any executed evidence, and Class 2 is
decoration — a name for a gap with nothing demonstrating the gap is real.

## What this does not do

It does not make either derivation **correct**. Nine rows authored from the declared vocabulary
check that the two mappings agree over the whole vocabulary rather than over a green run's
sample. A mapping that is wrong in the same direction on both sides still agrees, and this
population will not see it — the same non-claim the standing note states, unchanged by wider
coverage.

Nor does it lift Class 1 to structural. The invalid pair remains constructible; it becomes
unable to survive on any arm rather than on the arms that happened to execute. Construction
would require the two derivations to be one, which is the dissolution — the self-emitted v2
claim executor — not a check.

## Cost

Nine rows and five mutations. The rows are data; four of the five mutations are one-line edits
run against a single module's tests. That is small against the alternative, which is a
vocabulary whose coverage is whatever the last green run happened to contain.
