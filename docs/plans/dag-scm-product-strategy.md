# Source integration product strategy — agent landing wedge to hosted developer substrate

**Status:** DRAFT for operator review, 2026-07-27. This document is a strategy and promotion
record, not implementation authority. It exists so the long-term hosted-SCM vision, the first
sellable product, and the next experiment cannot silently collapse into one adoption request.

Semantic integration remains owned by
[source-intent integration](dag-scm-design.md). Publication and customer-code access remain owned
by [node/subtree visibility grants](node-subtree-visibility-grants.md). Logical source versus
storage remains owned by
[module identity versus storage](module-identity-storage-binding-design.md). Serving costs remain
owned by the cited `extdeps.pricing.*` and `gunbc.econ.scm_*` carriers summarized in DESIGN. This
document owns only **which product surface is exposed when, what evidence promotes it, and which
business assumptions are still hypotheses**.

The `A0`–`A5` labels below are plan coordinates, not proposed substrate types. Roadmap state stays
in `dag/gunbc/roadmap_authority.dag`; this file must not become a second status ledger.

## 0. The decision in one page

The destination remains ambitious:

> A hosted Daglang development substrate can eventually colocate source authority, semantic
> integration, agents, validation, compute, deployment, infrastructure intent, and design tools
> over one typed graph and one receipt model.

The first product is deliberately smaller:

> A Git-compatible agent landing layer accepts proposed changes, resynchronizes them against the
> current target, safely lands every mechanically determined result, and asks a developer only
> for an irreducible choice about what they want.

The first proof is smaller again:

> Run the same high-concurrency proposal workload through the existing Git path and through the
> Daglang path at one fixed safety contract; show the judgment, invalidated validation, wall time,
> and tokens displaced, then let a design partner observe the result without granting write
> authority.

This is the governing sequence:

```text
prove the mechanism
  -> shadow a real workflow
  -> earn authority to land into an existing Git target
  -> capture native authored proposals
  -> earn authority to host the native source state
  -> offer adjacent compute/agent/infra products
```

No calendar date, amount of code already written, or serving-cost advantage promotes a stage.
Promotion requires the previous stage's named customer and safety evidence. A failed gate narrows,
reworks, or stops the product; sunk implementation cost is not evidence.

## 1. What is being sold

The product is not repository storage and it is not “semantic merge” as an expert feature.

The customer buys:

- less time between an agent finishing useful work and that work landing safely;
- fewer human or LLM judgments spent on synchronization and integration;
- fewer full validation reruns after the accepted target advances;
- fewer clean-looking but semantically wrong combinations;
- one simple submission path instead of merge/rebase/force-push mechanics; and
- an inspectable receipt for every automatic result, question, or terminal refusal.

The product metric is therefore:

> **Judgment and validation work displaced at a fixed safety contract.**

Raw conflict reduction is not a product metric. An unsafe system can manufacture a perfect
conflict rate by choosing silently. Repository count, stored bytes, and Git object throughput are
operating measures, not proof of customer value.

The native human experience remains the source-intent contract:

```text
edit -> submit -> Landed
```

or one localized `Choice required` question with concrete previews and automatic continuation.
`Could not land` remains the plain, non-semantic terminal refusal when no permitted machine-owned
continuation exists. Proof terminology stays behind inspect.

## 2. Why this wedge, and why now

Agent concurrency changes the denominator. A human team may tolerate occasional branch repair
because authoring dominates integration. When many agents propose changes concurrently, integration
and repeated validation become a production line: a green proposal can become stale before it is
admitted, several agents can spend tokens independently rediscovering the same repair, and a
30-minute validation path can be repaid many times.

Two external observations make the wager testable:

