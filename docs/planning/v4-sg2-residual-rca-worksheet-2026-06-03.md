# v4 SG-2 Residual RCA Worksheet — 2026-06-03

> **Status:** DRAFT RCA WORKSHEET — residual routing on top of approved SG-2 worksheet (#4124).
> **Authority:** PR #4140 (`E0107` = 1,654, `E0282` = 1,007).
> **Existing worksheet:** `v4-sg2-type-expression-projection-worksheet-2026-05-30.md` (#3962 substrate, #4124 closure).

---

## Residual Shape

`E0107` missing-generic carrier histogram:

| Carrier | Count | Readout |
| --- | ---:| --- |
| `Outcome` | 740 | aliases/cached statics/signatures erase payload `T` |
| `TestClaimRun` | 292 | emitted claim run caches erase `S, A` |
| `Witness` | 135 | witness carrier instantiations lose `C` |
| `TestClaimEvalSubject` | 88 | evaluator subject generic path not preserved |
| `Validation` | 77 | refinement/validation caches erase `B` |
| `FreeMonoid` | 71 | collection source carrier instantiation missing at cache/signature sites |
| `Verdict` | 48 | verdict surface generic payload erased |
| `ClassifiedDependencyView` | 40 | lens/dependency generic erased |
| `Optional` | 25 | optional generic erased |
| `Generator` | 14 | generator generic erased |

`E0282` is concentrated in constructor/cache/closure sites after those
instantiations are erased. Top files include `v4_compiler_translate` (183),
`v4_workflow_ci` (58), `v4_lens_coverage` (45), `v4_lens_testgen` (38), and
`v4_std_target_model` (36).

Higher-kinded shape (`Homomorphism<C, Source, Target>` with `C<Source>` fields)
maps to `E0109` — a true realization-model gap requiring type-constructor encoding,
not a name-keyed arity table.

---

## §10.0-Adapted Residual Worksheet

```text
SG class:
  SG-2 residual — generic instantiation preservation across aliases, caches, statics, and signatures.

Representative emitted failure:
  CACHED.with(|c: &Rc<Outcome>| c.clone())
  CACHED.with(|c: &Rc<TestClaimRun>| c.clone())
  // Rust requires Outcome<T> and TestClaimRun<S,A>.

Immediate local patch:
  Add name-keyed emitter branches:
    Outcome -> Outcome<T>
    TestClaimRun -> TestClaimRun<S,A>
    Witness -> Witness<C>

Why forbidden:
  Duplicates generic-arity facts already present in substrate instantiation edges;
  requires an emitter edit for every new generic carrier (INVARIANTS P2).

DFS path:
  std authority:
    - generic carrier declarations (Outcome<T>, Witness<C>, Validation<B>, FreeMonoid<T>)
    - Instantiation connective with positional edges
  target authority:
    - TargetTypeExpressionProjection in v4.std.target_model
    - Rust row in extdeps/languages/rust.dag
  consumer:
    - type emission for aliases, function signatures, cached statics, closure annotations, constructor result types

Deepest unsound boundary:
  Approved SG-2 carrier describes target type-expression projection, but residual sites
  still read raw carrier names from alias/cache/signature paths instead of carrying the
  instantiated TargetTypeExpression to those sites.

Systemic fix:
  Extend SG-2 implementation to require every emitted type site to receive a
  TargetTypeExpression value, including cached statics and generated closure annotations.
  Fallback to raw carrier name is allowed only as typed ProjectionAbsent diagnostic/shim
  with explicit dissolve-on: aliases, cached statics, function signatures, constructor
  result types, and closure annotations all consume TargetTypeExpression.

Non-goals:
  - Name-keyed arity table for Outcome/TestClaimRun/Witness.
  - Treating E0282 as a separate inference feature before E0107 residual is reduced.
  - Patching emitted Rust files.

Falsification probe:
  Add generic carrier ProbePair<T, U> and use it in:
    (a) type alias,
    (b) cached static,
    (c) function parameter,
    (d) function return,
    (e) closure annotation.
  Emitted Rust must preserve ProbePair<X, Y> in all five sites without an emitter branch for ProbePair.

Metric allowed only as secondary:
  E0107/E0282 movement.
```

---

## Dispatch Decision

Do not create a new SG-2 substrate carrier. The residual is a **consumer coverage
slice** for the already-approved `TargetTypeExpressionProjection`: aliases, caches,
signatures, and closure annotations must all consume the same projected type
expression. Target Realization owns consumer coverage; Runtime/TestClaim owns the
falsification receipt.
