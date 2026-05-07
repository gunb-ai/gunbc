# Worker brief — Substrate `descent_execution_proof` carrier (γ)

**Sub-issue**: TBD (PM creates under #1939 post-this-brief landing; Evaluator's #1971 `Depends on:` retargets to that issue).
**Authority**: Director ratification of **option γ** at gunbc#828 #issuecomment-4395060514 (2026-05-07); supersedes the canvas at `docs/briefs/r3-substrate-descent-execution-proof-canvas.md` (canvas may be deleted after this brief lands).
**Closure predicate**: Evaluator E2 descent termination contract consumer (#1971); §10.3 row 966 row-text refresh to cite γ-disposition (2-variant residual, not 4-variant prose).

## Scope

Substrate-fact-introduction (P1 procedure): consumer-side typed substrate function for executor termination-contract verification. Composes with existing `DescentEvidence` lattice at `src/v3/std/termination.dag:14-17`; no parallel "absent/unknown" axis.

## Carrier shape (binding per Director γ ratification)

**Location**: `src/v3/std/termination.dag` (extend the existing file — `DescentEvidence` already lives there; sibling typing keeps the lattice + residual co-located). Verify-via-grep at HEAD that no near-neighbor file has prior claim.

```dag
type DescentResidual
  = EvidenceUnknown(DescentEvidence)   // wraps DescentUnknown for absent-evidence; wraps NonIncreasing for not-strict
  | EvidenceIncomplete                  // proof-construction state: per-path coverage incomplete
```

**Single substrate function**:

```dag
fn descent_execution_proof(
  dag:        &Dag,
  cluster:    ClusterId,
  port:       PortId,
) -> Result<DescentExecutionProof, DescentResidual>
```

**`DescentExecutionProof`** payload — verify shape at implementation time. Likely a per-path evidence map keyed by branch identifier with each entry carrying `DescentEvidence::Strict` (non-Strict per-path entries surface as `EvidenceUnknown(NonIncreasing)` residual via the executor's join semantics). If a richer witness shape is needed, surface as STOP-and-PING.

### Director-asked verification (canvas-shape time)

**STOP-and-PING the Mgr** if `EvidenceIncomplete` decomposes into payload-variants. Worker greps the existing executor-error surface for partial-coverage / timeout / depth-bound reasons:
- If `EvidenceIncomplete` is genuinely **unit-variant** (proof construction either completes for all paths or fails wholesale on one of the existing residual reasons): ratchet stops at 2-variant — proceed with γ as ratified.
- If `EvidenceIncomplete` carries a specific reason payload (e.g., `EvidenceIncomplete { reason: TimeoutReason | DepthBoundExceeded | EvaluatorErrorDuringProofConstruction }`): surface to Mgr; canvas may need a 4-decompose-to-3 update with payload-variant.

Per Director: "this is canvas-shape verification, not a re-ratification ask. If the worker hits substrate evidence that requires payload structure, surface as STOP-and-PING; otherwise proceed with γ as 2-variant."

## Acceptance gates (same-slice, all must pass)

1. **`DescentResidual` carrier landed** in `src/v3/std/termination.dag` (or chosen location post-grep) per ratified γ shape (modulo STOP-resolution on `EvidenceIncomplete` payload).
2. **`DescentExecutionProof` carrier landed** with witness payload shape (per-path evidence map or richer structure surfaced via STOP-and-PING).
3. **`descent_execution_proof()` substrate function landed** with the typed signature from §10.3 row 966 (verbatim per Director confirm).
4. **Evaluator E2 (#1971) consumes the carrier** in same-slice — proves multi-consumer composability is unnecessary for this carrier (Evaluator IS the consumer per closure predicate; carrier-with-single-consumer-as-interface anti-pattern doesn't apply because the carrier is consumer-cementing for executor termination contract). PR description names the Evaluator consumer call site.
5. **§10.3 row 966 row-text update**: ROADMAP cites γ-disposition (2-variant residual) replacing the prose `Missing | Unknown | Incomplete | NonStrict` 4-variant naming. Per Director: "ROADMAP §10.3 row 966 row text should update on Gate A landing to cite this ratification."
6. Bootstrap regen: `cargo test -p v3-compiler bootstrap_regen_fresh -- --ignored` clean.
7. Full suite: `cargo test --workspace --exclude v2-compiler-tests` green; `cargo clippy --all-targets -- -D warnings` clean.

## STOP / PING criteria

- **STOP** if `EvidenceIncomplete` decomposes into payload-variants per the Director-asked verification above — surface to Mgr (warm-wolf-698 / inbox #2068) before adding a payload; canvas update needed.
- **STOP** if `DescentExecutionProof` witness shape requires substantial new substrate (e.g., a fresh per-path-witness type with non-trivial bootstrap-regen impact) — surface scope-creep.
- **STOP** if §10.3 row 966's verbatim signature requires refinement at implementation time (`&Dag` may need module witness; `ClusterId` / `PortId` are existing Substrate types — verify via grep at brief-execution time; surface if drift).
- **PING** Evaluator Mgr (#2065 / `crisp-bat-13`) at PR-open time so they can retarget #1971 `Depends on:` to the carrier work-item AND begin consumer wiring against the same PR.

## Cross-Mgr coordination

- **Evaluator Mgr (#2065 / crisp-bat-13)**: same-slice consumer (E2 `descent_execution_proof` consumer at #1971). PING at PR-open; coordinate consumer wiring in same PR per acceptance gate #4.
- **Verification Mgr (#2075 / wise-bear-525)**: standing-program ratchet authoring is Verification's concern; no specific same-slice handoff expected unless ledger row needs to advance.

## Worker pin (Mgr disposition)

**quick-koi-190** — pre-authorized per §10.3 row 966 ("quick-koi-190 implementation already authorized through quick-crab"); also conceptually adjacent to T-E-P P1 work (DescentEvidence producer broadening) which quick-koi-190 has been on. Final pin at dispatch.

## Auto-spawn caveat

Per Director note 2026-05-07 + ratification at #4395060514: worker auto-spawn from Mgr-context is bug-affected (ctrl#217); HOLD dispatch on this brief until auto-spawn fix lands per L-sized-not-low-risk threshold, OR escalate via surgical-recreate path if Pattern A cascade or another critical path is blocked. Director ratifies surgical-recreate case-by-case.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-07 per Director γ-ratification at gunbc#828 #issuecomment-4395060514.
