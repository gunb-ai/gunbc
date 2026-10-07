# Served-model identity and endpoint selection — design

Status: **design, model-before-implement.** No `.dag` lands with this document. It names the homes,
the deliveries and the operator rulings still open. Owner: sunny-deer-146 (operator assignment,
2026-10-07). Contributing lanes: valiant-crab-775 / cool-carp-342 (Group A deployment record) and
smart-owl-201 (per-checkpoint weight scheme).

Revision 2 incorporates the side-chat review of revision 1 (fb5d363). The changes:

- admission becomes the mandatory input to every invocation;
- hard capacity admission uses the actual request bounds;
- the `CheckpointQuantization` and `HarnessServingUnit` migrations are decomposed rather than
  replaced by one identifier;
- deployment records are bound to the answering launch;
- the session's accepted alternatives are separated from the selected offer;
- several OpenRouter and pricing claims are corrected;
- the deliveries are re-sequenced around working consumers.

## 0. The defect

Every consumer that chooses where a model turn runs keys on a **model name string**, and that name
is a meaning fork (DESIGN §3): within one routing surface and one epoch, `glm-5.3-flash` names two
materially different contracts.

- **Sparks.** Group B serves the native fp8 checkpoint. Group A was brought up with NVIDIA's nvfp4
  checkpoint. Both answer `/v1/models` as `glm-5.3-flash`.
- **gunbc harness.** `gunbc.harness.harness_backend` `harness_observe_replica` admits the first
  advertised id that matches `gunbc.serving.admitted_model` `serving_admitted_units`. The closed unit
  `HarnessServingUnit::Glm53FlashOnVllm` covers both checkpoints.
- **ctrl mini-agent.** It has the same name-only join on the Sparks. On OpenRouter it ranks
  endpoints without reading `quantization`.

Worse, ctrl's selection is not binding on what it sends:

- When `openrouterRouting` finds no admissible endpoint, it returns `lookup_error`. `runTurn` logs
  this and sends the turn as plain `:floor`, which is OpenRouter's own selection with none of our
  exclusions.
- Compaction (`compactMessages`) and the step-limit wrap-up call never consult the selector at all.

So a stricter selector alone would *move* traffic onto the bypass rather than stop it.

The supplier-side facts already exist. `gunbc.spark.serving_arm` `ServingCheckpointRow` carries
checkpoint, revision and `CheckpointQuantization`, and `ServingRuntimeRow` carries the runtime's
reasoning projection. What is missing is the connection between four things: what the caller
requires, what a supplier establishably offers, what is selected, and what is actually invoked.
The earliest unjustified boundary (§6b) is therefore **the invocation**: it does not consume a
selection result. The served-name string is the second.

Interim mitigation, outside this design: ctrl#2282 delisted Group A's head from ctrl routing. Its
retirement trigger is stated in §6.

## 1. Demand and offer are different facts

This follows the distinction `gunbc.serving.turn_demand` `ServingTurnRequest` already draws. A
caller's requirement exists before supplier selection, and a supplier's implementation is offer
evidence. They meet in one compatibility relation.

**Demand: the session's accepted alternatives.** A session carries an explicit set of complete
implementation alternatives. Each is a row, never a cross product of independent sets:

| Alternative | Artifact | Revision | Weight representation |
|---|---|---|---|
| A | `zai-org/GLM-5.3-Flash` | its pinned revision | fp8 |
| B | `nvidia/GLM-5.3-Flash-NVFP4` | `da920bb0…` | nvfp4 |

Accepting A and B does not accept "native artifact, revision A, nvfp4". An exact-artifact
requirement is a singleton set. Two further policy facts are separate from this set: whether a
session may move between accepted alternatives turn to turn, and whether it should stay on one
endpoint.

**Offer: what a supplier establishably provides,** at a stated evidence strength:

- **Local (Sparks):** a checkpoint declaration bound to the answering launch (§2). Its strength is
  *established*.
- **External (OpenRouter):** the endpoint list's declaration (§3). Its strength is *declared by the
  provider*. A requirement that demands an exact checkpoint cannot be satisfied by a declaration
  that does not establish one. That requirement stays unsatisfied, and renaming a provider string
  cannot supply the missing evidence.

**The checkpoint facts are several facts, not one field.** `CheckpointQuantization` distinguishes
quantization *spelled on argv* from quantization *resolved from the checkpoint's config*. That
distinction decides whether `checkpoint_quantization_argv` emits a flag at all, and its `wire` can
name a **loader method** (`modelopt`) rather than a numeric representation. The existing rows show
why one flat tag cannot replace it:

- The NVIDIA GLM row reads a modelopt config that specifies nvfp4.
- The DeepSeek V4.1 row's config method is `fp8` with fp4 expert dtype.

