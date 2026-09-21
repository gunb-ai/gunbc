# CONVERGENCE-ONE — program record at wind-down

One convergence protocol and one fleet authority for persistent host effects. A subject is
**selected**, **observed goal-blind**, **assessed** against the goal, **planned**, **admitted
against the observed basis**, **applied**, **independently read back**, and given a **terminal
verdict**. Anything that calls itself convergence without running that lifecycle is a second
convergence algebra — a §3 authority fork — and the program's job was to end it.

This page is the record at the point the program was wound down (2026-09-20) under system
pressure, so that v1 performance and v2 migration take priority. It exists because the state
was held in a session's pinned plan and in seven lanes, none of which survive. Its roadmap home
is `fleet-closed-loop-converge`; the two remaining cuts that have no home of their own are
`fleet-convergence-census-closure` and `fleet-convergence-legacy-deletion`.

## What landed

- **C0** — the census of persistent host effects; dual-home made unwritable.
- **C1** — the canonical protocol bound; `ensure` is a projection of it.
- **C2** — one selected LiveDeploy roster. `deployment_spec_srv1` has exactly one home.
- **C3** — LiveDeploy scope, member basis, plan-side frontier discharge; and, in its remainder,
  the request arm binding `DesiredDeployment.selected_identity`.
- **C4** — the apply frontier row, honestly **awaiting** rather than falsely discharged
  (see *An undischarged frontier* below), plus a production gate on the protocol being bound.
- **C5** — readiness defork and the canonical readiness algebra.
- **C6** — owned roots folded into the member model.

## Ready but unmerged when the program stopped

Six pull requests were CLEAN with every check green, waiting on an operator merge:

| PR | Cut | What it carries |
|----|-----|-----------------|
| #11678 | C9a.1 | GCP Secret IAM: read the policy back **independently** after the write |
| #11676 | C7 | provider-state root as a `live_deploy` ensured member, read back — **now DIRTY** |
| #11689 | C11 | GitHub reads rebound onto the canonical join — **parked, see below** |
| #11732 | C9b | App-key rotation: exact-version verifier, contract, `DisableOnly` — **MERGED** |
| #11795 | — | `RungDropAmendment` carrier, so a later ruling on a standing drop has a home |
| #11828 | — | `mtcollins1_boot` narrowed to `FleetSshKeyNotConsumed` |

**#11689 is parked for the same reason, one rung further on.** It is complete and correct at
`2e63f9d53`: the rebind is done, and a scratch regeneration driver that an earlier `git add -A` had
swept in — no consumer, re-minting a generated workflow path as a literal — was found by review and
deleted, so the PR no longer carries a defect that would land. It then went DIRTY again within the
hour on the contended pair. It is left open, not closed.

**#11732 is parked green, and parking it leaves a real gap open.** At `e0c453f961` it had one
approval, no change requests, and all four checks green by name; its eighth review cleared the
thing most likely to be wrong in such a cut — that the frontier row names the add-version and
disable-prior *capability* rather than letting the verifier's own existence retire it. It was
stopped by queue position on the contended pair, twice. The consequence, stated so it can be
weighed rather than forgotten: **`ci-github-app-private-key` still has no rotation deadline, no
restore test and no rehearsed compromise answer on main, and nothing in the corpus can answer
"does the key in version N actually work."** That key is the root of trust for every org-admin
action CI takes.

**#11676 was deliberately not rebuilt.** It is approved and green at `c153cfa519`, but main moved
27 commits and edited five of its seven files, so resolving is content work rather than a
projection drop. Its only consumer is C8, which is held indefinitely, so rebuilding would deliver a
member nothing consumes at the cost of a full review and CI cycle — and it would rot again the next
time anyone touches `live_deploy`. The PR is left open rather than closed: the review history and
its annotations are worth more there than in a closed tab. The colliding files are
`live_deploy/emit.dag`, `live_deploy/spec.dag`, `roadmap/roadmap_dashboard_instance.dag`,
`test/claim/live_deploy/emit_test.dag` and `test/claim/live_deploy_unit_emission_oracle_witness_test.dag`.

