# A generic record literal admits the wrong field type silently, at six seams

**Subject:** the v1 frontend (`gunbc compile`), source→`.dag`-acceptance path.
**Ref:** built from `967b5bc1b9` (branch `session/calm-heron-887`), release `gunbc`,
built locally (`RUSTC_WRAPPER= CTRL_BUILD_MODE=local cargo build --release -p v1-compiler --bin gunbc`).
**Producer:** eleven single-module probe roots, each written from scratch, one module per arm
(a blocking error truncates the diagnostic set, so arms must not share a module).

## The class

DESIGN §4b names "values inhabit declared types" as the **ordinary compiler floor**, where a
failure is a below-baseline safety regression. A record literal of a **generic** type does not
hold that floor: the field expectation derived from the instantiation is a nameless node, and the
judgment that would refuse fails open on it. The wrong program is not merely un-refused — it is
**emitted**.

This is **below floor, not a rung on the ladder** (§4b), and it is the residue the
`declared_type_conformance_diags` note in `v1.compiler.infer` already named as its own lane:
*"generic record instantiation, and post-substitution generic field access."*

## Measured, with discriminating controls

Every arm shares one preamble: `type Box<T> { v: T }` and `type Plain { v: Int }`.

| arm | source | RC | verdict |
|---|---|---|---|
| `ctl_pos` | `Plain { v: 1 }` | 0 | control: conforming non-generic compiles |
| `ctl_neg` | `Plain { v: "s" }` | 1 | control: **instrument reds** — `type mismatch: expected 'Primitive(Int)', got 'Primitive(String)'` |
| `ret_generic_pos` | `fn f() -> Box<Int> { Box { v: 1 } }` | 0 | control: conforming generic compiles |
| `ret_generic` | `fn f() -> Box<Int> { Box { v: "s" } }` | **0** | **fail-open, emits** |
| `let_generic` | `let x: Box<Int> = Box { v: "s" }` | **0** | **fail-open, emits** |
| `field_generic` | `Holder { b: Box { v: "s" } }`, `b: Box<Int>` | **0** | **fail-open, emits** |
| `list_generic` | `fn f() -> List<Box<Int>> { [Box { v: "s" }] }` | **0** | **fail-open, emits** |
| `arg_generic` | `takes(b: Box { v: "s" })`, `fn takes(b: Box<Int>)` | **0** | **fail-open, emits** |
| `data_generic` | `data d: Box<Int> = Box { v: "s" }` | **0** | **fail-open, emits** |
| `missing_plain` | `Plain { }` | 1 | field **presence** holds |
| `missing_generic` | `Box { }` | 1 | field **presence** holds for generics too |

Two facts the table settles that reasoning would not. **Six seams**, not one — every position that
knows an instantiation loses it, so this is a property of the shared machinery rather than of any
one caller. And the failure is confined to the field **type** axis: `missing_generic` refuses, so
field **completeness** is unaffected, which rules out "generic declarations are simply not
processed" as the explanation.

## The mechanism, located by execution

Instrumentation was added to the executing stage0 mirror (`v1_compiler_infer`
`record_lit_instantiated_fields`, `infer_record_lit_structural`), measured, and removed.

1. The instantiation **does** reach the literal. For `ret_generic`:
   `RLIF tn=Some("Box") exp_some=true exp_name=Some("Box") exp_children=1 decl_params=Some(1)` —
   name compatible, arity matched, substitution branch entered.
2. The substitution is keyed correctly: `RLIF4 substkeys=["T"]`.
3. The declaration's field type node for `T` **carries no name to key on**:
   `RLIF4 field="v" fte_authored="" fte_rawname="" identspan=false inferred=Other children=0 label=""`.
   `substitute_generics` looks up `type_node_label(n)`, which is `""` here, so `T` is never
   substituted.
4. So the expectation reaching the judgment is nameless:
   `FT tn=Box fi=v expected_shape=Primitive() got_shape=Primitive(String)`
   against the control `FT tn=Plain fi=v expected_shape=Primitive(Int) got_shape=Primitive(String)`.
5. `kernel_value_declared_type_mismatch` returns `false` the moment `formal_name == ""`. That
   arm is the fail-open, and it is reached with a **nameless** formal, not a wrong one.

A second, independent fail-open sits on the same path and is **not** the cause: the substitution
value is read with `resolved_type(n: arg)`, whose `Absent` arm is `error_type` — also a nameless
node — and a type-argument node carries no `inferred` (`arg_inferred=0`,
`subst_names=[("T", "")]`). So even had step 3 keyed, the value substituted in would have been
nameless too. Both must be closed; closing either alone changes nothing (measured, below).

## Two repairs attempted and measured as NOT sufficient

Recorded because they are the cost of the next attempt, not a reason to stop looking. Each was
built and run against the full arm table; **every arm was byte-identical to the table above**.

1. **`record_lit_instantiated_fields` reads the type argument with `child_type_node` instead of
   `resolved_type`.** Closes the `error_type`-as-substitution-value fail-open (step 5 above) and is
   the existing authority for "the type this node stands for", already used at the list-element
   seam in the same file. No arm moved, because step 3 means the substitution is never applied.
2. **`resolve_item_types` carries `tr.resolved` onto the field's `inferred`** in the record and
   variant arms, which today compute the resolution of each field's authored type — with the type
   parameters bound via `env_with_type_variable_bindings` — and then **discard `tr.resolved`
   except for its `properties`**. No arm moved, which means the declaration inference reads is not
   the one this arm produces.

The discard in (2) is real and worth its own look; it is simply not on the path that feeds
`lookup_type_for` here.

## What is still open — the next step, stated so it is not re-derived

Where the `T` reference loses its name. The declaration reaching inference already carries a
field type node that is nameless, span-less and `CompilerError`-inferred, so the loss is upstream
of `resolve_item_types`' record arm. The specific question: **which stage produces the
`TypeBinding` that `lookup_type_for` returns for a generic type, and does it store the
pre- or post-`resolve_item_types` node?** (`build_type_env` in `v1.compiler.infer` is the
producer to read first; note its kernel `Present` payload uses the designed
`TypeVariable { id: ... }` representation, which is exactly what a type-parameter reference in a
generic declaration should resolve to and does not.)

Until that is answered the repair is a modelling decision in a stage DESIGN names load-bearing,
not a targeted inference edit — which is why nothing is changed here.

## Rung

Source→acceptance, generic record field type: **below floor — not a rung** (§4b). The
non-generic position is at least mechanically preventable (executing wall, discriminating RED plus
positive control, both above). The class rung is the minimum across paths, so citing the
non-generic path would be inflation. Next-rung trigger: the open question above.