`CheckpointWeightScheme` (smart-owl-201) and `CheckpointQuantization` are therefore reconciled as
**distinct facts derived from one canonical checkpoint declaration**:

- the weight representation, which may be per-component as in mixed expert dtypes;
- the loader/config source;
- argv emission or omission.

Neither is deleted until their relationship is modeled.

**`HarnessServingUnit` carries protocol behavior, not only identity.** Its consumers select:

- the request reasoning dialect (`harness_reasoning_kwargs_for`);
- the tool-call parser (`harness_unit_tool_call_parser`);
- the response framing (`harness_unit_reasoning_projection`);
- server-side vs. client-side reasoning split (`harness_unit_server_parses_reasoning`);
- the declared context ceiling (`harness_unit_declared_context_ceiling`).

Dissolving the enum means decomposing these roles into:

- **model/artifact identity**, the demand-side key;
- **serving behavior**, which request encoding and response decoding need. Where that is a runtime
  fact, it is read from `ServingRuntimeRow`, not re-derived from the checkpoint.

Replacing the enum's whole authority with a checkpoint reference would recreate the conflation at a
narrower grain.

The served name (`--served-model-name`) is a **wire alias**: realization, not identity.

## 2. Local deployment record, bound to the answering launch

There is one record per serving group, derived from the arm's own rows and never hand-typed. Owner:
cool-carp-342's lane.

```
group · head endpoint · wire alias · ServingCheckpoint · runtime · arm intent
```

A published record is a **declaration** until it is bound to the process that actually answers.
The machinery exists:

- `gunbc.spark.vllm_endpoint_process_launch` `VllmEndpointProcessLaunch` identifies the group,
  endpoint and process-start discriminator;
- `gunbc.spark.serving_offer` `spark_service_from_observation` checks the answering endpoint,
  placement, declared-unit provenance and rendezvous consistency.

The offer that selection consumes is the *bound* record. That binding is carried into selection and
into the invocation, so a replaced process invalidates its predecessor's selection even when the
alias is unchanged.

**Alias uniqueness** is required within the namespace an actual dispatch resolves. Two different
checkpoints may not share an alias that one dispatch resolves by name. It is not a universal
prohibition on a logical model name that groups several *accepted* implementations; that grouping
is the demand side's job (§1), not the alias's.

**Operator-qualification serving** (operator ruling 2026-10-07, B) is a third `ArmIntent` arm, and
it is meaningful only because admission consumes it. A qualification deployment is admitted for
requests that name it, not for ordinary traffic, so publishing one never enrolls it into the fleet's
general pool.

## 3. OpenRouter as a cited upstream

A new `extdeps.openrouter` module models the endpoint-list API as documented. The source is
`GET /api/v1/models/{author}/{slug}/endpoints`, with OpenRouter's own field names and the API
version read stated. Its observing adapter and first real consumer land with it, never the shape
alone (§3c). Observations are receipts in the observing layer, not upstream facts.

Corrections to revision 1's reading:

- **`name` is a display name**, not a model revision. The dated suffix seen on live responses is not
  a documented revision contract. A requirement for an exact revision is unsatisfiable from this
  API unless a field that establishes it is found.
- **`quantization` is a provider-declared category.** `fp4` spans MXFP4 and NVFP4, and `fp8` spans
  MXFP8. It establishes a category, never a subtype, and no "wider is acceptable" ordering exists in
  the API. `unknown` is its own arm.
- **Service tier is part of the offer.** `:floor` admits flex endpoints, and the reported serving
  tier determines the billed rate. Tier is carried as permitted (demand), established (offer) and
  reported (result).
- **`pricing.discount`** is recorded as observed. No expiry or future price is inferred from it, and
  it is applied exactly once, as the API defines. Revision 1's "temporary promotion priced over a
  horizon" is withdrawn.
- Data policy and region (`/zdr`, `/us` in tags): whether a structured field establishes them is a
  read obligation of this delivery. Tag spelling is not evidence.

## 4. Selection, and the invocation that must consume it

Selection inhabits the existing homes (§3b decision/selection and fabric/compute rows) and mints no
new decision algebra (§3d). Those homes protect only the obligations represented in their inputs,
so every property below arrives as a **declared obligation** of the request:

- **Compatibility.** The offer satisfies one accepted alternative at the evidence strength the
  requirement demands.
- **Capacity, against the actual request.** Let `P` be the rendered prompt (system text and tools
  included), `O` the completion budget actually emitted on the wire, `C` the context capacity, and
  `P_max` / `O_max` the endpoint's own caps where defined. The offer is admitted only if:
  - `P ≤ P_max`
  - `O ≤ O_max`
  - `P + O ≤ C`

  Predictions of output and cache behavior belong to **cost ranking**, not admission. If the
  allowance is reduced to fit, the reduced value is the admitted result, and it is what the request
  emits.
