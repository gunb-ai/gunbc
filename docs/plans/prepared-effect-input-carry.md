# Prepared effect input carried into nullary warm rows (PR two of the #10994 chain)

ADJUDICATED AND IMPLEMENTED. The model below was sent to eager-raven-113, fierce-lark-661 and
bright-boar-435 before any code was written; all three approved both decisions (2026-09-14), each
with one condition, and every condition is discharged in the landed change:

- the carried value's DISPOSITION is printed on the floor's phase line beside its content digest,
  so a run whose carrier refused is diagnosable rather than silent (bright-boar's condition on
  Decision 1);
- `std.materialization_ladder` is cited IN SOURCE, not only here: the four obligations are typed
  values in `v2.workflow.floor_prepared_effect_input_ladder`, decided by the ladder's own fold and
  exercised by `v2.test.floor_prepared_effect_input_ladder` (the condition both fierce-lark and
  eager-raven attached to Decision 2);
- the binding's absence after the tier is cleared has its own control (eager-raven's condition b).

Two things the evidence found that this document's first cut did not, recorded because both
changed the implementation rather than the prose:

1. `run_in_context` calls `call_function` DIRECTLY, reaching neither the cross-claim tier nor the
   prepared-input binding. A control that evaluated the producer that way asserted a value
   RECOMPUTE also produces — the decoration DESIGN section 4b names. The controls now evaluate
   through an ordinary call site and assert an OBSERVED SERVE from the tier's own hit observer.
2. With that fixed, the changed-carrier RED went red on the REAL implementation: the
   producer-to-carry map held a CLONE of the carry, so re-binding the acquisition left the key
   unchanged and the producer was served a value derived from the previous carrier. It now holds
   the acquisition NODE and reads its current binding at every key derivation.

## The gap, stated at the floor's own vocabulary

`v2.workflow.floor_pure_producer_share` admits two fill dispositions and only two:

- a WARM row is NULLARY and preparation-forceable — `install_pure_producer_share` evaluates it
  once in a Hermetic frame over its own module scope and the fill is billed to preparation;
- a CLAIM-FORCED row has claim-shaped arguments, so it fills inside the fold on first touch and
  the paying claim carries the fill.

A producer whose value depends on one effectful read of a committed carrier fits neither. It
cannot be warmed as a nullary pure row, because `store_cross_claim_pure_memo` is keyed on
(resolved fn node, portable argument row) and an effect-dispatching nullary call contributes
NOTHING to that key — the read's content is invisible to the key, which is the ctx-blind homonym
class in its content form: two different carrier contents would share one entry. And as a
claim-forced row it re-derives per claim, which is the rostered
`shared_precondition_re_derived_once_per_claim_frame`.

The live instance: `gunbc.roadmap_authority.roadmap_authority_projection` — one
`Filesystem.Read` of `dag/gunbc/roadmap/roadmap_acceptance_event_history.jsonl` (16 lines,
~10 KB) followed by a pure ~186 ms fold, demanded by seven required-lane claims and by #10994,
whose current blocker is an enrolment margin RED (306 ms then 279 ms against a 302 ms margin)
sitting on top of exactly this shared precondition.

## One fact that makes the carry legitimate rather than a memo of an effect

Hermetic mode already classifies this read as INPUT ACCESS, not a host effect: the
checkout-input carve-out in `v1_interpreter::eval_service_call` dispatches a readonly
`Filesystem.Read`/`List` WET when `hermetic_checkout_input_disposition` confirms the path under
the checkout root. The carrier is such a path. So the value is a function of the commit the run
prepared — constant for the whole prepared subject — and carrying it once is not caching an
effect, it is binding an input the run already holds fixed. Anything the disposition cannot
confirm (out-of-root, `.git`/`target`, unresolvable) is refused by that existing wall and is
therefore never carryable here either.

## The model

