# InputEnvelope: bounded input declaration for WorkDemand + CI corpus instance

## Why (one paragraph)

An unbounded input is the data-side of a non-terminating Loop (which requires DescentEvidence per std/induction.dag). A WorkDemand that does not declare its input envelope is therefore fail-open by construction: it can accept an arbitrary corpus with no declared ceiling, and the scheduler has no basis for admission control. Mirroring DescentEvidence (Strict | NonIncreasing | DescentUnknown), the InputEnvelope is BoundedInput (the declared ceiling, analog of Strict) or EnvelopeUnknown (honest fail-closed bottom, analog of DescentUnknown). Admission is decided by input_admitted, which is the first-class gate that closes the loop: EnvelopeUnknown => RefusedUndeclared (fail-closed); a declared axis whose actual count exceeds the ceiling => RefusedOverEnvelope; an actual axis not covered by any declared bound => RefusedOverEnvelope (undeclared axis = refuse); all within bounds => Admitted.

## P1 (this PR) — shape + CI corpus instance + admission witness

- InputSizeAxis closed enum: WitnessCount | SourceNodeCount | CorpusNodeCount
- InputBound record: axis: InputSizeAxis, max: Measure<Count, One, Nat>
- InputEnvelope coproduct: BoundedInput \{ bounds: List<InputBound> \} | EnvelopeUnknown
- AdmissionVerdict coproduct: Admitted | RefusedOverEnvelope \{ axis \} | RefusedUndeclared
- input_admitted function: EnvelopeUnknown => RefusedUndeclared; per-axis ceiling check
- WorkDemand.input_envelope required field added; existing demands carry EnvelopeUnknown (honest: ceiling not yet declared)
- gunbc_ci_corpus_envelope: the CI corpus as the first BoundedInput instance — ceiling values are operator-set operating-point ceilings, NOT derived quantities (P2 derives them; P1 scaffolds with explicit Scaffold-disposition values)
- input_envelope_admission_test.dag: 5 test fns covering at-limit, over-limit, undeclared-axis, EnvelopeUnknown, and a red-on-revert discriminating assertion

## P2 (follow-on, operator sign-off before landing) — ceiling derivation

- Derive the WitnessCount ceiling from the marker-driven corpus discovery roster (gunbc.ci_layer_roots witness_discovery_scan_dirs scan; count is known at CI-spec emit time)
- Derive SourceNodeCount and CorpusNodeCount from the measured per-file / per-tree node counts (the ci_floor_measurement infrastructure)
- Replace the Scaffold ceiling rows in gunbc_ci_corpus_envelope with derived values; the Scaffold disposition dissolves on this replacement
- Wire input_admitted into the scheduler / admission path as a fail-closed gate (operator controls timing; this PR's gate is a wall on WorkDemand construction, not a runtime check yet)

## Ceiling value disposition (P1 scaffolds)

The three ceiling values in gunbc_ci_corpus_envelope (WitnessCount: 2000, SourceNodeCount: 500000, CorpusNodeCount: 5000000) are operator-set operating-point ceilings. They are NOT derived quantities — the derivation is P2. These are explicit Scaffold-bearing values: they must be replaced with derived values when the P2 derivation lands. The numbers are chosen as round conservative ceilings well above current corpus size; do NOT treat them as authoritative measurements.

## Dissolution trigger (DESIGN §6)

Delete when P2 ceiling derivation lands and gunbc_ci_corpus_envelope carries derived values, input_admitted is wired as a runtime gate, and all WorkDemand instances carry either a real BoundedInput or an explicit escalation comment explaining why they are EnvelopeUnknown.
