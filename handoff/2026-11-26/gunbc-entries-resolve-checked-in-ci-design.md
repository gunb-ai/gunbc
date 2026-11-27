# Design: every declared entry a route uses is resolved by a check in CI

Row: `gunbc-entries-resolve-checked-in-ci` (layer 0, project `remote-build-bandwidth`, carrier
`dag/gunbc/witness/compiler_gate_workflow.dag`). Author: loyal-badger-240. Date: 2026-11-26.
Status: design for review — no code written, no CI step enrolled.

Ticket's own frame: "Design first — a cheap per-module check rather than a full closure — and keep it
under budget (`gunbc.design_ruling::ci_job_budgets_closed_to_growth`)." Red control: "reintroduce the
`claim_run` argument and the check fails." Out of scope: raising any CI time or memory budget; judging
witness bodies; **full-closure resolution as the mechanism**.

## 1. Where the cost lands today

A call-shape break is invisible at merge and paid in full by whoever runs the entry next.

| what broke / ran | how it was found | cost |
| --- | --- | --- |
| `a93c6559c97` passed `claim_run: false` to `gunbc_run_step_script_for_target`, which declares no `claim_run` (`dag/gunbc/site/gunb_ai_pages_workflow.dag:88`) | a `generated_artifact_gate main_wet_one` resolve, after the merge | 1391 s, 13.3 GiB (my figure; the row's text records 1252 s / 13.5 GiB for the same class) |
| the hostname-converge dispatch-admission step | its own step, twice | 1585 files, 301,548 ms, 6,084,572 KiB (runs 45770097040, 45773631715) |
| the client witness entry | a witness run, refusing | 704 s to refuse |
| the remote-build client entry, healthy | — (baseline for "what a resolve costs") | 906–909 files, 68–163 s, ~3.6 GiB; 832,637 ms on a contended host |

The merge that caused the first stayed green. That is the defect: **CI never resolves the entries the
fleet's own routes run**, so the first resolver in the tree is a person, on a step deadline, paying the
whole closure.

## 2. The population, measured, and how it stays one authority

The population must be **derived, not hand-typed** — a roster of entries someone maintains is a second
place deciding which entries matter (`gunbc.design_argument::single_authority`). It is derived from what
the tree already says: every `--entry <file>.dag --function <fn>` literal, which is how a route names the
entry it runs.

Measured on main today (`--entry X --function f` across `dag/`, `src/`, `test/`, `product/`,
`.github/workflows`, `scripts/`):

- **228 distinct (entry, function) pairs** across **171 entry files**; by area `dag/gunbc` 229 sites,
  `src` 13, `test` 5, `product` 4.
- Some hits are placeholders, not routes (`<probe>.dag`, `<row>.dag`, `/tmp/ann_priv/probe.dag`,
  `test_run.dag`): the rule must exclude literals that are not resolvable paths in the tree, and **refuse
  when a route names an entry file that does not exist** — that is itself one of the breaks worth
  catching.

The row's named subset is the enforcement set: instrument gates (`generated_artifact_gate main_wet_one`,
`extdeps_scope_placement_gate`, `emission_entry_instrument measure`), the fleet-converge steps
(`fleet_converge_plan_revision verify_checkout_at_plan_revision | admit_plan_run_pointer |
admit_apply_run_pointer | admit_dashboard_deploy_dispatch_inputs`, `apply_receipt
assert_dashboard_deploy_receipts`), and the remote-build entries (`remote_build_client remote_compile_cli |
remote_test_cli | remote_gate_cli`, `remote_build_serve remote_build_serve_cli`,
`remote_build_serve_front_door remote_build_serve_front_door_cli`). The rest of the 228 are covered by the
same rule automatically, because the rule reads literals rather than a list.

## 3. Why the check cannot be the engine, at first

