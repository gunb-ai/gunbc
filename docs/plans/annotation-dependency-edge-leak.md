# Source annotations create real dependency edges (§4c violation)

**Status: open defect, proven by execution. No fix landed.**
Recorded here because the finding outlives the pull request that discovered it
(gunbc#7996) — a comment erased with a PR is how a real finding gets lost.

## The claim

A dotted module path written inside a `//` source annotation becomes a **real
dependency edge** in the general source-loading fixpoint. Annotations are
supposed to be invisible to semantic passes; this one silently pulls a module
into the closure.

DESIGN §4c is explicit that this must not happen:

> Annotation capture and provenance are disjoint from semantic occurrence
> allocation: adding, deleting, or moving an annotation cannot alter any
> semantic occurrence identity, semantic graph, resolution result, semantic
> hash, or target-program bytes.

Adding an annotation here alters the resolution result. That is the violation.

## The mechanism

`v1_compiler.cli_run` `referenced_module_paths_in_text` walks raw source bytes
looking for dotted paths that match a known module. It carries **one** erasure —
string literals, including backslash escapes — and **no comment handling at
all**. Grepping the function body for any comment concept returns zero.

So the erasure set is one class short. A path inside a string literal is
correctly ignored; the same path inside a `//` annotation is read as a
reference.

## The two consumers

Both are in the general source-loading fixpoint, so this is not confined to a
diagnostic path:

- `v1_compiler.cli_run` `extend_with_reference_closure`
- `v1_compiler.cli_run` `reference_pull_paths_for_source`

*(Named by symbol, not by line, per DESIGN §3's cite-the-symbol rule. The
function moved by ~108 lines between 2026-08-07 morning and evening without
anyone touching it — a positional citation written that morning would already
be stale.)*

## Execution receipt

Discriminating input: one file whose **only** occurrence of a dotted module path
is inside a `//` comment.

```
baseline   (path absent)                -> []
perturbed  (path present in a comment)  -> ["std.occurrence_binding_candidates"]
```

The projection changes. Nothing else about the file changed, and nothing
semantic references that module.

## Why this is not merely cosmetic

The leak runs in the direction that widens rather than narrows, so today it
costs load and not correctness: an extra module enters the closure. Two reasons
it still matters.

1. It is **rung-inflating** with respect to §4c. The document states a
   structural guarantee — annotations cannot alter a resolution result — and the
   realization does not hold it. The stated rung and the executed rung disagree,
   which §4b names as worse than sitting low.
2. It makes annotation edits **semantically load-bearing**, which is precisely
   what §4c forbids. Deleting a comment can remove a module from the closure.
   Under a precise affected-set selection that is a silent narrowing, not a
   widening — the empty-observation narrow, with a comment as its trigger.

## Attainable ceiling

**Structurally impossible**, and cheaply. §4c already requires that semantic
passes receive only the annotation-erased projection. The correct fix is not to
add a comment-skipping arm beside the string-literal arm — that is a second
erasure convention maintained by hand, and the next lexical class (block
comments, if they are ever admitted) reopens it.

The fix is to run this scan over **the annotation-erased projection the design
already mandates**, so that no erasure logic lives in
`referenced_module_paths_in_text` at all. The string-literal arm currently there
is the same class of hand-rolled erasure and should dissolve with it.

## Next-rung trigger

The annotation-erased projection must be available at this point in the
source-loading fixpoint. Until then the class sits at **mitigatable** — the
failure occurs and is contained by the fact that it widens.

## Provenance

Found while building the N3-C pool-independence harness (gunbc#7996, branch
`n3-c-pool-independence`). That PR is held open on an operator ruling and is
red on an unrelated witness of its own
(`non_fold_residue_no_unrostered_or_stale`); its state decides nothing about
this finding, which was proven independently by the receipt above.
