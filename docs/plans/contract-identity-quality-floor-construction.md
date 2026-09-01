# Contract identity and the quality floor — two serial construction cuts

DESIGN §3 says a materially different contract needs a materially different name: *within one
naming surface and one declared effective version or epoch, holding the name constant may not
silently change the obligations, quality floor, refusal behavior, billing consequence, or remedy.*
DESIGN §4b adds the commercial half: *a dimension may remain opaque only above a falsifiable
quality floor with a named consequence*, and a claim without such a consequence *is marketing and
establishes no rung*.

Both sentences are prose today. Neither is a construction. This plan lands them as two serial cuts,
each with its own contribution boundary and its own acceptance receipt. They are deliberately **not**
reunified: the mature contract-identity consumer grades nothing, so bolting a floor onto it would be
a decoration, and the graded consumer needs the identity cut to land first.

## 0. What the census found, so this plan does not re-invent

- **Contract identity is not greenfield.** `gunbc.fleet_revision_acceptance` already carries
  `RequiredCiContractIdentity`, and its note already states the equality law this plan needs:
  identity names *which contract completed*, not what it did internally, because a `phases_run`
  field would make every phase added to the floor a breaking change to fleet admission. Every
  identity fact refuses with its own cause, and the module refuses to project a `Bool` over those
  arms. §2 therefore forbids minting a second contract-identity authority. Cut A generalizes this
  one.
- **The quality floor is greenfield.** No `QualityFloor` carrier exists in `dag/` or `src/`. The
  authorities to build on rather than fork are `std.claim_evidence` (`Claim<S, T, P, Scope, Bound>`),
  `std.spatial_knowledge` (uncertainty/method/obligation with a refusal cause and an admission arm),
  and `std.witness_admission` for the typed-refusal and axis-separation house style.
- **`gunbc.plans.deploy_convergence_observed_side` argues against authoring
  `service_contract_identity` today**, pending something that observes binary bytes. This plan does
  not contradict that row: it does not author a service/release identity, and Cut A's subject is the
  Required CI contract, whose identity that row does not claim.

## 1. Cut A — one canonical surface/name/epoch contract identity

### The defect this closes, stated as a live gap

`accept_required_ci_workflow_run` checks the workflow path and the repository, each with its own refusal arm,
and then admits. **It carries no epoch.** So a Required CI contract whose obligations changed
materially — a different floor composition, a different admission policy — is admitted by the fleet
under the same `workflow_path`, because the identity holding the name constant has no way to say
*which version of this contract*. That is exactly the §3 meaning fork: one name, two materially
different meanings, silently. The gap is authorable and therefore testable.

### The construction — an atomic root cut, not an additive shadow

One canonical contract identity whose grain is the §3 triple:

- **naming surface** — the space within which the name is held constant;
- **name** — the contract's stable name in that surface;
- **epoch** — the declared effective version, without which two materially different contracts
  cannot be distinguished.

The equality law is stated once, on the carrier, and is field equality over exactly those three —
never over the contract's internals, preserving the fleet note's rule. Each component mismatch keeps
its own refusal cause, and no arm projects to `Bool`.

`RequiredCiContractIdentity` is a **specialization of that shape**, not a parallel authority — the
generic triple and the Required CI inhabitant are one generic authority with a domain-specific
inhabitant, so the public domain name may survive. Its **old two-field representation may not.**

Cut A eliminates the old independently writable representation *in the same landing*. Acceptable
terminal forms are the specialization itself, or a domain wrapper whose only identity-bearing value
is the canonical triple. These are explicitly **not** acceptable, because each preserves two writable
answers and an alignment obligation between them:

- a record keeping `workflow_path` and `repository_full_name` *beside* a `canonical_identity`;
- an old constructor, a new constructor and a converter;
- the old path/repository checks plus a shadow generic comparator.

The final Cut A tree has **one production identity path**: the old two-field constructor is gone;
`required_ci_contract`, `RequiredCiSucceeded`, `AcceptedFleetRevision`, refusal rendering and every
witness consume the specialization; and no fallback can return to the old representation. That is the
root-level atomic transition the replacement doctrine requires — "atomic" governs the *authority
transition*, not whether the public domain name survives.

### Field classification, so contract identity does not widen into every admission prerequisite

