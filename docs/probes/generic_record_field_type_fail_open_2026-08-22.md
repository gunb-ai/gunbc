# A record literal of a generic type admits the wrong field type silently, at six seams

**Subject:** the v1 frontend (`gunbc compile`), source→`.dag`-acceptance path.
**Ref:** built from `967b5bc1b9` (branch `session/calm-heron-887`), release `gunbc`,
built locally (`RUSTC_WRAPPER= CTRL_BUILD_MODE=local cargo build --release -p v1-compiler --bin gunbc`
— sccache fails the `libc` build script).
**Producer:** single-module probe roots, each written from scratch, one module per arm
(a blocking error truncates the diagnostic set, so arms must not share a module).

## The class

DESIGN §4b names "values inhabit declared types" as the **ordinary compiler floor**, where a
failure is a below-baseline safety regression. A record literal of a type **declared with type
parameters** does not hold that floor. The wrong program is not merely un-refused — it is
**emitted**.

This is **below floor, not a rung on the ladder** (§4b), and it is the residue the
`declared_type_conformance_diags` note in `v1.compiler.infer` already named as its own lane:
*"generic record instantiation, and post-substitution generic field access."*

## Measured, with a paired nonzero on every zero

The instrument's silence had to be shown to be a reading rather than an instrument failure, so
**every fail-open arm carries a matched non-generic twin at the same seam**, in the same run, on
the same binary. Preamble `type Box<T> { v: T }` / `type Plain { v: Int }`; the offending value is
`"s"` against a declared `Int` in every row.

| seam | generic | non-generic twin |
|---|---|---|
| fn return | `fn f() -> Box<Int> { Box { v: "s" } }` → **RC=0, emits** | `Plain` → **RC=1** |
| let annotation | `let x: Box<Int> = ...` → **RC=0, emits** | `Plain` → **RC=1** |
| record field | `Holder { b: Box { v: "s" } }` → **RC=0, emits** | `Plain` → **RC=1** |
| list element | `[Box { v: "s" }]` → **RC=0, emits** | `Plain` → **RC=1** |
| call argument | `takes(b: Box { v: "s" })` → **RC=0, emits** | `Plain` → **RC=1** |
| module-scope `data` | `data d: Box<Int> = ...` → **RC=0, emits** | `Plain` → **RC=1** |

Every twin refuses with a located `type mismatch: expected 'Primitive(Int)', got
'Primitive(String)'`. A third control, `ret_gen_undecl` — an undeclared name inside the generic
literal — returns **RC=1**, so the generic module is genuinely compiled and its body judged.

The twin design does more than prove the instrument is live: it rules out *"that seam is simply
unchecked for any type"*, which a bare perturbation would have left open.

## Two discriminators that moved the diagnosis twice

**It is not about the type-parameter reference.** A generic declaration whose parameter is
**unused** and whose field is a plain kernel type still fails open:

```
type BoxI<T> { v: Int }     fn f() -> BoxI<Int> { BoxI { v: "s" } }   -> RC=0, emits
type Mix<T>  { k: Int  t: T }  wrong value in the `k: Int` field       -> RC=0, emits
```

So the trigger is that the declaration **carries type parameters at all**, not that a field
*mentions* one.

**The instantiated path is strictly worse than the fallback it preempts.** Force the instantiation
to bail out by supplying the wrong arity, and the field judgment on the *same declaration* comes
back:

```
arity_bail_bad  type BoxI<T> { v: Int }  fn f() -> BoxI<Int, Int> { BoxI { v: "s" } }
  RC=1  type BoxI expects 1 type arguments, got 2
  RC=1  type mismatch: expected 'Primitive(Int)', got 'Primitive(String)'   <-- FIRES
arity_fire_bad  type BoxI<T> { v: Int }  fn f() -> BoxI<Int>      { BoxI { v: "s" } }
  RC=0  compiled, 0 diagnostics                                             <-- preempted, silent
arity_mono_bad  type BoxI    { v: Int }  fn f() -> BoxI           { BoxI { v: "s" } }
  RC=1  type mismatch: expected 'Primitive(Int)', got 'Primitive(String)'
```

`record_lit_instantiated_fields` does not merely fail to *add* an expectation — it **preempts a
working one**.

## The mechanism, located

