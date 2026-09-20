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
| #11676 | C7 | provider-state root as a `live_deploy` ensured member, read back |
| #11689 | C11 | GitHub reads rebound onto the canonical observe-then-assess join |
| #11732 | C9b | App private-key rotation: exact-version verifier, contract, `DisableOnly` |
| #11795 | — | `RungDropAmendment` carrier, so a later ruling on a standing drop has a home |
| #11828 | — | `mtcollins1_boot` narrowed to `FleetSshKeyNotConsumed` |

**#11828 must not merge on checks alone.** Its acceptance is a wet dispatch of
`fleet-converge mode=mtcollins1_boot`: if that mode does need the fleet key, it fails at the
point of use, and discovering that on main is strictly worse than discovering it before.

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
- **The org-admin credential admission surface** (`admit_org_admin_credential` and its capability
  model) has had no production consumer since the 2026-09-05 supersession. Its census concluded
  DELETE: obligations 1–8 are met or dissolved by the live path, and the App-token migration
  **raised** the rung — a serial mint where each link exits on failure is construction where the
  dead surface had validation. The deletion was dispatched and not finished.
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
- **Heal cannot publish `.github/workflows`.** Any `.dag` change that reprojects a workflow must
  carry its own derived bytes, and nothing detects when it does not. Main's model and its
  executing workflow diverged this way once already.
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
- **Merge order is not a preference.** Out-of-order merges broke sibling cuts four times, and none
  of it was a defect in the changes.