| field | classification |
| --- | --- |
| `event.repository_full_name` | contract **naming surface** |
| `event.workflow_path` | contract **name** |
| subject-bound declared epoch | contract **epoch** |
| `head_repository_full_name` | provenance/trust check — *not* contract identity |
| `activity` | execution state — *not* contract identity |
| `conclusion` | outcome — *not* contract identity |
| `head_sha` | execution subject — *not* contract epoch |
| branch / originating event | default-branch proposition — *not* contract identity |

This preserves the module's current proposition split rather than absorbing every prerequisite of
fleet admission into the identity.

### The observed epoch must be a real production observation

**This is the load-bearing half, and the one an additive change would silently miss.** Today
`WorkflowRunEvent` carries workflow name, workflow id, workflow path, repositories, run id and head
sha — **no contract epoch** — and `required_ci_contract()` is constructed from the judge's *local*
authority and stamped into `RequiredCiSucceeded`. So adding `epoch` to `required_ci_contract()` alone
would stamp the *expected* epoch onto the accepted result and establish nothing about which epoch the
triggering run executed. A hand-built cross-epoch unit test would prove the comparator while the live
producer remained structurally incapable of observing a mismatch.

**The route this plan takes: exact-commit observation of the declared epoch at `event.head_sha`.**
The contract's name already resolves through the generated artifact identity
(`required_ci_contract` uses `artifact_path(a: WitnessFloorYamlArtifact)` =
`.github/workflows/witnesses.yml`). The declared epoch is emitted **into that same artifact** by its
workflow authority, so the observed epoch is read from the exact bytes the triggering run executed,
addressed explicitly at `event.head_sha` rather than from ambient checkout state. For a push-triggered
run the executed workflow file *is* the artifact at `head_sha`, which is the binding that makes this
the contract artifact the run actually executed.

**Why not the typed completion receipt — corrected, and the first reason was wrong.** An earlier
draft rejected this route on the premise that `floor_component_receipt_document` has no production
consumer and nothing emits it. That premise was false and is withdrawn: the receipt is real, rich
infrastructure — `gunbc.floor_component_receipt` declares the schema `floor-component-receipt/v1`,
the artifact name `floor-component-receipt`, the path `target/floor-component-receipt.json`, a typed
subject carrying workflow name, run id and head sha, a decoder, and an event-subject join. That is
most of what this route needs, already modelled.

The route is still not selected, for a materially different reason that the re-census establishes:

| question | finding at the census base |
| --- | --- |
| what emits and uploads the receipt | **nothing.** No writer of `target/floor-component-receipt.json` exists in `src/` or `dag/`, and no Rust carries the schema or its members. `witnesses.yml` uploads four artifacts — the floor disposition, the expected-red roster join, the long-home storage agreement and the per-claim cost receipt — and this is not one of them. |
| what downloads or renders it | **a workflow that no longer exists.** The module's prose describes an alert that downloads the artifact from a run id, in the present tense. `falsifier-alert.yml` was deleted in the 2026-08-15 CI bankruptcy, which DESIGN §4b rosters as a declared rung drop; only `witnesses.yml`, `fleet-converge.yml` and `fleet-desired.yml` remain. |
| what Cut A would need to add an epoch | an epoch member in the schema, the decoder and the subject join — **after** first restoring a producer and an upload step. |
| how fleet desired admission would obtain it | an artifact download bound to the triggering run id. The fleet lane has no such capability today. |
| how absence, malformed content, subject mismatch, incomplete run and unavailable artifact refuse | **partly already built, and an earlier draft overcharged this.** `gunbc.floor_component_receipt_document` already has typed `ReceiptMemberAbsent` and `ReceiptMemberMalformed` causes — including the duplicated-member, wrong-JSON-type and empty-string cases — plus the subject join. The genuinely missing causes are transport unavailability and any incomplete-run projection not already covered. |
| delta size versus exact-commit observation | **strictly larger**, and the difference is not the typed causes — it is the live writer, the upload step, the run-bound download, the epoch carriage, transport unavailability, and any absent incomplete-run projection. It front-loads resurrecting a dead transport before the epoch becomes observable at all. |

So the receipt route's obstacle is not that it was never built; it is that **its transport was
deleted and its authority's prose still recites it in the present tense.** That stale recital is
itself a finding — the same class DESIGN §4b names when it says a knowingly-false recital in a
canonical authority is premise contamination — and it is recorded here rather than fixed, because it
belongs to the receipt authority's owner and not to this cut.

