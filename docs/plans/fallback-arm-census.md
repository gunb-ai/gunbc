# Fallback-arm census — catch-all-that-answers lens (W2)

**Status:** LANDED (census + enrollment) · session `bright-newt-31` · PR #7260 · 2026-07-25  
**Charter row:** enforcement-intent / DESIGN §5 absorbing-fallback program (W2).  
**Exemplar defect:** wrap-decision bypass (`docs/probes/gate1_repr_mismatch_e0308_diagnosis_2026-07-24.md` — `_ => outcome_accepted(projected)` on open `Instantiation` wire).  
**End state (not this PR):** W3 promotes the lens to a compile refusal once zero-false-positive corpus-wide.

## Classifier (exactly one class per `_ =>` / wildcard match arm)

| Class | Verdict | Meaning |
|---|---|---|
| `Refuses` | ✓ | Body constructs a refusal-shaped value (error / Refused / typed absence that propagates). |
| `CompletesClosedTotal` | ✓ counted | Scrutinee is a closed coproduct; `_` covers enumerable remainder. Flagged: forfeits exhaustiveness when a variant is added. |
| `DeclaredInterim` | ✓ | Arm sits beside a declared dissolution marker (`dissolve_on` / `FrontierRow`). |
| `AnswersOnOpen` | ✗ | Body yields success-shaped value; unmatched space is open. **The defect.** |
| `ClassifierUnknown{cause}` | counted | Undecidable — never silently binned ✓. |

The classifier itself must not contain an answering `_`: every match over the class coproduct is total.

## Mechanism

- Pure reader over match arms + typed subject (`DeclarationRef` / path-site until body fields ground).
- Corpus fact producer: host seam over `decl_facts` + `MatchPattern::Wildcard` (same #5364 residue as `complexity_linearity_wildcard_facts` — v2 `.dag` has no `MatchPattern` introspection yet). Substrate fixtures classify without the host.
- Completeness: per file, five class counts sum to the **structural** wildcard-arm count (authority). Text `_ =>` grep (~2,586) is the charter's measured over-approx; gap is reported, never rounded away.

## Deliverables (this PR)

1. Lens + planted fixtures (per-PR fast lane) — `src/v2/lens/fallback_arm_census.dag` + `fallback_arm_census_planted_test.dag`.
2. Corpus census register (✗ + Unknown rows, routed by owning lane) — host builtins `fallback_arm_census_*`; long witness on `falsifier_substrate_long_lane`.
3. Seed compiled-fallback-table inventory (24 `.rs` files / ~198 mention sites) — disposition list only.
4. Enrollment: fixtures per-PR; corpus receipt on `falsifier_substrate_long_lane`; lens registered (`FallbackArmCensus`).

## Out of scope

408 seed `unwrap_or` sites; ExpectedOutcome (#7216 / W0); fixing owned arms (route wrap-decision, don't touch); new corpus-wide authority types outside the lens.
