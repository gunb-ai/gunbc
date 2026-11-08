# Preservation archive - session royal-dove-330 (node adhoc-189dd15f-1a8)

Preservation snapshot only, pushed on the instruction of silent-bee-845 (msg_0b5a422e, 2026-11-08)
after the operator wound down the keen-otter program (#16088). NO PR, NO merge, NO new work.
Nothing here was built, claimed, tested, regenerated, run or dispatched: the standing build policy
is WORKLOAD-based and bars any compiler/test/build/claim/regen workload in session trees. Every
"compile" statement in these files is a CI receipt or a PREDICTION, and the label says which.

## What pairs with what
- Branch archive/silent-bee-845-2026-11-08/royal-dove-330-change1 is the reviewed SOURCE alone:
  commit e3ba345342f0046711b904adc71ab2964de3a3d2, tree 8dedb58c302b982654b5c6f4fbd0acdb31dd879a,
  parent bda8255e415bf924e0bf941f8538f2ddec4c1770, base ed007978dc2a67096df619b1b91566458dfb2ffd,
  6 files changed / +845 / -102 at PATCH 5 and unchanged since. Independently rated SOURCE READY by
  keen-otter-392 and sleek-owl-397. It is NOT pushed as a PR and NOT merged, and this branch's
  bytes are exactly the reviewed ones.
- This branch (royal-dove-330-notes) is a child of that commit adding ONLY the archive/royal-dove-330/
  directory. No non-archive path differs from the source branch.

## Contents
- notes/       the design note (note.md, note2.md = sections 1-13, note3.md = section 14 generation
               proposal, body.md), plus every message this session sent (m_*.md, wcr.md, pm.md,
               corr.md, add1.md, pushnote.md, blocked3.md) and the four walk-cost record sources
               (wcr.md, corr.md, pm.md, add1.md).
- review/      the review material as published on #16006: PATCH 1-2 (comment_A/B), PATCH 3, 5, 6,
               7, 8 (comment.md = the comment body, commit.raw = raw commit bytes, cum.diff =
               cumulative base->revision patch, readback.md = the read-back verification), orig.gzb64
               = the sentinel payload whose sha256 and length were quoted in each comment, and
               verify/ = the verification transcripts. Each .raw/.diff pair reproduces the identity
               quoted in the matching #16006 comment by the rule stated there:
                 git hash-object -t commit <commit.raw>          -> the commit sha
                 git apply --index <cum.diff> on base + git write-tree -> the tree sha
- patches/     standalone patches: base->PATCH1 (0001-*.patch, two revisions), base->FINAL
               cumulative, and PATCH5->FINAL (comment-only), with change1_base.txt naming the base.
- evidence/    the read-only evidence tables the analysis rests on: allcaches.tsv (667 cache rows
               from `gh api caches --paginate`, looked up 2026-11-07T12:48:41-43Z),
               receipt_keys.tsv, main_commits.tsv, main60.tsv, pushruns.tsv (the seven completed
               main pushes), fm.txt (979 recurring_failure_mode module names at main),
               mainfiles.txt (the 46 files changed upstream since base, used for the zero-overlap
               check), member_controls.dag.txt, job48.log.
- run-logs/    the seven Actions run logs fetched read-only with `gh run view --log`, cited by run id
               in the notes: 42292182286 (main push, cut at 90:00), 42276359442 (PR, cut at 90),
               42339827972 (PR, passed 89m17s), 42341242245 (PR, cancelled 90m18s), and the three
               4230x/4231x runs carrying the Q1 host_effect.dag:248 admission error.

## Headline conclusions carried by these files (detail inside, not here)
- Emit steps: process total = compile_transaction + process floor EXACTLY (transactions=1); the two
  floors are 98.15% and 98.30% of their transactions; the cargo build sits INSIDE a floor and is
  never additive. Emitter: src/v1/stage0/src/cli_run/pre_entry_phase.rs:302/:309/:368/:378.
- Walk: at least 99% of a full walk is per-member acquisition_parsed; the lexer and grammar are
  prepared once per walk and the syntax tree is discarded; the tally machinery is under 1%; there is
  no duplicated work inside the walk to remove. Hits run 5-16 s against a measured full miss of
  1,366 s over 10,094 members (~106 KB/s, UNEXPLAINED - the walk step emits no phase rows).
- Cache/era: parse-verdict receipts are keyed by the HEAD's parse-compiler identity, so a stale era
  is indistinguishable from absence (reused=0, miss_identity_mismatch=10094, miss_entry_absent=0);
  16 identity eras and 296 receipt entries in about two days; last publisher 49d4bc36 at
  2026-11-07T11:03:53Z; distance 8 of a 12-candidate window (a forecast, not an exhausted budget).
- The publication-side loop is confirmed at key level; "loop re-arming" remains an INFERENCE pending
  causal joins, and memoizing the parse is barred as an answer by operator ruling R2 2026-10-08
  (recurring_failure_mode row parse_memo_hit_serves_occurrences_minted_on_an_abandoned_path).

## Still OPEN when the lane closed (all withheld, none of them this session's to decide)
witnesses.yml regeneration and its stamp; the generation frame; claim execution; real-producer
inhabitance of lookup, the CLI seam and the published artefact; same-subject cold-versus-reused
artefact equality; LAND. The controls have NEVER executed, so every RED/GREEN statement in review/
is a prediction.
