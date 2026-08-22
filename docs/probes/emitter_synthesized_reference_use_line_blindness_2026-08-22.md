# A source-derived use-line collector is structurally blind to emitter-synthesized references

**Measured 2026-08-22 on `integration/namespace-cut`. Filed, not started.**

## The claim

A candidate collector that walks **source** cannot propose a use-line for a reference the
**emitter** invents. This is a ceiling on the approach, not a bug in one collector: there is no
source antecedent for any walk to find.

## How it was reached

The E0425 "cannot find type" class on this branch was 105 occurrences over 37 identities. A repair
that carries a declared type's field types into the candidate population closed 23 identities
(105 → 32 occurrences; four-bucket receipt in the commit that landed it, with zero newly exposed).
The 14 remaining identities then split three ways, and the third bucket is this document's subject.

| bucket | rows | mechanism |
|---|---|---|
| dropped at the `StructRepr` filter | 1 | `AnnotationAttachmentRefusal`, discarded for being an enum |
| qualified field declarations | 5 | consistent with the access-peel reading |
| **no source occurrence at all** | **11** | **emitter-synthesized — this document** |

For the 11, the consuming `.dag` contains **zero occurrences of the name**, with string literals and
`//` annotations stripped: not qualified, not bare, nowhere. Verified on `TypeBinding` in
`v1.compiler.emit`, `FilePath` in `v1.compiler.emit_rust`, `Nat` in `v1.compiler.complexity`, and
`SubValueRelation` in `v1.compiler.infer_lookup`.

The emitted Rust shows where they come from — three distinct construct shapes, none written by any
author:

```rust
|a: &FilePath, b: &FilePath| { .. }             // generated sort comparator
v1_rt::rc_empty_map::<i64, Rc<TypeBinding>>()   // turbofish on a generated empty map
Option<Rc<Measure<(), (), Nat>>>                // emitter-produced generic instantiation
```

## Why this is a ceiling and not a defect count

Three independent source-side treatments were built and measured against this class, and all three
changed **0 of 166** mirrors: a type-declaration field-surface collector, seeding each item's own
name into the existing field expansion, and the access-peel reading. The repair that did move the
product (11 of 166) did so by reading the emitter's **own type summary**, not by walking source
harder. The 0-of-166 results are retroactively explained: a source walk cannot reach a name that is
not in source, so no amount of widening one gets there.

## The terminal shape already exists in the tree

`v1.compiler.emit_rust` `reference_derived_use_lines_note` already names the end state — derive
use-lines from the **emitted reference set** rather than from source references, which "needs no
roster of intercepts at all because an intercepted call emits no path". That note was written about
host-realized builtins. This population is disjoint from that one and arrives at the same
conclusion, so two independent paths now terminate at one shape.

## What is NOT claimed

- No count of how many future references this would prevent. The obvious join — dropped, absent from
  the use-line set, not yet referenced — returns 21357 rows over 132 identities because the predicate
  has no notion of *would need it*; it is an upper bound with no lower bound and it is not used here.
- The `StructRepr` filter's realized cost is **one** defect, and that one is already counted inside
  the 14. Its shape is wrong (it asks *is this a struct* when the requirement is *does this field
  type need an import*), but its priority is hygiene.
- This document does not propose the migration. It records the evidence that makes the item
  fundable: the 11 rows, their three construct shapes, the zero-source-occurrence proof, and the
  note it converges with.
