# Structured values at the `std.integer` `Int` formal

Probe fixtures for specimen seven of `gunbc.recurring_failure_mode`
`declared_type_wall_keyed_on_the_value_being_a_kernel`. Each module is one fn that calls
`std.integer` `int_to_decimal_string(value:)` with one argument shape:

| module | argument | expected |
|---|---|---|
| `gunbc.dcs_record_red` | a record parameter | refused |
| `gunbc.dcs_variant_red` | a local coproduct parameter | refused |
| `gunbc.dcs_variant_literal_red` | a nullary local variant literal | refused |
| `gunbc.dcs_bool_control` | a `std.types` `Bool` parameter | refused (seam control) |
| `gunbc.dcs_int_positive` | an `Int` parameter | accepted |
| `gunbc.dcs_int_literal_positive` | the literal `7` | accepted |
| `gunbc.dcs_int_arm_positive` | `NonNegative { n: n }` | accepted |

These are a measurement corpus, outside every gate on purpose: the reds are meant to refuse.

## Seed route

With a seed built from the revision under test, per probe:

    gunbc compile --source-root dag --source-root src/v2 \
      --source-root docs/measurements/direct_call_structured_value_at_int/dag \
      --entry docs/measurements/direct_call_structured_value_at_int/dag/gunbc/<probe>.dag \
      --output-dir <tmp> --target dag

## Native v2 route

The native driver binds only the layout roots (`gunbc.source_layout`), so a third root refuses
`native_lane_source_roots_refused`. Copy the probes into `dag/gunbc/` in a scratch checkout, take the
driver binary `gunbc test //gunbc/instruments:v2-native-census-scoped` emits (it is written under
that run's private probe root and removed when the run ends), and per probe:

    <emitted-driver> census-scoped //gunbc/<probe>:all dag src/v2

Read the probe's `cause_group` `fatal_reason` (refused) or its `[native-census-infer]` `outcome`
(`inferred` = accepted). Both routes and a large runner fit one `ctrl-build --remote` dispatch.
