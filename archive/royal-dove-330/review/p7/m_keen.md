Option (a) done, posted as #16006 issuecomment-6707115673. Commit bda8255e415bf924e0bf941f8538f2ddec4c1770, parent
56ead5acee36 (P6 preserved), tree 96103c723b4c5096142c50847bd18b3d14020e9a. One file,
dag/gunbc/parse_verdict_restore.dag, comment-only. Verified from the POSTED text: the commit payload hashes to
bda8255e415, the P5->P7 patch hashes to 0c42baf2..., its non-comment changed-line count is 0 across the three files
(dag/gunbc/parse_verdict_restore.dag, src/v2/cli/population_verbs.dag, src/v2/test/claim/cli/population_verbs.dag),
and the cumulative applies to ed007978dc2a with write-tree returning 96103c723b4c. Cumulative: 6 files, 80,771 bytes,
sha256 777ae6561298ff0f88e9a694ec35f7a7d929f96b4fbd855934fef5a28c66e0b0.

Your wording constraints, as implemented: the comment states ordinals 0..11 with ordinal 0 being the start itself
and 1..11 its first-parent predecessors; PR start github.event.pull_request.base.sha, push start
github.event.before; it says outright that nothing here excludes the commit under verification and that this code
enforces no such exclusion; "never the start" and "no caller asks for 0" are gone; the blanket "PR base is an
ancestor of head" is gone, replaced by the distinction you asked for (the job checks out github.sha, GitHub's merge
commit for the pull_request triggers, distinct from the source head github.event.pull_request.head.sha, and nothing
requires the reported base to be an ancestor of either); force-push before can be a non-ancestor, and a batched push
neither makes the two equal nor by itself fails anything; all of it framed as selection facts about event metadata,
with the standing contract stated FIRST and unchanged (parse_verdict_standing from head and base parse-compiler
identity plus the digest this walk recomputes from bytes on disk; an ancestor's position confers nothing); and an
explicit sentence that nothing under those event shapes has been executed. No guard added, no control added, no
refreeze.

One thing beyond the minimum wording, flagged rather than buried: the same sentence completes the start rule by
naming merge_group and workflow_dispatch as triggers that name no start, so every ordinal is a typed miss and their
walk is a full walk. My BASE_SHA expression is (event_name == 'pull_request' && event.pull_request.base.sha) ||
event.before, so on those two triggers it yields empty. If you would rather that half-sentence not appear, say so and
I will cut it in a further comment-only delta.
