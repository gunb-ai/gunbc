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

**Do not grep the spelling.** `expected: none` appears at 25 of `04_infer`'s 47 `infer_expr` call
sites, and the count does not support the alarm: match scrutinee, `if` condition, method receiver,
binary operands and lambda values have NO declared type in context, so `expected: none` is the
correct call there and not a lost annotation. The property that puts a row in this class is that
**a declared type WAS available at the site and was not threaded** — which is decidable only from
the declaration-side read below, never from counting the argument. Someone grepping the spelling
gets 25 and a false population. (Measured by swift-badger-524, who went looking for this class to
be larger than six rows and reported the check coming back empty.)

## Confirmed by execution vs flagged by structure

- **Confirmed members** (three-arm run, in `measured.md`): parameter default, field default. Both
  accept an undefined name; the in-body `let x = nosuchname_zzz` control refuses in the same run.
- **A first confirmation attempt was UNINFORMATIVE and is recorded rather than dropped.** Four
  service fixtures (exit-entry status pattern, transport property value, service-input field
  default, plus a positive control) all REFUSED — including the control, which was authored to be
  ACCEPTED. The cause was the fixture, not the compiler: `unresolved import: module 'std.types' not
  found`, because the arms were compiled against their own source root with no dependency pool. A
  run whose positive control refuses cannot read its negative arms, so nothing in that run is
  evidence about any of the four rows. Re-run with `--source-root dag` beside the arm's own root.

- **Second confirmation run, and it DISAGREES with the table above at one row.** Re-run with
  `--source-root dag` beside the arm's own root, so the control is meaningful this time:

  | fixture | verdict |
  |---|---|
  | control: an ordinary service, no undefined name | ACCEPTED (the control works) |
  | `exit { nosuchname_zzz => String "…" }` | **ACCEPTED** |
  | service `input { extra: List<String> = nosuchname_zzz }` | **ACCEPTED** |
  | `transport shell { argv: ["echo", nosuchname_zzz] }` | **ACCEPTED** |

  The first two confirm rows the table predicts. **The third contradicts it**: the table classes
  transport `properties` as WALKED (`infer_property_values` → `infer_expr`), and a walked position
  refuses an undefined name. So either the classification is wrong or the fixture reaches a
  different position than intended — a list ELEMENT inside a property value is not the property
  value.

  **The three ACCEPTs may also not be three facts.** If no expression inside a `service`
  declaration is inferred at all, they are one fact wearing three spellings, and counting them as
  three members would inflate the class exactly as the map-key cell inflates a refusal count. That
  is the open question, and it is being discriminated (an undefined name as the property value
  DIRECTLY versus inside its list, against a function-body control known to refuse) rather than
  guessed. Until it resolves, these rows stay FLAGGED.

- **Flagged by structure, not yet confirmed**: non-`svc_auth_source` item properties, `uses` config
  args, service exit-entry status patterns, transport children. None is counted as a member here.

The signature that unites them is the one the two confirmed members share: **a declaration node
whose expression child is read by resolve and reached by inference only through a presence test or
a passthrough.** `param_node_default_value(n: param) != none` and `titem.transport != none` are the
same tell.
