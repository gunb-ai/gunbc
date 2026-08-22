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

## The class is THREE rows, plus four census rows of settled disposition

Two stacked defects, which is why every single-cause theory died in turn.

### Row (a) — concrete-typed field in a parameter-carrying declaration. CAUSE FOUND, repair measured and NOT landable.

`type BoxI<T> { v: Int }` — parameter declared but **unused**, field a plain kernel type — fails
open at `BoxI { v: "s" }` against `BoxI<Int>`. So the trigger is that the declaration carries type
parameters at all, not that a field mentions one.

**Cause, measured:** a field node carries its declared type in `sf.inferred`; `children[0]` is a
stripped placeholder (`inf=true`, `field_node_type_expr` authored name `""`).
`record_lit_instantiated_fields` substitutes into `field_node_type_expr(sf)` — the placeholder —
while every other consumer reads `sf.inferred` first. Two readings of one field, and the
instantiated path took the wrong one, so it **preempted a working expectation with a nameless
node**. Forcing the instantiation to bail out on a wrong arity brings the judgment back on the same
declaration, which is what proves the preemption:

```
arity_bail_bad  BoxI<Int, Int>  RC=1 arity error + RC=1 type mismatch  <-- judgment FIRES
arity_fire_bad  BoxI<Int>       RC=0 compiled, 0 diagnostics           <-- preempted, silent
arity_mono_bad  non-generic     RC=1 type mismatch
```

**Repair built:** extract `field_declared_type_node` as the single authority for "a field's declared
type node" (`sf.inferred` first, `field_node_type_expr` as fallback) and point both the instantiated
path and the consumption site at it. `kernelfield_gen_bad` flips **RC=0 → RC=1** with a located
`expected 'Primitive(Int)', got 'Primitive(String)'`; `kernelfield_gen_ok` stays green.

**Why it is NOT landed — the corpus refuses it.** Whole-tree compile, `dag` + `src/v2`, at
`b4c59feb2b`. Baseline (unpatched, same tree, same binary vintage): **2** hard diagnostics, both
unrelated (`workflow CLI default`, `dag/gunbc/tools/review_codex.dag`). Patched: the run emits
**12** diagnostics over **8** files, none of which the baseline emits:

- **8 × `empty list literal: expected type is not a collection`** — `dag/std/claim_evidence.dag`,
  `dag/gunbc/source_integration_landing_spine.dag`, `src/v2/std/bounded_lattice_completeness.dag`
  (×2), `dag/extdeps/git/object_store.dag` (×2), `src/v2/lens/mandatory_tag.dag`,
  `dag/test/claim/host_effect_plan_witness_test.dag`. Every one a `fn(_) { [] }` / `fn(l) { [l] }`
  lambda body in a **function-typed** generic field (`EvidenceInferenceFold` and kin). With the
  substitution now producing a real type, the expectation reaching the lambda body is the
  **function type instead of its return type**. That is the repair being *incomplete*, not wrong,
  and it is the single next thing to fix.
- **4 × frontier-row count mismatch on receiver type `Primitive()`** — `extdeps.git.object_store`
  (`map`, `flat_map`), `extdeps.mercurial` (`any`), `gunbc.scm_compatibility.mercurial` (`map`).
  **These are not reds on correct code, and the row read is what establishes it.** Each is a
  declared row in `v1.compiler.infer` `unresolved_method_frontier`, keyed on receiver shape
  `Primitive()` — the `ReceiverTypeUnestablished` class, whose declared cause is deficit (2) in
  `method_existence_wall_note`: *a lambda parameter receiver whose type never propagates from the
  declared fn type it is bound under*, naming the `StoreObjectFold` lambdas in `object_store`
  explicitly. That is precisely the population row (a) starts supplying real types to, and the row's
  own diagnostic states the contract in both directions: fewer observed means *the deficit has
  partly dissolved and the row must be lowered or deleted so the ratchet keeps its new ground*. So
  these rows **encode the old, nameless answer**, and the ratchet firing is the repair working.

  Two honesty limits on that, stated rather than assumed. The **direction is inferred, not
  measured** — the counts ride on the diagnostic as fields and are absent from the captured message
  text, so a rerun must confirm *fewer* rather than *more*. And the two `object_store` rows are
  **confounded**: that module also carries two of the eight blocking empty-list errors, and a
  blocking error truncates the diagnostic set, which lowers an observed count for a reason that has
  nothing to do with the deficit. The two Mercurial rows carry **no** blocking error in the same
  run, so they are the unconfounded pair and the ones to read first.

  Not to be resolved by editing the counts. An expectation row edited to match new behaviour is
  indistinguishable from one edited to silence a regression unless the correct receiver type is
  established first — that is narrowing the wall in costume.

