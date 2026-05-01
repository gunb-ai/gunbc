# R2 PR-B.2 Runner-Extension Bundle — Docs Scoping for PR-B.2 / PR-B.3 / PR-B.4

**Status:** PROPOSAL — **single docs-only scoping bundle** for the PR-B
follow-on runner-extension implementation slices. This brief carves three
workstreams that extend `src/v3/compiler/src/test_runner.rs::run_claim`
(and its producer dispatch / value normalization helpers) to evaluate
`TestPredicate` variants the runner currently returns
`ClaimResult::NotYetImplemented` for, plus a per-target producer dispatch
path for `ForAllTargets`. **No Rust, no fixtures, no substrate enum
edits, no new `TestPredicate` variants** in this slice.

**Implementation split (per parent clarification):** the three workstreams
below ship as **separate implementation PRs** if size / review pressure
warrants — provisional naming `PR-B.2` (W1 L4 `DifferentialEquals`),
`PR-B.3` (W2 L7 `AlgebraicLaw`), `PR-B.4` (W3 L5 `ForAllTargets`). The
*docs scoping* (this brief) is the single bundle; the implementation may
or may not be one PR. All three remain fully Evaluator-owned.

**Parent designs:**
- [`r2-pr-b-body-evaluator-eager-baseline.md`](r2-pr-b-body-evaluator-eager-baseline.md)
  (PR-B.0 design lock).