The engine's resolve is **inclusive by construction**: to typecheck one call it resolves the callee's
whole closure (`resolve_entry_graph_inclusive`), which is exactly the cost the row exists to avoid
(`out_of_scope`: "full-closure resolution as the mechanism"). The only amortisation the CLI already has is
`gunbc run --function`, which is repeatable — "every named function runs in ONE process over ONE load of
the entry's closure" — so N functions of ONE entry cost one resolve. It does not help across entry files,
and N entries × ~70–1500 s is over budget by orders of magnitude. A shallow engine mode ("typecheck these
files against the signatures of their direct imports") would be the sound long-term answer, but it is a new
engine authority in `src/v1/stage0`, not a CI check, and it should be its own row.

## 4. The check

A call-shape disagreement lives in exactly two files: the caller's call site and the callee's `fn`
declaration. Nothing else is needed to see it. Rules, deliberately narrow so that a refusal means
something:

- **R1 attribute or skip.** A call is checked only when the callee name is *imported by name* into the
  caller and the imported module exists and declares `fn <name>`. Local declarations, qualified calls
  (`m.n(...)`) and homonyms are **skipped and counted** — guessing which `f` is meant is how an instrument
  starts crying wolf.
- **R2 labels at the call's top level only.** A nested `ident:` is a record field, not an argument label.
- **R3 literals and comments are blanked before scanning, offsets preserved.** This is not a detail: the
  first run produced **439 refusals**, and a sample of 12 showed 11 artifacts — a comma inside a string
  splitting an argument list, and a `>` in a comparison closing a generic that was never opened. After
  blanking literals/comments and opening a generic only where a generic can start, refusals fell to
  **182** and the attributed-call count rose from 178,896 to **178,190 with 153,892 skipped** (was
  243,651 skipped): the same two bugs were hiding calls, not just inventing them.
- **R4 one refusal: `PASS_A_LABEL`** — a label passed that the attributed declaration does not name. That
  is the `claim_run` shape exactly. A declared parameter absent at some call site is **reported, not
  refused**: one caller may legitimately be one of several, and refusing it would produce 424 findings on
  main today (`brand(std.types).name`, `second_count(std.measure).s`, …), most of them scanner reach.

Measured cost (whole tree, the worst case a PR could ask for; prototype `/tmp/call_shape.py`):
**102 s over 8,777 `.dag` files / 133,818,701 bytes**, single-threaded Python at `nice 15`. CI does not
need the tree: a PR changes a handful of files, and the check's per-file work is independent, so the
enrolled cost is **the changed files plus the declaration files they import** — seconds, with no engine
involved and no closure. Memory is a file buffer per module, three orders of magnitude under the
3.6 GiB a single entry resolve costs.

**Enrollment precondition.** 182 residual candidates must be adjudicated to zero real defects before the
check is allowed to refuse a PR — the sampled pattern is mis-attribution (a local wrapper shadowing an
imported name, or a multi-line call attributed to the wrong name), and `fail_closed`'s standard is a
consumer green by execution: an instrument that cries wolf gets ignored, which is worse than no instrument.
Adjudication is slice 2, not the design.

## 5. Budget fit

`gunbc.design_ruling::ci_job_budgets_closed_to_growth` (operator, 2026-11-07: "i do not want that timeout
increased", "we also have to ban any increases without explicit signoff/approval from me", and on memory:
"we are supposed to be lower CI memory budgets, not raising them") closes both time and memory budgets to
growth. So the check adds **no new budget row**: it is a step in the required gate, and its step takes its
timeout from the same declared row its neighbours read — `profile.roster.timeout` via
`gunbc.ci.repository_ci_profile::gunbc_roster_gate` (read at
`dag/gunbc/witness/compiler_gate_workflow.dag:1168,1240`; the emitter "keeps no copy of it, so the
required job's identity has exactly one spelling"). If the step's measured wall ever needs more than the
declared row allows, that is an escalation to the operator, not a new number in a file.

## 6. Red controls (all three required)

1. **The ticket's own arm:** reintroduce `claim_run: false` at `gunb_ai_pages_workflow.dag:88` → the check
   refuses, naming the caller line, the callee module and the absent parameter. This is the planted break
   the handback asks for.
2. **The other direction:** delete `claim_run` from a callee's declaration while a caller still passes it →
   refuses (proves it reads the declaration, not a remembered list).
3. **The main arm:** on unmodified main, after adjudication, the check passes with **zero** refusals, and
   its skip counts are reported in its own output. A check whose only evidence is a refusal it invented is
   the supplied-value failure this repo refuses everywhere else.

## 7. Overlap, so one fact does not get two instruments

- `remote-build-client-closure-cost` (my other row) is the same class in another route — an entry
  resolving more than its demands reach. That row owns *what an entry should resolve*; this row owns
  *"does the declared entry still resolve at all, cheaply, in CI"*. The instruments (`closure_bfs`,
  `entry_reads`, `cut_impact`, `call_shape`) are shared and live under this row as CI infrastructure;
  the demand-table findings stay in that row's boundary. Per sharp-pike-333's encoding this row is layer 0
  and does **not** own leaf extraction.
- `gunbc.drift_watch_workflow` compares generated artifacts against their sources; it does not resolve
  entries, so it cannot see a call-shape break (`#17129` proved that gap: the generator and the generated
  file can agree with each other and both disagree with the engine).

## 8. Decisions I need before slice 2

1. Is the residual-skip rate acceptable as a stated limit (R1 skips ~46% of candidate calls), or must the
   check also attribute qualified calls (`m.n(...)`) before enrollment? Attributing them is mechanical but
   widens the false-positive surface; my recommendation is to enroll narrow and widen with evidence.
2. Does the check live as a claim module (`test/claim/...`, run by `gunbc test`, reading source as the front
   door's `w_import_lines`/`w_source` rows do) or as a step running the compiled form? The claim form keeps
   it inside the model and the required lane; its own resolve cost must then be measured and disclosed like
   everything else in §4 — that measurement is the first thing slice 2 produces.
3. Which of the 228 literals are *routes* in the row's sense, if the parent wants the enforcement set
   narrowed below "every literal that names a real file".

## 9. ADDENDUM 2026-11-26 — the first .dag implementation attempt, and what the language said

The instrument was drafted as `dag/gunbc/instruments/declared_entry_call_shape.dag` (kept out of the tree
until it passes; the draft is at `/tmp/call_shape_dag_draft.dag`). Two things the row's text has to record,
because both cost time and both are cheap to state:

**The slim closure is real.** The draft's first run resolved in `resolve_entry_graph_inclusive ms=1159`
with `demand_acquired_modules=53` — its imports were `std.types`, `std.process`, `std.algebra`, `std.list`,
`std.optional`, `extdeps.filesystem.filesystem_io`. Against the same binary's `[pre-entry]` figures: the
serve request arm is 898 files / 143,992 ms / 3,435,568 KiB, and the #17114 front door 150 / 9,751 ms /
374,128 KiB. A call-shape check whose closure stays at this shape is **two orders of magnitude cheaper
than the thing it replaces**, which is §3's argument confirmed by measurement rather than by claim. (The
figure will grow as the instrument grows; keeping it pinned is the witness's job, §4.)

**The language has no list comprehension.** `[y | x <- xs, cond]` does not exist anywhere in the tree — the
only occurrences were in the draft. `[a | b <- …]` does not fail to *type*, it fails to *parse*
(`parse-failed … expected expression, found Pipe`), so the row's instrument must be written in the tree's
real idioms: `map`, `filter`, `flat_map`, `fold`, `enumerate`, `first_where`, `distinct_by_key`, and
method chains (`lines.skip(n: 1)`, `xs.first()` returning an Optional matched against `Present`/`Absent`
from `std.optional`). Adjacent facts learned the same way, all verified by grepping the tree for usage:
`var`, `while`, `for`-statements and integer indexing (`xs[0]`) are not in the language either; the verified
string vocabulary is `string_length(s:)`, `substring(s:, start:, end:)`, `split(s:, delimiter:)`,
`string_contains(s:, pattern:)`, `starts_with(s:, prefix:)`, `ends_with(s:, suffix:)`, `trim(s:)`,
`join(list, sep)`; list vocabulary is `length(xs:)`, `count(xs)`, `list_member(xs:, x:, eq:)`,
`distinct_by_key(xs, key)`; the `Filesystem.Read(path:)` → `.success`/`.content` idiom comes from
`test.claim.compute.remote_build_serve_front_door_witness`; and a `|` character inside a comprehension
position is refused by the parser, which is why the pair encoding became a record
(`type ImportedName { module: String, name: String }`) rather than a joined string — one reason among
several to prefer the record.

## 10. ENROLMENT — where a step actually runs, from the tree's own burned record

`.github/workflows/witnesses.yml` is a generated projection: its line 2 is
`Authority: gunbc.witness.compiler_gate_workflow compiler_gate_workflow; regen via
gunbc.instruments.generated_artifact_gate main_wet`, so the check's step belongs in that model and the YAML
is regenerated, never hand-edited (§ the generated-projection rule that review 98880 taught me on #17129).

Enrolment has a specific failure mode already documented in the repository, and it is the reason this note
exists: `gunbc.instruments.extdeps_scope_placement_gate` says of itself that **none of its three modeled
enrollments executes** — a `CommitSpecGate` row is not an executing entry, and a non-test-marked `…_passes`
fn is never planned; only `CommitWitnessClaim` rows on an `EXECUTING_COMMIT_SURFACES` surface become
entries that run, via `project_ci_floor_witness_entries` (`src/v1/stage0/src/cli_run.rs`). Its own words:
"the .dag call graph is not the execution graph, and 'runs in CI' is decided by the executing fold, never
by who calls whom in .dag." So the acceptance condition for this row is not "the check exists and is
green", it is **"a red check blocks a PR"** — demonstrated by pushing one of §6's reds and watching the
lane refuse it. Anything less than that witness is the extdeps gate's history repeated.

## 11. THE PRECISION DEBUG — six false-positive classes, what cured them, and what did not

The instrument parses, resolves in 1.2–4.8 s over 53 modules, runs, and reports. It is not yet shippable:
pointed at six real files it reports 189 / 206 / 32 / 11 / 89 / 723 label refusals where the truth is zero
(every one of those files compiles and its own witness lanes are green), so the red-vs-green delta that is
this row's whole acceptance condition cannot be demonstrated yet. The current draft is kept at
`/tmp/call_shape_dag_draft.dag`, out of the tree; the tree gets no instrument that cries wolf. What was
cured, in the order found, each by pointing the instrument at a real file and reading every refusal:

1. **A fragment with no `:` is a positional argument, not a label.** `provides_only(capabilities:
   [CargoCapability, RustcCapability, ...])` cut at the first `)` and split on commas handed back the list's
   own elements, and the instrument refused `gunb_ai_pages_workflow.dag` twice for "passing RustcCapability"
   to a function whose only parameter is `capabilities`. Cured; the same rule applied to the declared side.
2. **A comment is not code.** Full-line comments were scanned, so `remote_build_serve.dag` refused for
   "passing command" from a prose line, and this file refused `std.algebra.trim` for "passing prefix" and
   "passing pattern" — text taken from its own header's list of verified primitives. Cured by dropping
   full-line comments, then also stripping `//` to end-of-line.
3. **Nested-call labels are not the outer callee's.** `trim(s: cs_after(s: t, marker: "{"))` attributed
   `marker` to `trim`. Cured by keeping only fragments at depth zero — counting `()`, `{}` and `[]`.
4. **The argument list must be cut at its MATCHING `)`, not the first one.** This was the miss that matters:
   with first-`)` cutting, `gunbc_run_step_script_for_target(source_roots: ..., target:
   gunbc_run_step_target(module: ..., function: "..."), receipt_rel: none, claim_run: false)` ended the
   argument list at the nested call's close, so `claim_run` was invisible and the instrument was SILENT on
   the exact `claim_run` shape it exists to catch. Cured by accumulating pieces until the paren count that
   `name(` established returns to zero — and the counter must count PARENTHESES ONLY there: counting braces
   and brackets too made the walk run past the call's close and harvest labels from later calls ("minute
   from std.measure passes all_failed"). (a) and (3) were found while this file refused itself; (4) was
   found only by running the red control, which is why the row's acceptance condition is a demonstrated red.
5. **String literals — NOT the cause I took it for.** Blanking string contents (the oracle's R3 rule) was
   applied on the hypothesis that an unbalanced `(` inside a shell-script literal was letting the walk run
   on; it moved 723 to 723 and 92 to 89. Either the rule is right and something else dominates, or the rule
   is not what this language needs. Untested hypothesis, recorded as such.
6. **OPEN: the harvest that remains.** 723 refusals on `remote_build_client.dag`, all against `trim` from
   `std.algebra` (declared parameter `s`), carrying labels `stdout`, `stderr`, `text`, `fetched`, `work`,
   `detail`, `keep`, `delimiter`, `pattern`, `start`, `prefix` — the labels of dozens of *other* functions.
   That signature says the walk for `trim(` still runs across call boundaries somewhere, and I could not
   reproduce it by reading. The next step is not another guess: add a temporary trace entry that prints, for
   one (file, callee) pair, the harvested body text — read it, find the shape, fix that, and only then
   re-verify against the oracle per file. The oracle is the calibration precisely so that guessing is not
   the method: for these six files the Python oracle's refusal counts are the expected numbers, and the
   .dag instrument must equal them, file by file, before anything is enrolled.

The budget facts (§9) are unaffected by any of this: the closure is 53 modules and the resolve is 1.2–4.8 s,
because none of the precision work pulls in a hub. The rule for whoever continues: every fix must be
demonstrated by the RED firing and the GREEN arms staying at zero on these six files, and a fix that reduces
a count without the red firing is not a fix, it is a mute button.

## 12. THE INSTRUMENT PASSES ITS ARMS — the three real causes, and the tallies

`dag/gunbc/instruments/declared_entry_call_shape.dag` is in the tree and is green on every arm. §11's
"open" item is closed: the residual harvest was never the algorithm, it was three implementation defects,
each found by reading rather than guessing, and each recorded here because each is the kind that silently
makes an instrument useless.

1. **`count(split(s: f, delimiter: n))` counts PIECES, not occurrences** — one more than the delimiter
   appears ("a(b" splits to two pieces and holds one `(`). Every depth counter was inflated by one per
   fragment, so no argument list ever closed and each call harvested the labels of the calls after it: the
   723 invented refusals on `remote_build_client.dag`. Fixed with `cs_occurrences(hay:, needle:)`. The
   Python oracle never had this bug, which is the argument for keeping the oracle at all.
2. **The v2 corpus is rooted, not nested**: module `v2.workflow.gunbc_invoke_step_emit` is
   `src/v2/workflow/gunbc_invoke_step_emit.dag`. Treating the leading `v2.` as a directory meant the
   declaration file for exactly ONE import in `gunb_ai_pages_workflow.dag` could not be found — the import
   of `gunbc_run_step_script_for_target`, the callee of the historical `claim_run` break this row is named
   for. The disclosed skip count said `1 named imports skipped unattributed` while the instrument reported
   a clean pass on the red: the skip count is printed for exactly this reason, and a check that hides its
   own skips would have shipped blind.
3. **The parameter list is read with the same walk as an argument list.** The first version carried a
   "started" flag by testing `acc == ""`, so a declaration written across lines — `fn name(` alone on its
   line, one parameter per line, `) -> T {` — was never recognised at all and `declared` came back empty,
   which silently disables every refusal for that callee. Deleting that state machine and reusing
   `cs_call_body` (paren-matched) is both smaller and correct.
   Related: the same-file "declaration is not a call site" guard is GONE. The historical break was
   caller-and-declaration in one file, so a scanner that only follows imports cannot see it; a declaration's
   own parameter list is now scanned as if it were a call and contributes exactly its declared names, which
   the label rule accepts, so no special case is needed.

TALLIES, engine at this worktree's head, one `[pre-entry]` block per run.

| arm | expectation | result |
| --- | --- | --- |
| green: gunb_ai_pages_workflow.dag | 0 refusals | PASS (0 label, 0 missing, 0 skipped) |
| green: compute/remote_build_serve.dag | 0 | PASS |
| green: instruments/declared_entry_call_shape.dag (its own source) | 0 | PASS |
| green: compute/remote_build_serve_front_door.dag | 0 | PASS |
| green: witness/hosted_runner_shadow_gate_workflow.dag | 0 | PASS |
| green: compute/remote_build_client.dag | 0 | PASS |
| red 1: `claim_run: false` reintroduced at :88 | refuse | REFUSED — `gunbc_run_step_script_for_target from v2.workflow.gunbc_invoke_step_emit passes claim_run  declared: source_roots, target, receipt_rel` |
| red 2: caller passes `receipt_rel_removed_by_red_control:` | refuse | REFUSED — same callee, same located form |
| red 3: a route literal naming a missing .dag | refuse | REFUSED — `--entry names no file: dag/gunbc/site/gunb_ai_pages_bogus_missing.dag` |

COST OF THE CHECK ITSELF, measured: `resolve_entry_graph_inclusive ms=4965`, `closure_files=53`,
`max_rss_kib=175772` (172 MiB), one whole file scanned in ~6 s wall. The serve request arm on the same
binary: 898 files / 143,992 ms / 3,435,568 KiB. So the check is ~29× faster and ~20× lighter in RSS than
#17114's front door, and two orders of magnitude below the resolve it exists to replace — inside the
`ci_job_budgets_closed_to_growth` direction, which the operator's own words close to growth in both time
and memory.

STILL BEFORE THE ROW IS DONE: the witness lane (its own `w_import_lines` pin, the closure cap at 53, the
two refusals, the skip count, and an agreement arm against the oracle on these six files), the CI step in
`gunbc.witness.compiler_gate_workflow` with a timeout taken from `profile.roster.timeout`, and the
demonstrated red in the *witnesses* lane (§10) rather than at my keyboard. The refusals are two and stay
two; the forged-probe files stay untouched, per the manager's ruling that the engine is the authority on
coproduct heads.

## 13. THE CI STEP (half b) — what runs the instrument, on what, and against which base

The instrument is a scanner with no hands. This section is the hands: `dag/gunbc/witness/declared_entry_call_shape_step.dag`,
embedded by `gunbc.witness.compiler_gate_workflow` as a bound step of the required job (`capability_neutral`,
unguarded by the fast-lane reuse condition, since the check judges the pull request's diff on every pull
request and not only on a built run).

**The base.** `git diff --name-only <base>...HEAD -- '*.dag'`, merge-base semantics, where `<base>` is the
EVENT's base commit: `github.event.pull_request.base.sha` on a pull_request and `github.event.merge_group.base_sha`
on a merge_group (the workflow runs on `workflow_dispatch`, `merge_group`, `push` (main), `pull_request` (main),
`workflow_call` and a `schedule`; of those only the two carry a base). Both rejected spellings are pinned
absent by a claim, not just avoided: `GITHUB_BASE_REF` is a branch name and moves under the run, and a two-dot
diff against main counts main's own new commits as the PR's — charging a PR for someone else's break. The base
is fetched explicitly (`git cat-file -e` then a `git fetch --depth=1` fallback) because the checkout's depth is
not this step's fact to assume, and an unresolvable base REFUSES; only an empty diff skips.

**Who does the events with no base.** They are excluded by the step's `if:` rather than handed an empty diff.
A scheduled or dispatched run has no PR, so there is nothing this run changed for a check to judge; refusing it
would red every schedule for a reason nobody can act on, and green-ing it would be the silent pass this row
exists to retire. The condition is therefore pinned by a claim, so it cannot quietly widen.

**The skip/refuse split.** STEP skips on an empty diff and prints `skipped: no .dag change in this diff` — a
step that exits silently on the common path reads as a step that did not run. INSTRUMENT refuses an empty
scope, and refuses an unreadable file rather than skipping it. The step's skip is a judgement about a diff,
which the step owns; the instrument's would be about a file it was handed, which it does not.

**The reverse-importer pass, and which direction it may be wrong in.** Candidate importers come from
`git grep -l -F -f <stems> -- '*.dag'` over tracked files, where a stem is the changed path with its source
root and extension removed and its separators dotted. Mention is not import, so the index may over-include;
membership is decided by the instrument, which re-reads every candidate and re-parses its import lines with
the one parser. Over-inclusion costs seconds. Under-inclusion would be a missed refusal, so the stem match is
deliberately the widest reading of the name. This pass is the whole reason the check can see the defect it is
named for: the `claim_run` caller was never edited by anyone.

**A ceiling, and no floor.** Above 1200 `.dag` files in scope the step refuses — a diff that large is not this
run's diff (a mis-resolved base, a squash-merge ancestor), and judging it would burn an hour of a 90-minute job
on somebody else's tree. There is deliberately no lower bound: a one-file PR that touches a leaf nothing imports
is legal, and a check that reds it has bought a false alarm with the safety it sold. The instrument's refusal of
an empty scope is the only bottom.

**Cost.** Measured on this host, engine built at head: instrument entry `resolve_entry_graph_inclusive ms≈4788–5161`,
`closure_files=53`, `max_rss_kib≈175772–182708` (~172–178 MiB), ~5.2 s wall; the scan of a 54-file scope is ~24 s
(~0.35 s/file), so a 200-file diff is ~70 s — inside the job's existing roster timeouts, with no timeout raised
(`gunbc.design_ruling::ci_job_budgets_closed_to_growth`). For contrast, the same binary on the serve request arm:
898 files / 143,992 ms / 3,435,568 KiB.

**No envelope input, on purpose.** The required job sets `GUNBC_MEMORY_ENVELOPE_REQUIRED=slot` on its subject
steps; reading it is the TEST route's job (`v1_compiler.cli_run.target_invocation_host` `test_verb_checked`, the
same file declaring `MEMORY_ENVELOPE_REQUIRED_ENV`), and this step calls `gunbc run`. Putting the input on a step
whose verb never reads it writes a line into the shipped YAML asserting a bound nobody took — a receipt for a
measurement nobody made. The envelope lines the neighbouring walk steps carry are theirs, because their verb is
`test`. What bounds this step is its scope; what it costs is the measurement above.

**The invocation arrives by parameter, and that is a budget decision.** The first draft imported
`gunbc.parse_verdict_publish` (for `native_cli_bin()` and `parse_verdict_publish_memory_env()`) and
`gunbc.instrument_targets` (for the source-root pair) into the step module. The step module is imported by this
lane, so every claim here resolved them: one claim had not finished resolving after ten minutes and took
13 min 54 s. Taking `bin` and `source_roots` as parameters — supplied by the job that already owns those facts
elsewhere — brought the whole 18-claim lane to ~5 minutes in one process, and a claim now pins the module's
import list to `std.types`, `std.optional`, `extdeps.github.actions` and the `v2.cli.parse_verdict_invocation`
leaf, with `gunbc.` forbidden in it. A check enrolled under a budget law is not exempt from it, least of all in
its own test lane. The enrolment claim (that the required job's own bound-step list carries this step) lives in
`dag/test/claim/compiler_gate_emit_build_lane_witness.dag`, which pays for the workflow hub anyway.

**The string medium and shell text.** `${name}` in a string literal is a reference to a value in scope, and a
bare `{name}` is read as one too — so a shell script written literally in `.dag` is a program referring to
variables nobody declared (`undefined variable 'changed'`, `ambiguous reference 'base'`, and `git cat-file -e
$base^{commit}` refusing to resolve `commit`). `${{ github.* }}` survives because `{` cannot start an identifier.
The step assembles its variable references through `sh()`/`braces()` and the witness assembles its needles
through `shn()`, so the emitted shell is ordinary shell and the `.dag` stays a description of it.

**The generated file, and the byte boundary that nearly fooled me.** `.github/workflows/witnesses.yml` is
generated; the documented regen is `gunbc.instruments.generated_artifact_gate main_wet_one`, whose entry closure
is every emitter in the registry — measured elsewhere at 3,706 files / 1,391,228 ms / 13,336,092 KiB, which on a
shared host is the whole-tree build the budget law keeps out of a session. For this change the artifact was
regenerated through a throwaway entry that reached the one producer the registry names
(`gunbc.witness.compiler_gate_workflow::expected_compiler_gate_yml`): 1516 pool modules, 10 m 22 s, 6,565,452 KiB.
That entry is NOT committed — a second regen path beside the registry's authority is the worse fork — and its
first run proved why the diff, not trust, is the check: it wrote the file WITHOUT the two provenance-header lines,
because those are prepended by `gunbc.generated_artifact_emit` `artifact_generate` via
`gunbc.generated_workflow_provenance::generated_workflow_provenance_header`, not by the producer. The diff
showed two deletions; the entry was corrected; the final diff is 38 insertions and zero deletions. The
throwaway lives in the session's `/tmp` notes and in this paragraph, and its correctness criterion is stated
there: any hunk other than the new step means the path is wrong and the artifact goes back to `main_wet_one`.

**The author was caught by the row's own defect class.** While writing the lane, a claim called
`list_member(xs:, value:)`; the declared parameters are `(xs, x, eq)`. The engine's words were the same shape
this instrument predicts — `call shape mismatch calling 'list_member': no parameter named 'value' (declared:
[T, xs, x, eq])` — and it surfaced as an unobserved claim in ~5 minutes rather than as a CI resolve. That is the
row's argument, demonstrated on its own author.

**Evidence.** See the PR body for the tallies: the 19-claim instrument/step lane (§14 restates its figure), the
two new enrolment claims in the emit-build lane, and the executed arms of the emitted step body — the committed
YAML's script, extracted and run under bash with only the two GitHub expressions resolved (event base, job
binary), for: an empty diff, an event with no base, an unresolvable base, an over-budget scope, a real
PR-shaped scope, and a committed red whose caller nobody edited.

## 14. The step, executed — and the defects only execution found

Five arms were run against the emitted body of the COMMITTED YAML (the named step's `run:` block extracted
from `.github/workflows/witnesses.yml`, with only the two GitHub expressions resolved — the event's base and
the job's binary — and the rest left to bash), engine built from this checkout:

| arm | input | expected | observed |
|---|---|---|---|
| A1 | the base expression resolves empty — the push / schedule / dispatch / workflow_call shape | skip, loudly, rc 0 | `skipped: this event carries no base commit to diff against…`, rc 0 |
| A2 | base == HEAD, so the `.dag` diff is empty | skip, loudly, rc 0 | `skipped: no .dag change in this diff`, rc 0 |
| A3 | the event's base is neither in the checkout nor fetchable | REFUSE, rc 1 | `…is not in this checkout and could not be fetched: refusing to guess a scope`, rc 1 |
| A4 | the scope is above the budget (budget forced below the real figure) | REFUSE, rc 1 | `scope of 78 .dag files / 53638 lines is above the budget of 2 lines: … A tree-wide migration runs this instrument out of band and cites the run in the PR body`, rc 1 |
| A5 | a real PR-shaped scope, `git diff --name-only <merge-base>...HEAD`: 11 changed, 78 in scope, 53,638 lines | instrument PASS, rc 0 | `scope: 78 .dag files, 53638 lines (…)`, then silence, rc 0, 102 s on a host at load ~20 (93 s before the budget line was added) |

Two of the five found defects in the code rather than in the expectation, and neither is reachable from a
claim that only asks *did it refuse*.

**(1) The instrument refused its own fixtures.** A5's first run returned `2 label refusals, 1 missing entry
files`: the three fixture files that exist in order to be refused (`caller_bad`, `wide_caller`,
`route_missing`), swept in because they are `.dag` files in the diff. A check that reddens any PR touching the
files which prove it works is reverted within a day, so the skip rule had to widen — but not by exempting a
path, a package, or a name in a list, which is the shape of exemption that turns a check into a formality. The
*declaration* widened: a file saying it is a control is not judged (`MUST NOT RESOLVE`, `Enrolled RED`, and the
new `EXPECTED CALL-SHAPE REFUSAL`, the last one being this instrument's own vocabulary — a fixture is not a
module that must not resolve, it is a module that must be refused, and conflating the two would have made the
marker a lie in one direction or the other). What narrowed is the escape: a caller that NAMES a file in
`expected_refusal_files` judges it anyway. Claims name; CI names nothing. The pair is pinned by
`a_fixture_declared_a_control_is_skipped_by_a_scan_and_refused_when_named`, whose arms are the scan (0
findings, count 1) and the named judgment (1 finding). If the skip breaks, the PR adding a fixture reddens; if
the exception breaks, the claim that would otherwise report green without ever having observed a refusal goes
DID NOT HOLD.

**(2) The scan was almost twice as expensive as it needed to be, and the ceiling was measured in the wrong unit.**
A5 took 164.6 s for 78 files. The scan re-read every declaration file once per name imported from it, and
each read is an effect-machine round trip; reading each declaration once per scanned file (`CalleeText { path,
text }`, looked up by path) took it to 93 s. The same figure is what killed the original ceiling: at ~1.65 ms
per LINE the ruler has to be lines, not files, because 600 leaf fixtures and 600 workflow hubs are an order of
magnitude apart in cost. `declared_entry_call_shape_line_ceiling = "250000"` is ~7 minutes at the measured
rate, ~5× the largest scope this row has seen, and its refusal names the remedy — run the instrument out of
band and cite the run in the PR body — rather than leaving a tree-wide migration, which the repo's own
replacement-migration rule *requires* to be one change, with no way through. There is still no floor.

**Execution also caught the harness, not the code.** The first A5 handed the instrument a scope path built by
a Python f-string that had been left unmarked, so it received the literal text `{d}/changed.txt`; the
instrument answered with the empty-scope refusal, which is the correct answer to a scope file it cannot read.
It is worth recording because that is precisely the error class a claim written against real files cannot
distinguish from success: a wrong path and an empty diff both look like "nothing to check". It is why the
witness lane drives the instrument's own API and only the CI step reads lists off disk, and why the empty scope
refuses instead of passing.

**Two medium traps, recorded once each.** Record construction needs the brace form: `CalleeText(path: p,
text: …)` parses as a *call*, and a call whose callee identity cannot be established inside an effectful
function is refused at resolve — "effect summary incomplete … its effects cannot be joined"; `CalleeText {
path: p, text: … }` is a literal and carries no effect. And the shell idiom `{print $1+0}` inside a `.dag`
string never reaches the shell: `{` followed by an identifier is string interpolation, so the module would not
resolve. Both failures were loud at resolve; the second one changed what the step computes, because the
budget's measurement moved from `awk` to `cat | wc -l`, which is also the cheaper form.

**The lane's figure at this head, with the engine built from this checkout and the freshness gate satisfied:**
19 claims — 19 held, 0 did not hold, 0 unobserved, in one process, 291 s. The instrument's own entry still
resolves 53 modules (`resolve_entry_graph_inclusive ms=4982`, `max_rss_kib=184220`), and the two refusals are
unchanged: adding the control-skip exception added a caller-side fact, not a third thing the instrument
refuses.

**Binary freshness is a route gate, not a suggestion.** `gunbc test` refused twice with
`BinaryFreshnessUndecided — /cargo-target/release/gunbc: its dep-info names an input outside this worktree
(…/quiet-seal-506, …/crisp-pike-295); it was built from another checkout, so nothing here says whether it
answers for this one`. On a shared `CARGO_TARGET_DIR` another session's rebuild lands under you between
build and run, and — the asymmetry that matters — `gunbc run` does NOT apply the gate. An instrument
measurement can therefore come silently from someone else's build while a claim tally cannot. Copying the
binary aside does not satisfy it either: the gate requires hard-link twins under `deps`, i.e. the artifact
cargo actually built. Every figure from here is from a build at this head immediately followed by the lane in
the same shell.