- GitHub reported more than 180 million developers and 518.7 million pull requests in 2025, while
  describing coding agents as a material new source of repository activity
  ([GitHub Octoverse 2025](https://github.blog/news-insights/octoverse/octoverse-a-new-developer-joins-github-every-second-as-ai-leads-typescript-to-1/)).
- A July 2026 study of 33,596 agent-authored pull requests across 2,807 repositories found exact
  file overlap in 40.2% of repositories and 79.4% of pull requests over the study window; in a
  one-week window the figures were 53.4% and 95%, and detected textual conflicts were materially
  higher between different agents than within one agent
  ([Divergence of Coding Agent Behavior](https://arxiv.org/abs/2607.04697)). Those textual
  conflicts are a lower-bound observation, not semantic evidence.

Existing products validate that synchronization hurts, while leaving room above text:

| product surface | useful move | boundary this wager tests |
|---|---|---|
| [GitHub merge queue](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/merging-a-pull-request-with-a-merge-queue) | validates queued changes against the latest protected target | queueing and branch updates do not prove source-level joint intent |
| [Sapling](https://sapling-scm.com/docs/introduction/differences-git/) | server-side rebases pushes onto a target and simplifies branch mechanics | a push that needs a merge still needs deeper resolution |
| [Graphite](https://graphite.com/docs/merge-pull-requests) | automates synchronization, restacking, CI, and queued landing | genuine conflicts still return to manual sync/restack resolution |
| [Jujutsu](https://jj-vcs.github.io/jj/latest/conflicts/) | represents conflicts as first-class history instead of blocking every operation | conflict interpretation and resolution still reach the user |
| [GitButler](https://docs.gitbutler.com/overview) | virtual branches reduce worktree and branch friction | branch/hunk organization is still the collaboration abstraction |

Those products are not mistakes and Git is not the enemy. They establish the compatibility floor.
The Daglang claim is narrower: a typed program model, authenticated proposal operations, claims,
effects, dependency closure, and reusable receipts can safely determine cases that a text/branch
system cannot, without exposing the proof machinery to the user.

## 3. The adoption rule — Git-compatible, adoption-light

The first customer must not adopt a language, forge, object store, editor, CI provider, and source
control workflow simultaneously in order to test one integration claim.

The wedge therefore keeps:

- the customer's existing Git repository as the visible source and exit format;
- GitHub or GitLab as the initial review, identity, and collaboration surface;
- existing editors, build systems, and CI;
- ordinary clone, fetch, and build behavior; and
- a conventional branch or pull request as a lower-fidelity proposal inbox when native operation
  metadata is unavailable.

The wedge changes:

- the agent submission protocol first, because an organization can update an agent tool centrally
  without retraining every developer;
- synchronization and replay, which become system work;
- admission, which is target-scoped and exact-parent checked; and
- the default user vocabulary, which contains no merge strategy or rebase ritual.

“Built in Daglang” helps only when it produces visible receipts and dogfoods the same substrate.
It does not buy trust by assertion. Trust is earned through shadow comparison, fail-closed
behavior, reversible Git output, an inspectable/checkable core, customer-controlled code
boundaries, and a growing public falsifier corpus.

## 4. Design-partner profile

The first design partner is not “any developer willing to try a new VCS.” It is a team for whom
the narrow pain is already expensive enough to measure.

Strong selection signals:

- many coding-agent proposals concurrently target one repository or modeled subsystem;
- target movement routinely invalidates green work or forces replay;
- validation is long or expensive enough that reruns are visible;
- the team can provide a baseline of proposal, synchronization, CI, and human-intervention events;
- one technical champion can inspect receipts and distinguish an integration decision from a
  tooling outage;
- the team can begin in read-only shadow mode; and
- the source surface is inside an honestly supported fidelity fragment.

Weak initial fits:

- teams whose work rarely overlaps or whose validation is effectively free;
- buyers seeking only Git hosting price or a general forge replacement;
- repositories dominated by opaque assets with no declared semantic ingester;
- safety-critical production targets before the bounded contract and incident process are proven;
- organizations that require customer source to leave their trust boundary before any shadow
  evaluation; and
- teams expecting arbitrary-language mind-reading.

Discovery may begin before the write path exists. A useful early partner can supply anonymized
workload shapes and replayable fixtures, review the default questions, and run a local shadow
worker. Production write authority is earned later.

The design-partner agreement should state:

- shadow first; no source-target mutation;
- code remains inside the customer's declared placement and publication grants;
- no training or secondary use of source by default;
- raw source is not a business moat;
- derived mechanism gaps or aggregate performance leave the boundary only by explicit consent;
- every result can be inspected and every accepted state can be exported as ordinary Git; and
- the partner can exit without converting repository history or losing source.

## 5. Adoption milestones

The adoption milestones are orthogonal to the semantic plan's `P−1`–`P6` phases. A technical
carrier may land without authorizing a product stage; a product stage cannot promote without its
technical prerequisites.

### A0 — internal proof and public benchmark

**User adoption ask:** none. gunbc is the first workload.

**Authority:** existing Git main remains authoritative. The Daglang path is evaluation-only.

**Build:**

- the layered interaction/scenario corpus;
- a bounded proof-kernel slice over real Daglang proposals;
- a replay harness that presents the identical accepted state, proposal set, safety contract, and
  validation policy to the Git baseline and Daglang path;
- a 50-agent/10-overlap stress realization using measured rather than invented proposal and CI
  costs; and
- a public, inspectable receipt bundle.

**Measures:**

- normative choices per 100 submitted proposals;
- proposals landed without human/LLM judgment;
- false automatic integrations, escaped wrong results, and post-land corrective reverts;
- affected validation intervals reused versus rerun;
- wall time from proposal-ready to accepted;
- human minutes and agent tokens spent on integration;
- safe partial assessments preserved when a batch cannot commit; and
- result distribution by native-authored versus Git endpoint-import fidelity.

**Promotion gate:** at least one real gunbc workload demonstrates lower judgment or repeated
validation than the Git baseline under the same declared safety contract; every automatic result
has its required closure/admission/commit receipt; nearby RED controls defeat always-apply and
always-refuse implementations. No known wrong result may be hidden by aggregate metrics.

**Stop/narrow signal:** the measured advantage comes primarily from queueing, caching, or ordinary
target refresh rather than semantic information. In that case, sell or reuse the smaller landing/
receipt mechanism and do not claim a semantic moat.

### A1 — read-only design-partner shadow

**User adoption ask:** install a GitHub/GitLab integration or local worker with read access to
selected proposal/CI events. The existing workflow remains authoritative.

**Authority:** the customer's existing forge and target ref. The product writes no source refs.

**Build:**

- import ordinary branches/PRs at an honest fidelity profile;
- compute `would land`, one plain-language question, or a terminal refusal;
- compare that assessment with the team's actual merge/rebase/conflict/revert path;
- retain audience-authorized receipts without exporting raw source; and
- let the partner mark whether a proposed question reflected a genuine preference.

**Promotion gate:** the shadow path repeatedly predicts a safe, useful landing or a materially
better localized question; the partner can identify displaced work in its own receipts; false
automatic conclusions remain zero in the declared corpus/bound; and the partner explicitly asks
to grant a narrow write scope.

**Stop/narrow signal:** the partner will not grant read placement under an acceptable trust model,
the supported fidelity excludes the costly cases, or the shadow mostly restates the existing
forge's result.

### A2 — opt-in managed landing into Git

**User adoption ask:** route one protected target through the admission service. Developers and
agents still produce ordinary Git-compatible work.

**Authority:** the existing Git target remains source authority. The service receives only the
right to compare-and-advance that named ref under policy.

**Build:**

- exact-parent Git ref compare-and-swap;
- silent stale-parent replay/re-evaluation;
- independent read-back before `Landed`;
- one-parent/squash commit output;
- idempotent projection repair;
- explicit approval and rollback surfaces; and
- a conventional forge review projection where required.

**Promotion gate:** the managed target runs reliably across a declared observation window; no
normal journey requires rebase, conflict-marker editing, or force-push; operational failures do
not become semantic questions; ordinary clone/build/export and an exit drill pass; and at least
one partner chooses to keep the write path enabled.

**Stop/narrow signal:** reliability, permissions, or Git fidelity dominate the benefit. Keep the
product as shadow/advice until those are solved; do not compensate by requesting source authority.

### A3 — native agent proposal channel

**User adoption ask:** configure selected coding agents to submit authenticated authored
transformations and contracts rather than only endpoint commits. Human editors and Git remain
usable.

**Authority:** the Git target still remains authoritative; native proposal metadata is additional
evidence.

**Build:**

- stable proposal occurrence identity;
- explicit parent, reads/relies, modifies/frame, ensures/guarantees, dependencies, and atomicity;
- continuation support for one answered choice;
- exact authored-operation versus endpoint-delta fidelity in receipts; and
- SDK/tool-protocol adapters for the agent surfaces a design partner already uses.

**Promotion gate:** the native channel resolves or safely localizes materially more costly cases
than the same proposals imported as Git endpoints, without increasing wrong automatic results;
agents require no manual branch-topology operation; and removal of native metadata honestly
downgrades evidence while preserving the Git program.

**Stop/narrow signal:** agents cannot express useful operations/contracts without language-specific
ceremony that exceeds the judgment displaced, or endpoint evidence already captures nearly all
measured value.

### A4 — hosted native source authority

**User adoption ask:** explicitly migrate one target's accepted native state to the hosted
Daglang authority while retaining an ordinary Git projection and tested exit path.

**Authority:** the native target becomes authoritative only through a per-target, authorized
transition. Git becomes an idempotent compatibility projection. There is no implicit authority
flip.

**Prerequisites:**

- A2 and A3 promotion evidence;
- the semantic plan's complete Git fidelity and exact-parent/outbox receipts;
- visibility Stage 0 and the required publication capability profile;
- tenant isolation, placement, authorization, backup, restore, erasure, and incident contracts;
- packed grain-independent storage selected through the landed economics model; and
- at least one design partner whose requested capability actually requires native authority.

**Promotion gate:** migration, steady-state operation, Git projection, backup/restore, and
export/exit drills all pass; no customer code or hidden diagnostic crosses its grant; and the
customer affirmatively prefers the native authority for a measured benefit unavailable in A2/A3.

**Stop/narrow signal:** hosting adds reliability/compliance/privacy burden without increasing
integration fidelity or customer pull. Continue selling the Git-backed landing layer.

### A5 — colocated developer substrate

**User adoption ask:** separately opt into adjacent products that reuse already-trusted source,
claim, effect, and receipt boundaries.

Potential products:

- affected validation and CI compute;
- agent execution/workspace orchestration;
- remote realization of withheld nodes;
- deployment and infrastructure apply;
- policy/admission and audit;
- design, review, and planning tools; and
- compute/resource brokering through the existing fabric.

Each adjacent product must have its own value receipt, authority grant, data boundary, exit, and
promotion decision. Source hosting is a distribution option, not permission to bundle every
capability or observe every workload. Colocation is valuable only when the same modeled facts
remove a real boundary or duplicated computation for the customer.

## 6. Trust is the route to the moat

The moat is not Git object storage, branch UI, or privileged access to customer code. Incumbents
can copy surface mechanics, and private source is a liability unless tightly governed.

Compounding assets that are harder to copy together:

1. **One semantic substrate:** program graph, claims, effects, authority, affected closure,
   proposal contracts, and integration receipts share definitions instead of meeting through text.
2. **Pre-flattening proposal capture:** agent-authored operations survive before Git endpoints
   erase them.
3. **A small checkable kernel and public falsifier corpus:** proposed candidates may come from
   sophisticated procedures or LLMs, while acceptance depends on bounded, inspectable receipts.
4. **Fidelity inventory and graceful downgrade:** native, Git, opaque, and unavailable inputs
   state exactly what conclusions they license.
5. **Reversibility:** every accepted state projects to ordinary Git, so trying the product does
   not create a hostage decision.
6. **Trust and visibility by construction:** customer code, diagnostics, telemetry, and effects
   obey the same grant model.
7. **Operational learning:** aggregate, consented evidence about which model gaps and validation
   costs recur—not a corpus of customer source.
8. **Vertical reuse:** once trusted, the same receipts can remove duplicated CI, agent, deploy,
   and infra work without inventing another control plane.

Open-sourcing or publicly specifying the proposal/receipt/checker boundary is likely additive to
this moat: it makes automatic integration verifiable and lets a customer keep the reasoning path
inside its boundary. The hosted operation, fleet, collaboration service, and enterprise controls
can remain paid realizations. This is a hypothesis to test with partners, not a licensing decision
made by this document.

## 7. Distribution and commercial hypotheses

Word of mouth is an outcome, not an initial distribution plan. The product can reduce traditional
sales burden only after its receipts make value obvious inside the workflow.

Initial distribution:

- gunbc dogfood and a reproducible public benchmark;
- direct design-partner recruitment among agent-heavy teams;
- a read-only GitHub/GitLab app or local shadow worker;
- integrations with agent tool protocols before a new human SCM client;
- concrete before/after receipts shared only with customer permission; and
- a self-hostable/checkable path for teams that cannot expose source.

The first commercial package should price the constrained outcome, not repository storage. Test:

- free/local or no-cost shadow assessment;
- paid managed landing for an active target, repository, or proposal volume;
- usage-based compute only when the product actually supplies compute; and
- enterprise pricing for isolation, policy, audit, support, and hosted authority.

Per-seat pricing may fit human review products but maps poorly to an agent-heavy workload whose
value and load grow with proposals and validation. Pure success-fee pricing is also dangerous
because safety must never be pressured by landing volume. Pricing remains an experiment until
design-partner receipts show the stable value/load unit.

The landed GitLab economics establish an important correction: serving storage can be cheap while
R&D and distribution dominate the company. Low infrastructure cost creates runway; it does not
eliminate product work, trust work, support, or sales.

## 8. Business and product risks

| risk | early falsifier or containment |
|---|---|
| circular trust: an unproven language asks to own source | Git-backed shadow first, public checker/receipts, ordinary Git exit |
| unsafe automatic integration costs more than conflicts | fixed safety contract, active RED corpus, wrong-result accounting before auto-land rate |
| Daglang-only value is too narrow | measure a real agent-heavy Daglang wedge first; do not claim arbitrary-language semantics |
| Git compatibility consumes the roadmap | complete only the fidelity required by the selected partner; typed refusal elsewhere |
| incumbent queue/rebase tools capture most value | identical-workload benchmark separates synchronization/cache benefit from semantic benefit |
| customer-code “visibility” becomes surveillance | no training/secondary use by default; placement and publication grants; derived opt-in telemetry |
| hosted authority creates reliability/compliance burden too early | A4 is customer-pull and exit-drill gated; A2/A3 remain viable standalone products |
| agents produce unverifiable intent metadata | authenticated contracts are evidence, never self-licensing; parent policy and checker remain authoritative |
| vertical platform becomes an adoption bundle | each A5 capability is separately opted into and value-gated |
| no-sales assumption delays learning | direct design partnership is required before write authority; word of mouth follows proof |

## 9. Decisions to litigate

These remain explicit decisions; implementation must not choose them accidentally.

1. **First external surface.** GitHub matches the current dogfood repository; GitLab may offer a
   better first partner. Choose from the first qualified partner and available event/ref fidelity,
   while keeping the import interface forge-neutral.
2. **Open boundary.** Recommended hypothesis: public protocol, checker, receipts, scenario corpus,
   and self-hostable shadow path; paid hosted operation and enterprise controls. Validate whether
   this materially increases design-partner trust.
3. **First partner count.** Recommended hypothesis: one deeply instrumented partner, then two
   discriminating partners with different repository/agent shapes before general availability.
4. **Pricing unit.** Compare active target/repository, admitted proposal volume, and supplied
   compute. Reject a unit that rewards unsafe landing or hides an unbounded request/storage cost.
5. **Language scope.** Start with lossless Daglang. Add a surface only when its declared fidelity
   supports a named partner workload; never advertise general semantic merge from a parser alone.
6. **Authority flip.** Recommended rule: no hosted-native target until one successful managed-Git
   partner explicitly requests a capability that cannot be delivered with Git remaining
   authority.
7. **Customer data.** Recommended rule: source never becomes training or cross-customer product
   data by default. Only consented, audience-safe aggregate receipts may inform the mechanism
   roadmap.
8. **Review surface.** Determine whether the forge PR remains the default review UI through A3 or
   whether one plain proposal/choice surface displaces it for agents first.

## 10. Immediate work from this strategy

The next practical sequence is:

1. finish review of the C0 native interaction carrier and its journey witnesses;
2. land P−1 shared claim-indexed evidence before an SCM-local evidence vocabulary appears;
3. choose a bounded P1 kernel slice with real gunbc proposals;
4. define the A0 replay corpus and capture the current Git/CI/token baseline before optimizing it;
5. publish the benchmark method and receipts, including cases where Daglang refuses;
6. recruit one design partner for workload discovery and read-only shadow requirements;
7. build only the minimum forge/local adapter that partner's fidelity and placement require; and
8. return to this document with the A0/A1 receipts before authorizing a managed target, native
   object store, or forge UI.

The child C0 implementation in PR #7334 is model evidence for step 1. It does not by itself prove
the kernel, customer value, Git fidelity, or authority boundary.

## 11. Strategy acceptance and dissolution

This strategy is accepted when the operator has ruled on the eight decisions above and the roadmap
contains the A0/A1 wedge, managed-Git promotion, hosted-authority gate, and vertical-platform
option in dependency order.

It is falsified or narrowed when the receipts show that:

- ordinary queue/rebase/cache mechanics capture the measured benefit;
- the supported semantic fragment does not cover a costly partner workload;
- safe questions or automatic results are not materially better than the baseline;
- the trust/adoption cost exceeds the displaced work; or
- no qualified partner requests write authority after shadow use.

Sections dissolve as their authority transfers:

- workload and value measures -> cited benchmark/workload carriers and executing receipts;
- interaction rules -> source-intent carriers and witnesses;
- design-partner access/privacy -> visibility, grant, and placement carriers;
- Git/native authority transitions -> target/admission/projection carriers;
- serving and packing -> existing economics/storage carriers; and
- commercial choices -> operator-signed roadmap/decision carriers once tested.

Delete this document when A4 has either been accepted with an executing hosted consumer or
explicitly abandoned, and the remaining A5 options live as independently priced roadmap nodes.
Until then, update this plan instead of opening another SCM business-strategy ledger.
