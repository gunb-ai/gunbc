# The second axis, enumerated from the inference side

The grammar axis (14 `parse_type_expr` sites) is not the only cut. `resolve_expr_types`' `ExprVar`
arm returns the node unchanged with an empty diagnostic list, so RESOLVE refuses an undefined name
at no position at all — inference is what refuses one. That makes **declaration-site expression
subtrees that inference never walks** a second class, and it is not a subset of the fourteen.

Enumerating it from the RESOLVE side does not work: `resolve_expr_types(` has ~40 call sites in
`04_resolve` and most are the function's own recursive descent (child, base, val, guard, arm_body),
so separating entry points from recursion is fiddly rather than decidable. From the INFERENCE side
the set is closed and readable: which declaration node kinds does `04_infer` descend into, and for
each expression-bearing field, does it WALK the expression or only TEST ITS PRESENCE.

## The denominator

| declaration node | expression-bearing field | what inference does | class |
|---|---|---|---|
| `fn` item | `body` | `infer_expr` with the declared return as `expected` | walked, typed |
| `data` item | `body` (initializer) | `infer_expr` with the annotation as `expected` | walked, typed |
| transport | `properties` (incl. body / query / stdin) | `infer_property_values` → `infer_expr(expected: none)` | **walked, UNTYPED** |
| `fn` item | `properties`, name `svc_auth_source` | `infer_auth_source_properties` → `infer_expr(expected: none)` | **walked, UNTYPED** |
| param | `default_value` | presence test only (one site, the call-shape required test) | **NEVER WALKED** |
| field decl | `default_value` | not referenced anywhere in `04_infer` | **NEVER WALKED** |
| `fn` item | `properties`, any other name | `{ prop: p, diagnostics: [] }` — passed through | **NEVER WALKED** |
| `uses` resource | resource node `properties` (config args) | scope extended with the resource TYPE; the arg expressions untouched | **NEVER WALKED** |
| service `exit` entry | status-pattern expression | `04_infer` has no `exit`/status-pattern arm | **NEVER WALKED** |
| transport | `children` | `infer_transport_node` copies `children: t.children` unchanged | **NEVER WALKED** |

Three classes, not two, and the middle one is worth naming: **walked, UNTYPED** — inference
descends with `expected: none`, so an undefined name DOES refuse there while inhabitance has
nothing to compare against. That is a different defect from the never-walked rows and needs a
different repair (thread the declared type, not add a pass).

## Confirmed by execution vs flagged by structure

- **Confirmed members** (three-arm run, in `measured.md`): parameter default, field default. Both
  accept an undefined name; the in-body `let x = nosuchname_zzz` control refuses in the same run.
- **Flagged by structure, not yet fixtured**: non-`svc_auth_source` item properties, `uses` config
  args, service exit-entry status patterns, transport children. Each is a presence-test-or-passthrough
  row above; none has been executed, and none is counted as a member here.

The signature that unites them is the one the two confirmed members share: **a declaration node
whose expression child is read by resolve and reached by inference only through a presence test or
a passthrough.** `param_node_default_value(n: param) != none` and `titem.transport != none` are the
same tell.
