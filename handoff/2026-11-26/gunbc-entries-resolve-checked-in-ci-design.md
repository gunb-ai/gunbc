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
