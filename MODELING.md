> Part of: [THESIS.md](THESIS.md) — modeling guidelines ensure that every `.dag` construct is grounded in external fact, so the causal engine can validate it.

# DAG Modeling Guidelines

Companion to [INVARIANTS.md](INVARIANTS.md) (compiler invariants). This is the slim rule surface: the active modeling rules and principle names. The review rubric that operationalizes these principles — practices, dissolution findings, calibration — is [docs/modeling-discipline.md](docs/modeling-discipline.md).

## Core principle: shared facts, not preferences

Every node in a `.dag` model should be either an axiom grounded in an external source or a derivation composed from those axioms.

> **Purpose.** This modeling philosophy serves the project's core theme: the **derived homomorphism**. The compiler derives cross-target translation from correctly-modeled facts (N+M models, not N×M adapters) — so a faithful model is what the derivation rests on. See [THESIS.md](THESIS.md) → "The derived homomorphism" and [docs/thesis/the-derived-homomorphism.md](docs/thesis/the-derived-homomorphism.md).

This codebase treats modeling as a deductive system, not a preference document.

### No meta-language on top

If the current structure is missing a fact, the fix is to extend the structure rather than add annotations or metadata.

### Start with the fact

Every new construct starts by naming the external fact it models.

## Foundational primitive: truth-valued structure

The system's foundation is intentionally small: truth-valued structure justifies the richer engineering primitives the compiler actually reasons over.

### The single primitive

`Bool` is the unambiguous primitive; wider "primitives" like `Int` and `String` hide decisions that should be modeled explicitly.

### Why classical logic (and not something else)

The foundation matches classical digital computing, while the composition layer remains more general.

### Why Int and String are too wide

Named engineering primitives are useful, but their hidden decisions need explicit structural backing.

### The four-layer model

Surface sugar, composition, semantic kernel, and foundation are distinct layers that should not be collapsed.

### Foundational vs engineering primitives

The compiler reasons at the engineering-primitive layer, but those primitives still need a denotational story.

### Worked examples: how operations fall out

Operations should emerge from structure and declared laws, not from ad hoc feature-specific mechanisms.

### Worked examples: how test generation falls out

Test generation should likewise arise from declared structure, contracts, and composition.

### Set operations as compositions on truth

Collection and set operations are modeled as compositions over truth-valued structure rather than separate magic.

### Abstraction as surface choice

Abstraction is a surface decision over the same underlying structural facts, not a separate semantic layer.

### What qualifies as a shared fact

Facts are shared when disagreement resolves by reading a cited authority or objective structural derivation.

### What does NOT qualify

Preferences, invented canonicalizations, and hidden policy choices are not shared facts.

### Objective relationships

Cross-file and cross-domain links must reflect objective relationships rather than convenience groupings.

### Layering

Layering exists to preserve authority and keep derivations readable across the ontology.

## Principles

### M1: Types are compositional facts

Types decompose into smaller types that each assert one fact.

The canonical carrier for a compositional type is a **fact-bundle**: a `Conj` / record whose fields are **named edges** (`Edge { label: Named { name: … }, target: … }`), each field asserting one spec-read fact. Bare aliases (`type X = Y`) and positional-only `Conj` without named fields are under-modeled carriers — they assert an identity or shape while reading zero facts.

> Fact modeling is the **inputs facet** of the derived homomorphism: the facts a type asserts are what the compiler derives the cross-target map *from*.

Do not hand-roll a derived operation. If a function's behavior is determined entirely by the shape of a modeled type, it is re-deriving something the compiler already derives. The deficiency is in the model, not the code — model the missing fact; do not hand-roll the operation.

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

**Mechanical trigger:** any new `is_*`, `has_*`, `*_is_*`, `*_has_*`, `non_empty`, `is_empty`, or similar `Bool` helper over a coproduct that matches the value and returns true for one variant and false for another is **predicate dissolution until proven otherwise** — the property belongs on the variant (or as a declared fact the consumer matches on), not in a side classifier ([docs/modeling-discipline.md](docs/modeling-discipline.md) Practice 10).

### M9: DFS the ontology — every construct attaches to first principles

Every new construct should trace to its parent in the ontology before new ad hoc vocabulary is introduced.

This applies to **operations as well as types**: before hand-writing a fold, walker, or accumulator, DFS `std/` for the derived operation that already carries it. A hand-rolled fold whose accumulator shape coincides with an existing carrier *is* that carrier — e.g. a find-unique-row fold with a `Missing | Unique | Ambiguous` accumulator **is** `find_witness`; writing it locally duplicates the authority (Practice 11 parametric-duplication).

### M10: Concepts get proper homes, not flat slots

New concepts need real files and models before they get referenced from higher-level variants.

### Navigating the concept DAG: where to start

The `dsl/std/` tree is the concept DAG; read it from roots to compositions to domain vocabularies.
