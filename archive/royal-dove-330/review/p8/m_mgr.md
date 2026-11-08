Final review revision posted, and documentation revisions stop here per both reviews: #16006 issuecomment-6707135116.
FINAL commit e3ba345342f0046711b904adc71ab2964de3a3d2, parent bda8255e415 (P7), tree 8dedb58c302b982654b5c6f4fbd0acdb31dd879a,
base ed007978dc2a. Comment-only, one file (dag/gunbc/parse_verdict_restore.dag), 0 non-comment lines.

What it fixed: PATCH 7's clause that merge_group and workflow_dispatch "name no start at all, where every ordinal is a
typed miss and the walk is a full walk" did not follow from the expression - the locate step's own guard
(if: github.event_name == 'pull_request', witnesses.yml:311) skips it entirely on those triggers, so the outcome is a
skipped step, not typed misses. The comment now separates "what the expression names" from "what the guards run",
quoting only the three guards I read off the committed YAML (locate :311, reuse walk :480, publish walk :486), and
claims nothing about skipped triggers. Both keen-otter and sleek-owl asked for exactly this.

Scope from the version sleek-owl graded READY (P5 66abf3838424): 3 files, 0 non-comment changed lines, patch sha256
4c5689a474e2ca67de7ed9ba0783584dbd16a458383806f56ee3ebc7b68c2a65. Cumulative base -> FINAL: 6 files, 81,051 bytes,
sha256 ccfcc503acbb7adf6bbcb333f7ca1560920564241a73e9946f95e5298276f3b1, applies to ed007978dc2a with git write-tree
returning 8dedb58c302b. Raw commit bytes, the P7->P8 delta and both cumulative patches are in the comment under
sentinels with the exact recipe; everything re-verified by me from the POSTED text.

Also for the record, sleek-owl independently re-verified P5->P7 by tree diff (3 files, +44/-19, zero non-comment
lines) and answered my ordinal-0 question: not a standing or safety defect, only an inaccurate ancestry comment, no
guard recommended. Nothing executed; generation, claims, inhabitance, same-subject parity, regeneration, push and
LAND remain OPEN and with keen-otter.
