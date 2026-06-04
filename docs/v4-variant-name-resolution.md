# Variant-constructor name resolution is multi-authority and fail-open

**Status:** investigation + fix proposal (not yet landed)
**Scope traced:** v2 stage0 resolver + interpreter (what executes v4 `.dag` today). The v4
`.dag` resolve stage (`03_resolve` / `03_name_resolve`) is *unrun* and **not yet verified** to
share the defect — see [§7](#7-scope-caveat).
**Reproduce the census:** the script in [§8](#8-reproduction) regenerates the collision table.

---

## 1. The bug in one paragraph

A bare variant constructor (`D1`, `Bits64`, `Signed`, …) is resolved against a **flat
per-module map keyed by the bare name**, populated with last-write-wins overwrite and **no
collision check**. When two coproduct types declare the same constructor name, one silently
clobbers the other; which one wins is an artifact of declaration order, not of the program's
meaning. Resolution is **name-only** — the expected type is never consulted to disambiguate — and
the runtime tolerates the result because variant pattern matching compares **only the
variant-name string**, discarding the parent enum entirely. The distinction between two types
that share constructor names is therefore **unenforced**: it is documentary, not load-bearing.
The system runs green today only by the coincidence that colliding names happen to mean the same
thing across their enums.

---

## 2. The resolution mechanism (code evidence)

### 2.1 Constructors are registered in a flat name→binding map, last-write-wins

`src/v2/stage0/src/v2_compiler_infer_items.rs:195-225` — `variant_locals_from_items` folds every
coproduct's variants into one `HashMap<String, TypeBinding>` **keyed by the bare variant name**:

```rust
let child_name = authored_name_at(source_indices, child);   // e.g. "D1", "Bits64"
v2_rt::rc_map_insert(
    vacc,
    child_name,                       // KEY = bare name, no type qualifier
    Rc::new(TypeBinding { name: child_name, resolved: item /* the parent enum node */, .. }),
)
```

`rc_map_insert` **overwrites** on key collision. There is **no duplicate check** anywhere in the
resolver — the only `collides` diagnostic in the inferencer is for a type-param-vs-value-param
clash (`v2_compiler_infer_resolve.rs:2759`), unrelated to constructor names. So when
`DecimalDigit` and `NonZeroDecimalDigit` both declare `D1`, whichever type the fold visits **last**
claims `"D1"`; the other binding is dropped silently.

### 2.2 Lookup is name-only — never type-directed

`src/v2/stage0/src/v2_compiler_infer.rs:788-814` — `lookup_variant_parent_enum(scope, name)` is a
single `map_get(scope.locals, name)`. It takes **only the name**; the expected/scrutinee type is
not a parameter and cannot influence the result:

```rust
pub fn lookup_variant_parent_enum(scope, name) -> Option<String> {
    match v2_rt::map_get(&scope.locals, name) {       // name-only key
        Some(binding) => /* return binding's parent enum */ ,
        None => None,
    }
}
```

`infer_var_binding_kind` (`v2_compiler_infer.rs:816`) wraps it into a
`VariantValueBinding { parent_enum }`. Consequence: in

```
fn integer_nonzero_decimal_digit_widen(digit: NonZeroDecimalDigit) -> DecimalDigit {
  match digit { D1 => D1  ...  D9 => D9 }
}
```

the pattern `D1` (scrutinee `NonZeroDecimalDigit`) and the body `D1` (return `DecimalDigit`)
**resolve to the same single binding** — the last-wins enum — regardless of the two different
expected types. There is no mechanism that uses the expected type to pick between two `D1`s.

### 2.3 The runtime keys on the bare string and ignores the parent enum

`src/v2/stage0/src/v2_interpreter.rs:671-678` — a unit variant evaluates to
`Value::Variant { type_name: parent_enum, variant_name: name, .. }`.

`src/v2/stage0/src/v2_interpreter.rs:962-988` — pattern matching compares **only `variant_name`**
and explicitly discards both the pattern's `parent_enum` and the value's `type_name`:

```rust
MatchPattern::VariantPattern { name, parent_enum: _, .. } => match value {
    Value::Variant { variant_name, .. } => {        // value's type_name dropped by `..`
        if variant_name != name { return None }     // compares "D1" == "D1" only
        ...
```

So a value tagged `NonZeroDecimalDigit::D1` matches a `DecimalDigit::D1` pattern with no
complaint. The widen (`integer.dag:316-328`) is therefore a **structural no-op** wrapping a type
coercion the substrate never honors.

### 2.4 Proof the type distinction is unenforced

`integer_nonzero_decimal_digit_widen` (`src/v4/std/integer.dag:316`) declares
`NonZeroDecimalDigit -> DecimalDigit`, but by §2.2 both `D1`s bind to one enum. So either the body
returns the wrong enum vs. the declared return type, **or** the pattern matches a cross-type
scrutinee. Either way it is a type mismatch — and the function is **merged and green**. That is a
direct proof the checker does not enforce the `NonZeroDecimalDigit` ≠ `DecimalDigit` distinction.

---

## 3. Pervasiveness (census)

The collision is **not** specific to digits. Scanning every `src/v4/**/*.dag` coproduct
declaration (script in [§8](#8-reproduction)) finds **37 constructor names claimed by ≥2 distinct
types**, out of 2281 distinct constructor names. The worst offenders are the per-language
width/scalar/sign enums:

| constructor | # types claiming it | sample owners |
|---|---|---|
| `Bits64` | **16** | `BitsWidth`, `IntegerWidth`, `RustIntWidth`, `GoFloatWidth`, `JavaIntWidth`, `SwiftFloatWidth`, … |
| `Bits32` | 15 | (same width family) |
| `BoolScalar` | 9 | `CppScalar`, `GoScalar`, `RustScalar`, `JavaScalar`, … |
| `Bits16` | 9 | `BitsWidth`, `IntegerWidth`, `RustIntWidth`, … |
| `IntScalar` | 8 | `RustScalar`, `JavaScalar`, `PythonScalar`, `WasmScalar`, … |
| `Signed` / `Unsigned` | 8 each | `Signedness`, `RustIntKind`, `GoIntKind`, `MachineIntKind`, … |
| `Bits8` | 8 | width family |
| `FloatScalar` | 7 | `RustScalar`, `GoScalar`, `WasmScalar`, … |
| `Bits128` | 4 | `BitsWidth`, `RustIntWidth`, `MachineIntWidth`, `GoComplexWidth` |
| `Unrealized` | 3 | the three dissolution-lens verdict enums |
| `CharScalar`, `StringScalar`, `UnitScalar` | 3 each | per-language scalar enums |
| `D1`…`D9` | 2 each | `DecimalDigit`, `NonZeroDecimalDigit` |
| `Named` | 2 | `EdgeLabel`, `NodeRef` |
| `Signed`/`Float`/`Int`/`Pointer`/`Shared`/`Packed`/`Host`/`Absent`/`FailClosed`/`VoidScalar`/`StrScalar`/`NeverScalar`/`ReferenceLayerRc`/`ReferenceLayerBox` | 2 each | mixed |

(Full list of 37 in the script output.) Every one of these is a name whose meaning, at the
resolution layer, is whatever enum was folded last.

### Why it "works" today — and why that is fragile

The collisions are currently **benign by coincidence**: a colliding name happens to mean the same
thing across its enums (`Bits64` is always 64 bits; `Signed` is always signed), and matching keys
on the bare string, so the wrong type-tag never bites. The danger is that nothing *guarantees*
this:

- **Unenforced safety.** Any new enum that reuses a name with a *different* meaning would resolve
  silently to the wrong binding — no diagnostic. (`NonZeroDecimalDigit`'s whole reason to exist —
  "no leading zero" — is already unenforced today; §2.4.)
- **Fictional type safety.** A `RustIntWidth` value satisfies a `GoIntKind` pattern (`Bits64`
  matches `Bits64`), so the per-language type distinctions buy nothing at the boundary.
- **Source ambiguity.** A human reader (and the resolver) cannot tell which type a bare `Bits64`
  denotes without knowing fold order.

This is the [specification-without-execution trap](../INVARIANTS.md) made concrete: 37 latent
multi-authority bindings that pass every check because the distinction is never *exercised*.

---

## 4. Which invariants this violates

| # | invariant | how it's violated |
|---|---|---|
| ① | **DB-5: Substrate Keyed Lookup Is Single-Authority** (P2) | the constructor lookup is keyed by bare name and is multi-authority: 37 names have ≥2 claimants; collision resolves by silent overwrite. The fact "which type owns `Bits64`" lives in zero authoritative places. |
| ② | **P3: Fail-Closed** (*Fabricated fallback*) | an ambiguous constructor reference should fail with a typed "ambiguous constructor" diagnostic; instead the resolver **fabricates** a binding (last writer wins). Fail-open. |
| ③ | **P1: Modeling Faithfulness** (*Hollow alias* shape) | types declare distinctions (`NonZeroDecimalDigit` vs `DecimalDigit`, `RustIntWidth` vs `GoIntKind`) that the resolver + runtime erase. The declared distinction is documentary, not enforced. |
| ④ | **P1: Hand-rolled derived operation** (Practice 10) | `integer_nonzero_decimal_digit_widen` is a structural subtype injection the substrate should derive — it exists only because there is no refinement/subtype primitive. Untracked hand-rolled derived operation. |
| ⑤ | **P5: Progress Is Dissolution** (coproduct dissolution) | `NonZeroDecimalDigit` re-spells 9 of `DecimalDigit`'s constructors; the per-language width/scalar enums re-spell one width/sign vocabulary N times. One concept, many coproducts. |

① and ② are the **load-bearing substrate failures** (general: every colliding name, every
language). ③–⑤ are symptoms ① enables, plus the modeling duplication that *creates* the collisions.

---

## 5. The fix

Three layers, landable independently, in this order:

### 5.1 Hard fail-closed first (the teeth — do this regardless of the syntax debate)

Make `variant_locals_from_items` (or its caller) **detect a duplicate constructor name across
distinct parent types and emit a typed diagnostic** instead of overwriting. This is the P3 fix and
it is the cheapest high-value change: it converts 37 silent multi-authority bindings into a loud,
enumerable failure surface. Run it once and it *is* the census — the compiler tells you every
remaining collision.

Caveat: turning this on **fails the build today** (37 collisions). So it lands together with, or
just behind, the disambiguation syntax (§5.2) so colliding sites have an escape hatch. Sequencing:
(a) add qualified syntax → (b) migrate the 37 → (c) flip the duplicate-name check to hard-fail, so
the gate closes behind a clean tree and stays closed.

### 5.2 Qualified constructor syntax — `Type::Variant`

The real fix is to let a constructor reference name its type, so resolution is single-authority by
construction. Two options:

- **`Type::Variant` (C++-style, preferred).** Needs a new `::` token (`ShColonColon`) — today the
  lexer has `:` (`ShColon`) and `.` (`ShDot`) but **not** `::` (`v2_compiler_tokenize.rs`). Then a
  parse rule for a qualified constructor expr/pattern, and a resolver path that looks up
  `(Type, Variant)` instead of bare `Variant`. The `::` separator cleanly distinguishes
  type-qualification from field access (`.`) and module paths (`.`), which is exactly the C++
  rationale — it does not overload `.`.
- **`Type.Variant` (reuses existing `.`).** No new token — `.` / `ExprFieldAccess` already exist,
  and module paths already use `.` (`import v4.std.algebra`). Cheaper to land, but overloads `.`
  across module-qualification, field-access, and type-qualification, so the resolver must
  disambiguate by context. Consistency-with-modules argument, but muddier.

Recommendation: **`::`** — the extra token is small and the disambiguation is worth keeping
distinct from `.`. Either way, **bare `Variant` stays legal when unambiguous** (only one claimant);
`Type::Variant` is *required* only where §5.1's check reports a collision. That keeps the migration
proportional to the 37 real conflicts rather than a global rewrite.

Resolution then becomes type-directed at the qualified sites: `lookup_variant_parent_enum` gains a
`(name, qualifier)` form, and pattern matching (`v2_interpreter.rs:974`) compares the parent enum
**when the pattern carries a qualifier** (so `RustIntWidth::Bits64` stops matching a `GoIntKind`
value) — closing ③ at the runtime layer too.

### 5.3 Migrate the remainders + dissolve the duplication (P5)

With §5.1 enumerating collisions, walk the 37:

- **Genuinely distinct concepts that happen to share a name** (e.g. `EdgeLabel::Named` vs
  `NodeRef::Named`, `Optional::Absent` vs `LoopMeasureTargetLookup::Absent`) → qualify at the
  ambiguous use sites with `::`. Done.
- **Per-language re-spellings of one vocabulary** (the width/scalar/sign family — the bulk of the
  37) → these are the **refinement-by-duplicate-enum / per-language coproduct duplication** the
  fork-A sweep targets. The honest dissolution is a single shared `Bits{8,16,32,64,128}` /
  `Signedness` vocabulary in `std/`, with per-language types *constraining* it rather than
  *re-declaring* it. That removes the collisions at the source instead of qualifying around them.
- **`NonZeroDecimalDigit` / `NonZeroNat`** → model the refinement as `DecimalDigit` + a nonzero
  constraint/witness, deleting the parallel enum and the hand-rolled `widen` (④). Same sweep.

So §5.2 makes the *accidental* name-sharing safe, and §5.3 deletes the *duplication* that
generated most of it — the two are complementary, not alternatives.

---

## 6. Recommended sequencing

1. **Add `::` token + qualified constructor parse/resolve** (§5.2). Bare names still legal.
2. **Migrate the 37 collision sites** to qualified form where they're genuinely distinct (§5.3
   first bullet); start the width/scalar dissolution for the duplicated-vocabulary bulk.
3. **Flip on the duplicate-constructor hard-fail** (§5.1) once the tree is clean — the gate then
   stays closed by construction (DB-5 + P3 enforced going forward).
4. **Fork-A sweep** finishes the P5 dissolution (width/scalar shared vocab, `NonZero*` → base +
   constraint), each deleting its `widen` and its collisions.

This is consistent with the open `namespacing` PR (#4418, currently a stub) as the natural home
for steps 1–3.

---

## 7. Scope caveat

Everything in §2 is traced through **v2 stage0** — the bootstrap interpreter/resolver that runs v4
`.dag` today. The v4 `.dag` resolve stage (`src/v4/compiler/03_resolve.dag`,
`03_name_resolve.dag`) is not executed today and has **not been verified** to share the flat-name
keying; that check is a prerequisite before deciding whether #4418 must also cover the `.dag`
resolver. Failures ④ (hand-rolled widen) and ⑤ (duplicate enums) are language-level and hold
regardless of which resolver runs.

---

## 8. Reproduction

```python
import re, glob, collections
ctor = collections.defaultdict(list)
type_re = re.compile(r'^\s*type\s+([A-Za-z_][A-Za-z0-9_]*)')
alt_re  = re.compile(r'^\s*([=|])\s*([A-Z][A-Za-z0-9_]*)')
for f in glob.glob('src/v4/**/*.dag', recursive=True):
    cur = None
    for i, line in enumerate(open(f, encoding='utf-8', errors='replace'), 1):
        code = line.split('//')[0]
        m = type_re.match(code)
        if m: cur = m.group(1); continue
        a = alt_re.match(code)
        if a and cur: ctor[a.group(2)].append((cur, f, i))
coll = {n: v for n, v in ctor.items() if len({p for p, _, _ in v}) >= 2}
print(f"{len(coll)} colliding constructor names of {len(ctor)} total")
for n in sorted(coll, key=lambda n: -len({p for p, _, _ in coll[n]})):
    print(f"  {n!r} <- {sorted({p for p,_,_ in coll[n]})}")
```

Run from repo root: `python3 thisscript.py`. Heuristic (line-based; an `=`-introduced single-line
alias like `type RustI32 = Int32` is counted as a one-variant "coproduct", which adds a little
noise to the *total* but does not produce false *collisions* unless an alias RHS coincides with a
real variant name — verify any 2-claimant entry against the source before acting on it).
