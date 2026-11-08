Exact mapping of every claimed cache absence. Read-only cache API only; no run, dispatch or push.

LOOKUP. Command: gh api "repos/gunb-ai/gunbc/actions/caches?per_page=100" --paginate --jq '.actions_caches[] | [.key, .created_at, .last_accessed_at, .size_in_bytes] | @tsv'. Scope: repo-wide listing, NO ref filter and NO key= prefix query — the full retained set, 667 rows. Lookup time: 2026-11-07T12:48:41Z to 12:48:43Z (an earlier identical listing at ~12:41Z gave the same 667 rows). Key families: 301 gunbc-parse-identity-<sha>; 296 gunbc-materialization-parse-verdict-<identity>-<sha>, spread over 16 distinct identity components (largest: 3993d07dea46… 49 entries, afda9263c748… 46, 9583350e8dc5… 25, a5c4d44bebe7… 12); 41 cargo-ci-Linux-ARM64-<identity>; 29 release-bins-<identity>.

MATCHING RULE, and the identity answer. I matched the FULL 40-hex commit sha as a substring anywhere in the key, across ALL keys, so NO identity was used or needed: an entry published for a commit under ANY parse-compiler identity would have matched. My earlier sentence "8/8 post-revert commits have none" is therefore an identity-independent absence claim over the retained set — not "absent under the identity I guessed". I cannot derive a commit's identity from source: the identity component is produced by that run's "Compile the parse-compiler identity blob" step (a digest over the tree's tokenize+parse+schema closure), which I am not authorised to run. Limitation stated plainly: the API lists only what Actions still retains, so eviction would be indistinguishable from never-published; these commits are 1-3 h old.

PER COMMIT (receipts key family and identity key family; run status from the push-runs API at the same time):
- 0cfb85d8 (10:22Z): receipts keys 0, identity keys 0. Run 50445 conclusion=cancelled. Mechanism established by the manager: save steps skipped.
- f793a798 (11:03Z): receipts 0, identity 0. Run 50451 conclusion=cancelled. Same mechanism.
- ed007978 (11:11Z): receipts 0, identity 0. Run 50453 NOT concluded (in progress/queued since 11:12Z).
- a7de1afe (11:30Z): receipts 0, identity 0. Run 50459 not concluded.
- cce400af (11:41Z): receipts 0, identity 0. Run 50460 not concluded.
- 2340e0af (11:41Z): receipts 0, identity 0. Run 50461 not concluded.
- 8d22701a (11:42Z): receipts 0, identity 0. Run 50462 not concluded.
- bef04740 (12:38Z): receipts 0, identity 0. Run 50470 not concluded.
So for six of the eight, absence so far means only "no save has happened yet"; it does not yet say why. The two cancelled runs are the ones with an established mechanism, as you recorded.

49d4bc36 (merged 10:01Z under the restored 90) — NOT absent; run 50438 conclusion=success, both keys present:
- gunbc-materialization-parse-verdict-a5c4d44bebe7a2362cc905df8ecaabb827b2550671a038124ceb6344eea94d1f-49d4bc36b6bbdf29b539c6b38c37ddbb922633e0 — created 2026-11-07T11:03:53.114043Z, last_accessed 2026-11-07T12:43:12.537593Z, size 102,890
- gunbc-parse-identity-49d4bc36b6bbdf29b539c6b38c37ddbb922633e0 — created 2026-11-07T11:03:53.639773Z, last_accessed 2026-11-07T12:43:11.914089Z, size 241
So a completed main push under 90 does publish both artefacts, ~63 minutes after the merge, and the receipts entry was hit by a PR walk at 12:43Z, five minutes before my listing.

WORDING, held to your rules. "Loop re-arming" is an INFERENCE, not established by these rows: it needs the causal joins (which runs reached the save steps, which identities the PR walks asked for, which entries they got). An absence alone does not say why nothing was published; for 0cfb85d8 and f793a798 the skipped save steps do, and I have not claimed a cause for the other six. "PR walks will start failing" is a FORECAST: the distance from main's tip to the nearest published ancestor is 8 now (49d4bc36 at ordinal 8, bound 11), so the 12-candidate window is exhausted only past 12 consecutive unpublished ancestors, and an identity mismatch or an eviction can produce a miss independently of distance. Noted also: the final source-review target may advance beyond P7 only for the event-guard comment correction — that is exactly PATCH 8 (e3ba345342f, tree 8dedb58c302b, comment-only, one file), and generation, push and LAND stay with keen-otter.