**#11828 must not merge on checks alone.** Its acceptance is a wet dispatch of
`fleet-converge mode=mtcollins1_boot`: if that mode does need the fleet key, it fails at the
point of use, and discovering that on main is strictly worse than discovering it before.

**C9b's verifier has never run against the live surface.** All fifteen witnesses supply an
observation record; none executes the real mint, which needs the fleet-converge runner, the
federated principal and the live App. Three shapes are therefore *readings* of upstream rather than
observations: that the Secret Manager response carries the resolved version in its name field, that
`curl -D` writes a status line whose second word is the code, and that the mint step's refusal arms
leave their record lines before exiting. **The first dispatch of `app_key_version_verify` is the
inhabitance claim for that whole cut**, and it is cheap — read-only, one installation-token mint, no
mutation. If it refuses with an absent mint observation on a version that plainly exists, suspect
the record wire before the mint. Its actuation half (add-version, disable-prior) is blocked on
`gunbc.github_actions_wif` being absent from main: on a runner there is no federated identity for a
Secret Manager write, so the only honest route today is an operator workstation.

## Two operator acts C9b left open, one of them dated

**`rotate_by` on main is `2026-12-18`, and it is a placeholder a lane chose rather than policy.**
The verifier reads it on every run and refuses once it passes. So if nobody sets it, the first
thing that happens on that date is that a green verify run turns red with a deadline-passed
refusal. That is the honest behaviour, and it is also a trap for whoever meets it first: **the fix
is a policy decision and a rotation, not a code change.**

**Nothing in that cut has run against the live surface.** Every witness supplies its own
observation record. The first dispatch of `app_key_version_verify` against the currently enabled
version *is* the inhabitance claim for the whole thing, and it is cheap and safe — read-only, one
installation-token mint. Three shapes are readings of upstream rather than observations, so that is
where a first-dispatch failure will be: that the Secret Manager response carries the resolved
version in its name field, that `curl -D` writes a status line whose second word is the code, and
that the mint's refusal arms leave their record line before exiting. A refusal reporting an absent
mint observation on a version that plainly exists means the record wire, not the mint.

**The actuation half is blocked on identity, not on design.** Add-version and disable-prior with
independent readback need a federated identity for a Secret Manager *write*, and
`gunbc.github_actions_wif` is still absent from main — so a runner has none, and the only honest
route today is an operator workstation. Reaching for a print-token on a runner fails at runtime
rather than degrading, which is why the lane did not.

## Still held, and on what

- **C8 — wet apply.** Held until three proofs close: temporal order (control 6), Baseline vs
  PostApply (control 7), and apply-script path disjointness. C8 must **execute** a nonempty
  `ReleaseEffectPlan`; until it does, that terminal answers `AwaitingCapability`. C8 also owes
  the discharge of C4's frontier: naming its projection-routed apply carrier moves that row off
  a prose condition. Note the cost trap recorded by C4's lane — a bound discharge needs an
  observed population, and computing it costs ~60s over the live tree, so it must not go on the
  plan route blindly.
- **C9 — census closure by effect family.** The denominator is every non-terminal row in the C0
  census, not a keyword search. One effect family per change, each answering the same seven
  questions: who selected the desired state, what independently observed reality, what assessed
  the difference, what exact plan was admitted, what authority executed it, what independently
  read it back, and what old path was deleted or renamed. C9a (GCP IAM) was pulled forward
  because it touches production; C9a.2 (the bare-grant authorization closure) is unstarted.
- **C10 — deletion and the recurrence wall.** Deliberately last, and performs no new semantic
  migration: it deletes superseded routes and makes recurrence mechanically visible. Its gate
  must join against an independently discovered producer population, never a search for
  suspicious names.

## Unowned residue

- **C5's poll Ready-exit** is recorded unproven. It closes when a fixture can supply `Ready`
  into the poll outcome without materializing the live site.
