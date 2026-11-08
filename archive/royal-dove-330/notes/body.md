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
- seed/effect: whatever main's own seed admits. If the equality wall at host_effect.dag:248 still refuses at
  that tree, the fallback you already used for #16019 — generate on srv1 at a tree that is main plus this
  commit — is acceptable to me ONLY if the resulting diff is exactly §7 and nothing else; a generation that
  moves other members is evidence the seed is wrong, not that the output is fine.
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
D2. Whether to land Change 1 alone. Alone it reaches ~78 min slow / ~41 min fast on a HIT, and is still RED
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


--- BEGIN PATCH (git format-patch, applies to ed007978dc2a67096df619b1b91566458dfb2ffd) ---

From 4f68bf150a3384711c1824c0ef62dc3559d16350 Mon Sep 17 00:00:00 2001
From: royal-dove-330 <royal-dove-330@local>
Date: Sat, 7 Nov 2026 11:58:35 +0000
Subject: [PATCH] v2-bootstrap: let the default-branch walk stand on its
 ancestor's verified members

PREPARATION ONLY - not compiled, not run, no PR. The emitted
.github/workflows/witnesses.yml is NOT regenerated here; see the design
note for the generation decision that must accompany this.

The push route re-parsed all 10,106 bound modules every run (913-1462 s
observed) while the pull-request route re-parsed only misses (5-16 s),
because ancestor receipts were readable on one route only and the CLI
refused to combine a read base with publishing. Both were artifacts:

- gunbc.parse_verdict_restore gated the READ on the pull-request cache
  scope; the default-branch push now reads under materialization_restore_guard,
  and each walk keeps its own mutually exclusive guard so no run holds two.
- The locator starts below the commit under verification on both routes
  (the base; the commit main held before this push), so a run can never
  reuse receipts it has not produced.
- v2.cli.population_verbs .PopulationVerdictProducts carries the report
  line and the published receipts separately, so a walk that read a base
  publishes the lines it verified itself and still prints the counts.

Refs node://adhoc-189dd15f-1a8.
---
 dag/gunbc/parse_verdict_publish.dag           |  36 +++++-
 dag/gunbc/parse_verdict_restore.dag           |  72 ++++++++---
 .../claim/parse_verdict_restore_witness.dag   |  64 +++++++++-
 src/v2/cli/compile_cli.dag                    | 120 +++++++++++-------
 src/v2/cli/population_verbs.dag               |  64 ++++++++--
 src/v2/test/claim/cli/population_verbs.dag    |  56 ++++++++
 6 files changed, 329 insertions(+), 83 deletions(-)

