# R2 PR-B.2 Runner-Extension Bundle — Scope Brief

**Status:** PROPOSAL — docs-only scoping for the PR-B follow-on
runner-extension bundle. This brief carves three workstreams that extend
`src/v3/compiler/src/test_runner.rs::run_claim` (and its producer dispatch /
value normalization helpers) to evaluate `TestPredicate` variants the
runner currently returns `ClaimResult::NotYetImplemented` for, plus a
per-target producer dispatch path for `ForAllTargets`. **No Rust, no
fixtures, no substrate enum edits, no new `TestPredicate` variants** in
this slice.

**Parent designs:**
- [`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md)
  (PR-B.0 design lock).
- [`r2-pr-b-1-eager-evaluator-implementation-seed.md`](r2-pr-b-1-eager-evaluator-implementation-seed.md)
  (PR-B.1 implementation seed; #1292 merged).
- [`r2-evaluator-manager.md`](r2-evaluator-manager.md) PR-B row + Witness
  construction sub-lane.
**Discovery inputs (NOT authority):** R3 Verification findings #1299 and
#1307. PR-B.2 reads them as evidence that the runner-side gaps below
matter, but the authority for what PR-B.2 must do lives in the briefs and
substrate cited above; R3 findings cannot extend PR-B.2 scope.

## Bundle (three workstreams)

### W1 — `DifferentialEquals` lineage producers (Lane 1 / L4)

**Today:** `TestPredicate::DifferentialEquals { subject_ref, oracle_ref,
input_ref }` requires `input_ref` to inhabit `ProgramOutputBind`
(`test_runner.rs:2104`); evaluation of the subject and oracle on the
common input is gated on producers that materialize comparable outputs.
The runner currently `NotYetImplemented`s the path because no producer
binds emitted-Rust output or interpreter-evaluated `.dag` output to a
`ProgramOutputBind` carrier.

**PR-B.2 scope (W1):** add two **producer roles** the runner can dispatch
on at `run_claim` time, both reachable through the existing
`ProgramOutputBind` binding surface (no new `TestPredicate` variant, no
new `ProgramInputRole` substrate variant unless explicitly routed):

- **`rust_emit_output` producer** — read the claim's `source` /
  `file_name` (`TestClaim` single authority), emit Rust through the
  existing emitter, run the emitted binary in the runner's existing
  `ExecuteCommand` capability path, capture `stdout` (or another
  declared observation channel — see W3), normalize to the comparable
  value domain, and bind it to `ProgramOutputBind.output_ref`.
- **`dag_eval_output` producer** — evaluate the claim's body through the
  PR-B.1 eager body evaluator (`Value` over `EvalFrame` /
  `EvalStateStack`), normalize the resulting `Value` to the comparable
  domain, bind to `ProgramOutputBind.output_ref`.

`DifferentialEquals` becomes evaluable when **both** producers are
selectable and at least one comparable-domain normalization rule exists
(e.g. `Value::LiteralValue(LiteralBits::Int)` ↔ stdout `^-?\d+$`). See
W3 for the value-domain observation rule.

**Out of W1:** new `TestPredicate` variants; new substrate `ProgramInputRole`
variants; cross-target diffing (that is W3); reflection-based oracle
selection.

**Hard prerequisites:** PR-B.1 eager evaluator landed (so `dag_eval_output`
has a producer to call); PR-A.3 carrier ownership settled (so
`dag_eval_output` reads `EvalStrategy = ApplicativeOrder/LeftFirst` per
the PR-B.1 seed).

### W2 — `AlgebraicLaw` runner extension (Lane 1 / L7)

**Today:** `eval_algebraic_law_for_claim_program` in `test_runner.rs`
evaluates `AlgebraicLaw { law, lens_ref }` for `Associativity` only via
the `lens_apply::ASSOCIATIVITY_WITNESS_TRIPLES` sample table; other
`AlgebraicLawKind` variants are `NotYetImplemented` (`test_runner.rs:2189`).

**PR-B.2 scope (W2):** extend the runner to evaluate the two existing
`AlgebraicLawKind` inhabitants the substrate already declares but the
runner does not yet handle:

- **`Commutativity`** — runner-side check: for the named lens, apply over
  pairs `(a, b)` from a witness sample table analogous to
  `ASSOCIATIVITY_WITNESS_TRIPLES`, assert `lens(a, b) == lens(b, a)`
  pointwise via the value-domain comparison from W3. Sample table
  identity / size lives in `lens_apply` alongside the associativity one;
  PR-B.2 does not add new `TestPredicate` variants and does not propose
  a generator strategy.
- **`Identity`** — runner-side check: pick the lens's identity element
  (declared on the lens's algebra inhabitance, not local guesswork) and
  assert `lens(id, x) == x` and `lens(x, id) == x` over a single-arg
  sample table. **Hard precondition:** the substrate must already expose
  an identity-element edge on the lens's algebra inhabitance; if it does
  not, W2 reduces to `Commutativity` only and `Identity` deferred.

**`Distributivity` — explicit routing, NOT W2 scope:**
`AlgebraicLawKind = Associativity | Commutativity | Identity` per
`src/v3/std/verification.dag:103`. There is no `Distributivity` variant.
Adding one is a **substrate enum change** that PR-B.2 must NOT make
locally and must NOT silently work around (e.g. by overloading another
variant or by encoding distributivity through `LensOutputEquals` against a
hand-rolled distributivity oracle).

**Routing for `Distributivity`:** R3 findings asking for `Distributivity`
must open a substrate-fact-introduction proposal per `INVARIANTS.md` §P1
(3-step decision procedure: DAG-ancestor → coproduct-vs-coordinate →
primitive-vs-lens-extensible) before any worker writes runner code for
it. PR-B.2 names this routing explicitly so consumers do not mistake the
omission for an oversight.

**Out of W2:** any change to `AlgebraicLawKind`; structural drift away
from sample tables to first-class substrate law witnesses (that is the
declared dissolution trigger on the `AlgebraicLaw` predicate's scaffold
comment, which is post-PR-B.2 work — once lens-algebra facts are
first-class declarations consumable from `std.verification`).

**Hard prerequisites:** lens identity-element edge declared on the
algebra inhabitance for `Identity`; otherwise no PR-A / PR-B carrier
work blocks W2 — the runner extension is local to
`test_runner.rs::eval_algebraic_law_for_claim_program` plus the
`lens_apply` sample tables.

### W3 — `ForAllTargets` per-target producer dispatch (Lane 2 / L5)

**Today:** `TestPredicate::ForAllTargets { command, args, expect_exit_code }`
runs the same shell triple per emission target. The substrate-side
scaffold comment on this variant
(`src/v3/std/verification.dag:152-165`) calls out the dissolution
direction: collapse with `ExecuteCommand`, lift target-specific emission
+ verification facts into the target spec / runner tables, and narrow
the predicate to declarative edges. The current per-target observation
is exit-code only.

**PR-B.2 scope (W3):** add a runner-side **per-target producer dispatch**
that observes the **structural value domain** of each target's emitted
program output, not the raw exit code. The runner reads target identity
from the existing target spec layer, picks a per-target producer (akin
to W1's `rust_emit_output`, plus analogous `python_emit_output` and
`go_emit_output` selected by target), runs each, captures the
declared observation channel, and normalizes to a comparable structural
value before per-pair (or pair-and-oracle) equality checks.

**Initial value-domain coverage (sequential ship order):**

1. `Value::LiteralValue(LiteralBits::Int)` — integer literals; stdout
   normalization rule `^-?\d+$` after trim.
2. `Value::LiteralValue(LiteralBits::Bool)` — boolean literals; stdout
   normalization rule `true|false` after trim.
3. `Value::RecordValue(List<NamedField>)` — record-shaped outputs over
   the two scalar inhabitants above.

`Value::RecordValue` ships sequentially after the two scalars, not in
parallel; record observation requires the scalar normalizers to exist
first.

**Deferred (parallelizable later, NOT W3 scope):**
- `Value::LiteralValue(LiteralBits::String)` — string outputs; needs
  encoding/canonicalization rules per target (line endings, trailing
  whitespace).
- `Value::VariantValue { tag, payload }` — variant outputs; needs a
  per-target tag-rendering rule.
- `Value::CardinalityValue(LoopBound)` — cardinality outputs.
- List-shaped outputs; needs a per-target list-rendering rule.

**Out of W3:** `ExecuteCommand` collapse (substrate-side dissolution
trigger; not PR-B.2 scope); a `ForAllTargets` variant change; any
addition to `TestPredicate`.

**Hard prerequisites:** target spec layer exposes per-target emit
producers and a declared observation channel (today's
`ExecuteCommand`-style command/args is the lower bound; richer
observation channels are W3 dissolution trigger material, not a W3
prerequisite).

## Sequencing — sequential-now vs parallelizable-later

| Step                                                                      | Workstream | Sequence                      |
|---------------------------------------------------------------------------|-----------|-------------------------------|
| `dag_eval_output` producer wiring                                         | W1        | After PR-B.1 lands eager evaluator |
| `rust_emit_output` producer wiring                                        | W1        | Parallelizable with `dag_eval_output` (different code path) |
| Scalar value-domain normalization (`Int`, `Bool`)                         | W3        | Sequential — must land before W3 record + before W1 differential equality on `Value` |
| `Commutativity` runner check                                              | W2        | Parallelizable with W1 + W3   |
| `Identity` runner check                                                   | W2        | After lens identity-element edge declared on algebra inhabitance |
| `RecordValue` value-domain observation                                    | W3        | After scalar value-domain coverage  |
| `ForAllTargets` per-target dispatch beyond Rust                           | W3        | After Rust-only scalar coverage; Python and Go parallelizable per target |
| `Distributivity` runner support                                           | (out)     | **GATED on substrate enum addition via P1 procedure** — not in PR-B.2 |
| Variant / List / String value-domain observation                          | (out)     | Parallelizable later; not in PR-B.2 |

## Constraints (re-stated for reviewer audit)

- ✅ Docs-only this slice. No Rust, no fixtures, no substrate enum
  edits, no new `TestPredicate` variants.
- ✅ Implementation target is runner-side around
  `src/v3/compiler/src/test_runner.rs::run_claim` plus producer dispatch
  / value normalization helpers.
- ✅ Implementation gated post-PR-B.1 and after PR-A.3 carrier ownership
  settles.
- ✅ Sequential-vs-parallel split is explicit in the table above.
- ✅ R3 findings #1299 and #1307 are discovery inputs only; authority
  remains in `verification.dag` + parent design briefs.
- ✅ `AlgebraicLawKind::Distributivity` substrate gap is **routed**, not
  silently implemented.

## STOP+PING boundary

This brief does **not** propose:

- A new `TestPredicate` variant.
- A new `AlgebraicLawKind` inhabitant (`Distributivity` is routed to
  P1 substrate-fact-introduction).
- A new `ProgramInputRole` substrate variant.
- A new `Value` variant.
- A new substrate-side observation-channel carrier.

If during PR-B.2 implementation any reviewer or worker concludes that
one of the above is required to ship a workstream, **STOP and escalate
to the Director / Substrate Manager** before drafting code or amending
this brief. Silent variant-creation is forbidden.

## Acceptance gates (this brief)

- ✅ Three workstreams scoped with today's runner state, PR-B.2 work, hard
  prerequisites, and explicit out-of-scope items.
- ✅ `Distributivity` routed to P1 substrate-fact-introduction with
  rationale.
- ✅ Sequencing table separates sequential dependencies from
  parallelizable steps.
- ✅ STOP+PING boundary names every shape change PR-B.2 must not make
  silently.
- ✅ Docs-only PR.
