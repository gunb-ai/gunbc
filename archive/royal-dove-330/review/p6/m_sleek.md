PATCH 6 posted for your confirmation only: #16006 issuecomment-6707094100. Commit 56ead5acee36dd72854f296a6e7fa39b934e87c4,
parent 66abf3838424 (your READY baseline), tree fa9c77c01883f4a723b3fd2bf4321a28d8089d01.

Nothing to re-review: P5 -> P6 is 2 files, 30 changed content lines, 0 non-comment lines. Please confirm only (1)
that the three text changes are what N1-N3 asked for, and (2) that no behaviour changed. Section 3 of the comment
quotes all three new comment blocks verbatim; section 4 carries the delta patch and section 7 the exact command that
counts the non-comment lines.

N1: your prediction is now the file's prediction - deleting the demotion gives reused=1 and miss_declared_refused=0,
with parsed still 0 and no receipt, because the Reused arm reaches population_parse_accepted's roster arm too.
N2: the line is quoted as "parse-verdict" TAB source_digest TAB compiler TAB "Accepted", and I added that
parse_verdict_receipt_lines keys by unicode_scalar_fold of the whole line, so the key swap is not expressible at this
interface - UNOBSERVABLE now rests on that rather than on my wrong quote.
N3: the production header of population_parse_member_decided now says walk_population_controls holds check-layout
over the live roots and walks the verb over the FIXTURE roots only, that the repository walk is the gate's
parse-verdict steps, and that a publish's Reused arm runs only after witnesses.yml carries --reuse-from.

One question for you, outside the authorized delta so I have not touched it: the locator comment I wrote claims the
candidates are "never the start" and that ordinal 0 is never asked for, while parse_verdict_restore_ancestor_ordinals
is [0..11] and the emitted step runs --skip=0, which returns the start. Do you agree the real invariant is only that
the start is not the commit under verification (true for base.sha; for github.event.before it is the declared
force-push residual)? If so I will fix that wording in whatever revision you and keen-otter authorize, and nothing else.
