# A type parameter is being shadowed by a same-named global declaration

**Found 2026-08-17 on `integration/namespace-cut`. 22 of 185 diagnostics, one
cause.** Unlike the `Nat` class this is unambiguously a defect: no modeling
question has to be settled first.

## The collision

```
src/v2/std/witness.dag:4              type Witness<C>
                                        = Holds { value: C }
                                        | Violates { diagnostic: Diagnostic }

dag/extdeps/languages/c/subject.dag:13  type C
                                          = | CTarget
```

`C` is the type PARAMETER of `Witness`. It is also the name the C-language
subject module gives the C language itself. Post-cut, uses of the parameter
resolve to the global declaration:

```
src/v2/compiler/source_authority.dag:1284:20:
  error: type mismatch: expected 'Coproduct(C)', got 'Product(SourceAstEqual)'
```

`Coproduct(C)` is the giveaway — the parameter has become the C-language
coproduct. The refusal is even reported AT `dag/extdeps/languages/c/subject.dag`
for code in `src/v2`, which is how it was found: a diagnostic pointing at a file
that does not contain the name it is complaining about.

## Why this is the resolver, and why it is clearly wrong

DESIGN's namespace-only resolution says a reference is a **lexical lookup up the
ancestor chain**. A type parameter is a binder introduced by the declaration
being checked, so it is the NEAREST binding and must win over a module-scope
declaration anywhere in the corpus. It currently does not.

The import era hid this: `C` entered a file's name universe only if that file
imported it, and no file importing `extdeps.languages.c` also instantiated
`Witness`. Delete imports and every module-scope name is a candidate, so a
one-letter parameter now competes with a one-letter type.

## Scope, measured

`C` is the ONLY single-letter global type declaration in the corpus, so this
collision is exactly one name today. The other single letters appearing in
diagnostics -- `M` (4), `V` (2), `T` (1) -- have no global declaration and are a
DIFFERENT problem (`no field 'fact' on type 'M'` is a field access on an
unconstrained parameter, not a shadowing).

## Two candidate repairs, and why the choice matters

1. **Fix the resolver so a type parameter shadows.** This is the correct repair:
   it is a construction rule (§5), it fixes every future collision, and it makes
   the invalid state unreachable rather than avoided.
2. **Rename `type C`.** This makes the 22 rows disappear and leaves the defect
   live. The next single-letter type declaration -- or any corpus that adds one
   -- reintroduces it silently.

Repair 2 is tempting precisely because it is a one-line diff and the diagnostics
go away, which is the shape this branch should be most suspicious of. Renaming
may still be worth doing on modeling grounds (a one-letter type name is thin),
but it must not be done INSTEAD of the shadowing fix, or the branch will have
bought green by removing the only witness to a live resolver defect.

## Next step

Locate where type parameters are bound during resolution and confirm they are
absent from the lookup chain rather than merely ranked below module scope. The
discriminating control already exists in tree and needs no authoring: revert the
rename (if taken) and `Witness<C>` must still resolve to the parameter.

## Located candidate mechanism (2026-08-17) — REFUTED, see below

The exclusion machinery already exists, which narrows this from "the rule is
missing" to "the rule loses a race".

`v1.compiler.infer` `census_upgrade_type_decl_binding` builds an `excluded` set
from `node.params |> map(generic_param_name_at)` and passes it to
`qualify_decl_reference_positions`, so a declaration's own type parameters are
deliberately kept out of qualification. `census_qualify_leaf_binding` does the
same. So parameters are known and are meant to be protected.

The suspect is `stamp_type_param_occurrences`, which converts a bare node whose
label is a parameter name into `TypeVariable { id: nm }` -- but only under

```
if n.inferred == none { ...stamp... } else { n }
```

If reference resolution has already bound that node to the global `C`, then
`inferred` is populated and the stamp DECLINES rather than overriding. On that
reading the defect is an ORDERING one: a module-scope resolution that ran first
is treated as settled, and the nearer binder never gets to replace it. That also
explains why it appears only now -- the guard is unchanged, but before the cut
there was no reachable global `C` for resolution to have bound first.

**Deliberately not patched on this reading.** Five hypotheses died on the `Nat`
class today, every one of them plausible at this level of detail, and this is
load-bearing inference code. What is owed first is the discriminating
observation, not a diff:

- does `param_names` for `Witness` actually contain `C`? (if not, the fault is
  upstream in `generic_param_name_at` and the guard is innocent)
- is `inferred` already `Present` on the `C` node when `stamp_type_param_occurrences`
  reaches it? (if not, this whole reading is dead)

Either answer is one instrumented run. Both are cheap next to a wrong change in
`04_infer`.

## The candidate above is REFUTED, and its refutation names the real shape

`stamp_type_param_occurrences` has **no external caller**. In
`src/v1/04_infer.dag` it appears only at its own definition and its own
recursive call, and the same is true of the generated seed
(`v1_compiler_infer.rs`). It never runs. So its `inferred == none` guard cannot
be declining anything, and the ordering story is dead.

That is an INERT MECHANISM: someone wrote the machinery that marks a
parameter-named node as `TypeVariable { id: nm }`, and nothing ever calls it.
DESIGN names this class directly — registration is not enforcement, and an
inert lens is itself a lie. Here it is worse than inert, because its existence
made a wrong explanation look well-supported: I read a guard, found it
plausible, and wrote it down. The function reads exactly like the protection
this defect needs.

**What is actually live** is the `excluded` set in
`census_upgrade_type_decl_binding` / `census_qualify_leaf_binding`: a
declaration's own type parameters are kept OUT of qualification, so `C` inside
`Witness` is deliberately left as a bare name.

And that is the defect, stated properly:

> Type-parameter protection is implemented as **"do not qualify this name"**.
> Under import-scoped visibility that was sufficient — a bare `C` had no global
> binding unless the file imported one, and nothing importing the C-language
> subject also instantiated `Witness`. Under namespace-only resolution, leaving
> a name BARE is the opposite of protecting it: bare is precisely what resolves
> against every module-scope declaration in the corpus.

So nothing regressed in the exclusion logic. The cut changed what "bare" means,
and a mechanism whose whole strategy was to leave the name alone became a
mechanism that hands it to the global namespace.

## Repair direction (unchanged in preference, now better grounded)

The parameter must carry a POSITIVE marking that survives into resolution —
which is what the dead `stamp_type_param_occurrences` was evidently written to
do — rather than being protected by omission. Either wire that function in at
the point declarations are upgraded, or have resolution consult the enclosing
declaration's parameter names before module scope.

Renaming `type C` remains the wrong repair for the reason already given, and now
for a second: it would leave a corpus-wide rule ("a bare name is a global
reference") silently wrong for every generic declaration, with no witness left
to say so.
