# Source-intent integration — a first-principles SCM design

**Status:** operator-directed design seed, revised 2026-07-27. This revision supersedes this
file's 2026-07-25 premise that “merge = keyed diff over node identity.” It preserves the signed
visibility-first sequence and the landed economics corpus, but treats nodes, files, branches, and
three-way comparison as possible evidence or realizations—not as the foundation.

“Source-intent integration” is a descriptive name in this plan, not a proposed substrate type.
No carrier lands from this document alone. The first implementation step remains concept-DFS in
`std/` and a discriminating scenario corpus; the plan must not create an SCM-local nickname for a
concept the substrate already owns.

Roadmap carrier: ROADMAP §2, “SCM — source-intent integration, visibility-first.” Economics
grounding: DESIGN open thread “SCM economics — the GitLab 10-K corpus.” Visibility authority:
[node/subtree visibility grants](node-subtree-visibility-grants.md). Storage/surface authority:
[module identity vs storage](module-identity-storage-binding-design.md). Product wedge, design
partner, hosted-authority promotion, and adjacent-product sequencing:
[SCM product strategy](dag-scm-product-strategy.md). That strategy cannot weaken this document's
semantic, evidence, admission, or compatibility-fidelity contract; this document cannot promote a
product stage merely because its technical carrier exists. Git is the first adoption realization,
not the interface from which the native model is derived.

## 0. User contract — SCM-compatible, not SCM-shaped

The proof machinery below is an implementation contract, not product vocabulary. A developer
should not need to understand candidate closure, evidence relations, patch transport,
linearizability, three-way merge, or which compatibility realization is active. The native
workflow is:

> **edit → submit → Landed**

Synchronization, replay against the current accepted target, integration, validation, receipt
reuse, atomic advance, and compatibility projection are system responsibilities. A normal user
never chooses merge versus rebase, maintains branch/bookmark/channel topology, edits conflict
markers, or interprets proof/evidence terminology.

At the semantic integration boundary, there are only two semantic handoffs:

```text
Landed
The requested change was safely accepted.

or

Choice required
When foo was renamed to bar, should this new call follow that rename,
or should a separate foo remain?

[Follow the rename: bar(...)]  [Keep a separate foo(...)]
```

`Choice required` means that more than one materially different requested meaning remains, or that
two explicit requested outcomes cannot both be preserved. It asks exactly one localized question
in domain language, presents concrete alternatives with result previews, and retains enough
continuation state that one answer resumes the operation automatically. It never asks the user to
perform integration mechanics.

Missing observations, stale parents, incomplete models, retryable failures, and projection repair
remain internal work. The system exhausts its declared mechanical observations and bounded proof
procedures before interrupting. A machine-answerable issue stays inside the machine. Missing
information becomes a user question only when the missing information is genuinely a preference
that no observation can answer. The precise receipt and mechanics remain available on demand
through an explicit inspect/debug surface.

A third **non-semantic terminal outcome** is necessary for fail-closed honesty:

```text
Could not land
This change is not permitted to alter the production access policy.
Approval was not granted. No files were changed.
```

`Could not land` is not a choice and never converts a machine deficit into user work. It appears
only after the system has exhausted or ruled out every permitted retry, observation, repair,
approval route, or other machine-owned continuation. Internally it retains a typed, located,
counted refusal and an audience-authorized receipt reference; the primary message gives the plain
domain reason, who or what owns any real next action, and confirms that no transition committed.
Permanent authorization denial, policy refusal, unsupported proof bounds, and non-retryable effect
failure therefore cannot disappear into an inspect-only state. Non-terminal progress may be shown
as status, but it is not a semantic conclusion or a request for preference.

The first compatibility surface may look like an ordinary Git worktree and accept ordinary
commits or PRs, but Git is one import/export and target realization, not the native interaction
model. A submit operation imports the pending delta/proposal, replays it against the current
accepted target, and emits the target medium's ordinary representation when it lands. The initial
product emits an ordinary one-parent/squash Git commit; Mercurial and Pijul are modeled up front as
design-stress realizations, not silently promised as product-grade adapters. Ordinary native SCM
clients and expert plumbing remain available; none is the conceptual front door.

The interaction contract has executable acceptance conditions:

- no normal journey requires merge, rebase, cherry-pick, reset, conflict-marker editing, or
  force-push;
- no primary user message contains `candidate`, `closedness`, `evidence grade`, `reconciliation`,
  `constraint`, `unknown`, or `contradictory`;
- every user question is a genuine normative choice in domain language;
- every machine-answerable issue stays inside the machine;
- one answer resumes the suspended operation automatically; and
- every terminal refusal produces a plain `Could not land` message backed by a typed, located,
  counted internal refusal; and
- every accepted result remains exportable through each compatibility realization whose declared
  fidelity and capability profile the target requires; the initial product requirement is ordinary
  Git.

The first scenario design therefore includes both the internal layered receipt and this minimal
presentation projection. Internal state is retained and inspectable, but internal terminology does
not leak into the default UI merely because it exists.

## 1. Product objective — minimize judgment without pricing safety

The product thesis is:

> **The user makes changes and states preferences. The system owns integration. It lands every
> safely determined result and interrupts the user only for an irreducible choice about what they
> want; when policy, authority, or a terminal system limit makes landing impossible, it says so
> plainly without pretending that failure is a preference.**

The formal objective underneath that thesis is:

> **For the declared admission contract and abstraction, return the most informative conclusion
> whose soundness is established; claim uniqueness only when query-local candidate and evidence
> completeness are certified.**

Humans and LLMs are both expensive, lossy judgment providers. They should receive only the
irreducible normative choices. Everything else—alignment, replay, independence, invariant
checking, affected-set calculation, receipt reuse, and projection—should be mechanical.

The objective is **lexicographic**, not a scalar score in which a sufficiently cheap unsafe answer
can win:

1. hard constraints: authorization, semantic obligations, evidence soundness, confidentiality,
   and an exact-parent transition;
2. maximize safe information and work preserved: established claims, reusable receipts, and
   independent proposal groups whose disposition is known;
3. minimize human/LLM judgment, invalidated CI, compute, latency, storage, and egress; and
4. preserve Pareto-incomparable choices instead of inventing weights.

The motivating workload is an explicit **stress profile**, not a universal constant: 50 agents per
developer, 10 proposals touching the same modeled region, and a 30-minute CI path. Every proposal
invalidated after it was green spends at least another affected 30-minute validation interval;
serial invalidations also add wall-clock queue delay. The first model must carry these as workload
inputs so the benefit is measured rather than asserted.

A lower textual-conflict count is not itself success. An aggressive integrator can improve that
number by silently choosing wrong results. The product metric is **judgment displaced at a fixed
safety contract**. Safety is a prerequisite, not a finite cost. Even when a batch cannot commit,
the system returns every independently grounded assessment; a top-level unknown must not erase
safe partial deductions.

## 2. Separate the layers

Every SCM exposes its own useful native concepts: Git has commits/trees/refs, Mercurial has
changesets/manifests/bookmarks/phases, and Pijul has changes/dependencies/channels/pristine state.
Those are upstream storage, history, publication, and compatibility concepts. None is the program,
and none supplies the vocabulary of the native integration interface.

This lane keeps the following layers separate:

| concern | authority | never allowed to decide |
|---|---|---|
| recorded fact | a typed observation or authored artifact plus provenance | the claim it happens to resemble |
| claim + evidence relation | a proposition about a named subject, scope, target, and bound, linked to applicable supporting or challenging facts | “some truthy facts exist” |
| program model | the actual `Node` + `Edge` graph, claims, effects, and bounds | file layout, line position, or formatter output |
| proposed change | an authenticated authored transformation plus its explicit contract, dependencies, authority, and provenance | an inferred story about a user's unexpressed preference |
| semantic deduction | the closed zero/one/many or unclosed joint-result calculation | authorization, mutation, queue order, or path overlap |
| admission policy | required claim assessments and actor authority under the accepted parent policy | semantic satisfiability or realization cost |
| transition realization | exact-parent compare-and-advance, effects, and read-back | whether the semantic candidate space was closed |
| history + projection | grounded receipts over accepted states and downstream media such as Git, Mercurial, or Pijul | a second copy of the program or a retroactive semantic verdict |
| medium / transport | `.dag`, Rust, markdown, files, SCM-native stores, CLI/REST, remote storage | semantic correctness beyond the medium's declared decode fidelity |

Text and files have two downstream jobs, already named by the storage-binding design:

- **capture:** lift an edit made through a surface into a proposed graph transformation, with an
  explicit fidelity verdict; and
- **projection:** render the accepted graph through the declared medium.

A path move, import deletion, formatting pass, or source reorder may therefore be a semantic no-op
**under a declared language model and equivalence witness** even though its Git tree changes.
Conversely, two disjoint textual hunks may jointly violate one modeled invariant.

## 3. Grain comes from groups of units; SCM does not choose one

The recent group-of-units lane is a starting constraint, not yet a group algebra.
`gunbc.roster_registry.GroupMembership` is a **membership-provenance taxonomy** on a
`RosterRegistration`; it distinguishes membership known:

- **by containment**;
- **by derivation**;
- by a declared, counted **frontier** with a reason and dissolution trigger; or
- by an ungrounded nickname, which is the violation.

It does not itself carry a group, its members, closure semantics, or a polymorphic membership
relation. This is the useful content behind the working shorthand `Group<U>`; this plan must not
pretend the shorthand has landed or mint a competing generic group type merely because it is
convenient.

