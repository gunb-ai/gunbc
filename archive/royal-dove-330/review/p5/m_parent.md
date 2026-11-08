PATCH 5 is posted on #16006 as review material: issuecomment-6707041889
(https://github.com/gunb-ai/gunbc/pull/16006#issuecomment-6707041889). Append-only; PATCH 1-4 comments are
untouched. Nothing was compiled, claimed, regened, run, dispatched or pushed, and posting pushed nothing.

Identity: base ed007978dc2a67096df619b1b91566458dfb2ffd; PATCH 5 commit 66abf3838424b306ad42a2faf69264e0fde24a5e;
parent af11c8f53df85bef7df8279f87c6e96b39fbe502 (PATCH 4); tree 30be39d4d0aff5d24e22102eaa648496578042e1.
P4 -> P5 delta: one file, +117/-23, src/v2/test/claim/cli/population_verbs.dag. Cumulative base -> HEAD: 6 files,
78,578 bytes, sha256 50326385400cd4b4d3b339af6f24012c1e15a0357ad6b95ca6eca34cb72172ce. Raw commit bytes posted
(2,059 B, sha256 a96edb5974e9537105d07b3edcd62ba33a068c06870658217f5e6fcb36e708d9) alongside the patch, both under
sentinel-delimited GZB64 with an exact extraction recipe.

Read-back verified against the POSTED text: body length 40,383 matches; the commit payload decodes to the claimed
sha256 and git hash-object -t commit gives 66abf3838424...; the patch payload decodes to the claimed sha256,
applies cleanly to ed007978dc2a in a scratch clone (since removed) and git write-tree returns
30be39d4d0aff5d24e22102eaa648496578042e1. Only difference between posted and local body: one trailing blank line
GitHub appended.

All six minors closed: (1) the walk_population_controls citation corrected at all three sites - the runner walks
parse-population over the FIXTURE roots only (its own comment, present since 90549a23b6c3), the repository lookup
runs in the gate's parse-verdict steps, and the publishing walk's Reused arm only after witnesses.yml carries
--reuse-from; (2) judge_read's comment now says refused rows skip standing AND counted (no production counting
changed for prose); (3) new control a_declared_refused_member_is_demoted_and_publishes_nothing drives the combined
standing/counting/refusal route over a parsable member declared refused, with its own line in the base, asserting
miss_declared_refused=1, reused=0, parsed=0, one refusal, empty Present receipts; (4) the two mutation rows
relabelled UNOBSERVABLE / EQUIVALENT with reasons; (5) the suite-grain statement naming the four controls a
refuses-everything parser satisfies and the three that require a member to parse; (6) enumerate_bound_modules
named where the text said "the roster enumeration". Plus one defect of mine fixed: a block comment that read as if
it documented the first member-route control now states it governs the whole group.

Per your instruction this is the last review revision unless review finds something new. Nothing has been executed:
no CI job runs witnesses or v1 unit tests, so every RED/GREEN statement in the comment is a prediction, and the
generation, execution, push and LAND holds stay with keen-otter.