- **The org-admin credential admission surface** was censused and the answer is DELETE, but the
  deletion was never written: that lane produced analysis only and has **no PR**. Its findings
  exist nowhere else, so they are here.
  - Obligations 1–8 are met or dissolved by the live App-token path, and two carry the argument.
    **Genealogy is enforced by construction**: the mint is serial and each link exits on failure,
    so a token cannot exist with an incomplete genealogy — deleting a rung-2 validation that a
    rung-4 construction replaced. **Capability sufficiency is enforced better**: an unconditional
    probe that refuses on the real response proves more than a declared capability list, which is
    exactly why the dead model carried `CredentialOperationProbeRequired`.
  - **The sharpest fact in the census**: `ci_spec`'s own annotation says the interim PAT step *"was
    never taken, and it was never necessary."* So the whole `SecretMaterial` surface — custody,
    genealogy, rotation contract, continuity due-dates — modelled the lifecycle of a credential
    **that was never minted at all**. Not superseded; never inhabited. That turns the deletion from
    a cleanup into evidence about how the model got there.
  - **The delete set is grep-verified, not refusal-verified.** Under §3 the deletion *is* the
    census, so treat the set as a hypothesis and let the compiler refuse. Six declarations are
    already dangling on main today with no consumer at all, including the witness.
  - **Keep** `org_admin_app_key_unreadable_message`, `org_admin_installation_token_refused_message`
    and the two arms they name: `ci_spec`'s prelude renders them and a witness checks it. They are
    a declared frontier with a stated trigger, not dead code, and a dead-code sweep would wrongly
    take them. Do not touch `extdeps.github.org_admin_auth` — faithful upstream modeling with four
    live importers.
  - **Open, for the operator**: `docs/plans/org-admin-credential-acquisition.md` is the prose home
    of this model. Deleting it orphans two live annotations that cite it; keeping it leaves a plan
    document for a credential that was never minted.
- **#11760 would re-fork the authority #11795 just consolidated.** It still carries the postscript
  text inline on the drop's `restoration_trigger`. If it lands after #11795, that string must be
  deleted and replaced with a row under `dag/gunbc/rung_drop_amendment/`. Its author agreed. This
  is the only known change that would undo that consolidation.
- **The amendment witness is enrolled on no lane at all.** Its four claims were executed on the
  committed tree, but not through a claim harness: `claim_executor` has no per-module selection,
  and a whole-floor run is a CI job rather than a session one. They were run via `gunbc run
  --function`, which refuses to map a `Bool` to an exit code and *prints the value in the refusal*
  — a real execution of the real function, but not a harness run. Since required CI runs no claims,
  nothing today would notice if that file went red. If claims return to the merge path, it belongs
  in the first batch.
- **A read-write-re-read fold is unfalsifiable under the current replay harness**, and this is a
  frame-level gap rather than a defect in any subject. REST replay keys a fixture by exact
  invocation, and the first read and the re-read of such a fold are the *same* invocation — same
  operation, target and input digest — so no fixture set can answer 200 to one and 403 to the
  other. Sequencing is not expressible at all. Worse, an invocation's `input_digest` is minted
  host-side over the bound parameter environment and has no `.dag` constructor, so an authored
  fixture cannot reproduce it for any operation that takes inputs; the existing probe matches only
  because its operations take none. This is why C9a.1's unreadable-re-read stop is declared
  unexecuted for its route. It belongs to `v2.std.witness_evaluation`, and closing it is what
  would let that stop be executed rather than argued.
- **The projected `restoration_trigger`** on the namespace-wave-admission drop does not state
  that restoration half (1) was built and then deleted; #11795's amendment supplies that context
  in the projection, so check the rendered page before re-opening it.

## Findings worth keeping

These cost real time to establish and are not derivable from the diffs.

**An undischarged frontier was unwritable.** `converge_apply_for_host` calls `host_effect_apply`
directly and never the projection, so no honest consumer exists before C8 — but `FrontierStanding`
carried only *honest* and three defect arms. The corpus forced a choice between a citation that
lies and a protocol that can never bind, whose gate would then refuse every healthy run. C4
resolved it by inhabiting `std.dissolution`: an awaiting arm that can never be read as discharged,
refusing when the capability has arrived and the row was never updated. The closing sentence is
**derived** from the row's standing; authoring it would have been a second authority for
retirement.

