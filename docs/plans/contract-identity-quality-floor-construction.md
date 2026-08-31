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

- **Contract identity is not greenfield.** `gunbc.fleet.fleet_revision_acceptance` already carries
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

`required_ci_admission` checks the workflow path and the repository, each with its own refusal arm,
and then admits. **It carries no epoch.** So a Required CI contract whose obligations changed
materially — a different floor composition, a different admission policy — is admitted by the fleet
under the same `workflow_path`, because the identity holding the name constant has no way to say
*which version of this contract*. That is exactly the §3 meaning fork: one name, two materially
different meanings, silently. The gap is authorable and therefore testable.

### The construction

One canonical contract identity whose grain is the §3 triple:

- **naming surface** — the space within which the name is held constant;
- **name** — the contract's stable name in that surface;
- **epoch** — the declared effective version, without which two materially different contracts
  cannot be distinguished.

The equality law is stated once, on the carrier, and is field equality over exactly those three —
never over the contract's internals, preserving the fleet note's rule. Each component mismatch keeps
its own refusal cause, and no arm projects to `Bool`.

`RequiredCiContractIdentity` becomes an instance of that identity rather than a parallel authority.
Its existing `workflow_path` and `repository_full_name` refusal arms survive as the surface and name
components; the epoch is new.

### Consumer, and what makes the epoch load-bearing

The consumer is the existing production fold `required_ci_admission`. The epoch is load-bearing
because a cross-epoch composition **refuses with an exact contract-epoch mismatch cause before the
outcome is interpreted**. An epoch carried into a receipt that no decision reads would not count,
and is explicitly not what this cut lands.

### Acceptance receipt for Cut A

- **Epoch control (the discriminating RED).** Hold naming surface, name, revision, evidence and
  conclusion fixed; vary **only** the epoch. Required outcome: exactly one refusal, carrying the
  exact contract-epoch mismatch cause — not a generic identity refusal, and not a `Bool`.
  **Cardinality: exactly 1 refusal, 0 admissions.**
- **Paired control.** The same event at the matching epoch admits, through the same fold, with every
  other axis untouched. **Cardinality: exactly 1 admission, 0 refusals.**
- **Oracle independence.** The control constructs its expected identity from the identity authority's
  own constructor; the fold under test reaches its answer through `required_ci_admission`. Neither
  reconstructs the other's answer through a shared helper.

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

### Two binding modelling details

- **`LimitUnset` constructs no floor on that clock** — not an infinite threshold and not a trivially
  satisfied one. A contract may carry one bounded clock and one deliberately unbounded clock; the
  bounded axis is the quality-floor obligation. (`PositiveMillisecond` already makes a zero-budget
  policy unwritable; this is the same discipline on the other end.)
- **The consequence is derived from the closed exceeded arm**, never accepted as caller-supplied
  text. A constructor taking arbitrary consequence text would make malformed contracts writable and
  relocate the invariant into callers.

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
typed consequence. **Cardinality: exactly 1 exceeded, 0 admitted.**
Paired controls, unrelated axes fixed: `elapsed == limit` admits, and `elapsed < limit` admits
through the same arm — equality already belongs to the admitted side in the existing model.
**Cardinality: 2 admissions, 0 exceeded.**

**(ii) Authority-wiring falsifier.** Perturb the one consequence-code authority. Required outcome:
the production-emitted code moves with it, **or** generation/compilation refuses because the
production projection is no longer exhaustive. A stale production string is RED. This is *not* the
quality breach; it establishes that the named consequence consumed by production has one authority.
**Cardinality: 1 perturbation, 1 refusal-or-moved-value; 0 stale greens.**

The two are reported with separate cardinalities. "Some RED occurs" establishes nothing.

## 3. Closure honesty

| kind | item |
| --- | --- |
| authority (A) | the canonical contract identity carrier; `gunbc.fleet.fleet_revision_acceptance` `RequiredCiContractIdentity` becomes an instance |
| consumer (A) | `required_ci_admission`, the existing production fold |
| authority (B) | `std.evaluation_budget` — contract identity, bounded-clock policy, typed consequence |
| consumer (B) | the serve refusal path in `cli_run`, and the required-floor budget check |
| generated projection (B) | the emitted Rust carrying the consequence identity, regenerated by `main_wet` and drift-gated |
| witnesses | the epoch control and its pair (A); the semantic breach RED and its two controls, and the wiring falsifier (B) |
| dissolved by landing | this plan file, deleted by the second cut — its consumers are the two cuts, and a retained copy would be a second prose representation of a live authority |

## 4. Contribution boundary

Cut A touches the identity authority, the fleet acceptance module, and the A witnesses. Cut B touches
`std.evaluation_budget`, its emitted projection, the seed consumer call site, and the B witnesses.
Neither cut edits DESIGN.md or `gunbc.design_document`: this plan derives from §3 and §4b as they
already stand, and adds no paragraph to either.
