# The 14 grammar type positions, enumerated from the parser itself

Enumeration instrument, run against the tree at `abf7194e2b2`:

    grep -n "parse_type_expr" src/v1/02_parse.dag

`v1.compiler.parse` `parse_type_expr` is the single production that reads a type expression
from source. Every declared type in a `.dag` program is read by exactly one of its call sites,
so its call-site set IS the grammar's type-position enumeration — not a list someone
remembered to write down. Fourteen call sites, plus the definition itself.

| # | line | enclosing production | grammar position | source value flows in? |
|---|---|---|---|---|
| 1 | 970 | `parse_type_angle_arg` | type argument inside `X<...>` | yes, indirectly (element / value / instantiation) |
| 2 | 1967 | `parse_type_body_after_eq` | `type X = <T>` alias target | no — type-only |
| 3 | 2116 | `parse_positional_variant_type_fields` | positional variant payload `\| V(T)` | yes — at `V(v)` construction |
| 4 | 2245 | `parse_callable_type_expr` | callable type's return | yes — lambda body |
| 5 | 2253 | `parse_callable_param_types` | callable type's parameter | no — bound at application, not authored |
| 6 | 2388 | `parse_field` | record field declaration | yes — record-literal field |
| 7 | 2683 | `parse_uses_entry` | `uses` resource type | no — host boundary, no authored value |
| 8 | 2737 | `parse_optional_inferred` | function declared return | yes — body / early `return` |
| 9 | 3337 | `parse_exit_entries_acc` | service exit-status payload type | no — host boundary |
| 10 | 3437 | `parse_response_entries_acc` | service response type | no — host boundary |
| 11 | 3742 | `parse_data_after_kw` | `data x: T = v` annotation | yes — initializer |
| 12 | 3810 | `parse_param` | function parameter type | yes — call argument, and parameter default value |
| 13 | 4325 | `try_postfix` | `expr as T` cast target | yes — separately classified: a cast is an authored assertion |
| 14 | 4960 | `parse_let` | `let x: T = v` annotation | yes — bound expression |

Positions 2, 5, 7, 9, 10 are TYPE-ONLY: no source value can stand at them, so they carry a
disposition and no value obligation. Fabricating an obligation there would be a fabricated
refusal, which DESIGN section 5 forbids exactly as it forbids a fabricated success.
