## Defect (silent wrong values)

A unary minus whose operand is followed by an infix `+`/`-` parses with the minus covering the whole tail: `-1 - 2` parses as `-(1 - 2)` and evaluates to **1**, not -3; `-a + b` as `-(a + b)`; `-1 - 2 - 3` as `-((1 - 2) - 3)` = 4, not -6. The bang arm absorbs the operators above the same gate: `!a * b` as `!(a * b)`. Nothing refuses - interpreter, v2 evaluator and every emitter read the same wrong tree. Unaffected before: standalone negative literals; a minus followed by a comparison, a logical operator or a closing bracket; and `!a && b` / `!a || b` (the logical rows 7/5 already stopped the old gate - published as measured, see Evidence; an earlier draft of this body and of the RFM row wrongly claimed those as REDs).

## Cause

`src/v1/02_parse.dag` `parse_prefix` parsed each arm's operand at a **literal `min_bp: 12`** - below `+`/13, `*`/15, `|>`/17 and the postfix rows, so the operand swallowed every infix tail above the literal, while stopping at `<=11` (comparisons, `&&`, `||`, `??`): exactly the affected/unaffected split. The infix gate reads the declared table (`find_operator_bp` over `ctx.env.syntax_spec`); prefix operators had no declaration to read. A review then showed the **same class in the postfix arms**: `try_postfix` and the no-brace loop gated call/index/`as`/record-literal on literals `21 < min_bp`, `13 < min_bp`, `14 <= min_bp` - the `as` literal is what made the two parsers DISAGREE on cast nesting once the prefix moved (see Postfix half).

## Fix (one authority, not a number change)

Operator binding powers are **declared** beside `OperatorSpec` in `dag/std/syntax.dag`:
- `PrefixOperatorSpec { symbol, binding_power }` rows in `SyntaxSpec.prefix_operators` - `{"-", 18}, {"!", 18}` in both `dag` and `rust` language specs. `parse_prefix` looks its arm's power up through `ctx.env.syntax_spec.prefix_operators` - the same table the infix gate reads - and `Absent => parse_primary` (fail-closed: an undeclared prefix operator never gets an invented power). **No literal binding power survives** in the prefix arms; both ShMinus and ShBang were fixed together (DESIGN 6b sibling-arm generalization).
- `PostfixOperatorSpec { symbol, binding_power }` rows in `SyntaxSpec.postfix_operators` - `{"(", 21}, {"[", 21}, {"as", 19}, {"{", 13}` in both specs. Every postfix gate in `try_postfix` and `parse_expr_loop_no_brace` reads `declared_postfix_gate_admits` from the same table; **no literal gate threshold survives** (witnessed in the regenerated mirror: `21 < min_bp` / `13 < min_bp` / `14 <= min_bp` / `min_bp: 12` counts all zero). A symbol with no row is not consumed as that postfix operator - the ordinary downstream refusal reports the leftover token.

**Why 18 and 19 (the alignment choice).** v2's `dag_grammar_unary_expr_expr` / `dag_grammar_postfix_as_suffix_expr` are structurally closed productions: the unary operand takes exactly one postfix chain and `as` binds tightest. v1's declared rows were DERIVED to reproduce those meanings: 18 sits above every infix `left_bp` incl. `|>` 17 (so a pipe stays OUTSIDE the operand) and below the dot's 19 (so `.` goes INSIDE); `as` at 19 sits above the prefix rows (so `-a as T` is `-(a as T)` on BOTH parsers - the shape v2 has always produced; at the old literal 13 combined with the moved prefix it became `(-a) as T` in v1 alone, which is what the side chat caught) and below call/index 21. wise-raven-693's alternative prefix value 17 is rejected with a measured reason: v1's infix loop continues while `left_bp >= min_bp`, so at 17 a pipe inside the operand survives its own gate and `-a |> f` re-binds to `-(a |> f)` - the absorption class moved from additive to pipe (measured `v1=P(M(l))` against the agreed `M(P(l))`). 18 is the least declared value that keeps the pipe outside the operand.

## v2 owed agreement, not re-plumbing (manager ruling)

