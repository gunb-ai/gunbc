## Core principle: shared facts, not preferences

Every node in a `.dag` model should be either:
- **An axiom** — a fact cited from a standard, specification, or API doc
- **A derivation** — composed from axioms via an objective relationship

The modeling is a deductive system, not a design document. If someone
disputes a cross-section of the DAG, the resolution is "here's the spec"
— not "here's why I think this is a good abstraction."

At any cross-section of any DAG in the codebase, the content should be
**non-controversial** — a shared fact that people actually agree on.

### No meta-language on top

The `.dag` language is itself the meta-language for formalizing
intersubjective programmer agreements. Adding annotations or metadata
on top of `.dag` would create a meta-meta-language — another dimension
of intersubjectivity ("annotate this to fix this concept I don't like").

When a fact needs structural representation, define a proper `.dag`
structure with proper transforms. Algebraic laws are `.dag` functions,
not string annotations. Type facts are `.dag` data fields, not comments
or decorators.

The test: if you find yourself reaching for an annotation, you've found
a fact that the current structure doesn't capture. The fix is to extend
the structure — new type fields, new edge relationships, new function
signatures — not to paper over the gap with metadata.

### Start with the fact

Every new construct should begin by identifying the external fact it
models. This is not aspirational — it is the entry point for all design
work in this codebase.

| Kind of construct | Fact source |
|---|---|
| Algebraic structure | Mathematical axioms (associativity, identity, etc.) |
| Type definition | Language specification, standard, or structural derivation |
| External service | API documentation, protocol spec |
| Cross-language mapping | Shared algebraic structure (both targets inhabit the same algebra) |
| Refinement type | Narrowing predicate derivable from the base type's definition |

If you cannot name the fact, the construct is not ready. This is the
difference between this codebase and a design document: a design document
expresses preferences; this codebase expresses facts. An ungrounded
construct is a preference masquerading as a fact.

