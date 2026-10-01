# XL-2: service interface, realization binding, and `uses` — carrier design

Status: design only, no behaviour change. Owner: XL-2 (quiet-seal-543). Decisions recorded below were ruled by the XL-2 manager on 2026-10-01.
Governs: `gunbc.recurring_failure_mode` `service_interface_member_has_no_carrier`, `gunbc.recurring_failure_mode` `uses_clause_has_no_carrier`, and the two `v2.workflow.compile_door_cause_ownership` rows for `body_lowering_reason_service_realization_unreachable` and `body_lowering_reason_uses_clause_unmodeled`.
History: #12277 (declaration-grade service lowering), #12816 (`uses` parses and refuses at the clause).

## The question

Two refusals in `v2.compiler.body_lowering_fold` keep whole modules out of the XL-2 reference census, which reads the **full** normalize route, not census grade:

- `body_lower_service_decl` refuses every service carrying a set-aside member as `body_lowering_reason_service_realization_unreachable`.
- `body_lower_fn_uses_refusal_optional` refuses every fn carrying a `uses` clause as `body_lowering_reason_uses_clause_unmodeled`.

For each member, this document asks what fact it is, who consumes it, and which existing authority already owns it. It then gives the smallest lowering change after which every member either lowers into its carrier or refuses at a located diagnostic. No member is dropped.

## Population (measurement, to be re-derived)

Measured by a source grep at the base of this change. This is a sizing, **not an oracle** (DESIGN §5). The authoritative population is the set of modules the CI census run refuses under each reason, re-measured after each PR below lands.

| Subject | Modules | Note |
| --- | --- | --- |
| declares a `service` | 114 | all refuse whole on the full route today |
| — with a realization member (`transport` / `config` / `exit` / `response` / `mock_response`) | 111 | |
| — with only interface members (modifiers, io tails) | 3 | |
| fn or pattern header carrying `uses alias: Type` | 30 (97 rows) | no overlap with the service modules; no row reads its alias in the body |

**Consequence:** an operation-modifier carrier alone, the trigger the failure-mode row named, unblocks 3 modules. The service frontier is the realization members. The `compile_door_cause_ownership` flip trigger already said so, and this design adopts that sentence as the scope.

## Inventory: what each member means

### Service members

| Member | Fact | Layer (DESIGN §3) | v1 semantics | Downstream consumer |
| --- | --- | --- | --- | --- |
| `operation Op { input {…} output {…} }` | an arrow of the interface | interface | operation signature | resolve (call targets), infer (call typing). **Lowered today.** |
| io field `= default` | the value an omitted input takes | interface | `v1.compiler` `make_field_node` `default_value` | infer (call-site arity: a defaulted field may be omitted), eval |
| `readonly` | a declared claim that the operation's effect shape is `ReadEffect` | interface (a claim about the arrow's effect) | `OperationModifier.Readonly` (`v1` `00_core` `field_init_operation_modifier`) | `std.effects` `check_modifier_vs_derivation`; effect_demand (a read never needs a write grant) |
| `idempotent` | a declared claim that the effect shape is idempotent | interface | `OperationModifier.Idempotent` | `std.effects` `check_modifier_vs_derivation` / `is_idempotent_effect`; retry admission (DESIGN §4b names a non-idempotent effect under retry) |
| `hermetic` | a declared claim that the operation is admissible on the hermetic execution mode | interface (execution-mode axis, not an effect shape) | `OperationModifier.Hermetic` | `std.execution_mode` `Hermetic`; effect_demand keys mode as part of the demand |
| io field `from "key"` | which wire field fills a declared output | **realization** — a decode projection of one transport's observation | `field_node_from_key` | the transport's output projection (target_model / eval). It is not interface: another transport of the same shape need not have a wire key |
| `transport shell {…}` / `rest {…}` / `file` / local | which handler realizes the operation | realization | `classify_transport` | `std.effect_grant` `HandlerBinding` (`RealTransport`) via `extdeps.transports.shell` `ShellTransportConfig` / `extdeps.transports.rest` `RestTransportConfig` |
| `config {…}` | handler configuration at service grain | realization | service-level transport defaults | the same transport config, applied to each operation |
| `exit {…}` / `response {…}` | how an observation (exit status, streams, body) projects onto declared outputs | realization | output projection of the observation | the interpreter's declared-output projection (`v2.std.operation_realization` feeds it a `TransportObservation`) |
| `mock_response {…}` | one published response arm of one operation | realization (a hermetic-replay handler) | published mock | `std.hermetic_replay` `PublishedMockCase`, selected through `HandlerBinding.Replay` |

