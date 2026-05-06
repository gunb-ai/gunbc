# R3 Verification Witness-Shape Pattern Survey

**Status:** PROPOSAL - research-only synthesis. This survey consolidates the
runtime witness-shape patterns that emerged across the Verification gate
cluster. It does not authorize substrate edits, new `TestPredicate` variants,
fixture rewrites, or runner changes.

**Purpose:** future R3 Verification workers should be able to pick the correct
witness pattern from this file instead of re-reading the full PR chain.

## Canonical Inputs

- PR #1403: `r3-v-bridge-retirement-ledger-zero-witness-shape.md`.
- PR #1404 + PR #1416: first- and second-batch Free-Consequences witness
  shapes.
- PR #1408: Lane 1/2 readiness audits.
- PR #1412: Lane 2 corpus extension / Phase A-B observation domain.
- PR #1353 + PR #1354: TC1/TC2/TC3 unified consumers.
- PR #1352 + PR #1324: RustDagIsomorphism producer-first framing.

## Pattern Taxonomy

| Pattern | Gate family | Witness source | Runtime shape |
|---|---|---|---|
| **Substrate-fold** | `bridge_retirement_ledger_zero` | already-materialized substrate declaration | runner folds static carrier; no Evaluator runtime |
| **Lens<C>-fold** | Free-Consequences | lens reads over `(Dag, Behavior)` | Evaluator constructs lens reports, then runner compares/projections |
| **Consumer instance** | TC1/TC2/TC3, RustDagIsomorphism | Substrate-owned producer emits report | `BinaryDimensionReportEquals` shape-valid scaffold compares paired reports once producers exist |
| **Corpus-driven runtime** | L4/L5 | emitted/evaluated program observations | runner produces normalized values, then algebraic equality |
| **Harness skeleton** | Lane 1/Lane 2 readiness | fixture path, corpus identity, failure taxonomy | waits for producers; avoids new predicate variants |

The first decision for any future worker is category selection. Most blocking
review failures in this cluster came from using the wrong category: treating
parallelism as `DimensionReport<C>`, treating corpus runtime checks as lenses,
or treating substrate-fold gates as if they needed an Evaluator witness.

Predicate readiness matters too. `LensOutputEquals` and
`BinaryDimensionReportEquals` are live `TestPredicate` variants in
`src/v3/std/verification.dag`, but they are not equally complete runtime
surfaces: `LensOutputEquals` is runner-wired for the current scalar lens
scaffolds, while `BinaryDimensionReportEquals` is presently a shape-valid
consumer envelope that returns `NotYetImplemented` until concrete
`DimensionReport<C>` producers and equality evaluation land. This survey indexes
those existing consumer surfaces; it does not authorize predicate or runner
changes.

Witness-label readiness is separate from predicate readiness. Labels in the
formulas below, such as `bind_independence.green`,
`effect_commutativity.green`, `purity.green`, `strategy-order`, and
`evaluation-step`, are pending conceptual witness roles unless they appear in
the verified-anchor table below. Missing role carriers stay in the
Substrate-Introduction Catalog and must route through `INVARIANTS.md` P1 /
Substrate Manager before any future worker treats them as authority.

Verified live anchors at this PR head:

| Anchor | Current authority |
|---|---|
| `LensOutputEquals` | `src/v3/std/verification.dag:166-174`; runner dispatch at `src/v3/compiler/src/test_runner.rs:1579` |
| `DifferentialEquals` | `src/v3/std/verification.dag:175-181`; runner dispatch at `src/v3/compiler/src/test_runner.rs:1580` |
| `BinaryDimensionReportEquals` | `src/v3/std/verification.dag:183-194`; shape-valid runner dispatch at `src/v3/compiler/src/test_runner.rs:1581-1583` |
| `AlgebraicLaw` | `src/v3/std/verification.dag:195-203`; runner dispatch at `src/v3/compiler/src/test_runner.rs:1584` |
| `ForAllTargets` | `src/v3/std/verification.dag:153-161`; declared predicate, runner falls through to the generic unwired-predicate path today |
| `BridgeLedgerZero` | `src/v3/std/verification.dag:276`; runner dispatch at `src/v3/compiler/src/test_runner.rs:1592`; carrier at `src/v3/std/bridge_ledger.dag` |
| `DimensionReport<C>` | `src/v3/std/dimensions.dag:51` |
| `LanguageSpec` | `src/v3/std/emit_model.dag:303` |

## Substrate-Fold Gates

```
bridge_ledger_zero =
  canonical_ledger_decl
  |> rows
  |> all(row.status == BridgeStatus::Retired)
```

`BridgeLedgerZero` is the canonical substrate-fold pattern. Inputs are
`src/v3/std/bridge_ledger.dag` row statuses and the `BridgeLedgerZero` predicate
in `src/v3/std/verification.dag`. No Evaluator producer is needed. When the
final row flips to `Retired`, re-arm the integration expectation from `Fail` to
`Pass` in the same PR. Do not add dummy `source`, local runners, or alternate
predicates; the substrate declaration is the witness.

## Lens<C>-Fold Gates

Free-Consequences gates are the canonical `Lens<C>`-fold pattern. The formula
variables in this section are pending witness roles, not declarations of live
carrier names.

Auto-parallelism:

```
parallel_emit_eligible =
  bind_independence.green
  && effect_commutativity.green
  && cost.green_and_parallelization_meaningful
```

Auto-loop-parallelism:

```
parallel_loop_eligible =
  iteration_independence.green
  && effect_commutativity.green
  && cost.green_and_parallelization_meaningful
```

Auto-memoization:

