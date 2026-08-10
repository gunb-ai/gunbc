# Import-string rewire decomposition measurement

**Lane:** floor-prep-tax-program P1 follow-on
**Predecessor:** [entry-view assembly direction](entry-view-assembly-direction-receipt.md)
**Status:** measurement complete; binding application isolated; no optimization selected

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
with the modeled `.dag` result; the hand-maintained Rust caller only times those modeled
boundaries and accumulates the observation.

This is measurement scaffolding, not a Rust implementation of compiler semantics. Its
dissolve-on note is carried by `import_string_rewire_measurement_scaffold_note` in the
model authority.

## Exact representative subject

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

## Result

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

## Decision boundary

Index construction is not the next target: the two index rows together are below 3.5%
of import-string time in both reproductions. Module preparation is also below 8%.
Binding application is the only supported next target, but this receipt does **not**
choose a cache, a Rust fast path, or a map representation. That phase still combines
direct-import ambiguity membership work with persistent ancestry-map updates.

The next slice must distinguish repeated ambiguity membership probes from no-op versus
value-changing ancestry-map writes at the `.dag` model boundary. Only then can it choose
between changing the decision representation and eliminating duplicated construction.
Do not re-attempt the rejected entry-view memo mechanism merely because the row is large.

Receipts:

- `receipts/import-string-rewire-decomposition/representative-50-r1.txt`
- `receipts/import-string-rewire-decomposition/representative-50-r2.txt`
