# Review-sheet and census lane — state at wind-down

Written 2026-09-20 by zesty-crane-846 on an operator instruction to wind down and record
remaining items, so the system can prioritise v1 performance and v2 migration. Nothing here is
in progress.

**Read the census section before acting on any number in this lane's output.** The prospect rows
that exist are historical prototype output and are NOT authority for runner occupancy, spend, or
opportunity. That is the single most important thing this page carries.

## What landed

| PR | what |
|---|---|
| #11552 | the App-manifest acquisition MODEL and its route — **the flow is not executable end to end**, see below |
| #11581 | three format-converge defects a live spreadsheet found that no witness could |
| #11610 | the actuator consumes the plan's custody name instead of re-deriving it |
| #11564 | GitHub observation record and bounded scan producer — incomplete; completed by #11656 |
| #11669 | `gunbc.review_sheet_identity` — the review-sheet identity semantic kernel |
| #11656 | the three repairs #11564 merged without, plus a sealed scan outcome and an admitted page size |

**No external prerequisite of this lane is still open.** #11552, #11564, #11656, #11669, #11671,
#11677 and #11679 are all merged. Earlier revisions of this page described #11656 as open and the
census token as queued behind #11671 → #11677 → #11679; that is stale and the blockers are gone.

### #11552 acquisition is modeled, not achieved

`gunbc.github_app_acquisition` instructs the operator to "paste the manifest below into the
form's manifest field, and submit". **That instruction is not executable.** GitHub's ordinary
App-creation page carries no manifest field; the manifest protocol requires a form POST carrying
a `manifest` parameter, and a GET renders the blank create page. This was observed live: the
operator followed the step and reported a form with no such field.

Registration is also a different fact from INSTALLATION, and #11677's JWS/token route consumes an
App and an installation that have already been admitted — it creates neither. So the remaining
acquisition work is manifest submission, installation consent, callback capture, and credential
custody, and no receipt in this corpus establishes that the census App was registered, installed
and placed in usable custody.

## The one fact everything else rests on

An empty `appProperties` search establishes **"this OAuth client received no matching rows"**. It
never establishes **"no legacy-marked file exists"**. `appProperties` are private to the
*requesting application* — the OAuth client id the token was obtained with — and nothing in this
corpus observes the application identity of its own token. A service-account principal is a
different subject, so comparing service-account emails does not establish visibility.

That distinction is the whole content of the identity kernel, and it is why the kernel has no
"compatible empty search" arm at all.

## Remaining: the review-sheet migration cuts

The kernel landed in #11669. It does **not** repair the production converge; it provides the
authority a future repair is written against. The remaining cuts were designed but not built, and
they replace the approach in the closed #11596 rather than salvaging it — that PR decided the
meaning of each reading inside effectful routing, which is why it took ten review rounds.

- **Cut 2 (A)** — producers: the shared marker-query constructor, the Drive superseded-marker
  producer, and the subject-bound operator-declaration reader. They emit `LegacyExtentEvidence`
  and call the one complete join. They decide no standing. This cut discharges the kernel's
  enrolled consumer frontier, whose triggers name **execution** rather than import.
- **Cut 3 (C1)** — declared-id admission: reconcile a `LegacyFileSelection` against a standing, a
  contradicting observation, an exact file-by-id preflight and the intended subject. Until this
  runs, a selected file beside an unresolved extent stays unresolved.
- **Cut 4 (C2)** — `UpdateFileProperties` PATCH and the readback that asserts the patched file id
  rather than a count. The field mask must name every required field of its declared response
  type; `fields=id` against a `DriveFile` decodes short and reports a landed write as a transport
  failure.
- **Cut 5 (E)** — legacy-generation retirement.

**Three standing constraints for whoever resumes this.** The closed #11596's combined migration
route must not be executed. The operator's live sheet was repaired by hand, and the ordinary
steady-state adoption path was exercised live — two GETs, zero writes, existing sheet adopted.
And the marker must not be relocated again until the producer and admission cuts are resumed,
because a second relocation with no producer is how the first duplicate was minted.

## Remaining: the census API budget

The census needs a GitHub App installation token to lift its rate limit. This is **not** a
build — it is a reuse, settled across three lanes.

- `#11677` already signs an App JWT: `extdeps.auth.jws` for the RFC 7515 signing input,
  `extdeps.tools.openssl` as an enrolled host CLI, and `github.AppInstallationAccessTokens.Create`
  for the exchange. The key is read from a file, the signing input enters on stdin, the signature
  leaves on stdout; no key material reaches the evaluator or argv.
- **PRIMITIVE-EGRESS-0 admits that route conditionally.** It bans *ambient* primitives — registry
  names and interpreter arms realising pure computation the `.dag` could own. It does not ban a
  modeled external tool whose subject is genuinely external. The condition: the subject must be
  the **key custody boundary**, a signer interface over a key reference with openssl as one
  handler. Modeled as "RSA is realized by openssl" it fails, as an ambient primitive wearing a CLI.