Instrumentation was added to the executing stage0 mirror, measured, and removed. On
`arity_fire_bad` — a generic declaration whose field is the kernel type `Int`:

```
INST field="v" authored_fte="" authored_shape=Primitive() substituted_shape=Primitive()
FT   tn=BoxI fi=v expected_shape=Primitive()      got_shape=Primitive(String)
```

against the non-generic twin:

```
FT   tn=BoxI fi=v expected_shape=Primitive(Int)   got_shape=Primitive(String)
```

`authored_fte=""` is measured **before** substitution. **Substitution is innocent** — it faithfully
returns the nameless node it was given. The declaration reached by the instantiated path already
carries field type nodes stripped of their identity, while the declaration reached by the fallback
path, *for the same declaration in the same module*, carries named ones.

The two paths differ only in how they reach the declaration:

- instantiated path — `lookup_type_for(env, exp)`, keyed on the **expected node's `ident`**
- fallback path — `lookup_type_by_name(env, tn)`, keyed on the **name**

Both read `binding.resolved`, so a single binding cannot explain the divergence; the ident-keyed
route is reaching a different, identity-stripped node. `lookup_binding(env, ident)` does
`map_get(env.bindings, ident)` and, on a miss, feeds the same integer to
`intern_str(table, id)` and retries **by name** — so an occurrence id and an intern id are being
read out of one integer slot. That address-space question is where the next attempt should start.

Once the expectation is nameless, `kernel_value_declared_type_mismatch` returns `false` on
`formal_name == ""`. **That arm is a second, independent fail-open and deserves its own repair
even after the upstream one lands**: an upstream fix removes today's population, not the arm that
converts a nameless expectation into silence for whatever produces one next.

## Four repairs attempted and measured as NOT sufficient

Recorded because they are the cost of the next attempt, not a reason to stop looking. Each was
built and run against the full arm table; **every arm was byte-identical** each time.

1. `record_lit_instantiated_fields` reading the type argument with `child_type_node` instead of
   `resolved_type`. Closes a real second-order fail-open — `resolved_type`'s `Absent` arm is
   `error_type`, itself nameless, and a type-argument node carries no `inferred` — but the
   substitution is never applied, so nothing moves.
2. `resolve_item_types` carrying `tr.resolved` onto the field's `inferred`.
3. `resolve_item_types` installing `tr.resolved` as the field's **type_expr child** (`children[0]`),
   which is what `record_lit_instantiated_fields` actually reads. Correct slot, wrong function.
4. *(implied by 2 and 3)* anything else in `resolve_item_types`.

**One cause for all of them, established by reading the construction site rather than by another
build:** `ResolvedModule.module` is `module_occurrence_input_node(input)` — the raw
parsed/normalized node. `build_type_env`'s `local_bindings` folds **those** items through
`local_binding_for_item`, which copies `children` verbatim. `resolve_item_types` runs later, inside
`analyze_item`, and its output feeds `resolved_item` — **never the binding**. So `ResolvedModule`
means *import*-resolved, not *type*-resolved, and three repair cycles were spent editing a
function whose output the binding never sees.

## Found on the way, worth its own row

`v1.compiler.resolve` `resolve_field` does the right thing — it installs `type_resolved` into the
field via `make_field_node` — and has **zero callers**, while `resolve_item_types` hand-rolls a
lossy version beside it that discards `tr.resolved` except for its `properties`. Its sibling
`resolve_field_init` *is* wired, which makes this an **incomplete migration** rather than
abandoned scaffolding. It is emitted into the stage0 mirror, so the seed pays for it today.

Two authorities for "resolve a field", the correct one dead and the lossy one live, is a §3
violation at the grain §3 forbids. It is **not** established as this lane's repair — the
discriminators above moved the cause off it — but a cleanup sweep would be entitled to delete it
as a §5 dead scaffold, and that would remove a correct implementation while leaving the lossy one
as the only authority. Claim staked on gunbc#8901.

## Rung

Source→acceptance, record-literal field type on a type declared with parameters: **below floor —
not a rung** (§4b). The non-generic position is at least mechanically preventable (executing wall,
discriminating RED plus positive control, both above). The class rung is the minimum across paths,
so citing the non-generic path would be inflation. Next-rung trigger: the ident-versus-intern
address-space question above.