**That finding may not live only here.** Cut B deletes this plan, so a finding whose only home is
this file would be destroyed by its own program. Its persistent destination is a routed item against
the receipt authority's owning lane, opened before Cut A lands; this file records it, but does not
own it. If no owner accepts it, the alternative is repairing the stale recital before dissolution —
what is not permitted is letting the plan's deletion silently take the finding with it.

The receipt remains the honest upgrade if that lane is ever restored, at which point it is the
stronger observation.

### What the selected route must name

Because the epoch is observed from the emitted workflow, the plan names each link, and **a comment
or an embedded substring is not sufficient contract data** — the epoch is a structural member:

- **the structural member** — a top-level `env:` entry in the emitted workflow, carried as a
  `kv(key, value)` pair beside the four `GUNBC_*` members the authority already emits;
- **the emitting authority** — `gunbc.witness_floor_workflow`, which owns that env block and emits
  `WitnessFloorYamlArtifact`;
- **the exact-commit reader** — a read of that artifact path at `event.head_sha`, never the ambient
  checkout;
- **the parser** — a structural read of the env mapping, not a substring match;
- **typed causes**, each distinct and each refusing on its own axis: epoch **absent**, **malformed**,
  **duplicated**, commit **unreadable**, path **unreadable**, epoch **undecodable**;
- **the epoch carrier and its canonical wire representation** — the epoch is a declared version
  value, carried as the string value of one top-level `env:` key emitted by the workflow authority.
  Its canonical wire form is fixed by that authority (one member, one spelling, no alternate
  encodings), so "the same epoch written two ways" is not a state the reader has to reconcile;
- **the binding** — for a push-triggered run GitHub executes the workflow definition at `head_sha`,
  and the contract's name already resolves through that same artifact identity, so the bytes read are
  the workflow definition the observed run executed.

These routes are explicitly rejected, each because it observes the judge rather than the subject:
reading the current checkout's epoch; calling `required_ci_contract()` for both sides; deriving the
epoch from the admission binary; treating `workflow_id` as a semantic epoch; or stamping the required
epoch after path equality has already succeeded.

### Consumer, and what makes the epoch load-bearing

The consumer is the existing production fold `gunbc.fleet_revision_acceptance`
`accept_required_ci_workflow_run`, reached in production through the live composition
`gunbc.fleet_desired_admission` `fleet_desired_accepted_from_event`, which calls it, then the
default-branch observer, then the fleet-revision join. The epoch is load-bearing
because a cross-epoch composition **refuses with an exact contract-epoch mismatch cause before the
outcome is interpreted**. An epoch carried into a receipt that no decision reads would not count,
and is explicitly not what this cut lands.

### The guarantee this cut does and does not establish

Stated honestly, because the difference is what stops a later reader over-reading it:

- **cross-epoch admission** — *mechanically prevented* by the production join;
- **an unbumped material contract change** — governed by **declared-version diligence**, unless a
  separate classifier or contract-surface digest is added.

Whether a change is materially contract-changing is not derived by this construction, so forgetting
to bump the epoch remains possible. Cut A does **not** make all same-name meaning forks structurally
impossible. It gives the repository one place to declare a transition, and makes a declared
transition load-bearing.

### Acceptance receipt for Cut A

- **Epoch control (the discriminating RED), designed around the fact that the epoch lives in the
  tree.** The epoch is a structural member of a committed artifact, so *changing it changes the tree
  and therefore the commit identity*. "Hold the revision fixed and vary only the observed epoch" is
  not constructible against a real Git-backed read, and an earlier draft asked for exactly that. The
  experiment is therefore **two commits whose trees differ only in the epoch member**, with both
  revision-bearing propositions agreeing inside each case: in the matching case the declared epoch at
  `event.head_sha` equals the required epoch; in the mismatching case it does not, and nothing else
  differs. Required outcome: exactly one refusal carrying the exact contract-epoch mismatch cause and
  both epoch values — not a generic identity refusal, and not a `Bool`.
  **Cardinality: exactly 1 refusal, 0 admissions.**
  The RED traverses `fleet_desired_accepted_from_event`, so at least one real production-composition
  execution is enrolled; a Git-backed reader test and a typed admission-seam test may carry the
  cheaper axes, but they do not replace that execution.

- **Paired control.** The same event at the matching epoch admits, through the same fold, with every
  other axis untouched. **Cardinality: exactly 1 admission, 0 refusals.**
