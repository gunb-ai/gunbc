> Part of: [THESIS.md](THESIS.md) — modeling guidelines ensure that every `.dag` construct is grounded in external fact, so the causal engine can validate it.

# DAG Modeling Guidelines

Companion to [INVARIANTS.md](INVARIANTS.md) (compiler invariants). This file is now the slim rule surface; worked examples, audits, and implementation targets live under `docs/modeling/`.

## How this doc is organized

Read this file for the active modeling rules and principle names. Read `docs/modeling/` for extended examples, per-file audits, and appendix material.

## Core principle: shared facts, not preferences

Every node in a `.dag` model should be either an axiom grounded in an external source or a derivation composed from those axioms.

This codebase treats modeling as a deductive system, not a preference document. See [docs/modeling/core-principle-shared-facts-not-preferences.md](docs/modeling/core-principle-shared-facts-not-preferences.md).

### No meta-language on top

If the current structure is missing a fact, the fix is to extend the structure rather than add annotations or metadata.

See [docs/modeling/core-principle-shared-facts-not-preferences.md](docs/modeling/core-principle-shared-facts-not-preferences.md#no-meta-language-on-top).

### Start with the fact

Every new construct starts by naming the external fact it models.

See [docs/modeling/core-principle-shared-facts-not-preferences.md](docs/modeling/core-principle-shared-facts-not-preferences.md#start-with-the-fact).

## Foundational primitive: truth-valued structure

The system’s foundation is intentionally small: truth-valued structure justifies the richer engineering primitives the compiler actually reasons over.

The long-form derivation and worked examples live across the linked modeling notes below.

### The single primitive

`Bool` is the unambiguous primitive; wider “primitives” like `Int` and `String` hide decisions that should be modeled explicitly.

See [docs/modeling/the-single-primitive.md](docs/modeling/the-single-primitive.md).

### Why classical logic (and not something else)

The foundation matches classical digital computing, while the composition layer remains more general.

See [docs/modeling/why-classical-logic-and-not-something-else.md](docs/modeling/why-classical-logic-and-not-something-else.md).

### Why Int and String are too wide

Named engineering primitives are useful, but their hidden decisions need explicit structural backing.

See [docs/modeling/why-int-and-string-are-too-wide.md](docs/modeling/why-int-and-string-are-too-wide.md).

### The four-layer model

Surface sugar, composition, semantic kernel, and foundation are distinct layers that should not be collapsed.

See [docs/modeling/the-four-layer-model.md](docs/modeling/the-four-layer-model.md).

### Foundational vs engineering primitives

The compiler reasons at the engineering-primitive layer, but those primitives still need a denotational story.

See [docs/modeling/foundational-vs-engineering-primitives.md](docs/modeling/foundational-vs-engineering-primitives.md).

### Worked examples: how operations fall out

Operations should emerge from structure and declared laws, not from ad hoc feature-specific mechanisms.

See [docs/modeling/worked-examples-how-operations-fall-out.md](docs/modeling/worked-examples-how-operations-fall-out.md).

### Worked examples: how test generation falls out

Test generation should likewise arise from declared structure, contracts, and composition.

See [docs/modeling/worked-examples-how-test-generation-falls-out.md](docs/modeling/worked-examples-how-test-generation-falls-out.md).

### Set operations as compositions on truth

Collection and set operations are modeled as compositions over truth-valued structure rather than separate magic.

See [docs/modeling/set-operations-as-compositions-on-truth.md](docs/modeling/set-operations-as-compositions-on-truth.md).

### Abstraction as surface choice

Abstraction is a surface decision over the same underlying structural facts, not a separate semantic layer.

See [docs/modeling/abstraction-as-surface-choice.md](docs/modeling/abstraction-as-surface-choice.md).

### What qualifies as a shared fact

Facts are shared when disagreement resolves by reading a cited authority or objective structural derivation.

See [docs/modeling/what-qualifies-as-a-shared-fact.md](docs/modeling/what-qualifies-as-a-shared-fact.md).

### What does NOT qualify

Preferences, invented canonicalizations, and hidden policy choices are not shared facts.

See [docs/modeling/what-does-not-qualify.md](docs/modeling/what-does-not-qualify.md).

### Objective relationships

Cross-file and cross-domain links must reflect objective relationships rather than convenience groupings.

See [docs/modeling/objective-relationships.md](docs/modeling/objective-relationships.md).

### Layering

Layering exists to preserve authority and keep derivations readable across the ontology.

See [docs/modeling/layering.md](docs/modeling/layering.md).

## Principles

### M1: Types are compositional facts

Types decompose into smaller types that each assert one fact.

### M2: No duplicate type authorities

Every type is defined in exactly one place.

### M3: Extdeps model specs, not abstractions

`dsl/extdeps/` models real external systems from their actual specifications.

### M4: Closed sets are enums, not strings

Closed domains should be sum types, not stringly proxies.

### M5: Silence is fabrication

Missing data must stay missing or diagnostic, never silently default.

### M6: One result pattern, not N result types

Generic result carriers should replace families of bespoke structurally identical result records.

### M7: Data tables are single-authority

If the same fact lives in data and code, derive from the data and delete the duplicate code.

### M8: Predicates and dispatch are structural

Dispatch should work over structure, not string extraction.

### M10: Concepts get proper homes, not flat slots

New concepts need real files and models before they get referenced from higher-level variants.

### M9: DFS the ontology — every construct attaches to first principles

Every new construct should trace to its parent in the ontology before new ad hoc vocabulary is introduced.

See [docs/modeling/m9-dfs-the-ontology.md](docs/modeling/m9-dfs-the-ontology.md).

### Navigating the concept DAG: where to start

The `dsl/std/` tree is the concept DAG; read it from roots to compositions to domain vocabularies.

See [docs/modeling/navigating-the-concept-dag.md](docs/modeling/navigating-the-concept-dag.md).

## Exemplary models

The strongest reference models are preserved out of line so this file stays rule-sized.

### Foundation chain (reference implementation)

See [docs/modeling/foundation-chain-reference-implementation.md](docs/modeling/foundation-chain-reference-implementation.md).

### Other strong models

See [docs/modeling/other-strong-models.md](docs/modeling/other-strong-models.md).

## Per-file findings

The detailed per-file audit now lives under `docs/modeling/`.

### dsl/std/

See [docs/modeling/per-file-findings-dsl-std.md](docs/modeling/per-file-findings-dsl-std.md).

### dsl/extdeps/

See [docs/modeling/per-file-findings-dsl-extdeps.md](docs/modeling/per-file-findings-dsl-extdeps.md).

### src/v2/ (compiler)

See [docs/modeling/per-file-findings-src-v2-compiler.md](docs/modeling/per-file-findings-src-v2-compiler.md).

## Deleted files (this session)

The current deleted-file ledger stays intentionally brief here because the detailed implementation analysis moved out of line.

## Known future work

The full future-work queue and accepted debt are preserved in [docs/modeling/known-future-work.md](docs/modeling/known-future-work.md).

## Appendix: Preferred implementations

Preferred target implementations, design sketches, and replacement shapes now live in [docs/modeling/appendix-preferred-implementations.md](docs/modeling/appendix-preferred-implementations.md).
