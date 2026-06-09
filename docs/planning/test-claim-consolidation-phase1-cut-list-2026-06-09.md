# Test-Claim Consolidation #4553 — Phase 1 cut list

**PR:** #4576  
**Design authority:** `docs/planning/test-claim-consolidation-design-2026-06-08.md` §Phase 1  
**Gate:** Mgr-C substrate-only; no runner / CI transport / roster deletion

## Before → after

| artifact | before | after |
| -------- | ------ | ----- |
| `src/v4/std/verification.dag` | `TestClaim` coproduct only | + `BoolWitness` (`function: Symbol`), `TestClaimTransportMode`, `UnifiedTestClaimModality`, `UnifiedTestClaim` (both arms, dissolution receipts), `unified_test_claim_bool_witness` fail-closed `Outcome` projection |
| `src/v4/test/claim/workflow/v4_roster_pilot.dag` | `V4RosterPilotClaimRunRow` list only | + `function_symbol: Symbol` pins per row, `bool_witness_from_roster_row` mechanical projection (`function` String retained for shell transport) |
| `src/v4/test/claim/workflow/unified_test_claim_substrate_equivalence.dag` | absent | Phase 1 compile-time equivalence tranche (`substrate_equivalence_holds`) |

## Explicit non-changes (this PR)

| artifact | reason |
| -------- | ------ |
| `src/v4/compiler/05_eval.dag` | `TestClaimEvalSubject` binds at eval boundary — Phase 2 runner |
| `testclaim_corpus_runner.dag` | runner dispatch deferred Phase 2 |
| `v4_roster_pilot` row list / count | authoritative until C9–C11 (design §4.4) |
| `workflow/*_eval.dag` (10 files) | retire Phase 3 |
| lens-CI / smoke shell scripts | CI transport unification Phase 4 |

## Falsification receipts landed

| ID | receipt |
| -- | ------- |
| C3 | `rg 'UnifiedTestClaim\|BoolWitness' src/v4/std/verification.dag` |
| Phase 1 tranche | `witness_substrate_equivalence` in `unified_test_claim_substrate_equivalence.dag` |

## Deferred to Phase 2+

- `UnifiedClaimRun` / tagged `CorpusEvalReport` run receipts  
- `run_unified_corpus_eval` dispatch  
- E1 (`--claim-run` stdout equivalence)  
- `NodeCorpus.subject: TestClaimEvalSubject<Node>` field (std/eval layering)
