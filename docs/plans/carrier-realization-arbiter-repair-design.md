# Repair design: one host realization per carrier, reached through one identity-keyed authority (2026-08-21)

**Session:** `royal-dove-436`. **Work item:** `node://adhoc-c735d227-60b`.
**Authorized by:** `smart-ram-730`, after the arm census in
[`t2_t3_realization_route_2026-08-21.md`](../probes/t2_t3_realization_route_2026-08-21.md).
**Status: DESIGN. Nothing here is built, and one question is deliberately left to its owner (§5).**

## What the measurement forces, before any design choice is made

The arm census over the 52 shared-root sites is:

| arm | T2 | T3 |
|---|---:|---:|
| **A** — generic carrier signature (`list_append`, `length`, `is_empty`) | **25** | 0 |
| **B** — declared-structure constructor | 4 | 17 |
| **C** — direct carrier-to-carrier assignment | 3 | 0 |
| UNALIGNED — not attributed | 2 | 1 |

Arm A is what makes this a forced conclusion rather than a menu of options:

> `list_append<T>(left: FreeMonoid<T>, right: FreeMonoid<T>)` is emitted **once**, generically, so it
> has exactly **one** host parameter type. Its `.dag` callers pass values whose declared type is
> `String` — the same type, by `std.string_type` `String = FreeMonoid<Char>`. If the carrier has two
> host realizations, the generic function can only be one of them, and every call from the other is
> an E0308 **that no call-site edit can remove**, because the two host types are genuinely distinct.

So: **a modeled carrier with two host realizations cannot have a generic function over it.** The
repair is therefore not "make the two renderers agree at each site"; it is **one host realization
per carrier**, with every position deriving from it. Everything below follows from that.

This also disposes of two candidate repairs without anyone spending a day on them, which is what the
census was for:

- **A constructor-side fix** (teach `emit_typed_record_lit` the carrier decision) closes arm B —
  21 of 52 — and leaves arm A's 25 standing, the largest arm.
- **Anything relying on monomorphization** cannot help: the type renderer's test is
  `rust_host_text_carrier_elem_name(n) == "Char"`, a **syntactic** read of the authored element
  spelling, decided **before** instantiation. A generic `T` is never spelled `Char`.

## The authority already exists, is already correct, and is already consumed

This corrects a claim made earlier in this lane's PR body and messages. **`type_realization_decision`
does not have zero consumers.** `v1.compiler.coercion` `lookup_checkpoint` is a thin derivation of it
for every `decl_file != ""` caller, and `v1.compiler.trait_derive_emit`'s alias-hop arm reaches it
that way. The earlier claim came from grepping the *type* name `TypeRealizationDecision` rather than
the *function*; it was wrong.

The corrected picture is stronger, not weaker:

- `structural_declaration_modules_for("String")` = `["src/v2/std/text.dag", "dag/std/string_type.dag"]`.
- So `type_realization_decision` **would** refuse the native `String` spelling for a reference
  resolving to either module, and `coerce_primitive_type` would render the declaration's structure.
- But `render_rust_decl_type` and `render_rust_fn_sig_type` — and `render_rust_type`,
  `render_rust_type_without_applied_binding`, `render_rust_applied_type` — each **return on their
  first line** via `is_host_text_carrier_type` → `"String"`, unconditional on `decl_file`.

**A `String`-spelled reference reaching any of those five renderers cannot reach the authority.** The
roster row is therefore *inert for the class it was added for*: not a missing wall, an unreachable
one — DESIGN §6's coverage-by-illusion, and a §4b rung the corpus believes it holds. The same
three-line preamble is copied across five renderers, which is also the §2/§3 forked-logic tell.

**Bounding this claim honestly:** it is established by reading the five renderers' control flow, not
by executing a discriminating input. `is_host_text_carrier_type` is unconditionally the first
statement and returns, so no path through those five reaches `lookup_checkpoint` for a
`String`-spelled node; that much is decidable from the source. Whether some *sixth* renderer handles
some type position without the preamble is **not** established here.

## What the repair is

**Delete-first, per DESIGN §3.** X is the spelling-keyed short-circuit family; Y is
`type_realization_decision`. Y already exists, is already identity-keyed, and already answers. The
migration is to remove X and let every position derive from Y — not to add a third mechanism that
reconciles X with the value renderer.

Three steps, ordered so each is independently measurable:

1. **Make the authority reachable in type position.** Remove the `is_host_text_carrier_type`
   short-circuit from the five renderers; the carrier's host type comes from
   `type_realization_decision(dag_name, decl_file)`. The five copies collapse to zero, not to one
   shared helper — the decision they were approximating already has a home.
2. **Make the value renderer ask the same question.** `emit_typed_record_lit`'s
   `Cons`-under-`FreeMonoid` arm and its `rust_seed_host_container_base` zero-field arm are
   element-blind spelling tests (`List | FreeMonoid → "Vec"`, in full). They consult
   `type_realization_decision` for the *resolved declaration being constructed*, exactly as step 1's
   type positions do. `PointwisePower` / `PartialFunction` record literals route the same way, which
   is what closes T3's 17.
