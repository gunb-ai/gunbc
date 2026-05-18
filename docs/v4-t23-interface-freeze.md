# v4 T-23 Interface Freeze

This is the pinned interface contract for `src/v4/lens/application.dag`.
It freezes the boundary that downstream lens, report, synthesis, CI, and
agent-surface work may consume before the executable T-23 body lands.

## Authority

- `src/v4/lens/application.dag` is the only substrate home for
  `apply_lens`.
- `apply_lens` is a first-class declaration over `Node`, not an annotation
  channel and not a side table.
- Absence of an `apply_lens(..., Enforce { ... })` declaration means
  introspect-only. There is no implicit enforcement default.
- `apply_lens(..., Enforce { ... })` is the only advisory-to-fail-closed
  bridge for `std/report.dag` advisory output.

## Frozen Surface

T-23 owns these names and their boundary meanings:

- `SectionRef = DeclarationScope | NodeScope`: where a lens is applied.
- `IntrospectApplication<Output>`: advisory lens application at a section;
  no budget axis and no enforcement metadata.
- `EnforcedApplication<Output, Budget>`: opt-in contract over a lens result
  at a section with an explicit budget/comparator surface.
- `apply_lens(lens, section, config)`: the user-facing application
  declaration. The config chooses introspection or enforcement explicitly.
- `subterm_at(root: Node, p: Path) -> Outcome<Node>`: fail-closed structural
  read over `std/node.dag` `Path`.
- `apply_diff(root: Node, d: Diff) -> Outcome<Node>`: fail-closed,
  all-or-nothing sequential application of ordered `Edit`s.

`Outcome<T>` is the concrete v4 carrier from `std/diagnostic.dag`; older
prose that says `Result<T, Diagnostic>` refers to this same fail-closed
boundary and must not introduce a second result carrier.

## Non-Authorities

- No `query.dag`, query subsystem, or user-vs-kernel lens split.
- No annotation channel for enforcement.
- No second advisory-to-blocking bridge outside `apply_lens(..., Enforce)`.
- No total `apply_diff : (Node, Diff) -> Node`; external diffs can be stale
  and must fail closed.
- No AGENT-1-owned return carrier for affected sets; agent clients consume
  whatever `lens/affected_set.dag` declares.

## Consumers

Consumers may cite this file for the frozen T-23 interface while the
implementation remains scaffolded. If a downstream task needs any name,
field, or default that is not listed here or in `src/v4/lens/application.dag`,
that is a T-23 interface change and must be operator-ratified before use.
