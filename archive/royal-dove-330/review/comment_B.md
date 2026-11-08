# Change 1 (push-route receipt reuse) — PATCH 2 of 2, CURRENT REVISION — UNCOMPILED / UNEXECUTED review material

Posted as review material only: no push, no PR, no run, no dispatch, no build, no host contact.
**UNCOMPILED / UNEXECUTED.** Controls are not results; a missing symbol in one is NO_OBSERVATION, not a RED
(§12). Predictions about cache scope and ancestor selection are REVIEW CLAIMS, labelled as such.

    base commit  ed007978dc2a67096df619b1b91566458dfb2ffd
    commit       4df61c70a3fca7faa76b15011b4461256c0949e4   (local, not pushed)
    tree         3616deaab19f60f63132dc159365034cecbe1df7
    supersedes   4f68bf150a3384711c1824c0ef62dc3559d16350 (kept as history in the previous comment)
    patch bytes  49521
    sha256       0eee5c237b4fc1ec46124a0100b60595b69fcaf4a7fa2cdcbd04e21bb18badbc

## Extraction (verify before trusting)

    base64 -d < p2.b64 | gunzip > change1-rev.patch
    sha256sum change1-rev.patch   # must print 0eee5c237b4fc1ec46124a0100b60595b69fcaf4a7fa2cdcbd04e21bb18badbc
    wc -c < change1-rev.patch     # must print 49521
    git -C <clone-of-base> checkout ed007978dc2a67096df619b1b91566458dfb2ffd
    git -C <clone-of-base> apply -p1 change1-rev.patch

## GZB64 payload (gzip -9n | base64, single line)

```
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
```

---

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