### `uses alias: Type`

| Fact | v1 semantics | Owning authority today |
| --- | --- | --- |
| the resources a fn's body demands | `v1` `04_items` `ResourceRequirement` (`binding_name`, `resource`), consumed by the Rust emitter's call-site binding | **D13** (`gunbc.plans.demand_engine_program`): for a transparent body, demand is DERIVED (`DependencyDemand`, produced once by resolution). Authored restatement rows are deleted (USES-0, `docs/plans/uses-occurrence-census.md`). Binder, opaque-contract and policy rows move to their own carriers. Derivation does not yet name Network (`gunbc.rung_drop` `network_requirement_unrepresented_after_uses_cut`) |

## One carrier per fact

### Interface: carried on the operation's Arrow and on its input binders (ruled 2026-10-01)

The first draft of this section put modifiers as edges on the operation node and defaults on the payload field. Neither is admitted by the substrate: `v2.std.node` `arrow_signature_edges_conform` counts every Arrow named edge other than the body and the declared order as the one type-binder edge, and a payload field is a bare name-to-type edge with no slot for a default. The side-chat ruling (option A) places them:

1. **Effect and execution-mode claims are Arrow contract edges, kept apart.** `readonly` and `idempotent` are effect claims, checked against the derived effect shape by `std.effects` `check_modifier_vs_derivation`. They lower onto `^arrow_effect_claims_edge`. `hermetic` is a `std.execution_mode` claim and lowers onto `^arrow_execution_mode_claim_edge`. The surface `OperationModifier` stays one syntax vocabulary, and lowering projects each arm to its semantic home. Arrow conformance enumerates the legal labels and target shapes. Duplicates and unknown labels refuse. Contract edges are not binders, and canonical identity is order-independent.
2. **A default belongs to the binder, not to its type.** One binder representation (a required type, an optional single default, closed labels, one reader and builder) is reused by fn parameters, service io fields and record fields. Because that changes the existing name-to-type binder shape, it is a replacement migration (DESIGN §3): every producer and reader moves, and the two shapes never coexist.

Once these carriers exist, `InterfaceMemberUnmodeled` has no producer for modifiers or defaults. Wire keys move to the realization carrier (below), so that set-aside kind is deleted, not left empty.

### Realization: a sibling node, never inside the interface

Per DESIGN §3, transport is one of N handlers bound to the interface shape, and "the dispatch that selects a realization is itself realization". So the realization members lower into **one sibling node per service**, `^service_realization_binding`, emitted beside the service node in the same module:

- It is keyed per operation by the operation's resolved identity, the shape `v2.std.operation_argv` `OperationRef` (`path`, `service`, `operation`) that `v2.std.operation_realization` `OperationBinding` and the grant's `HandlerBinding` selection already key on. No new identity is minted.
- `transport` / `config` lower onto the transport's existing config record (`extdeps.transports.shell` `ShellTransportConfig`, `extdeps.transports.rest` `RestTransportConfig`, `extdeps.transports.file`). A transport kind with no config record refuses located (`body_lowering_reason_service_transport_unmodeled`). It is never coerced to a neighbour.
- `exit` / `response` lower as the operation's output projection: expressions over the transport's observation fields, lowered through the ordinary value reader so their references reach the census.
- `from "key"` lowers as one row of that projection (declared output field ← wire key). That is where it belongs: it is meaningful only relative to the transport whose observation it decodes.
- `mock_response` lowers onto `std.hermetic_replay` `PublishedMockCase` rows, keyed by the same `OperationRef`.

The interface node never references the sibling. The sibling references the interface through `OperationRef`. That direction is the acyclic one: an interface is complete without any realization, and a realization is meaningless without its interface.

**What the full route does with the sibling.** Resolution resolves its references, which is what puts them in the census. Infer types the projection expressions against the declared outputs. Eval and emission select a binding through the existing `HandlerBinding`. A stage that cannot yet consume the sibling **refuses at the sibling under its own stage cause**, so the old blanket `service_realization_unreachable` becomes a set of per-stage located causes. Once no member is set aside, `ServiceSetAside` and both set-aside reasons are deleted.

### `uses`: no carrier (ruled: follow D13), with a prerequisite

