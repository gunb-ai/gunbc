Follow-on to the absence mapping: the causal joins, from retained logs plus the cache listing. Read-only; nothing run, dispatched or pushed. This also replaces one detail in my earlier wording.

THE HIT, joined end to end (run 42362849618, v2-bootstrap job 128374654255; job log fetched read-only):
- 11:19:47.398Z Cache hit for: gunbc-parse-identity-49d4bc36b6bbdf29b539c6b38c37ddbb922633e0 -> restored 11:19:47.645Z
- 11:19:48.026Z Cache hit for: gunbc-materialization-parse-verdict-a5c4d44bebe7a2362cc905df8ecaabb827b2550671a038124ceb6344eea94d1f-49d4bc36b6bbdf29b539c6b38c37ddbb922633e0 -> restored 11:19:48.270Z
- 11:19:54.346Z parse-verdict reuse: ancestor=49d4bc36… reused=10101 miss_identity_absent=0 miss_identity_mismatch=0 miss_entry_absent=0 miss_digest_unseen=4 miss_declared_refused=0
Both artefacts restored, walk ~7 seconds.

THE FULL MISS, joined end to end (run 42339827972, lines 1413-1504 of the job log):
- ordinals 0..10 asked for gunbc-parse-identity-<sha> for eleven ancestors, all "Cache not found for input keys"
- 05:55:15.527Z Cache hit for: gunbc-parse-identity-fa354186e431… (the retained entry, created 2026-11-06T17:14:14Z) -> restored
- 05:55:16.154Z Cache not found for input keys: gunbc-materialization-parse-verdict-a5c4d44bebe7…-fa354186e431… (the receipts under the HEAD's identity for that ancestor)
- RESTORED_ANCESTOR: fa354186e431…; 06:18:02.969Z parse-verdict reuse: reused=0 miss_identity_absent=0 miss_identity_mismatch=10094 miss_entry_absent=0 miss_digest_unseen=0 miss_declared_refused=0 -> 22m46s

WHAT THAT ESTABLISHES, and the wording it fixes. The counters are ordered: with the restored ancestor's parse-compiler identity differing from the head's, every member is charged miss_identity_mismatch and miss_entry_absent stays 0 even though the receipts file was not restored. So the identity comparison short-circuits before the receipts are consulted: a stale parse-compiler identity blob makes the receipts irrelevant and the whole population re-parses. Corollary, which sharpens your rule that "mismatch or eviction can miss independently": the failure mode is not only distance but IDENTITY ERA. The receipts key is gunbc-materialization-parse-verdict-<HEAD's compiler_identity>-<ancestor sha> (witnesses.yml:323 etc.) while the standing check compares the restored gunbc-parse-identity-<ancestor> blob against the head's identity, so a walk can only hit an ancestor whose publication carries the head's current identity. Across 296 retained entries there are 16 identity eras in about two days, so era changes are frequent; a walk whose nearest published ancestor predates the current era pays a full walk at any distance.

Consequence for Change 1's expected runtime, stated as a bound rather than a promise: the push-route hit (6-16 s) requires the located ancestor's published identity to equal the head's parse-compiler identity. When the parse path (tokenize+parse+schema closure) has changed since that ancestor, the push walk pays the full miss (measured 1,366 s for 10,094 members) even though Change 1 removed the route asymmetry. Two receipts now show both branches at key and counter level. Still nothing executed by me, and same-subject cold-versus-reused artifact equality remains the unmet purity obligation.