- **Oracle independence, stated at the grain that matters.** "Expected comes from the constructor,
  the fold answers through admission" is *not* sufficient on its own. The executed one-axis control
  must have: the **required** identity derived from the Required CI authority; the **observed**
  identity decoded from the triggering-run subject at `event.head_sha`; the mutation applied to the
  **observed epoch only**; and an oracle that pattern-matches the exact mismatch cause and both
  epochs **without calling the identity equality/mismatch fold under test**. The RED must traverse
  `fleet_desired_accepted_from_event` — the live fleet-desired production composition — or the exact
  replacement production symbol Cut A introduces. A direct call to a generic identity comparator
  remains a useful unit witness but does not discharge the live production gap.

## 2. Cut B — the quality floor, instantiated for evaluation budget

### Subject, and why this consumer

`std.evaluation_budget` is a genuinely graded dimension — elapsed against a declared cpu or wall
limit — with typed verdict arms, and it is consumed in production by `v1_interpreter`, `cli_run` and
`v2.workflow.required_floor`, not only by witnesses.

### The consequence-ownership adjudication, and its evidence

The module's header says the kernel is caller-agnostic and each caller maps the result into *its own
refusal vocabulary*; the module also declares one stable machine code. Those can only coexist under
one terminal reading, and the census decides it:

1. The caller-specific vocabulary **exists and is typed**: `v1_interpreter` carries
   `EvaluationBudgetExceeded` (the neutral arm this carrier motivated), `EvalBudgetExceeded` (witness
   lane) and `WitnessWallBudgetExceeded` (falsifier lane). "Caller vocabulary" is realized as typed
   arms and guidance, not as wire codes.
2. The code's own note claims a *consumer-facing* stable identity, not a boundary-local one.
3. Production names std as the owner in its own words: the doc comment on `serve_budget_refusal_body`
   reads *"The stable `code` is the contract — `std.evaluation_budget`
   `evaluation_budget_refusal_code`"* — and then hand-spells the literal anyway.
4. **The discriminator.** A caller-owned code would be a member of a caller refusal protocol. There
   is no such protocol: `"code":"` occurs exactly once in `cli_run`, and it is this one. A family of
   size one is not a vocabulary.

**Terminal A.** The code is the universal machine identity of the generic breach. std keeps it, the
typed breach structurally determines it, production consumes the sole authority, and the CLI literal
disappears as an independently authored fact.

### The inverse quality order — the law, not examples of it

Elapsed time is a *cost*, so quality runs against it. Stated as a law, because without it a numerical
ceiling has merely been renamed a quality floor:

```
quality(a) >= quality(b)   iff   elapsed(a) <= elapsed(b)

meets the floor            iff   elapsed <= limit
breaches the floor         iff   elapsed >  limit
```

Equality therefore belongs to the *admitted* side by the law itself, not by convention — which is
also what the existing model already does, so the construction inherits the boundary rather than
re-choosing it.

### Two binding modelling details

- **`LimitUnset` constructs no floor on that clock** — not an infinite threshold and not a trivially
  satisfied one. A contract may carry one bounded clock and one deliberately unbounded clock; the
  bounded axis is the quality-floor obligation. (`PositiveMillisecond` already makes a zero-budget
  policy unwritable; this is the same discipline on the other end.)
- **The consequence is derived from the closed exceeded arm**, never accepted as caller-supplied
  text. A constructor taking arbitrary consequence text would make malformed contracts writable and
  relocate the invariant into callers.

### The terminal consequence law

```
EvaluationWithinBudget   -> no budget-refusal consequence

EvaluationBudgetExceeded { contract identity, entry, clock, elapsed, limit }
  -> a typed exceeded consequence
  -> code "evaluation_budget_exceeded"
```

Two constraints follow, and both are prohibitions rather than preferences.

**No freely writable consequence field.** A `consequence_code: NonEmptyStr` on the policy, on the
contract identity, or on the exceeded arm would make a budget contract with an arbitrary or
self-contradictory consequence *constructible*. There is no such field.

**The typed breach must determine delivery, not merely own the bytes.** Today's zero-argument
`evaluation_budget_refusal_code()` is an authority over the bytes, but by itself it does not
establish that the typed cause determines their delivery — production can still select the code
independently. Cut B closes that edge: the machine code is projected through the exceeded arm, or
through a consequence value only that arm can produce. The required property, stated so the
implementation shape stays free:

> A production `evaluation_budget_exceeded` response exists **because the execution produced the
> generic exceeded cause** — not because the boundary separately chose the same text.

Whether that is a fold over the verdict, a projection from a typed consequence, or an exhaustive
boundary mapping is left open; the property is not.

