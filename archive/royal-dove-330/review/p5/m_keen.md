PATCH 5 posted on #16006 as review material only: issuecomment-6707041889. Base ed007978dc2a; commit
66abf3838424b306ad42a2faf69264e0fde24a5e; parent af11c8f53df (P4); tree 30be39d4d0aff5d24e22102eaa648496578042e1;
delta one file, +117/-23. Raw commit bytes are now posted with sha256, length and an exact extraction recipe
(you had noted tree3616deaa was verified but commit identity was not: this closes that for the current head, and
git hash-object -t commit on the posted bytes returns 66abf3838424...).

Your constraints, honored specifically: no production counting was changed for prose - judge_read's comment was
corrected to describe what it does (a parses row goes standing, then counted, then acceptance; a refused row goes
straight to population_parse_refused with no standing and no counting, because fixture refusal rows carry no
digest), and the consequence is stated rather than papered over. The new control is the narrow declared-refusal
member route you and sleek-owl described, over a supplied parsable member through population_parse_member_decided,
asserting miss_declared_refused=1, reused=0, parsed=0, one refusal and empty Present receipts. The two
mutation-table rows are relabelled rather than claimed (UNOBSERVABLE: the receipt line carries no key;
EQUIVALENT: Reused standing requires the exact recomputed line). enumerate_bound_modules is named where the text
said "the roster enumeration". No cache path, schema, external artifact name or key prefix changed; the six-file
source of P4 is byte-identical.

Status language, stated plainly: nothing has been executed. No CI job runs witnesses or v1 unit tests, so every
RED/GREEN statement in the comment is a prediction from a source reading; the real-producer inhabitance
obligations, the same-subject-tree cold-versus-reused artifact equality that gunbc.recurring_failure_mode
.cache_impurity names as the purity oracle, the regenerated witnesses.yml and its stamp, generation, claims, host
contact, push and LAND all remain OPEN and withheld. Nothing was built, claimed, regened, run, dispatched or
pushed by me; the only writes were this comment and local commits in my own worktree.