- **Do not add an `rs256_sign` host primitive.** CRYPTO-0 is actively deleting host crypto
  primitives; a new one would be a second authority landing against that program. RSA is not in
  CRYPTO-0 wave 1 and a future modeled RSA is a different subject anyway — key inside the
  evaluator — so a keystore-shaped interface survives it.
- **The shape, from #11677's owner:** a second composed performer in `gunbc.github_effect_perform`,
  not a second module. Factor the `clock → claims → jws → openssl → Create` prefix into one
  function returning a `sole_constructor` installation-token carrier. Each performer consumes the
  carrier inside its own call. Never return a bare `Secret`, and never introduce a second
  environment-variable token a PAT could silently satisfy.
- **The live control** should run with the PAT route unavailable, so a successful code-search page
  is attributable only to the token producer rather than proving a token could be rendered.

Not blocked. #11671, #11677 and #11679 are all merged, so the route this describes is available
to build against today.

## Remaining: ntfy publisher token onto GCP custody

`gunbc.auth.approval_ntfy_deployment` `approval_ntfy_publisher_token_intervention` is a
`HumanIntervention` — an operator creates the ntfy user and records its token where the unit can
read it. The read side is already modeled. Delivery is not.

The design is agreed across three lanes and is **a consolidation of #11679 after it lands**, not a
sibling module: move it to a custody authority and make the controller key its first caller row.

- One fleet-converge mode, with the credential a dispatch choice over a **closed roster of custody
  rows**. One mode per credential grows a hand-enumerated roster — a known stall. A free-text
  credential input would be a stringly selector.
- Each row: `SecretRef`, path, owner principal, group principal, and a **mode arm**. `root:root
  0400` becomes the controller key's row rather than a constant.
- **The directory's owner/group/mode must be a row field too.** A `0640` token for a service group
  needs a directory that group can traverse (`0750 root:<group>`); today's check refuses any group
  bit.
- Mode stays a **closed arm**, never a caller octal. Derive the exact-mode leg and the
  forbidden-bits leg from the arm (`077` for 0400, `037` for 0640); the owner leg takes
  `-user`/`-group` from the row.

**A caveat that must not be lost.** A find-metadata readback establishes which principals *hold
permission bits*. It does **not** establish that a non-writer cannot open the store — that needs an
execution-as-another-user leg. `gunbc.rung_drop.approval_store_single_writer_by_agreement`'s
restoration trigger requires the stronger observation, and its row has since been amended to name a
permission-bits readback among the things that do *not* retire it. The custody work is
necessary-but-not-sufficient for that trigger.

Not blocked. #11679 is merged, so the custody authority this consolidates into exists today.

## Small, unblocked

Seven unused imports across `gunbc.ci_workflow_scan_producer` (`CodeSearchPage`,
`code_search_hits_read`, `GithubObservationRequest`, `ObservationBudgetStanding`) and its witness
(`CodeSearchPageComplete`, `CodeSearchPageTruncated`, `ObservationCompleteness`). All predate
#11656 and were deliberately left out of it to avoid widening a head-pinned repair. One small PR.

## The census rows are historical prototype output, not an authority

`gunbc-private:sheets-census` at `94fa118` holds `strategy.ci_prospect_census` — 57 rows carrying a
17-column set matching the formatted sheet's schema, transcribed rather than derived.

**Label them: historical prototype output; NOT current runner-occupancy, spend, or opportunity
authority.** The economic readings attached to those rows were shown not to have the meanings
assigned to them, and the specific errors are worth carrying because each is easy to repeat:

- workflow **wall duration is not summed runner occupancy** — a run lasting an hour may occupy one
  runner for a minute, or twenty runners concurrently;
- workflow-run **admission delay is not runner queue delay** — they are different waits with
  different causes;
- a **provider declaration can outlive actual provider execution**, so a declared runner label does
  not establish that provider ran the job;
- **provider adoption does not establish positive spend** — a self-hosted or free-tier runner is
  adoption with no bill.

So the derived totals that prototype displayed — a runner-minutes-per-day figure and an ARM-tier
economic projection — do not follow from what was observed, and the "five carry `CostOpportunity`"
classification must not be quoted as a finding. Treat the 57 rows as a shape demonstration.

**The bounded scan producer is necessary and NOT sufficient.** It yields a declared candidate
population; it cannot reproduce these rows or establish any economic reading, because the facts
those readings need are job-level and the scan is workflow-level. The minimal rebuilding route is
job-level observation:

```
jobs?filter=all
  → all pages and attempts
  → actual runner / provider identity
  → job occupancy
  → pre-start observation
  → cancelled occupancy
  → provider execution standing
  → private commercial projection
  → operator-safe sheet rows
```

That arc is carried as its own roadmap item rather than left in this prose.
