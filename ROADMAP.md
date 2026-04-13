# gunbc Roadmap

## Architecture

Two substrate primitives: **Node** and **Edge**. Everything else —
types, truth values, cardinality, product/coproduct — is compositional
modeling in `.dag`. Languages are coercion targets. Testing is
compilation.

**Bounded kernel invariant:** Node is the only recursive semantic
authority in the compiler IR. All durable recursive structures are
Node trees — recursion lives in the data (children list), not in
type definitions. Non-Node types are flat discriminants and data
tables. This makes descent provable by construction: any function
that walks Node.children is structurally bounded.

Full thesis: [THESIS.md](THESIS.md)
Architecture: [docs/architecture.md](docs/architecture.md)
Compiler laws and coercion model: [src/v2/compiler-laws.md](src/v2/compiler-laws.md)
Coercion design (algebra-keyed inhabitants): [docs/coercion-design.md](docs/coercion-design.md)
Testing strategy: [src/v2/tests/testing-strategy.md](src/v2/tests/testing-strategy.md)
Invariant enforcement: [INVARIANTS.md](INVARIANTS.md)
Modeling guidelines: [MODELING.md](MODELING.md)

---

## Thesis alignment — dependency order

See [THESIS.md](THESIS.md) for the full thesis. Tracks are ordered
by dependency: foundations first, dependents after. Readiness:

- 🟢 **Implement** — design clear, unblocked, ready to code
- 🟡 **Design** — concept clear, design decisions pending
- 🔴 **Vision** — concept-level work needed before design

```
LAYER 1: Foundations (no dependencies, all 🟢)
  Track 9  (std/ structures)  ──┐
  Track 8  (lattice)          ──┤
  Track 6  (algebra dispatch) ──┤── feed structural facts into IR
  Stream C (std/ foundation)  ──┘

LAYER 2: IR carries facts (depends on Layer 1)
  Track 1  (provenance) 🟢 ──── THE critical path
  Track 3  (Node.name)  🟢 ──── parallel, independent
  Track 7  (core tables) 🟢 ── dissolves as std/ types land

LAYER 3: Emission correctness (depends on Layer 2)
  Track 2  (language spec) 🟡 ── LS-4 borrow model needs design
  Track 4  (codegen)       🟢 ── depends on Track 2 partially
  Stream B (clone elision) 🟢 ── Layers 1-2 unblocked; Layer 3 needs LS-4

LAYER 4: End-to-end (depends on Layer 3)
  Track 5  (real program)  🟢 ── RE-3,4 remaining
  Track 10 (extdeps)       🟢 ── independent, data quality

LAYER 5: Thesis completion (depends on Layer 4)
  Track 13 (single emitter)        🟡 ── depends on Track 2 + 7
  Track 11 (runtime safety)        🟡 ── needs design (refinement types or total ops)
  Track 12 (verification)          🟢 ── partially unblocked (L4 can start, Track 5 🟢)

LAYER 6: Full vision (depends on Layer 5)
  Track 14 (omni-emission)         🔴 ── depends on Track 13; needs vision
  Free consequences (parallelism)  🔴 ── blocked on Tier 1 + ownership + purity
```

| Track | Thesis tier | Readiness | Blocked on |
|-------|------------|-----------|-----------|
| **Track 10 boundary wiring** | **Tier 1 (typed boundaries)** | **🟢 STAFF NOW** | **Nothing — trivial field changes** |
| **Track 9 record dedup** | **Tier 1 (no dup representations)** | **🟢 STAFF NOW** | **Nothing — merge identical types** |
| **Track 8 lattice (FermiDepth, Set)** | **Tier 1 (algebra)** | **🟢 STAFF NOW** | **Nothing — small** |
| Stream C / Track 8 / 9 | Tier 1 (structural facts) | 🟢 | Nothing |
| Track 6 | Tier 1 (string dispatch) | 🟢 | Nothing |
| **Track 1 (provenance)** | **Tier 1 (CX gate)** | **🟢** | **S4 in progress** |
| Track 3 (Node.name) | Tier 1 (structural identity) | 🟢 | Remaining n.name reads |
| Track 7 (core tables) | Tier 1 (single authority) | 🟢 | Track 9 partially |
| Track 2 (language spec) | Emission is mechanical | 🟡 | LS-4 borrow model design |
| Track 4 (codegen) | Emission is mechanical | 🟢 | Track 2 partially |
| Stream B (clone elision) | Tier 1 (ownership) | 🟢/🟡 | Layers 1-2 🟢, Layer 3 needs LS-4 |
| Track 5 (real program) | End-to-end validation | 🟢 | Track 4 |
| Track 10 (extdeps) | Data quality | 🟢 | Nothing |
| **Track 15 (CLI tool modeling)** | **M5 Phase 2** | **🟢** | **Nothing — landed in #418** |
| **Track 16 (CI YAML emission via .dag)** | **M5 Phase 3** | **🟢** | **Nothing — follow-up to #418** |
| **Track 17 (wire unused modeling)** | **Structural proof** | **🟢** | **Nothing — highest leverage** |
| **Track 18 (error mode taxonomy)** | **Tier 1 (no duplicate representations)** | **🟢** | **Nothing** |
| Track 13 (single emitter) | Emission is mechanical | 🟡 | Track 2 + 7 |
| Track 11 (runtime safety) | Tier 2 | 🟡 | Design phase |
| **Track 12 (verification)** | **Tier 3** | **🟢** | **Partially unblocked — L4 can start** |
| Track 14 (omni-emission) | Omni-emission | 🔴 | Track 13 + vision |

---

## Core thesis: close the model

gunbc is a closed system. All data is finite (Bit/Word64). All
iteration is bounded (fold/descend/repeat). Composition preserves
boundedness. In a closed system, properties like complexity,
ownership, termination, and space bounds are **consequences of the
model** — like conservation laws in physics. They should not require
separate analysis passes.

The compiler's main architectural problem is that the IR is not
closed: structural facts are computed during inference, discarded at
the TypeBinding boundary, then reconstructed downstream via heuristics.
This construct-discard-reconstruct pattern is the root cause of most
active work items.

See [src/v2/cx-design.md](src/v2/cx-design.md) for the full diagnosis,
including 6 confirmed instances across all compiler stages.

### The gap: TypeBinding is too narrow

```
type TypeBinding {
  name: String
  resolved: Node      // ← only the TYPE is preserved
                       //   provenance, ownership, cost — all discarded
}
```

Every downstream consumer that needs facts beyond the type must
reconstruct them:
- **Complexity:** 33 heuristics in annotate_descent_evidence (424 violations)
- **Ownership:** string name matching for fold accumulators
- **Emission:** compensates for upstream information loss (M2)
- **Core tables:** hand-maintained string-keyed semantic maps

### The fix: provenance on bindings

Thread the existing `SubValueRelation` (std/induction.dag) through
bindings — preserved from computation to consumption:

```
type TypeBinding {
  name: String
  resolved: Node
  provenance: SubValueRelation   // NEW — reuses existing authority
}
```

SubValueRelation already has the right vocabulary (StrictSubValue,
IteratedSubValue, ArithmeticDescent, PreservedValue, SubValueUnknown)
with InductiveField and ShrinkFactor. No new type — single authority.

**Estimated impact (projections, not measurements):** ~1100-1200
lines net dissolution in CX (~200 lines of classification logic
moves to binding sites; the reconstruction pass and threading
infrastructure dissolve). Ownership name-matching dissolves (~65
lines). Emission heuristics reduce. Total estimated net dissolution:
~1500+ lines across stages. See cx-design.md cleanup catalog for
per-function accounting.

Implementation plan: [src/v2/cx-design.md §Option B](src/v2/cx-design.md).

---

## Lane structure

Six mutually exclusive lanes. Each lane owns distinct files — two
lanes never modify the same file. All run in parallel.

```
            ┌─ Lane A: Inference ────────────────────────────┐
            │                                                 │
Bootstrap D ├─ Lane B: Emission ──────────────────────────────┤
  COMPLETE  │                                                 ├→ Node.name
            ├─ Lane C: Complexity ────────────────────────────┤   deletion
            │                                                 │  (cross-cutting
            ├─ Lane D: DSL Modeling ──────────────────────────┤   final phase)
            │                                                 │
            ├─ Lane E: Testing ───────────────────────────────┘
            │
            └─ PERF (continuous — parallel to all lanes)
```

### Lane file ownership