The v2 grammar-row parse's unary production is closed: structurally immune, so v2 owes AGREEMENT, not a re-plumb. Proven by execution: the landed claim `src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag` runs the same **22 texts** through BOTH parsers and requires identical canonical shapes AND the independently stated expected shape - defect cases (`-1 - 2`, `-a + b`, `-a * b`, `!a * b`), controls (`-(1 - 2)`, `-3`, `-1 < 0`, `a - b`, `a * b`, `-a < b`, `!a && b`, `!a || b`), and the boundary arms that pin the declared rows: cast (`-a as T`, `-(a as T)` -> `P(C(l))`; `a + b as T` -> `I(l,C(l))`), dot (`-a.b` -> `P(F(l))`; `(-a).b` -> `F(P(l))`), pipe (`a |> f` -> `M(l)`; `-a |> f`, `!a |> f` -> `M(P(l))`; `-a.b + c` -> `I(P(F(l)),l)`). The canon walkers key on node KIND and, for v2, the Transform HEAD'S ATOM IDENTITY - arity alone conflated cast with infix and pipe with infix, which let an earlier version of this claim agree vacuously; every expected string is MEASURED off both parsers by the shape probes (WIP branch `session/nimble-otter-791`, lanes cited below), not read off the tables.

## Postfix half: cast/dot/pipe parity (side-chat REQUEST_CHANGES item 3)

