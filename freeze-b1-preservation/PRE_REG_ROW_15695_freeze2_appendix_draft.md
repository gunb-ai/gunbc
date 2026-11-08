# DRAFT APPENDIX to PRE_REG_ROW_15695_freeze2.md (sha a31b8bb6…) — proposed, NOT yet folded in.
The frozen row file is left untouched so its hash stays reviewable; the parent asked for no silent edits.

Executor column (the point of msg_2d4214c8 §4): a PRE_REG row that does not name the lane that executes the
entry lets "predicted" be read as "observed". Every freeze-2 row should therefore carry:

| column | meaning |
|---|---|
| entry | claim file + function |
| leg | tree it is evaluated against (A/B) |
| expected | intended-semantics expectation, written before any run |
| executor | the exact executing lane: `claim_batch --entry <file> --functions <csv>` group, or `NO LANE` |
| observed | filled ONLY after the run, from a log with a measured sha; `UNOBSERVED` until then |

Applied to the #15695 rows as they stand today:

| entry | leg | expected | executor today | observed today |
|---|---|---|---|---|
| namespace_xl0/return_tail_position::a_tail_return_of_a_fn_body_is_accepted | A | PASS under candidate P / bare FAIL under dead-rung (HYPOTHESIS) | NO LANE (no workflow runs claim_batch; claim_executor runs only --verify-build-artifacts and --regen-round-cost; --required-ci/--required-lane are enrolled by no workflow) | the v17 packet only: FAIL (log 5d485d14…) |
| the other 16 rtp entries + 2 ce entries | A | per the frozen row set | NO LANE | v17 packet only: 11 rtp + 2 ce FAIL, 6 rtp PASS (vacuous, refusal-expecting) |

Consequences to record in the freeze-2 PRE_REG header, not per row:
- CI green (v2-bootstrap) is not verdict evidence for any claim; it executes `gunbc test
  //gunbc/instruments:self-host` and `:v2-native-cli` and never claim_batch/claim_executor.
- Qualification rows in dag/gunbc/compiler_frontend_program_status.dag whose owner is a v2.test.claim decl
  (24 of 27 PrerequisiteDelivered rows; 18 with measured_at == merge_sha) are CLAIMED, not VERIFIED, until the
  entry appears in an executor column with an observed log sha.