### One entry, projected from the identity — never duplicated beside it

`EvaluationBudgetPolicy` today stores `entry`, `cpu_limit`, `wall_limit`, and
`EvaluationBudgetExceeded` independently carries `entry`. Once the evaluation contract's canonical
name **is** the entry identity, a policy carrying both a contract *and* a separate `entry` field
would admit a policy about two different entries. That shape is invalid.

The terminal construction is therefore:

```
EvaluationBudgetPolicy { contract, cpu_limit, wall_limit }
entry(policy) = policy.contract.name
```

or an enclosing contract carrying the policy, with `entry` existing in exactly one place. The same
rule binds the result: "the breach carries exact identity and exact entry" means the entry is
**projected from the identity**, not supplied independently beside it. The machine response may
render an `entry` member, but its value derives from the one canonical identity.

Both sides of the grade decision stay identity-bound — an assessment carries the contract together
with its `Within | Exceeded` verdict. A bare `EvaluationWithinBudget` value, detached from the
contract it assessed, must not cross the production boundary to be joined by convention later.

### How production consumes the single authority

Not a grep equality gate over two literals — that retains the fork and adds a wall around it. The
repo already **emits Rust into the seed** from authorities (`v1_interpreter_dispatch_generated.rs`
from `gunbc.v1_interpreter_primitive_surface`; three stage0 files from `v2.compiler.self_host.*`),
regenerated by `main_wet` and drift-gated, and hand-written seed already consumes generated symbols
(`$crate::v1_interpreter_dispatch_generated::…` from `v1_interpreter`). So the emission route
demonstrably reaches hand-written seed call sites, and the serve refusal body can consume an emitted
constant rather than spelling one.

The seed bridge is therefore accepted only with: one source of consequence identity; automatic
production of the Rust representation; a perturbation control proving production moves with the
authority; an honest rung; and a dissolution trigger at self-host cutover. This must not read as
cementing Rust into a template — the emitted artifact is one authority-derived identity, not logic.

### Acceptance receipt for Cut B — two instruments, reported separately

**(i) Semantic breach RED.** Hold contract identity, consequence, entry and the other clock fixed;
cross exactly one declared limit at `elapsed = limit + delta`. Required outcome: **exactly one**
`EvaluationBudgetExceeded` carrying the exact contract identity, entry, clock, elapsed, limit and
typed consequence. **Cardinality — "exactly one" identified at the correct grain: one bounded clock crossed, one
`EvaluationBudgetExceeded` cause, one named machine consequence, one production refusal response;
0 admitted.** The other clock, the contract identity, entry, epoch and the consequence authority all
stay unchanged.
Paired controls, unrelated axes fixed: `elapsed == limit` admits, and `elapsed < limit` admits
through the same arm — equality already belongs to the admitted side in the existing model.
**Cardinality: 2 admissions, 0 exceeded.**

**(ii) Authority-wiring falsifier — a moved value, not a refusal.** For a *value* perturbation,
refusal is **not** an acceptable success: an emitter that always refuses would satisfy that test
while proving no authority-to-production binding at all. The required falsifier changes only the
`evaluation_budget_refusal_code` authority and then requires, in order:

- **without regeneration** — generated-artifact drift is RED;
- **after regeneration** — the generated Rust contains the perturbed value; the exact seed binary
  rebuilds; the real budget-exceeded path executes; **exactly one** machine refusal carries the
  perturbed code; and **zero** instances of the former code occur in that response.

**Cardinality: 1 perturbation → 1 drift RED before regen, 1 moved value after; 0 stale greens, 0
occurrences of the former code.**

A separate exhaustiveness mutation — adding a new consequence arm and requiring compilation or
generation to refuse until the fold handles it — is worth enrolling, but it is **not** a substitute
for the value-propagation falsifier and is reported separately.

### The generated projection's enrollment chain, proved rather than asserted

Reachability was the plan's remaining discovery obligation, so it is discharged here with the actual
symbols, censused at `main@1635a9ee06524711f8b7ff8def4dae9f9715b727`:

```
std.evaluation_budget                       authority over the consequence identity
  -> an authority-reading emitter fn        (beside expected_v1_interpreter_dispatch_generated_rs)
  -> a GeneratedArtifact variant            gunbc.generated_artifact, in the coproduct AND the
                                            committed roster, with an ArtifactLocation, a
                                            CommitRequired policy and an equality arm
  -> an artifact_generate arm               gunbc.generated_artifact_emit
  -> main_wet generation + drift            the same gate that adjudicates every committed artifact
  -> stage0 crate inclusion                 pub mod in src/v1/stage0/src/lib.rs
  -> serve_budget_refusal_body reads it     cli_run, the executing consumer
  -> rebuilt seed binary
  -> executed refusal response carries it
```

