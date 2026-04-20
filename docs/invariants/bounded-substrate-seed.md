## Bounded Substrate Seed

The Rust-native seed that exists before any `.dag` declaration loads is
a ratchet, not an escape hatch. The seed may stay flat or shrink; it may
not grow without explicit deletion elsewhere in the seed or a narrowly
argued exception that names why the new primitive is truly indivisible.

At the current reflection boundary the intended seed is minimal:

- Parser and tokenizer substrate for reading source text.
- Resolve / bootstrap machinery needed to load the first declarations.
- Primitive scalar groundings (`Int`, `Bool`, `String`) and the atomic
  identity handles (`NodeId`, `PortId`, `DeclarationId`, `SourceSpan`).

Everything else in the compiler substrate must live as a `.dag`
declaration. `Dag`, `Declaration`, `Behavior`, `TypeConnective`,
behavior payload records, and realization metadata are no longer valid
"just keep it in Rust" categories now that the reflection surface
exists. If a concept is structurally representable and used above the
seed boundary, the correct move is to declare it in `.dag` and attach a
realization, not to enlarge the seed.

The enforcement rule is monotonic: count seed primitives in a tracked
ratchet file or equivalent CI gate, and block PRs that increase the
count. The project can tolerate temporary seed holes only when the hole
is explicitly named, bounded, and shrinking.