- [`r2-pr-b-1-eager-evaluator-implementation-seed.md`](r2-pr-b-1-eager-evaluator-implementation-seed.md)
  (PR-B.1 implementation seed; #1292 merged).
- [`r2-evaluator-manager.md`](r2-evaluator-manager.md) PR-B row + Witness
  construction sub-lane.
**Discovery inputs (NOT authority):** R3 Verification findings #1299 and
#1307. This bundle reads them as evidence that the runner-side gaps below
matter, but the authority for what the bundle must do lives in the briefs and
substrate cited above; R3 findings cannot extend bundle scope.

## Bundle (three workstreams)

### W1 — `DifferentialEquals` lineage producers (Lane 1 / L4)

**Today:** `TestPredicate::DifferentialEquals { subject_ref, oracle_ref,
input_ref }` requires `input_ref` to inhabit `ProgramOutputBind`
(`test_runner.rs:2104`); evaluation of the subject and oracle on the
common input is gated on producers that materialize comparable outputs.
The runner currently `NotYetImplemented`s the path because no producer
binds emitted-Rust output or interpreter-evaluated `.dag` output to a
`ProgramOutputBind` carrier.

**Bundle scope — W1 (provisional impl PR-B.2):** add two **producer roles** the runner can dispatch
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

**Bundle scope — W2 (provisional impl PR-B.3):** extend the runner to evaluate the two existing
`AlgebraicLawKind` inhabitants the substrate already declares but the
runner does not yet handle:

- **`Commutativity`** — runner-side check: for the named lens, apply over
  pairs `(a, b)` from a witness sample table analogous to
  `ASSOCIATIVITY_WITNESS_TRIPLES`, assert `lens(a, b) == lens(b, a)`
  pointwise via the **shared value-domain comparator helper** also
  consumed by W3 (single authority — W2 must not fork its own copy; see
  Sequencing table for the dependency edge). Sample table
  identity / size lives in `lens_apply` alongside the associativity one;
  W2 does not add new `TestPredicate` variants and does not propose
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
Adding one is a **substrate enum change** that the bundle must NOT make
locally and must NOT silently work around (e.g. by overloading another
variant or by encoding distributivity through `LensOutputEquals` against a
hand-rolled distributivity oracle).

**Routing for `Distributivity`:** R3 findings asking for `Distributivity`
must open a substrate-fact-introduction proposal per `INVARIANTS.md` §P1
(3-step decision procedure: DAG-ancestor → coproduct-vs-coordinate →
primitive-vs-lens-extensible) before any worker writes runner code for
it. This bundle names the routing explicitly so consumers do not mistake the
omission for an oversight.

**Out of W2:** any change to `AlgebraicLawKind`; structural drift away
from sample tables to first-class substrate law witnesses (that is the
declared dissolution trigger on the `AlgebraicLaw` predicate's scaffold
comment, which is post-bundle work — once lens-algebra facts are
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

**Bundle scope — W3 (provisional impl PR-B.4):** add a runner-side **per-target producer dispatch**
that observes the **structural value domain** of each target's emitted
program output, not the raw exit code. The runner reads target identity
from the existing target spec layer, picks a per-target producer (akin
to W1's `rust_emit_output`, plus analogous `python_emit_output` and
`go_emit_output` selected by target), runs each, captures the program's
declared output, and normalizes to a comparable structural value before
per-pair (or pair-and-oracle) equality checks.

**Structural observation authority — W3 hard substrate prerequisite,
NOT silently in scope:** today's `ForAllTargets`
(`src/v3/std/verification.dag:160`) carries `{ command, args,
expect_exit_code }` only — exit-code is the entire observation surface,
and there is **no typed structural-output carrier** in `verification.dag`
that names "the program's structured output value over which a value-domain
comparison is taken". W3 implementation cannot proceed against an
exit-code-shaped observation; before any W3 worker writes runner code,
the **structural observation carrier must be added to the substrate via
the `INVARIANTS.md` §P1 substrate-fact-introduction procedure** (same
routing as `Distributivity`). Candidate shape — to be designed under P1,
not locked here:

```
type ProgramOutputObservation
  = ExitCodeOnly { expect_exit_code: Int }       // current ForAllTargets / ExecuteCommand
  | StructuredValue { channel: ObservationChannel, expected_value_kind: ValueKind }
```

…where `ObservationChannel` (stdout / stderr / declared file) and
`ValueKind` (Int / Bool / Record / …) become the typed surface the
runner dispatches on. The exact carrier shape is P1's call. **Until
that carrier lands, W3's structural-value-domain runner work is
docs-only design — no implementation worker may extract a structured
value through an ad-hoc stdout-pattern path silently parallel to
`ExecuteCommand` / `ForAllTargets`.**

Value-domain comparison is the **same shared comparator helper** W2's
`Commutativity` check consumes — single authority across the bundle, not
a per-workstream copy.

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
trigger; not in this bundle); a `ForAllTargets` variant change; any
addition to `TestPredicate`.

**Hard prerequisites:**
1. Target spec layer exposes per-target emit producers (Rust today;
   Python and Go to follow).
2. **Structural observation carrier landed via INVARIANTS §P1** — see
   "Structural observation authority" above. W3 implementation is
   blocked on this; routing it through P1 is non-negotiable.
3. PR-B.1 eager evaluator landed (so the `dag_eval_output` oracle from
   W1 is available as the comparison authority).

## Runner authority discipline

`src/v3/compiler/src/test_runner.rs` is **not** a parallel Rust
test-predicate authority. The thesis direction is `TestClaim` data +
generated target-language tests; bespoke runner arms are transitional
scaffolds, not durable evaluation surface. This bundle (W1/W2/W3) is
permitted only because each new arm has a **named evaluator / PB-Runtime
dissolution target**. New runner-arm or producer-path growth that lacks
a concrete dissolution target is **frozen**.

Per-workstream dissolution targets:

| Workstream | Transitional arm / path                              | Dissolution target                                                                                       |
|-----------|------------------------------------------------------|----------------------------------------------------------------------------------------------------------|
| W1        | `rust_emit_output` producer                          | PB-Runtime-generated target-language tests (`TestClaim` data → emitted Rust test) replace bespoke producer dispatch. |
| W1        | `dag_eval_output` producer                           | PR-B eager body evaluator (PR-B.1) + witness construction (PR-B witness half) become the canonical oracle; producer collapses into evaluator-call + witness emission. |
| W2        | `Commutativity` / `Identity` runner checks           | First-class lens-algebra law witnesses + reflected substrate facts (per `AlgebraicLaw` scaffold dissolution comment in `verification.dag`); T-LensProducer-Retirement consumes them. |
| W3        | `ForAllTargets` per-target producer dispatch         | `ExecuteCommand` / `ForAllTargets` substrate collapse to typed capability + scope (per `verification.dag` scaffold comments) + PB-Runtime-generated per-target tests; T-FixedPoint receipts replace exit-code-only verification. |
| W3        | Structural observation carrier (P1)                  | Once landed, becomes the substrate fact; runner reads it, not a Rust-side convention. |

**Discipline rules (binding on PR-B.2 / PR-B.3 / PR-B.4 implementation):**

- No new `TestPredicate` arm in `test_runner.rs` may land without a
  named dissolution target in the table above (or an amendment to this
  section in a follow-on PR).
- Runner arms MUST be marked transitional (e.g. comment + scaffold tag
  pointing at this section) at the implementation site.
- "Convention-only" runner observation (regex over stdout, ad-hoc
  parsing, environment-variable signaling, etc.) is **forbidden** — if
  a fact is observed, the substrate must name it. W3's
  `ProgramOutputObservation` P1 routing is the canonical example.
- This bundle must not become a second test-predicate language in
  Rust. Each W1/W2/W3 step is a runner-side scaffold over an existing
  declarative `TestPredicate` variant; no Rust-side predicate
  invention.

## Sequencing — sequential-now vs parallelizable-later

| Step                                                                      | Workstream | Sequence                      |
|---------------------------------------------------------------------------|-----------|-------------------------------|
| `dag_eval_output` producer wiring                                         | W1        | After PR-B.1 lands eager evaluator |
| `rust_emit_output` producer wiring                                        | W1        | Parallelizable with `dag_eval_output` (different code path) |
| Structural observation carrier (P1 substrate-fact-introduction)           | W3 (sub)  | **GATED on P1 procedure** — blocks all W3 implementation; docs-only until it lands |
| Scalar value-domain normalization (`Int`, `Bool`)                         | W3        | After observation carrier lands; sequential before W3 record + before W1 differential equality on `Value` |
| Shared value-domain comparator helper (`Int` / `Bool` scalar equality)    | W2 + W3   | Sequential — must land before `Commutativity` runner check and before W3 scalar value-domain normalization (single shared helper authority, not a per-workstream copy) |
| `Commutativity` runner check                                              | W2        | After shared comparator helper; otherwise parallelizable with W1 |
| `Identity` runner check                                                   | W2        | After lens identity-element edge declared on algebra inhabitance |
| `RecordValue` value-domain observation                                    | W3        | After scalar value-domain coverage  |
| `ForAllTargets` per-target dispatch beyond Rust                           | W3        | After Rust-only scalar coverage; Python and Go parallelizable per target |
| `Distributivity` runner support                                           | (out)     | **GATED on substrate enum addition via P1 procedure** — not in this bundle |
| Variant / List / String value-domain observation                          | (out)     | Parallelizable later; not in this bundle |

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
- A new substrate-side observation-channel carrier. **W3 explicitly
  routes this to P1** — it is a *named gated prerequisite*, not a
  silent omission. See "Structural observation authority" in W3.

If during PR-B.2 / PR-B.3 / PR-B.4 implementation any reviewer or worker concludes that
one of the above is required to ship a workstream, **STOP and escalate
to the Director / Substrate Manager** before drafting code or amending
this brief. Silent variant-creation is forbidden.

## Acceptance gates (this brief)

- ✅ Three workstreams scoped with today's runner state, bundle work, hard
  prerequisites, and explicit out-of-scope items.
- ✅ `Distributivity` routed to P1 substrate-fact-introduction with
  rationale.
- ✅ Sequencing table separates sequential dependencies from
  parallelizable steps.
- ✅ STOP+PING boundary names every shape change the bundle must not make
  silently.
- ✅ Docs-only PR.
