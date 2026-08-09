# CI-0 witness-execution census (commit-3 receipt, 2026-08-08)

Classification of the complete enrolled witness roster, exactly once, BEFORE
execution, per the CI-0 recut. Model authority:
`gunbc.witness_execution_class` (`WitnessExecutionClass`,
`NativeBlockedCause`, `classify_witness`, census folds); executed controls in
`gunbc.test.claim.witness_execution_class_test` (5 green incl. the
planted-duplicate RED).

## Measured population (head c13acf4d8f + census additions, 2026-08-08)

Counts are a function of the tree and move with every main merge (the first
draft's figures predated two merges); the numbers below are pinned to this
head, and the LAWS — not the numbers — are what the live witness
(`witness_live_roster_census_holds`, falsifier long lane) asserts.

- **9,353 test-fn identities across 1,272 files; 0 duplicate identities**
  (entry::function grain).
- File-grain `live_tree_disposition` projected to fn grain:
  - `SubstrateInputsOnly`: **6,118** fns
  - `ReadsLiveTree` declared: **913** fns
  - undeclared (= `ReadsLiveTree`, the fail-closed default): **2,322** fns

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
- `NativeBlocked { GeneralWitnessTargetModelUnavailable }`: **6,115** — every
  other hermetic fn (6,118 − the 3 carried members). ONE structural cause at
  class grain; the construct sub-census below is its plannable decomposition.
- `NativeLiveObservation`: **3,235** (913 declared + 2,322 undeclared-default).
  Also unemittable today for the same structural cause; classified on their
  own axis first because their receipts must bind the observed subject.
- `EffectObligation` / `InterpreterSubjectTest`: bounded residues inside the
  explicit `CiSpec.witness_entries` roster (execution-kind rows); no
  discovery-corpus fn is effect-dispatched without such a row.

## Outcome reading (operator merge bar, recut msg_c11d9173)

Native-carriable fraction today: **3 / 9,353 (0.03%)**. That is Outcome C for
commits 4–6 (whole-corpus one-bundle emission): the emitter cannot carry a
useful fraction of the enrolled roster, and this census is the falsification
receipt — the blocker is the general witness→TargetModel producer, which is
the body-lowering keystone, not a CI-0-sized fix.

Standing recommendation (decision is the operator's): commits 1–3 remain
individually sound — the absorbing-fallback deletion (§5 hard bar), the
lint-envelope root fix, the located-refusal stderr/build_log surfacing, and
the exactly-once classification model — while the whole-corpus emission is
falsified by this census until the general body producer lands.

## Declared classification precedence (operator ask, msg_f9334d68)

`declared_class_precedence` in `gunbc.witness_execution_class` is now DATA:
`EffectAxis > InterpreterSubjectAxis > LiveObservationAxis > CarriageAxis`,
with the justification on the carrier note and dual-description witnesses
(`witness_declared_precedence_decides_dual_descriptions` + a reversed-order
RED) asserting the classifier realizes exactly the declared order. An
undeclared tie-break is the silent-authority class this model exists to kill.

## Construct-grain sub-census of NativeBlocked (the producer PR's ranked worklist)

Surface-syntax grain over the 6,118 hermetic test-fn bodies (regex over
authored source — declared as approximate; the exact production census
arrives when the forward row-selected fold can parse-classify each body).
A fn counts once per construct it contains; every blocked body needs ALL its
constructs lowered before it rides native, so a stage "unlocks" a fn only
when it zeroes the fn's last missing construct.

| construct | fns containing it | share |
|---|---|---|
| named call (incl. recursion) | 5,859 | 95% |
| record construction | 2,832 | 46% |
| arrow-lambda sugar (`=>`) | 2,613 | 42% |
| match expression | 2,434 | 39% |
| field/method projection (`.`) | 2,038 | 33% |
| let binding | 1,919 | 31% |
| list literal | 1,050 | 17% |
| qualified cross-module ref | 467 | 7% |
| fold-family call | 285 | 4% |
| pipe chain | 82 | 1% |
| fn-literal closure | 65 | 1% |
| if/else | 54 | <1% |
| string interpolation | 8 | <1% |

**Reclassification finding:** 75 hermetic-declared fn bodies reference effect
surfaces (`Filesystem.`/`Exec.`/host-effect markers) — candidates to
reclassify to `EffectObligation`/`NativeLiveObservation` rather than stay
`NativeBlocked`; each is also a candidate lying-disposition row for the
falsifier's counted-divergence channel.

**Staging read:** Stage A of the general body producer (fn_decl/fn_literal →
Arrow, let → Bind, calls → Transform with resolved heads) attacks the top
six rows at once — they are the within-body core. The long tail
(interpolation, closures-as-values, pattern-payload binders) maps onto the
design doc's declared refusal rows.

## Terminal condition (operator, msg_b6cb7efd): tests entirely on v2

The end state is NO v1 execution in test paths: v2-emitted native for
hermetic witnesses, v2 evaluation with subject-bound receipts for
live-observation, and v1 executing ONLY where v1 is itself the subject
(`InterpreterSubjectTest` — a tight roster where every member must name v1
as its subject). The scoreboard metric per producer stage is
**v1-executions-remaining**, which must fall to the InterpreterSubjectTest
roster size and stop. Today that number is the whole roster minus 3.

## Commit B sweep receipt — canonical per-identity realization attempts (2026-08-09)

The circular census (classifier-agrees-with-enrollment) is replaced by
ACTUAL canonical realization attempts: per roster entry, the executor's own
`discover_source_root_reads_for_entry` closure, transported through the
`host_source_root_ingest_manifest` overlay (the frontier-probe survey
pattern), driven by `v2.workflow.realization_sweep` — ONE
`assemble_program_from_ingest` per entry, one exactly-one identity join →
`infer` → `compile_inferred TranslateTo` per authored test-fn identity.
Driver: `realization_sweep_survey` (local recipe;
`--entries-file <roster>`); a host failure is a counted row, never a
skipped entry.

Stratified sample (10 entries across dag/test/claim,
src/v2/test/claim/manual, src/v2/test/claim/emit — incl. both keystone
files, a large witness file, and small smoke files):

```
entries 10   identity_rows 116   host_error_rows 0
phase      exact_cause                          count
frontend   parse_g0_tokens_remain               100
frontend   tokenize_lex_e1_unrecognized_char     16
located    dag/extdeps/cache/materialization.dag 16   (tokenize bucket)
```

**Reading: 100% of sampled identities refuse at PhaseFrontend.** Assembly
parses every closure member, so the first member the v2 grammar cannot
parse gates the whole entry — no identity in this sample reached resolve,
infer, or translate through its real closure. The translate-stage buckets
measured earlier (cross-module named-call emission, staging-lex literals,
match-binder resolve) remain real but are BEHIND the frontend wall on
every real closure; they were measured on planted/synthetic closures.
Burn-down order is therefore: (1) frontend grammar coverage of the live
std/extdeps closure population (the `parse_g0_tokens_remain` residue —
located-file attribution for this bucket is a named gap: those diagnostics
carry PortLocus, not Textual, so the sweep's `located` column is empty for
them; the tokenize bucket locates exactly, all 16 rows at
`dag/extdeps/cache/materialization.dag`), then (2) the translate buckets,
then (3) the test-marker grammar extension (operator holds the std.grammar
design pass; the capture scaffold already unblocks measurement).

Laws asserted by execution (hermetic mechanism witnesses,
`dag/test/claim/realization_sweep_test.dag`): exactly-once accounting
(rows == authored test-fn roster), witness-absent RED (a located refusal
row, never an empty roster), receipt/histogram render every row. The two
assembly-bearing witnesses are enrolled on the falsifier substrate long
lane (measured ~16-17.5s each); identity-scan and RED stay per-PR.
