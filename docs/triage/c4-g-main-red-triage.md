# C4-G main-red triage (session wise-moth-507, manager warm-wolf-570)

Running table. One row per red in the queue. Statuses: REPRO | CULPRIT-FOUND | ROUTED | FIXED-FWD | VERIFIED-RESOLVED | SUPERSEDED (already fixed on main).

Repro method: fresh `git clone --filter=blob:none` of https://github.com/gunb-ai/gunbc.git on the ctrl-build remote runner, checked out at a printed SHA, private `CARGO_TARGET_DIR` per tree, build + run in one dispatch. `claim_batch --source-root dag --source-root src/v2 --entry <file> --functions <fns> --eval-budget-ms 300000` (fleet-converge.yml recipe).

| # | claim | queue SHA (as briefed) | status at current origin/main 7472ecb69a72 | culprit PR | owner | status |
|---|-------|------------------------|---------------------------------------------|-----------|-------|--------|
| 1 | test.claim.quarantine_probe_disposition_witness: every_quarantine_probe_derives_exactly_one_disposition, every_supplied_population_is_load_bearing_on_the_live_claim | 1bec8b5973f7 | **PASSES 14/14 at main b07157704cd** (receipt: sunny-eagle-60, remote claim_batch head-built, msg_35a1d622) | likely PR #14532 (5309187aede 'refuse an admitted quarantine probe held by nothing'); not proven by identity per manager | (fix landed) | FIXED-ON-MAIN (manager ruling msg_2265f35b: receipt accepted, no bisect). every_supplied_population_is_load_bearing_on_the_live_claim was split into 4 per-population claims by #14532; old name no longer exists on main |
| 2 | dag/test/claim/roadmap/roadmap_hierarchy_view_witness.dag compile refuse :326 "variant 'Document' not found in type 'Fragment'" | 8f139bb33ac | control PENDING (j06e99492) | PR #14351 (5cda378342f2) introduced; PR #14629 (237b03d48ce9) removed the dead arm 2026-10-21 | (fix landed) | SUPERSEDED? pending FAIL@culprit/PASS@parent + PASS@main proof |
| 3 | roadmap_authority witness_accepted_parents_precede_dependents FAIL (Absent arm) | n/a (current main) | PENDING (nimble D1/D2 receipts) | candidate: PR #13052 (b9cc71cf46d6, gunbai-bot, merged 2026-10-03) added edge(child: 2-scm-*, parent: fabric-bringup) x4 without accepting fabric-bringup | (manager escalation) | FIX IN FLIGHT: #14701 (approved+granted, lands after #14657) removes those edges — manager ruling msg_143c0ca0. Do NOT append an AcceptanceRecorded for fabric-bringup (an acceptance judgment nobody made). D1/D2 at main = before receipt. Static analysis: faithful Python replay of the real carrier shows the 4 accepted nodes (2-scm-git/mercurial/pijul-upstream-model, 2-scm-compatibility-shape) have parent fabric-bringup with no AcceptanceRecorded -> all(...) false |
| 4 | dag/test/claim/react_jsx_emit.dag refuses 0/4 'gunbhub_browse_page not indexed under gunbc.gunbhub_serve' | 3b63a9707b56 | control PENDING (j06e99492) | PR #14351 (5cda378342f2) narrowed closure; PR #14623 (1b732983b97f) added the imports 2026-10-21 | (fix landed) | SUPERSEDED? pending FAIL@culprit/PASS@parent + PASS@main proof; jolly-boar-495 confirmed not started |

## Notes

- All three briefed SHAs are ancestors of current origin/main (7472ecb69a7253d124e15908b886e2c4c52d2f7e).
- Item 2 lineage: `Fragment` lost the `Document` variant in #14351 (std.markup gave `MarkupDocument` its own type, 2026-10-19). The witness's `hv_nav_hrefs` arm was deleted 2026-10-21 by 237b03d48ce9 (#14629), whose own message cites "#14351 review 82244".
- Item 4 lineage: same #14351 narrowing; imports restored 2026-10-21 by 1b732983b97f (#14623).