Two declarations in `v2.workflow.floor_pure_producer_share`, beside the two existing rosters.

    type PreparedEffectInput {
      // The nullary declaration whose evaluation ACQUIRES the input. Resolved to a fn node at
      // preparation, exactly like every rostered producer — never admitted by bare name.
      acquisition: String
      // Why the acquisition is a run-constant checkout input rather than host state, and the
      // consumers the carry serves. Evidence fields, same discipline as RefusedShareCandidate:
      // name the instrument, never transcribe its output.
      ground: String
      measurement: String
    }

    type CarriedInputWarmRow {
      producer: String     // the pure fold, resolved to its fn node
      parameter: String    // the named parameter the carried value fills
      input: String        // which PreparedEffectInput.acquisition supplies it
      measurement: String
    }

    data floor_cross_claim_prepared_effect_inputs: List<PreparedEffectInput>
    data floor_cross_claim_carried_input_warm_rows: List<CarriedInputWarmRow>

### What preparation does

`install_pure_producer_share` gains one phase, ordered BEFORE the existing warm loop:

1. Resolve each `acquisition` to its fn node in a frame over the prepared subject (the existing
   `floor_authority_frame`, unchanged — Hermetic). An unresolved spelling or a module outside the
   subject is a stale row and STOPS THE LINE, exactly as a stale warm row does today.
2. Refuse at install any acquisition whose declaration is not nullary: a carried input with
   arguments is a different concept and must not be smuggled in under this one.
3. Evaluate it ONCE, under `observe_shared_build`, so the acquisition is billed to preparation
   and adjudicated by the same preparation refusal as every other shared build.
4. Reify the value totally to `PortableValue` — the SAME total walk publication already performs
   — and take its content identity as the structural digest over that form (`std.content_hash`
   `Fnv1a64Structural`). Printed on the `[floor-phase]` line, so the receipt says WHICH content
   was carried, not merely that something was.
5. Bind (fn node → carried `Value`) for the run's prepared subject, cleared with the tier by
   `clear_cross_claim_pure_memos`.

Then, for each `CarriedInputWarmRow`, warm the producer with a ONE-ELEMENT argument row
`[(parameter, carried value)]` instead of `[]`. Everything downstream is the existing mechanism
untouched: the store keys on (producer fn node, portable argument row) and serves only after the
full portable argument row verifies structurally equal.

### What a claim does

- A claim's call of the ACQUISITION declaration is served the carried value instead of
  re-dispatching the read. This is the only genuinely new serve path, and it is deliberately NOT
  the pure memo: the pure memo refuses any call that dispatched an effect (`effect_dispatch_count`
  guard), and that refusal is correct and stays. Admission here is a separate, roster-declared
  binding keyed on resolved fn-node identity, installed for one prepared subject.
- A claim's call of the PRODUCER passes the same value it just received, so the portable argument
  row equals the one preparation stored, and the existing cross-claim tier serves it — no new
  keying, no new retention, no new lookup rule.

### Why content identity is not a new mechanism

The carried value IS the key material: a changed carrier content produces a different portable
argument row, `cross_claim_portable_args_match` fails, and the claim recomputes. Stale serve is
therefore unwritable rather than guarded — the §5 preference for construction over validation.
Roster identity is the resolved fn node on both halves (acquisition and producer), so the
bare-name homonym class stays closed exactly as review 57446 closed it.

## Refusal arms (never an empty default)

| state | disposition |
|---|---|
| acquisition module outside the prepared subject | `PreparedEffectInputModuleOutsideSubject` — line stop |
| acquisition spelling resolves to no declaration | `PreparedEffectInputUnresolved` — line stop |
| acquisition declaration is not nullary | `PreparedEffectInputNotNullary` — line stop |
| acquisition evaluation errors (read refused, hermetic refusal, parse error in the fold) | `PreparedEffectInputAcquisitionFailed` — line stop, carrying the interpreter's own located error |
| acquired value fails total reification | `PreparedEffectInputNotPortable` — line stop, with the located path/kind from the refusal itself |
| carried-input warm row naming an unknown input | `CarriedInputWarmRowInputUnknown` — line stop |
| warm evaluates but the store refuses it | the existing `PureProducerShareWarmNotStored` — line stop |

