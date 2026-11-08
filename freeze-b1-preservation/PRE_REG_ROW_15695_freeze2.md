# STAGED PRE_REGISTRATION ROW — fold into the freeze-2 PRE_REG (owner sleek-seal-226, #15695; parent msg_31970ddf)

Status: HYPOTHESIS, not an observation. Written 2026-11-06, BEFORE any freeze-2 run. Expectations are frozen
here and may not be changed after output (rule: no expectation edits post-observation without a separately
justified contract review).

## Row R-15695-1 — the positive control of the rtp acceptance path
- claim file: src/v2/test/claim/namespace_xl0/return_tail_position.dag (blob 514c1d32f10dff96aee9298bc20aea1a2c6e1583 at 4507aea / 9c15c98c / f08a6ae4 / 495da688)
- entry/function: `a_tail_return_of_a_fn_body_is_accepted`
- fixture: `rtp_tail_ctl_source` = `module v2.test.rtp_tail_ctl` + imports {Bool, Int} + `fn rtp_probe(i: Int) -> Int { return i }`
  — no if, no match, no else-less if, so the #15695 condition-production change cannot reach it.
- expected verdict: `NativeCensusModuleResolved` (rtp_verdict_of :192-194); claim returns true iff the census
  module outcome is Resolved.

| leg | tree | PREDICTION under candidate P (head-read cause) | PREDICTION under the dead-rung hypothesis |
|-----|------|-----------------------------------------------|--------------------------------------------|
| A (repair head, #15999 head after the main merge) | repair head | **PASS** | **FAIL** (bare) |
| B (base leg, pre-repair base of the same packet) | its base | PASS for this entry only (fixture is grammar-neutral) | FAIL (bare) |

Discriminating power: this single row separates the two hypotheses, because the fixture is unaffected by the
parse fix. A PASS at A means the acceptance path can resolve a module and the v17 head reading was the cause;
a bare FAIL at A means nothing on the acceptance path resolves and the dead-rung hypothesis stands.

## Supporting evidence already measured (not a prediction)
- v17 at seed 495da688: this entry FAILED (bare). Log sha256 5d485d14f57a386d4c0b17e4751f48989033316d62e8bcfe9644be94d840d7dc.
- 7 acceptance entries and 4 unbound-read entries FAIL; the only 6 PASS are the FileRefused-expecting claims,
  which pass identically under an always-refuse fold.
- The PASS predictions for these claims were never observed: at 9c15c98c CI is a single job (v2-bootstrap, job
  126098202932, run 41687838568) whose log contains zero occurrences of any of the 17 claim names, runs
  `cargo test --release -p v1-compiler --lib --no-run` (compile-only) and a native-cli door probe, and never
  invokes claim_batch; `git grep return_tail_position` over .github/, tools/, *.toml is empty and
  src/v2/workflow/floor*.dag never names these claims.

## Requirement this row imposes on freeze-2
Bare FAILs must stop being the evidence class: the freeze-2 roster must carry a REASON-BEARING witness for the
acceptance entries (print the RtpVerdict fatal_reason/at, or add `rtp_tail_ctl_fatal_reason_is_resolved`), so a
FAIL distinguishes FileRefused(reason=...) from ResolveRefused(...) from RtpAbsent. Without it the row cannot
be adjudicated beyond PASS/FAIL.
