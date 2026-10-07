# Served-model identity and endpoint selection — design

Status: **design, model-before-implement.** No `.dag` lands with this document. It names the homes,
the phases and the operator rulings still open. Owner: sunny-deer-146 (operator assignment,
2026-10-07). Contributing lanes: valiant-crab-775 / cool-carp-342 (Group A deployment record) and
smart-owl-201 (per-checkpoint weight scheme).

## 0. The defect, in one paragraph

Every consumer that chooses where a model turn runs keys on a **model name string**, and that name
is a meaning fork (DESIGN §3): within one naming surface and one epoch, `glm-5.3-flash` names two
materially different contracts.

- **Sparks.** Group B serves the native fp8 checkpoint. Group A was brought up with NVIDIA's nvfp4
  checkpoint. Both answer `/v1/models` as `glm-5.3-flash`.
- **gunbc harness.** `gunbc.harness.harness_backend` `harness_observe_replica` admits the first
  advertised id that matches `gunbc.serving.admitted_model` `serving_admitted_units`. Its closed unit
  `HarnessServingUnit::Glm53FlashOnVllm` covers both checkpoints, so a seat can bind to either
  without the turn ever learning which.
- **ctrl mini-agent.** It has the same name-only join on the Sparks. On OpenRouter it ranks
  endpoints by step price without reading the per-endpoint `quantization` field. The cheapest
  endpoints are mostly fp4 or `unknown`, so "cheapest" largely means "most quantized or undisclosed".

The quantization fact already exists in the corpus, at
`gunbc.spark.serving_arm` `ServingCheckpointRow.quantization` (`CheckpointQuantization`), keyed by
`ServingCheckpoint` (`Glm53NativeFp8`, `Glm53FlashNvidiaNvfp4`). It just never reaches the
selection. This is a §6b case: the symptom is observed at selection, and the earliest unjustified
boundary is the **served identity**, which is a string where a checkpoint reference was available.

Interim mitigation already landed outside this design: ctrl#2282 delisted Group A's head from ctrl
routing. That is an exclusion, not a second copy of the quantization fact. It is retired by phase 2
below.

## 1. What a served model IS (the identity)

A turn's model identity is **(upstream model, checkpoint revision, weight scheme)**. It is not a
served name. Each component already has, or is gaining, one home:

| Component | Home | State |
|---|---|---|
| upstream model | `extdeps.zhipu.glm_5_3_flash` `glm_5_3_flash_model_id` (and each vendor's module) | exists |
| checkpoint + revision | `extdeps.nvidia.glm_5_3_flash_nvfp4` `glm_5_3_flash_nvfp4_manifest` (and peers); fleet projection `gunbc.spark.serving_arm` `ServingCheckpointRow` | exists |
| weight scheme | `CheckpointWeightScheme` (smart-owl-201, in flight): a closed, cited scheme per checkpoint (fp8 / nvfp4 / mxfp4 / bf16 …) | in flight |

The served name (`--served-model-name`) is demoted to a **wire alias**. It is realization, not
identity (§3: interface, realization and policy are three facts). Two deployments of different
checkpoints may never share an alias within one routing surface; that becomes a refusal, not a
convention (phase 2).

`CheckpointQuantization` and `CheckpointWeightScheme` must not become two answers to one question.
The scheme is the identity fact; the argv/config spelling is how a launch realizes it. Phase 1
reconciles them as a **replacement migration** (DESIGN §3, cut over at the root), not as a second
field beside the first.

## 2. The deployment record (Sparks)

There is one record per serving group, derived from the arm's own step rows and never hand-typed.
Owner: cool-carp-342's lane.

```
group · head endpoint · wire alias · ServingCheckpoint · weight scheme (derived) · arm intent
```

`ArmIntent` today is `ArmBoundedExperiment | ArmNormalServing`. The operator-qualification serving
mode (operator ruling 2026-10-07, B) is a third arm with its own name, so "normal serving" keeps one
meaning.

**Consumers, which DESIGN §3c requires to be named:**

- **gunbc harness.** `harness_observe_replica` stops admitting by advertised name. It joins the
  observed alias to the record, and the candidate carries the `ServingCheckpoint`.
  `HarnessServingUnit` either splits per checkpoint or is replaced by the checkpoint reference. Which
  of the two is a phase-1 modeling question, with a bias to replace, per the replacement-migration
  doctrine.
- **ctrl mini-agent.** It reads the published records instead of its hard-coded URL list (ctrl
  `sparks_endpoints.mjs` `DEFAULT_ENDPOINTS`), and a session pins an identity, not a name.

**Transport** (how ctrl reads the record: front-door endpoint vs. emitted file) is a realization
choice, made in phase 2. It is not part of the record.

## 3. OpenRouter as a cited upstream

A new `extdeps.openrouter` module models the endpoint-list API as it actually returns. The source is
`GET /api/v1/models/{author}/{slug}/endpoints`, and the module keeps OpenRouter's own field names
and states the version read. It stores no observations (§3, external upstream decomposition: a
read is a receipt in the observing layer).

Fields that bear on selection, each on a live response read on 2026-10-07:

| Field | Why it matters |
|---|---|
| `quantization` | `fp4` / `nvfp4` / `mxfp4` / `fp8` / `bf16` / `unknown`, a different weight scheme, priced lower |
| `name` (dated build, e.g. `…-20260826`) | model version; two endpoints can serve different revisions under one id |
| `context_length`, `max_prompt_tokens`, `max_completion_tokens` | capacity; the output cap varies per endpoint (e.g. 131,072) |
| `pricing.{prompt,completion,input_cache_read,input_cache_write}`, `pricing.discount` | cost; `discount` is a temporary promotion, while a session holds its endpoint for life |
| `supported_parameters` | whether tools and reasoning are honored |
| `status`, `uptime_last_{5m,30m,1d}` | health |
| `tag` suffixes (`/zdr`, `/us`, `/fast`, `/flex`) | data policy, region and tier; whether a structured field exists for each is a read obligation in phase 3 |

An OpenRouter endpoint's weight scheme is mapped onto the same `CheckpointWeightScheme` vocabulary.
`unknown` is its own arm. It is never coerced to a scheme.

## 4. One selection, consumed by both harnesses

Selection inhabits the existing homes (§3b decision/selection and fabric/compute rows). It mints no
new decision algebra (§3d):

- **Hard constraints exclude before ranking** (`std.decision` `HardConstraint`; in the fabric fold,
  an `OfferCandidacy` rejection arm):
  - the weight scheme is in the admitted set for the model;
  - the model revision matches the session's pinned revision;
  - the data policy is admitted;
  - the context, prompt and output caps fit the predicted step;
  - the required parameters are present;
  - the endpoint is healthy.
- **An unread field is evidence-missing, never admitted.** A missing `context_length` or
  `supported_parameters`, or a `quantization: unknown` when the policy requires a known scheme, is
  `SelectionNeedsEvidence` / a typed rejection. This closes the fail-open arm currently in ctrl
  `scripts/mini-agent/routing.mjs` `admit()`, where an absent value counts as fitting.
- **Ranking** is cost over the admitted set, the same priced axes as `product.fabric.selection`
  `select_supply`. A `discount` is carried on the offer, and its horizon is priced against the
  session's expected lifetime rather than read as the steady-state rate.
- **Pinning.** The admitted identity set (model, revision, scheme set, data policy) is fixed at
  session creation and is part of the selection receipt. A resume that would land outside it is a
  different decision (§3d: different constraints mean a different decision), never a silent
  re-route.
- **Receipt per turn.** The served endpoint, scheme, revision and the discount in force are recorded
  with usage, so cost and quality reports say what actually ran.

ctrl's mini-agent is a JS consumer outside the `.dag` closure. It consumes this selection through an
emitted projection or a served decision, chosen in phase 4. It does not re-implement it. Until then,
ctrl's routing is a declared, bounded divergence (§3b, diverges with a stated reason) with this
document as its trigger.

**Shadow pricing for self-hosted models** (ctrl `lib/model_pricing.mjs`) currently strips any
`:suffix` and prices nvfp4 at the fp8 reference rate. Under this design the reference row is keyed
by identity, including scheme. Where no honest reference exists for a scheme, the answer is a stated
"no reference", never the neighbouring row.

## 5. Operator rulings this design needs

These are policy rows in the selection, not code. Each is open:

1. **Admitted weight schemes per model.** Either an explicit operator list per model, or "the model
   maker's own published scheme or wider". Note that Moonshot's own Kimi K3 endpoint is mxfp4, so
   one global threshold such as "fp8 or better" would exclude a first-party build.
   **Recommendation: the explicit list.** It is a handful of rows, and nothing is inferred.
2. **Undisclosed quantization (`unknown`).** Exclude or admit. **Recommendation: exclude**
   (fail-closed; a session must be able to say what it ran).
3. **Data policy.** Whether non-ZDR endpoints may receive repository contents. Default:
   **ZDR-only** until ruled otherwise.

## 6. Phases

Each phase lands consumed in the same change (§3c). None is a scaffold.

1. **Identity.** Land `CheckpointWeightScheme` (smart-owl-201) and reconcile it with
   `CheckpointQuantization`. Replace `HarnessServingUnit`'s name-keyed GLM arm with checkpoint
   identity. First consumer: `harness_observe_replica`.
2. **Deployment record.** cool-carp-342 adds the third `ArmIntent` arm and publishes the per-group
   record. The harness and ctrl consume it. A shared alias across checkpoints on one routing surface
   is refused. ctrl re-lists Group A, which retires ctrl#2282.
3. **`extdeps.openrouter`.** The endpoint shape, cited. Resolve the data-policy and region read
   obligation from the actual response.
4. **Selection.** Constraints and ranking over both supplies through the existing homes. The
   rulings from §5 land as policy rows. ctrl's routing switches to the shared decision and retires
   its own `admit()`.
5. **Accounting.** Per-turn receipts and identity-keyed shadow pricing.

## 7. Out of scope

- Which models to offer at all. That is the operator's shortlist and stays a configured list.
- Quality qualification of a given scheme (the nvfp4-vs-fp8 comparison the operator is running).
  This design makes that comparison *attributable*; it does not perform it.