**A successful actuation return cannot mint convergence.** The GCP IAM writer classified the
policy returned *by its own write* and greened. The repair reads the policy back independently.
The load-bearing control is the one that fails when the write response carries the grant and the
re-read does not.

**Observation must be goal-blind.** `decode_registration_json` took the expected App identity
*into the decoder*, so a valid response describing another App was filed as *unobserved* rather
than observed-then-diverged — a known mismatch laundered through the refusal arm.

**Check that a check can fail.** Two witnesses were deleted rather than kept: one asserted that a
prose sentence contained certain words while its name claimed a structural impossibility, and one
tested a state that, once the member was derived from the spec, had no constructor at all. A
permanently green check is worse than none, because it gets cited as coverage.

**A stale-plan fingerprint must carry identity.** The LiveDeploy baseline hashed member *labels*,
so a changed executable digest, tree revision, dirty tree, unit document, running release or root
owner all left it unchanged. Caught by an external review after two approvals, including mine.

## A first-apply risk C8 must clear before it touches srv1

C7 models the provider-state root as an ensured directory, and the emitted command is
`install -d -m 0755 -o <service user> -g <service user> <codex home>`. **`install -d` chmods and
chowns a directory that already exists.** The mode is a constant of the ensured-directory arm,
not a fact read from the host, and nothing in the model refuses a widening.

So if srv1's provider-state root is currently `0700` — which is what a provider CLI creating its
own state directory under a umask would plausibly leave — **the first real apply silently widens a
credential directory to world-readable**, and the same command re-owns it if the live owner
differs. This was never verified against a host, because no wet effects were permitted; it is an
inference from reading both code paths, and its author flagged it rather than letting it pass as
settled.

**Before C8's first apply: stat the live directory** and compare its mode and owner against what
the ensured step emits. If the observed mode is tighter, the ensured-directory arm needs a mode
carried per subject — the `ManagedDirectory` derivation — **before** anything applies, not after.

**A typed outcome on the policy read is a fail-open, and the next reader will re-derive it.** The
obvious way to make C9a.1's inaccessible arm a typed value is to declare `RestOutcome` on
`GetSecretIamPolicy`. Do not. The other caller reads the policy in the node-pattern form, which
cannot branch, so a failed read there returns an empty policy with a blank etag that flows
straight into `SetSecretIamPolicy` — an unconditional whole-policy overwrite of a live IAM policy,
driven by a read that never happened. That is strictly worse than the defect C9a.1 repairs. The
ordering is therefore fixed: make that caller an `EffectPlan` step that can refuse **first**, and
only then declare the outcome.

## Process facts a later program will need

