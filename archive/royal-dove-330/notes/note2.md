# Change 1 (push-route receipt reuse): source prepared, one correction to the brief, one decision needed

Session royal-dove-330, node://adhoc-189dd15f-1a8. Read-only analysis + LOCAL SOURCE preparation only.

## 0. What has and has not executed

NOTHING was built, compiled, claimed, regened, run, pushed or dispatched by me: no cargo, rustc, rustup,
ctrl-build, local or remote. All "compile"/"duration" evidence is existing CI receipts read through
`gh run view` / `gh api` / committed YAML. The claim controls I added are UNEXECUTED, and CI does not
execute v1 claim tests at all, so their RED/GREEN status below is an argument, not a result.
Source is committed locally on `session/royal-dove-330` at `4f68bf150a3` (base `ed007978dc2a`), NOT pushed.
The emitted `.github/workflows/witnesses.yml` is NOT regenerated (that is a build workload and yours to
authorize). Patch bytes at the end.

## 1. CORRECTION: the three named .dag files cannot carry Change 1

The brief (and my first plan) scoped Change 1 to `parse_verdict_restore.dag`, `parse_verdict_publish.dag`
and `witness/compiler_gate_workflow.dag`. That is not sufficient, and shipping it would have turned main
RED rather than fast:

- `src/v2/cli/compile_cli.dag` (was `parse_population_reusing`, ~line 1040) REFUSED the combination
  outright: `--reuse-from` together with `--receipts` or `--write-identity` returned
  `held: false, detail: "--reuse-from does not combine with --write-identity or --receipts"`. So a
  workflow-only Change 1 makes the required step `Walk parse-population on default-branch push` refuse,
  exit non-zero, and fail the job. There was no claim control asserting that refusal (I grepped; only the
  two argument-shape refusals are controlled), which is why the boundary survived un-noticed.
- Two of the files named need no edit at all: `witness/compiler_gate_workflow.dag` already composes the
  restore-bound steps BEFORE both walks inside `compiler_gate_emit_build_tail_bound_steps()` and every step
  carries its own `if_condition`; `parse_verdict_publish.dag` needed only the new read guard and the extra
  argument. The real work is in 4 source files + 2 claim files (list in §5).

The refusal's own stated reason was true of the artifact and false as a law — that is §4.

## 2. The cost chain

Per v2-bootstrap job, from the receipts in the brief plus what I read in source:

