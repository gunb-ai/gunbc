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
    05_emit.dag          # InferredTree + TargetSpec -> TargetSource (omni-emission projection)
    05_eval.dag          # InferredTree + Inputs -> Value (THE PRIMARY execution path,
                         # THESIS:225; sibling of emit — eval executes, emit projects)

  lens/                  # dimensions (11 files, parallel after compiler)
    complexity.dag
    cost.dag             # Tier 1 + Tier 2 textbook (α(n)/log*/log log/sub-exp); UnknownCost floor
    parallelism.dag
    effect.dag
    ownership.dag
    idempotency.dag
    synthesis.dag        # cross-algorithm complexity (C7; advisory via Report carrier)
    coverage.dag         # meta-lens — L6/L7/impossible-bug/testgen coverage discipline (structural)
    testgen.dag          # producer side — reads substrate, emits TestClaim corpus (Phase 1.5)
    affected_set.dag     # incremental re-exec frontier; replaces detect-affected shell (Phase 1.5)
    application.dag      # apply_lens surface — opt-in depth + ONLY advisory→fail-closed bridge

  workflow/              # recursive-flex — work-direction in .dag (7 files)
    brief.dag            # typed Brief schema
    worker_output.dag    # WorkerOutput { dissolves: Set<HandResidual>, cited_anchor }
    doc_anchor.dag       # typed DocAnchor pointers into authority docs
    retirement.dag       # structural Retirement predicate (no list-length gaming)
    cycle.dag            # work cycle as data, lens-readable
    bootstrap.dag        # bootstrap orchestration AS DATA (seed-once → self-host
                         # → fixed-point); v2 interprets it; no build.rs/shell
    ci.dag               # CI pipeline AS DATA; .github/workflows/ci.yml is derived
                         # (THESIS:226 — adding a CI gate = editing this one file)

  bin/
    main.dag             # emits main.rs trampoline (0-floor compliant)

  test/
    claim/               # TestClaim data — no hand-Rust tests
    fixture/             # canonical input programs
```

**Total: 57 .dag files + 3 docs + 5 .gitkeep = 65 files at scaffold time.**

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

4. **Every TestClaim slots into one (Tier × Layer) cell** (per THESIS §168-182
   correctness tiers + TESTING.md §141 test layers — two orthogonal axes).
   - **Correctness Tier** (when bug is caught): Tier1 (compile-time) | Tier2 (runtime-totalized) | Tier3 (runtime-observed L4-L7)
   - **Test Layer** (where test runs, with target ratios per TESTING.md): Unit ~75% | Integration ~15% | Boundary ~10%
   Testgen output respects ratio targets; coverage lens verifies completeness
   across the (Tier × Layer × Substrate) cross-product. Workers writing
   manual TestClaims declare both axes in the claim metadata.

5. **Concept unifications are structurally enforced** (per THESIS §184-188).
   THESIS commits to four named unifications. Each lives in a single substrate
   file declared via the `// Unifies:` header field. Adding a parallel carrier
   for any unified concept is a substrate extension = STOP signal. The four:
   - **`coercion = emission`** — owned by `compiler/05_emit.dag`. No separate
     coercion engine; coercion logic lives in emission rules.
   - **`coercion cost = complexity`** — owned by `lens/complexity.dag`. The
     cost of converting between representations IS a complexity-lens read;
     no `CoercionCost` carrier.
   - **`language spec = transport spec = interpreter runtime`** — owned by
     `extdeps/languages/*.dag`. ONE substrate carrier per language for all
     three roles; different lenses read different facts from the same data.
   - **`idempotency + cancellation + redundancy = algebraic simplification`** —
     owned by `lens/idempotency.dag`. Three named runtime concerns are ONE
     mechanism; no `lens/cancellation.dag` or `lens/redundancy.dag`.

6. **Emission is mechanical; algebra-enforcement is the primary job**
   (per THESIS:13 "causal engine ... before emission becomes a mechanical
   translation" + THESIS:196 "the epistemic chain IS the emission algorithm;
   every emitter special case is evidence of an ungrounded concept upstream"
   + THESIS:441 "Emits ... as mechanical translation").
   The compiler's LOAD-BEARING work is validating the epistemic chain:
   `compiler/04_infer.dag` algebra-homomorphism search + `lens/*` +
   `std/algebra.dag` grounding. `compiler/05_emit.dag` is mechanical
   projection of that validated chain. An emitter special-case is a STOP
   signal — it means a concept is ungrounded upstream; fix the grounding
   in `std/algebra.dag` or the epistemic chain (`04_infer.dag`), NOT the
   emitter. A worker on T-10 writing `if target == X` special-cases has
   found an upstream grounding gap; escalate. Declared via `// Primary:`
   header field in `04_infer.dag` (owns enforcement) and `05_emit.dag`
   (mechanical projection).