Module membership is containment-derived; `GroupMembership.ByContainment` classifies the
provenance of that membership. The namespace/containment authority must still supply the actual
member relation and distinguish immediate children from descendant closure. An affected closure
is derivation-backed membership. A temporarily hand-curated migration set is frontier membership.
Files are storage realizations of such groups, not group authorities.

The integration algorithm is therefore **unit- and group-polymorphic**:

- a unit may be a declaration, binding edge, argument edge, claim, grammar row, effect, or another
  grounded fact;
- a proposal may affect one unit or a derived group of units;
- the evidence needed to align two observations depends on the unit's actual authority; and
- “same node,” “same module,” and “same file” are observations at different grains, never universal
  conflict definitions.

Identity remains involved, but only as detective evidence. A key can align two facts through time;
it does not decide what their joint result should be. No universal durable `ScmEntityId` is assumed.
Containment position, declaration reference, content identity, normalization, bounded extensional
identity, provenance, and an authored transformation can each supply different grades of evidence.
Unknown alignment stays unknown.

## 4. The native transaction is a proposed transformation, not an endpoint diff

A native proposal should eventually carry, by reuse of existing substrate concepts:

- a stable proposal **occurrence identity**, distinct from transformation/content identity;
- the authoritative target it proposes to advance;
- the exact accepted parent state it was authored against;
- the modeled transformation—what facts or relations it proposes to change;
- `reads` / `relies`: facts whose stability the proposal assumes;
- `modifies` / frame: the facts and scopes it is authorized to change;
- `ensures` / guarantees: facts established after application;
- claims, effects, resource bounds, and observable behavior relevant to admission;
- atomicity groups and whether proven-independent groups may commit separately;
- explicit dependencies and authored supersedes/cancels edges;
- authority to submit against the target/scope and, separately, to change policy, equivalence,
  publication, or approve world-writing/destructive effects;
- a derivable affected group and dependency closure;
- provenance, including the authoring surface if one captured it; and
- capture/decode fidelity.

These are **roles**, not a proposed `Intent` record. `EffectAttemptIntent`, `StandingIntent`, graph
intent in the orchestration lane, `ChangeSet`, temporal snapshots, and other neighboring concepts
must be DFS'd before any new carrier is named.

Here, **source intent** means only an authenticated authored transformation plus its explicit
contract and dependencies—not a reconstructed mental preference. “Obligation-preserving source
integration” or “contract-preserving reconciliation” is the narrower formal substrate even if the
product lane keeps the source-intent name.

An endpoint diff remains valuable, but it answers only “what observations differ?” A native
proposal answers “what transformation was requested, against which accepted state, and under
which obligations?” Capturing the operation before flattening it to two snapshots removes much of
the detective work a later three-way merge must redo. A frame condition is what makes “no extra
change” checkable: every result delta must be required by an admitted proposal or be an expressly
permitted canonicalization, never an unrelated solver repair.

### P−1 shared prerequisite — claim-indexed evidence

The dashboard/Codex incident and source integration require the same missing relation. A recorded
fact is not evidence in isolation. It becomes evidence only for a named claim under an explicit
inference rule, with a maximum conclusion:

```text
recorded fact
  -> evidence link {
       claim(subject, target, proposition, scope, bound),
       supports | challenges,
       inference rule,
       authority + provenance,
       freshness,
       fidelity,
       probe independence,
       maximum conclusion
     }
  -> claim assessment { supporting evidence, challenging evidence }
  -> admission/readiness policy over required assessments
  -> SCM outcome or dashboard lamp as a projection
```

For example, `codex login status` may support `CredentialRecorded` for one `CODEX_HOME`; it cannot
support `ProviderReady`. A successful live request can support a provider/request-capability claim
for its provider, scope, and freshness bound. A worker PID supports lifecycle facts, not provider
readiness. In SCM, a parse edge, typecheck, behavioral receipt, Git rename score, and native
authored operation are likewise facts with different claim relations and maximum conclusions.