```
memoize_eligible =
  purity.green
  && cost.green_for_repeated_work
```

Cross-target optimization:

```
target_cost_report =
  dag_algebra_cost + language_spec.realization_cost
```

Parallelism gates stay on the runner-wired `LensOutputEquals` scalar scaffold;
DB-3/DB-20 keep workflow parallelism out of `DimensionReport<C>`.
Memoization and cross-target cost gates are authored against the live
`BinaryDimensionReportEquals` shape-valid scaffold because they are cost-shaped
`DimensionReport<C>` consumers, but full report equality remains NYI until the
producer/evaluator side lands. Placeholders remain fail-closed until
`src/v3/lenses/parallelism.dag`, `Lens<Purity>`, `Lens<Cost>`,
`Lens<Effect-Commutativity>`, and the relevant independence producers exist and
R2-Evaluator can execute the reads.

## Consumer-Instance Gates

TC1/TC2/TC3 and RustDagIsomorphism are consumer instances. Modifier names here
identify producer obligations, not live report-carrier declarations unless a
producer has already landed. They should share producer-first discipline:

- Substrate owns the producer that creates the report.
- Verification owns the consumer fixture and coverage requirement.
- `BinaryDimensionReportEquals` is the current comparison envelope; today the
  runner validates paired `DimensionReport<C>` shape and defers real structural
  equality until producers exist.

TC modifiers: TC1 eta-equivalence compares `DimensionReport<C>` under `f` and
eta-expanded `lambda x.apply(f, [x])`; TC2 strategy-order compares reports
across at least two executable strategies; TC3 evaluation-step compares
evaluation-step witness reports and still gates full strict-fire on
T-FixedPoint. RustDagIsomorphism follows the same pattern: Substrate authors
`Lens<DagShapeReport>`, Rust extraction and `.dag` reflection each produce a
report, and `BinaryDimensionReportEquals` compares them. Dissolution occurs
when producers emit concrete reports and runner equality stops at real
structural comparison rather than shape-valid `NotYetImplemented`.

## Corpus-Driven Runtime Gates

Lane 1 and Lane 2 are runtime/corpus patterns, not `Lens<C>` instances.

Lane 1 L4 and L7 are still pending R3 implementation work per
[`docs/v3-modeling-analysis.md`](../v3-modeling-analysis.md) §"Tier 3 — Verification from structure"
(L4–L7 status rows) and
[`docs/thesis/r2-r3-thesis-mapping.md`](../thesis/r2-r3-thesis-mapping.md) §"Disposition table — Tier 3 (Verification from structure)"
(L4 / L7 disposition rows). The live runner surfaces listed
above are scaffolds, not closure claims. L4 is expected to use
`DifferentialEquals` over `rust_emit_output` and `dag_eval_output`;
`dag_eval_output` still requires real PR-B body evaluation, and the failure
taxonomy is emit failure, target run failure, evaluator failure, and value
mismatch. L7 is expected to use `AlgebraicLaw`; `Associativity` and
`Commutativity` have runner surface, `Identity` waits on an identity-element
edge, and `Distributivity` remains a P1 candidate. Lane 2 L5 is expected to use
`ForAllTargets` plus a structural observation carrier: exit code is
insufficient, and `ForAllTargets` is declared but not runner-wired today. Per
target: emit -> compile/run -> parse observation -> compare algebraic values.
PR #1412 sets Phase A to scalar `Int` / `Bool`, then Phase B to collections and
richer records. No byte/string stdout equality as authority.

## Substrate-Introduction Catalog

Route these through `INVARIANTS.md` P1 / Substrate Manager if absent:
bind-independence producer; iteration-independence producer;
DB-20-compatible effect-commutativity projection; purity producer; cost
producer for repeated work, loop body cost, and emitted target artifacts;
`LanguageSpec` realization-cost rows; `Lens<DagShapeReport>` producer;
strategy-order and evaluation-step producers; structural observation carrier for
L5 `ForAllTargets`; identity-element edge and distributivity law surface for
L7.

Verification may author coverage requirements and consuming fixtures. It must
not invent carriers, widen deferred predicates, or create a parallel predicate
when a producer is missing.

## Fleet Discipline Defaults

- **DB-3/DB-20:** workflow parallelism is ordinary lens data, never a
  `DimensionReport<C>` shortcut.
- **P1 routing:** missing facts are Substrate work, not local fixture patches.
- **OnceLock:** integration tests with expensive compile/setup share
  process-local setup via `OnceLock`.
- **Message-pinning:** assert typed `ClaimResult` variants, not diagnostic
  prose substrings, unless the diagnostic text is itself the contract.
- **Fail-closed placeholders:** placeholder lenses mismatch expected values
  until the real producer lands.
- **No dummy source:** structural report/ledger subjects should use typed
  predicate payloads as authority; non-empty source text is not a witness.

## Dissolution Map

| Pattern | Red state | Green transition |
|---|---|---|
| Substrate-fold | one or more non-retired rows | last substrate row flips to `Retired`; existing fold returns `Pass` |
| Lens<C>-fold | placeholder or NYI producer | real lens producer + Evaluator report construction lands |
| Consumer instance | report aliases / NYI equality | producer emits concrete report; `BinaryDimensionReportEquals` evaluates |
| Corpus runtime | producer or observation missing | runner produces normalized structural values for the corpus |
| Harness skeleton | readiness doc only | fixture path, producer refs, and failure taxonomy become executable |

## Non-Claims

- This survey does not close any gate.
- This survey does not authorize substrate edits or new predicate variants.
- This survey does not supersede the lane briefs; it indexes their witness
  shapes so implementation workers choose the right pattern quickly.