diff --git a/dag/gunbc/parse_verdict_publish.dag b/dag/gunbc/parse_verdict_publish.dag
index d7e0939c5cb..85f4d611354 100644
--- a/dag/gunbc/parse_verdict_publish.dag
+++ b/dag/gunbc/parse_verdict_publish.dag
@@ -7,7 +7,7 @@ import extdeps.github.actions { Step, RunStep, UsesStep, github_event_name_push,
 import gunbc.action_use_admission { cache_save_action }
 import v2.cli.population_verb_names { cli_verb_parse_population }
 import v2.cli.parse_verdict_invocation {
-  ParseVerdictInvocationArg, cli_option_compiler, cli_option_write_identity, cli_option_receipts,
+  ParseVerdictInvocationArg, cli_option_compiler, cli_option_reuse_from, cli_option_write_identity, cli_option_receipts,
   parse_verdict_invocation_command, parse_verdict_invocation_source_root_args
 }
 import gunbc.instrument_targets { v2_native_cli_source_roots }
@@ -37,6 +37,7 @@ import extdeps.github.expressions {
   StringLiteral,
   Equals,
   LogicalAnd,
+  LogicalOr,
   ExpressionTemplate,
   LiteralText,
   Interpolation,
@@ -81,6 +82,29 @@ fn materialization_publish_if() -> String {
   serialize_expression(expression: materialization_scope_guard(scope: materialization_publish_scope()))
 }
 
+// THE ROUTES THAT MAY READ AN ANCESTOR'S PUBLISHED VERDICTS. Reading is not publishing: it crosses no
+// write boundary, so it is admitted wherever the cache model says an entry is restorable -- read from
+// gha_cache_entry_restorable_by above, not restated here. A pull-request run restores the entries main
+// saved (materialization_publish_restorable_by_pull_request); a default-branch push restores the
+// entries earlier default-branch pushes saved, which is the only way a main commit can publish
+// verdicts for modules it did not touch. The reverse stays refused: an entry a pull request saved is
+// not restorable by the default branch, so a push never reads a pull request's entries.
+//
+// Admitting the READ decides no WALK. The two walks keep their own mutually exclusive guards --
+// gunbc.parse_verdict_restore.parse_verdict_reuse_walk_if on the pull-request route (it reports counts
+// and writes nothing) and materialization_publish_if on the default-branch push (it writes receipts and
+// the identity blob) -- so widening the read never puts a second walk into one run.
+fn materialization_restore_guard() -> Expression {
+  LogicalOr {
+    left: materialization_scope_guard(scope: GhaCachePullRequestMergeRefScope),
+    right: materialization_scope_guard(scope: materialization_publish_scope())
+  }
+}
+
+fn materialization_restore_if() -> String {
+  serialize_expression(expression: materialization_restore_guard())
+}
+
 fn materialization_gha_cache_key_prefix(family: String) -> String {
   join(["gunbc-materialization-", family, "-"], "")
 }
@@ -136,10 +160,20 @@ fn github_sha_expr() -> String {
 // THE WALK'S WHOLE ARGUMENT POPULATION, source-root tail included. See parse_verdict_reuse_args in
 // gunbc.parse_verdict_restore for why the roots are declared ARGUMENTS now rather than appended as a
 // rendered string: an option the invocation always passes has to be inside the shape the stamp certifies.
+//
+// IT READS THE NEAREST PUBLISHED ANCESTOR AND PUBLISHES, in one pass, over the restored directory the
+// pull-request walk reads too (parse_verdict_cache_working_directory) -- one spelling of where a base
+// lives, so the two routes cannot disagree about it. Reading is not standing: every module's digest is
+// recomputed and compared, the ancestor's parse-compiler identity must equal the one under verification,
+// and anything that fails either is parsed. What the pass saves is re-parsing members this commit did not
+// change, which is the whole of the walk's cost on a main push (a hit walk runs in seconds; a miss walk
+// in 15 to 24 minutes, measured on runs 42362849618 and 42339827972). It is why the receipts this run
+// publishes are the lines IT verified, not lines it inherited unexamined.
 fn parse_verdict_publish_args() -> List<ParseVerdictInvocationArg> {
   list_append(
     left: [
       ParseVerdictInvocationArg { option: cli_option_compiler, value: parse_verdict_compiler_expr() },
+      ParseVerdictInvocationArg { option: cli_option_reuse_from, value: parse_verdict_cache_working_directory },
       ParseVerdictInvocationArg { option: cli_option_write_identity, value: parse_verdict_identity_blob_file() },
       ParseVerdictInvocationArg { option: cli_option_receipts, value: parse_verdict_receipts_file() }
     ],
diff --git a/dag/gunbc/parse_verdict_restore.dag b/dag/gunbc/parse_verdict_restore.dag
index 1c55b05d0db..3661834f186 100644
--- a/dag/gunbc/parse_verdict_restore.dag
+++ b/dag/gunbc/parse_verdict_restore.dag
@@ -13,7 +13,7 @@ import v2.extdeps.languages.bash_build {
 }
 import std.optional { Optional, Present }
 import extdeps.languages.yaml.types { YamlKeyValue, yaml_string, kv }
-import extdeps.github.actions { Step, RunStep, UsesStep }
+import extdeps.github.actions { Step, RunStep, UsesStep, github_event_name_pull_request }
 import gunbc.action_use_admission { cache_restore_action }
 import gunbc.instrument_targets { v2_native_cli_source_roots }
 import v2.cli.population_verb_names { cli_verb_parse_population }
@@ -48,6 +48,8 @@ import gunbc.parse_verdict_schema {
 import gunbc.parse_verdict_publish {
   materialization_gha_cache_key_prefix,
   materialization_scope_guard,
+  materialization_restore_guard,
+  materialization_restore_if,
   parse_verdict_compiler_expr,
   parse_verdict_generation_one_bin,
   parse_verdict_identity_blob_file,
@@ -64,22 +66,58 @@ import gunbc.parse_verdict_publish {
 // never a pass and never an error (DESIGN.md section 5). The restore keys are exact; there are no
 // restore-keys, because a prefix hit would be a near identity and reuse is exact or it is a miss.
 //
-// The guard is derived from the pull-request scope in the cache model, as the publisher's is from the
-// default-branch scope, so the two sides cannot both fire on one event.
+// THE GUARD BOTH ROUTES READ UNDER is parse_verdict_restore_if below, which is derived from the cache
+// model (a pull request restores main's entries; a default-branch push restores the entries earlier
+// default-branch pushes saved). The two WALKS stay one per run: the pull-request walk is
+// parse_verdict_reuse_walk_if below, the push walk is materialization_publish_if in
+// gunbc.parse_verdict_publish, and they never both fire on one event.
 
-data parse_verdict_locate_step_name: String = "Locate first-parent ancestors of base.sha"
+data parse_verdict_locate_step_name: String = "Locate first-parent ancestors below the commit under verification"
 data parse_verdict_locate_step_id: String = "parse_ancestors"
 data parse_verdict_record_step_name: String = "Record the restored parse-verdict ancestor"
 data parse_verdict_reuse_walk_step_name: String = "Walk parse-population reusing the nearest published ancestor's receipts"
 
+// THE GUARD FOR READING A PUBLISHED ANCESTOR'S VERDICTS, derived from the cache model in
+// gunbc.parse_verdict_publish (materialization_restore_guard), which admits every route the model says
+// can restore an entry: a pull request restores what main saved, and a default-branch push restores
+// what earlier default-branch pushes saved. IT IS NOT THE GUARD FOR A WALK. Each walk keeps its own,
+// mutually exclusive guard -- parse_verdict_reuse_walk_if on the pull-request route, materialization_publish_if
+// on the push route -- so widening the read cannot put a second walk into one run and pay for the
+// population twice (DESIGN.md section 2).
 fn parse_verdict_restore_if() -> String {
+  materialization_restore_if()
+}
+
+// THE PULL-REQUEST WALK ONLY. It reads the restored entries, reports counts and writes nothing; with no
+// receipts to publish there is exactly one walk per pull-request run. The default-branch route takes NO
+// walk here: its walk is the publisher (gunbc.parse_verdict_publish.parse_verdict_publish_walk_step),
+// which reuses and saves in one pass. Asserted distinct from the publisher's guard by
+// test.claim.parse_verdict_restore_witness, because a run that fired both would double the walk.
+fn parse_verdict_reuse_walk_if() -> String {
   serialize_expression(expression: materialization_scope_guard(scope: GhaCachePullRequestMergeRefScope))
 }
 
-fn pull_request_base_sha_expr() -> String {
+// THE START OF THE FIRST-PARENT SEARCH, one per route, and on BOTH routes it excludes the commit under
+// verification -- a run must never be able to reuse the receipts it is itself producing. A pull request
+// searches the base it merges into. A default-branch push has no base, so it searches from
+// github.event.before: the commit main held BEFORE this push, which is the nearest commit that could
+// already have published. The commit being pushed is excluded by construction, not by arithmetic. When
+// `before` names no commit the run can resolve (a new branch, where GitHub reports the all-zeros sha), the
+// locator's git rev-list answers nothing, every ordinal is a typed miss, and the walk is a full walk --
+// the same shape an evicted entry takes, never an error and never a pass.
+fn parse_verdict_ancestor_base_sha_expr() -> String {
   serialize_template(template: ExpressionTemplate {
     segments: [Interpolation {
-      expression: ContextAccess { context: Github, path: ["event", "pull_request", "base", "sha"] }
+      expression: LogicalOr {
+        left: LogicalAnd {
+          left: Equals {
+            left: ContextAccess { context: Github, path: ["event_name"] },
+            right: StringLiteral { value: github_event_name_pull_request }
+          },
+          right: ContextAccess { context: Github, path: ["event", "pull_request", "base", "sha"] }
+        },
+        right: ContextAccess { context: Github, path: ["event", "before"] }
+      }
     }]
   })
 }
@@ -111,11 +149,15 @@ fn parse_verdict_restore_receipts_key(ordinal: Int) -> String {
   )
 }
 
-// THE LOCATOR: the first-parent ancestors of the base, newest first, to the declared ordinals, each
-// written as a step output, which the restore keys and the record step read. The script is a bash node tree emitted through bash_emit, as the
-// release-bins runners are; no shell is authored here. A `git rev-list` that cannot answer (a
-// base sha outside the checkout) leaves the output empty, and an empty sha is a key no entry carries,
-// so the restore is a typed miss rather than a failed required job.
+// THE LOCATOR: the first-parent ancestors below the commit under verification, newest first, to the
+// declared ordinals, each written as a step output, which the restore keys and the record step read. It
+// starts where parse_verdict_ancestor_base_sha_expr points -- the base on a pull request, the commit main
+// held before this push on the default branch -- and it EXCLUDES that start's own commit only by the
+// ordinal the caller asks for, which is why neither route may name the commit it is judging. The script is
+// a bash node tree emitted through bash_emit, as the release-bins runners are; no shell is authored here.
+// A `git rev-list` that cannot answer (a start outside the checkout, or the all-zeros sha GitHub reports
+// for a new branch) leaves the output empty, and an empty sha is a key no entry carries, so the restore is
+// a typed miss rather than a failed required job.
 fn parse_verdict_lit(t: String) -> Node { bash_build_word_lit(text: t) }
 fn parse_verdict_var(n: String) -> Node { bash_build_word_var(name: n) }
 fn parse_verdict_words(ws: List<Node>) -> Node { bash_build_word_concat_from_nodes(parts: ws) }
@@ -172,7 +214,7 @@ fn parse_verdict_locate_step() -> Step {
     id: Present { value: parse_verdict_locate_step_id },
     run: parse_verdict_locate_script(),
     shell: none,
-    env: Present { value: [kv(key: "BASE_SHA", value: yaml_string(s: pull_request_base_sha_expr()))] },
+    env: Present { value: [kv(key: "BASE_SHA", value: yaml_string(s: parse_verdict_ancestor_base_sha_expr()))] },
     working_directory: none,
     if_condition: Present { value: parse_verdict_restore_if() },
     continue_on_error: none,
@@ -201,7 +243,7 @@ fn parse_verdict_identity_hit(ordinal: Int) -> Expression {
 fn parse_verdict_identity_restore_condition(ordinal: Int) -> Expression {
   fold_list(
     xs: parse_verdict_restore_ancestor_ordinals,
-    empty: materialization_scope_guard(scope: GhaCachePullRequestMergeRefScope),
+    empty: materialization_restore_guard(),
     cons: fn(acc, earlier) {
       if earlier < ordinal {
         LogicalAnd { left: acc, right: NotEquals { left: parse_verdict_identity_hit_output(ordinal: earlier), right: StringLiteral { value: "true" } } }
@@ -213,7 +255,7 @@ fn parse_verdict_identity_restore_condition(ordinal: Int) -> Expression {
 }
 
 fn parse_verdict_receipts_restore_condition(ordinal: Int) -> Expression {
-  LogicalAnd { left: materialization_scope_guard(scope: GhaCachePullRequestMergeRefScope), right: parse_verdict_identity_hit(ordinal: ordinal) }
+  LogicalAnd { left: materialization_restore_guard(), right: parse_verdict_identity_hit(ordinal: ordinal) }
 }
 
 fn parse_verdict_cache_restore_step(name: String, id: Optional<String>, path: String, cache_key: String, condition: Expression) -> Step {
@@ -347,7 +389,7 @@ fn parse_verdict_reuse_walk_step() -> Step {
     shell: none,
     env: Present { value: parse_verdict_publish_memory_env() },
     working_directory: none,
-    if_condition: Present { value: parse_verdict_restore_if() },
+    if_condition: Present { value: parse_verdict_reuse_walk_if() },
     continue_on_error: none,
     timeout_minutes: none
   }
diff --git a/dag/test/claim/parse_verdict_restore_witness.dag b/dag/test/claim/parse_verdict_restore_witness.dag
index 78be01bd52d..1dd8a53f18d 100644
--- a/dag/test/claim/parse_verdict_restore_witness.dag
+++ b/dag/test/claim/parse_verdict_restore_witness.dag
@@ -6,14 +6,20 @@ import std.optional { Absent, Present }
 import extdeps.github.actions { Step, RunStep, UsesStep }
 import gunbc.action_use_admission { cache_restore_action, cache_save_action }
 import extdeps.github.action_release { action_release_uses_text }
-import gunbc.parse_verdict_publish { materialization_publish_if }
+import gunbc.parse_verdict_publish {
+  materialization_publish_if,
+  parse_verdict_cache_working_directory,
+  parse_verdict_publish_command,
+}
 import gunbc.parse_verdict_restore {
+  parse_verdict_ancestor_base_sha_expr,
   parse_verdict_identity_restore_step,
   parse_verdict_locate_script,
   parse_verdict_ancestor_sha_expr,
   parse_verdict_receipts_restore_step,
   parse_verdict_restore_if,
   parse_verdict_reuse_command,
+  parse_verdict_reuse_walk_if,
   parse_verdict_reuse_walk_step,
   parse_verdict_record_step,
   parse_verdict_record_script,
@@ -22,7 +28,7 @@ import gunbc.parse_verdict_restore {
 import gunbc.parse_verdict_publish { parse_verdict_identity_blob_file, parse_verdict_receipts_file }
 import extdeps.github.expressions { serialize_expression }
 
-fn restore_step_is_exact_restore_on_pull_request(step: Step, key_marker: String) -> Bool {
+fn restore_step_is_exact_restore_on_read_route(step: Step, key_marker: String) -> Bool {
   match step {
     UsesStep { name: _, id: _, uses: u, with: w, env: _, if_condition: c, continue_on_error: _, timeout_minutes: _ } =>
       (action_release_uses_text(action_release: u) == action_release_uses_text(action_release: cache_restore_action))
@@ -39,14 +45,58 @@ fn restore_step_is_exact_restore_on_pull_request(step: Step, key_marker: String)
   }
 }
 
-// POSITIVE CONTROL, ROUTE ASSERTED: both restores are the cache RESTORE action (not save), guarded by
-// the pull-request scope, with an exact key and no restore-keys, which a near-identity hit would need.
-test fn the_base_sha_entries_are_restored_exactly_on_pull_request_only() -> Bool {
-  restore_step_is_exact_restore_on_pull_request(step: parse_verdict_identity_restore_step(ordinal: 3), key_marker: "")
-    && restore_step_is_exact_restore_on_pull_request(step: parse_verdict_receipts_restore_step(ordinal: 3), key_marker: "")
+// POSITIVE CONTROL, ROUTE ASSERTED: both restores are the cache RESTORE action (not save), guarded by the
+// route that may READ an ancestor, with an exact key and no restore-keys, which a near-identity hit would
+// need. The read guard names BOTH routes -- the pull request (which restores what main saved) and the
+// default-branch push (which restores what earlier main pushes saved) -- and is still not the publisher's
+// guard, which is a conjunction of push AND the main ref and so admits no pull-request run.
+test fn the_base_sha_entries_are_restored_exactly_on_the_two_read_routes() -> Bool {
+  restore_step_is_exact_restore_on_read_route(step: parse_verdict_identity_restore_step(ordinal: 3), key_marker: "")
+    && restore_step_is_exact_restore_on_read_route(step: parse_verdict_receipts_restore_step(ordinal: 3), key_marker: "")
+    && string_contains(s: parse_verdict_restore_if(), pattern: "github.event_name == 'pull_request'")
+    && string_contains(s: parse_verdict_restore_if(), pattern: "github.event_name == 'push'")
+    && string_contains(s: parse_verdict_restore_if(), pattern: "refs/heads/main")
     && (parse_verdict_restore_if() != materialization_publish_if())
 }
 
+// POSITIVE CONTROL, THE WALKS ARE MUTUALLY EXCLUSIVE: reading widened to the push route must not have
+// given the push route a second walk. The pull-request walk is gated on the pull-request route ALONE --
+// not on the widened read guard, which would fire on a push too -- and it never names the publisher's
+// arguments; the publisher never runs on a pull request. A run that fired both would pay for the whole
+// population twice, which is the cost this change exists to remove (DESIGN.md section 2).
+test fn one_run_fires_exactly_one_walk_and_the_read_guard_is_not_a_walk_guard() -> Bool {
+  (step_condition(step: parse_verdict_reuse_walk_step()) == parse_verdict_reuse_walk_if())
+    && (parse_verdict_reuse_walk_if() != parse_verdict_restore_if())
+    && string_contains(s: parse_verdict_reuse_walk_if(), pattern: "github.event_name == 'pull_request'")
+    && (string_contains(s: parse_verdict_reuse_walk_if(), pattern: "'push'") == false)
+    && (string_contains(s: parse_verdict_reuse_command(), pattern: "--receipts") == false)
+    && (string_contains(s: parse_verdict_reuse_command(), pattern: "--write-identity") == false)
+    && string_contains(s: materialization_publish_if(), pattern: "'push'")
+    && (string_contains(s: materialization_publish_if(), pattern: "pull_request") == false)
+}
+
+// POSITIVE CONTROL, THE ANCESTOR START EXCLUDES WHAT IS BEING PRODUCED: the locator starts below the
+// commit under verification on both routes -- the pull request at its base, the push at the commit main
+// held before this push -- and NEVER at the head it is judging, which is how a run is kept from certifying
+// itself with receipts it has not produced yet.
+test fn the_ancestor_start_excludes_the_commit_under_verification() -> Bool {
+  let start = parse_verdict_ancestor_base_sha_expr()
+  string_contains(s: start, pattern: "github.event.pull_request.base.sha")
+    && string_contains(s: start, pattern: "github.event.before")
+    && (string_contains(s: start, pattern: "github.sha") == false)
+}
+
+// POSITIVE CONTROL, ONE BASE PATH: the publishing walk reuses and publishes in one pass, and reads the
+// SAME directory the pull-request walk reads -- one spelling of where a base lives, so the routes cannot
+// disagree about it and silently fall back to a full walk while believing they reused.
+test fn the_publishing_walk_reuses_the_same_base_the_pull_request_walk_reads() -> Bool {
+  let reuse_arg = concat(" --reuse-from ", parse_verdict_cache_working_directory)
+  string_contains(s: parse_verdict_publish_command(), pattern: reuse_arg)
+    && string_contains(s: parse_verdict_reuse_command(), pattern: reuse_arg)
+    && string_contains(s: parse_verdict_publish_command(), pattern: "--receipts")
+    && string_contains(s: parse_verdict_publish_command(), pattern: "--write-identity")
+}
+
 // The walk that consumes them is the reuse walk: it names --reuse-from, and writes nothing.
 test fn the_pull_request_walk_reuses_and_does_not_publish() -> Bool {
   string_contains(s: parse_verdict_reuse_command(), pattern: " --reuse-from ")
diff --git a/src/v2/cli/compile_cli.dag b/src/v2/cli/compile_cli.dag
index 20eb1153fe1..832a19c226c 100644
--- a/src/v2/cli/compile_cli.dag
+++ b/src/v2/cli/compile_cli.dag
@@ -23,7 +23,7 @@ import v2.compiler.module_acquisition {
   acquisition_ingest,
   enumerate_bound_modules
 }
-import v2.cli.population_verbs { PopulationReport, check_layout, parse_population, select_spelled_modules, parse_verdict_reuse_off, parse_verdict_reuse_requested }
+import v2.cli.population_verbs { PopulationReport, ParseVerdictReuse, check_layout, parse_population, select_spelled_modules, parse_verdict_reuse_off, parse_verdict_reuse_requested }
 import v2.cli.parse_verdict_reuse { parse_verdict_base_read }
 import extdeps.filesystem.filesystem_io { Filesystem }
 import v2.cli.population_verb_names { cli_verb_check_layout, cli_verb_parse_population, cli_verb_select_spelled_modules }
@@ -1001,53 +1001,26 @@ fn write_if_path(path: Optional<String>, content: String, what: String) -> Optio
   }
 }
 
+// THE ROUTES ARE NOT THE MODES. Which route a run is on is decided by the gate that launches it
+// (gunbc.parse_verdict_restore on a pull request, gunbc.parse_verdict_publish on a default-branch push);
+// what this function refuses is a combination of ARGUMENTS that means nothing, never a combination of
+// routes. It used to refuse reuse and publishing together, on the ground that "writing receipts from a
+// reuse walk would store counts as a verdict" -- true of the artifact as it was then, because report.text
+// was the counts whenever a base was read (v2.cli.population_verbs .population_parse_report) and the
+// writer wrote the text. It is no longer forced: the walk carries BOTH products
+// (v2.cli.population_verbs .PopulationVerdictProducts) and the writer writes the RECEIPTS PRODUCT, whose
+// lines a reused member and a parsed member record identically (v2.cli.population_verbs
+// .population_parse_accepted), a member being reported Reused only against the digest this walk recomputed
+// from the bytes on disk and a base whose parse-compiler identity equals the one under verification
+// (v2.cli.parse_verdict_reuse). So a default-branch run may stand on its ancestor's verified members and
+// still publish what it verified itself; a run whose ancestor is absent, unreadable, contradicted, evicted
+// or a different parse compiler simply parses everything, which is the shape it had before and never a pass.
 fn parse_population_written(
   bindings: List<SourceRootBinding>,
   compiler: Optional<String>,
   write_identity: Optional<String>,
   receipts: Optional<String>,
   reuse_from: Optional<String>
-) -> PopulationReport {
-  match reuse_from {
-    Present { value: dir } => parse_population_reusing(bindings: bindings, compiler: compiler, write_identity: write_identity, receipts: receipts, dir: dir)
-    Absent => parse_population_published(bindings: bindings, compiler: compiler, write_identity: write_identity, receipts: receipts)
-  }
-}
-
-// REUSE IS THE PULL-REQUEST MODE AND PUBLISHING IS THE MAIN MODE; they do not combine. Reuse reports
-// counts where publishing reports the receipts, so writing receipts from a reuse walk would store
-// counts as a verdict. The combination is refused rather than ordered.
-fn parse_population_reusing(
-  bindings: List<SourceRootBinding>,
-  compiler: Optional<String>,
-  write_identity: Optional<String>,
-  receipts: Optional<String>,
-  dir: String
-) -> PopulationReport {
-  match compiler {
-    Absent => PopulationReport { text: "", held: false, detail: "--reuse-from requires --compiler" }
-    Present { value: _ } =>
-      match write_identity {
-        Present { value: _ } => PopulationReport { text: "", held: false, detail: "--reuse-from does not combine with --write-identity or --receipts" }
-        Absent =>
-          match receipts {
-            Present { value: _ } => PopulationReport { text: "", held: false, detail: "--reuse-from does not combine with --write-identity or --receipts" }
-            Absent =>
-              parse_population(
-                bindings: bindings,
-                compiler: compiler,
-                reuse: parse_verdict_reuse_requested(base: parse_verdict_base_read(dir: dir))
-              )
-          }
-      }
-  }
-}
-
-fn parse_population_published(
-  bindings: List<SourceRootBinding>,
-  compiler: Optional<String>,
-  write_identity: Optional<String>,
-  receipts: Optional<String>
 ) -> PopulationReport {
   let needs_compiler =
     match write_identity { Present { value: _ } => true  Absent => match receipts { Present { value: _ } => true  Absent => false } }
@@ -1060,23 +1033,72 @@ fn parse_population_published(
           detail: "--write-identity and --receipts require --compiler"
         }
       Present { value: cid } => {
-        let report = parse_population(bindings: bindings, compiler: compiler, reuse: parse_verdict_reuse_off())
-        if !report.held {
-          report
+        let products = parse_population(
+          bindings: bindings,
+          compiler: compiler,
+          reuse: parse_verdict_reuse_for(reuse_from: reuse_from)
+        )
+        if !products.report.held {
+          products.report
         } else {
           match write_if_path(path: write_identity, content: cid, what: "parse-compiler identity") {
             Present { value: detail } => PopulationReport { text: "", held: false, detail: detail }
             Absent =>
-              match write_if_path(path: receipts, content: report.text, what: "parse-verdict receipts") {
-                Present { value: detail } => PopulationReport { text: "", held: false, detail: detail }
-                Absent => report
+              match products.receipts {
+                // A HELD REPORT WITH NO RECEIPTS PRODUCT IS A CONTRADICTION, not an empty artifact to write.
+                // The payload exists exactly when a compiler named the identity every line carries; writing
+                // an empty file here would publish "nothing was verified" as a base of zero verdicts, and a
+                // later run would read that as a complete answer. Refusing is the only honest outcome.
+                Absent =>
+                  PopulationReport {
+                    text: "",
+                    held: false,
+                    detail: "the walk held but produced no receipts payload to publish"
+                  }
+                Present { value: body } =>
+                  match write_if_path(path: receipts, content: body, what: "parse-verdict receipts") {
+                    Present { value: detail } => PopulationReport { text: "", held: false, detail: detail }
+                    Absent => products.report
+                  }
               }
           }
         }
       }
     }
   } else {
-    parse_population(bindings: bindings, compiler: compiler, reuse: parse_verdict_reuse_off())
+    match reuse_from {
+      Present { value: _ } =>
+        match compiler {
+          Absent => PopulationReport { text: "", held: false, detail: "--reuse-from requires --compiler" }
+          Present { value: _ } => {
+            let products = parse_population(
+              bindings: bindings,
+              compiler: compiler,
+              reuse: parse_verdict_reuse_for(reuse_from: reuse_from)
+            )
+            products.report
+          }
+        }
+      Absent => {
+        let products = parse_population(
+          bindings: bindings,
+          compiler: compiler,
+          reuse: parse_verdict_reuse_for(reuse_from: reuse_from)
+        )
+        products.report
+      }
+    }
+  }
+}
+
+// A BASE IS ASKED FOR, NEVER ASSUMED. Naming a directory says only where to look for an ancestor's
+// verdicts; it is not a claim that any of them stand. Each module's standing is decided afterwards, per
+// module, against the digest this walk recomputes, and an absent, unreadable, contradicted or
+// differently-compiled base answers MISS for every module -- a full walk, not a shortcut.
+fn parse_verdict_reuse_for(reuse_from: Optional<String>) -> ParseVerdictReuse {
+  match reuse_from {
+    Present { value: dir } => parse_verdict_reuse_requested(base: parse_verdict_base_read(dir: dir))
+    Absent => parse_verdict_reuse_off()
   }
 }
 
diff --git a/src/v2/cli/population_verbs.dag b/src/v2/cli/population_verbs.dag
index f5b2fd3e76e..fff8d55b99a 100644
--- a/src/v2/cli/population_verbs.dag
+++ b/src/v2/cli/population_verbs.dag
@@ -54,6 +54,39 @@ type PopulationReport {
   detail: String
 }
 
+// THE TWO PRODUCTS ONE parse-population WALK PRODUCES: what the process SAYS, and what a publishing route
+// WRITES. They were one value -- report.text carried the receipts when no base was read and the reuse counts
+// when one was (population_parse_report) -- which is the reason the two routes could not be one route: the
+// writer wrote the text, so a walk that had read a base would have published its counts line as a verdict.
+// Deriving both products from the one tally is what lets the default-branch route read an ancestor's
+// standing and still publish: the lines it writes are the lines a full walk writes for the same bytes
+// (population_parse_accepted records a reused member's line exactly as it records a parsed member's), and
+// the counts the operator reads still come from the same walk.
+type PopulationVerdictProducts {
+  report: PopulationReport,
+  receipts: Optional<String>
+}
+
+// THE RECEIPTS PAYLOAD, SPELLED ONCE. population_parse_report renders it and population_verdict_products
+// hands the same derivation to a writer; two spellings would be two truths about one artifact.
+fn population_parse_receipts_text(tally: PopulationParse) -> String {
+  join(list_reverse(xs: tally.accepted_receipts), "\n")
+}
+
+fn population_verdict_products(tally: PopulationParse) -> PopulationVerdictProducts {
+  PopulationVerdictProducts {
+    report: population_parse_report(tally: tally),
+    // NO COMPILER, NO RECEIPTS. A line is a claim about a parse compiler's identity, and with none named no
+    // line was ever recorded (population_parse_accepted appends only under a compiler). Absent is a refusal
+    // to write, not an empty file: an empty receipts file would restore as a base that answers nothing, and
+    // a base that answers nothing is how a walk starts believing it verified what it never read.
+    receipts: match tally.compiler {
+      Absent => Absent
+      Present { value: _ } => Present { value: population_parse_receipts_text(tally: tally) }
+    }
+  }
+}
+
 // ---- check-layout -----------------------------------------------------------------------------------
 
 type TreeFiles
@@ -514,7 +547,7 @@ fn population_parse_report(tally: PopulationParse) -> PopulationReport {
       Present { value: _ } =>
         PopulationReport {
           text: match tally.reuse.base {
-            Absent => join(list_reverse(xs: tally.accepted_receipts), "\n")
+            Absent => population_parse_receipts_text(tally: tally)
             Present { value: _ } => parse_verdict_reuse_counts_text(counts: tally.reuse.counts)
           },
           held: true,
@@ -544,29 +577,38 @@ fn population_declaration_duplicated_report(keys: List<String>) -> PopulationRep
   }
 }
 
-fn parse_population_listed(env: AcquisitionEnv, modules: List<String>, compiler: Optional<String>, reuse: ParseVerdictReuse) -> PopulationReport {
+fn parse_population_listed(env: AcquisitionEnv, modules: List<String>, compiler: Optional<String>, reuse: ParseVerdictReuse) -> PopulationVerdictProducts {
   match population_parse_begin(classes: parse_population_declared_refusal_classes(), compiler: compiler, reuse: reuse) {
-    PopulationDeclarationDuplicated { keys: keys } => population_declaration_duplicated_report(keys: keys)
-    PopulationBegun { tally: tally } => population_parse_report(tally: population_parse_go(env: env, pending: modules, tally: tally))
+    // NO PRODUCTS, NOT EMPTY PRODUCTS: nothing was enumerated, so there is no artifact to publish and
+    // "held: false" is the only truthful thing the run can say.
+    PopulationDeclarationDuplicated { keys: keys } =>
+      PopulationVerdictProducts { report: population_declaration_duplicated_report(keys: keys), receipts: Absent }
+    PopulationBegun { tally: tally } => population_verdict_products(tally: population_parse_go(env: env, pending: modules, tally: tally))
   }
 }
 
-fn parse_population_prepared(env: AcquisitionEnv, compiler: Optional<String>, reuse: ParseVerdictReuse) -> PopulationReport {
+fn parse_population_prepared(env: AcquisitionEnv, compiler: Optional<String>, reuse: ParseVerdictReuse) -> PopulationVerdictProducts {
   match enumerate_bound_modules(layout: env.layout, bindings: env.bindings) {
     BoundModulesRefused { subject: subject, cause: cause } =>
-      PopulationReport {
-        text: concat(subject, ": ", cause),
-        held: false,
-        detail: concat("could not enumerate the modules of ", subject)
+      PopulationVerdictProducts {
+        report: PopulationReport {
+          text: concat(subject, ": ", cause),
+          held: false,
+          detail: concat("could not enumerate the modules of ", subject)
+        },
+        receipts: Absent
       }
     BoundModulesListed { modules: modules } => parse_population_listed(env: env, modules: modules, compiler: compiler, reuse: reuse)
   }
 }
 
-fn parse_population(bindings: List<SourceRootBinding>, compiler: Optional<String>, reuse: ParseVerdictReuse) -> PopulationReport {
+fn parse_population(bindings: List<SourceRootBinding>, compiler: Optional<String>, reuse: ParseVerdictReuse) -> PopulationVerdictProducts {
   match acquisition_env(layout: repository_storage_layout(), bindings: bindings) {
     AcquisitionEnvUnprepared { diagnostics: _ } =>
-      PopulationReport { text: "", held: false, detail: "the dag grammar did not prepare; no module was observed" }
+      PopulationVerdictProducts {
+        report: PopulationReport { text: "", held: false, detail: "the dag grammar did not prepare; no module was observed" },
+        receipts: Absent
+      }
     AcquisitionEnvReady { env: env } => parse_population_prepared(env: env, compiler: compiler, reuse: reuse)
   }
 }
diff --git a/src/v2/test/claim/cli/population_verbs.dag b/src/v2/test/claim/cli/population_verbs.dag
index 9b62934f686..a25ed0b071a 100644
--- a/src/v2/test/claim/cli/population_verbs.dag
+++ b/src/v2/test/claim/cli/population_verbs.dag
@@ -41,6 +41,7 @@ import v2.cli.population_verbs {
   population_parse_begin,
   population_parse_refused,
   population_parse_report,
+  population_verdict_products,
 }
 import gunbc.parse_verdict_schema {
   parse_verdict_receipts_empty_detail,
@@ -305,6 +306,61 @@ test fn a_walk_over_no_enumerated_module_does_not_hold_and_names_the_declared_mo
     && string_contains(s: report.detail, pattern: "a parse over nothing is not a parse of zero")
 }
 
+// THE TWO PRODUCTS, SEPARATED. report.text and the published payload were one value, and that single slot
+// is what forced the gate to keep the two routes apart: with no base read the text IS the receipts payload,
+// and with a base read the text is the operator's counts line. A publishing route that reads a base must
+// therefore write the bytes a walk with no base would have written, while still printing the counts -- the
+// purity oracle for the default-branch route standing on an ancestor
+// (gunbc.recurring_failure_mode.cache_impurity: "byte-identical cached-vs-cold is the purity oracle"). The
+// two tallies below carry the SAME verdicts; only their route differs, so their payloads must not.
+test fn reading_a_base_changes_the_report_line_and_not_the_published_payload() -> Bool {
+  let off_tally = fold_list(
+    xs: every_declared_met,
+    empty: reuse_tally(reuse: parse_verdict_reuse_off()),
+    cons: fn(acc, judgment) { judge(tally: acc, judgment: judgment) }
+  )
+  let on_tally = fold_list(
+    xs: every_declared_met,
+    empty: reuse_tally(
+      reuse: reuse_base(i_base: optional_present(value: fixture_compiler), text: optional_present(value: reuse_receipts_text()))
+    ),
+    cons: fn(acc, judgment) { judge(tally: acc, judgment: judgment) }
+  )
+  let off = population_verdict_products(tally: off_tally)
+  let on = population_verdict_products(tally: on_tally)
+  match off.receipts {
+    Absent => false
+    Present { value: cold } =>
+      (cold == held_parse_report_text())
+        && (off.report.text == cold)
+        && (off.report.held == true)
+        && match on.receipts {
+          // THE PUBLISHED BYTES DO NOT MOVE. The route changed; the artifact did not.
+          Absent => false
+          Present { value: warm } =>
+            (warm == cold)
+              && (on.report.held == true)
+              // THE LOG LINE DOES MOVE: with a base read, the walk says what it reused.
+              && (on.report.text == parse_verdict_reuse_counts_text(counts: on_tally.reuse.counts))
+              && (on.report.text != cold)
+        }
+  }
+}
+
+// A HELD WALK WITH NO COMPILER HAS NO PAYLOAD TO PUBLISH, and Absent (not an empty string) is what makes
+// the writer refuse instead of publishing a base of zero verdicts that a later run would read as complete.
+test fn a_walk_with_no_compiler_offers_no_receipts_payload() -> Bool {
+  let no_compiler = fold_list(
+    xs: [JudgedParses { key: "dag:a/four.dag", source_digest: fixture_source_digest }],
+    empty: begun(classes: []),
+    cons: fn(acc, judgment) { judge(tally: acc, judgment: judgment) }
+  )
+  match population_verdict_products(tally: no_compiler).receipts {
+    Absent => true
+    Present { value: _ } => false
+  }
+}
+
 // REUSE: the base the restore step left behind, built through the SAME crossing the verb reads it
 // with (`parse_verdict_receipt_lines`), so a witness that supplies the text still executes the unfold.
 fn reuse_receipts_text() -> String {
-- 
2.47.3

--- END PATCH ---