The information-state floor for one claim preserves two independent bits—support and challenge:
support only, challenge only, both/conflicted, or neither/insufficient. This is the
[Belnap–Dunn](https://doi.org/10.1007/978-94-010-1161-7_2) shape for incomplete and inconsistent
information, not permission to reuse that logic's final type names before DFS. Conflicting
evidence is not semantic contradiction, and absence of evidence is not refutation.

[SACM](https://www.omg.org/spec/SACM/) is the external precedent for artifacts becoming evidence
through a claimed relation to a subject claim. [W3C PROV](https://www.w3.org/TR/prov-dm/) supplies
entity/activity/agent provenance; provenance helps assess an evidence link but does not establish
the target claim by itself.

The repo has partial precedents, not this shared carrier:

- `std.realization_reconcile.Grounding = Grounds | DoesNotGround` collapses challenge and
  insufficient evidence;
- `gunbc.readback_independence` already supplies one inference-validity rule: a positive result
  from a probe that may establish the subject is insufficient, while a negative result may remain
  informative;
- `std.upsert_decision.ObservationVerdict` preserves several world-state distinctions but is
  desired-versus-observed specific; and
- `std.observation` already owns process-progress events, so this concept must not overload
  `Observation`.

The shared evidence carrier lands in a **separate model-first PR** with three discriminating
consumers: Codex provider readiness, `gunbc.os_install_deduction`, and source integration. It is
P−1 for SCM P0. This plan names roles, not final carrier names, so the first DFS can converge on
existing authorities rather than extract three local versions later.

### Native history

Authority is **target-scoped**, not a property of a branch name. A project may have a primary
target, supported-release targets, or another explicitly admitted target, but each target has one
current accepted state and each proposal names exactly one target. A Git ref, Mercurial bookmark or
branch head, and Pijul channel are different upstream projections of target-like roles; checking
out, updating, switching, pulling, or receiving one does not make it the native authority.

The authoritative history for one target can be a linear sequence of accepted transitions:

```text
target T at accepted state S_n
  + explicitly selected transformations authored against known parents
  + current evidence and admission policy
  -> accepted transition receipt R_(n+1)
  -> target T at accepted state S_(n+1)
```

Workspaces may be concurrent, but branches and merge commits are not fundamental. A workspace is a
sandbox containing pending proposals. Its construction history is evidence, not accepted history.
Acceptance records the target, exact accepted predecessor, each proposal's authored parent, the
applied transformation set, the resulting state hash, and the evidence/receipts that grounded it.
Git commits/trees, Mercurial changesets/manifests, and Pijul channel/change state are a third thing:
compatibility projections or capture evidence for those facts.

This is not “last writer wins with a nicer log.” Before a stale-parent proposal can advance current
state, the system re-evaluates its transformation against current modeled state. Arrival order is
not intent. If two compatible proposals admit witnessed transports that form a commuting square,
validation may run in parallel and either serialization must yield the same declared-equivalent
result. If order changes meaning and no policy declares an order, the system asks rather than
letting queue timing choose.

Native transactions therefore do not require three endpoint snapshots as their primitive. The
authored parent remains necessary causal evidence; the operation is replayed and verified on the
new state. Three-way comparison remains necessary at a **lower-fidelity import/capture boundary**
where only base, left endpoint, and right endpoint survived.

No wall-clock fact supplies semantic order. Commit time, authoring time, upload time, observation
time, queue position, and “objectively arrived first” may be retained as audit or performance
evidence, but cannot rank proposals or recover intent. A transition precedes another because an
authorized receipt names its accepted predecessor; one proposal depends on another only through an
explicit parent/dependency edge. A declared domain-time contract may decide whether evidence is
fresh enough to use, but never turns chronology into proposal priority.

The accepted parent's model, claim set, inference rules, equivalence, and admission policy are
frozen for one evaluation. A proposal cannot license itself by deleting the invariant that rejects
it, broadening equivalence until alternatives collapse, or weakening its own evidence requirement.
A policy/model transition is evaluated under the accepted parent policy, separately authorized,
committed first, and affects only subsequent admissions.

### Deduction contract — one fold, not an interaction catalog

Native integration is one general reconciliation operation over modeled facts, not a growing
dispatch table for “rename versus call,” “move versus edit,” or every future pair of language
features. Conceptually:

```text
assessments =
  fold(applicable claim-indexed evidence, empty, compose checkable assessments)

proposal contract =
  fold(selected proposal occurrences, empty, compose contracts + dependency/atomicity graph)

candidates =
  bounded_closure(
    accepted target state,
    admitted authored transformations,
    witnessed context transports,
    declared canonicalizations,
    finite recovery alternatives
  )

semantic result =
  classify(quotient(candidates satisfying proposal contract, declared equivalence),
           closure certificates)

admission =
  decide(parent policy, authority, required claim assessments, semantic result)
```

The proposal fold accumulates transformations and obligations; it does **not** apply a list in
arrival order. “One fold” means one public reconciliation operation and a stable proof-composition
kernel, preferably with a small trusted checker. A domain may contribute ordinary facts, claims,
proof rules/procedures, and checkable certificates through shared interfaces. It does not add a
feature-pair arm to an SCM switch. A solver or LLM may propose a candidate or certificate; an
unverified proposal has no authority.

Safety, explicit obligations, grounding, and the required evidence profile are admission conditions,
not weighted preferences. A cheaper result cannot buy permission to violate one. Among candidates
that meet those conditions, preserve every fact whose change is not grounded. By default, cost
selects only among realizations of the **same semantically determined result**; it cannot decide
which materially different result the user intended. A contract may make a cost axis part of
intent explicitly, but absent that authority, several result equivalence classes are `Ambiguous`
even when one looks cheaper. Within one class, the declared cost order eliminates only strictly
dominated realizations and preserves incomparable alternatives rather than inventing scalar
weights.

New models participate by contributing their ordinary facts, relations, claims, effects,
equivalence, bounds, costs, inference procedures, and certificates to this kernel. The worked
examples below are discriminating inputs for the same operation, not implementation branches. If
adding a language concept requires a new pairwise integrator case rather than a shared semantic
interface, the missing work is a model/algebra seam to ground—not a twentieth-year special case to
preserve.

### Candidate closedness is a named substrate gap

`v2.std.find_witness.CandidateSet` accepts a caller-supplied finite list and a structural
closedness witness. `UniqueOnly` counts passing members of that list; it neither generates source
integration results nor proves the list exhaustive. Today `solve_constraints` constructs a
singleton candidate set containing the constraint root and derives “closedness” from that root
being well formed. That is a useful scaffold, not authority that every relevant integration result
was considered.

Uniqueness therefore requires separate, checkable certificates for:

1. **capture completeness** — every authored operation available through the declared input surface
   was recovered, while unavailable operation history stays unavailable;
2. **affected-scope closure** — every dependency relevant under the named relation and bound is
   included;
3. **rule closure** — every applicable inference/integration rule in the declared fragment was
   considered;
4. **candidate-generation closure** — every legal composition/transport in the bounded fragment was
   enumerated;
5. **admissibility** — every survivor satisfies its required claims and proposal contracts;
6. **equivalence-quotient validity** — the declared equivalence is justified and congruent enough
   for this admission context;
7. **multiplicity** — zero, one, or many equivalence classes remain; and
8. **commit validity** — the exact parent/model/policy/evidence inputs validated are still current.

P1 is deliberately a bounded, decidable fragment. Its candidate space is the finite closure of the
accepted state, admitted authored transformations, witnessed transports/rebases, declared
canonicalizations, and finitely enumerated recovery alternatives—not arbitrary synthesized
programs. Exhausting a candidate, proof-search, or equivalence bound yields an unclosed result,
never contradiction and never “the one candidate found so far.”

This objective follows abstract interpretation's vocabulary of
[sound approximation](https://cs.nyu.edu/~pcousot/COUSOTpapers/POPL77.shtml) and query-relative
[completeness](https://doi.org/10.1145/333979.333989): compute the most precise sound conclusion
available under a declared abstraction, and claim completeness only for the query and fragment
whose certificates establish it.

### Fail-closed deduction — ignorance never becomes an answer (DESIGN §5)

Daglang's advantage is information density: types, bindings, containment, effects, claims,
resource bounds, dependency closure, and execution receipts can all be premises that Git does not
have. That supports more deductions; it does not grant permission to infer an unexpressed premise.

This is not a new “humility” principle. It is DESIGN §5's existing rule—never fabricate a
plausible output; a failure arm refuses, never widens—applied to reconciliation. Returning the one
candidate the current model happened to expose, while silently treating every unmodeled fact as
irrelevant, would conflate ignorance with an answer in exactly the absorbing-fallback shape §5
forbids.

> A modeled fact `X` licenses consequences of `X`. It does not prove that no relevant, unmodeled
> fact `Y` exists.

The change set is closed in one useful sense: every accepted result change must be grounded in an
admitted proposal or declared canonicalization. The world of relevant meaning is not automatically
closed merely because the program typechecks. A closed coproduct closes its declared axis; a
derived affected/dependency closure closes only the relation and bound its evidence names; a
lossless surface closes only its declared construct set. A unique semantic result therefore
requires the specific certificates above, not a claim that all user intent or all program behavior
has been modeled. If a potentially relevant axis, dependency, effect, capture fragment, or proof
bound is missing, the semantic result space is unclosed at that evidence profile.

This is also how the model can evolve without making old confidence implicit. An admission receipt
names the model/contract inputs and closedness evidence it relied on. Adding a relevant model fact
invalidates the affected receipt and causes re-evaluation; absence in the older model never becomes
negative evidence by inertia.

Two live repo precedents carry the same fail-closed rule:

- `gunbc.os_install_deduction` carries an attested `observed_at` timestamp, but the timestamp does
  not strengthen the runtime verdict. Even a visible login prompt yields
  `KvmSuggestsOsBooted`, not `RuntimeOsInstalled`; the stronger conclusion is reserved for
  independent read-back evidence.
- `gunbc.membership_reconcile` turns a removed member with absent ownership evidence into
  `MemberTeardownRefused { cause: OwnershipUnknown }`. There is no effect arm that can reinterpret
  “ownership was not modeled” as permission to destroy the member.

These are precedents for the evidence discipline, not claims that source integration is an
OS-install or membership-diff algorithm.

## 5. Evidence, semantics, policy, transition, and projection stay separate

The prior four-arm sketch mixed deduction, policy, mutation, and execution state. These are
illustrative **roles**, not final carrier names:

```text
ClaimAssessment
  = SupportOnly | ChallengeOnly | BothConflicted | NeitherInsufficient

SemanticResultSpace<Result>
  = ClosedZero { witnessed_core, counterexample }
  | ClosedOne { result, closure_certificates, obligation_receipts }
  | ClosedMany { equivalence_classes, localized_blocking_question }
  | Unclosed { missing_certificate_or_exhausted_bound, partial_assessments }

AdmissionDecision
  = Admitted
  | Unauthorized
  | ApprovalRequired
  | PolicyRefused

TransitionRealization<Receipt>
  = Committed { receipt, independent_read_back }
  | RetryStaleParent
  | EffectFailed

ProjectionStatus
  = Projected
  | ProjectionPending
  | ProjectionFailed
```

The distinctions are load-bearing:

- `ClosedZero` is semantic unsatisfiability. This is the narrow meaning of **Contradictory**.
  Conflicting observations are `BothConflicted` evidence, not semantic contradiction.
- `ClosedOne` is a **UniqueGroundedResult**, not yet `Applied`. It may still be unauthorized,
  require approval, fail an effect, or lose the exact-parent race.
- `ClosedMany` is **Ambiguous** only after candidate closure is certified. It returns material
  alternatives and a localized blocking normative question.
- `Unclosed` is **Unknown**. Finding one plausible candidate without the required certificates is
  still unknown; exhausting a bound is unknown, not contradiction.
- **Applied is reserved for a successfully committed and independently read-back transition.**
  A unique result plus `Unauthorized` is never Applied. Losing the compare-and-advance race is
  `RetryStaleParent`, not Unknown. A committed native state whose Git export fails remains committed
  with `ProjectionFailed`/`ProjectionPending`; projection failure is not semantic rollback.

The primary surface does **not** expose the internal buckets. It projects the layers through the
§0 boundary:

```text
PrimaryOutcome
  = Landed { result_preview }
  | ChoiceRequired { one_domain_question, concrete_previews, continuation }
  | CouldNotLand {
      plain_domain_reason,
      responsible_party,
      real_next_action_or_none,
      typed_refusal_receipt
    }
```

A committed/read-back result becomes `Landed`. A closed-many result, or mutually incompatible
explicit requests, becomes `ChoiceRequired` only when a genuine user preference can discriminate
the alternatives. Those are the only two **semantic handoffs**. Unclosed search, missing
machine-observable facts, stale-parent retry, authorization routing, effect retry, and projection
repair remain internal work or non-semantic operational status while a permitted machine-owned
continuation exists—not questions disguised as work delegation. One answer adds the authenticated
preference and resumes the stored continuation automatically.

When no permitted continuation remains, the typed refusal projects to `CouldNotLand` rather than
vanishing into inspect state. A permanently unclosed fragment reports that the requested change
cannot yet be checked safely; denied authority reports that the change is not permitted; a
non-retryable effect failure reports the failed domain action. The wording never exposes the
internal result-arm name, never asks the user to perform reconciliation, and never implies a
preference could fix a non-preference failure. The receipt preserves the exact typed location and
cause for inspection, counting, and repair.

Receipts and inspect APIs preserve every underlying state. Every group keeps its partial
assessments even if the batch cannot partially commit: for example group A may be unique and
authorized, B closed-contradictory, C unclosed for missing live evidence, and D ambiguous. Partial
commit additionally requires a declared atomicity split and proven independence.

The admission policy states the required evidence profile and authority. A structurally universal
result must not be presented as behaviorally safe; a behaviorally bounded result names its bound.
Humans or LLMs may answer a genuine normative choice by submitting another authenticated
proposal/claim. Machine-observable model or evidence deficits stay with the system. A response does
not retroactively turn a guess into evidence.

A diagnostic promises only what it can actually compute. This plan chooses **subset-minimal within
a declared diagnostic bound** for an incompatible core; “minimal” never silently means globally
minimum-cardinality. “Localized blocking question” is the default phrase. A future diagnostic
optimizer may choose a lowest-cost discriminating observation/question only under an explicit
cost policy.

### Laws for the certified-compatible region

Parallel proposals are context-indexed patches, not context-free functions. A proposal authored
against `S` may apply only to `S` or through a witnessed transport into the other proposal's
context. Compatibility means a grounded commuting square—or joinability/confluence modulo the
declared equivalence—not literal equality of unchanged `P ∘ Q` and `Q ∘ P`. This is the
[patch-theory](https://darcs.net/Theory/MergersDocumentation) distinction between patch, context,
commutation/transport, and merge.

The laws are:

- **duplicate-delivery idempotence:** the same proposal occurrence ID contributes once; an
  arbitrary transformation such as increment/append need not be idempotent;
- **context validity:** a proposal applies only to its authored parent or through a witnessed
  transport;
- **commuting-square law:** certified-compatible parallel proposals transport to one
  declared-equivalent result;
- **batch-partition invariance:** changing batch boundaries does not change the result in the
  certified-compatible region;
- **grounded delta:** every result delta traces to an admitted proposal or expressly permitted
  canonicalization;
- **contract and frame preservation:** every admitted proposal's relies/ensures/guarantees,
  modifies authorization, dependencies, and atomicity contract hold;
- **n-way obligation preservation:** pairwise compatibility is insufficient; the whole selected
  set must satisfy cardinality, resource, security, and other joint invariants;
- **closedness-qualified uniqueness:** uniqueness is claimed only over the certified candidate and
  evidence scope;
- **exact-parent checked:** replay states what changed since authoring and never applies against an
  assumed parent;
- **parent-policy immutability:** a proposal cannot change the policy, equivalence, inference
  rules, or claims used to admit that same proposal;
- **grant non-widening:** integration never widens a grant or policy merely because a result is
  structurally or behaviorally valid;
- **metadata-inert:** erasing or permuting timestamps, arrival metadata, queue order, or transport
  envelope IDs cannot change the semantic result; and
- **order honesty:** absence of a unique transported result remains closed-zero, closed-many, or
  unclosed; first arrival never supplies the missing authority.

[CRDTs](https://inria.hal.science/inria-00609399) are a useful convergence comparison, while
[invariant confluence](https://arxiv.org/abs/1402.2237) names when application invariants permit
coordination-free composition. Neither licenses a universal merge law outside the declared
operations and invariants.

## 6. Worked examples

### 6.1 Different call arguments — often compatible, never assumed compatible

Accepted source:

```dag
foo(a, b)
```

Proposal P changes the argument bound to `foo`'s first parameter:

```dag
foo(c, b)
```

Proposal Q changes the argument bound to its second parameter:

```dag
foo(a, d)
```

A text merge sees two edits to one line and will ordinarily refuse. A node-key rule might say the
call node overlaps and also refuse. Neither answer uses all the available model.

The program graph can instead observe two distinct argument-binding edges. If it also establishes
that:

- both edges still target the same resolved `foo`;
- each replacement inhabits its parameter type/refinement;
- the combined call satisfies cross-argument preconditions;
- effects and resource claims remain admissible; and
- every required behavioral claim holds within its declared bound,

then witnessed transports can form a commuting square and the sole grounded semantic result is:

```dag
foo(c, d)
```

If `c` changes overload resolution, or `foo` requires the two arguments to share a region, the
result may be contradictory. If both rebinding interpretations are valid and materially different,
it is ambiguous. If the relevant relationship is not modeled, it is unknown. “Different argument
keys” is evidence that starts the check, not permission to skip it.

The reverse is equally important: two transformations touching the **same** coarse key are not
automatically contradictory. They may be identical, act on independent substructure, or normalize
to one transformation. Key intersection identifies where deeper reconciliation is required.

### 6.2 Rename plus a new reference — textual clean merge, semantic question

P renames declaration `foo` to `bar`. Q, authored concurrently, adds a call spelled `foo()` in a
different line or file. Git can merge this cleanly while leaving a broken reference.

With binding provenance, the native possibilities are explicit:

- Q targeted the same declaration and the rename transports the reference → `ClosedOne`, and after
  authorization plus exact-parent commit, `Applied`, emitting `bar()`;
- Q explicitly requires a distinct/public name `foo` to remain → `Contradictory`; or
- endpoint import cannot establish which declaration Q meant → `Ambiguous` or `Unknown`.

The system does not infer “they probably meant the renamed function” from spelling similarity.

### 6.3 Disjoint edits with an emergent invariant failure

P lowers one authorization threshold. Q broadens a separately stored audience grant. The edits
touch different units and files, but together violate a claim such as “untrusted callers can never
reach destructive effect E.” Integration returns `Contradictory` with that claim's counterexample.
A clean textual merge is not safety evidence.

### 6.4 Two valid results, no authority to choose

P changes a representation. Q inserts an adapter whose placement before or after that change
produces two valid but observably different programs. If both satisfy the current claims and no
modeled policy defines placement, the result is `Ambiguous`, not a deterministic tie-break.

### 6.5 Surface-only change

A formatter reorders declarations, a module moves between files, or namespace-derived imports
disappear while the graph stays identical **under the declared language model and equivalence
witness**. The semantic transformation is a no-op only under that observation model; storage
binding or projection provenance changes separately.

## 7. The safety quadrants and the real bar

“Compatible” below means compatible relative to the explicitly required model, claims, fidelity,
and bounds—not unknowable private thoughts:

| underlying joint result | textual Git says clean | textual Git says conflict |
|---|---|---|
| compatible | correct automatic merge | **false conflict**: unnecessary judgment and CI delay |
| incompatible | **silent wrong merge**: highest-cost failure | correct refusal, usually imprecise |

Daglang's opportunity is not merely moving cases from the right column to the left. It is:

- prove more compatible cases as `ClosedOne`, admit them, and return `Applied` only after commit;
- prove more incompatible cases and return a bounded subset-minimal `Contradictory` witness;
- expose genuinely normative choices as `Ambiguous`; and
- keep incomplete modeling visible as `Unknown`.

The strongest honest “true final intent” guarantee is:

> The accepted result is the unique result, up to declared equivalence, that satisfies all explicit
> proposals and required obligations, and every result change is grounded in those proposals.

No SCM can guarantee an unexpressed mental preference. Claiming that would convert missing
information into a silent decision.

### Evidence profile — a product, not a ladder

The evidence dimensions are orthogonal. A fresh independent behavioral probe with lossy operation
recovery is neither globally above nor below an exact structural parse with no behavioral
evidence. An admission policy sets minimum requirements **per axis**:

| profile axis | honest states / question |
|---|---|
| authored-operation recovery | native authored operation · exact endpoint delta only · structurally recovered operation · ambiguous recovery · unknown recovery |
| capture completeness | which declared input surface and construct set were closed? |
| structure | well formed; each captured delta represented once; no invented structural delta |
| resolution and typing | bindings, inhabitance, refinements, and namespace claims under a named model/version |
| cross-unit claims | which invariants hold over which affected relation and bound? |
| effects/resources/interference | grants, frame/rely/guarantee, bounds, temporal preconditions |
| behavior | which observables agree, under which environment and bound? |
| provenance/authority | who or what produced the fact, and may the actor submit/approve this scope? |
| freshness | at what provider/target generation and domain-time bound is the evidence applicable? |
| probe independence | could collecting the evidence establish the state it claims to observe? |
| normative input | only authenticated authored transformations, contracts, dependencies, and explicit answers |

`DecodeFidelity = Lossless` can establish exact endpoint decoding for its construct set. It cannot
reconstruct an authored operation that was never retained. Likewise a passing typecheck cannot
stand in for behavioral or authorization evidence. Each receipt carries the profile and policy
requirements; a UI may summarize it, but no dimension masquerades as another.

### Comparison evidence, not design authority

Git's documented default merge strategy combines non-overlapping textual changes and exposes
content conflicts when it cannot. Rename/copy pairing is a similarity calculation. A clean result
therefore means the configured merge machinery found no conflict; it is not a language-semantic
proof ([Git merge](https://git-scm.com/docs/git-merge),
[diffcore rename detection](https://git-scm.com/docs/gitdiffcore#_diffcore_rename_for_detecting_renames_and_copies)).

The empirical baseline reinforces the safety constraint:

- The ASE 2024
  [evaluation of version-control merge tools](https://doi.org/10.1145/3691620.3695075) reports,
  over its corpus, Git `ort` at 2,748 correct / 3,078 unhandled / 157 incorrect scenarios; roughly
  5.4% of Git's handled outputs in that experiment were incorrect. Its structural Java baseline
  handled more cases but also produced more incorrect results—evidence that “fewer conflicts”
  alone is the wrong objective.
- The 2026 preprint
  [On Correctness of Software Merge](https://arxiv.org/abs/2607.07987) gives a useful structural
  floor—parsability plus universal edit preservation—and checks it over 43,774 Java file
  scenarios. Its guarantee is deliberately syntactic/structural and depends on AST differencing;
  it does not establish behavioral or normative intent.

These baselines belong in the discriminating corpus. They do not define daglang's model.

## 8. Existing concept DFS — reuse, do not fork

The first modeling pass must try to compose these authorities:

| existing carrier | contribution | limit for this lane |
|---|---|---|
| `gunbc.roster_registry.GroupMembership` | containment/derivation/frontier provenance for an enrolled roster | not a group carrier, member relation, or closure algebra |
| `std.change.keyed_two_way_diff` | exact keyed endpoint observation | observes change; does not infer or reconcile peer proposals |
| `std.change.keyed_three_way_fold` | conservative base/observed/desired reconciliation | asymmetric desired-state/infrastructure shape; key overlap currently collapses to conflict |
| `gunbc.membership_reconcile` | one generic desired-vs-observed fold with stable member identity and an un-emittable ownership-unknown arm | infrastructure convergence and an epistemic precedent, not concurrent author intent |
| `v2.std.find_witness.CandidateSet` + `UniqueOnly` | finite closed candidate selection with no/unique/ambiguous outcomes | supplied structural candidates only; does not generate the SCM candidate space or prove behavioral completeness |
| `v2.std.constraints.ConstraintGraph` + `solve_constraints` | existing structural “find what satisfies” scaffold | currently supplies the well-formed root as a singleton candidate and uses its structural witness as closedness; not SCM candidate-generation authority |
| `std.realization_reconcile` | apply → read-back → grounding receipt shape | `Grounds | DoesNotGround` collapses challenge and insufficiency; not the shared evidence relation |
| `gunbc.readback_independence` | positive self-establishing probe is insufficient while negative evidence may survive | one inference-validity rule, not a general assessment carrier |
| `std.upsert_decision.ObservationVerdict` | preserves conflict/inaccessible/unknown world-state distinctions | desired-vs-observed specific; not a claim-indexed evidence relation |
| `std.observation` | process-progress event authority | reserves “observation” for that domain; do not overload it |
| `std.temporal_effect` | exact snapshots, intent hashes, idempotency, prior receipts, generations | effect-attempt vocabulary; do not rename it into generic source intent |
| `std.computation_identity` | structural/normalized/bounded-extensional evidence plus typed unknown | identity evidence, not user-intent identity |
| `std.perturbation` | response to changed inputs | a building block for bounded noninterference evidence |
| `std.realization.Independence` | `Independent | Dependent | Unknown` | currently effect-shape-specific and deliberately coarse |
| `std.pareto` + `std.realization_schedule.CostAccount` | dominance without hidden scalar weights; grounded time/space/power accounting | chooses among realizations of one semantic result by default; cannot trade cost for safety or supply missing semantic intent |
| `gunbc.os_install_deduction` | evidence-graded deduction where timestamps remain provenance and weak observations stay weak | discriminating consumer of the shared evidence relation, not an SCM outcome or solver |
| `v2.compiler.source_authority` + `DecodeFidelity` | ingest/emit authority and endpoint-decode honesty boundary | `Lossless | Lossy` says nothing about whether the authored operation survived |
| affected-set and materialization lanes | dependency-scoped validation and content-keyed receipt reuse | selection/caching must not decide semantic compatibility |
| `extdeps.git` + `extdeps.git.object_store` | cited Git operation, object, tree, commit, ref, and diff interface shapes | external compatibility authority; Git transport/policy and source integration stay separate |
| GitLab/Atlassian/Microsoft SEC, pricing, and `gunbc.econ.scm_*` carriers | grounded distribution/serving/agentic-stress economics | price the product and store; they do not imply integration semantics |

There is no Mercurial or Pijul extdeps authority in-tree today. That absence is a substrate gap, not
permission to infer their shapes from Git or to call a Git-shaped interface generic.

The existing keyed diff remains useful beneath the new model. Its meaning changes from “the merge
algorithm” to “one observation/capture engine.” In particular, the storage-binding plan's current
same-key refusal is the safe adapter available **before** semantic integration lands; it is not a
proof that same-key proposals are contradictory.

The same anti-fork rule applies to “deduction.” The first implementation must determine how the
existing finite-candidate, structural-constraint, Pareto, closure, and diagnostic authorities
compose. It must not land a generic `ScmSolver`, `ReconciliationConstraint`, or interaction-rule
registry beside them merely because the SCM is the first demanding consumer.

### Objective external concepts for P−1/P0 DFS

These are comparison authorities, not instructions to implement eleven subsystems before P0:

| concern | primary authority | immediate use |
|---|---|---|
| fact becomes evidence only for a claim | [OMG SACM](https://www.omg.org/spec/SACM/) | claim-indexed support/counterevidence rather than truthy fact bags |
| who/what produced a fact | [W3C PROV-DM](https://www.w3.org/TR/prov-dm/) | entity/activity/agent provenance, derivation, responsibility |
| support, challenge, conflict, absence | [Belnap, *A Useful Four-Valued Logic*](https://doi.org/10.1007/978-94-010-1161-7_2) | two independent support/challenge bits; preserve both and neither |
| sound maximum answer under abstraction | [Cousot & Cousot](https://cs.nyu.edu/~pcousot/COUSOTpapers/POPL77.shtml); [Giacobazzi–Ranzato–Scozzari](https://doi.org/10.1145/333979.333989) | sound approximation and query-relative completeness |
| proposal pre/post/frame | [Hoare](https://doi.org/10.1145/363235.363259) | requires/ensures plus the explicit frame |
| environmental interference | [Jones](https://doi.org/10.1145/69575.69577) | relies/guarantees for concurrent composition |
| context-sensitive concurrent operations | [Darcs patch theory](https://darcs.net/Theory/MergersDocumentation) | proposal context, transport, commuting squares |
| safe coordination avoidance | [CRDTs](https://inria.hal.science/inria-00609399); [invariant confluence](https://arxiv.org/abs/1402.2237) | convergence is separate from application-invariant preservation |
| two-way source/view maintenance | [lenses](https://www.cis.upenn.edu/~bcpierce/papers/lenses.pdf); [quotient lenses](https://www.cs.cornell.edu/~jnfoster/papers/quotient-lenses.pdf); [delta lenses](https://doi.org/10.5381/jot.2011.10.1.a6) | law-governed `BothWays` and declared equivalence |
| exact-parent atomic effect | [Herlihy & Wing](https://www.cs.cmu.edu/~wing/publications/HerlihyWing90.pdf); [Git `update-ref`](https://git-scm.com/docs/git-update-ref) | linearizable commit point and first Git CAS realization |
| safe diagnostics/remote execution | [Sabelfeld & Sands](https://doi.org/10.3233/JCS-2009-0352) | information flow, noninterference, explicit declassification |
| locked artifacts and erasure | [NIST SP 800-38D](https://csrc.nist.gov/pubs/sp/800/38/d/final), [SP 800-38F](https://csrc.nist.gov/pubs/sp/800/38/f/final), [SP 800-88r2](https://csrc.nist.gov/pubs/sp/800/88/r2/final) | AEAD, authenticated key wrap, conditional cryptographic erase |
| Git fidelity boundary | [Git object model](https://git-scm.com/docs/user-manual), [`gitattributes`](https://git-scm.com/docs/gitattributes), [partial clone](https://git-scm.com/docs/partial-clone) | complete import/export inventory, unavailable-object states |

### SCM plurality stress — three upstream models before one shared shape

The product wedge remains Git-first because that minimizes adoption cost. The technical boundary is
SCM-plural because adoption order is not design authority. Three upstreams are modeled before a
common compatibility carrier is allowed to land:

| upstream realization | upstream shapes that must remain distinct | design pressure |
|---|---|---|
| [Git](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects) | blob/tree/commit/tag objects, refs and symbolic refs, worktree/index, object/hash format, and exact-old-OID [`update-ref`](https://git-scm.com/docs/git-update-ref) | dominant compatibility and first writable product target; stresses snapshot/object fidelity and an explicit compare-and-advance primitive |
| [Mercurial](https://www.mercurial-scm.org/help/topics/glossary) | file revisions, manifests, changesets, DAG revision selection, bookmarks, named/topological branches, [phases](https://www.mercurial-scm.org/help/topics/phases), and separately stored experimental obsolescence markers | prevents `branch = ref`, `published = pushed`, or one history identity from becoming universal; stresses mutable/publication metadata orthogonal to file history |
| [Pijul](https://pijul.org/manual/getting_started.html) | working copy, recorded changes, dependency/context graph, pristine state, tree, conflicts retained in the graph, and [channels as sets of changes](https://pijul.org/manual/workflows/channels) | prevents `change = snapshot commit` and `target = pointer to one commit`; stresses first-class change identity, dependencies, commutation, and set-valued channel state |

These are design falsifiers, not a promise that all three are product-grade on day one. Their
upstream models land independently in `extdeps/`, with real names, citations, version/format
boundaries, and no common interface. Only after all three exist may DFS extract proven coincidence.
The core must not gain `ScmKind = Git | Mercurial | Pijul`, switch on vendor identity, or import an
upstream's branch/ref/channel vocabulary. Dispatch belongs in peripheral realization bindings
(DESIGN §3), so adding a fourth SCM is a new extdeps model plus bound handlers and witnesses—not a
kernel edit or a new coproduct arm.

The shared compatibility boundary is expected to need these **roles**, pending that DFS:

- observe an upstream state and its stable generation/version evidence without assuming one OID or
  linear revision number;
- capture endpoint and, where the upstream retains it, authored-change evidence at an explicit
  fidelity profile;
- identify a named upstream target without equating ref, bookmark, branch, or channel;
- state the target's conditional-advance guarantee: exact compare-and-advance, a differently
  grounded serialized/transactional guarantee, or capability unavailable/unknown;
- project the accepted program and allowed metadata into the upstream representation;
- independently read back the projected/advanced state;
- repair an idempotent projection without replaying the semantic transition; and
- preserve or explicitly downgrade upstream-specific history, publication, conflict, path/mode,
  opaque-content, partial-availability, and identity facts.

Not every SCM must provide every role. Missing exact-parent or round-trip fidelity is a typed
capability refusal for that target policy, not a fake implementation built from fetch timing,
local locks, or “most recent” state. The semantic kernel consumes native proposals, accepted
states, claims, and receipts; it never consumes a Git commit, Mercurial changeset, or Pijul change
as though that upstream object were the semantic contract.

The landed C0 interaction carrier (`gunbc.native_scm_interaction_contract`) is deliberately only a
presentation fixture. Its optional `git` projection and two Git-named scenarios are now a counted
frontier, not the shared adapter interface. They must not be consumed by the proof kernel. Their
dissolution trigger is the three-upstream compatibility-shape slice: replace the fixture slot with
adapter-indexed projection receipts derived from the extracted roles, while keeping the dead-simple
primary outcome byte/behavior stable.

## 9. Admission and CI — linear history without serial work

The native fast path:

1. Read target `T` at accepted state `S_n`; freeze its generation, model/schema, claim set,
   inference rules, equivalence, admission policy, and validation receipts.
2. Observe required current facts and assess them against named claims with explicit
   authority/provenance/fidelity/freshness/independence; missing evidence stays missing.
3. Author a modeled transformation directly, or capture an endpoint delta/operation from a
   declared surface at its honest recovery fidelity.
4. Select and record the proposal set at an explicit admission frontier. An authorized actor or
   declared policy may select it; transport arrival, timestamps, and scheduler iteration may not.
5. Derive the affected group and required claims, then compose proposal contracts and the
   dependency/atomicity graph.
6. Generate the bounded candidate closure, check the eight certificates in §4, and classify the
   semantic result space against `S_n`.
7. Reuse receipts whose content-addressed model/contract/dependency inputs are unchanged; validate
   only newly affected obligations.
8. Apply the accepted-parent admission policy and actor/effect authority. A unique semantic result
   can still be unauthorized, approval-required, or policy-refused.
9. Bind one target-transition realization whose declared capability satisfies the target policy.
   The first product realization writes a Git tree and ordinary one-parent commit, then executes
   `git update-ref <target-ref> <new-oid> <old-oid>`; an old-object mismatch returns
   `RetryStaleParent` and restarts from step 1, never `Unknown`. A Mercurial or Pijul realization
   may advance only after its own upstream model grounds an equivalent-enough guarantee for that
   policy; similarity of commands or local serialization is not evidence.
10. Independently read back the target through the same bound realization. Only a committed and
    grounded read-back returns `Applied`.
11. Project or enqueue every other declared surface. Projection is idempotent and records
    `Projected | ProjectionPending | ProjectionFailed`; it does not replay or roll back the
    semantic transition.

Linear accepted history does not imply serial validation. Compatible proposals and their affected
claims can evaluate in parallel; the acceptance log serializes receipts only after their
commuting-square and current-parent conditions are established.

The frontier solves a finite coordination question without pretending to know the future. A
proposal not yet observed cannot block progress. But if two known proposals are order-dependent,
automatically accepting whichever the scheduler happened to visit first would itself be a
normative choice. Once an authorized acceptance advances `T`, its receipt—not its timestamp—makes
the result the authoritative prior for later proposals.

This is the direct answer to green-then-main-advanced CI waste. A receipt should be keyed by the
actual modeled inputs of the claim, including any closedness evidence, not by “the whole branch is
still at this SHA.” If the target advances outside those inputs, the green receipt remains
grounded. If it advances inside them, the system revalidates the affected claim before acceptance.
A blanket rerun is an `Unknown`/modeling deficit, not a silent “fail-closed” success.

The Git-ref compare-and-swap is the first minimal **transition realization**, not the transition
interface and not a claim that Git trees become the program authority.
[`git update-ref`](https://git-scm.com/docs/git-update-ref) supplies the exact-old-object check
needed by the initial target policy. Other SCMs must ground their guarantee independently or expose
the capability as unavailable. When a later native target becomes authoritative and external SCMs
are asynchronous projections, an idempotent outbox binds the committed native transition to each
declared projection and repairs `ProjectionPending`/`ProjectionFailed` without reapplying the
semantic change.

Every transition receipt binds at least target + generation, parent/result state hashes, proposal
occurrence-set hash, admission-contract hash, model/schema + inference-rule versions, equivalence
definition + bound, evidence set + freshness, and affected-set derivation version. The generation
and policy/evidence hashes prevent an ABA-shaped acceptance in which the visible state hash returns
to an old value while the authority under which it was validated has changed.

## 10. Compatibility realizations are plural; Git is first, not authority

Compatibility is non-negotiable for adoption, but it is downstream. Product priority and design
coverage are intentionally different:

- Git is the first product-grade realization because current gunbc work, forges, CI, and likely
  design partners already use it.
- Mercurial and Pijul are first-wave design-stress realizations. Their upstream models and
  read-only round-trip fixtures must constrain the shared shape before that shape is called
  generic.
- Product-grade support for either is promoted only by a partner/workload receipt; the core remains
  unchanged whether that promotion occurs.

### Git export

- Each accepted native transition can emit an ordinary one-parent/squash Git commit and source
  tree. A team using squash-to-main sees the history shape it already expects.
- The streamlined submit path treats a branch/PR as a proposal inbox, not a peer authority. Users
  do not need to merge or rebase it onto the target; the admission service transports and
  re-evaluates the proposal against the current accepted target.
- Native proposal/evidence receipts may travel as optional metadata; a normal Git client can ignore
  them and still clone/build the projected tree.
- Files, paths, formatting, opaque blobs, and derived artifacts follow the storage-binding and
  materialization policies; they are not smuggled into the semantic model.

### Git import

External Git work supplies base and endpoint snapshots. Ingest recovers graph transformations with
an explicit fidelity result:

```text
NativeAuthoredOperation
| ExactEndpointDelta
| StructurallyRecoveredOperation
| AmbiguousOperationRecovery
| UnknownOperationRecovery
```

Those labels are illustrative roles pending DFS. `NativeAuthoredOperation` is possible only when
the operation or equivalent authenticated metadata survived. A lossless endpoint decode can prove
`ExactEndpointDelta`; it cannot recover the operation that produced the endpoint. Rename
similarity, matching content, and three-way ancestry are claim-indexed evidence. They never become
invented identity. A textual merge driver may remain as a compatibility fallback, but its clean
result carries only textual evidence and cannot claim the native safety profile.

The current `extdeps.git.object_store` is intentionally not treated as complete: `TreeEntry` omits
Git modes, and `StoreObject` omits tag objects even though `extdeps.git.ObjectType` has `TagObj`.
P4's import/export DFS must cover:

- tree-entry mode/type, including regular vs executable blobs, symlink blobs, subtrees, and
  gitlinks/submodules;
- annotated tag objects as distinct from lightweight tag refs;
- raw path representation and invalid/non-text path handling;
- repository object/hash format rather than assuming one OID width;
- `.gitattributes` text conversions, encodings, clean/smudge filters, and merge/diff drivers;
- opaque/binary content whose semantics are outside a declared ingester;
- unavailable objects in shallow/partial clones and unfetched submodules; and
- removal of native proposal metadata, which downgrades authored-operation evidence without
  changing the emitted program.

Git's own [object model](https://git-scm.com/docs/user-manual),
[`gitattributes`](https://git-scm.com/docs/gitattributes), [partial-clone
contract](https://git-scm.com/docs/partial-clone), and [hash-format
transition](https://git-scm.com/docs/hash-function-transition) are the external boundary. Ordinary
clone/build compatibility is not established until these fidelity axes round-trip or refuse
honestly.

### Mercurial stress boundary

Mercurial modeling must preserve its own distinctions rather than translate them into Git names:
portable changeset identity versus clone-local revision number; file revisions and manifests;
executable/symlink/copy facts; changeset parents and heads; named branches, movable bookmarks, and
tags; public/draft/secret phase monotonicity and publishing-server behavior; subrepositories;
repository-format requirements and unavailable data; and experimental obsolescence markers as
metadata orthogonal to file-history changesets. A bookmark is not renamed to a ref in extdeps, a
phase is not reduced to a visibility Boolean, and an obsolescence successor is not silently called
a rebase.

The first Mercurial fixture is read-only state/change capture plus round-trip projection over a
bounded repository. Managed target advance remains capability-unavailable until the upstream model
and an executing transport witness establish the exact concurrency/transaction guarantee required
by the target policy.

### Pijul stress boundary

Pijul modeling must preserve change identity and dependency/context facts, the recorded
change/pristine/tree/working-copy split, graph conflict facts, channel state as a set of changes,
channel-independent change identity, version/state identity, partial-path/change availability, and
the difference between applying a change and projecting a working copy. A change is not renamed to
a commit and a channel is not forced into one-head semantics merely to inhabit a Git-derived
adapter.

The first Pijul fixture imports and exports a bounded change/dependency/channel state and proves
that reordering independent change delivery does not change the native semantic receipt. This is
not permission to inherit Pijul's textual conflict semantics as Daglang semantic safety. Managed
target advance remains capability-unavailable until its upstream operation and read-back contract
meet the declared target policy.

### Cross-realization law

For one accepted native state and one declared surface fragment, changing only the compatibility
realization may change upstream bytes, history topology, publication metadata, and fidelity
receipts. It must not change the native semantic result, admission decision, or user question.
Removing an upstream capability may turn projection or target admission into a typed refusal; it
may not select a different semantic result. Adding a fourth SCM must require no edit to the proof
kernel or primary interaction carrier.

The first product can therefore be a Git-compatible semantic admission tool rather than a new
hosted object store. It proves the integration advantage while keeping clone, editor, CI, and forge
workflows intact. A native store is justified only after the semantic consumer displaces measured
cost.

## 11. Visibility, locked realizations, and customer trust

The [visibility-grants authority](node-subtree-visibility-grants.md) §11 owns the publication
capability profile, ciphertext/interface cut, admission/refusal rules, hole residues, placement
limits, residual-channel threat model, and conditional crypto-erase semantics. This plan neither
abbreviates nor redefines them.

The only integration consequences carried here are:

- the signed order remains: implement `Reference`/`Publish` first over today's two Git storage
  roots; and
- the same grant interface later constrains native history, storage, projection, and remote
  execution without becoming an integration/conflict predicate.

No proposal or successful transition can widen a publication grant merely because its program is
structurally or behaviorally valid, and no secrecy/publication claim can be inferred from semantic
compatibility. Grant/policy changes require separate authority and the parent-policy transition
rule in §4. The concerns compose through explicit grants and realization bindings only.

Diagnostics are themselves publication projections. A full internal proof may mention a locked
node, counterexample, affected closure, or alternate result that an audience is not authorized to
know exists. The default response must therefore be an audience-authorized projection—possibly a
redaction or commitment to hidden evidence—with no error, timing, size, or alternative-count
distinction that bypasses the visibility model. The same
[information-flow/declassification](https://doi.org/10.3233/JCS-2009-0352) rule governs remote
execution: metering prices an oracle; it does not prove that outputs, failures, effects, timing, or
resource use reveal only what policy authorizes.

Vertical integration can bring the product closer to real compiler/use-case failures. It must not
turn “visibility into customer code” into an undeclared surveillance business model. Raw source
stays governed by `Publish`/effect grants; telemetry is derived, consented, and minimal—evidence
profile, layered outcome summary, cost, and anonymized mechanism gap where possible. Trust is part
of the safety product, not a growth shortcut.

## 12. Product thesis and falsification

The idea is interesting because agent concurrency changes the economics of source control. A
30-minute validation path and many simultaneous proposals make integration quality a direct,
compounding compute/latency/token cost. Daglang has unusually rich evidence available because the
program, claims, compiler, affected set, and execution model can share one substrate.

The product advantage is therefore not “the language knows what the user meant.” It is:

> **The user makes changes and states preferences. The system owns integration. It lands every
> safely determined result and interrupts the user only for an irreducible choice about what they
> want; when a terminal refusal makes landing impossible, it says so plainly without fabricating a
> choice.**

The user-facing operation stays the §0 loop: edit, submit, then receive **Landed** or one localized
plain-language **Choice required** question with concrete previews and a stored continuation. The
system retries stale targets and seeks missing mechanical evidence itself. If every permitted
machine-owned continuation is exhausted, **Could not land** reports the plain domain reason and
confirms no transition committed. Internal `BothConflicted`, `Unclosed`, policy, transition, and
projection states remain inspectable receipts, never default error prose. The model can deepen for
decades without changing that public operation or growing a catalog of language-feature
interactions.

The initial wedge is narrow and credible:

- teams running many coding agents against the same modeled monorepo;
- long or expensive CI;
- a lossless daglang source model;
- Git-compatible input/output for the first product, with the compatibility shape already
  falsified against Mercurial and Pijul upstream models; and
- a dashboard showing judgment and validation work actually displaced.

The GitLab corpus supports the “cheap to serve” premise, not “distribution is free.” GitLab's
serving cost is a minority of revenue while selling and R&D dominate. Word of mouth is plausible
only if the product makes savings obvious in the existing workflow. Compatibility and receipts are
therefore part of the product, not marketing afterthoughts.

Track at least:

- automatic `Applied` rate by evidence profile and admission contract;
- semantic zero/one/many/unclosed, admission, transition, and projection rates and causes
  separately;
- default-surface `Landed | ChoiceRequired | CouldNotLand` rates, question count, time-to-answer,
  automatic continuation success, and refusal owners/causes;
- judgment requests, human minutes, and LLM tokens per accepted proposal;
- false-conflict rate relative to the required contract;
- detected and escaped wrong integrations;
- CI minutes and wall-clock delay invalidated, rerun, and reused;
- safe partial assessments and independent proposal groups preserved when a batch cannot commit;
- proposal queue latency under the 50-agent/10-overlap stress profile;
- compatibility-import fidelity distribution and round-trip fidelity by bound realization; and
- storage operations/bytes/egress under the landed packing and provider models.

The “unsafe automatic integration” target is not hand-waved as absolute. Every claim is scoped to
the declared model, fidelity, and bound, and the falsifier corpus must actively search for escaped
wrong results.

## 13. Discriminating scenario corpus

Before storage or forge work, land a model-level corpus whose expected outcomes separate the
designs. Each scenario asserts both the layered internal receipt and the §0 plain-language
projection:

| scenario | required internal result | default surface |
|---|---|---|
| `foo` arguments changed independently | `ClosedOne foo(c,d)` only with binding + joint-obligation + closure evidence; then admitted, committed, read back | `Landed`, with `foo(c,d)` preview |
| rename plus concurrent old-spelling call | witnessed transport, closed-zero/many, or unclosed; never broken-clean | `Landed`, or `Choice required`: follow the rename vs keep a separate `foo`, with previews |
| disjoint policy changes violate one claim | `ClosedZero` with a subset-minimal-within-bound claim core | `Choice required` only if the user can choose which explicit requirement to change; otherwise `Could not land` with the permitted policy reason |
| two valid order-dependent transformations | `ClosedMany` with the two material alternatives | `Choice required` with one localized placement/order question and two previews |
| non-idempotent append/increment delivered twice with one proposal occurrence ID | one contribution/application | `Landed`; no duplicate-work explanation required |
| formatting/reorder/file move | semantic no-op only under the named model/equivalence; projection delta separate | `Landed`, optionally noting no program behavior changed |
| login status says logged in; live request proves refresh invalid | `CredentialRecorded` supported; `ProviderReady` challenged/not supported | readiness consumer requests authentication only if needed; not an SCM question |
| positive probe can establish the state it observes | positive evidence is neither/insufficient; negative may remain evidence | system seeks an independent check internally |
| three proposals are pairwise compatible but jointly violate a cardinality/resource/security bound | `ClosedZero` with an n-way core | `Choice required` only for a genuine requirement tradeoff, with concrete previews |
| proposal removes the policy that would reject it | `PolicyRefused` under frozen parent policy | route the policy change separately; if refused, `Could not land`, never a semantic question |
| exactly one semantic result but proposer lacks authority | `ClosedOne + Unauthorized`, never Applied | route approval; if denied or unavailable, `Could not land`, never ask what the code should mean |
| contradiction would cite a locked node to an unauthorized audience | full internal proof + audience-authorized redacted/commitment-bearing projection | no hidden existence/content leak through wording, timing, or alternatives |
| target advances after validation | `RetryStaleParent`; automatic re-evaluation | no interruption and no rebase request |
| native transition commits but Git export fails | committed/read-back transition + `ProjectionFailed/Pending` outbox state | `Landed`; projection repair is inspectable operational status |
| endpoint decodes losslessly but authored operation is absent | `ExactEndpointDelta + UnknownOperationRecovery`, never native-authored evidence | system continues only if the admission contract proves the endpoint delta sufficient |
| lossy Git import | ambiguous/unknown recovery profile | no invented intent; ask only if a genuine preference remains, otherwise `Could not land` after recovery bounds exhaust |
| absent required behavioral bound | structural assessment survives; semantic space/admission remains unclosed | system seeks evidence; if the declared source/bound is terminally unavailable, `Could not land` |
| non-retryable effect failure before commit | `EffectFailed` with typed domain cause and no transition receipt | `Could not land`: plain failed action, responsible owner, no files changed |
| timestamps/arrival order are erased or permuted | identical semantic result and evidence assessments | identical outcome |
| one plausible result but candidate/dependency closure unproved | `Unclosed` with the plausible candidate retained as partial information | investigate until bounds exhaust, then `Could not land`; never land merely because nothing else was modeled |
| authorized prior transition adds a new invariant as ordinary data | later outcome changes through the same kernel | no new merge mode or user workflow |
| one batch has unique, contradictory, unknown, and ambiguous independent groups | all four partial assessments retained; partial commit only with declared atomicity + proven independence | land proven-independent work; at most one `Choice required`; terminal refused groups say `Could not land`, details on inspect |
| ordinary Git branch is stale but proposal transport is provable | target refresh/transport/CAS happens internally | submit succeeds without asking the user to merge or rebase |
| one accepted state projects through Git, Mercurial, and Pijul bounded fixtures | identical native semantic/admission receipt; realization-specific projection and fidelity receipts | identical primary outcome; SCM mechanics remain inspect-only |
| an SCM realization lacks the target policy's required conditional-advance guarantee | semantic result is retained; target admission/projection is capability-refused, never fabricated | machine uses another authorized target realization or eventually says `Could not land`; never asks the user to invent synchronization |
| a fourth SCM model/handler is added | no proof-kernel or primary-outcome edit | the same edit → submit → outcome contract |

Every committed/Applied case needs a perturbation that breaks one obligation or certificate and
turns the appropriate semantic/admission/transition layer red. Every refusal needs a nearby
compatible control, so “always refuse” cannot satisfy the suite. The timestamp case permutes and
erases incidental time metadata. The model-extension case adds its invariant as data through an
authorized earlier transition while holding the kernel fixed. The presentation witnesses reject
internal type/logic/proof vocabulary on the default surface and reject more than one simultaneous
normative question.

Git's default merge, the current keyed-diff adapter, and at least one structural merge baseline
should run on the same corpus. Mercurial and Pijul fixtures additionally exercise the compatibility
boundary, not their merge engines as semantic oracles. The comparison is evidence, not the design
authority.

## 14. Sequencing and acceptance

The previously signed visibility sequence stays intact. Integration work starts model-first and
does not wait for a native store:

1. **Visibility Stage 0 (already first):** `Publish`/`Reference` model, public/private Git roots,
   file-grain declarations, push guard, and existing-public-corpus stamp.
2. **R0 — three independent upstream-model PRs:** complete the Git extdeps boundary, then model
   Mercurial and Pijul from their own primary authorities. Each PR keeps its upstream's real
   object/change, target, identity, history, publication, path, format, and availability
   distinctions and deliberately declares no shared SCM interface.
   **Accept T1 per upstream:** a bounded fixture decodes, projects, and independently reads back its
   declared fragment; a nearby RED proves that one upstream-specific distinction cannot be
   represented as the tempting Git analogue. For Git this includes modes, symlinks, gitlinks, tags,
   paths, object format, and exact-old-OID ref advance. For Mercurial it includes portable changeset
   identity versus clone-local revision, manifests/file revisions, bookmarks/branches, phases, and
   optional evolution metadata. For Pijul it includes change dependency/context, pristine/tree
   state, conflict facts, and channels as sets of channel-independent changes. Write capability may
   remain honestly unavailable.
3. **P−1 — shared claim-indexed evidence carrier, separate PR:** after DFS, model recorded facts,
   scoped claims, support/challenge evidence links, maximum conclusions, assessments, and readiness
   policy once for Codex provider readiness, `os_install_deduction`, and SCM.
   **Accept T1:** login-status/live-request and self-establishing-probe scenarios preserve
   credential-vs-provider and support/challenge/neither distinctions; the three consumers use the
   same carrier rather than local enums.
4. **R1 — extract the compatibility realization shape:** only after all three R0 models land, run
   DFS over their executing fixtures and extract the smallest role interface actually shared. Keep
   upstream dispatch peripheral; capability support is data/receipts, never a central vendor enum
   or switch.
   **Accept T1/T2:** one fixed accepted-state/proposal fixture exercises all three and preserves
   C0-owned proposal identity, transition, and primary outcome while projection/fidelity receipts
   remain realization specific; R1 does not invent the later semantic/admission receipt. Removing a
   required conditional-advance capability refuses that target policy; a synthetic fourth adapter
   requires no primary-interaction edit and, once P1 exists, no proof-kernel edit. This slice
   replaces C0's optional `git` fixture slot and Git-named presentation cases with
   realization-indexed projection receipts without changing primary behavior.
   R0 and P−1 may proceed in parallel because neither is allowed to infer the other. R1 waits for
   all three R0 models; the proof kernel waits for P−1.
5. **P0 — user contract, cost, and layered scenario model:** carry the §0
   `Landed | ChoiceRequired | CouldNotLand` surface, stress profile, proposal contract roles,
   evidence profiles, semantic
   zero/one/many/unclosed space, admission, transition, projection, closure certificates,
   timestamp non-authority, and scenario corpus as `.dag` facts/witnesses.
   The landed C0 carrier from PR #7334 supplies twelve presentation-journey witnesses, not the
   semantic kernel, evidence relation, or compatibility interface; its optional Git projection is
   the explicit R1 frontier.
   **Accept T1:** every scenario produces its internal receipt and three-arm primary outcome, with
   only `Landed | ChoiceRequired` as semantic handoffs; nearby REDs prevent always-apply/refuse;
   removing a closure certificate changes `ClosedOne` to `Unclosed`; self-amendment,
   unauthorized-unique, exhausted-bound, and permanent-effect scenarios produce typed refusals and
   plain `CouldNotLand`. No normal journey requires merge/rebase/cherry-pick/reset/conflict
   markers/force-push; primary messages contain none of the §0 forbidden terms; every question is
   normative; machine-answerable work stays internal while a permitted continuation exists; one
   answer resumes automatically; every terminal refusal is visible; ordinary Git export remains
   valid.
6. **P1 — bounded source-integration proof kernel:** one public operation composes directly
   authored transformations over a small decidable program fragment. Domains contribute facts,
   claims, procedures, and checkable certificates; scenarios never add feature-pair arms.
   **Accept T1:** occurrence-ID dedup, context validity/transport, commuting squares,
   batch-partition invariance, grounded/frame-preserving deltas, n-way invariants, order honesty,
   and each closure certificate execute. Adding a new invariant as data changes a later outcome
   without editing the kernel.
7. **P2 — law-governed authoring capture:** one daglang surface supplies a quotient-delta-lens
   projection. Native authored operation and exact endpoint delta remain distinct.
   **Accept T2:** unchanged-view write-back, supported edit round-trip, sequential edit coherence,
   source-only information retention/declared canonicalization, and unsupported/ambiguous refusal
   all execute without mutation on refusal.
8. **P3 — target-scoped atomic admission + receipt reuse:** consume the R1 bound
   target-transition capability rather than an SCM name. The first writable realization uses Git
   `update-ref <ref> <new> <old>`; pending proposals re-evaluate against the accepted target, and
   affected receipts survive unrelated advances. Mercurial and Pijul fixtures remain read-only or
   capability-refused until their own executing transports ground the target policy's required
   guarantee.
   **Accept T2/T3:** two concurrent proposals accept without full CI replay; changed
   model/policy/evidence/affected inputs invalidate the receipt; a race returns
   `RetryStaleParent` and retries without user rebase; independent read-back gates Applied; a
   committed transition plus projection failure enters the idempotent outbox.
9. **P4 — compatibility realization family:** promote Git to the first product-grade realization:
   import an ordinary branch/PR with the §10 fidelity inventory and export accepted transitions as
   ordinary squash/one-parent commits. Keep Mercurial and Pijul at bounded real-fixture T2 unless a
   design-partner receipt justifies product-grade support.
   **Accept Git T3:** unmodified Git clone/build works across
   modes/symlinks/gitlinks/tags/attributes/opaque data/object formats; unavailable objects remain
   typed; native metadata removal lowers authored-operation evidence but never changes the emitted
   program. **Accept Mercurial/Pijul T2:** each declared fixture round-trips or refuses honestly and
   demonstrates cross-realization receipt invariance; it does not imply a writable hosted product.
10. **P5 — publication capability profile and remote realization:** after visibility Stage 0, land
   the capability product and execute withheld nodes through declared interfaces/effect grants
   where a consumer prices it, with audience-safe diagnostics and explicit declassification.
11. **P6 — native store/serving:** only after the semantic path is a named consumer; use the landed
   object-storage, packing, reliability, and regional-compute carriers. Never one stored object per
   semantic node by default.

The publication capability profile may advance independently after visibility Stage 0; it is not a
semantic-integration prerequisite. R0 is upstream grounding and P−1 is shared deduction substrate;
they may run concurrently, neither imports the other, and both precede the downstream consumer that
needs them. No phase is complete at T0 algebra alone: each names an executing consumer and a
discriminating red.

## 15. Non-goals

- Mind-reading or claiming certainty about unexpressed intent.
- Reducing conflict count by silently choosing a plausible result.
- A weighted “likely intent” score, timestamp/arrival tie-break, or pairwise interaction catalog
  standing where the general reconciliation fold and an honest ambiguity belong.
- A total evidence ladder, truthy fact bag, or last-observation Boolean collapse.
- Treating arbitrary transformations as idempotent/commutative, or treating pairwise compatibility
  as n-way safety.
- Arbitrary program synthesis or unbounded proof/equivalence search in the first fragment.
- Letting a proposal amend the policy, claim set, inference rules, or equivalence used to admit
  itself.
- Claiming the modeled program is globally complete; every automatic result is scoped to explicit
  closedness, fidelity, equivalence, and proof bounds.
- Making node identity, module identity, a file path, or a universal durable entity ID the
  definition of compatibility.
- Deriving a supposedly generic SCM interface from Git alone, adding
  `ScmKind = Git | Mercurial | Pijul`, or branching the proof kernel on a vendor.
- Treating an R0/T1 upstream fixture as a promise of product-grade Mercurial or Pijul hosting.
- Replacing existing hosting, object storage, or forge UI before the integration consumer proves
  value.
- One semantic node per billable storage object.
- General arbitrary-language semantic merge outside a declared `DecodeFidelity` boundary.
- Unbounded behavioral equivalence; every such claim names a bound.
- Centralized access to customer source as an assumed business advantage.
- Requiring ordinary users to understand proof terminology, merge strategies, rebase, or
  distributed-ref topology; those remain inspect/debug and compatibility concerns.

## Dissolution trigger (DESIGN §6)

This document is the one design seed for the SCM lane, not a second status ledger. Its sections
dissolve as follows:

- claim/evidence roles → the P−1 shared carrier and three executing consumers;
- semantic/admission/transition/projection/group roles → the DFS-selected `std/` and plan carriers
  plus executing witnesses;
- capture/storage content → `v2.compiler.source_authority` and the module-storage design;
- visibility/capability-profile content → the visibility-grants authority;
- admission/receipt content → the temporal-effect, affected-set, realization, and roadmap rows;
- Git/Mercurial/Pijul shapes → their independent `extdeps/` authorities; proven common
  compatibility roles → the R1 bound-realization interface plus peripheral handlers; and
- economics → the already-landed cited extdeps/econ carriers.

Delete this plan when the native admission consumer reaches P4/T3 and the registered roadmap/carrier
graph contains the remaining P5/P6 work. Until then, update this file rather than opening a parallel
SCM design.