| Lane | Owns | Never touches |
|------|------|--------------|
| **A: Inference** | 00_core, 02_parse, 04_resolve, 04_infer, 04_types, 04_patterns, 04_lookup, 04_items, 04_access, 04_service | 05_emit*, complexity, dsl/, tests/ |
| **B: Emission** | 05_emit, 05_emit_rust, 05_emit_go, 05_emit_python, 04_emit_info, dsl/extdeps/languages/*, dsl/extdeps/transports/* | 04_infer, 04_types, complexity, dsl/std/, tests/ |
| **C: Complexity** | complexity.dag, src/v2/cx-*.md | 04_*, 05_*, dsl/, tests/ |
| **D: DSL Modeling** | dsl/std/, dsl/extdeps/{llm,github,shell,cron,cloud,git}/, dsl/gunbc/, dsl/tools/, dsl/config/ | src/v2/*.dag (compiler sources) |
| **E: Testing** | src/v2/tests/, scripts/, std/verification.dag, compiler_tests_rust.dag, coercion.dag (test extraction) | 04_*, 05_emit*, complexity |

---

## CI gates

| Gate | Command | Status |
|------|---------|--------|
| Lint | `cargo clippy --workspace -- -D warnings` | GREEN |
| Tests | `cargo test -p v2-compiler-tests` | GREEN (394 pass) |
| Full DSL | `full_dsl_compiles -- --ignored` | GREEN |
| Diagnostic ratchet | `strict_compile_diagnostic_count -- --ignored` | 424 (honest, non-blocking) |
| L1 gate | `scripts/l1-ratchet.sh --check` | GREEN (0, hard gate) |
| Stage0 freshness | `scripts/check-stage0-freshness.sh` | GREEN (blocking) |

---

## Milestones to Gate 1

Four concrete goals, in priority order. Each has a clear done-criterion.

```
M1: CX gate → 0 violations (currently 420, ratchet 420)
    Done when: strict_compile_diagnostic_count = 0, gate is blocking
    Key blocker: OUTPUT PROVENANCE on function signatures.
      Same SubValueRelation already on input bindings (S1-S6),
      mirrored to outputs. Not a new system — completes the
      existing pattern. 3 touch points: infer from body, store
      on signature, consumers read at call sites.
    Done: infrastructure (#398), seed data (e61d199), type
      correction Map<String,…>→List (positional by return child),
      classify_argument reads provenance before hardcoded fallback,
      composition algebra (compose_sub_value_relations in std/),
      body inference for non-recursive functions (#402/#406),
      ExprMethodCall in body inference (pipe |> composes via func_sigs),
      S7 callback element contracts on algebra templates (#406).

    PATH TO ZERO (sequenced by dependency):

    Step 1: Per-field struct provenance consumption        ~-132
      When callee returns a struct, caller accesses .tokens →
      look up callee.output_provenance[field_index], compose.
      Wire child-indexed lookup in classify_let_value.
      Depends on: nothing (infrastructure ready).

    Step 2: Lambda param provenance from callee contracts   ~-50
      S7 callback_element_position is declared but CX
      annotate_descent doesn't yet propagate element
      provenance into lambda body classification.
      Depends on: Step 1 (validates composition pipeline).

    Step 3: Lexicographic proof construction                ~-150
      Wire TerminationProof from std/termination.dag into CX.
      Replace per-argument heuristic classification with
      proof CONSTRUCTORS that build TerminationProof values.
      Handles "locally increasing, globally decreasing":
        (pos increases, tokens shrinks) → lexicographic descent.
        (tree level same, list shrinks) → lexicographic descent.
      Types exist: TerminationProof, ProofEdge, RankingDimension.
      Checker exists: is_valid_proof, is_lexicographic_descent
        (std/graph.dag). Design: cx-computation-model.md.
      Depends on: Steps 1-2 (reliable per-argument evidence
        to compose lexicographically).

    Step 4: Universal checker replaces classify_* heuristics ~-80
      is_valid_proof becomes the single termination authority.
      Delete 5+ ad-hoc classification functions, hardcoded
      fallback tables (C3-C6). One function, not five heuristics.
      Depends on: Step 3.

    Step 5: Remaining edge cases                            ~-10
      Graph DFS (needs language primitive or work-list
      RankingDimension), arithmetic refinement.

    Violation landscape (354 total, down from 420):
      Parser SCC: 139 violations. Descent chains break at
      expect/expect_name (product-type returns without per-field
      output_provenance).
      Remaining ~215 from multi-param descent, arithmetic, graph DFS.

    Note: Stream D parser restructuring is DONE mechanically.
      Sum-type migration in progress — see Stream D below.

M2: Node.name deleted
    Done when: Node.name field removed, l1-ratchet = 0
    How: fix authored_name_at fallback, eliminate ~15 remaining reads
    Active: quick-owl-889
    Unblocks: Stream B Layer 1 (last-use clone elision)

M3: review.dag runs end-to-end
    Done when: review.dag compiles, builds, runs live against real APIs
    DONE via interpreter path (PR #409): `dag run review.dag` calls
      GitHub API (5 REST), Anthropic API (1 REST), shell (3 commands),
      posts review to PR. Rust emission path still needs RE-3 cleanup.

M4: Single emitter reads data, never decides
    Done when: 05_emit_rust/python/go.dag deleted, all emission from specs
    How: Lane C (coercion = emission, language plugins)
    Blocked on: M1 + M2 substantially complete
    Design: docs/single-emitter-design.md

M5: Meta-process modeling (bootstrap, CI, dev process)
    Done when: adding a Node field requires zero manual stage0 edits;
      CI gates derived from .dag declarations; `dag run` is the
      primary way to execute repo processes across all environments
    Enabler: .dag interpreter (Phase 0). `dag run foo.dag` is the
      primary development workflow; emission is a deployment optimization.
    Phase 0 (interpreter): DONE (PR #409)
      I-1: pure eval (all 21 ExprData variants, closures, algebra methods)
      I-2: shell service dispatch (std::process::Command)
      I-3: REST service dispatch (ureq, auth, JSON path extraction)
      Verified: `dag run review.dag` end-to-end against live APIs
    Phase 1 (bootstrap modeling): DONE (PR #418)
      compiler.dag as single authority for self-hosting cycle
      ci.dag gates derived from compiler.dag (zero hardcoded crate names)
      tools/regen.dag, tools/freshness.dag, tools/ratchet.dag built
      is_error_diagnostic fixed: CX/ownership non-fatal for interpreter
      Proven: `dag run check_l1_ratchet` end-to-end
    Phase 2 (tool modeling): NEXT — see Track 15
      Without this, Phase 1 only works in environments where bare
      command names (`cargo`, `grep`, `diff`) resolve correctly via
      PATH. Track 15 replaces PATH-based resolution with explicit
      tool registry + upsert (pattern from gunb.ai/tools/toolpaths).
    Phase 3 (CI as multi-artifact): after Phase 2 — see Track 16
      ci.yml becomes a generated artifact rendered from a typed
      Workflow declaration in ci.dag using extdeps/github/actions.dag
      types. The renderer is a .dag program (Shape B), not a compiler
      render target (Shape A) — YAML is data manipulation, not a
      programming language. See Track 16 for the full design.
    Phase 3.5 (wire existing modeling): PARALLEL — see Track 17
      ~770 lines of declared types in gunbc/workflow, gunbc/bootstrap,
      std/effects, std/resources, gunbc/auth have zero consumers.
      Before adding new modeling, make the existing modeling
      load-bearing. Each wiring is a separate PR (17a–17e) and
      can land in parallel with M1/M2/PERF. Unblocks the next
      thesis-level claims about idempotency, resources, and
      structural review output. Strictly cheaper than new design.
    Phase 3.6 (error mode taxonomy): PARALLEL — see Track 18
      Workflows in gunbc/tools/ each invent their own Result
      coproduct with duplicated ToolsMissing / Failed variants.
      Unify via std/errors.dag with ErrorClass + Retryability.
      Directly folds the three-workflow duplication from PR #418
      into a single authority.
    Phase 4 (bootstrap verification): after Phase 3.5 (Track 17b)
      The bootstrap loop today is a black box: regen runs, pass1
      vs pass2 diff says "converged" or "diverged." When it diverges
      (dark-emu-36-pr3 hit this: new binary detects a cycle the
      old didn't), the debug is manual — no visibility into WHICH
      stage diverged or WHY.

      bootstrap.dag already models the building blocks (195 lines,
      zero consumers): CompilerStage (8 stages), StageInput/Output,
      TransformContract (preserved/recomputed fields per transform),
      FieldPropagation (per-stage per-field), ChangeClassification,
      BootstrapStrategy (SinglePass/TwoPhase/Additive).

      Three levels, each building on the previous:

      Level 1 — Per-stage structural diffs (Track 17b extension):
        regen.dag runs old binary + new binary, compares output
        at EACH stage boundary (not just the final pass). When
        divergence occurs, the report says "Resolve stage output
        differs" instead of "fixed-point failed." bootstrap.dag's
        CompilerStage and ArtifactKind drive the comparison.
        Directly helps current dark-emu session.

      Level 2 — Stage contracts verified during bootstrap:
        Each stage declares a TransformContract: "Resolve takes
        a ParseTree, produces a ModuleGraph. Preserved fields:
        [ident, span]. Recomputed fields: [module_deps, exports]."
        The bootstrap loop checks: did the contract hold? If a
        stage recomputes a field it claims to preserve, the check
        fires at the stage boundary with a typed diff — not at the
        end as a cryptic fixed-point failure.

      Level 3 — Automatic strategy selection (M5 "done when"):
        ChangeClassification (AddField with default → Additive,
        RemoveField → TwoPhase, etc.) is computed from the diff
        between old and new .dag source. The bootstrap loop reads
        the classification, selects the BootstrapStrategy, and
        iterates automatically. TwoPhase changes get a staged
        rollout: old compiler produces bridge stage0, bridge
        stage0 produces final stage0. Zero manual edits.

      Level 3 is the M5 completion criterion: "adding a Node
      field requires zero manual stage0 edits." Levels 1-2 are
      the incremental path that makes Level 3 achievable.

    Phase 5 (dev process): future
    Design: docs/meta-process-design.md
    Hand-maintained Rust: v2_interpreter.rs (bootstrap), cli_run.rs
      (convertible once interpreter exposed as transport/built-in)
```

---

## Active work: close the model

All active tracks are facets of one problem: the IR doesn't carry
enough structure. Three parallel streams now, one phase after.

```
Stream A (provenance)   S1→S2→S3→S4→S5 ─→ C2→C3→C4→C5→C6
                                                  ↑
Stream B (ownership)    O1→O2→O3→O4→O5            │  (independent)
                                                  │
Stream C (std/)         C1  S8 ───────────────────┘
                        (both feed into CX consumer)
```

**Stream A: Provenance pipeline** (04_infer.dag, 04_env.dag, complexity.dag)
The critical path. S1-S5 build binding provenance. Then C2-C6 switch
CX to read it. S6-S7 (lambda provenance) need callee contract design
and are deferred within this stream.

**Stream B: Clone elision** (ownership.dag, 05_emit_rust.dag)
Layers 1+2 (O1-O5). Last-use elision and post-TCO ownership.
**Fully independent** — no dependency on Stream A.
Layer 3 (O6-O10, borrow propagation) is a separate design phase
blocked on LS-4.

**Stream C: std/ foundation** (std/induction.dag, std/algebra.dag, std/termination.dag)
C1 DONE. S8 Phase 1 DONE (DescentEvidence lattice inhabitants).
Phase 2 blocked on user-defined generic emission.

**Stream D: Structural parser** (02_parse.dag, compile.dag)
Restructure parser from integer position indexing to list consumption.
Target: eliminate 132+ CX violations (Category B) by construction.
Design: [src/v2/parser-design.md](src/v2/parser-design.md).

Phase 1 — Mechanical restructuring: DONE (0 ParserState references).
Phase 2 — Sum-type migration: IN PROGRESS.

  Parser helpers migrated from product-with-flag returns to sum types:
  - `eat` → `EatConsumed { token, tokens } | EatUnchanged { tokens }`
  - `advance` → `AdvanceOk { token, tokens } | AdvanceEof`
  - 47 eat callers + 3 advance callers migrated to match-destructure.

  `variant_provenance` field on function signatures carries per-variant
  per-field SubValueRelation. Inference populates by walking function
  bodies; CX-L2 consumer reads it in annotate_descent match arms.
  Pipeline fires end-to-end: 17 call sites consume eat's provenance,
  2 functions resolved.

  **Root cause fixed:** compute_variant_provenance was checking
  return_type.connective on a reference node (NoConnective); needed
  lookup_type to resolve the Disj definition. Also: DeclaredFuncSig
  in stage0 was missing the variant_provenance field.

  PRs:
  - #424 (free-owl-375): CX-R bridges, 350→340. Temporary heuristics.
  - #428 (sum-type-advance-pilot): Sum-type migration + pipeline. 354.

  Next steps (next PR):
  1. expect_name/expect output_provenance — 15 eat-consumer functions
     have descent chains that break at r.tokens (product return without
     per-field provenance). Closing this gap should resolve most of them.
  2. Clean up __ec/__eu match binding names (migration artifacts).
  3. Reconcile with PR #424 — variant_provenance makes some CX-R
     bridges redundant. Determine which bridges to keep vs delete.
  4. Stage0 regeneration + fixed-point verification.
  5. Gate: net reduction below 340 before merge to main.

Performance note: `tokens |> skip(1)` on `Rc<Vec<>>` is O(n) per
step (O(n²) total). Needs runtime representation design — not a
parser-specific fix.

### Active workboards

- **CX:** [src/v2/cx-design.md §Workboard](src/v2/cx-design.md)
  — S1-S8 shared, C1-C6 CX-specific, TDD plan, cleanup catalog
- **Ownership:** [src/v2/ownership-design.md §Workboard](src/v2/ownership-design.md)
  — O1-O10, violation classes, 3 layers, TDD plan, cleanup catalog

### Track 2: Language spec modeling + ownership (Lane B)

**Thesis:** Every target language has a spec. Model specs as .dag data
in `dsl/extdeps/languages/`. The emitter reads specs — never decides.

Closes the emission heuristic gap: codegen decisions become
spec-referenced data lookups instead of inline logic.

| Item | Status |
|------|--------|
| LS-1: Type cast rules | Partial (numeric casts validated in infer) |
| LS-2: Operator semantics | DONE (PR #355) |
| LS-3: Expression syntax | Not started |
| LS-4: Ownership/borrowing | See [ownership-design.md](src/v2/ownership-design.md) |
| LS-5: Visibility/module system | Not started |
| LS-6: Shared typed handlers | DONE (PR #355) |

**LS-4: Ownership — three layers to clone elimination**

Stage0 emits ~13,724 `.clone()` calls (verified 2026-04-12, post
dark-emu; previous count of 23,733 was stale). The ownership
analysis (PR #313) already computes the facts needed to eliminate
most of them. The gap: the emitter doesn't consume all the facts
it has. See [docs/perf/clone-elimination.md](docs/perf/clone-elimination.md)
for the cost model — most clones are `Rc::clone` (atomic
refcount++), not heap allocations. The real perf wins have come
from fact re-derivation elimination (merge_envs), not clone
counting.

Conceptual violation classes (design orientation — not yet
individually measured by the ratchet):
- **V1 (last-use clone):** Fan-out > 1, last use clones when it
  could move.
- **V2 (TCO-gated move):** Fan-out = 1 + owned, TCO gate zeroes
  the movable set.
- **V3 (fold fallback):** Proof says eligible, emitter emits
  try_unwrap + clone fallback.
- **V4 (read-as-clone):** Read edges emitted as `.clone()` when a
  borrow would suffice. Blocked on LS-4.

Current measurement: two coarse aggregates (`movable_but_cloned`
conflates V1+V2; `try_unwrap_fallbacks` approximates V3; V4 not
yet measured). Counting is scope-blind string matching — a
directional regression indicator, not a precise metric.

Three layers, sequenced by dependency:

| Layer | Size | Blocked on | Impact (est.) |
|-------|------|-----------|---------------|
| 1. Last-use elision | 1-2 PRs | Stable binding identity (Track 3) | ~2,000-4,000 clones |
| 2. Post-TCO ownership | 1 PR | Nothing | ~500-1,000 clones |
| 3. Borrow propagation | 3-5 PRs | LS-4 design | ~15,000-18,000 clones |

**Layer 1 (last-use elision):** For each binding with fan-out > 1,
the last use site moves instead of cloning. The ownership analysis
has the span data, but threading it through the emit boundary requires
stable binding identity (Track 3) — name-keyed fact tables collapse
distinct bindings. Blocked on Track 3.

**Layer 2 (post-TCO ownership):** TCO-eligible functions currently
zero the movable set (conservative). Fix: run ownership after TCO
transformation, not before. Fan-out=1 owned locals in TCO functions
can then move. Unblocked.

**Layer 3 (borrow propagation — the bulk):** Read-only function
parameters should be borrowed (`&Rc<T>`) instead of owned (`Rc<T>`).
Call sites pass `&x` instead of `x.clone()`. This is where ~90% of
unnecessary clones live.

Requires LS-4 language spec work:
1. Add borrow syntax to SharingStrategy — how does each target
   language express "I only need to read this"? Rust: `&T`.
   Go: pass-by-value for small types, pointer for large.
   Python: no-op (reference semantics).
2. For each function, determine which params are read-only (all
   edges are Read/Projected). Derivable from ownership proof.
3. Emit function signatures with borrows. Read-only params get
   `&Rc<T>` instead of `Rc<T>`.
4. Cascade: changes function signatures across entire emitted
   codebase — every call site must match. Atomic with stage0 regen.

Layer 3 is a design project (LanguageSpec borrow model), not just
a coding project. Layers 1+2 are unblocked and move the ratchet
immediately.

### Track 3: Structural identity / Node.name deletion (Lane A)

**Root cause:** `Node.name` (a string) is used as semantic authority.
Deletion requires declaration-driven identity.

| Item | Status |
|------|--------|
| L1 ratchet (type constructor comparisons) | 0 (hard gate, PR #352) |
| Declaration-driven algebra (Tiers 1-2.5) | DONE |
| source_text_at threading (D6 PR #356, #362) | Mostly done (~20 n.name reads remain) |
| Migrate accessor callers to `_at` variants (PR #378) | DONE — 109 sites migrated, 13 non-_at defs deleted |
| Per-file source_index at resolve boundary (PR #378) | DONE — resolve_modules takes Map\<String, NewlineIndex\> |
| InternTable as identity consumer | Pending: table exists in FrontendResult; threading to TypeEnv deferred until first real consumer |
| Fix `authored_name_at` fallback | Blocked: cross-module span mismatch still falls back to node.name |
| Node.name field deletion | Blocked by authored_name_at fallback + ~15 direct reads |

### Track 4: Codegen correctness (Lane B)

**Root cause:** Codegen decisions scattered across emitter heuristics
instead of derived from structural authorities.

| Item | Status |
|------|--------|
| CG-1: Authority consolidation (type rendering, sharing, ownership) | DONE |
| CG-2: Expression-level gaps (TLC-1..4) | TLC-1/2/3 done, TLC-4 partial (Rust REST body only) |
| CG-3: Parameterization (3 backends → 1 homomorphism) | Phases 1-3 done, 4-6 deferred |

### Track 5: Real-program emission (Lane B + D)

**Goal:** Compile real .dag programs to fully executable Rust.
First target: `gunbc/tools/review.dag` (PR review agent).

| Item | Status |
|------|--------|
| RE-1: Transport emission fidelity (REST, shell) | DONE (21/21) |
| RE-2: review.dag compiles (dry-run) | DONE |
| RE-3: review.dag passes live integration | DONE — PR #397 (functional), deferred structural items resolved (see below). |
| RE-4: Anthropic REST API end-to-end | Test added (requires ANTHROPIC_API_KEY) |

**RE-3 resolved architectural items** (formerly deferred, now structural):
- Shell channel contracts: `ShellOutputChannel` type defined in
  `dsl/extdeps/transports/shell.dag` (POSIX-grounded: Stdout, Stderr,
  StdoutLines, ExitSuccess). All shell operations across the codebase now
  declare `from` annotations. Legacy type-shape heuristic deleted.
- WireFormat structural type: `RestTransportConfig.content_type: String?`
  replaced with `response_format: WireFormat` (imports from `std.serialization`).
  Emitter reads structural type via `transport_response_format`.
- Qualified identity for ownership indexes: `ItemInfo` now carries
  `module_name`. Ownership/fold/read_only indexes use qualified keys only.
  Call sites resolve callee module via `ItemInfo` for qualified lookup.
- Structural Cargo model: `CargoDependency` type added to `dsl/extdeps/cargo.dag`.
  `emit_cargo_toml` refactored to structured `emit_cargo_dep` helper —
  each dependency is a discrete data item, not embedded in string literals.

### Track 6: Algebra field dispatch (Lane A)

**Partially resolved (M8/M9).** `ExprBinOp.algebra_field` and
`OperatorSpec.algebra_field` now use the structural `AlgebraFieldKind`
coproduct (defined in `std/syntax.dag`) instead of `String?`. Dispatch
is structural. Single-authority data table `algebra_field_entries` in
`std/syntax.dag` declares the kind→name mapping.

**Remaining:** Child lookup still goes through `find_child_named`
(string). `algebra_field_kind_name` in `04_types.dag` converts back
to strings for this lookup. The full structural fix requires typed
child identifiers on algebra Nodes so lookup is by kind, not name.

### Track 7: Core table dissolution + duplication cleanup (Lane A + D)

Hand-maintained string-keyed tables in `00_core.dag` should derive
from type declarations in `std/`:

| Table | What it maps | Fix |
|-------|-------------|-----|
| `expr_child_roles` | ExprData variants → accessor functions | Derive from type definition |
| `node_field_roles` | Node fields → structural roles | Derive from type definition |
| `function_size_effects` | Function names → size effects | Structural contracts in std/ (like CallbackContract) |

These dissolve as types move to `std/` and carry structural facts
(type definitions, algebra witnesses, operation contracts) — not
metadata or annotations.

Additional duplication surfaced by review (PR #371, external audit):
- ~~`emit_rust_default_value`~~ DISSOLVED (PR #377) — reads from
  `TypeCheckpoint.default_expr` instead of hand-coded type dispatch.
- ~~`rust_type_map` / `python_type_map` / `go_type_map`~~ DISSOLVED
  (PR #377) — dead code deleted, all three had zero callers after
  migration to `*_type_checkpoints`.
- ~~`go_source_extension` in extdeps/languages/go/emit.dag duplicates
  `go_scaffold.source_file_extension` in std/languages.dag.~~
  Resolved: duplicate deleted (PR #394).
- `keyword_to_name` in 02_parse.dag duplicates the tokenizer keyword
  table. Reconcile to single authority.
- ~~HashMap vs BTreeMap disagreement: `map_template` in std/languages.dag
  says `HashMap<{0},{1}>` but `empty_map` in rust/emit.dag uses
  `BTreeMap::new()`. Pick one, delete the other.~~
  Resolved: standardized on HashMap; BTreeMap declarations in runtime.dag
  were dead code (PR #394).
- ~~`rt_function_registry` mirrors in rust/emit.dag: `rt_functions` and
  `rt_bridge_function_names` are hand-maintained copies of data already
  in the registry. Comments acknowledge the debt.~~
  Resolved: both converted from `data` to computed `fn` using
  filter/fold over `rt_function_registry`, matching the existing
  pattern of `rt_ref_map_functions` and `rt_wraps_result`.
- CallableOf coverage: `filter`, `any`, `all` now have CallableOf
  in their param_types (PR #379). `sort_by` deferred — its callback
  semantics are unresolved (key-extractor in primitives.dag vs
  comparator in algebra.dag type spec). Ignored tests
  (wrong-callback-arity, wrong-return-type) blocked on sort_by
  resolution + inference-time CallableOf validation.

### Track 8: Lattice inhabitant consolidation (Lane D)

`std/algebra.dag` defines `Lattice<T>` and `BoundedLattice<T>` as
types, but no concrete type declares that it inhabits them.

**Surfaced by ChatGPT Pro audit (2026-04-12):**
- `FermiDepth` in `std/fermi.dag:18-46` manually reimplements a
  lattice join (`fermi_ordinal`, `fermi_gt`, `fermi_max`).
  `fermi_max` IS `Lattice<FermiDepth>.join`. Should declare as
  `Lattice<FermiDepth>` and fold the manual functions into the
  algebra vocabulary.
- `Set<T>` is mapped to `BooleanAlgebra` in a string lookup table
  (`std/types.dag:144`) but NOT structurally composed like
  `List<T> = FreeMonoid<T>` or `Map<K,V> = PartialFunction<K,V>`.
  The algebra membership is in comments and string tables, not in
  the type declaration.

**Status: Phase 1 DONE.** DescentEvidence declared as BoundedLattice
inhabitant with `evidence_rank`, `merge_evidence` (meet),
`join_evidence` (join), `optional_evidence_meet`, and
`map_evidence_merge_at`. All 6 ad-hoc merge functions now compose
through these primitives:

| Function | Was | Now |
|----------|-----|-----|
| `merge_evidence` | 9-line match | BoundedLattice.meet (canonical, unchanged) |
| `merge_optional_evidence` | 10-line match | `optional_evidence_meet(a, b)` |
| `merge_edge_evidence` | 14-line match | `map_evidence_merge_at(base, key, val)` |
| `merge_param_evidence` | 7-line fold | fold + projection + meet (comment only) |
| `merge_argument_relations` | 20-line non-commutative fold | `svr_min_by_rank` — now commutative |
| `merge_branch_usages` | 22-line inline merge | `map_usage_merge_at` + list concat |

**Phase 2 (blocked on user-defined generics in emission):** The
concrete lifters follow the generic pattern documented in
`std/algebra.dag` (optional_meet, map_merge_at, min_by, max_by).
When the emitter supports user-defined generics, these monomorphized
functions collapse into generic lifters parameterized by the element
meet.

**Connects to:** Track 1 (provenance composition uses lattice meet),
KF-3 (test generation can verify lattice laws automatically).

### Track 9: Missed algebraic structures in std/ (Lane D)

Surfaced by external audit (2026-04-10). Types or concepts that are
described but not structurally modeled in .dag:

| Item | Current state | Fix |
|------|--------------|-----|
| Encoding lattice | **DONE** — consolidated to `Encoding` in encoding.dag with BoundedLattice meet/join; `ContentEncoding` deleted; `FileClassification` moved to filesystem.dag | — |
| Stack\<T\> → FreeMonoid | **DONE** — imports algebra.dag; operations aligned to FreeMonoid vocabulary; inhabitation declared | — |
| User-defined generic emission | Generic functions (T, V, K params) parse and type-check but emit unresolved type variables in Rust | Emitter needs monomorphization or generic Rust output |

**Surfaced by ChatGPT Pro audit (2026-04-12) — record shape duplicates:**

| Pair | Files | Fix |
|------|-------|-----|
| `CargoDependency` / `CrateDep` | `extdeps/cargo.dag:25` / `extdeps/languages/rust/imports.dag:76` | Identical fields (name, version, features). Merge to one type. |
| `TransportResponse` / `HttpResponse` | `std/types.dag:472` / `std/types.dag:496` | Identical fields (status, headers, body). Merge to one. |
| `ShellResponse` / `CliResult` | `std/types.dag:484` / `std/types.dag:502` | Same 3 fields, different order. Merge. |
| Dual `Credential` | `std/types.dag:448` / `extdeps/cloud/cloud.dag:43` | Same concept name, DIFFERENT schemas. Rename cloud one to `CloudCredential` or unify. |

### Track 10: Extdeps modeling fidelity (Lane D)

Stringly-typed fields that should be structural, surfaced by external
audit (2026-04-10):

| Item | File | Fix |
|------|------|-----|
| `GitHubAuthToken.scopes: List<String>` | extdeps/github/github.dag | **Previously done** — uses `GitHubScope` enum |
| `ThinkingConfig.type: String` | extdeps/llm/anthropic.dag | **DONE** — `ThinkingMode = Enabled \| Disabled` |
| `LlmMessage.content: String` | extdeps/llm/llm.dag | **DONE** — `List<ContentBlock>` with `TextContent \| ImageContent` |
| `Gist.files: List<GistFile>` | extdeps/github/gists.dag | **Previously done** — `Map<String, GistFile>` |
| OpenAI string-path extraction | extdeps/llm/openai.dag | Structural field access (M8) |
| Policy defaults in `CloudSecretConfig` | std/types.dag | **DONE** — dead type deleted; operations define own inputs |
| `ProjectId` vs `GcpProjectId` | std/types.dag | **DONE** — renamed to `GcpProjectId`; 5 dead types deleted |

**Surfaced by ChatGPT Pro audit (2026-04-12) — typed enums bypassed
at service boundary:**

These are the highest-leverage Track 10 items: the closed type
EXISTS but the service I/O uses String/Bool instead. Each is a
single-field type change — no inference, no derivation, no new
modeling needed.

| Item | Declared type | Actual field | Fix |
|------|-------------|-------------|-----|
| Gist visibility | `GistVisibility = Public \| GistSecret` (gists.dag:18) | `Gist.public: Bool`, `Create.public: Bool` | Change to `visibility: GistVisibility` |
| PR state | `PullRequestState = PrOpen \| PrClosed \| PrMerged` (pulls.dag:26) | `PullRequest.state: String`, `List.state: String` | Change to `state: PullRequestState` |
| LLM stop reason | `StopReason = EndTurn \| MaxTokens \| StopSequence \| ToolUse` (llm.dag:55) | `anthropic Create.stop_reason: String`, `openai Create.finish_reason: String` | Change to typed enums |
| LLM model | `AnthropicModel` / `OpenAiModel` enum + spec tables | `model: String` at both service boundaries | Change to typed model enums |
| Review state | `PullReview.state: String` (pulls.dag:310) | — | Change to typed `ReviewState` enum (declare if missing) |

**Also surfaced — constant duplication:**

| Item | Authority | Duplicated at | Fix |
|------|-----------|-------------|-----|
| API base URL | `github.dag:70` `default_api_base` | `gists.dag:42`, `pulls.dag:60` hardcode `"https://api.github.com"` | Reference the constant |
| Per-page default | `github.dag:72` `default_per_page` | `pulls.dag:75` hardcodes `30` | Reference the constant |

**Also surfaced — semantic drift:**

| Item | Conflict | Fix |
|------|---------|-----|
| Rust `chars` | `std/languages.dag:1006` returns strings (`.to_string()`), `extdeps/languages/rust/emit.dag:60` returns ints (`c as i64`). `Char = Int` in types.dag. | Decide which is authority; delete the other. emit.dag (ints) matches the type declaration. |
| Python `chars` | `extdeps/languages/python/runtime.dag:43` returns string chars (`list()`), `emit.dag:94` returns ints (`ord()`). | Same decision — pick one. |

**Also surfaced — fabrication sentinels:**

| Item | File | What it emits | Fix |
|------|------|-------------|-----|
| `error_type_template` | go/emit.dag:107, python/emit.dag:115, rust/emit.dag:141 | `__EMIT_BUG_{0}__` / `compile_error!("{0}")` — fabrication string flows into emitted code | Should be a compile-time diagnostic, not a runtime string. Track 18 (ErrorClass). |
| `container_param_name_required` | std/types.dag:115 | `__BUG_NO_PROFILE_` string when container profile missing | Should fail closed, not fabricate. Track 18. |

**Also surfaced — test infrastructure:**

| Item | Files | Fix |
|------|-------|-----|
| Dual `source_roots()` | `bootstrap.rs:16` returns `(src/v2, dsl)` tuple; `helpers.rs:95` returns `[dsl, src/v2]` vec — different types AND different order | Consolidate to one function |

---

### Track 15: Holistic CLI tool modeling (Lane D, M5 Phase 2)

**Thesis:** Every external CLI binary is a hidden PATH dependency.
`.dag` programs that shell out to `cargo`, `grep`, `diff`, `cp`, etc.
rely on the environment resolving bare command names. This is exactly
the kind of implicit dependency the closed-model philosophy rejects:
the environment is an unmodeled input.

**Current state (post PR #418):** `gunbc.compiler` is the single authority
for the self-hosting cycle (source roots, crate names, command derivation).
`dag run check_l1_ratchet` executes end-to-end via the interpreter.
**BUT** every command derivation function emits bare command names
(`"cargo build"`, `"grep -rqE"`, etc.) that depend on PATH. `shell.Which.Check`
exists in `extdeps/shell.dag` with **zero consumers**. The `cargo`/colima
issue where `sh -lc` picks up the cargo alias inside the container and
fails is a direct symptom — bare `cargo` resolves to the wrong binary.

**Design reference:** `gunb.ai/tools/toolpaths/` — the sibling repo has
a proven pattern for holistic tool management. Key ideas:
1. `Tool { name, path, version, source, install_cmd }` — single source
   of truth registry
2. `InstallSource` enum — `Container | Rustup | Apt | Brew | Builtin`
   tells *how* to get the tool
3. `Ensure(tool) -> path` upsert — check expected path → check PATH →
   self-heal if source is self-healing → fail with actionable hint
4. Command strings use resolved absolute paths, not bare names
5. Platform/arch mappings are declared data, not inline detection logic

**Target state for gunbc:**

```dag
// dsl/extdeps/tools.dag (new)
type InstallSource
  = SourceBuiltin                            // POSIX tools: cp, rm, sh, grep
  | SourceRustup                             // cargo, rustc
  | SourceApt { package: NonEmptyStr }
  | SourceBrew { package: NonEmptyStr }
  | SourceContainer { image_ref: NonEmptyStr }

type CliTool {
  name: NonEmptyStr
  min_version: String?
  source: InstallSource
}

type ResolvedTool { tool: CliTool, path: FilePath }
type ResolveResult = Resolved { resolved: ResolvedTool } | NotFound { tool: CliTool, hint: String }

func resolve(tool: CliTool) -> ResolveResult uses sh: Shell { ... }

// dsl/gunbc/compiler.dag (extend)
data cargo_tool: CliTool = { name: "cargo", min_version: Some { value: "1.93.0" }, source: SourceRustup }
data grep_tool: CliTool = { name: "grep", min_version: None, source: SourceBuiltin }
data diff_tool, cp_tool, rm_tool, sh_tool: CliTool = ...

data required_tools: List<CliTool> = [cargo_tool, grep_tool, diff_tool, cp_tool, rm_tool, sh_tool]

// Command derivation takes resolved paths
fn build_command(cargo_path: FilePath, cycle: SelfHostingCycle) -> String {
  concat(cargo_path, " build -p ", cycle.generated.package_name, " --release")
}
```

**Every tool starts with resolution:**
```dag
func regenerate(cycle: SelfHostingCycle) -> RegenResult uses sh: Shell {
  let resolved = resolve_all(tools: required_tools)
  match resolved {
    Failed { missing: m } => return ToolsMissing { tools: m }
    Ok { cargo: c, ... } => { ... use c, not "cargo" ... }
  }
}
```

**Done when:**
- `dsl/extdeps/tools.dag` exists with `CliTool`, `InstallSource`, `ResolvedTool`
- `compiler.dag` declares all tools the compiler cycle depends on
- Every command derivation function takes resolved paths instead of building strings from bare names
- `regen.dag`, `freshness.dag`, `ratchet.dag` call `resolve_all` before dispatching
- `dag run regenerate_stage0` works inside colima without hitting the cargo alias
- Zero bare command names in any .dag file under `dsl/gunbc/`

**Blocked on:** nothing — immediate follow-up to PR #418. Everything
#418 built stays; this is additive.

**Why urgent:** without this, the meta-process modeling is architecturally
clean but practically broken in any environment where the PATH resolution
of `cargo`/`grep`/`diff` differs from expectation. The current code only
works because CI runs on a specific environment where bare names happen
to resolve correctly. That's a hidden dependency, not a verified one.

---

### Track 16: GitHub Actions YAML emission via .dag program (Lane D, M5 Phase 3)

**Thesis:** The CI workflow YAML should be **generated** from a typed
`Workflow` declaration that uses the `extdeps/github/actions.dag`
schema, not hand-maintained. The hand-maintained `.github/workflows/ci.yml`
is a parallel representation of what `gunbc/ci.dag` already declares
structurally — same risk we fixed for stage0.

**Current state (post Track 15):** `dsl/extdeps/github/actions.dag`
models the full GH Actions platform (Workflow, Job, Step, RunnerSpec,
LogAnnotation, ActionRef, MatrixStrategy) per the spec. **Zero
consumers.** `dsl/gunbc/ci.dag` declares 8 gates and a `ci_pipeline`
but NOT the surrounding workflow shape (triggers, runner, job, env,
permissions). `.github/workflows/ci.yml` is hand-maintained YAML that
calls `dag run run_ci_pipeline` — it's now a thin shim, but still
hand-maintained. ci_runner.dag executes the pipeline once invoked.

**Key architectural decision: Shape B (.dag program emits YAML), not
Shape A (compiler render target).**

The compiler emits real programming languages — Rust, Python, Go — via
its render targets. YAML is a configuration format, not a programming
language. Treating YAML as a compiler render target would be a category
error: it would grow the compiler core for a concern that belongs in
user code. Instead, a `.dag` program walks a `Workflow` value and
constructs a YAML string via `concat`/`fold`/`match`. The interpreter
runs the program; the program writes the file via `shell.Exec.Run`.

This is parallel to how `tools/ratchet.dag` produces grep commands —
data manipulation in a .dag program, not a compiler concern. It also
exercises the interpreter as the primary execution path (M5 thesis:
`dag run` is the development workflow).

**Target file layout:**

```
dsl/extdeps/github/actions.dag       Workflow/Job/Step types (exists)
dsl/gunbc/ci.dag                      data ci_workflow: Workflow (NEW)
dsl/gunbc/tools/yaml_emitter.dag      func render_workflow(wf) -> String (NEW)
dsl/gunbc/tools/ci_codegen.dag        func gen_ci_yml() (NEW)
                                      — calls render_workflow + writes file
dsl/gunbc/tools/freshness.dag         extended to also check ci.yml freshness
.github/workflows/ci.yml              becomes a generated artifact
```

**Workflow declaration shape (in ci.dag):**

```dag
import extdeps.github.actions {
  Workflow, Job, Step, RunnerSpec, HostedRunner, UbuntuLatest,
  WorkflowTrigger, Push, PullRequest, ActionRef, RunStep, UsesStep,
  PermissionLevel, PermRead, WorkflowPermissions, checkout_action,
  setup_rust_action, cache_action
}

data ci_workflow: Workflow = {
  name: "ci",
  on: [
    Push { branches: ["main"], paths: [] },
    PullRequest { branches: ["main"], types: [Opened, Synchronize, Reopened] }
  ],
  permissions: WorkflowPermissions { contents: PermRead, ... },
  env: { CARGO_TERM_COLOR: "always", RUSTFLAGS: "-D warnings" },
  jobs: [
    Job {
      id: "ci",
      runner: HostedRunner { label: UbuntuLatest },
      timeout_minutes: 45,
      steps: [
        UsesStep { uses: checkout_action, with: { fetch-depth: "1" } },
        UsesStep { uses: setup_rust_action, with: { toolchain: "1.93.0" } },
        UsesStep { uses: cache_action, with: { ... } },
        RunStep { name: "Build Compiler", run: build_command(...) },
        RunStep { name: "CI Pipeline", run: dag_run_command(...) }
      ]
    }
  ]
}
```

**YAML emitter shape (in tools/yaml_emitter.dag):**

```dag
func render_workflow(wf: Workflow) -> String {
  let header = "# GENERATED — do not edit. Regenerate via dag run gen_ci_yml.\n"
  concat(
    header,
    "name: ", wf.name, "\n",
    "on:\n", render_triggers(wf.on),
    "permissions:\n", render_permissions(wf.permissions),
    "env:\n", render_env_map(wf.env),
    "jobs:\n", render_jobs(wf.jobs)
  )
}

// ... render_triggers, render_jobs, render_steps with manual indent tracking
```

**Pure .dag string concatenation. No new compiler features needed.**

**Generator and freshness check:**

```dag
// tools/ci_codegen.dag
func gen_ci_yml() -> CodegenResult {
  match resolve_compiler_tools() {
    ToolsMissing { ... } => ...
    ToolsReady { tools: t } => {
      let yaml_text = render_workflow(wf: ci_workflow)
      let write = shell.Exec.Run(script: concat("cat > .github/workflows/ci.yml <<'EOF'\n", yaml_text, "\nEOF"))
      // ...
    }
  }
}

// tools/freshness.dag (extended)
//   reads committed ci.yml
//   compares to render_workflow(ci_workflow)
//   returns Stale if drift
```

**Done when:**
- `dsl/extdeps/github/actions.dag` has at least one structural consumer (`ci_workflow` data in `gunbc/ci.dag`)
- `dsl/gunbc/tools/yaml_emitter.dag` exists and produces byte-identical output to the current hand-maintained `ci.yml`
- `dsl/gunbc/tools/ci_codegen.dag` provides `gen_ci_yml()` entry point
- `dag run gen_ci_yml` regenerates `.github/workflows/ci.yml` and the result matches the committed file
- `dag run check_stage0_freshness` (or new equivalent) verifies the YAML matches the declaration
- A new CI gate proves the YAML is up-to-date (in CI, regenerate to a temp dir, diff against committed)

**Blocked on:** nothing — follow-up to PR #418. The schema (actions.dag),
the interpreter (`dag run`), and tool resolution (Track 15) are all in
place. This is the wire-up step.

**Why this is the next step:** with this, the meta-process modeling
chain is complete end-to-end. Every CI gate command, every workflow
trigger, every runner spec, every action ref traces back to typed
.dag declarations. The only hand-maintained content is the data
declarations themselves (which are the source of truth). Everything
else — gate commands, YAML, regen process, freshness checks — is
derived. Adding a CI gate is one .dag edit; the YAML regenerates
automatically.

**Open design question (resolve in PR):** should `gen_ci_yml()` write
the file via `cat > FILE <<EOF` (heredoc) or via a dedicated file-write
service in `extdeps/shell.dag`? The latter is more honest to the
extdeps modeling philosophy — `shell.Write.WriteFile` would be a
proper service operation, not a heredoc workaround.

---

### Track 17: Wire unused modeling (structural proof over paper modeling)

**Thesis:** A type with zero consumers is a paper exercise, not
structural proof. gunbc currently has ~770 lines of declared types
across `gunbc/workflow/types.dag`, `gunbc/bootstrap.dag`,
`gunbc/auth/credentials.dag`, and `std/effects.dag` with NO consumers
reading them. Per INVARIANTS.md §"Every feature by construction":
if the model isn't load-bearing, it isn't proving anything.

**The modeling consumption gap:**

| File | Lines | Consumers | Models |
|------|-------|-----------|--------|
| `gunbc/workflow/types.dag` | 335 | **0** | IntentSheet, IssueBinding, ClaimLease, StageRunKey, StageOutcome, PipelineRun, DesignReviewOutput, IssueLifecycleStage |
| `gunbc/bootstrap.dag` | 195 | **0** | CompilerStage, StageInput/Output, BootstrapStrategy |
| `std/effects.dag` | 210 | **0** (self-ref only) | EffectShape with derived idempotency |
| `gunbc/auth/credentials.dag` | 32 | **0** | Credential patterns |
| `std/resources.dag` | — | **0** (self-ref only) | ResourceHandle with acquire/release |

Each of these was modeled as a structural claim about the system.
None of them is checked by any compile-time path. Adding a new field
is free; removing one is free; breaking the semantics is free — there
are no consumers to break. **These are not structural facts. They are
decorative types.**

This is the M1 thesis problem applied to M5: single authorities
exist, but "consumers don't read them" (INVARIANTS §"Facts Flow
Forward"). The fix is the same: thread the authoritative facts
through at least one real consumer so the modeling becomes
load-bearing.

**Design reference:** this matches the-gunbai's architectural
principle that every type should appear in a Contract somewhere
— Provides, Requires, Claims, Imports, or Exports. A type not
mentioned in any contract is a candidate for deletion.

**Target wirings (each can land as a separate PR for clear
before/after benefit):**

- **PR 17a: `std/effects.dag` → extdeps REST operations**
  - `extdeps/github/pulls.dag` operations tagged with `EffectShape`
    (e.g., `CreatePullRequest: CreateEffect`, `MergePullRequest:
    UpsertEffect { key: PathParam { name: "pull_number" } }`,
    `GetPullRequest: ReadEffect`)
  - `extdeps/github/gists.dag`, `extdeps/llm/anthropic.dag` same
  - Add structural test: for every operation marked `UpsertEffect`
    with a `CompositeKey`, the compiler derives an
    `IdempotencyEvidence::LatticeEffect` and emits an
    `f(f(x)) == f(x)` test stub.
  - **Benefit:** idempotency becomes a compile-time property, not
    a comment. `compose_effects` gains its first real call site.
  - **Size:** medium. Each REST op needs a one-line annotation;
    the structural test is new infrastructure.

- **PR 17b: `gunbc/bootstrap.dag` → `gunbc/tools/regen.dag`**
  - Currently `regen.dag` hardcodes the 5-step sequence in nested
    matches (build → compile → copy → check → rebuild → recompile).
  - Replace with `let stages: List<CompilerStage> = [...]` driven
    from `bootstrap.dag`, folded over.
  - Each step's input/output derives from `StageInput`/`StageOutput`,
    not bare strings.
  - **Extension (M5 Phase 4 Level 1):** add per-stage structural
    diffs to the bootstrap loop. When old and new binary diverge,
    the report names the STAGE where divergence starts, not just
    "fixed-point failed." Directly addresses the dark-emu-36-pr3
    bootstrap regression (new binary detects a dependency cycle
    the old binary didn't — the diff would localize to the Resolve
    stage). See M5 Phase 4 for the 3-level plan.
  - **Benefit:** adding a new compiler stage is one edit to
    `bootstrap.dag`, not edits in regen/freshness/ci. Pipeline
    sequence becomes data. Bootstrap failures become diagnosable.
  - **Size:** medium (was small — the per-stage diff adds scope,
    but it's the most valuable part).

- **PR 17c: `gunbc/workflow/types.dag` → `gunbc/tools/review.dag`**
  - `review.dag` currently composes review output as free-form JSON.
  - Map its outputs onto `DesignReviewOutput`, `DesignFinding`,
    `ReviewConcern`, `SeverityLevel`.
  - Future: map review stages onto `IssueLifecycleStage`.
  - **Benefit:** review stops being a string-typed blob; severity
    becomes a typed concept; concern dimensions are a closed
    coproduct.
  - **Size:** medium. Requires touching `review.dag` output schema
    and any consumers.

- **PR 17d: `gunbc/auth/credentials.dag` → `extdeps/github/auth.dag`**
  - `extdeps/github/auth.dag` currently has its own credential
    pattern. Fold into `gunbc/auth/credentials.dag` as single
    authority.
  - **Benefit:** credential handling is one authority, not per-
    provider reinvention.
  - **Size:** small.

- **PR 17e: `std/resources.dag` → at least one resource consumer**
  - `resources.dag` declares `Filesystem` as a resource with
    acquire/release and file classification. No callers.
  - Candidate consumers: `gunbc/tools/freshness.dag` (reads
    generated files), `gunbc/tools/regen.dag` (writes generated
    files).
  - **Benefit:** filesystem access becomes a tracked resource, not
    an ambient capability.
  - **Size:** medium. Requires the interpreter to understand
    resource handles.

**Done when:**
- Every file in the modeling consumption gap table has ≥1 structural
  consumer (not just an import)
- Adding a new EffectShape variant, new CompilerStage, new
  DesignFinding field forces updates in the consumer — if it
  doesn't, the consumer isn't really reading the fact.
- The tests for each wiring are structural (generated from the
  declaration), not hand-written.

**Blocked on:** nothing — each wiring is additive and independent.
Track 17 should proceed in parallel with M1/M2/PERF work, because
it's about MAKING EXISTING MODELING PROVE THINGS, not new modeling.

**Why highest leverage:** every decorative type costs credibility
— "gunbc models X" is only true if X is checked. Wiring consumers
is strictly cheaper than designing new types and produces a real
correctness boost. Before adding another modeling track, we should
make the existing 770 lines load-bearing.

---

### Track 18: Error mode taxonomy in std/errors.dag

**Thesis:** Every `.dag` workflow that dispatches shell/REST calls
invents its own Result coproduct: `RegenResult = Converged |
Diverged | Failed { stage, stderr } | ToolsMissing { tool, hint }`,
`FreshnessCheckResult = Fresh | Stale { diff } | Failed { stderr }
| ToolsMissing { tool, hint }`, `L1RatchetResult = L1Passed |
L1Failed { report } | ToolsMissing { tool, hint }`. The
`ToolsMissing` variant repeats verbatim in all three — a dual
representation per INVARIANTS.md §"No duplicate representations."

The deeper issue: each workflow classifies failure ad-hoc.
`Failed { stderr }` is a bucket for "something went wrong at
transport level" that hides everything useful — was it a rate
limit? an auth failure? a missing binary? a network timeout? The
workflow can't distinguish "retry in 5 seconds" from "abort and
escalate."

**Design reference:** the-gunbai's integration contracts declare
error classes as first-class data:

```
type ErrorClass
  = RateLimit { retry_after: Duration? }
  | AuthFailure { reauth_hint: String }
  | NotFound { resource: String }
  | Timeout { elapsed: Duration }
  | Conflict { reason: String }
  | ToolsMissing { tool: String, hint: String }
  | TransportFailure { stderr: String }
```

Each operation declares which error classes it can produce; the
caller can pattern-match to decide retry vs escalate vs fail.

**Target state:**

```dag
// std/errors.dag (extend)
type ErrorClass
  = RateLimit { retry_after: Duration? }
  | AuthFailure { hint: String }
  | NotFound { resource: String }
  | Timeout { elapsed_ms: Milliseconds }
  | Conflict { reason: String }
  | ToolsMissing { tool: NonEmptyStr, install_hint: String }
  | TransportFailure { stderr: String }
  | Cancelled
  | InvalidInput { field: String, reason: String }

type Retryability
  = Retryable { backoff: BackoffStrategy }
  | NonRetryable
  | RequiresReauth
  | RequiresEscalation
```

**Wirings (per workflow):**

- `gunbc/tools/regen.dag`: replace `ToolsMissing` variant → use
  `ErrorClass::ToolsMissing`. Replace `Failed { stage, stderr }`
  → use `ErrorClass::TransportFailure` with stage as context.
- `gunbc/tools/freshness.dag`: same.
- `gunbc/tools/ratchet.dag`: same.
- `extdeps/llm/anthropic.dag`: REST 429 → `RateLimit`, 401 →
  `AuthFailure`, timeout → `Timeout`.
- `extdeps/github/*.dag`: REST 403 rate-limit → `RateLimit`, 404 →
  `NotFound`, 409 → `Conflict`.

**Retryability derivation:**

```dag
fn retryability(err: ErrorClass) -> Retryability {
  match err {
    RateLimit { retry_after: r } => Retryable { backoff: ... }
    AuthFailure { hint: _ } => RequiresReauth
    NotFound { resource: _ } => NonRetryable
    Timeout { elapsed_ms: _ } => Retryable { backoff: Exponential }
    Conflict { reason: _ } => RequiresEscalation
    ToolsMissing { tool: _, install_hint: _ } => RequiresEscalation
    TransportFailure { stderr: _ } => Retryable { backoff: Linear }
    Cancelled => NonRetryable
    InvalidInput { field: _, reason: _ } => NonRetryable
  }
}
```

**Done when:**
- `std/errors.dag` declares `ErrorClass` and `Retryability`
- Every workflow file in `gunbc/tools/` uses `ErrorClass` instead
  of its own ad-hoc result variant for failure cases (success
  variants stay per-workflow — they're domain-specific)
- `extdeps/github/*.dag` and `extdeps/llm/*.dag` REST operations
  map HTTP status codes to `ErrorClass`
- The generated tests validate: every declared `ErrorClass` variant
  has a corresponding `retryability` branch (exhaustiveness check)

**Blocked on:** nothing. Low risk, immediate benefit, directly
unifies a visible dual representation across cool-cod-501's
three workflow files.

**Why Track 18 unblocks Track 17e:** Track 17e (resources.dag →
freshness/regen) needs an error model that can distinguish "file
missing" (NotFound) from "permission denied" (AuthFailure-like)
from "disk full" (TransportFailure). Having `ErrorClass` means the
resource layer can return typed errors, not bare `Failed { stderr }`.

---

## Future tracks (thesis gaps — not yet active)

These are named so they don't drift out of sight. Each corresponds
to a thesis claim that has no active work.

### Track 11: Runtime safety (Tier 2)

**Thesis claim:** no internal operation can fail at runtime.

**Current state:** zero coverage. Division by zero, integer overflow,
string/array out-of-bounds, optional force-unwrap — all compile fine
and crash or silently produce wrong data at runtime.

**Design direction:** either prove preconditions at compile time
(refinement types: `NonZero<Int>`, `BoundedIndex<N>`) or make all
operations total (division returns `Option<Int>`, indexing returns
`Option<T>`). No partial functions in the runtime.

**Blocked on:** nothing conceptually. This is design work. The closed
system makes it tractable — all values are finite, so bounds are
decidable.

### Track 12: Verification from structure (Tier 3)

**Thesis claim:** the compiler generates verification from declarations.

In a causal engine, the structure IS the behavior specification.
The compiler has both the intent (declarations) and the output
(emitted code). Verification is: **does the emitted code reproduce
the declared intent?** This is not a separate test framework — it
is a free consequence of having a closed causal graph.

| What the compiler knows | What it can verify |
|---|---|
| Type `Order { amount: Float }` | Construct → serialize → deserialize → fields match |
| Service `get_order(id) -> Order via rest::get(...)` with `mock_response` | Call mock → response parses to declared type |
| Algebra law `FreeMonoid.concat is associative` | `concat(concat(a,b),c) == concat(a,concat(b,c))` for generated witnesses |
| Function `fn sum(xs: List<Int>) -> Int` | Input/output pairs derived from type inhabitants |
| `type Status = Active \| Inactive \| Suspended` | Exhaustive round-trip: every variant serializes and deserializes |

Traditional testing verifies behavior independently of code. Here,
behavior and structure are coupled — the declarations carry enough
information to derive both the code AND its tests. The compiler
emits both from the same source.

**Levels (from testing-strategy.md):**
- L4 (semantic correctness): execute emitted code, verify results
- L5 (cross-language equivalence): same .dag → same behavior in all targets
- L6 (exhaustive form coverage): every structural form compiles to every target
- L7 (algebraic law verification): operations obey declared laws

**Status (2026-04-12):** PARTIALLY UNBLOCKED. Track 5 is now 🟢
(RE-3/RE-4 remaining). The interpreter can evaluate .dag functions
directly. The emitter produces Rust/Python/Go code. L4 (semantic
correctness) can start: evaluate a .dag function in the interpreter,
emit and execute the same function in Rust, compare outputs. The
test infrastructure (`compile_to_resolved` in compile.dag) and the
interpreter (`dag run`) are both operational.

L7 (algebraic law verification) has its first concrete candidate:
Track 17a effects derivation generates `f(f(x)) == f(x)` test
obligations for idempotent REST ops. These are L7 tests in
obligation-as-data form, awaiting an execution runner.

**Blocked on:** nothing for L4 design + first tests. L5/L6 need
broader emission coverage. L7 execution needs a test runner that
reads obligation data and dispatches.

### Track 13: Single emitter (compiler-laws.md Lane C)

**Thesis claim:** emission is mechanical translation.

**Current state:** Phases 1–3.5 complete, Phase 4 verified (no code
change needed — orchestration was already callback-based), Phase 5
in progress. Python and Go per-language files have zero language-decision
branches for expressions, patterns, TCO, block statements, or func bodies.
31 per-language functions deleted, replaced by shared functions that read
LanguageSpec data. Match arm rendering uses shared `emit_match_arm_line`
+ `emit_arm_guard` (single authority for both TCO and non-TCO paths).
Pattern rendering unified via `VariantPatternSyntax`; guard syntax via
`guard_prefix` on `ExpressionSemantics` (fails closed when target has
no guard form). Return model (`empty_return_value`, `return_suffix`,
`suppress_unit_return`) drives func body unification.
Per-language files retain: type defs, func/workflow defs, entry
point/module rendering, transport implementations, sum type encoding.
Rust emitter is untouched (ownership logic, Phase 6).

**Progress:** Phase 1 ✓, Phase 2 ✓, Phase 3 ✓, Phase 3.5 ✓,
Phase 4 verified ✓, Phase 5 in progress, Phase 6 blocked on LS-4.

**Line counts:** Python 666, Go 689, shared 2983, Rust 5863.

**Target:** one emitter that reads `LanguageSpec` + `InhabitantDecl`
data per target. Adding a new target language means adding a new
`dsl/extdeps/languages/<lang>/` directory, not touching the compiler.

**Next:** Phase 5 requires LanguageSpec design work — return model
(Go multi-return vs Python single), type definition syntax, async
model, module system. The unification pattern is proven; what remains
is modeling the remaining language differences as data.
**Blocked on (for Phase 5-6):** Track 2 LS-4 (borrow model design).

### Track 14: Omni-emission

**Thesis claim:** one intent graph, many artifacts; emission topology
is part of declared intent.

**Current state:** `artifact.dag` is a placeholder (monolithic
single-artifact plan). The compiler takes one target at a time via
`compile_sources(sources, target)`. No mechanism for a `.dag` program
to declare multi-target intent or for the compiler to handle
cross-artifact glue (shared types, serialization contracts, API
surface consistency).

**Design direction:** artifact planning reads declared emission
targets from the `.dag` source. The compiler validates cross-artifact
type consistency (same type used in Rust server and TypeScript
frontend → serialization contracts agree). Each artifact is a
projection of the validated intent onto a specific target.

**Blocked on:** Track 13 (single emitter — need target-agnostic
emission before multi-target is meaningful).

---

## Bootstrap

**Status: COMPLETE.** Stage0 content is generated from .dag source.
Regenerated binary produces identical output when it self-compiles
(fixed-point convergence).

Hand-maintained files in stage0 (not generated, survive regen):

| File | Category | Convert to .dag? |
|------|----------|-----------------|
| `compiler_tests.rs` | Test harness (`#[cfg(test)]`) | Generated by emitter |
| `v2_interpreter.rs` | Interpreter (I-1/I-2/I-3) | Blocked: bootstrap chicken-and-egg — the interpreter evaluates .dag, so writing it in .dag requires compiling it to Rust first (needs M3 emission quality via Rust path) |
| `cli_run.rs` | CLI Run handler | Blocked: calls `v2_interpreter::run()` which is Rust-only. Convertible once interpreter is exposed as a built-in or service transport |

**Future direction:** Both `cli_run.rs` and `v2_interpreter.rs` should
become .dag source. `cli_run` is straightforward once the interpreter
is callable from .dag (expose as a transport or built-in). The
interpreter itself completes the bootstrap loop: write it in .dag,
compile to Rust via emission, the compiled Rust IS the interpreter.
This is M5 Phase 1+ work.

```
.dag source ──(v2-compiler)──▶ stage0 .rs ──(cargo/rustc)──▶ v2-compiler binary
     ▲                                                              │
     └──────────────────────────────────────────────────────────────┘
```

Source of truth: `.dag` files. Stage0 `.rs` is a derived artifact.

---

## PERF: Eliminate unnecessary work

**Status (2026-04-12):** Profiling identified fact re-derivation
(not clones) as the dominant bottleneck. PR #422 contains the
fix (merge_envs + data-only skip + InternTable wiring), measured
at ~50% self-compile speedup. Pending merge. The LESSON from
this investigation is more important than the speedup.

### What actually happened

The biggest win was NOT clone elimination. It was a 6-line fix
to `merge_envs` that eliminated fact re-derivation (PR #422,
pending merge). After PR #378 threaded a single `InternTable`
through `TypeEnv`, every env in the pipeline shared the same
table — but `merge_envs` still iterated and rebuilt a fresh
table from the merged envs. Per module × 2 merges × 3 envs ×
every string = ~20 seconds of pure waste.

**Impact breakdown (gist test pipeline):**

| Stage | Before | After | Δ |
|-------|--------|-------|---|
| Reconcile | 9.54s | 140ms | **68×** |
| Per-module reconcile | ~1.1s | ~5ms | **200×** |
| Total pipeline | 11.72s | 2.34s | **5×** |
| Self-compile (all files) | ~60-75s | 37.6s | **~2×** |

`.clone()` count is UNCHANGED (13,724). The speedup had nothing
to do with clones.

### The uncomfortable lesson

We wrote perf design docs focused on `.clone()` elimination
because clones are grep-able. 21,211 clones! Must be the
problem! But most clones are `Rc::clone` (refcount++, cheap).
**The actual bottleneck was one function doing O(n²) work on
data it could have read in O(1).**

This is a boundary/fact-flow violation: the upstream authority
(single InternTable) existed, but `merge_envs` sat at a
boundary and re-derived the fact instead of reading it.
Algebraically, `merge(a, a, a) = a` by idempotency — so KF-2
would also catch this — but the primary fix is boundary
discipline (Rule 3), not a missing optimizer.

### The 5 rules (from docs/perf/clone-elimination.md)

1. **Profile first.** Before writing any perf plan, run the
   profiler. Use it to INVALIDATE hypotheses, not gather data
   for one you already committed to.
2. **Audit for re-derivation, not just clones.** Inspect every
   `merge_*` / `combine_*` / `collect_*` / `unify_*` / `build_*_from_*`
   function. Ask: "is this rebuilding a fact that upstream
   already computed?"
3. **When you thread a fact forward, DELETE the old reconstruction.**
   PR2 threaded `InternTable` but didn't delete `merge_envs`'s
   reconstruction. That IS the bug.
4. **Watch for fact-flow violations in reviews.** Not arity
   ("N inputs → suspect") — legitimate folds share that shape.
   The test: does one input already carry the authoritative fact?
5. **Log every case as a KF-2 target.** When KF-2 lands, these
   become its test suite.

### Current state (post-dark-emu)

| Metric | Current | Target |
|--------|---------|--------|
| Self-compile wall time | 37.6s | Continue reducing via fact-flow audit |
| Perf ratchet | 55s | Continue lowering as fixes land |
| `.clone()` in stage0 | 13,724 | Not the priority metric anymore |
| `name.clone()` (heap alloc) | ~1,188 | 0 (via M2 ident:Int — modeling value) |
| node.name reads in compiler | 107 | 0 |
| Stages run on data-only files | 3 (was 8) | PR #422 (pending merge) — 82 of 143 files skip CX/ownership |
| merge_envs re-derivation | 0 (was 20s) | PR #422 (pending merge) |
| Other re-derivation hotspots | Unknown | **Audit needed** |

### What's next

**Not:** another clone elimination plan. That framing was wrong.

**Is:**

1. **Audit for other re-derivation hotspots.** Apply Rules 2
   and 3. Find the next merge_envs. This is the single highest-
   leverage perf work until we know the next bottleneck.
2. **M2 Node.name deletion** — still valuable for modeling
   (stable binding identity → Stream B Layer 1), but no longer
   the critical perf path.
3. **M1 Step 1 — per-field provenance on function signatures.**
   `variant_provenance` (Map<String, List<Map<String, SubValueRelation>>>)
   now populates for sum-type returns (PR #428). Infrastructure works
   end-to-end. Next: per-field `output_provenance` for product-type
   returns (expect_name, expect) to close descent chains in the
   parser SCC. See Stream D above for full status.
4. **Elevate KF-2.** We keep committing these bugs against
   ourselves. Building KF-2 catches the next merge_envs before
   it ships. This should move up in priority given how often
   we hit it.

Design: [docs/perf/clone-elimination.md](docs/perf/clone-elimination.md)

---

## Experimental

### Work orchestration (OaaS)

**Vision:** The management plane — tracking parallel work lanes,
spawning sessions, detecting blockers, escalating decisions — as a
`.dag` workflow running on cloud infrastructure.

**Why now:** Automated PR review (review.dag) already reduces daily
cognitive load. CI modeling (ci.dag) prevents forgotten gates.
Orchestration is the next level: an agent manages routine
coordination and escalates only what needs human judgment.

**Composition model:**
```
Level 0: Worker     → does implementation → produces PR
Level 1: Orchestrator → spawns workers → monitors → escalates unknowns
Level 2: User       → sets strategy → orchestrator handles execution
Level 3: Company    → executives → managers → orchestrators → workers
```

Each level reduces cognitive load for the level above by handling
everything it CAN handle and escalating what it CAN'T. Same .dag
workflow, stacked. The escalation path composes.

**Technology/features needed:**
- Cloud/VPS worker provisioning (`dsl/extdeps/compute/`)
- Agent session as a service transport (Claude Code, Codex as shell transports)
- Sandboxed environments (git worktrees, Docker)
- Observability dashboard (status of all managed work)
- Escalation protocol (structured questions with context)
- Risk analysis (expected progress vs actual, blocker detection)

**Inspired by:** OaaS from gunb.ai repo. Validated by: ctrl/ automated
review work (low effort, high yield, daily cognitive load reduction).

**Status:** Experimental. Tracking technology needs. Invest
incrementally — each piece (review automation, CI modeling, worker
provisioning) delivers standalone value while building toward the
full orchestration model.

---

## Killer features

These are capabilities that do not exist in any production system
today. They are the reason to use gunbc over writing Rust/Go/Python
directly. Each is grounded in the closed-model property: .dag
programs are decidable, Node-bounded, and finite — so the compiler
can prove things that are undecidable in general-purpose languages.

### KF-1: Complexity proof on every compile

Every function gets a proven asymptotic bound at compile time. Not a
lint — a structural proof. Grounded in three bounded primitives
(fold/descend/repeat) and type-derived descent facts.

**Status:** 424 honest violations (non-blocking). The provenance-on-
bindings work (Track 1) is the path to 0. Once provenance flows
through bindings, complexity is a consequence — not an analysis.

### KF-2: Reject suboptimal algorithms

The compiler refuses to compile code when a provably cheaper
equivalent exists. Requires cost ordering + equivalence catalog in
`std/optimization.dag`.

**Status:** Not built. Infrastructure close (CostShape per method,
CostExpr composition). Blocked by KF-1 (needs working cost algebra).

**Priority elevated (2026-04-12):** the `merge_envs` perf bug
was primarily a boundary/fact-flow violation (the boundary
re-derived a fact the upstream authority already provided).
The immediate fix is boundary discipline. But the algebraic
identity `merge(a, a, a) = a` IS a KF-2 case — a sufficiently
complete KF-2 would detect this as a cheaper equivalent. Every
boundary/fact-flow bug we find that also has an algebraic
simplification becomes a KF-2 test case. The immediate fix is
discipline; the structural fix is KF-2.
See [docs/perf/clone-elimination.md](docs/perf/clone-elimination.md).

### KF-3: Verification from structure (free)

In a causal engine, structure and behavior are coupled — the
declarations carry enough information to derive both code AND its
verification. The compiler emits both from the same source. Add a
type → verification appears. Add a service → integration test
appears. No hand-written tests needed for declared behavior.

**Status:** L0 (coercion tests from data) done. L4-L7 not built.
See Track 12 for the full plan.

### KF-4: Cross-language equivalence proof

All three target languages (Rust, Go, Python) produce correct,
equivalent output for the full .dag surface area.

**Status:** Partial. Rust emission working. Go/Python scaffolded.

### KF-5: Decidable high-level language

The language itself. All .dag programs are decidable. Undecidable
programs are structurally unrepresentable.

**Status:** Working. The bounded kernel invariant + three iteration
primitives + fail-closed on unknown descent.

### KF-6: Hardware compilation target (Verilog/SPICE)

Compile .dag programs to hardware description languages. Bounded
iteration maps directly to pipeline stages and FSMs.

**Status:** Design only. Blocked by KF-1 (complexity bounds →
pipeline depth).

### KF-7: Space complexity

Every function gets a proven space bound (peak memory). Grounded in
the same provenance model as time complexity.

**Status:** TCO detection working (PR #357). Full space analysis
blocked by Track 1 (provenance).

### KF-8: Optimality gate

Compile error if a function's complexity exceeds a declared bound.

**Status:** Deferred. Needs structural CostBound comparison (not
lossy ranking).

---

## Public release gates

The release is the conjunction of all gates. No partial credit.
Each gate maps to a thesis tier.

### Gate 1: Causal engine is closed (Tier 1)

| Criterion | Test | Track |
|-----------|------|-------|
| Provenance on bindings | 0 CX violations, reconstruction code deleted | Track 1 |
| Complexity gate blocking | CostUnknown = compile error | KF-1 |
| Language specs modeled | No inline target-language knowledge in emitter | Track 2 |
| Node.name deleted | l1-ratchet = 0, field deleted | Track 3 |
| Codegen from structural authority | CG acceptance criteria met | Track 4 |
| Real program compiles and runs | review.dag end-to-end | Track 5 |
| Performance | No test >2s, self-compile <30s | PERF |

### Gate 2: Runtime safety (Tier 2)

| Criterion | Test | Track |
|-----------|------|-------|
| All runtime operations total | No `.force()`, no unchecked division, no panics | Track 11 |
| Checked arithmetic | Overflow = compile error or checked op | Track 11 |
| Bounds safety | Out-of-bounds = compile error or Option return | Track 11 |

### Gate 3: Verification from structure (Tier 3)

| Criterion | Test | Track |
|-----------|------|-------|
| Semantic correctness (L4) | Emitted code executes, produces correct results | Track 12 |
| Cross-language equivalence (L5) | Same .dag → same behavior in all targets | Track 12 / KF-4 |
| Decidable language | Working (already met) | KF-5 |

### Gate 4: Demo quality

| Criterion | Test |
|-----------|------|
| One impressive demo | Compile .dag service → show Rust + Python → run live → show proofs |
| Documentation | README, getting-started, language reference |
| Clean install | `cargo install gunbc` works |

### Release dependency chain

```
Track 1 (provenance) ──→ KF-1 (CX gate) ──→ Gate 1 (causal engine)
Track 2 (language specs) ──→ Gate 1
Track 3 (Node.name) ──→ Gate 1
Track 4 (codegen) ──→ Gate 1
Track 5 (RE) ──→ Gate 1
                                                          │
Track 11 (runtime safety) ──────────────────────────→ Gate 2 (runtime)
                                                          │
Track 12 (verification) ────────────────────────────→ Gate 3 (tests)
Track 13 (single emitter) ──→ Track 14 (omni-emit)       │
                                                          │
                                    Gate 1 + Gate 2 + Gate 3 ──→ Gate 4 ──→ Release
```
