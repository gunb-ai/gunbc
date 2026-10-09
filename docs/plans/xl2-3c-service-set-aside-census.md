# XL-2 PR3c — `ServiceSetAside` consumer census

Status: enumeration only. Does not delete. Owner: XL-2 3c. Measured on `session/bold-carp-102` against current main. Authority for the cut: [service-interface-and-uses-carriers.md](service-interface-and-uses-carriers.md) PR 3c; DESIGN §3 delete-first (enumerate out-of-gate consumers by name before deleting a root outside the required gate).

Gate: `v2.workflow.required_floor` `required_gate_prefixes` (textual `starts_with` on the full `module` path) plus `required_gate_authored_modules` (named-module subtree). Prefixes do not match a `v2.`-prefixed name.

Cite symbols, not positions.

## Reasons as they stand

- `body_lowering_reason_realization_member_set_aside` — kind `RealizationMemberSetAside`
- `body_lowering_reason_interface_member_unmodeled` — kind `InterfaceMemberUnmodeled`
- Fatal on the full route: `body_lowering_reason_service_realization_unreachable` (owned row in `v2.workflow.compile_door_cause_ownership` `known_frontier_causes`, lane `SharedSelfHostCriticalPath`). The two set-aside reasons are pending advisories, not fatal last elements, and have no ownership rows.

## 1–2. Producers and readers

### Types and helpers — `v2.compiler.body_lowering_fold` — in-gate by compile closure

| Symbol | Role |
| --- | --- |
| `ServiceSetAsideKind` | `RealizationMemberSetAside` \| `InterfaceMemberUnmodeled` |
| `ServiceSetAside` | `{ kind, member }` |
| `service_set_aside_reason` | maps kind to the two reason symbols |
| `service_set_aside_diagnostics` | folds the list into located diagnostics |
| `admit_set_aside` | parameter of `body_lower_service_decl` and `body_lower_service_held` |
| `body_lower_service_held` | census admits interface + sibling with advisories; full route refuses the fatal |
| `body_lower_operation_acc_set_aside` | appends one row |
| `body_lower_set_aside_of` | maps a member list to rows |

Callers of `admit_set_aside`:

- `v2.compiler.normalize` `normalize_census_node` — `true` — in-gate closure
- `body_lower_production_emitted` — `false` — same module, in-gate closure

No `*.rs` consumers.

### Who mints `ServiceSetAside` rows (`v2.compiler.body_lowering_fold`)

- `body_lower_operation_transport` — transport **kind** enters the sibling; the member is also `RealizationMemberSetAside`
- `body_lower_operation_status_arms` — exit / response / mock_response lower into the sibling **and** append `RealizationMemberSetAside`
- `body_lower_service_member` on service-level `transport` — set-aside only
- `body_lower_realization_with_rows` — io `from` keys already become projection rows; the field items are still tagged `InterfaceMemberUnmodeled`
- Accumulators: `BodyLowerIoBlock.set_aside`, `BodyLowerOperationAcc.set_aside`, `BodyLowerRealizationAcc.members`, `BodyLowerServiceAcc.set_aside`

`body_lower_service_held` refuses the fatal when the list is nonempty, or when a realization leaf exists with an empty list (config-only). That hold is the 3c subject.

`body_lower_resource_with_capability` reuses the same list and the same fatal for a **capability** io tail (not a service).

### Second producer of the interface reason (not via the type)

`body_lower_field_decl_block_payload` refuses a **record-field** wire-key tail as `body_lowering_reason_interface_member_unmodeled`. 3c cannot delete that reason while this arm remains.

### In-gate claim — `v2.test.claim.namespace_xl0.reference_conservation`

Authored on `required_gate_authored_modules`. Fixture: planted `service rc.Svc` with `transport shell`.

- `the_refused_normalization_fixture_refuses_for_the_service_realization_reason_holds`
- `a_refused_normalization_carries_its_fatal_reason_located_in_text_holds`
- `a_refused_normalization_prints_its_reason_and_locus_on_the_summary_line_holds`

### Out-of-gate executing consumers (will not fail the required floor)

| Module | Symbols |
| --- | --- |
| `v2.test.parse.g0_service_decl_parse_probe` | `a_service_with_a_transport_is_refused_at_the_normalized_tree_door_holds`, `config_only_service_is_refused_for_its_set_aside_realization_holds` |
| `v2.test.claim.normalize.service_declaration_lowering` | `census_counts_realization_and_unmodeled_interface_apart_holds`, `full_door_refuses_a_service_with_a_set_aside_realization_holds` |
| `v2.test.claim.normalize.service_realization_lowering` | `a_config_only_realization_is_held_off_the_full_route_holds` |
| `v2.test.claim.normalize.resource_declaration_lowering` | `a_wire_key_tail_refuses_as_a_service_set_aside_RED` |
| `v2.test.claim.parse.record_field_tail` | `a_record_field_with_a_wire_key_refuses_at_its_tail_holds` |
| `v2.test.claim.parse.default_value` | `a_record_wire_key_keeps_its_interface_reason_holds` |
| `v2.test.compile_door_ledger_ownership` | does not name this cause; uniqueness over `known_frontier_causes` |

Prose (not executing): `gunbc.recurring_failure_mode` `service_interface_member_has_no_carrier`, `gunbc.compiler_frontend_program_status`, this plan's parent document.

## 3. Stages that assume absence of the sibling

Today the sibling never reaches the full route (`admit_set_aside: false`). After delete-first it is a module child under `realization_root_label`. Do not inherit `body_lowering_reason_service_realization_unreachable` for later stages.

| Stage | Now | 3c |
| --- | --- | --- |
| resolve | No `realization_root` reader. Design: resolve **should** walk sibling refs. `v2.compiler.symbol_index_fill` `symbol_index_fill_containment_edge` already skips the unspellable root. No new refuse-cause if resolve can walk values/types already in the leaf. |
| infer | No sibling consumer. Design: infer refuses at the sibling under its **own** owned cause (exit / response / mock not typed). Needs a `compile_door_cause_ownership` row. |
| emit / translate | No consumer. Emitting a service as if it had no handler is what the hold prevents. Needs an owned emit cause. |
| eval | No consumer. A call would run without transport / projection. Needs an owned eval cause. |

Also a **resource** cause: `body_lower_resource_with_capability` inherits the service blanket.

## 4. What goes red if the type is deleted

- Deleting `ServiceSetAside` / helpers / `admit_set_aside` without rewriting the producers fails compile of `v2.compiler.body_lowering_fold` (loud, in-gate closure).
- Deleting the fatal symbol without rewriting the three `reference_conservation` claims: those go red **on the required floor**.
- Deleting `body_lowering_reason_interface_member_unmodeled` while record-field wire keys still use it: out-of-gate parse claims go red; the record arm still produces it.
- Deleting `body_lowering_reason_realization_member_set_aside`: out-of-gate `service_declaration_lowering` claims go red; no in-gate claim names that symbol.

3b already lowers transport kind, config, arms, and `from` rows into the sibling. The list is a **second** hold on members that also lowered. Config-only is held by leaf presence alone. 3c cuts that second hold and the blanket fatal; it is not the first time the sibling exists at census grade.
