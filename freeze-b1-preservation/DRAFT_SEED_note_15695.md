# freeze-2 draft seed (LOCAL ONLY, nothing pushed, nothing posted)

measured in /var/tmp/freeze2-cut-15695/repo (0700) at 2026-11-07T07:35Z

| item | value |
|---|---|
| accepted base (parent) | 57ce2e787aa5227fc1840961c607981f4abdde86 (tree 5a14ac7ca50f43312760f092c86ebee7c679ce37) |
| seed diff applied | SEED_PROBE_SET_rebase_candidate.diff sha256 2852848fa9c630f96a4050d7c2c9de5da9963665d8216fc0473c79f5aa12461b (comment 6702987648) |
| apply | `git apply --check` rc 0 with NO fuzz, then `git apply` rc 0; exactly one file modified: src/v2/test/claim/namespace_xl0/return_tail_position.dag |
| DRAFT seed commit | c07e2b8b666e0d2c529a4b0c44475f28db6cc129 |
| DRAFT seed tree | d2bd176dc1ba1de4fff1387c145f91fb7ad5756c |
| branch | draft-seed-15695 (HEAD of the cut clone); other branches: freeze2-base-15695 = 57ce2e78, repair-main-merge = 0e283cba |
| `grep -c '^test fn '` on the rtp file | 21 (17 registered + 4 probes) |
| probe names present | rtp_ctl_spelling_is_resolved, _is_absent, _is_file_refused, _is_resolve_refused (4 `test fn` lines) |
| expected literals present | "RESOLVED" x2, "ABSENT" x2, "file_refused:<none>" x1, "resolve_refused:return_not_in_tail_position" x1 |
| probe block (both diffs) | 45 lines, sha256 af6f17477ac0f39591abb5345840e0eac827e388072afddb39f002b7eb17bd22 |

Note: probe_block_digest.py takes DIFFS, not .dag files; running it on the .dag prints "NO PROBE BLOCK" by
design (it looks for added lines and the 'FREEZE-2 REASON WITNESSES' comment). Presence inside the applied file
is instead proven by the name/literal measurements above.

Status: this is PREPARATION ONLY. It is not the frozen seed; it becomes the seed only after the packet review
(sleek-owl-397 -> keen-otter grant -> calm-crab-645) and the parent's word. No push, no grant ask, no run.
