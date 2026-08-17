# A generated twin diverged from its authority, and the deletion took a live input with it

**Found 2026-08-17 on `integration/namespace-cut`, while attempting to integrate
main.** Not a merge artifact — reproduced on a clean tree at `33501d72c99`.

## The divergence

    src/v1/05_emit_rust.dag              declares group_unlisted_type_names   (2 refs)
    src/v1/stage0/src/v1_compiler_emit_rust.rs   does not                     (0 refs)

The `.dag` is the authority; the `.rs` is its generated twin. With regen
suspended on this branch, the two are hand-synced, and this one was not.

A census over the five main twin pairs says it is the ONLY such case:

    05_emit_rust.dag           dag=613  rs=632   in_dag_not_rs=1   <- this
    04_infer.dag               dag=305  rs=337   in_dag_not_rs=0
    00_core.dag                dag=186  rs=217   in_dag_not_rs=0
    01_tokenize.dag            dag= 44  rs= 56   in_dag_not_rs=0
    05_emit_core_support.dag   dag= 35  rs= 36   in_dag_not_rs=0

(`in_rs_not_dag` is expected and not divergence: the seed carries hand-written
helpers with no `.dag` original.)

## Why the obvious repair is wrong

The tempting reading is "import-era machinery, delete it from the authority too
and the twins agree." `UnlistedImportUse` is the diagnostic for a name used but
not imported, imports are gone, so it looks dead.

It is not. `reference_derived_use_lines_note` in the same file says the resolver
signal is REUSED, and specifically reused BY the namespace cut:

> Namespace-only resolution (post-PR 6848) references cross-module names WITHOUT
> importing them, so the ref is KNOWN but the use-line is declined (advisory
> UnlistedImportUse, is_error_diagnostic=false) ... This pass derives the missing
> use-lines from the SAME resolver signal, split by reference kind ...
> (1) TYPE refs come from the resolver's UnlistedImportUse diagnostics ...
> zero-drift by construction, the resolver already applied its SVN mask AT
> RESOLVE TIME

So `UnlistedImportUse` stopped being enforcement and became the INPUT to
use-line synthesis for exactly the import-free modules this branch creates.
Deleting it does not remove dead weight; it starves mechanism (1).

## What actually happened

`28cc57e8256` — "Delete import-list enforcement at the root, and the Class B
apparatus it fed" — removed the enforcement at the root and removed the consumer
chain from the generated `.rs` consistently (the `.rs` has zero references, so it
is internally coherent). The `.dag` authority kept both the function and its call
site at `emit_rust.dag:2158-2159`, which still passes `unlisted_type_names:` into
`emit_module_full`.

The root deletion was correct as a delete-first cut. What it did not account for
is that one of X's outputs had acquired a SECOND consumer that survives X: the
enforcement died, the signal was still owed.

This is the census DESIGN describes as the one deletion surfaces — "what X was
hiding" versus "what depended on X" — with the twist that the dependent was
downstream of a diagnostic rather than of a call.

## Consequence, stated as a risk and not as a measurement

If mechanism (1) has no input, import-free modules can emit Rust missing TYPE
use-lines (E0412/E0433 class). I have NOT measured that here; the branch's
emitted-Rust health is the regen/self-host lane and regen is suspended, which is
precisely why this went unnoticed. Mechanisms (2) and (3) in the note — value-
position refs and item type-surface refs — are independent walks and unaffected.

## What is owed

1. Reconcile the twin. The `.rs` is the shipped seed and is coherent; the `.dag`
   carries a function and a call site with no generated counterpart. They must
   agree, and with regen suspended that is a hand edit to load-bearing emit code
   — deliberately NOT made here at the end of a long chain.
2. Replace input (1), or establish it is no longer needed. The note names its own
   successor: "derive use-lines from the module's bound-reference structure (P2a
   candidate-producer / BoundReferenceProvider) — this text-attestation gate is
   the conservative interim, not the destination." Under the cut the interim's
   input is gone, so the successor is now load-bearing rather than aspirational.
3. Only then integrate main. The merge is otherwise mapped and cheap.

## Integration state, for whoever resumes

`git merge origin/main` produces 9 conflicts. Seven are the same shape and
resolve mechanically — HEAD deleted an import block, main edited it, take HEAD:

    dag/gunbc/doc_graph_roots.dag
    dag/test/claim/doc_reachability_witness_test.dag
    src/v1/05_emit_core_support.dag
    src/v2/compiler/wrap_decision.dag
    src/v2/test/claim/wrap_decision_predicate_test.dag
    dag/gunbc/stage0_crate_partition_generated.dag   (also take main's added crate member)

One is a genuine restructure in `src/v1/05_emit_rust.dag`: main replaced a
boolean early-return with a nested `closed_alias_peel_verdict` /
`alias_decl_arity_verdict` match. Take main's arms and qualify `target: Rust` to
`target: v1.compiler.artifact.Rust` inside them — `Rust` has TWO declarers
(`v1.compiler.artifact.RenderTarget`'s variant and `extdeps.languages.rust`'s
`type Rust`), so bare would be a silent wrong binding. All nine of main's new
names (`ClosedAliasPeelZeroParam`, `AliasDeclArityHasParams`, …) are declared
exactly once and are safe bare — checked, not assumed.

The remaining two are the generated twins, blocked on (1) above.

## A tooling note worth keeping

I tried to mechanize the `05_emit_rust.dag` resolution by deriving my branch's
qualification map (bare leaf -> qualified path) and replaying it onto main's
version. The control — replay the transform on the merge-base and check it
reproduces my file byte-for-byte — FAILED, and the failure was instructive:
`emit_info` is a PARAMETER NAME in that file and also a module path
(`v1.compiler.emit_info`), so the replay qualified a local binder.

That is the occurrence-role hazard in miniature, demonstrated on my own tooling
rather than argued: a name-grain rewrite cannot distinguish a module reference
from a parameter that shares its spelling. It is the same reason the 5,783-row
census is an index and not an edit manifest.