3. **Retire the `decl_file == ""` residue.** `v1.compiler.coercion`'s DECLARED RESIDUE note names
   this as its own retirement trigger and states the population is "not counted here". **It is
   counted here: 8 production call sites and 2 test-generation sites**, all named below. None of
   them are the type-position renderers, which already pass real identity via
   `type_reference_decl_file` — so this residue is *not* the cause of the 52, and step 3 is
   independent of steps 1–2 rather than a prerequisite for them.

   | file | enclosing fn | passed name |
   |---|---|---|
   | `v1.compiler.05_emit` | `emit_literal` | `"String"` ×2, `"Symbol"` |
   | `v1.compiler.05_emit` | `render_node_type` | `"Refined"` |
   | `v1.compiler.05_emit_rust` | `rust_opaque_kernel_alias_carrier` | *(param)* |
   | `v1.compiler.05_emit_rust` | `rust_opaque_kernel_alias_type_decl` | *(param)* |
   | `v1.compiler.05_emit_rust` | `is_dag_value_type_name` | *(param)* |
   | `v1.compiler.05_emit_rust` | `emit_json_value_extract` | *(param)* |
   | `v1.compiler.coercion` | `template_application_tests` | `"Int"`, `"String"` (testgen, not production) |

## The one question this design does NOT answer, and why it is not mine to answer

Step 1 forces a direction, and the direction is a **policy about the seed's own representation**, not
a fact this measurement produced:

- **Structural** — `String` renders as the free monoid over `Char`, so every `String` in the seed
  becomes `Rc<Vec<i64>>`. This is the direction the corpus has **already declared**, in
  `structural_declaration_modules_for("String")`; the bypass is what makes that declaration inert. It
  is also a very large change to emitted output whose blast radius is **unmeasured**.
- **Native** — `String` renders as host `String`, and the free-monoid spellings follow it. This keeps
  the seed's current output but requires the generic arm-A signatures to be reachable as host
  `String`, which a single generic emission cannot express — so it needs a separate answer for arm A
  and is not obviously cheaper.

The corpus's own precedent says the choice must be made with a measured population, not by argument:
`checkpoint_table_bypasses_identity_note` records that the over-broad `Bool` roster row refused
native rendering for 5 unrelated fixtures and drove `required-regen` drift across the `std_*.rs`
mirrors, and calls that a fabricated-plausible-ceiling failure. Picking a direction here without the
same measurement would repeat it.

**So the next executable step is a measurement, not a merge:** remove the bypass in a scratch tree,
re-emit the same closure, and report **site conversion per arm** against
`t2_t3_realization_route_2026-08-21/arbiter_arms.tsv`. That is a bounded emit-only run, it produces
the blast-radius number the direction decision needs, and it merges nothing.

## Measurement protocol for any repair proposed under this design

Carried from `smart-ram-730`'s cautions, sharpened now that the arms exist.

1. **Report site conversion between arms, never category totals.** With two authorities live, a
   change can move a site from arm A to arm B and read as progress. The instrument is a per-site join
   against `arbiter_arms.tsv`, keyed on `(file, line)`, classifying each prior site as *closed*,
   *converted to arm X*, or *unchanged* — not an E0308 histogram difference.
2. **The discriminating control must show the SAME authority consulted in BOTH positions.** A control
   that only shows sites falling is equally consistent with one renderer's special case merely
   ceasing to fire — the absorbing-fallback shape (DESIGN §5) wearing a green build. The control
   must exhibit one declaration answered identically from a type position and a value position.
3. **Positive control, in the same emitted crate:** `pub type FreeMonoid<T> = Vec<T>` and
   `pub type List<Element> = Vec<Element>` resolve to the same host type today and contribute **zero**
   sites between themselves. Any repair that breaks that agreement must fail loudly; the control is
   a baseline that already holds, not one a repair has to create.
4. **Rung honesty (DESIGN §4b).** Removing an unreachable special case does not by itself climb a
   rung — it makes an existing wall reachable. The class stays *mechanically preventable* until a
   discriminating RED is enrolled and executes; the inert-wall finding above is precisely what an
   unexecuted claim looks like, and this design must not repeat it one level up.

## What this design does not establish

- **It is not costed.** No estimate is offered for steps 1–3, because the direction question above
  gates the blast radius and that number does not exist yet.
- **It does not claim the three arms share one fix.** Steps 1 and 2 are argued to reach arms A/C and
  B respectively; that argument is from the traced control flow, not from an executed change.
- **It does not touch the v1 freeze question.** Steps 1–3 all edit the v1 seed emitter. Under the
  DESIGN §3 purpose test that is admissible only insofar as it serves the v2 self-host program —
  which the E0308 board is — but the admission is the operator's to make, not this document's.