## The closed-system invariants

These are non-negotiable across all v4 work:

1. **No new files without operator ratification.** Every file in the tree is enumerated above. A worker proposing a new file is reporting a substrate gap — surface it, do not unilaterally add.
2. **No hand-Rust except the trampoline.** `bin/main.dag` emits a 1-line `include!()` trampoline (per `design-pure-bootstrap-zero.md:210`). Everything else is .dag.
3. **No file-splitting without operator ratification.** Each file is a typed pure function. If a worker thinks `04_infer.dag` should be five files, that's a substrate-design question, not a worker decision.
4. **Cost-of-change = 1.** Adding a new type/expression/transport edits exactly one file. If a change ripples, the substrate is wrong.
5. **Tests are TestClaim data.** Zero hand-Rust tests. Test surface lives in `test/claim/`.
6. **Workflow substrate first.** `workflow/*.dag` is implemented BEFORE any compiler work, so worker outputs are typed Brief/WorkerOutput/Retirement instances from day 1. The recursive-flex move is structural, not aspirational.
7. **`.dag` is the sole editable authority; Rust is never authority.**
   "Off Rust" means: no Rust is editable authority — not "no Rust exists
   anywhere" (the CPU always has a host; the seed is always *some*
   compiler). Three sub-invariants make this structural: (a) zero
   hand-Rust in `src/v4/` — closed file tree forbids adding it, .dag-only
   scaffold means none exists to regress; (b) emitted Rust is transient
   build-dir output, never committed; (c) bootstrap orchestration is
   `workflow/bootstrap.dag` (data, interpreted by frozen v2), never a
   `build.rs`/shell. The v4 binary is a content-addressed artifact
   reproducible from `.dag` via the frozen seed; its fixed-point hash is
   pinned. **The only way to change v4 behavior is editing `.dag`.** Rust
   cannot regress because none is authored and the binary hash is
   structurally locked (T-15 `BitIdentical`). This guarantee is in force
   from scaffold time — it does not wait for the 23 tasks; the tasks fill
   in behavior *under* an already-committed anti-regression structure.

## Bootstrap chain

The chain IS a file: `workflow/bootstrap.dag` (orchestration as data; v2
interprets it via `v2-compiler run`). NOT a `build.rs` or shell script —
those would reintroduce editable Rust authority (the v3 regression door).

```
stage −1  v2 binary (from src/v2/'s committed 1-residual Rust — the SEED,
            outside src/v4/, frozen + CI-gated, touched EXACTLY ONCE)
              compiles src/v4/*.dag → Rust (v2 emission style)
                                    → rustc → v4-stage0 binary
stage 0   v4-stage0 compiles src/v4/*.dag → Rust (v4's OWN emission style)
                                    → rustc → v4-stage1 binary
stage 1   v4-stage1 compiles src/v4/*.dag → Rust → rustc → v4-stage2 binary
fixpt     assert stage1-emitted == stage2-emitted  (BitIdentical)
          — fixed point is stage1==stage2, NOT stage0==stage1
            (stage0 is v2-emission-style; stage1+ is v4-style)
```

- **Seed used once**: v2 produces v4-stage0 from a Rust-only environment.
  After stage0 exists, v4 compiles itself; v2 is never in the loop again.
  Identical to gcc-needs-a-C-compiler-once / rustc-was-seeded-via-OCaml.
- **Language constraint**: v4 .dag stays in v2-syntax-compatible subset
  until v4 self-compiles. New syntax lands only after fixed point.
- **Emitted Rust is transient** (option (a)): build-dir only, never
  committed, never editable authority. `.dag` is sole authority.
- **The v4 binary is a content-addressed release artifact**: pinned at
  the fixed-point hash. Day-to-day, people edit `.dag` and run the
  shipped binary — zero Rust touched. Reproducibility = rebuild from
  `.dag` via frozen v2 seed, must reproduce the exact pinned hash.
  T-15's `BitIdentical` TestClaim IS the anti-regression mechanism: a
  drift = hash mismatch = CI red.

## Relationship to v2 and v3

- **`src/v2/`** (restored 2026-05-15 for honest comparison): historical reference. v2's `04_*` cluster (12 files) is studied as the cautionary tale on substrate inflation in the type-checking layer; v4 collapses this to a single `04_infer.dag` and treats any pressure to split as a substrate-design escalation.
- **`src/v3/`**: not deleted. Sources of importable design (`dsl/std/verification.dag` TestClaim schema, L2.5 design docs in `docs/r3-path-b-*.md`, lens framework conceptual design). Importable items are explicitly cited in TASKS.md per task. v3's hand-Rust is NOT imported under any circumstance.
