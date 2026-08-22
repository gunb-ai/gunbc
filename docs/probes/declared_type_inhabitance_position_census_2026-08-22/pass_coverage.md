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

## The denominator, and how it was derived

**Ten (declaration node kind, expression-bearing field) pairs.** The enumeration is not a list of
places anyone happened to look: it is taken from the DECLARATION CONSTRUCTORS in `v1.core` — the
`make_*_node` functions that can store an expression on a declaration — crossed with the fields
`04_resolve` actually populates. The constructors that carry an expression are
`make_param_node` / `make_resolved_param_node` (`default_value`), `make_field_node`
(`default_value`), `make_resource_use_node` (the resource node's `properties`, the config args),
`make_transport_node` (`properties`, `children`, `body`), and the item node's own `body`,
`children` and `properties`. So the denominator is those constructor slots, and its own denominator
is the constructor set — which is why a row can be added to the table only by adding a constructor
slot, not by remembering another place.

Counted against it: **6 never-walked of 10**, 2 walked-typed, 2 walked-untyped (both currently
UNCONFIRMED — see the retraction below). Of the 6, **2 are confirmed by execution** (parameter
default, field default) and 4 are flagged.

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

  **DISCRIMINATED, AND BOTH CANDIDATE EXPLANATIONS WERE WRONG.** The suspicion was that no
  expression inside a `service` declaration is inferred at all, which would have made the three
  ACCEPTs one fact wearing three spellings. Measured, one run:

  | fixture | verdict |
  |---|---|
  | `transport shell { argv: nosuchname_zzz }` — undefined name AS the property value | **REFUSED** |
  | `transport shell { argv: ["echo", nosuchname_zzz] }` — same name, one hop into its LIST | **ACCEPTED** |
  | ordinary service, no undefined name | ACCEPTED |
  | `let x = nosuchname_zzz` in a function body | REFUSED, `undefined variable 'nosuchname_zzz'` |
  | ordinary function body | ACCEPTED |

  So the service subtree IS inferred: the function-body controls behave, and one of the two service
  arms refuses.

  **AND THE STRUCTURE SAYS I NAMED THE POSITION WRONG.** `v1.core` `shell_transport_node` stores
  `argv` as the transport node's **`children`**, not as a property — properties carry `env` and
  `stdin`. So `argv: ["echo", nosuchname_zzz]` being accepted is NOT "a list element inside a walked
  property"; it is the table's own **transport `children` — NEVER WALKED** row, confirmed by
  execution. `infer_transport_node` copies `children: t.children` unchanged, and every
  `transport shell { argv: [...] }` in the corpus sits behind that copy.

  Two things follow, and the second is why the intermediate claim is recorded rather than quietly
  replaced. The flagged row is now a CONFIRMED member on its own terms. And the reason it looked
  like a new position was that the fixture was written from the SURFACE spelling (`argv:` looks like
  a field init) instead of from the node the parser builds — the same mistake as reading a refusal
  count without asking which layer refused.

  **THE REFUSAL WAS A PARSE ERROR, SO THE DISCRIMINATOR DID NOT DISCRIMINATE — RETRACTED.** Measured:
  `argv: nosuchname_zzz` refuses as `module index refused: 1 unparseable .dag source(s)`, which is
  the grammar rejecting a non-list argv and says nothing about inference. And the property-side arm
  written to settle it, `stdin: nosuchname_zzz` — `stdin` IS a transport property — is **ACCEPTED**,
  with an ordinary-service control accepted beside it.

  So the paragraph above claiming the service subtree is inferred rests on a refusal from the wrong
  layer, which is this census's own map-key lesson turned on its author. Retracted. Every service
  arm measured so far accepts: exit-entry status pattern, service-input field default, argv element,
  and now a transport property value.

  That is consistent with a single explanation — **no expression inside a `service` declaration is
  inferred at all** — which would make four apparent members ONE fact wearing four spellings, and
  would also contradict the table's `walked` row for transport properties. It is not yet
  established: the discriminating pair is a RESOLVE-level refusal inside a service (an unresolved
  type in `input`) against an INFER-level one (a type error in a property value), with the same
  type error in a function body as the positive control. Until that returns, all four service rows
  are FLAGGED, none is counted, and the `walked` classification for transport properties is
  UNCONFIRMED rather than confirmed.

- **Flagged by structure, not yet confirmed**: non-`svc_auth_source` item properties, `uses` config
  args, service exit-entry status patterns, transport children. None is counted as a member here.

The signature that unites them is the one the two confirmed members share: **a declaration node
whose expression child is read by resolve and reached by inference only through a presence test or
a passthrough.** `param_node_default_value(n: param) != none` and `titem.transport != none` are the
same tell.