With only the prefix rows declared, the 22-arm claim was RED on the cast arms: `a + b as T` gave v1 `C(I(l,l))` against v2 `I(l,C(l))`, and the pipes gave `P(M(l))` against `M(P(l))`. The declared postfix rows above are the fix; the same claim is rc=0 at the head. The two restored dot/pipe arms (`-a.b`, `-a |> f` - the side chat's "restore the advertised dot and pipe controls, with readers that distinguish those shapes") distinguish via the kind-keyed walkers: `P(F(l))` vs `F(P(l))` vs `M(P(l))` are three different strings.

**Cast census (DESIGN 5: an uncounted meaning change is a hard reject).** The full-corpus tree digest (175291 items / 9084 files, both bindings) under the historical powers (prefix 12, `as` 13 - the literals the old parser used) against the declared powers (18/19) differs at EXACTLY 12 authored items plus this PR's own fixture: `bed_partition.dag#5 process_allowance`, `bed_partition.dag#9 allocate_axis`, `cassette.dag#24 panel`, `cassette.dag#25 rear_socket`, `cassette.dag#30 guard`, `cassette.dag#36 rear_socket_bolt_clearances`, `rear_cooling.dag#13 rear_cooling_json`, `rear_fan_assembly.dag#10 guard`, `rear_fan_assembly.dag#11 fan_body`, `stack_geometry.dag#7 stack_pitch`, `stack_geometry.dag#10 pillar_height`, `stack_geometry.dag#14 stack_pitch_for_cooler` - all the shape `... + f(...) as Int` over `micrometer_count`'s Nat, where the two nestings evaluate identically, and which v2 has ALWAYS parsed the new way. They are deliberately NOT re-spelled: parenthesizing them to `(a + b) as T` would move BOTH parsers to a shape neither produces today. The site list, the counters and the digests are the census.

## Record literals under the prefix (review 84349): the `{` row moves 13 -> 18

The finding was verified before any change, and it is REAL at the family the gate decides. Measured
over both routes, twelve cells x declared-vs-faithful-old rows (shape probes, invocation
`jd1752ffb-...` see Evidence):

| cell | v1 at old rows | v1 at the first 13-row head | v2 grammar route |
|---|---|---|---|
| `-Foo { x: 1 }` | absorbed | absorbed | absorbed |
| `-Foo.Bar { x: 1 }` | **absorbed** | **declined -> `(-Foo.Bar)` + stray block** | **absorbed** |
| `!Foo.Bar { x: 1 }` | **absorbed** | **declined -> 2 stmts** | **absorbed** |
| `a + Foo.Bar { x: 1 }` | declined | declined | absorbed |
| `a * Foo.Bar { x: 1 }` | declined | declined | absorbed |

Two facts came out of the read. First, the plain family never reaches the declared gate: `Foo {` is
consumed earlier, by `parse_ident_expr`'s UNGATED uppercase-ident + `LBrace` branch, so the review's
arithmetic (which predicted `-Foo { x: 1 }` breaking) does not fire for the plain case -- that cell
agreed on every binding and is pinned by arms anyway. Second, the dotted family is gate-decided
(`try_postfix`'s `ExprFieldAccess` brace arm), and declaring the row at 13 -- the literal transcription
of the old refusal edge `14 <= min_bp` -- moved the record OUT of the unary operand once the prefix row
moved to 18: the old rule consumed at 0 and at the old prefix 12 but refused at 14 and above, a profile
this consume-iff scale cannot express at any single value. The false comments ("-Foo { x: 1 } keeps
binding its record literal...", in both language specs and the `PostfixOperatorSpec` doc) were written
against 13 and are deleted/rewritten.

DECLARED VALUE: 18 in BOTH languages, per the manager's ruling that both parsers read the one declared
precedence. The v2 grammar is the authority and its record suffix is glued at PRIMARY level
(`dag_grammar_primary_ident_suffix_expr`: optional(call-suffix | record-suffix)) -- the tightest tier,
above prefix and every infix. 18 (>= the pipe's right power 18, below the dot 19 and call/index 21)
reproduces the old arm at EVERY position where the old parser saw text (unary operands absorb, as
before), keeps call/index/cast untouched, and moves the three infix-right positions (additive 14,
multiplicative 16, pipe 18) from v1's historical decline onto v2's always-consume -- where the census
finds ZERO authored sites: 4 regex hits over the .dag tree, one real (`SpanLocation.Unlocated {` at
`00_core.dag:2853`, parsed at min_bp 0, unmoved by the row), three inside strings/comments.
Control-flow braces are untouched: the no-brace loop stops at `{` before ever consulting this row
(`parse_expr_loop_no_brace`), so `match Foo {` / `if Foo.Bar {` (both in the corpus) parse as before.

THE CLAIMS THAT GO RED ON REGRESSION: the agreement claim gains ten record arms (32 total),
expectations MEASURED at the head (invocation jb773f4f1), and the v1 canon lib gains `ExprRecordLit`
and `FieldInit` arms so a record prints `F(F(l))` for one field -- mirroring v2's CONJ-spine lowering
instead of collapsing to a leaf. The regression is the COMBINATION (prefix at 18 with the brace row
left at 13), and it is pinned on three independent channels:

1. BRACE-ONLY PHASE (invocation j504c81eb): with everything else at the declared rows and only the
   `{` row patched 18 -> 13 in both `.dag` specs, the claim fails on EXACTLY the four dotted arms --
   `ARM[-Foo.Bar { x: 1 }] v1=l v2=P(F(F(l))) want=P(F(F(l))) DISAGREE`, the same for `!Foo.Bar`, and
   the two infix cells `a + Foo.Bar` / `a * Foo.Bar` (`v1=l` vs `I(l,F(F(l)))`) -- while every other
   arm, including the plain `-Foo { x: 1 }` / `!Foo { x: 1 }` cells, stays green (the plain family is
   consumed by the ungated primary path, exactly as read). Restoring 18: rc=0 on all 32.
2. FAITHFUL-HISTORICAL PHASE (invocation j8a1e553b): rows 12/12/13/13 in BOTH specs AND BOTH stage0
   mirrors, binaries REBUILT between bindings (take 1, je034bfb6, exposed the trap: the values claim's
   parse and the digest bin use the rows COMPILED INTO THE BINARY from the mirrors, so patching
   without rebuilding re-measures the declared binding -- excluded from the receipts). Ten arms RED:
   the eight banked binding arms (`-1 - 2`, `-a + b`, `-a * b`, `!a * b`, `a + b as T`, `-a |> f`,
   `!a |> f`, `-a.b + c`) plus the two dotted-infix record cells; the two dotted UNARY cells are green
   there, because at the old prefix 12 the brace admits -- the review's regression is precisely the
   pairing, not either row alone.
3. DECODE + DIGEST: `parse_environment_decode_roundtrip` is green at the declared binding (6 passed)
   and FAILS at the historical one (the WireMigration arm carries the declared 21/21/19/18 -- the row
   is pinned inside the seed's own decode tests). The whole-corpus digest at both bindings (178,072
   items / 9,183 files / SKIPPED 1, the same banked headerless fragment) differs ONLY by the banked
   cast items plus the patched spec rows themselves -- and its four-fixture dotted-record control
   corpus DIFFS between bindings (`ctl_dotted_infix.dag` 3b81c84b -> a16544cc), so the zero corpus
   diff cannot come from the digest being blind to this family: the manager's digest-coverage request
   is satisfied with the control proving sensitivity.

## Corpus re-spells (prefix half)

`!x |> any(...)` -> `!(x |> any(...))` at 10 sites in 4 files (`dispatch_authority_acceptance` 52/53, `roster_acceptance` 72/73, `surface_witness` 38/169/170, `05_emit_rust` 3342), pinning current meaning with parentheses BEFORE the precedence commit, so the meaning-preserving edit is reviewable apart from the semantic one. Whole-tree digest at the re-spelled corpus, old vs new prefix binding: **diff EMPTY (0 diff-lines, 0 non-fixture)** - every re-spell kept its meaning. A grep candidate list of 43 lines was 38 already-parenthesized false positives (slice tests are blind to parens), so the tree digest, not the grep, is the census authority. #14490's census found no exposed site at its head; re-checked at main with the digest.

## The skipped file, named (manager ruling: SKIPPED 0 or justify)

**SKIPPED 1 at both bindings**, in both the prefix census (9050 files) and the cast census (9084 files). The skip is `src/v1/stage0/tests/fixtures/fact_cardinality_split_brace.dag`, refused at 1:1 `expected keyword 'module', found keyword 'data'`: it is a headerless fragment that EXISTS to be unparseable - the stage0 extractor test `extract_top_level_decls_captures_split_brace_body` (`src/v1/stage0/src/cli_run.rs`) and the `namespace_step0_canary_v2_witness` exclusion both name that malformed shape as their subject, so making it parse would destroy what it tests. Executed proof that no prefix-operator site is lost with it: the host tokenizer run over it reports **PREFIX-TOKEN-COUNT 0** (BuildBuddy invocation `749890d7-e4bb-47e3-bddf-a464e4d94af0`).

## Evidence (executed; every invocation id from its own runner stream)

- **RED, value channel** (faithful old binding: declared rows 18/19 patched to the historical 12/13 in the COMPILED language mirrors - the lever the seed parser reads - and rebuilt): `unary_prefix_value_arms_hold` refuses with exactly the defect's face - `-1 - 2 = 1 (want -3); -1 + 2 = -3 (want 1); -1 - 2 - 3 = 4 (want -6)` - three failing arms; `!t && f = false` and `!t || t = true` report CORRECT under the old binding (the bang logical arms were never REDs; published as controls); controls `-(1-2)==1`, `-1*2==-2`, `-1<0`, `-3==0-3` hold. BuildBuddy invocation `83b6b361-8dd2-475e-9aba-1e165f8c6c15`.
- **RED, shape channel** (same historical patch applied to the `.dag` language specs - the lever the claim's v1 route reads by EVALUATING `dag_parse_environment` from source; the two levers are independent and a wrong-lever patch leaves a claim green, which is how the earlier five-arm/14-arm drafts were found to be unmeasured): the 22-arm agreement claim refuses with exactly **8 DISAGREE arms** - `-1 - 2`, `-a + b`, `-a * b`, `!a * b` (`P(I(l,l))` vs `I(P(l),l)`), `a + b as T` (`C(I(l,l))` vs `I(l,C(l))`), `-a |> f`, `!a |> f` (`P(M(l))` vs `M(P(l))`), `-a.b + c` (`P(I(F(l),l))` vs `I(P(F(l)),l)`) - and 14 OK (the controls and `-a as T` hold under both bindings). BuildBuddy invocation `01d7e71a-245a-43f8-b058-abd7ba930421`.
- **GREEN battery at the head** (declared rows, no patches): `unary_prefix_value_arms_hold`, `unary_prefix_emitted_text_follows_the_tree`, the 32-arm `unary_prefix_v1_v2_agreement_reports`, `lex_token_class_membership_readers_hold` (#14559's claim), `v1_interpreter_primitive_roster_acceptance_contract_holds`, `v1_interpreter_primitive_dispatch_authority_acceptance_contract_holds`, `derived_surface_excludes_deleted_bridge_arm` - all rc=0 at head `8bdb00b48e6` — final battery BuildBuddy invocation `a2bd58e9-51db-4704-9e45-05afeab2b679`; the decode roundtrip suite runs 6/6 at the final head (receipt in the next bullet).
- **Emitted-Rust channel**: four-pole substring oracle over `compile_dag_rust_emit_check`, poles CALIBRATED BY OBSERVATION (windowed dump, BuildBuddy invocation `1568cdbd-614c-4486-8b2a-feee235f82f1`): Int operators emit runtime helpers, not literal `-`/`(` - the fixed triple emits `v1_rt::int_sub(v1_rt::int_sub(v1_rt::int_neg(1), 2), 3)` (neg on the 1); the bug fixture emits `v1_rt::int_neg(v1_rt::int_sub(v1_rt::int_sub(1, 2), 3))` (neg on the row); the bang poles likewise observed. Non-vacuity is executed: each bug-shaped fixture is emitted and matched, so the refuse-strings are real emissions.
- **Mirror regeneration, by the round, not by heal** (BuildBuddy invocation `b32bc874-796d-4941-b64d-81e561173864`): after merging main, the mirrors were regenerated over the merged `.dag` sources - `claim_executor --regen-round-cost` + heal + `seed_build_artifacts_regen`, both rounds rc=0 on all three commands, **round 2 wrote 0 files** (fixed point; a refused round is a hard failure in the lane, never a "fixed point"). Witnesses in the regenerated mirror: `find_prefix_binding_power` 3 reads, `declared_postfix_gate_admits` 8 reads, literal gates (`21 < min_bp`, `21 >= min_bp`, `13 < min_bp`, `14 <= min_bp`, `min_bp: 12`) 0, `PostfixOperatorSpec` 4 rows per language. Bootstrap note: the merge took MAIN'S mirrors as the round's STARTING binary (my pre-merge mirror could not parse main's corpus), and the round's OUTPUT is what lands - main's copy is not taken.
- **Decode suite at the final head**: the loader test initially refused with an unbalanced-initializer panic — its declaration strip counted the bracket inside the row literal `symbol: "["` as nesting, a defect of the TEST, not of the migration (the wire-level cases passed throughout). The strip now skips string bodies and comments; both declarations close, measured over the real module (dropped block ends at the row list's own `]`, next declaration follows). Suite 6/6 — BuildBuddy invocation `6c912323-50fd-4734-9688-11dd28556240`.
- **RECORD-CELL receipts (review 84349)**, three independent channels:
  - *Brace-only phase* (invocation `j504c81eb`): everything else at the declared rows, only the `{` row patched 18 -> 13 in both `.dag` specs -> the claim fails on EXACTLY the four dotted arms (`-Foo.Bar { x: 1 }`, `!Foo.Bar { x: 1 }`: `v1=l` vs `v2=P(F(F(l)))`; `a + Foo.Bar { x: 1 }`, `a * Foo.Bar { x: 1 }`: `v1=l` vs `I(l,F(F(l)))`), all other 28 arms green -- including the plain `-Foo`/`!Foo` cells (ungated primary path, as read). Restoring 18: rc=0 on all 32.
  - *Faithful-historical phase* (invocation `j8a1e553b`): 12/12/13/13 in BOTH specs AND BOTH stage0 mirrors, binaries REBUILT between bindings. Ten arms RED: the eight banked binding arms plus the two dotted-infix record cells; the two dotted UNARY cells are green there because at the old prefix 12 the brace admits -- the regression is the PAIRING (prefix 18 with the brace at 13), not either row alone. This phase also exposed a measurement trap (take 1, `je034bfb6`): the values claim's parse and the digest bin read the rows COMPILED INTO THE BINARY from the mirrors, so a mirror-only or `.dag`-only patch re-measures the declared binding -- the take-1 `values_old rc=0` and empty control diff were voided, not reported.
  - *Canon + digest*: v1 canon lib gains `ExprRecordLit`/`FieldInit` arms so a record prints `F(F(l))` for one field, mirroring v2's CONJ-spine lowering (strings MEASURED, invocation `jb773f4f1`; the shape probes `jd1752ffb` printed the tree-level split). The seed decode suite is green at the declared binding (6 passed) and FAILS at the historical one -- the row is pinned inside the seed's own decode tests. The whole-corpus digest at both bindings (178,072 items / 9,183 files / SKIPPED 1) differs only by the banked cast items plus the patched spec rows themselves, and its four-fixture dotted-record CONTROL corpus diffs between bindings (`ctl_dotted_infix.dag` 3b81c84b -> a16544cc), so the zero corpus diff cannot come from the digest being blind to this family (the manager's digest-coverage request, closed with the control that proves sensitivity).
- **Mirror regeneration at the record head** (invocation `j80c49fb5`): baseline mirror-dirty=0; round 1 wrote exactly the two extdeps syntax mirrors (`{` 13 -> 18 propagation); round 2 wrote 0 (fixed point). Witnesses: `find_prefix_binding_power` 3, `declared_postfix_gate_admits` 8, literal gates 0, rust rows 4.
- Probes, the digest bin, the census tool and the lane scripts stay OFF this branch; they are on the WIP branch `session/nimble-otter-791` and re-runnable from the lane scripts there.

## Historical-environment decode (baseline migration)

`SyntaxSpec` gained two required fields, so a revision older than this commit encodes an environment without them and `decode_environment_value` would refuse it (`ValueNotDecodable` -> baseline read as `NotEvaluated`). `src/v1/stage0/src/namespace_baseline.rs` now migrates: `operator_row_authority(ctx, revision)` asks the EVALUATION CONTEXT - built from the revision's own materialized tree - whether its `std.syntax` record `SyntaxSpec` declares the rows. A revision that declares BOTH decodes exactly as encoded (a missing field then REFUSES - a loud disagreement with the emitter, never a fill); a revision that declares NEITHER gets the `historical_operator_rows` (prefix 12/12; postfix 21/21/13/13 - the literals THAT revision's parser read, preserving its parse decisions under this binary's table-driven parser). A tree declaring exactly ONE field refuses with a stated cause (both fields were declared by one commit; such a tree is a defect, not a revision). The decision is made from the revision's SOURCE, never from which keys appear in the bytes - a shape-detected fill would silently restore the old powers for a current revision whose encoder lost a field, which is the silent-wrongness class this PR exists to close. Tests in `src/v1/stage0/tests/parse_environment_decode_roundtrip.rs`: a real pre-declaration revision (scratch repo, both files edited back, committed) through `load_parse_environment_at` - the base/head loader's own route - decodes with the historical rows and its declared rows untouched; a field-less wire under the Declared authority REFUSES naming the missing field; a field-less wire under PredatesDeclaration fills; a current document is never touched. The five new hand-Rust items are declared rows in `dag/gunbc/namespace_baseline_seed_growth.dag` (DESIGN 7). SEED-GROWTH CENSUS as the checkable receipt for the expansion (review 84469): `git show origin/main:dag/gunbc/namespace_baseline_seed_growth.dag | grep -c "DeclarationRef {"` = **33** on main vs **38** at this head -- the five added rows are `OperatorRowAuthority`, `operator_row_authority`, `historical_operator_rows`, `fill_historical_operator_rows`, `decode_environment_wire`, each naming its reason and migration trigger; the diff they account for is `+155` lines in `namespace_baseline.rs` and `+333` in `parse_environment_decode_roundtrip.rs` (the test file is not seed-retained, so it needs no row, but the line counts are given so the growth is checkable rather than asserted). No scaffold was deleted (none existed for this path -- the decoder is new, not a replacement), so the census is the receipt.

## Remaining debt (named, with its trigger)

Two operator tables still exist - v1 reads `SyntaxSpec` (scale 5..21), v2 reads its grammar rows (scale 1..7). v2's rows are the older authority for its closed productions; v1's rows were derived to reproduce the meanings those productions have always produced. The executed 32-arm agreement claim (22 binding + 10 record-literal arms) is the guard; the trigger to unify is the first operator added to one table but not the other, which starts failing that claim and names the RFM row `gunbc.recurring_failure_mode.unary_minus_absorbs_a_following_additive_operator`.

## Out of scope (stated, not skipped)

The `0 - 1 - k` / parenthesized spellings #14490 and this PR added to dodge the old binding may now be simplified - deliberately a LATER PR, per the manager's ruling, so this diff stays reviewable. v2's rows are not re-plumbed here (the ruling: agreement, not re-plumbing; the debt row carries the trigger).