**What the enrollment chain does and does not guarantee — the earlier claim is withdrawn.** A prior
draft asserted that a half-enrolled artifact cannot exist because every relevant site is an
exhaustive match. That is false of the population: `artifact_location`, `artifact_commit_policy`,
`artifact_eq` and `artifact_generate` are exhaustive, but `generated_artifact_registry` is an
**explicitly authored `List<GeneratedArtifact>`**. A new variant can be added to the coproduct and to
every exhaustive match while being omitted from that list, and the tree still compiles — in which
state `committed_generated_artifact_paths`, `main_wet` and the drift boundary never see the artifact.
The downstream path derivation begins from that authored population, so it cannot discover its own
missing member.

The narrower claim that survives, and it is the only one made here:

> Once a rostered variant is reached, its location, commit policy, equality and generation dispatch
> are compile-time exhaustive.

That is not population completeness. Cut B therefore takes the **mechanically preventive** correction
rather than claiming structural impossibility: a discriminating test in which the new variant and
every match arm remain present and **only its registry member is removed**, required to go RED.
**The honest rung for enrollment completeness is mechanically preventable, not structurally
impossible.** Deriving registry membership from the coproduct population would raise it, and is named
here as the next-rung trigger rather than smuggled in as an achievement.

**No semantic hitchhiking.** The consequence does **not** go into
`v1_interpreter_dispatch_generated.rs` merely because that path already exists: that artifact's
subject is interpreter primitive dispatch, and mixing two authorities into one generated file for
convenience is the §3 fork wearing a generated file's clothes. Cut B adds a **dedicated** generated
Rust projection.

**What is and is not forbidden.** The generated Rust may contain the literal, because it is a
mechanically derived projection of its authority. The forbidden state is the *independently
maintained* literal in `cli_run`. A generated file that is never compiled, or a compiled module the
response body never reads, is an unused twin and fails the consumer bar — which is exactly what the
two-part falsifier below is shaped to catch.

## 3. Closure honesty

| kind | item |
| --- | --- |
| authority (A) | the canonical contract identity carrier; `gunbc.fleet_revision_acceptance` `RequiredCiContractIdentity` becomes an instance |
| consumer (A) | `accept_required_ci_workflow_run` (the Required CI adjudication), reached in production through `fleet_desired_accepted_from_event` (the live fleet-desired composition) |
| observation (A) | the declared epoch emitted into `WitnessFloorYamlArtifact`, plus the decoder and the exact-commit read at `event.head_sha` that makes the **observed** epoch real |
| authority (B) | `std.evaluation_budget` — contract identity, bounded-clock policy, typed consequence |
| consumer (B) | the serve refusal path in `cli_run`, and the required-floor budget check |
| generated projection (B) | the emitted Rust carrying the consequence identity, regenerated by `main_wet` and drift-gated — enrolled as artifact identity, path, commit policy, generation arm, emitted file, crate enrollment, drift adjudication and executing consumer |
| witnesses | the epoch control and its pair (A); the semantic breach RED and its two controls, and the wiring falsifier (B) |
| dissolved by landing | this plan file, deleted by the second cut — its consumers are the two cuts, and a retained copy would be a second prose representation of a live authority |

## 4. Contribution boundary

**No generated budget-consequence bridge belongs in Cut A** merely because one plan describes both
cuts. Cut A lands and its tree is verified before Cut B mutates anything.

**Census base.** The reachability chain and every cited shape above were censused at
`main@1635a9ee06524711f8b7ff8def4dae9f9715b727`. The authority and consumer paths are rechecked immediately before the
contribution boundary is frozen, since main advances continuously.


Cut A touches the identity authority, the fleet acceptance module, the epoch's emission into the workflow artifact and its exact-commit observation at `event.head_sha`, and the A witnesses. The observation is part of the boundary, not an implementation detail: without it the epoch is stamped rather than observed. Cut B touches
`std.evaluation_budget`, its emitted projection, the seed consumer call site, and the B witnesses.
Neither cut edits DESIGN.md or `gunbc.design_document`: this plan derives from §3 and §4b as they
already stand, and adds no paragraph to either.
