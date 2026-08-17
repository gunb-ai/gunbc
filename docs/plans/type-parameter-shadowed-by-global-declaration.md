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
