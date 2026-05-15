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
    report.dag           # advisory carrier (NOT fail-closed Diagnostic); used by synthesis lens

  extdeps/               # external system contracts (15 files)
    languages/           # language models (direction-agnostic — emit AND ingest)
      rust.dag
      python.dag
      go.dag
      cpp.dag            # C++ (subsumes C subset); ISO/IEC 14882
      typescript.dag     # TypeScript + ECMAScript
    frameworks/          # framework substrates (UI / server / data)
      react.dag          # React: Component/Hook/Effect (frontload per operator 2026-05-15)
    formats/             # data format models (direction-agnostic)
      json.dag
      yaml.dag
      csv.dag
      toml.dag
      json_schema.dag
      openapi.dag
    process.dag          # OS process model (POSIX/SUS)
    file_system.dag      # OS file system model (POSIX file/directory operations)
    coordination.dag     # multi-program: Endpoint/DeploymentUnit/sync/async/stream/pubsub
                         # (effect-typed carriers over existing 5 L1 behaviors;
                         # NO 6th behavior per IN-B decision 2026-05-15)

  compiler/              # pipeline orchestrator + 6 stages (7 files)
    00_compile.dag       # orchestrator: (Source, TargetSpec) -> Result<TargetSource, Diagnostic>
    01_tokenize.dag      # FreeMonoid<Char> -> TokenStream
    02_parse.dag         # TokenStream -> ParseTree
    03_normalize.dag     # ParseTree -> NormalizedTree (sugar dissolution)
    03_resolve.dag       # NormalizedTree -> ResolvedTree (symbol binding)
    04_infer.dag         # ResolvedTree -> InferredTree (types/algebra/cardinality)
    05_emit.dag          # InferredTree + TargetSpec -> TargetSource

  lens/                  # dimensions (8 files, parallel after compiler)
    complexity.dag
    cost.dag             # Tier 1 + Tier 2 textbook (α(n)/log*/log log/sub-exp); UnknownCost floor
    parallelism.dag
    effect.dag
    ownership.dag
    idempotency.dag
    synthesis.dag        # cross-algorithm complexity (C7; advisory via Report carrier)
    coverage.dag         # meta-lens — L6/L7/impossible-bug/testgen coverage discipline (structural)

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

**Total: 51 .dag files + 3 docs + 4 .gitkeep = 58 files at scaffold time.**

## Anchor convention

Every v4 `.dag` file carries an `# Anchor:` line in its header. The anchor is
a citation to canonical knowledge that grounds the file's modeling — typically
a Wikipedia article, a language specification, a POSIX standard, or an
authoritative gunbc doc (THESIS.md, MODELING.md, INVARIANTS.md, memory entries).

The discipline:
- **Reviewers can validate the model against the anchor.** If `extdeps/process.dag`
  models a "Process" but the structure doesn't match what
  https://en.wikipedia.org/wiki/Process_(computing) says a process is, the
  reviewer surfaces it.
- **Workers ground modeling in shared facts.** Per `feedback_modeling_philosophy`
  and `feedback_epistemic_stacking`: every concept attaches to an explicit
  ontology rooted in canonical knowledge — no opaque names, no invented
  vocabulary disconnected from established meaning.
- **For external dependencies (`extdeps/`)** the anchor is mandatory and
  external (Wikipedia / spec). For internal substrate (`std/`, `compiler/`,
  `lens/`, `workflow/`) the anchor may be internal (THESIS, MODELING) when
  the concept is gunbc-specific, or external when it grounds in standard
  CS / math.
- **One anchor per file.** Multiple references for the same concept can be
  joined on one line; the file represents one cohesive concept and should
  have one canonical anchor for that concept.

## Zero-deferrals discipline (operator directive 2026-05-15)

v4 has NO deferrals. Any decision that would require a workaround, or
push the decision to a follow-up phase, is a **HARD STOP — escalate
to operator**.

This applies at every tier:
- **Worker tier**: hitting an unmodelable case, ambiguous substrate, or
  "I'll just do this for now" temptation → STOP, file an inbox message
  to operator with the decision shape, do not work around. The brief's
  ESCALATION TRIGGERS section is binding.
- **Audit tier**: every disposition is PROVEN, NOT-IN-V4 (with named
  reason), or OPERATOR-DECISION-REQUIRED. There is no R4-DEFERRED,
  no fast-follow, no canvas-blocked. See [`docs/v4-close-interrogation.md`](../../docs/v4-close-interrogation.md)
  §0 disposition vocabulary.
- **Substrate tier**: if a substrate decision is ambiguous, the file
  is NOT scaffolded until the operator decides. No "we'll figure out
  the modeling during the worker task."

There is no "v5 / v6 / R5". v4 is the shipping version. If something
is needed but not in v4, it goes through a fresh operator
scope-expansion decision (a v4 amendment) — not a deferral to a
future phase that doesn't exist.

The discipline exists because v3 failed exactly here: deferrals
created drift, drift created gaming, gaming required operator
intervention to catch. Zero-deferrals removes the drift surface at
its source.

## Architectural commitments (ratified during PR #3147 review)

These are substrate-level decisions that constrain every worker's modeling
freedom. They are structural, not process-discipline. Per-task briefs
reference this section when dispatching workers.

1. **`TypeNode` and `Behavior` are CLOSED enums** (per C1 stop-signal,
   THESIS:202). Adding a 7th type connective or 6th behavior requires
   explicit operator ratification of substrate extension. The closure is
   enforced in the substrate itself (Disj sum-type declaration in
   `std/node.dag`), not by review process — the compiler reads the closed
   enum and refuses to compile any program that synthesizes outside it.

2. **Tier 2 partial-op totalization lives in `std/primitive.dag`** (per
   THESIS:175-176). Each primitive's partial operations (divide, modulo,
   indexed access, force-unwrap) declare their totalization shape
   (`Result`-return / `Witness`-return / refinement-precondition) in the
   same file as the primitive itself. No separate "totalization registry."

3. **`Diagnostic` schema includes `suggested_correction`** (per THESIS:103-105
   "show the correct code"). Schema:
   `Diagnostic { reason: NamedReason, at: Locus, suggested_correction: Option<NodeFragment> }`.
   The "show the correct code" promise is structural — every Diagnostic
   site CAN carry a suggested fix. Lenses populate it where they have the
   structural information; absent fix is `None`, not a missing field.

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
