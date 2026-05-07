# Canvas — Substrate `descent_execution_proof` carrier (P1 substrate-fact-introduction)

**Parent**: gunbc#1939 (Substrate Mgr lane); will be parented under a PM-authored work-item under #1939 post-ratification.
**Authority**: `docs/r3-program-plan.md` §10.3 row Q-EVAL-Descent-Termination-Contract (line 966); Director ratification of split disposition at gunbc#828 #issuecomment-4394696074 (descent_execution_proof STANDS ALONE; folds-into-T-E-P explicitly rejected).
**Closure predicate**: Evaluator E2 descent termination contract consumer (#1971); fail-closed residual enumeration is the load-bearing structural fact.
**Status**: **canvas — Director-tier ratification needed on residual enumeration shape before worker brief authoring**.

## Scope

Substrate carrier for executor-side termination-contract verification:

```
descent_execution_proof(&Dag, ClusterId, PortId)
  -> Result<DescentExecutionProof, DescentResidual>
```

Per §10.3 row 966 the residual enumeration was provisionally named `Missing | Unknown | Incomplete | NonStrict`. The shape question is: **what is the minimum-irreducible variant set for `DescentResidual`?**

## Adjacent substrate (grep-verified at HEAD)

`src/v3/std/termination.dag:14-17` already defines:

```dag
type DescentEvidence
  = Strict
  | NonIncreasing
  | DescentUnknown
```

with `BoundedLattice<DescentEvidence>` ordering `DescentUnknown < NonIncreasing < Strict` (top = `Strict`, bottom = `DescentUnknown` per fail-closed discipline). `merge_evidence` / `join_evidence` are conservative-meet / optimistic-join.

So the residual question must reconcile with the existing 3-variant evidence type. **`Unknown` in the residual enumeration ≡ `DescentEvidence::DescentUnknown`**; reusing the existing carrier is the structural-cheap default unless a distinction emerges.

## Carrier-shape options for the residual enumeration

### Option α — 4-variant coproduct (as named in §10.3 row 966)

```dag
type DescentResidual
  = Missing       // no DescentEvidence carrier present at the call site
  | Unknown       // DescentEvidence::DescentUnknown surfaced
  | Incomplete    // per-path evidence exists but some paths uncovered
  | NonStrict     // evidence is NonIncreasing (not Strict)
```

**Pro**: matches §10.3 row 966 verbatim; explicit named cases; readable at the consumer.
**Con**: `Missing` and `Unknown` may collapse — both are "evidence-absent" with different cardinalities (no carrier vs `DescentUnknown` carrier). Per `feedback_coproduct_dissolution`, this is the kind of variant pair that should ratchet downward unless there's a load-bearing distinction. Same for `Incomplete` vs `NonStrict` — both are "evidence-present-but-insufficient", differing on which axis (path-coverage vs strictness).

### Option β — Dimensional product report

```dag
type DescentResidualReport {
  presence:     EvidencePresence    // CarrierPresent | CarrierAbsent
  completeness: PathCoverage         // AllPathsCovered | SomePathsUncovered
  strictness:   StrictnessVerdict   // Strict | NonIncreasing | NotApplicable
}
```

Decomposes the 4-coproduct into 3 orthogonal axes; the executor reports per-axis verdict; the "fail" condition is any non-top axis.

**Pro**: maximum structural decomposition per `feedback_coproduct_dissolution`; eliminates illegal-state question of whether `Incomplete` and `NonStrict` are mutually exclusive (the answer: no — a proof can be both); per-axis independent reasoning.
**Con**: heavier shape; requires defining 3 sub-types (`EvidencePresence`, `PathCoverage`, `StrictnessVerdict`) instead of 1; `StrictnessVerdict::NotApplicable` admits illegal state when carrier is absent (presence=CarrierAbsent + strictness=Strict shouldn't be representable). If the 3 axes don't actually compose orthogonally (i.e., some combinations are nonsensical), product shape is wrong.

### Option γ — Reuse `DescentEvidence` for "absent/unknown"; 2 additional residuals

```dag
type DescentResidual
  = EvidenceUnknown(DescentEvidence)   // wraps DescentUnknown (or NonIncreasing reported as not-strict)
  | EvidenceIncomplete                 // multi-path partial coverage
```

3-variant collapse of α: `Missing` and `Unknown` fold into a single `EvidenceUnknown(DescentEvidence)` payload (the `Missing` case is `DescentUnknown` synthesized by the executor when no carrier is present); `NonStrict` folds into `EvidenceUnknown(NonIncreasing)` since the existing lattice already distinguishes `NonIncreasing` from `Strict`; `Incomplete` remains separate because path-coverage IS structurally distinct from per-evidence strictness.

**Pro**: reuses `DescentEvidence` carrier per `feedback_audit_adjacent_authority_first`; ratchets variant count from 4 → 2 via dimensional folding; `EvidenceUnknown(Strict)` is unrepresentable by construction (Strict isn't a residual).
**Con**: payload-typed variant is heavier than bare-name variant; consumers must pattern-match on the payload to recover the original 4-case story; "Strict" appearing under `EvidenceUnknown` requires an inhabited-only-by-non-Strict refinement type or a runtime check (mild illegal-state risk).

## Mgr-tier recommendation

Provisional **γ**: ratchets the variant count via dimensional folding (per Director's coproduct-dissolution audit suggestion) while reusing the existing `DescentEvidence` carrier (services.dag-style "no parallel wrapper" precedent). The `Strict`-shouldn't-appear-here illegal-state risk is mild and addressable via either a refinement type (`DescentEvidence \ Strict`) or a fixture-load fail-closed check.

If γ's payload-pattern-match cost is unacceptable for consumer ergonomics, **α** with explicit Mgr-acknowledgment that `Missing` ≡ `Unknown` (rather than fold via constructor injection) is the second-best — names the cases verbatim from §10.3 at the cost of one redundant variant.

**β rejected** unless the 3 axes provably compose orthogonally — risk of admitting illegal states (carrier-absent + strictness=Strict) is real and would require additional refinement-shape work.

## Director ratification ask

1. **Pick α / β / γ** (or surface a fourth option). Provisional Mgr recommendation: **γ**.
2. Confirm the existing `DescentEvidence` 3-variant lattice at `termination.dag:14-17` is the authoritative starting point — i.e., the residual carrier composes with it rather than re-inventing it.
3. Confirm the typed signature `descent_execution_proof(&Dag, ClusterId, PortId) -> Result<DescentExecutionProof, DescentResidual>` from §10.3 row 966 is verbatim-binding, OR ratify a refinement (e.g., `&Dag` may need to be `&Dag` + module witness; `ClusterId` + `PortId` are existing Substrate types — verify via grep at worker brief time).

## On ratification — worker brief scope

Will author execution brief covering:
- `DescentResidual` carrier in `src/v3/std/termination.dag` (or sibling file if cleaner) per chosen option
- `DescentExecutionProof` carrier shape (witness payload — likely a mini-DAG or per-path evidence map)
- `descent_execution_proof()` substrate function signature in DSL with fail-closed body shape
- Acceptance: §1.8 gates per row 966 closure predicate; bootstrap regen + clippy + workspace tests green
- Cross-Mgr handoff: Evaluator (#1971 / crisp-bat-13) consumes carrier; Q-EVAL-Descent-Termination-Contract row 966 advances to `CONSUMER_LANDED`

## Worker pin (Mgr disposition)

**quick-koi-190** — already authorized through quick-crab per §10.3 row 966 ("quick-koi-190 implementation already authorized through quick-crab"); also currently on T-E-P P1 work which is conceptually adjacent (DescentEvidence producer broadening). Final pin at dispatch.

## Auto-spawn caveat

Per Director note 2026-05-07: worker auto-spawn from Mgr-context is bug-affected; HOLD dispatch on this canvas's worker brief until auto-spawn fix lands per L-sized-not-low-risk threshold.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-07 post-#2079 merge per Director serial-cadence direction at gunbc#828 #issuecomment-4394696074. Coproduct-dissolution audit per Director's micro-suggestion in same message.
