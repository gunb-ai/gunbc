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
| the resources a fn's body demands | `v1` `04_items` `ResourceRequirement` (`binding_name`, `resource`), consumed by the Rust emitter's call-site binding | **D13** (`gunbc.plans.demand_engine_program`): for a transparent body, demand is DERIVED (`DependencyDemand`, produced once by resolution). Authored restatement rows are deleted (USES-0, `docs/plans/uses-occurrence-census.md`). Binder, opaque-contract and policy rows move to their own carriers |

## One carrier per fact

### Interface: lowered onto the existing service node

The operation edge `body_lower_operation` builds today gains two carriers, each modelled once:

1. **Operation modifier.** The closed set `OperationModifier = Idempotent | Readonly | Hermetic` already exists in v1 `00_core`. It moves to `std.effects`, its semantic home: two of its three arms are claims checked there by `check_modifier_vs_derivation`, and v1 then imports it from std. This is a move, not a second declaration. Each modifier lowers as a childless named edge `^operation_modifier_<arm>` on the operation node. A repeated modifier refuses located (`body_lowering_reason_service_operation_modifier_repeated`). Whether the claim *agrees* with the derived effect shape stays with `check_modifier_vs_derivation`, which reads these edges once resolution establishes the shape. The lowering carries the claim. It does not judge it.
2. **Field default.** An io field's `= v` lowers as a `^field_default` edge whose target is the value expression, through the same value reader every expression position uses. A default the reader cannot lower refuses at the default, under the reader's own cause.

After step 1, `InterfaceMemberUnmodeled` has no producer for modifiers or defaults. Wire keys move to the realization carrier (below), so that set-aside kind is deleted, not left empty.

### Realization: a sibling node, never inside the interface

Per DESIGN §3, transport is one of N handlers bound to the interface shape, and "the dispatch that selects a realization is itself realization". So the realization members lower into **one sibling node per service**, `^service_realization_binding`, emitted beside the service node in the same module:

- It is keyed per operation by the operation's resolved identity, the shape `v2.std.operation_argv` `OperationRef` (`path`, `service`, `operation`) that `v2.std.operation_realization` `OperationBinding` and the grant's `HandlerBinding` selection already key on. No new identity is minted.
- `transport` / `config` lower onto the transport's existing config record (`extdeps.transports.shell` `ShellTransportConfig`, `extdeps.transports.rest` `RestTransportConfig`, `extdeps.transports.file`). A transport kind with no config record refuses located (`body_lowering_reason_service_transport_unmodeled`). It is never coerced to a neighbour.
- `exit` / `response` lower as the operation's output projection: expressions over the transport's observation fields, lowered through the ordinary value reader so their references reach the census.
- `from "key"` lowers as one row of that projection (declared output field ← wire key). That is where it belongs: it is meaningful only relative to the transport whose observation it decodes.
- `mock_response` lowers onto `std.hermetic_replay` `PublishedMockCase` rows, keyed by the same `OperationRef`.

The interface node never references the sibling. The sibling references the interface through `OperationRef`. That direction is the acyclic one: an interface is complete without any realization, and a realization is meaningless without its interface.

**What the full route does with the sibling.** Resolution resolves its references, which is what puts them in the census. Infer types the projection expressions against the declared outputs. Eval and emission select a binding through the existing `HandlerBinding`. A stage that cannot yet consume the sibling **refuses at the sibling under its own stage cause**, so the old blanket `service_realization_unreachable` becomes a set of per-stage located causes. Once no member is set aside, `ServiceSetAside` and both set-aside reasons are deleted.

### `uses`: no carrier (ruled: follow D13)

A v2 carrier for the authored row would be a second authority for `DependencyDemand`, the fact D13 rules DERIVED for every transparent body. All 97 measured rows sit in transparent bodies and none reads its alias, so each is a USES-0 restatement. Resolution of this half:

- The refusal stays. `body_lower_fn_uses_refusal_optional` keeps refusing at the clause, which also walls out new restatement arrivals. EFFECTS-1's open slice is the "nothing walls new rows out yet" gap that `gunbc.plans.demand_restatement_follow_up` records.
- The population is unblocked by **retiring restatement rows** under D13's existing criterion. Rows `demand_restatement_follow_up` already retains keep their stated triggers.
- What could survive is a named dependency binder (a distinct subject), an opaque contract (no body) or a policy envelope. Each moves to its own carrier when one first appears. The live corpus has none, so none is designed here.

## The smallest lowering change, member by member

| Member | Lowers into | Refuses (located) when |
| --- | --- | --- |
| `readonly` / `idempotent` / `hermetic` | `^operation_modifier_*` edge on the operation | repeated on one operation |
| io `= default` | `^field_default` edge | the value reader refuses the expression |
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
| 2 | `OperationModifier` moved to `std.effects` (v1 imports it, stage0 mirrors regenerated); interface lowering of modifiers and defaults; discriminating red per refusal row above | the 3 interface-only services | yes |
| 3 | `^service_realization_binding` sibling lowering of transport / config / exit / response / mock_response / `from`; delete `ServiceSetAside` and both set-aside reasons; per-stage located causes for any stage that cannot yet consume the sibling | the 111 realization-bearing services, up to the next stage's refusal | yes |
| 4 | flip `service_interface_member_has_no_carrier` and the cause-ownership rows; re-measure the census on the CI population | — | ledger only |
| `uses` (parallel, separate lane) | retire restatement rows by D13's criterion, module by module, measured by resolution | up to 30 modules | source rows deleted |

The `OperationModifier` move lands in PR 2, with its first v2 consumer, not in PR 1. A model with no consumer in its own change is dangling by DESIGN §3c.

**Caveat, stated so it is not read as a promise.** No service has ever reached resolve or infer on the full route. PR 3 will surface new downstream refusals, located at the sibling or the operation under the consuming stage's cause. The count of services that reach the census is therefore re-measured on the CI population after PR 3, not inferred from the grep above.