A v2 carrier for the authored row would be a second authority for `DependencyDemand`, the fact D13 rules DERIVED for every transparent body. **But D13's derivation is not yet complete for Network.** `gunbc.rung_drop` `network_requirement_unrepresented_after_uses_cut` records that derived demand (`ItemInfo.service_names`) names the services a Network effect reaches and never Network itself, and that earlier deletions of `uses net: Network` rows were a silent rung loss. The D13 follow-up audit found that nearly every live row binds Network. So a Network row is **not yet a restatement**, and deleting it now would repeat that loss.

Resolution of this half:

- **The refusal stays.** `body_lower_fn_uses_refusal_optional` keeps refusing at the clause. Until the prerequisite lands, the authored clause is the only carrier of a Network requirement, so a fn carrying one stays out of the census rather than lowering without its requirement.
- **Prerequisite (the rung drop's restoration trigger):** D13's `DependencyDemand` carrier derives each resource requirement keyed on the resource declaration's identity (`std.resources.Network`), so that the derived demand of every function authoring `uses net: Network` names Network.
- **Then** the restatement rows are retired under D13's criterion, measured by resolution, not grep.
- What could survive is a named dependency binder (a distinct subject), an opaque contract (no body) or a policy envelope. Each moves to its own carrier when one first appears. The live corpus has none, so none is designed here.

## The smallest lowering change, member by member

| Member | Lowers into | Refuses (located) when |
| --- | --- | --- |
| `readonly` / `idempotent` | `^arrow_effect_claims_edge` on the operation's Arrow | repeated on one operation |
| `hermetic` | `^arrow_execution_mode_claim_edge` on the operation's Arrow | repeated on one operation |
| io `= default` | the input field's binder default (2b) | the value reader refuses the expression |
| io `from "key"` | projection row in `^service_realization_binding` | the key is not a string literal |
| `transport` / `config` | transport config record in the sibling | the transport kind has no config record |
| `exit` / `response` | output projection in the sibling | the value reader refuses the expression |
| `mock_response` | `PublishedMockCase` row in the sibling | the case has no operation it can key on |
| any other member | — | `body_lowering_reason_service_member_unread` (unchanged) |
| `uses` entry | — (no carrier) | always: `body_lowering_reason_uses_clause_unmodeled` (unchanged) |

## Rollout

| PR | Content | Unblocks | Behaviour change |
| --- | --- | --- | --- |
| 1 (this) | this document; `uses_clause_has_no_carrier` trigger amended to D13; DESIGN CLI-section sentence amended (separate hunk) | — | none |
| 2a | Arrow contract edges (`^arrow_effect_claims_edge`, `^arrow_execution_mode_claim_edge`) and their conformance in `v2.std.node`. **DESIGN §3c declared frontier** (see below) | — | substrate only |
| 2b | the one binder representation with an optional default: a replacement migration of every binder producer and reader, measured first | — | yes |
| 2c | lowering of modifiers and io defaults onto 2a and 2b; discriminating red per refusal row above | the 3 interface-only services | yes |
| 3 | `^service_realization_binding` sibling lowering of transport / config / exit / response / mock_response / `from`; delete `ServiceSetAside` and both set-aside reasons; per-stage located causes for any stage that cannot yet consume the sibling | the 111 realization-bearing services, up to the next stage's refusal | yes |
| 4 | flip `service_interface_member_has_no_carrier` and the cause-ownership rows; re-measure the census on the CI population | — | ledger only |
| `uses` (separate lane) | first the resource-keyed `DependencyDemand` carrier (the rung drop's restoration trigger); only then retire restatement rows by D13's criterion, measured by resolution | up to 30 modules, and none before the carrier lands | carrier, then source rows deleted |

No model lands in PR 1.

**2a is a DESIGN §3c declared frontier, not dangling.** In 2a, its consumers by execution are the conformance wall itself (`v2.std.node` `arrow_signature_edges_conform`, reached by every `well_formed` check of an Arrow) and the discriminating test that exercises it. Its *production* consumer is 2c: the operation-modifier lowering in `v2.compiler.body_lowering_fold` `body_lower_operation` emits the two edges, and the resolve, infer and translate readers gain their metadata arms in the same change. **Transition trigger:** 2c lands. If 2c is abandoned, the two edges and their conformance are deleted, not left standing. 2a lands ahead of 2c because the ruling asked for the substrate change to be reviewable on its own. That separation is the stated reason for the frontier. A model with no consumer in its own change is dangling by DESIGN §3c.

**Caveat, stated so it is not read as a promise.** No service has ever reached resolve or infer on the full route. PR 3 will surface new downstream refusals, located at the sibling or the operation under the consuming stage's cause. The count of services that reach the census is therefore re-measured on the CI population after PR 3, not inferred from the grep above.
