# CI-0 witness-execution census (commit-3 receipt, 2026-08-08)

Classification of the complete enrolled witness roster, exactly once, BEFORE
execution, per the CI-0 recut. Model authority:
`gunbc.witness_execution_class` (`WitnessExecutionClass`,
`NativeBlockedCause`, `classify_witness`, census folds); executed controls in
`gunbc.test.claim.witness_execution_class_test` (5 green incl. the
planted-duplicate RED).

## Measured population (branch head, dag/** + src/v2/** `*_test.dag`)

- **8,828 test-fn identities across 1,121 files; 0 duplicate identities**
  (entry::function grain).
- File-grain `live_tree_disposition` projected to fn grain:
  - `SubstrateInputsOnly`: **5,868** fns (incl. 2 via the one
    qualified-spelling row in `crosstree_probe_test.dag`)
  - `ReadsLiveTree` declared: **888** fns
  - undeclared (= `ReadsLiveTree`, the fail-closed default `v2.std.live_tree`
    declares): **2,072** fns

## Classification against the emitter's actual dispatch surface

The v2 Rust emission path is **fixture-bound**: every emittable member carries
a hand-authored `TargetModel` (binding spellings + lex rows), and the general
witness→TargetModel producer does not exist — the general body producer is the
named keystone gap (DESIGN body-lowering thread; witness-realization plan P6
names the already-native-clean logic family as the first executable subset).
The refusal is structural (no producer to call), and its executed
discriminating evidence is the corpus's existing emit-refusal controls (the
canonical emitter refuses underived input rather than fabricating).

- `NativeRequired` (hermetic ∧ emitter carries): **3** — the selected-bundle
  members meet/join/complement, bundle identity `09d7d2e53554c783`, executed
  native with planted-red discrimination and interpreter-oracle equivalence.
- `NativeBlocked { GeneralWitnessTargetModelUnavailable }`: **5,865** — every
  other hermetic fn. ONE structural cause, not 5,865 independent diagnoses.
- `NativeLiveObservation`: **2,960** (888 declared + 2,072 undeclared-default).
  Also unemittable today for the same structural cause; classified on their
  own axis first because their receipts must bind the observed subject.
- `EffectObligation` / `InterpreterSubjectTest`: bounded residues inside the
  explicit `CiSpec.witness_entries` roster (execution-kind rows); no
  discovery-corpus fn is effect-dispatched without such a row.

## Outcome reading (operator merge bar, recut msg_c11d9173)

Native-carriable fraction today: **3 / 8,828 (0.03%)**. That is Outcome C for
commits 4–6 (whole-corpus one-bundle emission): the emitter cannot carry a
useful fraction of the enrolled roster, and this census is the falsification
receipt — the blocker is the general witness→TargetModel producer, which is
the body-lowering keystone, not a CI-0-sized fix.

Standing recommendation (decision is the operator's): commits 1–3 remain
individually sound — the absorbing-fallback deletion (§5 hard bar), the
lint-envelope root fix, the located-refusal stderr/build_log surfacing, and
the exactly-once classification model — while the whole-corpus emission is
falsified by this census until the general body producer lands.
