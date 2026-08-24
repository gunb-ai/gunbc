# LANE BRIEF — node://adhoc-602d7286-8ff
## Failure-evidence admission: bind the producer, do not promote the spelling

**OPERATOR RULING, 2026-08-24 08:20, and it supersedes the two earlier framings in this
brief's history.** Both were name-based and both were wrong in the same direction.

> **Yes: give the next free lane to failure-evidence admission before opening another
> board-root investigation lane.** But do not implement the wall as *"a witness name must end
> in `_holds` or `_passes`"* — that would preserve the current defect as policy. The right
> wall is: **an executing enrolled witness must have a resolvable failure-evidence producer.**
> The suffix convention may remain as a temporary compatibility lookup. It must not become the
> semantic authority.

**Superseded — do not build either:**

- *"refuse at enrolment for a witness whose name cannot derive a companion, counted then
  blocking"* (mine). This codifies the suffix as the eligibility rule. The operator tested
  exactly this hypothesis and rejected it.
- *"delete the suffix test so the derivation is total"* (deep-ant's, and better). It fixes
  one of four conflated states and leaves the mechanism `String → String`. The suffix stays
  as a **compatibility projection**, not as the law and not as a totalised law.

---

## The four states the suffix conflates

A single name-derivation answers all of these at once, and they are different facts:

    1. a companion spelling can be derived from this name
    2. a companion function actually exists
    3. that companion has the required type
    4. that companion reports the reason for THIS witness evaluation

`foo_holds` derives `foo_failure_receipt` **even when no such producer exists**. Conversely a
witness with a legitimate but arbitrary name may have an explicitly bound producer and must
not be refused because its spelling does not encode that fact.

So the wall must consume a real binding:

    WitnessIdentity { entry, function }
            ↓
    FailureEvidenceBinding { producer identity, evidence type }

not `String → String manipulation`.

## The denominator is NOT 12278

Admission is denominated over **enrolled ∩ scheduled ∩ expected to execute**, never over every
declaration. Some `test fn`s are offline recipes, deliberately declined long-home rows,
fixtures, unscheduled cadence members, dormant local probes. *A declaration with no current
executing consumer is a separate standing and must not be refused for lacking a runtime
channel that nothing invokes.*

I measured 12278 declared and flagged in passing that the floor's routed roster is ~10439 —
then quoted the 12278 anyway. **The routed roster is the denominator.** The join is:

    declared witness identity  ×  execution standing  ×  failure-evidence standing

with terminal dispositions along the lines of:

    ExecutingWithTypedEvidence
    ExecutingWithLegacyTextEvidence
    ExecutingWithoutFailureEvidence
    DeclaredButNotScheduled
    CompanionNameDerivableButProducerMissing
    ProducerPresentWithWrongType

**That is the only census this lane needs.** Do not turn it into a corpus-wide witness redesign.

## One authority, three projections

Do not preserve three independently implemented answers (`Bool` producer, `String` receipt
producer, diagnostic producer). The terminal shape is:

    one typed verdict/evidence producer
        ├── Bool projection        for legacy witness execution
        ├── Diagnostic projection  for the floor
        └── textual rendering      where a string is still required

Precedent already in tree: `LensVerdict<T>` distinguishes satisfaction, violation,
non-applicability and unrealized, and its non-success arms carry a `Diagnostic`.

**This does NOT mean minting another universal verdict carrier.** It means the failure-evidence
channel consumes the domain's actual verdict where one exists, and legacy text renders *from
that fact* rather than recomputing it.

## Rollout — staged, and Phase 1 must not alter admission

**Phase 1 — measure the active population, count-only.** Executing enrolled witnesses only,
with exact reconciliation: `executing total = typed + legacy + unbound`.

**Phase 2 — no-growth wall.** Immediately refuse a newly enrolled executing witness with no
evidence binding; an existing witness whose enrollment is modified and remains unbound; a
purported companion spelling whose producer does not exist; a producer whose result type is
wrong. **Grandfather the measured legacy population as explicit migration debt.** This closes
the writable hole without turning the roster red.

**Phase 3 — one end-to-end specimen, with a mutation control.**

    witness evaluates → typed non-success verdict exists → Bool projection is false
    → located diagnostic fetched from the SAME verdict → floor prints that diagnostic

    change the producer's diagnostic → floor output changes
    change only the Boolean wrapper  → evidence still identifies the producer result

That is what proves the channel is not decorative.

**Phase 4 — migrate by touch and by value**, in order: required-floor witnesses producing
opaque false; witnesses used as discriminators for emission/root repairs; high-cost witnesses
whose opaque failure triggers expensive reconstruction; remaining active legacy rows. Full
blocking only when the grandfathered population reaches zero **or the operator explicitly
ratifies a final cut**.

## Acceptance bar

    1. Actual active-roster denominator, not all test declarations.
    2. Explicit failure-evidence binding at witness identity grain.
    3. Legacy suffix lookup retained only as a compatibility projection.
    4. No-growth admission wall.
    5. One real *_verdict_diagnostic producer exercised end to end.
    6. Missing/wrong-type producer refuses at enrollment.
    7. Witness NotExecuted remains distinct from witness false.
    8. Legacy text receipt renders from the same evidence rather than recomputing.
    9. Exact grandfathered population and dissolution trigger.

**It must NOT:** rename 10,000 functions · migrate every witness in one PR · add another
general verdict type · rewrite the floor · block on declared-but-unscheduled tests · open a
census of what each witness ought to say.

---

## Measurements that stand (and how they were wrong before you inherit them)

    test fn at column zero (dag+src/v2+src/v1)   12278    <- NOT the denominator
    floor routed roster (DESIGN, older run)     ~10439    <- closer to it
    eligible by _holds/_passes suffix             1870
    ineligible by name                           10408
    authored *_failure_receipt                      16
    eligible witnesses WITH a companion              4
    authored *_verdict_diagnostic                    0

**READ THIS BEFORE RUNNING YOUR OWN SCAN.** An earlier revision said 6 companions and 3 pairs.
Both wrong, produced independently by two sessions from the same defective pattern: it matched
`^fn ` only, and **`func` is also a declaration keyword here** (269 against 36654 `fn`); every
missed companion was a `func`. A separate re-run then returned 2 `*_verdict_diagnostic`
producers — a pattern with no END anchor matching `witness_verdict_diagnostic_companion` (the
derivation itself) and `lens_verdict_diagnostic_locus_module` (substring only). Anchored, it is
0. Four defective greps in one night between two sessions; the only one that reached nobody was
caught because the finding happened to be interesting enough to double-check. Match both
keywords, anchor both ends.

## The duplication, measured — this is why (3) says *compatibility projection only*

`run_claim_failure_receipt` and `run_witness_verdict_diagnostic` are **byte-identical modulo one
string literal**: same `match v1_interpreter::run_in_context(ctx, function, false)`, same four
arms in the same order, same `Ok(Value::Str(s)) => s.to_string()`, same
`Err(NoSuchFunction { .. }) => String::new()`, same wrong-type and error formats — differing
only in whether the sentinel reads `failure_receipt_refused:` or
`witness_verdict_diagnostic_refused:`. **The only thing either carries that the other does not
is the word in its own error message.**

The appenders match too, and every call site is a lockstep pair — both invoked on adjacent
lines, same order, same target string, at all three sites (inside
`seed_runner_bool_false_failure_detail`, in `claim_executor`'s Bool(false) path, and in the
discovery-summary failure path). *No offsets given deliberately: an earlier revision carried
line numbers and they were wrong for the reader's tree the moment they were written — measured
from a branch several merges behind main, ~600 lines adrift on two of three. Name the symbol
and grep (§3).*

So this is a §2 duplicate as well as a §3 fork: two derivations, two identical runners, one of
which has never had a producer.

## Safe by construction — checked, do not redo

- **Missing companion costs nothing.** `run_claim_failure_receipt` maps
  `InterpError::NoSuchFunction` to `String::new()` and the caller appends only when non-empty.
- **No Rust mirror of the derivation.** `failure_receipt_companion_from_authority` calls the
  `.dag` function through the interpreter.
- **Floor discovery has no eligibility rule.** `witness_file_from_source` is
  `line.strip_prefix("test fn ")` plus a path filter, and nothing else sits between the
  declaration and the roster. (It is **column-zero only** — an indented `test fn` is silently
  undiscovered. Occupancy is zero today, so this is a quiet guard, not a dead one: **do not
  "fix" it**; it is noted because this lane edits the neighbourhood.)

## Three assertions of the old contract, in two languages

Only one is reachable by CI. A lane that updates the `.dag` pair and ships will believe it is
green.

1. `test_module_hygiene_hand_rust_equivalence_witness_test`
   `test_module_hygiene_failure_receipt_companion_absent_on_non_witness`
2. its sibling `codex_wet_enrolled_witnesses_resolve_failure_receipt_companion`
3. `src/v1/stage0/src/cli_run/test_module_hygiene_bridge.rs` — Rust unit tests asserting the
   suffix behaviour directly

> **ACCEPTANCE STEP, REQUIRED:** run `cargo test -p v1-compiler --lib test_module_hygiene`
> locally. The Rust suite was removed from CI 2026-07-11 and nothing else will run it.

## Do not disturb: the gates are the working example

Ten of the sixteen companions are effect-gate companions under `dag/tools/`
(`floor_effect_gate_witness` ×4, `dag_compile_clean_gate`, `generated_artifact_gate`,
`prose_row_introduction_gate`, `extdeps_scope_placement_gate`). The channel is unused across
the witness corpus and adopted **consistently by the gates** — the population that most needed
loudness found the convention; the population that could not reach it by name did not.
`floor_effect_gate_witness.dag` `floor_gate_failure_receipt_note` carries a hand-authored
"mute frontier" with counts; check whether this lane moves them.

## Priority, with the operator's correction to my claim

I argued "every future lane depends on it." **Too broad.** Rustc board failures already carry
located compiler diagnostics; this channel does not help an E0308 or E0004 trace. It directly
improves witness-based discriminators, construction guarantees, red controls, cost and
admission failures, behavioural receipt diagnosis, and any repair whose acceptance depends on a
Boolean witness.

    existing merge-ready and in-flight board repairs:  continue
    next free lane:                                    failure-evidence binding + no-growth wall
    new broad board-root investigation:                wait until that lane is staffed

**One lane only.** Once the no-growth wall and one executed diagnostic path exist, return the
lane to convergence rather than beginning a mass witness migration.

## Displaced cost, denominated

Three cold remote rebuilds in one day across two sessions, recovering located facts that
already existed at the producing boundary. The shape:

    producer computes a discriminating reason → witness projects to Bool → floor receives false
    → reason discarded → another lane performs a cold reconstruction

Every false witness currently invites a causal investigation that may reproduce an answer the
program had already computed.
