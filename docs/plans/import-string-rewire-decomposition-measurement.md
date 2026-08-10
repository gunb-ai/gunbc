# Import-string rewire decomposition measurement

**Lane:** floor-prep-tax-program P1 follow-on
**Predecessor:** [entry-view assembly direction](entry-view-assembly-direction-receipt.md)
**Status:** original measurement invalidated; corrected same-head streaming A–B–B–A control is liveness-confounded; no optimization selected

## Question and accounting basis

`rewire_type_env_import_str_binding_identity` was the largest surviving named
entry-assembly row, but one timer could not identify what construction was repeated.
This receipt partitions it at boundaries declared in `src/v1/04_infer.dag`:

1. build the closure-wide type-name/exporter index;
2. build the per-module exported-name index;
3. prepare each module's local/import/inherited-key view; and
4. apply ambiguity decisions and persistent-map rewrites.

The four timers are mutually exclusive and nested inside
`assembly_rewire_import_str`. They are inclusive rows in the top-level cost partition,
so the existing exclusive accounting law cannot double-count them. Work counts travel
with the modeled `.dag` result. Production remains on the original one-module-at-a-time
wrapper with no counters. A default-off bootstrap projection times the modeled
boundaries, prepares and applies one module at a time, releases the plan, and calls the
production per-key rewire authority before classifying its pointer-visible before/after
effects.

This is measurement scaffolding, not a Rust implementation of compiler semantics. Its
dissolve-on note is carried by `import_string_rewire_measurement_scaffold_note` in the
model authority.

## Invalidated first observation

The two original observations below are retained so the failure is auditable, but they
are not decision-grade. The host changed the live set by retaining plans and results for
the whole closure, and charged immutable counter updates inside every binding iteration.
That contaminated both peak RSS and the phase reported as dominant. They establish only
the behavior of that invalid instrumented program.

## Invalidated representative subject

Both repetitions used the frozen 50-entry cohort from
`receipts/entry-view-assembly-direction/cohort.tsv` in one `claim_batch` process, with
one `MultiEntryIndex` per repetition.

| Subject fact | Value |
|---|---:|
| binary SHA-256 | `92c497532dd9469b9f2351aa0fb2aaad6d2e1ac0c03c47ebbc1dc3e76148fa54` |
| cohort SHA-256 | `6256ec909647e60044636e3b30eacbbf4769f8bf80ef90d243c9532935c26db5` |
| entry groups / witnesses | 50 / 50 |
| outcomes, both runs | 47 PASS / 3 FAIL, identical names |
| process exit, both runs | 1 (the frozen subject is preserved, not silently repaired) |

The third failure is new relative to the older entry-view receipt because its frozen
function name no longer exists on current main. The other two remain hermetic-effect
refusals. Matching failure identities, module closures, binary hash, cohort hash, and
structural counts establish that the two measurements used the same subject.

## Invalidated result

| Import-string phase | repetition 1 | repetition 2 | share of import-string row |
|---|---:|---:|---:|
| type-name index | 1,056.1 ms | 1,138.2 ms | 2.77–3.04% |
| exported-name index | 136.9 ms | 185.3 ms | 0.39–0.45% |
| module preparation | 2,046.8 ms | 3,127.8 ms | 5.89–7.62% |
| **binding application** | **31,524.7 ms** | **36,590.9 ms** | **89.15–90.68%** |
| timer residual | 0.1 ms | 0.1 ms | <0.001% |
| **total import-string row** | **34,764.6 ms** | **41,042.3 ms** | 100% |

Process elapsed wall was 231.357 s and 235.171 s (1.65% spread). The additive
import-string row had an 18.06% spread and binding application a 16.07% spread; their
range, not either single run, is the decision input.

The structural work counts were identical in both runs:

| Work | Count |
|---|---:|
| module applications | 6,746 |
| direct-import export sets prepared | 30,723 |
| inherited keys / ambiguity checks | 6,928,314 |
| ancestry-map writes attempted | 6,606,131 (95.35% of checks) |
| string-map writes attempted | 0 |
| keys left unchanged | 322,183 (4.65% of checks) |

## Corrected streaming control

The corrected binary restored the original counter-free streaming wrapper and moved the
observer behind `GUNBC_REWIRE_IMPORT_STR_PROFILE=1`. The observer also remains streaming:
it prepares, applies, classifies, and releases one module at a time. These are the exact
subject coordinates for the attempted A–B–B–A:

| Subject fact | Value |
|---|---:|
| source head | `a26ec4fc4e9736dd0a08aaa4ceeedc74d261b91e` |
| binary SHA-256 | `780a21b1684df514e1576b88b2fbc6dfa8ceb6900d4eca9178f17d894fa7c1ad` |
| cohort SHA-256 | `6256ec909647e60044636e3b30eacbbf4769f8bf80ef90d243c9532935c26db5` |
| outcomes SHA-256, every completed arm | `649ccbedf3125b77436586408748b23ed99076c2f581905803b152f908d4d583` |
| subject SHA-256, every completed arm | `dc5ced06ee3ae9e65f01625e0d231379aa078b9ca19c2cab2ec60057193f7630` |
| closure SHA-256, every completed arm | `23cdd1b89183942a0faa6559666adc525842f269e2d23797179bab3b30ae6f66` |
| completed outcomes | 47 PASS / 3 FAIL |

Three arms completed and reconciled their timing partitions:

| Arm | process wall | peak RSS | import-string row |
|---|---:|---:|---:|
| baseline-r1 | 170.336 s | 8,150,912 KiB | 30.506 s |
| profile-r1 | 161.340 s | 8,107,256 KiB | 28.937 s |
| profile-r2 | 161.276 s | 8,118,392 KiB | 29.772 s |

`baseline-r2` emitted its start marker but no terminal row. It exceeded the declared
900-second per-arm bound, and the remote action still did not regain control after TERM
and KILL, so the action was cancelled. This independently reproduced the same fourth-arm
stall in the first corrected invocation. The missing baseline is not imputed from
baseline-r1, and the two profile arms are not compared against a one-row baseline as if
the required interleaving had completed. Instrumentation wall and RSS tax are therefore
**unmeasured**, not zero or negative.

The two completed observed arms agree on the following classification counts:

| Observed operation | Count |
|---|---:|
| module applications | 6,746 |
| direct-import export sets prepared | 30,723 |
| inherited keys considered | 6,928,314 |
| ancestry insertion attempts | 6,606,131 |
| ancestry key absent | 0 |
| ancestry same `Rc` identity | 6,572,738 (99.49% of insertion attempts) |
| ancestry value changed | 33,393 (0.51% of insertion attempts) |
| string insertion attempts | 0 |
| unchanged keys | 322,183 |

Those counts describe the default-off observer's workload. They do not select an
optimization while the control is liveness-confounded.

The first fresh-process `profile-verify` oracle recursively compared complete
`TypedModule` values. It reached the 899-second bound before emitting a witness outcome
(3,473,452 KiB peak RSS). That run is an invalid oracle-cost observation, not evidence
of semantic disagreement: recursive equality walked compiler graphs far outside the
rewire boundary it was meant to check.

The corrected oracle is field-complete at that boundary. It requires shared identity
for every `TypedModule` and `TypeEnv` field the rewire does not own, and exact key-set
plus binding-identity equality for the two maps it does own. On a fresh process it
completed the full cohort in 203.734 seconds (8,261,600 KiB peak RSS), reproduced 47 PASS
/ 3 FAIL with the same outcome and closure digests, and raised no equivalence assertion.
Its binary digest differs from the timed A–B–B–A binary because replacing the recursive
oracle changed the diagnostic binary; the source head and cohort are unchanged. This
semantic control is excluded from the timing-tax comparison.

## Decision boundary

No production-cost conclusion follows from either the invalidated phase shares or the
three completed corrected arms. The corrected arms establish exact outcome, subject,
closure, and work-count stability; they do not establish an instrumentation tax because
the second baseline never returned. Complete typed-module plus binding-identity
equivalence is a separate semantic control and cannot repair the missing timing arm.

Prior testing has already ruled out precomputing direct-import ambiguity as the next
mechanism. The observed classification makes same-identity persistent-map insertion the
next **candidate to discriminate**, not an implementation selected by this receipt. Do
not re-attempt the rejected entry-view memo mechanism merely because the row is large.

Receipts:

- `receipts/import-string-rewire-decomposition/representative-50-r1.txt`
- `receipts/import-string-rewire-decomposition/representative-50-r2.txt`
- `receipts/import-string-rewire-decomposition/streaming-profile-ab-confounded.txt`
- `receipts/import-string-rewire-decomposition/run_streaming_profile_ab.sh`