- **The required CI check builds the compiler and runs no witnesses** (#11742), and the
  namespace-wave-admission gate was removed (#11774). A green check means *it compiles*. Lanes
  must run what they claim and say what they ran. Three composition breaks reached main in one
  day — each PR green against its own base, red only in composition — and the repair commits say
  so in their titles.
- **A PR failing the generated-artifact gate self-heals.** Since #11791, `heal.yml` is
  dispatch-only and the repair moved into the required run's fleet lane: the branch picks up a
  `gunbai-bot` commit within ~10 minutes. Authors must **wait for the publisher**, never
  hand-produce the bytes and never push over a sealed candidate — doing so invalidates it and
  restarts the cycle.
- **Heal regenerates `.github/workflows` but cannot publish it, and nothing detects that.** It does
  push — a heal commit landed `.gitattributes` and `docs/design-rung-drops.md` on one branch — so
  the constraint is *workflow paths specifically*, consistent with an App token that cannot write
  them. On one head heal reported success, logged writing the 89,904-byte workflow in its own
  workspace, and emitted a candidate manifest holding only the two non-workflow paths. Since #11742
  removed the regeneration gate from PR CI, a stale workflow projection is now **invisible**: that
  drift survived three pushes and four green heal runs and was caught only by reviewers. Any PR
  changing a `ci_spec` entry target inherits this, and must carry its own derived bytes.
- **`fleet_converge_workflow.dag` and its generated `.yml` are the contended pair**: every lane that
  adds a dispatch mode touches the same two rows, so PRs there go DIRTY on queue position rather
  than on any defect. A conflict region there can split a step, and a keep-both resolution is not
  safe by reading — the class and its fourth specimen are rostered at
  `gunbc.recurring_failure_mode.merge_region_excludes_shared_tail`.
- **The verification that discriminates, for anyone hand-committing a projection:** push, then read
  heal's repair-candidate *manifest* on that head. An empty entry set against two entries on the
  prior head is a real signal. A byte count matching heal's own is corroboration only — a different
  delta could preserve length.
- **No session can execute a `gunbc run` remotely.** BuildBuddy executors expose no cgroup memory
  limit, so `gunbc.host_budget_source` refuses with `HostBudgetUnreadable` before planning; a lane
  that bound a limit by hand got SIGKILLed instead. Briefs that offer "CI or one remote dispatch"
  as the escape from local memory pressure are offering something that does not exist today.
- **The local regeneration hazard is LOAD, not a ceiling.** The same command made no progress in
  40 minutes at host load ~270 and finished in about 20 at load ~66, writing correct bytes. Each
  `gunbc run` loads the whole corpus, so runs must be serial — running claims alongside a regen
  makes both thrash. Do not conclude the local arm is dead; check the load first.
- **A workflow run's branch column is not the ref it built.** `heal.yml` checks out an input SHA,
  so the branch column names where the dispatch fired. Reading it as the built ref produced a
  false "main is red" alarm and a dispatched repair lane against innocent substrate files.
- **`git merge origin/main` in a session worktree is dangerous and looks fine.** The worktree is a
  shallow clone, so the merge base is wrong. On one lane it reported two trivially additive
  conflicts and produced a commit that reverted 99 files and 6,886 lines of other lanes' work —
  including `witnesses.yml` back to a pre-#11742 revision, so CI then ran an old job roster and
  went **green on a revision that had reverted main**. The safe recipe: `reset --hard origin/main`,
  check out your own files, then assert that every deleted line in `git diff origin/main` is one
  you wrote.
- **A session worktree is a shallow clone, and git lies about it confidently.** Two lanes hit this
  with different symptoms: one was told it was 231 commits *ahead* of main and not an ancestor
  (truth after `git fetch --deepen=200`: 0 ahead, 26 behind, ancestor yes), the other had a merge
  refuse with *unrelated histories* while GitHub's compare API reported eleven ahead and four
  behind. It also re-shallows, so the false readings come back. Run
  `git rev-parse --is-shallow-repository` before believing any ancestor or count answer — a missing
  merge base here is a fetch-depth artifact, never a force-pushed main.
- **Do not revert main's generated-artifact drift out of your own PR.** The auto-heal bot pushes
  exactly that regeneration back onto your branch, and its commit **resets the approval tally**. The
  only real choice is whose commit carries the bytes, and the bot's is better because it is
  attributable.
- **`extdeps.time.rfc3339` models no arithmetic and no parse**, only a conditional lexical
  comparator. So a period-shaped deadline (`create_time + N days`) is not expressible, which is why
  C9b's rotation contract carries an absolute `rotate_by` that a completed rotation must move
  forward by hand. A period first owes rfc3339 parse and arithmetic.
- **A one-arm coproduct is not a coproduct.** `type X = OnlyArm` — a name, equals, a single bare
  arm, no pipe — parses as a type *alias* to an unknown name and fails with *name not found in
  module*, attributed to the importing module rather than to the declaration. It cost a lane a
  cycle. C9b's restore cadence became a record naming the consumer that discharges it, which is
  better anyway: the intent builds its roster from that record, so the field is read rather than
  declared beside the thing it describes.
- **Merge order is not a preference.** Out-of-order merges broke sibling cuts four times, and none
  of it was a defect in the changes.
