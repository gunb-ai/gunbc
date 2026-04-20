### No case enumeration for open sets

When behavior varies by type, variant, or category, prefer a single
algorithm that walks the structure over a match/list that enumerates
known cases. Enumerated lists rot: every new case requires updating
every list, and the compiler won't tell you which lists you missed.

**The test:** if adding a new type/variant requires editing a match arm
somewhere other than the type definition itself, the code has an
enumeration that should be replaced with a structural walk.

Matching on a closed enum (`WrapperKind::List | Set | Optional | ...`)
is fine — adding a variant is a compiler error. The problem is
open-ended lists keyed by strings, type names, or error message
substrings.

**Structural prevention:** Data tables loaded at pipeline startup,
not match arms in code. The `SyntaxSpec` pattern: keywords, operators,
and item forms are data in `.dag` files. `parse_item` reads the data
— there are no match arms to add. The same pattern applies to method
dispatch (algebra types in `std/algebra.dag`), type dispatch (structural
properties on nodes), and container ops (templates in `LanguageSpec`).
The escape hatch is `if name == "..."` branches; the fix is a data
lookup where the data is the `.dag` source itself.