One arm is deliberately NOT a floor refusal, and it is the one worth arguing about: when the
acquisition SUCCEEDS and returns its domain type's own refusal arm (here
`RoadmapAcceptanceEventHistoryLoadRefused`), the floor carries that value faithfully and every
claim receives the identical typed refusal it would have computed for itself. The floor does not
read domain types and must not decide that one variant of a caller's coproduct means "stop". The
alternative — the floor stopping the line on a refusal arm — would require the floor to know
`gunbc.roadmap_authority`'s vocabulary, which is the layer inversion §3 forbids. Stated here as a
deliberate divergence so it is decided rather than discovered.

## Home in the materialization domain (§3b), and one stated divergence

This inhabits the materialization / carried-value home: a computation identity (the resolved
declaration) joined to declared inputs (the carried value's content digest), a provider whose
scope reaches the demands' least common visible ancestor (preparation, which is the LCA of the
seven claims), a key representing every declared input the result depends on, and retention
spanning exactly the obligated lifetime (one prepared floor run). §2's ordering is respected in
the direction it demands: the demand graph is minimized FIRST — the repetition is authored
duplication whose shared-state ancestor is preparation, so the repair is the CARRY, and the
existing cross-claim tier is merely where the carried value is held.

The divergence, stated: this does not mint a new `std.materialization_provider.ArtifactRequest`
variant. `ArtifactRequest` keys durable, cross-run artifacts by identity digest with a realization
receipt naming the provider; the carried input is an in-process value bound for the lifetime of
one prepared subject, with no store, no path, and no cross-run reuse. Minting a variant there
would put a run-scoped binding into the durable-artifact algebra. If the reviewers read that the
other way, the alternative is a real one and this is the decision to take before code.

## Evidence

### Executed at fixture grain, through the real `install_pure_producer_share` path

`cargo test --release -p v1-compiler --lib pure_producer_share` — 17 passed, 0 failed. Five of
them are this change's:

- the input is acquired ONCE and its producer warmed over it, with the producer then SERVED
  (asserted from the tier's hit observer, not from the value);
- **the discriminating RED**: a changed carrier content is not served the value derived from the
  old one. Discrimination receipt — with `cross_claim_key_args` returning `None` (the
  empty-argument-row implementation this control targets), this test FAILS; with the real
  implementation all 17 pass;
- the prepared-input binding is absent after the tier is cleared, and the producer answers for
  itself again;
- a carried row naming an input nobody prepares stops the line;
- a non-nullary acquisition is refused.

### Still owed at live grain (stated as owed, not as done)

## Evidence plan (the live arm)

1. **Identical verdict.** The seven required-lane claims plus #10994's forecast witness produce
   the same verdicts with the rows enrolled as without. A carry that changes a verdict is a
   defect, not a speedup.
2. **Present-vs-absent, per row, on one tree.** Two `workflow_dispatch` runs on a branch ref
   pinned to an exact sha (the corrected instrument recorded in the roster header: a pushed side
   branch triggers nothing, and a PR pair measures two different merge refs), reading
   `required_floor_claim_cost.tsv` and the `[floor-shared-fill]` ledger. Admission is
   serve-below-recompute at identity grain; a row that does not clear it is withdrawn into
   `floor_cross_claim_refused_candidates` rather than defended by a family aggregate.
3. **No stale serve (the discriminating RED).** A control that mutates the carrier's content and
   shows the previously carried value is NOT served: the portable argument row differs, the
   producer recomputes, and the verdict follows the new content. Red goes green only by the key
   actually carrying content identity — an entry keyed on the empty argument row passes the happy
   path and fails this control.
4. **Acquisition is dispatched once.** The effect-dispatch count over a floor run is 1 for the
   carrier read with the rows enrolled, N without.