The repair is recorded here rather than shipped because narrowing it until the corpus greens is how
a wall keeps its name and loses its population. What the measurement actually shows is that **the
instantiated path's silence was masking these representation gaps**, so the wall cannot land ahead
of them.

### Row (c) — expectation not projected through an arrow. NEW, 8 sites, exposed by (a).

At a lambda body inside a **function-typed** generic field, the substituted expectation arrives as
the *function* type rather than its *return* type, so `fn(_) { [] }` fails with `empty list literal:
expected type is not a collection`. **A second finding, not a cost of the first** — the position was
unreachable before, because nothing ever delivered a real type to it. Row (a) cannot land ahead of
this, so this is where the next owner starts.

### Row (b) — type-parameter-typed field (`v: T`). OPEN.

All six seams still RC=0 under the row (a) repair, and `Pair<A, B> { a: A, b: B }` with them.
`sf.inferred` for a `v: T` field holds a nameless node, because `T` is not resolvable when the
binding is constructed. Distinct from (a) and unaddressed.

## Repairs attempted and measured as NOT sufficient

Recorded because they are the cost of the next attempt. Each was built and run against the full arm
table; **every arm was byte-identical** each time.

1. `record_lit_instantiated_fields` reading the type argument with `child_type_node` instead of
   `resolved_type`. Closes a real second-order fail-open — `resolved_type`'s `Absent` arm is
   `error_type`, itself nameless — but the substitution is never applied, so nothing moves.
2. `resolve_item_types` carrying `tr.resolved` onto the field's `inferred`.
3. `resolve_item_types` installing `tr.resolved` as the field's type_expr child (`children[0]`).

**One cause for all three, established by reading the construction site rather than by another
build:** `ResolvedModule.module` is `module_occurrence_input_node(input)` — the raw parsed node.
`build_type_env`'s `local_bindings` folds **those** items through `local_binding_for_item`, which
copies `children` verbatim. `resolve_item_types` runs later, inside `analyze_item`, and its output
feeds `resolved_item` — **never the binding**. `ResolvedModule` means *import*-resolved, not
*type*-resolved.

## A refuted lead, recorded so it is not re-run

The ident-versus-intern address-space theory is **dead**, by source read plus measurement.
`Node.ident` is set from `intern(...).id` in exactly three places — the module node and two import
nodes — so a type use site carries `ident=None`; measured `exp_ident=None`, the ident arm is never
taken, and both `lookup_type_for` and `lookup_type_by_name` return **identical** field lists.

Two §5 defects were found on that path and are real but unrelated to this bug, each worth its own
row: `intern_str` returns `""` on a miss in a `String`-not-`String?` carrier, so a refusal is
unrepresentable — and its arm is **unreachable from all three live call sites**, since the other two
iterate `map_keys(bindings)`, which are intern ids by construction, so it can be proved dead and
deleted. And `lookup_binding` does not refuse on a miss; it retries through a different lookup,
uncounted.

## Found on the way, worth its own row

`v1.compiler.resolve` `resolve_field` installs `type_resolved` into the field via `make_field_node`
and has **zero callers**, while `resolve_item_types` hand-rolls a lossy version beside it. Its
sibling `resolve_field_init` *is* wired, which makes this an **incomplete migration** rather than
abandoned scaffolding, and it is emitted into the stage0 mirror. It is **not** this lane's repair —
the discriminators moved the cause off it — but a cleanup sweep would be entitled to delete it as a
§5 dead scaffold and leave the lossy copy as the only authority. Claim staked on gunbc#8901.

## Rung

Source→acceptance, record-literal field type on a type declared with parameters: **below floor —
not a rung** (§4b), for both rows. Row (a) has a cause and a measured repair blocked on
row (c); row (c) has a located population and no repair; row (b) has neither. The non-generic position is at least
mechanically preventable. Class rung is the minimum across paths, so citing the non-generic path
would be inflation.