| stage | cost | why |
|---|---|---|
| compiler + v1 harness | ~11 min | unrelated to this change |
| emit self-host, emit native-cli | 26m12 + 28m17 (run 42339827972) | each re-runs frontend/normalize/**reconcile 7-8min**/analyses/**emit ~4min** over an identical `source_tree=`; sharing is deliberately off (`GUNBC_NATIVE_PRODUCT_REUSE=fresh`, `RUSTC_WRAPPER=""`) |
| restore 12 ancestor identities | ~6 s | actions/cache, PR route only |
| parse-population walk, PR route | **5-16 s on a hit**, 1366 s on a miss | per member: read + content-hash, consult standing, parse ONLY on miss |
| parse-population walk, push route | **913-1462 s, every run** | no read path at all: all 24 restore steps `SKIPPED`, reuse walk `SKIPPED`, zero occurrences of `--reuse-from` in run 42292182286 |

Three facts that set the size of the prize:

1. The walk is BINARY, not proportional. Hits: `reused=10101 miss_digest_unseen=4` in 6 s
   (42362849618), `reused=10102 miss=1` in 6 s, `reused=10087 miss=16` in 16 s. The one measured miss
   (42339827972) was `reused=0 miss_identity_mismatch=10094` in 1366 s. Reused members are still read and
   content-hashed first, and that whole 10,106-file read+hash is ~5 s, so parsing is ~99.6% of a walk.
2. The push route's 15-24 min is therefore a single missing read, not a workload property.
3. The job is BIMODAL for reasons outside this change: 7 main pushes today give a fast mode of 63.0 min
   (n=2) and a slow mode of 98.3 min (n=5), with every step 1.42-1.70x slower in slow mode. 13 concurrent
   PR witness runs were on the same self-hosted arm64 pool at 11:20. Contention is a hypothesis I have not
   tested; it matters because it decides whether 90 minutes has any margin at all.

## 3. The loop in the brief: real, but not currently armed

Confirmed from source: `parse_verdict_publish_walk_step` and both save steps are gated on
`materialization_publish_if()` = push AND `refs/heads/main`, so only main pushes publish, and a run
cancelled during its walk publishes nothing (the loop silent-koi-402 named). But since `49d4bc36` restored
the 90-minute ceiling, main pushes have completed and published again — 42362849618 and 42362837474
restored 10,101-10,102 identities from main entries seconds later. So the self-reinforcing state was a
consequence of running over budget, not a standing condition. Fixing the budget dissolves it; nothing needs
to be done about the loop itself, and no cache change is warranted (DESIGN 2).

## 4. Earliest unjustified boundary (DESIGN 6b)

`src/v2/cli/population_verbs.dag`, `population_parse_report` (was lines 513-519):

    text: match tally.reuse.base {
      Absent            => join(list_reverse(xs: tally.accepted_receipts), "\n")   // the ARTIFACT
      Present { value: _ } => parse_verdict_reuse_counts_text(counts: ...)          // the LOG LINE
    }

One `String` field serves two different products, chosen by whether a base was read. Everything downstream
inherits that: the writer wrote `report.text`, so a walk that had read a base would have written its counts
summary into the receipts file — hence the CLI refusal, whose wording ("writing receipts from a reuse walk
would store counts as a verdict") is exactly this coupling stated as a rule.

The payload does not actually depend on the route. `population_verbs.dag:437` routes a `Reused` member
through the SAME `population_parse_accepted` a parsed member goes through, and that function builds the
line from `parse_verdict_stored_accepted_line(source_digest, compiler)` — the identical construction the
standing rule looks up (`gunbc/parse_verdict_schema.dag:96`: Reused iff `i_base == i_head` AND the base's
receipt set contains that exact line). Same bytes, same parse compiler, same line. So a *reporting*
limitation was being enforced as a *semantic* prohibition, and it was costing the default branch 15-24
minutes per push. That is the earliest link that is wrong without anything downstream being able to fix it.

## 5. Smallest change, with exact source scope

One idea: give the walk two products instead of one, and let the default branch read what the pull request
already reads. 6 files, +329/-83.

`dag/gunbc/parse_verdict_publish.dag`
- +`LogicalOr` import; +`cli_option_reuse_from` import.
- NEW `materialization_restore_guard()` / `materialization_restore_if()`: the routes that may READ, derived
  from the cache model's own `gha_cache_entry_restorable_by` (extdeps/cache/github_actions.dag: saved on the
  default branch is restorable by anyone; saved by a pull request is restorable by pull requests only). A
  push therefore reads what pushes saved and can never read what a PR saved.
- `parse_verdict_publish_args()`: +`--reuse-from parse_verdict_cache_working_directory`, the SAME data row
  the pull-request walk reads, so the routes cannot disagree about where a base lives.

`dag/gunbc/parse_verdict_restore.dag`
- `parse_verdict_restore_if()` now returns `materialization_restore_if()`; it gates only reads
  (locate, 12 identity restores, 12 receipts restores, record).
- NEW `parse_verdict_reuse_walk_if()` = the pull-request scope guard alone; `parse_verdict_reuse_walk_step()`
  now uses it. Without this a push would fire BOTH walks and pay for the population twice.
- `parse_verdict_identity_restore_condition` and `parse_verdict_receipts_restore_condition`: fold seed /
  left operand changed from the pull-request scope guard to `materialization_restore_guard()`.
- NEW `parse_verdict_ancestor_base_sha_expr()` replaces `pull_request_base_sha_expr()` at the locator's
  `BASE_SHA`: pull request -> `github.event.pull_request.base.sha`; push -> `github.event.before`. Both
  EXCLUDE the commit under verification — on the push route the head is excluded by construction rather
  than by arithmetic, which is the guard keen-otter asked for. Unresolvable `before` (new branch,
  all-zeros sha) -> typed miss -> full walk, never a pass.
- Step name `Locate first-parent ancestors of base.sha` -> `... below the commit under verification`
  (single `data`, referenced only by the step and the capability list; no test binds the string).

`src/v2/cli/population_verbs.dag`
- NEW `type PopulationVerdictProducts { report, receipts }`, NEW `population_parse_receipts_text(tally)`
  (the payload spelled ONCE — `population_parse_report` renders it, the writer writes the same derivation),
  NEW `population_verdict_products(tally)`; `parse_population_listed / _prepared / parse_population` return
  the pair. `receipts` is `Absent`, not empty, when no compiler was named.

`src/v2/cli/compile_cli.dag`
- `parse_population_written` absorbs the two former functions and the refusal: it writes the identity blob
  from `--compiler`, writes the receipts from `products.receipts`, and returns `products.report`. A held
  walk with no receipts payload now REFUSES instead of writing an empty artifact. The two argument-shape
  refusals are preserved verbatim. NEW `parse_verdict_reuse_for(reuse_from)` builds the read.

`dag/test/claim/parse_verdict_restore_witness.dag`, `src/v2/test/claim/cli/population_verbs.dag`: §9.

Preserved, deliberately untouched: the required validation population (10,106 bound modules, enumerated
from the head tree every run), roster/duplicate/declared-refusal cross-checks, the per-member digest
recomputation, the exact-digest + identity standing rule, `held` semantics, the identity gate (I verified
the 42339827972 mismatch was a legitimate invalidation: of the 96-module parse closure, exactly one file,
`dag/std/algebra.dag`, was in the real diff — do NOT weaken this), the save-step gating, and no new CI jobs
or steps.

## 6. Expected runtime, and how it was derived

Components are the five measured step durations of today's seven completed main pushes (seconds,
`cc / harness / self-host / native-cli / walk`), read from `gh api .../jobs`:

    fast  49d4bc36  226  186 1085 1274  913  = 3684 s (61.4 min)
    fast  28a32a38  237  190 1231 1191 1027  = 3876 s (64.6 min)
    slow  bc229924  392  317 1755 1977 1453  = 5894 s (98.2 min)
    slow  b1df04d5  363  324 1877 1774 1462  = 5800 s (96.7 min)
    slow  700fe500  376  315 1933 1983 1328  = 5935 s (98.9 min)   (f55b403a, 64eec001 similar)

Two things fall out. Mean pre-walk work is 4606 s (76.8 min) in slow mode and 2810 s (46.8 min) in fast
mode; and every one of those seven pushes paid a FULL walk (913-1462 s) — including the two fast ones,
which is the direct signature of a route with no read path.

Change 1 turns the push walk into the hit shape (~5-16 s measured on the pull-request route) and adds the
newly enabled read steps to the push route (locate + record + 24 cache restores, budgeted generously at
~25 s from the pull-request route's ~6 s for 12 restores):

- slow: 4606 + 35 = **~4641 s = 77.4 min**, about 12 min under the ceiling.
- fast: 2810 + 35 = **~2845 s = 47.4 min**.

It is NOT sufficient when the walk misses. A miss costs 913-1462 s instead of ~10 s, so slow mode lands at
4606 + 1462 = **101 min** and fast at 46.8 + 15.2 = **62 min**: the ceiling is breached exactly when slow
mode and a miss coincide. A miss is not rare — it is every push that changes the parse-compiler closure,
every cache eviction — and the restores carry an exact key with no restore-keys (asserted by the existing
control), so an evicted entry is a hard miss with no fallback.

For that case the duplicated work that matters is the emit prefix, not the walk: `reconcile` (7-8 min) and
`emit` (~4 min) run twice over an identical `source_tree=` in the two emit steps, ~1150-1900 s of the
pre-walk total. Removing it would put a slow-mode miss at ~83 min. I have NOT designed that change and its
estimate is unverified; `GUNBC_NATIVE_PRODUCT_REUSE=fresh` and the emptied `RUSTC_WRAPPER` are deliberate, so
"just enable sharing" is not the answer and it needs its own boundary analysis.

Corroboration that landing this does not itself force a miss: none of the 6 files is in the parse-compiler
import closure (my static resolver: 95 modules; it reproduces the one in-closure file the brief names,
`dag/std/algebra.dag`), so the merge's own identity should equal its ancestor's and the first push should
hit. The authority for that is the identity comparison at run time, not my resolver — see item 1 of §10.

## 7. Expected generated diff to `.github/workflows/witnesses.yml` — EXPECTED, NOT GENERATOR OUTPUT

Derived by reading the emitting functions, not by running the generator (running it is a build workload).
One file, ~30 lines, no step added, removed or reordered:

1. line 25 `GUNBC_COMPILER_GATE_CONTRACT_STAMP: e57b498c8813770c` -> a new 16-hex value. The stamp folds
   `[reuse_args, publish_args]` (compiler_gate_contract_stamp.dag:159-163), so adding `--reuse-from` to the
   publishing args changes it. This is the hard coupling: the pre-build guard compares the running workflow's
   env value against the tree's committed copy and refuses `StampMismatch` / `StampAbsent` otherwise
   (witnesses.yml:74-77), so this change CANNOT merge unregenerated.
2. line 281 step name -> `Locate first-parent ancestors below the commit under verification`.
3. line 310 `BASE_SHA: ${{ github.event.pull_request.base.sha }}` ->
   `${{ github.event_name == 'pull_request' && github.event.pull_request.base.sha || github.event.before }}`.
4. 26 `if:` lines widen — the locate step, the record step, and all 24 ancestor restores — from
   `github.event_name == 'pull_request'` to the two-route read guard, parenthesised where it is nested under
   an `&&` (the serializer is precedence-aware: extdeps/github/expressions.dag `serialize_operand_with`), and
   unparenthesised at top level, where `&&` out-binds `||` and the meaning is unchanged.
5. line 480 `if: github.event_name == 'pull_request'` on the reuse walk: UNCHANGED (same serialized text).
6. the publishing walk's `run:` gains ` --reuse-from .` between `--compiler <expr>` and
   `--write-identity`.

Nothing else in the repo's emitted YAML moves: the stamp literal is embedded only by
`compiler_gate_workflow.dag` (no other workflow imports `compiler_gate_contract_stamp_yaml_value`); the
parse-verdict restore/publish step builders are consumed only inside that authority; and the qualification
floor takes `prelude + subject` only and explicitly excludes the tail and both stamp steps
(compiler_gate_workflow.dag:589-612), so `runner_qualification_floor_job`, the fleet, heal, dispatch and
witness-floor workflows are untouched. No step ordering changes.

## 8. Generation proposal — the decision that is yours, keen-otter

Route issue, exactly as silent-bee-845 framed it: `main_wet_one` at main's seed refuses on the
`dag/gunbc/host/host_effect.dag:248` equality wall, and #16019's YAML was generated on srv1 at #15992's
tree (an alternate seed). Proposal:

- generator entry: `gunbc.witness.compiler_gate_workflow`, emitted through the ordinary wet-one effect that
  writes `.github/workflows/witnesses.yml` — not a hand edit, and not a targeted single-file emitter if one
  exists only for srv1-style alternate seeds.
- subject tree: `4f68bf150a3` (or its squash commit once you fix the base), containing all 6 source files.
- seed/effect: whatever main's own seed admits. I have no grant for any host and am not proposing one: the
  srv1 alternate-seed route named in the brief was ANOTHER SESSION'S REPORTED ACTION, not an authorization
  from keen-otter, and it is neither precedent nor authority. It is removed from this proposal.
- LIMIT, STATED PLAINLY: an exact predicted diff and a passing contract-stamp guard do NOT establish
  regeneration correctness or generator provenance. The predicted diff is my reading of the emitting
  functions; the stamp only proves the committed YAML and the running source agree on a hash, which a stale
  pair can also satisfy. Provenance needs the recorded seed, binary, command and effect below, reviewed
  before any run.
- bounds: ONE regeneration round. If a second round would repair something the first left stale, that is
  `a_required_regen_round_repairs_the_tree_back_to_its_own_stale_emitter` and should stop for review.
- verification of the generation itself, in this order: (a) `git diff` touches only
  `.github/workflows/witnesses.yml`; (b) the diff matches §7 item for item, including the stamp value change;
  (c) the committed-YAML claim tests that read the file pass (`the_committed_witnesses_yml_binds_the_sha_identity_save`,
  `the_committed_witnesses_yml_publish_walk_names_the_digest_authority`, and the stamp guard's own tests in
  compiler_gate_contract_stamp_witness.dag); (d) one PR run, which must stay green, then one main push (§10).
- I will not run any of this, and cannot: it is a build workload.

## 9. Witness / claim changes — UNEXECUTED

`dag/test/claim/parse_verdict_restore_witness.dag`
- renamed and strengthened `the_base_sha_entries_are_restored_exactly_on_pull_request_only` ->
  `..._on_the_two_read_routes`: now asserts the read guard names BOTH routes and is still not the
  publisher's. Nothing was dropped; the old assertions are kept. (This test's old name and comment became
  false under the change, so leaving it would have been the silent-green kind of stale.)
- NEW `one_run_fires_exactly_one_walk_and_the_read_guard_is_not_a_walk_guard`: the reuse walk's condition is
  the pull-request guard, is NOT the widened read guard, does not name `'push'`, and the reuse command names
  neither `--receipts` nor `--write-identity`; the publisher names `'push'` and not `pull_request`. This is
  the control for the regression I most feared — widening the walk gate and paying for the population twice.
- NEW `the_ancestor_start_excludes_the_commit_under_verification`: names base.sha and event.before, and must
  NOT contain `github.sha`.
- NEW `the_publishing_walk_reuses_the_same_base_the_pull_request_walk_reads`: both commands carry the
  identical ` --reuse-from <dir>` spelling, and the publisher still carries `--receipts` + `--write-identity`.

`src/v2/test/claim/cli/population_verbs.dag`
- NEW `reading_a_base_changes_the_report_line_and_not_the_published_payload`: two tallies with the same
  verdicts, one reading a base — `receipts` byte-equal to the cold payload and to
  `held_parse_report_text()`, while `report.text` moves to the counts line. This is
  `gunbc.recurring_failure_mode.cache_impurity`'s own oracle ("byte-identical cached-vs-cold is the purity
  oracle") applied to the artifact this change starts publishing.
- NEW `a_walk_with_no_compiler_offers_no_receipts_payload`: `Absent`, so the writer refuses rather than
  publishing a zero-verdict base that a later run would read as complete.

Expected RED before the source change, GREEN after: the two route controls and the same-base control (the
publisher had no `--reuse-from`); the payload test (products did not exist); the walk-exclusivity control
fails differently before — before the change `parse_verdict_reuse_walk_if` does not exist, so it is a
compile-level RED. Nothing here is a result until someone runs `gunbc test`, which CI does not do.

## 10. Evidence needed to validate

1. One main push after landing, log read for: the new counts line on the publishing walk
   (`reused=… miss_…=`), its duration in SECONDS not minutes, and `reused > 0`. A `reused=0` with a fast
   step would mean the read silently did not happen — that is the failure mode of §7 item 3/6 and the single
   most important thing to look at.
2. Purity, the decisive one: the receipts published by a reusing push, byte-compared against the receipts a
   full walk publishes for the same tree. The two runs are naturally available (a hit push vs a
   closure-changing push), and both artifacts are in the Actions cache under their own keys. Equality is the
   claim; inequality means reuse changed what is published and the change comes out.
3. The identity gate still firing: a closure-changing push must log `miss_identity_mismatch` near the
   population size, as 42339827972 did.
4. The push route's restore steps actually hitting: `Restore parse-verdict receipts for base ancestor N`
   reporting a hit at some N (today they are all SKIPPED on push).
5. The contract-stamp guard passing, which is the only proof the regeneration matched the source.
6. Job duration across >=5 pushes in each mode, because the fast/slow bimodality (63 vs 98 min) dominates
   the margin. If slow mode alone exceeds 90 min after this, Change 1 has bought the walk but not the budget,
   and the contention hypothesis (§2) becomes the next thing to measure.
7. The new claim controls, executed by whoever is authorised to run v1 claims.

## 11. Decisions I need

D1. Authorization for the CLI layer. §5's `population_verbs.dag` + `compile_cli.dag` edits remove a
documented refusal. They are outside the "three .dag files" I was authorized for, and Change 1 does not
exist without them — but removing a refusal is an acceptance-semantics change and I am not treating my own
reasoning as approval.
D2. Whether to land Change 1 alone. Alone it reaches ~77.4 min slow / ~47.4 min fast on a HIT, and is still RED
on a miss. If you want the ceiling honoured unconditionally, Change 1 and the emit-prefix change have to be
one reviewed unit; if you accept "red when the parse compiler changed", land Change 1 now and take the miss
case knowingly.
D3. One invariant my change newly depends on, which you may want tightened before merge: the base is read
from the same two paths the publishing walk later overwrites (`gunbc-parse-verdict-receipts.tsv`,
`gunbc-parse-compiler-identity`). No ancestor artifact can be saved under the head's key today because the
save steps have no `continue-on-error` and only run after a successful walk — I checked the committed YAML
(0 occurrences across the whole parse block). The stricter design separates input and output paths in
`gunbc/parse_verdict_schema.dag`, at the cost of renaming published cache artifacts. Your call; I did not
expand the patch to cover it.
D4. `parsed` counts a reused member (`population_parse_accepted` increments it for both routes), so the
"something was parsed" floor is satisfied by reused members. Existing controls cover the digest and identity
gates and the counts line distinguishes the two, but I am naming it rather than letting it pass as obvious.

Patch bytes follow.


## 12. What CI would and would not cover (static import closure, computed twice)

Roots: `v2.cli.compile_cli` (229 modules) and `v2.compiler.source_root_eval` (244 modules), the two emitted
closures the v2-bootstrap job builds.

| file | in a closure? | CI coverage of this patch |
|---|---|---|
| `src/v2/cli/compile_cli.dag` | IN `v2.cli.compile_cli` | **typechecked** by the emit+build step |
| `src/v2/cli/population_verbs.dag` | IN `v2.cli.compile_cli` | **typechecked** by the emit+build step |
| `dag/gunbc/parse_verdict_restore.dag` | out of both | parsed as a bound module by the walk; NOT typechecked |
| `dag/gunbc/parse_verdict_publish.dag` | out of both | same |
| `src/v2/test/claim/cli/population_verbs.dag` | out of both | **no CI coverage at all** |
| `dag/test/claim/parse_verdict_restore_witness.dag` | out of both | **no CI coverage at all** |

Consequences, stated plainly: CI would catch a type error in the CLI half of this patch (the half that
removes the refusal), which is the half where a mistake matters most. CI would NOT typecheck the two gate
files, and would not compile or run any control I added. A missing symbol in a control is therefore
**NO_OBSERVATION, not a RED** — I make no RED claim from compilation failure anywhere. (Whether the walk's
per-member parse resolves imports is not something I verified; I claim only "not typechecked by the emit
steps", as a review claim.)

## 13. Against keen-otter's acceptance list

(1) Purity. Accepted: a hit push versus a closure-changing push is NOT the control, and my first payload
test proved product rendering only — it is now labelled exactly that. Added: `a_reused_member_publishes_the_
line_a_parsed_member_would`, `a_mismatched_base_still_publishes_only_what_this_walk_verified`,
`a_mixed_walk_publishes_the_same_lines_in_the_same_order` (ordering is asserted, because the file IS the
artifact), `a_base_without_the_member_line_misses_it_and_publishes_its_own`,
`the_named_ancestor_changes_nothing_about_standing`, plus the writer's Absent control. Each names its
discriminatory mutation in its comment, and each asserts the route it claims (standing actually Reused /
actually Miss*) rather than assuming it. THE SAME-SUBJECT-TREE COLD-vs-REUSED COMPARISON IS STILL NOT
PRODUCED: a claim test cannot open a filesystem here (the file's fixtures are tallies, not an
`AcquisitionEnv`), so the real one-tree/two-runs comparison remains evidence item 2 of §10 and cannot be
made by me at all without a build.
(2) `event.before`. Accepted that it is not necessarily an ancestor. Traced: `gunbc/parse_verdict_schema.dag:81
parse_verdict_standing` decides standing from the head identity, the base identity, and the line built from
the digest THIS walk recomputes — it never reads the ancestor, and the ancestor appears only in the cache
key and the counts line. Controlled by `the_named_ancestor_changes_nothing_about_standing`. I therefore
NARROWED the claim (gate comment + witness comment) instead of enforcing the relation, and named the
enforcement I deliberately did not add (`git merge-base --is-ancestor` via `bash_build_if_from_stmts`) as
D5 below. Zero/unresolvable `before` -> all ordinals empty -> typed miss -> full walk; missing identity or
receipts blob -> `MissIdentityAbsent`/`MissEntryAbsent` (existing controls); malformed-but-well-formed line
set lacking the member -> `MissDigestUnseen` (new control). Merge/batched pushes and `before == sha` are
named in the gate comment as hit-rate and log-honesty residuals, NOT as soundness — labelled review claims.
(3) D3, separate paths. Traced all readers/writers: the two names are `gunbc-parse-compiler-identity` and
`gunbc-parse-verdict-receipts.tsv` (schema.dag:108-109), written by the restore steps (restore.dag:291,301)
and by the CLI's `--write-identity`/`--receipts`, read by the save steps (publish.dag:250,262) and by
`parse_verdict_base_read` via `parse_verdict_restored_path(dir, basename)`. CONCLUSION: the input and output
paths cannot be separated with one save/restore pair, because actions/cache archives and restores by
workspace-relative path — the saved `path:` must equal the restored `path:`, or the entry restores nothing,
and archiving both names would hand the next run its ancestor's copy instead of this commit's verified
output. Separate names would need a copy step (a new emitted shell node) or a second cache entry: both are
wider than this patch and neither preserves the external artifact names for free. Failure analysis: a failed
identity or receipts write returns `held: false` and a non-zero exit, and no save step has
`continue-on-error` (0 occurrences across the whole parse block of the committed YAML), so no save can run
without both artifacts freshly written; a save that dies between the receipts and identity saves leaves a
receipts entry with no identity entry, which reads back as `MissIdentityAbsent` for every member -> full
walk, never a pass; retry residue is cleared because the lane checkout sets `fetch-depth: 0` and does not
disable actions/checkout's default workspace clean. I did NOT expand the schema on this. If you want strict
separation anyway, it is a schema change plus a copy step and I want it reviewed as such.
(4) D4, the floor. Corrected, and my §11 D4 was wrong in its worry: the publishing route's floor is
`is_empty(tally.accepted_receipts)` — the ACCEPTED/VALIDATED population — not `tally.parsed`. The
`parsed == 0` floor belongs to the no-receipts route, which cannot reuse at all (`--reuse-from` requires
`--compiler`). So the floor's intended requirement is not weakened by this patch, and the "reused members
satisfy the floor" case is pre-existing behaviour already measured on the pull-request route (42362849618:
`reused=10101`, held). The counts line keeps them distinguishable: `reused=` versus the five miss kinds,
which together are the physically-parsed count.
(5) Callers. `parse_population` has exactly three call sites, all inside `parse_population_written` (all use
`.report` or `.receipts`); `parse_population_listed`/`_prepared` have no caller outside `population_verbs`
(only a comment mention); the claim file's `verdict()` helper calls `population_parse_report` directly and is
unaffected, which is why the return-type change needed no test-side signature edit. The overstated comment
about an empty receipts set is corrected in place: standing is a digest-built line LOOKUP
(`parse_verdict_standing`), so an empty set is an honest all-miss, and `Absent` guards the different and
worse case of a run reporting itself held while publishing no payload.

D5 (new, mine): whether to add the `git merge-base --is-ancestor` guard to the locator. I recommend NOT
adding it in this patch for the reason in (2), and want that recorded rather than buried.