- **Parameters.** Tools and reasoning are present where required.
- **Policy rows** from §5: permitted tier, data policy.
- **Health** is a declared predicate, not a word: what is read (`status`, an uptime window, our own
  cooldown) and the threshold. It is a policy row.

**Missing evidence for a required property cannot satisfy the request.** It is
`SelectionNeedsEvidence` or a typed rejection. This closes ctrl `routing.mjs` `admit()`, where an
absent `context_length` or `supported_parameters` counts as fitting.

**Ranking** is cost over the admitted set, on the priced axes of `product.fabric.selection`
`select_supply`.

**The invocation consumes an admitted result, and nothing else.** Every covered model call takes
one input, the **admitted invocation**, which binds:

- the normalized request;
- the selected offer, with its launch binding (local) or provider order (external);
- the permitted routing, which for OpenRouter means `provider.order` = the selection and fallbacks
  off;
- the applicable policy.

A refusal or missing-evidence result has **no inference route**. There is no `:floor` fallback.

This applies to ordinary turns, compaction, the wrap-up call and retries. Each derives its own
functional requirements (a compaction prompt has its own `P` and `O`), while keeping the session's
restrictions.

**Endpoint affinity is policy, not fact.** Today `runTurn` re-plans every step and `planRoute` may
move a session for projected cost. Whether that continues is the affinity row of §1, stated
explicitly. Revision 1's claim that "a session holds its endpoint for life" was false and is
withdrawn.

**Per-call record.** The record keeps four kinds of fact distinct:

- requested: the accepted alternatives and the bounds;
- advertised: the offer's declarations;
- enforced: what the request bound, e.g. the provider order;
- reported: what the supplier says served, the charge, and the tier.

Reported charges stay distinct from our estimates.

ctrl's mini-agent sits outside the `.dag` closure. It consumes this contract through an emitted
projection or a served decision, chosen in delivery 1. It does not keep a second implementation.
Until then ctrl's routing is a declared, bounded divergence (§3b) with this document as its trigger.

## 5. Operator rulings

These gate the **activation** of particular admission policies, not the local serving contract.

| Question | Recommendation | Gates |
|---|---|---|
| Allowed implementations per model | Explicit complete alternatives per model/workload (§1 rows). No global "fp8 or better", since e.g. Moonshot's own Kimi K3 endpoint is mxfp4 | activating that model's admission policy |
| Undisclosed quantization (`unknown`) | Insufficient for any requirement that names an allowed scheme; recorded honestly, never coerced | routes using such a requirement |
| Repository data policy | ZDR-only — **a proposal, not adopted policy**, until the operator rules | activating external routing for repository-bearing sessions |

## 6. Deliveries

Each delivery lands with its working consumer (§3c). None is a scaffold. Minimal per-call identity
and provenance are part of the first delivery that executes, not a later one.

1. **Local serving contract.**
   - Reconcile the checkpoint facts (§1) and decompose `HarnessServingUnit` into identity plus
     serving behavior.
   - Bind the deployment record to the answering launch (§2), including the third `ArmIntent` arm.
   - Carry the caller's accepted alternatives through `harness_observe_replica` / harness seat
     selection and through ctrl's Sparks path into the invocation.
   - Make the admitted invocation mandatory on every covered call, including ctrl's compaction and
     wrap-up.
   - It needs no §5 ruling.
2. **OpenRouter contract.** The cited `extdeps.openrouter` shape, its observing adapter, and ctrl's
   OpenRouter route as consumer: the admitted requirements become the actual request controls, and
   the `:floor` fallback is deleted. The §5 rows activate here as they are ruled.
3. **Pricing and presentation.**
   - Offer-specific cost comparison.
   - Identity-keyed shadow pricing for self-hosted models: ctrl `lib/model_pricing.mjs` currently
     strips the `:suffix` and prices nvfp4 at the fp8 reference. Where no honest reference exists
     for an identity, the answer is "no reference".
   - Reporting that keeps reported charges apart from estimates.

**ctrl#2282 is retired by working behavior,** not by a published record. Delivery 1 must show that
ctrl's Sparks route enforces the accepted-alternatives requirement at invocation.

**Discriminating acceptance for delivery 1:**

- An fp8-only requirement cannot execute against the nvfp4 offer.
- Rejecting every offer produces no inference request on any path (turn, compaction, wrap-up,
  retry).
- Compaction and wrap-up keep the session's restrictions.
- The actual request bounds, not predictions, govern admission.
- Replacing the answering process invalidates the predecessor's binding.
- The migration preserves quantization loading behavior (argv emission/omission) and runtime
  response decoding.

## 7. Out of scope

- Which models to offer at all. That is the operator's shortlist and stays a configured list.
- Quality qualification of a given implementation (the nvfp4-vs-fp8 comparison the operator is
  running). This design makes that comparison attributable; it does not perform it.
