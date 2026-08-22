# Installing a candidate tree from a generator older than main silently reverts main, and greens

**Status:** standing hazard. Confirmed by specimen, 2026-08-22, on `integration/namespace-cut`.
Reported by `deep-ant-102`, who is relaying it to other tree-installing lanes.

## The claim

A candidate tree emitted by a bootstrap generator whose vintage predates a main
commit **does not contain what that commit added**. Installing the tree wholesale
deletes it. The deletion **compiles**, because an absent capability is not a type
error — it is an absent capability.

## The specimen

Validating an unrelated repair, the candidate tree at
`/tmp/bootwt/target/stage0-regen-candidate/src` differed from the committed
`v1_interpreter_dispatch_generated.rs` by six lines, all deletions:

```
FreeCallSortedMapKeys,
"sorted_map_keys" => Some(EvalBuiltinArm::FreeCallSortedMapKeys),
("free_call.sorted_map_keys") => { ...EvalBuiltinArm::FreeCallSortedMapKeys };
MethodCallSortedMapKeys,
"sorted_map_keys" => Some(EvalAlgebraMethodArm::MethodCallSortedMapKeys),
("method_call.sorted_map_keys") => { ...EvalAlgebraMethodArm::MethodCallSortedMapKeys };
```

Those arms are gunbc#8841, merged to main the same morning. The generator was
built before them, so it cannot emit them. Installing the tree would have
reverted another lane's landed work, and nothing would have gone red.

## Why this is not the vintage gap already being managed

They are different failures with different defenses, and only one of them is loud:

| | stale generator emits a WRONG artifact | stale generator emits an artifact MISSING what it never knew about |
|---|---|---|
| symptom | rustc error | clean build |
| detected by | the build | nothing |
| cost | an hour | another lane's merged commit, silently |

The whole-tree diff does not help: the tree legitimately differs everywhere,
because that is what a regeneration is. The deletion is one hunk among hundreds
of authentic ones — the "hybrid of authentic parts" shape, where the
non-conflicting half is the trap.

## The defense, and its honest rung

**Per-file attribution on every install:** for each file taken from a candidate
tree, every changed line must trace to the `.dag` change being validated. A hunk
that does not is either main's work or a vintage artifact, and the file is not
installed.

That is what caught this specimen. It is **mitigatable**, not higher: it is a
discipline the installing author must remember, with nothing enforcing it. A
generator built from main's HEAD would make the class structurally impossible,
which is unavailable precisely when it is most needed — the branch that requires
a bootstrap generator is the branch that does not compile.

**Next-rung trigger:** the install step refuses any file whose candidate-vs-
committed diff contains a hunk deleting a symbol absent from the `.dag` change
under validation. That is decidable from the two trees plus the diff, and needs
no new authority.

## Scope

Any lane installing an emitted tree from a bootstrap host it did not rebuild
from current main. On `integration/namespace-cut` the gap is 1674 hunks in
`v1_compiler_emit_rust.rs` alone, and it grows with every main merge the branch
does not absorb.
