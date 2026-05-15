# v4 — Structural File Tree (closed system)

This is the **closed file tree** for v4. Every file is enumerated below. New files require explicit operator ratification (substrate extension = stop signal). The discipline that v3 lacked: file-tree-as-substrate-categorization, cost-of-change = 1.

## Why v4 exists

v2 proved 1-residual hand-Rust is achievable but had modeling gaps. v3 had richer modeling ambition but accumulated 192 hand-Rust files plus paper-shrink debt because the work-direction substrate was prose, not data. v4 is the synthesis: **v2's residual discipline + v3's modeling depth + recursive-flex applied from day 1.**

## File tree

```
src/v4/
  STRUCTURE.md           # this file
  BRIEF_TEMPLATE.md      # the worker brief shape (immutable across tasks)
  TASKS.md               # 15 XL tasks defining "v4 done"

  std/                   # substrate primitives (8 files)
    node.dag             # 6 type connectives + 5 L1 behaviors (substrate root)
    algebra.dag          # Magma/Monoid/BoolAlgebra/FreeMonoid + inhabitance
    cardinality.dag      # cardinality refinement, P4 decidability
    witness.dag          # Witness<C> — fail-closed lens reads, no Option::None
    diagnostic.dag       # structural Diagnostic { reason, at }
    primitive.dag        # Int/Float/String/Char/Bool with inhabitance
    collection.dag       # bounded containers
    verification.dag     # TestClaim schema (imported from v3)

  extdeps/languages/     # target language specs (3 files, Shape A targets)
    rust.dag
    python.dag
    go.dag

  compiler/              # pipeline stages (6 files, serial dependency)
    01_tokenize.dag      # FreeMonoid<Char> -> TokenStream
    02_parse.dag         # TokenStream -> ParseTree
    03_normalize.dag     # ParseTree -> NormalizedTree (sugar dissolution)
    03_resolve.dag       # NormalizedTree -> ResolvedTree (symbol binding)
    04_infer.dag         # ResolvedTree -> InferredTree (types/algebra/cardinality)
    05_emit.dag          # InferredTree + TargetSpec -> TargetSource

  lens/                  # dimensions (6 files, parallel after compiler)
    complexity.dag
    cost.dag
    parallelism.dag
    effect.dag
    ownership.dag
    idempotency.dag

  workflow/              # recursive-flex — work-direction in .dag (5 files)
    brief.dag            # typed Brief schema
    worker_output.dag    # WorkerOutput { dissolves: Set<HandResidual>, cited_anchor }
    doc_anchor.dag       # typed DocAnchor pointers into authority docs
    retirement.dag       # structural Retirement predicate (no list-length gaming)
    cycle.dag            # work cycle as data, lens-readable

  bin/
    main.dag             # emits main.rs trampoline (0-floor compliant)

  test/
    claim/               # TestClaim data — no hand-Rust tests
    fixture/             # canonical input programs
```

**Total: 28 .dag files + 3 docs + test directories = 31 files at scaffold time.**

## The closed-system invariants

These are non-negotiable across all v4 work:

1. **No new files without operator ratification.** Every file in the tree is enumerated above. A worker proposing a new file is reporting a substrate gap — surface it, do not unilaterally add.
2. **No hand-Rust except the trampoline.** `bin/main.dag` emits a 1-line `include!()` trampoline (per `design-pure-bootstrap-zero.md:210`). Everything else is .dag.
3. **No file-splitting without operator ratification.** Each file is a typed pure function. If a worker thinks `04_infer.dag` should be five files, that's a substrate-design question, not a worker decision.
4. **Cost-of-change = 1.** Adding a new type/expression/transport edits exactly one file. If a change ripples, the substrate is wrong.
5. **Tests are TestClaim data.** Zero hand-Rust tests. Test surface lives in `test/claim/`.
6. **Workflow substrate first.** `workflow/*.dag` is implemented BEFORE any compiler work, so worker outputs are typed Brief/WorkerOutput/Retirement instances from day 1. The recursive-flex move is structural, not aspirational.

## Bootstrap chain

- **Stage minus one**: v2's compiled binary (proven self-hosted at 1-residual). Used to compile v4's first .dag pass.
- **Language constraint**: v4 .dag is written in v2-syntax-compatible subset until v4 self-compiles. New syntax additions land only after v4 can compile itself.
- **Self-host fixed point**: v4 compiles `compiler/*.dag` end-to-end and produces bit-identical output to its prior self-build.

## Relationship to v2 and v3

- **`src/v2/`** (restored 2026-05-15 for honest comparison): historical reference. v2's `04_*` cluster (12 files) is studied as the cautionary tale on substrate inflation in the type-checking layer; v4 collapses this to a single `04_infer.dag` and treats any pressure to split as a substrate-design escalation.
- **`src/v3/`**: not deleted. Sources of importable design (`dsl/std/verification.dag` TestClaim schema, L2.5 design docs in `docs/r3-path-b-*.md`, lens framework conceptual design). Importable items are explicitly cited in TASKS.md per task. v3's hand-Rust is NOT imported under any circumstance.
