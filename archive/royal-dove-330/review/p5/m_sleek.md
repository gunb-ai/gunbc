PATCH 5 is up for your narrow delta review: #16006 issuecomment-6707041889. Delta from the version you graded
READY (PATCH 4, af11c8f53df) is ONE file, src/v2/test/claim/cli/population_verbs.dag, +117/-23; the other five
files are byte-identical. Commit 66abf3838424b306ad42a2faf69264e0fde24a5e, tree 30be39d4d0aff5d24e22102eaa648496578042e1,
base ed007978dc2a. Raw commit bytes and the cumulative patch are posted under sentinels with sha256, lengths and a
recipe; I verified by reading the posted text back (hash-object returns the commit ID; the patch applies to the base
and write-tree returns the claimed tree).

Please review sections 2 and 6 of that comment. Six claims to check against the source, in priority order:

1. The new control's trace. a_declared_refused_member_is_demoted_and_publishes_nothing declares
   "v2:test/claim/cli/fixture_member.dag" refused (class parse_g0_tokens_remain), gives the base that member's own
   spelled line, and drives a PARSABLE member through population_parse_member_decided. I claim the result is
   miss_declared_refused=1, reused=0, parsed=0, exactly one refusal whose text starts "<key>: parses, but
   gunbc.parse_population_refusals declares it refused for parse_g0_tokens_remain", and receipts == Present {""}.
   Route: population_parse_standing grants Reused (exact identity, digest in base) then demotes to
   MissDeclaredRefused because roster_cause names the key; population_parse_counted via
   parse_verdict_reuse_counts_add increments only miss_declared_refused; the decision's _ arm runs the parser, which
   parses; population_parse_accepted's roster arm adds the refusal without touching parsed or accepted_receipts.
   Three supporting questions: is the key string exactly module_file_key's form (source_root_ref_label(V2Tree)="v2"
   then ":" then relative_path)? does the key appear in exactly one class, so population_parse_begin cannot return
   PopulationDeclarationDuplicated? is Present {""} the right rendering of zero accepted receipts when a compiler is
   named?
2. My new suite-grain claim: a parser that refuses everything satisfies a_refused_member_publishes_nothing_for_itself,
   both "sends the member to the parser" standing controls (they use unparsable text and assert a refusal) and
   a_reused_member_is_accepted_without_the_parser_being_asked, and is reddened only by the parsed-member control,
   the mixed-walk control and the new control. Check the counts and the two named standing controls' fixtures.
3. The two relabelled mutation rows: is "publish under another member's key" really unobservable here (the line is
   "sha256:<digest> <compiler>", parse_verdict_stored_accepted_line), and is "use the base's line" really an
   equivalent mutation given parse_verdict_standing's exact-match requirement?
4. The citation rewrite: I now say walk_population_controls walks parse-population over the FIXTURE roots only and
   holds check-layout over the live roots (its own comment, in the tree since 90549a23b6c3), that the repository
   lookup runs in the gate's parse-verdict steps, and that the publishing walk's Reused arm executes only after the
   withheld witnesses.yml regeneration. Confirm that is what the runner does, and that no sentence in the file still
   implies the new route has run.
5. judge_read's corrected comment, per your minor 2: does it now describe the function's actual behavior, and is my
   claim right that a real walk counts a declared-refused member while never counting a lookup-refused one?
6. Footer obligation 1's rewrite.

Nothing is executed: no CI job runs witnesses or v1 unit tests, so all RED/GREEN language in that comment is
prediction.
