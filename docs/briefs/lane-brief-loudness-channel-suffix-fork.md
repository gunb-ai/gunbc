# LANE BRIEF — node://adhoc-602d7286-8ff

## The defect, stated as §3 rather than as a coverage gap

Two authorities answer ONE question, "is this a witness":

- **structural** — `test fn` at column zero. `v1_compiler.cli_run` `witness_file_from_source`
  is the whole of the floor's discovery: `line.strip_prefix("test fn ")`, collect the name,
  and a path `exclude_substrings` filter above it. No annotation, no registry, no naming rule.
- **name suffix** — `gunbc.test_module_hygiene` `failure_receipt_companion`:
  `_holds`/`_passes` -> `_failure_receipt`, else `optional_absent()`.

They disagree on **10408 of 12278** witnesses. That is not a defect population to count
down; it is the measured disagreement rate between two authorities for one fact, which is
§3 nicknaming applied to a PREDICATE rather than to a type.

## Measured (static scan, dag + src/v2 + src/v1, BOTH `fn` and `func` keywords)

    test fn at column zero                12278
    test func at column zero                  0
    eligible by name suffix                1870   (15.2%)
    INELIGIBLE BY NAME                    10408   (84.8%)
    authored *_failure_receipt                16
    eligible witnesses WITH a companion        4
    authored *_verdict_diagnostic              0

CAVEAT ON THESE NUMBERS: static regex over declarations. Population confirmed, BEHAVIOUR not
executed.

**READ THIS BEFORE YOU RUN YOUR OWN GREP.** An earlier revision said 6 companions and 3 pairs.
Both numbers were wrong, both sessions produced them independently, and the cause was the same
in both: the pattern matched `^fn ` only. **`func` is a real declaration keyword in this
corpus** — 269 uses against 36654 `fn` — and every missed companion was a `func`. A separate
re-run then returned 2 `*_verdict_diagnostic` producers, which would have refuted the zero;
that was a pattern with no END anchor matching `witness_verdict_diagnostic_companion` (the
derivation itself) and `lens_verdict_diagnostic_locus_module` (substring only). Anchored on
the declaration form, the count is 0, now verified independently by two sessions.

Four defective greps in one night between two sessions. The only one that reached nobody was
caught because the finding was interesting enough to double-check before sending — **that is
not a discipline anyone can rely on.** Match both keywords and anchor both ends.

## The change

`failure_receipt_companion` stops answering IS THIS A WITNESS and answers only WHAT WOULD
THIS WITNESS'S COMPANION BE CALLED. Total, no name condition. Same for
`witness_verdict_diagnostic_companion`.

**DO NOT append naively.** `X_holds` + `_failure_receipt` = `X_holds_failure_receipt`, which
BREAKS THE FOUR PAIRS THAT WORK TODAY. Correct shape: strip `_holds`/`_passes` WHEN PRESENT,
otherwise use the full name.

## Safe by construction — already checked, do not redo

- **Missing companion costs nothing.** `run_claim_failure_receipt` maps
  `InterpError::NoSuchFunction` to `String::new()` and the caller appends only when non-empty.
  Under a total derivation the ~12274 witnesses with no companion behave exactly as today:
  no error, no fabricated text, no cost outside a red.
- **No Rust mirror of the derivation.** `failure_receipt_companion_from_authority` calls the
  `.dag` function through the interpreter. One authority to edit.

## THE BLOCKER — three assertions of the fork, in two languages

All three assert that `failure_receipt_companion` DECIDES witness-hood. All three go red on a
total derivation. Updating them **is the substance of the change, not churn around it**.

1. `dag/test/claim/test_module_hygiene_hand_rust_equivalence_witness_test.dag`
   `test_module_hygiene_failure_receipt_companion_absent_on_non_witness` — asserts
   `Absent` for `"ordinary_fn"`.
2. its sibling in the same file, `codex_wet_enrolled_witnesses_resolve_failure_receipt_companion`.
3. `src/v1/stage0/src/cli_run/test_module_hygiene_bridge.rs` — RUST UNIT TESTS asserting the
   suffix behaviour directly (`failure_receipt_companion_from_authority("ordinary_fn")`, and
   the `w_thing_holds` / `w_thing_passes` pairs).

These are not merely three assertions to update. They are three assertions **IN TWO
LANGUAGES** of a contract being deleted, and **only one of the three is reachable by CI**.
A lane that updates the `.dag` pair and ships will believe it is green.

**(3) WILL NOT RED CI.** The Rust suite was removed from CI 2026-07-11 (operator ruling) and
runs locally only.

> **ACCEPTANCE STEP, REQUIRED:** run
> `cargo test -p v1-compiler --lib test_module_hygiene`
> locally. Nothing else will run it. Leaving a stale assertion in the language CI does not
> check is precisely how this fork stayed invisible — splitting it out would reproduce the
> condition that created it.

## Two witnesses this lane owes

- the updated non-witness case, now asserting `Present` with the derived name;
- a NEW one proving a name with NEITHER suffix derives a companion. **That is the
  discriminating RED for the whole change — it fails on main today.**

## Do not disturb: the gates are the working example

Ten of the sixteen companions are effect-gate companions under `dag/tools/`
(`floor_effect_gate_witness` x4, `dag_compile_clean_gate`, `generated_artifact_gate`,
`prose_row_introduction_gate`, `extdeps_scope_placement_gate`). The channel is unused across
the WITNESS corpus and deliberately used by the gates — the population that most needed it.
`dag/tools/floor_effect_gate_witness.dag` `floor_gate_failure_receipt_note` carries a
hand-authored "mute frontier" with counts; check whether this change moves them.

This strengthens the case for totalising rather than merely adding to the number. It is not
"nobody uses it" — **the population that most needed loudness found the convention and
adopted it consistently, and the population that could not reach it by name did not.** The
strip-then-append shape already guarantees the gates are undisturbed, since every one of them
is a `_holds`/`_passes` pair today.

## `*_verdict_diagnostic` — PRECONDITION, not a footnote

Zero producers, corpus-wide, ever, while its derivation AND consumer both exist. Inert by
§6's definition, and not excusable as new or niche. **READ WHAT THE CONSUMER DOES WITH IT
BEFORE PROPOSING DELETION** — zero producers makes it inert, it does not make it
interchangeable with the failure receipt, and deleting a channel whose consumer does
something the other cannot is how a 0% arm turns out to have been the only route to
something.

## The precedent to cite, and it points the right way

`witness_file_from_source` already carries a standing note: *"ONE FACT, TWO COMPUTATIONS, IN
ONE BINARY — a §3 defect, recorded here rather than silently carried"*, about that text scan
versus `reads_live_tree_effective`. It was resolved by observing the two consumers ask
DIFFERENT questions, and deleting the floor's copy rather than unifying. **Here the two
consumers ask the SAME question, so the same reasoning gives the opposite disposition —
unify.** A precedent that flips once you check which question each consumer asks is stronger
than one that simply agrees.

## One line, recorded not actioned

The discovery scan is COLUMN-ZERO only (`strip_prefix` on the raw line). An indented
`test fn` is silently undiscovered — not refused, not counted, absent. Occupancy is ZERO
today, so this is a quiet guard and not a dead one: **do not "fix" it**. It is noted because
this lane is editing the neighbourhood and the first person to indent a witness inside a
block gets a test that never runs and never says so.

## Why this is worth a lane

Displaced cost, denominated: three cold remote builds in one night, across two sessions, all
spent recovering located information the producing code already had in hand. When a wall
refuses, the floor says `returned Bool(false)` and the reason it computed is discarded.
